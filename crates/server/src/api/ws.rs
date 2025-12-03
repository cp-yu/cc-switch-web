use std::collections::HashSet;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::rpc::{RpcRequest, RpcResponse, RpcError};
use crate::state::ServerState;
use super::dispatch::dispatch_command;

#[derive(Deserialize)]
pub struct WsAuthQuery {
    auth: Option<String>,
}

pub async fn upgrade_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
    Query(query): Query<WsAuthQuery>,
) -> impl IntoResponse {
    // Validate token
    if let Some(expected_token) = &state.auth_token {
        match query.auth.as_deref() {
            Some(token) if token == expected_token => {}
            _ => return StatusCode::UNAUTHORIZED.into_response(),
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<ServerState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut subscriptions: HashSet<String> = HashSet::new();

    // Channel for sending responses back to the WebSocket
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Task to send messages to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Event subscription task
    let mut event_rx = state.event_bus.subscribe();
    let event_tx = tx.clone();
    let event_task = tokio::spawn(async move {
        let local_subs: HashSet<String> = HashSet::new();
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    if local_subs.contains(&event.name) {
                        let notification = RpcResponse::notification(
                            "event",
                            serde_json::json!({
                                "name": event.name,
                                "payload": event.payload
                            }),
                        );
                        let text = serde_json::to_string(&notification).unwrap();
                        if event_tx.send(text).await.is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Message receiving loop
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            if let Some(response) = handle_message(&mut subscriptions, &text).await {
                if tx.send(response).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup
    drop(tx);
    send_task.abort();
    event_task.abort();
}

async fn handle_message(
    subscriptions: &mut HashSet<String>,
    text: &str,
) -> Option<String> {
    let request: RpcRequest = match serde_json::from_str(text) {
        Ok(req) => req,
        Err(_) => {
            let err = RpcResponse::error(None, RpcError::parse_error());
            return Some(serde_json::to_string(&err).unwrap());
        }
    };

    // Notification (no response needed)
    if request.is_notification() {
        handle_notification(&request);
        return None;
    }

    let id = request.id.clone().unwrap();

    // System commands
    let response = match request.method.as_str() {
        "event.subscribe" => {
            let event = request
                .params
                .get("event")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            subscriptions.insert(event.to_string());
            RpcResponse::success(id, serde_json::json!({ "ok": true }))
        }
        "event.unsubscribe" => {
            let event = request
                .params
                .get("event")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            subscriptions.remove(event);
            RpcResponse::success(id, serde_json::json!({ "ok": true }))
        }
        "ping" => RpcResponse::success(id, serde_json::json!({ "pong": true })),
        _ => {
            // Business commands
            match dispatch_command(&request.method, &request.params).await {
                Ok(result) => RpcResponse::success(id, result),
                Err(err) => RpcResponse::error(Some(id), err),
            }
        }
    };

    Some(serde_json::to_string(&response).unwrap())
}

fn handle_notification(request: &RpcRequest) {
    match request.method.as_str() {
        "ping" => { /* Ignore */ }
        _ => {
            tracing::debug!("Received notification: {}", request.method);
        }
    }
}
