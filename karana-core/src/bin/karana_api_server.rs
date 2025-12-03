//! Kāraṇa OS API Server Binary
//!
//! Run with: cargo run --bin karana-api-server
//!
//! This exposes the Kāraṇa OS functionality via HTTP/WebSocket for the React simulator.

use karana_core::api::start_api_server;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::info!("🚀 Starting Kāraṇa OS API Server...");
    
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    
    // Start the server
    start_api_server(Some(port)).await;
}
