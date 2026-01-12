# Voice AI Integration Plan - Task Master Concepts

**Date:** January 12, 2026  
**Based on:** Keith Schacht's Task Master architecture  
**Target:** karana-os voice-first OS experience

---

## 🎯 Core Principles from Task Master

1. **Voice Input + Visual Output** - The winning combination
   - Speaking is 2-3× faster than typing
   - Visual feedback has much higher bandwidth than audio responses
   - Stream-of-consciousness natural language works great with LLMs

2. **Tool-Based Architecture** - Not just chat, but actions
   - Voice → Intent Classification → Tool Call → State Update → Visual Feedback
   - Tools have clear parameters and return values
   - Undo capability for all actions

3. **Continuous Listening** - Not push-to-talk
   - Always-on voice activity detection (VAD)
   - Ambient context awareness
   - Natural conversation flow

4. **Stateful Context** - The AI knows what's on screen
   - "The third item" → Resolves to specific UI element
   - "Move that up" → Understands spatial references
   - Full state passed to AI for ambiguous references

---

## 📊 Current karana-os Voice Capabilities

### ✅ Already Implemented:

#### **Voice Pipeline** (`voice_pipeline.rs`)
```rust
- Real-time microphone capture (CPAL)
- Voice Activity Detection (VAD)
- 16kHz PCM audio processing
- Whisper STT integration
- Wake word detection ("karana")
- Streaming transcription support
```

#### **Speech Recognition** (`ml/speech.rs`)
```rust
- Whisper-based transcription
- Language detection
- Streaming audio buffer (30s capacity)
- Partial transcription results
- Confidence scoring
```

#### **Voice Assistant** (`assistant/mod.rs`)
```rust
- State machine (Idle → Listening → Processing → Speaking)
- Intent classification
- Command routing
- Speech synthesis (TTS)
- Suggestion generation
```

### ⚠️ Missing Task Master Features:

1. **Tool-Based Actions** - Need structured tool registry
2. **Real-time UI Updates** - WebSocket for instant feedback
3. **Continuous Listening Mode** - Currently requires wake word
4. **Stateful Context** - AI doesn't see current screen state
5. **Visual Feedback During Speech** - No waveform/indicators

---

## ❓ Do You Need Task Master's Code?

**Short Answer: NO** - You can implement this entirely with open source tools.

### **Task Master's Stack (Proprietary):**
- **LiveKit Cloud** (Paid service, ~$0.01/min for WebRTC + AI routing)
- **Rails + ActionCable** (Backend framework)
- **ElevenLabs** (Paid voice synthesis)
- **Partial code extraction** (Not a complete standalone app)

### **karana-os Stack (Already Have & Open Source):**
- **Whisper** (OpenAI, MIT license) ✓ Already integrated
- **CPAL** (Audio I/O, MIT/Apache) ✓ Already integrated
- **Tokio** (Async runtime, MIT) ✓ Already integrated
- **React + TypeScript** (UI framework) ✓ Already integrated
- **Browser WebSocket API** (Built-in) ✓ Available

### **What You Actually Need (All Open Source):**

#### **For Audio Streaming:**
Choose one approach:

**Option A: Simple HTTP Polling** (Easiest, start here)
```typescript
// No WebRTC needed - just poll for updates
setInterval(() => {
  fetch('/api/voice/status').then(r => r.json()).then(update UI);
}, 100);
```

**Option B: Native WebSocket** (Better performance)
```rust
// Use tokio-tungstenite (MIT license)
[dependencies]
tokio-tungstenite = "0.21"  # WebSocket server
```

**Option C: WebRTC** (Lowest latency, most complex)
```rust
// Use webrtc-rs (MIT license) - Only if you need <50ms latency
[dependencies]
webrtc = "0.9"  # Full WebRTC stack in Rust
```

#### **Recommendation:** Start with **Option A** (HTTP polling), upgrade to **Option B** (WebSocket) when needed. Skip WebRTC unless doing real-time video calls.

---

## 🏗️ Integration Architecture (100% Open Source)

### **Recommended Stack:**

```text
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React/TypeScript)               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Voice UI    │  │ Tool Results │  │  App State   │      │
│  │  Waveform    │  │  Visual      │  │  Context     │      │
│  │  Indicator   │  │  Feedback    │  │  Tracker     │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
│                    ┌───────▼───────┐                         │
│                    │  WebSocket    │ ← Real-time updates     │
│                    │  Connection   │                         │
│                    └───────┬───────┘                         │
└────────────────────────────┼─────────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────────┐
│              Backend (Rust karana-core)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Voice       │→ │  Intent      │→ │  Tool        │      │
│  │  Pipeline    │  │  Router      │  │  Registry    │      │
│  │  (Whisper)   │  │  (ReAct)     │  │  (Actions)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                              │               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────▼───────┐      │
│  │  State       │  │  WebSocket   │  │  Tool        │      │
│  │  Manager     │← │  Broadcaster │← │  Executor    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Implementation Steps

### **Phase 1: Voice UI Components** (Frontend)

#### 1. Create `VoiceController.tsx`
```typescript
interface VoiceControllerProps {
  onTranscript: (text: string) => void;
  onToolResult: (result: ToolResult) => void;
  appState: AppState;
}

// Features:
- Continuous listening mode toggle
- Visual waveform during speech
- Real-time partial transcription display
- Tool execution feedback
- Undo last action button
```

#### 2. Create `ToolFeedbackOverlay.tsx`
```typescript
// Shows visual confirmation of voice actions:
- "✓ Created task: Review PR"
- "✓ Moved item to top"
- "⟲ Undoing last action..."
- Tool confidence indicator
```

#### 3. Enhance `ChatInterface.tsx`
```typescript
// Add:
- Voice input button (push-to-talk + continuous mode)
- Streaming transcription display
- Tool call visualization
- State context indicator (what AI can see)
```

---

### **Phase 2: Tool Registry** (Backend)

#### 1. Implement `tool_registry.rs`
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    execution_history: Vec<ToolExecution>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult>;
    async fn undo(&self, execution_id: &str) -> Result<()>;
}
```

#### 2. Implement Core Tools:
```rust
// UI Manipulation Tools
- NavigateTool: "Open settings", "Go back"
- SelectTool: "Click the third button", "Select that item"
- ScrollTool: "Scroll down", "Go to the top"

// Task Management Tools
- CreateTaskTool: "Add task to review code"
- UpdateTaskTool: "Mark that done", "Change priority"
- DeleteTaskTool: "Delete the first task"
- ReorderTaskTool: "Move it up", "Put that at the bottom"

// System Tools
- LaunchAppTool: "Open camera", "Start music player"
- WalletTool: "Check my balance", "Send 10 tokens"
- WeatherTool: "What's the weather?"

// Smart Home Tools (for AR glasses)
- ControlDeviceTool: "Turn on lights", "Set temp to 72"
```

---

### **Phase 3: State Context System**

#### 1. Create `state_context.rs`
```rust
pub struct StateContext {
    pub visible_elements: Vec<UIElement>,
    pub active_app: Option<String>,
    pub recent_actions: Vec<Action>,
    pub user_focus: Option<String>,
}

impl StateContext {
    /// Resolve ambiguous references
    pub fn resolve_reference(&self, text: &str) -> Option<UIElement> {
        // "that" → last mentioned element
        // "the third one" → 3rd visible element
        // "the button on the right" → spatial resolution
    }
}
```

#### 2. Integrate with Intent Router:
```rust
// Modify query_router.rs to accept context
pub async fn route_query(
    query: &str,
    context: &StateContext
) -> RouteDecision {
    // Example: "click that" becomes "click element_id_123"
    let resolved_query = context.resolve_ambiguous_terms(query);
    // ... existing routing logic
}
```

---

### **Phase 4: WebSocket Real-time Updates**

#### 1. Implement WebSocket Server:
```rust
// ws_server.rs
pub struct WsServer {
    clients: Arc<Mutex<Vec<WebSocket>>>,
    state_broadcaster: StateBroadcaster,
}

impl WsServer {
    pub async fn broadcast_tool_result(&self, result: ToolResult) {
        let msg = serde_json::to_string(&result).unwrap();
        for client in self.clients.lock().unwrap().iter() {
            client.send(msg.clone()).await.ok();
        }
    }
}
```

#### 2. Frontend WebSocket Client:
```typescript
// services/wsService.ts
export class WsService {
  private ws: WebSocket;
  
  connect() {
    this.ws = new WebSocket('ws://localhost:8080/ws');
    
    this.ws.onmessage = (event) => {
      const toolResult = JSON.parse(event.data);
      this.handleToolResult(toolResult);
    };
  }
  
  private handleToolResult(result: ToolResult) {
    // Update UI immediately
    // Show visual feedback
    // Update app state
  }
}
```

---

### **Phase 5: Continuous Listening Mode**

#### 1. Enhance Voice Pipeline:
```rust
// Add to voice_pipeline.rs
pub enum ListeningMode {
    WakeWord,      // "karana" activation
    Continuous,    // Always listening
    PushToTalk,    // Manual button press
}

impl VoicePipeline {
    pub fn set_listening_mode(&mut self, mode: ListeningMode) {
        self.config.mode = mode;
    }
    
    pub async fn process_continuous(&mut self) -> Option<String> {
        // Stream VAD → Auto segment → Transcribe → Reset
        // No wake word required
    }
}
```

#### 2. Privacy Controls:
```typescript
// Add to settings
interface VoiceSettings {
  mode: 'wakeWord' | 'continuous' | 'pushToTalk';
  privacyMode: boolean; // LED indicator when listening
  autoMute: {
    enabled: boolean;
    keywords: string[]; // "private", "confidential"
  };
}
```

---

## 🎨 UI/UX Enhancements

### **Voice Interaction Indicators:**

1. **Listening State:**
```typescript
<div className="voice-indicator">
  <WaveformVisualizer audioLevel={audioLevel} />
  <div className="status">
    {mode === 'listening' && '🎤 Listening...'}
    {mode === 'processing' && '⚡ Processing...'}
    {mode === 'executing' && '🔧 Executing...'}
  </div>
</div>
```

2. **Tool Execution Feedback:**
```typescript
<Toast type="success">
  ✓ Task created: "Review PR #123"
  <button onClick={undo}>↶ Undo</button>
</Toast>
```

3. **Confidence Indicators:**
```typescript
// Show when AI confidence is low
{confidence < 0.7 && (
  <div className="clarification">
    Did you mean:
    <ul>
      {suggestions.map(s => <li onClick={() => confirm(s)}>{s}</li>)}
    </ul>
  </div>
)}
```

---

## 🚀 Quick Wins (Start Here)

### **Week 1: Voice UI Polish**
1. Add waveform visualization to existing VoiceMode component
2. Implement push-to-talk and continuous listening toggle
3. Show real-time partial transcription
4. Add voice command suggestions (context-aware)

### **Week 2: Tool System Foundation**
1. Create `ToolRegistry` with 5 core tools
2. Integrate with existing `ReActAgent`
3. Add undo capability
4. Wire up WebSocket for instant feedback

### **Week 3: Context Awareness**
1. Implement `StateContext` system
2. Pass visible UI elements to AI
3. Add reference resolution ("that", "the third one")
4. Test with simulator UI

### **Week 4: Production Polish**
1. Add privacy controls and indicators
2. Optimize VAD for ambient noise
3. Implement confidence thresholds
4. Add error recovery and fallbacks

---

## 📈 Success Metrics

Track these to measure voice AI effectiveness:

```typescript
interface VoiceAnalytics {
  // Performance
  avgTranscriptionTime: number;
  avgToolExecutionTime: number;
  
  // Accuracy
  successfulCommands: number;
  failedCommands: number;
  confidenceDistribution: number[];
  
  // User Behavior
  voiceVsTextRatio: number;
  avgCommandsPerSession: number;
  mostUsedTools: string[];
  
  // Errors
  ambiguousReferences: number;
  lowConfidenceQueries: number;
  undoRate: number;
}
```

---

## 🔐 Security & Privacy

### **Voice Data Handling:**
1. **On-Device Processing** - Whisper runs locally (already implemented ✓)
2. **No Cloud Storage** - Transcripts never leave device
3. **Privacy Mode** - Auto-mute on sensitive keywords
4. **Visual Indicators** - Always show when mic is active

### **Tool Authorization:**
```rust
pub enum ToolPermission {
    Read,           // Query data
    Write,          // Modify state
    Execute,        // Run commands
    SystemControl,  // Critical actions
}

// Require explicit user confirmation for sensitive tools
if tool.permission() == ToolPermission::SystemControl {
    request_user_confirmation().await?;
}
```

---

## 🎯 Example Voice Interactions

### **Task Management:**
```
User: "Add a task to review the PR"
AI: ✓ Created task: "Review the PR" [shows in UI]

User: "Mark that as done"
AI: ✓ Completed task: "Review the PR"

User: "Actually, undo that"
AI: ⟲ Undone: Task marked incomplete

User: "Move it to the top"
AI: ✓ Reordered: "Review the PR" moved to position 1
```

### **System Control:**
```
User: "Open the camera"
AI: ✓ Launching Camera app [camera opens]

User: "Take a picture"
AI: ✓ Photo captured [shows preview]

User: "What's in this image?"
AI: 🔍 Analyzing... [vision AI processes]
AI: "I see a laptop, coffee mug, and notebook on a desk"
```

### **Contextual Queries:**
```
User: "What's the weather?"
AI: "Currently 68°F and sunny in San Francisco"

User: "Should I bring an umbrella?"
AI: "No rain expected today. Clear skies all day."

User: "What about tomorrow?"
AI: "60% chance of rain tomorrow afternoon. Yes, bring one."
```

---

## 🔗 Integration with Existing Systems

### **Leverages Current karana-os:**

1. **Voice Pipeline** → Already has Whisper + VAD ✓
2. **ReAct Agent** → Can route to tools ✓
3. **Query Router** → Intent classification ✓
4. **Oracle System** → Tool registry foundation ✓
5. **Simulator UI** → Perfect for testing ✓

### **Minimal New Dependencies:**

```toml
# Cargo.toml additions
[dependencies]
tokio-tungstenite = "0.21"  # WebSocket
serde_json = "1.0"          # Already has ✓
```

```json
// package.json additions
{
  "dependencies": {
    "wavesurfer.js": "^7.0.0",  // Audio waveform
    // ws already built into browsers ✓
  }
}
```

---

## 📚 References

### **Inspiration:**
- **Task Master Demo:** https://taskmaster.keithschacht.com
- **Blog Post:** https://keithschacht.com/2025/Dec/27/voice-first-todo-list-that-updates-live-as-you-talk/

### **Open Source Libraries You'll Use:**
- **Whisper** (STT): https://github.com/openai/whisper (MIT) - Already integrated ✓
- **tokio-tungstenite** (WebSocket): https://github.com/snapview/tokio-tungstenite (MIT)
- **wavesurfer.js** (Audio viz): https://github.com/wavesurfer-js/wavesurfer.js (BSD)
- **cpal** (Audio I/O): https://github.com/RustAudio/cpal (Apache-2.0) - Already integrated ✓

### **Optional Advanced Libraries:**
- **webrtc-rs** (Real-time): https://github.com/webrtc-rs/webrtc (MIT) - Only if needed
- **piper-tts** (Voice synthesis): https://github.com/rhasspy/piper (MIT) - Better than ElevenLabs
- **silero-vad** (Better VAD): https://github.com/snakers4/silero-vad (MIT)

### **karana-os Docs:**
- `/docs/PHASE_6_IMPLEMENTATION_REPORT.md`
- `/karana-core/src/voice_pipeline.rs`
- `/karana-core/src/ml/speech.rs`

---

## ✅ Next Steps

1. **Review this plan** with the team
2. **Prioritize features** based on use cases
3. **Start with Quick Wins** (Week 1 tasks)
4. **Iterate based on user feedback**

The foundation is already strong in karana-os. This plan leverages existing systems while adding the "magic" of Task Master's voice-first UX.
