# Kāraṇa OS: AR/Smart Glasses Evolution Plan

> **Vision**: A sovereign, symbiotic operating system for the post-smartphone era. Kāraṇa OS on Smart Glasses transforms the device from a passive notification screen into an active cognitive extension, secured by Zero-Knowledge proofs and governed by a decentralized swarm.

## 🗺️ Roadmap Overview

This roadmap evolves the current v1.0 prototype (Linux-based Userspace Monad) into v2.0 (AR-Native Firmware).

| Phase | Focus | Key Deliverables | Status |
|-------|-------|------------------|--------|
| **1** | **IoT & Power** | Power Management Atom, Battery AI, Mesh Networking Prep. | **COMPLETED** |
| **2** | **Multimodal I/O** | Eye-tracking (sim), Voice Command (STT), Haptic Feedback. | **COMPLETED** |
| **3** | **AI Co-Pilot** | Context-aware AR overlays, Vision (Image Captioning). | **IN PROGRESS** |
| **4** | **Privacy & Identity** | DID (Decentralized ID), ZK-Signatures for glances. | Pending |
| **5** | **Ecosystem** | Cross-device sync (Phone <-> Glass), AR Bazaar. | Pending |
| **6** | **Production** | Hardware certification, Beta fleet, Public Launch. | Pending |

> **CRITICAL MANDATE**: All features must implement **Real Functionality** where possible. Simulations are only acceptable if hardware is physically absent (e.g., Eye Tracker), but even then, the *logic* must be production-ready (e.g., processing real audio files for Voice, reading real system battery for Power).

---

## 🛠️ Phase 1: IoT Low-Power Adaptation (Completed)

**Objective**: Transform the "Desktop Monad" into an efficient "Eternal Node" suitable for battery-powered devices.

### 1.1 Power Management Atom (`src/hardware/power.rs`)
*   **Status**: Implemented.
*   **Features**:
    *   `PowerManager` struct with profiles (Performance, Balanced, LowPower, Hibernation).
    *   **Real Implementation**: Uses `sysinfo` to read host battery state (if available). Falls back to simulation only if no battery is detected.
    *   Integrated into `KaranaHardware` and UI Header.

### 1.2 Battery AI Prediction
*   **Status**: Partially Implemented (via Power Profiles).
*   **Logic**: System now exposes power state to the AI context.

---

## 👁️ Phase 2: Multimodal Core (Completed)

**Objective**: Replace keyboard/mouse with Gaze, Voice, and Gesture.

### 2.1 Gaze Tracking (Real Mouse Proxy)
*   **Status**: Implemented (`src/hardware/input.rs`, `src/ui.rs`).
*   **Logic**: 
    *   `MultimodalInput` struct tracks `Gaze(x, y)`.
    *   **Real Implementation**: Captures `Event::Mouse` from the terminal to update gaze coordinates in real-time.
    *   UI Header displays real-time Gaze coordinates.

### 2.2 Voice Command
*   **Status**: Implemented (`src/ai/mod.rs`).
*   **Stack**: `candle-transformers` (Whisper) + `symphonia`.
*   **Real Functionality**: 
    *   `transcribe <file>` command reads real audio files.
    *   Runs Whisper Tiny.en inference on CPU.
    *   Auto-downloads model on first use.

### 2.3 Haptic Feedback
*   **Status**: Implemented (`src/hardware/haptic.rs`).
*   **Stack**: `evdev` crate (Linux Input Subsystem).
*   **Real Functionality**:
    *   Scans `/dev/input/event*` for devices with `FORCEFEEDBACK` capability.
    *   Sends real Rumble effects if hardware is present.
    *   Gracefully degrades to logging if no haptic device is found.

---

## 🧠 Phase 3: AI Co-Pilot (In Progress)

**Objective**: The OS thinks *before* you act.

### 3.1 Vision / Context Awareness
*   **Status**: In Progress.
*   **Stack**: `candle-transformers` (BLIP - Bootstrapping Language-Image Pre-training).
*   **Real Functionality**:
    *   `analyze <image_path>` command.
    *   Loads real image files.
    *   Generates a caption describing the scene (e.g., "a person holding a coffee cup").
    *   This serves as the "Context" for the AR Overlay.

### 3.2 AR Compositor
*   **Status**: Planned.
*   **UI**: Migrate TUI concepts to a graphical overlay (using `wgpu` or `druid`).
*   **HUD**: Heads-Up Display for vital stats (Time, Battery, Next Meeting).

---

## 🛠️ Phase 1: IoT Low-Power Adaptation (Current Focus)

**Objective**: Transform the "Desktop Monad" into an efficient "Eternal Node" suitable for battery-powered devices.

### 1.1 Power Management Atom (`src/hardware/power.rs`)
*   **Concept**: A dedicated actor that monitors energy consumption and enforces power profiles.
*   **Profiles**:
    *   `Performance`: Full AI inference, high refresh rate (Plugged in).
    *   `Balanced`: Standard operation (Active usage).
    *   `LowPower`: Reduced polling, dimmed AR, batched proofs (Idle).
    *   `Hibernation`: RAM-to-Disk, wake-on-interrupt only (<5% battery).
*   **Implementation**:
    *   Use `sysinfo` to read actual battery state (on laptops/Linux devices).
    *   Implement `PowerManager` struct to switch profiles dynamically.

### 1.2 Battery AI Prediction
*   **Concept**: Use the internal AI (Phi-3/Rule-based) to predict battery drain based on current intents.
*   **Logic**: `predict_drain(active_apps, screen_brightness, network_load) -> TimeRemaining`.
*   **Action**: If `TimeRemaining < UserGoal`, suggest closing background atoms.

### 1.3 Mesh Networking Prep
*   **Concept**: Glasses should not rely solely on Wi-Fi. They must form ad-hoc meshes.
*   **Implementation**:
    *   Enhance `KaranaSwarm` to support peer discovery optimization based on power state.
    *   (Future) Add Bluetooth Low Energy (BLE) transport.

---

## 👁️ Phase 2: Multimodal Core (Next)

**Objective**: Replace keyboard/mouse with Gaze, Voice, and Gesture.

### 2.1 Gaze Tracking (Simulated)
*   **Input**: WebCam stream or Mouse position as proxy for "Gaze".
*   **Logic**: "Dwelling" on an item for >500ms triggers a "Select" intent.

### 2.2 Voice Command
*   **Stack**: `whisper-rs` or `coqui-stt` for local speech-to-text.
*   **Wake Word**: "Hey Kāraṇa" triggers the intent loop.

### 2.3 AR Compositor
*   **UI**: Migrate TUI concepts to a graphical overlay (using `wgpu` or `druid`).
*   **HUD**: Heads-Up Display for vital stats (Time, Battery, Next Meeting).

---

## 🧠 Phase 3: AI Co-Pilot

**Objective**: The OS thinks *before* you act.

*   **Context Awareness**: Analyze video feed (simulated) to identify objects.
*   **Proactive Intents**: "I see you are looking at a QR code. Scan it?"

---

## 🔒 Phase 4: Privacy & Identity

**Objective**: You own your biometrics.

*   **ZK-Biometrics**: Prove "I am the user" without sending retina scan data to the cloud.
*   **DID**: Self-Sovereign Identity for logging into dApps via AR.

---

## 🔄 Phase 5: Ecosystem

**Objective**: Seamless handoff.

*   **Universal Clipboard**: Copy on Glass, Paste on Phone.
*   **App Streaming**: Render heavy apps on PC, stream pixels to Glass.

---

## 🚀 Phase 6: Launch

**Objective**: Hardware integration.

*   **Target Hardware**: Raspberry Pi Zero 2 W (Dev Kit), Rockchip RK3588 (High End).
*   **Distribution**: Flashing tools, OTA updates.
