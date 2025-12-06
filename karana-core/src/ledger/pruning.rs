// Phase 53: Ledger Pruning System
//
// Automatically prunes old blocks while:
// 1. Keeping high-value transactions (payments, governance)
// 2. Maintaining cryptographic chain integrity
// 3. Creating compressed summaries
// 4. Supporting cold storage archival

use super::{PruningConfig, DataCategory, LedgerStats};
use crate::chain::{Block, Transaction, TransactionData};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;

/// Manages pruning of old blockchain data
pub struct PruningManager {
    config: PruningConfig,
    cold_storage_path: Option<PathBuf>,
    pruned_blocks: HashSet<u64>,
}

/// Summary of a pruned block that maintains chain integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunedBlockSummary {
    /// Block height
    pub height: u64,
    
    /// Block hash (unchanged)
    pub hash: String,
    
    /// Parent hash (unchanged)
    pub parent_hash: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Validator
    pub validator: String,
    
    /// Number of transactions in original block
    pub tx_count: usize,
    
    /// Merkle root of all transaction hashes
    pub tx_merkle_root: String,
    
    /// High-value transactions kept in full
    pub kept_transactions: Vec<Transaction>,
    
    /// Hashes of pruned transactions (for verification)
    pub pruned_tx_hashes: Vec<String>,
    
    /// Statistics
    pub original_size_bytes: usize,
    pub compressed_size_bytes: usize,
}

impl PruningManager {
    /// Create a new pruning manager
    pub fn new(config: PruningConfig, cold_storage_path: Option<PathBuf>) -> Self {
        Self {
            config,
            cold_storage_path,
            pruned_blocks: HashSet::new(),
        }
    }
    
    /// Check if a block should be pruned
    pub fn should_prune(&self, block: &Block, current_height: u64) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        // Don't prune recent blocks
        if block.header.height > current_height.saturating_sub(self.config.min_blocks_to_keep) {
            return false;
        }
        
        // Check age
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age_days = (current_time - block.header.timestamp) / 86400;
        
        age_days > self.config.retention_days as u64
    }
    
    /// Prune a block, returning a compressed summary
    pub fn prune_block(&mut self, block: &Block) -> Result<PrunedBlockSummary> {
        // Separate transactions into kept vs pruned
        let (kept, pruned): (Vec<_>, Vec<_>) = block.transactions.iter()
            .partition(|tx| self.should_keep_transaction(tx));
        
        // Calculate merkle root of all transaction hashes
        let tx_hashes: Vec<String> = block.transactions.iter()
            .map(|tx| tx.hash.clone())
            .collect();
        let merkle_root = self.calculate_merkle_root(&tx_hashes);
        
        // Get pruned transaction hashes
        let pruned_tx_hashes: Vec<String> = pruned.iter()
            .map(|tx| tx.hash.clone())
            .collect();
        
        // Calculate sizes
        let original_size = bincode::serialize(&block)
            .context("Failed to serialize block")?
            .len();
        
        let summary = PrunedBlockSummary {
            height: block.header.height,
            hash: block.hash.clone(),
            parent_hash: block.header.parent_hash.clone(),
            timestamp: block.header.timestamp,
            validator: block.header.validator.clone(),
            tx_count: block.transactions.len(),
            tx_merkle_root: merkle_root,
            kept_transactions: kept.into_iter().cloned().collect(),
            pruned_tx_hashes,
            original_size_bytes: original_size,
            compressed_size_bytes: 0, // Will be updated after compression
        };
        
        let compressed_size = bincode::serialize(&summary)
            .context("Failed to serialize summary")?
            .len();
        
        let mut summary = summary;
        summary.compressed_size_bytes = compressed_size;
        
        // Mark as pruned
        self.pruned_blocks.insert(block.header.height);
        
        Ok(summary)
    }
    
    /// Check if a transaction should be kept (not pruned)
    fn should_keep_transaction(&self, tx: &Transaction) -> bool {
        for category in &self.config.keep_categories {
            match (category, &tx.data) {
                (DataCategory::Payments, TransactionData::Transfer { .. }) => return true,
                (DataCategory::Governance, TransactionData::Propose { .. }) => return true,
                (DataCategory::Governance, TransactionData::Vote { .. }) => return true,
                (DataCategory::HighValueIntents, TransactionData::IntentAttestation { .. }) => {
                    // Keep high-value intents (this could be more sophisticated)
                    return true;
                }
                _ => {}
            }
        }
        false
    }
    
    /// Calculate merkle root of transaction hashes
    fn calculate_merkle_root(&self, hashes: &[String]) -> String {
        if hashes.is_empty() {
            return hex::encode(Sha256::digest(b"empty"));
        }
        
        if hashes.len() == 1 {
            return hashes[0].clone();
        }
        
        // Simple merkle tree implementation
        let mut current_level: Vec<String> = hashes.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    chunk[0].clone()
                };
                
                let hash = hex::encode(Sha256::digest(combined.as_bytes()));
                next_level.push(hash);
            }
            
            current_level = next_level;
        }
        
        current_level[0].clone()
    }
    
    /// Archive a block to cold storage
    pub async fn archive_block(&self, block: &Block) -> Result<()> {
        if let Some(ref cold_path) = self.cold_storage_path {
            fs::create_dir_all(cold_path).await
                .context("Failed to create cold storage directory")?;
            
            let block_path = cold_path.join(format!("block_{:08}.bin", block.header.height));
            
            let data = bincode::serialize(&block)
                .context("Failed to serialize block for archival")?;
            
            // Compress with zstd
            let compressed = zstd::encode_all(&data[..], 3)
                .context("Failed to compress block")?;
            
            fs::write(&block_path, compressed).await
                .context("Failed to write archived block")?;
        }
        
        Ok(())
    }
    
    /// Restore a block from cold storage
    pub async fn restore_block(&self, height: u64) -> Result<Block> {
        let cold_path = self.cold_storage_path.as_ref()
            .context("Cold storage not configured")?;
        
        let block_path = cold_path.join(format!("block_{:08}.bin", height));
        
        let compressed = fs::read(&block_path).await
            .context("Failed to read archived block")?;
        
        let data = zstd::decode_all(&compressed[..])
            .context("Failed to decompress block")?;
        
        let block: Block = bincode::deserialize(&data)
            .context("Failed to deserialize block")?;
        
        Ok(block)
    }
    
    /// Get pruning statistics
    pub fn get_stats(&self, total_blocks: u64) -> PruningStats {
        PruningStats {
            total_blocks,
            pruned_blocks: self.pruned_blocks.len() as u64,
            active_blocks: total_blocks - self.pruned_blocks.len() as u64,
            pruning_enabled: self.config.enabled,
            retention_days: self.config.retention_days,
        }
    }
    
    /// Check if a block has been pruned
    pub fn is_pruned(&self, height: u64) -> bool {
        self.pruned_blocks.contains(&height)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningStats {
    pub total_blocks: u64,
    pub pruned_blocks: u64,
    pub active_blocks: u64,
    pub pruning_enabled: bool,
    pub retention_days: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{BlockHeader, TransactionData};
    
    fn create_test_block(height: u64, timestamp: u64) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: "0".repeat(64),
                height,
                timestamp,
                state_root: String::new(),
                validator: "test".to_string(),
            },
            transactions: vec![
                Transaction {
                    sender: "alice".to_string(),
                    data: TransactionData::Transfer {
                        to: "bob".to_string(),
                        amount: 100,
                    },
                    signature: "sig1".to_string(),
                    nonce: 1,
                    public_key: None,
                    hash: "tx1".to_string(),
                    timestamp,
                },
            ],
            hash: "block_hash".to_string(),
        }
    }
    
    #[test]
    fn test_should_prune_recent_block() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 1000,
            keep_categories: vec![],
        };
        
        let manager = PruningManager::new(config, None);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let block = create_test_block(1000, current_time - 100);
        
        // Recent block should not be pruned
        assert!(!manager.should_prune(&block, 1100));
    }
    
    #[test]
    fn test_should_prune_old_block() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 100,
            keep_categories: vec![],
        };
        
        let manager = PruningManager::new(config, None);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Block from 100 days ago
        let old_time = current_time - (100 * 86400);
        let block = create_test_block(100, old_time);
        
        // Old block should be pruned
        assert!(manager.should_prune(&block, 10000));
    }
    
    #[test]
    fn test_prune_block() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 1000,
            keep_categories: vec![DataCategory::Payments],
        };
        
        let mut manager = PruningManager::new(config, None);
        let block = create_test_block(1, 1000000);
        
        let summary = manager.prune_block(&block).unwrap();
        
        assert_eq!(summary.height, 1);
        assert_eq!(summary.tx_count, 1);
        assert_eq!(summary.kept_transactions.len(), 1); // Payment kept
        assert!(summary.compressed_size_bytes < summary.original_size_bytes);
    }
    
    #[test]
    fn test_merkle_root_empty() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 1000,
            keep_categories: vec![],
        };
        
        let manager = PruningManager::new(config, None);
        let root = manager.calculate_merkle_root(&[]);
        assert!(!root.is_empty());
    }
    
    #[test]
    fn test_merkle_root_single() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 1000,
            keep_categories: vec![],
        };
        
        let manager = PruningManager::new(config, None);
        let root = manager.calculate_merkle_root(&["hash1".to_string()]);
        assert_eq!(root, "hash1");
    }
    
    #[test]
    fn test_merkle_root_multiple() {
        let config = PruningConfig {
            enabled: true,
            retention_days: 90,
            min_blocks_to_keep: 1000,
            keep_categories: vec![],
        };
        
        let manager = PruningManager::new(config, None);
        let root = manager.calculate_merkle_root(&[
            "hash1".to_string(),
            "hash2".to_string(),
            "hash3".to_string(),
        ]);
        assert!(!root.is_empty());
        assert_ne!(root, "hash1");
    }
}
