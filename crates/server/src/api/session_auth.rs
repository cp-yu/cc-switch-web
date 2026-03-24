use axum::http::HeaderMap;

use crate::state::ServerState;

pub(crate) const SESSION_COOKIE_NAME: &str = "cc-switch-session";

pub(crate) fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            if cookie.starts_with(SESSION_COOKIE_NAME) {
                cookie
                    .strip_prefix(SESSION_COOKIE_NAME)
                    .and_then(|s| s.strip_prefix('='))
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
}

pub(crate) fn has_valid_session(state: &ServerState, headers: &HeaderMap) -> bool {
    extract_session_cookie(headers)
        .map(|token| state.session_store.validate_session(&token))
        .unwrap_or(false)
}
