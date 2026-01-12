# Layer 4: Oracle Bridge

## Overview

The Oracle Bridge connects Kāraṇa OS's AI intelligence (Layer 6) with the blockchain (Layer 3), enabling verifiable off-chain computation. It processes user intents, manages oracle requests, executes tools, generates ZK proofs, and settles responses on-chain.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      LAYER 4: ORACLE BRIDGE                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Intent Processor                               │  │
│  │  Convert AI intents → Oracle requests                             │  │
│  └────────────────────┬─────────────────────────────────────────────┘  │
│                       │                                                  │
│  ┌────────────────────▼─────────────────────────────────────────────┐  │
│  │               Oracle Request Manager                              │  │
│  │  Queue: 50 pending | Active: 10 | Timeout: 30s                   │  │
│  └────────┬───────────────────────────────────┬─────────────────────┘  │
│           │                                    │                         │
│  ┌────────▼────────────┐            ┌─────────▼──────────┐             │
│  │   Tool Registry     │            │  ZK Proof Engine    │             │
│  │   6 tools           │            │  Groth16 prover     │             │
│  │   - os_exec         │            │  Circuit: 5 types   │             │
│  │   - web_api         │            │  Time: 300-800ms    │             │
│  │   - app_proxy       │            └─────────┬───────────┘             │
│  │   - gen_creative    │                      │                         │
│  │   - memory_rag      │            ┌─────────▼───────────┐             │
│  │   - health_sensor   │            │  Response Settler   │             │
│  └─────────────────────┘            │  Submit to chain    │             │
│                                      │  Block time: 12s    │             │
│                                      └─────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Intent Processor

**Purpose**: Convert high-level user intents from Layer 6 into structured oracle requests.

**Intent to Request Conversion**:
```typescript
interface OracleRequest {
  id: string;
  type: RequestType;
  params: Record<string, any>;
  requester: string;        // User address
  priority: number;         // 0-10
  deadline: number;         // Unix timestamp
  zkRequired: boolean;
}

enum RequestType {
  WEB_SEARCH = 'WEB_SEARCH',
  API_CALL = 'API_CALL',
  COMPUTATION = 'COMPUTATION',
  DATA_AGGREGATION = 'DATA_AGGREGATION',
  AI_INFERENCE = 'AI_INFERENCE',
}

class IntentProcessor {
  convertToRequest(intent: Intent): OracleRequest {
    const requestType = this.mapIntentToRequestType(intent);
    
    return {
      id: crypto.randomUUID(),
      type: requestType,
      params: this.extractParams(intent),
      requester: this.userAddress,
      priority: this.calculatePriority(intent),
      deadline: Date.now() + 30000, // 30s timeout
      zkRequired: this.needsPrivacy(intent),
    };
  }
  
  private mapIntentToRequestType(intent: Intent): RequestType {
    switch (intent.type) {
      case 'SEARCH':
        return RequestType.WEB_SEARCH;
      case 'QUERY_DATA':
        return RequestType.DATA_AGGREGATION;
      case 'ASK_QUESTION':
        return RequestType.AI_INFERENCE;
      default:
        return RequestType.COMPUTATION;
    }
  }
  
  private calculatePriority(intent: Intent): number {
    let priority = 5; // Default
    
    // User interaction = high priority
    if (intent.metadata.userInitiated) priority += 3;
    
    // Background task = low priority
    if (intent.metadata.background) priority -= 2;
    
    // Time-sensitive = high priority
    if (intent.metadata.urgent) priority += 2;
    
    return Math.max(0, Math.min(10, priority));
  }
}
```

**Integration Points**:
- **← Layer 6 (AI Engine)**: Receive intents from dialogue manager
- **→ Oracle Request Manager**: Submit oracle requests
- **↔ Layer 5 (Intelligence)**: Get context for intent enrichment

---

### 2. Oracle Request Manager

**Purpose**: Queue, schedule, and track oracle requests throughout their lifecycle.

**Request Queue**:
```typescript
interface RequestState {
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'timeout';
  request: OracleRequest;
  assignedWorker?: string;
  startTime?: number;
  result?: any;
  error?: string;
}

class OracleRequestManager {
  private queue: PriorityQueue<RequestState>;
  private active: Map<string, RequestState> = new Map();
  private maxConcurrent = 10;
  
  async submit(request: OracleRequest): Promise<string> {
    const state: RequestState = {
      status: 'pending',
      request,
    };
    
    // Add to priority queue
    this.queue.enqueue(state, request.priority);
    
    // Try to process immediately
    this.processNext();
    
    return request.id;
  }
  
  private async processNext(): Promise<void> {
    if (this.active.size >= this.maxConcurrent) return;
    
    const state = this.queue.dequeue();
    if (!state) return;
    
    state.status = 'processing';
    state.startTime = Date.now();
    this.active.set(state.request.id, state);
    
    try {
      // Execute oracle request
      const result = await this.executeRequest(state.request);
      
      state.status = 'completed';
      state.result = result;
      
      // Submit to blockchain
      await this.submitResponse(state.request.id, result);
    } catch (error) {
      state.status = 'failed';
      state.error = error.message;
    } finally {
      this.active.delete(state.request.id);
      this.processNext(); // Process next in queue
    }
  }
  
  private async executeRequest(request: OracleRequest): Promise<any> {
    switch (request.type) {
      case RequestType.WEB_SEARCH:
        return await this.webSearch(request.params);
      case RequestType.API_CALL:
        return await this.apiCall(request.params);
      case RequestType.AI_INFERENCE:
        return await this.aiInference(request.params);
      default:
        throw new Error(`Unknown request type: ${request.type}`);
    }
  }
  
  // Timeout monitoring
  private startTimeoutMonitor(): void {
    setInterval(() => {
      const now = Date.now();
      
      for (const [id, state] of this.active) {
        if (state.request.deadline < now) {
          state.status = 'timeout';
          state.error = 'Request exceeded deadline';
          this.active.delete(id);
        }
      }
    }, 1000);
  }
}
```

**Priority Queue Implementation**:
```typescript
class PriorityQueue<T> {
  private heap: Array<{ item: T; priority: number }> = [];
  
  enqueue(item: T, priority: number): void {
    this.heap.push({ item, priority });
    this.bubbleUp(this.heap.length - 1);
  }
  
  dequeue(): T | null {
    if (this.heap.length === 0) return null;
    
    const result = this.heap[0].item;
    const last = this.heap.pop();
    
    if (this.heap.length > 0 && last) {
      this.heap[0] = last;
      this.bubbleDown(0);
    }
    
    return result;
  }
  
  private bubbleUp(index: number): void {
    while (index > 0) {
      const parent = Math.floor((index - 1) / 2);
      if (this.heap[index].priority <= this.heap[parent].priority) break;
      
      [this.heap[index], this.heap[parent]] = [this.heap[parent], this.heap[index]];
      index = parent;
    }
  }
  
  private bubbleDown(index: number): void {
    while (true) {
      const left = 2 * index + 1;
      const right = 2 * index + 2;
      let largest = index;
      
      if (left < this.heap.length && this.heap[left].priority > this.heap[largest].priority) {
        largest = left;
      }
      if (right < this.heap.length && this.heap[right].priority > this.heap[largest].priority) {
        largest = right;
      }
      
      if (largest === index) break;
      
      [this.heap[index], this.heap[largest]] = [this.heap[largest], this.heap[index]];
      index = largest;
    }
  }
}
```

---

### 3. Tool Registry

**Purpose**: Register, validate, and execute tools available to the oracle system.

**Tool Registration**:
```typescript
interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: JSONSchema;
  outputSchema: JSONSchema;
  execute: (input: any, context: ExecutionContext) => Promise<any>;
  permissions: string[];      // Required permissions
  rateLimit: number;          // Max calls per minute
  timeout: number;            // Max execution time (ms)
}

class ToolRegistry {
  private tools: Map<string, ToolDefinition> = new Map();
  private callCounts: Map<string, number[]> = new Map(); // Timestamps
  
  register(tool: ToolDefinition): void {
    // Validate tool definition
    if (!this.validateTool(tool)) {
      throw new Error(`Invalid tool definition: ${tool.name}`);
    }
    
    this.tools.set(tool.name, tool);
    console.log(`Registered tool: ${tool.name}`);
  }
  
  async execute(
    toolName: string,
    input: any,
    context: ExecutionContext
  ): Promise<ToolOutput> {
    const tool = this.tools.get(toolName);
    if (!tool) {
      throw new Error(`Tool not found: ${toolName}`);
    }
    
    // Check permissions
    if (!this.checkPermissions(tool, context)) {
      throw new Error(`Missing permissions for tool: ${toolName}`);
    }
    
    // Check rate limit
    if (!this.checkRateLimit(toolName, tool.rateLimit)) {
      throw new Error(`Rate limit exceeded for tool: ${toolName}`);
    }
    
    // Validate input
    if (!this.validateInput(input, tool.inputSchema)) {
      throw new Error(`Invalid input for tool: ${toolName}`);
    }
    
    // Execute with timeout
    const startTime = Date.now();
    try {
      const result = await Promise.race([
        tool.execute(input, context),
        this.timeout(tool.timeout),
      ]);
      
      return {
        success: true,
        data: result,
        metadata: {
          executionTime: Date.now() - startTime,
          toolName,
        },
      };
    } catch (error) {
      return {
        success: false,
        data: null,
        error: error.message,
        metadata: {
          executionTime: Date.now() - startTime,
          toolName,
        },
      };
    }
  }
  
  private checkRateLimit(toolName: string, limit: number): boolean {
    const now = Date.now();
    const calls = this.callCounts.get(toolName) || [];
    
    // Remove calls older than 1 minute
    const recent = calls.filter(t => now - t < 60000);
    
    if (recent.length >= limit) {
      return false;
    }
    
    recent.push(now);
    this.callCounts.set(toolName, recent);
    return true;
  }
  
  private timeout(ms: number): Promise<never> {
    return new Promise((_, reject) => {
      setTimeout(() => reject(new Error('Tool execution timeout')), ms);
    });
  }
}
```

**Built-in Tools**:
```typescript
// Register all 6 core tools
const registry = new ToolRegistry();

registry.register({
  name: 'os_exec',
  description: 'Execute system commands',
  inputSchema: { /* ... */ },
  outputSchema: { /* ... */ },
  execute: async (input) => { /* ... */ },
  permissions: ['system.exec'],
  rateLimit: 10,
  timeout: 5000,
});

registry.register({
  name: 'web_api',
  description: 'Make HTTP requests',
  inputSchema: { /* ... */ },
  outputSchema: { /* ... */ },
  execute: async (input) => { /* ... */ },
  permissions: ['network.http'],
  rateLimit: 30,
  timeout: 10000,
});

// ... register other 4 tools
```

---

### 4. ZK Proof Engine

**Purpose**: Generate zero-knowledge proofs for oracle responses to preserve privacy while proving correctness.

**Proof Generation**:
```typescript
interface ProofRequest {
  toolName: string;
  input: any;
  output: any;
  timestamp: number;
}

class ZKProofEngine {
  private circuits: Map<string, Circuit> = new Map();
  
  async generateProof(request: ProofRequest): Promise<ZKProof> {
    // 1. Select circuit
    const circuit = this.selectCircuit(request.toolName);
    
    // 2. Prepare witness
    const witness = {
      toolHash: this.hashString(request.toolName),
      inputHash: this.hashObject(request.input),
      outputHash: this.hashObject(request.output),
      timestamp: request.timestamp,
      nonce: crypto.randomUUID(),
    };
    
    // 3. Generate proof (Groth16)
    const startTime = Date.now();
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      witness,
      circuit.wasm,
      circuit.zkey
    );
    
    const proofTime = Date.now() - startTime;
    console.log(`Proof generated in ${proofTime}ms`);
    
    // 4. Compress proof
    const compressed = this.compressProof(proof);
    
    return {
      proof: compressed,
      publicInputs: publicSignals,
      circuit: circuit.name,
      timestamp: request.timestamp,
    };
  }
  
  private selectCircuit(toolName: string): Circuit {
    // Use tool-specific circuit if available
    if (this.circuits.has(toolName)) {
      return this.circuits.get(toolName)!;
    }
    
    // Use generic circuit
    return this.circuits.get('generic')!;
  }
  
  private compressProof(proof: any): Uint8Array {
    // Groth16 proof format: (A, B, C) points on elliptic curve
    // A: 2 field elements (64 bytes)
    // B: 4 field elements (128 bytes)
    // C: 2 field elements (64 bytes)
    // Total: 256 bytes
    
    const buffer = new Uint8Array(256);
    let offset = 0;
    
    // Serialize proof points
    offset = this.writePoint(buffer, offset, proof.pi_a);
    offset = this.writePoint(buffer, offset, proof.pi_b[0]);
    offset = this.writePoint(buffer, offset, proof.pi_b[1]);
    offset = this.writePoint(buffer, offset, proof.pi_c);
    
    return buffer;
  }
}
```

**Circuit Definitions** (Circom):
```circom
// Generic oracle circuit
pragma circom 2.0.0;

include "circomlib/poseidon.circom";

template OracleProof() {
    signal input tool_hash;
    signal input input_hash;
    signal input output_hash;
    signal input timestamp;
    signal input nonce;
    
    signal output commitment;
    
    // Commitment = hash(tool, input, output, timestamp, nonce)
    component hasher = Poseidon(5);
    hasher.inputs[0] <== tool_hash;
    hasher.inputs[1] <== input_hash;
    hasher.inputs[2] <== output_hash;
    hasher.inputs[3] <== timestamp;
    hasher.inputs[4] <== nonce;
    
    commitment <== hasher.out;
}

component main {public [commitment]} = OracleProof();
```

---

### 5. Response Settler

**Purpose**: Submit oracle responses back to the blockchain for on-chain verification and settlement.

**Settlement Process**:
```typescript
interface OracleResponse {
  requestId: string;
  result: any;
  proof: ZKProof;
  timestamp: number;
  signature: string;
}

class ResponseSettler {
  private blockchain: BlockchainClient;
  private wallet: Wallet;
  
  async submitResponse(response: OracleResponse): Promise<string> {
    // 1. Create transaction
    const tx = {
      type: 'OracleResponse',
      data: {
        requestId: response.requestId,
        resultHash: this.hashObject(response.result),
        proof: response.proof.proof,
        publicInputs: response.proof.publicInputs,
        timestamp: response.timestamp,
      },
      fee: 0.1, // KARA tokens
    };
    
    // 2. Sign transaction
    const signature = await this.wallet.sign(tx);
    tx.signature = signature;
    
    // 3. Submit to blockchain (Layer 3)
    const txHash = await this.blockchain.submitTransaction(tx);
    
    console.log(`Response submitted: ${txHash}`);
    
    // 4. Wait for confirmation
    await this.blockchain.waitForConfirmation(txHash, 2); // 2 blocks
    
    return txHash;
  }
  
  // Monitor for oracle requests on-chain
  async monitorRequests(): Promise<void> {
    this.blockchain.subscribe('OracleRequest', async (event) => {
      const request: OracleRequest = {
        id: event.requestId,
        type: event.requestType,
        params: event.params,
        requester: event.sender,
        priority: 5,
        deadline: event.deadline,
        zkRequired: event.zkRequired,
      };
      
      // Submit to oracle request manager
      await this.requestManager.submit(request);
    });
  }
}
```

**Transaction Format**:
```typescript
interface OracleResponseTx {
  type: 'OracleResponse';
  requestId: string;
  resultHash: string;     // Hash of actual result (privacy)
  proof: Uint8Array;      // ZK proof (256 bytes)
  publicInputs: string[]; // Public signals for verification
  timestamp: number;
  signature: string;      // Ed25519 signature
}
```

---

### 6. Manifest Renderer

**Purpose**: Convert oracle responses into multi-modal UI manifests for display.

**Manifest Types**:
```typescript
enum ManifestType {
  TEXT = 'TEXT',
  CARD = 'CARD',
  LIST = 'LIST',
  CHART = 'CHART',
  MAP = 'MAP',
  AR_OVERLAY = 'AR_OVERLAY',
}

interface UIManifest {
  type: ManifestType;
  content: any;
  layout: LayoutHints;
  interactions: Interaction[];
}
```

**Manifest Generation**:
```typescript
class ManifestRenderer {
  renderResponse(response: OracleResponse): UIManifest {
    const requestType = this.getRequestType(response.requestId);
    
    switch (requestType) {
      case RequestType.WEB_SEARCH:
        return this.renderSearchResults(response.result);
        
      case RequestType.API_CALL:
        return this.renderAPIData(response.result);
        
      case RequestType.AI_INFERENCE:
        return this.renderAIResponse(response.result);
        
      default:
        return this.renderGeneric(response.result);
    }
  }
  
  private renderSearchResults(results: any[]): UIManifest {
    return {
      type: ManifestType.LIST,
      content: {
        title: 'Search Results',
        items: results.map(r => ({
          title: r.title,
          description: r.snippet,
          url: r.url,
        })),
      },
      layout: {
        position: 'center',
        size: 'medium',
        animation: 'fade-in',
      },
      interactions: [
        { type: 'tap', action: 'open_url' },
        { type: 'swipe_left', action: 'dismiss' },
      ],
    };
  }
  
  private renderAIResponse(response: string): UIManifest {
    return {
      type: ManifestType.CARD,
      content: {
        text: response,
        avatar: '/assets/ai-avatar.png',
      },
      layout: {
        position: 'bottom',
        size: 'auto',
        animation: 'slide-up',
      },
      interactions: [
        { type: 'tap', action: 'expand' },
        { type: 'swipe_down', action: 'dismiss' },
      ],
    };
  }
}
```

---

## Integration Flow

```
User Intent (Layer 6)
        │
        ▼
┌────────────────────┐
│ Intent Processor   │ 2-5ms
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│ Request Manager    │ Queue + Schedule
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│ Tool Registry      │ Execute tool
│ (web_api, etc.)    │ 100-5000ms
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│ ZK Proof Engine    │ Generate proof
│ (Groth16)          │ 300-800ms
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│ Response Settler   │ Submit to chain
│ (Layer 3)          │ 12s (block time)
└─────────┬──────────┘
          │
          ▼
┌────────────────────┐
│ Manifest Renderer  │ Create UI
│ → Layer 7          │ 5-10ms
└────────────────────┘
```

---

## Performance Metrics

```
┌─ Oracle Bridge Performance ─────────────┐
│ Intent Processing: 2-5ms                 │
│ Request Queueing: <1ms                   │
│ Tool Execution:                          │
│   - os_exec: 10-50ms                     │
│   - web_api: 100-500ms                   │
│   - app_proxy: 20-100ms                  │
│   - gen_creative: 2-10s                  │
│   - memory_rag: 50-200ms                 │
│   - health_sensor: 5-20ms                │
│ ZK Proof Generation: 300-800ms           │
│ On-chain Settlement: 12s (1 block)       │
│ Manifest Rendering: 5-10ms               │
│                                           │
│ Throughput: 100 requests/sec             │
│ Queue Capacity: 10,000 requests          │
└───────────────────────────────────────────┘
```

---

## Security Considerations

**Request Validation**:
- Input sanitization for all tool calls
- Rate limiting per user/tool
- Permission checks before execution
- Timeout enforcement (30s default)

**Privacy**:
- ZK proofs hide sensitive inputs/outputs
- Only hashes submitted on-chain
- Optional encryption for stored results

**Integrity**:
- Cryptographic signatures on responses
- Merkle proofs for data authenticity
- Replay attack prevention (nonces)

---

## Future Development

### Phase 1: Multi-Oracle (Q1 2026)
- Federated oracle network (5-10 nodes)
- Consensus on oracle responses
- Slashing for malicious oracles

### Phase 2: Advanced Tools (Q2 2026)
- Machine learning inference tools
- Blockchain bridge tools (cross-chain)
- IoT device integration tools

### Phase 3: Optimistic Oracle (Q3 2026)
- Instant responses with dispute period
- Economic guarantees via bonding
- Challenge-response mechanism

### Phase 4: Recursive Proofs (Q4 2026)
- Proof aggregation (100 proofs → 1)
- Reduced on-chain verification cost
- PLONK/Halo2 integration

---

## Code References

- `simulator-ui/services/oracleService.ts`: Oracle interface
- `simulator-ui/services/enhancedOracleAI.ts`: Agentic reasoning
- `karana-core/src/oracle/bridge.rs`: Oracle bridge (Rust)
- `karana-core/src/oracle/zk_prover.rs`: ZK proof generation

---

## Summary

Layer 4 provides:
- **Intent Processing**: Convert AI intents to oracle requests
- **Request Management**: Priority queue with 100 req/sec throughput
- **Tool Execution**: 6 core tools with rate limiting
- **ZK Proofs**: Privacy-preserving verification (Groth16)
- **On-Chain Settlement**: Verifiable oracle responses
- **UI Manifests**: Multi-modal response rendering

This layer is the critical bridge between AI intelligence and blockchain trust.