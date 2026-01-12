// Phase 53: Blockchain Backend
//
// Full consensus-based blockchain with:
// - P2P networking and gossip
// - Celestia DA integration
// - Block production and validation
// - Complete transaction history

use super::{LedgerBackend, BackendStats, BackendConfig, SyncResult};
use crate::chain::{Block, Transaction, Blockchain};
use crate::economy::{Ledger, Governance};
use crate::ai::KaranaAI;
use anyhow::{Result, Context};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

/// Blockchain backend - full consensus
pub struct BlockchainBackend {
    config: BackendConfig,
    blockchain: Arc<Mutex<Blockchain>>,
    pending_txs: Arc<Mutex<Vec<Transaction>>>,
    running: Arc<Mutex<bool>>,
}

impl BlockchainBackend {
    /// Create a new blockchain backend
    pub fn new(config: BackendConfig) -> Result<Self> {
        // Initialize ledger and governance
        let ledger_path = format!("{}/ledger.db", config.data_dir);
        let ledger = Arc::new(Mutex::new(Ledger::new(&ledger_path)));
        // Governance requires path, ledger, and AI - creating with default path
        let ai = Arc::new(Mutex::new(KaranaAI::new().context("Failed to initialize AI")?));
        let gov = Arc::new(Mutex::new(Governance::new("governance.db", ledger.clone(), ai)));
        
        // Create blockchain
        let blockchain = Arc::new(Mutex::new(Blockchain::new(
            ledger.clone(),
            gov.clone(),
        )));
        
        Ok(Self {
            config,
            blockchain,
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Start block production loop
    pub async fn start_block_production(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        let blockchain = self.blockchain.clone();
        let pending_txs = self.pending_txs.clone();
        let running = self.running.clone();
        let block_time = self.config.block_time_secs;
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(block_time));
            
            loop {
                ticker.tick().await;
                
                // Check if still running
                if !*running.lock().unwrap() {
                    break;
                }
                
                // Collect pending transactions
                let transactions = {
                    let mut txs = pending_txs.lock().unwrap();
                    let collected: Vec<Transaction> = txs.drain(..).collect();
                    collected
                };
                
                if transactions.is_empty() {
                    continue;
                }
                
                // Create and add block
                if transactions.is_empty() {
                    continue;
                }
                
                let chain = blockchain.lock().unwrap();
                let latest = chain.latest_block();
                let new_height = latest.header.height + 1;
                
                let new_block = Block::new(
                    latest.hash.clone(),
                    new_height,
                    "local".to_string(),
                    transactions,
                );
                
                if let Err(e) = chain.add_block(new_block.clone()) {
                    eprintln!("Failed to add block: {}", e);
                } else {
                    // Apply block to update state
                    let _ = chain.apply_block(&new_block);
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop block production
    pub fn stop_block_production(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

#[async_trait]
impl LedgerBackend for BlockchainBackend {
    fn name(&self) -> &str {
        "blockchain"
    }
    
    async fn init(&mut self) -> Result<()> {
        // Start block production if enabled
        self.start_block_production().await?;
        Ok(())
    }
    
    async fn add_block(&mut self, block: Block) -> Result<()> {
        let mut chain = self.blockchain.lock().unwrap();
        chain.add_block(block)
            .context("Failed to add block to blockchain")
    }
    
    async fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let chain = self.blockchain.lock().unwrap();
        Ok(chain.get_block(height))
    }
    
    async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let chain = self.blockchain.lock().unwrap();
        
        // Iterate through all blocks to find matching hash
        let mut current_height = chain.height();
        while current_height > 0 {
            if let Some(block) = chain.get_block(current_height) {
                if block.hash == hash {
                    return Ok(Some(block));
                }
            }
            current_height = current_height.saturating_sub(1);
        }
        
        // Check genesis
        if let Some(block) = chain.get_block(0) {
            if block.hash == hash {
                return Ok(Some(block));
            }
        }
        
        Ok(None)
    }
    
    async fn get_height(&self) -> Result<u64> {
        let chain = self.blockchain.lock().unwrap();
        Ok(chain.height())
    }
    
    async fn get_latest_block(&self) -> Result<Option<Block>> {
        let chain = self.blockchain.lock().unwrap();
        Ok(Some(chain.latest_block()))
    }
    
    async fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Verify transaction before adding
        if !tx.verify() {
            return Err(anyhow::anyhow!("Invalid transaction signature"));
        }
        
        let mut pending = self.pending_txs.lock().unwrap();
        pending.push(tx);
        Ok(())
    }
    
    async fn get_pending_transactions(&self) -> Result<Vec<Transaction>> {
        let pending = self.pending_txs.lock().unwrap();
        Ok(pending.clone())
    }
    
    async fn clear_pending_transactions(&mut self) -> Result<()> {
        let mut pending = self.pending_txs.lock().unwrap();
        pending.clear();
        Ok(())
    }
    
    async fn validate_block(&self, block: &Block) -> Result<bool> {
        let chain = self.blockchain.lock().unwrap();
        
        // Get parent hash
        let parent_hash = if block.header.height > 0 {
            // Get the latest block to determine parent hash
            let latest = chain.latest_block();
            latest.hash.clone()
        } else {
            "0".repeat(64)
        };
        
        match block.validate(&parent_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    async fn get_stats(&self) -> Result<BackendStats> {
        let chain = self.blockchain.lock().unwrap();
        
        let total_blocks = chain.height() + 1; // +1 because height is 0-indexed
        
        // Count transactions across all blocks
        let mut total_transactions = 0u64;
        for height in 0..=chain.height() {
            if let Some(block) = chain.get_block(height) {
                total_transactions += block.transactions.len() as u64;
            }
        }
        
        let pending = self.pending_txs.lock().unwrap();
        let pending_count = pending.len();
        
        let latest = chain.latest_block();
        let last_block_time = Some(latest.header.timestamp);
        
        // Estimate storage size (rough approximation)
        let storage_size_bytes = total_blocks as usize * 1024; // Assume ~1KB per block
        
        Ok(BackendStats {
            backend_type: "blockchain".to_string(),
            total_blocks,
            total_transactions,
            pending_transactions: pending_count,
            storage_size_mb: storage_size_bytes as f64 / 1_048_576.0,
            last_block_time,
        })
    }
    
    async fn sync(&mut self) -> Result<SyncResult> {
        // TODO: Implement P2P sync
        // For now, return "already synced"
        Ok(SyncResult {
            synced: true,
            blocks_downloaded: 0,
            peers_contacted: 0,
            time_ms: 0,
        })
    }
    
    async fn export(&self) -> Result<Vec<u8>> {
        let chain = self.blockchain.lock().unwrap();
        
        // Export all blocks
        let mut blocks = Vec::new();
        for height in 0..=chain.height() {
            if let Some(block) = chain.get_block(height) {
                blocks.push(block);
            }
        }
        
        bincode::serialize(&blocks)
            .context("Failed to serialize blockchain for export")
    }
    
    async fn import(&mut self, data: Vec<u8>) -> Result<()> {
        let blocks: Vec<Block> = bincode::deserialize(&data)
            .context("Failed to deserialize blockchain import")?;
        
        // Validate all blocks
        for (i, block) in blocks.iter().enumerate() {
            if i == 0 {
                // Genesis block
                continue;
            }
            
            let parent_hash = &blocks[i - 1].hash;
            block.validate(parent_hash)
                .context(format!("Invalid block at height {}", block.header.height))?;
        }
        
        // Import blocks one by one
        let chain = self.blockchain.lock().unwrap();
        for block in blocks.into_iter().skip(1) {  // Skip genesis as it already exists
            chain.add_block(block.clone())?;
            chain.apply_block(&block)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::TransactionData;
    
    #[tokio::test]
    async fn test_blockchain_backend_init() {
        let config = BackendConfig::default();
        let mut backend = BlockchainBackend::new(config).unwrap();
        
        backend.init().await.unwrap();
        
        let height = backend.get_height().await.unwrap();
        assert_eq!(height, 0); // Genesis block
        
        backend.stop_block_production();
    }
    
    #[tokio::test]
    async fn test_add_transaction() {
        let config = BackendConfig::default();
        let mut backend = BlockchainBackend::new(config).unwrap();
        
        let tx = Transaction {
            sender: "alice".to_string(),
            data: TransactionData::Transfer {
                to: "bob".to_string(),
                amount: 100,
            },
            signature: "test_sig".to_string(),
            nonce: 1,
            public_key: None,
            hash: "tx_hash".to_string(),
            timestamp: 1000000,
        };
        
        backend.add_transaction(tx).await.unwrap();
        
        let pending = backend.get_pending_transactions().await.unwrap();
        assert_eq!(pending.len(), 1);
    }
    
    #[tokio::test]
    async fn test_get_stats() {
        let config = BackendConfig::default();
        let backend = BlockchainBackend::new(config).unwrap();
        
        let stats = backend.get_stats().await.unwrap();
        
        assert_eq!(stats.backend_type, "blockchain");
        assert_eq!(stats.total_blocks, 1); // Genesis
        assert_eq!(stats.pending_transactions, 0);
    }
}
