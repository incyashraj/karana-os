// Encryption service for Kāraṇa OS

use super::{EncryptionAlgorithm, SecurityError};

/// Encryption service for data protection
pub struct EncryptionService {
    algorithm: EncryptionAlgorithm,
    default_key_size: usize,
}

impl EncryptionService {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self {
            algorithm,
            default_key_size: algorithm.key_size(),
        }
    }
    
    /// Generate a random encryption key
    pub fn generate_key(&self) -> Vec<u8> {
        // Simulated - would use cryptographically secure random
        vec![0u8; self.default_key_size]
    }
    
    /// Generate a key with specific size
    pub fn generate_key_with_size(&self, size: usize) -> Vec<u8> {
        vec![0u8; size]
    }
    
    /// Encrypt data with given key
    pub fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if key.len() < self.algorithm.key_size() {
            return Err(SecurityError::EncryptionFailed);
        }
        
        // Generate nonce
        let nonce = self.generate_nonce();
        
        // Simulated encryption - would use actual crypto library
        let mut encrypted = nonce.clone();
        encrypted.extend_from_slice(data);
        
        // Add authentication tag (simulated)
        encrypted.extend_from_slice(&[0u8; 16]);
        
        Ok(encrypted)
    }
    
    /// Decrypt data with given key
    pub fn decrypt(&self, encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if key.len() < self.algorithm.key_size() {
            return Err(SecurityError::DecryptionFailed);
        }
        
        let nonce_size = self.algorithm.nonce_size();
        let tag_size = 16;
        
        if encrypted.len() < nonce_size + tag_size {
            return Err(SecurityError::DecryptionFailed);
        }
        
        // Extract nonce and ciphertext
        let _nonce = &encrypted[..nonce_size];
        let ciphertext = &encrypted[nonce_size..encrypted.len() - tag_size];
        let _tag = &encrypted[encrypted.len() - tag_size..];
        
        // Simulated decryption
        Ok(ciphertext.to_vec())
    }
    
    /// Generate random nonce
    fn generate_nonce(&self) -> Vec<u8> {
        vec![0u8; self.algorithm.nonce_size()]
    }
    
    /// Get the encryption algorithm
    pub fn algorithm(&self) -> EncryptionAlgorithm {
        self.algorithm
    }
    
    /// Derive a key from password
    pub fn derive_key(&self, password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
        // Simulated PBKDF2 key derivation
        let mut key = password.as_bytes().to_vec();
        key.extend_from_slice(salt);
        
        // Would iterate actual KDF
        for _ in 0..iterations.min(10) {
            // Simulated iteration
        }
        
        key.resize(self.default_key_size, 0);
        key
    }
    
    /// Hash data using SHA-256
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        // Simulated SHA-256
        let mut hash = data.to_vec();
        hash.resize(32, 0);
        hash
    }
    
    /// Compute HMAC
    pub fn hmac(&self, data: &[u8], key: &[u8]) -> Vec<u8> {
        // Simulated HMAC-SHA256
        let mut mac = data.to_vec();
        mac.extend_from_slice(key);
        mac.resize(32, 0);
        mac
    }
    
    /// Verify HMAC
    pub fn verify_hmac(&self, data: &[u8], key: &[u8], expected: &[u8]) -> bool {
        let computed = self.hmac(data, key);
        constant_time_compare(&computed, expected)
    }
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    
    diff == 0
}

/// Key management system
pub struct KeyManager {
    master_key: Option<Vec<u8>>,
    key_store: std::collections::HashMap<String, EncryptedKey>,
    key_metadata: std::collections::HashMap<String, KeyMetadata>,
}

/// Encrypted key storage
#[derive(Debug, Clone)]
pub struct EncryptedKey {
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: EncryptionAlgorithm,
}

/// Metadata about a stored key
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    pub key_id: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub purpose: KeyPurpose,
    pub algorithm: EncryptionAlgorithm,
    pub rotations: u32,
    pub last_used: Option<u64>,
}

/// Purpose of encryption key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyPurpose {
    DataEncryption,
    KeyEncryption,
    Authentication,
    Signing,
    Transport,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            master_key: None,
            key_store: std::collections::HashMap::new(),
            key_metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Initialize with master key
    pub fn initialize(&mut self, master_key: Vec<u8>) {
        self.master_key = Some(master_key);
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.master_key.is_some()
    }
    
    /// Store a key
    pub fn store_key(&mut self, key_id: &str, key: &[u8], purpose: KeyPurpose) 
        -> Result<(), SecurityError> 
    {
        let master = self.master_key.as_ref()
            .ok_or(SecurityError::KeyNotFound)?;
        
        // Encrypt the key with master key
        let encryption = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let encrypted = encryption.encrypt(key, master)?;
        
        self.key_store.insert(key_id.to_string(), EncryptedKey {
            encrypted_data: encrypted,
            nonce: vec![0u8; 12],
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        });
        
        self.key_metadata.insert(key_id.to_string(), KeyMetadata {
            key_id: key_id.to_string(),
            created_at: 0,
            expires_at: None,
            purpose,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            rotations: 0,
            last_used: None,
        });
        
        Ok(())
    }
    
    /// Retrieve a key
    pub fn get_key(&self, key_id: &str) -> Result<Vec<u8>, SecurityError> {
        let master = self.master_key.as_ref()
            .ok_or(SecurityError::KeyNotFound)?;
        
        let encrypted = self.key_store.get(key_id)
            .ok_or(SecurityError::KeyNotFound)?;
        
        let encryption = EncryptionService::new(encrypted.algorithm);
        encryption.decrypt(&encrypted.encrypted_data, master)
    }
    
    /// Delete a key
    pub fn delete_key(&mut self, key_id: &str) -> bool {
        let removed = self.key_store.remove(key_id).is_some();
        self.key_metadata.remove(key_id);
        removed
    }
    
    /// Rotate a key
    pub fn rotate_key(&mut self, key_id: &str) -> Result<(), SecurityError> {
        let old_key = self.get_key(key_id)?;
        let metadata = self.key_metadata.get(key_id)
            .ok_or(SecurityError::KeyNotFound)?
            .clone();
        
        // Generate new key
        let encryption = EncryptionService::new(metadata.algorithm);
        let new_key = encryption.generate_key();
        
        // Store new key
        self.store_key(key_id, &new_key, metadata.purpose)?;
        
        // Update rotation count
        if let Some(meta) = self.key_metadata.get_mut(key_id) {
            meta.rotations += 1;
        }
        
        // Would also need to re-encrypt data encrypted with old key
        let _ = old_key; // Securely dispose of old key
        
        Ok(())
    }
    
    /// List all key IDs
    pub fn list_keys(&self) -> Vec<&String> {
        self.key_store.keys().collect()
    }
    
    /// Get key metadata
    pub fn get_metadata(&self, key_id: &str) -> Option<&KeyMetadata> {
        self.key_metadata.get(key_id)
    }
    
    /// Check if key exists
    pub fn has_key(&self, key_id: &str) -> bool {
        self.key_store.contains_key(key_id)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Secure random number generator wrapper
pub struct SecureRandom;

impl SecureRandom {
    /// Generate random bytes
    pub fn bytes(count: usize) -> Vec<u8> {
        // Simulated - would use OS-provided CSPRNG
        vec![0u8; count]
    }
    
    /// Generate random u64
    pub fn u64() -> u64 {
        // Simulated
        0
    }
    
    /// Generate random u32
    pub fn u32() -> u32 {
        0
    }
    
    /// Generate random bytes in range
    pub fn u32_range(min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min // Simulated
    }
}

/// Data signing service
pub struct SigningService {
    algorithm: SigningAlgorithm,
}

/// Signing algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SigningAlgorithm {
    Ed25519,
    EcdsaP256,
    EcdsaP384,
    Rsa2048,
    Rsa4096,
}

impl SigningService {
    pub fn new(algorithm: SigningAlgorithm) -> Self {
        Self { algorithm }
    }
    
    /// Generate a key pair
    pub fn generate_key_pair(&self) -> (Vec<u8>, Vec<u8>) {
        // Simulated - returns (private_key, public_key)
        let key_size = match self.algorithm {
            SigningAlgorithm::Ed25519 => 32,
            SigningAlgorithm::EcdsaP256 => 32,
            SigningAlgorithm::EcdsaP384 => 48,
            SigningAlgorithm::Rsa2048 => 256,
            SigningAlgorithm::Rsa4096 => 512,
        };
        
        (vec![0u8; key_size], vec![0u8; key_size])
    }
    
    /// Sign data
    pub fn sign(&self, data: &[u8], private_key: &[u8]) -> Vec<u8> {
        // Simulated signature
        let mut sig = data.to_vec();
        sig.extend_from_slice(private_key);
        sig.truncate(64);
        sig
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        // Simulated verification
        !data.is_empty() && !signature.is_empty() && !public_key.is_empty()
    }
    
    /// Get algorithm
    pub fn algorithm(&self) -> SigningAlgorithm {
        self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_service_creation() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(service.algorithm(), EncryptionAlgorithm::Aes256Gcm);
    }
    
    #[test]
    fn test_generate_key() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let key = service.generate_key();
        assert_eq!(key.len(), 32);
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let key = service.generate_key();
        let data = b"Hello, World!";
        
        let encrypted = service.encrypt(data, &key).unwrap();
        let decrypted = service.decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, data);
    }
    
    #[test]
    fn test_encrypt_invalid_key() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let short_key = vec![0u8; 8]; // Too short
        let data = b"Test";
        
        let result = service.encrypt(data, &short_key);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_derive_key() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let salt = b"random_salt";
        
        let key = service.derive_key("password123", salt, 10000);
        assert_eq!(key.len(), 32);
    }
    
    #[test]
    fn test_hash() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let hash = service.hash(b"test data");
        assert_eq!(hash.len(), 32);
    }
    
    #[test]
    fn test_hmac() {
        let service = EncryptionService::new(EncryptionAlgorithm::Aes256Gcm);
        let key = b"secret_key";
        let data = b"message";
        
        let mac = service.hmac(data, key);
        assert!(service.verify_hmac(data, key, &mac));
    }
    
    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"hello", b"hello"));
        assert!(!constant_time_compare(b"hello", b"world"));
        assert!(!constant_time_compare(b"short", b"longer"));
    }
    
    #[test]
    fn test_key_manager() {
        let mut manager = KeyManager::new();
        
        assert!(!manager.is_initialized());
        
        manager.initialize(vec![0u8; 32]);
        assert!(manager.is_initialized());
    }
    
    #[test]
    fn test_key_manager_store_retrieve() {
        let mut manager = KeyManager::new();
        manager.initialize(vec![0u8; 32]);
        
        let key = vec![1, 2, 3, 4, 5];
        manager.store_key("test_key", &key, KeyPurpose::DataEncryption).unwrap();
        
        assert!(manager.has_key("test_key"));
        
        let retrieved = manager.get_key("test_key").unwrap();
        assert_eq!(retrieved, key);
    }
    
    #[test]
    fn test_key_manager_delete() {
        let mut manager = KeyManager::new();
        manager.initialize(vec![0u8; 32]);
        
        manager.store_key("test_key", &[1, 2, 3], KeyPurpose::DataEncryption).unwrap();
        assert!(manager.has_key("test_key"));
        
        assert!(manager.delete_key("test_key"));
        assert!(!manager.has_key("test_key"));
    }
    
    #[test]
    fn test_key_manager_rotate() {
        let mut manager = KeyManager::new();
        manager.initialize(vec![0u8; 32]);
        
        manager.store_key("rotate_key", &[1, 2, 3], KeyPurpose::DataEncryption).unwrap();
        
        let result = manager.rotate_key("rotate_key");
        assert!(result.is_ok());
        
        let meta = manager.get_metadata("rotate_key").unwrap();
        assert_eq!(meta.rotations, 1);
    }
    
    #[test]
    fn test_secure_random() {
        let bytes = SecureRandom::bytes(32);
        assert_eq!(bytes.len(), 32);
    }
    
    #[test]
    fn test_signing_service() {
        let service = SigningService::new(SigningAlgorithm::Ed25519);
        let (private_key, public_key) = service.generate_key_pair();
        
        let data = b"message to sign";
        let signature = service.sign(data, &private_key);
        
        assert!(service.verify(data, &signature, &public_key));
    }
    
    #[test]
    fn test_signing_algorithms() {
        let service = SigningService::new(SigningAlgorithm::EcdsaP256);
        assert_eq!(service.algorithm(), SigningAlgorithm::EcdsaP256);
    }
}
