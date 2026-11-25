//! YouTrack issue keys (`INT-290`, `IS-61`) found in branch names, session
//! titles, window titles and URLs.

/// Uppercase prefixes that look like issue keys but are standards or
/// encodings (`UTF-8`, `SHA-256`, `ISO-8601`).
const NOT_PROJECTS: [&str; 14] = [
    "AES", "CVE", "GPT", "HTTP", "ISO", "MD", "PEP", "RFC", "RGB", "SHA", "TLS", "UTF", "WCAG",
    "WIN",
];

/// Issue keys written in uppercase anywhere in `text` (titles, URLs, commit
/// subjects), in order of first appearance, without duplicates.
pub fn issue_keys(text: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut rest = text;
    let mut previous: Option<char> = None;
    while let Some(start) = rest.find(|c: char| c.is_ascii_uppercase()) {
        let before = rest[..start].chars().next_back().or(previous);
        let candidate = &rest[start..];
        let starts_word = before.is_none_or(|c| !c.is_ascii_alphanumeric());
        if starts_word && let Some((key, length)) = key_at(candidate, false) {
            if !keys.contains(&key) {
                keys.push(key);
            }
            previous = candidate[..length].chars().next_back();
            rest = &candidate[length..];
            continue;
        }
        let skip = candidate
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(candidate.len())
            .max(1);
        previous = candidate[..skip].chars().next_back();
        rest = &candidate[skip..];
    }
    keys
}

/// Issue keys at the start of a branch path segment, in any case:
/// `claude/int-290-order-sync-unchanged` → `INT-290`. Later words such as
/// `batch-2` are not keys.
pub fn branch_issue_keys(branch: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for segment in branch.split('/') {
        if let Some((key, _)) = key_at(segment, true)
            && !keys.contains(&key)
        {
            keys.push(key);
        }
    }
    keys
}

/// A key at the very start of `text`: project letters/digits, `-`, a number
/// without a leading zero, then a word end. Returns the key and its length.
fn key_at(text: &str, any_case: bool) -> Option<(String, usize)> {
    let project_length = text
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(text.len());
    let project = &text[..project_length];
    let letter_first = project.starts_with(|c: char| c.is_ascii_alphabetic());
    let case_ok = any_case || !project.chars().any(|c| c.is_ascii_lowercase());
    if !(2..=10).contains(&project_length) || !letter_first || !case_ok {
        return None;
    }
    let after_dash = text[project_length..].strip_prefix('-')?;
    let digits = after_dash
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after_dash.len());
    let number = &after_dash[..digits];
    let ends_word = after_dash[digits..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric());
    let project = project.to_ascii_uppercase();
    let valid = !number.is_empty()
        && !number.starts_with('0')
        && ends_word
        && !NOT_PROJECTS.contains(&project.as_str());
    valid.then(|| (format!("{project}-{number}"), project_length + 1 + digits))
}

#[cfg(test)]
mod tests {
    use super::{branch_issue_keys, issue_keys};

    #[test]
    fn finds_uppercase_keys_in_titles_and_urls() {
        let cases: [(&str, &[&str]); 8] = [
            ("INT-177 Job runner migration batch 2", &["INT-177"]),
            ("[IS-61] One catalog read model (#1935)", &["IS-61"]),
            (
                "https://acme.youtrack.cloud/issue/PD-260/one-sql",
                &["PD-260"],
            ),
            ("SEL-54 and SEL-54 again, then IS-61", &["SEL-54", "IS-61"]),
            ("UTF-8, SHA-256, ISO-8601, RFC-7231", &[]),
            ("int-290 lower, MID-007, A-1, ABCDEFGHIJK-1, xINT-5", &[]),
            ("Pull request #1814 · [INT-290] Order sync", &["INT-290"]),
            ("OrderSync.fs — Shop", &[]),
        ];
        for (text, expected) in cases {
            assert_eq!(issue_keys(text), expected, "in {text:?}");
        }
    }

    #[test]
    fn finds_keys_only_at_branch_segment_starts() {
        let cases: [(&str, &[&str]); 5] = [
            ("claude/int-290-order-sync-unchanged", &["INT-290"]),
            ("codex/int-177-batch-2", &["INT-177"]),
            ("IS-61", &["IS-61"]),
            ("codex/memorius-slice-0-8875", &[]),
            ("main", &[]),
        ];
        for (branch, expected) in cases {
            assert_eq!(branch_issue_keys(branch), expected, "in {branch:?}");
        }
    }
}
