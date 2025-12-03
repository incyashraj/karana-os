use std::sync::{Arc, Mutex};
use anyhow::Result;
use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use crate::ai::KaranaAI;

// Atom 4: The Sovereign Economy

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KaraToken {
    pub balance: u128,
    pub staked: u128,
    pub reputation: f32, // 0.0 - 1.0
}

pub struct Ledger {
    // Map of NodeID/PubKey -> Token Balance
    // accounts: HashMap<String, KaraToken>,
    db: DB,
}

impl Ledger {
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).expect("Failed to open Ledger DB");
        
        Self {
            db,
        }
    }

    pub fn mint(&mut self, recipient: &str, amount: u128) {
        let mut account = self.get_account(recipient);
        account.balance += amount;
        self.save_account(recipient, &account);
        log::info!("Atom 4 (Economy): 🪙 Minted {} KARA to Node '{}'. Balance: {} | Staked: {}", amount, recipient, account.balance, account.staked);
    }

    pub fn transfer(&mut self, sender: &str, recipient: &str, amount: u128) -> Result<()> {
        let mut sender_acc = self.get_account(sender);
        if sender_acc.balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }
        let mut recipient_acc = self.get_account(recipient);
        
        sender_acc.balance -= amount;
        recipient_acc.balance += amount;
        
        self.save_account(sender, &sender_acc);
        self.save_account(recipient, &recipient_acc);
        log::info!("Atom 4 (Economy): 💸 Transferred {} KARA from '{}' to '{}'", amount, sender, recipient);
        Ok(())
    }
    
    /// Debit (subtract) from an account's balance
    pub fn debit(&mut self, account_id: &str, amount: u64) {
        let mut account = self.get_account(account_id);
        account.balance = account.balance.saturating_sub(amount as u128);
        self.save_account(account_id, &account);
        log::debug!("[LEDGER] Debited {} from {}", amount, account_id);
    }
    
    /// Credit (add) to an account's balance
    pub fn credit(&mut self, account_id: &str, amount: u64) {
        let mut account = self.get_account(account_id);
        account.balance += amount as u128;
        self.save_account(account_id, &account);
        log::debug!("[LEDGER] Credited {} to {}", amount, account_id);
    }
    
    /// Unstake tokens
    pub fn unstake(&mut self, account_id: &str, amount: u64) -> Result<()> {
        let mut account = self.get_account(account_id);
        if account.staked < amount as u128 {
            return Err(anyhow::anyhow!("Insufficient staked balance"));
        }
        account.staked -= amount as u128;
        account.balance += amount as u128;
        self.save_account(account_id, &account);
        log::info!("Atom 4 (Economy): 🔓 Node '{}' unstaked {} KARA.", account_id, amount);
        Ok(())
    }

    pub fn stake(&mut self, account_id: &str, amount: u128) -> Result<()> {
        let mut account = self.get_account(account_id);
        if account.balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance to stake"));
        }
        account.balance -= amount;
        account.staked += amount;
        self.save_account(account_id, &account);
        log::info!("Atom 4 (Economy): 🔒 Node '{}' staked {} KARA.", account_id, amount);
        Ok(())
    }

    pub fn slash(&mut self, account_id: &str, amount: u128, reason: &str) {
        let mut account = self.get_account(account_id);
        let slashed = std::cmp::min(account.staked, amount);
        account.staked -= slashed;
        account.reputation *= 0.8; // Reputation hit
        self.save_account(account_id, &account);
        log::info!("Atom 4 (Economy): ⚔️ SLASHED Node '{}' for {}. Burned {} KARA. Reputation: {:.2}", account_id, reason, slashed, account.reputation);
    }

    pub fn get_balance(&self, account: &str) -> u128 {
        self.get_account(account).balance
    }

    pub fn get_account(&self, recipient: &str) -> KaraToken {
        match self.db.get(recipient.as_bytes()) {
            Ok(Some(data)) => serde_json::from_slice(&data).unwrap_or_default(),
            _ => KaraToken { balance: 0, staked: 0, reputation: 1.0 },
        }
    }

    fn save_account(&self, recipient: &str, account: &KaraToken) {
        let data = serde_json::to_vec(account).unwrap();
        self.db.put(recipient.as_bytes(), data).expect("Failed to save account");
    }
}

pub struct ProofOfStorage {
    ledger: Arc<Mutex<Ledger>>,
}

impl ProofOfStorage {
    pub fn new(ledger: Arc<Mutex<Ledger>>) -> Self {
        Self { ledger }
    }

    /// Simulates a Challenge-Response protocol.
    /// In a full implementation, the `proof` would be a ZK-SNARK or Merkle Branch.
    pub fn verify_and_reward(&self, node_id: &str, expected_hash: [u8; 32], proof_data: &[u8]) -> Result<()> {
        log::info!("Atom 4 (Economy): 🛡️ Verifying Proof of Storage...");
        
        // Verify ZK Proof
        let is_valid = if proof_data.is_empty() {
            false
        } else {
            crate::zk::verify_proof(proof_data, expected_hash)
        };

        if is_valid {
            log::info!("Atom 4 (Economy): ✅ ZK Proof Accepted. Issuing Reward.");
            self.ledger.lock().unwrap().mint(node_id, 10); // 10 KARA reward
        } else {
            log::info!("Atom 4 (Economy): ❌ ZK Proof Rejected.");
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    pub ai_analysis: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
}

pub struct Governance {
    ledger: Arc<Mutex<Ledger>>,
    db: DB,
    next_id: u64,
    ai: Arc<Mutex<KaranaAI>>,
}

impl Governance {
    pub fn new(path: &str, ledger: Arc<Mutex<Ledger>>, ai: Arc<Mutex<KaranaAI>>) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).expect("Failed to open Governance DB");
        
        // Load next_id
        let next_id = match db.get(b"next_id") {
            Ok(Some(val)) => u64::from_le_bytes(val.try_into().unwrap()),
            _ => 1,
        };

        Self {
            ledger,
            db,
            next_id,
            ai,
        }
    }

    pub fn create_proposal(&mut self, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        // Save next_id
        self.db.put(b"next_id", self.next_id.to_le_bytes()).expect("Failed to save next_id");

        // Atom 4: AI Analysis of Proposal
        let prompt = format!("Analyze this governance proposal for risks/benefits: '{}'. Keep it short.", description);
        let analysis = self.ai.lock().unwrap().predict(&prompt, 30).unwrap_or_else(|_| "Analysis failed".to_string());

        let proposal = Proposal {
            id,
            title: description.to_string(), // Use description as title for now
            description: description.to_string(),
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active,
            ai_analysis: analysis.trim().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.save_proposal(&proposal);
        log::info!("Atom 4 (Governance): 📜 Proposal #{} Created: '{}'", id, description);
        log::info!("Atom 4 (Governance): 🤖 AI Analysis: {}", proposal.ai_analysis);
        id
    }

    pub fn vote(&mut self, proposal_id: u64, voter: &str, approve: bool) -> Result<()> {
        let _balance = self.ledger.lock().unwrap().get_balance(voter);
        // Allow voting even with 0 balance for demo, or check stake?
        // Let's say voting power = balance + stake
        let account = self.ledger.lock().unwrap().get_account(voter);
        let power = account.balance + account.staked;

        if power == 0 {
            log::info!("Atom 4 (Governance): ⚠️ Node '{}' has no Power (KARA/Stake) to vote.", voter);
            return Ok(());
        }

        if let Some(mut proposal) = self.get_proposal(proposal_id) {
            if proposal.status != ProposalStatus::Active {
                log::info!("Atom 4 (Governance): ⚠️ Proposal #{} is closed.", proposal_id);
                return Ok(());
            }

            if approve {
                proposal.votes_for += power as u64;
            } else {
                proposal.votes_against += power as u64;
            }
            
            self.save_proposal(&proposal);
            log::info!("Atom 4 (Governance): 🗳️ Node '{}' voted {} with power {}.", voter, if approve { "YES" } else { "NO" }, power);
        } else {
             log::info!("Atom 4 (Governance): ⚠️ Proposal #{} not found.", proposal_id);
        }
        Ok(())
    }
    
    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<Proposal> {
        let mut proposals = Vec::new();
        
        // Iterate through all proposals (simple approach)
        for id in 1..self.next_id {
            if let Some(p) = self.get_proposal(id) {
                if p.status == ProposalStatus::Active {
                    proposals.push(p);
                }
            }
        }
        
        proposals
    }

    fn save_proposal(&self, proposal: &Proposal) {
        let key = format!("proposal:{}", proposal.id);
        let data = serde_json::to_vec(proposal).unwrap();
        self.db.put(key.as_bytes(), data).expect("Failed to save proposal");
    }

    pub fn get_proposal(&self, id: u64) -> Option<Proposal> {
        let key = format!("proposal:{}", id);
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => serde_json::from_slice(&data).ok(),
            _ => None,
        }
    }
}
