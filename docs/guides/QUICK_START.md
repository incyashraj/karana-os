# 🚀 Quick Start: Testing AI-Driven Kāraṇa OS

## Status: READY TO TEST

Both servers are running and the complete AI system integration is operational!

---

## Servers Running

✅ **Backend**: http://localhost:8080 (Rust API)
✅ **Frontend**: http://localhost:3000 (React Simulator)

---

## How to Test

### 1. Open the Simulator
Navigate to: **http://localhost:3000**

### 2. Start Chat
Click the **💬 Chat** button (top-right) or press `C`

### 3. Try These Commands

#### Basic Hardware Control
```
"take a photo"
"brightness 70%"
"battery status"
"volume up"
"enable power saver"
```

#### Wallet & Blockchain
```
"create wallet"
"check balance"
"show transactions"
"send 50 KARA to Alice"
```

#### Vision & Intelligence
```
"what am I looking at"
"analyze scene"
```

#### App Management
```
"open youtube"
"install instagram"
"list apps"
"close youtube"
```

#### Interface Control
```
"hide HUD"
"show HUD"
"enable gesture tracking"
"enter AR mode"
"exit AR mode"
```

#### System Services
```
"run diagnostics"
"system health"
"check for updates"
"enable paranoid security"
"security status"
```

#### AR/Spatial
```
"create anchor here"
"list anchors"
"open browser tab"
"list tabs"
```

#### Multi-Layer Operations
```
"battery low, optimize everything"
"set timer 5 minutes and navigate home"
"take a photo and send to Mom"
```

---

## What to Look For

### ✅ Success Indicators

1. **Chat Response**: Oracle responds with helpful message
2. **Toast Notifications**: Bottom-right corner shows action confirmations
3. **System Updates**: UI changes reflect operations (mode switches, overlays, etc.)
4. **State Sync**: systemState tracks changes in real-time

### 📊 Example Success Flow

**Input**: `"battery status"`

**Expected**:
- Chat shows: "🔋 Battery: 85%, ⚡ Profile: balanced, 🕐 Runtime: ~180min, 🌡️ Thermal: normal"
- No errors in console
- State updated in systemState

---

## Debug Console

Press `F12` to open browser console and check:
- No errors (green = good)
- System state logs (Oracle processing, action execution)
- Activity log: `systemState.getActivityLog()`

---

## Advanced Testing

### Check System State
```javascript
// In browser console
systemState.getState()  // See complete state
systemState.getLayer('layer1_hardware')  // Check hardware
systemState.getLayer('layer3_blockchain')  // Check wallet
systemState.getContextForAI()  // See what AI knows
```

### Check Activity Log
```javascript
systemState.getActivityLog()  // Last 50 actions
systemState.getActivityLog('HARDWARE')  // Hardware actions only
```

### Test Multi-Step Operations
```
"send 50 KARA to Bob"
→ If no wallet: "Creating wallet first..."
→ Creates wallet
→ Then: "Preparing to send 50 KARA to Bob"
→ Asks confirmation
```

---

## Keyboard Shortcuts

- `C` - Toggle Chat
- `V` - Vision Analysis (take photo)
- `W` - Toggle Wallet
- `T` - Toggle Timers
- `N` - Toggle Notifications
- `S` - Settings
- `?` - Help

---

## Expected Behavior

### ✅ Operations That Work Fully
- All chat interactions
- State management
- UI updates (mode changes, overlays)
- Toast notifications
- Wallet operations (create, balance, transactions)
- Vision analysis (via backend AI)
- Timer management
- Android app tracking

### ⏳ Operations Using Simulated Responses
(Until backend APIs added)
- Hardware adjustments (brightness, volume)
- Network peer status
- System diagnostics
- OTA updates
- AR anchor persistence

**Note**: Even simulated operations work perfectly in the frontend - users get immediate feedback and state updates.

---

## Troubleshooting

### If Chat Doesn't Respond
1. Check backend is running: `curl http://localhost:8080/api/health`
2. Check browser console for errors
3. Try: "test" or "hello" to verify Oracle is processing

### If Actions Don't Execute
1. Check browser console for errors in `executeEnhancedAction`
2. Verify the command matches a known intent
3. Try a simpler command first (e.g., "battery status")

### If UI Doesn't Update
1. Check React components are mounted
2. Verify systemState is updating: `systemState.getState()`
3. Check state dependencies in useCallback hooks

---

## Demo Script

### 5-Minute Demo Flow

**1. Introduction** (30s)
- Open simulator
- Show HUD, camera feed
- Point out natural UI

**2. Basic Control** (1min)
```
"battery status" → Shows 85%, 180min
"take a photo" → Vision analysis runs
"brightness 50%" → Display dims
```

**3. Wallet Operations** (1.5min)
```
"create wallet" → Generates Ed25519 keys
"check balance" → Shows 1000 KARA
"send 50 KARA to Alice" → Transaction created (asks confirmation)
```

**4. App Management** (1min)
```
"list apps" → Shows installed apps
"open instagram" → Offers to install
"install instagram" → Installs app
"open instagram" → Launches
```

**5. System Control** (1min)
```
"run diagnostics" → 2s health check
"system health" → 95% score
"enable paranoid security" → Maximum privacy
"security status" → Shows settings
```

**Finale**: "battery low, optimize everything" → Multi-layer optimization

---

## Success Criteria

After testing, you should see:

✅ Oracle understands natural language across all layers
✅ Actions execute with appropriate feedback
✅ State synchronizes in real-time
✅ Multi-step operations work (dependencies detected)
✅ Confirmations requested for sensitive operations
✅ UI updates reflect system changes
✅ No console errors

---

## What Makes This Special

Unlike traditional OSes:
- **No menus to navigate** - just speak naturally
- **AI understands context** - knows system state
- **Multi-layer coordination** - operations span hardware/blockchain/apps
- **Intelligent suggestions** - Oracle anticipates needs
- **Privacy-first** - confirmations for sensitive operations

**This is the future of human-computer interaction.** 🚀

---

## Next: Try Your Own Commands!

The Oracle is smart and context-aware. Try:
- Complex requests: "take a photo and send it to Mom with 10 KARA"
- System queries: "what apps are running"
- Settings: "enable night mode"
- Status checks: "how many peers"

**Have fun exploring!** The entire OS is at your command. 🎉
