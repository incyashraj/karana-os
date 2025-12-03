# Kāraṇa OS Architecture

> **Kāraṇa** (कारण) - Sanskrit for "cause" or "instrument" - The cause that enables sovereign computing.

## Overview

Kāraṇa OS is a **self-sovereign operating system** designed specifically for wearable computing, particularly smart glasses. It combines blockchain technology, edge AI, spatial computing, and privacy-first principles to create a truly personal computing experience where the user owns their data, identity, and compute.

**Current Status: 221 tests passing across all modules**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           KĀRAṆA OS STACK                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    INTERFACE LAYER                                   │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │  Voice  │ │ Camera  │ │   HUD   │ │ Haptic  │ │  Gaze   │       │   │
│  │  │(Whisper)│ │ (BLIP)  │ │ (AR/XR) │ │Feedback │ │Tracking │       │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │   │
│  └───────┴───────────┴───────────┴───────────┴───────────┴─────────────┘   │
│                                  │                                          │
│  ┌───────────────────────────────┴─────────────────────────────────────┐   │
│  │                    SPATIAL AR LAYER                                  │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  World    │ │  Spatial  │ │   SLAM    │ │   Room    │           │   │
│  │  │  Coords   │ │  Anchors  │ │  Engine   │ │  Mapping  │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  AR Tabs  │ │   Tab     │ │ Gaze/Voice│ │    Tab    │           │   │
│  │  │  Manager  │ │  Browser  │ │Interaction│ │  Renderer │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐   │
│  │                     ORACLE LAYER                                     │   │
│  │         (AI ↔ Blockchain Bridge / Intent Processing / ZK Proofs)    │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │   Veil    │ │ Manifest  │ │ Use Cases │ │  Intent   │           │   │
│  │  │ (Intent)  │ │ (Haptic)  │ │(Scenarios)│ │  Proofs   │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐   │
│  │                  INTELLIGENCE LAYER                                  │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  Context  │ │  Memory   │ │ Learning  │ │ Proactive │           │   │
│  │  │ Awareness │ │  System   │ │  Engine   │ │ Suggest.  │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐   │
│  │                   BLOCKCHAIN LAYER                                   │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │   Chain   │ │  Ledger   │ │Governance │ │  Wallet   │           │   │
│  │  │ (Blocks)  │ │  (KARA)   │ │   (DAO)   │ │ (Ed25519) │           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐   │
│  │                    NETWORK LAYER                                     │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐            │   │
│  │  │   libp2p    │ │  Celestia   │ │    ZK Proofs        │            │   │
│  │  │  (Gossip)   │ │    (DA)     │ │ (Privacy/Verify)    │            │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘            │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│  ┌─────────────────────────────┴───────────────────────────────────────┐   │
│  │                   HARDWARE LAYER                                     │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │   │
│  │  │  Virtual  │ │  Power    │ │  Display  │ │  Sensors  │           │   │
│  │  │  Glasses  │ │ Manager   │ │ Waveguide │ │IMU/GPS/Dep│           │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Module Deep Dive

### 1. Spatial AR System (`spatial/`)

The foundation for persistent AR experiences. Allows content to be "pinned" in physical space.

#### World Coordinates (`world_coords.rs`)

```rust
/// A position in the real world combining GPS and local SLAM coordinates
pub struct WorldPosition {
    pub local: LocalCoord,      // SLAM-relative (x, y, z in meters)
    pub room_id: Option<RoomId>, // Which room we're in
    pub gps: Option<GpsCoord>,   // Outdoor GPS coordinates
    pub floor: Option<i32>,      // Building floor number
    pub confidence: f32,         // Position confidence (0.0 - 1.0)
}

impl WorldPosition {
    /// Create position from local SLAM coordinates
    pub fn from_local(x: f32, y: f32, z: f32) -> Self;
    
    /// Create position in a specific room
    pub fn in_room(local: LocalCoord, room: RoomId) -> Self;
    
    /// Create outdoor position with GPS
    pub fn outdoor(gps: GpsCoord) -> Self;
    
    /// Calculate distance to another position
    pub fn distance_to(&self, other: &WorldPosition) -> f32;
}
```

#### Spatial Anchors (`anchor.rs`)

Persistent markers in the real world where AR content can be attached:

```rust
pub struct SpatialAnchor {
    pub id: AnchorId,
    pub position: WorldPosition,
    pub orientation: Quaternion,
    pub visual_signature: VisualHash,  // For relocalization
    pub content_hash: ContentHash,     // Integrity verification
    pub content: AnchorContent,        // What's pinned here
    pub state: AnchorState,            // Active, Degraded, Lost
    pub confidence: f32,               // Tracking confidence
}

pub enum AnchorContent {
    Text { text: String },
    Browser { url: String, title: Option<String>, scroll_position: f32 },
    Video { url: String, position_secs: f32, is_playing: bool },
    CodeEditor { file_path: String, cursor_line: u32, language: String },
    Game { game_id: String, state_hash: [u8; 32] },
    Model3D { model_url: String, scale: f32 },
    Waypoint { destination: String, step_number: u32 },
    Custom { app_id: String, state: Vec<u8> },
}
```

#### SLAM Engine (`slam.rs`)

Visual Simultaneous Localization and Mapping:

```rust
pub struct SlamEngine {
    config: SlamConfig,
    state: SlamState,
    features: FeatureDatabase,
    keyframes: Vec<Keyframe>,
    current_pose: Pose6DOF,
    map_points: Vec<MapPoint>,
}

impl SlamEngine {
    /// Process a new camera frame
    pub fn process_frame(&mut self, frame: &CameraFrame) -> SlamResult;
    
    /// Get current 6-DOF pose (position + orientation)
    pub fn current_pose(&self) -> &Pose6DOF;
    
    /// Check if tracking is healthy
    pub fn is_tracking(&self) -> bool;
    
    /// Export map for persistence
    pub fn export_map(&self) -> SlamMap;
}
```

#### Relocalization (`relocalize.rs`)

Re-finding position after tracking loss:

```rust
pub struct Relocalizer {
    stored_keyframes: Vec<StoredKeyframe>,
    place_recognition: PlaceRecognitionIndex,
}

impl Relocalizer {
    /// Attempt to relocalize from current view
    pub fn try_relocalize(&self, frame: &CameraFrame) -> Option<RelocalizationResult>;
    
    /// Match visual features against stored keyframes
    fn match_features(&self, features: &[Feature]) -> Vec<KeyframeMatch>;
}
```

---

### 2. Persistent AR Tabs (`ar_tabs/`)

Browser-like tabs that exist in physical space. "Pin a browser to your kitchen counter."

#### Tab Core (`tab.rs`)

```rust
pub struct ARTab {
    pub id: TabId,
    pub anchor: SpatialAnchor,       // Where in the world
    pub content: TabContent,          // What's displayed
    pub size: TabSize,                // Physical dimensions
    pub state: TabState,              // Active, Minimized, etc.
    pub style: TabStyle,              // Visual appearance
    pub interaction_zone: InteractionZone,  // Hit testing area
}

pub enum TabContent {
    Browser(BrowserState),      // Web pages
    VideoPlayer(VideoState),    // Videos
    CodeEditor(CodeState),      // Code with syntax highlighting
    Document(DocumentState),    // PDFs, documents
    Game(GameState),           // Games
    Widget(WidgetState),       // Clocks, weather, etc.
    Custom(CustomContent),     // Third-party content
}

pub enum TabSize {
    Small,    // Post-it note (0.2m x 0.15m)
    Medium,   // Clipboard (0.4m x 0.3m)
    Large,    // TV-sized (0.8m x 0.5m)
    Full,     // Wall-sized (1.5m x 1.0m)
}

pub enum TabStyle {
    Glass,       // Transparent with blur
    Solid,       // Opaque background
    Holographic, // Sci-fi hologram effect
    Neon,        // Glowing edges
    Minimal,     // Just content, no chrome
}
```

#### Tab Manager (`manager.rs`)

```rust
pub struct TabManager {
    tabs: HashMap<TabId, ARTab>,
    focus_history: VecDeque<TabId>,
    layout_mode: LayoutMode,
    location_groups: HashMap<String, Vec<TabId>>,
}

pub enum LayoutMode {
    Free,      // Tabs placed anywhere
    Grid,      // Auto-arranged grid
    Stack,     // Overlapping stack
    Carousel,  // Circular arrangement
    Dock,      // Pinned to edges
}

impl TabManager {
    /// Pin a new tab in space
    pub fn pin_tab(&mut self, content: TabContent, size: TabSize, 
                   anchor: SpatialAnchor, location_hint: Option<&str>) -> Result<TabId>;
    
    /// Focus a specific tab
    pub fn focus(&mut self, id: TabId) -> Result<()>;
    
    /// Minimize tab (still in space, but shrunk)
    pub fn minimize(&mut self, id: TabId) -> Result<()>;
    
    /// Close and remove tab
    pub fn close(&mut self, id: TabId) -> Result<()>;
    
    /// Handle relocalization (anchors moved)
    pub fn on_relocalize(&mut self, updates: &[(AnchorId, SpatialAnchor)]);
}
```

#### Gaze & Voice Interaction (`interaction.rs`)

```rust
pub struct TabInteraction {
    gaze: GazeTracker,
    dwell_config: DwellConfig,
    target_tab: Option<TabId>,
}

pub struct DwellConfig {
    pub select_time_ms: u32,    // Default: 500ms gaze to select
    pub cancel_distance: f32,   // How far gaze can drift
    pub feedback_start_ms: u32, // When to show progress
}

impl TabInteraction {
    /// Process gaze update
    pub fn on_gaze(&mut self, gaze_point: &WorldPosition, tabs: &[&ARTab], 
                   timestamp_ms: u64) -> Option<InteractionEvent>;
    
    /// Process voice command
    pub fn on_voice(&mut self, command: &str) -> Option<InteractionEvent>;
}

pub enum InteractionEvent {
    GazeEnter(TabId),
    GazeExit(TabId),
    DwellProgress { tab_id: TabId, progress: f32 },
    DwellSelect(TabId),
    CursorMove { tab_id: TabId, position: (f32, f32) },
    VoiceCommand { tab_id: Option<TabId>, command: VoiceTabCommand },
}

// Voice commands for tabs
pub enum VoiceTabCommand {
    Scroll { direction: ScrollDirection, amount: ScrollAmount },
    Close,
    Minimize,
    Maximize,
    NextTab,
    PrevTab,
    GoBack,
    GoForward,
    Reload,
    Navigate(String),  // URL or search query
}
```

#### Tab Renderer (`render.rs`)

```rust
pub struct TabRenderer {
    config: RenderConfig,
    frame_buffer: CompositeFrame,
    depth_buffer: DepthBuffer,
    tab_states: HashMap<TabId, TabRenderState>,
}

impl TabRenderer {
    /// Render all visible tabs
    pub fn render_tabs(&mut self, tabs: &[&ARTab], viewer_pos: &WorldPosition,
                       view_matrix: &[[f32; 4]; 4], 
                       projection_matrix: &[[f32; 4]; 4]) -> &CompositeFrame;
}

pub struct CompositeFrame {
    pub width: u32,
    pub height: u32,
    pub overlays: Vec<TabOverlay>,
}

pub struct TabOverlay {
    pub tab_id: TabId,
    pub screen_rect: ScreenRect,
    pub depth: f32,
    pub style: TabStyle,
    pub state: TabState,
    pub content_ready: bool,
}
```

---

### 3. Oracle Layer (`oracle/`)

The bridge between natural language, AI, and blockchain operations.

#### Oracle Veil (`veil.rs`)

```rust
pub struct OracleVeil {
    intent_prover: IntentProver,
    action_executor: ActionExecutor,
    manifest_renderer: ManifestRenderer,
}

impl OracleVeil {
    /// Process a natural language command
    pub async fn process_command(&mut self, command: &str, 
                                  context: &UserContext) -> Result<OracleResponse>;
    
    /// Execute an intent with ZK proof
    pub async fn execute_intent(&mut self, intent: &Intent, 
                                 proof: &IntentProof) -> Result<ExecutionResult>;
}

pub struct OracleResponse {
    pub action: OracleAction,
    pub ui_manifest: UIManifest,
    pub haptic_pattern: Option<HapticPattern>,
    pub voice_response: Option<String>,
}
```

#### Manifest System (`manifest.rs`)

Defines how to render responses to the user:

```rust
pub struct UIManifest {
    pub ar_overlays: Vec<AROverlay>,
    pub whisper: Option<WhisperNotification>,
    pub haptic: Option<HapticPattern>,
}

pub struct AROverlay {
    pub overlay_type: AROverlayType,
    pub position: OverlayPosition,
    pub content: String,
    pub duration_ms: Option<u64>,
}

pub enum AROverlayType {
    Toast,           // Brief notification
    Card,            // Information card
    Confirmation,    // Yes/No dialog
    Progress,        // Loading/progress indicator
    Navigation,      // Turn-by-turn arrow
    Highlight,       // Object highlight
}

pub enum HapticPattern {
    Confirm,         // Short double-tap
    Alert,           // Attention-getting buzz
    Navigation,      // Directional guidance
    Heartbeat,       // Gentle pulse
    Custom(Vec<HapticPulse>),
}
```

#### Use Cases (`use_cases.rs`)

Real-world scenario implementations:

```rust
// Restaurant bill splitting
pub async fn split_restaurant_bill(receipt_image: &[u8], 
                                    party_size: usize) -> Result<BillSplit>;

// Transit navigation with AR
pub async fn navigate_transit(destination: &str, 
                               current_pos: &WorldPosition) -> Result<TransitRoute>;

// Smart shopping with price comparison
pub async fn shopping_assistant(product_image: &[u8]) -> Result<ShoppingInfo>;
```

---

### 4. Hardware Abstraction (`hardware/`)

#### Virtual Glasses Simulator (`simulator/`)

Full hardware simulation for development without physical devices:

```rust
pub struct VirtualGlasses {
    pub display: VirtualDisplay,
    pub camera: VirtualCamera,
    pub imu: VirtualIMU,
    pub audio: VirtualAudio,
    pub battery: VirtualBattery,
    pub status: DeviceStatus,
}

impl VirtualGlasses {
    /// Simulate a frame of operation
    pub fn tick(&mut self, delta_ms: u64);
    
    /// Render current state to terminal
    pub fn render_tui(&self) -> String;
}
```

#### Power Management (`power.rs`)

```rust
pub struct PowerManager {
    battery_level: f32,
    power_profile: PowerProfile,
    thermal_state: ThermalState,
}

pub enum PowerProfile {
    Performance,    // Full power, all features
    Balanced,       // Default mode
    PowerSaver,     // Extended battery life
    UltraSaver,     // Minimum functionality
}

impl PowerManager {
    /// Get estimated remaining runtime
    pub fn estimate_remaining_minutes(&self) -> u32;
    
    /// Check if thermal throttling is needed
    pub fn needs_throttle(&self) -> bool;
}
```

---

### 5. Zero-Knowledge Proofs (`zk/`)

Privacy-preserving verification:

```rust
pub struct IntentProof {
    pub intent_type: IntentType,
    pub commitment: [u8; 32],
    pub range_proof: Option<RangeProof>,
    pub authorization_proof: AuthorizationProof,
}

pub enum IntentType {
    Transfer,
    Stake,
    Vote,
    Query,
    Capture,
    Navigate,
}

impl IntentProof {
    /// Create a proof that authorizes an intent without revealing details
    pub fn create(intent: &Intent, witness: &IntentWitness) -> Result<Self>;
    
    /// Verify a proof is valid
    pub fn verify(&self, public_inputs: &PublicInputs) -> bool;
}
```

---

### 6. Core Infrastructure

#### Wallet (`wallet.rs`)

```rust
pub struct KaranaWallet {
    keypair: Ed25519Keypair,
    did: String,              // did:karana:<base58>
    device_id: String,
}

impl KaranaWallet {
    /// Generate new wallet with 24-word mnemonic
    pub fn generate(device_id: &str) -> Result<WalletCreationResult>;
    
    /// Restore from mnemonic
    pub fn restore(mnemonic: &str, device_id: &str) -> Result<Self>;
    
    /// Sign data with Ed25519
    pub fn sign(&self, data: &[u8]) -> Signature;
    
    /// Get DID (Decentralized Identifier)
    pub fn did(&self) -> &str;
}
```

#### Blockchain (`chain.rs`)

```rust
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<SignedTransaction>,
    pub state_root: [u8; 32],
}

pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub prev_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub proposer: PublicKey,
    pub signature: Signature,
}

impl Block {
    /// Verify block signature and all transactions
    pub fn verify(&self) -> bool;
}
```

#### Voice Processing (`voice.rs`)

```rust
pub struct VoiceProcessor {
    vad: VoiceActivityDetector,
    wake_word: WakeWordDetector,
    transcriber: WhisperTranscriber,
}

impl VoiceProcessor {
    /// Check if wake word was spoken
    pub fn detect_wake_word(&self, audio: &[f32]) -> bool;
    
    /// Transcribe speech to text
    pub fn transcribe(&self, audio: &[f32]) -> Result<String>;
}

// Wake word variants (phonetic matching)
const WAKE_WORDS: &[&str] = &[
    "hey karana", "okay karana", "hi karana",
    "hey karna", "okay karna",  // Common mispronunciations
    "hey carana", "okay carana",
];
```

---

## Security Model

### Cryptographic Stack

| Layer | Algorithm | Purpose |
|-------|-----------|---------|
| Identity | Ed25519 | Transaction signing, DID verification |
| Storage | AES-256-GCM | Wallet encryption at rest |
| Key Derivation | PBKDF2-SHA256 | Password → encryption key |
| ZK Proofs | Groth16 | Privacy-preserving intent verification |
| Hashing | SHA-256 / Blake3 | Block hashes, Merkle roots |
| Visual Signatures | Perceptual Hash | Relocalization matching |

### Privacy Guarantees

1. **Local-First**: All AI inference runs on-device
2. **ZK Intents**: Prove authorization without revealing details
3. **Encrypted Storage**: Wallet and sensitive data encrypted at rest
4. **No Cloud Dependency**: Works fully offline

---

## Testing

```bash
# Run all tests (221 passing)
cargo test --lib

# Module breakdown:
# spatial:      45 tests (world coords, anchors, SLAM, relocalization)
# ar_tabs:      62 tests (tabs, manager, browser, interaction, render)
# oracle:       25 tests (veil, manifest, use cases)
# zk:            8 tests (intent proofs, range proofs)
# wallet:        6 tests
# chain:         4 tests
# voice:         7 tests
# hardware:     15 tests (simulator, devices, power)
# glasses:      12 tests
# timer:         5 tests
# notifications: 8 tests
# ... and more
```

---

## Getting Started

```bash
# Clone
git clone https://github.com/incyashraj/karana-os
cd karana-os/karana-core

# Build
cargo build --release

# Run tests
cargo test --lib

# Run with real camera (Linux)
cargo build --release --features v4l2
```

---

## Roadmap

### Completed ✅
- [x] Spatial AR System (world coords, anchors, SLAM, relocalization)
- [x] Persistent AR Tabs (pinned browsers, gaze interaction, voice control)
- [x] Oracle with ZK Intent Proofs
- [x] Hardware Abstraction Layer
- [x] Virtual Glasses Simulator
- [x] 221 tests passing

### In Progress 🚧
- [ ] Oracle Tab Integration (voice → pin tab)
- [ ] Multi-device Sync
- [ ] Celestia DA Integration

### Planned 📋
- [ ] App Marketplace (governance-approved dApps)
- [ ] Hardware Wallet Support
- [ ] Mesh Networking (peer-to-peer without internet)
- [ ] Real Hardware Support (XREAL, Rokid, custom)

---

## License

MIT License - Built for the sovereign future.

---

*Kāraṇa OS: Your glasses, your data, your rules.*
