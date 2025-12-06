// Kāraṇa OS - Phase 56: Testing Module
// Comprehensive reliability testing framework

pub mod fault_injection;
pub mod chaos;
pub mod recovery;

pub use fault_injection::{
    FaultInjector, FaultInjection, FaultType, InjectionResult,
    InjectorConfig, InjectorStats, ImpactAssessment,
};

pub use chaos::{
    ChaosTester, ChaosScenario, ChaosTestResult, ChaosConfig,
    ExpectedOutcome, RecoverySummary,
};

pub use recovery::{
    RecoveryValidator, RecoveryCheck, ValidationResult, ValidationReport,
    CheckType,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Integrated reliability testing framework
pub struct ReliabilityTester {
    fault_injector: Arc<FaultInjector>,
    chaos_tester: Arc<ChaosTester>,
    recovery_validator: Arc<RecoveryValidator>,
    enabled: Arc<RwLock<bool>>,
}

impl ReliabilityTester {
    /// Create new reliability tester
    pub fn new() -> Self {
        let injector_config = InjectorConfig {
            enabled: false,  // Disabled by default
            max_concurrent_faults: 3,
            ..Default::default()
        };
        
        let chaos_config = ChaosConfig {
            enabled: false,  // Disabled by default
            ..Default::default()
        };
        
        Self {
            fault_injector: Arc::new(FaultInjector::new(injector_config)),
            chaos_tester: Arc::new(ChaosTester::new(chaos_config)),
            recovery_validator: Arc::new(RecoveryValidator::new()),
            enabled: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Enable reliability testing (USE WITH CAUTION - causes real failures)
    pub async fn enable(&self) {
        *self.enabled.write().await = true;
    }
    
    /// Disable reliability testing
    pub async fn disable(&self) {
        *self.enabled.write().await = false;
    }
    
    /// Run complete reliability test suite
    pub async fn run_full_suite(&self) -> Result<ReliabilityReport> {
        if !*self.enabled.read().await {
            return Err(anyhow::anyhow!("Reliability testing is disabled"));
        }
        
        // Run chaos tests
        let chaos_results = self.chaos_tester.run_all().await?;
        
        // Run recovery validation
        let validation_results = self.recovery_validator.validate().await?;
        let validation_report = self.recovery_validator.generate_report(&validation_results);
        
        // Get injection stats
        let injection_stats = self.fault_injector.stats().await;
        
        // Build comprehensive report
        let chaos_pass_rate = self.chaos_tester.pass_rate().await;
        let validation_pass_rate = validation_report.pass_rate;
        
        Ok(ReliabilityReport {
            chaos_tests_run: chaos_results.len(),
            chaos_tests_passed: chaos_results.iter().filter(|r| r.passed).count(),
            chaos_pass_rate,
            validation_report,
            injection_stats,
            overall_reliability_score: self.calculate_reliability_score(
                chaos_pass_rate,
                validation_pass_rate,
            ),
        })
    }
    
    /// Calculate overall reliability score
    fn calculate_reliability_score(&self, chaos_pass_rate: f32, validation_pass_rate: f32) -> f32 {
        // Weighted average: chaos tests 60%, validation 40%
        (chaos_pass_rate * 0.6) + (validation_pass_rate * 0.4)
    }
    
    /// Get fault injector
    pub fn fault_injector(&self) -> Arc<FaultInjector> {
        self.fault_injector.clone()
    }
    
    /// Get chaos tester
    pub fn chaos_tester(&self) -> Arc<ChaosTester> {
        self.chaos_tester.clone()
    }
    
    /// Get recovery validator
    pub fn recovery_validator(&self) -> Arc<RecoveryValidator> {
        self.recovery_validator.clone()
    }
}

/// Comprehensive reliability report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityReport {
    pub chaos_tests_run: usize,
    pub chaos_tests_passed: usize,
    pub chaos_pass_rate: f32,
    pub validation_report: ValidationReport,
    pub injection_stats: InjectorStats,
    pub overall_reliability_score: f32,
}

impl ReliabilityReport {
    /// Check if system meets reliability threshold
    pub fn meets_threshold(&self, min_score: f32) -> bool {
        self.overall_reliability_score >= min_score
    }
    
    /// Generate summary string
    pub fn summary(&self) -> String {
        format!(
            "Reliability Score: {:.1}%\nChaos Tests: {}/{} passed ({:.1}%)\n{}",
            self.overall_reliability_score * 100.0,
            self.chaos_tests_passed,
            self.chaos_tests_run,
            self.chaos_pass_rate * 100.0,
            self.validation_report.summary()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reliability_tester_creation() {
        let tester = ReliabilityTester::new();
        assert!(!*tester.enabled.read().await);
    }
    
    #[tokio::test]
    async fn test_enable_disable() {
        let tester = ReliabilityTester::new();
        
        tester.enable().await;
        assert!(*tester.enabled.read().await);
        
        tester.disable().await;
        assert!(!*tester.enabled.read().await);
    }
    
    #[test]
    fn test_reliability_score_calculation() {
        let tester = ReliabilityTester::new();
        
        let score = tester.calculate_reliability_score(0.9, 1.0);
        assert!((score - 0.94).abs() < 0.01);
    }
}
