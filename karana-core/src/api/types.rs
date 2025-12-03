//! API Types - Request/Response structures for the HTTP API

use serde::{Deserialize, Serialize};

// ============================================================================
// Wallet Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub did: String,
    pub public_key: String,
    pub balance: u64,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct WalletCreationResponse {
    pub did: String,
    pub public_key: String,
    pub recovery_phrase: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreWalletRequest {
    pub mnemonic: String,
}

#[derive(Debug, Deserialize)]
pub struct SignTransactionRequest {
    pub action: String,          // "TRANSFER", "STAKE", etc.
    pub recipient: String,       // DID or address
    pub amount: u64,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignedTransactionResponse {
    pub tx_hash: String,
    pub signature: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub timestamp: u64,
    pub nonce: u64,
}

// ============================================================================
// Identity Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DidInfo {
    pub did: String,
    pub created_at: u64,
    pub biometric_bound: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateDidRequest {
    pub biometric_hash: Option<String>, // Optional biometric binding
}

// ============================================================================
// AI Vision Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct VisionAnalysisRequest {
    pub image_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysisResponse {
    pub detected_object: String,
    pub category: String,
    pub description: String,
    pub confidence: f32,
    pub related_tags: Vec<String>,
    pub processing_time_ms: u64,
}

// ============================================================================
// Oracle (NLP Intent) Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OracleIntentRequest {
    pub text: String,
    pub context: Option<OracleContextData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleContextData {
    pub vision_object: Option<String>,
    pub wallet_balance: Option<u64>,
    pub active_app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    Speak,
    Transfer,
    Analyze,
    Navigate,
    Timer,
    Wallet,
    OpenApp,
    CloseApp,
    PlayVideo,
    OpenBrowser,
    TakeNote,
    SetReminder,
    PlayMusic,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleIntentResponse {
    pub intent_type: IntentType,
    pub content: String,
    pub data: Option<IntentData>,
    pub requires_confirmation: bool,
    pub suggested_actions: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentData {
    pub amount: Option<u64>,
    pub recipient: Option<String>,
    pub location: Option<String>,
    pub duration: Option<String>,
    pub app_type: Option<String>,
    pub url: Option<String>,
    pub query: Option<String>,
    pub memo: Option<String>,
}

// ============================================================================
// Celestia DA Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DaSubmitRequest {
    pub data: String,           // Base64 encoded data
    pub namespace: Option<String>, // Custom namespace (default: "karana")
}

#[derive(Debug, Serialize)]
pub struct DaSubmitResponse {
    pub tx_hash: String,
    pub height: u64,
    pub namespace: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DaStatusResponse {
    pub tx_hash: String,
    pub status: String,        // "pending", "confirmed", "failed"
    pub confirmations: u32,
    pub height: Option<u64>,
}

// ============================================================================
// WebSocket Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    // Server -> Client
    WalletUpdate {
        balance: u64,
        last_tx_hash: Option<String>,
    },
    TransactionConfirmed {
        tx_hash: String,
        status: String,
    },
    VisionResult {
        request_id: String,
        result: VisionAnalysisResponse,
    },
    OracleResponse {
        request_id: String,
        intent: OracleIntentResponse,
    },
    OsState {
        mode: String,
        battery: u8,
        connected: bool,
    },
    
    // Client -> Server
    Subscribe {
        channel: String, // "wallet", "transactions", "vision", "oracle"
    },
    Unsubscribe {
        channel: String,
    },
    Ping,
    Pong,
}

// ============================================================================
// Generic Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ============================================================================
// OS State Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OsMode {
    Idle,
    Analyzing,
    Oracle,
    Navigation,
    Wallet,
}

#[derive(Debug, Serialize)]
pub struct OsStateInfo {
    pub mode: OsMode,
    pub version: String,
    pub uptime_seconds: u64,
    pub wallet_connected: bool,
    pub camera_active: bool,
}

// ============================================================================
// Transaction History
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub tx_type: String,       // "TRANSFER", "REWARD", "STAKE"
    pub amount: u64,
    pub recipient: String,
    pub sender: String,
    pub timestamp: u64,
    pub status: String,        // "PENDING", "CONFIRMED", "FAILED"
    pub signature: Option<String>,
    pub da_tx_hash: Option<String>, // Celestia submission hash
}
