//! Kāraṇa OS API Server
//!
//! HTTP/WebSocket server that exposes core OS functionality to web frontends.

use std::sync::Arc;
use std::net::SocketAddr;
use axum::http::Method;
use tower_http::cors::{CorsLayer, Any};

use crate::api::state::AppState;
use crate::api::routes::create_routes;
use crate::oracle::veil::OracleVeil;

/// Default port for the API server
pub const DEFAULT_PORT: u16 = 8080;

/// Start the API server (standalone mode without OracleVeil)
/// 
/// # Arguments
/// * `port` - Port to listen on (default: 8080)
/// 
/// # Example
/// ```no_run
/// use karana_core::api::start_api_server;
/// 
/// #[tokio::main]
/// async fn main() {
///     start_api_server(Some(8080)).await;
/// }
/// ```
pub async fn start_api_server(port: Option<u16>) {
    let port = port.unwrap_or(DEFAULT_PORT);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Create shared state (standalone mode)
    let state = AppState::new();
    
    // Configure CORS for React frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);
    
    // Build router with routes
    let app = create_routes(state)
        .layer(cors);
    
    log::info!("╔══════════════════════════════════════════════════════════════╗");
    log::info!("║         Kāraṇa OS API Server (Standalone Mode)               ║");
    log::info!("╠══════════════════════════════════════════════════════════════╣");
    log::info!("║  🌐 HTTP:      http://localhost:{}                          ║", port);
    log::info!("║  🔌 WebSocket: ws://localhost:{}/ws                         ║", port);
    log::info!("║  ⚠️  OracleVeil: NOT CONNECTED (using legacy Oracle)         ║");
    log::info!("╠══════════════════════════════════════════════════════════════╣");
    log::info!("║  Endpoints:                                                  ║");
    log::info!("║    POST /api/wallet/create     - Create new wallet           ║");
    log::info!("║    POST /api/wallet/restore    - Restore from mnemonic       ║");
    log::info!("║    GET  /api/wallet/info       - Get wallet info             ║");
    log::info!("║    POST /api/wallet/sign       - Sign transaction            ║");
    log::info!("║    POST /api/ai/vision         - Analyze image               ║");
    log::info!("║    POST /api/ai/oracle         - Process NLP intent          ║");
    log::info!("║    POST /api/da/submit         - Submit to Celestia          ║");
    log::info!("║    GET  /api/os/state          - Get OS state                ║");
    log::info!("╚══════════════════════════════════════════════════════════════╝");
    
    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Start the API server with OracleVeil integration (full Monad mode)
/// 
/// # Arguments
/// * `port` - Port to listen on
/// * `veil` - OracleVeil instance connected to Monad command channels
pub async fn start_api_server_with_veil(port: u16, veil: OracleVeil) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Create state with OracleVeil
    let state = AppState::with_oracle_veil(veil);
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);
    
    let app = create_routes(state).layer(cors);
    
    log::info!("╔══════════════════════════════════════════════════════════════╗");
    log::info!("║         Kāraṇa OS API Server (Monad Integrated)              ║");
    log::info!("╠══════════════════════════════════════════════════════════════╣");
    log::info!("║  🌐 HTTP:      http://localhost:{}                          ║", port);
    log::info!("║  🔌 WebSocket: ws://localhost:{}/ws                         ║", port);
    log::info!("║  🔮 OracleVeil: CONNECTED → Monad Command Channel            ║");
    log::info!("║  🔐 ZK Proofs:  ENABLED for all state mutations              ║");
    log::info!("╠══════════════════════════════════════════════════════════════╣");
    log::info!("║  Flow: API Request → OracleVeil → ZK-Sign → Monad → Backend  ║");
    log::info!("╚══════════════════════════════════════════════════════════════╝");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Start the API server with custom state (for testing)
pub async fn start_api_server_with_state(port: u16, state: Arc<AppState>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);
    
    let app = create_routes(state).layer(cors);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
