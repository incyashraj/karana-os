// Phase 53: Ledger Optimization & Pluggable Backend
//
// This module provides:
// 1. Ledger pruning and checkpointing
// 2. Pluggable backend architecture (blockchain vs signed-log)
// 3. Storage optimization
//
// Goals:
// - Reduce storage by 80% through intelligent pruning
// - Enable "no-blockchain" mode for personal devices
// - Cryptographic compression of historical data

pub mod pruning;
pub mod checkpointing;
pub mod backends;
pub mod optimizer;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Configuration for ledger behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerConfig {
    /// Backend type: "blockchain" or "signed-log"
    pub backend_type: BackendType,
    
    /// Pruning settings
    pub pruning: PruningConfig,
    
    /// Checkpointing settings
    pub checkpointing: CheckpointConfig,
    
    /// Storage optimization settings
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackendType {
    /// Full blockchain with consensus (current behavior)
    Blockchain,
    /// Simple signed log without consensus (for personal devices)
    SignedLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningConfig {
    /// Enable automatic pruning
    pub enabled: bool,
    
    /// Prune blocks older than this many days
    pub retention_days: u32,
    
    /// Keep at least this many recent blocks
    pub min_blocks_to_keep: u64,
    
    /// Categories of data to keep indefinitely
    pub keep_categories: Vec<DataCategory>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataCategory {
    Payments,
    Governance,
    Identity,
    HighValueIntents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable automatic checkpointing
    pub enabled: bool,
    
    /// Create checkpoint every N blocks
    pub interval_blocks: u64,
    
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    
    /// Compress checkpoints
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Maximum storage quota in MB
    pub max_storage_mb: u64,
    
    /// Enable compression of old blocks
    pub compress_old_blocks: bool,
    
    /// Move pruned data to cold storage
    pub cold_storage_path: Option<String>,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::Blockchain,
            pruning: PruningConfig {
                enabled: true,
                retention_days: 90,
                min_blocks_to_keep: 1000,
                keep_categories: vec![
                    DataCategory::Payments,
                    DataCategory::Governance,
                ],
            },
            checkpointing: CheckpointConfig {
                enabled: true,
                interval_blocks: 1000,
                max_checkpoints: 10,
                compress: true,
            },
            optimization: OptimizationConfig {
                max_storage_mb: 500,
                compress_old_blocks: true,
                cold_storage_path: None,
            },
        }
    }
}

impl LedgerConfig {
    /// Minimal configuration for constrained devices
    pub fn minimal() -> Self {
        Self {
            backend_type: BackendType::SignedLog,
            pruning: PruningConfig {
                enabled: true,
                retention_days: 30,
                min_blocks_to_keep: 100,
                keep_categories: vec![DataCategory::Payments],
            },
            checkpointing: CheckpointConfig {
                enabled: true,
                interval_blocks: 100,
                max_checkpoints: 3,
                compress: true,
            },
            optimization: OptimizationConfig {
                max_storage_mb: 50,
                compress_old_blocks: true,
                cold_storage_path: Some("/tmp/karana-cold".to_string()),
            },
        }
    }
    
    /// Personal device configuration (no blockchain)
    pub fn personal() -> Self {
        Self {
            backend_type: BackendType::SignedLog,
            pruning: PruningConfig {
                enabled: true,
                retention_days: 60,
                min_blocks_to_keep: 500,
                keep_categories: vec![
                    DataCategory::Payments,
                    DataCategory::Identity,
                ],
            },
            checkpointing: CheckpointConfig {
                enabled: true,
                interval_blocks: 500,
                max_checkpoints: 5,
                compress: true,
            },
            optimization: OptimizationConfig {
                max_storage_mb: 200,
                compress_old_blocks: true,
                cold_storage_path: None,
            },
        }
    }
}

/// Statistics about ledger storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerStats {
    pub total_blocks: u64,
    pub pruned_blocks: u64,
    pub active_blocks: u64,
    pub total_size_mb: f64,
    pub compressed_size_mb: f64,
    pub checkpoint_count: usize,
    pub last_pruned: Option<u64>,
    pub last_checkpoint: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = LedgerConfig::default();
        assert_eq!(config.backend_type, BackendType::Blockchain);
        assert!(config.pruning.enabled);
        assert!(config.checkpointing.enabled);
    }
    
    #[test]
    fn test_minimal_config() {
        let config = LedgerConfig::minimal();
        assert_eq!(config.backend_type, BackendType::SignedLog);
        assert_eq!(config.pruning.retention_days, 30);
        assert_eq!(config.optimization.max_storage_mb, 50);
    }
    
    #[test]
    fn test_personal_config() {
        let config = LedgerConfig::personal();
        assert_eq!(config.backend_type, BackendType::SignedLog);
        assert_eq!(config.pruning.retention_days, 60);
    }
}
