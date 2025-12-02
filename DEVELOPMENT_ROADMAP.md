# Kāraṇa OS: Development Roadmap v0.7 "Intent Engine"

> **Goal**: Transform "registers but no output" into "real end-to-end action"
> **Timeline**: 4-6 weeks (~100 hours)
> **Target**: Doubt-free testing on emulator, ready for glasses deployment

---

## Current State Assessment

| Component | Status | Issue |
|-----------|--------|-------|
| Monad Core | ✅ Compiles | Stubs echo without action |
| AI (Candle) | ✅ Loads | Predictions are placeholder text |
| ZK (Arkworks) | ✅ Proves | Not wired to real data flow |
| Swarm (libp2p) | ✅ Boots | No real peer broadcast |
| Storage | ✅ RocksDB | Writes don't reflect in UI |
| UI (Druid/Ratatui) | ✅ Renders | Shows intent, not result |
| Chain (CometBFT) | ⚠️ Stub | No real tx attestation |

**Diagnosis**: The monad logs intents but atoms are placeholders. Fix: Wire real I/O.

---

## Phase 7.1: Flush Core Stubs (Week 1, Day 1-2)
**Category**: Integration/Core
**Effort**: 8 hours

### Objective
Replace echo stubs with real file I/O and process execution.

### Implementation Checklist
- [ ] `storage.write()` → Real file to `/tmp/karana-*`
- [ ] `runtime.execute()` → Real `std::process::Command`
- [ ] `monad.ignite()` → Chain all atom calls with real output
- [ ] Add structured logging for each atom action

### Success Gate
```bash
# Input
> tune battery

# Expected Output (logs + file)
[STORAGE] Written: /tmp/karana-battery.conf (23 bytes)
[RUNTIME] Executed: echo "governor=powersave"
[MONAD] Intent "tune battery" completed in 45ms
```

---

## Phase 7.2: AI Action Engine (Week 1, Day 3-4)
**Category**: AI
**Effort**: 12 hours

### Objective
Make Phi-3/TinyLlama output structured JSON actions that the monad can execute.

### Implementation Checklist
- [ ] Prompt engineering: Force JSON output `{"action": "...", "value": "..."}`
- [ ] Parse AI response with `serde_json`
- [ ] Map actions to runtime commands
- [ ] Fallback for unparseable responses

### Action Schema
```json
{
  "action": "set_config",
  "target": "power.governor",
  "value": "powersave",
  "confidence": 0.92
}
```

### Success Gate
```bash
# Input
> tune battery for low power

# AI Output (parsed)
[AI] Predicted action: set_config(power.governor, powersave) @ 92% confidence
[RUNTIME] Applied: /tmp/karana-power.conf updated
```

---

## Phase 7.3: Swarm Real Relay (Week 2, Day 1-2)
**Category**: Communication
**Effort**: 10 hours

### Objective
libp2p broadcasts to real peers (emulator nodes) with echo confirmation.

### Implementation Checklist
- [ ] Setup 3-node local testnet (different ports)
- [ ] Gossipsub topic `/karana/intents/1.0.0`
- [ ] Broadcast intent completion with ZK proof
- [ ] Track peer acknowledgments

### Test Setup
```bash
# Terminal 1 (Node A)
KARANA_P2P_PORT=4001 cargo run

# Terminal 2 (Node B)
KARANA_P2P_PORT=4002 KARANA_PEER=/ip4/127.0.0.1/tcp/4001 cargo run

# Terminal 3 (Node C)
KARANA_P2P_PORT=4003 KARANA_PEER=/ip4/127.0.0.1/tcp/4001 cargo run
```

### Success Gate
```
[SWARM] Broadcast "tune battery" to /karana/intents
[SWARM] Ack from peer 12D3KooW...abc (Node B) in 45ms
[SWARM] Ack from peer 12D3KooW...def (Node C) in 52ms
[SWARM] Consensus: 3/3 nodes received intent
```

---

## Phase 7.4: UI Output Flush (Week 2, Day 3-4)
**Category**: UI/Frontend
**Effort**: 12 hours

### Objective
Panels show real data from storage; haptic feedback on completion.

### Implementation Checklist
- [ ] Read `/tmp/karana-*` files for panel content
- [ ] Graph widget for numeric data (battery %, shard allocation)
- [ ] Haptic simulation (log "VIBRATE" for emulator, real GPIO for glasses)
- [ ] Toast notifications for intent completion

### UI States
```
┌─────────────────────────────────┐
│ ⚡ INTENT COMPLETED             │
├─────────────────────────────────┤
│ Action: tune battery            │
│ Result: governor=powersave      │
│ Proof:  0x7a2f...8e1c (ZK ✓)   │
│ Peers:  3/3 acknowledged        │
│ Chain:  Tx 0xabc...def          │
├─────────────────────────────────┤
│ [HAPTIC] ━━━━━━━━━━ 100ms pulse │
└─────────────────────────────────┘
```

### Success Gate
- Panel renders real config file content
- "Haptic: Success pulse 100ms" logged
- < 500ms from intent to render

---

## Phase 7.5: Chain Attestation (Week 3, Day 1-2)
**Category**: Blockchain
**Effort**: 8 hours

### Objective
CometBFT records intent completions with ZK proofs.

### Implementation Checklist
- [ ] `broadcast_tx_commit` to local CometBFT
- [ ] Tx payload: `{intent, proof_hash, timestamp}`
- [ ] Query block explorer for confirmation
- [ ] L3 bridge stub (Alloy RPC to Arbitrum testnet)

### Tx Format
```json
{
  "intent": "tune battery",
  "proof": "0x7a2f...8e1c",
  "actor": "did:karana:user123",
  "timestamp": 1733100000
}
```

### Success Gate
```
[CHAIN] Tx broadcast: 0xdef...789
[CHAIN] Included in block #1234 (karana-devnet)
[CHAIN] Finality: 2/3 validators signed
```

---

## Phase 7.6: ZK Real Proofs (Week 3, Day 3)
**Category**: Blockchain/Security
**Effort**: 6 hours

### Objective
Generate real Groth16 proofs for intent data (not just stubs).

### Implementation Checklist
- [ ] Circuit: Prove `hash(intent_data) == commitment`
- [ ] Batch proving (queue intents, prove every 5s)
- [ ] Proof verification before broadcast
- [ ] Cache proving keys (avoid 5s cold start)

### Performance Target
| Metric | Current | Target |
|--------|---------|--------|
| Proof Gen | 5s | 200ms (batched) |
| Verify | 50ms | 10ms |
| Proof Size | 256B | 128B (compressed) |

### Success Gate
- Proof generated for every intent
- Verification logged before chain tx
- < 500ms total ZK overhead

---

## Phase 7.7: End-to-End Integration (Week 4)
**Category**: Integration/Core
**Effort**: 15 hours

### Objective
Wire all atoms into seamless flow: Input → AI → ZK → Storage → Swarm → Chain → UI

### The Complete Flow
```
USER INPUT          "tune battery"
     ↓
[AI PARSE]          → {"action": "set_config", "value": "powersave"}
     ↓
[ZK PROVE]          → Proof: 0x7a2f...8e1c
     ↓
[STORAGE WRITE]     → /tmp/karana-battery.conf
     ↓
[SWARM BROADCAST]   → 3/3 peers ack
     ↓
[CHAIN ATTEST]      → Tx 0xdef...789 in block #1234
     ↓
[UI RENDER]         → Panel: "Tuned ✓" + Haptic pulse
     ↓
[TOTAL TIME]        → 1.2s end-to-end
```

### Integration Test Script
```bash
#!/bin/bash
# test_e2e.sh

# Start 3-node swarm
KARANA_P2P_PORT=4001 cargo run &
KARANA_P2P_PORT=4002 cargo run &
KARANA_P2P_PORT=4003 cargo run &

# Wait for boot
sleep 5

# Send intent via IPC
echo "tune battery" | nc localhost 9000

# Verify outputs
cat /tmp/karana-battery.conf        # Should exist
curl localhost:26657/status         # Should show new block
grep "Haptic" /tmp/karana.log       # Should show pulse
```

### Success Gate
- 100% intent completion rate
- < 3s end-to-end latency
- All atoms produce real output
- Zero stub echoes remaining

---

## Phase 7.8: Emulator Testing (Week 4)
**Category**: Testing
**Effort**: 10 hours

### Objective
QEMU + ADB simulation for glasses-like environment.

### Setup
```bash
# QEMU ARM64 with simulated sensors
qemu-system-aarch64 \
  -M virt -cpu cortex-a72 -m 4G \
  -kernel Image -initrd rootfs.cpio \
  -append "console=ttyAMA0" \
  -device virtio-gpu-pci \
  -netdev user,id=net0,hostfwd=tcp::4001-:4001 \
  -device virtio-net-pci,netdev=net0

# Inside QEMU
./karana-core --glasses-mode
```

### Simulated Hardware
| Real Hardware | Emulator Substitute |
|---------------|---------------------|
| Eye Tracker | Mouse position → Gaze |
| IMU | Random acceleration |
| Microphone | WAV file input |
| Haptic Motor | Log "VIBRATE" |
| AR Display | X11 window |

### Success Gate
- Full boot in QEMU < 10s
- Intent loop works identically to host
- Battery/power simulation active

---

## Success Metrics: v0.7 "Intent Engine"

| Metric | Target | Measurement |
|--------|--------|-------------|
| Intent Success Rate | 100% | 50 test intents, 0 failures |
| End-to-End Latency | < 3s | Timer from input to haptic |
| AI Parse Accuracy | 80% | 20 varied prompts |
| Swarm Delivery | 90% | 3-node testnet |
| Chain Finality | 100% | All intents attested |
| Memory Usage | < 2GB | `htop` during load |
| Battery Sim Drain | < 0.5%/min | Simulated metric |

---

## Post v0.7: Glasses Deployment Path

### Hardware Acquisition
1. **XREAL Air 2 Pro** ($450) - Display
2. **Orange Pi 5 Plus 16GB** ($140) - Compute
3. **USB-C Cable** - Link
4. **65W PD Power Bank** - Power

### Deployment Steps
```bash
# 1. Flash Orange Pi with Armbian
sudo dd if=armbian.img of=/dev/sdX bs=4M

# 2. Clone and build
git clone https://github.com/user/karana-os
cd karana-os/karana-core
cargo build --release --target aarch64-unknown-linux-gnu

# 3. Configure display output
export DISPLAY=:0  # USB-C DisplayPort to glasses

# 4. Run
./target/release/karana-core --glasses-mode
```

### First Glasses Test
1. Boot Orange Pi with glasses connected
2. See Karana HUD in AR view
3. Type "help" → AI responds with tutorial
4. Type "tune battery" → Full E2E flow
5. Feel haptic pulse (once GPIO wired)

---

## Development Commands

```bash
# Run with full logging
RUST_LOG=debug cargo run

# Run without TUI (headless/IPC mode)
NO_TUI=1 cargo run

# Run with simulated glasses mode
KARANA_GLASSES_SIM=1 cargo run

# Run integration tests
cargo test --test integration

# Check for stub remnants
grep -r "TODO\|STUB\|placeholder" src/
```

---

*"From stubs to sovereignty. Every intent, a proven action."*
