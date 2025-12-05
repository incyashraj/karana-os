// Kāraṇa OS - System Diagnostics
// Health monitoring, metrics collection, and system diagnostics

pub mod health;
pub mod metrics;
pub mod profiler;
pub mod watchdog;

pub use health::*;
pub use metrics::*;
pub use profiler::*;
pub use watchdog::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// System diagnostics coordinator
pub struct SystemDiagnostics {
    /// Health monitor
    health: HealthMonitor,
    /// Metrics collector
    metrics: MetricsCollector,
    /// System profiler
    profiler: SystemProfiler,
    /// Watchdog
    watchdog: SystemWatchdog,
    /// Diagnostic state
    state: DiagnosticState,
    /// Configuration
    config: DiagnosticsConfig,
}

/// Diagnostic state
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticState {
    /// Normal operation
    Healthy,
    /// Degraded but functional
    Degraded(Vec<String>),
    /// Critical issues detected
    Critical(Vec<String>),
    /// System in recovery mode
    Recovery,
}

/// Diagnostics configuration
#[derive(Debug, Clone)]
pub struct DiagnosticsConfig {
    /// Health check interval (ms)
    pub health_check_interval_ms: u64,
    /// Metrics collection interval (ms)
    pub metrics_interval_ms: u64,
    /// Enable detailed profiling
    pub profiling_enabled: bool,
    /// Watchdog timeout (ms)
    pub watchdog_timeout_ms: u64,
    /// Max metrics history
    pub max_metrics_history: usize,
    /// Alert thresholds
    pub thresholds: AlertThresholds,
}

/// Alert thresholds
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// CPU usage warning (%)
    pub cpu_warning: f32,
    /// CPU usage critical (%)
    pub cpu_critical: f32,
    /// Memory usage warning (%)
    pub memory_warning: f32,
    /// Memory usage critical (%)
    pub memory_critical: f32,
    /// Temperature warning (°C)
    pub temp_warning: f32,
    /// Temperature critical (°C)
    pub temp_critical: f32,
    /// Battery warning (%)
    pub battery_warning: f32,
    /// Battery critical (%)
    pub battery_critical: f32,
    /// Frame drop warning threshold
    pub frame_drop_warning: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            temp_warning: 40.0,
            temp_critical: 50.0,
            battery_warning: 20.0,
            battery_critical: 10.0,
            frame_drop_warning: 5.0,
        }
    }
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            health_check_interval_ms: 1000,
            metrics_interval_ms: 100,
            profiling_enabled: false,
            watchdog_timeout_ms: 5000,
            max_metrics_history: 1000,
            thresholds: AlertThresholds::default(),
        }
    }
}

impl SystemDiagnostics {
    /// Create new diagnostics system
    pub fn new() -> Self {
        Self::with_config(DiagnosticsConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: DiagnosticsConfig) -> Self {
        Self {
            health: HealthMonitor::new(&config),
            metrics: MetricsCollector::new(config.max_metrics_history),
            profiler: SystemProfiler::new(),
            watchdog: SystemWatchdog::new(config.watchdog_timeout_ms),
            state: DiagnosticState::Healthy,
            config,
        }
    }

    /// Run diagnostic check
    pub fn run_diagnostics(&mut self) -> DiagnosticReport {
        let start = Instant::now();

        // Collect current metrics
        let metrics = self.metrics.collect_snapshot();

        // Run health checks
        let health_results = self.health.check_all(&metrics, &self.config.thresholds);

        // Update state based on health results
        self.state = self.determine_state(&health_results);

        // Pet watchdog
        self.watchdog.pet();

        DiagnosticReport {
            timestamp: Instant::now(),
            state: self.state.clone(),
            metrics,
            health_results,
            duration: start.elapsed(),
        }
    }

    fn determine_state(&self, results: &[HealthCheckResult]) -> DiagnosticState {
        let critical: Vec<_> = results.iter()
            .filter(|r| r.severity == Severity::Critical)
            .map(|r| r.message.clone())
            .collect();

        let warnings: Vec<_> = results.iter()
            .filter(|r| r.severity == Severity::Warning)
            .map(|r| r.message.clone())
            .collect();

        if !critical.is_empty() {
            DiagnosticState::Critical(critical)
        } else if !warnings.is_empty() {
            DiagnosticState::Degraded(warnings)
        } else {
            DiagnosticState::Healthy
        }
    }

    /// Get current state
    pub fn state(&self) -> &DiagnosticState {
        &self.state
    }

    /// Start profiling session
    pub fn start_profiling(&mut self, name: &str) -> ProfilingSession {
        self.profiler.start_session(name)
    }

    /// Get metrics history
    pub fn metrics_history(&self) -> &[MetricsSnapshot] {
        self.metrics.history()
    }

    /// Register watchdog callback
    pub fn on_watchdog_timeout<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.watchdog.on_timeout(callback);
    }

    /// Is system healthy?
    pub fn is_healthy(&self) -> bool {
        matches!(self.state, DiagnosticState::Healthy)
    }
}

impl Default for SystemDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic report
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// Report timestamp
    pub timestamp: Instant,
    /// System state
    pub state: DiagnosticState,
    /// Current metrics
    pub metrics: MetricsSnapshot,
    /// Health check results
    pub health_results: Vec<HealthCheckResult>,
    /// Time to generate report
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_creation() {
        let diag = SystemDiagnostics::new();
        assert!(diag.is_healthy());
    }

    #[test]
    fn test_default_thresholds() {
        let thresholds = AlertThresholds::default();
        assert!((thresholds.cpu_warning - 70.0).abs() < 0.01);
        assert!((thresholds.temp_critical - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_run_diagnostics() {
        let mut diag = SystemDiagnostics::new();
        let report = diag.run_diagnostics();
        assert!(report.duration.as_millis() < 1000);
    }

    #[test]
    fn test_diagnostic_state() {
        let state = DiagnosticState::Healthy;
        assert_eq!(state, DiagnosticState::Healthy);

        let degraded = DiagnosticState::Degraded(vec!["test".to_string()]);
        assert!(matches!(degraded, DiagnosticState::Degraded(_)));
    }
}
