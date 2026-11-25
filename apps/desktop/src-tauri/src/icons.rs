//! `appicon://localhost/<app id>` serves the app's icon as a PNG, so the
//! WebView loads and caches icons like any image.

use std::borrow::Cow;

use percent_encoding::percent_decode_str;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Runtime, UriSchemeContext, UriSchemeResponder};

pub const SCHEME: &str = "appicon";

#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri's URI scheme handler signature"
)]
pub fn respond<R: Runtime>(
    _context: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app_id = percent_decode_str(request.uri().path().trim_start_matches('/'))
        .decode_utf8_lossy()
        .into_owned();
    // Icon files are read from disk; keep that off the WebView's thread.
    tauri::async_runtime::spawn_blocking(move || {
        let response = match memorius_sources::app_icon_png(&app_id) {
            Some(png) => Response::builder()
                .header(header::CONTENT_TYPE, "image/png")
                .header(header::CACHE_CONTROL, "max-age=86400")
                .body(Cow::Owned(png)),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Cow::Borrowed(&[][..])),
        };
        if let Ok(response) = response {
            responder.respond(response);
        }
    });
}
