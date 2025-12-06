// Feature Gates - Runtime feature enabling/disabling with dependencies
// Phase 48: Safe feature management and emergency controls

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Feature identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    // Hardware features
    Camera,
    Microphone,
    Display,
    Sensors,
    
    // Network features
    P2PNetworking,
    InternetAccess,
    CloudSync,
    
    // Blockchain features
    FullBlockchain,
    LightBlockchain,
    Transactions,
    SmartContracts,
    
    // AI features
    VisionModels,
    VoiceRecognition,
    NaturalLanguage,
    KnowledgeGraph,
    
    // Interface features
    ARTabs,
    HUD,
    VoiceInterface,
    GestureControl,
    
    // App features
    ThirdPartyApps,
    NativeApps,
    AppStore,
    
    // System features
    Telemetry,
    Diagnostics,
    Updates,
    Backup,
    
    // Collaborative features
    Collaboration,
    Swarm,
    SharedContext,
}

impl Feature {
    /// Get feature name
    pub fn name(&self) -> &'static str {
        match self {
            Feature::Camera => "camera",
            Feature::Microphone => "microphone",
            Feature::Display => "display",
            Feature::Sensors => "sensors",
            Feature::P2PNetworking => "p2p_networking",
            Feature::InternetAccess => "internet_access",
            Feature::CloudSync => "cloud_sync",
            Feature::FullBlockchain => "full_blockchain",
            Feature::LightBlockchain => "light_blockchain",
            Feature::Transactions => "transactions",
            Feature::SmartContracts => "smart_contracts",
            Feature::VisionModels => "vision_models",
            Feature::VoiceRecognition => "voice_recognition",
            Feature::NaturalLanguage => "natural_language",
            Feature::KnowledgeGraph => "knowledge_graph",
            Feature::ARTabs => "ar_tabs",
            Feature::HUD => "hud",
            Feature::VoiceInterface => "voice_interface",
            Feature::GestureControl => "gesture_control",
            Feature::ThirdPartyApps => "third_party_apps",
            Feature::NativeApps => "native_apps",
            Feature::AppStore => "app_store",
            Feature::Telemetry => "telemetry",
            Feature::Diagnostics => "diagnostics",
            Feature::Updates => "updates",
            Feature::Backup => "backup",
            Feature::Collaboration => "collaboration",
            Feature::Swarm => "swarm",
            Feature::SharedContext => "shared_context",
        }
    }
    
    /// Get default dependencies for this feature
    pub fn default_dependencies(&self) -> Vec<Feature> {
        match self {
            // Hardware has no dependencies
            Feature::Camera | Feature::Microphone | Feature::Display | Feature::Sensors => vec![],
            
            // Network features
            Feature::P2PNetworking => vec![],
            Feature::InternetAccess => vec![Feature::P2PNetworking],
            Feature::CloudSync => vec![Feature::InternetAccess],
            
            // Blockchain features
            Feature::FullBlockchain => vec![Feature::P2PNetworking],
            Feature::LightBlockchain => vec![Feature::P2PNetworking],
            Feature::Transactions => vec![Feature::LightBlockchain],
            Feature::SmartContracts => vec![Feature::FullBlockchain, Feature::Transactions],
            
            // AI features
            Feature::VisionModels => vec![Feature::Camera],
            Feature::VoiceRecognition => vec![Feature::Microphone],
            Feature::NaturalLanguage => vec![],
            Feature::KnowledgeGraph => vec![],
            
            // Interface features
            Feature::ARTabs => vec![Feature::Display, Feature::Camera],
            Feature::HUD => vec![Feature::Display],
            Feature::VoiceInterface => vec![Feature::VoiceRecognition, Feature::NaturalLanguage],
            Feature::GestureControl => vec![Feature::Camera, Feature::Sensors],
            
            // App features
            Feature::ThirdPartyApps => vec![],
            Feature::NativeApps => vec![],
            Feature::AppStore => vec![Feature::InternetAccess],
            
            // System features
            Feature::Telemetry => vec![Feature::InternetAccess],
            Feature::Diagnostics => vec![],
            Feature::Updates => vec![Feature::InternetAccess],
            Feature::Backup => vec![Feature::CloudSync],
            
            // Collaborative features
            Feature::Collaboration => vec![Feature::P2PNetworking],
            Feature::Swarm => vec![Feature::Collaboration, Feature::KnowledgeGraph],
            Feature::SharedContext => vec![Feature::Collaboration],
        }
    }
}

/// Feature state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureState {
    /// Feature is enabled and operational
    Enabled,
    
    /// Feature is disabled
    Disabled,
    
    /// Feature is temporarily disabled due to dependencies
    DisabledByDependency,
    
    /// Feature is disabled by emergency kill switch
    Killed,
}

/// Feature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Feature identifier
    pub feature: Feature,
    
    /// Current state
    pub state: FeatureState,
    
    /// Whether user can manually control this feature
    pub user_controllable: bool,
    
    /// Custom dependencies (overrides defaults)
    pub dependencies: Option<Vec<Feature>>,
    
    /// Description
    pub description: String,
}

/// Feature gate manager
pub struct FeatureGateManager {
    features: Arc<RwLock<HashMap<Feature, FeatureConfig>>>,
    change_listeners: Arc<RwLock<Vec<Box<dyn Fn(&Feature, FeatureState) + Send + Sync>>>>,
}

impl FeatureGateManager {
    /// Create new feature gate manager
    pub fn new() -> Self {
        Self {
            features: Arc::new(RwLock::new(HashMap::new())),
            change_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Initialize with default features
    pub async fn initialize_defaults(&self) {
        let features = vec![
            (Feature::Camera, "Camera hardware access", true),
            (Feature::Microphone, "Microphone hardware access", true),
            (Feature::Display, "Display hardware access", false),
            (Feature::Sensors, "Sensor hardware access", true),
            (Feature::P2PNetworking, "Peer-to-peer networking", true),
            (Feature::InternetAccess, "Internet connectivity", true),
            (Feature::CloudSync, "Cloud synchronization", true),
            (Feature::FullBlockchain, "Full blockchain mode", true),
            (Feature::LightBlockchain, "Light blockchain mode", true),
            (Feature::Transactions, "Blockchain transactions", false),
            (Feature::SmartContracts, "Smart contract execution", true),
            (Feature::VisionModels, "Vision AI models", true),
            (Feature::VoiceRecognition, "Voice recognition", true),
            (Feature::NaturalLanguage, "Natural language processing", true),
            (Feature::KnowledgeGraph, "Knowledge graph", true),
            (Feature::ARTabs, "Augmented reality tabs", true),
            (Feature::HUD, "Heads-up display", false),
            (Feature::VoiceInterface, "Voice interface", true),
            (Feature::GestureControl, "Gesture control", true),
            (Feature::ThirdPartyApps, "Third-party apps", true),
            (Feature::NativeApps, "Native apps", true),
            (Feature::AppStore, "App store", true),
            (Feature::Telemetry, "Telemetry collection", true),
            (Feature::Diagnostics, "Diagnostics", true),
            (Feature::Updates, "Over-the-air updates", true),
            (Feature::Backup, "Cloud backup", true),
            (Feature::Collaboration, "Collaborative features", true),
            (Feature::Swarm, "Swarm intelligence", true),
            (Feature::SharedContext, "Shared context", true),
        ];
        
        let mut feature_map = self.features.write().await;
        for (feature, description, user_controllable) in features {
            feature_map.insert(
                feature.clone(),
                FeatureConfig {
                    feature: feature.clone(),
                    state: FeatureState::Enabled,
                    user_controllable,
                    dependencies: None,
                    description: description.to_string(),
                },
            );
        }
    }
    
    /// Check if a feature is enabled
    pub async fn is_enabled(&self, feature: &Feature) -> bool {
        let features = self.features.read().await;
        if let Some(config) = features.get(feature) {
            matches!(config.state, FeatureState::Enabled)
        } else {
            false
        }
    }
    
    /// Enable a feature
    pub async fn enable(&self, feature: &Feature) -> Result<()> {
        self.set_state(feature, FeatureState::Enabled).await
    }
    
    /// Disable a feature
    pub async fn disable(&self, feature: &Feature) -> Result<()> {
        self.set_state(feature, FeatureState::Disabled).await
    }
    
    /// Set feature state
    async fn set_state(&self, feature: &Feature, new_state: FeatureState) -> Result<()> {
        let mut features = self.features.write().await;
        
        let config = features.get_mut(feature)
            .ok_or_else(|| anyhow!("Feature not found: {:?}", feature))?;
        
        let old_state = config.state;
        
        // Cannot enable a killed feature directly
        if old_state == FeatureState::Killed && new_state == FeatureState::Enabled {
            return Err(anyhow!("Cannot enable killed feature - revive it first"));
        }
        
        // Check if feature is user controllable (except for kill switch)
        if !config.user_controllable && !matches!(new_state, FeatureState::Killed) {
            return Err(anyhow!("Feature is not user controllable"));
        }
        
        config.state = new_state;
        drop(features);
        
        // Update dependent features
        if matches!(new_state, FeatureState::Disabled | FeatureState::Killed) {
            self.disable_dependents(feature).await?;
        } else if matches!(new_state, FeatureState::Enabled) && old_state != FeatureState::Enabled {
            // Check if dependencies are satisfied
            if !self.check_dependencies(feature).await? {
                return Err(anyhow!("Dependencies not satisfied for {:?}", feature));
            }
        }
        
        // Notify listeners
        self.notify_listeners(feature, new_state).await;
        
        Ok(())
    }
    
    /// Emergency kill switch - immediately disable a feature
    pub async fn kill(&self, feature: &Feature) -> Result<()> {
        let mut features = self.features.write().await;
        
        let config = features.get_mut(feature)
            .ok_or_else(|| anyhow!("Feature not found: {:?}", feature))?;
        
        config.state = FeatureState::Killed;
        drop(features);
        
        // Disable all dependents
        self.disable_dependents(feature).await?;
        
        // Notify listeners
        self.notify_listeners(feature, FeatureState::Killed).await;
        
        Ok(())
    }
    
    /// Revive a killed feature
    pub async fn revive(&self, feature: &Feature) -> Result<()> {
        let mut features = self.features.write().await;
        
        let config = features.get_mut(feature)
            .ok_or_else(|| anyhow!("Feature not found: {:?}", feature))?;
        
        if config.state != FeatureState::Killed {
            return Err(anyhow!("Feature is not killed"));
        }
        
        config.state = FeatureState::Disabled;
        drop(features);
        
        Ok(())
    }
    
    /// Check if dependencies are satisfied
    async fn check_dependencies(&self, feature: &Feature) -> Result<bool> {
        let features = self.features.read().await;
        
        let config = features.get(feature)
            .ok_or_else(|| anyhow!("Feature not found: {:?}", feature))?;
        
        let deps = config.dependencies.clone()
            .unwrap_or_else(|| feature.default_dependencies());
        
        for dep in deps {
            if let Some(dep_config) = features.get(&dep) {
                if !matches!(dep_config.state, FeatureState::Enabled) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Disable all features that depend on this one
    async fn disable_dependents(&self, feature: &Feature) -> Result<()> {
        let features = self.features.read().await;
        
        // Find all features that depend on this one
        let mut dependents = Vec::new();
        for (f, config) in features.iter() {
            let deps = config.dependencies.clone()
                .unwrap_or_else(|| f.default_dependencies());
            
            if deps.contains(feature) {
                dependents.push(f.clone());
            }
        }
        drop(features);
        
        // Disable dependents
        for dependent in dependents {
            let mut features = self.features.write().await;
            if let Some(config) = features.get_mut(&dependent) {
                if matches!(config.state, FeatureState::Enabled) {
                    config.state = FeatureState::DisabledByDependency;
                }
            }
            drop(features);
            
            // Recursively disable their dependents using Box::pin for async recursion
            Box::pin(self.disable_dependents(&dependent)).await?;
        }
        
        Ok(())
    }
    
    /// Get all features and their states
    pub async fn get_all_features(&self) -> Vec<FeatureConfig> {
        self.features.read().await
            .values()
            .cloned()
            .collect()
    }
    
    /// Get features by state
    pub async fn get_features_by_state(&self, state: FeatureState) -> Vec<Feature> {
        self.features.read().await
            .values()
            .filter(|c| c.state == state)
            .map(|c| c.feature.clone())
            .collect()
    }
    
    /// Add change listener
    pub async fn add_listener<F>(&self, listener: F)
    where
        F: Fn(&Feature, FeatureState) + Send + Sync + 'static,
    {
        self.change_listeners.write().await.push(Box::new(listener));
    }
    
    /// Notify all listeners of a state change
    async fn notify_listeners(&self, feature: &Feature, state: FeatureState) {
        let listeners = self.change_listeners.read().await;
        for listener in listeners.iter() {
            listener(feature, state);
        }
    }
}

impl Default for FeatureGateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature_gate_creation() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        let features = manager.get_all_features().await;
        assert!(!features.is_empty());
    }
    
    #[tokio::test]
    async fn test_enable_disable_feature() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        // Disable camera
        manager.disable(&Feature::Camera).await.unwrap();
        assert!(!manager.is_enabled(&Feature::Camera).await);
        
        // Enable camera
        manager.enable(&Feature::Camera).await.unwrap();
        assert!(manager.is_enabled(&Feature::Camera).await);
    }
    
    #[tokio::test]
    async fn test_kill_switch() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        // Kill feature
        manager.kill(&Feature::VisionModels).await.unwrap();
        assert!(!manager.is_enabled(&Feature::VisionModels).await);
        
        // Cannot enable killed feature
        let result = manager.enable(&Feature::VisionModels).await;
        assert!(result.is_err());
        
        // Revive feature
        manager.revive(&Feature::VisionModels).await.unwrap();
        
        // Now can enable
        manager.enable(&Feature::VisionModels).await.unwrap();
        assert!(manager.is_enabled(&Feature::VisionModels).await);
    }
    
    #[tokio::test]
    async fn test_dependency_tracking() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        // Disable P2P networking
        manager.disable(&Feature::P2PNetworking).await.unwrap();
        
        // Internet access should be disabled by dependency
        let internet_config = manager.features.read().await
            .get(&Feature::InternetAccess)
            .cloned();
        
        if let Some(config) = internet_config {
            assert_eq!(config.state, FeatureState::DisabledByDependency);
        }
    }
    
    #[tokio::test]
    async fn test_get_features_by_state() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        manager.disable(&Feature::Telemetry).await.unwrap();
        
        let disabled = manager.get_features_by_state(FeatureState::Disabled).await;
        assert!(disabled.contains(&Feature::Telemetry));
    }
    
    #[tokio::test]
    async fn test_change_listener() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        let changed = Arc::new(RwLock::new(false));
        let changed_clone = Arc::clone(&changed);
        
        manager.add_listener(move |_feature, _state| {
            let changed = Arc::clone(&changed_clone);
            tokio::spawn(async move {
                *changed.write().await = true;
            });
        }).await;
        
        manager.disable(&Feature::Camera).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(*changed.read().await);
    }
    
    #[tokio::test]
    async fn test_non_controllable_feature() {
        let manager = FeatureGateManager::new();
        manager.initialize_defaults().await;
        
        // Display is not user controllable
        let result = manager.disable(&Feature::Display).await;
        assert!(result.is_err());
        
        // But can be killed
        manager.kill(&Feature::Display).await.unwrap();
        assert!(!manager.is_enabled(&Feature::Display).await);
    }
}
