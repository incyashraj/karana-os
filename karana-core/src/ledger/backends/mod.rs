// Phase 53: Pluggable Backend Architecture
//
// Provides abstraction over storage backends:
// 1. BlockchainBackend - Full consensus-based chain (current)
// 2. SignedLogBackend - Simple cryptographic log (personal devices)
//
// This allows "no-blockchain" mode for devices that never join a swarm

pub mod blockchain;
pub mod signed_log;

use crate::chain::{Block, Transaction};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// Trait that all ledger backends must implement
#[async_trait]
pub trait LedgerBackend: Send + Sync {
    /// Get the name of this backend
    fn name(&self) -> &str;
    
    /// Initialize the backend
    async fn init(&mut self) -> Result<()>;
    
    /// Add a new block
    async fn add_block(&mut self, block: Block) -> Result<()>;
    
    /// Get a block by height
    async fn get_block(&self, height: u64) -> Result<Option<Block>>;
    
    /// Get a block by hash
    async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>>;
    
    /// Get the current chain height
    async fn get_height(&self) -> Result<u64>;
    
    /// Get the latest block
    async fn get_latest_block(&self) -> Result<Option<Block>>;
    
    /// Add a transaction to the pending pool
    async fn add_transaction(&mut self, tx: Transaction) -> Result<()>;
    
    /// Get pending transactions
    async fn get_pending_transactions(&self) -> Result<Vec<Transaction>>;
    
    /// Clear pending transactions (after block creation)
    async fn clear_pending_transactions(&mut self) -> Result<()>;
    
    /// Validate a block
    async fn validate_block(&self, block: &Block) -> Result<bool>;
    
    /// Get backend statistics
    async fn get_stats(&self) -> Result<BackendStats>;
    
    /// Sync with peers (for distributed backends)
    async fn sync(&mut self) -> Result<SyncResult>;
    
    /// Export data for backup
    async fn export(&self) -> Result<Vec<u8>>;
    
    /// Import data from backup
    async fn import(&mut self, data: Vec<u8>) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStats {
    pub backend_type: String,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub pending_transactions: usize,
    pub storage_size_mb: f64,
    pub last_block_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced: bool,
    pub blocks_downloaded: u64,
    pub peers_contacted: usize,
    pub time_ms: u64,
}

/// Factory for creating ledger backends
pub struct BackendFactory;

impl BackendFactory {
    /// Create a backend from config
    pub async fn create(backend_type: &str, config: BackendConfig) -> Result<Box<dyn LedgerBackend>> {
        match backend_type.to_lowercase().as_str() {
            "blockchain" => {
                let backend = blockchain::BlockchainBackend::new(config)?;
                Ok(Box::new(backend))
            }
            "signed-log" | "signedlog" | "log" => {
                let backend = signed_log::SignedLogBackend::new(config)?;
                Ok(Box::new(backend))
            }
            _ => {
                Err(anyhow::anyhow!("Unknown backend type: {}", backend_type))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub data_dir: String,
    pub enable_p2p: bool,
    pub enable_celestia: bool,
    pub block_time_secs: u64,
    pub max_block_size: usize,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            data_dir: "./karana-ledger".to_string(),
            enable_p2p: true,
            enable_celestia: true,
            block_time_secs: 30,
            max_block_size: 1_000_000, // 1MB
        }
    }
}

impl BackendConfig {
    /// Personal device configuration (no P2P, no Celestia)
    pub fn personal() -> Self {
        Self {
            data_dir: "./karana-ledger".to_string(),
            enable_p2p: false,
            enable_celestia: false,
            block_time_secs: 60,
            max_block_size: 500_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = BackendConfig::default();
        assert!(config.enable_p2p);
        assert!(config.enable_celestia);
        assert_eq!(config.block_time_secs, 30);
    }
    
    #[test]
    fn test_personal_config() {
        let config = BackendConfig::personal();
        assert!(!config.enable_p2p);
        assert!(!config.enable_celestia);
        assert_eq!(config.block_time_secs, 60);
    }
}
