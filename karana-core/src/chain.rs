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
}

impl Transaction {
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
            public_key: None, // Legacy attestation
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
    
    // Create unsigned transaction first to get the message
    let mut tx = Transaction {
        sender: wallet.did().to_string(),
        data,
        signature: String::new(),
        nonce,
        public_key: Some(wallet.public_key_hex()),
    };
    
    // Get message and sign it
    let message = tx.signing_message();
    let signature = wallet.sign(&message);
    tx.signature = hex::encode(signature.to_bytes());
    
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
        };
        
        // Should pass verification in legacy mode
        assert!(tx.verify(), "Legacy transaction should verify");
    }
}
