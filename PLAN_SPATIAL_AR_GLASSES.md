# Kāraṇa OS: Spatial AR Glasses - Master Development Plan

## Vision: The All-in-One Spatial Computer

**Goal**: Replace phones and PCs with AR glasses that overlay persistent digital interfaces onto the physical world, mediated entirely by the Oracle Veil.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SPATIAL SYMBIOSIS ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────────────────┐      │
│   │   USER      │────→│   ORACLE    │────→│   SPATIAL MANIFEST      │      │
│   │ (Gaze/Voice)│     │   VEIL      │     │  (AR Tabs + Haptic)     │      │
│   └─────────────┘     └──────┬──────┘     └─────────────────────────┘      │
│                              │                                              │
│                       ┌──────▼──────┐                                      │
│                       │  ZK-PROVE   │                                      │
│                       │  + ANCHOR   │                                      │
│                       └──────┬──────┘                                      │
│                              │                                              │
│   ┌──────────────────────────┼──────────────────────────────────────┐     │
│   │                    BACKEND WEAVE                                 │     │
│   │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐│     │
│   │  │  SLAM    │  │  SWARM   │  │  CHAIN   │  │  SPATIAL STORE   ││     │
│   │  │ Anchors  │  │ Compute  │  │  State   │  │  (Tab Registry)  ││     │
│   │  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘│     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│   "Intent: Pin browser to counter" → Tab floats in kitchen                 │
│   "Intent: Pin Minecraft to table" → AR blocks anchor to furniture         │
│   "Intent: Nav to store" → AR arrows stick to real-world path              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase Overview & Timeline

| Phase | Name | Duration | Key Deliverables |
|-------|------|----------|------------------|
| 1 | AR Persistence Layer | 2 weeks | SpatialAnchor, SLAM, ZK-attestation |
| 2 | Persistent AR Tabs | 2 weeks | Browser wrapper, tab pinning, relocalization |
| 3 | Gaming & Entertainment | 2 weeks | Spatial games, passthrough AR, controllers |
| 4 | Daily Life Use Cases | 4 weeks | Productivity, communication, health, nav |
| 5 | Swarm Offloading | 2 weeks | Compute offload, battery optimization |
| 6 | Hardware Integration | 2 weeks | Rokid/XReal SDK, real sensors |

**Total**: 14 weeks to "spatial symbiosis" MVP

---

## Phase 1: AR Persistence Layer (Weeks 1-2)

### Goal
Build the foundation for persistent AR tabs that "stick" to real-world locations and survive user movement.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SPATIAL PERSISTENCE SYSTEM                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ World Coord  │───→│ SpatialAnchor│───→│  ZK-Attested    │  │
│  │ (GPS+IMU+    │    │   Registry   │    │  Anchor Store   │  │
│  │  VisualSLAM) │    │              │    │  (On-Chain)     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                     │             │
│         │            ┌──────▼──────┐             │             │
│         │            │ Relocalize  │◄────────────┘             │
│         └───────────→│   Engine    │                           │
│                      │ (On Return) │                           │
│                      └─────────────┘                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### New Files to Create

```
karana-core/src/
├── spatial/
│   ├── mod.rs              # Module exports
│   ├── anchor.rs           # SpatialAnchor struct + registry
│   ├── slam.rs             # SLAM integration (ORB-SLAM3 wrapper)
│   ├── relocalize.rs       # Re-localization engine
│   ├── world_coords.rs     # GPS + IMU + visual coordinate fusion
│   └── persistence.rs      # ZK-attested anchor storage
```

### Core Structures

```rust
// spatial/anchor.rs
/// A point in the real world where AR content is pinned
pub struct SpatialAnchor {
    pub id: AnchorId,
    pub position: WorldPosition,         // GPS + local offset
    pub orientation: Quaternion,         // Rotation
    pub visual_signature: VisualHash,    // For relocalization
    pub content_hash: Hash,              // ZK-proven content integrity
    pub created_at: u64,
    pub owner_did: String,
    pub confidence: f32,                 // 0.0-1.0 tracking quality
}

/// World position combining global (GPS) and local (SLAM) coordinates
pub struct WorldPosition {
    pub global: Option<GpsCoord>,        // Outdoor: lat/lon/alt
    pub local: LocalCoord,               // Indoor: SLAM xyz
    pub reference_frame: ReferenceFrame, // Room signature for relocalization
}

/// Reference frame for a location (room/area signature)
pub struct ReferenceFrame {
    pub signature: Vec<u8>,              // ORB features or neural embedding
    pub dimensions: (f32, f32, f32),     // Estimated room size
    pub features_count: u32,             // SLAM feature density
}
```

### Key Functions

```rust
// Create and pin an anchor
pub async fn create_anchor(
    position: WorldPosition,
    content: &ARContent,
) -> Result<SpatialAnchor>;

// Relocalize when returning to a location
pub async fn relocalize(
    current_frame: &CameraFrame,
    known_anchors: &[SpatialAnchor],
) -> Result<Vec<RelocatedAnchor>>;

// ZK-prove anchor integrity (no tampering since creation)
pub async fn prove_anchor_integrity(
    anchor: &SpatialAnchor,
) -> Result<IntegrityProof>;
```

### SLAM Integration (ORB-SLAM3 Wrapper)

```rust
// spatial/slam.rs
pub struct SlamEngine {
    /// ORB-SLAM3 instance (via C++ FFI or OpenCV wrapper)
    slam: OrbSlam3,
    /// Current map of visual features
    map: SlamMap,
    /// Tracking state
    state: TrackingState,
}

impl SlamEngine {
    /// Process a camera frame and update position
    pub fn track(&mut self, frame: &CameraFrame) -> Result<Pose>;
    
    /// Get visual signature for current location
    pub fn get_location_signature(&self) -> VisualHash;
    
    /// Try to relocalize in a known map
    pub fn relocalize(&mut self, frame: &CameraFrame) -> Option<Pose>;
    
    /// Save map for persistence
    pub fn save_map(&self, path: &Path) -> Result<()>;
    
    /// Load previously saved map
    pub fn load_map(&mut self, path: &Path) -> Result<()>;
}
```

### Integration with Oracle Veil

```rust
// oracle/veil.rs - Add spatial intent handling
impl OracleVeil {
    /// Handle spatial intents (pin, relocalize, etc.)
    pub async fn handle_spatial_intent(&mut self, intent: SpatialIntent) -> ManifestOutput {
        match intent {
            SpatialIntent::PinContent { content, location_hint } => {
                // 1. Get current world position from SLAM
                let position = self.slam.get_current_position().await?;
                
                // 2. Create spatial anchor
                let anchor = self.spatial.create_anchor(position, &content).await?;
                
                // 3. ZK-prove and store on chain
                let proof = self.zk.prove_anchor(&anchor).await?;
                self.chain.store_anchor(anchor, proof).await?;
                
                // 4. Return manifest output
                ManifestOutput {
                    whisper: format!("📌 Pinned {} to {}", content.name, location_hint),
                    haptic: HapticPattern::Success,
                    overlay: Some(AROverlay::anchor_confirmation(anchor)),
                    ..Default::default()
                }
            }
            SpatialIntent::Relocalize => {
                // Find and restore all anchors for current location
                let anchors = self.spatial.relocalize_all().await?;
                ManifestOutput {
                    whisper: format!("🔄 Restored {} pinned items", anchors.len()),
                    overlays: anchors.into_iter().map(|a| a.to_overlay()).collect(),
                    ..Default::default()
                }
            }
        }
    }
}
```

### Tests for Phase 1

```rust
#[test]
fn test_anchor_creation() {
    let position = WorldPosition::local(1.0, 0.5, 2.0);
    let content = ARContent::text("Test Note");
    let anchor = create_anchor(position, &content).await.unwrap();
    
    assert!(anchor.confidence > 0.8);
    assert_eq!(anchor.content_hash, content.hash());
}

#[test]
fn test_relocalization() {
    // Create anchor
    let anchor = create_test_anchor();
    
    // Simulate leaving and returning
    let frame = simulate_camera_frame_at(anchor.position);
    let relocated = relocalize(&frame, &[anchor.clone()]).await.unwrap();
    
    assert_eq!(relocated.len(), 1);
    assert!(relocated[0].confidence > 0.9);
}

#[test]
fn test_zk_anchor_integrity() {
    let anchor = create_test_anchor();
    let proof = prove_anchor_integrity(&anchor).await.unwrap();
    
    assert!(verify_integrity_proof(&anchor, &proof));
}
```

---

## Phase 2: Persistent AR Tabs (Weeks 3-4)

### Goal
Enable "pin browser to counter" style persistent tabs that float in space.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      AR TAB SYSTEM                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  "Intent: Pin browser to counter"                               │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐                                            │
│  │  TabManager     │───┐                                        │
│  │  - create_tab() │   │                                        │
│  │  - pin_tab()    │   │     ┌──────────────────────────────┐  │
│  │  - focus_tab()  │   └────→│      ARTab Instance          │  │
│  └─────────────────┘         │  ┌─────────────────────────┐ │  │
│                              │  │ Browser (WASM Chromium) │ │  │
│  User Interaction:           │  │ URL: nytimes.com        │ │  │
│  - Gaze to focus             │  └─────────────────────────┘ │  │
│  - Voice to scroll           │  Position: Kitchen Counter   │  │
│  - Pinch to resize           │  Size: 80cm x 50cm          │  │
│                              │  Anchor: anchor_abc123       │  │
│                              └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### New Files

```
karana-core/src/
├── ar_tabs/
│   ├── mod.rs           # Module exports
│   ├── tab.rs           # ARTab struct and lifecycle
│   ├── browser.rs       # Chromium WASM wrapper
│   ├── manager.rs       # TabManager for multi-tab handling
│   ├── interaction.rs   # Gaze/voice/gesture tab control
│   └── render.rs        # Metal/OpenGL tab rendering
```

### Core Structures

```rust
// ar_tabs/tab.rs
/// A persistent AR tab pinned in space
pub struct ARTab {
    pub id: TabId,
    pub anchor: SpatialAnchor,           // Where it's pinned
    pub content: TabContent,             // Browser, app, media, etc.
    pub size: TabSize,                   // Width x height in meters
    pub state: TabState,                 // Active, background, minimized
    pub interaction_zone: InteractionZone,
    pub last_accessed: u64,
}

/// Content types a tab can display
pub enum TabContent {
    Browser(BrowserInstance),            // Web content
    VideoPlayer(VideoStream),            // Netflix, YouTube
    CodeEditor(EditorState),             // VS Code-like
    Document(DocumentView),              // PDF, notes
    Game(GameWorld),                     // Spatial game
    Custom(CustomARApp),                 // Third-party AR apps
}

/// Tab size in real-world units
pub struct TabSize {
    pub width_m: f32,                    // Width in meters
    pub height_m: f32,                   // Height in meters
    pub depth_m: f32,                    // For 3D content
    pub fov_fraction: f32,               // What % of FOV it occupies
}

impl Default for TabSize {
    fn default() -> Self {
        Self {
            width_m: 0.8,                // 80cm wide (like a small TV)
            height_m: 0.5,               // 50cm tall
            depth_m: 0.0,                // 2D by default
            fov_fraction: 0.4,           // 40% of horizontal FOV
        }
    }
}
```

### Browser Wrapper (Chromium WASM)

```rust
// ar_tabs/browser.rs
/// WASM-based browser for AR tabs
pub struct BrowserInstance {
    /// Servo or CEF-based browser engine
    engine: BrowserEngine,
    /// Current URL
    url: String,
    /// Rendered frame buffer
    frame_buffer: FrameBuffer,
    /// Scroll position
    scroll: ScrollState,
    /// Input state (gaze cursor position)
    cursor: Option<(f32, f32)>,
}

impl BrowserInstance {
    pub fn navigate(&mut self, url: &str) -> Result<()>;
    pub fn scroll(&mut self, delta_y: f32);
    pub fn click(&mut self, x: f32, y: f32);
    pub fn type_text(&mut self, text: &str);
    pub fn render_frame(&mut self) -> &FrameBuffer;
    
    /// Voice-controlled scrolling
    pub fn voice_scroll(&mut self, direction: ScrollDirection, amount: ScrollAmount) {
        match (direction, amount) {
            (ScrollDirection::Down, ScrollAmount::Page) => self.scroll(1.0),
            (ScrollDirection::Down, ScrollAmount::Half) => self.scroll(0.5),
            (ScrollDirection::Up, ScrollAmount::Page) => self.scroll(-1.0),
            // ...
        }
    }
}
```

### Tab Manager

```rust
// ar_tabs/manager.rs
pub struct TabManager {
    /// All open tabs
    tabs: HashMap<TabId, ARTab>,
    /// Currently focused tab
    focused: Option<TabId>,
    /// Tabs visible in current FOV
    visible: Vec<TabId>,
    /// Tab layout engine
    layout: TabLayout,
}

impl TabManager {
    /// Create and pin a new tab
    pub async fn pin_tab(
        &mut self,
        content: TabContent,
        location_hint: &str,
        anchor: SpatialAnchor,
    ) -> Result<TabId>;
    
    /// Focus a tab (brings to front, enables interaction)
    pub fn focus(&mut self, tab_id: TabId);
    
    /// Minimize a tab (shrinks to icon)
    pub fn minimize(&mut self, tab_id: TabId);
    
    /// Close a tab
    pub fn close(&mut self, tab_id: TabId);
    
    /// Get all tabs in current FOV
    pub fn get_visible_tabs(&self, fov: &FieldOfView) -> Vec<&ARTab>;
    
    /// Update tabs after relocalization
    pub fn on_relocalize(&mut self, anchors: &[RelocatedAnchor]);
}
```

### Tab Interaction (Gaze/Voice/Gesture)

```rust
// ar_tabs/interaction.rs
pub struct TabInteraction {
    /// Gaze tracker
    gaze: GazeTracker,
    /// Current gaze target
    gaze_target: Option<TabId>,
    /// Dwell timer for selection
    dwell_timer: DwellTimer,
}

impl TabInteraction {
    /// Process gaze update
    pub fn on_gaze(&mut self, point: GazePoint, tabs: &[ARTab]) -> Option<InteractionEvent> {
        // Find which tab is gazed at
        let target = tabs.iter().find(|t| t.interaction_zone.contains(point));
        
        if let Some(tab) = target {
            self.gaze_target = Some(tab.id);
            
            // Start dwell timer
            if self.dwell_timer.check(500) {
                return Some(InteractionEvent::DwellSelect(tab.id));
            }
            
            // Update cursor position within tab
            let local = tab.interaction_zone.to_local(point);
            return Some(InteractionEvent::CursorMove(tab.id, local));
        }
        
        self.gaze_target = None;
        None
    }
    
    /// Process voice command for tabs
    pub fn on_voice(&self, cmd: &str, focused_tab: Option<TabId>) -> Option<InteractionEvent> {
        match cmd.to_lowercase().as_str() {
            "scroll down" => Some(InteractionEvent::Scroll(focused_tab?, ScrollDirection::Down)),
            "scroll up" => Some(InteractionEvent::Scroll(focused_tab?, ScrollDirection::Up)),
            "click" => Some(InteractionEvent::Click(focused_tab?)),
            "close tab" => Some(InteractionEvent::Close(focused_tab?)),
            "minimize" => Some(InteractionEvent::Minimize(focused_tab?)),
            _ => None,
        }
    }
    
    /// Process gesture (pinch to resize, etc.)
    pub fn on_gesture(&self, gesture: GestureType, focused_tab: Option<TabId>) -> Option<InteractionEvent> {
        match gesture {
            GestureType::Pinch { scale } => Some(InteractionEvent::Resize(focused_tab?, scale)),
            GestureType::Swipe { direction } => Some(InteractionEvent::SwipeNav(focused_tab?, direction)),
            _ => None,
        }
    }
}
```

### Oracle Integration for Tabs

```rust
// Add to oracle/veil.rs
impl OracleVeil {
    pub async fn handle_tab_intent(&mut self, intent: TabIntent) -> ManifestOutput {
        match intent {
            TabIntent::PinBrowser { url, location } => {
                // 1. Create browser instance
                let browser = BrowserInstance::new(&url).await?;
                
                // 2. Get current position for anchor
                let position = self.spatial.get_position().await?;
                let anchor = self.spatial.create_anchor(position, &ARContent::Tab).await?;
                
                // 3. Create and register tab
                let tab_id = self.tabs.pin_tab(
                    TabContent::Browser(browser),
                    &location,
                    anchor,
                ).await?;
                
                ManifestOutput {
                    whisper: format!("📌 Browser pinned to {}", location),
                    haptic: HapticPattern::Success,
                    overlay: Some(self.tabs.get_tab(tab_id)?.to_overlay()),
                    ..Default::default()
                }
            }
            TabIntent::OpenVideo { url, location } => {
                // Similar to browser but with video player
                let video = VideoStream::new(&url).await?;
                let position = self.spatial.get_position().await?;
                let anchor = self.spatial.create_anchor(position, &ARContent::Tab).await?;
                
                let tab_id = self.tabs.pin_tab(
                    TabContent::VideoPlayer(video),
                    &location,
                    anchor,
                ).await?;
                
                ManifestOutput {
                    whisper: format!("🎬 Video pinned to {}", location),
                    haptic: HapticPattern::Success,
                    overlay: Some(self.tabs.get_tab(tab_id)?.to_overlay()),
                    ..Default::default()
                }
            }
            TabIntent::FocusTab { tab_id } => {
                self.tabs.focus(tab_id);
                ManifestOutput {
                    whisper: "Tab focused".to_string(),
                    haptic: HapticPattern::Tap,
                    ..Default::default()
                }
            }
        }
    }
}
```

---

## Phase 3: Gaming & Entertainment (Weeks 5-6)

### Goal
Enable "Pin Minecraft to table" style spatial games with persistent worlds.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SPATIAL GAMING SYSTEM                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  "Intent: Pin Minecraft to table"                               │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                  GameWorld                                   ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ ││
│  │  │ World State │  │  Physics    │  │  Passthrough AR     │ ││
│  │  │ (Blocks,    │  │  Engine     │  │  (Real table +      │ ││
│  │  │  Entities)  │  │  (Collisions│  │   virtual blocks)   │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ ││
│  │                                                              ││
│  │  Anchored to: Table surface                                  ││
│  │  Size: 1m x 0.6m (table dimensions)                         ││
│  │  Persistence: ZK-proven world hash                          ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  Input Methods:                                                  │
│  - Controllers: Physical game controllers                       │
│  - Gestures: Pinch = place block, Swipe = rotate view          │
│  - Voice: "Build wall", "Spawn enemy"                          │
│  - Gaze: Look at block to select                               │
└─────────────────────────────────────────────────────────────────┘
```

### New Files

```
karana-core/src/
├── gaming/
│   ├── mod.rs               # Module exports
│   ├── world.rs             # GameWorld struct
│   ├── physics.rs           # Physics engine wrapper
│   ├── passthrough.rs       # AR passthrough integration
│   ├── controllers.rs       # Controller input handling
│   ├── persistence.rs       # ZK-proven game state
│   └── multiplayer.rs       # P2P game sync via swarm
```

### Core Structures

```rust
// gaming/world.rs
pub struct GameWorld {
    pub id: WorldId,
    pub anchor: SpatialAnchor,           // Table/floor anchor
    pub bounds: WorldBounds,             // Play area dimensions
    pub state: WorldState,               // Blocks, entities, etc.
    pub physics: PhysicsEngine,          // Collision detection
    pub render_mode: RenderMode,         // Passthrough AR settings
}

/// Game world state (voxel example like Minecraft)
pub struct WorldState {
    pub blocks: HashMap<BlockCoord, BlockType>,
    pub entities: Vec<Entity>,
    pub player: PlayerState,
    pub last_modified: u64,
    pub state_hash: Hash,                // For ZK integrity
}

/// Passthrough AR settings
pub struct RenderMode {
    pub passthrough_opacity: f32,        // 0.0 = full VR, 1.0 = full AR
    pub occlusion_enabled: bool,         // Real objects occlude virtual
    pub shadow_casting: bool,            // Virtual objects cast shadows
    pub depth_mesh: bool,                // Use ToF for accurate depth
}
```

### Passthrough AR Integration

```rust
// gaming/passthrough.rs
pub struct PassthroughAR {
    /// Camera feed
    camera: CameraFeed,
    /// Depth sensor (ToF)
    depth: DepthSensor,
    /// Surface detection
    surfaces: Vec<DetectedSurface>,
    /// Occlusion mesh
    occlusion_mesh: OcclusionMesh,
}

impl PassthroughAR {
    /// Detect surfaces (tables, floors, walls)
    pub async fn detect_surfaces(&mut self) -> Vec<DetectedSurface>;
    
    /// Get depth at a point (for occlusion)
    pub fn get_depth(&self, x: f32, y: f32) -> f32;
    
    /// Update occlusion mesh from depth
    pub fn update_occlusion(&mut self);
    
    /// Render game world with passthrough
    pub fn render(
        &self,
        world: &GameWorld,
        camera_pose: &Pose,
    ) -> CompositeFrame;
}
```

### Game Types to Support

```rust
pub enum GameType {
    /// Voxel building (Minecraft-style)
    VoxelBuilder {
        grid_size: f32,      // Block size in meters
        max_height: u32,     // Max build height
    },
    
    /// Board games
    BoardGame {
        board: BoardConfig,  // Chess, checkers, etc.
        pieces: Vec<Piece>,
    },
    
    /// Racing/vehicles
    Racing {
        track: TrackConfig,
        vehicles: Vec<Vehicle>,
    },
    
    /// Strategy/tower defense
    Strategy {
        map: MapConfig,
        units: Vec<Unit>,
    },
    
    /// Puzzle/physics
    Puzzle {
        elements: Vec<PuzzleElement>,
        physics: PhysicsConfig,
    },
}
```

### Multiplayer via Swarm

```rust
// gaming/multiplayer.rs
pub struct MultiplayerSession {
    /// Session ID
    session_id: SessionId,
    /// Connected players
    players: HashMap<PeerId, PlayerInfo>,
    /// Swarm connection
    swarm: SwarmHandle,
    /// State sync protocol
    sync: StateSyncProtocol,
}

impl MultiplayerSession {
    /// Broadcast local action to all players
    pub async fn broadcast_action(&self, action: GameAction) -> Result<()> {
        let msg = SyncMessage::Action {
            player: self.local_player,
            action,
            timestamp: now(),
        };
        self.swarm.gossip(msg).await
    }
    
    /// Handle incoming sync message
    pub async fn on_sync_message(&mut self, msg: SyncMessage) -> Option<WorldDelta> {
        match msg {
            SyncMessage::Action { player, action, .. } => {
                // Apply action to world
                Some(self.world.apply_action(player, action))
            }
            SyncMessage::StateSync { state_hash, delta } => {
                // Reconcile state
                self.reconcile_state(state_hash, delta)
            }
        }
    }
}
```

---

## Phase 4: Daily Life Use Cases (Weeks 7-10)

### 4.1 Productivity (Week 7)

#### AR Code Editor
```rust
// productivity/code_editor.rs
pub struct ARCodeEditor {
    pub anchor: SpatialAnchor,           // Pinned to desk
    pub files: Vec<OpenFile>,
    pub active_file: Option<FileId>,
    pub cursor: CursorPosition,
    pub selection: Option<Selection>,
    pub language_server: LSPClient,      // For completions
}

impl ARCodeEditor {
    /// Voice-controlled editing
    pub fn voice_edit(&mut self, cmd: &str) -> Result<EditAction> {
        match parse_voice_edit(cmd) {
            VoiceEdit::GoToLine(n) => self.goto_line(n),
            VoiceEdit::SelectFunction(name) => self.select_function(&name),
            VoiceEdit::InsertSnippet(snippet) => self.insert(&snippet),
            VoiceEdit::Undo => self.undo(),
            VoiceEdit::Redo => self.redo(),
        }
    }
    
    /// Gaze-based line selection
    pub fn gaze_select(&mut self, gaze: GazePoint) -> Option<LineRange> {
        let line = self.screen_to_line(gaze)?;
        self.select_line(line)
    }
}
```

#### Meeting Notes
```rust
// productivity/meeting.rs
pub struct MeetingAssistant {
    pub anchor: SpatialAnchor,           // Pinned to meeting room
    pub transcript: Vec<TranscriptSegment>,
    pub action_items: Vec<ActionItem>,
    pub participants: Vec<Participant>,
    pub recording: Option<AudioStream>,
}

impl MeetingAssistant {
    /// Real-time transcription
    pub async fn transcribe(&mut self, audio: &[f32]) -> Result<TranscriptSegment>;
    
    /// Extract action items via AI
    pub async fn extract_actions(&self) -> Vec<ActionItem>;
    
    /// Generate meeting summary
    pub async fn summarize(&self) -> MeetingSummary;
    
    /// ZK-sign transcript (for privacy)
    pub async fn sign_transcript(&self) -> ZkTranscriptProof;
}
```

### 4.2 Communication (Week 8)

#### AR Video Calls
```rust
// communication/video_call.rs
pub struct ARVideoCall {
    pub anchor: SpatialAnchor,           // Where participant appears
    pub peer_id: PeerId,
    pub video_stream: VideoStream,
    pub audio_stream: AudioStream,
    pub encryption: E2EEncryption,       // ZK-encrypted
}

impl ARVideoCall {
    /// Initiate call (swarm-based, no central server)
    pub async fn dial(peer_did: &str) -> Result<Self>;
    
    /// Show caller as floating AR window
    pub fn render(&self, pose: &Pose) -> AROverlay;
    
    /// ZK-prove call metadata (without revealing content)
    pub async fn prove_call_occurred(&self) -> CallProof;
}
```

#### Persistent Chat Bubbles
```rust
// communication/spatial_chat.rs
pub struct SpatialChatRoom {
    pub anchor: SpatialAnchor,           // "Family group in living room"
    pub group_id: GroupId,
    pub messages: Vec<ChatMessage>,
    pub participants: Vec<Did>,
}

impl SpatialChatRoom {
    /// Pin chat to location
    pub async fn pin(&self, location: &str) -> Result<()>;
    
    /// Show as floating bubbles
    pub fn render(&self) -> Vec<AROverlay>;
    
    /// Voice reply
    pub async fn voice_reply(&mut self, audio: &[f32]) -> Result<()>;
}
```

### 4.3 Health & Fitness (Week 9)

#### Fitness Tracker AR
```rust
// health/fitness.rs
pub struct FitnessTracker {
    /// Route anchors (path taken)
    pub route: Vec<SpatialAnchor>,
    pub stats: RunStats,
    pub heart_rate: HeartRateMonitor,
    pub hydration_alerts: Vec<HydrationAlert>,
}

impl FitnessTracker {
    /// AR pace overlay stuck to path
    pub fn render_pace_overlay(&self, current_pos: &WorldPosition) -> AROverlay {
        AROverlay::Text {
            content: format!("🏃 {}:{:02}/km", self.stats.pace_min, self.stats.pace_sec),
            position: current_pos.offset(0.0, 1.5, 0.0), // Above head
            style: WhisperStyle::Urgent,
        }
    }
    
    /// Haptic hydration reminder
    pub fn check_hydration(&self, distance_km: f32) -> Option<HapticPattern> {
        if distance_km > 2.0 && !self.stats.hydrated_recently {
            Some(HapticPattern::DoubleVibe)
        } else {
            None
        }
    }
    
    /// ZK-attest workout (share "Proved 10K" without raw data)
    pub async fn prove_workout(&self) -> WorkoutProof;
}
```

### 4.4 Navigation & Shopping (Week 10)

#### AR Navigation
```rust
// navigation/ar_nav.rs
pub struct ARNavigation {
    /// Route as spatial anchors
    pub route: Vec<RouteAnchor>,
    /// Current progress
    pub current_step: usize,
    /// ETA
    pub eta_minutes: u32,
    /// Turn-by-turn instructions
    pub instructions: Vec<NavInstruction>,
}

impl ARNavigation {
    /// Render AR arrows on path
    pub fn render_arrows(&self, current_pos: &WorldPosition) -> Vec<AROverlay> {
        self.route[self.current_step..]
            .iter()
            .take(3) // Show next 3 waypoints
            .map(|anchor| AROverlay::Arrow {
                from: current_pos.clone(),
                to: anchor.position.clone(),
                color: if anchor.is_turn { "yellow" } else { "green" },
            })
            .collect()
    }
    
    /// Turn alert with haptic
    pub fn check_turn(&self, pos: &WorldPosition) -> Option<TurnAlert> {
        let next = &self.route[self.current_step];
        if pos.distance_to(&next.position) < 10.0 && next.is_turn {
            Some(TurnAlert {
                direction: next.turn_direction.clone(),
                haptic: HapticPattern::from_direction(&next.turn_direction),
            })
        } else {
            None
        }
    }
}
```

#### Shopping AR
```rust
// navigation/shopping.rs
pub struct ShoppingAR {
    /// Store map (shared via swarm)
    pub store_map: StoreMap,
    /// Shopping list
    pub list: Vec<ShoppingItem>,
    /// Product scanner
    pub scanner: ProductScanner,
}

impl ShoppingAR {
    /// Scan product and show info overlay
    pub async fn scan_product(&self, frame: &CameraFrame) -> Option<ProductOverlay> {
        let barcode = self.scanner.detect(frame)?;
        let product = self.lookup_product(&barcode).await?;
        
        Some(ProductOverlay {
            name: product.name,
            price: product.price,
            reviews: self.get_dao_reviews(&barcode).await, // Decentralized reviews
            position: frame.center_point(),
        })
    }
    
    /// Navigate to item in store
    pub async fn find_item(&self, item: &str) -> Option<ARNavigation> {
        let location = self.store_map.find(item)?;
        ARNavigation::route_to(location)
    }
}
```

---

## Phase 5: Swarm Offloading (Weeks 11-12)

### Goal
Offload heavy computation to the swarm network to save battery.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SWARM OFFLOAD SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Local (Glasses)              Swarm Network                     │
│  ┌─────────────┐              ┌─────────────────────────────┐  │
│  │ Light Tasks │              │  Heavy Computation          │  │
│  │ - UI render │   ◄─────►   │  - AI inference             │  │
│  │ - Sensors   │   libp2p    │  - ZK proving               │  │
│  │ - Audio     │              │  - Video encoding           │  │
│  └─────────────┘              │  - SLAM map building        │  │
│                               └─────────────────────────────┘  │
│                                                                  │
│  Offload Decision:                                              │
│  - Battery < 30%? Offload more                                  │
│  - Low latency task? Keep local                                │
│  - Heavy AI? Always offload                                     │
└─────────────────────────────────────────────────────────────────┘
```

### New Files

```
karana-core/src/
├── offload/
│   ├── mod.rs               # Module exports
│   ├── scheduler.rs         # Offload decision engine
│   ├── tasks.rs             # Task types and serialization
│   ├── executor.rs          # Local vs remote execution
│   └── billing.rs           # KARA token incentives
```

### Core Structures

```rust
// offload/scheduler.rs
pub struct OffloadScheduler {
    /// Current battery level
    battery: BatteryState,
    /// Network quality
    network: NetworkQuality,
    /// Task queue
    queue: VecDeque<OffloadTask>,
    /// Swarm handle
    swarm: SwarmHandle,
}

impl OffloadScheduler {
    /// Decide whether to offload a task
    pub fn should_offload(&self, task: &OffloadTask) -> OffloadDecision {
        // Always offload heavy AI
        if task.task_type == TaskType::AIInference && task.model_size_mb > 100 {
            return OffloadDecision::Remote;
        }
        
        // Offload ZK proving if battery low
        if task.task_type == TaskType::ZkProve && self.battery.percentage < 30.0 {
            return OffloadDecision::Remote;
        }
        
        // Keep latency-sensitive tasks local
        if task.latency_requirement_ms < 50 {
            return OffloadDecision::Local;
        }
        
        // Default: based on battery
        if self.battery.percentage < 20.0 {
            OffloadDecision::Remote
        } else {
            OffloadDecision::Local
        }
    }
    
    /// Execute task (locally or remotely)
    pub async fn execute(&mut self, task: OffloadTask) -> Result<TaskResult> {
        match self.should_offload(&task) {
            OffloadDecision::Local => self.execute_local(task).await,
            OffloadDecision::Remote => self.execute_remote(task).await,
        }
    }
    
    /// Execute on remote swarm node
    async fn execute_remote(&self, task: OffloadTask) -> Result<TaskResult> {
        // Find available compute node
        let node = self.swarm.find_compute_node(task.requirements).await?;
        
        // Send task
        let result = node.execute(task).await?;
        
        // Pay in KARA tokens
        self.pay_for_compute(node.peer_id, task.compute_cost).await?;
        
        Ok(result)
    }
}
```

### Task Types

```rust
// offload/tasks.rs
pub enum TaskType {
    /// AI model inference
    AIInference {
        model: ModelId,
        input: Vec<u8>,
    },
    
    /// ZK proof generation
    ZkProve {
        circuit: CircuitId,
        witness: Vec<u8>,
    },
    
    /// Video encoding/transcoding
    VideoEncode {
        frames: Vec<Frame>,
        codec: Codec,
    },
    
    /// SLAM map building
    SlamMapping {
        keyframes: Vec<KeyFrame>,
    },
    
    /// Heavy rendering
    Render {
        scene: SceneData,
        quality: RenderQuality,
    },
}
```

### Billing (KARA Token Incentives)

```rust
// offload/billing.rs
pub struct ComputeBilling {
    /// Local wallet
    wallet: KaranaWallet,
    /// Price oracle
    price_oracle: PriceOracle,
}

impl ComputeBilling {
    /// Calculate cost for a task
    pub fn calculate_cost(&self, task: &OffloadTask) -> u64 {
        let base_cost = match task.task_type {
            TaskType::AIInference { .. } => 10,   // 10 KARA
            TaskType::ZkProve { .. } => 50,       // 50 KARA (expensive)
            TaskType::VideoEncode { .. } => 20,  // 20 KARA
            TaskType::SlamMapping { .. } => 30,  // 30 KARA
            TaskType::Render { .. } => 15,       // 15 KARA
        };
        
        // Adjust for network congestion
        let congestion_multiplier = self.price_oracle.get_congestion_factor();
        (base_cost as f32 * congestion_multiplier) as u64
    }
    
    /// Pay a compute provider
    pub async fn pay(&self, peer: PeerId, amount: u64) -> Result<TxHash> {
        self.wallet.transfer(&peer.to_did(), amount, Some("compute")).await
    }
    
    /// Earn by providing compute
    pub async fn receive_payment(&self, from: PeerId, amount: u64) -> Result<()> {
        // Verified by swarm consensus
        log::info!("Received {} KARA for compute from {}", amount, from);
        Ok(())
    }
}
```

---

## Phase 6: Hardware Integration (Weeks 13-14)

### Rokid/XReal SDK Integration

```rust
// hardware/glasses_sdk.rs
pub enum GlassesSDK {
    Rokid(RokidSDK),
    XReal(XRealSDK),
    Generic(GenericARSDK),
}

impl GlassesSDK {
    /// Initialize glasses connection
    pub async fn connect() -> Result<Self>;
    
    /// Get camera feed
    pub fn get_camera(&self) -> &CameraFeed;
    
    /// Get IMU data
    pub fn get_imu(&self) -> &IMUStream;
    
    /// Get eye tracking
    pub fn get_eye_tracker(&self) -> Option<&EyeTracker>;
    
    /// Render to display
    pub fn render(&mut self, frame: &CompositeFrame);
    
    /// Trigger haptic
    pub fn haptic(&self, pattern: &HapticPattern);
}
```

### NPU Acceleration

```rust
// hardware/npu.rs
pub struct NPUAccelerator {
    /// NPU device handle
    device: NPUDevice,
    /// Loaded models
    models: HashMap<ModelId, LoadedModel>,
}

impl NPUAccelerator {
    /// Run inference on NPU
    pub async fn infer(&self, model: ModelId, input: &Tensor) -> Result<Tensor>;
    
    /// Check if model fits in NPU memory
    pub fn can_load(&self, model: &ModelInfo) -> bool;
    
    /// Offload to CPU if NPU busy
    pub async fn infer_with_fallback(&self, model: ModelId, input: &Tensor) -> Result<Tensor>;
}
```

---

## Testing Strategy

### QEMU Testing (from Phase 4 work)

```rust
// Use QemuTestHarness from simulator/qemu.rs
#[tokio::test]
async fn test_spatial_ar_e2e() {
    let mut harness = QemuTestHarness::new(SwarmConfig {
        validators: 1,
        glasses: 3,
        ..Default::default()
    });
    
    harness.setup().unwrap();
    
    // Test 1: Pin browser tab
    let result = harness.adb.shell(1, "intent pin_browser https://nytimes.com kitchen").unwrap();
    assert!(result.contains("pinned"));
    
    // Test 2: Relocalize on return
    let result = harness.adb.shell(1, "relocalize").unwrap();
    assert!(result.contains("restored"));
    
    // Test 3: Multiplayer game sync
    harness.adb.shell(1, "intent pin_game minecraft table").unwrap();
    harness.adb.shell(2, "join_game instance_1").unwrap();
    // Verify both see same world state
    
    harness.teardown().unwrap();
}
```

### Webcam Simulation for Gaze/SLAM

```rust
#[test]
fn test_slam_with_simulated_camera() {
    let mut webcam = WebcamProxy::new(PathBuf::from("/dev/video0"));
    webcam.start_streaming().unwrap();
    
    let mut slam = SlamEngine::new();
    
    // Feed synthetic frames
    for i in 0..100 {
        let frame = webcam.get_frame().unwrap();
        let pose = slam.track(&frame.into()).unwrap();
        
        // Verify tracking is working
        if i > 10 {
            assert!(pose.confidence > 0.5);
        }
    }
}
```

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tab Pin Success Rate | 95% | TBD |
| Relocalization Accuracy | 99% | TBD |
| Tab Load Time | < 500ms | TBD |
| Game Frame Rate | 60fps | TBD |
| Voice Command Accuracy | 90% | TBD |
| Battery Life (mixed AR) | 8+ hours | TBD |
| Swarm Offload Latency | < 200ms | TBD |
| Daily Tasks Replaceable | 80% | TBD |

---

## File Summary: What to Create

### Phase 1 (AR Persistence)
- `src/spatial/mod.rs`
- `src/spatial/anchor.rs`
- `src/spatial/slam.rs`
- `src/spatial/relocalize.rs`
- `src/spatial/world_coords.rs`
- `src/spatial/persistence.rs`

### Phase 2 (AR Tabs)
- `src/ar_tabs/mod.rs`
- `src/ar_tabs/tab.rs`
- `src/ar_tabs/browser.rs`
- `src/ar_tabs/manager.rs`
- `src/ar_tabs/interaction.rs`
- `src/ar_tabs/render.rs`

### Phase 3 (Gaming)
- `src/gaming/mod.rs`
- `src/gaming/world.rs`
- `src/gaming/physics.rs`
- `src/gaming/passthrough.rs`
- `src/gaming/controllers.rs`
- `src/gaming/persistence.rs`
- `src/gaming/multiplayer.rs`

### Phase 4 (Daily Life)
- `src/productivity/mod.rs` (code_editor, meeting)
- `src/communication/mod.rs` (video_call, spatial_chat)
- `src/health/mod.rs` (fitness, vitals)
- `src/navigation/mod.rs` (ar_nav, shopping)

### Phase 5 (Offload)
- `src/offload/mod.rs`
- `src/offload/scheduler.rs`
- `src/offload/tasks.rs`
- `src/offload/executor.rs`
- `src/offload/billing.rs`

### Phase 6 (Hardware)
- `src/hardware/glasses_sdk.rs`
- `src/hardware/npu.rs`

---

## Immediate Next Steps

1. ✅ Review and approve this plan
2. 🔲 Create `src/spatial/mod.rs` and `anchor.rs` - Core persistence
3. 🔲 Create `src/spatial/slam.rs` - ORB-SLAM3 wrapper
4. 🔲 Add `SpatialIntent` handling to `oracle/veil.rs`
5. 🔲 Create `src/ar_tabs/tab.rs` - Basic tab structure
6. 🔲 Test with QEMU harness

---

*Spatial AR Glasses Plan v1.0 - December 3, 2025*
*Target: Phone/PC replacement for 80% of daily tasks by Q2 2026*
