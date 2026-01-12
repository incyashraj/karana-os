# Kāraṇa OS: AI-Driven System Control Architecture

## Vision
Make the entire Kāraṇa OS completely operable through natural language AI. Users should be able to control ALL 9 layers + cross-cutting systems through simple conversation with the Oracle AI.

## Current System Architecture (9 Layers)

```
Layer 9: System Services      → OTA, Security, Diagnostics, Recovery
Layer 8: Applications         → Timer, Navigation, Social, Settings, Wellness
Layer 7: Interface            → Voice, HUD, Gestures, Gaze, AR Rendering
Layer 6: AI Engine            → NLU, Dialogue, Reasoning, Action Execution
Layer 5: Intelligence         → Multimodal Fusion, Scene Understanding, Memory
Layer 4: Oracle Bridge        → Intent Processing, Manifest Rendering, ZK
Layer 3: Blockchain           → Chain, Ledger, Governance, Wallet, Celestia DA
Layer 2: P2P Network          → libp2p, mDNS, Gossip, Sync
Layer 1: Hardware             → Camera, Sensors, Audio, Display, Power
```

## AI Control Capabilities Required

### Layer 1: Hardware Control
**User Intent**: "show me what the camera sees", "adjust brightness", "check battery"
**AI Actions**:
- Camera: capture, record, adjust exposure, switch modes
- Sensors: read IMU, GPS, temperature, proximity
- Display: brightness, color correction, refresh rate
- Audio: volume, mic sensitivity, noise cancellation
- Power: battery status, thermal state, power profiles

### Layer 2: Network Control
**User Intent**: "check network", "connect to peer", "sync blockchain"
**AI Actions**:
- Peer discovery and connection
- Message broadcasting
- Block synchronization
- Network diagnostics

### Layer 3: Blockchain Control  
**User Intent**: "send 50 KARA to Mom", "check balance", "view transactions"
**AI Actions**:
- Wallet creation/restoration
- Transaction signing and submission
- Balance queries
- Transaction history
- Governance voting

### Layer 4: Oracle Bridge (Self-Awareness)
**User Intent**: "explain what you're thinking", "show me the proof"
**AI Actions**:
- Intent classification
- ZK proof generation
- Manifest creation
- Self-introspection

### Layer 5: Intelligence
**User Intent**: "what am I looking at", "predict what I need"
**AI Actions**:
- Scene understanding
- Object detection
- Multimodal fusion (voice + vision + gesture)
- Context prediction

### Layer 6: AI Engine (Already Implemented)
**User Intent**: Natural conversation
**AI Actions**:
- NLU processing
- Dialogue management
- Reasoning
- Action execution

### Layer 7: Interface Control
**User Intent**: "show AR overlay", "start voice command", "track my hand"
**AI Actions**:
- HUD element toggling
- Voice mode switching
- Gesture recognition on/off
- Gaze tracking activation
- AR rendering controls

### Layer 8: Application Control
**User Intent**: "set timer 5 minutes", "navigate home", "check wellness stats"
**AI Actions**:
- Timer/alarm management
- Navigation routing
- Social contacts
- Settings modification
- Wellness monitoring
- Android app launching

### Layer 9: System Services Control
**User Intent**: "check for updates", "run diagnostics", "enable privacy mode"
**AI Actions**:
- OTA update triggering
- Security mode switching
- Diagnostic tests
- Recovery operations
- Log viewing

## Implementation Strategy

### Phase 1: Comprehensive System State Manager ✅
**File**: `simulator-ui/services/systemState.ts`
**Purpose**: Single source of truth for ENTIRE system state
**Includes**:
```typescript
- Hardware state (camera, sensors, battery, display)
- Network state (peers, sync status, connections)
- Blockchain state (wallet, balance, transactions, blocks)
- AR state (anchors, tabs, gestures, gaze)
- Application state (timers, navigation, settings, wellness)
- System services state (OTA, diagnostics, security)
```

### Phase 2: Enhanced Oracle AI ✅
**File**: `simulator-ui/services/oracleAI.ts`
**Enhancements**:
- Detect intents for ALL 9 layers
- Generate execution plans across multiple layers
- Coordinate complex multi-step operations
- Provide real-time feedback and status

### Phase 3: Backend API Expansion
**File**: `karana-core/src/api/handlers.rs`
**New Endpoints**:
```rust
// Hardware
POST /api/hardware/camera/capture
POST /api/hardware/camera/record
GET  /api/hardware/sensors/imu
GET  /api/hardware/battery/status
POST /api/hardware/display/brightness

// Network
GET  /api/network/peers
POST /api/network/connect
GET  /api/network/sync/status

// AR/Spatial
POST /api/ar/anchor/create
GET  /api/ar/anchors
POST /api/ar/tab/open
GET  /api/ar/tabs
POST /api/ar/gesture/enable

// Applications
POST /api/timer/create
GET  /api/timer/list
POST /api/navigation/route
GET  /api/wellness/stats
POST /api/settings/update

// System Services
POST /api/system/ota/check
POST /api/system/diagnostics/run
POST /api/system/security/mode
GET  /api/system/logs
```

### Phase 4: Unified Execution Layer
**File**: `simulator-ui/App.tsx`
**Integration**:
- Connect Oracle AI to SystemState
- Map AI intents to actual system operations
- Coordinate cross-layer operations
- Provide unified feedback

## Example User Flows

### Flow 1: Multi-Layer Operation
```
User: "take a photo and send it to Mom with 10 KARA"

AI Process:
1. Layer 1 (Hardware) → Capture camera frame
2. Layer 5 (Intelligence) → Analyze image quality
3. Layer 8 (Apps) → Compress and prepare for sending
4. Layer 3 (Blockchain) → Create transaction with attachment
5. Layer 4 (Oracle) → Generate ZK proof
6. Layer 2 (Network) → Broadcast to peers
7. Layer 7 (Interface) → Show HUD confirmation
```

### Flow 2: System Optimization
```
User: "battery is low, optimize everything"

AI Process:
1. Layer 1 (Hardware) → Check battery level (18%)
2. Layer 9 (System) → Switch to power-saving mode
3. Layer 7 (Interface) → Reduce display brightness
4. Layer 5 (Intelligence) → Disable background AI
5. Layer 3 (Blockchain) → Switch to minimal mode
6. Layer 8 (Apps) → Pause non-critical timers
7. Layer 1 (Hardware) → Enable thermal throttling
```

### Flow 3: Privacy Mode
```
User: "enable maximum privacy"

AI Process:
1. Layer 9 (System) → Activate ephemeral mode
2. Layer 3 (Blockchain) → Enable ZK for all transactions
3. Layer 1 (Hardware) → Disable camera/mic when idle
4. Layer 2 (Network) → Route through Tor
5. Layer 8 (Apps) → Set auto-delete timers
6. Layer 7 (Interface) → Hide sensitive HUD elements
```

### Flow 4: AR Workspace Setup
```
User: "set up my workspace"

AI Process:
1. Layer 1 (Hardware) → Read GPS location
2. Layer 5 (Intelligence) → Detect room boundaries via SLAM
3. AR Layer → Create spatial anchors at key points
4. AR Layer → Open browser tab (left), code editor (center), terminal (right)
5. Layer 7 (Interface) → Configure gesture controls for tabs
6. Layer 8 (Apps) → Load workspace settings from profile
7. SystemState → Save workspace configuration
```

## Architecture Diagram: AI-Driven System

```
┌─────────────────────────────────────────────────────────────┐
│                        USER                                  │
│              "battery is low, optimize"                      │
└────────────────────────┬────────────────────────────────────┘
                         │ Natural Language
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   ORACLE AI (Layer 6)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. Detect Intent: SYSTEM_OPTIMIZATION                │  │
│  │ 2. Query SystemState: battery=18%, temp=45°C         │  │
│  │ 3. Generate Plan: 7-step optimization                │  │
│  │ 4. Execute Across Layers                             │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────┬────────────────────────────────────────────────┘
             │ Orchestrated Actions
             ▼
┌─────────────────────────────────────────────────────────────┐
│                  SYSTEM STATE MANAGER                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Complete System State (All 9 Layers)                 │  │
│  │ - Hardware: camera, sensors, battery, display        │  │
│  │ - Network: peers, sync, connections                  │  │
│  │ - Blockchain: wallet, balance, transactions          │  │
│  │ - AR: anchors, tabs, gestures                        │  │
│  │ - Apps: timers, navigation, settings                 │  │
│  │ - System: OTA, diagnostics, security                 │  │
│  └──────────────────────────────────────────────────────┘  │
└─┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬─┘
  │      │      │      │      │      │      │      │      │
  ▼      ▼      ▼      ▼      ▼      ▼      ▼      ▼      ▼
┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐
│L1 │  │L2 │  │L3 │  │L4 │  │L5 │  │L6 │  │L7 │  │L8 │  │L9 │
│HW │  │NET│  │BC │  │ORC│  │INT│  │AI │  │UI │  │APP│  │SYS│
└───┘  └───┘  └───┘  └───┘  └───┘  └───┘  └───┘  └───┘  └───┘
  │      │      │      │      │      │      │      │      │
  └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  BACKEND API        │
              │  (Rust - 186k LOC)  │
              └─────────────────────┘
```

## Success Metrics

✅ **User can control 100% of system features through voice**
✅ **AI understands context across all 9 layers**
✅ **Complex multi-layer operations work seamlessly**
✅ **System state always synchronized with AI awareness**
✅ **No manual UI interaction required for any feature**

## Implementation Timeline

1. **Phase 1** (1-2 hours): System State Manager - Complete system awareness
2. **Phase 2** (2-3 hours): Enhanced Oracle AI - Detect and plan for all layers
3. **Phase 3** (3-4 hours): Backend API - Expose all system operations
4. **Phase 4** (2-3 hours): Unified Execution - Connect AI to real operations
5. **Phase 5** (1 hour): Testing - Validate complex flows

Total: ~10-13 hours of development

## Next Steps

1. ✅ Create `systemState.ts` - Comprehensive state management
2. ✅ Enhance `oracleAI.ts` - Full system intent detection
3. Add backend handlers for all system operations
4. Wire up execution layer in `App.tsx`
5. Test multi-layer AI operations
