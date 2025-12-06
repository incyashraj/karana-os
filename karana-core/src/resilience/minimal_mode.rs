// Minimal Mode - Ultra-reliable fallback mode
// Phase 48: Ensure system works even under adverse conditions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Minimal mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimalModeState {
    /// Normal full-feature operation
    Normal,
    
    /// Minimal mode active
    Active,
    
    /// Transitioning to minimal mode
    Activating,
    
    /// Transitioning back to normal
    Deactivating,
}

/// Reason for entering minimal mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimalModeReason {
    /// User manually activated
    Manual,
    
    /// Critical battery level
    CriticalBattery,
    
    /// Thermal emergency
    ThermalEmergency,
    
    /// Memory exhaustion
    MemoryExhausted,
    
    /// Multiple layer failures
    LayerFailures(Vec<String>),
    
    /// Camera failure
    CameraFailure,
    
    /// Network failure
    NetworkFailure,
    
    /// Recovery from crash
    CrashRecovery,
}

/// Minimal mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalModeConfig {
    /// Automatically activate on critical conditions
    pub auto_activate: bool,
    
    /// Maximum memory usage in bytes (default: 10MB)
    pub max_memory_bytes: u64,
    
    /// Target CPU usage percentage (default: 5%)
    pub target_cpu_percent: f32,
    
    /// Features available in minimal mode
    pub enabled_features: MinimalFeatures,
}

impl Default for MinimalModeConfig {
    fn default() -> Self {
        Self {
            auto_activate: true,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            target_cpu_percent: 5.0,
            enabled_features: MinimalFeatures::default(),
        }
    }
}

/// Features available in minimal mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalFeatures {
    /// Basic HUD display
    pub hud: bool,
    
    /// Voice commands (keyword spotting only)
    pub voice_commands: bool,
    
    /// Wallet functionality
    pub wallet: bool,
    
    /// Emergency notifications
    pub notifications: bool,
    
    /// Time display
    pub time_display: bool,
    
    /// Battery indicator
    pub battery_indicator: bool,
}

impl Default for MinimalFeatures {
    fn default() -> Self {
        Self {
            hud: true,
            voice_commands: true,
            wallet: true,
            notifications: true,
            time_display: true,
            battery_indicator: true,
        }
    }
}

/// Statistics about minimal mode usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MinimalModeStatistics {
    /// Total activations
    pub total_activations: u64,
    
    /// Total time spent in minimal mode (seconds)
    pub total_time_secs: u64,
    
    /// Last activation reason
    pub last_reason: Option<MinimalModeReason>,
    
    /// Last activation timestamp
    pub last_activation: u64,
    
    /// Successful transitions
    pub successful_transitions: u64,
    
    /// Failed transitions
    pub failed_transitions: u64,
}

/// Minimal mode manager
pub struct MinimalModeManager {
    state: Arc<RwLock<MinimalModeState>>,
    config: Arc<RwLock<MinimalModeConfig>>,
    stats: Arc<RwLock<MinimalModeStatistics>>,
    current_reason: Arc<RwLock<Option<MinimalModeReason>>>,
    activation_time: Arc<RwLock<Option<std::time::Instant>>>,
}

impl MinimalModeManager {
    /// Create new minimal mode manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MinimalModeState::Normal)),
            config: Arc::new(RwLock::new(MinimalModeConfig::default())),
            stats: Arc::new(RwLock::new(MinimalModeStatistics::default())),
            current_reason: Arc::new(RwLock::new(None)),
            activation_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: MinimalModeConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(MinimalModeState::Normal)),
            config: Arc::new(RwLock::new(config)),
            stats: Arc::new(RwLock::new(MinimalModeStatistics::default())),
            current_reason: Arc::new(RwLock::new(None)),
            activation_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Get current state
    pub async fn get_state(&self) -> MinimalModeState {
        *self.state.read().await
    }
    
    /// Check if minimal mode is active
    pub async fn is_active(&self) -> bool {
        matches!(self.get_state().await, MinimalModeState::Active)
    }
    
    /// Activate minimal mode
    pub async fn activate(&self, reason: MinimalModeReason) -> Result<()> {
        let mut state = self.state.write().await;
        
        if *state == MinimalModeState::Active {
            return Ok(()); // Already active
        }
        
        *state = MinimalModeState::Activating;
        drop(state);
        
        // Store reason
        *self.current_reason.write().await = Some(reason.clone());
        
        // Record activation time
        *self.activation_time.write().await = Some(std::time::Instant::now());
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_activations += 1;
        stats.last_reason = Some(reason);
        stats.last_activation = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        drop(stats);
        
        // Perform transition
        self.transition_to_minimal().await?;
        
        // Set state to active
        let mut state = self.state.write().await;
        *state = MinimalModeState::Active;
        
        // Update success counter
        self.stats.write().await.successful_transitions += 1;
        
        Ok(())
    }
    
    /// Deactivate minimal mode
    pub async fn deactivate(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        if *state != MinimalModeState::Active {
            return Ok(()); // Not active
        }
        
        *state = MinimalModeState::Deactivating;
        drop(state);
        
        // Calculate time spent in minimal mode
        if let Some(activation_time) = *self.activation_time.read().await {
            let duration = activation_time.elapsed().as_secs();
            self.stats.write().await.total_time_secs += duration;
        }
        
        // Clear activation time and reason
        *self.activation_time.write().await = None;
        *self.current_reason.write().await = None;
        
        // Perform transition
        self.transition_to_normal().await?;
        
        // Set state to normal
        let mut state = self.state.write().await;
        *state = MinimalModeState::Normal;
        
        // Update success counter
        self.stats.write().await.successful_transitions += 1;
        
        Ok(())
    }
    
    /// Perform transition to minimal mode
    async fn transition_to_minimal(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Stop all AR tabs and collaborative features
        // 2. Disable networking except for critical updates
        // 3. Shut down heavy AI models
        // 4. Switch to minimal HUD
        // 5. Enable only keyword spotting for voice
        // 6. Keep wallet and notifications active
        
        // Simulate transition
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Perform transition to normal mode
    async fn transition_to_normal(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Re-enable networking
        // 2. Start AI models based on current profile
        // 3. Restore full HUD
        // 4. Re-enable AR tabs
        // 5. Resume collaborative features
        
        // Simulate transition
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Check if should auto-activate based on conditions
    pub async fn should_auto_activate(
        &self,
        battery_level: f32,
        temperature: f32,
        memory_used: u64,
    ) -> Option<MinimalModeReason> {
        let config = self.config.read().await;
        
        if !config.auto_activate {
            return None;
        }
        
        // Check critical battery
        if battery_level < 5.0 {
            return Some(MinimalModeReason::CriticalBattery);
        }
        
        // Check thermal emergency
        if temperature > 90.0 {
            return Some(MinimalModeReason::ThermalEmergency);
        }
        
        // Check memory exhaustion
        if memory_used > 3_500_000_000 { // 3.5GB on 4GB system
            return Some(MinimalModeReason::MemoryExhausted);
        }
        
        None
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> MinimalModeConfig {
        self.config.read().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config<F>(&self, updater: F)
    where
        F: FnOnce(&mut MinimalModeConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut *config);
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> MinimalModeStatistics {
        self.stats.read().await.clone()
    }
    
    /// Get current activation reason
    pub async fn get_current_reason(&self) -> Option<MinimalModeReason> {
        self.current_reason.read().await.clone()
    }
    
    /// Check if feature is available in minimal mode
    pub async fn is_feature_available(&self, feature: MinimalFeature) -> bool {
        let state = self.get_state().await;
        
        if state != MinimalModeState::Active {
            return true; // All features available in normal mode
        }
        
        let config = self.config.read().await;
        
        match feature {
            MinimalFeature::HUD => config.enabled_features.hud,
            MinimalFeature::VoiceCommands => config.enabled_features.voice_commands,
            MinimalFeature::Wallet => config.enabled_features.wallet,
            MinimalFeature::Notifications => config.enabled_features.notifications,
            MinimalFeature::TimeDisplay => config.enabled_features.time_display,
            MinimalFeature::BatteryIndicator => config.enabled_features.battery_indicator,
            MinimalFeature::ARTabs => false,
            MinimalFeature::Networking => false,
            MinimalFeature::Collaboration => false,
            MinimalFeature::VisionModels => false,
        }
    }
}

impl Default for MinimalModeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature that can be queried
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimalFeature {
    HUD,
    VoiceCommands,
    Wallet,
    Notifications,
    TimeDisplay,
    BatteryIndicator,
    ARTabs,
    Networking,
    Collaboration,
    VisionModels,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_minimal_mode_creation() {
        let manager = MinimalModeManager::new();
        assert_eq!(manager.get_state().await, MinimalModeState::Normal);
        assert!(!manager.is_active().await);
    }
    
    #[tokio::test]
    async fn test_activate_minimal_mode() {
        let manager = MinimalModeManager::new();
        
        manager.activate(MinimalModeReason::Manual).await.unwrap();
        
        assert_eq!(manager.get_state().await, MinimalModeState::Active);
        assert!(manager.is_active().await);
        
        let reason = manager.get_current_reason().await;
        assert_eq!(reason, Some(MinimalModeReason::Manual));
    }
    
    #[tokio::test]
    async fn test_deactivate_minimal_mode() {
        let manager = MinimalModeManager::new();
        
        manager.activate(MinimalModeReason::Manual).await.unwrap();
        assert!(manager.is_active().await);
        
        manager.deactivate().await.unwrap();
        assert!(!manager.is_active().await);
        assert_eq!(manager.get_state().await, MinimalModeState::Normal);
    }
    
    #[tokio::test]
    async fn test_auto_activation_battery() {
        let manager = MinimalModeManager::new();
        
        let reason = manager.should_auto_activate(3.0, 50.0, 1_000_000_000).await;
        assert_eq!(reason, Some(MinimalModeReason::CriticalBattery));
    }
    
    #[tokio::test]
    async fn test_auto_activation_thermal() {
        let manager = MinimalModeManager::new();
        
        let reason = manager.should_auto_activate(50.0, 95.0, 1_000_000_000).await;
        assert_eq!(reason, Some(MinimalModeReason::ThermalEmergency));
    }
    
    #[tokio::test]
    async fn test_auto_activation_memory() {
        let manager = MinimalModeManager::new();
        
        let reason = manager.should_auto_activate(50.0, 50.0, 3_800_000_000).await;
        assert_eq!(reason, Some(MinimalModeReason::MemoryExhausted));
    }
    
    #[tokio::test]
    async fn test_feature_availability_normal_mode() {
        let manager = MinimalModeManager::new();
        
        // All features available in normal mode
        assert!(manager.is_feature_available(MinimalFeature::HUD).await);
        assert!(manager.is_feature_available(MinimalFeature::ARTabs).await);
        assert!(manager.is_feature_available(MinimalFeature::VisionModels).await);
    }
    
    #[tokio::test]
    async fn test_feature_availability_minimal_mode() {
        let manager = MinimalModeManager::new();
        manager.activate(MinimalModeReason::Manual).await.unwrap();
        
        // Basic features available
        assert!(manager.is_feature_available(MinimalFeature::HUD).await);
        assert!(manager.is_feature_available(MinimalFeature::Wallet).await);
        
        // Advanced features disabled
        assert!(!manager.is_feature_available(MinimalFeature::ARTabs).await);
        assert!(!manager.is_feature_available(MinimalFeature::VisionModels).await);
    }
    
    #[tokio::test]
    async fn test_statistics_tracking() {
        let manager = MinimalModeManager::new();
        
        manager.activate(MinimalModeReason::CriticalBattery).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        manager.deactivate().await.unwrap();
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_activations, 1);
        assert_eq!(stats.successful_transitions, 2); // activate + deactivate
        assert!(stats.total_time_secs >= 1);
    }
    
    #[tokio::test]
    async fn test_config_update() {
        let manager = MinimalModeManager::new();
        
        manager.update_config(|config| {
            config.auto_activate = false;
            config.max_memory_bytes = 5 * 1024 * 1024;
        }).await;
        
        let config = manager.get_config().await;
        assert!(!config.auto_activate);
        assert_eq!(config.max_memory_bytes, 5 * 1024 * 1024);
    }
}
