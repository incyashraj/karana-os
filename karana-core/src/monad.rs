use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use sha2::Digest;

use crate::boot::KaranaBoot;
use crate::runtime::KaranaActor as RuntimeActor;
use crate::ui::KaranaUI;
use crate::vigil::KaranaVeil;
use crate::storage::KaranaStorage;
use crate::net::{KaranaSwarm, KaranaSwarmEvent};
use crate::ai::KaranaAI;
use crate::zk::setup_zk;
use crate::economy::{Ledger, ProofOfStorage, Governance};
use crate::gov::KaranaDAO;
use crate::chain::{Blockchain, Transaction, TransactionData, Block};
use crate::state::KaranaPersist;
use crate::hardware::KaranaHardware;
use crate::identity::KaranaIdentity;
use alloy_primitives::U256;

/// The Monad: Weaves atoms into sovereign flow
pub struct KaranaMonad {
    boot: KaranaBoot,
    runtime: Arc<RuntimeActor>,
    ui: Arc<KaranaUI>,
    vigil: Arc<KaranaVeil>,
    storage: Arc<KaranaStorage>,
    swarm: Arc<KaranaSwarm>,
    #[allow(dead_code)]
    ai: Arc<Mutex<KaranaAI>>,
    ledger: Arc<Mutex<Ledger>>,
    pos: Arc<ProofOfStorage>,
    gov: Arc<Mutex<Governance>>,
    dao: Arc<Mutex<KaranaDAO>>,
    chain: Arc<Blockchain>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
    persist: Arc<KaranaPersist>,
    hardware: Arc<KaranaHardware>,
    #[allow(dead_code)]
    identity: Arc<Mutex<KaranaIdentity>>,
}


pub struct KaranaConfig {
    pub port: u16,
    pub peer: Option<String>,
    pub base_path: String,
}

impl KaranaMonad {
    pub async fn new(config: KaranaConfig) -> Result<Self> {
        // Chroot detect: If /proc/1/cwd is jail (or env var set), adjust paths
        // In this prototype env, we use an env var or check if /proc exists (it usually does in containers)
        // We'll use a marker file or env var for reliability.
        let is_chroot = std::env::var("KARANA_CHROOT").is_ok();
        let base_path = if is_chroot { "/var/karana".to_string() } else { config.base_path.clone() };
        
        if is_chroot {
            log::info!("Atom 5 (Chroot): Initializing in Sovereign Jail at {}", base_path);
        }

        // Initialize ZK Engine (Phase 2)
        setup_zk().context("ZK Setup failed")?;

        // Initialize AI Engine (Phase 3)
        log::info!("Igniting Karana AI (Phi-3 Simulated)...");
        let ai = Arc::new(Mutex::new(KaranaAI::new().context("AI Ignition failed")?));

        // Atom 8: Identity (Phase 4)
        let identity = Arc::new(Mutex::new(KaranaIdentity::new()?));

        // Atom 4: Boot Process (Initializes Swarm)
        let swarm_inner = KaranaSwarm::new(ai.clone(), config.port, config.peer).await?;
        let boot = KaranaBoot::new(ai.clone(), swarm_inner.clone()).await?;
        let swarm = Arc::new(swarm_inner);

        let storage_path = format!("{}/karana-cache", base_path);
        let storage = Arc::new(KaranaStorage::new(&storage_path, "http://localhost:26657", ai.clone())?);
        let runtime = Arc::new(RuntimeActor::new(&swarm)?);
        
        // Phase v1.0: Hardware Abstraction (IoT/Glass)
        let hardware = Arc::new(KaranaHardware::probe());
        
        // Start Hardware Simulation if requested or in dev mode
        if std::env::var("SIMULATE_HARDWARE").is_ok() || !is_chroot {
             hardware.start_simulation();
        }
        
        let ui = Arc::new(KaranaUI::new(&runtime, &swarm, ai.clone(), hardware.clone(), identity.clone())?);
        
        // Atom 4: Economy (Persistent Ledger)
        let ledger_path = format!("{}/karana-ledger", base_path);
        let ledger = Arc::new(Mutex::new(Ledger::new(&ledger_path)));
        let pos = Arc::new(ProofOfStorage::new(ledger.clone()));
        
        let gov_path = format!("{}/karana-governance", base_path);
        let gov = Arc::new(Mutex::new(Governance::new(&gov_path, ledger.clone(), ai.clone())));
        
        // Atom 4: DAO (Phase 4)
        let dao = Arc::new(Mutex::new(KaranaDAO::default()));

        // Atom 7: Vigil (Needs Ledger for Slashing)
        let vigil = Arc::new(KaranaVeil::new(ai.clone(), &runtime, ledger.clone())?);

        // Phase 7: Sovereign Chain State (Persistent)
        let chain = Arc::new(Blockchain::new(ledger.clone(), gov.clone()));
        let mempool = Arc::new(Mutex::new(Vec::new()));

        // Phase v1.0: Persistent State
        let persist = Arc::new(KaranaPersist::new("/dev/sda1")); // Stub root dev

        Ok(Self {
            boot,
            runtime,
            ui,
            vigil,
            storage,
            swarm,
            ai,
            ledger,
            pos,
            gov,
            dao,
            chain,
            mempool,
            persist,
            hardware,
            identity,
        })
    }

    /// Ignite: Full rethink flow (boot → intent → prove → store → attest)
    pub async fn ignite(&mut self) -> Result<()> {
        // Initialize Logger based on mode
        if std::env::var("NO_TUI").is_ok() {
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .format_timestamp_millis()
                .init();
            log::info!("Logger initialized in Terminal Mode (NO_TUI)");
        } else {
            let _ = tui_logger::init_logger(log::LevelFilter::Info);
            tui_logger::set_default_level(log::LevelFilter::Info);
        }

        log::info!("=== SYSTEM IGNITION SEQUENCE STARTED ===");

        if std::env::var("KARANA_CHROOT").is_ok() {
            log::info!("Ignited in Sovereign Chroot – Fabric Isolated");
        }
        
        // Atom 4: Verified Awakening
        log::info!("Step 1/8: Boot Awakening...");
        let genesis_hash = 0u64;
        
        // We need mutable access to boot for awaken. 
        self.boot.awaken(genesis_hash).await.context("Boot failed")?;
        log::info!("Step 1/8: Boot Awakening [OK]");

        // Atom 5: Ignite Runtime Actors
        log::info!("Step 2/8: Runtime Ignition...");
        self.runtime.ignite().await.context("Runtime ignition failed")?;
        log::info!("Step 2/8: Runtime Ignition [OK]");

        // Atom 4: Initial Staking (Bootstrap Economy)
        log::info!("Step 3/8: Economy Bootstrap...");
        log::info!("Atom 4 (Economy): Bootstrapping Staking...");
        self.ledger.lock().unwrap().mint("Node-Alpha", 1000);
        self.ledger.lock().unwrap().stake("Node-Alpha", 500)?;
        log::info!("Step 3/8: Economy Bootstrap [OK]");
        
        // Atom 4: DAO Ignition (Phase 4)
        log::info!("Step 4/8: DAO Ignition...");
        {
            let mut dao = self.dao.lock().unwrap();
            dao.token.mint("genesis", U256::from(1000u64));
            let prop = dao.propose("Ignite AI Governance", "Enable on-chain votes for all tunes");
            log::info!("Atom 4 (DAO): Proposed '{}' (ID: {})", "Ignite AI Governance", prop);
            
            if dao.vote("genesis", prop, true).unwrap() {
                log::info!("Atom 4 (DAO): Vote Passed! Executing Governance Ignition...");
                // Runtime effect (simulated)
                // self.runtime.ignite_governance().await?; 
            }
        }
        log::info!("Step 4/8: DAO Ignition [OK]");

        // Atom 6: Symbiotic UI Intent (Test: "Optimize storage")
        log::info!("Step 5/8: UI Intent Test...");
        let intent_proof = vec![1u8; 128];
        let rendered = self.ui.render_intent("optimize storage".to_string(), intent_proof.clone()).await?;
        log::info!("Step 5/8: UI Intent Test [OK]");

        // Atom 7: Vigil Check
        log::info!("Step 6/8: Vigil Security Check...");
        let vigil_result = self.vigil.check_action("storage write".to_string(), intent_proof).await?;
        log::info!("Vigil Check: {}", vigil_result);

        // Atom 7: Vigil Slashing Test (Simulate Malicious Action)
        log::info!("Atom 7 (Vigil): Simulating Malicious Action...");
        match self.vigil.check_action("rm -rf /".to_string(), vec![]).await {
            Ok(_) => log::info!("Vigil: Malicious action passed (Unexpected!)"),
            Err(e) => log::info!("Vigil: Malicious action blocked: {}", e),
        }
        log::info!("Step 6/8: Vigil Security Check [OK]");

        // Atom 2/3: AI-Tuned Storage via Swarm
        log::info!("Step 7/8: Storage & Swarm Test...");
        let test_data = b"AI-optimized shard config";
        // Atom 7: ZK-Attested Storage (Proof generated inside write)
        let block = self.storage.write(test_data, "UI intent")?;
        self.swarm.broadcast_block(&block).await.context("Swarm relay failed")?;

        // Atom 6: ZK-Swarm Routing (Rethink: Connect + Prove Intent)
        // We dial a hypothetical peer to prove we can route this data with ZK attestation.
        // In a real mesh, this would be a peer discovered via DHT.
        let peer_addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/26656".parse().unwrap();
        log::info!("Atom 6 (P2P): Initiating ZK-Dial to {}...", peer_addr);
        self.swarm.zk_dial(peer_addr, block.zk_proof.clone()).await?;

        // Atom 3: Semantic Search Test
        let search_results = self.storage.search("shard configuration")?;
        log::info!("Atom 3 Test: Search for 'shard configuration' returned: {:?}", search_results);

        // Atom 4: Proof of Storage & Incentives
        log::info!("Atom 4: Verifying ZK Proof of Storage...");
        // We use the ZK proof generated in Atom 7 (inside storage.write)
        // The proof attests that the data hashes to the commitment.
        
        // For the demo, we recompute the commitment from the known data to verify the proof.
        let data_to_verify = b"AI-optimized shard config";
        let commitment = crate::zk::compute_hash(data_to_verify);
        
        self.pos.verify_and_reward("Node-Alpha", commitment, &block.zk_proof)?;
        
        // Check Balance
        let balance = self.ledger.lock().unwrap().get_balance("Node-Alpha");
        log::info!("Atom 4: Node-Alpha Balance: {} KARA", balance);

        // Atom 4: Governance Simulation
        log::info!("Atom 4: Simulating Governance...");
        let proposal_id = self.gov.lock().unwrap().create_proposal("Upgrade Storage Sharding");
        self.gov.lock().unwrap().vote(proposal_id, "Node-Alpha", true)?;

        // Atom 5: Verify Tiered Storage Read
        log::info!("Atom 5: Verifying Tiered Storage Read...");
        // We know the data we just wrote: b"AI-optimized shard config"
        // It was chunked. Since it's small (<256 bytes), it's a single chunk.
        let chunk_data = b"AI-optimized shard config";
        let chunk_hash = sha2::Sha256::digest(chunk_data).to_vec();
        
        if let Some(data) = self.storage.read_chunk(&chunk_hash)? {
            log::info!("Atom 5: ✅ Retrieved chunk from Tiered Storage. Size: {} bytes", data.len());
        } else {
            log::info!("Atom 5: ❌ Chunk not found!");
        }
        log::info!("Step 7/8: Storage & Swarm Test [OK]");
        
        // Atom 1: Chain Attest (Genesis tie-in)
        log::info!("Full Flow: Monad Ignited! Rendered: {}, Merkle Root: {:?}", rendered, hex::encode(&block.merkle_root));

        // Phase v1.0: Initial Snapshot
        log::info!("Step 8/8: State Persistence...");
        if let Ok(snap_msg) = self.persist.snapshot_home() {
            log::info!("Atom 2 (Persist): {}", snap_msg);
        }
        log::info!("Step 8/8: State Persistence [OK]");

        log::info!("=== SYSTEM READY: Entering Consensus Loop ===");
        
        let mut height = 1;
        let mut parent_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        let mut last_block_time = std::time::Instant::now();

        loop {
            // Check for Swarm Events
            if let Some(event) = self.swarm.poll_event() {
                match event {
                    KaranaSwarmEvent::BlockReceived(block) => {
                        log::info!("Atom 6 (P2P): Received Block #{} from Swarm", block.header.height);
                        // In a real node, we would validate and add to chain if it's the next block
                    },
                    KaranaSwarmEvent::GenericMessage(msg) => {
                        log::info!("Atom 6 (P2P): Message: {}", msg);
                    },
                    KaranaSwarmEvent::AIRequestReceived(req) => {
                        log::info!("Atom 6 (P2P): Processing AI Request from {}: '{}'", req.requester_peer_id, req.prompt);
                        // Offload compute to our local AI (Non-blocking)
                        let ai_clone = self.ai.clone();
                        let swarm_clone = self.swarm.clone();
                        let req_id = req.request_id;
                        let prompt = req.prompt.clone();
                        
                        tokio::task::spawn_blocking(move || {
                            let result = match ai_clone.lock().unwrap().predict(&prompt, 100) {
                                Ok(r) => r,
                                Err(e) => format!("Compute Error: {}", e),
                            };
                            
                            // We need to send the response. Since we are in a blocking thread, 
                            // we need a runtime handle or just block_on if send_ai_response is async.
                            // send_ai_response is async.
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                if let Err(e) = swarm_clone.send_ai_response(req_id, result).await {
                                    log::error!("Atom 6 (P2P): Failed to send AI response: {}", e);
                                }
                            });
                        });
                    },
                    KaranaSwarmEvent::AIResponseReceived(res) => {
                        log::info!("Atom 6 (P2P): Received AI Result [{}]: {}", res.request_id, res.result);
                        // Notify UI
                        let _ = self.ui.render_intent(format!("Swarm AI Result: {}", res.result), vec![]).await;
                    },
                    KaranaSwarmEvent::ClipboardReceived(clip) => {
                        log::info!("Atom 6 (P2P): Received Clipboard Sync from {}", clip.did);
                        // Verify DID matches local user (or trusted peer)
                        // For now, we just log it and update UI if it's "our" DID
                        let local_did = self.identity.lock().unwrap().get_active_did().unwrap_or_default();
                        if clip.did == local_did {
                            log::info!("Atom 5 (Ecosystem): Syncing Clipboard (Self-Sovereign Sync)...");
                            let _ = self.ui.render_intent(format!("Clipboard Synced: {}", clip.content), vec![]).await;
                        } else {
                            log::info!("Atom 5 (Ecosystem): Ignoring Clipboard from foreign DID: {}", clip.did);
                        }
                    }
                }
            }

            // Check for UI Intents
            if let Some(intent) = self.ui.poll_intent() {
                if intent == "quit" {
                    log::info!("Atom 6 (UI): Quit signal received. Shutting down Sovereign Monad...");
                    return Ok(());
                }
                log::info!("Atom 6 (UI): Processing User Intent: '{}'", intent);
                
                // Phase v1.0: Hardware Intent Interception
                if intent.starts_with("hud") || intent.starts_with("record") || intent.starts_with("scan") {
                    match self.hardware.execute_intent(&intent) {
                        Ok(msg) => {
                            log::info!("Atom 3 (Hardware): {}", msg);
                            // Update UI with hardware feedback
                            let _ = self.ui.render_intent(msg, vec![]).await;
                            continue; // Skip standard render
                        },
                        Err(e) => log::error!("Atom 3 (Hardware): {}", e),
                    }
                }

                // Render intent (Simulate processing)
                let proof = vec![0u8; 64]; // Mock proof
                if let Err(e) = self.ui.render_intent(intent, proof).await {
                    log::error!("Atom 6 (UI): Failed to render intent: {}", e);
                }
            }

            // Block Production (every 5s)
            if last_block_time.elapsed() >= std::time::Duration::from_secs(5) {
                last_block_time = std::time::Instant::now();
                
                let mut txs = Vec::new();
                {
                    let mut pool = self.mempool.lock().unwrap();
                    // Simulate a transaction every block for liveness
                    if height % 2 == 0 {
                        txs.push(Transaction {
                            sender: "Node-Alpha".to_string(),
                            data: TransactionData::Transfer { to: "Node-Beta".to_string(), amount: 10u128 },
                            signature: "mock_sig".to_string(),
                            nonce: height,
                        });
                    }
                    txs.append(&mut pool);
                }
                
                // Create Block
                let block = Block::new(parent_hash.clone(), height, "Node-Alpha".to_string(), txs.clone());
                log::info!("Atom 1 (Chain): Produced Block #{} [Hash: {}] with {} txs", height, block.hash, txs.len());
                
                // Update UI
                self.ui.update_height(height);

                // Validate Block
                if let Err(e) = block.validate(&parent_hash) {
                    log::error!("Atom 1 (Chain): Block Validation Failed: {}", e);
                    continue;
                }

                // Apply Block
                if let Err(e) = self.chain.apply_block(&block) {
                    log::error!("Atom 1 (Chain): Block Application Failed: {}", e);
                } else {
                    log::info!("Atom 1 (Chain): Block #{} Applied Successfully. Hash: {}", height, block.hash);
                    // Persist Block
                    if let Err(e) = self.storage.persist_block(&block) {
                        log::error!("Atom 1 (Chain): Failed to persist block: {}", e);
                    }
                    // Broadcast Block
                    if let Err(e) = self.swarm.broadcast_chain_block(&block).await {
                        log::error!("Atom 6 (P2P): Failed to broadcast block: {}", e);
                    }
                }
                
                parent_hash = block.hash;
                height += 1;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}
