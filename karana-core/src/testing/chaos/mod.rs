// Kāraṇa OS - Phase 56: Chaos Testing Suite
// Automated chaos scenarios for reliability validation

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use rand::Rng;

use super::fault_injection::{FaultInjection, FaultType, InjectionResult};

/// Chaos test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosScenario {
    pub name: String,
    pub description: String,
    pub duration: Duration,
    pub faults: Vec<FaultInjection>,
    pub concurrent: bool,  // Run faults concurrently or sequentially
    pub expected_outcome: ExpectedOutcome,
}

/// Expected outcome of chaos test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub should_recover: bool,
    pub max_recovery_time: Duration,
    pub allow_data_loss: bool,
    pub allow_user_visible_impact: bool,
}

/// Chaos test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosTestResult {
    pub scenario_name: String,
    pub started_at: u64,
    pub completed_at: u64,
    pub passed: bool,
    pub failures: Vec<String>,
    pub injection_results: Vec<InjectionResult>,
    pub recovery_summary: RecoverySummary,
}

/// Recovery summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySummary {
    pub total_faults: usize,
    pub recovered: usize,
    pub failed: usize,
    pub avg_recovery_time: Duration,
    pub max_recovery_time: Duration,
}

/// Chaos testing suite
pub struct ChaosTester {
    scenarios: Arc<RwLock<Vec<ChaosScenario>>>,
    test_results: Arc<RwLock<Vec<ChaosTestResult>>>,
    config: ChaosConfig,
}

/// Chaos testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    /// Enable chaos testing
    pub enabled: bool,
    
    /// Randomize fault timing
    pub randomize_timing: bool,
    
    /// Add random faults during test
    pub add_random_faults: bool,
    
    /// Random fault probability (0.0 to 1.0)
    pub random_fault_probability: f32,
    
    /// Maximum test duration
    pub max_test_duration: Duration,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            randomize_timing: true,
            add_random_faults: false,
            random_fault_probability: 0.1,
            max_test_duration: Duration::from_secs(300),
        }
    }
}

impl ChaosTester {
    /// Create new chaos tester
    pub fn new(config: ChaosConfig) -> Self {
        let mut scenarios = Vec::new();
        
        // Add predefined scenarios
        scenarios.push(Self::camera_failure_scenario());
        scenarios.push(Self::network_partition_scenario());
        scenarios.push(Self::resource_exhaustion_scenario());
        scenarios.push(Self::cascade_failure_scenario());
        scenarios.push(Self::thermal_stress_scenario());
        
        Self {
            scenarios: Arc::new(RwLock::new(scenarios)),
            test_results: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }
    
    /// Camera failure scenario
    fn camera_failure_scenario() -> ChaosScenario {
        ChaosScenario {
            name: "Camera Failure Recovery".to_string(),
            description: "Simulate camera hardware failure and verify graceful degradation".to_string(),
            duration: Duration::from_secs(30),
            faults: vec![
                FaultInjection::new(FaultType::CameraFailure, "camera_driver")
                    .with_duration(Duration::from_secs(10)),
            ],
            concurrent: false,
            expected_outcome: ExpectedOutcome {
                should_recover: true,
                max_recovery_time: Duration::from_secs(5),
                allow_data_loss: false,
                allow_user_visible_impact: true,  // Camera unavailable is user-visible
            },
        }
    }
    
    /// Network partition scenario
    fn network_partition_scenario() -> ChaosScenario {
        ChaosScenario {
            name: "Network Partition".to_string(),
            description: "Simulate complete network loss and verify offline operation".to_string(),
            duration: Duration::from_secs(60),
            faults: vec![
                FaultInjection::new(FaultType::NetworkPartition, "network_layer")
                    .with_duration(Duration::from_secs(30)),
            ],
            concurrent: false,
            expected_outcome: ExpectedOutcome {
                should_recover: true,
                max_recovery_time: Duration::from_secs(10),
                allow_data_loss: false,
                allow_user_visible_impact: false,  // Should work offline
            },
        }
    }
    
    /// Resource exhaustion scenario
    fn resource_exhaustion_scenario() -> ChaosScenario {
        ChaosScenario {
            name: "Resource Exhaustion".to_string(),
            description: "Simulate memory and storage pressure simultaneously".to_string(),
            duration: Duration::from_secs(45),
            faults: vec![
                FaultInjection::new(FaultType::MemoryPressure, "ml_runtime")
                    .with_duration(Duration::from_secs(20)),
                FaultInjection::new(FaultType::StorageFull, "ledger")
                    .with_duration(Duration::from_secs(20)),
            ],
            concurrent: true,
            expected_outcome: ExpectedOutcome {
                should_recover: true,
                max_recovery_time: Duration::from_secs(15),
                allow_data_loss: false,
                allow_user_visible_impact: true,
            },
        }
    }
    
    /// Cascade failure scenario
    fn cascade_failure_scenario() -> ChaosScenario {
        ChaosScenario {
            name: "Cascade Failure".to_string(),
            description: "Multiple sequential failures to test recovery resilience".to_string(),
            duration: Duration::from_secs(90),
            faults: vec![
                FaultInjection::new(FaultType::NetworkPartition, "network")
                    .with_duration(Duration::from_secs(10)),
                FaultInjection::new(FaultType::CameraFailure, "camera")
                    .with_duration(Duration::from_secs(10)),
                FaultInjection::new(FaultType::MemoryPressure, "ml")
                    .with_duration(Duration::from_secs(10)),
            ],
            concurrent: false,
            expected_outcome: ExpectedOutcome {
                should_recover: true,
                max_recovery_time: Duration::from_secs(30),
                allow_data_loss: false,
                allow_user_visible_impact: true,
            },
        }
    }
    
    /// Thermal stress scenario
    fn thermal_stress_scenario() -> ChaosScenario {
        ChaosScenario {
            name: "Thermal Emergency".to_string(),
            description: "Simulate thermal emergency and verify throttling response".to_string(),
            duration: Duration::from_secs(60),
            faults: vec![
                FaultInjection::new(FaultType::ThermalEmergency, "thermal_governor")
                    .with_duration(Duration::from_secs(30)),
            ],
            concurrent: false,
            expected_outcome: ExpectedOutcome {
                should_recover: true,
                max_recovery_time: Duration::from_secs(45),
                allow_data_loss: false,
                allow_user_visible_impact: true,
            },
        }
    }
    
    /// Run a specific chaos scenario
    pub async fn run_scenario(&self, scenario_name: &str) -> Result<ChaosTestResult> {
        if !self.config.enabled {
            return Err(anyhow!("Chaos testing is disabled"));
        }
        
        let scenarios = self.scenarios.read().await;
        let scenario = scenarios
            .iter()
            .find(|s| s.name == scenario_name)
            .ok_or_else(|| anyhow!("Scenario {} not found", scenario_name))?
            .clone();
        drop(scenarios);
        
        self.execute_scenario(scenario).await
    }
    
    /// Execute a chaos scenario
    async fn execute_scenario(&self, scenario: ChaosScenario) -> Result<ChaosTestResult> {
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let mut injection_results = Vec::new();
        let mut failures = Vec::new();
        
        if scenario.concurrent {
            // Run faults concurrently
            let handles: Vec<_> = scenario.faults
                .iter()
                .map(|fault| {
                    let fault = fault.clone();
                    tokio::spawn(async move {
                        // Simulate fault injection and recovery
                        tokio::time::sleep(fault.duration).await;
                        Self::simulate_injection_result(fault)
                    })
                })
                .collect();
            
            for handle in handles {
                match handle.await {
                    Ok(result) => injection_results.push(result),
                    Err(e) => failures.push(format!("Concurrent fault failed: {}", e)),
                }
            }
        } else {
            // Run faults sequentially
            for fault in &scenario.faults {
                let result = self.run_fault(fault).await?;
                
                if !result.recovered_successfully {
                    failures.push(format!(
                        "Fault {:?} did not recover successfully",
                        result.fault_type
                    ));
                }
                
                injection_results.push(result);
            }
        }
        
        let completed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Build recovery summary
        let recovery_summary = self.build_recovery_summary(&injection_results);
        
        // Check if test passed
        let passed = self.evaluate_test_pass(&scenario, &injection_results, &recovery_summary);
        
        if !passed && failures.is_empty() {
            failures.push("Test did not meet expected outcomes".to_string());
        }
        
        let result = ChaosTestResult {
            scenario_name: scenario.name.clone(),
            started_at,
            completed_at,
            passed,
            failures,
            injection_results,
            recovery_summary,
        };
        
        // Record result
        self.test_results.write().await.push(result.clone());
        
        Ok(result)
    }
    
    /// Run a single fault injection
    async fn run_fault(&self, fault: &FaultInjection) -> Result<InjectionResult> {
        // Add random timing variation if configured
        if self.config.randomize_timing {
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0..1000);
            tokio::time::sleep(Duration::from_millis(jitter)).await;
        }
        
        // Simulate fault duration
        tokio::time::sleep(fault.duration).await;
        
        Ok(Self::simulate_injection_result(fault.clone()))
    }
    
    /// Simulate an injection result (placeholder for actual implementation)
    fn simulate_injection_result(fault: FaultInjection) -> InjectionResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Simulate recovery based on severity
        let recovered = fault.fault_type.severity() < 9;
        let recovery_time = if recovered {
            Some(Duration::from_millis(
                (fault.fault_type.severity() as u64 * 500) + 500
            ))
        } else {
            None
        };
        
        InjectionResult {
            fault_type: fault.fault_type,
            target_component: fault.target_component,
            injected_at: now,
            duration: fault.duration,
            recovery_time,
            recovered_successfully: recovered,
            error_logs: vec![format!("Simulated fault: {:?}", fault.fault_type)],
            impact: super::fault_injection::ImpactAssessment {
                user_visible: fault.fault_type.severity() >= 7,
                data_loss: fault.fault_type == FaultType::LedgerCorruption,
                service_degradation: true,
                affected_layers: vec!["simulated".to_string()],
                recovery_actions: vec!["simulated recovery".to_string()],
            },
        }
    }
    
    /// Build recovery summary from results
    fn build_recovery_summary(&self, results: &[InjectionResult]) -> RecoverySummary {
        let total_faults = results.len();
        let recovered = results.iter().filter(|r| r.recovered_successfully).count();
        let failed = total_faults - recovered;
        
        let recovery_times: Vec<Duration> = results
            .iter()
            .filter_map(|r| r.recovery_time)
            .collect();
        
        let avg_recovery_time = if !recovery_times.is_empty() {
            let total: Duration = recovery_times.iter().sum();
            total / recovery_times.len() as u32
        } else {
            Duration::from_secs(0)
        };
        
        let max_recovery_time = recovery_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::from_secs(0));
        
        RecoverySummary {
            total_faults,
            recovered,
            failed,
            avg_recovery_time,
            max_recovery_time,
        }
    }
    
    /// Evaluate if test passed
    fn evaluate_test_pass(
        &self,
        scenario: &ChaosScenario,
        results: &[InjectionResult],
        summary: &RecoverySummary,
    ) -> bool {
        let expected = &scenario.expected_outcome;
        
        // Check recovery expectation
        if expected.should_recover && summary.failed > 0 {
            return false;
        }
        
        // Check max recovery time
        if summary.max_recovery_time > expected.max_recovery_time {
            return false;
        }
        
        // Check data loss constraint
        if !expected.allow_data_loss {
            if results.iter().any(|r| r.impact.data_loss) {
                return false;
            }
        }
        
        // Check user visibility constraint
        if !expected.allow_user_visible_impact {
            if results.iter().any(|r| r.impact.user_visible) {
                return false;
            }
        }
        
        true
    }
    
    /// Run all scenarios
    pub async fn run_all(&self) -> Result<Vec<ChaosTestResult>> {
        let scenarios = self.scenarios.read().await.clone();
        let mut results = Vec::new();
        
        for scenario in scenarios {
            match self.execute_scenario(scenario).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Scenario failed: {}", e),
            }
        }
        
        Ok(results)
    }
    
    /// Get test results
    pub async fn results(&self) -> Vec<ChaosTestResult> {
        self.test_results.read().await.clone()
    }
    
    /// Get pass rate
    pub async fn pass_rate(&self) -> f32 {
        let results = self.test_results.read().await;
        if results.is_empty() {
            return 0.0;
        }
        
        let passed = results.iter().filter(|r| r.passed).count();
        passed as f32 / results.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chaos_tester_creation() {
        let tester = ChaosTester::new(ChaosConfig::default());
        let scenarios = tester.scenarios.read().await;
        assert!(scenarios.len() >= 5);
    }
    
    #[tokio::test]
    async fn test_run_camera_scenario() {
        let mut config = ChaosConfig::default();
        config.enabled = true;
        
        let tester = ChaosTester::new(config);
        let result = tester.run_scenario("Camera Failure Recovery").await.unwrap();
        
        assert_eq!(result.scenario_name, "Camera Failure Recovery");
        assert!(!result.injection_results.is_empty());
    }
    
    #[tokio::test]
    async fn test_recovery_summary() {
        let mut config = ChaosConfig::default();
        config.enabled = true;
        
        let tester = ChaosTester::new(config);
        let result = tester.run_scenario("Network Partition").await.unwrap();
        
        assert_eq!(result.recovery_summary.total_faults, 1);
        assert!(result.recovery_summary.avg_recovery_time > Duration::from_secs(0));
    }
    
    #[tokio::test]
    async fn test_cascade_failure() {
        let mut config = ChaosConfig::default();
        config.enabled = true;
        
        let tester = ChaosTester::new(config);
        let result = tester.run_scenario("Cascade Failure").await.unwrap();
        
        assert_eq!(result.injection_results.len(), 3);
        assert!(result.recovery_summary.total_faults >= 3);
    }
    
    #[tokio::test]
    async fn test_pass_rate() {
        let mut config = ChaosConfig::default();
        config.enabled = true;
        
        let tester = ChaosTester::new(config);
        
        let _ = tester.run_scenario("Camera Failure Recovery").await;
        let _ = tester.run_scenario("Network Partition").await;
        
        let pass_rate = tester.pass_rate().await;
        assert!(pass_rate >= 0.0 && pass_rate <= 1.0);
    }
}
