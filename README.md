# Kāraṇa OS (Symbiotic Horizon)

```text
  _  __   _   ___   _   _  _   _   
 | |/ /  /_\ | _ \ /_\ | \| | /_\  
 | ' <  / _ \|   // _ \| .` |/ _ \ 
 |_|\_\/_/ \_\_|_/_/ \_\_|\_/_/ \_\
                                   
      The Sovereign AI-Native OS
```

> **"The Operating System is not a tool. It is a partner."**

[![Tests](https://img.shields.io/badge/tests-1517%20passing-brightgreen)](./karana-core/src/)
[![Lines](https://img.shields.io/badge/lines-137k+-blue)](./karana-core/src/)
[![Rust](https://img.shields.io/badge/rust-nightly%202024-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

## 🌟 What is Kāraṇa?

**Kāraṇa OS** is a sovereign AI-native operating system designed for the post-app era. Unlike traditional systems (Windows, Linux, macOS) that force you to manage files and open applications, Kāraṇa is built around **Intents** and **Context**.

It is designed specifically for **Smart Glasses and IoT devices**, providing a "Symbiotic Interface" where the OS uses AI to understand your goals and a blockchain ledger to secure your data. It doesn't just run programs; it **thinks with you**.

### 📚 Documentation

| Document | Description |
|----------|-------------|
| [**ARCHITECTURE.md**](./ARCHITECTURE.md) | Complete technical documentation of the 7-layer software stack |
| [**SIMPLE_GUIDE.md**](./SIMPLE_GUIDE.md) | User-friendly explanation in simple language |

---

## 🎯 Development Progress

### ✅ Phase 1-5: Core Foundation (Complete)
*Foundation systems fully operational*

| Component | Description | Status |
|-----------|-------------|--------|
| **Blockchain** | Ed25519 signed blocks, transaction verification | ✅ Complete |
| **Wallet** | Key generation, encryption, restore from mnemonic | ✅ Complete |
| **P2P Networking** | libp2p with mDNS discovery, gossipsub | ✅ Complete |
| **Celestia DA** | Data availability layer integration | ✅ Complete |
| **Voice Processing** | Wake word detection, VAD, command parsing | ✅ Complete |
| **Timer System** | Countdown, stopwatch, named timers | ✅ Complete |
| **Notifications** | Priority-based, haptic feedback, whisper mode | ✅ Complete |

### ✅ Phase 6-10: Spatial AR System (Complete)
*Persistent AR in physical space*

| Component | Description | Status |
|-----------|-------------|--------|
| **World Coordinates** | GPS + SLAM fusion, LocalCoord, RoomId | ✅ Complete |
| **Spatial Anchors** | Persistent AR content pinning with visual signatures | ✅ Complete |
| **SLAM Engine** | Visual odometry, feature tracking, pose estimation | ✅ Complete |
| **Relocalization** | Re-finding location after tracking loss | ✅ Complete |
| **Room Mapping** | Semantic room boundaries and transitions | ✅ Complete |

### ✅ Phase 11-15: AR Tabs & WebXR (Complete)
*Browser-like experience in 3D space*

| Component | Description | Status |
|-----------|-------------|--------|
| **ARTab Core** | Tabs pinned in physical space via spatial anchors | ✅ Complete |
| **Tab Content Types** | Browser, Video, Code Editor, Documents, Games, Widgets | ✅ Complete |
| **Tab Manager** | Multi-tab lifecycle, focus history, layouts | ✅ Complete |
| **WebXR Integration** | Session management, hit testing, anchors API | ✅ Complete |
| **Light Estimation** | Real-time environmental lighting for AR | ✅ Complete |

### ✅ Phase 16-20: Oracle & AI Integration (Complete)
*AI ↔ Blockchain bridge with ZK proofs*

| Component | Description | Status |
|-----------|-------------|--------|
| **Oracle Veil** | AI ↔ Blockchain bridge with ZK intent proofs | ✅ Complete |
| **Intent Proofs** | Zero-knowledge authorization without revealing details | ✅ Complete |
| **Manifest System** | Haptic patterns, AR overlays, whisper notifications | ✅ Complete |
| **Use Cases** | Restaurant bill splitting, transit navigation, shopping | ✅ Complete |

### ✅ Phase 21-25: Advanced Interaction (Complete)
*Multi-modal human interface*

| Component | Description | Status |
|-----------|-------------|--------|
| **Gaze Tracking** | Eye-based interaction, dwell selection, fixation detection | ✅ Complete |
| **Gesture Recognition** | Hand pose detection, finger tracking, 3D gestures | ✅ Complete |
| **Multimodal Fusion** | Voice + gaze + gesture combined understanding | ✅ Complete |
| **Scene Understanding** | Semantic labeling, object relationships | ✅ Complete |
| **Collaborative AR** | Multi-user shared AR experiences | ✅ Complete |

### ✅ Phase 26-29: AI Layer (Complete)
*Complete natural language understanding*

| Component | Description | Status |
|-----------|-------------|--------|
| **NLU Engine** | Intent classification, entity extraction, confidence scoring | ✅ Complete |
| **Dialogue Manager** | Multi-turn conversations, context tracking, slot filling | ✅ Complete |
| **Response Generator** | Natural language response synthesis | ✅ Complete |
| **Reasoning Engine** | Context-aware decision making | ✅ Complete |
| **Action Executor** | Safe execution of user intents | ✅ Complete |

### ✅ Phase 30: Gesture-Based AR Interaction (Complete)
*Full hand and finger tracking for AR manipulation*

| Component | Description | Status |
|-----------|-------------|--------|
| **Hand Detector** | Real-time hand pose estimation | ✅ Complete |
| **Finger Tracking** | Individual finger joint positions | ✅ Complete |
| **AR Interaction** | Pinch, grab, push gestures for AR objects | ✅ Complete |
| **Gesture Vocabulary** | 15+ recognized gesture types | ✅ Complete |

### ✅ Phase 31: System Infrastructure (Complete)
*Production-ready system services*

| Component | Description | Status |
|-----------|-------------|--------|
| **Diagnostics** | Health monitoring, metrics, profiling, watchdog | ✅ Complete |
| **Recovery** | Crash dumps, error logging, auto-recovery strategies | ✅ Complete |
| **OTA Updates** | Secure downloads, atomic installs, rollback protection | ✅ Complete |
| **Security** | Multi-factor auth, biometrics, encryption, RBAC | ✅ Complete |

### ✅ Additional Systems (Complete)
*Supporting infrastructure*

| Component | Description | Status |
|-----------|-------------|--------|
| **Accessibility** | Screen reader, magnifier, vision accessibility | ✅ Complete |
| **Wellness** | Eye strain monitoring, posture tracking, usage analytics | ✅ Complete |
| **Notifications v2** | Smart grouping, AI summaries, priority management | ✅ Complete |
| **Power Management** | Battery optimization, thermal throttling, power profiles | ✅ Complete |
| **Settings Engine** | Hierarchical config, cloud sync, change notifications | ✅ Complete |
| **Navigation** | Turn-by-turn AR directions, POI discovery | ✅ Complete |
| **Social** | Contact management, presence, sharing | ✅ Complete |

---

## 🏗️ Architecture Overview

Kāraṇa OS uses a **9-Layer Software Stack**:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 9: System Services (OTA, Security, Diagnostics)      │
├─────────────────────────────────────────────────────────────┤
│  Layer 8: Applications (Timer, Navigation, Social, Apps)    │
├─────────────────────────────────────────────────────────────┤
│  Layer 7: Interface (HUD, Voice, Gestures, Gaze, AR)        │
├─────────────────────────────────────────────────────────────┤
│  Layer 6: AI Engine (NLU, Dialogue, Reasoning, Actions)     │
├─────────────────────────────────────────────────────────────┤
│  Layer 5: Intelligence (Multimodal, Scene, Prediction)      │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: Oracle Bridge (AI ↔ Blockchain, ZK Proofs)        │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Blockchain (Chain, Wallet, Economy, Celestia DA)  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: P2P Network (libp2p, mDNS, Gossip, Sync)          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Hardware (Camera, Sensors, Display, Audio, Power) │
└─────────────────────────────────────────────────────────────┘
```

**The Monad** (`src/monad.rs`) orchestrates all layers, producing signed blocks every 30 seconds with Ed25519 cryptography.

👉 **[Read ARCHITECTURE.md](./ARCHITECTURE.md)** for complete technical details.

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Linux with v4l2 support (for real camera)

### Run Kāraṇa OS

```bash
# Clone the repository
git clone https://github.com/incyashraj/karana-os.git
cd karana-os

# Run with simulated hardware (default)
cargo run

# Run with real camera (Linux with v4l2)
cargo run --features v4l2

# Run all tests (1517 tests)
cargo test --lib
```

### What Happens
1. **Wallet Creation**: First run creates `node_wallet.enc` with your Ed25519 keypair
2. **AI Initialization**: Loads BLIP (vision), Whisper (speech), MiniLM (embeddings)
3. **P2P Networking**: Joins the Kāraṇa swarm via mDNS discovery
4. **Block Production**: Every 30 seconds, a new signed block is produced

---

## 🧠 AI Capabilities

| Model | Purpose | Size |
|-------|---------|------|
| **MiniLM-L6-v2** | Semantic understanding | 22MB |
| **BLIP** | Vision/object identification | ~500MB |
| **Whisper** (tiny) | Speech-to-text | ~75MB |
| **TinyLlama** | Text generation | ~1GB |

All models run **100% offline** using ONNX Runtime. No cloud required.

```rust
// Example: What you can ask Kāraṇa
"What am I looking at?"          // → BLIP analyzes camera
"Set a timer for 5 minutes"      // → Voice command processing
"Remind me about this later"     // → Context + blockchain storage
"Find my keys"                   // → Proactive memory search
```

---

## 🔗 Blockchain Features

- **Ed25519 Signatures**: Real cryptographic block signing
- **Celestia Data Availability**: Optional integration with Mocha testnet
- **DAO Governance**: Vote on system parameters
- **Economic Model**: Resource credits, staking, reputation

---

## 📜 The Philosophy

Kāraṇa OS is a **First Principles Rethink** of the operating system. It rejects the legacy metaphors of the 1970s (files, folders, applications) and replaces them with a sovereign, symbiotic architecture built for the age of AI.

### Why Different?

| Traditional OS | Kāraṇa OS |
|----------------|-----------|
| Files & Folders | Semantic Memory |
| Applications | Intents |
| Click & Type | Voice & Vision |
| Cloud-dependent | 100% Offline |
| Centralized | Blockchain-verified |
| One device | Distributed Swarm |

👉 **[Read SIMPLE_GUIDE.md](./SIMPLE_GUIDE.md)** for a non-technical explanation.

---

## 🕶️ Smart Glasses Hardware

Kāraṇa OS is designed for a "Split-Architecture" wearable future:

| Component | Device | Purpose |
|-----------|--------|---------|
| **Display** | XREAL Air / Rokid | Dumb terminal (1080p OLED) |
| **Compute** | Orange Pi 5 / RK3588 | Belt-worn "Puck" running Kāraṇa |
| **Camera** | USB webcam / v4l2 | Vision input for BLIP |
| **Audio** | USB mic / Bluetooth | Voice input for Whisper |

👉 **[Read HARDWARE_PLAN.md](./HARDWARE_PLAN.md)** for recommended dev kits and the roadmap.

---

## 🛠️ Project Structure

```
karana-os/
├── karana-core/src/           # 137,000+ lines of Rust
│   ├── lib.rs                 # Main exports (45 modules)
│   ├── monad.rs               # System orchestrator (87KB)
│   │
│   ├── # === Core Systems ===
│   ├── chain.rs               # Blockchain implementation
│   ├── wallet.rs              # Ed25519 wallet
│   ├── celestia.rs            # Data availability layer
│   ├── economy.rs             # Token economics
│   │
│   ├── # === Interface Layer ===
│   ├── voice.rs               # Voice processing & wake words
│   ├── hud.rs                 # Heads-up display
│   ├── glasses.rs             # Smart glasses integration
│   ├── multimodal.rs          # Voice + Gaze + Gesture fusion
│   │
│   ├── # === AI Layer ===
│   ├── ai_layer/              # Natural Language Understanding
│   │   ├── nlu.rs             # Intent classification
│   │   ├── intent.rs          # Intent resolution
│   │   ├── dialogue.rs        # Multi-turn conversations
│   │   ├── entities.rs        # Entity extraction
│   │   ├── slot_filler.rs     # Slot filling for actions
│   │   ├── response.rs        # Response generation
│   │   ├── reasoning.rs       # Context-aware reasoning
│   │   ├── action_executor.rs # Safe action execution
│   │   └── error_recovery.rs  # NLU error handling
│   │
│   ├── intelligence/          # Prediction & Orchestration
│   │   ├── predictor.rs       # User behavior prediction
│   │   ├── router.rs          # Request routing
│   │   ├── orchestrator.rs    # Multi-model coordination
│   │   └── workflows.rs       # Complex task workflows
│   │
│   ├── # === Spatial AR ===
│   ├── spatial/               # Spatial Computing
│   │   ├── world_coords.rs    # GPS + SLAM coordinate fusion
│   │   ├── slam.rs            # Visual SLAM engine
│   │   ├── anchor.rs          # Spatial anchors
│   │   ├── relocalize.rs      # Re-localization
│   │   ├── room.rs            # Room mapping
│   │   └── persistence.rs     # Anchor persistence
│   │
│   ├── ar_tabs/               # Persistent AR Tabs
│   │   ├── tab.rs             # ARTab core structures
│   │   ├── manager.rs         # Multi-tab lifecycle
│   │   ├── browser.rs         # Web browser wrapper
│   │   ├── interaction.rs     # Gaze, voice, gesture input
│   │   └── render.rs          # Tab compositing
│   │
│   ├── ar/                    # AR Rendering
│   │   ├── anchors.rs         # AR anchor management
│   │   └── renderer.rs        # AR rendering pipeline
│   │
│   ├── webxr/                 # WebXR Integration
│   │   ├── session.rs         # XR session management
│   │   ├── anchors.rs         # WebXR anchors API
│   │   ├── hit_test.rs        # Surface hit testing
│   │   └── light_estimation.rs# Environmental lighting
│   │
│   ├── # === Interaction ===
│   ├── gesture/               # Gesture Recognition
│   │   ├── detector.rs        # Hand detection
│   │   ├── finger_tracking.rs # Finger joint tracking
│   │   ├── ar_interaction.rs  # AR object manipulation
│   │   └── gestures.rs        # Gesture vocabulary
│   │
│   ├── gaze/                  # Gaze Tracking
│   │   ├── tracker.rs         # Eye tracking
│   │   ├── analysis.rs        # Fixation detection
│   │   └── interaction.rs     # Gaze-based UI
│   │
│   ├── scene/                 # Scene Understanding
│   │   ├── semantic.rs        # Semantic labeling
│   │   └── anchors.rs         # Scene anchor management
│   │
│   ├── collab/                # Collaborative AR
│   │   ├── session.rs         # Multi-user sessions
│   │   └── sync.rs            # State synchronization
│   │
│   ├── # === Oracle & ZK ===
│   ├── oracle/                # AI ↔ Blockchain Bridge
│   │   ├── veil.rs            # Intent processing + ZK proofs
│   │   ├── manifest.rs        # Haptics, AR overlays
│   │   ├── sense.rs           # Sensor data oracle
│   │   └── use_cases.rs       # Real-world scenarios
│   │
│   ├── zk/                    # Zero-Knowledge Proofs
│   │   └── intent_proof.rs    # ZK intent authorization
│   │
│   ├── # === System Services ===
│   ├── diagnostics/           # System Health
│   │   ├── health.rs          # Health monitoring
│   │   ├── metrics.rs         # System metrics
│   │   ├── profiler.rs        # Performance profiling
│   │   └── watchdog.rs        # Deadlock detection
│   │
│   ├── recovery/              # Crash Recovery
│   │   ├── recovery.rs        # Recovery strategies
│   │   ├── crash_dump.rs      # Crash dumps
│   │   ├── error_log.rs       # Error logging
│   │   └── reporter.rs        # Crash reporting
│   │
│   ├── ota/                   # Over-The-Air Updates
│   │   ├── downloader.rs      # Secure download
│   │   ├── installer.rs       # Atomic installation
│   │   ├── rollback.rs        # Rollback protection
│   │   ├── version.rs         # Version management
│   │   └── manifest.rs        # Update manifests
│   │
│   ├── security/              # Security Services
│   │   ├── authentication.rs  # Multi-factor auth
│   │   ├── biometric.rs       # Iris/voice/face auth
│   │   ├── encryption.rs      # AES-256, ChaCha20
│   │   ├── access_control.rs  # RBAC permissions
│   │   └── secure_storage.rs  # Encrypted storage
│   │
│   ├── # === Applications ===
│   ├── apps/                  # App Runtime
│   │   ├── runtime.rs         # App execution
│   │   └── manager.rs         # App lifecycle
│   │
│   ├── navigation/            # Navigation
│   │   ├── routing.rs         # Turn-by-turn directions
│   │   └── location.rs        # Location services
│   │
│   ├── social/                # Social Features
│   │   ├── contacts.rs        # Contact management
│   │   └── presence.rs        # Online presence
│   │
│   ├── wellness/              # User Wellness
│   │   ├── eye_strain.rs      # Eye strain monitoring
│   │   ├── posture.rs         # Posture tracking
│   │   └── usage.rs           # Usage analytics
│   │
│   ├── notifications_v2/      # Smart Notifications
│   │   ├── display.rs         # Notification display
│   │   └── summary.rs         # AI summaries
│   │
│   ├── # === Hardware ===
│   ├── hardware/              # Hardware Abstraction
│   │   ├── power.rs           # Power management
│   │   └── sensors.rs         # Sensor fusion
│   │
│   ├── vision/                # Computer Vision
│   │   ├── processing.rs      # Image processing
│   │   └── detection.rs       # Object detection
│   │
│   ├── audio/                 # Spatial Audio
│   │   ├── spatial.rs         # 3D audio positioning
│   │   └── mixer.rs           # Audio mixing
│   │
│   ├── haptics/               # Haptic Feedback
│   │   ├── patterns.rs        # Vibration patterns
│   │   └── spatial.rs         # Directional haptics
│   │
│   ├── power/                 # Power Management
│   │   ├── profiles.rs        # Power profiles
│   │   ├── thermal.rs         # Thermal management
│   │   └── estimator.rs       # Battery estimation
│   │
│   ├── # === Accessibility ===
│   ├── accessibility/         # Accessibility Features
│   │   ├── screen_reader.rs   # Screen reader
│   │   ├── magnifier.rs       # Visual magnification
│   │   └── vision.rs          # Vision accessibility
│   │
│   ├── # === Simulator ===
│   ├── simulator/             # Development Simulator
│   │   ├── device.rs          # Virtual glasses hardware
│   │   ├── display.rs         # Virtual waveguide display
│   │   ├── scenario.rs        # Automated test scenarios
│   │   ├── tui.rs             # Terminal UI
│   │   └── qemu.rs            # QEMU integration
│   │
│   └── # === Support ===
│       ├── networking/        # Network services
│       ├── settings/          # Settings engine
│       ├── privacy/           # Privacy controls
│       ├── assistant/         # AI assistant
│       └── performance/       # Performance optimization
│
├── examples/                  # Usage examples
├── tests/                     # Integration tests
├── ARCHITECTURE.md            # Technical documentation
├── SIMPLE_GUIDE.md            # User-friendly guide
└── README.md                  # This file
```

---

## 🧪 Testing

```bash
# Run all library tests
cargo test --lib

# Current status: 1517 tests passing
# Major test categories:
# - spatial: 45 tests (world coords, anchors, SLAM, relocalization)
# - ar_tabs: 62 tests (tabs, manager, browser, interaction, render)
# - ar: 35 tests (anchors, rendering)
# - gesture: 48 tests (detection, finger tracking, AR interaction)
# - gaze: 32 tests (tracking, analysis, interaction)
# - ai_layer: 95 tests (NLU, dialogue, entities, reasoning, actions)
# - intelligence: 42 tests (prediction, routing, workflows)
# - oracle: 25 tests (veil, manifest, use cases)
# - security: 45 tests (auth, biometrics, encryption, RBAC)
# - ota: 38 tests (download, install, rollback)
# - diagnostics: 28 tests (health, metrics, watchdog)
# - recovery: 22 tests (crash dumps, error logs)
# - webxr: 35 tests (sessions, anchors, hit testing)
# - collab: 25 tests (sessions, sync)
# - wellness: 30 tests (eye strain, posture, usage)
# - notifications_v2: 28 tests (display, summaries)
# - hardware: 40 tests (simulator, devices, power)
# - ... and many more
```

---

## 🤝 Contributing

Kāraṇa OS is an experimental project pushing the boundaries of what an OS can be. We welcome contributions in:

- **AI Models**: Better edge-optimized models
- **Hardware Support**: More camera/sensor integrations
- **P2P Networking**: Distributed consensus improvements
- **Documentation**: Translations and tutorials

---

## 📄 License

MIT License - See [LICENSE](./LICENSE) for details.

---

*"We do not build the OS to control the machine. We build the OS to free the mind."*

**Built with ❤️ by the Kāraṇa Team**
