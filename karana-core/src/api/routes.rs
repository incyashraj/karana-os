//! API Routes configuration

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
};

use crate::api::state::AppState;
use crate::api::handlers;

/// Build all API routes
pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Wallet endpoints
        .route("/api/wallet/create", post(handlers::create_wallet))
        .route("/api/wallet/restore", post(handlers::restore_wallet))
        .route("/api/wallet/info", get(handlers::get_wallet_info))
        .route("/api/wallet/sign", post(handlers::sign_transaction))
        .route("/api/wallet/transactions", get(handlers::get_transactions))
        
        // AI endpoints
        .route("/api/ai/vision", post(handlers::analyze_vision))
        .route("/api/ai/oracle", post(handlers::process_oracle))
        
        // Use Case endpoints (Phase 2: Glasses-Ready)
        .route("/api/use-case", post(handlers::execute_use_case))
        
        // Confirmation endpoints (for sensitive operations)
        .route("/api/confirm/pending", get(handlers::get_pending_actions))
        .route("/api/confirm/action", post(handlers::confirm_action))
        
        // Manifest endpoints (AR whispers / haptic output)
        .route("/api/manifest", get(handlers::get_manifest_state))
        .route("/api/manifest/clear", post(handlers::clear_manifest))
        
        // Celestia DA endpoints
        .route("/api/da/submit", post(handlers::submit_to_da))
        .route("/api/da/status/{tx_hash}", get(handlers::get_da_status))
        
        // OS state
        .route("/api/os/state", get(handlers::get_os_state))
        
        // WebSocket
        .route("/ws", get(handlers::ws_handler))
        
        // Health check
        .route("/health", get(health_check))
        
        // Add shared state
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Kāraṇa OS API Server - OK"
}
