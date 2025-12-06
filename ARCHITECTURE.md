# Kāraṇa OS - Technical Architecture

> The operating system is not a tool. It is a partner.

**Status: 2,058 tests | 180,000+ LOC Rust | Phases 1-40 Complete**

---

## Layered Architecture Stack (9 Layers)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Layer 9: System Services (OTA, Security, Diagnostics, Recovery)         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 8: Applications (Timer, Navigation, Social, Settings, Wellness)   │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 7: Interface (Voice, HUD, Gestures, Gaze, AR Rendering)           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 6: AI Engine (NLU, Dialogue, Reasoning, Action Execution)         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 5: Intelligence (Multimodal Fusion, Scene Understanding, Memory)  │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 4: Oracle Bridge (Intent Processing, Manifest Rendering, ZK)      │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 3: Blockchain (Chain, Ledger, Governance, Wallet, Celestia DA)    │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 2: P2P Network (libp2p, mDNS, Gossip, Sync)                       │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 1: Hardware (Camera, Sensors, Audio, Display, Power)              │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Architectural Patterns

### 1. Monad Pattern - Central Orchestrator

The **Monad** (`src/monad.rs`) is the single source of truth orchestrating all 9 layers:

```rust
pub struct Karana {
    // Layer 1: Hardware
    pub hardware: Arc<HardwareManager>,
    
    // Layer 2: Network
    pub p2p: Arc<P2PNetwork>,
    
    // Layer 3: Blockchain
    pub chain: Arc<Blockchain>,
    pub wallet: Arc<KaranaWallet>,
    pub ledger: Arc<Ledger>,
    pub celestia: Arc<CelestiaClient>,
    
    // Layer 4: Oracle
    pub oracle: Arc<OracleVeil>,
    pub intent_prover: Arc<IntentProver>,
    
    // Layer 5: Intelligence
    pub multimodal: Arc<MultimodalFusion>,
    pub scene_understanding: Arc<SceneUnderstanding>,
    
    // Layer 6: AI
    pub nlu: Arc<NLUEngine>,
    pub dialogue: Arc<DialogueManager>,
    pub reasoning: Arc<ReasoningEngine>,
    
    // Layer 7: Interface
    pub voice: Arc<VoiceCommandManager>,
    pub hud: Arc<HUDManager>,
    pub ar: Arc<ARRenderer>,
    pub gaze: Arc<GazeTracker>,
    pub gesture: Arc<GestureRecognizer>,
    
    // Layer 8: Applications
    pub timer: Arc<TimerManager>,
    pub notifications: Arc<NotificationManager>,
    pub navigation: Arc<NavigationEngine>,
    pub social: Arc<SocialManager>,
    pub settings: Arc<SettingsManager>,
    pub wellness: Arc<WellnessManager>,
    
    // Layer 9: System Services
    pub diagnostics: Arc<DiagnosticsManager>,
    pub recovery: Arc<RecoveryManager>,
    pub security: Arc<SecurityManager>,
    pub ota: Arc<OTAManager>,
    
    pub fn tick(&mut self, delta_ms: u64) -> Result<()>,
    pub fn process_intent(&mut self, intent: &Intent) -> Result<CommandResult>,
}
```

**Tick Loop (30 second blocks):**
```
Every 30 seconds:
  1. Collect sensor data from Layer 1
  2. Update position & state (Layer 3)
  3. Process voice/input (Layer 7)
  4. Run AI inference (Layer 6)
  5. Update oracle state (Layer 4)
  6. Propose block (Layer 3)
  7. Broadcast to network (Layer 2)
  8. Render AR output (Layer 7)
```

---

## Layer Breakdown & Data Flow

### Layer 1: Hardware Abstraction

```
Physical Hardware
    ↓
┌─────────────────────────────────┐
│   Hardware Drivers              │
│ ┌──────────────────────────────┐│
│ │ CameraDriver    (v4l2/sim)   ││
│ │ IMUSensor       (accel/gyro) ││
│ │ AudioDriver     (mic/speaker)││
│ │ DisplayDriver   (OLED output)││
│ │ PowerDriver     (battery mgmt)││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Sensor Fusion                 │
│ ┌──────────────────────────────┐│
│ │ IMU Fusion (Pose6DOF)        ││
│ │ GPS/SLAM Fusion (Position)   ││
│ │ Light Probe (Environment)    ││
│ │ Battery/Thermal Monitor      ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ CameraFrame, SensorData, BatteryStatus
```

**Key Types:**
```rust
pub trait CameraDriver: Send + Sync {
    fn capture_frame(&mut self) -> Result<CameraFrame>;
    fn set_resolution(&mut self, w: u32, h: u32) -> Result<()>;
}

pub struct SensorFusion {
    pub imu: IMUData,           // accel, gyro, mag
    pub pose: Pose6DOF,         // position + orientation
    pub velocity: Vector3,
    pub confidence: f32,
}

pub struct BatteryStatus {
    pub level: f32,             // 0.0-1.0
    pub temperature: f32,       // Celsius
    pub estimated_minutes: u32,
    pub thermal_state: ThermalState,
}
```

---

### Layer 2: P2P Network

```
┌─────────────────────────────────┐
│   libp2p Stack                  │
│ ┌──────────────────────────────┐│
│ │ mDNS Discovery               ││
│ │ Gossipsub Pubsub             ││
│ │ Kad DHT                      ││
│ │ Noise Protocol (Encryption)  ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Network Messages
┌─────────────────────────────────┐
│   Message Types                 │
│ ┌──────────────────────────────┐│
│ │ BlockProposal { block }      ││
│ │ BlockVote { height, hash }   ││
│ │ Transaction { signed_tx }    ││
│ │ StateSync { blocks... }      ││
│ │ Peer { peer_id, addr }       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Gossip to all peers
```

**Key Types:**
```rust
pub struct P2PNetwork {
    pub swarm: Swarm<NetworkBehaviour>,
    pub peers: HashMap<PeerId, PeerInfo>,
    pub pending_blocks: VecDeque<SignedBlock>,
    
    pub broadcast(&self, msg: NetworkMessage) -> Result<()>,
    pub get_peer_info(&self, peer_id: PeerId) -> Option<PeerInfo>,
}

pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: u64,
    pub blocks_shared: u32,
    pub reputation: f32,
}

pub enum NetworkMessage {
    BlockProposal(SignedBlock),
    BlockVote { height: u64, block_hash: [u8; 32] },
    Transaction(SignedTransaction),
    StateSync(Vec<Block>),
}
```

---

### Layer 3: Blockchain & Consensus

```
┌─────────────────────────────────┐
│   Block Production (30s)        │
│ ┌──────────────────────────────┐│
│ │ 1. Collect txs from pool    ││
│ │ 2. Create BlockBody         ││
│ │ 3. Compute state_root       ││
│ │ 4. Sign with Ed25519        ││
│ │ 5. Broadcast                ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Block Verification           │
│ ┌──────────────────────────────┐│
│ │ Check signature              ││
│ │ Verify all txs               ││
│ │ Validate merkle_root         ││
│ │ Check chain continuity       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   State Machine                 │
│ ┌──────────────────────────────┐│
│ │ Apply transactions           ││
│ │ Update account balances      ││
│ │ Record intents (for proofs)  ││
│ │ Update ledger state          ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
```

**Key Types:**
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
    pub signature: Signature,  // Ed25519
}

pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Signature,
    pub public_key: PublicKey,
}

pub enum Transaction {
    Transfer { to: String, amount: u64 },
    Stake { amount: u64 },
    Vote { governance_id: u64, choice: u32 },
    Intent { intent_proof: IntentProof },
    Custom(Vec<u8>),
}

pub struct Ledger {
    pub accounts: HashMap<String, AccountState>,
    pub nonce: HashMap<String, u64>,
    pub recorded_intents: Vec<RecordedIntent>,
}
```

**Wallet Integration:**
```rust
pub struct KaranaWallet {
    pub keypair: Ed25519Keypair,
    pub did: String,  // did:karana:base58(pubkey)
    
    pub sign(&self, data: &[u8]) -> Signature,
    pub verify(&self, sig: &Signature, data: &[u8]) -> bool,
}
```

---

### Layer 4: Oracle Bridge

```
Intent (Voice/Gesture Input)
    ↓
┌─────────────────────────────────┐
│   Veil - Intent Processor       │
│ ┌──────────────────────────────┐│
│ │ 1. Parse intent              ││
│ │ 2. Validate with ZK proof    ││
│ │ 3. Check permissions (RBAC)  ││
│ │ 4. Reserve gas               ││
│ │ 5. Create execution plan     ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Manifest - Output Renderer    │
│ ┌──────────────────────────────┐│
│ │ 1. AR overlays (Toast/Card)  ││
│ │ 2. Haptic feedback           ││
│ │ │ 3. Voice response (TTS)     ││
│ │ 4. Visual effects            ││
│ │ 5. Update blockchain state   ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ CommandResult
```

**Key Types:**
```rust
pub struct OracleVeil {
    pub process_command(
        &mut self,
        command: &str,
        context: &UserContext,
    ) -> Result<OracleResponse>,
}

pub struct OracleResponse {
    pub action: OracleAction,
    pub ui_manifest: UIManifest,
    pub haptic_pattern: Option<HapticPattern>,
    pub voice_response: Option<String>,
    pub blockchain_intent: Option<Intent>,
}

pub struct UIManifest {
    pub ar_overlays: Vec<AROverlay>,
    pub whisper: Option<WhisperNotification>,
    pub haptic: Option<HapticPattern>,
    pub duration_ms: Option<u64>,
}

pub enum AROverlay {
    Toast { text: String, duration_ms: u32 },
    Card { title: String, content: String },
    Confirmation { prompt: String, options: Vec<String> },
    Navigation { instruction: String, direction: Vector3 },
    Highlight { target: Vector3, color: Color },
}

pub struct IntentProof {
    pub intent: Intent,
    pub commitment: [u8; 32],
    pub range_proof: Option<RangeProof>,
    pub authorization_proof: AuthorizationProof,
}
```

---

### Layer 5: Intelligence

```
Scene Understanding
    ↓
┌─────────────────────────────────┐
│   Multimodal Fusion             │
│ ┌──────────────────────────────┐│
│ │ Voice Input (text)           ││
│ │ Gaze Input (eye direction)   ││
│ │ Gesture Input (hand pose)    ││
│ │ Context Input (location)     ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Combined signal
┌─────────────────────────────────┐
│   Scene Understanding           │
│ ┌──────────────────────────────┐│
│ │ Object detection             ││
│ │ Semantic labeling            ││
│ │ Relationship graphs          ││
│ │ Attention prediction         ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ SceneDescription
┌─────────────────────────────────┐
│   Memory & Prediction           │
│ ┌──────────────────────────────┐││
│ │ Episodic memory (events)     │││
│ │ Semantic memory (facts)      │││
│ │ Procedural memory (skills)   │││
│ │ Predictive model (next step) │││
│ └──────────────────────────────┘││
└─────────────────────────────────┘│
    ↓ Context for Layer 6
```

**Key Types:**
```rust
pub struct MultimodalFusion {
    pub fuse_inputs(
        &self,
        voice: Option<&str>,
        gaze: Option<&GazePoint>,
        gesture: Option<&GestureType>,
        context: &VoiceContext,
    ) -> FusedCommand,
}

pub struct FusedCommand {
    pub primary_intent: Intent,
    pub confidence: f32,
    pub source: CommandSource,
    pub context_relevance: f32,
}

pub enum CommandSource {
    VoiceOnly,
    GazeOnly,
    GestureOnly,
    VoiceGaze,
    VoiceGesture,
    GazeGesture,
    All,
}

pub struct SceneDescription {
    pub foreground: Vec<DetectedObject>,
    pub background: Vec<DetectedObject>,
    pub relationships: Vec<ObjectRelationship>,
    pub activity: Option<ActivityLabel>,
    pub lighting: LightProbe,
}

pub struct DetectedObject {
    pub class_id: u32,
    pub class_name: String,
    pub position: Vector3,
    pub bbox: BoundingBox,
    pub confidence: f32,
}
```

---

### Layer 6: AI Engine

```
FusedCommand Input
    ↓
┌─────────────────────────────────┐
│   NLU Engine                    │
│ ┌──────────────────────────────┐│
│ │ Intent Classifier            ││
│ │  ├─ Pattern matching         ││
│ │  ├─ Confidence scoring       ││
│ │  └─ Alternative intents      ││
│ │ Entity Extractor             ││
│ │  ├─ Slot identification      ││
│ │  └─ Value normalization      ││
│ │ Semantic Parser              ││
│ │  └─ Slot filling             ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ SemanticFrame
┌─────────────────────────────────┐
│   Dialogue Manager              │
│ ┌──────────────────────────────┐│
│ │ Conversation state (FSM)     ││
│ │ Turn-taking management       ││
│ │ Context update               ││
│ │ Clarification detection      ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ DialogueState
┌─────────────────────────────────┐
│   Reasoning Engine              │
│ ┌──────────────────────────────┐│
│ │ Forward chaining (if-then)   ││
│ │ Constraint satisfaction      ││
│ │ Conflict resolution          ││
│ │ Explanation generation       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ InferenceResult
┌─────────────────────────────────┐
│   Action Executor               │
│ ┌──────────────────────────────┐│
│ │ Safety validation            ││
│ │ Permission check (RBAC)      ││
│ │ Atomic execution             ││
│ │ Rollback on failure          ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ ExecutionResult
```

**Key Types:**
```rust
pub struct NLUEngine {
    pub classify_intent(&self, text: &str) -> IntentClassification,
    pub extract_entities(&self, text: &str) -> Vec<Entity>,
    pub parse_semantics(&self, text: &str) -> SemanticFrame,
}

pub struct IntentClassification {
    pub primary: Intent,
    pub confidence: f32,
    pub alternatives: Vec<(Intent, f32)>,
}

pub enum Intent {
    Navigate { destination: String },
    OpenApp { app: String },
    Query { question: String },
    Control { device: String, action: String },
    Message { recipient: String, body: String },
    Capture { media_type: MediaType },
    Custom(String),
}

pub struct SemanticFrame {
    pub intent: Intent,
    pub slots: HashMap<String, SlotValue>,
    pub confidence: f32,
    pub complete: bool,
}

pub struct DialogueManager {
    pub state: DialogueState,
    pub history: VecDeque<DialogueTurn>,
    pub context: DialogueContext,
}

pub enum DialogueState {
    Idle,
    Listening,
    Processing,
    AwaitingConfirmation { prompt: String },
    AwaitingInput { input_type: InputType },
    Executing { action: String },
    Complete { result: String },
}

pub struct ReasoningEngine {
    pub facts: Vec<Fact>,
    pub rules: Vec<LogicalRule>,
    
    pub infer(&self, new_fact: &Fact) -> Vec<Fact>,
    pub explain(&self, conclusion: &Fact) -> ExplanationPath,
}

pub struct ActionExecutor {
    pub execute(
        &mut self,
        action: &IntentAction,
        context: &ExecutionContext,
    ) -> ExecutionResult,
}
```

---

### Layer 7: Interface

```
AR Renderer (GPU)
    ├─ Camera frame
    ├─ AR Tab positions
    ├─ HUD widgets
    ├─ Particle effects
    ├─ Lighting probe
    └─ Occlusion maps
         ↓
    ┌─────────────────────────────────┐
    │   Compositor                    │
    │ ┌──────────────────────────────┐│
    │ │ Layer composition            ││
    │ │ Depth sorting                ││
    │ │ Alpha blending               ││
    │ │ Post-processing              ││
    │ └──────────────────────────────┘│
    └─────────────────────────────────┘
         ↓ FrameBuffer → Display

Voice Input → Voice Pipeline
    ├─ VAD (Voice Activity Detection)
    ├─ Wake word detection
    ├─ Transcription (Whisper)
    └─ NLU → Dialogue → Oracle
         ↓ CommandResult

Gaze Input → Gaze Tracker
    ├─ Eye tracking
    ├─ Fixation detection
    ├─ Dwell selection (500ms)
    └─ Focus state
         ↓ GazeEvent

Gesture Input → Hand Detector → Gesture Recognizer
    ├─ Hand pose estimation
    ├─ Finger tracking
    ├─ Gesture classification (15+ types)
    └─ Confidence scoring
         ↓ GestureType
```

**Key Types:**
```rust
pub struct ARRenderer {
    pub scene: ARScene,
    pub camera: Camera,
    pub lighting: LightingEngine,
    pub effects: EffectsEngine,
    
    pub render(&mut self) -> FrameBuffer,
}

pub struct VoiceCommandManager {
    pub nlu_engine: Arc<NLUEngine>,
    pub synthesizer: Arc<VoiceSynthesizer>,
    pub context_manager: Arc<VoiceContextManager>,
    pub listener: Arc<ContinuousListener>,
    pub shortcuts: Arc<ShortcutManager>,
    
    pub process(&mut self, audio: &[f32]) -> CommandResult,
}

pub struct GazeTracker {
    pub current_gaze: GazePoint,
    pub fixation_duration: u32,
    pub dwell_threshold: u32,
    
    pub process_gaze(&mut self, raw: &EyeTrackingData) -> GazeEvent,
}

pub struct GestureRecognizer {
    pub models: HashMap<GestureType, GestureModel>,
    
    pub recognize(&self, hand: &HandPose) -> Option<(GestureType, f32)>,
}

pub enum GestureType {
    Pinch { strength: f32 },
    Grab { force: f32 },
    Point { target: Vector3 },
    Swipe { direction: Vector3, velocity: f32 },
    Rotate { axis: Vector3, angle: f32 },
    // ... 10+ more
}
```

---

### Layer 8: Applications

```
┌─────────────────────────────────┐
│ Timer       Navigation  Social   │
│ ├─ Countdown ├─ Routing  ├─ Chat│
│ ├─ Stopwatch ├─ POI      ├─ Pres│
│ └─ Lap       └─ Guidance └─ Cont│
├─────────────────────────────────┤
│ Settings    Wellness   Notifs   │
│ ├─ Profiles ├─ Eyes     ├─ Smart│
│ ├─ Cloud    ├─ Posture  ├─ Group│
│ └─ Sync     └─ Usage    └─ Primt│
└─────────────────────────────────┘
      ↓
    Layer 3 (Blockchain storage)
    Layer 4 (Oracle for execution)
    Layer 6 (AI for intelligence)
```

**Architecture Pattern:**
```rust
pub trait Application: Send + Sync {
    fn update(&mut self, delta_ms: u32, context: &AppContext) -> Result<()>,
    fn on_input(&mut self, event: &InputEvent) -> InputResponse,
    fn render(&self, target: &mut FrameBuffer) -> Result<()>,
}

pub struct AppContext {
    pub location: Location,
    pub time: SystemTime,
    pub user_preferences: HashMap<String, Value>,
    pub blockchain_state: Arc<Ledger>,
}

pub struct ApplicationManager {
    pub apps: HashMap<String, Box<dyn Application>>,
    pub active_app: Option<String>,
    
    pub launch(&mut self, app_id: &str) -> Result<()>,
    pub update_all(&mut self, delta_ms: u32) -> Result<()>,
}
```

---

### Layer 9: System Services

```
┌─────────────────────────────────┐
│   Diagnostics                   │
│ ├─ CPU/Memory metrics           │
│ ├─ Frame rate monitoring        │
│ ├─ Thermal management           │
│ ├─ Watchdog timer               │
│ └─ Performance profiling        │
├─────────────────────────────────┤
│   Security & Recovery           │
│ ├─ Multi-factor auth            │
│ ├─ Crash dumps                  │
│ ├─ Error logging                │
│ ├─ Auto-recovery                │
│ └─ Rollback points              │
├─────────────────────────────────┤
│   OTA Updates                   │
│ ├─ Secure download              │
│ ├─ Signature verification       │
│ ├─ Atomic install               │
│ └─ Rollback protection          │
└─────────────────────────────────┘
```

**Key Types:**
```rust
pub struct DiagnosticsManager {
    pub metrics: SystemMetrics,
    pub event_log: Vec<DiagnosticEvent>,
    
    pub report(&self) -> PerformanceReport,
    pub collect_metrics(&mut self),
}

pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub battery_level: f32,
    pub temperature: f32,
    pub frame_rate: f32,
}

pub struct OTAManager {
    pub check_updates(&self) -> Option<UpdateInfo>,
    pub download(&mut self, update: &UpdateInfo) -> Result<()>,
    pub install(&mut self) -> Result<()>,
    pub rollback(&mut self) -> Result<()>,
}
```

---

## Data Flow Examples

### Example 1: Voice Command → Action

```
1. Audio Input (Layer 1)
   └─ Microphone captures audio chunk

2. Voice Pipeline (Layer 7)
   ├─ VAD: "Is this speech?" → Yes
   ├─ Wake word: "Hey Karana" detected
   └─ Whisper: Transcribe → "navigate to home"

3. Multimodal Fusion (Layer 5)
   └─ Voice only (no gesture, gaze) → confidence 0.95

4. NLU Engine (Layer 6)
   ├─ Intent: Navigate
   ├─ Slot[destination]: "home"
   └─ Confidence: 0.97

5. Dialogue Manager (Layer 6)
   ├─ Current state: Idle
   ├─ New state: Executing
   └─ Next state: Complete

6. Oracle (Layer 4)
   ├─ Create Intent { Navigate { "home" } }
   ├─ Check ZK proof ✓
   ├─ Generate Manifest with AR overlay
   └─ Synthesize: "Navigating to home"

7. AR Renderer (Layer 7)
   ├─ Show navigation arrow
   ├─ Play haptic feedback
   └─ Output voice response

8. Blockchain (Layer 3)
   └─ Record intent in next block
```

### Example 2: Gesture Input → Tab Interaction

```
1. Camera Frame (Layer 1)
   └─ Hand in frame

2. Hand Detection (Layer 7)
   ├─ Hand pose estimation
   └─ Keypoint positions

3. Gesture Recognition (Layer 7)
   ├─ Pattern matching: "Pinch gesture"
   └─ Confidence: 0.94

4. AR Tab Focus (Layer 5)
   ├─ Gaze + gesture fusion
   ├─ Pinch on AR Tab → Click event
   └─ Tab gets focus

5. Multimodal Fusion (Layer 5)
   └─ Gesture + gaze combined → User intent clear

6. Application (Layer 8)
   └─ Tab processes click event

7. Oracle (Layer 4)
   └─ Create Intent { Control { "tab", "click" } }

8. Blockchain (Layer 3)
   └─ Record in ledger
```

### Example 3: Scene Understanding → Proactive Suggestion

```
1. Camera Frame (Layer 1)
   └─ Current scene: Kitchen

2. Scene Understanding (Layer 5)
   ├─ Detect: Stove, Pan, Ingredients
   ├─ Activity: Cooking
   └─ Attention: On stove

3. Memory (Layer 5)
   └─ Recall: User made soup yesterday

4. Prediction (Layer 5)
   └─ User likely cooking soup again

5. NLU (Layer 6)
   └─ Generate Intent { Query { "soup recipe?" } }

6. Oracle (Layer 4)
   ├─ Check ZK proof ✓
   └─ Generate Toast overlay: "Recipe: Tomato Soup"

7. AR Renderer (Layer 7)
   └─ Show AR overlay with recipe steps

8. Blockchain (Layer 3)
   └─ Record proactive suggestion
```

---

## Communication Patterns

### Synchronous (Blocking)

```rust
// Within same thread/async task
let result = voice_manager.process(audio)?;
let intent = nlu_engine.classify_intent(&text)?;
let scene = scene_understanding.analyze(frame)?;
```

### Asynchronous (Non-blocking)

```rust
// Across threads
pub struct EventBus {
    pub send_event(&self, event: SystemEvent) -> Result<()>,
    pub subscribe(&self, topic: &str) -> Receiver<SystemEvent>,
}

pub enum SystemEvent {
    IntentDetected(Intent),
    BlockProduced(Block),
    UserInput(InputEvent),
    AssetLoaded(AssetId),
}
```

### Message Passing (for Network)

```rust
// Over network via libp2p
pub enum NetworkMessage {
    BlockProposal(SignedBlock),
    StateSync(Vec<Block>),
    Transaction(SignedTransaction),
    PeerInfo(PeerInfo),
}
```

---

## State Management

### Global State (Monad)

The Monad holds mutable state for all layers. State is updated atomically each tick:

```
Tick 1: Read inputs → Process → Update state → Broadcast
Tick 2: Read inputs → Process → Update state → Broadcast
...
Tick 30: Produce block with all state changes
```

### Component-Level State (Immutable for reading)

```rust
// Thread-safe read access
pub fn get_current_gaze(&self) -> GazePoint {
    self.gaze.current_gaze.clone()
}

// Mutable update (requires &mut self)
pub fn update_voice_context(&mut self, ctx: VoiceContext) {
    self.voice.context = ctx;
}
```

### Distributed State (Blockchain)

```rust
// State stored in ledger (immutable history)
pub struct Ledger {
    pub blocks: Vec<Block>,
    pub state_root: [u8; 32],
    pub accounts: HashMap<String, AccountState>,
}

// Celestia DA layer (proof of availability)
pub struct CelestiaClient {
    pub submit_blob(&self, data: &[u8]) -> Result<BlobProof>,
    pub verify_availability(&self, proof: &BlobProof) -> bool,
}
```

---

## Performance Model

```
Resource Budget (30-second block):
├─ CPU: ~80% available for AI inference
├─ Memory: ~512 MB (heap) + 256 MB (stack)
├─ GPU: Real-time AR rendering (60 FPS)
├─ Network: Gossip messages (minimal bandwidth)
├─ Storage: RocksDB local state (append-only)
└─ Audio: Continuous listening + synthesis

Latency Targets:
├─ Voice → Intent: < 500ms
├─ Gaze + Gesture → Action: < 100ms
├─ AR Rendering → Display: < 16ms (60 FPS)
├─ Block proposal → Broadcast: < 1s
└─ Oracle → Response: < 100ms
```

---

## Extension Points

### Adding a New Layer

```rust
// 1. Define component
pub struct MyComponent {
    pub state: MyState,
}

impl MyComponent {
    pub fn update(&mut self, inputs: &InputData) -> Result<OutputData>,
    pub fn on_intent(&mut self, intent: &Intent) -> Result<()>,
}

// 2. Add to Monad
pub struct Karana {
    pub my_component: Arc<MyComponent>,
}

// 3. Call in tick loop
pub fn tick(&mut self, delta_ms: u64) -> Result<()> {
    let output = self.my_component.update(inputs)?;
    // ...
}

// 4. Expose as API
pub async fn my_api(&self, params: InputData) -> Result<OutputData> {
    self.my_component.expose_functionality(params)
}
```

### Adding a New Intent Type

```rust
// 1. Define in ai_layer/intent.rs
pub enum Intent {
    MyNewIntent { param1: String, param2: u32 },
}

// 2. Add NLU pattern in voice/nlu.rs
"my command" → Intent::MyNewIntent { param1, param2 }

// 3. Add executor in ai_layer/action_executor.rs
impl ActionExecutor {
    fn execute_my_new_intent(&mut self, params) -> ExecutionResult,
}

// 4. Add manifest generator in oracle/manifest.rs
Intent::MyNewIntent { .. } → UIManifest { ar_overlays, ... }
```

### Adding a New Application

```rust
// 1. Implement Application trait
pub struct MyApp {
    pub state: AppState,
}

impl Application for MyApp {
    fn update(&mut self, delta_ms: u32, ctx: &AppContext) -> Result<()>,
    fn on_input(&mut self, event: &InputEvent) -> InputResponse,
    fn render(&self, target: &mut FrameBuffer) -> Result<()>,
}

// 2. Register in application_manager
app_manager.register("my_app", Box::new(MyApp::new()))?;

// 3. Voice command to launch
"open my app" → Intent::OpenApp { app: "my_app" }
```

---

## Concurrency Model

```
Monad Thread (Main loop)
├─ Tick every 16ms (60 FPS)
├─ Process all layer updates
├─ Produce blocks every 30s
└─ Broadcast to network

Audio Thread
├─ Capture microphone continuously
├─ VAD processing
└─ Send chunks to voice pipeline

Render Thread
├─ GPU operations
├─ AR composition
└─ Display output

Network Thread (libp2p)
├─ Gossip processing
├─ Block sync
└─ Peer discovery

Worker Pool (Tokio)
├─ Async I/O operations
├─ File operations
├─ Network requests
└─ Long-running tasks
```

**Synchronization:**
```rust
// Atomic state updates
pub struct AtomicState {
    pub inner: Arc<Mutex<State>>,
}

pub fn update(&self, func: impl FnOnce(&mut State)) {
    let mut state = self.inner.lock();
    func(&mut state);
}

// Lock-free if possible
pub struct RingBuffer<T: Clone> {
    pub push(&mut self, item: T),
    pub pop(&mut self) -> Option<T>,
}
```

---

## Security Architecture

```
┌─────────────────────────────────┐
│   Cryptographic Foundation      │
│ ├─ Ed25519 (signing)            │
│ ├─ AES-256-GCM (encryption)     │
│ ├─ PBKDF2-SHA256 (key derivation)
│ ├─ Groth16 (ZK proofs)          │
│ └─ Blake3 (hashing)             │
├─────────────────────────────────┤
│   Permission Model (RBAC)       │
│ ├─ Resource ownership           │
│ ├─ Action permissions           │
│ ├─ Time-based locks             │
│ └─ User consent verification    │
├─────────────────────────────────┤
│   Privacy (by Design)           │
│ ├─ Local-first processing       │
│ ├─ ZK intent proofs (no details)│
│ ├─ Encrypted storage            │
│ └─ No telemetry/tracking        │
└─────────────────────────────────┘
```

---

## Summary

Kāraṇa OS uses a **9-layer modular architecture** where:

1. **Hardware** provides raw sensor inputs
2. **Network** enables peer discovery & state sync
3. **Blockchain** records immutable transaction history
4. **Oracle** bridges AI decisions to blockchain operations
5. **Intelligence** fuses multimodal inputs & predicts user intent
6. **AI Engine** classifies intents & executes safely
7. **Interface** renders AR/voice/haptic outputs
8. **Applications** provide domain-specific functionality
9. **System Services** ensure reliability & security

The **Monad** orchestrator runs a 30-second tick loop, updating all layers synchronously while maintaining thread-safe state through atomic operations and message passing.
