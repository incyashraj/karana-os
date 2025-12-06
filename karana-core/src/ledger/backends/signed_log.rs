// Phase 53: Signed Log Backend
//
// Simplified cryptographic log without consensus:
// - No P2P networking
// - No block production
// - Simple append-only signed entries
// - Perfect for personal devices that never join swarms

use super::{LedgerBackend, BackendStats, BackendConfig, SyncResult};
use crate::chain::{Block, Transaction, BlockHeader};
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

/// Signed log entry (simpler than full block)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Sequential entry number
    pub index: u64,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Hash of previous entry
    pub prev_hash: String,
    
    /// Transactions in this entry
    pub transactions: Vec<Transaction>,
    
    /// Entry hash
    pub hash: String,
    
    /// Signature from device owner
    pub signature: String,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        index: u64,
        prev_hash: String,
        transactions: Vec<Transaction>,
        signing_key: &SigningKey,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Calculate hash (without signature)
        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        for tx in &transactions {
            hasher.update(tx.hash.as_bytes());
        }
        let hash = hex::encode(hasher.finalize());
        
        // Sign the hash
        let signature_bytes = signing_key.sign(hash.as_bytes());
        let signature = hex::encode(signature_bytes.to_bytes());
        
        Self {
            index,
            timestamp,
            prev_hash,
            transactions,
            hash,
            signature,
        }
    }
    
    /// Verify entry signature
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<bool> {
        let sig_bytes = hex::decode(&self.signature)
            .context("Failed to decode signature")?;
        
        if sig_bytes.len() != 64 {
            return Ok(false);
        }
        
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);
        
        Ok(verifying_key.verify(self.hash.as_bytes(), &signature).is_ok())
    }
    
    /// Convert to Block format for compatibility
    pub fn to_block(&self, validator: String) -> Block {
        Block {
            header: BlockHeader {
                parent_hash: self.prev_hash.clone(),
                height: self.index,
                timestamp: self.timestamp,
                state_root: String::new(),
                validator,
            },
            transactions: self.transactions.clone(),
            hash: self.hash.clone(),
        }
    }
}

/// Signed log backend - no consensus, personal device only
pub struct SignedLogBackend {
    config: BackendConfig,
    entries: Arc<Mutex<Vec<LogEntry>>>,
    pending_txs: Arc<Mutex<Vec<Transaction>>>,
    signing_key: Arc<SigningKey>,
    verifying_key: Arc<VerifyingKey>,
}

impl SignedLogBackend {
    /// Create a new signed log backend
    pub fn new(config: BackendConfig) -> Result<Self> {
        // Generate or load device signing key
        let mut csprng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        // Create genesis entry
        let genesis = LogEntry {
            index: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            prev_hash: "0".repeat(64),
            transactions: vec![],
            hash: hex::encode(Sha256::digest(b"genesis")),
            signature: hex::encode(signing_key.sign(b"genesis").to_bytes()),
        };
        
        Ok(Self {
            config,
            entries: Arc::new(Mutex::new(vec![genesis])),
            pending_txs: Arc::new(Mutex::new(Vec::new())),
            signing_key: Arc::new(signing_key),
            verifying_key: Arc::new(verifying_key),
        })
    }
    
    /// Create a new entry from pending transactions
    pub fn create_entry(&self) -> Result<()> {
        let mut pending = self.pending_txs.lock().unwrap();
        
        if pending.is_empty() {
            return Ok(());
        }
        
        let transactions: Vec<Transaction> = pending.drain(..).collect();
        
        let mut entries = self.entries.lock().unwrap();
        let last_entry = entries.last()
            .ok_or_else(|| anyhow::anyhow!("No genesis entry"))?;
        
        let new_entry = LogEntry::new(
            last_entry.index + 1,
            last_entry.hash.clone(),
            transactions,
            &self.signing_key,
        );
        
        entries.push(new_entry);
        
        Ok(())
    }
}

#[async_trait]
impl LedgerBackend for SignedLogBackend {
    fn name(&self) -> &str {
        "signed-log"
    }
    
    async fn init(&mut self) -> Result<()> {
        // No initialization needed for signed log
        Ok(())
    }
    
    async fn add_block(&mut self, block: Block) -> Result<()> {
        // Convert block to log entry
        let mut entries = self.entries.lock().unwrap();
        
        let entry = LogEntry {
            index: block.header.height,
            timestamp: block.header.timestamp,
            prev_hash: block.header.parent_hash.clone(),
            transactions: block.transactions.clone(),
            hash: block.hash.clone(),
            signature: "imported".to_string(),
        };
        
        entries.push(entry);
        Ok(())
    }
    
    async fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let entries = self.entries.lock().unwrap();
        
        Ok(entries.iter()
            .find(|e| e.index == height)
            .map(|e| e.to_block("device".to_string())))
    }
    
    async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let entries = self.entries.lock().unwrap();
        
        Ok(entries.iter()
            .find(|e| e.hash == hash)
            .map(|e| e.to_block("device".to_string())))
    }
    
    async fn get_height(&self) -> Result<u64> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.last()
            .map(|e| e.index)
            .unwrap_or(0))
    }
    
    async fn get_latest_block(&self) -> Result<Option<Block>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.last()
            .map(|e| e.to_block("device".to_string())))
    }
    
    async fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Verify transaction
        if !tx.verify() {
            return Err(anyhow::anyhow!("Invalid transaction signature"));
        }
        
        let mut pending = self.pending_txs.lock().unwrap();
        pending.push(tx);
        
        // Auto-create entry when transactions accumulate
        if pending.len() >= 10 {
            drop(pending); // Release lock
            self.create_entry()?;
        }
        
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
        // For signed log, just verify parent hash
        if block.header.height == 0 {
            return Ok(true); // Genesis
        }
        
        let entries = self.entries.lock().unwrap();
        
        if let Some(prev_entry) = entries.get((block.header.height - 1) as usize) {
            Ok(block.header.parent_hash == prev_entry.hash)
        } else {
            Ok(false)
        }
    }
    
    async fn get_stats(&self) -> Result<BackendStats> {
        let entries = self.entries.lock().unwrap();
        let pending = self.pending_txs.lock().unwrap();
        
        let total_blocks = entries.len() as u64;
        let total_transactions: u64 = entries.iter()
            .map(|e| e.transactions.len() as u64)
            .sum();
        
        let last_block_time = entries.last()
            .map(|e| e.timestamp);
        
        // Estimate storage size
        let storage_size_bytes: usize = entries.iter()
            .map(|e| bincode::serialize(e).unwrap_or_default().len())
            .sum();
        
        Ok(BackendStats {
            backend_type: "signed-log".to_string(),
            total_blocks,
            total_transactions,
            pending_transactions: pending.len(),
            storage_size_mb: storage_size_bytes as f64 / 1_048_576.0,
            last_block_time,
        })
    }
    
    async fn sync(&mut self) -> Result<SyncResult> {
        // Signed log doesn't sync
        Ok(SyncResult {
            synced: true,
            blocks_downloaded: 0,
            peers_contacted: 0,
            time_ms: 0,
        })
    }
    
    async fn export(&self) -> Result<Vec<u8>> {
        let entries = self.entries.lock().unwrap();
        bincode::serialize(&*entries)
            .context("Failed to serialize log for export")
    }
    
    async fn import(&mut self, data: Vec<u8>) -> Result<()> {
        let entries: Vec<LogEntry> = bincode::deserialize(&data)
            .context("Failed to deserialize log import")?;
        
        // Verify all entries
        for (i, entry) in entries.iter().enumerate() {
            if i == 0 {
                continue; // Genesis
            }
            
            if !entry.verify(&self.verifying_key)? {
                return Err(anyhow::anyhow!("Invalid entry signature at index {}", entry.index));
            }
        }
        
        let mut log = self.entries.lock().unwrap();
        *log = entries;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::TransactionData;
    
    #[test]
    fn test_log_entry_creation() {
        let mut csprng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut csprng);
        
        let tx = Transaction {
            sender: "alice".to_string(),
            data: TransactionData::Transfer {
                to: "bob".to_string(),
                amount: 100,
            },
            signature: "sig".to_string(),
            nonce: 1,
            public_key: None,
            hash: "tx_hash".to_string(),
            timestamp: 1000000,
        };
        
        let entry = LogEntry::new(1, "prev_hash".to_string(), vec![tx], &signing_key);
        
        assert_eq!(entry.index, 1);
        assert_eq!(entry.transactions.len(), 1);
        assert!(!entry.signature.is_empty());
    }
    
    #[test]
    fn test_log_entry_verification() {
        let mut csprng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        let entry = LogEntry::new(1, "prev_hash".to_string(), vec![], &signing_key);
        
        assert!(entry.verify(&verifying_key).unwrap());
    }
    
    #[tokio::test]
    async fn test_signed_log_backend() {
        let config = BackendConfig::personal();
        let mut backend = SignedLogBackend::new(config).unwrap();
        
        backend.init().await.unwrap();
        
        let height = backend.get_height().await.unwrap();
        assert_eq!(height, 0); // Genesis
    }
    
    #[tokio::test]
    async fn test_add_transaction_signed_log() {
        let config = BackendConfig::personal();
        let mut backend = SignedLogBackend::new(config).unwrap();
        
        let tx = Transaction {
            sender: "alice".to_string(),
            data: TransactionData::Transfer {
                to: "bob".to_string(),
                amount: 100,
            },
            signature: "sig".to_string(),
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
    async fn test_signed_log_stats() {
        let config = BackendConfig::personal();
        let backend = SignedLogBackend::new(config).unwrap();
        
        let stats = backend.get_stats().await.unwrap();
        
        assert_eq!(stats.backend_type, "signed-log");
        assert_eq!(stats.total_blocks, 1); // Genesis
        assert!(stats.storage_size_mb < 1.0);
    }
}
