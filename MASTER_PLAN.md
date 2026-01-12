# Kāraṇa OS - AI-First Architecture Master Plan

## VISION
Make AI the ONLY way users interact with the OS. Every command, every query, every action flows through AI → OS → Response.

---

## CURRENT STATE ASSESSMENT

### What's Broken:
1. **Commercial Viability**: Gemini API requires individual API keys (impossible for consumer product)
2. **Reliability**: AI crashes on simple queries ("hi", "who's president")
3. **Trust**: If AI can't open camera, how can we trust it with wallet/blockchain?
4. **Fragility**: Multiple layers with undefined property errors
5. **No Fallback**: When Gemini fails, system gives up

### What's Working:
1. ✅ 9-layer OS architecture (good foundation)
2. ✅ System state management (systemState.ts)
3. ✅ System context (systemContext.ts)
4. ✅ Action execution layer (executeEnhancedAction in App.tsx)
5. ✅ UI components (ChatInterface, HUD, etc.)

---

## SOLUTION ARCHITECTURE

### Two-Tier Intelligence System

```
USER INPUT
    ↓
┌─────────────────────────────────────────────────┐
│  TIER 1: LOCAL AI (Always Available)           │
│  - Lightweight model (1-3GB)                    │
│  - Runs locally (no API, no internet)          │
│  - Handles 90% of OS commands                   │
│  - Intent classification, entity extraction     │
│  - Command routing to OS layers                 │
│  - Fast, private, reliable                      │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│  TIER 2: CLOUD AI (Optional Enhancement)       │
│  - Gemini/GPT for complex queries               │
│  - User opts in (optional)                      │
│  - General knowledge, advanced reasoning        │
│  - Only for non-OS operations                   │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│  OS EXECUTION LAYER                              │
│  - Takes structured intents from AI             │
│  - Routes to appropriate OS layer               │
│  - Executes actions (camera, wallet, etc.)     │
│  - Returns results to AI                        │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│  RESPONSE GENERATION                             │
│  - AI formats results naturally                 │
│  - Presents to user conversationally            │
│  - Suggests follow-up actions                   │
└─────────────────────────────────────────────────┘
```

---

## PHASE 1: LOCAL AI FOUNDATION (Week 1)

### Goal: Replace Gemini with local model that ALWAYS works

### Options for Local AI:

#### Option 1: ONNX Runtime (Recommended for Web)
- **Model**: Phi-3-mini (3.8B params, 2GB)
- **Speed**: 50-100 tokens/sec on CPU
- **Deployment**: Runs in browser with ONNX Runtime Web
- **Pros**: No server needed, private, fast
- **Cons**: Model download ~2GB first time

#### Option 2: WebLLM (Browser-based)
- **Model**: Llama-3.2-1B or Phi-3.5-mini
- **Speed**: 20-50 tokens/sec with WebGPU
- **Deployment**: Pure browser, no backend
- **Pros**: Completely client-side
- **Cons**: Requires WebGPU support

#### Option 3: Rust Backend with Candle (Best Performance)
- **Model**: Phi-3-mini or Mistral-7B-Instruct
- **Speed**: 200-500 tokens/sec
- **Deployment**: Runs in Rust backend (already have one)
- **Pros**: Fast, reliable, full control
- **Cons**: Requires backend running

#### **RECOMMENDED: Hybrid Approach**
1. **Rust Backend** (candle + Phi-3-mini): Primary for OS commands
2. **Fallback to pattern matching**: If backend down
3. **Optional cloud**: For general knowledge (user choice)

### Tasks:
- [ ] Set up candle-rs in Rust backend
- [ ] Download Phi-3-mini-instruct model
- [ ] Create `/api/ai/process` endpoint
- [ ] Build intent classifier with model
- [ ] Test: "open camera", "battery status", "take photo"

---

## PHASE 2: INTENT UNDERSTANDING (Week 1-2)

### Goal: AI understands ANY user request and maps to OS operations

### Intent Classification System:

```typescript
interface Intent {
  // What layer of OS to touch
  layer: 'HARDWARE' | 'NETWORK' | 'BLOCKCHAIN' | 'INTELLIGENCE' | 
         'INTERFACE' | 'APPLICATIONS' | 'SYSTEM_SERVICES' | 'SPATIAL';
  
  // Specific operation
  operation: string;  // e.g., 'CAMERA_CAPTURE', 'WALLET_TRANSFER'
  
  // Parameters extracted from user input
  params: Record<string, any>;
  
  // Confidence score
  confidence: number;
  
  // Does this need confirmation?
  requiresConfirmation: boolean;
  
  // What resources does this need?
  resources: {
    battery?: number;  // % needed
    storage?: number;  // MB needed
    network?: boolean;
    permissions?: string[];
  };
}

interface AIResponse {
  // Natural language response to user
  message: string;
  
  // Actions to execute
  intents: Intent[];
  
  // Suggested follow-ups
  suggestions: string[];
  
  // Does AI need more info?
  needsClarification: boolean;
  clarificationQuestion?: string;
}
```

### Training Data Structure:
```json
{
  "utterance": "take a picture",
  "intents": [{
    "layer": "HARDWARE",
    "operation": "CAMERA_CAPTURE",
    "params": {},
    "confidence": 0.95
  }]
},
{
  "utterance": "how much battery do I have left",
  "intents": [{
    "layer": "HARDWARE",
    "operation": "POWER_STATUS",
    "params": {},
    "confidence": 0.98
  }]
},
{
  "utterance": "send 5 KARA to mom",
  "intents": [{
    "layer": "BLOCKCHAIN",
    "operation": "WALLET_TRANSFER",
    "params": {
      "amount": 5,
      "recipient": "{{contact:mom}}"
    },
    "confidence": 0.92,
    "requiresConfirmation": true
  }]
}
```

### Tasks:
- [ ] Build training dataset (500+ examples covering all OS operations)
- [ ] Create system prompt that teaches model about OS layers
- [ ] Implement entity extraction (amounts, times, contacts, apps)
- [ ] Test with edge cases (ambiguous, misspelled, multi-intent)

---

## PHASE 3: OS EXECUTION LAYER (Week 2)

### Goal: Reliable execution of any intent AI produces

### Execution Architecture:

```typescript
class OSExecutor {
  async execute(intent: Intent): Promise<ExecutionResult> {
    // 1. Validate intent
    if (!this.canExecute(intent)) {
      return { success: false, error: "Insufficient permissions" };
    }
    
    // 2. Check resources
    const resources = await this.checkResources(intent.resources);
    if (!resources.available) {
      return { success: false, error: "Insufficient resources" };
    }
    
    // 3. Route to appropriate handler
    const handler = this.getHandler(intent.layer, intent.operation);
    
    // 4. Execute with timeout
    const result = await Promise.race([
      handler.execute(intent.params),
      timeout(30000)  // 30s timeout
    ]);
    
    // 5. Update system state
    systemState.recordExecution(intent, result);
    
    // 6. Return result
    return result;
  }
}
```

### Handler Structure:

```typescript
// Layer 1 - Hardware Handlers
class HardwareHandler {
  async CAMERA_CAPTURE(params: any) {
    const stream = await navigator.mediaDevices.getUserMedia({ video: true });
    const photo = await captureFrame(stream);
    return { success: true, photo };
  }
  
  async POWER_STATUS(params: any) {
    const battery = await systemState.getLayer('layer1_hardware').power;
    return { success: true, battery };
  }
  
  async DISPLAY_BRIGHTNESS(params: { value: number }) {
    await setDisplayBrightness(params.value);
    return { success: true, brightness: params.value };
  }
}

// Layer 3 - Blockchain Handlers
class BlockchainHandler {
  async WALLET_CREATE(params: any) {
    const wallet = await karanaApi.createWallet();
    return { success: true, wallet };
  }
  
  async WALLET_TRANSFER(params: { amount: number, recipient: string }) {
    // Validate
    if (params.amount > balance) {
      return { success: false, error: "Insufficient balance" };
    }
    
    // Execute
    const tx = await karanaApi.sendTransaction(params);
    return { success: true, transaction: tx };
  }
}
```

### Tasks:
- [ ] Implement handlers for all 9 layers
- [ ] Add resource checking (battery, storage, network)
- [ ] Add error handling and retry logic
- [ ] Add execution timeouts
- [ ] Test each handler independently

---

## PHASE 4: RESPONSE GENERATION (Week 3)

### Goal: AI explains what it did naturally

### Response Templates:

```typescript
const RESPONSE_TEMPLATES = {
  CAMERA_CAPTURE: {
    success: "📸 Photo captured! {preview}",
    failure: "Couldn't access camera: {error}. Check permissions?",
    suggestions: ["Analyze this photo", "Take another", "Open gallery"]
  },
  
  POWER_STATUS: {
    success: "🔋 Battery at {level}% ({time} remaining)",
    failure: "Couldn't check battery status",
    suggestions: ["Enable power saving", "Close running apps", "Check what's draining battery"]
  },
  
  WALLET_TRANSFER: {
    success: "✅ Sent {amount} KARA to {recipient}. Transaction: {txId}",
    failure: "Transfer failed: {error}",
    suggestions: ["Check balance", "View transaction history", "Retry transfer"]
  }
};
```

### Dynamic Response Generation:

```typescript
async function generateResponse(
  intent: Intent,
  result: ExecutionResult,
  context: SystemContext
): Promise<AIResponse> {
  
  // Get template
  const template = RESPONSE_TEMPLATES[intent.operation];
  
  // Fill in variables
  let message = template[result.success ? 'success' : 'failure'];
  message = fillTemplate(message, { ...intent.params, ...result });
  
  // Add context-aware suggestions
  const suggestions = await getSuggestions(intent, result, context);
  
  return {
    message,
    suggestions,
    data: result
  };
}
```

### Tasks:
- [ ] Create response templates for all operations
- [ ] Implement template variable filling
- [ ] Add context-aware suggestions
- [ ] Handle partial success (multi-intent)
- [ ] Test response quality

---

## PHASE 5: LEARNING & IMPROVEMENT (Week 3-4)

### Goal: AI gets better over time

### User Profile:

```typescript
interface UserProfile {
  // Frequently used commands
  frequentCommands: Map<string, number>;
  
  // Known contacts
  contacts: Map<string, string>;  // name → address
  
  // Preferences
  preferences: {
    defaultBrightness: number;
    defaultVolume: number;
    securityMode: 'paranoid' | 'balanced' | 'convenience';
    voiceInputEnabled: boolean;
  };
  
  // Usage patterns
  patterns: {
    morningRoutine: string[];  // Commands at 6-9am
    eveningRoutine: string[];  // Commands at 6-9pm
    workHours: string[];       // Commands at 9-5pm
  };
  
  // Success/failure tracking
  statistics: {
    totalCommands: number;
    successRate: number;
    averageConfidence: number;
  };
}
```

### Context Manager:

```typescript
class ContextManager {
  // Remember conversation
  private history: Message[] = [];
  
  addUserMessage(text: string) {
    this.history.push({ role: 'user', content: text, timestamp: Date.now() });
  }
  
  addAssistantMessage(text: string) {
    this.history.push({ role: 'assistant', content: text, timestamp: Date.now() });
  }
  
  // Resolve references
  resolvePronouns(text: string): string {
    // "send it to him" → "send photo to john"
    return text;
  }
  
  // Get relevant context for current query
  getContext(): EnrichedContext {
    return {
      recentMessages: this.history.slice(-10),
      lastAction: this.getLastAction(),
      timeOfDay: this.getTimeOfDay(),
      location: this.getLocation(),
      recentlyUsedApps: this.getRecentApps()
    };
  }
}
```

### Tasks:
- [ ] Implement user profile storage (localStorage)
- [ ] Track command frequency
- [ ] Learn contact names
- [ ] Detect usage patterns
- [ ] Improve suggestions based on history

---

## PHASE 6: ADVANCED INTELLIGENCE (Week 4)

### Multi-Intent Handling:

```typescript
// "take a photo and send 5 KARA to mom"
const intents = [
  {
    layer: 'HARDWARE',
    operation: 'CAMERA_CAPTURE',
    params: {},
    sequence: 1
  },
  {
    layer: 'BLOCKCHAIN',
    operation: 'WALLET_TRANSFER',
    params: { amount: 5, recipient: '{{contact:mom}}' },
    sequence: 2,
    dependencies: [1]  // Wait for intent 1
  }
];
```

### Proactive Assistance:

```typescript
class ProactiveAI {
  // Detect low battery while user is active
  async checkBattery() {
    if (battery < 20% && user.isActive) {
      suggest("Battery low. Enable power saving mode?");
    }
  }
  
  // Detect user patterns
  async checkRoutines() {
    if (time === "7:00 AM" && user.morningRoutine.includes("weather")) {
      proactivelySay("Good morning! Weather today: sunny, 75°F");
    }
  }
  
  // Detect potential issues
  async checkHealth() {
    if (storage < 10%) {
      suggest("Storage almost full. Want me to clean cache?");
    }
  }
}
```

### Tasks:
- [ ] Implement multi-intent parsing
- [ ] Add dependency resolution
- [ ] Build proactive monitoring
- [ ] Add pattern detection
- [ ] Test complex scenarios

---

## IMPLEMENTATION ROADMAP

### Week 1: Local AI Foundation
**Days 1-2**: Set up Rust backend with candle-rs
- Install candle dependencies
- Download Phi-3-mini model
- Create `/api/ai/process` endpoint
- Test basic inference

**Days 3-4**: Intent Classification
- Build training dataset (100 examples)
- Create system prompt
- Implement entity extraction
- Test with basic commands

**Days 5-7**: Integration
- Connect frontend to local AI
- Remove Gemini dependency
- Add fallback pattern matching
- Test end-to-end flow

### Week 2: OS Execution
**Days 8-10**: Layer Handlers
- Implement Layer 1 (Hardware) handlers
- Implement Layer 3 (Blockchain) handlers
- Implement Layer 8 (Applications) handlers
- Add resource checking

**Days 11-12**: Error Handling
- Add timeouts
- Add retry logic
- Handle permission errors
- Handle resource errors

**Days 13-14**: Testing
- Test each handler
- Test error cases
- Test timeouts
- Integration testing

### Week 3: Polish & Intelligence
**Days 15-17**: Response Generation
- Create response templates
- Implement suggestion engine
- Add context-aware responses
- Test response quality

**Days 18-20**: Learning System
- Implement user profile
- Track command frequency
- Learn contacts
- Detect patterns

**Day 21**: Testing & Refinement
- End-to-end testing
- Edge case testing
- Performance testing
- Bug fixing

### Week 4: Advanced Features
**Days 22-24**: Multi-Intent
- Parse complex commands
- Resolve dependencies
- Execute sequences
- Test multi-step operations

**Days 25-27**: Proactive AI
- Implement monitoring
- Add proactive suggestions
- Detect patterns
- Test automation

**Day 28**: Final Polish
- Performance optimization
- UI/UX refinement
- Documentation
- Deployment preparation

---

## SUCCESS METRICS

### Reliability:
- ✅ 95%+ success rate on basic commands
- ✅ Zero crashes on any user input
- ✅ Works offline (no API dependency)

### Intelligence:
- ✅ Understands 90%+ of natural language commands
- ✅ Correctly extracts entities (names, amounts, times)
- ✅ Handles multi-intent commands
- ✅ Resolves pronouns and references

### Performance:
- ✅ <500ms response time (local AI)
- ✅ <50MB memory overhead
- ✅ Works on mobile devices

### Trust:
- ✅ Camera control: 100% reliable
- ✅ Wallet operations: 100% reliable with confirmation
- ✅ System operations: 100% reliable with safety checks

---

## ALTERNATIVE: SIMPLE BUT EFFECTIVE

If local AI is too complex initially, we can start with:

### Hybrid Pattern + Template System:

```typescript
// 1. Pattern matching for OS commands (fast, reliable)
const patterns = {
  camera: /take (a )?photo|capture|snap|picture/i,
  battery: /battery|power|charge/i,
  brightness: /brightness|display/i,
  wallet_create: /create wallet|new wallet/i,
  wallet_balance: /balance|how much|funds/i,
  wallet_transfer: /send|transfer|pay/i,
};

// 2. Entity extraction
function extractEntities(text) {
  const amount = text.match(/\d+(\.\d+)?/)?.[0];
  const contact = text.match(/to (\w+)/)?.[1];
  return { amount, contact };
}

// 3. Template-based responses
const responses = {
  camera_success: "📸 Photo captured!",
  battery_status: "🔋 Battery at {level}% ({time} remaining)",
  wallet_transfer: "Sending {amount} KARA to {recipient}. Confirm?",
};

// 4. Gemini ONLY for unknown queries (optional)
if (!matchedPattern && geminiEnabled) {
  // Ask Gemini for help
}
```

This gives us:
- ✅ 100% reliable for OS commands
- ✅ No API dependency for core functions
- ✅ Optional Gemini for general questions
- ✅ Can ship today, improve over time

---

## RECOMMENDATION

**Start with Phase 1 Option 3 (Rust Backend + Phi-3-mini)**

Why?
1. You already have Rust backend infrastructure
2. Phi-3-mini is small (2GB), fast, and smart enough
3. No browser compatibility issues
4. Full control over model behavior
5. Can fine-tune for OS-specific tasks
6. Commercially viable (no API costs)

Then iterate:
- Week 1: Get basic AI working locally
- Week 2: Perfect OS command execution
- Week 3: Add intelligence & learning
- Week 4: Advanced features & polish

This gives us a TRUSTWORTHY, RELIABLE, COMMERCIALLY VIABLE AI-first OS.

---

## NEXT STEPS

I can start implementing immediately. Which approach do you prefer?

**Option A**: Rust backend with local model (recommended)
**Option B**: Simple pattern matching + templates (ship faster)
**Option C**: Hybrid (patterns for OS, optional cloud for general)

Once you decide, I'll begin Phase 1 implementation right away.
