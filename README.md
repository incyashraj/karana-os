# Kāraṇa OS (Kāraṇa-Core)

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

## 📜 The Philosophy: Questioning Everything

Kāraṇa OS is a **First Principles Rethink** of the operating system. It rejects the legacy metaphors of the 1970s (files, folders, permissions, applications) and replaces them with a sovereign, symbiotic architecture built for the age of AI and Zero-Knowledge Cryptography.

## 📜 The Philosophy: Questioning Everything

We started with a blank slate and a dangerous question: **"If we built an OS today, knowing what we know about AI, ZK-Proofs, and Decentralization, what would it look like?"**

### The Rejections
1.  **Reject "Files"**: Why do I have to manage files? The OS should manage **Data**, indexed semantically by AI and attested by ZK proofs.
2.  **Reject "Apps"**: Why are applications walled gardens? The OS should expose **Capabilities** (Intents) that can be composed dynamically.
3.  **Reject "Permissions"**: Why do I trust a checkbox? The OS should require **Proofs** (Vigil) for every state change.
4.  **Reject "Dumb Kernels"**: Why is the kernel passive? The OS should be an **Active Monad**, constantly optimizing itself and the user's intent.

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

## 🏗️ Architecture: The Sovereign Monad

Kāraṇa is not a kernel in the traditional sense. It is a **Userspace Monad** that runs on top of a minimal hardware abstraction (currently Linux, eventually a microkernel).

### Key Components
*   **The Monad (`src/monad.rs`)**: The central event loop that weaves all Atoms together. It handles the boot process, intent loop, and consensus.
*   **Symbiotic UI (`src/ui.rs`)**: A 4-panel TUI (Text User Interface) that visualizes the system's "thought process", blockchain state, and user intents.
*   **Bazaar (`src/market/mod.rs`)**: A decentralized app store where "apps" are just bundles of intents, verified by ZK proofs and ranked by DAO reputation.
*   **Hardware Abstraction (`src/hardware/mod.rs`)**: Native support for IoT and Smart Glasses (HUD, Lidar, Camera) treated as first-class intents.
*   **ZK-Engine (`src/zk/mod.rs`)**: Uses `arkworks` (Groth16) to generate cryptographic proofs for storage and execution.

---

## 🚀 Getting Started

### Prerequisites
*   Rust (latest stable)
*   Linux / macOS (or WSL2)
*   `build-essential` (for compiling dependencies)

### Installation
We provide a bundled installer script to set up the environment, build the core, and initialize the runtime directories.

```bash
git clone https://github.com/incyashraj/karana-os.git
cd karana-os
chmod +x install.sh
./install.sh
```

### Running the OS
After installation, use the launcher script:
```bash
./start_karana.sh
```

### The Symbiotic Interface
Once running, you will see the **4-Panel TUI**:
1.  **Header**: System Status (Balance, Block Height).
2.  **Symbiotic Interface**: The main output area (or App Window).
3.  **Consensus Stream**: Live logs of the blockchain and AI reasoning.
4.  **User Intent**: Your command input.

### Commands (Intents)
Kāraṇa understands natural language and specific commands:

*   **App Management**:
    *   `search app rust` -> Find apps in the Bazaar.
    *   `install rust-native-ide` -> ZK-Verify and install.
    *   `run terminal` -> Launch the sovereign terminal.
    *   `run files` -> Launch the file manager.

*   **Hardware (Smart Glass Mode)**:
    *   `hud on` -> Activate Head-Up Display.
    *   `record video` -> Start camera recording.
    *   `scan environment` -> Use Lidar/AI to detect objects.

*   **System**:
    *   `optimize storage` -> AI analyzes and compresses data.
    *   `boot` -> Simulate a full system boot sequence.
    *   `help` -> Ask the AI for guidance.

---

## 🛠️ Development History

### Phase v0.1 - v0.5: The Foundation
*   Built the **TUI** using `ratatui`.
*   Implemented the **Mock AI** (Phi-3 stub) to simulate intelligence without heavy GPU reqs.
*   Created the **Blockchain Ledger** and **DAO** logic.

### Phase v0.8: The Symbiotic Frontend
*   Refined the UI into the "Symbiotic" layout.
*   Added **Fuzzy Matching** for intents.

### Phase v1.0: Ecosystem Forge (Current)
*   **Real Apps**: Terminal and File Manager are now functional.
*   **Bazaar**: Implemented the ZK-verified app store.
*   **Persistence**: Added Btrfs-style snapshots with ZK proofs.
*   **IoT Support**: Added the Hardware Abstraction Layer for Smart Glasses.

---

## 🔮 Future Roadmap
*   **Sub-Phase 4**: Community DAO Dashboard (Tauri).
*   **Phase 2**: Migration to a bare-metal microkernel (Redox/seL4).
*   **Phase 3**: Full AR/VR Desktop Environment.

---

*"We do not build the OS to control the machine. We build the OS to free the mind."*
