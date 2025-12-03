//! Settings storage backends

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use super::schema::SettingValue;

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// In-memory only
    Memory,
    /// File-based
    File,
    /// Encrypted file
    EncryptedFile,
    /// Database
    Database,
}

/// Settings storage
#[derive(Debug)]
pub struct SettingsStorage {
    /// Backend type
    backend: StorageBackend,
    /// File path (for file backends)
    file_path: Option<PathBuf>,
    /// In-memory cache
    cache: HashMap<String, SettingValue>,
    /// Last sync time
    last_sync: std::time::Instant,
    /// Encryption key (if encrypted)
    encryption_key: Option<Vec<u8>>,
}

impl SettingsStorage {
    /// Create new settings storage
    pub fn new() -> Self {
        Self {
            backend: StorageBackend::Memory,
            file_path: None,
            cache: HashMap::new(),
            last_sync: std::time::Instant::now(),
            encryption_key: None,
        }
    }
    
    /// Create file-based storage
    pub fn with_file(path: PathBuf) -> Self {
        Self {
            backend: StorageBackend::File,
            file_path: Some(path),
            cache: HashMap::new(),
            last_sync: std::time::Instant::now(),
            encryption_key: None,
        }
    }
    
    /// Create encrypted file storage
    pub fn with_encrypted_file(path: PathBuf, key: Vec<u8>) -> Self {
        Self {
            backend: StorageBackend::EncryptedFile,
            file_path: Some(path),
            cache: HashMap::new(),
            last_sync: std::time::Instant::now(),
            encryption_key: Some(key),
        }
    }
    
    /// Get backend type
    pub fn backend(&self) -> StorageBackend {
        self.backend
    }
    
    /// Save settings to storage
    pub fn save(&mut self, settings: &HashMap<String, SettingValue>) -> Result<(), std::io::Error> {
        // Update cache
        self.cache = settings.clone();
        self.last_sync = std::time::Instant::now();
        
        match self.backend {
            StorageBackend::Memory => {
                // Already in cache
                Ok(())
            }
            StorageBackend::File => {
                if let Some(path) = &self.file_path {
                    let json = serde_json::to_string_pretty(settings)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    
                    // Create parent directories
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    
                    let mut file = std::fs::File::create(path)?;
                    file.write_all(json.as_bytes())?;
                    Ok(())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No file path configured",
                    ))
                }
            }
            StorageBackend::EncryptedFile => {
                if let (Some(path), Some(_key)) = (&self.file_path, &self.encryption_key) {
                    let json = serde_json::to_string(settings)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    
                    // In real implementation, would encrypt json with key
                    // For now, just store as-is (placeholder for encryption)
                    let encrypted = json.as_bytes().to_vec();
                    
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    
                    let mut file = std::fs::File::create(path)?;
                    file.write_all(&encrypted)?;
                    Ok(())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Missing file path or encryption key",
                    ))
                }
            }
            StorageBackend::Database => {
                // Database implementation would go here
                Ok(())
            }
        }
    }
    
    /// Load settings from storage
    pub fn load(&mut self) -> Result<HashMap<String, SettingValue>, std::io::Error> {
        match self.backend {
            StorageBackend::Memory => {
                Ok(self.cache.clone())
            }
            StorageBackend::File => {
                if let Some(path) = &self.file_path {
                    if !path.exists() {
                        return Ok(HashMap::new());
                    }
                    
                    let mut file = std::fs::File::open(path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    
                    let settings: HashMap<String, SettingValue> = serde_json::from_str(&contents)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    
                    self.cache = settings.clone();
                    self.last_sync = std::time::Instant::now();
                    
                    Ok(settings)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No file path configured",
                    ))
                }
            }
            StorageBackend::EncryptedFile => {
                if let (Some(path), Some(_key)) = (&self.file_path, &self.encryption_key) {
                    if !path.exists() {
                        return Ok(HashMap::new());
                    }
                    
                    let mut file = std::fs::File::open(path)?;
                    let mut encrypted = Vec::new();
                    file.read_to_end(&mut encrypted)?;
                    
                    // In real implementation, would decrypt with key
                    // For now, just read as-is (placeholder for decryption)
                    let json = String::from_utf8(encrypted)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    
                    let settings: HashMap<String, SettingValue> = serde_json::from_str(&json)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    
                    self.cache = settings.clone();
                    self.last_sync = std::time::Instant::now();
                    
                    Ok(settings)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Missing file path or encryption key",
                    ))
                }
            }
            StorageBackend::Database => {
                // Database implementation would go here
                Ok(self.cache.clone())
            }
        }
    }
    
    /// Delete storage
    pub fn delete(&mut self) -> Result<(), std::io::Error> {
        self.cache.clear();
        
        if let Some(path) = &self.file_path {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Check if storage exists
    pub fn exists(&self) -> bool {
        match self.backend {
            StorageBackend::Memory => !self.cache.is_empty(),
            StorageBackend::File | StorageBackend::EncryptedFile => {
                self.file_path.as_ref().map(|p| p.exists()).unwrap_or(false)
            }
            StorageBackend::Database => true, // Assume database exists
        }
    }
    
    /// Get cache
    pub fn cache(&self) -> &HashMap<String, SettingValue> {
        &self.cache
    }
}

impl Default for SettingsStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_storage() {
        let mut storage = SettingsStorage::new();
        
        let mut settings = HashMap::new();
        settings.insert("test.key".to_string(), SettingValue::Bool(true));
        
        assert!(storage.save(&settings).is_ok());
        
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.get("test.key"), Some(&SettingValue::Bool(true)));
    }
    
    #[test]
    fn test_backend_type() {
        let storage = SettingsStorage::new();
        assert_eq!(storage.backend(), StorageBackend::Memory);
    }
}
