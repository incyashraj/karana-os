# Kāraṇa OS (Symbiotic Horizon)

```text
  _  __   _   ___   _   _  _   _   
 | |/ /  /_\ | _ \ /_\ | \| | /_\  
 | ' <  / _ \|   // _ \| .` |/ _ \ 
 |_|\_\/_/ \_\_|_/_/ \_\_|\_/_/ \_\
                                   
      The Sovereign AI-Native OS
```

> **"The Operating System is not a tool. It is a partner."**

[![Tests](https://img.shields.io/badge/tests-61%20passing-brightgreen)](./src/)
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
| [**HARDWARE_PLAN.md**](./HARDWARE_PLAN.md) | Hardware requirements and recommended dev kits |

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
├── src/
│   ├── lib.rs          # Main exports
│   ├── monad.rs        # System orchestrator (732 lines)
│   ├── chain.rs        # Blockchain implementation
│   ├── wallet.rs       # Ed25519 wallet (496 lines)
│   ├── oracle.rs       # AI ↔ Blockchain bridge
│   ├── camera.rs       # V4L2 camera support (434 lines)
│   ├── voice.rs        # Voice processing
│   ├── hud.rs          # Heads-up display
│   ├── ai/
│   │   ├── mod.rs      # AI engine (1270 lines)
│   │   └── assistant.rs
│   ├── celestia.rs     # Data availability layer
│   ├── economy.rs      # Token economics
│   ├── learning.rs     # Adaptive learning
│   └── ...
├── tests/              # 61 tests
├── examples/           # Usage examples
├── ARCHITECTURE.md     # Technical docs
├── SIMPLE_GUIDE.md     # User-friendly docs
└── README.md           # This file
```

---

## 🧪 Testing

```bash
# Run all library tests
cargo test --lib

# Current status: 61 tests passing
# - wallet: 6 tests
# - chain: 4 tests  
# - voice: 7 tests
# - camera: 2 tests
# - timer: 5 tests
# - ai: 12 tests
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
