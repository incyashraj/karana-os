// Phase 53: Storage Optimizer
//
// Monitors and enforces storage quotas:
// 1. Automatic cleanup when approaching limits
// 2. Compression of old data
// 3. Cold storage archival
// 4. Storage usage reporting

use super::{LedgerConfig, LedgerStats};
use crate::ledger::pruning::{PruningManager, PrunedBlockSummary};
use crate::ledger::checkpointing::{CheckpointManager, Checkpoint};
use crate::chain::Block;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use tokio::fs;

/// Manages storage optimization
pub struct StorageOptimizer {
    config: LedgerConfig,
    pruning: PruningManager,
    checkpointing: CheckpointManager,
    storage_dir: PathBuf,
}

impl StorageOptimizer {
    /// Create a new storage optimizer
    pub fn new(config: LedgerConfig, storage_dir: PathBuf) -> Self {
        let pruning = PruningManager::new(
            config.pruning.clone(),
            config.optimization.cold_storage_path.as_ref().map(PathBuf::from),
        );
        
        let checkpoint_dir = storage_dir.join("checkpoints");
        let checkpointing = CheckpointManager::new(
            config.checkpointing.clone(),
            checkpoint_dir,
        );
        
        Self {
            config,
            pruning,
            checkpointing,
            storage_dir,
        }
    }
    
    /// Check if storage quota is exceeded
    pub async fn is_over_quota(&self) -> Result<bool> {
        let usage = self.get_storage_usage().await?;
        let quota_bytes = self.config.optimization.max_storage_mb * 1_048_576;
        
        Ok(usage > quota_bytes)
    }
    
    /// Get current storage usage in bytes
    pub async fn get_storage_usage(&self) -> Result<u64> {
        let mut total = 0u64;
        
        let mut entries = fs::read_dir(&self.storage_dir).await
            .context("Failed to read storage directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                total += metadata.len();
            }
        }
        
        Ok(total)
    }
    
    /// Optimize storage by pruning, checkpointing, and compression
    pub async fn optimize(
        &mut self,
        blocks: &mut Vec<Block>,
        current_height: u64,
    ) -> Result<OptimizationResult> {
        let start_time = std::time::Instant::now();
        let initial_usage = self.get_storage_usage().await?;
        
        let mut result = OptimizationResult {
            blocks_pruned: 0,
            blocks_compressed: 0,
            checkpoints_created: 0,
            bytes_freed: 0,
            duration_ms: 0,
        };
        
        // Step 1: Create checkpoint if needed
        if self.checkpointing.should_create_checkpoint(current_height) {
            let state = self.extract_state_snapshot(blocks)?;
            let _checkpoint = self.checkpointing.create_checkpoint(blocks, state).await?;
            result.checkpoints_created += 1;
        }
        
        // Step 2: Prune old blocks
        let blocks_to_prune: Vec<(usize, Block)> = blocks.iter()
            .enumerate()
            .filter(|(_, b)| self.pruning.should_prune(b, current_height))
            .map(|(i, b)| (i, b.clone()))
            .collect();
        
        for (_idx, block) in &blocks_to_prune {
            // Archive to cold storage if configured
            if let Err(e) = self.pruning.archive_block(block).await {
                eprintln!("Failed to archive block {}: {}", block.header.height, e);
            }
            
            // Create pruned summary
            let _summary = self.pruning.prune_block(block)?;
            result.blocks_pruned += 1;
        }
        
        // Remove pruned blocks from active storage
        blocks.retain(|b| !self.pruning.is_pruned(b.header.height));
        
        // Step 3: Compress old blocks if enabled
        if self.config.optimization.compress_old_blocks {
            let compress_result = self.compress_old_blocks(blocks).await?;
            result.blocks_compressed = compress_result;
        }
        
        // Step 4: Check if still over quota
        let final_usage = self.get_storage_usage().await?;
        result.bytes_freed = initial_usage.saturating_sub(final_usage);
        result.duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Step 5: If still over quota, be more aggressive
        if self.is_over_quota().await? {
            self.aggressive_cleanup(blocks).await?;
        }
        
        Ok(result)
    }
    
    /// Extract state snapshot from blocks
    fn extract_state_snapshot(&self, blocks: &[Block]) -> Result<crate::ledger::checkpointing::StateSnapshot> {
        use std::collections::HashMap;
        use crate::chain::TransactionData;
        
        let mut balances = HashMap::new();
        let mut nonces = HashMap::new();
        
        // Replay all transactions to build state
        for block in blocks {
            for tx in &block.transactions {
                match &tx.data {
                    TransactionData::Transfer { to, amount } => {
                        // Deduct from sender
                        let sender_balance = balances.entry(tx.sender.clone()).or_insert(0u128);
                        *sender_balance = sender_balance.saturating_sub(*amount);
                        
                        // Add to recipient
                        let recipient_balance = balances.entry(to.clone()).or_insert(0u128);
                        *recipient_balance += amount;
                    }
                    _ => {}
                }
                
                // Update nonce
                let nonce = nonces.entry(tx.sender.clone()).or_insert(0u64);
                *nonce = (*nonce).max(tx.nonce);
            }
        }
        
        let total_supply = balances.values().sum();
        
        Ok(crate::ledger::checkpointing::StateSnapshot {
            balances,
            nonces,
            proposals: HashMap::new(),
            stakes: HashMap::new(),
            total_supply,
        })
    }
    
    /// Compress old blocks
    async fn compress_old_blocks(&self, blocks: &[Block]) -> Result<usize> {
        let mut compressed_count = 0;
        
        let compress_dir = self.storage_dir.join("compressed");
        fs::create_dir_all(&compress_dir).await
            .context("Failed to create compressed directory")?;
        
        // Compress blocks older than 30 days
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        for block in blocks {
            let age_days = (current_time - block.header.timestamp) / 86400;
            
            if age_days > 30 {
                let block_path = compress_dir.join(format!("block_{:08}.zst", block.header.height));
                
                // Skip if already compressed
                if block_path.exists() {
                    continue;
                }
                
                let data = bincode::serialize(block)
                    .context("Failed to serialize block for compression")?;
                
                let compressed = zstd::encode_all(&data[..], 3)
                    .context("Failed to compress block")?;
                
                fs::write(&block_path, compressed).await
                    .context("Failed to write compressed block")?;
                
                compressed_count += 1;
            }
        }
        
        Ok(compressed_count)
    }
    
    /// Aggressive cleanup when over quota
    async fn aggressive_cleanup(&mut self, blocks: &mut Vec<Block>) -> Result<()> {
        // Remove all but essential blocks
        let essential_height = blocks.last()
            .map(|b| b.header.height)
            .unwrap_or(0)
            .saturating_sub(100);
        
        blocks.retain(|b| b.header.height >= essential_height);
        
        Ok(())
    }
    
    /// Get comprehensive storage statistics
    pub async fn get_stats(&self) -> Result<LedgerStats> {
        let usage_bytes = self.get_storage_usage().await?;
        let quota_bytes = self.config.optimization.max_storage_mb * 1_048_576;
        
        let pruning_stats = self.pruning.get_stats(0); // Would need total blocks
        let checkpoint_stats = self.checkpointing.get_stats();
        
        Ok(LedgerStats {
            total_blocks: pruning_stats.total_blocks,
            pruned_blocks: pruning_stats.pruned_blocks,
            active_blocks: pruning_stats.active_blocks,
            total_size_mb: usage_bytes as f64 / 1_048_576.0,
            compressed_size_mb: usage_bytes as f64 / 1_048_576.0, // Approximation
            checkpoint_count: checkpoint_stats.checkpoint_count,
            last_pruned: None,
            last_checkpoint: checkpoint_stats.latest_checkpoint_height,
        })
    }
    
    /// Run automatic optimization loop
    pub async fn auto_optimize_loop(
        &mut self,
        blocks: &mut Vec<Block>,
    ) -> Result<()> {
        use tokio::time::{interval, Duration};
        
        let mut ticker = interval(Duration::from_secs(3600)); // Every hour
        
        loop {
            ticker.tick().await;
            
            let current_height = blocks.last()
                .map(|b| b.header.height)
                .unwrap_or(0);
            
            if let Err(e) = self.optimize(blocks, current_height).await {
                eprintln!("Optimization failed: {}", e);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub blocks_pruned: usize,
    pub blocks_compressed: usize,
    pub checkpoints_created: usize,
    pub bytes_freed: u64,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{BlockHeader, Transaction, TransactionData};
    
    fn create_test_block(height: u64, timestamp: u64) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: "0".repeat(64),
                height,
                timestamp,
                state_root: String::new(),
                validator: "test".to_string(),
            },
            transactions: vec![],
            hash: format!("block_{}", height),
        }
    }
    
    #[tokio::test]
    async fn test_storage_optimizer_creation() {
        let config = LedgerConfig::default();
        let temp_dir = std::env::temp_dir().join("karana_optimizer_test");
        
        let _optimizer = StorageOptimizer::new(config, temp_dir.clone());
        
        // Cleanup
        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
    
    #[tokio::test]
    async fn test_optimize() {
        let config = LedgerConfig::minimal();
        let temp_dir = std::env::temp_dir().join("karana_optimizer_test_optimize");
        tokio::fs::create_dir_all(&temp_dir).await.unwrap();
        
        let mut optimizer = StorageOptimizer::new(config, temp_dir.clone());
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut blocks = vec![
            create_test_block(1, current_time - (100 * 86400)), // 100 days old
            create_test_block(2, current_time - (50 * 86400)),  // 50 days old
            create_test_block(3, current_time - 100),            // Recent
        ];
        
        let result = optimizer.optimize(&mut blocks, 3).await.unwrap();
        
        // Should have pruned old blocks
        assert!(result.blocks_pruned > 0 || blocks.len() < 3);
        
        // Cleanup
        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
}
