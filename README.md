# Kāraṇa OS (Symbiotic Horizon)

```text
  _  __   _   ___   _   _  _   _   
 | |/ /  /_\ | _ \ /_\ | \| | /_\  
 | ' <  / _ \|   // _ \| .` |/ _ \ 
 |_|\_\/_/ \_\_|_/_/ \_\_|\_/_/ \_\
                                   
      The Sovereign AI-Native OS
```

> **"The Operating System is not a tool. It is a partner."**

[![Tests](https://img.shields.io/badge/tests-221%20passing-brightgreen)](./karana-core/src/)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange)](https://www.rust-lang.org/)
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

### ✅ Phase 1: Spatial AR System (Complete)
*159 tests passing*

| Component | Description | Status |
|-----------|-------------|--------|
| **World Coordinates** | GPS + SLAM fusion, LocalCoord, RoomId | ✅ Complete |
| **Spatial Anchors** | Persistent AR content pinning with visual signatures | ✅ Complete |
| **SLAM Engine** | Visual odometry, feature tracking, pose estimation | ✅ Complete |
| **Relocalization** | Re-finding location after tracking loss | ✅ Complete |
| **Room Mapping** | Semantic room boundaries and transitions | ✅ Complete |

### ✅ Phase 2: Persistent AR Tabs (Complete)
*62 tests, 4,260 lines of code*

| Component | Description | Status |
|-----------|-------------|--------|
| **ARTab Core** | Tabs pinned in physical space via spatial anchors | ✅ Complete |
| **Tab Content Types** | Browser, Video, Code Editor, Documents, Games, Widgets | ✅ Complete |
| **Tab Manager** | Multi-tab lifecycle, focus history, layouts | ✅ Complete |
| **Browser Wrapper** | Navigation, scrolling, voice control | ✅ Complete |
| **Gaze Interaction** | Dwell selection (500ms), cursor tracking | ✅ Complete |
| **Voice Commands** | "scroll down", "close tab", "go to google.com" | ✅ Complete |
| **Tab Renderer** | Depth sorting, MVP projection, compositing | ✅ Complete |

### ✅ Oracle & AI Integration (Complete)

| Component | Description | Status |
|-----------|-------------|--------|
| **Oracle Veil** | AI ↔ Blockchain bridge with ZK intent proofs | ✅ Complete |
| **Intent Proofs** | Zero-knowledge authorization without revealing details | ✅ Complete |
| **Manifest System** | Haptic patterns, AR overlays, whisper notifications | ✅ Complete |
| **Use Cases** | Restaurant bill splitting, transit navigation, shopping | ✅ Complete |

### ✅ Hardware Abstraction (Complete)

| Component | Description | Status |
|-----------|-------------|--------|
| **Virtual Glasses** | Full hardware simulation for development | ✅ Complete |
| **Power Management** | Battery simulation, thermal throttling | ✅ Complete |
| **Display System** | Waveguide simulation, brightness, color temp | ✅ Complete |
| **Sensor Fusion** | IMU, GPS, depth camera integration | ✅ Complete |
| **Scenario Runner** | Automated testing of real-world scenarios | ✅ Complete |

### ✅ Core Infrastructure (Complete)

| Component | Description | Status |
|-----------|-------------|--------|
| **Blockchain** | Ed25519 signed blocks, transaction verification | ✅ Complete |
| **Wallet** | Key generation, encryption, restore from mnemonic | ✅ Complete |
| **P2P Networking** | libp2p with mDNS discovery, gossipsub | ✅ Complete |
| **Celestia DA** | Data availability layer integration | ✅ Complete |
| **Voice Processing** | Wake word detection, VAD, command parsing | ✅ Complete |
| **Timer System** | Countdown, stopwatch, named timers | ✅ Complete |
| **Notifications** | Priority-based, haptic feedback, whisper mode | ✅ Complete |

---

## 🏗️ Architecture Overview

Kāraṇa OS uses a **7-Layer Software Stack**:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 7: Interface (HUD, Voice, Gestures)                  │
├─────────────────────────────────────────────────────────────┤
│  Layer 6: Applications (Timer, Notifications, Proactive)    │
├─────────────────────────────────────────────────────────────┤
│  Layer 5: AI Engine (Vision, Voice, Language, Learning)     │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: Oracle Bridge (AI ↔ Blockchain)                   │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Blockchain (Chain, Wallet, Economy, Celestia DA)  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: P2P Network (libp2p, mDNS, Gossip)                │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Hardware (Camera, Sensors, Display, Compute)      │
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
git clone https://github.com/AumSahay  /karana-os.git
cd karana-os

# Run with simulated hardware (default)
cargo run

# Run with real camera (Linux with v4l2)
cargo run --features v4l2

# Run all tests (61 tests)
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
├── karana-core/src/
│   ├── lib.rs              # Main exports
│   ├── monad.rs            # System orchestrator
│   ├── chain.rs            # Blockchain implementation
│   ├── wallet.rs           # Ed25519 wallet
│   ├── camera.rs           # V4L2 camera support
│   ├── voice.rs            # Voice processing & wake words
│   ├── glasses.rs          # Smart glasses integration
│   ├── hud.rs              # Heads-up display
│   │
│   ├── spatial/            # 🆕 Spatial AR System
│   │   ├── mod.rs          # Spatial system orchestration
│   │   ├── world_coords.rs # GPS + SLAM coordinate fusion
│   │   ├── anchor.rs       # Spatial anchors for AR content
│   │   ├── slam.rs         # Visual SLAM engine
│   │   ├── relocalize.rs   # Re-localization after tracking loss
│   │   └── room.rs         # Room mapping and boundaries
│   │
│   ├── ar_tabs/            # 🆕 Persistent AR Tabs
│   │   ├── mod.rs          # Module exports
│   │   ├── tab.rs          # ARTab core structures
│   │   ├── manager.rs      # Multi-tab lifecycle management
│   │   ├── browser.rs      # Web browser wrapper
│   │   ├── interaction.rs  # Gaze, voice, gesture input
│   │   └── render.rs       # Tab compositing and projection
│   │
│   ├── oracle/             # AI ↔ Blockchain Bridge
│   │   ├── mod.rs          # Oracle exports
│   │   ├── veil.rs         # Intent processing with ZK proofs
│   │   ├── manifest.rs     # Haptics, AR overlays, whispers
│   │   └── use_cases.rs    # Real-world scenario implementations
│   │
│   ├── hardware/           # Hardware Abstraction Layer
│   │   ├── mod.rs          # Hardware manager
│   │   ├── power.rs        # Battery and thermal management
│   │   └── sensors.rs      # IMU, GPS, depth sensors
│   │
│   ├── simulator/          # Development Simulator
│   │   ├── mod.rs          # Simulator orchestration
│   │   ├── device.rs       # Virtual glasses hardware
│   │   ├── display.rs      # Virtual waveguide display
│   │   ├── scenario.rs     # Automated test scenarios
│   │   └── tui.rs          # Terminal UI for simulation
│   │
│   ├── zk/                 # Zero-Knowledge Proofs
│   │   └── intent_proof.rs # ZK intent authorization
│   │
│   ├── ai/                 # AI Engine
│   │   ├── mod.rs          # AI model management
│   │   └── assistant.rs    # Conversational AI
│   │
│   ├── celestia.rs         # Data availability layer
│   ├── economy.rs          # Token economics
│   ├── learning.rs         # Adaptive learning
│   ├── notifications.rs    # Notification system
│   ├── timer.rs            # Timer and stopwatch
│   └── ...
│
├── tests/                  # Integration tests
├── examples/               # Usage examples
├── ARCHITECTURE.md         # Technical documentation
├── SIMPLE_GUIDE.md         # User-friendly guide
└── README.md               # This file
```

---

## 🧪 Testing

```bash
# Run all library tests
cargo test --lib

# Current status: 221 tests passing
# Modules with tests:
# - spatial: 45 tests (world coords, anchors, SLAM, relocalization)
# - ar_tabs: 62 tests (tabs, manager, browser, interaction, render)
# - oracle: 25 tests (veil, manifest, use cases)
# - zk: 8 tests (intent proofs, range proofs)
# - wallet: 6 tests
# - chain: 4 tests  
# - voice: 7 tests
# - hardware: 15 tests (simulator, devices, power)
# - glasses: 12 tests
# - timer: 5 tests
# - notifications: 8 tests
# - ... and more
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
