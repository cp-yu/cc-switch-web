use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cc_switch_server::{
    api::{invoke_handler, upgrade_handler},
    create_event_bus,
    load_auth_config, SessionStore,
    ServerState,
};

// 嵌入前端静态文件（构建时从 dist 目录读取）
#[derive(RustEmbed)]
#[folder = "../../dist/"]
struct Assets;

// 静态文件处理器
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // 如果路径为空或者不包含扩展名，返回 index.html（SPA 路由支持）
    let path = if path.is_empty() || (!path.contains('.') && !path.starts_with("api/")) {
        "index.html"
    } else {
        path
    };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // 对于 SPA，非 API 请求返回 index.html
            if !path.starts_with("api/") {
                if let Some(content) = Assets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(content.data.into_owned()))
                        .unwrap();
                }
            }
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 Not Found"))
                .unwrap()
        }
    }
}

// 健康检查和欢迎页面
async fn welcome_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>CC-Switch Web</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #2563eb; }
        .info { background: #f1f5f9; padding: 20px; border-radius: 8px; }
        code { background: #e2e8f0; padding: 2px 6px; border-radius: 4px; }
        a { color: #2563eb; }
    </style>
</head>
<body>
    <h1>🚀 CC-Switch Web Server</h1>
    <div class="info">
        <p><strong>Status:</strong> Running</p>
        <p><strong>API Endpoints:</strong></p>
        <ul>
            <li>HTTP: <code>POST /api/invoke</code></li>
            <li>WebSocket: <code>GET /api/ws</code></li>
        </ul>
        <p><strong>Frontend:</strong> <a href="/">Open Web UI</a></p>
    </div>
</body>
</html>"#)
}

/// 检查端口是否可用
fn is_port_available(host: &str, port: u16) -> bool {
    StdTcpListener::bind(format!("{}:{}", host, port)).is_ok()
}

/// 查找可用端口（从指定端口开始，最多尝试 100 个端口）
fn find_available_port(host: &str, start_port: u16) -> Option<u16> {
    for port in start_port..start_port.saturating_add(100) {
        if is_port_available(host, port) {
            return Some(port);
        }
    }
    None
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cc_switch_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create event bus
    let event_bus = create_event_bus(100);

    // Load auth configuration (None means auth is disabled)
    let auth_config = load_auth_config();
    if auth_config.is_some() {
        tracing::info!("Web authentication enabled");
    } else {
        tracing::info!("Web authentication disabled (no config found)");
    }

    // Create session store
    let session_store = Arc::new(SessionStore::new());

    // Spawn session cleanup task (runs every hour)
    let cleanup_store = Arc::clone(&session_store);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            cleanup_store.cleanup_expired();
            tracing::debug!("Session cleanup completed");
        }
    });

    // Create server state
    let auth_token = std::env::var("CC_SWITCH_AUTH_TOKEN").ok();
    let state = ServerState::new(auth_token, event_bus, session_store, auth_config);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build API routes
    let api_routes = Router::new()
        .route("/invoke", post(invoke_handler))
        .route("/ws", get(upgrade_handler))
        .with_state(state);

    // Check if frontend assets are embedded
    let has_frontend = Assets::get("index.html").is_some();

    let app = if has_frontend {
        tracing::info!("Frontend assets embedded, serving SPA");
        Router::new()
            .nest("/api", api_routes)
            .route("/health", get(welcome_handler))
            .fallback(static_handler)
            .layer(cors)
    } else {
        tracing::warn!("No frontend assets found, running in API-only mode");
        tracing::warn!("Build frontend first: pnpm build:web");
        Router::new()
            .route("/", get(welcome_handler))
            .route("/health", get(welcome_handler))
            .nest("/api", api_routes)
            .layer(cors)
    };

    // Get port from environment or use default
    let requested_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(17666);

    // Get host from environment or use default
    let host = std::env::var("CC_SWITCH_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    // Check if auto-port selection is enabled (default: true)
    let auto_port = std::env::var("CC_SWITCH_AUTO_PORT")
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);

    // Find available port
    let port = if is_port_available(&host, requested_port) {
        requested_port
    } else if auto_port {
        eprintln!();
        eprintln!("⚠️  Port {} is already in use", requested_port);
        match find_available_port(&host, requested_port + 1) {
            Some(p) => {
                eprintln!("   Automatically using port {} instead", p);
                eprintln!("   To disable auto-port: CC_SWITCH_AUTO_PORT=false");
                eprintln!();
                p
            }
            None => {
                eprintln!("❌ Error: Could not find an available port");
                eprintln!("   Tried ports {} to {}", requested_port, requested_port + 100);
                eprintln!();
                eprintln!("   Solutions:");
                eprintln!("   1. Stop the process using port {}: lsof -ti:{} | xargs kill", requested_port, requested_port);
                eprintln!("   2. Use a different port: CC_SWITCH_PORT=8080 ./cc-switch-web");
                eprintln!();
                std::process::exit(1);
            }
        }
    } else {
        eprintln!();
        eprintln!("❌ Error: Port {} is already in use", requested_port);
        eprintln!();
        eprintln!("   Solutions:");
        eprintln!("   1. Stop the process using this port:");
        eprintln!("      lsof -ti:{} | xargs kill", requested_port);
        eprintln!();
        eprintln!("   2. Use a different port:");
        eprintln!("      CC_SWITCH_PORT=8080 ./cc-switch-web");
        eprintln!();
        eprintln!("   3. Enable auto-port selection (default):");
        eprintln!("      CC_SWITCH_AUTO_PORT=true ./cc-switch-web");
        eprintln!();
        std::process::exit(1);
    };

    let addr = format!("{}:{}", host, port);

    println!();
    println!("╔════════════════════════════════════════════════════╗");
    println!("║           CC-Switch Web Server v0.1.0              ║");
    println!("╠════════════════════════════════════════════════════╣");
    if has_frontend {
        println!("║  🌐 Web UI:    http://{}:{:<21}║", host, port);
    }
    println!("║  📡 API:       http://{}:{}/api{:14}║", host, port, "");
    println!("║  🔌 WebSocket: ws://{}:{}/api/ws{:11}║", host, port, "");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║  Press Ctrl+C to stop                              ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!();

    tracing::info!("Starting CC-Switch server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("❌ Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }
}
