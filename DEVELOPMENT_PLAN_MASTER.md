# Kāraṇa OS Development Plan - Master Document

## Version 2.0: Oracle Veil Implementation

### Document Index
1. `PLAN_ORACLE_VEIL.md` - Core Oracle AI Gateway Implementation
2. `PLAN_MULTIMODAL_INPUT.md` - Gaze/Voice/Gesture Input System
3. `PLAN_ZK_INTENT_CHAIN.md` - ZK-Proven Intent Pipeline
4. `PLAN_MINIMAL_MANIFEST.md` - AR Whispers & Haptic Output
5. `PLAN_BACKEND_INTEGRATION.md` - Monad Rewiring for Oracle
6. `PLAN_HARDWARE_GLASSES.md` - Rokid/XReal Hardware Integration
7. `PLAN_SPATIAL_AR_GLASSES.md` - **NEW** Persistent AR Tabs & Spatial Computing

---

## Development Philosophy

### Core Principle: AI-Centric Symbiotic Oracle
The user **never** interacts with the system directly. Everything flows through the Oracle:

```
User Intent (Voice/Gaze) → Oracle Parse → ZK-Sign → Monad Execute → Oracle Manifest
```

### Simulator vs Production
- **Simulator (React)**: Development & debugging tool - shows internal state
- **Production (Glasses)**: Pure Oracle interface - minimal AR + haptic

---

## Phase Overview

### Phase 1: Oracle Veil Core (Week 1-2)
Build the central `OracleVeil` struct that mediates all user↔backend communication.

**Deliverables:**
- `src/oracle/veil.rs` - Main Oracle struct
- `src/oracle/sense.rs` - Multimodal input processing
- `src/oracle/manifest.rs` - Minimal output rendering
- ZK-signed intent chain

### Phase 2: Multimodal Input (Week 2-3)
Implement gaze tracking, voice processing, and gesture recognition.

**Deliverables:**
- OpenCV gaze tracking
- Voice MFCC extraction (beyond transcription)
- IMU gesture detection
- Tensor fusion for combined understanding

### Phase 3: Backend Rewiring (Week 3-4)
Modify Monad to be commanded exclusively by Oracle via channels.

**Deliverables:**
- `tokio::mpsc` command channels
- Oracle-only execution API
- Remove direct user access to atoms

### Phase 4: Minimal Manifest (Week 4-5)
Replace UI panels with AR whispers and haptic patterns.

**Deliverables:**
- Metal shader for AR text overlays
- Haptic motor driver integration
- "Whisper" response formatting

### Phase 5: Hardware Integration (Week 5-6)
Connect to real Rokid/XReal glasses hardware.

**Deliverables:**
- Rokid SDK integration
- NPU acceleration for AI
- Real sensor drivers

---

## Architecture Target

```
┌─────────────────────────────────────────────────────────────────┐
│                     ORACLE VEIL (Sole Interface)                │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────┐   │
│  │ MultimodalSense│→│   Phi-3 AI    │→│  MinimalManifest  │   │
│  │ (Gaze/Voice)  │  │ (Intent Parse)│  │ (AR Whisper/Haptic│   │
│  └───────────────┘  └───────┬───────┘  └───────────────────┘   │
│                             │                                   │
│                      ┌──────▼──────┐                           │
│                      │  ZK-Sign    │                           │
│                      │  (Groth16)  │                           │
│                      └──────┬──────┘                           │
├─────────────────────────────┼───────────────────────────────────┤
│            BACKEND WEAVE    │    (Invisible to User)           │
│                      ┌──────▼──────┐                           │
│                      │   Monad     │                           │
│           ┌──────────┴──────────────┴──────────┐              │
│           │                                     │              │
│     ┌─────▼─────┐ ┌─────▼─────┐ ┌──────▼──────┐ ┌────▼────┐  │
│     │  Storage  │ │  Runtime  │ │    Swarm    │ │  Chain  │  │
│     └───────────┘ └───────────┘ └─────────────┘ └─────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria

### v2.0 Oracle Veil Release
- [ ] User can complete 10 different intents purely via voice
- [ ] All intents are ZK-proven before execution
- [ ] No visual UI - only AR whispers + haptic
- [ ] < 500ms intent-to-response latency
- [ ] Works on QEMU ARM64 emulator

### Metrics
| Metric | Target |
|--------|--------|
| Intent Success Rate | 95% |
| ZK Proof Time | < 200ms (batched) |
| Voice Recognition | > 90% accuracy |
| Haptic Response | < 50ms |
| Memory Usage | < 500MB |

---

## File Structure Changes

### New Files to Create:
```
karana-core/src/
├── oracle/
│   ├── mod.rs          (existing - update)
│   ├── veil.rs         (NEW - main OracleVeil)
│   ├── sense.rs        (NEW - multimodal input)
│   ├── manifest.rs     (NEW - AR/haptic output)
│   └── dialogue.rs     (NEW - conversation state)
├── zk/
│   ├── mod.rs          (existing - update)
│   └── intent_proof.rs (NEW - intent-specific proofs)
├── hardware/
│   ├── mod.rs          (existing - update)
│   ├── gaze.rs         (NEW - eye tracking)
│   ├── haptic.rs       (existing - enhance)
│   └── ar_render.rs    (NEW - Metal shaders)
```

### Files to Modify:
- `monad.rs` - Add Oracle command channels
- `api/handlers.rs` - Route all through Oracle
- `ai/mod.rs` - Add Phi-3 support

---

## Development Order

1. **Start with `oracle/veil.rs`** - Core struct that orchestrates everything
2. **Add ZK intent signing** - Modify `zk/mod.rs` for intent proofs
3. **Build `oracle/sense.rs`** - Multimodal input (start with voice-only)
4. **Wire Monad commands** - Channel-based Oracle→Monad communication
5. **Build `oracle/manifest.rs`** - Start with log output, add AR later
6. **Enhance `ai/mod.rs`** - Phi-3 model loading
7. **Hardware integration** - Gaze tracking, real haptic

---

## Immediate Next Steps

1. Review and approve this plan
2. Create `oracle/veil.rs` with the core `OracleVeil` struct
3. Modify `zk/mod.rs` to add `prove_intent()` function
4. Update `monad.rs` to add `tokio::mpsc` channels for Oracle commands
5. Test basic voice→Oracle→ZK→Monad→Response flow

---

*Master Plan v2.0 - December 3, 2025*
