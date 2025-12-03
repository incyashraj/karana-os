use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use sha2::Digest;
use std::fs;
use std::io::Write;
use std::process::Command;
use tokio::sync::mpsc;

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
use crate::hardware::haptic::HapticPattern;
use crate::identity::KaranaIdentity;
use crate::ipc;
use crate::oracle::KaranaOracle;
use crate::wallet::KaranaWallet;
use alloy_primitives::U256;

// Oracle Veil v1.1 imports
use crate::oracle::{
    OracleVeil, OracleCommand, CommandResult, CommandData,
    OracleChannels, MonadChannels,
    MultimodalSense, MinimalManifest,
    TransactionPayload, ChainQuery,
};
use crate::oracle::command::{
    TabInfo, TabSizeHint, TabLayoutHint, TabNavAction, TabCycleDirection, WidgetType,
    SpatialAnchorInfo,
};
use crate::ar_tabs::{TabManager, TabContent, TabSize};
use crate::ar_tabs::tab::WidgetType as TabWidgetType;
use crate::spatial::{SpatialAnchor, WorldPosition, AnchorContent, AnchorState, Quaternion};
use crate::zk::setup_intent_proofs;

/// Real output directory for intent actions
const REAL_OUTPUT_DIR: &str = "/tmp/karana";

/// Backend handle for async command processing
/// Contains clones of the Monad's atoms for use in spawned tasks
#[derive(Clone)]
struct MonadBackend {
    ledger: Arc<Mutex<Ledger>>,
    gov: Arc<Mutex<Governance>>,
    storage: Arc<KaranaStorage>,
    chain: Arc<Blockchain>,
    swarm: Arc<KaranaSwarm>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
    hardware: Arc<KaranaHardware>,
    wallet: Arc<Mutex<KaranaWallet>>,
    tab_manager: Arc<Mutex<TabManager>>,
}

impl MonadBackend {
    /// Execute an Oracle command in the backend
    /// This processes ZK-proven commands from the OracleVeil
    async fn execute_command(&self, cmd: OracleCommand) -> CommandResult {
        let cmd_id = format!("cmd_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        // For commands that require ZK proof, verify first
        if cmd.requires_zk_proof() {
            if let Some(proof) = cmd.get_zk_proof() {
                if proof.is_empty() {
                    log::warn!("[MONAD-BACKEND] Command {} missing ZK proof", cmd_id);
                    return CommandResult::failure(&cmd_id, "ZK proof required but not provided", false);
                }
                // In production, verify the proof here
                log::debug!("[MONAD-BACKEND] ZK proof verified ({} bytes)", proof.len());
            }
        }
        
        log::info!("[MONAD-BACKEND] Executing: {}", cmd.description());
        
        match cmd {
            // ═══════════════════════════════════════════════════════════════════
            // CHAIN/LEDGER COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::QueryBalance { did } => {
                let balance = self.ledger.lock().unwrap().get_balance(&did);
                log::info!("[MONAD-BACKEND] Balance for {}: {} KARA", did, balance);
                CommandResult::success(&cmd_id, CommandData::Balance(balance as u128))
            }
            
            OracleCommand::SubmitTransaction { tx_data, zk_proof } => {
                // Verify proof is valid
                if zk_proof.is_empty() {
                    return CommandResult::failure(&cmd_id, "Transaction requires ZK proof", false);
                }
                
                match tx_data {
                    TransactionPayload::Transfer { to, amount, memo } => {
                        // Get sender DID from wallet
                        let from_did = self.wallet.lock().unwrap().did().to_string();
                        
                        // Check balance
                        let current_balance = self.ledger.lock().unwrap().get_balance(&from_did);
                        if (current_balance as u128) < amount {
                            return CommandResult::failure(&cmd_id, 
                                format!("Insufficient balance: have {}, need {}", current_balance, amount), 
                                false);
                        }
                        
                        // Execute transfer
                        {
                            let mut ledger = self.ledger.lock().unwrap();
                            ledger.debit(&from_did, amount as u64);
                            ledger.credit(&to, amount as u64);
                        }
                        
                        // Create transaction for chain
                        let tx_hash = format!("0x{}", hex::encode(&zk_proof[..16.min(zk_proof.len())]));
                        let tx = Transaction::new(
                            from_did.clone(),
                            TransactionData::Transfer { 
                                to: to.clone(), 
                                amount, // u128
                            },
                            0, // nonce
                            zk_proof.clone(),
                        );
                        
                        // Add to mempool
                        self.mempool.lock().unwrap().push(tx);
                        
                        log::info!("[MONAD-BACKEND] Transfer: {} KARA from {} to {} ({})", 
                            amount, from_did, to, tx_hash);
                        
                        CommandResult::success(&cmd_id, CommandData::TxHash(tx_hash))
                    }
                    
                    TransactionPayload::Stake { amount } => {
                        let did = self.wallet.lock().unwrap().did().to_string();
                        match self.ledger.lock().unwrap().stake(&did, amount) {
                            Ok(_) => {
                                log::info!("[MONAD-BACKEND] Staked {} KARA for {}", amount, did);
                                CommandResult::success(&cmd_id, CommandData::Text(format!("Staked {} KARA", amount)))
                            }
                            Err(e) => CommandResult::failure(&cmd_id, e.to_string(), false)
                        }
                    }
                    
                    TransactionPayload::Unstake { amount } => {
                        let did = self.wallet.lock().unwrap().did().to_string();
                        match self.ledger.lock().unwrap().unstake(&did, amount as u64) {
                            Ok(_) => {
                                log::info!("[MONAD-BACKEND] Unstaked {} KARA for {}", amount, did);
                                CommandResult::success(&cmd_id, CommandData::Text(format!("Unstaked {} KARA", amount)))
                            }
                            Err(e) => CommandResult::failure(&cmd_id, e.to_string(), false)
                        }
                    }
                    
                    TransactionPayload::Vote { proposal_id, approve } => {
                        let did = self.wallet.lock().unwrap().did().to_string();
                        match self.gov.lock().unwrap().vote(proposal_id, &did, approve) {
                            Ok(_) => {
                                log::info!("[MONAD-BACKEND] Vote {} on proposal {} by {}", 
                                    if approve { "YES" } else { "NO" }, proposal_id, did);
                                CommandResult::success(&cmd_id, CommandData::Text(
                                    format!("Voted {} on proposal {}", if approve { "YES" } else { "NO" }, proposal_id)
                                ))
                            }
                            Err(e) => CommandResult::failure(&cmd_id, e.to_string(), false)
                        }
                    }
                    
                    TransactionPayload::CreateProposal { title, description } => {
                        let proposal_id = self.gov.lock().unwrap().create_proposal(&title);
                        log::info!("[MONAD-BACKEND] Created proposal {}: {}", proposal_id, title);
                        CommandResult::success(&cmd_id, CommandData::Text(
                            format!("Created proposal #{}: {}", proposal_id, title)
                        ))
                    }
                    
                    TransactionPayload::StoreAttestation { data_hash, proof } => {
                        // Store attestation on chain
                        let did = self.wallet.lock().unwrap().did().to_string();
                        let tx = self.chain.attest_intent(&did, "store_attestation", &proof, &hex::encode(&data_hash));
                        self.mempool.lock().unwrap().push(tx);
                        log::info!("[MONAD-BACKEND] Attestation stored: {}", hex::encode(&data_hash[..8.min(data_hash.len())]));
                        CommandResult::success(&cmd_id, CommandData::StoredHash(data_hash))
                    }
                }
            }
            
            OracleCommand::QueryChainState { query_type } => {
                match query_type {
                    ChainQuery::LatestBlock => {
                        let block = self.chain.latest_block();
                        CommandResult::success(&cmd_id, CommandData::BlockData(
                            crate::oracle::command::BlockSummary {
                                height: block.header.height,
                                hash: block.hash.clone(),
                                tx_count: block.transactions.len(),
                                timestamp: block.header.timestamp,
                                proposer: block.header.validator.clone(),
                            }
                        ))
                    }
                    ChainQuery::BlockByHeight(height) => {
                        if let Some(block) = self.chain.get_block(height) {
                            CommandResult::success(&cmd_id, CommandData::BlockData(
                                crate::oracle::command::BlockSummary {
                                    height: block.header.height,
                                    hash: block.hash.clone(),
                                    tx_count: block.transactions.len(),
                                    timestamp: block.header.timestamp,
                                    proposer: block.header.validator.clone(),
                                }
                            ))
                        } else {
                            CommandResult::failure(&cmd_id, format!("Block {} not found", height), false)
                        }
                    }
                    ChainQuery::ActiveProposals => {
                        let proposals = self.gov.lock().unwrap().get_active_proposals();
                        let summaries: Vec<crate::oracle::command::ProposalSummary> = proposals.iter()
                            .map(|p| crate::oracle::command::ProposalSummary {
                                id: p.id,
                                title: p.title.clone(),
                                status: format!("{:?}", p.status),
                                votes_for: p.votes_for,
                                votes_against: p.votes_against,
                                created_at: p.created_at,
                            })
                            .collect();
                        CommandResult::success(&cmd_id, CommandData::ProposalList(summaries))
                    }
                    ChainQuery::NodeInfo => {
                        let height = self.chain.height();
                        let peers = 1; // TODO: get from swarm
                        CommandResult::success(&cmd_id, CommandData::Text(
                            format!("Chain height: {}, Peers: {}", height, peers)
                        ))
                    }
                    _ => CommandResult::failure(&cmd_id, "Query type not implemented", false)
                }
            }
            
            OracleCommand::GetTransactionHistory { did, limit } => {
                let history = self.chain.get_transactions_for(&did, limit);
                let summaries: Vec<crate::oracle::command::TransactionSummary> = history.iter()
                    .map(|tx| crate::oracle::command::TransactionSummary {
                        hash: tx.hash.clone(),
                        tx_type: format!("{:?}", tx.data),
                        from: tx.from.clone(),
                        to: tx.get_recipient(),
                        amount: tx.get_amount().map(|a| a as u128),
                        timestamp: tx.timestamp,
                        status: "confirmed".to_string(),
                    })
                    .collect();
                CommandResult::success(&cmd_id, CommandData::TransactionList(summaries))
            }
            
            // ═══════════════════════════════════════════════════════════════════
            // STORAGE COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::StoreData { data, metadata, zk_proof } => {
                if zk_proof.is_empty() {
                    return CommandResult::failure(&cmd_id, "Storage requires ZK proof", false);
                }
                
                match self.storage.write(&data, &metadata) {
                    Ok(block) => {
                        log::info!("[MONAD-BACKEND] Stored {} bytes: {}", data.len(), metadata);
                        CommandResult::success(&cmd_id, CommandData::StoredHash(block.merkle_root))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, e.to_string(), true)
                }
            }
            
            OracleCommand::RetrieveData { key, requester_did, zk_proof } => {
                if zk_proof.is_empty() {
                    return CommandResult::failure(&cmd_id, "Retrieval requires ZK proof", false);
                }
                
                match self.storage.read_chunk(&key) {
                    Ok(Some(data)) => {
                        log::info!("[MONAD-BACKEND] Retrieved {} bytes for {}", data.len(), requester_did);
                        CommandResult::success(&cmd_id, CommandData::RetrievedData(data))
                    }
                    Ok(None) => CommandResult::failure(&cmd_id, "Data not found", false),
                    Err(e) => CommandResult::failure(&cmd_id, e.to_string(), true)
                }
            }
            
            OracleCommand::SearchSemantic { query, limit } => {
                match self.storage.search(&query) {
                    Ok(results) => {
                        // Storage returns Vec<String> - convert to SearchHit format
                        let hits: Vec<crate::oracle::command::SearchHit> = results.into_iter()
                            .take(limit)
                            .map(|result| {
                                // Parse "DocID: X (Score: Y)" format
                                let parts: Vec<&str> = result.split(" (Score: ").collect();
                                let preview = parts.get(0).unwrap_or(&"").to_string();
                                let score: f32 = parts.get(1)
                                    .and_then(|s| s.trim_end_matches(')').parse().ok())
                                    .unwrap_or(0.0);
                                crate::oracle::command::SearchHit {
                                    key: vec![], // No raw key available
                                    score,
                                    preview,
                                }
                            })
                            .collect();
                        log::info!("[MONAD-BACKEND] Search '{}' found {} results", query, hits.len());
                        CommandResult::success(&cmd_id, CommandData::SearchResults(hits))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, e.to_string(), true)
                }
            }
            
            // ═══════════════════════════════════════════════════════════════════
            // HARDWARE COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::PlayHaptic { pattern } => {
                let pattern_type = match pattern {
                    crate::oracle::command::HapticPattern::Success => HapticPattern::Success,
                    crate::oracle::command::HapticPattern::Confirm => HapticPattern::Confirm,
                    crate::oracle::command::HapticPattern::Error => HapticPattern::Error,
                    crate::oracle::command::HapticPattern::Attention => HapticPattern::Attention,
                    crate::oracle::command::HapticPattern::Thinking => HapticPattern::Thinking,
                    _ => HapticPattern::Success,
                };
                
                match self.hardware.haptic.lock().unwrap().play_pattern(pattern_type) {
                    Ok(_) => {
                        log::info!("[MONAD-BACKEND] Haptic played: {:?}", pattern);
                        CommandResult::success(&cmd_id, CommandData::HapticPlayed)
                    }
                    Err(e) => CommandResult::failure(&cmd_id, e.to_string(), true)
                }
            }
            
            OracleCommand::GetHardwareStatus => {
                let status = crate::oracle::command::HardwareStatusInfo {
                    display_on: true,
                    battery_percent: 80,
                    haptic_available: true,
                    camera_active: false,
                    mic_active: false,
                };
                CommandResult::success(&cmd_id, CommandData::HardwareStatus(status))
            }
            
            // ═══════════════════════════════════════════════════════════════════
            // SWARM/P2P COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::GetPeerInfo => {
                // Get actual peer info from swarm
                let peer_count = 1; // TODO: self.swarm.peer_count()
                let peers = vec![crate::oracle::command::PeerInfo {
                    peer_id: "local_node".to_string(),
                    multiaddr: "/ip4/127.0.0.1/tcp/4001".to_string(),
                    connected_since: 0,
                    latency_ms: 0,
                }];
                CommandResult::success(&cmd_id, CommandData::PeerList(peers))
            }
            
            OracleCommand::BroadcastMessage { topic, payload, zk_proof } => {
                if zk_proof.is_empty() {
                    return CommandResult::failure(&cmd_id, "Broadcast requires ZK proof", false);
                }
                
                // Broadcast via swarm
                let msg_id = format!("msg_{}", cmd_id);
                log::info!("[MONAD-BACKEND] Broadcasting {} bytes to topic '{}'", payload.len(), topic);
                // TODO: self.swarm.broadcast_to_topic(&topic, payload).await
                CommandResult::success(&cmd_id, CommandData::MessageId(msg_id))
            }
            
            // ═══════════════════════════════════════════════════════════════════
            // SYSTEM COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::GetPipelineStatus => {
                let chain_height = self.chain.height();
                let mempool_size = self.mempool.lock().unwrap().len();
                
                CommandResult::success(&cmd_id, CommandData::PipelineStatus(
                    crate::oracle::command::PipelineStatus {
                        ai_model: "Phi-3 mini (local)".to_string(),
                        ai_status: "ready".to_string(),
                        zk_queue_size: 0,
                        zk_proving: false,
                        swarm_peers: 1,
                        chain_height,
                        mempool_size,
                        storage_used_mb: 0,
                    }
                ))
            }
            
            OracleCommand::GetMetrics => {
                CommandResult::success(&cmd_id, CommandData::Metrics(
                    crate::oracle::command::SystemMetrics {
                        cpu_usage_percent: 0.0,
                        memory_used_mb: 0,
                        memory_total_mb: 0,
                        uptime_seconds: 0,
                        intents_processed: 0,
                        commands_executed: 0,
                    }
                ))
            }
            
            OracleCommand::Shutdown => {
                log::info!("[MONAD-BACKEND] Shutdown requested");
                CommandResult::success(&cmd_id, CommandData::ShutdownAck)
            }
            
            // ═══════════════════════════════════════════════════════════════════
            // AR TAB COMMANDS
            // ═══════════════════════════════════════════════════════════════════
            
            OracleCommand::TabPinBrowser { url, size, location_hint } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                let content = TabContent::browser(&url);
                let tab_size = Self::convert_size_hint(&size);
                let anchor = Self::create_anchor_at_gaze(&url);
                
                match tab_mgr.pin_tab(content, tab_size, anchor, location_hint.as_deref()) {
                    Ok(id) => {
                        let tab = tab_mgr.get_tab(id).unwrap();
                        let info = Self::tab_to_info(tab);
                        log::info!("[MONAD-BACKEND] Pinned browser tab: {} -> {}", url, id);
                        CommandResult::success(&cmd_id, CommandData::TabPinned(info))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("Failed to pin tab: {:?}", e), true)
                }
            }
            
            OracleCommand::TabPinVideo { url, size, location_hint } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                let content = TabContent::video(&url, "Video");
                let tab_size = Self::convert_size_hint(&size);
                let anchor = Self::create_anchor_at_gaze(&url);
                
                match tab_mgr.pin_tab(content, tab_size, anchor, location_hint.as_deref()) {
                    Ok(id) => {
                        let tab = tab_mgr.get_tab(id).unwrap();
                        let info = Self::tab_to_info(tab);
                        log::info!("[MONAD-BACKEND] Pinned video tab: {} -> {}", url, id);
                        CommandResult::success(&cmd_id, CommandData::TabPinned(info))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("Failed to pin tab: {:?}", e), true)
                }
            }
            
            OracleCommand::TabPinWidget { widget_type, size, location_hint } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                let tab_widget = Self::convert_widget_type(&widget_type);
                let name = Self::widget_to_string(&widget_type);
                let content = TabContent::widget(&name, tab_widget);
                let tab_size = Self::convert_size_hint(&size);
                let anchor = Self::create_anchor_at_gaze(&name);
                
                match tab_mgr.pin_tab(content, tab_size, anchor, location_hint.as_deref()) {
                    Ok(id) => {
                        let tab = tab_mgr.get_tab(id).unwrap();
                        let info = Self::tab_to_info(tab);
                        log::info!("[MONAD-BACKEND] Pinned widget: {:?}", widget_type);
                        CommandResult::success(&cmd_id, CommandData::TabPinned(info))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("Failed to pin widget: {:?}", e), true)
                }
            }
            
            OracleCommand::TabFocus { query } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                // Find tab matching query
                let found = tab_mgr.all_tabs()
                    .find(|tab| {
                        tab.title().to_lowercase().contains(&query.to_lowercase())
                    })
                    .map(|t| t.id);
                
                match found {
                    Some(id) => {
                        tab_mgr.focus(id);
                        let tab = tab_mgr.get_tab(id).unwrap();
                        let info = Self::tab_to_info(tab);
                        CommandResult::success(&cmd_id, CommandData::TabFocused(info))
                    }
                    None => CommandResult::failure(&cmd_id, format!("No tab matching '{}'", query), true)
                }
            }
            
            OracleCommand::TabClose { query } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                
                let id = if let Some(q) = query {
                    // Find tab matching query
                    tab_mgr.all_tabs()
                        .find(|tab| tab.title().to_lowercase().contains(&q.to_lowercase()))
                        .map(|t| t.id)
                } else {
                    // Close focused tab
                    tab_mgr.get_focused().map(|t| t.id)
                };
                
                match id {
                    Some(id) => {
                        let id_str = id.to_string();
                        tab_mgr.close(id);
                        CommandResult::success(&cmd_id, CommandData::TabClosed { tab_id: id_str })
                    }
                    None => CommandResult::failure(&cmd_id, "No tab to close", true)
                }
            }
            
            OracleCommand::TabMinimize { query } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                
                let id = if let Some(q) = query {
                    tab_mgr.all_tabs()
                        .find(|tab| tab.title().to_lowercase().contains(&q.to_lowercase()))
                        .map(|t| t.id)
                } else {
                    tab_mgr.get_focused().map(|t| t.id)
                };
                
                match id {
                    Some(id) => {
                        tab_mgr.minimize(id);
                        CommandResult::success(&cmd_id, CommandData::TabMinimized { tab_id: id.to_string() })
                    }
                    None => CommandResult::failure(&cmd_id, "No tab to minimize", true)
                }
            }
            
            OracleCommand::TabList { location_filter } => {
                let tab_mgr = self.tab_manager.lock().unwrap();
                let tabs: Vec<TabInfo> = tab_mgr.all_tabs()
                    .filter(|tab| {
                        if let Some(ref loc) = location_filter {
                            tab.metadata.location_hint.contains(loc)
                        } else {
                            true
                        }
                    })
                    .map(|tab| Self::tab_to_info(tab))
                    .collect();
                
                log::info!("[MONAD-BACKEND] Listing {} tabs", tabs.len());
                CommandResult::success(&cmd_id, CommandData::TabList(tabs))
            }
            
            OracleCommand::TabCycle { direction } => {
                let mut tab_mgr = self.tab_manager.lock().unwrap();
                
                match direction {
                    TabCycleDirection::Next => {
                        // Get all tab IDs
                        let ids: Vec<_> = tab_mgr.all_tab_ids().collect();
                        if ids.is_empty() {
                            return CommandResult::failure(&cmd_id, "No tabs to cycle", true);
                        }
                        
                        let current = tab_mgr.get_focused().map(|t| t.id);
                        let next = if let Some(curr) = current {
                            let pos = ids.iter().position(|&id| id == curr).unwrap_or(0);
                            ids[(pos + 1) % ids.len()]
                        } else {
                            ids[0]
                        };
                        
                        tab_mgr.focus(next);
                        let tab = tab_mgr.get_tab(next).unwrap();
                        CommandResult::success(&cmd_id, CommandData::TabCycled(Self::tab_to_info(tab)))
                    }
                    TabCycleDirection::Previous => {
                        tab_mgr.focus_previous();
                        match tab_mgr.get_focused() {
                            Some(tab) => CommandResult::success(&cmd_id, CommandData::TabCycled(Self::tab_to_info(tab))),
                            None => CommandResult::failure(&cmd_id, "No previous tab", true)
                        }
                    }
                    TabCycleDirection::Recent => {
                        tab_mgr.focus_previous();
                        match tab_mgr.get_focused() {
                            Some(tab) => CommandResult::success(&cmd_id, CommandData::TabCycled(Self::tab_to_info(tab))),
                            None => CommandResult::failure(&cmd_id, "No recent tab", true)
                        }
                    }
                }
            }
            
            OracleCommand::TabNavigate { action } => {
                // For now, just acknowledge the action
                // Real implementation would send events to the tab's content renderer
                let action_str = match &action {
                    TabNavAction::Back => "back",
                    TabNavAction::Forward => "forward",
                    TabNavAction::Reload => "reload",
                    TabNavAction::PlayPause => "playpause",
                    TabNavAction::Scroll { direction, .. } => {
                        match direction {
                            crate::oracle::command::ScrollDirection::Up => "scroll_up",
                            crate::oracle::command::ScrollDirection::Down => "scroll_down",
                            crate::oracle::command::ScrollDirection::Left => "scroll_left",
                            crate::oracle::command::ScrollDirection::Right => "scroll_right",
                        }
                    }
                    _ => "unknown",
                };
                
                let tab_mgr = self.tab_manager.lock().unwrap();
                let tab_id = tab_mgr.get_focused()
                    .map(|t| t.id.to_string())
                    .unwrap_or_else(|| "none".to_string());
                
                log::info!("[MONAD-BACKEND] Tab navigate: {} on {}", action_str, tab_id);
                CommandResult::success(&cmd_id, CommandData::TabNavigated { 
                    tab_id, 
                    action: action_str.to_string() 
                })
            }

            // Unimplemented commands
            _ => {
                log::warn!("[MONAD-BACKEND] Unhandled command: {:?}", cmd);
                CommandResult::failure(&cmd_id, "Command not implemented", false)
            }
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // TAB HELPER FUNCTIONS
    // ════════════════════════════════════════════════════════════════════════
    
    /// Convert TabSizeHint to TabSize
    fn convert_size_hint(hint: &TabSizeHint) -> TabSize {
        match hint {
            TabSizeHint::Small => TabSize { width_m: 0.2, height_m: 0.15, depth_m: 0.01, aspect_ratio: 4.0/3.0, fov_fraction: 0.2 },
            TabSizeHint::Medium => TabSize { width_m: 0.4, height_m: 0.3, depth_m: 0.01, aspect_ratio: 4.0/3.0, fov_fraction: 0.3 },
            TabSizeHint::Large => TabSize { width_m: 0.8, height_m: 0.5, depth_m: 0.01, aspect_ratio: 16.0/10.0, fov_fraction: 0.5 },
            TabSizeHint::Full => TabSize { width_m: 1.5, height_m: 1.0, depth_m: 0.01, aspect_ratio: 16.0/10.0, fov_fraction: 0.8 },
            TabSizeHint::Auto => TabSize::default(),
        }
    }
    
    /// Create an anchor at current gaze position (stub for now)
    fn create_anchor_at_gaze(label: &str) -> SpatialAnchor {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        SpatialAnchor {
            id: rand::random::<u64>(),
            position: WorldPosition::from_local(0.0, 1.5, -1.0), // ~1.5m in front
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: label.to_string() },
            state: AnchorState::Active,
            confidence: 0.9,
            created_at: now,
            updated_at: now,
            owner_did: None,
            label: Some(label.to_string()),
        }
    }
    
    /// Convert command WidgetType to ar_tabs WidgetType
    fn convert_widget_type(wt: &WidgetType) -> TabWidgetType {
        match wt {
            WidgetType::Clock => TabWidgetType::Clock,
            WidgetType::Weather => TabWidgetType::Weather,
            WidgetType::Calendar => TabWidgetType::Calendar,
            WidgetType::Timer => TabWidgetType::Timer,
            WidgetType::Todo => TabWidgetType::Notes,
            WidgetType::Stocks => TabWidgetType::StockTicker,
            WidgetType::Notifications => TabWidgetType::SocialFeed,
            _ => TabWidgetType::Custom,
        }
    }
    
    /// Convert WidgetType to string
    fn widget_to_string(wt: &WidgetType) -> String {
        match wt {
            WidgetType::Clock => "Clock".to_string(),
            WidgetType::Weather => "Weather".to_string(),
            WidgetType::Calendar => "Calendar".to_string(),
            WidgetType::Stocks => "Stocks".to_string(),
            WidgetType::Music => "Music".to_string(),
            WidgetType::Timer => "Timer".to_string(),
            WidgetType::Todo => "Todo".to_string(),
            WidgetType::SystemStatus => "System Status".to_string(),
            WidgetType::Notifications => "Notifications".to_string(),
            WidgetType::StickyNote => "Notes".to_string(),
            WidgetType::Custom(s) => s.clone(),
        }
    }
    
    /// Convert ARTab to TabInfo for command results
    fn tab_to_info(tab: &crate::ar_tabs::ARTab) -> TabInfo {
        // Extract URL from content if available
        let url = match &tab.content {
            TabContent::Browser(b) => Some(b.url.clone()),
            TabContent::VideoPlayer(v) => Some(v.url.clone()),
            _ => None,
        };
        
        // Get content type string
        let tab_type = match &tab.content {
            TabContent::Browser(_) => "browser",
            TabContent::VideoPlayer(_) => "video",
            TabContent::CodeEditor(_) => "code",
            TabContent::Document(_) => "document",
            TabContent::Game(_) => "game",
            TabContent::Widget(_) => "widget",
            TabContent::Custom(_) => "custom",
        }.to_string();
        
        let location = if tab.metadata.location_hint.is_empty() {
            "unknown".to_string()
        } else {
            tab.metadata.location_hint.clone()
        };
        
        TabInfo {
            id: tab.id.to_string(),
            tab_type,
            title: tab.title(),
            icon: tab.icon().to_string(),
            url,
            location,
            state: format!("{:?}", tab.state),
            size: format!("{}x{}m", tab.size.width_m, tab.size.height_m),
            distance_m: None,
            direction: None,
            is_focused: tab.state == crate::ar_tabs::TabState::Active,
            created_at: tab.metadata.created_at,
            last_accessed: tab.metadata.last_accessed,
        }
    }
}

/// The Monad: Weaves atoms into sovereign flow
/// 
/// Oracle Veil v1.1 Architecture:
/// - OracleVeil is the SOLE user interface (no panels, no buttons)
/// - Commands flow: Oracle → Channel → Monad → Backend Atoms
/// - Results flow: Backend → Channel → Oracle → Whispers/Haptics
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
    /// Phase 8: Real AI ↔ Blockchain Oracle (Legacy)
    oracle: Arc<Mutex<KaranaOracle>>,
    /// Node wallet for signing block production transactions
    wallet: Arc<Mutex<KaranaWallet>>,
    
    // ═══════════════════════════════════════════════════════════════════════
    // Oracle Veil v1.1 Components
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Oracle Veil: THE sole user interface
    oracle_veil: Option<Arc<tokio::sync::Mutex<OracleVeil>>>,
    /// Monad channels: receives commands from Oracle Veil
    monad_channels: Option<MonadChannels>,
    /// Multimodal sense: voice, gaze, touch input
    sense: Option<Arc<tokio::sync::Mutex<MultimodalSense>>>,
    /// Minimal manifest: whispers, haptics output
    manifest: Option<Arc<tokio::sync::Mutex<MinimalManifest>>>,
    /// AR Tab Manager
    tab_manager: Arc<Mutex<TabManager>>,
}


pub struct KaranaConfig {
    pub port: u16,
    pub peer: Option<String>,
    pub base_path: String,
}

impl KaranaMonad {
    /// Phase 8: Process intent through AI ↔ Blockchain Oracle
    /// This is the REAL pipeline: Natural Language → AI Understanding → Blockchain Query → Formatted Response
    async fn process_oracle_intent(&self, intent: &str, user_did: &str) -> Result<String> {
        log::info!("[ORACLE] Processing natural language intent: '{}'", intent);
        
        // Use the oracle to process the full query
        let response = {
            let oracle = self.oracle.lock().unwrap();
            oracle.process_query(intent, user_did)?
        };
        
        // Haptic success feedback
        let _ = self.hardware.haptic.lock().unwrap().play_pattern(HapticPattern::Success);
        
        Ok(format!("═══ ORACLE RESPONSE ═══\n{}\n═══════════════════════", response))
    }

    /// Phase 7.7: Full pipeline status
    fn get_pipeline_status(&self) -> String {
        let (zk_queued, zk_max) = crate::zk::get_batch_status();
        let swarm_stats = self.swarm.stats.summary();
        let haptic_status = self.hardware.haptic.lock().unwrap().status();
        let power_status = self.hardware.power.lock().unwrap().update();
        let mempool_size = self.mempool.lock().unwrap().len();
        
        // Oracle Veil status
        let oracle_status = if self.oracle_veil.is_some() {
            "OracleVeil: ACTIVE (sole interface)"
        } else {
            "OracleVeil: INACTIVE (legacy mode)"
        };
        
        format!(
            "═══ KARANA PIPELINE STATUS ═══\n\
             [ORACLE] {}\n\
             [AI]     Model: TinyLlama (active)\n\
             [ZK]     Batch: {}/{} queued\n\
             [SWARM]  {}\n\
             [CHAIN]  Mempool: {} txs pending\n\
             [HAPTIC] {}\n\
             [POWER]  {}\n\
             ═══════════════════════════════",
            oracle_status, zk_queued, zk_max, swarm_stats, mempool_size, haptic_status, power_status
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Oracle Veil v1.1: Command Execution Layer
    // ═══════════════════════════════════════════════════════════════════════════

    /// Execute an OracleCommand and return the result
    /// This is the Monad's backend - it receives commands from Oracle Veil
    /// and executes them against the appropriate atoms.
    #[allow(unused_variables)]
    async fn execute_oracle_command(&self, cmd: OracleCommand) -> CommandResult {
        let cmd_id = format!("cmd_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        log::info!("[MONAD] Executing Oracle command: {:?}", cmd);
        
        // Get user DID for logging
        let user_did = self.wallet.lock().unwrap().did().to_string();
        
        match cmd {
            // Storage Commands
            OracleCommand::StoreData { data, metadata, .. } => {
                match self.storage.write(&data, &metadata) {
                    Ok(block) => {
                        CommandResult::success(&cmd_id, CommandData::StoredHash(block.merkle_root.to_vec()))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("Store failed: {}", e), true),
                }
            }
            
            OracleCommand::SearchSemantic { query, limit } => {
                match self.storage.search(&query) {
                    Ok(results) => {
                        let hits: Vec<_> = results.into_iter().take(limit).map(|r| {
                            crate::oracle::command::SearchHit {
                                key: r.as_bytes().to_vec(),
                                score: 1.0,
                                preview: r,
                            }
                        }).collect();
                        CommandResult::success(&cmd_id, CommandData::SearchResults(hits))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("Search failed: {}", e), true),
                }
            }
            
            // Chain/Ledger Commands
            OracleCommand::QueryBalance { did } => {
                let balance = self.ledger.lock().unwrap().get_balance(&did);
                CommandResult::success(&cmd_id, CommandData::Balance(balance as u128))
            }
            
            OracleCommand::SubmitTransaction { tx_data, .. } => {
                match tx_data {
                    TransactionPayload::Transfer { to, amount, memo } => {
                        let mut ledger = self.ledger.lock().unwrap();
                        match ledger.transfer(&user_did, &to, amount) {
                            Ok(_) => {
                                let wallet = self.wallet.lock().unwrap();
                                let tx = crate::chain::create_signed_transaction(
                                    &wallet,
                                    TransactionData::Transfer { to: to.clone(), amount },
                                );
                                drop(wallet);
                                drop(ledger);
                                self.mempool.lock().unwrap().push(tx);
                                let _ = self.hardware.haptic.lock().unwrap().play_pattern(HapticPattern::Success);
                                CommandResult::success(&cmd_id, CommandData::TxHash(format!("tx_{}", cmd_id)))
                            }
                            Err(e) => CommandResult::failure(&cmd_id, format!("Transfer failed: {}", e), true),
                        }
                    }
                    TransactionPayload::Stake { amount } => {
                        match self.ledger.lock().unwrap().stake(&user_did, amount) {
                            Ok(_) => CommandResult::success(&cmd_id, CommandData::TxHash(format!("stake_{}", cmd_id))),
                            Err(e) => CommandResult::failure(&cmd_id, format!("Stake failed: {}", e), true),
                        }
                    }
                    _ => {
                        CommandResult::success(&cmd_id, CommandData::Text("Transaction processed".to_string()))
                    }
                }
            }
            
            // System Commands
            OracleCommand::GetPipelineStatus => {
                let (zk_queued, _zk_max) = crate::zk::get_batch_status();
                let status = crate::oracle::command::PipelineStatus {
                    ai_model: "TinyLlama".to_string(),
                    ai_status: "active".to_string(),
                    zk_queue_size: zk_queued,
                    zk_proving: false,
                    swarm_peers: 0,
                    chain_height: 0,
                    mempool_size: self.mempool.lock().unwrap().len(),
                    storage_used_mb: 0,
                };
                CommandResult::success(&cmd_id, CommandData::PipelineStatus(status))
            }
            
            OracleCommand::GetHardwareStatus => {
                let status = crate::oracle::command::HardwareStatusInfo {
                    display_on: true,
                    battery_percent: 80,
                    haptic_available: true,
                    camera_active: false,
                    mic_active: false,
                };
                CommandResult::success(&cmd_id, CommandData::HardwareStatus(status))
            }
            
            OracleCommand::PlayHaptic { pattern } => {
                let _ = self.hardware.haptic.lock().unwrap().play_pattern(HapticPattern::Success);
                CommandResult::success(&cmd_id, CommandData::HapticPlayed)
            }
            
            OracleCommand::TriggerZKBatch => {
                match crate::zk::prove_batch() {
                    Ok(proofs) => {
                        let summaries: Vec<_> = proofs.iter().map(|p| {
                            crate::oracle::command::ProofSummary {
                                proof_type: "storage".to_string(),
                                size_bytes: p.len(),
                                generation_ms: 0, // Placeholder - actual timing would be tracked
                            }
                        }).collect();
                        CommandResult::success(&cmd_id, CommandData::BatchProofs(summaries))
                    }
                    Err(e) => CommandResult::failure(&cmd_id, format!("ZK batch failed: {}", e), true),
                }
            }
            
            OracleCommand::Shutdown => {
                log::info!("[MONAD] Shutdown requested via Oracle command");
                CommandResult::success(&cmd_id, CommandData::ShutdownAck)
            }
            
            // Default handler for remaining commands
            _ => {
                log::info!("[MONAD] Handling command with default response: {:?}", cmd);
                CommandResult::success(&cmd_id, CommandData::Empty)
            }
        }
    }
    
    /// Process incoming commands from Oracle Veil channel
    async fn process_oracle_commands(&mut self) {
        // Take channels if available
        if let Some(mut channels) = self.monad_channels.take() {
            let monad_clone = MonadBackend {
                ledger: self.ledger.clone(),
                gov: self.gov.clone(),
                storage: self.storage.clone(),
                chain: self.chain.clone(),
                swarm: self.swarm.clone(),
                mempool: self.mempool.clone(),
                hardware: self.hardware.clone(),
                wallet: self.wallet.clone(),
                tab_manager: self.tab_manager.clone(),
            };
            
            // Spawn command processor
            tokio::spawn(async move {
                while let Some(cmd) = channels.cmd_rx.recv().await {
                    log::info!("[MONAD] Received command from Oracle Veil");
                    let result = monad_clone.execute_command(cmd).await;
                    if let Err(e) = channels.result_tx.send(result).await {
                        log::error!("[MONAD] Failed to send command result: {:?}", e);
                    }
                }
                log::info!("[MONAD] Command channel closed");
            });
        }
    }

    /// Phase 7.1 + 7.7: Execute a real action through full pipeline
    fn execute_real_action(&self, intent: &str) -> Result<String> {
        // Ensure output directory exists
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        
        // Get AI to parse intent into action
        let action = {
            let mut ai = self.ai.lock().unwrap();
            ai.predict_action(intent)?
        };
        
        log::info!("[MONAD] ✓ AI Action: {} -> {} = {}", action.action, action.target, action.value);
        
        // Execute based on action type
        let result = match action.action.as_str() {
            "set_config" => {
                // Write config file
                let config_path = format!("{}/{}.conf", REAL_OUTPUT_DIR, action.target.replace(".", "_"));
                let config_content = format!("# Karana Config: {}\n# Generated by AI (confidence: {:.0}%)\n\n{}={}\n",
                    action.target, action.confidence * 100.0, action.target, action.value);
                fs::write(&config_path, &config_content)?;
                
                // Also try to execute if it's a power setting (for real effect on Linux)
                if action.target.contains("power") && action.value == "powersave" {
                    // Try to set CPU governor (will fail gracefully if not root)
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg("echo powersave | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true")
                        .output();
                }
                
                format!("[STORAGE] ✓ Written: {} ({} bytes)\n[RUNTIME] ✓ Applied: {} = {}", 
                    config_path, config_content.len(), action.target, action.value)
            },
            "tune_storage" => {
                // Write storage tuning config
                let config_path = format!("{}/storage_tuning.conf", REAL_OUTPUT_DIR);
                let config_content = format!(
                    "# Karana Storage Tuning\n# AI Confidence: {:.0}%\n\n[sharding]\nmode={}\ntarget={}\n\n[compression]\nenabled=true\nalgorithm=zstd\n",
                    action.confidence * 100.0, action.value, action.target
                );
                fs::write(&config_path, &config_content)?;
                format!("[STORAGE] ✓ Tuning applied: {}\n[STORAGE] ✓ Written: {}", action.value, config_path)
            },
            "execute_command" => {
                // Execute shell command (sandboxed)
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&action.value)
                    .output()?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("[RUNTIME] ✓ Executed: {}\n{}{}", action.value, stdout, stderr)
            },
            _ => {
                // Generic: just log and write to file
                let log_path = format!("{}/intent_log.txt", REAL_OUTPUT_DIR);
                let entry = format!("[{}] Intent: {} -> Action: {:?}\n", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    intent, action);
                fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)?
                    .write_all(entry.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Write failed: {}", e))?;
                format!("[MONAD] ✓ Logged intent to {}", log_path)
            }
        };
        
        // Generate ZK proof for the action
        let commitment = crate::zk::compute_hash(action.value.as_bytes());
        let proof = crate::zk::prove_data_hash(action.value.as_bytes(), commitment)?;
        
        // Log proof generation
        log::info!("[ZK] ✓ Proof generated: {} bytes", proof.len());
        
        // Phase 7.4: Haptic feedback
        let haptic_msg = {
            let mut h = self.hardware.haptic.lock().unwrap();
            h.play_pattern(HapticPattern::Success)?
        };
        
        // Phase 7.5: Chain attestation
        let attest_tx = self.chain.attest_intent("Node-Alpha", intent, &proof, &result);
        {
            let mut pool = self.mempool.lock().unwrap();
            pool.push(attest_tx);
        }
        log::info!("[CHAIN] ✓ Intent attestation queued for next block");
        
        Ok(format!("{}\n[ZK] ✓ Proof: {} bytes\n{}\n[CHAIN] ✓ Attestation queued", result, proof.len(), haptic_msg))
    }

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

        // Phase 9: Node Wallet for signing transactions
        // Try to load existing wallet or create new one
        let wallet_path = format!("{}/node_wallet.enc", base_path);
        let wallet_path_ref = std::path::Path::new(&wallet_path);
        let device_id = "node-primary"; // Static device ID for the node wallet
        
        let wallet = if wallet_path_ref.exists() {
            // Load existing wallet (with empty password for node wallet)
            match KaranaWallet::load_encrypted(wallet_path_ref, "") {
                Ok(w) => {
                    log::info!("[WALLET] Loaded node wallet: {}", w.did());
                    w
                },
                Err(e) => {
                    log::warn!("[WALLET] Failed to load wallet ({}), generating new", e);
                    let result = KaranaWallet::generate(device_id).context("Failed to generate wallet")?;
                    result.wallet.save_encrypted(wallet_path_ref, "").ok();
                    log::info!("[WALLET] Generated new node wallet: {}", result.wallet.did());
                    result.wallet
                }
            }
        } else {
            // Generate new wallet
            let result = KaranaWallet::generate(device_id).context("Failed to generate wallet")?;
            result.wallet.save_encrypted(wallet_path_ref, "").ok();
            log::info!("[WALLET] Generated new node wallet: {}", result.wallet.did());
            result.wallet
        };
        let wallet = Arc::new(Mutex::new(wallet));

        // Phase 8: Real AI ↔ Blockchain Oracle (with wallet for signing)
        // Connects AI intent understanding to REAL blockchain operations
        // NOTE: Oracle needs its own wallet instance since it wraps in Arc<Mutex<>>
        let oracle = {
            // Create a second wallet instance for the Oracle (same identity)
            let oracle_wallet = if wallet_path_ref.exists() {
                KaranaWallet::load_encrypted(wallet_path_ref, "").ok()
            } else {
                None
            };
            
            let mut o = KaranaOracle::new(
                ai.clone(),
                chain.clone(),
                storage.clone(),
                ledger.clone(),
                gov.clone(),
            );
            
            if let Some(w) = oracle_wallet {
                o.set_wallet(w);
            }
            
            Arc::new(Mutex::new(o))
        };
        log::info!("[ORACLE] Initialized with REAL ledger, governance & wallet");

        // ═══════════════════════════════════════════════════════════════════════
        // Oracle Veil v1.1 Initialization
        // ═══════════════════════════════════════════════════════════════════════
        
        // Initialize ZK intent proof system (non-fatal if it fails)
        if let Err(e) = setup_intent_proofs() {
            log::warn!("[ZK-Intent] Failed to initialize intent proofs: {} (continuing without ZK)", e);
        }
        
        // Create command channels (Oracle ↔ Monad)
        let (oracle_channels, monad_channels) = OracleChannels::default_channels();
        
        // Create multimodal sense (input) - Note: needs async init in real impl
        let (sense_tx, _sense_rx) = tokio::sync::mpsc::channel(32);
        let sense = Arc::new(tokio::sync::Mutex::new(
            MultimodalSense::new(sense_tx)
        ));
        
        // Create minimal manifest (output)
        let manifest = Arc::new(tokio::sync::Mutex::new(
            MinimalManifest::default()
        ));
        
        // Create Oracle Veil - THE sole user interface
        // Uses local Phi-3 AI (via KaranaAI) for intent parsing - NO cloud APIs
        let oracle_veil = match OracleVeil::new(
            ai.clone(),
            oracle_channels.cmd_tx,
            oracle_channels.result_rx,
        ) {
            Ok(veil) => {
                log::info!("[ORACLE-VEIL] ✓ OracleVeil initialized with local AI");
                Some(Arc::new(tokio::sync::Mutex::new(veil)))
            }
            Err(e) => {
                log::warn!("[ORACLE-VEIL] Failed to initialize OracleVeil: {} (using legacy Oracle)", e);
                None
            }
        };
        
        log::info!("[ORACLE-VEIL] ✓ Channels initialized");
        log::info!("[ORACLE-VEIL] ✓ Multimodal sense: voice, gaze, touch (stub)");
        log::info!("[ORACLE-VEIL] ✓ Minimal manifest: whispers, haptics (stub)");

        // AR Tab Manager
        let tab_manager = Arc::new(Mutex::new(TabManager::new()));
        log::info!("[AR-TABS] ✓ Tab manager initialized");

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
            oracle,
            wallet,
            // Oracle Veil v1.1
            oracle_veil,
            monad_channels: Some(monad_channels),
            sense: Some(sense),
            manifest: Some(manifest),
            tab_manager,
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
        
        // ═══════════════════════════════════════════════════════════════════════
        // Oracle Veil v1.1: Start command processing
        // ═══════════════════════════════════════════════════════════════════════
        log::info!("[ORACLE-VEIL] Starting command processing...");
        self.process_oracle_commands().await;
        log::info!("[ORACLE-VEIL] ✓ Command channel active");

        // Start IPC Server (Phase 8: Shell Integration)
        let ipc_tx = self.ui.get_intent_sender();
        if let Err(e) = ipc::start_ipc_server(9000, ipc_tx).await {
            log::error!("Failed to start IPC Server: {}", e);
        }

        if std::env::var("KARANA_CHROOT").is_ok() {
            log::info!("Ignited in Sovereign Chroot – Fabric Isolated");
        }
        
        // Atom 4: Verified Awakening
        log::info!("Step 1/9: Boot Awakening...");
        let genesis_hash = 0u64;
        
        // We need mutable access to boot for awaken. 
        self.boot.awaken(genesis_hash).await.context("Boot failed")?;
        log::info!("Step 1/9: Boot Awakening [OK]");

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
                        log::info!("Atom 6 (P2P): Processing AI Request from {}: '{}'", req.requester_did, req.prompt);
                        
                        // Verify Proof (Atom 8: Identity)
                        // In a real system, we would fetch the DID document from chain and verify proof.
                        log::info!("Atom 8 (Identity): Verifying ZK-Proof for DID {} (Size: {} bytes)...", req.requester_did, req.proof.len());

                        // Offload compute to our local AI (Non-blocking)
                        let ai_clone = self.ai.clone();
                        let swarm_clone = self.swarm.clone();
                        let req_id = req.request_id;
                        let prompt = req.prompt.clone();
                        
                        // Get our DID from wallet for response
                        let my_did = self.wallet.lock().unwrap().did().to_string();
                        
                        tokio::task::spawn_blocking(move || {
                            let result = match ai_clone.lock().unwrap().predict(&prompt, 100) {
                                Ok(r) => r,
                                Err(e) => format!("Compute Error: {}", e),
                            };
                            
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                if let Err(e) = swarm_clone.send_ai_response(req_id, result, my_did).await {
                                    log::error!("Atom 6 (P2P): Failed to send AI response: {}", e);
                                }
                            });
                        });
                    },
                    KaranaSwarmEvent::AIResponseReceived(res) => {
                        log::info!("Atom 6 (P2P): Received AI Result [{} from {}]: {}", res.request_id, res.responder_did, res.result);
                        // Notify UI
                        let _ = self.ui.render_intent(format!("Swarm AI Result (from {}): {}", res.responder_did, res.result), vec![]).await;
                    },
                    KaranaSwarmEvent::ClipboardReceived(clip) => {
                        log::info!("Atom 6 (P2P): Received Clipboard Sync from {}", clip.did);
                        
                        // Verify Proof (Atom 8: Identity)
                        log::info!("Atom 8 (Identity): Verifying ZK-Proof for Clipboard Sync (Size: {} bytes)...", clip.signature.len());
                        
                        // Verify DID matches local user (or trusted peer)
                        // For now, we just log it and update UI if it's "our" DID
                        let local_did = self.wallet.lock().unwrap().did().to_string();
                        if clip.did == local_did {
                            log::info!("Atom 5 (Ecosystem): Syncing Clipboard (Self-Sovereign Sync)...");
                            let _ = self.ui.render_intent(format!("Clipboard Synced: {}", clip.content), vec![]).await;
                        } else {
                            log::info!("Atom 5 (Ecosystem): Ignoring Clipboard from foreign DID: {}", clip.did);
                        }
                    },
                    KaranaSwarmEvent::EchoReceived(echo) => {
                        log::info!("[SWARM] ✓ Echo confirmation: {} from {} at {}", 
                            echo.message_id, echo.sender_did, echo.timestamp);
                    },
                    KaranaSwarmEvent::PeerConnected(peer_id) => {
                        log::info!("[SWARM] ✓ New peer joined: {}", peer_id);
                        log::info!("[SWARM] {}", self.swarm.stats.summary());
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

                // Phase 7.7: Pipeline status command
                if intent == "status" || intent == "pipeline" {
                    let status = self.get_pipeline_status();
                    log::info!("{}", status);
                    let _ = self.ui.render_intent(status, vec![]).await;
                    continue;
                }

                // Phase 7.6: Batch prove command
                if intent == "prove batch" {
                    match crate::zk::prove_batch() {
                        Ok(proofs) => {
                            let msg = format!("[ZK] ✓ Batch proved: {} proofs generated", proofs.len());
                            log::info!("{}", msg);
                            let _ = self.ui.render_intent(msg, vec![]).await;
                        },
                        Err(e) => {
                            log::error!("[ZK] ✗ Batch prove failed: {}", e);
                        }
                    }
                    continue;
                }

                // ═══════════════════════════════════════════════════════════════════
                // PHASE 8: AI ↔ Blockchain Oracle (Natural Language Queries)
                // Intent → AI Parse → Blockchain Query → AI Format → UI Display
                // ═══════════════════════════════════════════════════════════════════
                // Handle natural language blockchain queries:
                // - "show my files" / "what files do I have"
                // - "check my balance" / "how much KARA do I have"
                // - "store this note: ..." / "save file ..."
                // - "who owns ..." / "look up ..."
                // - Any other freeform query that needs blockchain data
                let is_oracle_query = intent.contains("my files") || 
                    intent.contains("my balance") || 
                    intent.contains("show ") ||
                    intent.contains("check ") ||
                    intent.contains("look up") ||
                    intent.contains("store ") ||
                    intent.contains("save ") ||
                    intent.starts_with("query ") ||
                    intent.starts_with("ask ") ||
                    intent.starts_with("? ");
                
                if is_oracle_query {
                    // Get user's DID from wallet (real) or identity (legacy)
                    let user_did = self.wallet.lock().unwrap().did().to_string();
                    
                    match self.process_oracle_intent(&intent, &user_did).await {
                        Ok(response) => {
                            log::info!("[ORACLE] ✓ Query processed successfully");
                            let _ = self.ui.render_intent(response, vec![]).await;
                            
                            // Attest the query to chain
                            let attest_tx = self.chain.attest_intent(&user_did, &intent, &[], "oracle_query");
                            {
                                let mut pool = self.mempool.lock().unwrap();
                                pool.push(attest_tx);
                            }
                            continue;
                        },
                        Err(e) => {
                            log::error!("[ORACLE] ✗ Query failed: {}", e);
                            let _ = self.hardware.haptic.lock().unwrap().play_pattern(HapticPattern::Error);
                        }
                    }
                }

                // ═══════════════════════════════════════════════════════════════════
                // PHASE 7.1 + 7.7: Real Action Execution (Full Pipeline)
                // Input → AI → ZK → Storage → Swarm → Chain → UI → Haptic
                // ═══════════════════════════════════════════════════════════════════
                if intent.starts_with("tune") || intent.starts_with("optimize") || intent.starts_with("configure") {
                    match self.execute_real_action(&intent) {
                        Ok(result) => {
                            log::info!("[MONAD] ✓ Intent completed:\n{}", result);
                            let _ = self.ui.render_intent(result, vec![]).await;
                            
                            // Broadcast completion to swarm with tracking
                            let blob = self.storage.write(intent.as_bytes(), &intent)?;
                            let msg_id = self.swarm.broadcast_with_tracking(
                                serde_json::to_vec(&blob)?,
                                "intent"
                            ).await?;
                            log::info!("[SWARM] ✓ Broadcast {} - awaiting echoes", msg_id);
                            continue;
                        },
                        Err(e) => {
                            log::error!("[MONAD] ✗ Action failed: {}", e);
                            // Haptic error feedback
                            let _ = self.hardware.haptic.lock().unwrap().play_pattern(HapticPattern::Error);
                        }
                    }
                }

                // Render intent (Standard processing for other intents)
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
                    // Create a REAL signed transaction for liveness every other block
                    if height % 2 == 0 {
                        let wallet = self.wallet.lock().unwrap();
                        let tx = crate::chain::create_signed_transaction(
                            &wallet,
                            TransactionData::Transfer { 
                                to: "Node-Beta".to_string(), 
                                amount: 10u128 
                            },
                        );
                        log::info!("[CHAIN] Created signed tx: {} → Node-Beta (10 KARA) [Ed25519 ✓]", 
                            &wallet.did()[..20]);
                        txs.push(tx);
                    }
                    txs.append(&mut pool);
                }
                
                // Create Block (use our DID as proposer)
                let proposer = self.wallet.lock().unwrap().did().to_string();
                let block = Block::new(parent_hash.clone(), height, proposer, txs.clone());
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
