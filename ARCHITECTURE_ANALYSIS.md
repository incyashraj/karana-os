# Kāraṇa OS Architecture Analysis

## Current Implementation vs. Target Vision (v1.1 "Oracle Veil")

### Executive Summary

After thorough analysis, there are **significant architectural differences** between the target vision and current implementation. The target vision describes an **AI-Centric Symbiotic Oracle** where:

1. **The AI Oracle is the SOLE user interface** - no panels, buttons, or menus
2. **All operations go through ZK-proven intent chains**
3. **Minimal AR output** - "whispers" and haptic pulses only
4. **Candle Phi-3** as the core Oracle model (not TinyLlama)
5. **Backend weave is invisible** - Oracle mediates everything

The current implementation has:
- A full React-based simulator UI with multiple panels, buttons, and modes
- Partial Oracle integration (but not as sole interface)
- Working ZK proofs (Groth16/Arkworks)
- TinyLlama instead of Phi-3 (with embedding fallback)
- Frontend-driven intent processing (not backend-mediated)

---

## Detailed Component Analysis

### 1. Oracle Veil (AI Gateway) - **MAJOR GAPS**

#### Target Architecture (from plan):
```rust
pub struct OracleVeil {
    ai: Phi3,  // Phi-3 mini quantized
    device: Device,
    monad: KaranaMonad,
}

impl OracleVeil {
    pub async fn mediate(&mut self, input: MultimodalInput) -> Result<Manifest> {
        // Step 1: Parse Intent (Gaze/Voice → Embed)
        // Step 2: ZK-Sign (Prove "This intent is mine")
        // Step 3: Command Backend (Oracle to Monad)
        // Step 4: Process & Manifest (AI Re-Present)
    }
}
```

#### Current Implementation:
- **File**: `karana-core/src/oracle/mod.rs`
- Uses rule-based intent parsing (not real Phi-3)
- No `OracleVeil` struct - instead `Oracle` and `KaranaOracle` wrappers
- No multimodal input parsing (gaze/voice sensors)
- No ZK-signing of intents before execution
- Separate frontend handles UI manifestation

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Phi-3 AI Model | Candle Phi-3 q4.gguf | TinyLlama + Rule-based | ❌ Different |
| Multimodal Input | Gaze/Voice/Gesture | Text only | ❌ Missing |
| ZK Intent Signing | Every intent proven | Optional, not integrated | ❌ Partial |
| Minimal Manifest | Haptic + AR whispers | Full UI panels | ❌ Wrong approach |
| Sole Interface | Oracle only | Oracle + UI modes | ❌ Architectural |

---

### 2. Backend Weave (Monad) - **GOOD, NEEDS WIRING**

#### Target Architecture:
```rust
impl KaranaMonad {
    pub async fn execute_intent(&mut self, intent: &str, proof: Vec<u8>) -> Result<String> {
        // Vigil first → Runtime spawn → Storage act → Swarm relay → DAO attest
    }
}
```

#### Current Implementation:
- **File**: `karana-core/src/monad.rs`
- Has all atoms: Boot, Runtime, UI, Vigil, Storage, Swarm, AI, Ledger, DAO, Chain
- `process_oracle_intent()` exists but doesn't follow the target flow
- `execute_real_action()` partially implements the pipeline

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Monad Structure | Weaves atoms silently | Correct structure | ✅ Good |
| Oracle Mediation | Oracle commands monad | Direct user calls | ⚠️ Needs rewiring |
| ZK Proof Flow | Proof required for all | Optional proof | ⚠️ Partial |
| Channel Commands | tokio::mpsc | Direct calls | ⚠️ Different |

---

### 3. Input Sense (Multimodal) - **NOT IMPLEMENTED**

#### Target Architecture:
```rust
pub struct MultimodalSense {
    camera: VideoCapture,  // OpenCV
}

impl MultimodalSense {
    pub fn capture(&mut self) -> Result<MultimodalInput> {
        // Gaze features + Voice MFCC
    }
}
```

#### Current Implementation:
- **No equivalent file exists**
- Camera exists (`camera.rs`) but for vision capture, not gaze tracking
- Voice exists (`voice.rs`) but only for wake word + transcription
- No gaze/eye tracking
- No gesture recognition
- No multimodal fusion

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Gaze Tracking | OpenCV eye vector | Not implemented | ❌ Missing |
| Voice Features | MFCC embedding | Whisper transcription | ⚠️ Different |
| Gesture | IMU-based | Not implemented | ❌ Missing |
| Multimodal Fusion | Tensor embedding | Not implemented | ❌ Missing |

---

### 4. Output Manifest (Minimal AR/Haptic) - **WRONG APPROACH**

#### Target Architecture:
- **No Druid/UI canvas** - Oracle manifests via haptic + AR stubs
- "Whisper overlays" - faint 3D text
- Haptic pulses as primary feedback
- Metal shaders for AR (not React)

#### Current Implementation:
- **Full React frontend** with panels, buttons, docks
- Multiple UI modes: ANALYZING, ORACLE, WALLET, NAVIGATION, AR_WORKSPACE
- Rich visual components (ChatInterface, HUD, CameraFeed)
- Haptic is simulated/stubbed

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| UI Approach | Minimal Oracle whispers | Full React UI | ❌ Architectural |
| Haptic | Real motor patterns | Simulated | ⚠️ Stub |
| AR Rendering | Metal shaders | Web-based | ❌ Different |
| Interaction | Voice/Gaze primary | Click/Type primary | ❌ Different |

---

### 5. ZK System - **GOOD FOUNDATION**

#### Target Architecture:
- Groth16 proofs for every intent
- ZK-sign before execution
- Proof verification before broadcast

#### Current Implementation:
- **Files**: `karana-core/src/zk/mod.rs`, `storage_proof.rs`
- Real Arkworks/Groth16 implementation ✅
- `prove_data_hash()` and `verify_proof()` working
- Batch proving implemented (`prove_batch()`)
- Keys cached to file

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Groth16 | Yes | Yes | ✅ Good |
| Intent Signing | Every intent | Only storage | ⚠️ Partial |
| Batch Proving | Yes | Yes | ✅ Good |
| Proof Verification | Pre-broadcast | Pre-storage | ⚠️ Partial |

---

### 6. Chain/Consensus - **PARTIAL**

#### Target Architecture:
- CometBFT for local transactions
- L3 bridge (Arbitrum testnet)
- Every intent attested on-chain

#### Current Implementation:
- Custom chain in `chain.rs` (not CometBFT)
- No L3 bridge
- Block production working
- Transaction signing with Ed25519 ✅

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Consensus | CometBFT | Custom impl | ⚠️ Different |
| L3 Bridge | Arbitrum | Not implemented | ❌ Missing |
| Intent Attest | All intents | Some intents | ⚠️ Partial |
| Ed25519 | Yes | Yes | ✅ Good |

---

### 7. Swarm/P2P - **GOOD**

#### Target Architecture:
- libp2p mesh
- Whisper relays
- KARA-earn offloads

#### Current Implementation:
- **File**: `karana-core/src/net.rs`
- libp2p with Gossipsub ✅
- Block broadcast working
- AI request offload working
- Echo tracking implemented

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| libp2p | Yes | Yes | ✅ Good |
| Block Relay | Yes | Yes | ✅ Good |
| AI Offload | Yes | Yes | ✅ Good |
| BLE Whispers | Yes | Not implemented | ❌ Missing |

---

### 8. Governance (DAO) - **GOOD FOUNDATION**

#### Target Architecture:
- AI-prompted DAO nudges
- Haptic votes (pulse = yes/no)
- KARA staking

#### Current Implementation:
- **Files**: `gov/mod.rs`, `economy.rs`
- DAO with proposals/voting working
- KARA token with staking
- AI governance analysis

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| DAO Structure | Yes | Yes | ✅ Good |
| AI Nudges | Whispered | Not integrated | ⚠️ Missing |
| Haptic Votes | Pulse = vote | Not implemented | ❌ Missing |
| KARA Staking | Yes | Yes | ✅ Good |

---

### 9. Hardware Interface - **PARTIAL**

#### Target Architecture:
- Rokid/XReal glasses
- NPU acceleration
- eBPF hooks for syscalls
- Real gaze camera, IMU, haptic motor

#### Current Implementation:
- **File**: `hardware/mod.rs`
- Simulation mode
- Haptic patterns defined
- HUD rendering stubs
- No real hardware integration

#### Gap Assessment:
| Feature | Target | Current | Status |
|---------|--------|---------|--------|
| Hardware Probing | eBPF | Simulated | ⚠️ Stub |
| NPU | Yes | CPU only | ❌ Missing |
| Haptic | Real motor | Simulated | ⚠️ Stub |
| IMU | Real | Not implemented | ❌ Missing |

---

### 10. AI Model - **DIFFERENT APPROACH**

#### Target:
- **Candle Phi-3 q4.gguf** (3.8B params)
- Multi-modal understanding
- Intent → Backend dialogue

#### Current:
- TinyLlama 1.1B (often fails to load)
- BERT embeddings for intent matching
- Rule-based fallback

#### Assessment:
The current approach uses **semantic embedding matching** which is actually more reliable for intent parsing than LLM generation. However, it's not the target Phi-3 model and lacks the conversational dialogue capability described in the plan.

---

## Summary: What's Working vs What Needs Change

### ✅ Working Well (Keep)
1. ZK proof system (Groth16/Arkworks)
2. Swarm networking (libp2p)
3. Storage with Merkle trees
4. Ed25519 wallet/signing
5. DAO/Governance structure
6. Semantic embedding for intent matching
7. Block production chain

### ⚠️ Needs Rewiring (Modify)
1. Oracle should be sole interface (hide UI)
2. ZK proofs should sign every intent
3. Monad execution should be Oracle-commanded
4. Haptic/AR output needs to replace UI panels

### ❌ Missing (Build New)
1. Multimodal input sense (gaze/voice fusion)
2. Metal AR shader rendering
3. Real Phi-3 model integration
4. CometBFT consensus
5. L3 bridge to Arbitrum
6. Real hardware drivers (Rokid SDK)
7. eBPF system hooks
8. BLE whisper networking

### ❌ Wrong Approach (Redesign)
1. **Full React UI** → Should be minimal Oracle whispers
2. **Frontend-driven intent** → Should be backend Oracle mediated
3. **Multiple UI modes** → Should be single Oracle conversation
4. **Visual buttons/panels** → Should be voice/gaze + haptic

---

## Recommended Path Forward

Given the significant architectural differences, I recommend:

### Option A: Pivot to Target Architecture (Major Rewrite)
- Build new `OracleVeil` as described
- Replace React UI with minimal AR whispers
- Implement multimodal input sense
- Wire all operations through Oracle → ZK → Monad

### Option B: Evolve Current Architecture (Incremental)
- Keep React simulator as **development/testing tool**
- Build true Oracle Veil for **production glasses**
- Use current backend (mostly good) with better Oracle integration
- Add ZK signing to all intent paths

### Recommendation: **Option B** (Phased Approach)
1. Keep simulator for development visibility
2. Build proper `OracleVeil` module
3. Add multimodal input processing
4. Wire ZK proofs through entire intent chain
5. Add minimal AR manifest for glasses-specific output
6. Eventually, glasses run only Oracle (no React)

---

*Analysis completed: December 3, 2025*
