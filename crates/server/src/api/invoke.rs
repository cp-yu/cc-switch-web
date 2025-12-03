use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::ServerState;
use super::dispatch::dispatch_command;

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
    Json(req): Json<InvokeRequest>,
) -> impl IntoResponse {
    match dispatch_command(&state, &req.command, &req.payload).await {
        Ok(result) => (
            StatusCode::OK,
            Json(InvokeResponse {
                result: Some(result),
                error: None,
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(InvokeResponse {
                result: None,
                error: Some(err.message),
            }),
        ),
    }
}
