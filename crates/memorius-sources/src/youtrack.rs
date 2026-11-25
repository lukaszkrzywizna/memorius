//! The user's own work items in YouTrack's REST API: read them, and log an
//! approved worklog as a new one. The token comes from the caller (the
//! vault).

use std::time::Duration;

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use memorius_core::tracker::TrackerWorklog;
use serde_json::Value;

const PAGE: usize = 500;
const FIELDS: &str = "id,date,duration(minutes),text,type(name),issue(idReadable,summary)";

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into()
}

fn get(base: &str, token: &str, path: &str, query: &[(&str, &str)]) -> Result<Value, String> {
    let url = format!("{}/api/{path}", base.trim().trim_end_matches('/'));
    let mut request = agent()
        .get(&url)
        .header("Authorization", &format!("Bearer {}", token.trim()))
        .header("Accept", "application/json");
    for (key, value) in query {
        request = request.query(*key, *value);
    }
    let mut response = request.call().map_err(|error| match error {
        ureq::Error::StatusCode(401 | 403) => "YouTrack refused the token".to_owned(),
        ureq::Error::StatusCode(404) => "No YouTrack API at this address".to_owned(),
        other => from_server(&format!("Cannot reach YouTrack: {other}"), token),
    })?;
    response.body_mut().read_json().map_err(|error| {
        from_server(
            &format!("YouTrack sent an unexpected answer: {error}"),
            token,
        )
    })
}

/// A message with text from the server, safe to store and show: never the
/// token (a hostile server could echo it back), and short.
fn from_server(text: &str, token: &str) -> String {
    let token = token.trim();
    let text = if token.is_empty() {
        text.to_owned()
    } else {
        text.replace(token, "[token]")
    };
    text.chars().take(300).collect()
}

fn post(base: &str, token: &str, path: &str, body: &Value) -> Result<Value, String> {
    let url = format!("{}/api/{path}", base.trim().trim_end_matches('/'));
    let mut response = agent()
        .post(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", &format!("Bearer {}", token.trim()))
        .header("Accept", "application/json")
        .send_json(body)
        .map_err(|error| from_server(&format!("Cannot reach YouTrack: {error}"), token))?;
    let status = response.status().as_u16();
    let answer: Value = response.body_mut().read_json().unwrap_or(Value::Null);
    match status {
        200..=299 => Ok(answer),
        401 | 403 => Err("YouTrack refused the token".to_owned()),
        _ => Err(answer["error_description"]
            .as_str()
            .or_else(|| answer["error"].as_str())
            .map_or_else(
                || format!("YouTrack answered {status}"),
                |text| from_server(text, token),
            )),
    }
}

/// The work item types (id, name) of the instance.
pub fn work_item_types(base: &str, token: &str) -> Result<Vec<(String, String)>, String> {
    let types = get(
        base,
        token,
        "admin/timeTrackingSettings/workItemTypes",
        &[("fields", "id,name")],
    )?;
    Ok(types
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some((
                item["id"].as_str()?.to_owned(),
                item["name"].as_str()?.to_owned(),
            ))
        })
        .collect())
}

/// The short names (`INT`, `SEL`) of the projects the token owner can see.
pub fn projects(base: &str, token: &str) -> Result<Vec<String>, String> {
    let projects = get(
        base,
        token,
        "admin/projects",
        &[("fields", "shortName"), ("$top", "1000")],
    )?;
    Ok(projects
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|project| Some(project["shortName"].as_str()?.to_uppercase()))
        .collect())
}

/// Logs `minutes` on `issue` for `date` (`YYYY-MM-DD`) as the token owner;
/// returns the new work item's id.
pub fn log_work(
    base: &str,
    token: &str,
    issue: &str,
    date: &str,
    minutes: u32,
    text: &str,
    type_id: Option<&str>,
) -> Result<String, String> {
    // YouTrack keeps a work item's day as midnight UTC.
    let day_ms = date
        .parse::<Date>()
        .and_then(|day| day.to_zoned(TimeZone::UTC))
        .map_err(|error| format!("{date} is not a date: {error}"))?
        .timestamp()
        .as_millisecond();
    let mut body = serde_json::json!({
        "date": day_ms,
        "duration": { "minutes": minutes },
        "text": text,
    });
    if let Some(id) = type_id {
        body["type"] = serde_json::json!({ "id": id });
    }
    let created = post(
        base,
        token,
        &format!("issues/{issue}/timeTracking/workItems?fields=id"),
        &body,
    )?;
    created["id"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "YouTrack did not return the new work item".to_owned())
}

/// A work item that already logs this worklog, so a retry after a crash or
/// an agent's run does not log it twice.
pub fn already_logged<'a>(
    items: &'a [TrackerWorklog],
    issue: &str,
    date: &str,
    minutes: u32,
    text: &str,
) -> Option<&'a TrackerWorklog> {
    items.iter().find(|item| {
        item.issue == issue
            && item.date == date
            && item.minutes == minutes
            && item.text.as_deref().map(str::trim) == Some(text.trim())
    })
}

/// The token owner's name, to show that the connection works.
pub fn me(base: &str, token: &str) -> Result<String, String> {
    let user = get(base, token, "users/me", &[("fields", "login,fullName")])?;
    Ok(user["fullName"]
        .as_str()
        .or_else(|| user["login"].as_str())
        .unwrap_or_default()
        .to_owned())
}

/// The token owner's work items dated `from..=to` (`YYYY-MM-DD`).
pub fn work_items(
    base: &str,
    token: &str,
    from: &str,
    to: &str,
) -> Result<Vec<TrackerWorklog>, String> {
    let mut items = Vec::new();
    loop {
        let skip = items.len().to_string();
        let top = PAGE.to_string();
        let page = get(
            base,
            token,
            "workItems",
            &[
                ("author", "me"),
                ("startDate", from),
                ("endDate", to),
                ("fields", FIELDS),
                ("$top", &top),
                ("$skip", &skip),
            ],
        )?;
        let parsed = parse_work_items(&page);
        let count = page.as_array().map_or(0, Vec::len);
        items.extend(parsed);
        if count < PAGE {
            return Ok(items);
        }
    }
}

/// YouTrack stores a work item's day as midnight UTC.
fn parse_work_items(page: &Value) -> Vec<TrackerWorklog> {
    page.as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let date = Timestamp::from_millisecond(item["date"].as_i64()?)
                .ok()?
                .to_zoned(TimeZone::UTC)
                .date()
                .to_string();
            Some(TrackerWorklog {
                id: item["id"].as_str()?.to_owned(),
                issue: item["issue"]["idReadable"].as_str()?.to_owned(),
                summary: item["issue"]["summary"].as_str().map(str::to_owned),
                date,
                minutes: u32::try_from(item["duration"]["minutes"].as_u64()?).ok()?,
                text: item["text"].as_str().map(str::to_owned),
                work_type: item["type"]["name"].as_str().map(str::to_owned),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;

    use super::*;

    #[test]
    fn server_text_never_carries_the_token_and_stays_short() {
        let echoed = from_server("bad header: Bearer perm:secret", " perm:secret ");

        assert_eq!(echoed, "bad header: Bearer [token]");
        assert_eq!(from_server(&"x".repeat(1000), "perm:secret").len(), 300);
    }

    /// Answers each request with the next body and returns the request lines
    /// and authorization headers it saw.
    fn serve(bodies: Vec<String>) -> (String, std::thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let mut seen = Vec::new();
            for body in bodies {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    let lower = line.to_lowercase();
                    if let Some(value) = lower.strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap();
                    }
                    if line.starts_with("GET")
                        || line.starts_with("POST")
                        || lower.starts_with("authorization")
                    {
                        seen.push(line.trim().to_owned());
                    }
                }
                let mut request_body = vec![0; length];
                reader.read_exact(&mut request_body).unwrap();
                if length > 0 {
                    seen.push(String::from_utf8(request_body).unwrap());
                }
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
            seen
        });
        (base, handle)
    }

    #[test]
    fn work_items_are_read_with_the_token_and_their_utc_day() {
        let page = r#"[
          {"id":"1-1","date":1792022400000,"duration":{"minutes":35},"text":"Auth0 sign-in",
           "type":{"name":"Development"},"issue":{"idReadable":"SEL-78","summary":"Auth0"}},
          {"id":"1-2","date":1792108800000,"duration":{"minutes":15},"text":null,"type":null,
           "issue":{"idReadable":"PD-282","summary":null}}
        ]"#;
        let (base, server) = serve(vec![page.to_owned()]);

        let items =
            work_items(&format!("{base}/"), " secret ", "2026-10-15", "2026-10-16").unwrap();

        let seen = server.join().unwrap();
        assert!(seen[0].contains("author=me") && seen[0].contains("startDate=2026-10-15"));
        assert_eq!(seen[1].to_lowercase(), "authorization: bearer secret");
        assert_eq!(
            items
                .iter()
                .map(|item| (item.issue.as_str(), item.date.as_str(), item.minutes))
                .collect::<Vec<_>>(),
            [("SEL-78", "2026-10-15", 35), ("PD-282", "2026-10-16", 15)]
        );
        assert_eq!(items[0].work_type.as_deref(), Some("Development"));
    }

    #[test]
    fn the_token_owner_is_named() {
        let (base, server) = serve(vec![
            r#"{"login":"lk","fullName":"Ann Example"}"#.to_owned(),
        ]);

        assert_eq!(me(&base, "t").unwrap(), "Ann Example");
        server.join().unwrap();
    }

    #[test]
    fn a_worklog_is_logged_with_its_utc_day_and_type() {
        let (base, server) = serve(vec![r#"{"id":"164-9"}"#.to_owned()]);

        let id = log_work(&base, "t", "SEL-78", "2026-10-16", 35, "Auth0", Some("7-1")).unwrap();

        let seen = server.join().unwrap();
        assert_eq!(id, "164-9");
        assert!(seen[0].starts_with("POST /api/issues/SEL-78/timeTracking/workItems"));
        let body: Value = serde_json::from_str(seen.last().unwrap()).unwrap();
        assert_eq!(body["date"], 1_792_108_800_000_i64);
        assert_eq!(
            (
                body["duration"]["minutes"].as_u64(),
                body["type"]["id"].as_str()
            ),
            (Some(35), Some("7-1"))
        );
    }

    #[test]
    fn an_existing_work_item_counts_as_logged() {
        let item = TrackerWorklog {
            id: "1".into(),
            issue: "SEL-78".into(),
            summary: None,
            date: "2026-10-16".into(),
            minutes: 35,
            text: Some("Auth0 ".into()),
            work_type: None,
        };
        let items = [item];

        assert!(already_logged(&items, "SEL-78", "2026-10-16", 35, "Auth0").is_some());
        assert!(already_logged(&items, "SEL-78", "2026-10-16", 30, "Auth0").is_none());
    }
}
