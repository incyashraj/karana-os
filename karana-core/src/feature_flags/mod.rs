// Kāraṇa OS - Phase 57: Feature Flag System
// Runtime feature toggling and build profile management

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Feature identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeatureId(pub String);

impl FeatureId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Feature flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: FeatureId,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub profile_requirements: Vec<BuildProfile>,
    pub dependencies: Vec<FeatureId>,
    pub experimental: bool,
    pub stability: FeatureStability,
}

/// Feature stability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureStability {
    /// Experimental, may be unstable
    Experimental,
    
    /// Beta, mostly stable but may change
    Beta,
    
    /// Stable, production-ready
    Stable,
    
    /// Deprecated, will be removed
    Deprecated,
}

/// Build profile levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildProfile {
    /// Minimal profile - core features only
    Minimal,
    
    /// Standard profile - most features
    Standard,
    
    /// Full profile - all features
    Full,
    
    /// Development profile - includes debug features
    Development,
}

impl BuildProfile {
    /// Check if this profile includes another profile's features
    pub fn includes(&self, other: BuildProfile) -> bool {
        match (self, other) {
            (BuildProfile::Full, _) => true,
            (BuildProfile::Development, _) => true,
            (BuildProfile::Standard, BuildProfile::Minimal) => true,
            (BuildProfile::Standard, BuildProfile::Standard) => true,
            (BuildProfile::Minimal, BuildProfile::Minimal) => true,
            _ => false,
        }
    }
    
    /// Get memory budget for this profile (MB)
    pub fn memory_budget_mb(&self) -> usize {
        match self {
            Self::Minimal => 256,
            Self::Standard => 512,
            Self::Full => 1024,
            Self::Development => 2048,
        }
    }
    
    /// Get compute budget (relative scale)
    pub fn compute_budget(&self) -> f32 {
        match self {
            Self::Minimal => 0.3,
            Self::Standard => 0.7,
            Self::Full => 1.0,
            Self::Development => 1.5,
        }
    }
}

/// Feature flag manager
pub struct FeatureFlagManager {
    features: Arc<RwLock<HashMap<FeatureId, Feature>>>,
    current_profile: Arc<RwLock<BuildProfile>>,
    overrides: Arc<RwLock<HashMap<FeatureId, bool>>>,
}

impl FeatureFlagManager {
    /// Create new feature flag manager
    pub fn new(profile: BuildProfile) -> Self {
        let manager = Self {
            features: Arc::new(RwLock::new(HashMap::new())),
            current_profile: Arc::new(RwLock::new(profile)),
            overrides: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Register default features
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = manager.register_default_features().await;
            })
        });
        
        manager
    }
    
    /// Register default Kāraṇa OS features
    async fn register_default_features(&self) -> Result<()> {
        // Core features (always enabled in Minimal+)
        self.register(Feature {
            id: FeatureId::new("voice_commands"),
            name: "Voice Commands".to_string(),
            description: "Voice control interface".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Minimal],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("camera_capture"),
            name: "Camera Capture".to_string(),
            description: "Photo and video capture".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Minimal],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("basic_ar"),
            name: "Basic AR".to_string(),
            description: "Simple AR overlays".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Minimal],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        // Standard features
        self.register(Feature {
            id: FeatureId::new("blockchain"),
            name: "Blockchain Ledger".to_string(),
            description: "Full blockchain consensus".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Standard],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("ai_models"),
            name: "AI Models".to_string(),
            description: "Local AI inference (Whisper, BLIP, TinyLlama)".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Standard],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("scene_understanding"),
            name: "Scene Understanding".to_string(),
            description: "Advanced spatial awareness".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Standard],
            dependencies: vec![FeatureId::new("ai_models")],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("multimodal_fusion"),
            name: "Multimodal Fusion".to_string(),
            description: "Voice + gesture + gaze fusion".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Standard],
            dependencies: vec![FeatureId::new("voice_commands")],
            experimental: false,
            stability: FeatureStability::Beta,
        }).await?;
        
        // Full features
        self.register(Feature {
            id: FeatureId::new("governance"),
            name: "Governance System".to_string(),
            description: "On-chain governance and voting".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Full],
            dependencies: vec![FeatureId::new("blockchain")],
            experimental: false,
            stability: FeatureStability::Stable,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("zk_proofs"),
            name: "Zero-Knowledge Proofs".to_string(),
            description: "Privacy-preserving computations".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Full],
            dependencies: vec![FeatureId::new("blockchain")],
            experimental: false,
            stability: FeatureStability::Beta,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("distributed_compute"),
            name: "Distributed Compute".to_string(),
            description: "Edge cloud integration".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Full],
            dependencies: vec![FeatureId::new("ai_models")],
            experimental: false,
            stability: FeatureStability::Beta,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("ar_collaboration"),
            name: "AR Collaboration".to_string(),
            description: "Multi-user shared AR spaces".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Full],
            dependencies: vec![FeatureId::new("basic_ar")],
            experimental: true,
            stability: FeatureStability::Experimental,
        }).await?;
        
        // Experimental features
        self.register(Feature {
            id: FeatureId::new("webxr_bridge"),
            name: "WebXR Bridge".to_string(),
            description: "Access web content in AR".to_string(),
            enabled: false,
            profile_requirements: vec![BuildProfile::Full],
            dependencies: vec![FeatureId::new("basic_ar")],
            experimental: true,
            stability: FeatureStability::Experimental,
        }).await?;
        
        self.register(Feature {
            id: FeatureId::new("wellness_monitoring"),
            name: "Wellness Monitoring".to_string(),
            description: "Health and wellness tracking".to_string(),
            enabled: false,
            profile_requirements: vec![BuildProfile::Standard],
            dependencies: vec![],
            experimental: true,
            stability: FeatureStability::Experimental,
        }).await?;
        
        Ok(())
    }
    
    /// Register a new feature
    pub async fn register(&self, feature: Feature) -> Result<()> {
        self.features
            .write()
            .await
            .insert(feature.id.clone(), feature);
        Ok(())
    }
    
    /// Check if a feature is enabled
    pub async fn is_enabled(&self, id: &FeatureId) -> bool {
        // Check overrides first
        if let Some(&enabled) = self.overrides.read().await.get(id) {
            return enabled;
        }
        
        // Check feature definition
        let features = self.features.read().await;
        if let Some(feature) = features.get(id) {
            // Check if current profile supports this feature
            let profile = *self.current_profile.read().await;
            let profile_ok = feature.profile_requirements.iter()
                .any(|req| profile.includes(*req));
            
            if !profile_ok {
                return false;
            }
            
            // Check dependencies
            for dep in &feature.dependencies {
                if !self.is_enabled_internal(&features, dep, &profile) {
                    return false;
                }
            }
            
            return feature.enabled;
        }
        
        false
    }
    
    /// Internal helper for dependency checking
    fn is_enabled_internal(
        &self,
        features: &HashMap<FeatureId, Feature>,
        id: &FeatureId,
        profile: &BuildProfile,
    ) -> bool {
        if let Some(feature) = features.get(id) {
            let profile_ok = feature.profile_requirements.iter()
                .any(|req| profile.includes(*req));
            profile_ok && feature.enabled
        } else {
            false
        }
    }
    
    /// Override a feature flag
    pub async fn set_override(&self, id: FeatureId, enabled: bool) -> Result<()> {
        self.overrides.write().await.insert(id, enabled);
        Ok(())
    }
    
    /// Remove override
    pub async fn clear_override(&self, id: &FeatureId) {
        self.overrides.write().await.remove(id);
    }
    
    /// Get all features
    pub async fn list_features(&self) -> Vec<Feature> {
        self.features.read().await.values().cloned().collect()
    }
    
    /// Get features by profile
    pub async fn features_for_profile(&self, profile: BuildProfile) -> Vec<Feature> {
        self.features
            .read()
            .await
            .values()
            .filter(|f| {
                f.profile_requirements.iter()
                    .any(|req| profile.includes(*req))
            })
            .cloned()
            .collect()
    }
    
    /// Set current build profile
    pub async fn set_profile(&self, profile: BuildProfile) {
        *self.current_profile.write().await = profile;
    }
    
    /// Get current profile
    pub async fn current_profile(&self) -> BuildProfile {
        *self.current_profile.read().await
    }
    
    /// Get feature statistics
    pub async fn stats(&self) -> FeatureStats {
        let features = self.features.read().await;
        let profile = *self.current_profile.read().await;
        
        let total = features.len();
        let mut enabled = 0;
        let mut experimental = 0;
        let mut by_stability = HashMap::new();
        
        for feature in features.values() {
            let feature_enabled = self.is_enabled_internal(&features, &feature.id, &profile);
            if feature_enabled {
                enabled += 1;
            }
            if feature.experimental {
                experimental += 1;
            }
            *by_stability.entry(format!("{:?}", feature.stability)).or_insert(0) += 1;
        }
        
        FeatureStats {
            total_features: total,
            enabled_features: enabled,
            experimental_features: experimental,
            current_profile: profile,
            features_by_stability: by_stability,
        }
    }
}

/// Feature statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    pub total_features: usize,
    pub enabled_features: usize,
    pub experimental_features: usize,
    pub current_profile: BuildProfile,
    pub features_by_stability: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_build_profile_hierarchy() {
        assert!(BuildProfile::Full.includes(BuildProfile::Minimal));
        assert!(BuildProfile::Full.includes(BuildProfile::Standard));
        assert!(BuildProfile::Standard.includes(BuildProfile::Minimal));
        assert!(!BuildProfile::Minimal.includes(BuildProfile::Standard));
    }
    
    #[tokio::test]
    async fn test_feature_registration() {
        let manager = FeatureFlagManager::new(BuildProfile::Standard);
        
        let feature = Feature {
            id: FeatureId::new("test_feature"),
            name: "Test".to_string(),
            description: "Test feature".to_string(),
            enabled: true,
            profile_requirements: vec![BuildProfile::Minimal],
            dependencies: vec![],
            experimental: false,
            stability: FeatureStability::Stable,
        };
        
        manager.register(feature).await.unwrap();
        assert!(manager.is_enabled(&FeatureId::new("test_feature")).await);
    }
    
    #[tokio::test]
    async fn test_profile_filtering() {
        let manager = FeatureFlagManager::new(BuildProfile::Minimal);
        
        // Blockchain requires Standard profile
        assert!(!manager.is_enabled(&FeatureId::new("blockchain")).await);
        
        // Voice commands work in Minimal
        assert!(manager.is_enabled(&FeatureId::new("voice_commands")).await);
        
        // Switch to Standard profile
        manager.set_profile(BuildProfile::Standard).await;
        assert!(manager.is_enabled(&FeatureId::new("blockchain")).await);
    }
    
    #[tokio::test]
    async fn test_feature_dependencies() {
        let manager = FeatureFlagManager::new(BuildProfile::Full);
        
        // Scene understanding depends on AI models
        assert!(manager.is_enabled(&FeatureId::new("scene_understanding")).await);
        
        // Disable AI models via override
        manager.set_override(FeatureId::new("ai_models"), false).await.unwrap();
        
        // Scene understanding should be disabled too
        assert!(!manager.is_enabled(&FeatureId::new("scene_understanding")).await);
    }
    
    #[tokio::test]
    async fn test_feature_overrides() {
        let manager = FeatureFlagManager::new(BuildProfile::Standard);
        
        // Enable experimental feature
        manager.set_override(FeatureId::new("webxr_bridge"), true).await.unwrap();
        assert!(manager.is_enabled(&FeatureId::new("webxr_bridge")).await);
        
        // Clear override
        manager.clear_override(&FeatureId::new("webxr_bridge")).await;
        assert!(!manager.is_enabled(&FeatureId::new("webxr_bridge")).await);
    }
    
    #[tokio::test]
    async fn test_feature_stats() {
        let manager = FeatureFlagManager::new(BuildProfile::Standard);
        let stats = manager.stats().await;
        
        assert!(stats.total_features > 0);
        assert!(stats.enabled_features > 0);
        assert!(stats.experimental_features > 0);
        assert_eq!(stats.current_profile, BuildProfile::Standard);
    }
    
    #[tokio::test]
    async fn test_memory_budgets() {
        assert_eq!(BuildProfile::Minimal.memory_budget_mb(), 256);
        assert_eq!(BuildProfile::Standard.memory_budget_mb(), 512);
        assert_eq!(BuildProfile::Full.memory_budget_mb(), 1024);
    }
}
