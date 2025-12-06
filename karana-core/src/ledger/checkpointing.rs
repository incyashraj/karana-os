// Phase 53: Ledger Checkpointing System
//
// Creates cryptographic snapshots of chain state:
// 1. Fast sync from checkpoints without full history
// 2. Chain verification from compressed summaries
// 3. Recovery from corruption
// 4. Reduced storage overhead

use super::{CheckpointConfig};
use crate::chain::{Block, BlockHeader};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// A checkpoint representing chain state at a specific height
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Height this checkpoint was created at
    pub height: u64,
    
    /// Hash of the block at this height
    pub block_hash: String,
    
    /// Timestamp of checkpoint creation
    pub timestamp: u64,
    
    /// Merkle root of all blocks up to this height
    pub chain_merkle_root: String,
    
    /// State snapshot (simplified - could be full state trie)
    pub state_snapshot: StateSnapshot,
    
    /// Summary of blocks since last checkpoint
    pub block_summaries: Vec<BlockSummary>,
    
    /// Checkpoint metadata
    pub metadata: CheckpointMetadata,
}

/// Simplified state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Account balances
    pub balances: HashMap<String, u128>,
    
    /// Account nonces
    pub nonces: HashMap<String, u64>,
    
    /// Active proposals
    pub proposals: HashMap<u64, ProposalSummary>,
    
    /// Staking amounts
    pub stakes: HashMap<String, u128>,
    
    /// Total supply
    pub total_supply: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: u64,
    pub title: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: String,
}

/// Summary of a block for checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub tx_count: usize,
    pub validator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Size of full chain up to this checkpoint (bytes)
    pub full_chain_size: usize,
    
    /// Size of this checkpoint (bytes)
    pub checkpoint_size: usize,
    
    /// Compression ratio
    pub compression_ratio: f64,
    
    /// Creation time
    pub created_at: u64,
    
    /// Checkpoint format version
    pub version: u32,
}

/// Manages checkpoint creation and restoration
pub struct CheckpointManager {
    config: CheckpointConfig,
    checkpoint_dir: PathBuf,
    checkpoints: Vec<Checkpoint>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(config: CheckpointConfig, checkpoint_dir: PathBuf) -> Self {
        Self {
            config,
            checkpoint_dir,
            checkpoints: Vec::new(),
        }
    }
    
    /// Check if a checkpoint should be created at this height
    pub fn should_create_checkpoint(&self, height: u64) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        height > 0 && height % self.config.interval_blocks == 0
    }
    
    /// Create a checkpoint from current chain state
    pub async fn create_checkpoint(
        &mut self,
        blocks: &[Block],
        state: StateSnapshot,
    ) -> Result<Checkpoint> {
        let height = blocks.last()
            .map(|b| b.header.height)
            .unwrap_or(0);
        
        let block_hash = blocks.last()
            .map(|b| b.hash.clone())
            .unwrap_or_default();
        
        // Calculate chain merkle root
        let block_hashes: Vec<String> = blocks.iter()
            .map(|b| b.hash.clone())
            .collect();
        let chain_merkle_root = self.calculate_chain_merkle_root(&block_hashes);
        
        // Create block summaries
        let block_summaries: Vec<BlockSummary> = blocks.iter()
            .map(|b| BlockSummary {
                height: b.header.height,
                hash: b.hash.clone(),
                parent_hash: b.header.parent_hash.clone(),
                timestamp: b.header.timestamp,
                tx_count: b.transactions.len(),
                validator: b.header.validator.clone(),
            })
            .collect();
        
        // Calculate sizes
        let full_chain_size = blocks.iter()
            .map(|b| bincode::serialize(b).unwrap_or_default().len())
            .sum();
        
        let checkpoint = Checkpoint {
            height,
            block_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            chain_merkle_root,
            state_snapshot: state,
            block_summaries,
            metadata: CheckpointMetadata {
                full_chain_size,
                checkpoint_size: 0, // Will update after serialization
                compression_ratio: 0.0,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                version: 1,
            },
        };
        
        let checkpoint_size = bincode::serialize(&checkpoint)
            .context("Failed to serialize checkpoint")?
            .len();
        
        let compression_ratio = if full_chain_size > 0 {
            checkpoint_size as f64 / full_chain_size as f64
        } else {
            1.0
        };
        
        let mut checkpoint = checkpoint;
        checkpoint.metadata.checkpoint_size = checkpoint_size;
        checkpoint.metadata.compression_ratio = compression_ratio;
        
        // Save to disk
        self.save_checkpoint(&checkpoint).await?;
        
        // Add to memory
        self.checkpoints.push(checkpoint.clone());
        
        // Enforce max checkpoints limit
        self.prune_old_checkpoints().await?;
        
        Ok(checkpoint)
    }
    
    /// Calculate merkle root of all block hashes
    fn calculate_chain_merkle_root(&self, hashes: &[String]) -> String {
        if hashes.is_empty() {
            return hex::encode(Sha256::digest(b"empty"));
        }
        
        if hashes.len() == 1 {
            return hashes[0].clone();
        }
        
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
    
    /// Save checkpoint to disk
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        fs::create_dir_all(&self.checkpoint_dir).await
            .context("Failed to create checkpoint directory")?;
        
        let checkpoint_path = self.checkpoint_dir
            .join(format!("checkpoint_{:08}.bin", checkpoint.height));
        
        let data = bincode::serialize(checkpoint)
            .context("Failed to serialize checkpoint")?;
        
        // Optionally compress
        let final_data = if self.config.compress {
            zstd::encode_all(&data[..], 3)
                .context("Failed to compress checkpoint")?
        } else {
            data
        };
        
        fs::write(&checkpoint_path, final_data).await
            .context("Failed to write checkpoint")?;
        
        Ok(())
    }
    
    /// Load checkpoint from disk
    pub async fn load_checkpoint(&self, height: u64) -> Result<Checkpoint> {
        let checkpoint_path = self.checkpoint_dir
            .join(format!("checkpoint_{:08}.bin", height));
        
        let data = fs::read(&checkpoint_path).await
            .context("Failed to read checkpoint")?;
        
        // Decompress if needed
        let final_data = if self.config.compress {
            zstd::decode_all(&data[..])
                .context("Failed to decompress checkpoint")?
        } else {
            data
        };
        
        let checkpoint: Checkpoint = bincode::deserialize(&final_data)
            .context("Failed to deserialize checkpoint")?;
        
        Ok(checkpoint)
    }
    
    /// Get the most recent checkpoint
    pub fn get_latest_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }
    
    /// Get checkpoint at or before a specific height
    pub fn get_checkpoint_before(&self, height: u64) -> Option<&Checkpoint> {
        self.checkpoints.iter()
            .rev()
            .find(|c| c.height <= height)
    }
    
    /// Prune old checkpoints to stay within max limit
    async fn prune_old_checkpoints(&mut self) -> Result<()> {
        while self.checkpoints.len() > self.config.max_checkpoints {
            if let Some(oldest) = self.checkpoints.first() {
                let checkpoint_path = self.checkpoint_dir
                    .join(format!("checkpoint_{:08}.bin", oldest.height));
                
                // Delete file
                if checkpoint_path.exists() {
                    fs::remove_file(&checkpoint_path).await
                        .context("Failed to delete old checkpoint")?;
                }
                
                // Remove from memory
                self.checkpoints.remove(0);
            }
        }
        
        Ok(())
    }
    
    /// Load all checkpoints from disk
    pub async fn load_all_checkpoints(&mut self) -> Result<()> {
        let mut entries = fs::read_dir(&self.checkpoint_dir).await
            .context("Failed to read checkpoint directory")?;
        
        let mut checkpoints = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(height_str) = file_name
                        .strip_prefix("checkpoint_")
                        .and_then(|s| s.strip_suffix(".bin"))
                    {
                        if let Ok(height) = height_str.parse::<u64>() {
                            match self.load_checkpoint(height).await {
                                Ok(checkpoint) => checkpoints.push(checkpoint),
                                Err(e) => {
                                    eprintln!("Failed to load checkpoint at height {}: {}", height, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by height
        checkpoints.sort_by_key(|c| c.height);
        
        self.checkpoints = checkpoints;
        
        Ok(())
    }
    
    /// Verify checkpoint integrity
    pub fn verify_checkpoint(&self, checkpoint: &Checkpoint) -> bool {
        // Verify merkle root matches block summaries
        let block_hashes: Vec<String> = checkpoint.block_summaries.iter()
            .map(|s| s.hash.clone())
            .collect();
        
        let computed_root = self.calculate_chain_merkle_root(&block_hashes);
        
        computed_root == checkpoint.chain_merkle_root
    }
    
    /// Get statistics about checkpoints
    pub fn get_stats(&self) -> CheckpointStats {
        let total_size: usize = self.checkpoints.iter()
            .map(|c| c.metadata.checkpoint_size)
            .sum();
        
        let total_chain_size: usize = self.checkpoints.iter()
            .map(|c| c.metadata.full_chain_size)
            .sum();
        
        let avg_compression = if !self.checkpoints.is_empty() {
            self.checkpoints.iter()
                .map(|c| c.metadata.compression_ratio)
                .sum::<f64>() / self.checkpoints.len() as f64
        } else {
            0.0
        };
        
        CheckpointStats {
            checkpoint_count: self.checkpoints.len(),
            total_checkpoint_size_mb: total_size as f64 / 1_048_576.0,
            total_chain_size_mb: total_chain_size as f64 / 1_048_576.0,
            average_compression_ratio: avg_compression,
            latest_checkpoint_height: self.checkpoints.last().map(|c| c.height),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    pub checkpoint_count: usize,
    pub total_checkpoint_size_mb: f64,
    pub total_chain_size_mb: f64,
    pub average_compression_ratio: f64,
    pub latest_checkpoint_height: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn create_test_block(height: u64) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: "0".repeat(64),
                height,
                timestamp: 1000000 + height,
                state_root: String::new(),
                validator: "test".to_string(),
            },
            transactions: vec![],
            hash: format!("block_{}", height),
        }
    }
    
    fn create_test_state() -> StateSnapshot {
        let mut balances = HashMap::new();
        balances.insert("alice".to_string(), 1000);
        balances.insert("bob".to_string(), 500);
        
        StateSnapshot {
            balances,
            nonces: HashMap::new(),
            proposals: HashMap::new(),
            stakes: HashMap::new(),
            total_supply: 1500,
        }
    }
    
    #[test]
    fn test_should_create_checkpoint() {
        let config = CheckpointConfig {
            enabled: true,
            interval_blocks: 100,
            max_checkpoints: 10,
            compress: true,
        };
        
        let temp_dir = std::env::temp_dir().join("karana_checkpoint_test");
        let manager = CheckpointManager::new(config, temp_dir);
        
        assert!(!manager.should_create_checkpoint(0));
        assert!(!manager.should_create_checkpoint(50));
        assert!(manager.should_create_checkpoint(100));
        assert!(manager.should_create_checkpoint(200));
    }
    
    #[tokio::test]
    async fn test_create_checkpoint() {
        let config = CheckpointConfig {
            enabled: true,
            interval_blocks: 100,
            max_checkpoints: 10,
            compress: true,
        };
        
        let temp_dir = std::env::temp_dir().join("karana_checkpoint_test_create");
        let mut manager = CheckpointManager::new(config, temp_dir.clone());
        
        let blocks = vec![
            create_test_block(1),
            create_test_block(2),
            create_test_block(3),
        ];
        
        let state = create_test_state();
        
        let checkpoint = manager.create_checkpoint(&blocks, state).await.unwrap();
        
        assert_eq!(checkpoint.height, 3);
        assert_eq!(checkpoint.block_summaries.len(), 3);
        assert!(checkpoint.metadata.compression_ratio < 1.0);
        
        // Cleanup
        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
    
    #[test]
    fn test_verify_checkpoint() {
        let config = CheckpointConfig {
            enabled: true,
            interval_blocks: 100,
            max_checkpoints: 10,
            compress: true,
        };
        
        let temp_dir = std::env::temp_dir().join("karana_checkpoint_test_verify");
        let manager = CheckpointManager::new(config, temp_dir);
        
        let checkpoint = Checkpoint {
            height: 100,
            block_hash: "hash".to_string(),
            timestamp: 1000000,
            chain_merkle_root: "empty_root".to_string(),
            state_snapshot: create_test_state(),
            block_summaries: vec![],
            metadata: CheckpointMetadata {
                full_chain_size: 1000,
                checkpoint_size: 100,
                compression_ratio: 0.1,
                created_at: 1000000,
                version: 1,
            },
        };
        
        // Empty summaries should produce different root
        assert!(!manager.verify_checkpoint(&checkpoint));
    }
}
