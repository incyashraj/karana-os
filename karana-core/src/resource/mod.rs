// Resource Management Module - Phase 46
// Adaptive resource monitoring and intelligent workload management

pub mod monitor;
pub mod adaptive_ledger;
pub mod ai_profiles;
pub mod thermal; // Phase 55

pub use monitor::{
    ResourceMonitor, ResourceSnapshot, ResourceLevel, 
    ResourcePrediction, ResourceStatistics,
};

pub use adaptive_ledger::{
    AdaptiveLedger, LedgerMode, IntentType,
    AdaptiveLedgerConfig, LedgerStatistics,
};

pub use ai_profiles::{
    AIProfileManager, AIProfile, AICapability,
    AIProfileConfig, AIProfileStatistics,
};

pub use thermal::{
    ThermalGovernor, ThermalState, ThermalConfig, ThermalPrediction,
    ThrottlingAction, ThermalStats,
};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Integrated resource management coordinator
pub struct ResourceCoordinator {
    pub monitor: Arc<ResourceMonitor>,
    pub ledger: Arc<AdaptiveLedger>,
    pub ai_profiles: Arc<AIProfileManager>,
    is_running: Arc<RwLock<bool>>,
}

impl ResourceCoordinator {
    /// Create new resource coordinator
    pub fn new() -> Self {
        let monitor = Arc::new(ResourceMonitor::new());
        let ledger = Arc::new(AdaptiveLedger::new(monitor.clone()));
        let ai_profiles = Arc::new(AIProfileManager::new(monitor.clone()));
        
        Self {
            monitor,
            ledger,
            ai_profiles,
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start all resource management subsystems
    pub async fn start(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        
        // Start resource monitoring
        self.monitor.start_monitoring().await;
        
        // Start auto-update loops
        self.ledger.start_auto_update().await;
        self.ai_profiles.start_auto_update().await;
        
        *running = true;
        
        Ok(())
    }
    
    /// Stop all resource management subsystems
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
    }
    
    /// Get comprehensive system status
    pub async fn get_status(&self) -> ResourceSystemStatus {
        let snapshot = self.monitor.get_snapshot().await;
        let resource_level = self.monitor.get_resource_level().await;
        let ledger_mode = self.ledger.get_mode().await;
        let ai_profile = self.ai_profiles.get_profile().await;
        
        ResourceSystemStatus {
            resource_level,
            cpu_usage: snapshot.cpu_usage,
            memory_usage_percent: snapshot.memory_usage_percent(),
            temperature: snapshot.temperature,
            battery_level: snapshot.battery_level,
            is_charging: snapshot.is_charging,
            ledger_mode,
            ai_profile,
            is_running: *self.is_running.read().await,
        }
    }
}

impl Default for ResourceCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive resource system status
#[derive(Debug, Clone)]
pub struct ResourceSystemStatus {
    pub resource_level: ResourceLevel,
    pub cpu_usage: f32,
    pub memory_usage_percent: f32,
    pub temperature: f32,
    pub battery_level: f32,
    pub is_charging: bool,
    pub ledger_mode: LedgerMode,
    pub ai_profile: AIProfile,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = ResourceCoordinator::new();
        
        let status = coordinator.get_status().await;
        assert!(!status.is_running);
    }
    
    #[tokio::test]
    async fn test_coordinator_start_stop() {
        let coordinator = ResourceCoordinator::new();
        
        coordinator.start().await.unwrap();
        let status = coordinator.get_status().await;
        assert!(status.is_running);
        
        // Wait a bit for systems to initialize
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        coordinator.stop().await;
        let status = coordinator.get_status().await;
        assert!(!status.is_running);
    }
    
    #[tokio::test]
    async fn test_integrated_status() {
        let coordinator = ResourceCoordinator::new();
        coordinator.start().await.unwrap();
        
        // Wait for monitoring to start
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        let status = coordinator.get_status().await;
        
        assert!(status.cpu_usage >= 0.0);
        assert!(status.memory_usage_percent >= 0.0);
        assert!(status.battery_level >= 0.0 && status.battery_level <= 100.0);
        assert!(matches!(status.resource_level, 
            ResourceLevel::Abundant | ResourceLevel::Normal | 
            ResourceLevel::Constrained | ResourceLevel::Critical
        ));
    }
}
