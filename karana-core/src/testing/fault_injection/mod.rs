// Kāraṇa OS - Phase 56: Chaos Engineering & Fault Injection
// Proactive reliability testing through controlled failure injection

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Types of faults that can be injected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaultType {
    /// Camera hardware failure
    CameraFailure,
    
    /// Network partition or disconnect
    NetworkPartition,
    
    /// Partial blockchain/ledger corruption
    LedgerCorruption,
    
    /// Memory pressure (OOM scenarios)
    MemoryPressure,
    
    /// Disk full condition
    StorageFull,
    
    /// Model inference timeout
    ModelTimeout,
    
    /// Sensor data corruption
    SensorCorruption,
    
    /// Battery critical (forced shutdown)
    BatteryCritical,
    
    /// Thermal emergency
    ThermalEmergency,
    
    /// OTA update failure
    OTAFailure,
    
    /// Process crash
    ProcessCrash,
    
    /// Deadlock scenario
    Deadlock,
}

impl FaultType {
    /// Get typical duration for this fault
    pub fn typical_duration(&self) -> Duration {
        match self {
            Self::CameraFailure => Duration::from_secs(5),
            Self::NetworkPartition => Duration::from_secs(30),
            Self::LedgerCorruption => Duration::from_secs(10),
            Self::MemoryPressure => Duration::from_secs(20),
            Self::StorageFull => Duration::from_secs(15),
            Self::ModelTimeout => Duration::from_millis(500),
            Self::SensorCorruption => Duration::from_secs(3),
            Self::BatteryCritical => Duration::from_secs(60),
            Self::ThermalEmergency => Duration::from_secs(45),
            Self::OTAFailure => Duration::from_secs(120),
            Self::ProcessCrash => Duration::from_millis(100),
            Self::Deadlock => Duration::from_secs(5),
        }
    }
    
    /// Get severity level (0-10, higher is more severe)
    pub fn severity(&self) -> u8 {
        match self {
            Self::CameraFailure => 5,
            Self::NetworkPartition => 4,
            Self::LedgerCorruption => 8,
            Self::MemoryPressure => 7,
            Self::StorageFull => 6,
            Self::ModelTimeout => 3,
            Self::SensorCorruption => 5,
            Self::BatteryCritical => 9,
            Self::ThermalEmergency => 9,
            Self::OTAFailure => 7,
            Self::ProcessCrash => 8,
            Self::Deadlock => 10,
        }
    }
    
    /// Check if this fault requires immediate recovery
    pub fn requires_immediate_recovery(&self) -> bool {
        self.severity() >= 8
    }
}

/// Fault injection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultInjection {
    pub fault_type: FaultType,
    pub target_component: String,
    pub duration: Duration,
    pub delay_before_injection: Duration,
    pub probability: f32,  // 0.0 to 1.0
    pub repeat_count: usize,
    pub repeat_interval: Duration,
}

impl FaultInjection {
    /// Create a new fault injection with defaults
    pub fn new(fault_type: FaultType, target: &str) -> Self {
        Self {
            fault_type,
            target_component: target.to_string(),
            duration: fault_type.typical_duration(),
            delay_before_injection: Duration::from_secs(0),
            probability: 1.0,
            repeat_count: 1,
            repeat_interval: Duration::from_secs(10),
        }
    }
    
    /// Set probability of injection
    pub fn with_probability(mut self, prob: f32) -> Self {
        self.probability = prob.max(0.0).min(1.0);
        self
    }
    
    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
    
    /// Set repeat count
    pub fn with_repeats(mut self, count: usize, interval: Duration) -> Self {
        self.repeat_count = count;
        self.repeat_interval = interval;
        self
    }
}

/// Result of a fault injection test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    pub fault_type: FaultType,
    pub target_component: String,
    pub injected_at: u64,
    pub duration: Duration,
    pub recovery_time: Option<Duration>,
    pub recovered_successfully: bool,
    pub error_logs: Vec<String>,
    pub impact: ImpactAssessment,
}

/// Assessment of fault impact on system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub user_visible: bool,
    pub data_loss: bool,
    pub service_degradation: bool,
    pub affected_layers: Vec<String>,
    pub recovery_actions: Vec<String>,
}

/// Fault injection framework
pub struct FaultInjector {
    active_faults: Arc<RwLock<HashMap<String, ActiveFault>>>,
    injection_history: Arc<RwLock<Vec<InjectionResult>>>,
    config: InjectorConfig,
    stats: Arc<RwLock<InjectorStats>>,
}

/// Currently active fault
#[derive(Debug, Clone)]
struct ActiveFault {
    injection: FaultInjection,
    injected_at: u64,
    recovery_started_at: Option<u64>,
}

/// Injector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectorConfig {
    /// Enable fault injection
    pub enabled: bool,
    
    /// Maximum concurrent faults
    pub max_concurrent_faults: usize,
    
    /// Automatic recovery timeout
    pub recovery_timeout: Duration,
    
    /// Record detailed logs
    pub detailed_logging: bool,
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // Off by default for safety
            max_concurrent_faults: 3,
            recovery_timeout: Duration::from_secs(30),
            detailed_logging: true,
        }
    }
}

/// Injection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectorStats {
    pub total_injections: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub avg_recovery_time_ms: f32,
    pub faults_by_type: HashMap<String, usize>,
}

impl FaultInjector {
    /// Create new fault injector
    pub fn new(config: InjectorConfig) -> Self {
        Self {
            active_faults: Arc::new(RwLock::new(HashMap::new())),
            injection_history: Arc::new(RwLock::new(Vec::new())),
            config,
            stats: Arc::new(RwLock::new(InjectorStats::default())),
        }
    }
    
    /// Inject a fault into the system
    pub async fn inject(&self, injection: FaultInjection) -> Result<String> {
        if !self.config.enabled {
            return Err(anyhow!("Fault injection is disabled"));
        }
        
        let active = self.active_faults.read().await;
        if active.len() >= self.config.max_concurrent_faults {
            return Err(anyhow!(
                "Maximum concurrent faults ({}) reached",
                self.config.max_concurrent_faults
            ));
        }
        drop(active);
        
        // Check probability
        let random = rand::random::<f32>();
        if random > injection.probability {
            return Err(anyhow!(
                "Injection skipped due to probability ({:.2} > {:.2})",
                random,
                injection.probability
            ));
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let fault_id = format!("{}_{}", injection.target_component, now);
        
        let active_fault = ActiveFault {
            injection: injection.clone(),
            injected_at: now,
            recovery_started_at: None,
        };
        
        self.active_faults
            .write()
            .await
            .insert(fault_id.clone(), active_fault);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_injections += 1;
        *stats.faults_by_type
            .entry(format!("{:?}", injection.fault_type))
            .or_insert(0) += 1;
        
        Ok(fault_id)
    }
    
    /// Simulate recovery from a fault
    pub async fn recover(&self, fault_id: &str) -> Result<InjectionResult> {
        let mut active = self.active_faults.write().await;
        
        let fault = active.get_mut(fault_id)
            .ok_or_else(|| anyhow!("Fault {} not found", fault_id))?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        if fault.recovery_started_at.is_none() {
            fault.recovery_started_at = Some(now);
        }
        
        let recovery_time = Duration::from_secs(now - fault.recovery_started_at.unwrap());
        
        // Check if recovery timeout exceeded
        let recovered_successfully = recovery_time < self.config.recovery_timeout;
        
        // Simulate recovery actions based on fault type
        let (impact, recovery_actions) = self.assess_impact_and_recovery(&fault.injection);
        
        let result = InjectionResult {
            fault_type: fault.injection.fault_type,
            target_component: fault.injection.target_component.clone(),
            injected_at: fault.injected_at,
            duration: fault.injection.duration,
            recovery_time: Some(recovery_time),
            recovered_successfully,
            error_logs: self.collect_error_logs(&fault.injection),
            impact,
        };
        
        // Remove from active faults
        active.remove(fault_id);
        drop(active);
        
        // Record result
        self.injection_history.write().await.push(result.clone());
        
        // Update statistics
        let mut stats = self.stats.write().await;
        if recovered_successfully {
            stats.successful_recoveries += 1;
        } else {
            stats.failed_recoveries += 1;
        }
        
        let total_recoveries = stats.successful_recoveries + stats.failed_recoveries;
        if total_recoveries > 0 {
            stats.avg_recovery_time_ms = 
                (stats.avg_recovery_time_ms * (total_recoveries - 1) as f32 
                    + recovery_time.as_millis() as f32) 
                / total_recoveries as f32;
        }
        
        Ok(result)
    }
    
    /// Assess impact and determine recovery actions
    fn assess_impact_and_recovery(&self, injection: &FaultInjection) -> (ImpactAssessment, Vec<String>) {
        let mut recovery_actions = Vec::new();
        
        let (user_visible, data_loss, degradation, affected_layers) = match injection.fault_type {
            FaultType::CameraFailure => {
                recovery_actions.push("Restart camera driver".to_string());
                recovery_actions.push("Fallback to audio-only mode".to_string());
                (true, false, true, vec!["hardware".to_string(), "vision".to_string()])
            }
            
            FaultType::NetworkPartition => {
                recovery_actions.push("Switch to offline mode".to_string());
                recovery_actions.push("Queue operations for sync".to_string());
                (false, false, true, vec!["networking".to_string(), "oracle".to_string()])
            }
            
            FaultType::LedgerCorruption => {
                recovery_actions.push("Restore from last checkpoint".to_string());
                recovery_actions.push("Validate blockchain integrity".to_string());
                recovery_actions.push("Resync from peers if needed".to_string());
                (false, true, true, vec!["blockchain".to_string(), "ledger".to_string()])
            }
            
            FaultType::MemoryPressure => {
                recovery_actions.push("Unload unused models".to_string());
                recovery_actions.push("Clear caches".to_string());
                recovery_actions.push("Downgrade to lite models".to_string());
                (false, false, true, vec!["ml".to_string(), "ai_engine".to_string()])
            }
            
            FaultType::StorageFull => {
                recovery_actions.push("Prune old ledger data".to_string());
                recovery_actions.push("Compress historical data".to_string());
                recovery_actions.push("Delete cached files".to_string());
                (false, false, true, vec!["storage".to_string(), "ledger".to_string()])
            }
            
            FaultType::ModelTimeout => {
                recovery_actions.push("Cancel inference request".to_string());
                recovery_actions.push("Retry with lighter model".to_string());
                (false, false, true, vec!["ml".to_string()])
            }
            
            FaultType::BatteryCritical => {
                recovery_actions.push("Emergency power save mode".to_string());
                recovery_actions.push("Suspend all AI models".to_string());
                recovery_actions.push("Disable non-essential features".to_string());
                (true, false, true, vec!["power".to_string(), "all".to_string()])
            }
            
            FaultType::ThermalEmergency => {
                recovery_actions.push("Throttle all compute".to_string());
                recovery_actions.push("Offload to companion device".to_string());
                recovery_actions.push("Emergency cooldown period".to_string());
                (true, false, true, vec!["thermal".to_string(), "all".to_string()])
            }
            
            _ => {
                recovery_actions.push("Generic recovery procedure".to_string());
                (false, false, true, vec!["unknown".to_string()])
            }
        };
        
        let impact = ImpactAssessment {
            user_visible,
            data_loss,
            service_degradation: degradation,
            affected_layers,
            recovery_actions: recovery_actions.clone(),
        };
        
        (impact, recovery_actions)
    }
    
    /// Collect error logs for fault
    fn collect_error_logs(&self, injection: &FaultInjection) -> Vec<String> {
        vec![
            format!("FAULT: {:?} injected into {}", injection.fault_type, injection.target_component),
            format!("Duration: {:?}", injection.duration),
            format!("Severity: {}/10", injection.fault_type.severity()),
        ]
    }
    
    /// Get all active faults
    pub async fn active_faults(&self) -> Vec<(String, FaultInjection)> {
        self.active_faults
            .read()
            .await
            .iter()
            .map(|(id, f)| (id.clone(), f.injection.clone()))
            .collect()
    }
    
    /// Get injection history
    pub async fn history(&self) -> Vec<InjectionResult> {
        self.injection_history.read().await.clone()
    }
    
    /// Get statistics
    pub async fn stats(&self) -> InjectorStats {
        self.stats.read().await.clone()
    }
    
    /// Clear all active faults (for testing)
    pub async fn clear_all(&self) {
        self.active_faults.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fault_severity() {
        assert_eq!(FaultType::Deadlock.severity(), 10);
        assert_eq!(FaultType::ModelTimeout.severity(), 3);
        assert!(FaultType::BatteryCritical.requires_immediate_recovery());
        assert!(!FaultType::NetworkPartition.requires_immediate_recovery());
    }
    
    #[test]
    fn test_fault_injection_builder() {
        let injection = FaultInjection::new(FaultType::CameraFailure, "camera_driver")
            .with_probability(0.5)
            .with_duration(Duration::from_secs(10))
            .with_repeats(3, Duration::from_secs(5));
        
        assert_eq!(injection.probability, 0.5);
        assert_eq!(injection.duration, Duration::from_secs(10));
        assert_eq!(injection.repeat_count, 3);
    }
    
    #[tokio::test]
    async fn test_fault_injection() {
        let mut config = InjectorConfig::default();
        config.enabled = true;
        
        let injector = FaultInjector::new(config);
        
        let injection = FaultInjection::new(FaultType::NetworkPartition, "network_layer");
        let fault_id = injector.inject(injection).await.unwrap();
        
        assert!(!fault_id.is_empty());
        
        let active = injector.active_faults().await;
        assert_eq!(active.len(), 1);
    }
    
    #[tokio::test]
    async fn test_fault_recovery() {
        let mut config = InjectorConfig::default();
        config.enabled = true;
        config.recovery_timeout = Duration::from_secs(5);
        
        let injector = FaultInjector::new(config);
        
        let injection = FaultInjection::new(FaultType::CameraFailure, "camera");
        let fault_id = injector.inject(injection).await.unwrap();
        
        // Simulate some time passing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let result = injector.recover(&fault_id).await.unwrap();
        
        assert!(result.recovered_successfully);
        assert!(result.recovery_time.is_some());
        assert!(!result.impact.affected_layers.is_empty());
    }
    
    #[tokio::test]
    async fn test_max_concurrent_faults() {
        let mut config = InjectorConfig::default();
        config.enabled = true;
        config.max_concurrent_faults = 2;
        
        let injector = FaultInjector::new(config);
        
        let _ = injector.inject(FaultInjection::new(FaultType::CameraFailure, "cam1")).await;
        let _ = injector.inject(FaultInjection::new(FaultType::NetworkPartition, "net1")).await;
        
        // Third injection should fail
        let result = injector.inject(FaultInjection::new(FaultType::MemoryPressure, "mem1")).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_injection_stats() {
        let mut config = InjectorConfig::default();
        config.enabled = true;
        
        let injector = FaultInjector::new(config);
        
        // Inject and recover multiple faults
        for i in 0..3 {
            let injection = FaultInjection::new(
                FaultType::ModelTimeout,
                &format!("model_{}", i),
            );
            let fault_id = injector.inject(injection).await.unwrap();
            let _ = injector.recover(&fault_id).await;
        }
        
        let stats = injector.stats().await;
        assert_eq!(stats.total_injections, 3);
        assert_eq!(stats.successful_recoveries, 3);
    }
}
