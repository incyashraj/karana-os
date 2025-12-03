//! # Kāraṇa OS HTTP/WebSocket API Server
//!
//! Exposes the core OS functionality to web-based frontends like the React simulator.
//!
//! ## Endpoints
//! 
//! ### Wallet
//! - POST /api/wallet/create - Create new wallet with mnemonic
//! - POST /api/wallet/restore - Restore wallet from mnemonic
//! - GET  /api/wallet/info - Get wallet DID and public key
//! - POST /api/wallet/sign - Sign a transaction
//!
//! ### Identity  
//! - GET  /api/identity/did - Get active DID
//! - POST /api/identity/create - Create new DID with biometrics
//!
//! ### AI
//! - POST /api/ai/vision - Analyze image for object detection
//! - POST /api/ai/oracle - Process natural language intent
//!
//! ### Celestia DA
//! - POST /api/da/submit - Submit data to Celestia
//! - GET  /api/da/status/:tx_hash - Check submission status
//!
//! ### WebSocket
//! - WS   /ws - Real-time updates for transactions, analysis, OS state

pub mod server;
pub mod routes;
pub mod handlers;
pub mod types;
pub mod state;

pub use server::start_api_server;
