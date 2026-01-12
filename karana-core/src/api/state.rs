//! Shared Application State for the API server

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use crate::wallet::KaranaWallet;
use crate::api::types::{Transaction, OsMode, PendingAction, WhisperOverlay, ManifestState};
use crate::oracle::command::{OracleCommand, CommandResult, CommandData, OracleChannels, MonadChannels};
use crate::oracle::veil::OracleVeil;
use crate::assistant::ToolRegistry;

/// Default expiration time for pending actions (60 seconds)
const PENDING_ACTION_TTL_SECS: u64 = 60;

/// Default whisper duration (3 seconds)
pub const DEFAULT_WHISPER_DURATION_MS: u64 = 3000;

/// Tracked whisper with timing info
struct TrackedWhisper {
    overlay: WhisperOverlay,
    expires_at: u64, // Unix timestamp ms
}

/// Shared state accessible across all API handlers
pub struct AppState {
    /// The active wallet (if one exists)
    pub wallet: RwLock<Option<KaranaWallet>>,
    
    /// Token balance (in a real system, this would come from the chain)
    pub balance: RwLock<u64>,
    
    /// Transaction history
    pub transactions: RwLock<Vec<Transaction>>,
    
    /// Transaction nonce counter
    pub nonce: RwLock<u64>,
    
    /// Current OS mode
    pub mode: RwLock<OsMode>,
    
    /// WebSocket subscribers by channel
    pub ws_subscribers: RwLock<HashMap<String, Vec<tokio::sync::mpsc::Sender<String>>>>,
    
    /// Celestia RPC endpoint
    pub celestia_endpoint: Option<String>,
    
    /// Pending actions awaiting user confirmation
    pub pending_actions: RwLock<HashMap<String, PendingAction>>,
    
    /// Active whispers for manifest rendering
    pub whispers: RwLock<Vec<TrackedWhisper>>,
    
    /// Last haptic pattern played
    pub last_haptic: RwLock<Option<String>>,
    
    /// Oracle Veil for mediated intent processing (optional - None for standalone API mode)
    pub oracle_veil: Option<Arc<Mutex<OracleVeil>>>,
    
    /// Tool Registry for executing Oracle intents
    pub tool_registry: Option<Arc<ToolRegistry>>,
}

impl AppState {
    /// Create standalone API state (without OracleVeil)
    pub fn new() -> Arc<Self> {
        // Initialize default tool registry
        let tool_registry = match ToolRegistry::new() {
            Ok(registry) => Some(Arc::new(registry)),
            Err(e) => {
                log::warn!("[AppState] Failed to initialize ToolRegistry: {}", e);
                None
            }
        };
        
        Arc::new(Self {
            wallet: RwLock::new(None),
            balance: RwLock::new(1000), // Start with 1000 KARA for testing
            transactions: RwLock::new(Vec::new()),
            nonce: RwLock::new(0),
            mode: RwLock::new(OsMode::Idle),
            ws_subscribers: RwLock::new(HashMap::new()),
            celestia_endpoint: std::env::var("CELESTIA_RPC_URL").ok(),
            pending_actions: RwLock::new(HashMap::new()),
            whispers: RwLock::new(Vec::new()),
            last_haptic: RwLock::new(None),
            oracle_veil: None,
            tool_registry,
        })
    }
    
    /// Create API state with OracleVeil (for full Monad integration)
    pub fn with_oracle_veil(veil: OracleVeil) -> Arc<Self> {
        // Initialize default tool registry
        let tool_registry = match ToolRegistry::new() {
            Ok(registry) => Some(Arc::new(registry)),
            Err(e) => {
                log::warn!("[AppState] Failed to initialize ToolRegistry: {}", e);
                None
            }
        };
        
        Arc::new(Self {
            wallet: RwLock::new(None),
            balance: RwLock::new(1000),
            transactions: RwLock::new(Vec::new()),
            nonce: RwLock::new(0),
            mode: RwLock::new(OsMode::Idle),
            ws_subscribers: RwLock::new(HashMap::new()),
            celestia_endpoint: std::env::var("CELESTIA_RPC_URL").ok(),
            pending_actions: RwLock::new(HashMap::new()),
            whispers: RwLock::new(Vec::new()),
            last_haptic: RwLock::new(None),
            oracle_veil: Some(Arc::new(Mutex::new(veil))),
            tool_registry,
        })
    }
    
    /// Get the next nonce
    pub async fn next_nonce(&self) -> u64 {
        let mut nonce = self.nonce.write().await;
        let current = *nonce;
        *nonce += 1;
        current
    }
    
    /// Update balance
    pub async fn update_balance(&self, new_balance: u64) {
        *self.balance.write().await = new_balance;
    }
    
    /// Subtract from balance
    pub async fn debit(&self, amount: u64) -> bool {
        let mut balance = self.balance.write().await;
        if *balance >= amount {
            *balance -= amount;
            true
        } else {
            false
        }
    }
    
    /// Add to balance
    pub async fn credit(&self, amount: u64) {
        let mut balance = self.balance.write().await;
        *balance += amount;
    }
    
    /// Add a transaction to history
    pub async fn add_transaction(&self, tx: Transaction) {
        let mut transactions = self.transactions.write().await;
        transactions.insert(0, tx); // Most recent first
        // Keep only last 100 transactions
        if transactions.len() > 100 {
            transactions.pop();
        }
    }
    
    /// Set OS mode
    pub async fn set_mode(&self, mode: OsMode) {
        *self.mode.write().await = mode;
    }
    
    /// Add a pending action awaiting confirmation
    pub async fn add_pending_action(&self, action: PendingAction) {
        let mut pending = self.pending_actions.write().await;
        pending.insert(action.id.clone(), action);
    }
    
    /// Get a pending action by ID
    pub async fn get_pending_action(&self, id: &str) -> Option<PendingAction> {
        let pending = self.pending_actions.read().await;
        pending.get(id).cloned()
    }
    
    /// Remove a pending action (after confirmation or rejection)
    pub async fn remove_pending_action(&self, id: &str) -> Option<PendingAction> {
        let mut pending = self.pending_actions.write().await;
        pending.remove(id)
    }
    
    /// Get all pending actions (filtering expired ones)
    pub async fn get_all_pending_actions(&self) -> Vec<PendingAction> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut pending = self.pending_actions.write().await;
        
        // Remove expired actions
        pending.retain(|_, action| action.expires_at > now);
        
        // Return remaining actions
        pending.values().cloned().collect()
    }
    
    /// Create a new pending action with default TTL
    pub fn create_pending_action(
        action_type: crate::api::types::IntentType,
        description: String,
        data: Option<crate::api::types::IntentData>,
        zk_proof: Option<Vec<u8>>,
        confidence: f32,
    ) -> PendingAction {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        PendingAction {
            id: uuid::Uuid::new_v4().to_string(),
            action_type,
            description,
            data,
            zk_proof: zk_proof.map(|p| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &p)),
            created_at: now,
            expires_at: now + PENDING_ACTION_TTL_SECS,
            confidence,
        }
    }
    
    /// Broadcast message to WebSocket subscribers
    pub async fn broadcast(&self, channel: &str, message: &str) {
        let subscribers = self.ws_subscribers.read().await;
        if let Some(subs) = subscribers.get(channel) {
            for tx in subs {
                let _ = tx.send(message.to_string()).await;
            }
        }
    }
    
    // ========================================================================
    // Manifest / Whisper Methods
    // ========================================================================
    
    /// Add a whisper overlay, returns the whisper ID
    pub async fn add_whisper(
        &self,
        content: String,
        style: &str,
        position: &str,
        duration_ms: Option<u64>,
    ) -> String {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let duration = duration_ms.unwrap_or(DEFAULT_WHISPER_DURATION_MS);
        let whisper_id = uuid::Uuid::new_v4().to_string();
        
        let whisper = TrackedWhisper {
            overlay: WhisperOverlay {
                id: whisper_id.clone(),
                content,
                style: style.to_string(),
                position: position.to_string(),
                remaining_ms: duration,
            },
            expires_at: now_ms + duration,
        };
        
        let mut whispers = self.whispers.write().await;
        whispers.push(whisper);
        
        // Keep only last 5 whispers
        if whispers.len() > 5 {
            whispers.remove(0);
        }
        
        whisper_id
    }
    
    /// Set last haptic pattern
    pub async fn set_haptic(&self, pattern: &str) {
        *self.last_haptic.write().await = Some(pattern.to_string());
    }
    
    /// Get current manifest state
    pub async fn get_manifest_state(&self) -> ManifestState {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let mut whispers = self.whispers.write().await;
        
        // Remove expired whispers
        whispers.retain(|w| w.expires_at > now_ms);
        
        // Convert to output format with remaining time
        let active_whispers: Vec<WhisperOverlay> = whispers.iter()
            .map(|w| WhisperOverlay {
                id: w.overlay.id.clone(),
                content: w.overlay.content.clone(),
                style: w.overlay.style.clone(),
                position: w.overlay.position.clone(),
                remaining_ms: w.expires_at.saturating_sub(now_ms),
            })
            .collect();
        
        let last_haptic = self.last_haptic.read().await.clone();
        let mode = self.mode.read().await.clone();
        
        ManifestState {
            whispers: active_whispers,
            last_haptic,
            mode: format!("{:?}", mode),
        }
    }
    
    /// Clear all whispers
    pub async fn clear_whispers(&self) {
        self.whispers.write().await.clear();
    }
    
    // ========================================================================
    // WebSocket Oracle Event Broadcasting
    // ========================================================================
    
    /// Broadcast Oracle thinking state
    pub async fn broadcast_oracle_thinking(&self, intent: &str, stage: &str) {
        let msg = serde_json::to_string(&crate::api::types::WsMessage::OracleThinking {
            intent: intent.to_string(),
            stage: stage.to_string(),
        }).unwrap();
        self.broadcast("oracle", &msg).await;
    }
    
    /// Broadcast Oracle whisper
    pub async fn broadcast_oracle_whisper(
        &self,
        id: &str,
        content: &str,
        style: &str,
        position: &str,
        duration_ms: u64,
    ) {
        let msg = serde_json::to_string(&crate::api::types::WsMessage::OracleWhisper {
            id: id.to_string(),
            content: content.to_string(),
            style: style.to_string(),
            position: position.to_string(),
            duration_ms,
        }).unwrap();
        self.broadcast("oracle", &msg).await;
    }
    
    /// Broadcast Oracle haptic feedback
    pub async fn broadcast_oracle_haptic(&self, pattern: &str, intensity: f32) {
        let msg = serde_json::to_string(&crate::api::types::WsMessage::OracleHaptic {
            pattern: pattern.to_string(),
            intensity,
        }).unwrap();
        self.broadcast("oracle", &msg).await;
    }
    
    /// Broadcast Oracle confirmation request
    pub async fn broadcast_oracle_confirmation(
        &self,
        action_id: &str,
        action_type: &str,
        description: &str,
        expires_at: u64,
        confidence: f32,
    ) {
        let msg = serde_json::to_string(&crate::api::types::WsMessage::OracleConfirmation {
            action_id: action_id.to_string(),
            action_type: action_type.to_string(),
            description: description.to_string(),
            expires_at,
            confidence,
        }).unwrap();
        self.broadcast("oracle", &msg).await;
    }
    
    /// Broadcast Oracle error
    pub async fn broadcast_oracle_error(&self, request_id: Option<&str>, error: &str, recoverable: bool) {
        let msg = serde_json::to_string(&crate::api::types::WsMessage::OracleError {
            request_id: request_id.map(|s| s.to_string()),
            error: error.to_string(),
            recoverable,
        }).unwrap();
        self.broadcast("oracle", &msg).await;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wallet: RwLock::new(None),
            balance: RwLock::new(1000),
            transactions: RwLock::new(Vec::new()),
            nonce: RwLock::new(0),
            mode: RwLock::new(OsMode::Idle),
            ws_subscribers: RwLock::new(HashMap::new()),
            celestia_endpoint: None,
            pending_actions: RwLock::new(HashMap::new()),
            whispers: RwLock::new(Vec::new()),
            last_haptic: RwLock::new(None),
            oracle_veil: None,
        }
    }
}
