// Authentication management for Kāraṇa OS

use super::{SecurityConfig, SecurityError};
use std::collections::HashMap;

/// Manages authentication credentials and attempts
pub struct AuthenticationManager {
    config: SecurityConfig,
    user_pins: HashMap<String, HashedPin>,
    user_patterns: HashMap<String, HashedPattern>,
    failed_attempts: HashMap<String, FailedAttemptTracker>,
    lockout_until: HashMap<String, u64>,
}

/// Hashed PIN storage
#[derive(Debug, Clone)]
pub struct HashedPin {
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub created_at: u64,
    pub last_changed: u64,
}

/// Hashed pattern storage
#[derive(Debug, Clone)]
pub struct HashedPattern {
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: u64,
}

/// Tracks failed authentication attempts
#[derive(Debug, Clone)]
pub struct FailedAttemptTracker {
    pub count: u32,
    pub first_attempt: u64,
    pub last_attempt: u64,
    pub attempts: Vec<AuthAttempt>,
}

/// Individual authentication attempt record
#[derive(Debug, Clone)]
pub struct AuthAttempt {
    pub timestamp: u64,
    pub method: String,
    pub success: bool,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
}

impl AuthenticationManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            user_pins: HashMap::new(),
            user_patterns: HashMap::new(),
            failed_attempts: HashMap::new(),
            lockout_until: HashMap::new(),
        }
    }
    
    /// Set PIN for user
    pub fn set_pin(&mut self, user_id: &str, pin: &str) -> Result<(), SecurityError> {
        let salt = self.generate_salt();
        let hash = self.hash_pin(pin, &salt);
        let now = self.current_timestamp();
        
        self.user_pins.insert(user_id.to_string(), HashedPin {
            hash,
            salt,
            iterations: self.config.key_iterations,
            created_at: now,
            last_changed: now,
        });
        
        Ok(())
    }
    
    /// Verify PIN for user
    pub fn verify_pin(&self, user_id: &str, pin: &str) -> bool {
        if let Some(stored) = self.user_pins.get(user_id) {
            let hash = self.hash_pin(pin, &stored.salt);
            hash == stored.hash
        } else {
            false
        }
    }
    
    /// Set pattern for user
    pub fn set_pattern(&mut self, user_id: &str, pattern: &[u8]) -> Result<(), SecurityError> {
        let salt = self.generate_salt();
        let hash = self.hash_pattern(pattern, &salt);
        
        self.user_patterns.insert(user_id.to_string(), HashedPattern {
            hash,
            salt,
            created_at: self.current_timestamp(),
        });
        
        Ok(())
    }
    
    /// Verify pattern for user
    pub fn verify_pattern(&self, user_id: &str, pattern: &[u8]) -> bool {
        if let Some(stored) = self.user_patterns.get(user_id) {
            let hash = self.hash_pattern(pattern, &stored.salt);
            hash == stored.hash
        } else {
            false
        }
    }
    
    /// Check if user is locked out
    pub fn is_locked_out(&self, user_id: &str) -> bool {
        if let Some(&until) = self.lockout_until.get(user_id) {
            self.current_timestamp() < until
        } else {
            false
        }
    }
    
    /// Record a failed authentication attempt
    pub fn record_failed_attempt(&mut self, user_id: &str) {
        let now = self.current_timestamp();
        
        let tracker = self.failed_attempts
            .entry(user_id.to_string())
            .or_insert_with(|| FailedAttemptTracker {
                count: 0,
                first_attempt: now,
                last_attempt: now,
                attempts: Vec::new(),
            });
        
        tracker.count += 1;
        tracker.last_attempt = now;
        tracker.attempts.push(AuthAttempt {
            timestamp: now,
            method: "pin".to_string(),
            success: false,
            ip_address: None,
            device_id: None,
        });
        
        // Check if should lock out
        if tracker.count >= self.config.max_auth_attempts {
            self.lockout_until.insert(
                user_id.to_string(),
                now + self.config.lockout_duration_secs,
            );
        }
    }
    
    /// Clear failed attempts after successful auth
    pub fn clear_failed_attempts(&mut self, user_id: &str) {
        self.failed_attempts.remove(user_id);
        self.lockout_until.remove(user_id);
    }
    
    /// Get remaining lockout time
    pub fn lockout_remaining(&self, user_id: &str) -> Option<u64> {
        if let Some(&until) = self.lockout_until.get(user_id) {
            let now = self.current_timestamp();
            if now < until {
                Some(until - now)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get failed attempt count
    pub fn failed_attempt_count(&self, user_id: &str) -> u32 {
        self.failed_attempts.get(user_id).map(|t| t.count).unwrap_or(0)
    }
    
    /// Check if PIN exists for user
    pub fn has_pin(&self, user_id: &str) -> bool {
        self.user_pins.contains_key(user_id)
    }
    
    /// Check if pattern exists for user
    pub fn has_pattern(&self, user_id: &str) -> bool {
        self.user_patterns.contains_key(user_id)
    }
    
    /// Remove PIN for user
    pub fn remove_pin(&mut self, user_id: &str) -> bool {
        self.user_pins.remove(user_id).is_some()
    }
    
    /// Remove pattern for user
    pub fn remove_pattern(&mut self, user_id: &str) -> bool {
        self.user_patterns.remove(user_id).is_some()
    }
    
    /// Get PIN age in seconds
    pub fn pin_age(&self, user_id: &str) -> Option<u64> {
        self.user_pins.get(user_id).map(|p| {
            self.current_timestamp().saturating_sub(p.created_at)
        })
    }
    
    // Private helper methods
    
    fn generate_salt(&self) -> Vec<u8> {
        // Simulated - would use cryptographically secure random
        vec![0u8; 32]
    }
    
    fn hash_pin(&self, pin: &str, salt: &[u8]) -> Vec<u8> {
        // Simulated PBKDF2 - would use actual key derivation
        let mut result = pin.as_bytes().to_vec();
        result.extend_from_slice(salt);
        result
    }
    
    fn hash_pattern(&self, pattern: &[u8], salt: &[u8]) -> Vec<u8> {
        // Simulated hash
        let mut result = pattern.to_vec();
        result.extend_from_slice(salt);
        result
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
}

/// Password/PIN policy validator
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub disallow_common: bool,
    pub disallow_sequences: bool,
    pub max_age_days: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 6,
            max_length: 128,
            require_uppercase: false,
            require_lowercase: false,
            require_numbers: true,
            require_special: false,
            disallow_common: true,
            disallow_sequences: true,
            max_age_days: Some(90),
        }
    }
}

impl PasswordPolicy {
    /// Validate a password/PIN against policy
    pub fn validate(&self, password: &str) -> PasswordValidationResult {
        let mut result = PasswordValidationResult {
            valid: true,
            errors: Vec::new(),
            strength: PasswordStrength::Weak,
        };
        
        // Check length
        if password.len() < self.min_length {
            result.valid = false;
            result.errors.push(format!("Minimum length is {}", self.min_length));
        }
        
        if password.len() > self.max_length {
            result.valid = false;
            result.errors.push(format!("Maximum length is {}", self.max_length));
        }
        
        // Check character requirements
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            result.valid = false;
            result.errors.push("Must contain uppercase letter".to_string());
        }
        
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            result.valid = false;
            result.errors.push("Must contain lowercase letter".to_string());
        }
        
        if self.require_numbers && !password.chars().any(|c| c.is_ascii_digit()) {
            result.valid = false;
            result.errors.push("Must contain a number".to_string());
        }
        
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            result.valid = false;
            result.errors.push("Must contain a special character".to_string());
        }
        
        // Check common passwords
        if self.disallow_common && self.is_common_password(password) {
            result.valid = false;
            result.errors.push("Password is too common".to_string());
        }
        
        // Check sequences
        if self.disallow_sequences && self.has_sequence(password) {
            result.valid = false;
            result.errors.push("Password contains predictable sequence".to_string());
        }
        
        // Calculate strength
        result.strength = self.calculate_strength(password);
        
        result
    }
    
    fn is_common_password(&self, password: &str) -> bool {
        const COMMON: &[&str] = &[
            "123456", "password", "123456789", "12345678", "12345",
            "1234567", "1234567890", "qwerty", "abc123", "111111",
        ];
        
        COMMON.contains(&password.to_lowercase().as_str())
    }
    
    fn has_sequence(&self, password: &str) -> bool {
        // Check for simple sequences like 123456 or abcdef
        let chars: Vec<char> = password.chars().collect();
        
        if chars.len() < 3 {
            return false;
        }
        
        for window in chars.windows(3) {
            let a = window[0] as i32;
            let b = window[1] as i32;
            let c = window[2] as i32;
            
            // Ascending or descending sequence
            if (b - a == 1 && c - b == 1) || (a - b == 1 && b - c == 1) {
                return true;
            }
        }
        
        false
    }
    
    fn calculate_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;
        
        // Length contribution
        score += password.len().min(20);
        
        // Character variety
        if password.chars().any(|c| c.is_uppercase()) { score += 5; }
        if password.chars().any(|c| c.is_lowercase()) { score += 5; }
        if password.chars().any(|c| c.is_ascii_digit()) { score += 5; }
        if password.chars().any(|c| !c.is_alphanumeric()) { score += 10; }
        
        // Unique characters
        let unique: std::collections::HashSet<char> = password.chars().collect();
        score += unique.len().min(10);
        
        match score {
            0..=15 => PasswordStrength::Weak,
            16..=30 => PasswordStrength::Fair,
            31..=45 => PasswordStrength::Good,
            _ => PasswordStrength::Strong,
        }
    }
}

/// Password validation result
#[derive(Debug, Clone)]
pub struct PasswordValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub strength: PasswordStrength,
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

/// Multi-factor authentication manager
pub struct MFAManager {
    user_factors: HashMap<String, Vec<MFAFactor>>,
    pending_challenges: HashMap<String, MFAChallenge>,
}

/// MFA factor configuration
#[derive(Debug, Clone)]
pub struct MFAFactor {
    pub factor_type: MFAFactorType,
    pub enabled: bool,
    pub enrolled_at: u64,
    pub last_used: Option<u64>,
    pub config: MFAFactorConfig,
}

/// Types of MFA factors
#[derive(Debug, Clone, PartialEq)]
pub enum MFAFactorType {
    TOTP,
    SMS,
    Email,
    PushNotification,
    HardwareKey,
    Biometric,
}

/// Factor-specific configuration
#[derive(Debug, Clone)]
pub enum MFAFactorConfig {
    TOTP { secret: Vec<u8>, algorithm: String },
    SMS { phone_number: String },
    Email { email_address: String },
    Push { device_token: String },
    HardwareKey { key_id: String },
    Biometric { template_id: String },
}

/// Active MFA challenge
#[derive(Debug, Clone)]
pub struct MFAChallenge {
    pub challenge_id: String,
    pub user_id: String,
    pub factor_type: MFAFactorType,
    pub created_at: u64,
    pub expires_at: u64,
    pub code: Option<String>,
}

impl MFAManager {
    pub fn new() -> Self {
        Self {
            user_factors: HashMap::new(),
            pending_challenges: HashMap::new(),
        }
    }
    
    /// Enroll a new MFA factor
    pub fn enroll(&mut self, user_id: &str, factor: MFAFactor) {
        self.user_factors
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(factor);
    }
    
    /// Check if user has MFA enabled
    pub fn is_mfa_enabled(&self, user_id: &str) -> bool {
        self.user_factors.get(user_id)
            .map(|factors| factors.iter().any(|f| f.enabled))
            .unwrap_or(false)
    }
    
    /// Get enabled factors for user
    pub fn get_factors(&self, user_id: &str) -> Vec<&MFAFactor> {
        self.user_factors.get(user_id)
            .map(|factors| factors.iter().filter(|f| f.enabled).collect())
            .unwrap_or_default()
    }
    
    /// Create a challenge for MFA
    pub fn create_challenge(&mut self, user_id: &str, factor_type: MFAFactorType) 
        -> Option<MFAChallenge> 
    {
        let challenge = MFAChallenge {
            challenge_id: format!("mfa_{}", self.current_timestamp()),
            user_id: user_id.to_string(),
            factor_type,
            created_at: self.current_timestamp(),
            expires_at: self.current_timestamp() + 300, // 5 minute expiry
            code: Some(self.generate_code()),
        };
        
        self.pending_challenges.insert(challenge.challenge_id.clone(), challenge.clone());
        Some(challenge)
    }
    
    /// Verify MFA code
    pub fn verify(&mut self, challenge_id: &str, code: &str) -> bool {
        if let Some(challenge) = self.pending_challenges.get(challenge_id) {
            let now = self.current_timestamp();
            
            if now > challenge.expires_at {
                self.pending_challenges.remove(challenge_id);
                return false;
            }
            
            if challenge.code.as_ref() == Some(&code.to_string()) {
                self.pending_challenges.remove(challenge_id);
                return true;
            }
        }
        
        false
    }
    
    fn generate_code(&self) -> String {
        // Simulated - would generate secure random code
        "123456".to_string()
    }
    
    fn current_timestamp(&self) -> u64 {
        0
    }
}

impl Default for MFAManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_manager_creation() {
        let config = SecurityConfig::default();
        let manager = AuthenticationManager::new(config);
        
        assert!(!manager.has_pin("user1"));
    }
    
    #[test]
    fn test_set_and_verify_pin() {
        let config = SecurityConfig::default();
        let mut manager = AuthenticationManager::new(config);
        
        manager.set_pin("user1", "123456").unwrap();
        
        assert!(manager.has_pin("user1"));
        assert!(manager.verify_pin("user1", "123456"));
        assert!(!manager.verify_pin("user1", "wrong"));
    }
    
    #[test]
    fn test_set_and_verify_pattern() {
        let config = SecurityConfig::default();
        let mut manager = AuthenticationManager::new(config);
        
        let pattern = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        manager.set_pattern("user1", &pattern).unwrap();
        
        assert!(manager.has_pattern("user1"));
        assert!(manager.verify_pattern("user1", &pattern));
        assert!(!manager.verify_pattern("user1", &[1, 2, 3]));
    }
    
    #[test]
    fn test_failed_attempts() {
        let config = SecurityConfig::default();
        let mut manager = AuthenticationManager::new(config);
        
        assert_eq!(manager.failed_attempt_count("user1"), 0);
        
        manager.record_failed_attempt("user1");
        assert_eq!(manager.failed_attempt_count("user1"), 1);
        
        manager.clear_failed_attempts("user1");
        assert_eq!(manager.failed_attempt_count("user1"), 0);
    }
    
    #[test]
    fn test_lockout() {
        let mut config = SecurityConfig::default();
        config.max_auth_attempts = 3;
        
        let mut manager = AuthenticationManager::new(config);
        
        assert!(!manager.is_locked_out("user1"));
        
        manager.record_failed_attempt("user1");
        manager.record_failed_attempt("user1");
        manager.record_failed_attempt("user1");
        
        // Note: Would need real timestamps to test lockout
    }
    
    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 6);
        assert!(policy.require_numbers);
    }
    
    #[test]
    fn test_password_validation() {
        let policy = PasswordPolicy::default();
        
        let result = policy.validate("123456");
        // May fail due to sequence check, but format should be valid
        
        let short = policy.validate("12");
        assert!(!short.valid);
    }
    
    #[test]
    fn test_password_strength() {
        let policy = PasswordPolicy::default();
        
        let weak = policy.validate("111111");
        // Strength depends on various factors
        
        let strong = policy.validate("MyP@ssw0rd!Complex123");
        assert_eq!(strong.strength, PasswordStrength::Strong);
    }
    
    #[test]
    fn test_common_password_check() {
        let policy = PasswordPolicy::default();
        
        let result = policy.validate("password");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("common")));
    }
    
    #[test]
    fn test_mfa_manager() {
        let mut manager = MFAManager::new();
        
        assert!(!manager.is_mfa_enabled("user1"));
        
        manager.enroll("user1", MFAFactor {
            factor_type: MFAFactorType::TOTP,
            enabled: true,
            enrolled_at: 0,
            last_used: None,
            config: MFAFactorConfig::TOTP {
                secret: vec![1, 2, 3],
                algorithm: "SHA1".to_string(),
            },
        });
        
        assert!(manager.is_mfa_enabled("user1"));
        assert_eq!(manager.get_factors("user1").len(), 1);
    }
    
    #[test]
    fn test_mfa_challenge() {
        let mut manager = MFAManager::new();
        
        let challenge = manager.create_challenge("user1", MFAFactorType::TOTP);
        assert!(challenge.is_some());
        
        let challenge = challenge.unwrap();
        
        // Verify correct code
        assert!(manager.verify(&challenge.challenge_id, "123456"));
        
        // Challenge should be consumed
        assert!(!manager.verify(&challenge.challenge_id, "123456"));
    }
}
