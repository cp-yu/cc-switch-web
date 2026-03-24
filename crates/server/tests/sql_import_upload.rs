use std::sync::Arc;

use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    routing::post,
    Router,
};
use cc_switch::{AppState, Database};
use cc_switch_core::CoreContext;
use cc_switch_server::{
    api::import_sql_upload_handler,
    create_event_bus,
    AuthConfig, ServerState, SessionStore,
};
use tower::util::ServiceExt;

#[tokio::test]
async fn unauthenticated_sql_upload_is_rejected_when_web_auth_is_enabled() {
    let db = Arc::new(Database::memory().expect("in-memory database"));
    let state = Arc::new(ServerState {
        auth_token: None,
        event_bus: create_event_bus(8),
        core: CoreContext::from_app_state(AppState::new(db)),
        session_store: Arc::new(SessionStore::new()),
        auth_config: Some(AuthConfig {
            password_hash: "test-hash".to_string(),
        }),
    });

    let app = Router::new()
        .route("/api/import-config", post(import_sql_upload_handler))
        .with_state(state);

    let boundary = "X-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"config.sql\"\r\nContent-Type: application/sql\r\n\r\n-- CC Switch SQLite 导出\nSELECT 1;\r\n--{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/import-config")
                .header(
                    CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn invalid_sql_upload_does_not_pollute_existing_database() {
    let db = Arc::new(Database::memory().expect("in-memory database"));
    let before = db.export_sql_string().expect("export before");
    let state = Arc::new(ServerState {
        auth_token: None,
        event_bus: create_event_bus(8),
        core: CoreContext::from_app_state(AppState::new(db.clone())),
        session_store: Arc::new(SessionStore::new()),
        auth_config: None,
    });

    let app = Router::new()
        .route("/api/import-config", post(import_sql_upload_handler))
        .with_state(state);

    let boundary = "X-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"config.sql\"\r\nContent-Type: application/sql\r\n\r\n-- CC Switch SQLite 导出\nTHIS IS NOT VALID SQL;\r\n--{boundary}--\r\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/import-config")
                .header(
                    CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let after = db.export_sql_string().expect("export after");
    assert_eq!(before, after, "failed upload should not mutate existing data");
}
