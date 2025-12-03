//! Oracle Command System - Channel-based communication between Oracle and Monad
//!
//! This module defines the command protocol that allows the OracleVeil to be the SOLE
//! interface to the backend. All user intents flow through:
//!
//! User Intent → Oracle Parse → ZK-Sign → Command Channel → Monad Execute → Result
//!
//! The Monad ONLY accepts commands from the Oracle via these channels.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// ============================================================================
// ORACLE COMMANDS - What the Oracle can ask the Monad to do
// ============================================================================

/// Commands that ONLY the Oracle can send to the Monad
/// Every command that mutates state MUST include a ZK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleCommand {
    // ═══════════════════════════════════════════════════════════════════════
    // STORAGE COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Store data with ZK proof of intent
    StoreData {
        data: Vec<u8>,
        metadata: String,
        zk_proof: Vec<u8>,
    },
    
    /// Retrieve data by key (requires access proof)
    RetrieveData {
        key: Vec<u8>,
        requester_did: String,
        zk_proof: Vec<u8>,
    },
    
    /// Semantic search across stored data
    SearchSemantic {
        query: String,
        limit: usize,
    },
    
    /// Get user's stored files list
    ListUserFiles {
        did: String,
        limit: usize,
    },
    
    // ═══════════════════════════════════════════════════════════════════════
    // CHAIN/LEDGER COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Submit a signed transaction
    SubmitTransaction {
        tx_data: TransactionPayload,
        zk_proof: Vec<u8>,
    },
    
    /// Query balance for a DID
    QueryBalance {
        did: String,
    },
    
    /// Query chain state (blocks, txs, proposals)
    QueryChainState {
        query_type: ChainQuery,
    },
    
    /// Get transaction history for a DID
    GetTransactionHistory {
        did: String,
        limit: usize,
    },
    
    // ═══════════════════════════════════════════════════════════════════════
    // SWARM/P2P COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Broadcast a message to the swarm
    BroadcastMessage {
        topic: String,
        payload: Vec<u8>,
        zk_proof: Vec<u8>,
    },
    
    /// Dial a specific peer
    DialPeer {
        multiaddr: String,
    },
    
    /// Get swarm peer info
    GetPeerInfo,
    
    /// Sync clipboard across devices
    SyncClipboard {
        content: String,
        did: String,
        zk_proof: Vec<u8>,
    },
    
    // ═══════════════════════════════════════════════════════════════════════
    // RUNTIME COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Execute a WASM module
    ExecuteWasm {
        module_hash: Vec<u8>,
        params: Vec<u8>,
        gas_limit: u64,
    },
    
    /// Schedule a delayed task
    ScheduleTask {
        task_id: String,
        delay_ms: u64,
        command: Box<OracleCommand>,
    },
    
    /// Cancel a scheduled task
    CancelTask {
        task_id: String,
    },
    
    // ═══════════════════════════════════════════════════════════════════════
    // HARDWARE COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Play a haptic pattern
    PlayHaptic {
        pattern: HapticPattern,
    },
    
    /// Update AR display overlay
    UpdateAROverlay {
        overlay: AROverlay,
    },
    
    /// Get hardware status
    GetHardwareStatus,
    
    // ═══════════════════════════════════════════════════════════════════════
    // SYSTEM COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Get full pipeline status
    GetPipelineStatus,
    
    /// Trigger ZK batch proving
    TriggerZKBatch,
    
    /// Get system metrics
    GetMetrics,
    
    /// Graceful shutdown
    Shutdown,
    
    // ═══════════════════════════════════════════════════════════════════════
    // SPATIAL AR COMMANDS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Pin content at user's current position
    SpatialPinHere {
        content_type: String,
        label: Option<String>,
    },
    
    /// Pin content at a described location
    SpatialPinAt {
        content_type: String,
        target: String,
    },
    
    /// Find nearby anchored content
    SpatialFindNearby {
        radius_m: f32,
    },
    
    /// Navigate to an anchor by label or type
    SpatialNavigateTo {
        query: String,
    },
    
    /// Remove an anchor
    SpatialRemoveAnchor {
        anchor_id: Option<u64>,
    },
    
    /// Save current room for relocalization
    SpatialSaveRoom {
        name: String,
    },
    
    /// List all user's spatial anchors
    SpatialListAnchors,
    
    /// Open a browser tab at current position
    SpatialOpenTab {
        url: String,
    },
}

// ============================================================================
// COMMAND PAYLOADS
// ============================================================================

/// Transaction payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// Transfer tokens to another DID
    Transfer {
        to: String,
        amount: u128,
        memo: Option<String>,
    },
    
    /// Stake tokens for consensus participation
    Stake {
        amount: u128,
    },
    
    /// Unstake previously staked tokens
    Unstake {
        amount: u128,
    },
    
    /// Vote on a governance proposal
    Vote {
        proposal_id: u64,
        approve: bool,
    },
    
    /// Attest data storage on-chain
    StoreAttestation {
        data_hash: Vec<u8>,
        proof: Vec<u8>,
    },
    
    /// Create a governance proposal
    CreateProposal {
        title: String,
        description: String,
    },
}

/// Chain query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainQuery {
    /// Get the latest block
    LatestBlock,
    
    /// Get block by height
    BlockByHeight(u64),
    
    /// Get transaction by hash
    TransactionByHash(String),
    
    /// Get proposal status
    ProposalStatus(u64),
    
    /// Get all active proposals
    ActiveProposals,
    
    /// Get node/chain info
    NodeInfo,
    
    /// Get staking info for a DID
    StakingInfo(String),
}

/// Haptic feedback patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticPattern {
    /// Single short pulse - action completed
    Success,
    
    /// Double tap - confirmation needed
    Confirm,
    
    /// Triple harsh - error occurred
    Error,
    
    /// Escalating pulse - attention needed
    Attention,
    
    /// Gentle repeating - processing
    Thinking,
    
    /// Directional tick - navigation
    Navigation { direction: NavigationDirection },
    
    /// Custom pattern with pulses
    Custom { pulses: Vec<HapticPulse> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticPulse {
    pub duration_ms: u32,
    pub intensity: f32,
    pub pause_after_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationDirection {
    Left,
    Right,
    Up,
    Down,
    Forward,
}

/// AR overlay specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AROverlay {
    /// Type of overlay
    pub overlay_type: AROverlayType,
    
    /// Content to display
    pub content: String,
    
    /// Position on screen (normalized 0.0-1.0)
    pub position: (f32, f32),
    
    /// Duration in milliseconds (0 = persistent)
    pub duration_ms: u64,
    
    /// Style for rendering
    pub style: WhisperStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AROverlayType {
    /// Simple text whisper
    Whisper,
    
    /// Status indicator
    Status,
    
    /// Navigation arrow with direction
    Navigation,
    
    /// Navigation arrow (legacy alias)
    NavigationArrow,
    
    /// Object highlight box
    Highlight { bounds: (f32, f32, f32, f32) },
    
    /// Progress indicator (bar or spinner)
    Progress { percent: f32 },
    
    /// Confirmation checkmark
    Confirmation,
    
    /// Warning/error indicator
    Warning,
    
    /// Timer countdown
    Timer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WhisperStyle {
    /// Low opacity, small font - minimal distraction
    Subtle,
    
    /// Standard visibility
    Normal,
    
    /// High contrast, larger - important info
    Emphasized,
    
    /// Red tint, pulsing - urgent alert
    Alert,
}

// ============================================================================
// COMMAND RESULTS
// ============================================================================

/// Result of executing an OracleCommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    /// Command succeeded
    Success {
        command_id: String,
        data: CommandData,
    },
    
    /// Command failed
    Failure {
        command_id: String,
        error: String,
        recoverable: bool,
    },
    
    /// Command is pending (async operation)
    Pending {
        command_id: String,
        estimated_ms: u64,
    },
}

/// Data returned from successful commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandData {
    // ═══ Storage Results ═══
    StoredHash(Vec<u8>),
    RetrievedData(Vec<u8>),
    SearchResults(Vec<SearchHit>),
    FileList(Vec<FileEntry>),
    
    // ═══ Chain Results ═══
    TxHash(String),
    Balance(u128),
    BlockData(BlockSummary),
    TransactionList(Vec<TransactionSummary>),
    ProposalInfo(ProposalSummary),
    ProposalList(Vec<ProposalSummary>),
    StakingData(StakingSummary),
    
    // ═══ Swarm Results ═══
    MessageId(String),
    PeerConnected(String),
    PeerList(Vec<PeerInfo>),
    ClipboardSynced,
    
    // ═══ Runtime Results ═══
    WasmOutput(Vec<u8>),
    TaskScheduled(String),
    TaskCancelled,
    
    // ═══ Hardware Results ═══
    HapticPlayed,
    OverlayUpdated,
    HardwareStatus(HardwareStatusInfo),
    
    // ═══ System Results ═══
    PipelineStatus(PipelineStatus),
    BatchProofs(Vec<ProofSummary>),
    Metrics(SystemMetrics),
    ShutdownAck,
    
    // ═══ Spatial AR Results ═══
    /// Anchor was created
    AnchorCreated(SpatialAnchorInfo),
    /// List of nearby anchors
    NearbyAnchors(Vec<SpatialAnchorInfo>),
    /// Navigation path to anchor
    NavigationPath { distance_m: f32, direction: String },
    /// Anchor was removed
    AnchorRemoved(u64),
    /// Room was saved
    RoomSaved { room_id: String, anchor_count: usize },
    /// List of all anchors
    AnchorList(Vec<SpatialAnchorInfo>),
    /// Spatial tab opened
    SpatialTabOpened { anchor_id: u64, url: String },
    
    // ═══ Generic ═══
    Empty,
    Text(String),
}

// ============================================================================
// RESULT DATA TYPES
// ============================================================================

/// Spatial anchor information for command results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAnchorInfo {
    /// Unique anchor ID
    pub id: u64,
    /// Content type (text, browser, video, etc.)
    pub content_type: String,
    /// Human-readable label
    pub label: Option<String>,
    /// Distance from user (if known)
    pub distance_m: Option<f32>,
    /// Direction description (e.g., "to your left")
    pub direction: Option<String>,
    /// Position (x, y, z) in local coords
    pub position: [f32; 3],
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// When created
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub key: Vec<u8>,
    pub score: f32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub hash: Vec<u8>,
    pub name: String,
    pub size_bytes: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub tx_count: usize,
    pub timestamp: u64,
    pub proposer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub tx_type: String,
    pub from: String,
    pub to: Option<String>,
    pub amount: Option<u128>,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingSummary {
    pub staked_amount: u128,
    pub rewards_earned: u128,
    pub delegations: Vec<(String, u128)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub multiaddr: String,
    pub connected_since: u64,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatusInfo {
    pub display_on: bool,
    pub battery_percent: u8,
    pub haptic_available: bool,
    pub camera_active: bool,
    pub mic_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub ai_model: String,
    pub ai_status: String,
    pub zk_queue_size: usize,
    pub zk_proving: bool,
    pub swarm_peers: usize,
    pub chain_height: u64,
    pub mempool_size: usize,
    pub storage_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    pub proof_type: String,
    pub size_bytes: usize,
    pub generation_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub uptime_seconds: u64,
    pub intents_processed: u64,
    pub commands_executed: u64,
}

// ============================================================================
// CHANNEL TYPES
// ============================================================================

/// Channel buffer size for command queue
pub const COMMAND_BUFFER_SIZE: usize = 256;

/// Channels held by the Oracle to communicate with Monad
pub struct OracleChannels {
    /// Send commands to Monad
    pub cmd_tx: mpsc::Sender<OracleCommand>,
    
    /// Receive results from Monad
    pub result_rx: mpsc::Receiver<CommandResult>,
}

/// Channels held by the Monad to receive commands from Oracle
pub struct MonadChannels {
    /// Receive commands from Oracle
    pub cmd_rx: mpsc::Receiver<OracleCommand>,
    
    /// Send results back to Oracle
    pub result_tx: mpsc::Sender<CommandResult>,
}

impl OracleChannels {
    /// Create a new pair of channels for Oracle ↔ Monad communication
    pub fn new(buffer_size: usize) -> (Self, MonadChannels) {
        let (cmd_tx, cmd_rx) = mpsc::channel(buffer_size);
        let (result_tx, result_rx) = mpsc::channel(buffer_size);
        
        (
            OracleChannels { cmd_tx, result_rx },
            MonadChannels { cmd_rx, result_tx },
        )
    }
    
    /// Create with default buffer size
    pub fn default_channels() -> (Self, MonadChannels) {
        Self::new(COMMAND_BUFFER_SIZE)
    }
}

// ============================================================================
// COMMAND HELPERS
// ============================================================================

impl OracleCommand {
    /// Check if this command requires a ZK proof
    pub fn requires_zk_proof(&self) -> bool {
        matches!(
            self,
            OracleCommand::StoreData { .. }
                | OracleCommand::RetrieveData { .. }
                | OracleCommand::SubmitTransaction { .. }
                | OracleCommand::BroadcastMessage { .. }
                | OracleCommand::SyncClipboard { .. }
        )
    }
    
    /// Get the ZK proof from a command (if present)
    pub fn get_zk_proof(&self) -> Option<&[u8]> {
        match self {
            OracleCommand::StoreData { zk_proof, .. } => Some(zk_proof),
            OracleCommand::RetrieveData { zk_proof, .. } => Some(zk_proof),
            OracleCommand::SubmitTransaction { zk_proof, .. } => Some(zk_proof),
            OracleCommand::BroadcastMessage { zk_proof, .. } => Some(zk_proof),
            OracleCommand::SyncClipboard { zk_proof, .. } => Some(zk_proof),
            _ => None,
        }
    }
    
    /// Get a human-readable description of the command
    pub fn description(&self) -> String {
        match self {
            OracleCommand::StoreData { metadata, .. } => format!("Store data: {}", metadata),
            OracleCommand::RetrieveData { requester_did, .. } => {
                format!("Retrieve data for {}", requester_did)
            }
            OracleCommand::SearchSemantic { query, limit } => {
                format!("Search '{}' (limit {})", query, limit)
            }
            OracleCommand::ListUserFiles { did, .. } => format!("List files for {}", did),
            OracleCommand::SubmitTransaction { tx_data, .. } => match tx_data {
                TransactionPayload::Transfer { to, amount, .. } => {
                    format!("Transfer {} to {}", amount, to)
                }
                TransactionPayload::Stake { amount } => format!("Stake {}", amount),
                TransactionPayload::Unstake { amount } => format!("Unstake {}", amount),
                TransactionPayload::Vote { proposal_id, approve } => {
                    format!("Vote {} on proposal {}", if *approve { "YES" } else { "NO" }, proposal_id)
                }
                TransactionPayload::StoreAttestation { .. } => "Store attestation".to_string(),
                TransactionPayload::CreateProposal { title, .. } => {
                    format!("Create proposal: {}", title)
                }
            },
            OracleCommand::QueryBalance { did } => format!("Query balance for {}", did),
            OracleCommand::QueryChainState { query_type } => match query_type {
                ChainQuery::LatestBlock => "Query latest block".to_string(),
                ChainQuery::BlockByHeight(h) => format!("Query block {}", h),
                ChainQuery::TransactionByHash(h) => format!("Query tx {}", h),
                ChainQuery::ProposalStatus(id) => format!("Query proposal {}", id),
                ChainQuery::ActiveProposals => "Query active proposals".to_string(),
                ChainQuery::NodeInfo => "Query node info".to_string(),
                ChainQuery::StakingInfo(did) => format!("Query staking for {}", did),
            },
            OracleCommand::GetPipelineStatus => "Get pipeline status".to_string(),
            OracleCommand::Shutdown => "Shutdown".to_string(),
            _ => format!("{:?}", std::mem::discriminant(self)),
        }
    }
}

impl CommandResult {
    /// Create a success result
    pub fn success(command_id: impl Into<String>, data: CommandData) -> Self {
        CommandResult::Success {
            command_id: command_id.into(),
            data,
        }
    }
    
    /// Create a failure result
    pub fn failure(command_id: impl Into<String>, error: impl Into<String>, recoverable: bool) -> Self {
        CommandResult::Failure {
            command_id: command_id.into(),
            error: error.into(),
            recoverable,
        }
    }
    
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, CommandResult::Success { .. })
    }
    
    /// Get the command ID
    pub fn command_id(&self) -> &str {
        match self {
            CommandResult::Success { command_id, .. } => command_id,
            CommandResult::Failure { command_id, .. } => command_id,
            CommandResult::Pending { command_id, .. } => command_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_channel_creation() {
        let (oracle_ch, monad_ch) = OracleChannels::default_channels();
        
        // Channels should be usable
        assert!(!oracle_ch.cmd_tx.is_closed());
        drop(monad_ch);
        // After dropping monad channels, oracle tx should detect closure
    }
    
    #[test]
    fn test_command_requires_proof() {
        let store_cmd = OracleCommand::StoreData {
            data: vec![1, 2, 3],
            metadata: "test".to_string(),
            zk_proof: vec![0; 64],
        };
        assert!(store_cmd.requires_zk_proof());
        
        let status_cmd = OracleCommand::GetPipelineStatus;
        assert!(!status_cmd.requires_zk_proof());
    }
    
    #[test]
    fn test_command_description() {
        let cmd = OracleCommand::SubmitTransaction {
            tx_data: TransactionPayload::Transfer {
                to: "alice".to_string(),
                amount: 100,
                memo: None,
            },
            zk_proof: vec![],
        };
        
        let desc = cmd.description();
        assert!(desc.contains("Transfer"));
        assert!(desc.contains("alice"));
    }
}
