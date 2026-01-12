# 🎯 Oracle Tool Execution - Quick Reference

## Start System
```bash
# Terminal 1 - Backend
cd karana-os/karana-core
cargo run --release

# Terminal 2 - Frontend  
cd karana-os/kāraṇa-os-simulator
npm run dev

# Terminal 3 - Test
cd karana-os
./test_oracle_tools.sh
```

## Voice Commands
```
"open camera"              → Launch camera app
"check balance"            → Show wallet balance
"send 50 KARA to alice"    → Transfer funds
"navigate to SF"           → Start navigation
"take note: buy milk"      → Create task
"set reminder: meeting"    → Create reminder
"play jazz music"          → Play music
"search the web"           → Open browser
"play cats video"          → Play video
```

## Verify It's Working
```bash
# Check logs for:
[API] ✓ Tool executed: Camera application launched
[API] ✓ Tool executed: Current balance: 1000 KARA
[API] ✓ Tool executed: Navigation started to San Francisco
```

## Architecture (Simplified)
```
Voice → Oracle → tool_bridge → ToolRegistry → Actual Action ✅
```

## Key Files
- `oracle/tool_bridge.rs` - Intent → Tool mapping
- `api/handlers.rs` - Execution integration (line ~353)
- `api/state.rs` - ToolRegistry initialization

## Status: ✅ OPERATIONAL
All voice commands execute actual OS tools.
No more "text only" responses!

---
*Quick ref for karana-os Oracle system*
