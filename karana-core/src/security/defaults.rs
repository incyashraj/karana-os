// Kāraṇa OS - Phase 59: Security Defaults
// Safe permission presets, spending guards, recovery mechanisms

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security preset levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityPreset {
    /// Maximum security, minimal permissions
    Paranoid,
    
    /// High security, essential permissions only
    High,
    
    /// Balanced security and convenience
    Balanced,
    
    /// Lower security, more convenience
    Relaxed,
}

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Camera,
    Microphone,
    Location,
    Blockchain,
    Storage,
    Network,
    Sensors,
    Biometrics,
}

impl SecurityPreset {
    /// Get default permissions for preset
    pub fn default_permissions(&self) -> HashMap<Permission, bool> {
        let mut perms = HashMap::new();
        
        match self {
            Self::Paranoid => {
                perms.insert(Permission::Camera, false);
                perms.insert(Permission::Microphone, false);
                perms.insert(Permission::Location, false);
                perms.insert(Permission::Blockchain, false);
                perms.insert(Permission::Storage, true);
                perms.insert(Permission::Network, false);
                perms.insert(Permission::Sensors, false);
                perms.insert(Permission::Biometrics, false);
            }
            Self::High => {
                perms.insert(Permission::Camera, true);
                perms.insert(Permission::Microphone, true);
                perms.insert(Permission::Location, false);
                perms.insert(Permission::Blockchain, true);
                perms.insert(Permission::Storage, true);
                perms.insert(Permission::Network, false);
                perms.insert(Permission::Sensors, true);
                perms.insert(Permission::Biometrics, false);
            }
            Self::Balanced => {
                perms.insert(Permission::Camera, true);
                perms.insert(Permission::Microphone, true);
                perms.insert(Permission::Location, true);
                perms.insert(Permission::Blockchain, true);
                perms.insert(Permission::Storage, true);
                perms.insert(Permission::Network, true);
                perms.insert(Permission::Sensors, true);
                perms.insert(Permission::Biometrics, true);
            }
            Self::Relaxed => {
                // All permissions enabled
                for perm in [
                    Permission::Camera,
                    Permission::Microphone,
                    Permission::Location,
                    Permission::Blockchain,
                    Permission::Storage,
                    Permission::Network,
                    Permission::Sensors,
                    Permission::Biometrics,
                ] {
                    perms.insert(perm, true);
                }
            }
        }
        
        perms
    }
}

/// Spending limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingGuard {
    pub daily_limit: f64,
    pub transaction_limit: f64,
    pub require_confirmation: bool,
    pub cooldown_seconds: u64,
}

impl Default for SpendingGuard {
    fn default() -> Self {
        Self {
            daily_limit: 100.0,
            transaction_limit: 10.0,
            require_confirmation: true,
            cooldown_seconds: 30,
        }
    }
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub recovery_phrase_enabled: bool,
    pub social_recovery_enabled: bool,
    pub trusted_contacts: Vec<String>,
    pub recovery_threshold: usize,
    pub backup_encrypted: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            recovery_phrase_enabled: true,
            social_recovery_enabled: false,
            trusted_contacts: Vec::new(),
            recovery_threshold: 2,
            backup_encrypted: true,
        }
    }
}

/// Security manager
pub struct SecurityManager {
    preset: SecurityPreset,
    permissions: HashMap<Permission, bool>,
    spending_guard: SpendingGuard,
    recovery: RecoveryConfig,
    daily_spent: f64,
    last_transaction_time: u64,
}

impl SecurityManager {
    /// Create new security manager with preset
    pub fn new(preset: SecurityPreset) -> Self {
        Self {
            preset,
            permissions: preset.default_permissions(),
            spending_guard: SpendingGuard::default(),
            recovery: RecoveryConfig::default(),
            daily_spent: 0.0,
            last_transaction_time: 0,
        }
    }
    
    /// Check if permission is granted
    pub fn has_permission(&self, perm: Permission) -> bool {
        *self.permissions.get(&perm).unwrap_or(&false)
    }
    
    /// Request permission
    pub fn request_permission(&mut self, perm: Permission) -> Result<bool> {
        // In real implementation, this would show UI prompt
        Ok(self.has_permission(perm))
    }
    
    /// Check if transaction is allowed
    pub fn check_transaction(&mut self, amount: f64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Check transaction limit
        if amount > self.spending_guard.transaction_limit {
            if self.spending_guard.require_confirmation {
                return Err(anyhow!(
                    "Transaction of {:.2} exceeds limit of {:.2}, confirmation required",
                    amount,
                    self.spending_guard.transaction_limit
                ));
            }
        }
        
        // Check daily limit
        if self.daily_spent + amount > self.spending_guard.daily_limit {
            return Err(anyhow!(
                "Daily spending limit of {:.2} would be exceeded",
                self.spending_guard.daily_limit
            ));
        }
        
        // Check cooldown
        if now - self.last_transaction_time < self.spending_guard.cooldown_seconds {
            return Err(anyhow!(
                "Transaction cooldown active, wait {} seconds",
                self.spending_guard.cooldown_seconds - (now - self.last_transaction_time)
            ));
        }
        
        Ok(())
    }
    
    /// Record transaction
    pub fn record_transaction(&mut self, amount: f64) -> Result<()> {
        self.check_transaction(amount)?;
        
        self.daily_spent += amount;
        self.last_transaction_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        Ok(())
    }
    
    /// Set spending guard
    pub fn set_spending_guard(&mut self, guard: SpendingGuard) {
        self.spending_guard = guard;
    }
    
    /// Get recovery config
    pub fn recovery_config(&self) -> &RecoveryConfig {
        &self.recovery
    }
    
    /// Set recovery config
    pub fn set_recovery_config(&mut self, config: RecoveryConfig) {
        self.recovery = config;
    }
    
    /// Reset daily spending
    pub fn reset_daily_spending(&mut self) {
        self.daily_spent = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_presets() {
        let paranoid = SecurityPreset::Paranoid.default_permissions();
        assert!(!paranoid[&Permission::Camera]);
        assert!(!paranoid[&Permission::Network]);
        
        let relaxed = SecurityPreset::Relaxed.default_permissions();
        assert!(relaxed[&Permission::Camera]);
        assert!(relaxed[&Permission::Network]);
    }
    
    #[test]
    fn test_spending_guard() {
        let mut manager = SecurityManager::new(SecurityPreset::Balanced);
        
        // Small transaction should work
        assert!(manager.check_transaction(5.0).is_ok());
        manager.record_transaction(5.0).unwrap();
        
        // Large transaction should fail
        assert!(manager.check_transaction(50.0).is_err());
    }
    
    #[test]
    fn test_daily_limit() {
        let mut manager = SecurityManager::new(SecurityPreset::Balanced);
        
        manager.record_transaction(90.0).unwrap();
        
        // Should exceed daily limit
        assert!(manager.check_transaction(20.0).is_err());
    }
    
    #[test]
    fn test_permissions() {
        let manager = SecurityManager::new(SecurityPreset::High);
        
        assert!(manager.has_permission(Permission::Camera));
        assert!(!manager.has_permission(Permission::Network));
    }
}
