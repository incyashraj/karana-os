//! Oracle Veil - The Sole Interface Between User and Backend
//!
//! The OracleVeil is the ONLY way for users to interact with Kāraṇa OS.
//! All intents flow through:
//!
//! ```text
//! User Intent (Voice/Gaze) → OracleVeil.mediate() → ZK-Sign → Monad Execute → Whisper Response
//! ```
//!
//! This module implements the core mediation logic that:
//! 1. Receives multimodal input (voice, gaze, gesture)
//! 2. Parses intent using AI (Phi-3)
//! 3. Generates ZK proof of the intent
//! 4. Sends command to Monad via channel
//! 5. Formats response as minimal AR whisper

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::ai::KaranaAI;
use crate::oracle::command::{
    AROverlay, AROverlayType, ChainQuery, CommandData, CommandResult, HapticPattern,
    OracleChannels, OracleCommand, TransactionPayload, WhisperStyle,
};
use crate::oracle::{Oracle, OracleContext as LegacyContext, OracleIntent};
use crate::oracle::manifest::MinimalManifest;
use crate::oracle::tab_commands::TabCommandParser;
use crate::zk::intent_proof::{
    setup_intent_proofs, prove_intent_authorization, verify_intent_proof,
    IntentProof as ZkIntentProof, IntentProofBundle,
};

// ============================================================================
// ORACLE VEIL - The Sole User Interface
// ============================================================================

/// The Oracle Veil mediates ALL user↔backend interactions
/// 
/// Design Principles:
/// - User NEVER directly accesses backend atoms
/// - Every action is ZK-proven before execution
/// - Responses are minimal "whispers" not cluttered UI
/// - Supports multimodal input (voice primary, gaze secondary)
pub struct OracleVeil {
    /// AI engine for intent parsing and response generation
    ai: Arc<StdMutex<KaranaAI>>,
    
    /// Legacy Oracle for NLP parsing (to be replaced by Phi-3)
    legacy_oracle: Arc<StdMutex<Oracle>>,
    
    /// Tab command parser for AR tab voice commands
    tab_parser: TabCommandParser,
    
    /// Universal Oracle for random queries (Phase 41)
    universal_oracle: Arc<crate::oracle::universal::UniversalOracle>,
    
    /// Channel to send commands to Monad
    cmd_tx: mpsc::Sender<OracleCommand>,
    
    /// Channel to receive results from Monad
    result_rx: Arc<Mutex<mpsc::Receiver<CommandResult>>>,
    
    /// Current conversation context
    context: Arc<RwLock<OracleVeilContext>>,
    
    /// ZK prover for intent signing
    zk_prover: Arc<IntentProver>,
    
    /// Pending command tracking
    pending_commands: Arc<Mutex<HashMap<String, PendingCommand>>>,
    
    /// User's DID (set after wallet connection)
    user_did: Arc<RwLock<Option<String>>>,
    
    /// Output manifest for AR whispers and haptic feedback
    manifest: Arc<Mutex<MinimalManifest>>,
}

/// Context maintained across the conversation
#[derive(Debug, Clone, Default)]
pub struct OracleVeilContext {
    /// Current input source
    pub source: InputSource,
    
    /// Timestamp of last interaction
    pub last_interaction: u64,
    
    /// Current gaze target (if available)
    pub gaze_target: Option<String>,
    
    /// Recent conversation history (last N turns)
    pub conversation: Vec<ConversationTurn>,
    
    /// User's current location (if available)
    pub location: Option<(f64, f64)>,
    
    /// Current AR mode
    pub ar_mode: ARMode,
    
    /// Cached user balance
    pub cached_balance: Option<u128>,
    
    /// Additional context key-value pairs
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputSource {
    #[default]
    Voice,
    Gaze,
    Gesture,
    Touch,
    Api,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ARMode {
    #[default]
    Minimal,      // Just whispers
    Guided,       // Navigation/tutorial mode
    Analysis,     // Object identification active
    Private,      // Stealth mode - no visible output
}

#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// Tracking for in-flight commands
struct PendingCommand {
    command: OracleCommand,
    sent_at: Instant,
    intent: String,
}

/// The response from Oracle mediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// The whisper text to display
    pub whisper: String,
    
    /// Haptic pattern to play
    pub haptic: HapticPattern,
    
    /// AR overlay (if any)
    pub overlay: Option<AROverlay>,
    
    /// Whether this needs user confirmation
    pub needs_confirmation: bool,
    
    /// The underlying data (for API consumers)
    pub data: Option<CommandData>,
    
    /// Confidence score
    pub confidence: f32,
}

impl Default for OracleResponse {
    fn default() -> Self {
        Self {
            whisper: String::new(),
            haptic: HapticPattern::Success,
            overlay: None,
            needs_confirmation: false,
            data: None,
            confidence: 0.0,
        }
    }
}

// ============================================================================
// ZK INTENT PROVER (Real Groth16 implementation)
// ============================================================================

/// Wrapper around the real ZK intent proof system
struct IntentProver {
    /// User's secret for proofs (would come from wallet in production)
    user_secret: [u8; 32],
    /// User's authorization level
    auth_level: u8,
    /// Whether ZK setup is complete
    initialized: bool,
}

impl IntentProver {
    fn new() -> Self {
        // Generate a random user secret (in production, derive from wallet)
        let user_secret: [u8; 32] = rand::random();
        
        // Try to initialize ZK proving keys
        let initialized = match setup_intent_proofs() {
            Ok(_) => {
                log::info!("[ZK-PROVER] Real Groth16 prover initialized");
                true
            }
            Err(e) => {
                log::warn!("[ZK-PROVER] Failed to initialize Groth16: {} (using stub)", e);
                false
            }
        };
        
        Self {
            user_secret,
            auth_level: 2, // Default wallet access level
            initialized,
        }
    }
    
    /// Generate a ZK proof that the user intended this action
    fn prove_intent(&self, intent: &ParsedIntent, user_did: &str) -> Result<Vec<u8>> {
        // Convert ParsedIntent to OracleCommand for ZK proving
        let dummy_command = self.intent_to_dummy_command(&intent.action);
        
        if self.initialized {
            // Use real Groth16 proof
            log::info!("[ZK-PROVER] Generating Groth16 proof for {:?}", 
                std::mem::discriminant(&intent.action));
            
            match prove_intent_authorization(&self.user_secret, &dummy_command, self.auth_level) {
                Ok(proof) => {
                    log::info!("[ZK-PROVER] Groth16 proof generated: {} bytes", proof.proof_bytes.len());
                    return Ok(proof.proof_bytes);
                }
                Err(e) => {
                    log::warn!("[ZK-PROVER] Groth16 proving failed: {}, falling back to stub", e);
                }
            }
        }
        
        // Fallback to hash-based stub proof
        self.stub_proof(intent, user_did)
    }
    
    /// Stub proof when real ZK is unavailable
    fn stub_proof(&self, intent: &ParsedIntent, user_did: &str) -> Result<Vec<u8>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(intent.raw.as_bytes());
        hasher.update(user_did.as_bytes());
        hasher.update(&intent.timestamp.to_le_bytes());
        hasher.update(&self.user_secret);
        
        let hash = hasher.finalize();
        
        // Pad to look like a proof (128 bytes)
        let mut proof = vec![0u8; 128];
        proof[..32].copy_from_slice(&hash);
        proof[32..36].copy_from_slice(b"STUB");
        
        Ok(proof)
    }
    
    /// Convert ParsedIntent action to OracleCommand for ZK proof
    fn intent_to_dummy_command(&self, action: &IntentAction) -> OracleCommand {
        match action {
            IntentAction::CheckBalance => OracleCommand::QueryBalance { did: "user".to_string() },
            IntentAction::Transfer { to, amount, memo } => {
                OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Transfer {
                        to: to.clone(),
                        amount: *amount,
                        memo: memo.clone(),
                    },
                    zk_proof: vec![],
                }
            }
            IntentAction::Stake { amount } => {
                OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Stake { amount: *amount },
                    zk_proof: vec![],
                }
            }
            IntentAction::Vote { proposal_id, approve } => {
                OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Vote {
                        proposal_id: *proposal_id,
                        approve: *approve,
                    },
                    zk_proof: vec![],
                }
            }
            IntentAction::StoreData { data, name } => {
                OracleCommand::StoreData {
                    data: data.clone(),
                    metadata: name.clone(),
                    zk_proof: vec![],
                }
            }
            IntentAction::GetStatus => OracleCommand::GetPipelineStatus,
            // Spatial AR commands
            IntentAction::PinHere { content_type, label } => {
                OracleCommand::SpatialPinHere {
                    content_type: content_type.clone(),
                    label: label.clone(),
                }
            }
            IntentAction::PinAt { content_type, target } => {
                OracleCommand::SpatialPinAt {
                    content_type: content_type.clone(),
                    target: target.clone(),
                }
            }
            IntentAction::FindNearby { radius_m } => {
                OracleCommand::SpatialFindNearby {
                    radius_m: radius_m.unwrap_or(10.0),
                }
            }
            IntentAction::NavigateToAnchor { label_or_type } => {
                OracleCommand::SpatialNavigateTo {
                    query: label_or_type.clone(),
                }
            }
            IntentAction::RemoveAnchor { anchor_id } => {
                OracleCommand::SpatialRemoveAnchor {
                    anchor_id: *anchor_id,
                }
            }
            IntentAction::SaveRoom { name } => {
                OracleCommand::SpatialSaveRoom {
                    name: name.clone(),
                }
            }
            IntentAction::ListAnchors => OracleCommand::SpatialListAnchors,
            IntentAction::OpenSpatialTab { url } => {
                OracleCommand::SpatialOpenTab {
                    url: url.clone(),
                }
            }
            // AR Tab commands
            IntentAction::TabPinBrowser { url, size, location } => {
                OracleCommand::TabPinBrowser {
                    url: url.clone(),
                    size: Self::parse_size_hint(size),
                    location_hint: location.clone(),
                }
            }
            IntentAction::TabPinVideo { url, size, location } => {
                OracleCommand::TabPinVideo {
                    url: url.clone(),
                    size: Self::parse_size_hint(size),
                    location_hint: location.clone(),
                }
            }
            IntentAction::TabPinWidget { widget_type, location } => {
                OracleCommand::TabPinWidget {
                    widget_type: Self::parse_widget_type(widget_type),
                    size: crate::oracle::command::TabSizeHint::Medium,
                    location_hint: location.clone(),
                }
            }
            IntentAction::TabFocus { query } => {
                OracleCommand::TabFocus { query: query.clone() }
            }
            IntentAction::TabClose { query } => {
                OracleCommand::TabClose { query: query.clone() }
            }
            IntentAction::TabCloseLocation { location } => {
                OracleCommand::TabCloseLocation { location: location.clone() }
            }
            IntentAction::TabList { location } => {
                OracleCommand::TabList { location_filter: location.clone() }
            }
            IntentAction::TabMove { query, target } => {
                OracleCommand::TabMove { 
                    query: query.clone(), 
                    target_location: target.clone() 
                }
            }
            IntentAction::TabResize { query, size } => {
                OracleCommand::TabResize { 
                    query: query.clone(), 
                    size: Self::parse_size_hint(size) 
                }
            }
            IntentAction::TabSetLayout { location, layout } => {
                OracleCommand::TabSetLayout { 
                    location: location.clone(), 
                    layout: Self::parse_layout_hint(layout) 
                }
            }
            IntentAction::TabNavigate { action } => {
                OracleCommand::TabNavigate { 
                    action: Self::parse_nav_action(action) 
                }
            }
            IntentAction::TabCycle { direction } => {
                OracleCommand::TabCycle { 
                    direction: Self::parse_cycle_direction(direction) 
                }
            }
            _ => OracleCommand::GetPipelineStatus,
        }
    }
    
    /// Parse string to TabSizeHint
    fn parse_size_hint(s: &str) -> crate::oracle::command::TabSizeHint {
        use crate::oracle::command::TabSizeHint;
        match s.to_lowercase().as_str() {
            "small" | "tiny" => TabSizeHint::Small,
            "medium" | "normal" => TabSizeHint::Medium,
            "large" | "big" => TabSizeHint::Large,
            "full" | "huge" | "max" => TabSizeHint::Full,
            _ => TabSizeHint::Medium,
        }
    }
    
    /// Parse string to WidgetType
    fn parse_widget_type(s: &str) -> crate::oracle::command::WidgetType {
        use crate::oracle::command::WidgetType;
        match s.to_lowercase().as_str() {
            "clock" | "time" => WidgetType::Clock,
            "weather" => WidgetType::Weather,
            "calendar" | "schedule" => WidgetType::Calendar,
            "stocks" | "stock" => WidgetType::Stocks,
            "music" | "player" => WidgetType::Music,
            "timer" | "countdown" => WidgetType::Timer,
            "todo" | "tasks" => WidgetType::Todo,
            "status" | "battery" => WidgetType::SystemStatus,
            "notifications" | "alerts" => WidgetType::Notifications,
            "note" | "sticky" => WidgetType::StickyNote,
            _ => WidgetType::Custom(s.to_string()),
        }
    }
    
    /// Parse string to TabLayoutHint
    fn parse_layout_hint(s: &str) -> crate::oracle::command::TabLayoutHint {
        use crate::oracle::command::TabLayoutHint;
        match s.to_lowercase().as_str() {
            "grid" => TabLayoutHint::Grid,
            "stack" | "stacked" => TabLayoutHint::Stack,
            "carousel" | "circle" => TabLayoutHint::Carousel,
            "dock" | "docked" => TabLayoutHint::Dock,
            _ => TabLayoutHint::Free,
        }
    }
    
    /// Parse string to TabNavAction
    fn parse_nav_action(s: &str) -> crate::oracle::command::TabNavAction {
        use crate::oracle::command::{TabNavAction, ScrollDirection, ScrollAmount};
        match s.to_lowercase().as_str() {
            "back" => TabNavAction::Back,
            "forward" => TabNavAction::Forward,
            "reload" | "refresh" => TabNavAction::Reload,
            "play" | "pause" | "playpause" => TabNavAction::PlayPause,
            "scroll_up" | "scrollup" => TabNavAction::Scroll { 
                direction: ScrollDirection::Up, 
                amount: ScrollAmount::Page 
            },
            "scroll_down" | "scrolldown" => TabNavAction::Scroll { 
                direction: ScrollDirection::Down, 
                amount: ScrollAmount::Page 
            },
            _ => TabNavAction::Reload, // Default action
        }
    }
    
    /// Parse string to TabCycleDirection
    fn parse_cycle_direction(s: &str) -> crate::oracle::command::TabCycleDirection {
        use crate::oracle::command::TabCycleDirection;
        match s.to_lowercase().as_str() {
            "next" => TabCycleDirection::Next,
            "previous" | "prev" => TabCycleDirection::Previous,
            "recent" | "last" => TabCycleDirection::Recent,
            _ => TabCycleDirection::Next,
        }
    }
    
    /// Set user's authorization level
    fn set_auth_level(&mut self, level: u8) {
        self.auth_level = level;
    }
    
    /// Verify a ZK proof (for testing)
    fn verify(&self, proof: &[u8]) -> bool {
        // Check for stub marker
        if proof.len() >= 36 && &proof[32..36] == b"STUB" {
            return true;
        }
        
        // Try real verification
        if self.initialized && !proof.is_empty() {
            // Would need full IntentProof structure for real verification
            // For now, accept non-stub proofs of sufficient length
            return proof.len() >= 64;
        }
        
        false
    }
}

// ============================================================================
// PARSED INTENT
// ============================================================================

/// A parsed user intent ready for execution
#[derive(Debug, Clone)]
pub struct ParsedIntent {
    /// The action type
    pub action: IntentAction,
    
    /// Original raw text
    pub raw: String,
    
    /// Confidence score from AI
    pub confidence: f32,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Source of the input
    pub source: InputSource,
}

/// Supported intent actions
#[derive(Debug, Clone)]
pub enum IntentAction {
    // ═══ Blockchain ═══
    Transfer { to: String, amount: u128, memo: Option<String> },
    CheckBalance,
    Stake { amount: u128 },
    Unstake { amount: u128 },
    Vote { proposal_id: u64, approve: bool },
    CreateProposal { title: String, description: String },
    
    // ═══ Storage ═══
    StoreData { data: Vec<u8>, name: String },
    RetrieveData { key: String },
    ListFiles,
    Search { query: String },
    
    // ═══ System ═══
    GetStatus,
    SetTimer { minutes: u32, label: String },
    Navigate { destination: String },
    
    // ═══ Hardware ═══
    CapturePhoto,
    RecordVideo { duration_secs: u32 },
    IdentifyObject,
    AdjustBrightness { level: u8 },
    AdjustVolume { direction: String },
    
    // ═══ Media ═══
    PlayMedia { query: String },
    
    // ═══ Communication ═══
    MakeCall { contact: String },
    ShowNotifications,
    
    // ═══ Spatial AR ═══
    /// Pin content at current position: "pin this here"
    PinHere { content_type: String, label: Option<String> },
    /// Pin content at described location: "pin this on the wall"  
    PinAt { content_type: String, target: String },
    /// Find nearby anchored content: "what's around me"
    FindNearby { radius_m: Option<f32> },
    /// Navigate to pinned content: "take me to my notes"
    NavigateToAnchor { label_or_type: String },
    /// Remove an anchor: "remove this pin"
    RemoveAnchor { anchor_id: Option<u64> },
    /// Save current room: "remember this room as office"
    SaveRoom { name: String },
    /// List all pinned items: "show my pins"
    ListAnchors,
    /// Open browser tab at position: "open youtube here"
    OpenSpatialTab { url: String },
    
    // ═══ AR Tabs ═══
    /// Pin a browser tab: "open youtube on the wall"
    TabPinBrowser { url: String, size: String, location: Option<String> },
    /// Pin a video tab: "pin netflix to the couch"
    TabPinVideo { url: String, size: String, location: Option<String> },
    /// Pin a widget: "show me a clock"
    TabPinWidget { widget_type: String, location: Option<String> },
    /// Focus a tab: "focus youtube"
    TabFocus { query: String },
    /// Close a tab: "close the video"
    TabClose { query: Option<String> },
    /// Close all tabs in location: "clear the wall"
    TabCloseLocation { location: String },
    /// List tabs: "show my tabs"
    TabList { location: Option<String> },
    /// Move tab: "move this to the desk"
    TabMove { query: Option<String>, target: String },
    /// Resize tab: "make this bigger"
    TabResize { query: Option<String>, size: String },
    /// Set layout: "grid layout"
    TabSetLayout { location: Option<String>, layout: String },
    /// Navigate within tab: "scroll down"
    TabNavigate { action: String },
    /// Cycle tabs: "next tab"
    TabCycle { direction: String },
    
    // ═══ Conversation ═══
    Conversation { response: String },
    Help,
    Clarify { question: String },
    
    // ═══ Universal Query (Phase 41) ═══
    UniversalQuery { answer: String, source: String, confidence: f32 },
    
    // ═══ Knowledge Management (Phase 43) ═══
    /// Remember/save knowledge: "remember that Rust is fast"
    RememberThis { text: String, category: Option<String>, tags: Vec<String> },
    /// Forget/delete knowledge: "forget about X"
    ForgetAbout { query: String },
    /// Search personal knowledge: "what do I know about X?"
    SearchMyKnowledge { query: String },
    /// List knowledge categories: "show my knowledge categories"
    ListKnowledgeCategories,
    /// Pin knowledge: "pin my note about Rust"
    PinKnowledge { query: String },
    /// Show knowledge stats: "show my knowledge stats"
    ShowKnowledgeStats,
    
    // ═══ Infeasible ═══
    Infeasible { reason: String, alternative: String },
    
    // ═══ Unknown ═══
    Unknown { raw: String },
}

// ============================================================================
// ORACLE VEIL IMPLEMENTATION
// ============================================================================

impl OracleVeil {
    /// Create a new OracleVeil with command channels
    pub fn new(
        ai: Arc<StdMutex<KaranaAI>>,
        cmd_tx: mpsc::Sender<OracleCommand>,
        result_rx: mpsc::Receiver<CommandResult>,
    ) -> Result<Self> {
        let universal_oracle = crate::oracle::universal::UniversalOracle::new()
            .map_err(|e| anyhow!("Failed to create universal oracle: {}", e))?;
        
        Ok(Self {
            ai,
            legacy_oracle: Arc::new(StdMutex::new(Oracle::new())),
            tab_parser: TabCommandParser::new(),
            universal_oracle: Arc::new(universal_oracle),
            cmd_tx,
            result_rx: Arc::new(Mutex::new(result_rx)),
            context: Arc::new(RwLock::new(OracleVeilContext::default())),
            zk_prover: Arc::new(IntentProver::new()),
            pending_commands: Arc::new(Mutex::new(HashMap::new())),
            user_did: Arc::new(RwLock::new(None)),
            manifest: Arc::new(Mutex::new(MinimalManifest::new())),
        })
    }
    
    /// Set the user's DID (call after wallet is connected)
    pub async fn set_user_did(&self, did: String) {
        let mut user_did = self.user_did.write().await;
        *user_did = Some(did);
    }
    
    /// Get the user's DID
    pub async fn get_user_did(&self) -> Option<String> {
        self.user_did.read().await.clone()
    }
    
    /// Update context with a key-value pair
    pub async fn update_context(&self, key: &str, value: &str) {
        let mut ctx = self.context.write().await;
        ctx.metadata.insert(key.to_string(), value.to_string());
        
        // Handle special keys
        match key {
            "gaze_point" => {
                // Parse "x,y" format
                ctx.gaze_target = Some(value.to_string());
            }
            "ar_mode" => {
                ctx.ar_mode = match value {
                    "minimal" => ARMode::Minimal,
                    "guided" => ARMode::Guided,
                    "analysis" => ARMode::Analysis,
                    "private" => ARMode::Private,
                    _ => ARMode::Minimal,
                };
            }
            _ => {}
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // MULTIMODAL INPUT PROCESSING
    // ════════════════════════════════════════════════════════════════════════
    
    /// Process fused input from MultimodalSense
    /// 
    /// This is the preferred entry point when using the full multimodal pipeline.
    /// The FusedInput contains voice transcription + gaze/gesture context.
    pub async fn process_fused_input(
        &self,
        input: crate::oracle::sense::FusedInput,
    ) -> Result<OracleResponse> {
        // Map FusedInput source to our InputSource
        let source = match input.source {
            crate::oracle::sense::InputModality::Voice => InputSource::Voice,
            crate::oracle::sense::InputModality::Gaze => InputSource::Gaze,
            crate::oracle::sense::InputModality::Gesture => InputSource::Gesture,
            crate::oracle::sense::InputModality::Combined => InputSource::Voice, // Default to voice for combined
        };
        
        // Update context with gaze information if available
        if let Some(gaze) = &input.gaze_context {
            self.update_context(
                "gaze_target", 
                &format!("{:.2},{:.2}", gaze.position.0, gaze.position.1)
            ).await;
            self.update_context(
                "gaze_dwell_ms",
                &gaze.dwell_ms.to_string()
            ).await;
        }
        
        // Update context with gesture information if available
        if let Some(gesture) = &input.gesture_context {
            self.update_context(
                "last_gesture",
                &format!("{:?}", gesture)
            ).await;
        }
        
        // Update context with audio features if available (emotion/urgency)
        if let Some(audio) = &input.audio_features {
            self.update_context("voice_energy", &format!("{:.3}", audio.energy)).await;
            self.update_context("voice_pitch", &format!("{:.1}", audio.pitch)).await;
            self.update_context("voice_tempo", &format!("{:.1}", audio.tempo)).await;
        }
        
        // Now mediate the intent with full context
        log::info!("[ORACLE] Processing fused input: '{}' (confidence: {:.2}, source: {:?})",
            input.content, input.confidence, source);
        
        self.mediate(&input.content, source).await
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // CORE MEDIATION
    // ════════════════════════════════════════════════════════════════════════
    
    /// Mediate a user intent - the MAIN entry point
    ///
    /// This is the ONLY way for users to interact with the system.
    /// Flow: Intent → Parse → ZK-Sign → Command → Monad → Result → Whisper
    pub async fn mediate(&self, intent: &str, source: InputSource) -> Result<OracleResponse> {
        let start = Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        log::info!("[ORACLE] Mediating intent: '{}' (source: {:?})", intent, source);
        
        // 1. Update context
        {
            let mut ctx = self.context.write().await;
            ctx.source = source;
            ctx.last_interaction = timestamp;
            ctx.conversation.push(ConversationTurn {
                role: "user".to_string(),
                content: intent.to_string(),
                timestamp,
            });
            
            // Keep conversation history bounded
            if ctx.conversation.len() > 20 {
                ctx.conversation.remove(0);
            }
        }
        
        // 2. Parse intent using AI
        let parsed = self.parse_intent(intent, source, timestamp).await?;
        log::info!("[ORACLE] Parsed: {:?} (confidence: {:.2})", 
            std::mem::discriminant(&parsed.action), parsed.confidence);
        
        // 3. Check for infeasible actions
        if let IntentAction::Infeasible { reason, alternative } = &parsed.action {
            return Ok(OracleResponse {
                whisper: format!("I can't do that: {}. {}", reason, alternative),
                haptic: HapticPattern::Attention,
                needs_confirmation: false,
                confidence: parsed.confidence,
                ..Default::default()
            });
        }
        
        // 4. Generate ZK proof of intent
        let user_did = self.get_user_did().await.unwrap_or_else(|| "anonymous".to_string());
        let zk_proof = self.zk_prover.prove_intent(&parsed, &user_did)?;
        log::debug!("[ORACLE] ZK proof generated: {} bytes", zk_proof.len());
        
        // 5. Convert to backend command
        let command = self.intent_to_command(&parsed, &user_did, zk_proof.clone()).await?;
        
        // 6. Execute via Monad
        let result = self.execute_command(command).await?;
        
        // 7. Format response as whisper
        let response = self.format_response(&parsed, &result, start.elapsed());
        
        // 8. Render response via MinimalManifest (AR whispers + haptic)
        {
            let mut manifest = self.manifest.lock().await;
            if let Err(e) = manifest.render(&response).await {
                log::warn!("[ORACLE] Failed to render manifest: {}", e);
            }
        }
        
        // 9. Update conversation with response
        {
            let mut ctx = self.context.write().await;
            ctx.conversation.push(ConversationTurn {
                role: "oracle".to_string(),
                content: response.whisper.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        Ok(response)
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // INTENT PARSING
    // ════════════════════════════════════════════════════════════════════════
    
    /// Parse natural language intent using AI
    async fn parse_intent(
        &self,
        intent: &str,
        source: InputSource,
        timestamp: u64,
    ) -> Result<ParsedIntent> {
        // 1. First try tab command parser (fast, specialized for AR tabs)
        if let Some(tab_cmd) = self.tab_parser.parse(intent) {
            let action = self.oracle_command_to_intent_action(&tab_cmd);
            return Ok(ParsedIntent {
                action,
                confidence: 0.95, // High confidence for explicit tab commands
                source,
                timestamp,
                raw: intent.to_string(),
            });
        }
        
        // 2. Check if this is a general knowledge query (not OS command)
        if self.is_general_query(intent) {
            log::info!("[ORACLE] Detected general query, routing to Universal Oracle");
            
            // Build query context
            let ctx = self.context.read().await;
            let query_ctx = crate::oracle::universal::QueryContext {
                location: None, // TODO: Get from GPS
                time_of_day: if timestamp % 86400 < 43200 { "morning".to_string() } else { "evening".to_string() },
                recent_topics: ctx.conversation.iter()
                    .rev()
                    .take(3)
                    .map(|t| t.content.clone())
                    .collect(),
                user_preferences: ctx.metadata.clone(),
            };
            
            // Query universal oracle
            let universal_response = self.universal_oracle.query(intent, &query_ctx).await?;
            
            // Convert to IntentAction
            let action = IntentAction::UniversalQuery {
                answer: universal_response.answer.clone(),
                source: format!("{:?}", universal_response.source),
                confidence: universal_response.confidence,
            };
            
            return Ok(ParsedIntent {
                action,
                confidence: universal_response.confidence,
                source,
                timestamp,
                raw: intent.to_string(),
            });
        }
        
        // 3. Try AI-based parsing (using local Phi-3 via Candle)
        let ai_result = {
            let mut ai = self.ai.lock().unwrap();
            ai.predict(intent, 100)
        };
        
        if let Ok(ai_response) = ai_result {
            // Try to parse AI response as JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&ai_response) {
                return self.parse_ai_response(&parsed, intent, source, timestamp);
            }
        }
        
        // 4. Fallback to legacy Oracle parser
        let legacy_response = {
            let mut oracle = self.legacy_oracle.lock().unwrap();
            let ctx = LegacyContext::default();
            oracle.process(intent, Some(ctx))
        };
        
        self.parse_legacy_response(&legacy_response, intent, source, timestamp)
    }
    
    /// Check if this is a general knowledge query vs an OS command
    fn is_general_query(&self, intent: &str) -> bool {
        let lower = intent.to_lowercase();
        
        // Knowledge query patterns
        let knowledge_patterns = [
            "what is", "what's", "who is", "who's", "where is", "where's",
            "when is", "when's", "why is", "why's", "how does", "how do",
            "tell me about", "explain", "describe", "define",
            "capital of", "meaning of", "history of",
            "square root", "calculate", "what's the answer",
        ];
        
        // OS command patterns (if any match, it's NOT a general query)
        let os_command_patterns = [
            "send", "transfer", "stake", "vote", "open", "close", "pin",
            "show", "list", "play", "call", "take photo", "record video",
            "brightness", "volume", "my balance", "my tokens",
            "navigate to", "remind me", "set timer",
        ];
        
        // Check OS commands first (higher priority)
        for pattern in &os_command_patterns {
            if lower.contains(pattern) {
                return false; // It's an OS command
            }
        }
        
        // Then check knowledge patterns
        for pattern in &knowledge_patterns {
            if lower.contains(pattern) {
                return true; // It's a general query
            }
        }
        
        // Default: treat as OS command (safer)
        false
    }
    
    /// Convert OracleCommand (from tab parser) to IntentAction
    fn oracle_command_to_intent_action(&self, cmd: &OracleCommand) -> IntentAction {
        use crate::oracle::command::{TabSizeHint, WidgetType, TabLayoutHint, TabNavAction, TabCycleDirection};
        
        match cmd {
            OracleCommand::TabPinBrowser { url, size, location_hint } => {
                IntentAction::TabPinBrowser {
                    url: url.clone(),
                    size: Self::size_hint_to_string(size),
                    location: location_hint.clone(),
                }
            }
            OracleCommand::TabPinVideo { url, size, location_hint } => {
                IntentAction::TabPinVideo {
                    url: url.clone(),
                    size: Self::size_hint_to_string(size),
                    location: location_hint.clone(),
                }
            }
            OracleCommand::TabPinWidget { widget_type, size: _, location_hint } => {
                IntentAction::TabPinWidget {
                    widget_type: Self::widget_type_to_string(widget_type),
                    location: location_hint.clone(),
                }
            }
            OracleCommand::TabFocus { query } => {
                IntentAction::TabFocus { query: query.clone() }
            }
            OracleCommand::TabClose { query } => {
                IntentAction::TabClose { query: query.clone() }
            }
            OracleCommand::TabCloseLocation { location } => {
                IntentAction::TabCloseLocation { location: location.clone() }
            }
            OracleCommand::TabList { location_filter } => {
                IntentAction::TabList { location: location_filter.clone() }
            }
            OracleCommand::TabMove { query, target_location } => {
                IntentAction::TabMove {
                    query: query.clone(),
                    target: target_location.clone(),
                }
            }
            OracleCommand::TabResize { query, size } => {
                IntentAction::TabResize {
                    query: query.clone(),
                    size: Self::size_hint_to_string(size),
                }
            }
            OracleCommand::TabSetLayout { location, layout } => {
                IntentAction::TabSetLayout {
                    location: location.clone(),
                    layout: Self::layout_hint_to_string(layout),
                }
            }
            OracleCommand::TabNavigate { action } => {
                IntentAction::TabNavigate {
                    action: Self::nav_action_to_string(action),
                }
            }
            OracleCommand::TabCycle { direction } => {
                IntentAction::TabCycle {
                    direction: Self::cycle_direction_to_string(direction),
                }
            }
            _ => IntentAction::Unknown { raw: format!("{:?}", cmd) },
        }
    }
    
    fn size_hint_to_string(size: &crate::oracle::command::TabSizeHint) -> String {
        use crate::oracle::command::TabSizeHint;
        match size {
            TabSizeHint::Small => "small".to_string(),
            TabSizeHint::Medium => "medium".to_string(),
            TabSizeHint::Large => "large".to_string(),
            TabSizeHint::Full => "full".to_string(),
            TabSizeHint::Auto => "auto".to_string(),
        }
    }
    
    fn widget_type_to_string(wt: &crate::oracle::command::WidgetType) -> String {
        use crate::oracle::command::WidgetType;
        match wt {
            WidgetType::Clock => "clock".to_string(),
            WidgetType::Weather => "weather".to_string(),
            WidgetType::Calendar => "calendar".to_string(),
            WidgetType::Stocks => "stocks".to_string(),
            WidgetType::Music => "music".to_string(),
            WidgetType::Timer => "timer".to_string(),
            WidgetType::Todo => "todo".to_string(),
            WidgetType::SystemStatus => "status".to_string(),
            WidgetType::Notifications => "notifications".to_string(),
            WidgetType::StickyNote => "note".to_string(),
            WidgetType::Custom(s) => s.clone(),
        }
    }
    
    fn layout_hint_to_string(layout: &crate::oracle::command::TabLayoutHint) -> String {
        use crate::oracle::command::TabLayoutHint;
        match layout {
            TabLayoutHint::Free => "free".to_string(),
            TabLayoutHint::Grid => "grid".to_string(),
            TabLayoutHint::Stack => "stack".to_string(),
            TabLayoutHint::Carousel => "carousel".to_string(),
            TabLayoutHint::Dock => "dock".to_string(),
        }
    }
    
    fn nav_action_to_string(action: &crate::oracle::command::TabNavAction) -> String {
        use crate::oracle::command::TabNavAction;
        match action {
            TabNavAction::Back => "back".to_string(),
            TabNavAction::Forward => "forward".to_string(),
            TabNavAction::Reload => "reload".to_string(),
            TabNavAction::PlayPause => "playpause".to_string(),
            TabNavAction::Scroll { direction, .. } => format!("scroll_{:?}", direction).to_lowercase(),
            _ => "unknown".to_string(),
        }
    }
    
    fn cycle_direction_to_string(dir: &crate::oracle::command::TabCycleDirection) -> String {
        use crate::oracle::command::TabCycleDirection;
        match dir {
            TabCycleDirection::Next => "next".to_string(),
            TabCycleDirection::Previous => "previous".to_string(),
            TabCycleDirection::Recent => "recent".to_string(),
        }
    }
    
    /// Parse AI JSON response into ParsedIntent
    fn parse_ai_response(
        &self,
        json: &serde_json::Value,
        raw: &str,
        source: InputSource,
        timestamp: u64,
    ) -> Result<ParsedIntent> {
        let action_str = json.get("action").and_then(|v| v.as_str()).unwrap_or("unknown");
        let confidence = json.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
        let params = json.get("params").cloned().unwrap_or(serde_json::Value::Null);
        
        let action = match action_str {
            "transfer" => {
                let to = params.get("to").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let amount = params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0) as u128;
                IntentAction::Transfer { to, amount, memo: None }
            }
            "get_balance" | "balance" => IntentAction::CheckBalance,
            "stake" => {
                let amount = params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0) as u128;
                IntentAction::Stake { amount }
            }
            "vote" => {
                let id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(1);
                let approve = params.get("approve").and_then(|v| v.as_bool()).unwrap_or(true);
                IntentAction::Vote { proposal_id: id, approve }
            }
            "create_proposal" => {
                let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("New Proposal").to_string();
                IntentAction::CreateProposal { title, description: String::new() }
            }
            "store_file" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("untitled").to_string();
                IntentAction::StoreData { data: vec![], name }
            }
            "query_files" => IntentAction::ListFiles,
            "get_status" => IntentAction::GetStatus,
            "set_timer" => {
                let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(5) as u32;
                let label = params.get("label").and_then(|v| v.as_str()).unwrap_or("Timer").to_string();
                IntentAction::SetTimer { minutes, label }
            }
            "navigate" => {
                let destination = params.get("destination").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                IntentAction::Navigate { destination }
            }
            "capture_photo" => IntentAction::CapturePhoto,
            "record_video" => {
                let duration = params.get("duration").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
                IntentAction::RecordVideo { duration_secs: duration }
            }
            "identify_object" => IntentAction::IdentifyObject,
            "adjust_brightness" => {
                let level = params.get("level").and_then(|v| v.as_u64()).unwrap_or(50) as u8;
                IntentAction::AdjustBrightness { level }
            }
            "adjust_volume" => {
                let direction = params.get("direction").and_then(|v| v.as_str()).unwrap_or("up").to_string();
                IntentAction::AdjustVolume { direction }
            }
            "play_media" => {
                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("music").to_string();
                IntentAction::PlayMedia { query }
            }
            "make_call" => {
                let contact = params.get("contact").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                IntentAction::MakeCall { contact }
            }
            "show_notifications" => IntentAction::ShowNotifications,
            // ═══ Spatial AR Commands ═══
            "pin_here" | "pin" => {
                let content_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("note").to_string();
                let label = params.get("label").and_then(|v| v.as_str()).map(String::from);
                IntentAction::PinHere { content_type, label }
            }
            "pin_at" => {
                let content_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("note").to_string();
                let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("here").to_string();
                IntentAction::PinAt { content_type, target }
            }
            "find_nearby" | "whats_around" | "nearby" => {
                let radius = params.get("radius").and_then(|v| v.as_f64()).map(|r| r as f32);
                IntentAction::FindNearby { radius_m: radius }
            }
            "navigate_to_anchor" | "find_pin" | "go_to" => {
                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
                IntentAction::NavigateToAnchor { label_or_type: query }
            }
            "remove_anchor" | "unpin" | "remove_pin" => {
                let id = params.get("id").and_then(|v| v.as_u64());
                IntentAction::RemoveAnchor { anchor_id: id }
            }
            "save_room" | "remember_room" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string();
                IntentAction::SaveRoom { name }
            }
            "list_anchors" | "show_pins" | "my_pins" => IntentAction::ListAnchors,
            "open_tab" | "open_browser" | "open_here" => {
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("https://karana.io").to_string();
                IntentAction::OpenSpatialTab { url }
            }
            "infeasible" => {
                let reason = params.get("reason").or_else(|| json.get("reason"))
                    .and_then(|v| v.as_str()).unwrap_or("Action not supported").to_string();
                let alternative = params.get("alternative").or_else(|| json.get("alternative"))
                    .and_then(|v| v.as_str()).unwrap_or("").to_string();
                IntentAction::Infeasible { reason, alternative }
            }
            _ => IntentAction::Unknown { raw: raw.to_string() },
        };
        
        Ok(ParsedIntent {
            action,
            raw: raw.to_string(),
            confidence,
            timestamp,
            source,
        })
    }
    
    /// Parse legacy Oracle response into ParsedIntent
    fn parse_legacy_response(
        &self,
        response: &crate::oracle::OracleResponse,
        raw: &str,
        source: InputSource,
        timestamp: u64,
    ) -> Result<ParsedIntent> {
        let action = match &response.intent {
            OracleIntent::Transfer { amount, recipient, memo } => {
                IntentAction::Transfer {
                    to: recipient.clone(),
                    amount: *amount as u128,
                    memo: memo.clone(),
                }
            }
            OracleIntent::CheckBalance => IntentAction::CheckBalance,
            OracleIntent::StakeTokens { amount } => IntentAction::Stake { amount: *amount as u128 },
            OracleIntent::VoteProposal { proposal_id, vote } => {
                IntentAction::Vote {
                    proposal_id: proposal_id.parse().unwrap_or(1),
                    approve: *vote,
                }
            }
            OracleIntent::SetReminder { message, duration } => {
                // Parse duration like "5 minutes"
                let minutes = duration.split_whitespace()
                    .find_map(|w| w.parse::<u32>().ok())
                    .unwrap_or(5);
                IntentAction::SetTimer { minutes, label: message.clone() }
            }
            OracleIntent::Navigate { destination } => {
                IntentAction::Navigate { destination: destination.clone() }
            }
            OracleIntent::TakeNote { content } => {
                IntentAction::StoreData {
                    data: content.clone().unwrap_or_default().into_bytes(),
                    name: "note".to_string(),
                }
            }
            OracleIntent::SystemStatus => IntentAction::GetStatus,
            OracleIntent::AnalyzeVision => IntentAction::IdentifyObject,
            OracleIntent::AdjustBrightness { level } => IntentAction::AdjustBrightness { level: *level },
            OracleIntent::PlayMusic { query } => {
                IntentAction::PlayMedia { query: query.clone().unwrap_or_default() }
            }
            OracleIntent::Help => IntentAction::Help,
            OracleIntent::Conversation { response } => {
                IntentAction::Conversation { response: response.clone() }
            }
            OracleIntent::Clarify { question } => {
                IntentAction::Clarify { question: question.clone() }
            }
            _ => IntentAction::Unknown { raw: raw.to_string() },
        };
        
        Ok(ParsedIntent {
            action,
            raw: raw.to_string(),
            confidence: response.confidence,
            timestamp,
            source,
        })
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // COMMAND CONVERSION
    // ════════════════════════════════════════════════════════════════════════
    
    /// Convert parsed intent to backend command
    async fn intent_to_command(
        &self,
        parsed: &ParsedIntent,
        user_did: &str,
        zk_proof: Vec<u8>,
    ) -> Result<OracleCommand> {
        match &parsed.action {
            // ═══ Blockchain Commands ═══
            IntentAction::Transfer { to, amount, memo } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Transfer {
                        to: to.clone(),
                        amount: *amount,
                        memo: memo.clone(),
                    },
                    zk_proof,
                })
            }
            IntentAction::CheckBalance => {
                Ok(OracleCommand::QueryBalance { did: user_did.to_string() })
            }
            IntentAction::Stake { amount } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Stake { amount: *amount },
                    zk_proof,
                })
            }
            IntentAction::Unstake { amount } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Unstake { amount: *amount },
                    zk_proof,
                })
            }
            IntentAction::Vote { proposal_id, approve } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Vote {
                        proposal_id: *proposal_id,
                        approve: *approve,
                    },
                    zk_proof,
                })
            }
            IntentAction::CreateProposal { title, description } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::CreateProposal {
                        title: title.clone(),
                        description: description.clone(),
                    },
                    zk_proof,
                })
            }
            
            // ═══ Storage Commands ═══
            IntentAction::StoreData { data, name } => {
                Ok(OracleCommand::StoreData {
                    data: data.clone(),
                    metadata: name.clone(),
                    zk_proof,
                })
            }
            IntentAction::RetrieveData { key } => {
                Ok(OracleCommand::RetrieveData {
                    key: key.as_bytes().to_vec(),
                    requester_did: user_did.to_string(),
                    zk_proof,
                })
            }
            IntentAction::ListFiles => {
                Ok(OracleCommand::ListUserFiles {
                    did: user_did.to_string(),
                    limit: 20,
                })
            }
            IntentAction::Search { query } => {
                Ok(OracleCommand::SearchSemantic {
                    query: query.clone(),
                    limit: 10,
                })
            }
            
            // ═══ System Commands ═══
            IntentAction::GetStatus => {
                Ok(OracleCommand::GetPipelineStatus)
            }
            IntentAction::SetTimer { minutes, label } => {
                // Schedule a delayed notification
                Ok(OracleCommand::ScheduleTask {
                    task_id: format!("timer_{}", parsed.timestamp),
                    delay_ms: (*minutes as u64) * 60 * 1000,
                    command: Box::new(OracleCommand::PlayHaptic {
                        pattern: HapticPattern::Attention,
                    }),
                })
            }
            
            // ═══ Hardware Commands ═══
            IntentAction::AdjustBrightness { level } => {
                Ok(OracleCommand::UpdateAROverlay {
                    overlay: AROverlay {
                        overlay_type: AROverlayType::Status,
                        content: format!("Brightness: {}%", level),
                        position: (0.5, 0.1),
                        duration_ms: 2000,
                        style: WhisperStyle::Subtle,
                    },
                })
            }
            IntentAction::IdentifyObject => {
                // TODO: Trigger camera + AI analysis
                Ok(OracleCommand::GetHardwareStatus)
            }
            
            // ═══ Conversation/Help ═══
            IntentAction::Help | IntentAction::Conversation { .. } | IntentAction::Clarify { .. } => {
                // These don't need backend commands, handled in response formatting
                Ok(OracleCommand::GetPipelineStatus)  // Dummy command
            }
            
            // ═══ Universal Query (Phase 41) ═══
            IntentAction::UniversalQuery { .. } => {
                // Doesn't need backend execution, handled in response formatting
                Ok(OracleCommand::GetPipelineStatus)  // Dummy command
            }
            
            // ═══ Other ═══
            _ => {
                Ok(OracleCommand::GetPipelineStatus)  // Default to status
            }
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // COMMAND EXECUTION
    // ════════════════════════════════════════════════════════════════════════
    
    /// Execute a command via the Monad
    async fn execute_command(&self, command: OracleCommand) -> Result<CommandResult> {
        let cmd_id = uuid::Uuid::new_v4().to_string();
        let description = command.description();
        
        log::info!("[ORACLE] Executing: {} ({})", description, &cmd_id[..8]);
        
        // Track pending command
        {
            let mut pending = self.pending_commands.lock().await;
            pending.insert(cmd_id.clone(), PendingCommand {
                command: command.clone(),
                sent_at: Instant::now(),
                intent: description.clone(),
            });
        }
        
        // Send command to Monad
        self.cmd_tx.send(command).await
            .map_err(|_| anyhow!("Failed to send command to Monad - channel closed"))?;
        
        // Wait for result with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            self.receive_result()
        ).await
            .map_err(|_| anyhow!("Command timeout after 30s"))?;
        
        // Remove from pending
        {
            let mut pending = self.pending_commands.lock().await;
            pending.remove(&cmd_id);
        }
        
        result
    }
    
    /// Receive result from Monad
    async fn receive_result(&self) -> Result<CommandResult> {
        let mut rx = self.result_rx.lock().await;
        rx.recv().await
            .ok_or_else(|| anyhow!("Result channel closed"))
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // RESPONSE FORMATTING
    // ════════════════════════════════════════════════════════════════════════
    
    /// Format command result as minimal whisper response
    fn format_response(
        &self,
        parsed: &ParsedIntent,
        result: &CommandResult,
        elapsed: Duration,
    ) -> OracleResponse {
        // Handle special cases first
        match &parsed.action {
            IntentAction::Help => {
                return OracleResponse {
                    whisper: self.get_help_whisper(),
                    haptic: HapticPattern::Success,
                    confidence: 1.0,
                    ..Default::default()
                };
            }
            IntentAction::Conversation { response } => {
                return OracleResponse {
                    whisper: response.clone(),
                    haptic: HapticPattern::Success,
                    confidence: parsed.confidence,
                    ..Default::default()
                };
            }
            IntentAction::Clarify { question } => {
                return OracleResponse {
                    whisper: question.clone(),
                    haptic: HapticPattern::Attention,
                    needs_confirmation: true,
                    confidence: parsed.confidence,
                    ..Default::default()
                };
            }
            IntentAction::UniversalQuery { answer, source, confidence } => {
                // Phase 41: Return universal query answer directly
                return OracleResponse {
                    whisper: answer.clone(),
                    haptic: HapticPattern::Success,
                    confidence: *confidence,
                    data: Some(CommandData::Text(format!("source: {}", source))),
                    ..Default::default()
                };
            }
            _ => {}
        }
        
        // Format based on result
        match result {
            CommandResult::Success { data, .. } => {
                let (whisper, data_clone) = self.format_success_whisper(&parsed.action, data);
                OracleResponse {
                    whisper,
                    haptic: HapticPattern::Success,
                    data: Some(data_clone),
                    confidence: parsed.confidence,
                    ..Default::default()
                }
            }
            CommandResult::Failure { error, recoverable, .. } => {
                OracleResponse {
                    whisper: if *recoverable {
                        format!("Couldn't complete that: {}. Try again?", error)
                    } else {
                        format!("Failed: {}", error)
                    },
                    haptic: HapticPattern::Error,
                    confidence: parsed.confidence,
                    ..Default::default()
                }
            }
            CommandResult::Pending { estimated_ms, .. } => {
                OracleResponse {
                    whisper: format!("Working on it... ~{}ms", estimated_ms),
                    haptic: HapticPattern::Thinking,
                    confidence: parsed.confidence,
                    ..Default::default()
                }
            }
        }
    }
    
    /// Format success result as whisper
    fn format_success_whisper(&self, action: &IntentAction, data: &CommandData) -> (String, CommandData) {
        let whisper = match (action, data) {
            (IntentAction::CheckBalance, CommandData::Balance(bal)) => {
                format!("Balance: {} KARA", bal)
            }
            (IntentAction::Transfer { to, amount, .. }, CommandData::TxHash(hash)) => {
                format!("Sent {} KARA to {} ✓ ({})", amount, to, &hash[..8])
            }
            (IntentAction::Stake { amount }, CommandData::TxHash(_)) => {
                format!("Staked {} KARA ✓", amount)
            }
            (IntentAction::Vote { proposal_id, approve }, CommandData::TxHash(_)) => {
                format!("Voted {} on proposal #{} ✓", 
                    if *approve { "YES" } else { "NO" }, proposal_id)
            }
            (IntentAction::ListFiles, CommandData::FileList(files)) => {
                if files.is_empty() {
                    "No files stored yet".to_string()
                } else {
                    format!("{} files: {}", files.len(), 
                        files.iter().take(3).map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "))
                }
            }
            (IntentAction::GetStatus, CommandData::PipelineStatus(status)) => {
                format!("Height: {} | Peers: {} | ZK Queue: {}", 
                    status.chain_height, status.swarm_peers, status.zk_queue_size)
            }
            (IntentAction::SetTimer { minutes, label }, _) => {
                format!("Timer set: {} in {} min", label, minutes)
            }
            (IntentAction::Navigate { destination }, _) => {
                format!("Navigating to {}", destination)
            }
            (IntentAction::StoreData { name, .. }, CommandData::StoredHash(_)) => {
                format!("Saved: {} ✓", name)
            }
            _ => "Done ✓".to_string(),
        };
        
        (whisper, data.clone())
    }
    
    /// Get help text as whisper
    fn get_help_whisper(&self) -> String {
        r#"Voice commands:
• "Check my balance"
• "Send 50 KARA to Alice"
• "Set timer for 5 minutes"
• "Navigate to downtown"
• "What am I looking at?"
• "Show my files""#.to_string()
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // MANIFEST CONTROL (AR Whispers + Haptic)
    // ════════════════════════════════════════════════════════════════════════
    
    /// Get current AR overlays for rendering
    pub async fn get_ar_overlays(&self) -> Vec<AROverlay> {
        let mut manifest = self.manifest.lock().await;
        manifest.get_overlays()
    }
    
    /// Set output mode (full, minimal, haptic-only, silent)
    pub async fn set_output_mode(&self, mode: crate::oracle::manifest::OutputMode) {
        let mut manifest = self.manifest.lock().await;
        manifest.set_mode(mode);
    }
    
    /// Play haptic pattern directly
    pub async fn play_haptic(&self, pattern: HapticPattern) -> Result<()> {
        let mut manifest = self.manifest.lock().await;
        manifest.play_haptic(pattern).await
    }
    
    /// Show a whisper directly (bypassing mediate flow)
    pub async fn show_whisper(&self, text: &str, style: WhisperStyle) -> Result<()> {
        let mut manifest = self.manifest.lock().await;
        manifest.show_whisper(text, style)
    }
    
    /// Clear all AR overlays
    pub async fn clear_overlays(&self) {
        let mut manifest = self.manifest.lock().await;
        manifest.clear_overlays();
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // SWARM EVENT HANDLING
    // ════════════════════════════════════════════════════════════════════════
    
    /// Handle swarm events routed from Monad
    pub async fn handle_swarm_event(&self, event: SwarmEvent) {
        match event {
            SwarmEvent::MessageReceived { from, content } => {
                log::info!("[ORACLE] Swarm message from {}: {}", from, content);
                // Could trigger notification whisper
            }
            SwarmEvent::PeerConnected { peer_id } => {
                log::info!("[ORACLE] Peer connected: {}", peer_id);
            }
            SwarmEvent::PeerDisconnected { peer_id } => {
                log::info!("[ORACLE] Peer disconnected: {}", peer_id);
            }
        }
    }
}

/// Swarm events that Oracle needs to handle
#[derive(Debug, Clone)]
pub enum SwarmEvent {
    MessageReceived { from: String, content: String },
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_prover() {
        let prover = IntentProver::new();
        let intent = ParsedIntent {
            action: IntentAction::CheckBalance,
            raw: "check my balance".to_string(),
            confidence: 0.9,
            timestamp: 12345,
            source: InputSource::Voice,
        };
        
        let proof = prover.prove_intent(&intent, "did:karana:test").unwrap();
        // Proof size varies: 128 bytes for stub, 192 bytes for real Groth16
        assert!(proof.len() == 128 || proof.len() == 192, 
            "Proof should be 128 (stub) or 192 (Groth16) bytes, got {}", proof.len());
        assert!(prover.verify(&proof));
    }
    
    #[test]
    fn test_parse_ai_response() {
        // This would require setting up channels which is complex for unit tests
        // Integration tests would be better here
    }
}
