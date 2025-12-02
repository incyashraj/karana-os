use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};
use crate::economy::{Ledger, Governance};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    Transfer { to: String, amount: u128 },
    Stake { amount: u128 },
    Propose { title: String, description: String },
    Vote { proposal_id: u64, approve: bool },
    /// Phase 7.5: Intent attestation with ZK proof
    IntentAttestation { 
        intent: String, 
        proof_hash: String,  // Hash of ZK proof
        result_hash: String, // Hash of action result
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub data: TransactionData,
    pub signature: String, // Hex encoded signature
    pub nonce: u64,
}

impl Transaction {
    pub fn verify(&self) -> bool {
        // In a real implementation, we would:
        // 1. Serialize (sender, data, nonce)
        // 2. Verify signature against sender's public key (assumed to be 'sender' here)
        // For this prototype, we assume if signature is present, it's valid.
        !self.signature.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub state_root: String,
    pub validator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(parent_hash: String, height: u64, validator: String, transactions: Vec<Transaction>) -> Self {
        let mut block = Self {
            header: BlockHeader {
                parent_hash,
                height,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                state_root: String::new(), // Placeholder
                validator,
            },
            transactions,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let data = serde_json::to_vec(&self.header).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        for tx in &self.transactions {
            hasher.update(serde_json::to_vec(tx).unwrap());
        }
        hex::encode(hasher.finalize())
    }

    pub fn validate(&self, parent_hash: &str) -> Result<()> {
        if self.header.parent_hash != parent_hash {
            return Err(anyhow::anyhow!("Invalid parent hash"));
        }
        if self.calculate_hash() != self.hash {
            return Err(anyhow::anyhow!("Invalid block hash"));
        }
        for tx in &self.transactions {
            if !tx.verify() {
                return Err(anyhow::anyhow!("Invalid transaction signature"));
            }
        }
        Ok(())
    }
}

pub struct Blockchain {
    ledger: Arc<Mutex<Ledger>>,
    gov: Arc<Mutex<Governance>>,
}

impl Blockchain {
    pub fn new(ledger: Arc<Mutex<Ledger>>, gov: Arc<Mutex<Governance>>) -> Self {
        Self { ledger, gov }
    }

    pub fn apply_block(&self, block: &Block) -> Result<()> {
        for tx in &block.transactions {
            match &tx.data {
                TransactionData::Transfer { to, amount } => {
                    self.ledger.lock().unwrap().transfer(&tx.sender, to, *amount)?;
                }
                TransactionData::Stake { amount } => {
                    self.ledger.lock().unwrap().stake(&tx.sender, *amount)?;
                }
                TransactionData::Propose { title: _, description } => {
                    // Title ignored in economy::Governance for now
                    self.gov.lock().unwrap().create_proposal(description);
                }
                TransactionData::Vote { proposal_id, approve } => {
                    self.gov.lock().unwrap().vote(*proposal_id, &tx.sender, *approve)?;
                }
                TransactionData::IntentAttestation { intent, proof_hash, result_hash, timestamp } => {
                    // Phase 7.5: Record intent completion on chain
                    log::info!("[CHAIN] ✓ Intent attested: '{}' at {} [proof: {}..., result: {}...]", 
                        intent, timestamp, &proof_hash[..8.min(proof_hash.len())], &result_hash[..8.min(result_hash.len())]);
                    // In a full implementation, this would update a state trie
                }
            }
        }
        Ok(())
    }

    /// Phase 7.5: Create an attestation transaction for an intent completion
    pub fn attest_intent(&self, sender: &str, intent: &str, proof: &[u8], result: &str) -> Transaction {
        let proof_hash = hex::encode(Sha256::digest(proof));
        let result_hash = hex::encode(Sha256::digest(result.as_bytes()));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Transaction {
            sender: sender.to_string(),
            data: TransactionData::IntentAttestation {
                intent: intent.to_string(),
                proof_hash,
                result_hash,
                timestamp,
            },
            signature: "attested".to_string(),
            nonce: timestamp,
        }
    }
}
