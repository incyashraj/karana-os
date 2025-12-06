// Resilience Module - Fault tolerance and graceful degradation
// Phase 48: Comprehensive resilience system

pub mod minimal_mode;
pub mod health_monitor;
pub mod feature_gates;
pub mod chaos;

pub use minimal_mode::{
    MinimalModeManager, MinimalModeState, MinimalModeReason, MinimalModeConfig,
    MinimalModeStatistics, MinimalFeatures, MinimalFeature,
};

pub use health_monitor::{
    HealthMonitor, HealthStatus, CircuitState, Layer, HealthCheckResult,
    LayerHealth, HealthMonitorConfig, CircuitBreakerConfig,
};

pub use feature_gates::{
    FeatureGateManager, Feature, FeatureState, FeatureConfig,
};

pub use chaos::{
    ChaosTestFramework, ChaosScenario, ChaosTestConfig, ChaosTestResult,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resilience coordinator integrating all subsystems
pub struct ResilienceCoordinator {
    minimal_mode: Arc<MinimalModeManager>,
    health_monitor: Arc<HealthMonitor>,
    feature_gates: Arc<FeatureGateManager>,
    chaos_framework: Arc<ChaosTestFramework>,
    running: Arc<RwLock<bool>>,
}

impl ResilienceCoordinator {
    /// Create new resilience coordinator
    pub fn new() -> Self {
        Self {
            minimal_mode: Arc::new(MinimalModeManager::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
            feature_gates: Arc::new(FeatureGateManager::new()),
            chaos_framework: Arc::new(ChaosTestFramework::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start all resilience systems
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        // Initialize feature gates
        self.feature_gates.initialize_defaults().await;
        
        // Start health monitoring
        self.health_monitor.start().await?;
        
        // Start coordination loop
        let coordinator = self.clone();
        tokio::spawn(async move {
            coordinator.coordination_loop().await;
        });
        
        Ok(())
    }
    
    /// Stop all resilience systems
    pub async fn stop(&self) {
        *self.running.write().await = false;
        self.health_monitor.stop().await;
        self.chaos_framework.disable().await;
    }
    
    /// Main coordination loop
    async fn coordination_loop(&self) {
        while *self.running.read().await {
            // Check if we should activate minimal mode
            self.check_minimal_mode_activation().await;
            
            // Check for unhealthy layers and disable features
            self.handle_unhealthy_layers().await;
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
    
    /// Check if minimal mode should be activated
    async fn check_minimal_mode_activation(&self) {
        if self.minimal_mode.is_active().await {
            return; // Already active
        }
        
        // Get system health
        let unhealthy_layers = self.health_monitor.get_unhealthy_layers().await;
        
        // Activate minimal mode if critical systems are down
        if unhealthy_layers.len() >= 3 {
            let _ = self.minimal_mode.activate(
                MinimalModeReason::LayerFailures(
                    unhealthy_layers.iter().map(|l| l.name().to_string()).collect()
                )
            ).await;
        }
    }
    
    /// Handle unhealthy layers by disabling related features
    async fn handle_unhealthy_layers(&self) {
        let unhealthy = self.health_monitor.get_unhealthy_layers().await;
        
        for layer in unhealthy {
            // Map layer to features and disable them
            let features = self.map_layer_to_features(&layer);
            for feature in features {
                let _ = self.feature_gates.disable(&feature).await;
            }
        }
    }
    
    /// Map layer to features
    fn map_layer_to_features(&self, layer: &Layer) -> Vec<Feature> {
        match layer {
            Layer::Hardware => vec![Feature::Camera, Feature::Microphone, Feature::Display],
            Layer::P2P => vec![Feature::P2PNetworking, Feature::Collaboration],
            Layer::Ledger => vec![Feature::FullBlockchain, Feature::Transactions],
            Layer::Oracle => vec![],
            Layer::Intelligence => vec![Feature::KnowledgeGraph],
            Layer::AI => vec![Feature::VisionModels, Feature::VoiceRecognition, Feature::NaturalLanguage],
            Layer::Interface => vec![Feature::ARTabs, Feature::HUD],
            Layer::Apps => vec![Feature::ThirdPartyApps, Feature::NativeApps],
            Layer::System => vec![Feature::Updates, Feature::Diagnostics],
        }
    }
    
    /// Get minimal mode manager
    pub fn minimal_mode(&self) -> &MinimalModeManager {
        &self.minimal_mode
    }
    
    /// Get health monitor
    pub fn health_monitor(&self) -> &HealthMonitor {
        &self.health_monitor
    }
    
    /// Get feature gate manager
    pub fn feature_gates(&self) -> &FeatureGateManager {
        &self.feature_gates
    }
    
    /// Get chaos test framework
    pub fn chaos_framework(&self) -> &ChaosTestFramework {
        &self.chaos_framework
    }
    
    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> ResilienceSystemStatus {
        ResilienceSystemStatus {
            minimal_mode_active: self.minimal_mode.is_active().await,
            minimal_mode_reason: self.minimal_mode.get_current_reason().await,
            layer_health: self.health_monitor.get_all_health().await,
            enabled_features: self.feature_gates.get_features_by_state(FeatureState::Enabled).await.len(),
            disabled_features: self.feature_gates.get_features_by_state(FeatureState::Disabled).await.len(),
            chaos_testing_active: self.chaos_framework.is_enabled().await,
            active_chaos_scenarios: self.chaos_framework.get_active_scenarios().await.len(),
        }
    }
}

impl Clone for ResilienceCoordinator {
    fn clone(&self) -> Self {
        Self {
            minimal_mode: Arc::clone(&self.minimal_mode),
            health_monitor: Arc::clone(&self.health_monitor),
            feature_gates: Arc::clone(&self.feature_gates),
            chaos_framework: Arc::clone(&self.chaos_framework),
            running: Arc::clone(&self.running),
        }
    }
}

impl Default for ResilienceCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive resilience system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceSystemStatus {
    pub minimal_mode_active: bool,
    pub minimal_mode_reason: Option<MinimalModeReason>,
    pub layer_health: Vec<LayerHealth>,
    pub enabled_features: usize,
    pub disabled_features: usize,
    pub chaos_testing_active: bool,
    pub active_chaos_scenarios: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resilience_coordinator_creation() {
        let coordinator = ResilienceCoordinator::new();
        let status = coordinator.get_system_status().await;
        
        assert!(!status.minimal_mode_active);
        assert!(!status.chaos_testing_active);
    }
    
    #[tokio::test]
    async fn test_resilience_coordinator_start() {
        let coordinator = ResilienceCoordinator::new();
        coordinator.start().await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let status = coordinator.get_system_status().await;
        assert!(status.enabled_features > 0);
        
        coordinator.stop().await;
    }
    
    #[tokio::test]
    async fn test_minimal_mode_integration() {
        let coordinator = ResilienceCoordinator::new();
        coordinator.start().await.unwrap();
        
        // Manually activate minimal mode
        coordinator.minimal_mode().activate(MinimalModeReason::Manual).await.unwrap();
        
        let status = coordinator.get_system_status().await;
        assert!(status.minimal_mode_active);
        assert_eq!(status.minimal_mode_reason, Some(MinimalModeReason::Manual));
        
        coordinator.stop().await;
    }
}
