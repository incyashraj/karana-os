# AI Oracle System Analysis & Fix Plan

**Date:** January 12, 2026  
**Status:** 🔧 System Analysis Complete - Ready to Fix  

---

## 📊 Current State Analysis

### What Exists

#### 1. **Oracle Module** (`karana-core/src/oracle/`)
**Location:** `/karana-core/src/oracle/mod.rs`

**Strengths:**
- ✅ Comprehensive intent parsing (50+ patterns)
- ✅ Pattern matching for transfers, apps, navigation, reminders
- ✅ Conversational responses
- ✅ Legacy KaranaOracle wrapper for backwards compatibility

**Implementation:**
```rust
pub struct Oracle {
    conversation_history: Vec<ConversationTurn>,
    current_context: OracleContext,
    user_preferences: HashMap<String, String>,
}

// Process method - synchronous, pattern-based
pub fn process(&mut self, input: &str, context: Option<OracleContext>) -> OracleResponse
```

**Weaknesses:**
- ❌ **Not connected to Tool Registry** - Only returns text responses
- ❌ **No actual tool execution** - Just message generation
- ❌ **Synchronous** - Doesn't await async tool execution
- ❌ **No WebSocket integration** - Can't broadcast real-time updates

#### 2. **Universal Oracle** (`karana-core/src/oracle/universal.rs`)
**Location:** `/karana-core/src/oracle/universal.rs`

**Strengths:**
- ✅ RAG (Retrieval Augmented Generation) with embeddings
- ✅ Multi-source knowledge: local, swarm, web search, Wikipedia
- ✅ Caching layer for performance
- ✅ Async architecture

**Implementation:**
```rust
pub struct UniversalOracle {
    local_knowledge: Arc<LocalKnowledgeBase>,
    swarm_knowledge: Arc<SwarmKnowledge>,
    web_search: Option<Arc<WebSearchEngine>>,
    offline_kb: Option<Arc<StdMutex<OfflineKnowledgeBase>>>,
    search_cache: Arc<SearchCache>,
    embedding_cache: Arc<EmbeddingCache>,
}
```

**Weaknesses:**
- ❌ **Knowledge-focused only** - No OS control or tool execution
- ❌ **Not integrated with voice pipeline** - Separate system
- ❌ **No Tool Registry connection** - Can't launch apps, control OS

#### 3. **New Voice AI System** (Recent Addition)
**Location:** Multiple files

**Components:**
- ✅ `query_router.rs` - Intent classification
- ✅ `tool_registry.rs` - Tool execution with 5 tools
- ✅ `state_context.rs` - UI state tracking
- ✅ `voice_handler.rs` - Voice → Tool pipeline
- ✅ `react_agent.rs` - Multi-step reasoning
- ✅ `ws_server.rs` - Real-time WebSocket updates

**Strengths:**
- ✅ Complete async architecture
- ✅ Real tool execution (navigate, launch_app, create_task, weather, wallet)
- ✅ WebSocket broadcasting
- ✅ Context awareness

**Weaknesses:**
- ❌ **Not connected to Oracle** - Parallel system
- ❌ **No NLP parsing** - Relies only on QueryRouter patterns
- ❌ **Missing Oracle's extensive intent patterns** - Less comprehensive

#### 4. **API Layer** (`karana-core/src/api/`)
**Location:** `/karana-core/src/api/handlers.rs`

**Endpoint:** `POST /api/ai/oracle`

**Current Implementation:**
```rust
pub async fn process_oracle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OracleIntentRequest>,
) -> impl IntoResponse {
    // Try OracleVeil first (ZK-signed)
    if let Some(ref veil) = state.oracle_veil {
        match veil.mediate(&req.text, InputSource::Api).await {
            Ok(response) => { /* ... */ }
        }
    }
    
    // Fallback to legacy Oracle (pattern matching only)
    let mut oracle = crate::oracle::Oracle::new();
    let response = oracle.process(&req.text, None);
    // ...
}
```

**Weaknesses:**
- ❌ **Uses legacy Oracle** - No tool execution
- ❌ **No Voice Handler integration** - Misses new capabilities
- ❌ **No ReAct agent** - Can't do multi-step reasoning

#### 5. **Frontend** (`simulator-ui/`)
**Location:** Multiple TypeScript files

**OracleService:**
```typescript
class UniversalOracleService {
  async mediate(request: string): Promise<OracleManifest> {
    // Try real backend first
    if (this.useRealBackend) {
      const backendResponse = await karanaApi.processOracleIntent(request);
      // ...
    }
    
    // Fallback: Simulated response
    // ...
  }
}
```

**Weaknesses:**
- ❌ **Has fallback mode** - Uses simulated responses when backend fails
- ❌ **Duplicated logic** - Frontend simulates what backend should do

---

## 🔍 Core Problems

### Problem 1: **Three Disconnected Systems**

```
┌─────────────────────────────────────────────────────────┐
│                   CURRENT STATE                          │
│                                                          │
│   ┌──────────────┐   ┌───────────────┐   ┌──────────┐  │
│   │   Oracle     │   │ Universal     │   │  Voice   │  │
│   │   (Legacy)   │   │    Oracle     │   │  System  │  │
│   │              │   │               │   │          │  │
│   │ • Patterns   │   │ • RAG/KB      │   │ • Tools  │  │
│   │ • No tools   │   │ • Knowledge   │   │ • WS     │  │
│   │ • Sync       │   │ • No tools    │   │ • Async  │  │
│   └──────┬───────┘   └───────┬───────┘   └────┬─────┘  │
│          │                   │                 │        │
│          └───────────────────┴─────────────────┘        │
│                    NOT CONNECTED!                        │
└─────────────────────────────────────────────────────────┘
```

**Impact:** User says "open camera" → Oracle just returns text, doesn't actually open camera

### Problem 2: **API Uses Wrong Oracle**

```
Frontend Request
      ↓
API: process_oracle()
      ↓
Legacy Oracle.process() → Only pattern matching, no tool execution
      ↓
Text response (camera doesn't actually open)
```

**Impact:** Voice commands work in isolation but don't execute actual OS actions

### Problem 3: **No Unified Entry Point**

**Current:** User input can come through:
- Voice → VoiceHandler → QueryRouter → Tools ✅
- API → Oracle → Text response only ❌
- WebSocket → ? (not connected)

**Should be:** All inputs → Unified AIOracle → Tool execution

### Problem 4: **Missing Reasoning Layer**

```
User: "Should I bring an umbrella?"

Current Flow:
  Oracle → Pattern match → "I don't understand"

Needed Flow:
  AIOracle → ReAct agent → Weather tool → Reasoning → "Yes, 80% rain"
```

---

## 🎯 Solution Architecture

### Unified AI Oracle System

```
┌──────────────────────────────────────────────────────────────┐
│                     UNIFIED ORACLE                            │
│                                                               │
│   ┌────────────────────────────────────────────────────┐    │
│   │              AIOracle (New Unified)                 │    │
│   │                                                     │    │
│   │  Components:                                       │    │
│   │  • QueryRouter (intent classification)            │    │
│   │  • Tool Registry (actual execution)               │    │
│   │  • StateContext (memory & references)             │    │
│   │  • ReAct Agent (multi-step reasoning)             │    │
│   │  • DialogueManager (conversation)                 │    │
│   │  • TTS Service (voice output)                     │    │
│   │  • WebSocket Server (real-time updates)           │    │
│   │                                                     │    │
│   │  + Legacy Oracle patterns (intent detection)      │    │
│   │  + Universal Oracle (RAG/knowledge)               │    │
│   └──────────────────────┬──────────────────────────────┘    │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
    ┌────────┐      ┌──────────┐      ┌──────────┐
    │ Voice  │      │   API    │      │  WebUI   │
    │ Input  │      │ Endpoint │      │  Chat    │
    └────────┘      └──────────┘      └──────────┘
```

### Data Flow

```
1. USER INPUT
   "Open camera and take a photo"
        ↓
2. INTENT CLASSIFICATION (QueryRouter + Oracle patterns)
   Intent: OSControl
   Confidence: 0.94
   Entities: { app: "camera", action: "capture" }
        ↓
3. CONTEXT ENRICHMENT (StateContext)
   - Check if camera already open
   - User preferences
   - Recent conversation
        ↓
4. EXECUTION DECISION
   Simple query? → Direct tool execution
   Complex query? → ReAct multi-step reasoning
        ↓
5. TOOL EXECUTION (ToolRegistry)
   Tool 1: launch_app("camera") → "Camera opened"
   Tool 2: capture_photo() → "Photo saved"
        ↓
6. RESPONSE GENERATION
   Natural language: "✓ Camera opened. Photo saved to gallery."
   Confidence: 0.98
        ↓
7. OUTPUT
   - WebSocket broadcast → Real-time UI update
   - TTS speak → Voice feedback
   - Return response → API caller
```

---

## 🔧 Implementation Plan

### Phase 1: Create Unified AIOracle ✅ (DONE)

**File:** `karana-core/src/assistant/ai_oracle.rs`

**Created struct:**
```rust
pub struct AIOracle {
    query_router: Arc<QueryRouter>,
    tool_registry: Arc<ToolRegistry>,
    state_context: Arc<RwLock<StateContext>>,
    react_agent: Option<Arc<ReActAgent>>,
    dialogue_manager: Arc<Mutex<DialogueManager>>,
    reasoner: Arc<ChainOfThoughtReasoner>,
    tts_service: Option<Arc<TtsService>>,
    ws_server: Option<Arc<WsServer>>,
    history: Arc<Mutex<VecDeque<ConversationMessage>>>,
    mode: OracleMode,
}
```

**Key methods:**
- `async fn process(&self, input: &str) -> Result<OracleResponse>`
- Intent classification + tool execution + reasoning

### Phase 2: Integrate Oracle Patterns into AIOracle 🔧 (TODO)

**Goal:** Merge Oracle's excellent pattern matching with AIOracle

**Actions:**
1. Extract Oracle's intent parsing logic
2. Add as fallback in QueryRouter
3. Keep all 50+ patterns (transfers, apps, reminders, etc.)

**Benefit:** Best of both worlds - patterns + tools

### Phase 3: Update API Handler 🔧 (TODO)

**File:** `karana-core/src/api/handlers.rs`

**Change:**
```rust
pub async fn process_oracle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OracleIntentRequest>,
) -> impl IntoResponse {
    // OLD: Use legacy Oracle
    // let mut oracle = crate::oracle::Oracle::new();
    // let response = oracle.process(&req.text, None);
    
    // NEW: Use unified AIOracle
    let oracle = state.ai_oracle.read().await;
    let response = oracle.process(&req.text).await?;
    
    // Broadcast via WebSocket
    state.ws_server.broadcast_tool_result(...).await?;
    
    // Convert to API response format
    let api_response = OracleIntentResponse {
        intent_type: response.intent_type,
        content: response.text,
        confidence: response.confidence,
        // ...
    };
}
```

### Phase 4: Update Voice Server 🔧 (TODO)

**File:** `karana-core/src/bin/voice_server.rs`

**Change:**
```rust
// OLD: VoiceHandler with separate components
let voice_handler = VoiceCommandHandler::new(...);

// NEW: Use AIOracle directly
let ai_oracle = AIOracle::new(
    tool_registry,
    state_context,
    Some(tts_service),
    Some(ws_server),
);

// Voice pipeline connects to oracle
voice_pipeline.on_transcript(|text| {
    oracle.process(text).await
});
```

### Phase 5: Frontend Cleanup 🔧 (TODO)

**File:** `simulator-ui/services/oracleService.ts`

**Change:**
```typescript
// REMOVE fallback simulation
async mediate(request: string): Promise<OracleManifest> {
    // Always use real backend
    const backendResponse = await karanaApi.processOracleIntent(request);
    return this.convertToManifest(backendResponse);
    
    // DELETE: Simulated fallback code
}
```

### Phase 6: Knowledge Integration 🔧 (TODO)

**Goal:** Connect Universal Oracle's RAG to AIOracle

**Implementation:**
```rust
impl AIOracle {
    async fn execute_with_knowledge(&self, query: &str) -> Result<OracleResponse> {
        // 1. Check if query needs knowledge lookup
        if self.requires_knowledge(query) {
            // 2. Query Universal Oracle's RAG
            let knowledge = self.universal_oracle.query(query, &context).await?;
            
            // 3. Synthesize response
            return Ok(OracleResponse {
                text: knowledge.answer,
                source: "knowledge_base",
                confidence: knowledge.confidence,
                // ...
            });
        }
        
        // Fall through to tool execution
        self.execute_direct(query).await
    }
}
```

---

## 📋 Immediate Next Steps

### Step 1: Export AIOracle ✅ (DO NOW)

```rust
// karana-core/src/assistant/mod.rs
pub mod ai_oracle;
pub use ai_oracle::*;
```

### Step 2: Add Oracle Patterns to QueryRouter 🔧 (DO NOW)

Merge Oracle's `parse_intent()` logic into QueryRouter as enhanced pattern matching layer

### Step 3: Update API Handler 🔧 (DO NOW)

Replace legacy Oracle with AIOracle in `handlers::process_oracle`

### Step 4: Update App State 🔧 (DO NOW)

Add AIOracle to AppState:
```rust
pub struct AppState {
    pub ai_oracle: Arc<RwLock<AIOracle>>,
    // ... existing fields
}
```

### Step 5: Test End-to-End 🧪 (DO NOW)

```bash
# Terminal 1: Start backend
cd karana-core
cargo run --bin voice_server

# Terminal 2: Start frontend
cd simulator-ui
npm run dev

# Browser: Test
http://localhost:5173
Say: "Open camera"
Expected: Camera actually opens
```

---

## 🎯 Success Criteria

### Must Have
- [ ] User says "open camera" → Camera app launches
- [ ] User says "send 50 to Alice" → Transaction dialog appears
- [ ] User says "should I bring umbrella?" → Weather checked → Answer given
- [ ] WebSocket broadcasts tool execution in real-time
- [ ] Voice transcription → Tool execution → UI update (full flow)

### Nice to Have
- [ ] RAG knowledge queries work ("What is quantum computing?")
- [ ] Multi-step reasoning works ("Find coffee shop near me and navigate")
- [ ] Conversation context maintained ("Open it" after "show camera")

---

## 📊 Current Status

```
System Component Status:
├─ QueryRouter        ✅ Implemented
├─ ToolRegistry       ✅ Implemented (5 tools)
├─ StateContext       ✅ Implemented
├─ VoiceHandler       ✅ Implemented
├─ ReActAgent         ✅ Implemented
├─ WebSocketServer    ✅ Implemented
├─ AIOracle (Unified) ✅ Created (needs integration)
├─ API Integration    ❌ Using legacy Oracle
├─ Voice Server       ❌ Not using AIOracle
└─ Frontend           ❌ Has fallback mode

Integration Status: 30% Complete
Expected Completion: 1-2 hours of focused work
Blocking Issue: API handler not using AIOracle
```

---

## 🚀 Quick Win Strategy

**Fastest path to working system:**

1. **Export AIOracle** (2 minutes)
2. **Update API handler** (15 minutes) ← BIGGEST IMPACT
3. **Remove frontend fallback** (5 minutes)
4. **Test basic commands** (10 minutes)
5. **Verify tool execution** (10 minutes)

**Total time to functional system: ~45 minutes**

Then iterate:
- Add more tools
- Improve intent accuracy
- Add RAG knowledge
- Optimize performance

---

**Next Action:** Export AIOracle and update API handler to use it

**File to Edit:** `/karana-core/src/api/handlers.rs` line ~240 (process_oracle function)

**Ready to proceed?** ✅
