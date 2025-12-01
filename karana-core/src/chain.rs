use serde::{Serialize, Deserialize};
use alloy_primitives::U256;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use crate::gov::Proposal;
use crate::storage::MerkleTree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    Transfer { to: String, amount: U256 },
    Stake { amount: U256 },
    Propose { title: String, description: String },
    Vote { proposal_id: u32, approve: bool },
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
                state_root: String::new(), // Placeholder for Merkle root
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
}

// The Sovereign State Machine
#[derive(Default)]
pub struct ChainState {
    pub balances: HashMap<String, U256>,
    pub staked: HashMap<String, U256>,
    pub proposals: HashMap<u32, Proposal>,
    pub next_proposal_id: u32,
}

impl ChainState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            staked: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 0,
        }
    }

    pub fn apply(&mut self, tx: &Transaction, sender: &str) -> anyhow::Result<()> {
        match tx {
            Transaction::Transfer { to, amount } => {
                let balance = self.balances.entry(sender.to_string()).or_default();
                if *balance < *amount { return Err(anyhow::anyhow!("Insufficient funds")); }
                *balance -= *amount;
                *self.balances.entry(to.clone()).or_default() += *amount;
            }
            Transaction::Stake { amount } => {
                let balance = self.balances.entry(sender.to_string()).or_default();
                if *balance < *amount { return Err(anyhow::anyhow!("Insufficient funds to stake")); }
                *balance -= *amount;
                *self.staked.entry(sender.to_string()).or_default() += *amount;
            }
            Transaction::Propose { title, description } => {
                let id = self.next_proposal_id;
                self.next_proposal_id += 1;
                self.proposals.insert(id, Proposal {
                    id,
                    title: title.clone(),
                    description: description.clone(),
                    yes_votes: U256::ZERO,
                    no_votes: U256::ZERO,
                    quorum: U256::from(100u64),
                });
            }
            Transaction::Vote { proposal_id, approve } => {
                let stake = self.staked.get(sender).cloned().unwrap_or(U256::ZERO);
                if stake == U256::ZERO { return Err(anyhow::anyhow!("No stake")); }
                
                if let Some(prop) = self.proposals.get_mut(proposal_id) {
                    if *approve { prop.yes_votes += stake; } else { prop.no_votes += stake; }
                }
            }
        }
        Ok(())
    }

    pub fn calculate_root(&self) -> String {
        // Deterministic serialization: Sort keys
        let mut data = Vec::new();
        
        let mut balances_vec: Vec<_> = self.balances.iter().collect();
        balances_vec.sort_by_key(|k| k.0);
        for (k, v) in balances_vec {
            data.extend_from_slice(k.as_bytes());
            data.extend_from_slice(&v.to_be_bytes::<32>());
        }

        let mut staked_vec: Vec<_> = self.staked.iter().collect();
        staked_vec.sort_by_key(|k| k.0);
        for (k, v) in staked_vec {
            data.extend_from_slice(k.as_bytes());
            data.extend_from_slice(&v.to_be_bytes::<32>());
        }
        
        // If empty, return empty hash
        if data.is_empty() {
            return hex::encode(Sha256::digest(b"").to_vec());
        }

        let tree = MerkleTree::new(&data);
        hex::encode(tree.root)
    }
}
