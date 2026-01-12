# Universal Oracle AI (Phase 54)

## Overview

The Universal Oracle AI is Kāraṇa OS's agentic reasoning system, combining multi-step chain-of-thought reasoning with tool execution to answer complex queries. It bridges on-chain and off-chain worlds, providing verifiable AI services to smart contracts and users.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    UNIVERSAL ORACLE AI (Phase 54)                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                   Agentic Reasoning Engine                      │    │
│  │  - Chain-of-Thought: Multi-step planning                        │    │
│  │  - Tool Selection: Dynamic tool routing                         │    │
│  │  - Memory: Long-term context + feedback loops                   │    │
│  └────────┬───────────────────────────────────────────────────────┘    │
│           │                                                              │
│  ┌────────▼────────────────────────────────────────────────────────┐   │
│  │                     Tool Registry (6 tools)                      │   │
│  ├──────────────────────────────────────────────────────────────────┤   │
│  │ 1. os_exec       │ Execute shell commands                       │   │
│  │ 2. web_api       │ HTTP requests to external APIs               │   │
│  │ 3. app_proxy     │ Control Kāraṇa apps                          │   │
│  │ 4. gen_creative  │ Generate images/audio/video                  │   │
│  │ 5. memory_rag    │ Retrieve from long-term memory               │   │
│  │ 6. health_sensor │ Query health data (HR, SpO2)                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│           │                                                              │
│  ┌────────▼────────────────────────────────────────────────────────┐   │
│  │              ZK Proof Generation (Privacy Layer)                 │   │
│  │  - Prove tool execution without revealing inputs                 │   │
│  │  - On-chain verification (Groth16)                               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Agentic Reasoning Engine

**Purpose**: Decompose complex queries into multi-step plans, execute tools, and synthesize answers.

**Reasoning Loop**:
```typescript
class AgenticOracle {
  async process(query: string, context: Context): Promise<OracleResponse> {
    const steps: ReasoningStep[] = [];
    let currentThought = query;
    
    // Max 10 reasoning steps
    for (let i = 0; i < 10; i++) {
      // 1. Generate next thought
      const thought = await this.llm.generateThought(currentThought, steps, context);
      
      // 2. Decide if tool needed
      const toolDecision = await this.decideTool(thought);
      
      if (toolDecision.tool) {
        // 3. Execute tool
        const observation = await this.executeTool(
          toolDecision.tool,
          toolDecision.input
        );
        
        steps.push({
          thought,
          tool: toolDecision.tool,
          input: toolDecision.input,
          observation,
          timestamp: Date.now(),
        });
        
        // 4. Check if answer complete
        if (this.isComplete(observation, query)) {
          return {
            answer: this.synthesizeAnswer(steps),
            reasoning: steps,
            confidence: this.calculateConfidence(steps),
            zkProof: await this.generateProof(steps),
          };
        }
        
        currentThought = this.refineQuery(query, observation);
      } else {
        // Direct answer
        steps.push({ thought, timestamp: Date.now() });
        return {
          answer: thought,
          reasoning: steps,
          confidence: 0.85,
        };
      }
    }
    
    return {
      answer: "Unable to answer with confidence.",
      reasoning: steps,
      confidence: 0.3,
    };
  }
}
```

**Example Reasoning Chain**:
```
Query: "What's the weather in Paris and should I bring an umbrella?"

Step 1:
  Thought: "I need weather data for Paris"
  Tool: web_api
  Input: { url: "https://api.weather.com/v3/wx/forecast/daily/5day?geocode=48.8566,2.3522" }
  Observation: { temp: 15°C, precipitation: 70%, wind: 20km/h }

Step 2:
  Thought: "70% rain is significant, check user preferences"
  Tool: memory_rag
  Input: { query: "umbrella preferences" }
  Observation: "User brings umbrella when rain >50%"

Step 3:
  Thought: "Synthesize recommendation"
  Answer: "It's 15°C in Paris with 70% chance of rain. Based on your 
           preferences, you should bring an umbrella."
  Confidence: 0.92
```

---

### 2. Tool Registry

**Tool Interface**:
```typescript
interface Tool {
  name: string;
  description: string;
  inputSchema: JSONSchema;
  execute(input: any, context: Context): Promise<ToolOutput>;
}

interface ToolOutput {
  success: boolean;
  data: any;
  error?: string;
  metadata: {
    executionTime: number;
    cost?: number; // For API calls
  };
}
```

#### Tool 1: OS Exec

**Purpose**: Execute shell commands on the device.

```typescript
class OSExecTool implements Tool {
  name = 'os_exec';
  description = 'Execute shell commands on the local system';
  inputSchema = {
    type: 'object',
    properties: {
      command: { type: 'string' },
      args: { type: 'array', items: { type: 'string' } },
    },
    required: ['command'],
  };
  
  async execute(input: { command: string; args?: string[] }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      // Security: Whitelist allowed commands
      if (!this.isAllowed(input.command)) {
        throw new Error(`Command not allowed: ${input.command}`);
      }
      
      const result = await this.runCommand(input.command, input.args || []);
      
      return {
        success: true,
        data: { stdout: result.stdout, stderr: result.stderr, exitCode: result.exitCode },
        metadata: { executionTime: Date.now() - startTime },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
  
  private isAllowed(command: string): boolean {
    const whitelist = ['ls', 'cat', 'grep', 'find', 'wc', 'head', 'tail'];
    return whitelist.includes(command);
  }
}
```

#### Tool 2: Web API

**Purpose**: Make HTTP requests to external APIs.

```typescript
class WebAPITool implements Tool {
  name = 'web_api';
  description = 'Make HTTP requests to external APIs';
  inputSchema = {
    type: 'object',
    properties: {
      method: { type: 'string', enum: ['GET', 'POST', 'PUT', 'DELETE'] },
      url: { type: 'string', format: 'uri' },
      headers: { type: 'object' },
      body: { type: 'object' },
    },
    required: ['method', 'url'],
  };
  
  async execute(input: {
    method: string;
    url: string;
    headers?: Record<string, string>;
    body?: any;
  }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      const response = await fetch(input.url, {
        method: input.method,
        headers: input.headers,
        body: input.body ? JSON.stringify(input.body) : undefined,
      });
      
      const data = await response.json();
      
      return {
        success: response.ok,
        data,
        metadata: {
          executionTime: Date.now() - startTime,
          cost: this.calculateCost(input.url), // Based on API pricing
        },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
}
```

#### Tool 3: App Proxy

**Purpose**: Control Kāraṇa applications (timer, navigation, etc.).

```typescript
class AppProxyTool implements Tool {
  name = 'app_proxy';
  description = 'Control Kāraṇa applications';
  inputSchema = {
    type: 'object',
    properties: {
      app: { type: 'string', enum: ['timer', 'navigation', 'social', 'settings', 'wellness'] },
      action: { type: 'string' },
      params: { type: 'object' },
    },
    required: ['app', 'action'],
  };
  
  async execute(input: {
    app: string;
    action: string;
    params?: any;
  }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      const appService = this.getAppService(input.app);
      const result = await appService[input.action](input.params);
      
      return {
        success: true,
        data: result,
        metadata: { executionTime: Date.now() - startTime },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
}

// Example usage:
// { app: 'timer', action: 'start', params: { duration: 300, label: 'Tea' } }
// { app: 'navigation', action: 'navigate', params: { destination: 'Home' } }
```

#### Tool 4: Gen Creative

**Purpose**: Generate images, audio, or video using AI models.

```typescript
class GenCreativeTool implements Tool {
  name = 'gen_creative';
  description = 'Generate images, audio, or video';
  inputSchema = {
    type: 'object',
    properties: {
      type: { type: 'string', enum: ['image', 'audio', 'video'] },
      prompt: { type: 'string' },
      style: { type: 'string' },
    },
    required: ['type', 'prompt'],
  };
  
  async execute(input: {
    type: 'image' | 'audio' | 'video';
    prompt: string;
    style?: string;
  }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      let data;
      
      switch (input.type) {
        case 'image':
          data = await this.generateImage(input.prompt, input.style);
          break;
        case 'audio':
          data = await this.generateAudio(input.prompt);
          break;
        case 'video':
          data = await this.generateVideo(input.prompt);
          break;
      }
      
      return {
        success: true,
        data: { url: data.url, format: data.format },
        metadata: {
          executionTime: Date.now() - startTime,
          cost: this.calculateCost(input.type),
        },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
  
  private async generateImage(prompt: string, style?: string): Promise<{ url: string; format: string }> {
    // Use Stable Diffusion or DALL-E API
    const response = await fetch('https://api.openai.com/v1/images/generations', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${process.env.OPENAI_API_KEY}` },
      body: JSON.stringify({ prompt, style, size: '512x512' }),
    });
    
    const data = await response.json();
    return { url: data.data[0].url, format: 'png' };
  }
}
```

#### Tool 5: Memory RAG

**Purpose**: Retrieve information from long-term memory using RAG (Retrieval-Augmented Generation).

```typescript
class MemoryRAGTool implements Tool {
  name = 'memory_rag';
  description = 'Retrieve from long-term memory';
  inputSchema = {
    type: 'object',
    properties: {
      query: { type: 'string' },
      limit: { type: 'number', default: 5 },
    },
    required: ['query'],
  };
  
  private vectorDB: VectorDatabase;
  
  async execute(input: { query: string; limit?: number }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      // 1. Embed query
      const queryEmbedding = await this.embedText(input.query);
      
      // 2. Search vector DB
      const results = await this.vectorDB.search(queryEmbedding, input.limit || 5);
      
      // 3. Rerank by relevance
      const reranked = this.rerank(results, input.query);
      
      return {
        success: true,
        data: reranked.map(r => ({
          content: r.content,
          similarity: r.score,
          timestamp: r.timestamp,
        })),
        metadata: { executionTime: Date.now() - startTime },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
  
  private async embedText(text: string): Promise<number[]> {
    // Use sentence-transformers or OpenAI embeddings
    const response = await fetch('https://api.openai.com/v1/embeddings', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${process.env.OPENAI_API_KEY}` },
      body: JSON.stringify({ model: 'text-embedding-ada-002', input: text }),
    });
    
    const data = await response.json();
    return data.data[0].embedding;
  }
}
```

#### Tool 6: Health Sensor

**Purpose**: Query health sensor data (heart rate, SpO2, steps, etc.).

```typescript
class HealthSensorTool implements Tool {
  name = 'health_sensor';
  description = 'Query health sensor data';
  inputSchema = {
    type: 'object',
    properties: {
      metric: { type: 'string', enum: ['heart_rate', 'spo2', 'steps', 'calories', 'sleep'] },
      timeRange: { type: 'string', enum: ['current', 'today', 'week', 'month'] },
    },
    required: ['metric'],
  };
  
  async execute(input: {
    metric: string;
    timeRange?: string;
  }): Promise<ToolOutput> {
    const startTime = Date.now();
    
    try {
      const data = await this.querySensor(input.metric, input.timeRange || 'current');
      
      return {
        success: true,
        data,
        metadata: { executionTime: Date.now() - startTime },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: { executionTime: Date.now() - startTime },
      };
    }
  }
  
  private async querySensor(metric: string, timeRange: string): Promise<any> {
    // Read from health manager (Layer 1)
    switch (metric) {
      case 'heart_rate':
        return { value: await this.healthManager.getHeartRate(), unit: 'bpm' };
      case 'spo2':
        return { value: await this.healthManager.getSpO2(), unit: '%' };
      case 'steps':
        return { value: await this.healthManager.getSteps(timeRange), unit: 'steps' };
      default:
        throw new Error(`Unknown metric: ${metric}`);
    }
  }
}
```

---

### 3. ZK Proof Generation

**Purpose**: Generate zero-knowledge proofs for tool executions to preserve privacy while proving correctness.

**Proof Structure**:
```typescript
interface ZKProof {
  proof: Uint8Array;      // Groth16 proof (compressed)
  publicInputs: string[]; // Public signals (hashed outputs)
  circuit: string;        // Circuit identifier
}
```

**Proof Generation**:
```typescript
class ZKProofGenerator {
  async proveToolExecution(
    tool: string,
    input: any,
    output: any
  ): Promise<ZKProof> {
    // 1. Select circuit based on tool
    const circuit = this.getCircuit(tool);
    
    // 2. Prepare witness
    const witness = {
      toolName: this.hashString(tool),
      inputHash: this.hashObject(input),
      outputHash: this.hashObject(output),
      timestamp: Date.now(),
    };
    
    // 3. Generate proof
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      witness,
      circuit.wasmFile,
      circuit.zkeyFile
    );
    
    return {
      proof: this.compressProof(proof),
      publicInputs: publicSignals,
      circuit: circuit.name,
    };
  }
  
  // Verify proof on-chain
  async verifyOnChain(proof: ZKProof, contractAddress: string): Promise<boolean> {
    const contract = new ethers.Contract(contractAddress, VERIFIER_ABI, this.provider);
    
    const isValid = await contract.verifyProof(
      proof.proof,
      proof.publicInputs
    );
    
    return isValid;
  }
}
```

**ZK Circuits** (Circom):
```circom
// Circuit for proving web API call without revealing URL/response
template WebAPIProof() {
    signal input url_hash;
    signal input response_hash;
    signal input timestamp;
    signal output commitment;
    
    // Commitment = hash(url_hash, response_hash, timestamp)
    component hasher = Poseidon(3);
    hasher.inputs[0] <== url_hash;
    hasher.inputs[1] <== response_hash;
    hasher.inputs[2] <== timestamp;
    
    commitment <== hasher.out;
}

component main = WebAPIProof();
```

---

### 4. Long-Term Memory

**Purpose**: Store conversation history, user preferences, and learned patterns for personalization.

**Memory Structure**:
```typescript
interface MemoryEntry {
  id: string;
  type: 'conversation' | 'preference' | 'fact' | 'feedback';
  content: string;
  embedding: number[];  // 1536-dim vector (OpenAI ada-002)
  timestamp: number;
  importance: number;   // 0.0-1.0
  accessCount: number;
}
```

**Memory Manager**:
```typescript
class MemoryManager {
  private vectorDB: VectorDatabase;
  private maxMemories = 10000;
  
  async store(content: string, type: string): Promise<void> {
    const embedding = await this.embedText(content);
    const importance = this.calculateImportance(content, type);
    
    const memory: MemoryEntry = {
      id: crypto.randomUUID(),
      type,
      content,
      embedding,
      timestamp: Date.now(),
      importance,
      accessCount: 0,
    };
    
    await this.vectorDB.insert(memory);
    
    // Evict old low-importance memories
    await this.evictOldMemories();
  }
  
  async recall(query: string, k: number = 5): Promise<MemoryEntry[]> {
    const queryEmbedding = await this.embedText(query);
    const results = await this.vectorDB.search(queryEmbedding, k);
    
    // Update access count
    for (const result of results) {
      await this.vectorDB.update(result.id, { accessCount: result.accessCount + 1 });
    }
    
    return results;
  }
  
  private calculateImportance(content: string, type: string): number {
    let importance = 0.5;
    
    // User preferences are more important
    if (type === 'preference') importance += 0.3;
    
    // Feedback is important for learning
    if (type === 'feedback') importance += 0.2;
    
    // Long content is more important
    if (content.length > 200) importance += 0.1;
    
    return Math.min(importance, 1.0);
  }
  
  private async evictOldMemories(): Promise<void> {
    const count = await this.vectorDB.count();
    
    if (count > this.maxMemories) {
      // Remove oldest, lowest importance memories
      const toRemove = count - this.maxMemories;
      await this.vectorDB.deleteLowest('importance', toRemove);
    }
  }
}
```

---

## Oracle Request Flow

```
User Query
    │
    ▼
┌────────────────────────────────────┐
│ Intent Classification (Layer 6)    │
│ "What's the weather?" → Weather    │
└──────────────┬─────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│ Oracle Bridge (Layer 4)             │
│ Create OracleRequest                │
└──────────────┬─────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│ Agentic Reasoning Loop              │
│ Step 1: "Need weather API"          │
│ Step 2: Call web_api tool           │
│ Step 3: Parse response              │
└──────────────┬─────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│ ZK Proof Generation                 │
│ Prove tool execution                │
└──────────────┬─────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│ Submit to Blockchain (Layer 3)     │
│ OracleResponse transaction          │
└──────────────┬─────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│ UI Rendering (Layer 7)              │
│ Display answer + reasoning          │
└────────────────────────────────────┘
```

---

## Performance Metrics

```
┌─ Oracle Performance ────────────────────┐
│ Reasoning Loop: 200-500ms                │
│ Tool Execution:                          │
│   - os_exec: 10-50ms                     │
│   - web_api: 100-500ms                   │
│   - app_proxy: 20-100ms                  │
│   - gen_creative: 2-10s                  │
│   - memory_rag: 50-200ms                 │
│   - health_sensor: 5-20ms                │
│                                           │
│ ZK Proof Generation: 300-800ms           │
│ On-chain Submission: 12s (block time)    │
│                                           │
│ Total (Simple): 300-1000ms               │
│ Total (Complex): 2-15s                   │
└───────────────────────────────────────────┘
```

---

## Code References

- `simulator-ui/services/oracleService.ts`: Oracle interface
- `simulator-ui/services/enhancedOracleAI.ts`: Agentic reasoning
- `karana-core/src/oracle/universal.rs`: Oracle bridge (Rust)

---

## Summary

The Universal Oracle AI provides:
- **Agentic Reasoning**: Multi-step chain-of-thought
- **Tool Execution**: 6 tools for diverse capabilities
- **ZK Proofs**: Privacy-preserving verification
- **Long-Term Memory**: RAG-based recall
- **On-Chain Integration**: Verifiable AI for smart contracts

This bridges the gap between AI intelligence and blockchain trust.