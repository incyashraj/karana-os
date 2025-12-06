// AI Profiles - Dynamic model selection based on resources
// Phase 46: Intelligent AI workload management

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::monitor::{ResourceLevel, ResourceMonitor};

/// AI computational profile level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AIProfile {
    /// Minimal AI: keyword spotting + rule-based NLU only
    UltraLow,
    
    /// Basic AI: Whisper Tiny + MiniLM only
    Basic,
    
    /// Standard AI: Current ONNX suite with dynamic downsampling
    Standard,
    
    /// Advanced AI: Full models + optional cloud augmentation
    Advanced,
}

impl AIProfile {
    /// Get recommended profile for resource level
    pub fn recommended_for_level(level: ResourceLevel) -> Self {
        match level {
            ResourceLevel::Abundant => AIProfile::Advanced,
            ResourceLevel::Normal => AIProfile::Standard,
            ResourceLevel::Constrained => AIProfile::Basic,
            ResourceLevel::Critical => AIProfile::UltraLow,
        }
    }
    
    /// Check if specific AI capability is enabled
    pub fn has_capability(&self, capability: AICapability) -> bool {
        use AICapability::*;
        use AIProfile::*;
        
        match capability {
            KeywordSpotting => true, // Always available
            RuleBasedNLU => true,    // Always available
            
            WhisperTiny => *self >= Basic,
            MiniLM => *self >= Basic,
            
            WhisperBase => *self >= Standard,
            BLIP => *self >= Standard,
            TinyLlama => *self >= Standard,
            
            WhisperLarge => *self >= Advanced,
            LargeVisionModel => *self >= Advanced,
            CloudAugmentation => *self >= Advanced,
        }
    }
    
    /// Get maximum concurrent model instances
    pub fn max_concurrent_models(&self) -> usize {
        match self {
            AIProfile::UltraLow => 1,
            AIProfile::Basic => 2,
            AIProfile::Standard => 3,
            AIProfile::Advanced => 5,
        }
    }
    
    /// Get update interval for continuous tasks (e.g., scene understanding)
    pub fn update_interval_ms(&self) -> u64 {
        match self {
            AIProfile::UltraLow => 5000,  // 5 seconds
            AIProfile::Basic => 2000,     // 2 seconds
            AIProfile::Standard => 500,   // 500ms
            AIProfile::Advanced => 100,   // 100ms
        }
    }
}

impl Default for AIProfile {
    fn default() -> Self {
        AIProfile::Standard
    }
}

/// AI capability types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AICapability {
    KeywordSpotting,
    RuleBasedNLU,
    WhisperTiny,
    WhisperBase,
    WhisperLarge,
    MiniLM,
    BLIP,
    TinyLlama,
    LargeVisionModel,
    CloudAugmentation,
}

/// AI Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProfileConfig {
    /// Current profile
    pub current_profile: AIProfile,
    
    /// Auto-switch based on resources
    pub auto_switch_enabled: bool,
    
    /// Minimum profile to allow
    pub minimum_profile: AIProfile,
    
    /// Intent-aware scheduling enabled
    pub intent_aware_scheduling: bool,
    
    /// Idle timeout before downsampling (seconds)
    pub idle_timeout_secs: u64,
    
    /// Thermal throttling enabled
    pub thermal_throttling: bool,
    
    /// Thermal threshold for throttling (Celsius)
    pub thermal_threshold: f32,
}

impl Default for AIProfileConfig {
    fn default() -> Self {
        Self {
            current_profile: AIProfile::Standard,
            auto_switch_enabled: true,
            minimum_profile: AIProfile::Basic,
            intent_aware_scheduling: true,
            idle_timeout_secs: 300, // 5 minutes
            thermal_throttling: true,
            thermal_threshold: 75.0,
        }
    }
}

/// AI Profile manager
pub struct AIProfileManager {
    config: Arc<RwLock<AIProfileConfig>>,
    resource_monitor: Arc<ResourceMonitor>,
    last_interaction: Arc<RwLock<std::time::Instant>>,
    stats: Arc<RwLock<AIProfileStatistics>>,
    active_models: Arc<RwLock<Vec<String>>>,
}

impl AIProfileManager {
    /// Create new AI profile manager
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Self {
        Self {
            config: Arc::new(RwLock::new(AIProfileConfig::default())),
            resource_monitor,
            last_interaction: Arc::new(RwLock::new(std::time::Instant::now())),
            stats: Arc::new(RwLock::new(AIProfileStatistics::default())),
            active_models: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(resource_monitor: Arc<ResourceMonitor>, config: AIProfileConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            resource_monitor,
            last_interaction: Arc::new(RwLock::new(std::time::Instant::now())),
            stats: Arc::new(RwLock::new(AIProfileStatistics::default())),
            active_models: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Get current AI profile
    pub async fn get_profile(&self) -> AIProfile {
        self.config.read().await.current_profile
    }
    
    /// Set AI profile manually
    pub async fn set_profile(&self, profile: AIProfile) -> Result<()> {
        let mut config = self.config.write().await;
        
        if profile < config.minimum_profile {
            return Err(anyhow!("Profile {:?} is below minimum {:?}", profile, config.minimum_profile));
        }
        
        let old_profile = config.current_profile;
        config.current_profile = profile;
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.profile_changes += 1;
        stats.last_profile = old_profile;
        
        Ok(())
    }
    
    /// Update profile based on resources and activity
    pub async fn update_profile(&self) -> Result<()> {
        let config = self.config.read().await;
        
        if !config.auto_switch_enabled {
            return Ok(());
        }
        
        let intent_aware = config.intent_aware_scheduling;
        let idle_timeout = config.idle_timeout_secs;
        let thermal_throttling = config.thermal_throttling;
        let thermal_threshold = config.thermal_threshold;
        
        drop(config); // Release read lock
        
        // Get current snapshot
        let snapshot = self.resource_monitor.get_snapshot().await;
        let resource_level = self.resource_monitor.get_resource_level().await;
        
        // Check idle state
        let is_idle = if intent_aware {
            let last_interaction = self.last_interaction.read().await;
            last_interaction.elapsed().as_secs() > idle_timeout
        } else {
            false
        };
        
        // Check thermal throttling
        let thermal_stress = thermal_throttling && snapshot.temperature > thermal_threshold;
        
        // Determine target profile
        let mut target_profile = AIProfile::recommended_for_level(resource_level);
        
        // Downgrade if idle
        if is_idle && target_profile > AIProfile::Basic {
            target_profile = AIProfile::Basic;
        }
        
        // Downgrade if thermal stress
        if thermal_stress && target_profile > AIProfile::Basic {
            target_profile = AIProfile::Basic;
        }
        
        let current_profile = self.get_profile().await;
        
        // Switch if different
        if target_profile != current_profile {
            self.set_profile(target_profile).await?;
        }
        
        Ok(())
    }
    
    /// Record user interaction (resets idle timer)
    pub async fn record_interaction(&self) {
        *self.last_interaction.write().await = std::time::Instant::now();
    }
    
    /// Check if AI capability is available
    pub async fn has_capability(&self, capability: AICapability) -> bool {
        let profile = self.get_profile().await;
        profile.has_capability(capability)
    }
    
    /// Register active model
    pub async fn register_model(&self, model_name: String) -> Result<()> {
        let profile = self.get_profile().await;
        let max_models = profile.max_concurrent_models();
        
        let mut active = self.active_models.write().await;
        
        if active.len() >= max_models {
            return Err(anyhow!("Maximum concurrent models ({}) reached for profile {:?}", max_models, profile));
        }
        
        active.push(model_name);
        Ok(())
    }
    
    /// Unregister active model
    pub async fn unregister_model(&self, model_name: &str) {
        let mut active = self.active_models.write().await;
        active.retain(|m| m != model_name);
    }
    
    /// Get active model count
    pub async fn active_model_count(&self) -> usize {
        self.active_models.read().await.len()
    }
    
    /// Start auto-update loop
    pub async fn start_auto_update(&self) {
        let self_clone = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self_clone.update_profile().await {
                    eprintln!("Error updating AI profile: {}", e);
                }
            }
        });
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> AIProfileStatistics {
        self.stats.read().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config<F>(&self, updater: F) 
    where
        F: FnOnce(&mut AIProfileConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut *config);
    }
}

impl Clone for AIProfileManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            resource_monitor: self.resource_monitor.clone(),
            last_interaction: self.last_interaction.clone(),
            stats: self.stats.clone(),
            active_models: self.active_models.clone(),
        }
    }
}

/// AI Profile usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AIProfileStatistics {
    pub profile_changes: u64,
    pub last_profile: AIProfile,
    pub total_model_loads: u64,
    pub thermal_throttles: u64,
    pub idle_downgrades: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_ordering() {
        assert!(AIProfile::Advanced > AIProfile::Standard);
        assert!(AIProfile::Standard > AIProfile::Basic);
        assert!(AIProfile::Basic > AIProfile::UltraLow);
    }
    
    #[test]
    fn test_recommended_profile() {
        assert_eq!(
            AIProfile::recommended_for_level(ResourceLevel::Abundant),
            AIProfile::Advanced
        );
        assert_eq!(
            AIProfile::recommended_for_level(ResourceLevel::Critical),
            AIProfile::UltraLow
        );
    }
    
    #[test]
    fn test_capabilities() {
        let ultra_low = AIProfile::UltraLow;
        let basic = AIProfile::Basic;
        let standard = AIProfile::Standard;
        let advanced = AIProfile::Advanced;
        
        // Everyone has keywords
        assert!(ultra_low.has_capability(AICapability::KeywordSpotting));
        
        // Only Basic+ has Whisper
        assert!(!ultra_low.has_capability(AICapability::WhisperTiny));
        assert!(basic.has_capability(AICapability::WhisperTiny));
        
        // Only Standard+ has BLIP
        assert!(!basic.has_capability(AICapability::BLIP));
        assert!(standard.has_capability(AICapability::BLIP));
        
        // Only Advanced has cloud
        assert!(!standard.has_capability(AICapability::CloudAugmentation));
        assert!(advanced.has_capability(AICapability::CloudAugmentation));
    }
    
    #[tokio::test]
    async fn test_profile_manager_creation() {
        let monitor = Arc::new(ResourceMonitor::new());
        let manager = AIProfileManager::new(monitor);
        
        assert_eq!(manager.get_profile().await, AIProfile::Standard);
    }
    
    #[tokio::test]
    async fn test_set_profile() {
        let monitor = Arc::new(ResourceMonitor::new());
        let manager = AIProfileManager::new(monitor);
        
        manager.set_profile(AIProfile::Advanced).await.unwrap();
        assert_eq!(manager.get_profile().await, AIProfile::Advanced);
        
        // Try to set below minimum
        let result = manager.set_profile(AIProfile::UltraLow).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_model_registration() {
        let monitor = Arc::new(ResourceMonitor::new());
        let manager = AIProfileManager::new(monitor);
        
        // Set to Basic (max 2 models)
        manager.set_profile(AIProfile::Basic).await.unwrap();
        
        manager.register_model("whisper".to_string()).await.unwrap();
        manager.register_model("minilm".to_string()).await.unwrap();
        
        // Third should fail
        let result = manager.register_model("blip".to_string()).await;
        assert!(result.is_err());
        
        assert_eq!(manager.active_model_count().await, 2);
        
        // Unregister and try again
        manager.unregister_model("whisper").await;
        manager.register_model("blip".to_string()).await.unwrap();
        
        assert_eq!(manager.active_model_count().await, 2);
    }
    
    #[tokio::test]
    async fn test_interaction_tracking() {
        let monitor = Arc::new(ResourceMonitor::new());
        let manager = AIProfileManager::new(monitor);
        
        manager.record_interaction().await;
        
        // Check that last_interaction was updated (can't be too old)
        let last = manager.last_interaction.read().await;
        assert!(last.elapsed().as_secs() < 1);
    }
}
