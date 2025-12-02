# Kāraṇa Shell (Symbiotic Horizon)

This is the GUI frontend for Kāraṇa OS. It connects to the `karana-core` kernel via TCP IPC.

## Prerequisites
You need a system with GTK/Cairo libraries installed (Linux/macOS/Windows).
- **Ubuntu/Debian**: `sudo apt install libgtk-3-dev libcairo2-dev libpango1.0-dev`
- **Fedora**: `sudo dnf install gtk3-devel cairo-devel pango-devel`
- **macOS**: `brew install gtk+3 cairo pango`

## How to Run
1. Ensure `karana-core` is running (locally or on a remote server).
   - If remote, update `src/client.rs` to point to the correct IP (default: `127.0.0.1:9000`).
2. Run the shell:
   ```bash
   cargo run
   ```

## Features (Phase 8)
- **Intent Orb**: Glowing, pulsing interface for AI interaction.
- **Adaptive Panels**: ZK-verified content cards (Code, Graphs).
- **DAO Nudge**: Governance overlays for system decisions.
- **IPC Client**: Sends natural language intents ("code", "tune battery") to the core.
