use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cc_switch_server::{
    api::{invoke_handler, upgrade_handler},
    create_event_bus,
    ServerState,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cc_switch_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create event bus
    let event_bus = create_event_bus(100);

    // Create server state
    let auth_token = std::env::var("CC_SWITCH_AUTH_TOKEN").ok();
    let state = ServerState::new(auth_token, event_bus);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let api_routes = Router::new()
        .route("/invoke", post(invoke_handler))
        .route("/ws", get(upgrade_handler))
        .with_state(state);

    let app = Router::new()
        .route("/", get(|| async { Html("<h1>CC-Switch Server</h1><p>API: /api/ws (WebSocket) | /api/invoke (HTTP)</p>") }))
        .nest("/api", api_routes)
        .layer(cors);

    // Get port from environment or use default
    let port = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3160);

    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("Starting CC-Switch server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
