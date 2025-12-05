// Security module for Kāraṇa OS
// Handles authentication, encryption, and access control

pub mod authentication;
pub mod encryption;
pub mod biometric;
pub mod access_control;
pub mod secure_storage;

pub use authentication::*;
pub use encryption::*;
pub use biometric::*;
pub use access_control::*;
pub use secure_storage::*;

use std::collections::HashMap;

/// Main security manager
pub struct SecurityManager {
    config: SecurityConfig,
    auth_manager: AuthenticationManager,
    encryption_service: EncryptionService,
    biometric_auth: BiometricAuth,
    access_controller: AccessController,
    secure_store: SecureStorage,
    sessions: HashMap<String, SecuritySession>,
    audit_log: Vec<SecurityEvent>,
    threat_detector: ThreatDetector,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Require authentication on boot
    pub require_boot_auth: bool,
    /// Auto-lock timeout in seconds
    pub auto_lock_timeout: u64,
    /// Enable biometric authentication
    pub enable_biometric: bool,
    /// Enable PIN/password
    pub enable_pin: bool,
    /// Minimum PIN length
    pub min_pin_length: u32,
    /// Maximum authentication attempts
    pub max_auth_attempts: u32,
    /// Lockout duration after failed attempts (seconds)
    pub lockout_duration_secs: u64,
    /// Enable secure enclave
    pub use_secure_enclave: bool,
    /// Encryption algorithm
    pub encryption_algorithm: EncryptionAlgorithm,
    /// Key derivation iterations
    pub key_iterations: u32,
    /// Enable threat detection
    pub enable_threat_detection: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_boot_auth: true,
            auto_lock_timeout: 300, // 5 minutes
            enable_biometric: true,
            enable_pin: true,
            min_pin_length: 6,
            max_auth_attempts: 5,
            lockout_duration_secs: 300, // 5 minutes
            use_secure_enclave: true,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_iterations: 100000,
            enable_threat_detection: true,
        }
    }
}

/// Encryption algorithms supported
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionAlgorithm {
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
    XChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    pub fn key_size(&self) -> usize {
        match self {
            Self::Aes128Gcm => 16,
            Self::Aes256Gcm | Self::ChaCha20Poly1305 | Self::XChaCha20Poly1305 => 32,
        }
    }
    
    pub fn nonce_size(&self) -> usize {
        match self {
            Self::Aes128Gcm | Self::Aes256Gcm | Self::ChaCha20Poly1305 => 12,
            Self::XChaCha20Poly1305 => 24,
        }
    }
}

/// Active security session
#[derive(Debug, Clone)]
pub struct SecuritySession {
    pub id: String,
    pub user_id: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub auth_methods: Vec<AuthMethod>,
    pub permissions: Vec<Permission>,
    pub is_locked: bool,
}

/// Authentication method used
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    Pin,
    Biometric(BiometricType),
    Pattern,
    VoiceAuth,
    DevicePairing,
    NfcToken,
}

/// Security event for audit logging
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: u64,
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub details: String,
    pub severity: SecuritySeverity,
}

/// Types of security events
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityEventType {
    AuthenticationAttempt,
    AuthenticationSuccess,
    AuthenticationFailed,
    SessionCreated,
    SessionDestroyed,
    SessionLocked,
    SessionUnlocked,
    PermissionGranted,
    PermissionDenied,
    DataAccess,
    DataEncrypted,
    DataDecrypted,
    ThreatDetected,
    SecurityPolicyChanged,
    DevicePaired,
    DeviceUnpaired,
}

/// Security event severity
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: config.clone(),
            auth_manager: AuthenticationManager::new(config.clone()),
            encryption_service: EncryptionService::new(config.encryption_algorithm),
            biometric_auth: BiometricAuth::new(),
            access_controller: AccessController::new(),
            secure_store: SecureStorage::new(config.use_secure_enclave),
            sessions: HashMap::new(),
            audit_log: Vec::new(),
            threat_detector: ThreatDetector::new(config.enable_threat_detection),
        }
    }
    
    /// Authenticate user
    pub fn authenticate(&mut self, credentials: &Credentials) -> Result<String, SecurityError> {
        // Log attempt
        self.log_event(SecurityEventType::AuthenticationAttempt, None, 
                      &format!("Auth method: {:?}", credentials.method()), SecuritySeverity::Info);
        
        // Check lockout
        if self.auth_manager.is_locked_out(&credentials.user_id) {
            self.log_event(SecurityEventType::AuthenticationFailed, Some(&credentials.user_id),
                          "Account locked out", SecuritySeverity::Warning);
            return Err(SecurityError::AccountLockedOut);
        }
        
        // Verify credentials
        let valid = match credentials {
            Credentials::Pin { user_id, pin } => {
                self.auth_manager.verify_pin(user_id, pin)
            }
            Credentials::Biometric { user_id, data } => {
                self.biometric_auth.verify(user_id, data)
            }
            Credentials::Pattern { user_id, pattern } => {
                self.auth_manager.verify_pattern(user_id, pattern)
            }
        };
        
        if valid {
            // Create session
            let session_id = self.create_session(&credentials.user_id(), credentials.method());
            
            self.log_event(SecurityEventType::AuthenticationSuccess, Some(&credentials.user_id()),
                          "Authentication successful", SecuritySeverity::Info);
            
            Ok(session_id)
        } else {
            self.auth_manager.record_failed_attempt(&credentials.user_id());
            
            self.log_event(SecurityEventType::AuthenticationFailed, Some(&credentials.user_id()),
                          "Invalid credentials", SecuritySeverity::Warning);
            
            Err(SecurityError::InvalidCredentials)
        }
    }
    
    /// Create a new session
    fn create_session(&mut self, user_id: &str, auth_method: AuthMethod) -> String {
        let session_id = self.generate_session_id();
        let now = self.current_timestamp();
        
        let session = SecuritySession {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            last_activity: now,
            auth_methods: vec![auth_method],
            permissions: self.access_controller.get_user_permissions(user_id),
            is_locked: false,
        };
        
        self.sessions.insert(session_id.clone(), session);
        
        self.log_event(SecurityEventType::SessionCreated, Some(user_id),
                      &format!("Session {} created", session_id), SecuritySeverity::Info);
        
        session_id
    }
    
    /// Lock a session
    pub fn lock_session(&mut self, session_id: &str) -> Result<(), SecurityError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(SecurityError::SessionNotFound)?;
        
        session.is_locked = true;
        
        self.log_event(SecurityEventType::SessionLocked, Some(&session.user_id.clone()),
                      "Session locked", SecuritySeverity::Info);
        
        Ok(())
    }
    
    /// Unlock a session with re-authentication
    pub fn unlock_session(&mut self, session_id: &str, credentials: &Credentials) 
        -> Result<(), SecurityError> 
    {
        let session = self.sessions.get(session_id)
            .ok_or(SecurityError::SessionNotFound)?;
        
        if session.user_id != credentials.user_id() {
            return Err(SecurityError::InvalidCredentials);
        }
        
        // Verify credentials
        let valid = match credentials {
            Credentials::Pin { user_id, pin } => {
                self.auth_manager.verify_pin(user_id, pin)
            }
            Credentials::Biometric { user_id, data } => {
                self.biometric_auth.verify(user_id, data)
            }
            Credentials::Pattern { user_id, pattern } => {
                self.auth_manager.verify_pattern(user_id, pattern)
            }
        };
        
        if valid {
            let session = self.sessions.get_mut(session_id).unwrap();
            session.is_locked = false;
            session.last_activity = self.current_timestamp();
            
            if !session.auth_methods.contains(&credentials.method()) {
                session.auth_methods.push(credentials.method());
            }
            
            let user_id = session.user_id.clone();
            self.log_event(SecurityEventType::SessionUnlocked, Some(&user_id),
                          "Session unlocked", SecuritySeverity::Info);
            
            Ok(())
        } else {
            self.auth_manager.record_failed_attempt(&credentials.user_id());
            Err(SecurityError::InvalidCredentials)
        }
    }
    
    /// Terminate a session
    pub fn terminate_session(&mut self, session_id: &str) -> Result<(), SecurityError> {
        let session = self.sessions.remove(session_id)
            .ok_or(SecurityError::SessionNotFound)?;
        
        self.log_event(SecurityEventType::SessionDestroyed, Some(&session.user_id),
                      "Session terminated", SecuritySeverity::Info);
        
        Ok(())
    }
    
    /// Check if session is valid and active
    pub fn validate_session(&mut self, session_id: &str) -> Result<bool, SecurityError> {
        let now = self.current_timestamp();
        
        let session = self.sessions.get_mut(session_id)
            .ok_or(SecurityError::SessionNotFound)?;
        
        // Check if locked
        if session.is_locked {
            return Ok(false);
        }
        
        // Check auto-lock timeout
        if now - session.last_activity > self.config.auto_lock_timeout {
            session.is_locked = true;
            self.log_event(SecurityEventType::SessionLocked, Some(&session.user_id.clone()),
                          "Session auto-locked due to inactivity", SecuritySeverity::Info);
            return Ok(false);
        }
        
        // Update activity
        session.last_activity = now;
        
        Ok(true)
    }
    
    /// Check permission for session
    pub fn check_permission(&self, session_id: &str, permission: &Permission) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            if session.is_locked {
                return false;
            }
            session.permissions.contains(permission)
        } else {
            false
        }
    }
    
    /// Encrypt data
    pub fn encrypt(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>, SecurityError> {
        let key = self.secure_store.get_key(key_id)
            .ok_or(SecurityError::KeyNotFound)?;
        
        self.encryption_service.encrypt(data, &key)
    }
    
    /// Decrypt data
    pub fn decrypt(&self, encrypted: &[u8], key_id: &str) -> Result<Vec<u8>, SecurityError> {
        let key = self.secure_store.get_key(key_id)
            .ok_or(SecurityError::KeyNotFound)?;
        
        self.encryption_service.decrypt(encrypted, &key)
    }
    
    /// Generate and store a new encryption key
    pub fn generate_key(&mut self, key_id: &str) -> Result<(), SecurityError> {
        let key = self.encryption_service.generate_key();
        self.secure_store.store_key(key_id, &key)
    }
    
    /// Register biometric data
    pub fn enroll_biometric(&mut self, user_id: &str, data: &BiometricData) 
        -> Result<(), SecurityError> 
    {
        self.biometric_auth.enroll(user_id, data)
    }
    
    /// Get audit log
    pub fn audit_log(&self) -> &[SecurityEvent] {
        &self.audit_log
    }
    
    /// Get events by severity
    pub fn events_by_severity(&self, min_severity: SecuritySeverity) -> Vec<&SecurityEvent> {
        self.audit_log
            .iter()
            .filter(|e| e.severity >= min_severity)
            .collect()
    }
    
    /// Check for active threats
    pub fn check_threats(&mut self) -> Vec<ThreatAlert> {
        if self.config.enable_threat_detection {
            self.threat_detector.analyze(&self.audit_log)
        } else {
            Vec::new()
        }
    }
    
    /// Set PIN for user
    pub fn set_pin(&mut self, user_id: &str, pin: &str) -> Result<(), SecurityError> {
        if pin.len() < self.config.min_pin_length as usize {
            return Err(SecurityError::PinTooShort);
        }
        
        self.auth_manager.set_pin(user_id, pin)
    }
    
    /// Change PIN
    pub fn change_pin(&mut self, user_id: &str, old_pin: &str, new_pin: &str) 
        -> Result<(), SecurityError> 
    {
        if !self.auth_manager.verify_pin(user_id, old_pin) {
            return Err(SecurityError::InvalidCredentials);
        }
        
        self.set_pin(user_id, new_pin)
    }
    
    /// Get session info
    pub fn get_session(&self, session_id: &str) -> Option<&SecuritySession> {
        self.sessions.get(session_id)
    }
    
    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<&SecuritySession> {
        self.sessions.values().filter(|s| !s.is_locked).collect()
    }
    
    // Helper methods
    
    fn log_event(&mut self, event_type: SecurityEventType, user_id: Option<&str>, 
                 details: &str, severity: SecuritySeverity) 
    {
        self.audit_log.push(SecurityEvent {
            timestamp: self.current_timestamp(),
            event_type,
            user_id: user_id.map(String::from),
            session_id: None,
            details: details.to_string(),
            severity,
        });
    }
    
    fn generate_session_id(&self) -> String {
        format!("sess_{}", self.current_timestamp())
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
}

/// User credentials for authentication
#[derive(Debug, Clone)]
pub enum Credentials {
    Pin { user_id: String, pin: String },
    Biometric { user_id: String, data: BiometricData },
    Pattern { user_id: String, pattern: Vec<u8> },
}

impl Credentials {
    pub fn user_id(&self) -> String {
        match self {
            Self::Pin { user_id, .. } => user_id.clone(),
            Self::Biometric { user_id, .. } => user_id.clone(),
            Self::Pattern { user_id, .. } => user_id.clone(),
        }
    }
    
    pub fn method(&self) -> AuthMethod {
        match self {
            Self::Pin { .. } => AuthMethod::Pin,
            Self::Biometric { data, .. } => AuthMethod::Biometric(data.biometric_type),
            Self::Pattern { .. } => AuthMethod::Pattern,
        }
    }
}

/// Security-specific errors
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    InvalidCredentials,
    AccountLockedOut,
    SessionNotFound,
    SessionExpired,
    KeyNotFound,
    EncryptionFailed,
    DecryptionFailed,
    PinTooShort,
    BiometricNotEnrolled,
    BiometricEnrollmentFailed,
    PermissionDenied,
    StorageError,
    SecureEnclaveError,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::AccountLockedOut => write!(f, "Account is locked out"),
            Self::SessionNotFound => write!(f, "Session not found"),
            Self::SessionExpired => write!(f, "Session has expired"),
            Self::KeyNotFound => write!(f, "Encryption key not found"),
            Self::EncryptionFailed => write!(f, "Encryption failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed"),
            Self::PinTooShort => write!(f, "PIN is too short"),
            Self::BiometricNotEnrolled => write!(f, "Biometric not enrolled"),
            Self::BiometricEnrollmentFailed => write!(f, "Biometric enrollment failed"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::StorageError => write!(f, "Secure storage error"),
            Self::SecureEnclaveError => write!(f, "Secure enclave error"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Threat alert
#[derive(Debug, Clone)]
pub struct ThreatAlert {
    pub threat_type: ThreatType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub timestamp: u64,
    pub recommended_action: String,
}

/// Types of threats
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatType {
    BruteForceAttempt,
    UnauthorizedAccess,
    DataExfiltration,
    MaliciousApp,
    NetworkAttack,
    PhysicalTampering,
    AnomalousActivity,
}

/// Threat detector
pub struct ThreatDetector {
    enabled: bool,
    thresholds: ThreatThresholds,
}

#[derive(Debug, Clone)]
pub struct ThreatThresholds {
    pub failed_auth_threshold: u32,
    pub failed_auth_window_secs: u64,
    pub suspicious_access_threshold: u32,
}

impl Default for ThreatThresholds {
    fn default() -> Self {
        Self {
            failed_auth_threshold: 5,
            failed_auth_window_secs: 300,
            suspicious_access_threshold: 10,
        }
    }
}

impl ThreatDetector {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            thresholds: ThreatThresholds::default(),
        }
    }
    
    pub fn analyze(&self, events: &[SecurityEvent]) -> Vec<ThreatAlert> {
        if !self.enabled {
            return Vec::new();
        }
        
        let mut alerts = Vec::new();
        
        // Check for brute force
        let failed_auths = events
            .iter()
            .filter(|e| e.event_type == SecurityEventType::AuthenticationFailed)
            .count();
        
        if failed_auths >= self.thresholds.failed_auth_threshold as usize {
            alerts.push(ThreatAlert {
                threat_type: ThreatType::BruteForceAttempt,
                severity: SecuritySeverity::Warning,
                description: format!("{} failed authentication attempts detected", failed_auths),
                timestamp: 0,
                recommended_action: "Enable additional authentication factors".to_string(),
            });
        }
        
        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.require_boot_auth);
        assert!(config.enable_biometric);
        assert_eq!(config.min_pin_length, 6);
    }
    
    #[test]
    fn test_encryption_algorithm() {
        assert_eq!(EncryptionAlgorithm::Aes128Gcm.key_size(), 16);
        assert_eq!(EncryptionAlgorithm::Aes256Gcm.key_size(), 32);
        assert_eq!(EncryptionAlgorithm::XChaCha20Poly1305.nonce_size(), 24);
    }
    
    #[test]
    fn test_security_manager_creation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        
        assert!(manager.active_sessions().is_empty());
    }
    
    #[test]
    fn test_credentials() {
        let cred = Credentials::Pin {
            user_id: "user1".to_string(),
            pin: "123456".to_string(),
        };
        
        assert_eq!(cred.user_id(), "user1");
        assert_eq!(cred.method(), AuthMethod::Pin);
    }
    
    #[test]
    fn test_security_severity_ordering() {
        assert!(SecuritySeverity::Info < SecuritySeverity::Warning);
        assert!(SecuritySeverity::Warning < SecuritySeverity::Critical);
        assert!(SecuritySeverity::Critical < SecuritySeverity::Emergency);
    }
    
    #[test]
    fn test_security_error_display() {
        let error = SecurityError::InvalidCredentials;
        assert_eq!(error.to_string(), "Invalid credentials");
    }
    
    #[test]
    fn test_threat_detector() {
        let detector = ThreatDetector::new(true);
        
        let events = vec![
            SecurityEvent {
                timestamp: 0,
                event_type: SecurityEventType::AuthenticationFailed,
                user_id: Some("user1".to_string()),
                session_id: None,
                details: "Failed".to_string(),
                severity: SecuritySeverity::Warning,
            };
            10
        ];
        
        let alerts = detector.analyze(&events);
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].threat_type, ThreatType::BruteForceAttempt);
    }
    
    #[test]
    fn test_threat_detector_disabled() {
        let detector = ThreatDetector::new(false);
        let events = vec![];
        
        let alerts = detector.analyze(&events);
        assert!(alerts.is_empty());
    }
}
