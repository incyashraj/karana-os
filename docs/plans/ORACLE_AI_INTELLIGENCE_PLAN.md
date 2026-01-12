# Oracle AI Intelligence Plan
## Making Kāraṇa OS Oracle Production-Grade Intelligent

**Date**: December 7, 2025  
**Status**: Planning Phase  
**Priority**: CRITICAL - Primary user interface for entire OS

---

## 🎯 VISION

The Oracle AI is the **PRIMARY TUNNEL** through which users interact with their smartglasses and the entire Kāraṇa OS. It must be:
- **Insanely intelligent** - Understanding context, nuance, multi-step commands, ambiguity
- **Proactive** - Anticipating needs, offering suggestions, learning patterns
- **Natural** - Conversational, human-like, emotionally aware
- **Reliable** - Works offline, handles errors gracefully, never confuses users
- **Fast** - Sub-500ms response time for most operations
- **Complete** - Controls ALL 9 layers + cross-cutting systems seamlessly

---

## 📊 CURRENT STATE ANALYSIS

### ✅ What Works
1. **Basic Pattern Matching** - Simple keyword detection (e.g., "take photo", "check balance")
2. **System State Awareness** - Knows about all 9 layers through SystemStateManager
3. **Action Execution** - Can trigger operations across all layers
4. **Gemini Integration** - Falls back to Gemini for knowledge queries
5. **Conversation History** - Maintains last 20 messages

### ❌ Critical Gaps (Why it's not usable)

#### 1. **Intent Classification is Primitive**
- **Current**: Regex pattern matching with `matchesAny()` - brittle and limited
- **Problem**: Can't handle:
  - Ambiguous queries ("it's too bright" → display brightness? camera exposure?)
  - Multi-intent commands ("take a photo and send it to mom")
  - Contextual references ("do that again", "what was I looking at?")
  - Spelling variations ("batry status", "brightnes 50%")
  - Synonyms ("snap a pic", "capture image", "photograph this")
  - Natural language ("can you help me see what this is?")

#### 2. **No Contextual Understanding**
- **Current**: Each query processed in isolation
- **Problem**: Can't handle:
  - Follow-up questions ("what about WhatsApp?" after "is Instagram installed?")
  - Pronouns ("install it", "open that app", "send him 5 KARA")
  - Time references ("my last transaction", "yesterday's photos")
  - Spatial references ("the app I just opened", "this object")

#### 3. **No Learning or Personalization**
- **Current**: Static responses, no user preference memory
- **Problem**: Can't:
  - Learn user patterns ("I always send KARA to Alice" → suggest Alice automatically)
  - Remember preferences ("I prefer paranoid security mode")
  - Adapt to usage habits (user checks battery every morning → proactive alert)
  - Improve from mistakes (user corrects misunderstanding → doesn't repeat)

#### 4. **Limited Gemini Integration**
- **Current**: Only used for knowledge queries, not intent classification
- **Problem**: Missing AI's strongest capability - understanding natural language
- **Why**: Current architecture: User → Pattern Matching → Action, Gemini only if low confidence
- **Should Be**: User → Gemini (Intent + Context) → Action Plan → Execute

#### 5. **No Multi-Step Planning**
- **Current**: Single action execution
- **Problem**: Can't handle:
  - "Take a photo, analyze it, and send to mom" (3 steps)
  - "Install Instagram, then open it and post my last photo" (4 steps)
  - "Check battery, if low enable power save mode" (conditional logic)

#### 6. **Poor Error Handling**
- **Current**: Generic fallback messages
- **Problem**: User has no idea what went wrong or how to fix it

#### 7. **No Confirmation Flow**
- **Current**: Auto-confirms everything in executeEnhancedAction
- **Problem**: Can't ask user for clarification or permission

#### 8. **Offline Capability Weak**
- **Current**: Falls back to pattern matching if Gemini unavailable
- **Problem**: Dramatically reduced capability without internet

---

## 🏗️ ARCHITECTURE REDESIGN

### New Architecture Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INPUT                              │
│          (Voice/Text: "Take a photo and send to mom")           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   1. INPUT PREPROCESSING                         │
│  - Spelling correction (batry → battery)                        │
│  - Normalization (lowercase, trim, expand contractions)         │
│  - Tokenization (split into meaningful units)                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   2. CONTEXT ENRICHMENT                          │
│  - Add conversation history (last 5 exchanges)                  │
│  - Add system state (battery, wallet, apps, camera, etc.)      │
│  - Add user profile (preferences, habits, history)             │
│  - Add temporal context (time of day, recent actions)          │
│  - Add spatial context (location, looking at, nearby objects)  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│          3. INTENT CLASSIFICATION (Multi-Model Approach)         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ LOCAL MODEL  │  │ GEMINI 2.0   │  │ PATTERN      │         │
│  │ (TinyLlama)  │  │ FLASH        │  │ MATCHING     │         │
│  │ Offline,Fast │  │ Accurate     │  │ Fallback     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│         ↓                 ↓                  ↓                   │
│  ┌─────────────────────────────────────────────────────┐       │
│  │         ENSEMBLE VOTING (Confidence Scoring)         │       │
│  │  - If all agree (0.95+ confidence) → proceed         │       │
│  │  - If disagree → use highest confidence              │       │
│  │  - If all low (<0.6) → ask clarification             │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                  │
│  OUTPUT: Intent[] = [                                           │
│    { layer: 'HARDWARE', operation: 'CAMERA_CAPTURE' },         │
│    { layer: 'INTELLIGENCE', operation: 'VISION_ANALYZE' },     │
│    { layer: 'BLOCKCHAIN', operation: 'SEND_TOKEN',             │
│      params: { recipient: 'mom', amount: <auto> } }            │
│  ]                                                              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   4. DEPENDENCY RESOLUTION                       │
│  - Check prerequisites (need wallet before transfer?)           │
│  - Order actions (install before open, capture before analyze)  │
│  - Handle conflicts (can't take photo while recording)          │
│  - Estimate resources (battery, network, time)                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   5. CONFIRMATION MANAGER                        │
│  - High-stakes operations → Ask user (transfer money, delete)   │
│  - Ambiguous params → Ask clarification (which mom contact?)    │
│  - Multi-step → Show plan, get approval                         │
│  - Low-stakes → Auto-execute with toast notification            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   6. EXECUTION ORCHESTRATOR                      │
│  - Execute actions in dependency order                          │
│  - Stream progress updates to user                              │
│  - Handle errors gracefully (rollback if needed)                │
│  - Collect execution results for next step                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   7. RESPONSE GENERATION                         │
│  - Use Gemini to create natural, context-aware response         │
│  - Include what was done, what's next, any issues               │
│  - Add proactive suggestions based on context                   │
│  - Update conversation history                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   8. LEARNING & FEEDBACK LOOP                    │
│  - Log: input → intent → actions → result → user reaction      │
│  - Update user profile with preferences                         │
│  - Train local model on corrections                             │
│  - Improve confidence thresholds                                │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📋 STEP-BY-STEP IMPLEMENTATION PLAN

### **PHASE 1: Foundation (Days 1-3)**
**Goal**: Fix critical architecture gaps

#### Task 1.1: Enhanced Intent Classification System
- **File**: `simulator-ui/services/intentClassifier.ts` (NEW)
- **Features**:
  ```typescript
  interface IntentClassificationResult {
    intents: IntentAction[];           // Multiple intents from one query
    confidence: number;                // Overall confidence (0-1)
    ambiguities: string[];             // What needs clarification
    contextUsed: string[];             // Which context was used
    alternativeInterpretations: string[]; // Other possible meanings
  }
  
  class IntentClassifier {
    // Multi-model ensemble
    async classifyWithEnsemble(input, context): Result
    
    // Gemini-powered classification
    async classifyWithGemini(input, context): Result
    
    // Local model classification (offline)
    async classifyWithLocalModel(input, context): Result
    
    // Pattern matching fallback
    classifyWithPatterns(input, context): Result
    
    // Spell correction
    correctSpelling(input): string
    
    // Entity extraction (names, numbers, times, etc.)
    extractEntities(input): Entity[]
  }
  ```

#### Task 1.2: Context Manager
- **File**: `simulator-ui/services/contextManager.ts` (NEW)
- **Features**:
  ```typescript
  interface EnrichedContext {
    conversationHistory: Message[];      // Last 10 exchanges
    systemState: CompleteSystemState;    // Current OS state
    userProfile: UserProfile;            // Preferences, habits
    temporalContext: {
      timeOfDay: string;
      recentActions: Action[];           // Last 5 actions
      activeTimers: Timer[];
    };
    spatialContext: {
      location?: GeoLocation;
      lookingAt?: VisionObject;
      nearbyDevices: Device[];
    };
  }
  
  class ContextManager {
    enrichContext(rawInput: string): EnrichedContext
    resolvePronouns(input: string, context: EnrichedContext): string
    resolveTemporal(input: string, context: EnrichedContext): string
  }
  ```

#### Task 1.3: User Profile System
- **File**: `simulator-ui/services/userProfile.ts` (NEW)
- **Features**:
  ```typescript
  interface UserProfile {
    preferences: {
      defaultSecurityMode: 'paranoid' | 'standard' | 'relaxed';
      favoriteApps: string[];
      frequentContacts: Map<string, string>; // "mom" → wallet address
      preferredBrightness: number;
    };
    learningData: {
      commandPatterns: Map<string, number>; // Frequency of commands
      errorCorrections: Map<string, string>; // User corrections
      dismissedSuggestions: string[];
    };
    statistics: {
      totalCommands: number;
      successRate: number;
      averageConfidence: number;
    };
  }
  
  class UserProfileManager {
    getProfile(): UserProfile
    updatePreference(key: string, value: any): void
    recordCommandPattern(command: string): void
    recordCorrection(wrong: string, correct: string): void
  }
  ```

---

### **PHASE 2: Intelligence Layer (Days 4-7)**
**Goal**: Make Oracle genuinely intelligent

#### Task 2.1: Gemini-Powered Intent Engine
- **File**: `simulator-ui/services/geminiIntentEngine.ts` (NEW)
- **Features**:
  ```typescript
  class GeminiIntentEngine {
    // Use Gemini's structured output for intent classification
    async classifyIntent(input: string, context: EnrichedContext): Promise<{
      intents: IntentAction[];
      confidence: number;
      reasoning: string;
      needsClarification: boolean;
      clarificationQuestions: string[];
    }>
    
    // Gemini generates optimal system prompt from current state
    private buildDynamicSystemPrompt(context: EnrichedContext): string
    
    // Use Gemini to resolve ambiguities
    async resolveAmbiguity(input: string, options: string[], context): Promise<string>
  }
  ```

- **System Prompt Template**:
  ```
  You are the Oracle AI of Kāraṇa OS smart glasses. You control a 9-layer OS.
  
  CURRENT SYSTEM STATE:
  - Hardware: Battery ${battery}%, Camera ${cameraState}, Display ${brightness}%
  - Network: ${peerCount} peers, Sync ${syncStatus}
  - Blockchain: Wallet ${walletExists ? 'exists' : 'none'}, Balance ${balance} KARA
  - Apps: Installed: [${installedApps}], Running: [${runningApps}]
  - User Profile: Security ${securityMode}, Frequent Commands: [${topCommands}]
  - Last 3 actions: [${recentActions}]
  - Conversation: [${last5Messages}]
  
  USER PREFERENCES:
  - "mom" refers to wallet address: ${contacts.mom}
  - Preferred brightness: ${prefBrightness}%
  - Usually checks battery at ${usualBatteryCheckTime}
  
  Your task: Classify user's intent into 1 or more actions.
  
  Available Operations: [list all 50+ operations]
  
  For input: "${userInput}"
  
  Return JSON:
  {
    "intents": [
      {"layer": "HARDWARE", "operation": "CAMERA_CAPTURE", "params": {}, "confidence": 0.95},
      {"layer": "INTELLIGENCE", "operation": "VISION_ANALYZE", "params": {}, "confidence": 0.90}
    ],
    "reasoning": "User wants to take a photo and identify the object",
    "needsClarification": false,
    "clarificationQuestions": []
  }
  
  Rules:
  1. If ambiguous (confidence < 0.7), set needsClarification=true
  2. Extract all entities (numbers, names, times, apps)
  3. Use conversation history to resolve pronouns
  4. Consider temporal context (time of day, recent actions)
  5. Be proactive - suggest related actions
  ```

#### Task 2.2: Multi-Step Action Planner
- **File**: `simulator-ui/services/actionPlanner.ts` (NEW)
- **Features**:
  ```typescript
  interface ActionPlan {
    steps: ActionStep[];
    estimatedDuration: number; // ms
    requiresConfirmation: boolean;
    resourceRequirements: {
      battery: number;  // mAh
      network: boolean;
      camera: boolean;
    };
    risks: string[]; // "Will spend 10 KARA", "Will take 30 seconds"
  }
  
  class ActionPlanner {
    // Convert intents → ordered execution plan
    async plan(intents: IntentAction[], context: EnrichedContext): Promise<ActionPlan>
    
    // Detect dependencies (wallet before transfer, install before open)
    private detectDependencies(intents: IntentAction[]): Dependency[]
    
    // Optimize execution order
    private optimizeOrder(steps: ActionStep[]): ActionStep[]
    
    // Check if plan is feasible
    private validatePlan(plan: ActionPlan, context: EnrichedContext): ValidationResult
  }
  ```

#### Task 2.3: Proactive Suggestion Engine
- **File**: `simulator-ui/services/suggestionEngine.ts` (NEW)
- **Features**:
  ```typescript
  class SuggestionEngine {
    // Generate contextual suggestions
    async generateSuggestions(context: EnrichedContext): Promise<string[]>
    
    // Pattern-based suggestions (user always does X after Y)
    private detectPatterns(history: Action[]): Pattern[]
    
    // Time-based suggestions (morning: battery check)
    private timeBasedSuggestions(time: Date): string[]
    
    // State-based suggestions (low battery → power save mode)
    private stateBasedSuggestions(state: CompleteSystemState): string[]
  }
  ```

---

### **PHASE 3: Execution Layer (Days 8-10)**
**Goal**: Reliable, transparent action execution

#### Task 3.1: Confirmation Manager
- **File**: `simulator-ui/services/confirmationManager.ts` (NEW)
- **Features**:
  ```typescript
  interface ConfirmationRequest {
    plan: ActionPlan;
    question: string; // "Send 10 KARA to mom (did:example:mom)?"
    options: ['Confirm', 'Cancel', 'Modify'];
    defaultOption: string;
    timeout: number; // Auto-cancel after 30s
  }
  
  class ConfirmationManager {
    // Show confirmation modal and wait for user response
    async requestConfirmation(request: ConfirmationRequest): Promise<'confirm' | 'cancel' | 'modify'>
    
    // Determine if action needs confirmation
    needsConfirmation(action: IntentAction): boolean
    
    // Generate clear, concise confirmation message
    formatConfirmation(plan: ActionPlan): string
  }
  ```

#### Task 3.2: Execution Orchestrator
- **File**: `simulator-ui/services/executionOrchestrator.ts` (NEW)
- **Features**:
  ```typescript
  class ExecutionOrchestrator {
    // Execute action plan with progress streaming
    async execute(plan: ActionPlan, onProgress: (update: ExecutionUpdate) => void): Promise<ExecutionResult>
    
    // Execute single action with retry logic
    private async executeAction(action: IntentAction): Promise<ActionResult>
    
    // Rollback on failure
    private async rollback(completedActions: IntentAction[]): Promise<void>
    
    // Handle errors gracefully
    private handleError(error: Error, action: IntentAction): ErrorResponse
  }
  ```

---

### **PHASE 4: Offline & Performance (Days 11-14)**
**Goal**: Fast, works offline

#### Task 4.1: Local Intent Model (TinyLlama/Phi-3)
- **File**: `simulator-ui/services/localIntentModel.ts` (NEW)
- **Model**: TinyLlama-1.1B or Phi-3-mini (runs in browser with WebGPU)
- **Training**: Fine-tune on Kāraṇa OS commands dataset
- **Features**:
  ```typescript
  class LocalIntentModel {
    // Load model into WebGPU
    async initialize(): Promise<void>
    
    // Fast local classification (50-100ms)
    async classify(input: string, context: string): Promise<IntentClassificationResult>
    
    // Update model with user corrections
    async updateWeights(correction: Correction): Promise<void>
  }
  ```

#### Task 4.2: Response Caching
- **File**: `simulator-ui/services/responseCache.ts` (NEW)
- **Features**:
  ```typescript
  class ResponseCache {
    // Cache frequent queries
    get(input: string, context: string): CachedResponse | null
    
    // Intelligent cache invalidation
    invalidate(trigger: StateChange): void
    
    // Prefetch likely next queries
    async prefetch(context: EnrichedContext): Promise<void>
  }
  ```

#### Task 4.3: Performance Optimization
- Parallel execution of independent actions
- Debouncing for voice input
- WebWorker for model inference
- IndexedDB for persistent cache

---

### **PHASE 5: Learning & Personalization (Days 15-18)**
**Goal**: Adapt to user over time

#### Task 5.1: Feedback Loop
- **File**: `simulator-ui/services/feedbackLoop.ts` (NEW)
- **Features**:
  ```typescript
  class FeedbackLoop {
    // Record interaction for learning
    recordInteraction(input: string, intent: Intent, result: ExecutionResult, userReaction: 'positive' | 'negative' | 'neutral'): void
    
    // Analyze patterns in feedback
    analyzePatterns(): LearningInsights
    
    // Update confidence thresholds
    updateThresholds(insights: LearningInsights): void
  }
  ```

#### Task 5.2: Personalization Engine
- **File**: `simulator-ui/services/personalizationEngine.ts` (NEW)
- **Features**:
  - Learn frequent command sequences ("every morning: battery → brightness 50%")
  - Adapt responses to user communication style (formal vs casual)
  - Remember user corrections ("actually send 5 KARA, not 10")
  - Predict next action based on patterns

---

### **PHASE 6: Advanced Features (Days 19-21)**
**Goal**: Make it insanely smart

#### Task 6.1: Multi-Modal Input Processing
- Voice + Gesture: "Open this app" (pointing at app icon)
- Voice + Gaze: "Send KARA to him" (looking at contact)
- Voice + Vision: "Buy this" (looking at product)

#### Task 6.2: Contextual Awareness
- Location-aware: "Where's the nearest coffee shop?" (uses GPS)
- Time-aware: "Good morning" at 8am → shows battery, weather, calendar
- App-aware: In YouTube → "next video", "full screen"

#### Task 6.3: Proactive Assistance
- Battery at 15% → "Enable power save mode?"
- Receiving KARA → "You received 10 KARA from Alice"
- Calendar reminder → "Meeting in 10 minutes, navigate there?"

---

## 🧪 TESTING STRATEGY

### Test Categories

#### 1. **Intent Classification Accuracy**
Test Dataset: 1000 diverse queries across all categories
- Simple commands: "take photo", "check balance"
- Complex commands: "take a photo, analyze it, and send to mom with 5 KARA"
- Ambiguous: "it's too bright" (display? camera? outside?)
- Misspelled: "chck batry staus"
- Contextual: "do that again", "send it to him"

**Success Criteria**: 95%+ accuracy

#### 2. **Response Time**
- Simple intent classification: <200ms
- Complex multi-intent: <500ms
- Full execution with confirmation: <2s
- Offline mode: <300ms

#### 3. **Conversation Flow**
Test multi-turn conversations:
```
User: "Is Instagram installed?"
Oracle: "No, but I can install it for you."
User: "Yes, do it"
Oracle: "Installing Instagram... Done! Want me to open it?"
User: "Yes"
Oracle: "Launching Instagram..."
```

#### 4. **Edge Cases**
- No internet connection
- Low battery during intensive task
- Conflicting commands
- Rapid-fire commands
- Voice input with background noise

---

## 📊 SUCCESS METRICS

### Quantitative
- **Intent Accuracy**: 95%+ correct classification
- **Response Time**: <500ms for 90% of queries
- **Success Rate**: 98%+ commands executed successfully
- **User Retry Rate**: <5% (user doesn't have to repeat command)
- **Offline Capability**: 80%+ commands work offline

### Qualitative
- **User Feedback**: "Feels natural and intelligent"
- **Error Handling**: Users understand errors and how to fix them
- **Proactivity**: Users appreciate suggestions
- **Trust**: Users comfortable with auto-execution

---

## 🚀 ROLLOUT STRATEGY

### Week 1: Internal Alpha
- Deploy to development team
- Collect edge case feedback
- Fix critical bugs

### Week 2: Closed Beta
- 50 early adopters
- Detailed feedback sessions
- Iterate on UX

### Week 3: Public Beta
- Open to all testers
- Monitor analytics
- Scale infrastructure

### Week 4: Production Release
- Full rollout
- Monitoring & support
- Continuous improvement

---

## 🛠️ TECHNICAL REQUIREMENTS

### Frontend
- React 19+ (current: ✅)
- TypeScript 5+ (current: ✅)
- WebGPU for local models
- IndexedDB for caching
- Web Workers for parallel processing

### Backend (Rust)
- Update `/api/ai/oracle` endpoint
- Add intent classification endpoint
- Add learning feedback endpoint
- Add user profile CRUD endpoints

### AI Models
- **Gemini 2.0 Flash**: Primary intent classification
- **TinyLlama-1.1B**: Offline fallback (fine-tuned)
- **Whisper-tiny**: Voice transcription (if voice enabled)

### Infrastructure
- Redis for response caching
- PostgreSQL for user profiles & learning data
- S3 for model weights

---

## 💰 RESOURCE ESTIMATE

### Development Time
- **Total**: 21 working days (3 weeks)
- **Team**: 2 engineers (1 AI/ML, 1 Full-stack)

### Costs
- Gemini API: ~$50/month for 10,000 users (with caching)
- Model hosting: $20/month (Cloudflare R2)
- Database: $30/month (Supabase)

**Total Monthly Cost**: ~$100 for 10,000 users = $0.01/user/month

---

## 🎯 PHASE 1 PRIORITY TASKS (Start Immediately)

1. **Create IntentClassifier with Gemini** (Day 1)
   - File: `simulator-ui/services/intentClassifier.ts`
   - Gemini structured output for intent classification
   - Test with 20 diverse queries

2. **Build ContextManager** (Day 2)
   - File: `simulator-ui/services/contextManager.ts`
   - Enrich context with conversation history + system state
   - Test pronoun resolution

3. **Implement Confirmation Manager** (Day 3)
   - File: `simulator-ui/services/confirmationManager.ts`
   - Modal UI for confirmations
   - Connect to execution flow

4. **Wire Everything to App.tsx** (Day 3)
   - Replace current `enhancedOracle.process()` flow
   - Add progress streaming UI
   - Test end-to-end

---

## 📝 NEXT STEPS

1. **Get approval on this plan**
2. **Set up development branch**: `oracle-intelligence-v2`
3. **Create Gemini API key** if not already available
4. **Start Phase 1, Task 1.1**: Enhanced Intent Classifier

---

**This is the path to making Oracle AI production-grade intelligent.**  
**Every user interaction flows through Oracle - it MUST be perfect.**

