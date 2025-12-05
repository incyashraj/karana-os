// Secure storage for Kāraṇa OS

use super::SecurityError;
use std::collections::HashMap;

/// Secure storage for sensitive data
pub struct SecureStorage {
    use_secure_enclave: bool,
    keys: HashMap<String, SecureKey>,
    secrets: HashMap<String, SecureSecret>,
    certificates: HashMap<String, Certificate>,
    encryption_key: Option<Vec<u8>>,
}

/// Securely stored key
#[derive(Debug, Clone)]
pub struct SecureKey {
    pub id: String,
    pub key_type: KeyType,
    pub key_data: Vec<u8>,
    pub created_at: u64,
    pub is_extractable: bool,
    pub usage: KeyUsage,
}

/// Type of key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    Symmetric,
    PrivateKey,
    PublicKey,
    SharedSecret,
}

/// Allowed key usage
#[derive(Debug, Clone)]
pub struct KeyUsage {
    pub encrypt: bool,
    pub decrypt: bool,
    pub sign: bool,
    pub verify: bool,
    pub derive: bool,
    pub wrap: bool,
    pub unwrap: bool,
}

impl Default for KeyUsage {
    fn default() -> Self {
        Self {
            encrypt: true,
            decrypt: true,
            sign: false,
            verify: false,
            derive: false,
            wrap: false,
            unwrap: false,
        }
    }
}

/// Securely stored secret
#[derive(Debug, Clone)]
pub struct SecureSecret {
    pub id: String,
    pub encrypted_value: Vec<u8>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub access_count: u32,
    pub last_accessed: Option<u64>,
}

/// Certificate storage
#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: String,
    pub certificate_type: CertificateType,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: u64,
    pub not_after: u64,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub is_trusted: bool,
}

/// Certificate type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CertificateType {
    RootCA,
    IntermediateCA,
    EndEntity,
    SelfSigned,
}

impl SecureStorage {
    pub fn new(use_secure_enclave: bool) -> Self {
        Self {
            use_secure_enclave,
            keys: HashMap::new(),
            secrets: HashMap::new(),
            certificates: HashMap::new(),
            encryption_key: None,
        }
    }
    
    /// Initialize storage with master key
    pub fn initialize(&mut self, master_key: Vec<u8>) {
        self.encryption_key = Some(master_key);
    }
    
    /// Check if storage is initialized
    pub fn is_initialized(&self) -> bool {
        self.encryption_key.is_some()
    }
    
    /// Store an encryption key
    pub fn store_key(&mut self, key_id: &str, key_data: &[u8]) -> Result<(), SecurityError> {
        let encrypted = self.encrypt_data(key_data)?;
        
        self.keys.insert(key_id.to_string(), SecureKey {
            id: key_id.to_string(),
            key_type: KeyType::Symmetric,
            key_data: encrypted,
            created_at: self.current_timestamp(),
            is_extractable: true,
            usage: KeyUsage::default(),
        });
        
        Ok(())
    }
    
    /// Retrieve an encryption key
    pub fn get_key(&self, key_id: &str) -> Option<Vec<u8>> {
        self.keys.get(key_id)
            .and_then(|k| self.decrypt_data(&k.key_data).ok())
    }
    
    /// Delete a key
    pub fn delete_key(&mut self, key_id: &str) -> bool {
        self.keys.remove(key_id).is_some()
    }
    
    /// Store a secret
    pub fn store_secret(&mut self, secret_id: &str, value: &[u8]) -> Result<(), SecurityError> {
        let encrypted = self.encrypt_data(value)?;
        
        self.secrets.insert(secret_id.to_string(), SecureSecret {
            id: secret_id.to_string(),
            encrypted_value: encrypted,
            created_at: self.current_timestamp(),
            expires_at: None,
            access_count: 0,
            last_accessed: None,
        });
        
        Ok(())
    }
    
    /// Retrieve a secret
    pub fn get_secret(&mut self, secret_id: &str) -> Option<Vec<u8>> {
        // Update access tracking
        if let Some(secret) = self.secrets.get_mut(secret_id) {
            secret.access_count += 1;
            secret.last_accessed = Some(self.current_timestamp());
            
            // Check expiration
            if let Some(expires) = secret.expires_at {
                if self.current_timestamp() > expires {
                    return None;
                }
            }
            
            self.decrypt_data(&secret.encrypted_value).ok()
        } else {
            None
        }
    }
    
    /// Delete a secret
    pub fn delete_secret(&mut self, secret_id: &str) -> bool {
        self.secrets.remove(secret_id).is_some()
    }
    
    /// Store with expiration
    pub fn store_secret_with_expiry(
        &mut self, 
        secret_id: &str, 
        value: &[u8],
        expires_in_secs: u64,
    ) -> Result<(), SecurityError> {
        let encrypted = self.encrypt_data(value)?;
        let now = self.current_timestamp();
        
        self.secrets.insert(secret_id.to_string(), SecureSecret {
            id: secret_id.to_string(),
            encrypted_value: encrypted,
            created_at: now,
            expires_at: Some(now + expires_in_secs),
            access_count: 0,
            last_accessed: None,
        });
        
        Ok(())
    }
    
    /// Store a certificate
    pub fn store_certificate(&mut self, cert: Certificate) {
        self.certificates.insert(cert.id.clone(), cert);
    }
    
    /// Get a certificate
    pub fn get_certificate(&self, cert_id: &str) -> Option<&Certificate> {
        self.certificates.get(cert_id)
    }
    
    /// Delete a certificate
    pub fn delete_certificate(&mut self, cert_id: &str) -> bool {
        self.certificates.remove(cert_id).is_some()
    }
    
    /// Get trusted root certificates
    pub fn trusted_roots(&self) -> Vec<&Certificate> {
        self.certificates.values()
            .filter(|c| c.certificate_type == CertificateType::RootCA && c.is_trusted)
            .collect()
    }
    
    /// Verify certificate chain
    pub fn verify_certificate(&self, cert: &Certificate) -> CertificateVerificationResult {
        let now = self.current_timestamp();
        
        let mut result = CertificateVerificationResult {
            valid: true,
            errors: Vec::new(),
            chain_length: 0,
        };
        
        // Check validity period
        if now < cert.not_before {
            result.valid = false;
            result.errors.push("Certificate not yet valid".to_string());
        }
        
        if now > cert.not_after {
            result.valid = false;
            result.errors.push("Certificate has expired".to_string());
        }
        
        // Check if issuer is trusted (simplified)
        if cert.certificate_type != CertificateType::SelfSigned {
            let issuer_trusted = self.certificates.values()
                .any(|c| c.subject == cert.issuer && c.is_trusted);
            
            if !issuer_trusted {
                result.valid = false;
                result.errors.push("Issuer not trusted".to_string());
            }
        }
        
        result
    }
    
    /// Check if key exists
    pub fn has_key(&self, key_id: &str) -> bool {
        self.keys.contains_key(key_id)
    }
    
    /// Check if secret exists
    pub fn has_secret(&self, secret_id: &str) -> bool {
        self.secrets.contains_key(secret_id)
    }
    
    /// List all key IDs
    pub fn list_keys(&self) -> Vec<&String> {
        self.keys.keys().collect()
    }
    
    /// List all secret IDs
    pub fn list_secrets(&self) -> Vec<&String> {
        self.secrets.keys().collect()
    }
    
    /// Get storage statistics
    pub fn statistics(&self) -> StorageStatistics {
        StorageStatistics {
            key_count: self.keys.len(),
            secret_count: self.secrets.len(),
            certificate_count: self.certificates.len(),
            uses_secure_enclave: self.use_secure_enclave,
        }
    }
    
    /// Clear all expired secrets
    pub fn clear_expired(&mut self) -> usize {
        let now = self.current_timestamp();
        let initial = self.secrets.len();
        
        self.secrets.retain(|_, secret| {
            secret.expires_at.map(|exp| exp > now).unwrap_or(true)
        });
        
        initial - self.secrets.len()
    }
    
    // Private helpers
    
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if self.use_secure_enclave {
            // Would use secure enclave for encryption
            Ok(data.to_vec())
        } else {
            // Software encryption
            let key = self.encryption_key.as_ref()
                .ok_or(SecurityError::StorageError)?;
            
            // Simulated encryption
            let mut encrypted = data.to_vec();
            for (i, byte) in encrypted.iter_mut().enumerate() {
                *byte ^= key[i % key.len()];
            }
            
            Ok(encrypted)
        }
    }
    
    fn decrypt_data(&self, encrypted: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if self.use_secure_enclave {
            // Would use secure enclave for decryption
            Ok(encrypted.to_vec())
        } else {
            let key = self.encryption_key.as_ref()
                .ok_or(SecurityError::StorageError)?;
            
            // Simulated decryption (XOR is symmetric)
            let mut decrypted = encrypted.to_vec();
            for (i, byte) in decrypted.iter_mut().enumerate() {
                *byte ^= key[i % key.len()];
            }
            
            Ok(decrypted)
        }
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
}

/// Certificate verification result
#[derive(Debug, Clone)]
pub struct CertificateVerificationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub chain_length: u32,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub key_count: usize,
    pub secret_count: usize,
    pub certificate_count: usize,
    pub uses_secure_enclave: bool,
}

/// Keychain interface for credential storage
pub struct Keychain {
    storage: SecureStorage,
    app_credentials: HashMap<String, HashMap<String, Credential>>,
}

/// Stored credential
#[derive(Debug, Clone)]
pub struct Credential {
    pub id: String,
    pub service: String,
    pub account: String,
    pub password: Vec<u8>,
    pub created_at: u64,
    pub modified_at: u64,
    pub access_group: Option<String>,
}

impl Keychain {
    pub fn new(use_secure_enclave: bool) -> Self {
        Self {
            storage: SecureStorage::new(use_secure_enclave),
            app_credentials: HashMap::new(),
        }
    }
    
    /// Initialize keychain
    pub fn initialize(&mut self, master_key: Vec<u8>) {
        self.storage.initialize(master_key);
    }
    
    /// Store a credential
    pub fn store_credential(
        &mut self,
        app_id: &str,
        service: &str,
        account: &str,
        password: &[u8],
    ) -> Result<(), SecurityError> {
        let credential = Credential {
            id: format!("{}_{}", service, account),
            service: service.to_string(),
            account: account.to_string(),
            password: password.to_vec(),
            created_at: 0,
            modified_at: 0,
            access_group: None,
        };
        
        // Store encrypted in secure storage
        let secret_id = format!("{}_{}_{}", app_id, service, account);
        self.storage.store_secret(&secret_id, password)?;
        
        // Store metadata
        self.app_credentials
            .entry(app_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(credential.id.clone(), credential);
        
        Ok(())
    }
    
    /// Get a credential
    pub fn get_credential(&mut self, app_id: &str, service: &str, account: &str) 
        -> Option<Vec<u8>> 
    {
        let secret_id = format!("{}_{}_{}", app_id, service, account);
        self.storage.get_secret(&secret_id)
    }
    
    /// Delete a credential
    pub fn delete_credential(&mut self, app_id: &str, service: &str, account: &str) -> bool {
        let secret_id = format!("{}_{}_{}", app_id, service, account);
        let cred_id = format!("{}_{}", service, account);
        
        if let Some(creds) = self.app_credentials.get_mut(app_id) {
            creds.remove(&cred_id);
        }
        
        self.storage.delete_secret(&secret_id)
    }
    
    /// List credentials for app
    pub fn list_credentials(&self, app_id: &str) -> Vec<&Credential> {
        self.app_credentials.get(app_id)
            .map(|creds| creds.values().collect())
            .unwrap_or_default()
    }
    
    /// Delete all credentials for app
    pub fn clear_app_credentials(&mut self, app_id: &str) -> usize {
        let creds = self.app_credentials.remove(app_id);
        creds.map(|c| c.len()).unwrap_or(0)
    }
}

/// Secure note storage
pub struct SecureNotes {
    storage: SecureStorage,
    notes: HashMap<String, SecureNote>,
}

/// A secure note
#[derive(Debug, Clone)]
pub struct SecureNote {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub is_locked: bool,
}

impl SecureNotes {
    pub fn new(use_secure_enclave: bool) -> Self {
        Self {
            storage: SecureStorage::new(use_secure_enclave),
            notes: HashMap::new(),
        }
    }
    
    /// Initialize secure notes
    pub fn initialize(&mut self, master_key: Vec<u8>) {
        self.storage.initialize(master_key);
    }
    
    /// Create a note
    pub fn create_note(&mut self, title: &str, content: &str) -> Result<String, SecurityError> {
        let note_id = format!("note_{}", self.notes.len());
        
        self.storage.store_secret(&note_id, content.as_bytes())?;
        
        self.notes.insert(note_id.clone(), SecureNote {
            id: note_id.clone(),
            title: title.to_string(),
            created_at: 0,
            modified_at: 0,
            is_locked: false,
        });
        
        Ok(note_id)
    }
    
    /// Read a note
    pub fn read_note(&mut self, note_id: &str) -> Option<String> {
        if let Some(note) = self.notes.get(note_id) {
            if note.is_locked {
                return None;
            }
        }
        
        self.storage.get_secret(note_id)
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }
    
    /// Update a note
    pub fn update_note(&mut self, note_id: &str, content: &str) -> Result<(), SecurityError> {
        if let Some(note) = self.notes.get_mut(note_id) {
            if note.is_locked {
                return Err(SecurityError::PermissionDenied);
            }
            note.modified_at = 0; // Would be actual timestamp
        }
        
        self.storage.store_secret(note_id, content.as_bytes())
    }
    
    /// Delete a note
    pub fn delete_note(&mut self, note_id: &str) -> bool {
        self.notes.remove(note_id);
        self.storage.delete_secret(note_id)
    }
    
    /// Lock a note
    pub fn lock_note(&mut self, note_id: &str) {
        if let Some(note) = self.notes.get_mut(note_id) {
            note.is_locked = true;
        }
    }
    
    /// Unlock a note
    pub fn unlock_note(&mut self, note_id: &str) {
        if let Some(note) = self.notes.get_mut(note_id) {
            note.is_locked = false;
        }
    }
    
    /// List all notes
    pub fn list_notes(&self) -> Vec<&SecureNote> {
        self.notes.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_storage_creation() {
        let storage = SecureStorage::new(false);
        assert!(!storage.is_initialized());
    }
    
    #[test]
    fn test_initialize_storage() {
        let mut storage = SecureStorage::new(false);
        storage.initialize(vec![0u8; 32]);
        assert!(storage.is_initialized());
    }
    
    #[test]
    fn test_store_and_get_key() {
        let mut storage = SecureStorage::new(false);
        storage.initialize(vec![0u8; 32]);
        
        let key_data = vec![1, 2, 3, 4, 5];
        storage.store_key("test_key", &key_data).unwrap();
        
        assert!(storage.has_key("test_key"));
        
        let retrieved = storage.get_key("test_key").unwrap();
        assert_eq!(retrieved, key_data);
    }
    
    #[test]
    fn test_store_and_get_secret() {
        let mut storage = SecureStorage::new(false);
        storage.initialize(vec![0u8; 32]);
        
        let secret = b"my secret value";
        storage.store_secret("my_secret", secret).unwrap();
        
        let retrieved = storage.get_secret("my_secret").unwrap();
        assert_eq!(retrieved, secret);
    }
    
    #[test]
    fn test_delete_key() {
        let mut storage = SecureStorage::new(false);
        storage.initialize(vec![0u8; 32]);
        
        storage.store_key("key1", &[1, 2, 3]).unwrap();
        assert!(storage.has_key("key1"));
        
        assert!(storage.delete_key("key1"));
        assert!(!storage.has_key("key1"));
    }
    
    #[test]
    fn test_certificate_storage() {
        let mut storage = SecureStorage::new(false);
        
        storage.store_certificate(Certificate {
            id: "cert1".to_string(),
            certificate_type: CertificateType::RootCA,
            subject: "Test CA".to_string(),
            issuer: "Test CA".to_string(),
            serial_number: "001".to_string(),
            not_before: 0,
            not_after: u64::MAX,
            public_key: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            is_trusted: true,
        });
        
        assert!(storage.get_certificate("cert1").is_some());
        assert_eq!(storage.trusted_roots().len(), 1);
    }
    
    #[test]
    fn test_keychain() {
        let mut keychain = Keychain::new(false);
        keychain.initialize(vec![0u8; 32]);
        
        keychain.store_credential(
            "app1",
            "example.com",
            "user@example.com",
            b"password123"
        ).unwrap();
        
        let password = keychain.get_credential("app1", "example.com", "user@example.com");
        assert!(password.is_some());
        assert_eq!(password.unwrap(), b"password123");
    }
    
    #[test]
    fn test_keychain_delete() {
        let mut keychain = Keychain::new(false);
        keychain.initialize(vec![0u8; 32]);
        
        keychain.store_credential("app1", "service", "account", b"pass").unwrap();
        
        assert!(keychain.delete_credential("app1", "service", "account"));
        assert!(keychain.get_credential("app1", "service", "account").is_none());
    }
    
    #[test]
    fn test_secure_notes() {
        let mut notes = SecureNotes::new(false);
        notes.initialize(vec![0u8; 32]);
        
        let note_id = notes.create_note("Test Note", "Secret content").unwrap();
        
        let content = notes.read_note(&note_id).unwrap();
        assert_eq!(content, "Secret content");
    }
    
    #[test]
    fn test_secure_notes_lock() {
        let mut notes = SecureNotes::new(false);
        notes.initialize(vec![0u8; 32]);
        
        let note_id = notes.create_note("Test", "Content").unwrap();
        
        notes.lock_note(&note_id);
        assert!(notes.read_note(&note_id).is_none());
        
        notes.unlock_note(&note_id);
        assert!(notes.read_note(&note_id).is_some());
    }
    
    #[test]
    fn test_storage_statistics() {
        let mut storage = SecureStorage::new(true);
        storage.initialize(vec![0u8; 32]);
        
        storage.store_key("k1", &[1, 2, 3]).unwrap();
        storage.store_secret("s1", b"secret").unwrap();
        
        let stats = storage.statistics();
        assert_eq!(stats.key_count, 1);
        assert_eq!(stats.secret_count, 1);
        assert!(stats.uses_secure_enclave);
    }
    
    #[test]
    fn test_key_usage() {
        let usage = KeyUsage::default();
        assert!(usage.encrypt);
        assert!(usage.decrypt);
        assert!(!usage.sign);
    }
}
