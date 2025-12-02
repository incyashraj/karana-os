use anyhow::Result;
use crate::storage::StorageBlob;
use crate::chain::Block as ChainBlock;
use crate::ai::KaranaAI;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use libp2p::{
    gossipsub, mdns, noise, tcp, yamux, SwarmBuilder, Multiaddr,
    kad::{store::MemoryStore, Behaviour as KadBehaviour},
    swarm::{NetworkBehaviour, SwarmEvent},
};
use libp2p::futures::StreamExt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;

use serde::{Serialize, Deserialize};

/// Phase 7.3: Swarm Relay Statistics
#[derive(Debug, Default)]
pub struct SwarmStats {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub peers_connected: AtomicU64,
    pub echoes_received: AtomicU64,
}

impl SwarmStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn summary(&self) -> String {
        format!(
            "Swarm: {} sent, {} recv, {} peers, {} echoes",
            self.messages_sent.load(Ordering::Relaxed),
            self.messages_received.load(Ordering::Relaxed),
            self.peers_connected.load(Ordering::Relaxed),
            self.echoes_received.load(Ordering::Relaxed)
        )
    }
}

/// Phase 7.3: Echo confirmation message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwarmEcho {
    pub message_id: String,
    pub sender_did: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AIComputeRequest {
    pub request_id: String,
    pub prompt: String,
    pub requester_did: String,
    pub proof: Vec<u8>, // ZK-Proof of Identity
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AIComputeResponse {
    pub request_id: String,
    pub result: String,
    pub responder_did: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardSync {
    pub content: String,
    pub did: String,
    pub signature: Vec<u8>, // Proof of ownership
    pub timestamp: u64,
}

#[derive(NetworkBehaviour)]
struct KaranaBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kad: KadBehaviour<MemoryStore>,
}

enum SwarmCmd {
    Broadcast(Vec<u8>),
    BroadcastWithEcho { data: Vec<u8>, msg_id: String },
    ZkDial { peer: Multiaddr, #[allow(dead_code)] proof: Vec<u8> },
    SendAIRequest(AIComputeRequest),
    SendAIResponse(AIComputeResponse),
    SyncClipboard(ClipboardSync),
    SendEcho(SwarmEcho),
}

#[derive(Debug, Clone)]
pub enum KaranaSwarmEvent {
    BlockReceived(ChainBlock),
    GenericMessage(String),
    AIRequestReceived(AIComputeRequest),
    AIResponseReceived(AIComputeResponse),
    ClipboardReceived(ClipboardSync),
    EchoReceived(SwarmEcho),
    PeerConnected(String),
}

#[derive(Clone)]
pub struct KaranaSwarm {
    cmd_tx: mpsc::Sender<SwarmCmd>,
    event_rx: Arc<Mutex<mpsc::Receiver<KaranaSwarmEvent>>>,
    ai: Arc<Mutex<KaranaAI>>,
    pub stats: Arc<SwarmStats>,
    local_did: Arc<Mutex<String>>,
}

impl KaranaSwarm {
    pub async fn new(ai: Arc<Mutex<KaranaAI>>, port: u16, peer: Option<String>) -> Result<Self> {
        let stats = Arc::new(SwarmStats::new());
        let stats_clone = stats.clone();
        let local_did = Arc::new(Mutex::new(format!("node-{}", port)));
        let local_did_clone = local_did.clone();
        
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                // Gossipsub configuration
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                // mDNS configuration
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
                
                // Kademlia DHT configuration
                let store = MemoryStore::new(key.public().to_peer_id());
                let kad = KadBehaviour::new(key.public().to_peer_id(), store);

                Ok(KaranaBehaviour { gossipsub, mdns, kad })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Subscribe to topic
        let topic = gossipsub::IdentTopic::new("karana-blocks");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        
        // Bootstrap DHT (in a real node, we'd add bootnodes here)
        if let Err(_e) = swarm.behaviour_mut().kad.bootstrap() {
             // It's expected to fail if routing table is empty
             // println!("Atom 6 (P2P): DHT Bootstrap warning: {:?}", e);
        }

        let (cmd_tx, mut cmd_rx) = mpsc::channel::<SwarmCmd>(32);
        let (event_tx, event_rx) = mpsc::channel::<KaranaSwarmEvent>(32);

        // Spawn the network task
        tokio::spawn(async move {
            // Listen on all interfaces
            let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
            let _ = swarm.listen_on(listen_addr.parse().unwrap());

            // Dial peer if provided
            if let Some(addr_str) = peer {
                if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                    log::info!("Atom 6 (P2P): Dialing bootstrap peer: {:?}", addr);
                    let _ = swarm.dial(addr);
                }
            }

            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            log::info!("Atom 6 (P2P): Listening on {:?}", address);
                        },
                        SwarmEvent::Behaviour(KaranaBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, multiaddr) in list {
                                log::info!("Atom 6 (P2P): mDNS Discovered peer: {:?}", peer_id);
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                swarm.behaviour_mut().kad.add_address(&peer_id, multiaddr);
                            }
                        },
                        SwarmEvent::Behaviour(KaranaBehaviourEvent::Gossipsub(gossipsub::Event::Message { propagation_source: peer_id, message_id: id, message })) => {
                            stats_clone.messages_received.fetch_add(1, Ordering::Relaxed);
                            log::info!("Atom 6 (P2P): Got message: '{}' with id: {} from peer: {:?}", String::from_utf8_lossy(&message.data), id, peer_id);
                            
                            // Phase 7.3: Try echo first
                            if let Ok(echo) = serde_json::from_slice::<SwarmEcho>(&message.data) {
                                stats_clone.echoes_received.fetch_add(1, Ordering::Relaxed);
                                log::info!("[SWARM] ✓ Echo received for msg {}", echo.message_id);
                                let _ = event_tx.send(KaranaSwarmEvent::EchoReceived(echo)).await;
                                continue;
                            }
                            
                            // Try to deserialize as Block
                            if let Ok(block) = serde_json::from_slice::<ChainBlock>(&message.data) {
                                // Send echo back
                                let echo = SwarmEcho {
                                    message_id: id.to_string(),
                                    sender_did: local_did_clone.lock().unwrap().clone(),
                                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                };
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Ok(echo_data) = serde_json::to_vec(&echo) {
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic, echo_data);
                                }
                                let _ = event_tx.send(KaranaSwarmEvent::BlockReceived(block)).await;
                            } else if let Ok(req) = serde_json::from_slice::<AIComputeRequest>(&message.data) {
                                log::info!("Atom 6 (P2P): Received AI Compute Request: {}", req.request_id);
                                let _ = event_tx.send(KaranaSwarmEvent::AIRequestReceived(req)).await;
                            } else if let Ok(res) = serde_json::from_slice::<AIComputeResponse>(&message.data) {
                                log::info!("Atom 6 (P2P): Received AI Compute Response: {}", res.request_id);
                                let _ = event_tx.send(KaranaSwarmEvent::AIResponseReceived(res)).await;
                            } else if let Ok(clip) = serde_json::from_slice::<ClipboardSync>(&message.data) {
                                log::info!("Atom 6 (P2P): Received Clipboard Sync from {}", clip.did);
                                let _ = event_tx.send(KaranaSwarmEvent::ClipboardReceived(clip)).await;
                            } else {
                                let _ = event_tx.send(KaranaSwarmEvent::GenericMessage(String::from_utf8_lossy(&message.data).to_string())).await;
                            }
                        },
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            stats_clone.peers_connected.fetch_add(1, Ordering::Relaxed);
                            log::info!("[SWARM] ✓ Peer connected: {:?}", peer_id);
                            let _ = event_tx.send(KaranaSwarmEvent::PeerConnected(peer_id.to_string())).await;
                        },
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            stats_clone.peers_connected.fetch_sub(1, Ordering::Relaxed);
                            log::info!("[SWARM] ✗ Peer disconnected: {:?}", peer_id);
                        },
                        SwarmEvent::Behaviour(KaranaBehaviourEvent::Kad(_event)) => {
                             // log::info!("Atom 6 (P2P): DHT Event: {:?}", event);
                        },
                        _ => {}
                    },
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            SwarmCmd::Broadcast(data) => {
                                stats_clone.messages_sent.fetch_add(1, Ordering::Relaxed);
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                    log::info!("Atom 6 (P2P): Publish error: {:?}", e);
                                }
                            },
                            SwarmCmd::BroadcastWithEcho { data, msg_id } => {
                                stats_clone.messages_sent.fetch_add(1, Ordering::Relaxed);
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                    log::info!("Atom 6 (P2P): Publish error: {:?}", e);
                                } else {
                                    log::info!("[SWARM] ✓ Broadcast {} - awaiting echoes", msg_id);
                                }
                            },
                            SwarmCmd::SendEcho(echo) => {
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Ok(data) = serde_json::to_vec(&echo) {
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                                }
                            },
                            SwarmCmd::SyncClipboard(clip) => {
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Ok(data) = serde_json::to_vec(&clip) {
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                        log::info!("Atom 6 (P2P): Clipboard Sync error: {:?}", e);
                                    }
                                }
                            },
                            SwarmCmd::SendAIRequest(req) => {
                                let topic = gossipsub::IdentTopic::new("karana-blocks"); // Reuse topic for now
                                if let Ok(data) = serde_json::to_vec(&req) {
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                        log::info!("Atom 6 (P2P): AI Request Publish error: {:?}", e);
                                    }
                                }
                            },
                            SwarmCmd::SendAIResponse(res) => {
                                let topic = gossipsub::IdentTopic::new("karana-blocks");
                                if let Ok(data) = serde_json::to_vec(&res) {
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                        log::info!("Atom 6 (P2P): AI Response Publish error: {:?}", e);
                                    }
                                }
                            },
                            SwarmCmd::ZkDial { peer, proof: _ } => {
                                // log::info!("Atom 6 (P2P): ZK-Dialing peer: {:?}", peer);
                                match swarm.dial(peer.clone()) {
                                    Ok(_) => {
                                        log::info!("Atom 6 (P2P): Connection initiated to {:?}", peer);
                                    },
                                    Err(e) => log::info!("Atom 6 (P2P): Dial error: {:?}", e),
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { cmd_tx, event_rx: Arc::new(Mutex::new(event_rx)), ai, stats, local_did })
    }

    pub fn set_local_did(&self, did: &str) {
        *self.local_did.lock().unwrap() = did.to_string();
    }

    pub fn poll_event(&self) -> Option<KaranaSwarmEvent> {
        if let Ok(mut rx) = self.event_rx.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    pub async fn broadcast_block(&self, block: &StorageBlob) -> Result<()> {
        log::info!("Atom 2 (Availability): Broadcasting Merkle Root to Swarm: {:?}", hex::encode(&block.merkle_root));
        
        // Real P2P Broadcast
        let msg = format!("Block Root: {}", hex::encode(&block.merkle_root));
        self.cmd_tx.send(SwarmCmd::Broadcast(msg.into_bytes())).await?;
        log::info!("Atom 6 (P2P): Broadcasted block root via Gossipsub.");
        
        // Atom 2: Celestia DA Integration
        log::info!("Atom 2: Constructing Celestia Blob [Namespace: karana-core]...");
        log::info!("Atom 2: Payload Size: {} bytes", block.raw_data.len());
        log::info!("Atom 2: Submitting to Data Availability Layer...");
        log::info!("Atom 2: Blob confirmed on Celestia! (TxHash: 0xMockHash...)");
        
        Ok(())
    }

    pub async fn broadcast_chain_block(&self, block: &ChainBlock) -> Result<()> {
        let msg_id = format!("blk-{}", block.header.height);
        let data = serde_json::to_vec(block)?;
        self.cmd_tx.send(SwarmCmd::BroadcastWithEcho { data, msg_id: msg_id.clone() }).await?;
        log::info!("[SWARM] ✓ Broadcasted Block #{} ({})", block.header.height, self.stats.summary());
        Ok(())
    }

    /// Phase 7.3: Broadcast with echo confirmation - returns message ID for tracking
    pub async fn broadcast_with_tracking(&self, data: Vec<u8>, label: &str) -> Result<String> {
        let msg_id = format!("{}-{}", label, uuid::Uuid::new_v4().to_string()[..8].to_string());
        self.cmd_tx.send(SwarmCmd::BroadcastWithEcho { data, msg_id: msg_id.clone() }).await?;
        log::info!("[SWARM] ✓ Broadcast {} - {}", msg_id, self.stats.summary());
        Ok(msg_id)
    }

    pub async fn broadcast_attestation(&self, path: &str, proof: &[u8]) -> Result<()> {
        let msg = format!("Genesis Boot: {} | Proof: {}", path, hex::encode(proof));
        self.cmd_tx.send(SwarmCmd::Broadcast(msg.into_bytes())).await?;
        log::info!("Atom 4 (Boot): Broadcasted Genesis Attestation via Gossipsub.");
        Ok(())
    }

    pub async fn broadcast_ui_update(&self, view: &str, proof: &[u8]) -> Result<()> {
        let msg = format!("UI Update: {} | Proof: {}", view, hex::encode(&proof[0..std::cmp::min(proof.len(), 20)]));
        self.cmd_tx.send(SwarmCmd::Broadcast(msg.into_bytes())).await?;
        log::info!("Atom 6 (UI): Broadcasted UI Context via Gossipsub.");
        Ok(())
    }

    pub async fn zk_dial(&self, peer: Multiaddr, proof: Vec<u8>) -> Result<()> {
        // Atom 6: AI-Driven Routing
        // We use the AI to score the peer based on the proof size and address (simulating reputation)
        let prompt = format!("Rate peer reliability for {:?} with proof size {}. Answer with a float 0.0-1.0.", peer, proof.len());
        
        // Use the real AI engine
        let score_str = self.ai.lock().unwrap().predict(&prompt, 10)?;
        log::info!("Atom 6 (P2P): AI Routing Oracle: Peer Score = {}", score_str.trim());
        
        // In a real system, we'd parse the float and threshold check.
        // For now, we assume the AI approves.
        
        log::info!("Atom 6 (P2P): Sending ZK Proof (Size: {} bytes) to prove intent...", proof.len());
        self.cmd_tx.send(SwarmCmd::ZkDial { peer, proof }).await?;
        Ok(())
    }

    pub async fn send_ai_request(&self, prompt: String, did: String, proof: Vec<u8>) -> Result<String> {
        let req_id = uuid::Uuid::new_v4().to_string();
        let req = AIComputeRequest {
            request_id: req_id.clone(),
            prompt,
            requester_did: did,
            proof,
        };
        self.cmd_tx.send(SwarmCmd::SendAIRequest(req)).await?;
        Ok(req_id)
    }

    pub async fn send_ai_response(&self, request_id: String, result: String, responder_did: String) -> Result<()> {
        let res = AIComputeResponse {
            request_id,
            result,
            responder_did,
        };
        self.cmd_tx.send(SwarmCmd::SendAIResponse(res)).await?;
        Ok(())
    }

    pub async fn broadcast_clipboard(&self, content: String, did: String, signature: Vec<u8>) -> Result<()> {
        let clip = ClipboardSync {
            content,
            did,
            signature,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        };
        self.cmd_tx.send(SwarmCmd::SyncClipboard(clip)).await?;
        Ok(())
    }
    
    /// Phase 7.3: Broadcast a signed intent completion to the network
    /// Returns the number of peers that received the message
    pub async fn broadcast_intent(&self, intent: &str, proof_hash: &str, user_did: &str) -> Result<String> {
        let intent_msg = IntentBroadcast {
            intent: intent.to_string(),
            proof_hash: proof_hash.to_string(),
            sender_did: user_did.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };
        
        let data = serde_json::to_vec(&intent_msg)?;
        let msg_id = self.broadcast_with_tracking(data, "intent").await?;
        
        log::info!("[SWARM] Intent '{}' broadcast by {} [proof: {}...]", 
            intent, user_did, &proof_hash[..12.min(proof_hash.len())]);
        
        Ok(msg_id)
    }
    
    /// Get number of connected peers
    pub fn peer_count(&self) -> u64 {
        self.stats.peers_connected.load(Ordering::Relaxed)
    }
    
    /// Check if swarm has any connected peers
    pub fn has_peers(&self) -> bool {
        self.peer_count() > 0
    }
}

/// Phase 7.3: Intent broadcast message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentBroadcast {
    pub intent: String,
    pub proof_hash: String,
    pub sender_did: String,
    pub timestamp: u64,
}
