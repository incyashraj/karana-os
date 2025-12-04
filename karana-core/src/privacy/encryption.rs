//! Encryption System for Kāraṇa OS AR Glasses
//!
//! Data encryption for sensitive information stored on device
//! and transmitted over networks.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Encryption level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncryptionLevel {
    /// No encryption
    None,
    /// Basic encryption (AES-128)
    Basic,
    /// Standard encryption (AES-256)
    Standard,
    /// High security (AES-256 + additional measures)
    High,
    /// Maximum security (hardware-backed + AES-256-GCM)
    Maximum,
}

impl EncryptionLevel {
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            EncryptionLevel::None => "No encryption",
            EncryptionLevel::Basic => "AES-128 encryption",
            EncryptionLevel::Standard => "AES-256 encryption",
            EncryptionLevel::High => "AES-256-GCM with key derivation",
            EncryptionLevel::Maximum => "Hardware-backed AES-256-GCM with secure enclave",
        }
    }
    
    /// Key size in bits
    pub fn key_size_bits(&self) -> usize {
        match self {
            EncryptionLevel::None => 0,
            EncryptionLevel::Basic => 128,
            EncryptionLevel::Standard => 256,
            EncryptionLevel::High => 256,
            EncryptionLevel::Maximum => 256,
        }
    }
}

/// Key type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Symmetric key
    Symmetric,
    /// Asymmetric public key
    PublicKey,
    /// Asymmetric private key
    PrivateKey,
    /// Key encryption key
    KEK,
    /// Data encryption key
    DEK,
    /// Session key
    Session,
}

/// Key information
#[derive(Debug, Clone)]
pub struct KeyInfo {
    /// Key identifier
    pub id: String,
    /// Key type
    pub key_type: KeyType,
    /// Encryption level
    pub level: EncryptionLevel,
    /// Creation time
    pub created_at: Instant,
    /// Expiration time
    pub expires_at: Option<Instant>,
    /// Is hardware-backed
    pub hardware_backed: bool,
    /// Usage count
    pub use_count: u64,
    /// Key purpose
    pub purpose: String,
}

/// Encryption result
#[derive(Debug, Clone)]
pub struct EncryptionResult {
    /// Encrypted data (simulated as bytes)
    pub ciphertext: Vec<u8>,
    /// Initialization vector
    pub iv: Vec<u8>,
    /// Authentication tag (for AEAD)
    pub auth_tag: Option<Vec<u8>>,
    /// Key ID used
    pub key_id: String,
}

/// Decryption result
#[derive(Debug)]
pub enum DecryptionResult {
    /// Successful decryption
    Success(Vec<u8>),
    /// Key not found
    KeyNotFound,
    /// Authentication failed
    AuthenticationFailed,
    /// Key expired
    KeyExpired,
    /// Decryption error
    Error(String),
}

/// Data classification for encryption policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataClassification {
    /// Public data, no encryption needed
    Public,
    /// Internal data, basic encryption
    Internal,
    /// Confidential data, standard encryption
    Confidential,
    /// Sensitive personal data
    SensitivePersonal,
    /// Biometric data
    Biometric,
    /// Health data
    Health,
    /// Financial data
    Financial,
    /// Authentication credentials
    Credentials,
}

impl DataClassification {
    /// Required encryption level
    pub fn required_encryption(&self) -> EncryptionLevel {
        match self {
            DataClassification::Public => EncryptionLevel::None,
            DataClassification::Internal => EncryptionLevel::Basic,
            DataClassification::Confidential => EncryptionLevel::Standard,
            DataClassification::SensitivePersonal => EncryptionLevel::High,
            DataClassification::Biometric => EncryptionLevel::Maximum,
            DataClassification::Health => EncryptionLevel::High,
            DataClassification::Financial => EncryptionLevel::High,
            DataClassification::Credentials => EncryptionLevel::Maximum,
        }
    }
}

/// Encryption manager
#[derive(Debug)]
pub struct EncryptionManager {
    /// Managed keys (simulated - in real implementation, keys would be in secure enclave)
    keys: HashMap<String, KeyInfo>,
    /// Default encryption level
    default_level: EncryptionLevel,
    /// Data classification policies
    policies: HashMap<DataClassification, EncryptionLevel>,
    /// Hardware security module available
    hsm_available: bool,
    /// Secure enclave available
    secure_enclave_available: bool,
    /// Key rotation interval
    key_rotation_interval: Duration,
}

impl EncryptionManager {
    /// Create new encryption manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_level: EncryptionLevel::Standard,
            policies: Self::default_policies(),
            hsm_available: false,
            secure_enclave_available: true, // Simulated
            key_rotation_interval: Duration::from_secs(30 * 24 * 3600), // 30 days
        }
    }
    
    /// Default policies
    fn default_policies() -> HashMap<DataClassification, EncryptionLevel> {
        let mut policies = HashMap::new();
        for classification in [
            DataClassification::Public,
            DataClassification::Internal,
            DataClassification::Confidential,
            DataClassification::SensitivePersonal,
            DataClassification::Biometric,
            DataClassification::Health,
            DataClassification::Financial,
            DataClassification::Credentials,
        ] {
            policies.insert(classification, classification.required_encryption());
        }
        policies
    }
    
    /// Generate new key
    pub fn generate_key(
        &mut self,
        key_type: KeyType,
        level: EncryptionLevel,
        purpose: &str,
        validity: Option<Duration>,
    ) -> String {
        let id = format!("key_{}", self.keys.len() + 1);
        let now = Instant::now();
        
        let key_info = KeyInfo {
            id: id.clone(),
            key_type,
            level,
            created_at: now,
            expires_at: validity.map(|v| now + v),
            hardware_backed: level == EncryptionLevel::Maximum && self.secure_enclave_available,
            use_count: 0,
            purpose: purpose.to_string(),
        };
        
        self.keys.insert(id.clone(), key_info);
        id
    }
    
    /// Encrypt data
    pub fn encrypt(&mut self, data: &[u8], key_id: &str) -> Option<EncryptionResult> {
        let key = self.keys.get_mut(key_id)?;
        
        // Check expiration
        if let Some(expires) = key.expires_at {
            if Instant::now() >= expires {
                return None;
            }
        }
        
        key.use_count += 1;
        
        // Simulated encryption - in real implementation, this would use actual crypto
        let iv = vec![0u8; 16]; // Would be random
        let auth_tag = if key.level >= EncryptionLevel::High {
            Some(vec![0u8; 16]) // Would be computed
        } else {
            None
        };
        
        // "Encrypted" data (just XOR for simulation)
        let ciphertext: Vec<u8> = data.iter().map(|b| b ^ 0xAA).collect();
        
        Some(EncryptionResult {
            ciphertext,
            iv,
            auth_tag,
            key_id: key_id.to_string(),
        })
    }
    
    /// Decrypt data
    pub fn decrypt(&mut self, encrypted: &EncryptionResult) -> DecryptionResult {
        let key = match self.keys.get_mut(&encrypted.key_id) {
            Some(k) => k,
            None => return DecryptionResult::KeyNotFound,
        };
        
        // Check expiration
        if let Some(expires) = key.expires_at {
            if Instant::now() >= expires {
                return DecryptionResult::KeyExpired;
            }
        }
        
        key.use_count += 1;
        
        // Simulated decryption (reverse XOR)
        let plaintext: Vec<u8> = encrypted.ciphertext.iter().map(|b| b ^ 0xAA).collect();
        
        DecryptionResult::Success(plaintext)
    }
    
    /// Encrypt for classification
    pub fn encrypt_classified(
        &mut self,
        data: &[u8],
        classification: DataClassification,
    ) -> Option<EncryptionResult> {
        let required_level = self.policies
            .get(&classification)
            .copied()
            .unwrap_or(self.default_level);
        
        // Find or create appropriate key
        let key_id = self.find_or_create_key(required_level);
        self.encrypt(data, &key_id)
    }
    
    /// Find or create key of appropriate level
    fn find_or_create_key(&mut self, level: EncryptionLevel) -> String {
        // Find existing valid key
        for (id, key) in &self.keys {
            if key.level >= level {
                if let Some(expires) = key.expires_at {
                    if Instant::now() < expires {
                        return id.clone();
                    }
                } else {
                    return id.clone();
                }
            }
        }
        
        // Create new key
        self.generate_key(KeyType::DEK, level, "auto-generated", Some(self.key_rotation_interval))
    }
    
    /// Rotate key
    pub fn rotate_key(&mut self, old_key_id: &str) -> Option<String> {
        let old_key = self.keys.get(old_key_id)?;
        let level = old_key.level;
        let purpose = old_key.purpose.clone();
        
        // Generate new key
        let new_id = self.generate_key(
            KeyType::DEK,
            level,
            &format!("{} (rotated)", purpose),
            Some(self.key_rotation_interval),
        );
        
        Some(new_id)
    }
    
    /// Delete key
    pub fn delete_key(&mut self, key_id: &str) -> bool {
        self.keys.remove(key_id).is_some()
    }
    
    /// Get key info
    pub fn get_key_info(&self, key_id: &str) -> Option<&KeyInfo> {
        self.keys.get(key_id)
    }
    
    /// List all keys
    pub fn list_keys(&self) -> Vec<&KeyInfo> {
        self.keys.values().collect()
    }
    
    /// Check if key is expired
    pub fn is_key_expired(&self, key_id: &str) -> bool {
        if let Some(key) = self.keys.get(key_id) {
            if let Some(expires) = key.expires_at {
                return Instant::now() >= expires;
            }
        }
        false
    }
    
    /// Set encryption policy
    pub fn set_policy(&mut self, classification: DataClassification, level: EncryptionLevel) {
        self.policies.insert(classification, level);
    }
    
    /// Get policy for classification
    pub fn get_policy(&self, classification: DataClassification) -> EncryptionLevel {
        self.policies
            .get(&classification)
            .copied()
            .unwrap_or(self.default_level)
    }
    
    /// Check if secure enclave is available
    pub fn has_secure_enclave(&self) -> bool {
        self.secure_enclave_available
    }
    
    /// Set secure enclave availability
    pub fn set_secure_enclave(&mut self, available: bool) {
        self.secure_enclave_available = available;
    }
    
    /// Get encryption stats
    pub fn stats(&self) -> EncryptionStats {
        let total_keys = self.keys.len();
        let hardware_backed = self.keys.values().filter(|k| k.hardware_backed).count();
        let expired = self.keys.values().filter(|k| {
            k.expires_at.map(|e| Instant::now() >= e).unwrap_or(false)
        }).count();
        let total_uses: u64 = self.keys.values().map(|k| k.use_count).sum();
        
        EncryptionStats {
            total_keys,
            hardware_backed_keys: hardware_backed,
            expired_keys: expired,
            total_operations: total_uses,
            secure_enclave_available: self.secure_enclave_available,
        }
    }
}

impl Default for EncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Encryption statistics
#[derive(Debug, Clone)]
pub struct EncryptionStats {
    /// Total keys
    pub total_keys: usize,
    /// Hardware-backed keys
    pub hardware_backed_keys: usize,
    /// Expired keys
    pub expired_keys: usize,
    /// Total encryption/decryption operations
    pub total_operations: u64,
    /// Secure enclave available
    pub secure_enclave_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_manager_creation() {
        let em = EncryptionManager::new();
        assert_eq!(em.default_level, EncryptionLevel::Standard);
    }
    
    #[test]
    fn test_key_generation() {
        let mut em = EncryptionManager::new();
        
        let key_id = em.generate_key(
            KeyType::DEK,
            EncryptionLevel::Standard,
            "test",
            None,
        );
        
        assert!(em.get_key_info(&key_id).is_some());
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let mut em = EncryptionManager::new();
        
        let key_id = em.generate_key(
            KeyType::DEK,
            EncryptionLevel::Standard,
            "test",
            None,
        );
        
        let data = b"Hello, World!";
        let encrypted = em.encrypt(data, &key_id).unwrap();
        
        assert_ne!(encrypted.ciphertext, data);
        
        if let DecryptionResult::Success(decrypted) = em.decrypt(&encrypted) {
            assert_eq!(decrypted, data);
        } else {
            panic!("Decryption failed");
        }
    }
    
    #[test]
    fn test_classified_encryption() {
        let mut em = EncryptionManager::new();
        
        let data = b"Biometric data";
        let encrypted = em.encrypt_classified(data, DataClassification::Biometric);
        
        assert!(encrypted.is_some());
    }
    
    #[test]
    fn test_key_info() {
        let mut em = EncryptionManager::new();
        
        let key_id = em.generate_key(
            KeyType::PrivateKey,
            EncryptionLevel::Maximum,
            "auth",
            Some(Duration::from_secs(3600)),
        );
        
        let info = em.get_key_info(&key_id).unwrap();
        assert_eq!(info.level, EncryptionLevel::Maximum);
        assert!(info.expires_at.is_some());
    }
    
    #[test]
    fn test_encryption_levels() {
        assert!(EncryptionLevel::Maximum > EncryptionLevel::Basic);
        assert_eq!(EncryptionLevel::Standard.key_size_bits(), 256);
    }
    
    #[test]
    fn test_data_classification_policy() {
        let em = EncryptionManager::new();
        
        assert_eq!(
            em.get_policy(DataClassification::Biometric),
            EncryptionLevel::Maximum
        );
        assert_eq!(
            em.get_policy(DataClassification::Public),
            EncryptionLevel::None
        );
    }
    
    #[test]
    fn test_key_rotation() {
        let mut em = EncryptionManager::new();
        
        let old_key = em.generate_key(
            KeyType::DEK,
            EncryptionLevel::Standard,
            "test",
            None,
        );
        
        let new_key = em.rotate_key(&old_key);
        assert!(new_key.is_some());
        assert_ne!(old_key, new_key.unwrap());
    }
    
    #[test]
    fn test_key_deletion() {
        let mut em = EncryptionManager::new();
        
        let key_id = em.generate_key(KeyType::DEK, EncryptionLevel::Basic, "test", None);
        assert!(em.delete_key(&key_id));
        assert!(em.get_key_info(&key_id).is_none());
    }
    
    #[test]
    fn test_encryption_stats() {
        let mut em = EncryptionManager::new();
        
        em.generate_key(KeyType::DEK, EncryptionLevel::Maximum, "test", None);
        em.generate_key(KeyType::DEK, EncryptionLevel::Standard, "test2", None);
        
        let stats = em.stats();
        assert_eq!(stats.total_keys, 2);
    }
}
