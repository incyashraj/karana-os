# Voice AI Example Commands

**Date:** January 12, 2026  
**Status:** Ready to test  

---

## 🎤 Navigation Commands

### Basic Navigation
```
"Go to home"
"Navigate to settings"
"Go back"
"Take me to the main screen"
```

**Expected Behavior:**
- Tool: `navigate(destination="home/settings/back")`
- UI navigates to the specified screen
- Feedback: "Navigated to [destination]"

---

## 📱 App Launching

### Launch Applications
```
"Open camera"
"Launch the wallet"
"Start music player"
"Open browser"
"Show me the weather"
"Open maps"
```

**Expected Behavior:**
- Tool: `launch_app(app_name="camera/wallet/music/browser/weather/maps")`
- App opens in spatial container
- Feedback: "✓ Launched [app]"

---

## ✅ Task Management

### Create Tasks
```
"Add task to review PR"
"Create task: call John"
"Add reminder to buy groceries"
"Task to finish documentation"
```

**Expected Behavior:**
- Tool: `create_task(task="[description]")`
- Task appears in task list
- Feedback: "✓ Created task: '[description]'"

### Undo Tasks
```
"Undo that"
"Undo the last action"
"Cancel that"
```

**Expected Behavior:**
- Tool: `undo()`
- Last action reversed
- Feedback: "⟲ Undone: [action]"

---

## 🌤️ Weather Queries

### Check Weather
```
"What's the weather?"
"Weather in London"
"How's the weather today?"
"Will it rain tomorrow?"
```

**Expected Behavior:**
- Tool: `weather(location="current/London")`
- Weather info displayed
- Feedback: "Weather in [location]: 72°F, Sunny"

### Follow-up Questions
```
User: "What's the weather?"
AI: "Currently 68°F and sunny"
User: "Should I bring an umbrella?"
AI: "No rain expected. Clear skies all day."
```

---

## 💰 Wallet Operations

### Check Balance
```
"What's my balance?"
"Show wallet balance"
"How many tokens do I have?"
```

**Expected Behavior:**
- Tool: `wallet(action="balance")`
- Balance displayed
- Feedback: "Your balance: 100.0 KARA tokens"

### Send/Receive
```
"Send tokens"
"Receive payment"
"Show receive address"
```

**Expected Behavior:**
- Tool: `wallet(action="send/receive")`
- Appropriate wallet screen shown

---

## 🗣️ Conversational Commands

### Greetings
```
"Hello"
"Hi there"
"Hey Karana"
```

**Expected Response:**
- "Hello! How can I help you?"

### Thanks
```
"Thank you"
"Thanks"
"Appreciate it"
```

**Expected Response:**
- "You're welcome!"

### Help
```
"What can you do?"
"Help me"
"Show commands"
```

**Expected Response:**
- List of capabilities

---

## 🎯 Contextual References

### Using "that" / "it"
```
User: "Open camera"
[Camera opens]
User: "Close that"
[Camera closes - context remembers "camera"]
```

### Using Ordinals
```
[UI shows 3 buttons]
User: "Click the first button"
[First button clicked]

User: "Click the third one"
[Third button clicked]
```

### Using Position
```
[UI shows multiple elements]
User: "Click the top button"
User: "Select the one on the left"
User: "Open the right panel"
```

**Expected Behavior:**
- StateContext resolves reference to specific UI element
- Action performed on correct element

---

## 🔗 Chained Commands

### Multiple Actions
```
"Open wallet and check my balance"
"Launch camera and take a photo"
"Go to settings and show system info"
```

**Expected Behavior:**
- Multiple tools executed in sequence
- Each action confirmed separately

---

## 🧪 Testing Scenarios

### Scenario 1: Morning Routine
```
1. "Hello" → Greeting response
2. "What's the weather?" → Weather info
3. "Should I bring an umbrella?" → Contextual answer
4. "Open calendar" → Calendar app launches
5. "Add task to call mom" → Task created
```

### Scenario 2: App Management
```
1. "Open camera" → Camera launches
2. "Take a photo" → Photo captured
3. "Close that" → Camera closes (contextual)
4. "Show me the gallery" → Gallery opens
```

### Scenario 3: Corrections
```
1. "Add task to review code" → Task created
2. "Actually, undo that" → Task removed
3. "Add task to review PR instead" → New task created
```

### Scenario 4: Multi-step Query
```
1. "What's my wallet balance?" → Balance shown
2. "Is that enough to buy coffee?" → Oracle reasoning
3. "Send 5 tokens to John" → Transaction initiated
```

---

## 🎨 Testing UI Feedback

### Voice Activity Indicators
- 🎤 Microphone icon pulses when listening
- 📊 Waveform shows audio level
- 💬 Real-time transcription appears
- ⚡ Tool results show with confidence score
- ✓ Success feedback with green badge
- ❌ Error feedback with red badge

### Expected Animations
- Waveform animates during speech
- Smooth fade-in for transcription
- Tool result slides in from bottom
- Undo button appears after actions

---

## 🐛 Edge Cases to Test

### Empty/Invalid Input
```
"[silence]" → No action
"Ummm..." → Uncertain, no action
"Asdfghjkl" → Error: "I didn't understand that"
```

### Ambiguous Commands
```
"Open it" (with no context) → "What would you like to open?"
"Do that thing" → Clarification requested
```

### Long Commands
```
"Open the camera app and then take a photo and after that show me the gallery and finally close everything"
→ Should handle gracefully or break into steps
```

### Interruptions
```
[User speaks]
[User interrupts mid-sentence]
→ Should reset and start over
```

---

## 📊 Success Metrics

Track these during testing:

```typescript
{
  totalCommands: number,
  successfulExecutions: number,
  failedExecutions: number,
  avgConfidence: number,
  avgResponseTime: number,
  undoRate: number,
  clarificationsNeeded: number
}
```

### Target Metrics
- Success rate: >95%
- Average confidence: >0.85
- Response time: <500ms
- Undo rate: <5%

---

## 🚀 Running the Tests

### 1. Start Backend
```bash
cd karana-core
cargo run --bin voice_server
```

### 2. Start Frontend
```bash
cd simulator-ui
npm run dev
```

### 3. Open Browser
```
http://localhost:5173
```

### 4. Test Voice Commands
1. Click microphone button
2. Grant microphone permission
3. Say command
4. Verify:
   - Transcription appears
   - Tool executes
   - UI updates
   - Feedback shows

---

## 🔍 Debugging

### Check Backend Logs
```bash
# Should see:
[VOICE] Processing: 'open camera'
[TOOL] Executing: launch_app
[TOOL] ✓ Success: Launched camera
[WS] Broadcasting tool result
```

### Check Browser Console
```javascript
// Should see:
[Voice] open camera
[Tool] {tool_name: "launch_app", result: "Launched camera", ...}
[WS] ✓ Connected
```

### Check WebSocket Connection
```bash
# In browser DevTools Network tab:
# Look for ws://localhost:8080 connection
# Status should be "101 Switching Protocols"
```

---

## ✅ Test Checklist

- [ ] WebSocket connects successfully
- [ ] Microphone permission granted
- [ ] Waveform visualizes audio
- [ ] Voice transcription works
- [ ] Navigation commands execute
- [ ] App launching works
- [ ] Task creation works
- [ ] Weather queries work
- [ ] Wallet commands work
- [ ] Conversational responses work
- [ ] Contextual references resolve
- [ ] Undo functionality works
- [ ] Tool execution feedback shows
- [ ] TTS speaks responses (if enabled)
- [ ] UI updates reflect tool results

---

## 🎯 Next Steps

After basic testing passes:

1. **Performance Testing**
   - Test with background noise
   - Test with fast speech
   - Test with accents
   - Measure latency

2. **Stress Testing**
   - Rapid fire commands
   - Very long commands
   - Many concurrent users

3. **Integration Testing**
   - Test with real apps
   - Test with real data
   - Test error recovery

4. **User Testing**
   - Get feedback on naturalness
   - Identify missing commands
   - Improve error messages

---

## 📝 Report Issues

If something doesn't work:

1. Check backend logs
2. Check browser console
3. Verify WebSocket connection
4. Check tool registry has tool
5. Verify state context
6. Test with simpler command

Common fixes:
- Restart voice server
- Clear browser cache
- Check microphone permission
- Rebuild frontend
- Check port 8080 available
