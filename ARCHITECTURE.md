# Kāraṇa OS Architecture

> **Kāraṇa** (कारण) - Sanskrit for "cause" or "instrument" - The cause that enables sovereign computing.

## Overview

Kāraṇa OS is a **self-sovereign operating system** designed specifically for wearable computing, particularly smart glasses. It combines blockchain technology, edge AI, and privacy-first principles to create a truly personal computing experience where the user owns their data, identity, and compute.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        KĀRAṆA OS STACK                              │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │   Voice     │ │   Camera    │ │    HUD      │ │   Haptic    │   │
│  │  (Whisper)  │ │   (BLIP)    │ │  (AR/XR)    │ │  Feedback   │   │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘   │
│         │               │               │               │           │
│  ┌──────┴───────────────┴───────────────┴───────────────┴──────┐   │
│  │                    ORACLE LAYER                              │   │
│  │         (AI ↔ Blockchain Bridge / Intent Processing)        │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                             │                                       │
│  ┌──────────────────────────┴───────────────────────────────────┐   │
│  │                 INTELLIGENCE LAYER                           │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │   │
│  │  │ Context │ │ Memory  │ │Learning │ │Proactive│            │   │
│  │  │Awareness│ │ System  │ │ Engine  │ │Suggest. │            │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘            │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                             │                                       │
│  ┌──────────────────────────┴───────────────────────────────────┐   │
│  │                   BLOCKCHAIN LAYER                           │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │   │
│  │  │  Chain  │ │ Ledger  │ │Governan.│ │ Wallet  │            │   │
│  │  │ (Blocks)│ │ (KARA)  │ │  (DAO)  │ │(Ed25519)│            │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘            │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                             │                                       │
│  ┌──────────────────────────┴───────────────────────────────────┐   │
│  │                    NETWORK LAYER                             │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │   │
│  │  │   libp2p    │ │  Celestia   │ │    ZK Proofs        │    │   │
│  │  │  (Gossip)   │ │    (DA)     │ │ (Privacy/Verify)    │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Software Stack

### Layer 1: Hardware Interface

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Voice** | Whisper (tiny.en) | Real-time speech-to-text with wake word detection |
| **Camera** | BLIP + V4L2 | Image capture and AI-powered scene understanding |
| **HUD** | Custom AR Renderer | Heads-up display with notifications, timers, navigation |
| **Haptic** | Pattern-based Feedback | Tactile responses for confirmations and alerts |

### Layer 2: Oracle Layer (AI ↔ Blockchain Bridge)

The Oracle is the heart of Kāraṇa OS - it bridges natural language to blockchain operations:

```
User: "Hey Karana, send 50 tokens to Alice"
         ↓
   [Voice Recognition - Whisper]
         ↓
   [Intent Parsing - Semantic Embeddings]
         ↓
   [Action Extraction - {action: "transfer", to: "Alice", amount: 50}]
         ↓
   [Blockchain Execution - Ed25519 Signed Transaction]
         ↓
   [UI Formatting - HUD Display]
         ↓
HUD: "✓ Sent 50 KARA to Alice [Ed25519 ✓]"
```

### Layer 3: Intelligence System

| Module | Function |
|--------|----------|
| **Context Awareness** | Time of day, location, activity detection |
| **Memory System** | Conversation history, user facts, anaphora resolution |
| **Learning Engine** | Preference learning, pattern recognition, adaptive responses |
| **Proactive Suggestions** | Anticipates user needs based on patterns |

### Layer 4: Blockchain Infrastructure

| Component | Implementation | Purpose |
|-----------|----------------|---------|
| **Chain** | Custom PoS with Ed25519 | Block production, transaction verification |
| **Ledger** | RocksDB-backed | Token balances, staking, transfers |
| **Governance** | On-chain DAO | Proposals, voting, AI-analyzed decisions |
| **Wallet** | BIP-39 + Ed25519 | 24-word mnemonic, AES-256-GCM encrypted storage |

### Layer 5: Network & Data Availability

| Component | Technology | Purpose |
|-----------|------------|---------|
| **P2P Network** | libp2p (Gossipsub + Kademlia) | Peer discovery, intent broadcasting |
| **Data Availability** | Celestia (Mocha testnet) | State commitments, transaction batches |
| **Privacy** | ZK-SNARKs (Groth16) | Proof of storage, identity verification |

---

## Core Modules

### 1. Wallet (`wallet.rs`) - Self-Sovereign Identity

```rust
// BIP-39 compatible, 24-word mnemonic
let result = KaranaWallet::generate("device-id")?;
println!("Backup phrase: {}", result.recovery_phrase.display_for_backup());

// Ed25519 signatures for all transactions
let signature = wallet.sign(b"transaction data");

// DID format: did:karana:<base58(sha256(pubkey)[0:20])>
let did = wallet.did(); // "did:karana:7Xg9P2..."
```

**Security Features:**
- Device-bound key derivation
- AES-256-GCM encrypted storage
- Zeroize on drop (memory safety)
- Real Ed25519 signatures (not simulated)

### 2. Oracle (`oracle.rs`) - Natural Language → Blockchain

The Oracle translates human intent into blockchain operations:

```rust
// User speaks: "What's my balance?"
let response = oracle.process_query("What's my balance?", user_did)?;
// Returns formatted HUD output:
// ╭─── ✓ Wallet ───╮
// │ 1000 KARA      │
// ╰────────────────╯
```

**Supported Actions:**
- `transfer` - Send tokens
- `stake` - Lock tokens for voting power
- `create_proposal` - Submit governance proposals
- `vote` - Vote on proposals
- `capture_photo` - Take picture with glasses
- `identify_object` - AI-powered object recognition
- `set_timer` - Countdown timers
- `navigate` - Turn-by-turn directions

### 3. Voice (`voice.rs`) - Always-Listening AI

```rust
// Wake word detection with phonetic variants
"Hey Karana" | "Okay Karna" | "Hi Carana" → Activated

// Real-time transcription
voice.start_listening()?;
let text = voice.stop_and_transcribe()?;
// Uses Whisper tiny.en model for low-latency inference
```

### 4. Camera (`camera.rs`) - Visual Intelligence

```rust
// Simulated mode (always works)
let result = camera.capture()?;

// Real V4L2 capture (--features v4l2)
let result = camera.capture()?; // Uses /dev/video0

// AI-powered analysis
let result = camera.capture_and_analyze_with_ai(&mut ai)?;
// result.detected_objects = ["person", "laptop", "coffee cup"]
```

### 5. Chain (`chain.rs`) - Verified Transactions

```rust
// Create signed transaction
let tx = create_signed_transaction(&wallet, TransactionData::Transfer {
    to: "did:karana:recipient".to_string(),
    amount: 100,
});

// Verify Ed25519 signature
assert!(tx.verify()); // Cryptographically verified
```

---

## What Makes Kāraṇa OS Different

### 1. **True Self-Sovereignty**
Unlike cloud-dependent assistants (Siri, Alexa, Google Assistant), Kāraṇa runs entirely on your device:
- No data leaves your glasses without explicit consent
- Your wallet keys never touch a server
- AI inference happens locally (Whisper, BLIP, embeddings)

### 2. **Blockchain-Native Identity**
Your identity is a cryptographic keypair, not an email/password:
- **DID-based**: `did:karana:7Xg9P2...` 
- **Portable**: Export your 24-word phrase, restore anywhere
- **Verified**: Every action is Ed25519 signed

### 3. **AI That Learns YOU**
The Intelligence Layer adapts to your patterns:
- Learns your common recipients for transfers
- Predicts actions based on time/location
- Remembers context across conversations

### 4. **Governance-First Design**
Users collectively govern the OS through on-chain proposals:
- Propose features/changes
- Vote with staked tokens
- AI analyzes proposals for impact

### 5. **Glasses-Aware AI**
The AI knows what's possible on smart glasses:
```
User: "Open VS Code"
AI: "⚠️ Smart glasses can't run desktop IDEs like VS Code
     💡 I can show code snippets in your HUD, or sync notes 
        to review on your desktop later"
```

---

## Perfect for Smart Glasses

### Why Kāraṇa + Smart Glasses?

| Challenge | Kāraṇa Solution |
|-----------|-----------------|
| **Limited Display** | Minimal HUD design, voice-first interaction |
| **Battery Constraints** | Edge AI (quantized models), efficient Rust code |
| **Privacy Concerns** | Local processing, encrypted storage, ZK proofs |
| **No Keyboard** | Wake word + natural language commands |
| **Context Switching** | Proactive suggestions, ambient awareness |

### Smart Glasses Features

```
┌─────────────────────────────────────────┐
│  👁️ HUD Elements                        │
├─────────────────────────────────────────┤
│  ⏱️ Timer: 4:32 remaining              │
│  🔔 3 notifications                     │
│  💰 Balance: 1,247 KARA                 │
│  📍 Navigation: Turn left in 50m       │
│  👤 Person detected (Alice - contact)  │
└─────────────────────────────────────────┘
```

### Voice Commands

| Say This | Kāraṇa Does |
|----------|-------------|
| "Hey Karana, what am I looking at?" | Camera capture → BLIP analysis → HUD display |
| "Send 50 tokens to Bob" | Parse → Sign → Broadcast → Confirm |
| "Set a timer for 5 minutes" | Timer starts, HUD shows countdown |
| "What's the governance proposal about?" | Fetches, AI summarizes, reads aloud |
| "Navigate to the coffee shop" | AR arrows overlay on real world |

---

## Security Model

### Cryptographic Guarantees

| Layer | Algorithm | Purpose |
|-------|-----------|---------|
| Identity | Ed25519 | Transaction signing, DID verification |
| Storage | AES-256-GCM | Wallet encryption at rest |
| Key Derivation | PBKDF2-SHA256 | Password → encryption key |
| Proofs | Groth16 ZK-SNARKs | Privacy-preserving verification |
| Hashing | SHA-256 | Block hashes, Merkle roots |

### Device Binding
Wallet keys are derived from:
1. BIP-39 mnemonic (24 words)
2. Device-specific identifier
3. Optional password

This means even if someone gets your mnemonic, they can't use it without your device.

---

## Getting Started

### Build & Run

```bash
# Clone
git clone https://github.com/user/karana-os
cd karana-os/karana-core

# Build (default - simulated camera)
cargo build --release

# Build with real camera support (requires libv4l2-dev)
cargo build --release --features v4l2

# Run tests
cargo test --lib

# Run interactive demo
cargo run --example glasses_interactive
```

### Create Your Identity

```bash
# Generate wallet during onboarding
cargo run --example onboarding_demo

# Your 24-word recovery phrase will be displayed
# BACK THIS UP SECURELY - it's your identity!
```

---

## Future Roadmap

- [ ] **Celestia Integration**: Submit state roots to DA layer
- [ ] **Multi-device Sync**: Same identity across glasses, phone, desktop
- [ ] **App Marketplace**: Install verified dApps from governance-approved bazaar
- [ ] **Hardware Wallet**: Dedicated secure element for key storage
- [ ] **Mesh Networking**: Peer-to-peer without internet

---

## License

Apache 2.0 - Built for the sovereign future.

---

*Kāraṇa OS: Your glasses, your data, your rules.*
