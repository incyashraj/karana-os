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
    pub signature: String, // Hex encoded Ed25519 signature
    pub nonce: u64,
    /// Optional: public key for verification (hex encoded)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Transaction hash (computed)
    #[serde(default)]
    pub hash: String,
    /// Timestamp
    #[serde(default)]
    pub timestamp: u64,
}

/// Transaction with additional metadata for querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithMeta {
    pub hash: String,
    pub from: String,
    pub data: TransactionData,
    pub timestamp: u64,
}

impl TransactionWithMeta {
    /// Get recipient if this is a transfer
    pub fn get_recipient(&self) -> Option<String> {
        match &self.data {
            TransactionData::Transfer { to, .. } => Some(to.clone()),
            _ => None,
        }
    }
    
    /// Get amount if this is a transfer or stake
    pub fn get_amount(&self) -> Option<u64> {
        match &self.data {
            TransactionData::Transfer { amount, .. } => Some(*amount as u64),
            TransactionData::Stake { amount } => Some(*amount as u64),
            _ => None,
        }
    }
}

impl Transaction {
    /// Create a new transaction with computed hash
    pub fn new(sender: String, data: TransactionData, nonce: u64, signature: Vec<u8>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut tx = Self {
            sender,
            data,
            signature: hex::encode(&signature),
            nonce,
            public_key: None,
            hash: String::new(),
            timestamp,
        };
        
        // Compute hash
        let hash_data = serde_json::to_vec(&tx).unwrap_or_default();
        tx.hash = hex::encode(Sha256::digest(&hash_data));
        tx
    }
    
    /// Get recipient if this is a transfer
    pub fn get_recipient(&self) -> Option<String> {
        match &self.data {
            TransactionData::Transfer { to, .. } => Some(to.clone()),
            _ => None,
        }
    }
    
    /// Get amount if this is a transfer or stake
    pub fn get_amount(&self) -> Option<u64> {
        match &self.data {
            TransactionData::Transfer { amount, .. } => Some(*amount as u64),
            TransactionData::Stake { amount } => Some(*amount as u64),
            _ => None,
        }
    }
    
    /// Get the message bytes that should be signed
    pub fn signing_message(&self) -> Vec<u8> {
        let signing_data = serde_json::json!({
            "sender": self.sender,
            "data": self.data,
            "nonce": self.nonce
        });
        serde_json::to_vec(&signing_data).unwrap_or_default()
    }
    
    /// Verify the transaction signature
    /// 
    /// If public_key is provided, performs real Ed25519 verification.
    /// Otherwise falls back to legacy mode (signature non-empty).
    pub fn verify(&self) -> bool {
        // Empty signature is always invalid
        if self.signature.is_empty() {
            return false;
        }
        
        // If we have a public key, do real verification
        if let Some(ref pubkey_hex) = self.public_key {
            return self.verify_ed25519(pubkey_hex);
        }
        
        // Legacy mode: accept any non-empty signature
        // (for backwards compatibility with old transactions)
        true
    }
    
    /// Perform real Ed25519 signature verification
    fn verify_ed25519(&self, pubkey_hex: &str) -> bool {
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        
        // Decode public key
        let pubkey_bytes = match hex::decode(pubkey_hex) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        if pubkey_bytes.len() != 32 {
            return false;
        }
        
        let pubkey_array: [u8; 32] = match pubkey_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        
        let verifying_key = match VerifyingKey::from_bytes(&pubkey_array) {
            Ok(key) => key,
            Err(_) => return false,
        };
        
        // Decode signature
        let sig_bytes = match hex::decode(&self.signature) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        if sig_bytes.len() != 64 {
            return false;
        }
        
        let sig_array: [u8; 64] = match sig_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        
        let signature = match Signature::from_bytes(&sig_array) {
            sig => sig,
        };
        
        // Get message to verify
        let message = self.signing_message();
        
        // Verify
        verifying_key.verify(&message, &signature).is_ok()
    }
    
    /// Verify that the sender DID matches the public key
    /// 
    /// DIDs are of form: did:karana:<base58(sha256(pubkey)[0:20])>
    pub fn verify_sender_did(&self) -> bool {
        if let Some(ref pubkey_hex) = self.public_key {
            // Decode public key
            let pubkey_bytes = match hex::decode(pubkey_hex) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            };
            
            // Compute expected DID
            let hash = Sha256::digest(&pubkey_bytes);
            let did_suffix = bs58::encode(&hash[..20]).into_string();
            let expected_did = format!("did:karana:{}", did_suffix);
            
            self.sender == expected_did
        } else {
            // Can't verify without public key
            true
        }
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
    /// Block storage (in-memory for now)
    blocks: Mutex<Vec<Block>>,
}

impl Blockchain {
    pub fn new(ledger: Arc<Mutex<Ledger>>, gov: Arc<Mutex<Governance>>) -> Self {
        // Create genesis block
        let genesis = Block::new(
            "0".repeat(64),
            0,
            "genesis".to_string(),
            vec![],
        );
        
        Self { 
            ledger, 
            gov,
            blocks: Mutex::new(vec![genesis]),
        }
    }
    
    /// Get current chain height
    pub fn height(&self) -> u64 {
        let blocks = self.blocks.lock().unwrap();
        blocks.last().map(|b| b.header.height).unwrap_or(0)
    }
    
    /// Get the latest block
    pub fn latest_block(&self) -> Block {
        let blocks = self.blocks.lock().unwrap();
        blocks.last().cloned().unwrap_or_else(|| Block::new("0".repeat(64), 0, "genesis".to_string(), vec![]))
    }
    
    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<Block> {
        let blocks = self.blocks.lock().unwrap();
        blocks.iter().find(|b| b.header.height == height).cloned()
    }
    
    /// Add a new block to the chain
    pub fn add_block(&self, block: Block) -> Result<()> {
        let mut blocks = self.blocks.lock().unwrap();
        let expected_height = blocks.last().map(|b| b.header.height + 1).unwrap_or(0);
        
        if block.header.height != expected_height {
            return Err(anyhow::anyhow!("Invalid block height: expected {}, got {}", expected_height, block.header.height));
        }
        
        blocks.push(block);
        Ok(())
    }
    
    /// Get transactions for a specific DID
    pub fn get_transactions_for(&self, did: &str, limit: usize) -> Vec<TransactionWithMeta> {
        let blocks = self.blocks.lock().unwrap();
        let mut result = Vec::new();
        
        for block in blocks.iter().rev() {
            for tx in &block.transactions {
                if tx.sender == did || tx.get_recipient().as_deref() == Some(did) {
                    result.push(TransactionWithMeta {
                        hash: format!("0x{}", hex::encode(sha2::Sha256::digest(serde_json::to_vec(tx).unwrap_or_default()))[..16].to_string()),
                        from: tx.sender.clone(),
                        data: tx.data.clone(),
                        timestamp: block.header.timestamp,
                    });
                    if result.len() >= limit {
                        return result;
                    }
                }
            }
        }
        
        result
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
            public_key: None, // Legacy attestation
            hash: String::new(),
            timestamp,
        }
    }
}

/// Helper to create a properly signed transaction
pub fn create_signed_transaction(
    wallet: &crate::wallet::KaranaWallet,
    data: TransactionData,
) -> Transaction {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let timestamp = nonce; // Use nonce as timestamp
    
    // Create unsigned transaction first to get the message
    let mut tx = Transaction {
        sender: wallet.did().to_string(),
        data,
        signature: String::new(),
        nonce,
        public_key: Some(wallet.public_key_hex()),
        hash: String::new(),
        timestamp,
    };
    
    // Get message and sign it
    let message = tx.signing_message();
    let signature = wallet.sign(&message);
    tx.signature = hex::encode(signature.to_bytes());
    
    // Compute hash
    let hash_data = serde_json::to_vec(&tx).unwrap_or_default();
    tx.hash = hex::encode(Sha256::digest(&hash_data));
    
    tx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::KaranaWallet;
    
    #[test]
    fn test_transaction_signing_and_verification() {
        // Create a wallet
        let result = KaranaWallet::generate("test-device").unwrap();
        let wallet = result.wallet;
        
        // Create a signed transaction
        let tx = create_signed_transaction(
            &wallet,
            TransactionData::Transfer {
                to: "did:karana:recipient".to_string(),
                amount: 100,
            },
        );
        
        // Verify the transaction
        assert!(tx.verify(), "Transaction should verify");
        assert!(tx.verify_sender_did(), "Sender DID should match public key");
    }
    
    #[test]
    fn test_invalid_signature_rejected() {
        // Create a wallet
        let result = KaranaWallet::generate("test-device").unwrap();
        let wallet = result.wallet;
        
        // Create a signed transaction
        let mut tx = create_signed_transaction(
            &wallet,
            TransactionData::Transfer {
                to: "did:karana:recipient".to_string(),
                amount: 100,
            },
        );
        
        // Tamper with the signature
        tx.signature = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string();
        
        // Verification should fail
        assert!(!tx.verify(), "Tampered transaction should not verify");
    }
    
    #[test]
    fn test_wrong_sender_rejected() {
        // Create a wallet
        let result = KaranaWallet::generate("test-device").unwrap();
        let wallet = result.wallet;
        
        // Create a signed transaction
        let mut tx = create_signed_transaction(
            &wallet,
            TransactionData::Transfer {
                to: "did:karana:recipient".to_string(),
                amount: 100,
            },
        );
        
        // Change the sender (impersonation attempt)
        tx.sender = "did:karana:evil".to_string();
        
        // DID verification should fail
        assert!(!tx.verify_sender_did(), "Wrong sender DID should not verify");
    }
    
    #[test]
    fn test_legacy_transaction_accepted() {
        // Legacy transaction without public key
        let tx = Transaction {
            sender: "legacy:user".to_string(),
            data: TransactionData::Transfer {
                to: "recipient".to_string(),
                amount: 50,
            },
            signature: "legacy_sig".to_string(),
            nonce: 12345,
            public_key: None,
            hash: String::new(),
            timestamp: 0,
        };
        
        // Should pass verification in legacy mode
        assert!(tx.verify(), "Legacy transaction should verify");
    }
}
