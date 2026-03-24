use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::state::ServerState;

use super::session_auth::has_valid_session;

pub async fn export_sql_download_handler(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if state.auth_config.is_some() && !has_valid_session(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "message": "Unauthorized"
            })),
        )
            .into_response();
    }

    match cc_switch_core::export_config_as_sql(&state.core) {
        Ok((file_name, sql_bytes)) => {
            let mut response = Response::new(Body::from(sql_bytes));
            *response.status_mut() = StatusCode::OK;
            let headers = response.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/sql; charset=utf-8"),
            );
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            let disposition = format!(r#"attachment; filename="{file_name}""#);
            if let Ok(value) = HeaderValue::from_str(&disposition) {
                headers.insert(header::CONTENT_DISPOSITION, value);
            }
            response.into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": err
            })),
        )
            .into_response(),
    }
}
