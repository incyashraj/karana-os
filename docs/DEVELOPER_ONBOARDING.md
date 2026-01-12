# Developer Onboarding Guide

Welcome to Kāraṇa OS development! This guide will get you from zero to productive contributor.

## 📋 Table of Contents

1. [Prerequisites & System Requirements](#prerequisites--system-requirements)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Structure](#project-structure)
4. [Building & Running](#building--running)
5. [Testing Strategy](#testing-strategy)
6. [Development Workflow](#development-workflow)
7. [Debugging Tips](#debugging-tips)
8. [Common Issues & Solutions](#common-issues--solutions)
9. [Architecture Deep Dive](#architecture-deep-dive)
10. [Contributing Guidelines](#contributing-guidelines)

---

## Prerequisites & System Requirements

### Required Software

```bash
# 1. Rust (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should be 1.70 or higher

# 2. Node.js (18+) for simulator
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
node --version  # Should be 18.x or higher

# 3. Git
sudo apt install git
```

### Platform-Specific Dependencies

#### Ubuntu/Debian
```bash
# Core development tools
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# GTK/Cairo (for karana-shell GUI)
sudo apt install -y libgtk-3-dev libcairo2-dev libpango1.0-dev

# V4L2 (for real camera support)
sudo apt install -y libv4l-dev v4l-utils

# Optional: Camera testing
sudo apt install -y cheese guvcview
```

#### Fedora/RHEL
```bash
sudo dnf install -y gcc gcc-c++ pkg-config openssl-devel
sudo dnf install -y gtk3-devel cairo-devel pango-devel
sudo dnf install -y libv4l-devel v4l-utils
```

#### macOS
```bash
brew install gtk+3 cairo pango pkg-config openssl
```

### Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2 cores | 4+ cores (Rust compilation is parallel) |
| **RAM** | 4 GB | 8+ GB (16 GB for full stack dev) |
| **Storage** | 10 GB | 20+ GB (Rust toolchain + dependencies + cache) |
| **Camera** | Any USB/integrated | HD camera for AR testing |
| **GPU** | Optional | Recommended for AR rendering |

### Optional Tools

```bash
# Code formatting
rustup component add rustfmt
rustup component add clippy

# Documentation generation
cargo install cargo-watch  # Auto-rebuild on file changes
cargo install cargo-tree   # Dependency visualization
cargo install cargo-bloat  # Binary size analysis
```

---

## Development Environment Setup

### 1. Clone the Repository

```bash
git clone https://github.com/incyashraj/karana-os.git
cd karana-os
```

### 2. Verify Project Structure

```bash
tree -L 2 -I 'target|node_modules'
```

Expected output:
```
karana-os/
├── karana-core/          # Main Rust system (186K+ LOC)
│   ├── src/
│   ├── examples/
│   └── Cargo.toml
├── karana-shell/         # GTK GUI client
├── simulator/            # React-based AR simulator (deprecated)
├── kāraṇa-os-simulator/  # New Vite-based simulator
├── karana-cache/         # RocksDB state (generated)
├── karana-governance/    # DAO state (generated)
├── karana-ledger/        # Blockchain state (generated)
├── docs/                 # Technical documentation (12 files)
└── README.md
```

### 3. Build Core System

```bash
# Development build (faster compilation, debug symbols)
cd karana-core
cargo build

# Release build (optimized, slower compilation)
cargo build --release

# With V4L2 camera support (Linux only)
cargo build --features v4l2
```

**Build Time Expectations:**
- First build: 5-15 minutes (downloads + compiles 200+ dependencies)
- Incremental builds: 5-30 seconds (only changed modules)
- Clean release build: 10-20 minutes

### 4. Run Test Suite

```bash
# Run all tests (2295+ tests)
cargo test --lib

# Run specific module tests
cargo test --lib blockchain
cargo test --lib ai
cargo test --lib oracle

# Run with output
cargo test -- --nocapture

# Run tests in parallel (faster)
cargo test --lib -- --test-threads=8
```

### 5. Setup Simulator

```bash
cd ../kāraṇa-os-simulator
npm install
npm run dev  # Starts at http://localhost:3000
```

### 6. Start API Server (Backend)

```bash
cd ../karana-core
cargo run --bin karana-api-server

# Or with custom port
PORT=8080 cargo run --bin karana-api-server
```

**Full Stack Setup:**
- **Backend**: `http://localhost:8080` (Rust API server)
- **Frontend**: `http://localhost:3000` (React simulator)

---

## Project Structure

### High-Level Organization

```
karana-os/
├── karana-core/src/          # Core Rust implementation (186K+ LOC)
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── monad.rs             # Main system orchestrator
│   ├── api/                 # HTTP/WebSocket API server
│   ├── ai/                  # NLU, dialogue, reasoning (Layer 6)
│   ├── ar/                  # AR tabs, spatial anchors (Layers 5, 7)
│   ├── blockchain/          # Chain, consensus, wallet (Layer 3)
│   ├── camera/              # Hardware camera abstraction (Layer 1)
│   ├── celestia/            # Data availability integration
│   ├── consensus/           # PoS Tendermint consensus
│   ├── dao/                 # Governance system
│   ├── embedding/           # Semantic embeddings (BERT, CLIP)
│   ├── gesture/             # Hand tracking, gesture recognition
│   ├── hardware/            # Hardware abstraction layer
│   ├── hud/                 # Heads-up display rendering
│   ├── installer/           # OS installation wizard
│   ├── intelligence/        # Computer vision, scene understanding (Layer 5)
│   ├── market/              # App bazaar (dApp store)
│   ├── network/             # libp2p P2P networking (Layer 2)
│   ├── nlu/                 # Natural language understanding
│   ├── onboarding/          # First-time user setup
│   ├── oracle/              # Oracle Bridge Layer (Layer 4)
│   ├── ota/                 # Over-the-air updates
│   ├── pkg/                 # App package manager
│   ├── proactive/           # Proactive suggestions
│   ├── security/            # Security hardening
│   ├── slam/                # Simultaneous localization and mapping
│   ├── system_services/     # Background services (Layer 9)
│   ├── timer/               # Timer app
│   ├── ui/                  # UI abstractions
│   ├── ux/                  # User experience (personas, hints)
│   ├── voice/               # Voice UI (wake word, VAD, TTS)
│   ├── wallet/              # Crypto wallet
│   └── zk/                  # Zero-knowledge proofs
│
├── karana-core/examples/     # Runnable examples
│   ├── full_stack_demo.rs   # Complete system demo
│   ├── oracle_demo.rs       # Oracle system demo
│   └── zk_demo.rs           # ZK proof demo
│
├── karana-core/src/bin/      # Binary executables
│   └── karana_api_server.rs # HTTP API server
│
├── docs/                     # Technical documentation
│   ├── README.md            # Documentation index
│   ├── LAYER_1_HARDWARE.md  # Hardware abstraction
│   ├── LAYER_2_NETWORK.md   # P2P networking
│   ├── LAYER_3_BLOCKCHAIN.md # Blockchain layer
│   ├── LAYER_4_ORACLE.md    # Oracle bridge
│   ├── LAYER_5_INTELLIGENCE.md # Computer vision
│   ├── LAYER_6_AI_ENGINE.md # AI engine
│   ├── LAYER_7_INTERFACE.md # User interface
│   ├── LAYER_8_APPLICATIONS.md # Applications
│   ├── LAYER_9_SYSTEM_SERVICES.md # System services
│   ├── AR_TRACKING_SYSTEM.md # AR tracking
│   ├── UNIVERSAL_ORACLE_AI.md # Oracle AI
│   └── ZERO_KNOWLEDGE_PROOFS.md # ZK proofs
│
└── kāraṇa-os-simulator/      # Web-based AR simulator
    ├── src/
    │   ├── App.tsx          # Main simulator app
    │   ├── components/      # React components
    │   └── services/        # API client, Gemini integration
    └── package.json
```

### Key Files to Know

| File | Purpose | When to Edit |
|------|---------|--------------|
| `karana-core/src/monad.rs` | System orchestrator, boots all layers | Adding new system components |
| `karana-core/src/api/mod.rs` | HTTP/WebSocket API routes | Adding new API endpoints |
| `karana-core/src/ai/mod.rs` | AI engine (NLU, dialogue, reasoning) | Improving AI responses |
| `karana-core/src/oracle/mod.rs` | Oracle Bridge Layer | Adding new tools/capabilities |
| `karana-core/src/blockchain/chain.rs` | Blockchain core logic | Modifying consensus or block structure |
| `karana-core/src/network/gossip.rs` | P2P message routing | Changing network topology |
| `karana-core/Cargo.toml` | Rust dependencies | Adding new crates |

---

## Building & Running

### Development Modes

#### 1. Core System Only (CLI)

```bash
cd karana-core
cargo run
```

This starts the full Kāraṇa OS kernel with:
- P2P networking on port 9000
- Blockchain with RocksDB persistence
- AI engine with simulated models
- API server on port 8080

#### 2. API Server (Backend for Simulator)

```bash
cd karana-core
cargo run --bin karana-api-server

# Custom port
PORT=8080 cargo run --bin karana-api-server
```

#### 3. Full Stack (Backend + Frontend)

**Terminal 1 (Backend):**
```bash
cd karana-core
cargo run --bin karana-api-server
```

**Terminal 2 (Frontend):**
```bash
cd ../kāraṇa-os-simulator
npm run dev
```

Open browser: `http://localhost:3000`

#### 4. Run Examples

```bash
cd karana-core

# Full system demo
cargo run --example full_stack_demo

# Oracle system demo
cargo run --example oracle_demo

# ZK proof demo
cargo run --example zk_demo
```

### Build Profiles

```bash
# Debug build (fast compile, slow runtime, debug symbols)
cargo build

# Release build (slow compile, fast runtime, optimized)
cargo build --release

# With V4L2 camera support
cargo build --features v4l2

# Check without building (fast syntax check)
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Environment Variables

```bash
# Logging levels
export RUST_LOG=info           # info, debug, trace, warn, error
export RUST_LOG=karana_core=debug,libp2p=info

# API server port
export PORT=8080

# Celestia DA (optional)
export CELESTIA_AUTH_TOKEN=your_token
export CELESTIA_NODE_URL=http://localhost:26658

# Gemini API (for simulator)
export VITE_GEMINI_API_KEY=your_key
```

---

## Testing Strategy

### Unit Tests

```bash
# Run all tests
cargo test --lib

# Specific module
cargo test --lib blockchain::tests
cargo test --lib ai::tests::test_intent_classification

# Show output
cargo test -- --nocapture

# Parallel execution (faster)
cargo test --lib -- --test-threads=8
```

### Integration Tests

```bash
# Run examples as integration tests
cargo run --example full_stack_demo
cargo run --example oracle_demo
```

### Performance Testing

```bash
# Build in release mode
cargo build --release

# Run benchmarks (if available)
cargo bench

# Profile binary size
cargo install cargo-bloat
cargo bloat --release
```

### Test Coverage

Current test coverage:
- **Total Tests**: 2295+ passing
- **Blockchain**: 300+ tests (consensus, wallet, chain)
- **AI**: 200+ tests (NLU, dialogue, reasoning)
- **Oracle**: 150+ tests (intent processing, ZK proofs)
- **Network**: 180+ tests (P2P, gossip, discovery)
- **AR**: 120+ tests (spatial anchors, SLAM, tabs)

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }

    #[tokio::test]  // For async tests
    async fn test_async_example() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

---

## Development Workflow

### 1. Daily Development Cycle

```bash
# 1. Pull latest changes
git pull origin master

# 2. Create feature branch
git checkout -b feature/your-feature-name

# 3. Make changes
# Edit files...

# 4. Format code
cargo fmt

# 5. Run linter
cargo clippy

# 6. Run tests
cargo test --lib

# 7. Commit changes
git add .
git commit -m "feat: add new feature"

# 8. Push to remote
git push origin feature/your-feature-name
```

### 2. Adding a New Feature

**Example: Adding a new AI tool**

1. **Define the tool interface** (`karana-core/src/oracle/tools.rs`):
```rust
pub struct MyNewTool {
    name: String,
}

impl Tool for MyNewTool {
    fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        // Implementation
        Ok(ToolResult {
            success: true,
            data: json!({"message": "Tool executed"}),
            error: None,
        })
    }
}
```

2. **Register the tool** (`karana-core/src/oracle/mod.rs`):
```rust
pub fn register_default_tools(registry: &mut ToolRegistry) {
    registry.register(Box::new(MyNewTool::new()));
}
```

3. **Add tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_my_new_tool() {
        let tool = MyNewTool::new();
        let result = tool.execute(&params);
        assert!(result.is_ok());
    }
}
```

4. **Update documentation** (`docs/LAYER_4_ORACLE.md`):
```markdown
### My New Tool
Description of what the tool does...
```

5. **Run tests and commit**:
```bash
cargo test --lib oracle
git commit -m "feat(oracle): add MyNewTool for X functionality"
```

### 3. Hot Reload Development

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on file changes
cargo watch -x run

# Auto-test on changes
cargo watch -x test
```

---

## Debugging Tips

### 1. Enable Debug Logging

```bash
# Full debug output
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=karana_core::ai=debug cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run
```

### 2. Using GDB/LLDB

```bash
# Build with debug symbols
cargo build

# Run in debugger
rust-gdb target/debug/karana-core

# Commands:
# break main
# run
# step
# continue
# print variable_name
```

### 3. Common Debug Patterns

```rust
// Add to problematic function
println!("DEBUG: variable = {:?}", variable);

// Use dbg! macro (automatically prints file:line)
dbg!(some_value);

// Conditional logging
log::debug!("Processing intent: {}", intent);
log::info!("Successfully completed operation");
log::warn!("Potential issue detected");
log::error!("Error occurred: {}", err);
```

### 4. Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --example full_stack_demo

# Open flamegraph.svg in browser
```

---

## Common Issues & Solutions

### Issue 1: Compilation Errors

**Problem:** `error: linking with 'cc' failed`

**Solution:**
```bash
# Install build tools
sudo apt install build-essential pkg-config libssl-dev
```

---

### Issue 2: RocksDB Errors

**Problem:** `Error: RocksDB: Corruption: checksum mismatch`

**Solution:**
```bash
# Delete corrupted databases
rm -rf karana-cache/ karana-governance/ karana-ledger/

# Restart system (will recreate)
cargo run
```

---

### Issue 3: Port Already in Use

**Problem:** `Error: Address already in use (os error 98)`

**Solution:**
```bash
# Find process using port 8080
lsof -i :8080

# Kill process
kill -9 <PID>

# Or use different port
PORT=8081 cargo run --bin karana-api-server
```

---

### Issue 4: Camera Access Denied

**Problem:** `Camera error: Permission denied`

**Solution:**
```bash
# Add user to video group
sudo usermod -a -G video $USER

# Log out and log back in
```

---

### Issue 5: Out of Memory During Build

**Problem:** `error: could not compile due to out of memory`

**Solution:**
```bash
# Limit parallel jobs
cargo build -j 2

# Or increase swap
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

---

### Issue 6: libp2p Connection Failures

**Problem:** `Failed to connect to peer`

**Solution:**
```bash
# Check firewall
sudo ufw allow 9000/tcp

# Run with mDNS discovery
cargo run -- boot --port 9000

# Check logs
RUST_LOG=libp2p=debug cargo run
```

---

## Architecture Deep Dive

### System Boot Sequence

```
1. main.rs
   ├─► Parse CLI arguments
   └─► Create KaranaMonad

2. monad.rs::ignite()
   ├─► Initialize RocksDB state
   ├─► Boot Layer 1 (Hardware)
   ├─► Boot Layer 2 (Network)
   │   ├─► Start libp2p swarm
   │   ├─► Start mDNS discovery
   │   └─► Start GossipSub routing
   ├─► Boot Layer 3 (Blockchain)
   │   ├─► Load chain state
   │   ├─► Initialize wallet
   │   └─► Start consensus
   ├─► Boot Layer 4 (Oracle)
   │   ├─► Register tools
   │   ├─► Initialize ZK prover
   │   └─► Start intent processor
   ├─► Boot Layer 5 (Intelligence)
   │   ├─► Load vision models
   │   ├─► Initialize scene understanding
   │   └─► Start memory system
   ├─► Boot Layer 6 (AI Engine)
   │   ├─► Load language models
   │   ├─► Initialize NLU
   │   └─► Start dialogue manager
   ├─► Boot Layer 7 (Interface)
   │   ├─► Initialize voice UI
   │   ├─► Start gesture tracking
   │   └─► Initialize HUD renderer
   ├─► Boot Layer 8 (Applications)
   │   └─► Load installed apps
   └─► Boot Layer 9 (System Services)
       ├─► Start OTA updater
       ├─► Start security monitor
       └─► Start diagnostics

3. API Server (if started)
   ├─► Start HTTP server (port 8080)
   └─► Start WebSocket server
```

### Request Flow Example

**User command: "What's the weather?"**

```
1. Voice UI (Layer 7)
   └─► Speech recognition → "What's the weather?"

2. AI Engine (Layer 6)
   ├─► NLU: Intent = weather_query
   ├─► Entity extraction: location = current
   └─► Dialogue: Generate response plan

3. Oracle Bridge (Layer 4)
   ├─► Convert intent to oracle request
   ├─► Select tool: web_api
   ├─► Execute: fetch weather API
   ├─► Generate ZK proof of execution
   └─► Return response

4. Blockchain (Layer 3)
   ├─► Submit oracle response transaction
   └─► Settle on-chain (12s block time)

5. Interface (Layer 7)
   ├─► Render weather in HUD
   └─► Speak response via TTS
```

### State Management

**RocksDB Persistence:**
- `karana-cache/`: Temporary cache (evictable)
- `karana-governance/`: DAO proposals and votes
- `karana-ledger/`: Blockchain state (immutable)

**In-Memory State:**
- AI conversation context (short-term memory)
- Network peer connections
- Active AR anchors
- Running applications

---

## Contributing Guidelines

### Code Style

1. **Formatting**: Always run `cargo fmt` before committing
2. **Linting**: Fix all `cargo clippy` warnings
3. **Documentation**: Add rustdoc comments for public APIs
4. **Tests**: Include tests for new functionality

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(oracle): add new web scraping tool
fix(blockchain): resolve consensus deadlock issue
docs(layers): update Layer 4 documentation
test(ai): add tests for intent classification
refactor(network): optimize peer discovery
```

### Pull Request Process

1. Create feature branch from `master`
2. Make changes with clear commits
3. Run full test suite: `cargo test --lib`
4. Update relevant documentation in `docs/`
5. Push branch and create PR
6. Request review from maintainers
7. Address feedback
8. Merge after approval

### Review Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New code has tests (>80% coverage)
- [ ] Documentation updated
- [ ] No performance regressions
- [ ] Follows Rust best practices
- [ ] No unnecessary dependencies added

---

## Useful Commands Cheat Sheet

```bash
# Building
cargo build                    # Debug build
cargo build --release          # Release build
cargo check                    # Fast syntax check
cargo clean                    # Remove build artifacts

# Testing
cargo test --lib               # Run all tests
cargo test --lib module        # Test specific module
cargo test -- --nocapture      # Show output
cargo test -- --test-threads=1 # Sequential tests

# Formatting & Linting
cargo fmt                      # Format code
cargo clippy                   # Run linter
cargo clippy -- -D warnings    # Fail on warnings

# Documentation
cargo doc --open               # Generate and open docs
cargo doc --no-deps --open     # Docs without dependencies

# Running
cargo run                      # Run main binary
cargo run --bin name           # Run specific binary
cargo run --example name       # Run example
cargo run --release            # Run optimized build

# Dependencies
cargo tree                     # Show dependency tree
cargo update                   # Update dependencies
cargo outdated                 # Check for outdated deps

# Profiling
cargo build --release          # Build optimized
perf record ./target/release/karana-core
perf report                    # View profile

# Debugging
RUST_LOG=debug cargo run       # Debug logging
RUST_BACKTRACE=1 cargo run     # Stack traces
rust-gdb target/debug/binary   # Run in debugger
```

---

## Next Steps

1. **Read the Architecture**: Start with `docs/README.md` and layer docs
2. **Run Examples**: Try `cargo run --example full_stack_demo`
3. **Explore Code**: Navigate `karana-core/src/` modules
4. **Make Small Change**: Fix a typo, add a log message, write a test
5. **Join Community**: Connect with other developers
6. **Pick an Issue**: Look for "good first issue" labels
7. **Build Something**: Create a new app or tool

---

## Resources

- **Documentation**: `docs/README.md` (12 comprehensive guides)
- **Examples**: `karana-core/examples/` (runnable demos)
- **Main README**: `../README.md` (project overview)
- **Architecture Doc**: `../ARCHITECTURE.md` (technical deep dive)
- **Quick Start**: `../QUICK_START.md` (testing guide)

---

## Getting Help

1. **Check Documentation**: Read relevant layer docs in `docs/`
2. **Search Issues**: Look for similar problems on GitHub
3. **Ask in Community**: Join Discord/Slack/Matrix
4. **Create Issue**: Detailed description, error logs, steps to reproduce
5. **Read Source**: The code is well-commented, use `grep` liberally

---

**Welcome to Kāraṇa OS development!** 🚀

You're now part of building the future of sovereign, AI-native operating systems.
