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
    // ═══ Server -> Client: Wallet Events ═══
    WalletUpdate {
        balance: u64,
        last_tx_hash: Option<String>,
    },
    TransactionConfirmed {
        tx_hash: String,
        status: String,
    },
    
    // ═══ Server -> Client: Vision Events ═══
    VisionResult {
        request_id: String,
        result: VisionAnalysisResponse,
    },
    
    // ═══ Server -> Client: Oracle Events ═══
    /// Oracle is processing (show thinking indicator)
    OracleThinking {
        intent: String,
        stage: String,  // "parsing", "zk_proving", "executing", "manifesting"
    },
    
    /// Oracle whisper (AR text overlay)
    OracleWhisper {
        id: String,
        content: String,
        style: String,      // "subtle", "emphasized", "urgent", "info", "success", "error"
        position: String,   // "top_left", "top_right", "center", "bottom"
        duration_ms: u64,
    },
    
    /// Oracle haptic feedback
    OracleHaptic {
        pattern: String,    // "success", "confirm", "error", "attention", "thinking"
        intensity: f32,     // 0.0 - 1.0
    },
    
    /// Oracle requires confirmation
    OracleConfirmation {
        action_id: String,
        action_type: String,
        description: String,
        expires_at: u64,
        confidence: f32,
    },
    
    /// Oracle completed intent processing
    OracleResponse {
        request_id: String,
        intent: OracleIntentResponse,
    },
    
    /// Oracle error
    OracleError {
        request_id: Option<String>,
        error: String,
        recoverable: bool,
    },
    
    // ═══ Server -> Client: System Events ═══
    OsState {
        mode: String,
        battery: u8,
        connected: bool,
    },
    
    /// System status update
    SystemStatus {
        ai_model: String,
        zk_queue: usize,
        swarm_peers: usize,
        chain_height: u64,
    },
    
    // ═══ Client -> Server ═══
    Subscribe {
        channel: String, // "wallet", "transactions", "vision", "oracle", "system"
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

// ============================================================================
// Pending Confirmation Types
// ============================================================================

/// A pending action awaiting user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Unique identifier for this pending action
    pub id: String,
    /// Type of action (TRANSFER, STAKE, VOTE, etc.)
    pub action_type: IntentType,
    /// Human-readable description of the action
    pub description: String,
    /// Structured data for the action
    pub data: Option<IntentData>,
    /// ZK proof of intent (base64 encoded)
    pub zk_proof: Option<String>,
    /// When this action was created (Unix timestamp)
    pub created_at: u64,
    /// When this action expires (Unix timestamp)
    pub expires_at: u64,
    /// Confidence level of the intent recognition
    pub confidence: f32,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmActionRequest {
    /// ID of the pending action to confirm
    pub action_id: String,
    /// Whether to confirm (true) or reject (false)
    pub approved: bool,
}

#[derive(Debug, Serialize)]
pub struct ConfirmActionResponse {
    /// Whether the action was processed successfully
    pub success: bool,
    /// The resulting transaction hash (if approved)
    pub tx_hash: Option<String>,
    /// Response message
    pub message: String,
}

// ============================================================================
// Manifest Output Types (AR Whispers / Haptic)
// ============================================================================

/// Current manifest state for frontend rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestState {
    /// Active whisper overlays
    pub whispers: Vec<WhisperOverlay>,
    /// Last haptic pattern played
    pub last_haptic: Option<String>,
    /// Output mode
    pub mode: String,
}

/// A whisper overlay for AR display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperOverlay {
    /// Unique ID for this whisper
    pub id: String,
    /// Text content
    pub content: String,
    /// Style: "subtle", "emphasized", "urgent", "info", "success", "error"
    pub style: String,
    /// Position: "top_left", "top_right", "center", etc.
    pub position: String,
    /// Remaining duration in milliseconds (0 = permanent until dismissed)
    pub remaining_ms: u64,
}

// ============================================================================
// Use Case Types (Phase 2: Glasses Use Cases)
// ============================================================================

/// Request for use case execution
#[derive(Debug, Clone, Deserialize)]
pub struct UseCaseRequest {
    /// Category: "productivity", "health", "social", "navigation"
    pub category: String,
    /// Specific intent within the category
    pub intent: String,
    /// Parameters for the use case
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Response from use case execution
#[derive(Debug, Clone, Serialize)]
pub struct UseCaseResponse {
    /// Success status
    pub success: bool,
    /// Whisper message for AR
    pub whisper: String,
    /// Haptic pattern to play
    pub haptic: String,
    /// AR overlay content (if any)
    pub overlay: Option<AROverlayResponse>,
    /// Confidence score
    pub confidence: f32,
    /// Generated files or artifacts
    pub artifacts: Vec<String>,
}

/// AR overlay response
#[derive(Debug, Clone, Serialize)]
pub struct AROverlayResponse {
    /// Content to display
    pub content: String,
    /// Position (x, y) normalized 0.0-1.0
    pub position: (f32, f32),
    /// Duration in ms (0 = persistent)
    pub duration_ms: u64,
    /// Overlay type: "whisper", "status", "navigation", "highlight", "progress"
    pub overlay_type: String,
    /// Style: "subtle", "normal", "emphasized"
    pub style: String,
}
