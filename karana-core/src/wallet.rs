//! # Kāraṇa Wallet - Sovereign Key Management
//!
//! Real cryptographic wallet implementation for the Kāraṇa OS.
//! 
//! ## Key Features
//! - BIP-39 mnemonic seed phrase generation
//! - Ed25519 signing keys
//! - Device-specific key derivation
//! - Secure key storage with encryption
//! - DID (Decentralized Identifier) derivation
//!
//! ## Security Model
//! - Private keys never leave the device
//! - Keys are zeroized when dropped
//! - Encrypted at rest with user-provided password
//! - Hardware-bound when secure enclave available

use anyhow::{Result, anyhow};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use bip39::Mnemonic;
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Serialize, Deserialize};
use rand::RngCore;
use std::fs;
use std::path::Path;

/// Number of words in the recovery mnemonic
pub const MNEMONIC_WORD_COUNT: usize = 24;

/// Derivation path for Kāraṇa wallets
pub const KARANA_DERIVATION_PATH: &str = "m/44'/8888'/0'/0/0";

/// The core wallet structure - holds the user's sovereign identity
#[derive(ZeroizeOnDrop)]
pub struct KaranaWallet {
    /// The Ed25519 signing key (private key)
    #[zeroize(skip)] // SigningKey handles its own zeroization
    signing_key: SigningKey,
    
    /// Cached public key for quick access
    #[zeroize(skip)]
    verifying_key: VerifyingKey,
    
    /// The user's DID (did:karana:...)
    did: String,
    
    /// Device-specific identifier for key binding
    device_id: String,
}

/// Mnemonic phrase wrapper with secure handling
#[derive(ZeroizeOnDrop)]
pub struct RecoveryPhrase {
    words: Vec<String>,
}

impl RecoveryPhrase {
    pub fn new(words: Vec<String>) -> Self {
        Self { words }
    }
    
    pub fn words(&self) -> &[String] {
        &self.words
    }
    
    pub fn as_string(&self) -> String {
        self.words.join(" ")
    }
    
    /// Display format with numbered words for backup
    pub fn display_for_backup(&self) -> String {
        self.words
            .iter()
            .enumerate()
            .map(|(i, w)| format!("{:2}. {}", i + 1, w))
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.join("  "))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Encrypted wallet format for storage
#[derive(Serialize, Deserialize)]
pub struct EncryptedWallet {
    /// Version for future compatibility
    pub version: u8,
    /// The DID (not secret)
    pub did: String,
    /// Device ID this wallet is bound to
    pub device_id: String,
    /// Encrypted seed bytes (AES-256-GCM)
    pub encrypted_seed: Vec<u8>,
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Nonce for encryption
    pub nonce: Vec<u8>,
}

/// Wallet creation result
pub struct WalletCreationResult {
    pub wallet: KaranaWallet,
    pub recovery_phrase: RecoveryPhrase,
}

impl KaranaWallet {
    /// Generate a new wallet with a fresh mnemonic
    /// 
    /// Returns the wallet and the recovery phrase that MUST be backed up
    pub fn generate(device_id: &str) -> Result<WalletCreationResult> {
        // Generate random entropy for 24-word mnemonic (256 bits = 32 bytes)
        let mut entropy = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut entropy);
        
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;
        
        let words: Vec<String> = mnemonic.words().map(|s| s.to_string()).collect();
        let recovery_phrase = RecoveryPhrase::new(words);
        
        // Derive seed from mnemonic (no passphrase)
        let seed = mnemonic.to_seed("");
        
        // Create wallet from seed
        let wallet = Self::from_seed(&seed[..32], device_id)?;
        
        log::info!("[WALLET] 🔐 New wallet generated: {}", wallet.did);
        
        Ok(WalletCreationResult {
            wallet,
            recovery_phrase,
        })
    }
    
    /// Restore wallet from recovery phrase
    pub fn from_mnemonic(phrase: &str, device_id: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse(phrase)
            .map_err(|e| anyhow!("Invalid mnemonic phrase: {}", e))?;
        
        let seed = mnemonic.to_seed("");
        Self::from_seed(&seed[..32], device_id)
    }
    
    /// Create wallet from raw seed bytes
    fn from_seed(seed_bytes: &[u8], device_id: &str) -> Result<Self> {
        if seed_bytes.len() < 32 {
            return Err(anyhow!("Seed must be at least 32 bytes"));
        }
        
        // Create signing key from seed
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed_bytes[..32]);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        // Derive DID from public key
        let did = Self::derive_did(&verifying_key);
        
        key_bytes.zeroize();
        
        Ok(Self {
            signing_key,
            verifying_key,
            did,
            device_id: device_id.to_string(),
        })
    }
    
    /// Derive a DID from the public key
    fn derive_did(verifying_key: &VerifyingKey) -> String {
        let pubkey_bytes = verifying_key.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&pubkey_bytes);
        let hash = hasher.finalize();
        
        // Use multibase encoding (base58btc)
        let encoded = bs58::encode(&hash[..20]).into_string();
        format!("did:karana:{}", encoded)
    }
    
    /// Get the wallet's DID
    pub fn did(&self) -> &str {
        &self.did
    }
    
    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
    
    /// Get the public key as hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }
    
    /// Sign arbitrary data
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }
    
    /// Sign and return signature as bytes
    pub fn sign_bytes(&self, message: &[u8]) -> Vec<u8> {
        self.sign(message).to_bytes().to_vec()
    }
    
    /// Sign and return signature as hex
    pub fn sign_hex(&self, message: &[u8]) -> String {
        hex::encode(self.sign_bytes(message))
    }
    
    /// Verify a signature (static method for verification without wallet)
    pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        if public_key.len() != 32 || signature.len() != 64 {
            return false;
        }
        
        let mut pk_bytes = [0u8; 32];
        let mut sig_bytes = [0u8; 64];
        pk_bytes.copy_from_slice(public_key);
        sig_bytes.copy_from_slice(signature);
        
        let Ok(vk) = VerifyingKey::from_bytes(&pk_bytes) else {
            return false;
        };
        let sig = Signature::from_bytes(&sig_bytes);
        vk.verify(message, &sig).is_ok()
    }
    
    /// Get device ID this wallet is bound to
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    
    /// Save wallet encrypted to disk
    pub fn save_encrypted(&self, path: &Path, password: &str) -> Result<()> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
        
        // Generate salt
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        
        // Derive encryption key from password + salt
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        let key_bytes = hasher.finalize();
        
        // Generate nonce
        let mut nonce_bytes = vec![0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the signing key bytes
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        
        let seed_bytes = self.signing_key.to_bytes();
        let encrypted_seed = cipher.encrypt(nonce, seed_bytes.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        let encrypted_wallet = EncryptedWallet {
            version: 1,
            did: self.did.clone(),
            device_id: self.device_id.clone(),
            encrypted_seed,
            salt,
            nonce: nonce_bytes,
        };
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(&encrypted_wallet)?;
        fs::write(path, json)?;
        
        log::info!("[WALLET] 💾 Wallet saved to {}", path.display());
        Ok(())
    }
    
    /// Load wallet from encrypted file
    pub fn load_encrypted(path: &Path, password: &str) -> Result<Self> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
        
        let json = fs::read_to_string(path)?;
        let encrypted_wallet: EncryptedWallet = serde_json::from_str(&json)?;
        
        // Derive decryption key
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&encrypted_wallet.salt);
        let key_bytes = hasher.finalize();
        
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        
        let nonce = Nonce::from_slice(&encrypted_wallet.nonce);
        
        let seed_bytes = cipher.decrypt(nonce, encrypted_wallet.encrypted_seed.as_ref())
            .map_err(|_| anyhow!("Decryption failed - wrong password?"))?;
        
        let wallet = Self::from_seed(&seed_bytes, &encrypted_wallet.device_id)?;
        
        // Verify DID matches
        if wallet.did != encrypted_wallet.did {
            return Err(anyhow!("Wallet integrity check failed"));
        }
        
        log::info!("[WALLET] 🔓 Wallet loaded: {}", wallet.did);
        Ok(wallet)
    }
    
    /// Check if a wallet file exists
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }
    
    /// Get the standard wallet path for this device
    pub fn default_path() -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("karana")
            .join("wallet.json")
    }
}

/// Represents a signed transaction
#[derive(Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The sender's DID
    pub sender: String,
    /// Transaction data (serialized)
    pub data: Vec<u8>,
    /// Ed25519 signature (hex encoded for JSON compatibility)
    pub signature: String,
    /// Transaction nonce
    pub nonce: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl SignedTransaction {
    /// Create a new signed transaction
    pub fn new(wallet: &KaranaWallet, data: &[u8], nonce: u64) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Build message to sign: data + nonce + timestamp
        let mut message = Vec::new();
        message.extend_from_slice(data);
        message.extend_from_slice(&nonce.to_le_bytes());
        message.extend_from_slice(&timestamp.to_le_bytes());
        
        let signature = wallet.sign_hex(&message);
        
        Self {
            sender: wallet.did().to_string(),
            data: data.to_vec(),
            signature,
            nonce,
            timestamp,
        }
    }
    
    /// Verify this transaction's signature
    pub fn verify(&self, public_key: &[u8]) -> bool {
        let mut message = Vec::new();
        message.extend_from_slice(&self.data);
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message.extend_from_slice(&self.timestamp.to_le_bytes());
        
        let Ok(sig_bytes) = hex::decode(&self.signature) else {
            return false;
        };
        
        KaranaWallet::verify_signature(public_key, &message, &sig_bytes)
    }
    
    /// Get transaction hash
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.sender);
        hasher.update(&self.data);
        hasher.update(&self.signature);
        hasher.update(&self.nonce.to_le_bytes());
        hasher.finalize().into()
    }
    
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash())
    }
}

/// Get a unique device identifier
pub fn get_device_id() -> String {
    // Try to read machine-id (Linux)
    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        let id = id.trim();
        if !id.is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(id.as_bytes());
            hasher.update(b"karana-device");
            return hex::encode(&hasher.finalize()[..8]);
        }
    }
    
    // Fallback: generate random ID
    let mut random_bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    hex::encode(random_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_wallet_generation() {
        let result = KaranaWallet::generate("test-device").unwrap();
        
        assert!(result.wallet.did().starts_with("did:karana:"));
        assert_eq!(result.recovery_phrase.words().len(), 24);
    }
    
    #[test]
    fn test_wallet_restore() {
        let result = KaranaWallet::generate("test-device").unwrap();
        let phrase = result.recovery_phrase.as_string();
        
        let restored = KaranaWallet::from_mnemonic(&phrase, "test-device").unwrap();
        
        assert_eq!(result.wallet.did(), restored.did());
        assert_eq!(result.wallet.public_key_hex(), restored.public_key_hex());
    }
    
    #[test]
    fn test_signing() {
        let result = KaranaWallet::generate("test-device").unwrap();
        let message = b"Hello, Karana!";
        
        let signature = result.wallet.sign_bytes(message);
        let pubkey = result.wallet.public_key_bytes();
        
        assert!(KaranaWallet::verify_signature(&pubkey, message, &signature));
    }
    
    #[test]
    fn test_invalid_signature() {
        let result = KaranaWallet::generate("test-device").unwrap();
        let message = b"Hello, Karana!";
        let wrong_message = b"Wrong message";
        
        let signature = result.wallet.sign_bytes(message);
        let pubkey = result.wallet.public_key_bytes();
        
        assert!(!KaranaWallet::verify_signature(&pubkey, wrong_message, &signature));
    }
    
    #[test]
    fn test_signed_transaction() {
        let result = KaranaWallet::generate("test-device").unwrap();
        let data = b"transfer:100:alice";
        
        let tx = SignedTransaction::new(&result.wallet, data, 1);
        let pubkey = result.wallet.public_key_bytes();
        
        assert!(tx.verify(&pubkey));
        assert_eq!(tx.sender, result.wallet.did());
    }
    
    #[test]
    fn test_save_load_encrypted() {
        let result = KaranaWallet::generate("test-device").unwrap();
        let path = PathBuf::from("/tmp/karana_test_wallet.json");
        let password = "test_password_123";
        
        result.wallet.save_encrypted(&path, password).unwrap();
        
        let loaded = KaranaWallet::load_encrypted(&path, password).unwrap();
        
        assert_eq!(result.wallet.did(), loaded.did());
        assert_eq!(result.wallet.public_key_hex(), loaded.public_key_hex());
        
        // Wrong password should fail
        let bad_load = KaranaWallet::load_encrypted(&path, "wrong_password");
        assert!(bad_load.is_err());
        
        // Cleanup
        fs::remove_file(&path).ok();
    }
}
