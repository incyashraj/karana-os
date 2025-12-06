// Adaptive Ledger - Dynamic blockchain operation modes
// Phase 46: Intelligent ledger management for constrained devices

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::monitor::{ResourceLevel, ResourceMonitor};

/// Ledger operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerMode {
    /// Full blockchain with 30s blocks, all intents recorded
    Full,
    
    /// Light mode: only high-value actions on chain, rest in signed log
    Light,
    
    /// Minimal: local signed event log only, no blockchain consensus
    Minimal,
}

impl LedgerMode {
    /// Get recommended mode for resource level
    pub fn recommended_for_level(level: ResourceLevel) -> Self {
        match level {
            ResourceLevel::Abundant => LedgerMode::Full,
            ResourceLevel::Normal => LedgerMode::Full,
            ResourceLevel::Constrained => LedgerMode::Light,
            ResourceLevel::Critical => LedgerMode::Minimal,
        }
    }
    
    /// Get block interval for this mode
    pub fn block_interval_secs(&self) -> u64 {
        match self {
            LedgerMode::Full => 30,
            LedgerMode::Light => 60,
            LedgerMode::Minimal => 0, // No blocks
        }
    }
    
    /// Check if intent should be on-chain for this mode
    pub fn should_record_on_chain(&self, intent_type: &IntentType) -> bool {
        match self {
            LedgerMode::Full => true,
            LedgerMode::Light => intent_type.is_high_value(),
            LedgerMode::Minimal => false,
        }
    }
}

/// Intent classification for ledger recording
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentType {
    /// Financial transactions
    Payment,
    
    /// Governance voting
    Governance,
    
    /// Permission changes
    Permission,
    
    /// App interactions
    AppInteraction,
    
    /// Query or read-only
    Query,
    
    /// System configuration
    SystemConfig,
}

impl IntentType {
    /// Check if this is a high-value intent that should always be on-chain
    pub fn is_high_value(&self) -> bool {
        matches!(self, 
            IntentType::Payment | 
            IntentType::Governance | 
            IntentType::Permission |
            IntentType::SystemConfig
        )
    }
}

/// Adaptive ledger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveLedgerConfig {
    /// Current operation mode
    pub current_mode: LedgerMode,
    
    /// Auto-switch based on resources
    pub auto_switch_enabled: bool,
    
    /// Minimum mode to allow (user constraint)
    pub minimum_mode: LedgerMode,
    
    /// Resource level thresholds for switching
    pub switch_threshold_constrained: f32, // Battery % threshold
    pub switch_threshold_critical: f32,
    
    /// Hysteresis to prevent mode flapping
    pub switch_hysteresis_secs: u64,
}

impl Default for AdaptiveLedgerConfig {
    fn default() -> Self {
        Self {
            current_mode: LedgerMode::Full,
            auto_switch_enabled: true,
            minimum_mode: LedgerMode::Light, // Never go below Light by default
            switch_threshold_constrained: 30.0,
            switch_threshold_critical: 15.0,
            switch_hysteresis_secs: 60, // Wait 1 min before switching back
        }
    }
}

/// Adaptive ledger manager
pub struct AdaptiveLedger {
    config: Arc<RwLock<AdaptiveLedgerConfig>>,
    resource_monitor: Arc<ResourceMonitor>,
    last_mode_change: Arc<RwLock<std::time::Instant>>,
    stats: Arc<RwLock<LedgerStatistics>>,
}

impl AdaptiveLedger {
    /// Create new adaptive ledger
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Self {
        Self {
            config: Arc::new(RwLock::new(AdaptiveLedgerConfig::default())),
            resource_monitor,
            last_mode_change: Arc::new(RwLock::new(std::time::Instant::now())),
            stats: Arc::new(RwLock::new(LedgerStatistics::default())),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(resource_monitor: Arc<ResourceMonitor>, config: AdaptiveLedgerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            resource_monitor,
            last_mode_change: Arc::new(RwLock::new(std::time::Instant::now())),
            stats: Arc::new(RwLock::new(LedgerStatistics::default())),
        }
    }
    
    /// Get current ledger mode
    pub async fn get_mode(&self) -> LedgerMode {
        self.config.read().await.current_mode
    }
    
    /// Set ledger mode manually
    pub async fn set_mode(&self, mode: LedgerMode) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Check if mode is allowed
        if mode < config.minimum_mode {
            return Err(anyhow!("Mode {:?} is below minimum allowed mode {:?}", mode, config.minimum_mode));
        }
        
        let old_mode = config.current_mode;
        config.current_mode = mode;
        
        // Update last change time
        *self.last_mode_change.write().await = std::time::Instant::now();
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.mode_changes += 1;
        stats.last_mode = old_mode;
        
        Ok(())
    }
    
    /// Update mode based on current resources (called periodically)
    pub async fn update_mode(&self) -> Result<()> {
        let config = self.config.read().await;
        
        if !config.auto_switch_enabled {
            return Ok(());
        }
        
        // Check hysteresis
        let last_change = self.last_mode_change.read().await;
        let elapsed = last_change.elapsed().as_secs();
        
        if elapsed < config.switch_hysteresis_secs {
            return Ok(()); // Too soon to switch again
        }
        
        drop(config); // Release read lock before potential write
        
        // Get resource level
        let resource_level = self.resource_monitor.get_resource_level().await;
        let snapshot = self.resource_monitor.get_snapshot().await;
        
        // Determine recommended mode
        let recommended = LedgerMode::recommended_for_level(resource_level);
        
        // Additional battery-based logic
        let battery_mode = if snapshot.battery_level < 15.0 && !snapshot.is_charging {
            LedgerMode::Minimal
        } else if snapshot.battery_level < 30.0 && !snapshot.is_charging {
            LedgerMode::Light
        } else {
            LedgerMode::Full
        };
        
        // Take the more conservative mode
        let target_mode = if battery_mode < recommended {
            battery_mode
        } else {
            recommended
        };
        
        let current_mode = self.get_mode().await;
        
        // Switch if different
        if target_mode != current_mode {
            self.set_mode(target_mode).await?;
        }
        
        Ok(())
    }
    
    /// Check if intent should be recorded on blockchain
    pub async fn should_record_on_chain(&self, intent_type: IntentType) -> bool {
        let mode = self.get_mode().await;
        mode.should_record_on_chain(&intent_type)
    }
    
    /// Get current block interval
    pub async fn get_block_interval(&self) -> u64 {
        let mode = self.get_mode().await;
        mode.block_interval_secs()
    }
    
    /// Start auto-update loop
    pub async fn start_auto_update(&self) {
        let self_clone = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self_clone.update_mode().await {
                    eprintln!("Error updating ledger mode: {}", e);
                }
            }
        });
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> LedgerStatistics {
        self.stats.read().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config<F>(&self, updater: F) 
    where
        F: FnOnce(&mut AdaptiveLedgerConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut *config);
    }
}

impl Clone for AdaptiveLedger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            resource_monitor: self.resource_monitor.clone(),
            last_mode_change: self.last_mode_change.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Ledger usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerStatistics {
    pub mode_changes: u64,
    pub last_mode: LedgerMode,
    pub on_chain_intents: u64,
    pub off_chain_intents: u64,
    pub blocks_created: u64,
}

impl Default for LedgerMode {
    fn default() -> Self {
        LedgerMode::Full
    }
}

impl PartialOrd for LedgerMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LedgerMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use LedgerMode::*;
        match (self, other) {
            (Full, Full) => std::cmp::Ordering::Equal,
            (Full, _) => std::cmp::Ordering::Greater,
            (Light, Full) => std::cmp::Ordering::Less,
            (Light, Light) => std::cmp::Ordering::Equal,
            (Light, Minimal) => std::cmp::Ordering::Greater,
            (Minimal, Minimal) => std::cmp::Ordering::Equal,
            (Minimal, _) => std::cmp::Ordering::Less,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ledger_mode_ordering() {
        assert!(LedgerMode::Full > LedgerMode::Light);
        assert!(LedgerMode::Light > LedgerMode::Minimal);
        assert!(LedgerMode::Full > LedgerMode::Minimal);
    }
    
    #[test]
    fn test_recommended_mode() {
        assert_eq!(
            LedgerMode::recommended_for_level(ResourceLevel::Abundant),
            LedgerMode::Full
        );
        assert_eq!(
            LedgerMode::recommended_for_level(ResourceLevel::Constrained),
            LedgerMode::Light
        );
        assert_eq!(
            LedgerMode::recommended_for_level(ResourceLevel::Critical),
            LedgerMode::Minimal
        );
    }
    
    #[test]
    fn test_intent_recording() {
        let full_mode = LedgerMode::Full;
        let light_mode = LedgerMode::Light;
        let minimal_mode = LedgerMode::Minimal;
        
        // Full mode records everything
        assert!(full_mode.should_record_on_chain(&IntentType::Payment));
        assert!(full_mode.should_record_on_chain(&IntentType::Query));
        
        // Light mode only records high-value
        assert!(light_mode.should_record_on_chain(&IntentType::Payment));
        assert!(!light_mode.should_record_on_chain(&IntentType::Query));
        
        // Minimal records nothing
        assert!(!minimal_mode.should_record_on_chain(&IntentType::Payment));
    }
    
    #[tokio::test]
    async fn test_adaptive_ledger_creation() {
        let monitor = Arc::new(ResourceMonitor::new());
        let ledger = AdaptiveLedger::new(monitor);
        
        let mode = ledger.get_mode().await;
        assert_eq!(mode, LedgerMode::Full);
    }
    
    #[tokio::test]
    async fn test_set_mode() {
        let monitor = Arc::new(ResourceMonitor::new());
        let ledger = AdaptiveLedger::new(monitor);
        
        ledger.set_mode(LedgerMode::Light).await.unwrap();
        assert_eq!(ledger.get_mode().await, LedgerMode::Light);
        
        // Try to set below minimum
        let result = ledger.set_mode(LedgerMode::Minimal).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_block_interval() {
        let monitor = Arc::new(ResourceMonitor::new());
        let ledger = AdaptiveLedger::new(monitor);
        
        assert_eq!(ledger.get_block_interval().await, 30);
        
        ledger.set_mode(LedgerMode::Light).await.unwrap();
        assert_eq!(ledger.get_block_interval().await, 60);
        
        ledger.update_config(|config| {
            config.minimum_mode = LedgerMode::Minimal;
        }).await;
        
        ledger.set_mode(LedgerMode::Minimal).await.unwrap();
        assert_eq!(ledger.get_block_interval().await, 0);
    }
    
    #[tokio::test]
    async fn test_statistics() {
        let monitor = Arc::new(ResourceMonitor::new());
        let ledger = AdaptiveLedger::new(monitor);
        
        ledger.set_mode(LedgerMode::Light).await.unwrap();
        ledger.set_mode(LedgerMode::Full).await.unwrap();
        
        let stats = ledger.get_statistics().await;
        assert_eq!(stats.mode_changes, 2);
    }
}
