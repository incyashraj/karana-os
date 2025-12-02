# Kāraṇa OS (Symbiotic Horizon)

```text
  _  __   _   ___   _   _  _   _   
 | |/ /  /_\ | _ \ /_\ | \| | /_\  
 | ' <  / _ \|   // _ \| .` |/ _ \ 
 |_|\_\/_/ \_\_|_/_/ \_\_|\_/_/ \_\
                                   
      The Sovereign AI-Native OS
```

> **"The Operating System is not a tool. It is a partner."**

## 🌟 What is Kāraṇa?
**Kāraṇa OS** is an experimental operating system designed for the post-app era. Unlike traditional systems (Windows, Linux, macOS) that force you to manage files and open applications, Kāraṇa is built around **Intents** and **Context**.

It is designed specifically for **Smart Glasses and IoT devices**, providing a "Symbiotic Interface" where the OS uses AI to understand your goals and Zero-Knowledge Proofs to secure your data. It doesn't just run programs; it thinks with you.

## 🏗️ Architecture: Distributed Symbiosis

Kāraṇa OS is split into two sovereign components:

1.  **Kāraṇa Core (`karana-core`)**: The headless kernel. It runs the Blockchain, AI Engine, ZK-Prover, and P2P Swarm. It can run on a server, a Raspberry Pi, or in the background of your desktop.
2.  **Kāraṇa Shell (`karana-shell`)**: The "Symbiotic Horizon" GUI. It is a native Rust application (using `druid`) that renders the Intent Orb, Adaptive Panels, and DAO Nudges. It connects to the Core via a local IPC socket.

---

## 🚀 Getting Started

### 1. Run the Core (The Brain)
The Core handles all logic and state. It must be running first.

```bash
cd karana-core
# Run in headless mode (starts IPC server on port 9000)
NO_TUI=1 cargo run
```

### 2. Run the Shell (The Face)
The Shell provides the graphical interface. **Note:** This requires a desktop environment with GTK libraries installed.

**Prerequisites (Local Machine):**
*   **Ubuntu/Debian**: `sudo apt install libgtk-3-dev libcairo2-dev libpango1.0-dev`
*   **macOS**: `brew install gtk+3 cairo pango`
*   **Fedora**: `sudo dnf install gtk3-devel cairo-devel pango-devel`

**Launch:**
```bash
cd karana-shell
cargo run
```

### 3. Symbiosis
Once both are running:
1.  Type **"code"** in the Shell -> The Core verifies the intent and the Shell spawns a Code Editor panel.
2.  Type **"tune battery"** -> The Core triggers a DAO proposal, and the Shell displays a "Vote YES" nudge.

---

## 📜 The Philosophy: Questioning Everything

Kāraṇa OS is a **First Principles Rethink** of the operating system. It rejects the legacy metaphors of the 1970s (files, folders, permissions, applications) and replaces them with a sovereign, symbiotic architecture built for the age of AI and Zero-Knowledge Cryptography.

### The Atom Analogy
We deconstructed the OS into **Atoms**—indivisible, verifiable units of reality.
*   **Atom 1 (Chain)**: The immutable timeline of truth.
*   **Atom 2 (Persist)**: The sovereign storage of state (ZK-Snapshots).
*   **Atom 3 (Intelligence)**: The AI brain (Phi-3) and Hardware senses.
*   **Atom 4 (Economy)**: The flow of value (DAO, Ledger).
*   **Atom 5 (Runtime)**: The execution environment (Actors).
*   **Atom 6 (Interface)**: The symbiotic feedback loop (TUI/HUD).
*   **Atom 7 (Vigil)**: The immune system (Security/Slashing).

---

## 🛠️ Development History

### Phase v0.1 - v0.5: The Foundation
*   Built the **TUI** using `ratatui`.
*   Implemented the **Mock AI** (Phi-3 stub) to simulate intelligence without heavy GPU reqs.
*   Created the **Blockchain Ledger** and **DAO** logic.

### Phase v0.8: The Symbiotic Frontend
*   Refined the UI into the "Symbiotic" layout.
*   Added **Fuzzy Matching** for intents.

### Phase v1.0: Ecosystem Forge
*   **Real Apps**: Terminal and File Manager are now functional.
*   **Bazaar**: Implemented the ZK-verified app store.
*   **Persistence**: Added Btrfs-style snapshots with ZK proofs.
*   **IoT Support**: Added the Hardware Abstraction Layer for Smart Glasses.

### Phase v2.0: Symbiotic Horizon (Current)
*   **Split Architecture**: Decoupled Core and Shell.
*   **GUI Shell**: Implemented `karana-shell` with Druid (Orb, Panels, Nudge).
*   **IPC Layer**: TCP bridge between Core and Shell.

## 🕶️ Hardware & Vision
Kāraṇa OS is designed for a "Split-Architecture" wearable future.
*   **Display**: Smart Glasses (XREAL/Rokid) acting as a dumb terminal.
*   **Compute**: A belt-worn "Puck" (Orange Pi 5 / RK3588) running the Core.

👉 **[Read the Full Hardware Plan](./HARDWARE_PLAN.md)** for recommended dev kits and the roadmap to a "State of the Art" device.

---

*"We do not build the OS to control the machine. We build the OS to free the mind."*
