// Kāraṇa OS - Health Monitor
// System health checking and alerting

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{DiagnosticsConfig, AlertThresholds, MetricsSnapshot};

/// Health monitor for system components
pub struct HealthMonitor {
    /// Registered health checks
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
    /// Check history
    history: Vec<HealthCheckResult>,
    /// Max history size
    max_history: usize,
    /// Last check time per component
    last_checks: HashMap<String, Instant>,
}

/// Health check trait
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult;
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Component name
    pub component: String,
    /// Check passed
    pub passed: bool,
    /// Severity level
    pub severity: Severity,
    /// Message
    pub message: String,
    /// Current value (if applicable)
    pub value: Option<f64>,
    /// Threshold (if applicable)
    pub threshold: Option<f64>,
    /// Timestamp
    pub timestamp: Instant,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational
    Info,
    /// Warning - degraded performance
    Warning,
    /// Critical - immediate attention needed
    Critical,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(config: &DiagnosticsConfig) -> Self {
        let mut monitor = Self {
            checks: Vec::new(),
            history: Vec::new(),
            max_history: 1000,
            last_checks: HashMap::new(),
        };

        // Register default health checks
        monitor.register_default_checks();
        monitor
    }

    fn register_default_checks(&mut self) {
        self.checks.push(Box::new(CpuHealthCheck));
        self.checks.push(Box::new(MemoryHealthCheck));
        self.checks.push(Box::new(TemperatureHealthCheck));
        self.checks.push(Box::new(BatteryHealthCheck));
        self.checks.push(Box::new(FrameRateHealthCheck));
        self.checks.push(Box::new(StorageHealthCheck));
        self.checks.push(Box::new(NetworkHealthCheck));
    }

    /// Run all health checks
    pub fn check_all(&mut self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        // First collect all check results
        for check in &self.checks {
            let result = check.check(metrics, thresholds);
            results.push(result);
        }

        // Then update state
        for result in &results {
            self.last_checks.insert(result.component.clone(), result.timestamp);
            self.history.push(result.clone());
        }

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        results
    }

    /// Register custom health check
    pub fn register_check<C: HealthCheck + 'static>(&mut self, check: C) {
        self.checks.push(Box::new(check));
    }

    fn add_to_history(&mut self, result: HealthCheckResult) {
        self.history.push(result);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get failed checks
    pub fn get_failures(&self) -> Vec<&HealthCheckResult> {
        self.history.iter()
            .filter(|r| !r.passed)
            .collect()
    }

    /// Get checks by severity
    pub fn get_by_severity(&self, severity: Severity) -> Vec<&HealthCheckResult> {
        self.history.iter()
            .filter(|r| r.severity == severity)
            .collect()
    }
}

// Default health checks

struct CpuHealthCheck;
impl HealthCheck for CpuHealthCheck {
    fn name(&self) -> &str { "cpu" }
    
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult {
        let cpu_usage = metrics.cpu_usage;
        let (passed, severity) = if cpu_usage >= thresholds.cpu_critical {
            (false, Severity::Critical)
        } else if cpu_usage >= thresholds.cpu_warning {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "cpu".to_string(),
            passed,
            severity,
            message: format!("CPU usage: {:.1}%", cpu_usage),
            value: Some(cpu_usage as f64),
            threshold: Some(thresholds.cpu_warning as f64),
            timestamp: Instant::now(),
        }
    }
}

struct MemoryHealthCheck;
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str { "memory" }
    
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult {
        let memory_usage = metrics.memory_usage;
        let (passed, severity) = if memory_usage >= thresholds.memory_critical {
            (false, Severity::Critical)
        } else if memory_usage >= thresholds.memory_warning {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "memory".to_string(),
            passed,
            severity,
            message: format!("Memory usage: {:.1}%", memory_usage),
            value: Some(memory_usage as f64),
            threshold: Some(thresholds.memory_warning as f64),
            timestamp: Instant::now(),
        }
    }
}

struct TemperatureHealthCheck;
impl HealthCheck for TemperatureHealthCheck {
    fn name(&self) -> &str { "temperature" }
    
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult {
        let temp = metrics.temperature;
        let (passed, severity) = if temp >= thresholds.temp_critical {
            (false, Severity::Critical)
        } else if temp >= thresholds.temp_warning {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "temperature".to_string(),
            passed,
            severity,
            message: format!("Temperature: {:.1}°C", temp),
            value: Some(temp as f64),
            threshold: Some(thresholds.temp_warning as f64),
            timestamp: Instant::now(),
        }
    }
}

struct BatteryHealthCheck;
impl HealthCheck for BatteryHealthCheck {
    fn name(&self) -> &str { "battery" }
    
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult {
        let battery = metrics.battery_level;
        let (passed, severity) = if battery <= thresholds.battery_critical {
            (false, Severity::Critical)
        } else if battery <= thresholds.battery_warning {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "battery".to_string(),
            passed,
            severity,
            message: format!("Battery: {:.0}%", battery),
            value: Some(battery as f64),
            threshold: Some(thresholds.battery_warning as f64),
            timestamp: Instant::now(),
        }
    }
}

struct FrameRateHealthCheck;
impl HealthCheck for FrameRateHealthCheck {
    fn name(&self) -> &str { "framerate" }
    
    fn check(&self, metrics: &MetricsSnapshot, thresholds: &AlertThresholds) -> HealthCheckResult {
        let drop_rate = metrics.frame_drop_rate;
        let (passed, severity) = if drop_rate > thresholds.frame_drop_warning * 2.0 {
            (false, Severity::Critical)
        } else if drop_rate > thresholds.frame_drop_warning {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "framerate".to_string(),
            passed,
            severity,
            message: format!("Frame drop rate: {:.1}%", drop_rate),
            value: Some(drop_rate as f64),
            threshold: Some(thresholds.frame_drop_warning as f64),
            timestamp: Instant::now(),
        }
    }
}

struct StorageHealthCheck;
impl HealthCheck for StorageHealthCheck {
    fn name(&self) -> &str { "storage" }
    
    fn check(&self, metrics: &MetricsSnapshot, _thresholds: &AlertThresholds) -> HealthCheckResult {
        let storage_pct = (metrics.storage_used as f32 / metrics.storage_total.max(1) as f32) * 100.0;
        let (passed, severity) = if storage_pct >= 95.0 {
            (false, Severity::Critical)
        } else if storage_pct >= 85.0 {
            (true, Severity::Warning)
        } else {
            (true, Severity::Info)
        };

        HealthCheckResult {
            component: "storage".to_string(),
            passed,
            severity,
            message: format!("Storage: {:.1}% used", storage_pct),
            value: Some(storage_pct as f64),
            threshold: Some(85.0),
            timestamp: Instant::now(),
        }
    }
}

struct NetworkHealthCheck;
impl HealthCheck for NetworkHealthCheck {
    fn name(&self) -> &str { "network" }
    
    fn check(&self, metrics: &MetricsSnapshot, _thresholds: &AlertThresholds) -> HealthCheckResult {
        let (passed, severity, message) = if !metrics.network_connected {
            (false, Severity::Warning, "Network disconnected".to_string())
        } else if metrics.network_latency_ms > 500.0 {
            (true, Severity::Warning, format!("High latency: {:.0}ms", metrics.network_latency_ms))
        } else {
            (true, Severity::Info, format!("Network OK: {:.0}ms latency", metrics.network_latency_ms))
        };

        HealthCheckResult {
            component: "network".to_string(),
            passed,
            severity,
            message,
            value: Some(metrics.network_latency_ms as f64),
            threshold: Some(500.0),
            timestamp: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_metrics() -> MetricsSnapshot {
        MetricsSnapshot {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            memory_used_bytes: 500_000_000,
            memory_total_bytes: 1_000_000_000,
            temperature: 35.0,
            battery_level: 80.0,
            battery_charging: false,
            frame_rate: 60.0,
            frame_drop_rate: 1.0,
            storage_used: 50_000_000_000,
            storage_total: 100_000_000_000,
            network_connected: true,
            network_latency_ms: 50.0,
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_cpu_health_check() {
        let check = CpuHealthCheck;
        let thresholds = AlertThresholds::default();
        
        let mut metrics = mock_metrics();
        let result = check.check(&metrics, &thresholds);
        assert!(result.passed);
        assert_eq!(result.severity, Severity::Info);

        // Test warning
        metrics.cpu_usage = 75.0;
        let result = check.check(&metrics, &thresholds);
        assert!(result.passed);
        assert_eq!(result.severity, Severity::Warning);

        // Test critical
        metrics.cpu_usage = 95.0;
        let result = check.check(&metrics, &thresholds);
        assert!(!result.passed);
        assert_eq!(result.severity, Severity::Critical);
    }

    #[test]
    fn test_memory_health_check() {
        let check = MemoryHealthCheck;
        let thresholds = AlertThresholds::default();
        let metrics = mock_metrics();
        
        let result = check.check(&metrics, &thresholds);
        assert!(result.passed);
    }

    #[test]
    fn test_battery_health_check() {
        let check = BatteryHealthCheck;
        let thresholds = AlertThresholds::default();
        
        let mut metrics = mock_metrics();
        metrics.battery_level = 5.0;
        
        let result = check.check(&metrics, &thresholds);
        assert!(!result.passed);
        assert_eq!(result.severity, Severity::Critical);
    }

    #[test]
    fn test_health_monitor() {
        let config = DiagnosticsConfig::default();
        let mut monitor = HealthMonitor::new(&config);
        let metrics = mock_metrics();
        let thresholds = AlertThresholds::default();
        
        let results = monitor.check_all(&metrics, &thresholds);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
    }
}
