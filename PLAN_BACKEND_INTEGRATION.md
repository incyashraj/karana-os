# PLAN_BACKEND_INTEGRATION.md
# Monad Rewiring for Oracle-Centric Architecture

## Overview

This plan transforms the Monad from a multi-access backend to an **Oracle-exclusive** system where:
- **ONLY the Oracle** can issue commands to atoms (Storage, Runtime, Swarm, Chain)
- All user intents flow through Oracle → ZK-Sign → Monad execute
- Direct API access is **removed** (no HTTP handlers calling atoms directly)
- Backend becomes invisible to user—they only see Oracle whispers

---

## Current State Analysis

### Current `monad.rs` Architecture
```rust
pub struct KaranaMonad {
    boot: KaranaBoot,
    runtime: Arc<RuntimeActor>,
    ui: Arc<KaranaUI>,                    // ← Direct UI access
    vigil: Arc<KaranaVeil>,
    storage: Arc<KaranaStorage>,          // ← Direct storage access
    swarm: Arc<KaranaSwarm>,              // ← Direct swarm access
    ai: Arc<Mutex<KaranaAI>>,
    ledger: Arc<Mutex<Ledger>>,
    chain: Arc<Blockchain>,               // ← Direct chain access
    oracle: Arc<Mutex<KaranaOracle>>,     // ← Oracle exists but doesn't control
    wallet: Arc<Mutex<KaranaWallet>>,
    // ...
}
```

**Problems:**
1. UI/API can directly call `storage.write()`, `swarm.broadcast()`, `chain.apply_block()`
2. Oracle is just another component, not the sole gateway
3. No enforcement that intents are ZK-proven before execution
4. Monad event loop handles raw intents, not Oracle-mediated commands

---

## Target Architecture

### Oracle-Commanded Monad
```rust
pub struct KaranaMonad {
    // ═══ ORACLE LAYER (User-Facing) ═══
    oracle: Arc<OracleVeil>,              // Sole interface
    
    // ═══ BACKEND ATOMS (Oracle-Only Access) ═══
    atoms: BackendAtoms,                   // Private struct
    
    // ═══ COMMAND CHANNEL ═══
    cmd_rx: mpsc::Receiver<OracleCommand>, // Oracle sends commands here
    result_tx: mpsc::Sender<CommandResult>, // Results back to Oracle
}

struct BackendAtoms {
    boot: KaranaBoot,
    runtime: Arc<RuntimeActor>,
    storage: Arc<KaranaStorage>,
    swarm: Arc<KaranaSwarm>,
    chain: Arc<Blockchain>,
    ledger: Arc<Mutex<Ledger>>,
    wallet: Arc<Mutex<KaranaWallet>>,
    vigil: Arc<KaranaVeil>,
}
```

---

## Command Channel Design

### `src/oracle/command.rs` (NEW)

```rust
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

/// Commands that ONLY the Oracle can send to the Monad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleCommand {
    // ═══ STORAGE COMMANDS ═══
    StoreData {
        data: Vec<u8>,
        metadata: String,
        zk_proof: Vec<u8>,        // REQUIRED: Proof of intent
    },
    RetrieveData {
        key: Vec<u8>,
        requester_did: String,
        zk_proof: Vec<u8>,
    },
    SearchSemantic {
        query: String,
        limit: usize,
    },
    
    // ═══ CHAIN COMMANDS ═══
    SubmitTransaction {
        tx_data: TransactionPayload,
        zk_proof: Vec<u8>,
    },
    QueryBalance {
        did: String,
    },
    QueryChainState {
        query_type: ChainQuery,
    },
    
    // ═══ SWARM COMMANDS ═══
    BroadcastMessage {
        topic: String,
        payload: Vec<u8>,
        zk_proof: Vec<u8>,
    },
    DialPeer {
        multiaddr: String,
    },
    
    // ═══ RUNTIME COMMANDS ═══
    ExecuteWasm {
        module_hash: Vec<u8>,
        params: Vec<u8>,
        gas_limit: u64,
    },
    ScheduleTask {
        task_id: String,
        delay_ms: u64,
        command: Box<OracleCommand>,
    },
    
    // ═══ SYSTEM COMMANDS ═══
    GetPipelineStatus,
    TriggerZKBatch,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionPayload {
    Transfer { to: String, amount: u128 },
    Stake { amount: u128 },
    Unstake { amount: u128 },
    Vote { proposal_id: u64, approve: bool },
    StoreAttestation { data_hash: Vec<u8>, proof: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainQuery {
    LatestBlock,
    BlockByHeight(u64),
    TransactionByHash(String),
    ProposalStatus(u64),
    NodeInfo,
}

/// Result of executing an OracleCommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    Success {
        command_id: String,
        data: CommandData,
    },
    Failure {
        command_id: String,
        error: String,
        recoverable: bool,
    },
    Pending {
        command_id: String,
        estimated_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandData {
    // Storage results
    StoredHash(Vec<u8>),
    RetrievedData(Vec<u8>),
    SearchResults(Vec<SearchHit>),
    
    // Chain results
    TxHash(String),
    Balance(u128),
    BlockData(BlockSummary),
    
    // Swarm results
    MessageId(String),
    PeerConnected(String),
    
    // Runtime results
    WasmOutput(Vec<u8>),
    TaskScheduled(String),
    
    // System results
    PipelineStatus(PipelineStatus),
    BatchProofs(Vec<ProofSummary>),
    ShutdownAck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub key: Vec<u8>,
    pub score: f32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub tx_count: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub ai_model: String,
    pub zk_queue_size: usize,
    pub swarm_peers: usize,
    pub chain_height: u64,
    pub mempool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    pub proof_type: String,
    pub size_bytes: usize,
    pub generation_ms: u64,
}

/// Channel handles for Oracle ↔ Monad communication
pub struct OracleChannels {
    pub cmd_tx: mpsc::Sender<OracleCommand>,
    pub result_rx: mpsc::Receiver<CommandResult>,
}

impl OracleChannels {
    pub fn new(buffer_size: usize) -> (Self, MonadChannels) {
        let (cmd_tx, cmd_rx) = mpsc::channel(buffer_size);
        let (result_tx, result_rx) = mpsc::channel(buffer_size);
        
        (
            OracleChannels { cmd_tx, result_rx },
            MonadChannels { cmd_rx, result_tx },
        )
    }
}

pub struct MonadChannels {
    pub cmd_rx: mpsc::Receiver<OracleCommand>,
    pub result_tx: mpsc::Sender<CommandResult>,
}
```

---

## Monad Rewiring Implementation

### Phase 1: Encapsulate Backend Atoms

**File: `src/monad.rs` (Modified)**

```rust
use crate::oracle::command::{OracleCommand, CommandResult, CommandData, MonadChannels};
use crate::oracle::veil::OracleVeil;
use tokio::sync::mpsc;

/// Backend atoms - ONLY accessible by Monad internally
struct BackendAtoms {
    boot: KaranaBoot,
    runtime: Arc<RuntimeActor>,
    storage: Arc<KaranaStorage>,
    swarm: Arc<KaranaSwarm>,
    chain: Arc<Blockchain>,
    ledger: Arc<Mutex<Ledger>>,
    gov: Arc<Mutex<Governance>>,
    wallet: Arc<Mutex<KaranaWallet>>,
    vigil: Arc<KaranaVeil>,
    hardware: Arc<KaranaHardware>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
    zk_engine: Arc<ZKEngine>,
}

/// The Monad: Now exclusively commanded by Oracle
pub struct KaranaMonad {
    // ═══ ORACLE (Public Interface) ═══
    oracle: Arc<OracleVeil>,
    
    // ═══ BACKEND (Private) ═══
    atoms: BackendAtoms,
    
    // ═══ COMMAND CHANNEL ═══
    channels: MonadChannels,
}

impl KaranaMonad {
    pub async fn new(config: KaranaConfig) -> Result<(Self, OracleChannels)> {
        // ... existing atom initialization ...
        
        // Create command channels
        let (oracle_channels, monad_channels) = OracleChannels::new(256);
        
        // Create Oracle with its channel handle
        let oracle = Arc::new(OracleVeil::new(
            ai.clone(),
            oracle_channels.cmd_tx.clone(),
            oracle_channels.result_rx,
        )?);
        
        // Encapsulate atoms
        let atoms = BackendAtoms {
            boot,
            runtime,
            storage,
            swarm,
            chain,
            ledger,
            gov,
            wallet,
            vigil,
            hardware,
            mempool,
            zk_engine: Arc::new(ZKEngine::new()?),
        };
        
        Ok((
            Self {
                oracle,
                atoms,
                channels: monad_channels,
            },
            oracle_channels,
        ))
    }
    
    /// Main loop: Process ONLY Oracle commands
    pub async fn run(&mut self) -> Result<()> {
        log::info!("=== MONAD: Oracle-Commanded Mode ===");
        
        let mut block_timer = tokio::time::interval(Duration::from_secs(5));
        let mut height = 1u64;
        let mut parent_hash = "0".repeat(64);
        
        loop {
            tokio::select! {
                // ═══ ORACLE COMMAND ═══
                Some(cmd) = self.channels.cmd_rx.recv() => {
                    let result = self.execute_command(cmd).await;
                    if let Err(e) = self.channels.result_tx.send(result).await {
                        log::error!("[MONAD] Failed to send result: {}", e);
                    }
                }
                
                // ═══ SWARM EVENTS ═══
                event = self.poll_swarm_event() => {
                    if let Some(e) = event {
                        // Route swarm events TO Oracle for processing
                        self.oracle.handle_swarm_event(e).await;
                    }
                }
                
                // ═══ BLOCK PRODUCTION ═══
                _ = block_timer.tick() => {
                    if let Ok(block) = self.produce_block(&mut height, &mut parent_hash).await {
                        log::info!("[CHAIN] Block #{} produced", block.header.height);
                    }
                }
            }
        }
    }
    
    /// Execute a command from the Oracle
    async fn execute_command(&self, cmd: OracleCommand) -> CommandResult {
        let cmd_id = uuid::Uuid::new_v4().to_string();
        
        match cmd {
            // ═══ STORAGE ═══
            OracleCommand::StoreData { data, metadata, zk_proof } => {
                // Verify ZK proof FIRST
                if !self.verify_intent_proof(&zk_proof) {
                    return CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Invalid ZK proof for storage intent".into(),
                        recoverable: false,
                    };
                }
                
                match self.atoms.storage.write(&data, &metadata) {
                    Ok(block) => CommandResult::Success {
                        command_id: cmd_id,
                        data: CommandData::StoredHash(block.merkle_root.clone()),
                    },
                    Err(e) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: e.to_string(),
                        recoverable: true,
                    },
                }
            }
            
            OracleCommand::RetrieveData { key, requester_did, zk_proof } => {
                // Verify requester has access
                if !self.verify_access_proof(&requester_did, &key, &zk_proof) {
                    return CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Access denied".into(),
                        recoverable: false,
                    };
                }
                
                match self.atoms.storage.read_chunk(&key) {
                    Ok(Some(data)) => CommandResult::Success {
                        command_id: cmd_id,
                        data: CommandData::RetrievedData(data),
                    },
                    Ok(None) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Data not found".into(),
                        recoverable: false,
                    },
                    Err(e) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: e.to_string(),
                        recoverable: true,
                    },
                }
            }
            
            OracleCommand::SearchSemantic { query, limit } => {
                match self.atoms.storage.search(&query) {
                    Ok(results) => {
                        let hits: Vec<SearchHit> = results.into_iter()
                            .take(limit)
                            .map(|(key, score, preview)| SearchHit { key, score, preview })
                            .collect();
                        CommandResult::Success {
                            command_id: cmd_id,
                            data: CommandData::SearchResults(hits),
                        }
                    }
                    Err(e) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: e.to_string(),
                        recoverable: true,
                    },
                }
            }
            
            // ═══ CHAIN ═══
            OracleCommand::SubmitTransaction { tx_data, zk_proof } => {
                if !self.verify_intent_proof(&zk_proof) {
                    return CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Invalid ZK proof for transaction".into(),
                        recoverable: false,
                    };
                }
                
                let tx = self.create_transaction(tx_data);
                let tx_hash = tx.hash.clone();
                
                self.atoms.mempool.lock().unwrap().push(tx);
                
                CommandResult::Success {
                    command_id: cmd_id,
                    data: CommandData::TxHash(tx_hash),
                }
            }
            
            OracleCommand::QueryBalance { did } => {
                let balance = self.atoms.ledger.lock().unwrap().get_balance(&did);
                CommandResult::Success {
                    command_id: cmd_id,
                    data: CommandData::Balance(balance),
                }
            }
            
            OracleCommand::QueryChainState { query_type } => {
                match query_type {
                    ChainQuery::LatestBlock => {
                        let block = self.atoms.chain.latest_block();
                        CommandResult::Success {
                            command_id: cmd_id,
                            data: CommandData::BlockData(BlockSummary {
                                height: block.header.height,
                                hash: block.hash.clone(),
                                tx_count: block.transactions.len(),
                                timestamp: block.header.timestamp,
                            }),
                        }
                    }
                    // ... other query types
                    _ => CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Query type not implemented".into(),
                        recoverable: false,
                    },
                }
            }
            
            // ═══ SWARM ═══
            OracleCommand::BroadcastMessage { topic, payload, zk_proof } => {
                if !self.verify_intent_proof(&zk_proof) {
                    return CommandResult::Failure {
                        command_id: cmd_id,
                        error: "Invalid ZK proof for broadcast".into(),
                        recoverable: false,
                    };
                }
                
                match self.atoms.swarm.broadcast_to_topic(&topic, payload).await {
                    Ok(msg_id) => CommandResult::Success {
                        command_id: cmd_id,
                        data: CommandData::MessageId(msg_id),
                    },
                    Err(e) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: e.to_string(),
                        recoverable: true,
                    },
                }
            }
            
            // ═══ SYSTEM ═══
            OracleCommand::GetPipelineStatus => {
                let (zk_queued, _) = crate::zk::get_batch_status();
                CommandResult::Success {
                    command_id: cmd_id,
                    data: CommandData::PipelineStatus(PipelineStatus {
                        ai_model: "Phi-3 q4".into(),
                        zk_queue_size: zk_queued,
                        swarm_peers: self.atoms.swarm.peer_count(),
                        chain_height: self.atoms.chain.height(),
                        mempool_size: self.atoms.mempool.lock().unwrap().len(),
                    }),
                }
            }
            
            OracleCommand::TriggerZKBatch => {
                match crate::zk::prove_batch() {
                    Ok(proofs) => {
                        let summaries: Vec<ProofSummary> = proofs.iter()
                            .map(|p| ProofSummary {
                                proof_type: "groth16".into(),
                                size_bytes: p.len(),
                                generation_ms: 0, // TODO: track timing
                            })
                            .collect();
                        CommandResult::Success {
                            command_id: cmd_id,
                            data: CommandData::BatchProofs(summaries),
                        }
                    }
                    Err(e) => CommandResult::Failure {
                        command_id: cmd_id,
                        error: e.to_string(),
                        recoverable: true,
                    },
                }
            }
            
            OracleCommand::Shutdown => {
                log::info!("[MONAD] Shutdown requested by Oracle");
                CommandResult::Success {
                    command_id: cmd_id,
                    data: CommandData::ShutdownAck,
                }
            }
            
            _ => CommandResult::Failure {
                command_id: cmd_id,
                error: "Command not implemented".into(),
                recoverable: false,
            },
        }
    }
    
    fn verify_intent_proof(&self, proof: &[u8]) -> bool {
        if proof.is_empty() {
            log::warn!("[MONAD] Empty ZK proof - rejecting");
            return false;
        }
        // Verify Groth16 proof
        crate::zk::verify_proof(proof).unwrap_or(false)
    }
    
    fn verify_access_proof(&self, did: &str, key: &[u8], proof: &[u8]) -> bool {
        // Verify the requester's DID has access to this key
        // For now, accept if proof is valid
        self.verify_intent_proof(proof)
    }
}
```

---

## Phase 2: Remove Direct API Access

### `src/api/handlers.rs` (Modified)

**Before:**
```rust
// Direct atom access - WRONG
pub async fn handle_storage(req: StorageRequest, storage: Arc<KaranaStorage>) -> Response {
    storage.write(&req.data, &req.metadata)?; // Direct call!
}
```

**After:**
```rust
use crate::oracle::veil::OracleVeil;

/// ALL API requests go through Oracle
pub async fn handle_request(
    req: UserRequest,
    oracle: Arc<OracleVeil>,
) -> Response {
    // User request → Oracle mediation → ZK-signed command → Monad
    match oracle.mediate(req.intent, req.context).await {
        Ok(response) => Response::success(response.whisper),
        Err(e) => Response::error(e.to_string()),
    }
}

// Individual handlers become Oracle intent formatters
pub async fn handle_storage_api(
    Json(req): Json<StorageApiRequest>,
    State(oracle): State<Arc<OracleVeil>>,
) -> impl IntoResponse {
    let intent = format!("store data: {}", req.description);
    oracle.mediate(intent, OracleContext::api()).await
}

pub async fn handle_query_api(
    Json(req): Json<QueryApiRequest>,
    State(oracle): State<Arc<OracleVeil>>,
) -> impl IntoResponse {
    let intent = format!("query: {}", req.query);
    oracle.mediate(intent, OracleContext::api()).await
}
```

---

## Phase 3: Oracle-Monad Integration

### OracleVeil Command Sending

**In `src/oracle/veil.rs`:**

```rust
impl OracleVeil {
    /// Send command to Monad and await result
    async fn execute(&self, cmd: OracleCommand) -> Result<CommandData> {
        // Send command
        self.cmd_tx.send(cmd).await
            .map_err(|_| anyhow!("Monad channel closed"))?;
        
        // Await result with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            self.result_rx.recv()
        ).await
            .map_err(|_| anyhow!("Command timeout"))?
            .ok_or_else(|| anyhow!("No result received"))?;
        
        match result {
            CommandResult::Success { data, .. } => Ok(data),
            CommandResult::Failure { error, .. } => Err(anyhow!(error)),
            CommandResult::Pending { estimated_ms, .. } => {
                // Wait for completion
                tokio::time::sleep(Duration::from_millis(estimated_ms)).await;
                self.result_rx.recv().await
                    .ok_or_else(|| anyhow!("Pending result never arrived"))?
                    .into()
            }
        }
    }
    
    /// Mediate user intent to backend command
    pub async fn mediate(&self, intent: &str, ctx: OracleContext) -> Result<OracleResponse> {
        // 1. Parse intent
        let parsed = self.parse_intent(intent, &ctx).await?;
        
        // 2. Generate ZK proof of intent
        let zk_proof = self.prover.prove_intent(&parsed)?;
        
        // 3. Create backend command
        let cmd = self.intent_to_command(parsed, zk_proof)?;
        
        // 4. Execute via Monad
        let result = self.execute(cmd).await?;
        
        // 5. Format response as whisper
        let whisper = self.format_whisper(&result, &ctx);
        
        Ok(OracleResponse { whisper, data: Some(result) })
    }
    
    fn intent_to_command(&self, parsed: ParsedIntent, zk_proof: Vec<u8>) -> Result<OracleCommand> {
        match parsed.action {
            IntentAction::Store { data, metadata } => {
                Ok(OracleCommand::StoreData { data, metadata, zk_proof })
            }
            IntentAction::Retrieve { key, did } => {
                Ok(OracleCommand::RetrieveData { key, requester_did: did, zk_proof })
            }
            IntentAction::Transfer { to, amount } => {
                Ok(OracleCommand::SubmitTransaction {
                    tx_data: TransactionPayload::Transfer { to, amount },
                    zk_proof,
                })
            }
            IntentAction::QueryBalance { did } => {
                Ok(OracleCommand::QueryBalance { did })
            }
            IntentAction::Search { query, limit } => {
                Ok(OracleCommand::SearchSemantic { query, limit })
            }
            IntentAction::Status => {
                Ok(OracleCommand::GetPipelineStatus)
            }
            // ... other actions
        }
    }
}
```

---

## Phase 4: Event Loop Transformation

### Before (Current):
```rust
// In ignite() - processes raw intents
loop {
    if let Some(intent) = self.ui.poll_intent() {
        // Direct handling - WRONG
        if intent.starts_with("tune") {
            self.execute_real_action(&intent)?;  // Direct atom access
        }
    }
}
```

### After (Oracle-Commanded):
```rust
// In run() - processes only Oracle commands
loop {
    tokio::select! {
        // Oracle commands ONLY
        Some(cmd) = self.channels.cmd_rx.recv() => {
            let result = self.execute_command(cmd).await;
            self.channels.result_tx.send(result).await?;
        }
        
        // Swarm events go TO Oracle
        Some(event) = self.poll_swarm() => {
            self.oracle.handle_swarm_event(event).await;
        }
        
        // Block production (autonomous)
        _ = block_timer.tick() => {
            self.produce_block().await?;
        }
    }
}
```

---

## File Structure

```
karana-core/src/
├── monad.rs                    # MODIFIED: Oracle-commanded
├── oracle/
│   ├── mod.rs                  # MODIFIED: Re-export command
│   ├── veil.rs                 # NEW: OracleVeil struct
│   ├── command.rs              # NEW: OracleCommand enum
│   ├── sense.rs                # NEW: Multimodal input
│   └── manifest.rs             # NEW: Output formatting
├── api/
│   ├── mod.rs                  # MODIFIED: Oracle-only routes
│   └── handlers.rs             # MODIFIED: All through Oracle
```

---

## Implementation Phases

### Phase 1: Command Channel (Day 1-2)
- [ ] Create `src/oracle/command.rs` with `OracleCommand` enum
- [ ] Add `tokio::mpsc` channels to Monad
- [ ] Test basic command flow

### Phase 2: Encapsulate Atoms (Day 2-3)
- [ ] Create `BackendAtoms` struct
- [ ] Move atoms from public to private
- [ ] Update `KaranaMonad::new()`

### Phase 3: Command Executor (Day 3-4)
- [ ] Implement `execute_command()` for all commands
- [ ] Add ZK proof verification gate
- [ ] Test each command type

### Phase 4: Oracle Integration (Day 4-5)
- [ ] Wire OracleVeil to command channels
- [ ] Implement `intent_to_command()` mapping
- [ ] Test full mediation flow

### Phase 5: API Rewiring (Day 5-6)
- [ ] Modify handlers to use Oracle
- [ ] Remove direct atom access
- [ ] Test API → Oracle → Monad → Response

### Phase 6: Event Loop (Day 6-7)
- [ ] Replace `ignite()` with `run()`
- [ ] Remove UI intent polling
- [ ] Route swarm events through Oracle

---

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_oracle_command_flow() {
    let (monad, oracle_channels) = KaranaMonad::new(config).await?;
    
    // Send storage command
    oracle_channels.cmd_tx.send(OracleCommand::StoreData {
        data: b"test".to_vec(),
        metadata: "test".into(),
        zk_proof: generate_test_proof(),
    }).await?;
    
    // Receive result
    let result = oracle_channels.result_rx.recv().await?;
    assert!(matches!(result, CommandResult::Success { .. }));
}

#[tokio::test]
async fn test_zk_proof_required() {
    // Command without proof should fail
    let result = monad.execute_command(OracleCommand::StoreData {
        data: b"test".to_vec(),
        metadata: "test".into(),
        zk_proof: vec![],  // Empty proof
    }).await;
    
    assert!(matches!(result, CommandResult::Failure { .. }));
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_mediation_flow() {
    let oracle = setup_oracle().await;
    
    // User intent → Oracle → ZK → Monad → Response
    let response = oracle.mediate(
        "store my grocery list: milk, eggs, bread",
        OracleContext::voice(),
    ).await?;
    
    assert!(response.whisper.contains("stored"));
}
```

---

## Migration Checklist

- [ ] Create `oracle/command.rs`
- [ ] Add `MonadChannels` to `KaranaMonad`
- [ ] Create `BackendAtoms` struct
- [ ] Implement `execute_command()` for:
  - [ ] `StoreData`
  - [ ] `RetrieveData`
  - [ ] `SearchSemantic`
  - [ ] `SubmitTransaction`
  - [ ] `QueryBalance`
  - [ ] `QueryChainState`
  - [ ] `BroadcastMessage`
  - [ ] `GetPipelineStatus`
  - [ ] `TriggerZKBatch`
  - [ ] `Shutdown`
- [ ] Wire OracleVeil command sending
- [ ] Update API handlers
- [ ] Replace `ignite()` with `run()`
- [ ] Remove direct atom exports
- [ ] Update tests

---

## Cargo.toml Additions

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4"] }
# Already have tokio with mpsc
```

---

*PLAN_BACKEND_INTEGRATION.md - December 3, 2025*
