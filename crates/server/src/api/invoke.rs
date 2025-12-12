use std::sync::Arc;
use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::ServerState;
use super::dispatch::dispatch_command;

/// Cookie name for web session authentication
const SESSION_COOKIE_NAME: &str = "cc-switch-session";

/// Methods that bypass authentication check (public endpoints)
const PUBLIC_METHODS: &[&str] = &["auth.status", "auth.login", "auth.check"];

/// Extract session token from cookie header
fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
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

#[derive(Deserialize)]
pub struct InvokeRequest {
    pub command: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Serialize)]
pub struct InvokeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn invoke_handler(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(req): Json<InvokeRequest>,
) -> impl IntoResponse {
    // Special handling for auth.check: validate session from Cookie
    if req.command == "auth.check" {
        let session_token = extract_session_cookie(&headers);
        let valid = session_token
            .map(|token| state.session_store.validate_session(&token))
            .unwrap_or(false);

        return (
            StatusCode::OK,
            HeaderMap::new(),
            Json(InvokeResponse {
                result: Some(serde_json::json!({ "valid": valid })),
                error: None,
            }),
        );
    }

    // Auth check: skip if auth disabled or method is public
    if state.auth_config.is_some() && !PUBLIC_METHODS.contains(&req.command.as_str()) {
        let session_token = extract_session_cookie(&headers);
        let is_valid = session_token
            .map(|token| state.session_store.validate_session(&token))
            .unwrap_or(false);

        if !is_valid {
            return (
                StatusCode::UNAUTHORIZED,
                HeaderMap::new(),
                Json(InvokeResponse {
                    result: None,
                    error: Some("Unauthorized".to_string()),
                }),
            );
        }
    }

    match dispatch_command(&state, &req.command, &req.payload).await {
        Ok(result) => {
            let mut headers = HeaderMap::new();

            // Set cookie on successful auth.login
            if req.command == "auth.login" {
                if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
                    if success {
                        if let Some(token) = result.get("token").and_then(|v| v.as_str()) {
                            let cookie = format!(
                                "cc-switch-session={}; HttpOnly; SameSite=Strict; Max-Age=604800; Path=/",
                                token
                            );
                            if let Ok(cookie_value) = HeaderValue::from_str(&cookie) {
                                headers.insert(SET_COOKIE, cookie_value);
                            }
                        }
                    }
                }
            }

            (
                StatusCode::OK,
                headers,
                Json(InvokeResponse {
                    result: Some(result),
                    error: None,
                }),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            Json(InvokeResponse {
                result: None,
                error: Some(err.message),
            }),
        ),
    }
}
