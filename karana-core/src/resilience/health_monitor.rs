// Health Monitor - Per-layer health checks and circuit breakers
// Phase 48: Fault resilience through proactive monitoring

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status of a layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Layer is healthy
    Healthy,
    
    /// Layer is degraded but functional
    Degraded,
    
    /// Layer is unhealthy and should be avoided
    Unhealthy,
    
    /// Layer has failed and is offline
    Failed,
    
    /// Health status unknown
    Unknown,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed, operations allowed
    Closed,
    
    /// Circuit is open, operations blocked
    Open,
    
    /// Circuit is half-open, testing if recovery is possible
    HalfOpen,
}

/// Layer identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    Hardware,
    P2P,
    Ledger,
    Oracle,
    Intelligence,
    AI,
    Interface,
    Apps,
    System,
}

impl Layer {
    /// Get all layers in dependency order
    pub fn all() -> Vec<Layer> {
        vec![
            Layer::Hardware,
            Layer::P2P,
            Layer::Ledger,
            Layer::Oracle,
            Layer::Intelligence,
            Layer::AI,
            Layer::Interface,
            Layer::Apps,
            Layer::System,
        ]
    }
    
    /// Get layer name
    pub fn name(&self) -> &'static str {
        match self {
            Layer::Hardware => "hardware",
            Layer::P2P => "p2p",
            Layer::Ledger => "ledger",
            Layer::Oracle => "oracle",
            Layer::Intelligence => "intelligence",
            Layer::AI => "ai",
            Layer::Interface => "interface",
            Layer::Apps => "apps",
            Layer::System => "system",
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Layer being checked
    pub layer: Layer,
    
    /// Health status
    pub status: HealthStatus,
    
    /// Response time in milliseconds
    pub response_time_ms: u64,
    
    /// Error message if unhealthy
    pub error: Option<String>,
    
    /// Timestamp of check
    pub timestamp: u64,
}

/// Circuit breaker for a layer
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    opened_at: Option<Instant>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            opened_at: None,
            config,
        }
    }
    
    /// Record a successful operation
    fn record_success(&mut self) {
        self.success_count += 1;
        
        // In half-open state, enough successes close the circuit
        if self.state == CircuitState::HalfOpen && self.success_count >= self.config.half_open_successes {
            self.state = CircuitState::Closed;
            self.failure_count = 0;
            self.success_count = 0;
            self.opened_at = None;
        }
    }
    
    /// Record a failed operation
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        // Open circuit if threshold exceeded
        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
            self.opened_at = Some(Instant::now());
            self.success_count = 0;
        }
    }
    
    /// Check if operation is allowed
    fn is_allowed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.config.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.failure_count = 0;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    fn get_state(&self) -> CircuitState {
        self.state
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    
    /// Number of successes to close circuit from half-open
    pub half_open_successes: u32,
    
    /// Timeout before attempting recovery
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            half_open_successes: 3,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Health monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Health check interval
    pub check_interval: Duration,
    
    /// Timeout for health checks
    pub check_timeout: Duration,
    
    /// Circuit breaker config
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(10),
            check_timeout: Duration::from_secs(5),
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

/// Layer health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerHealth {
    pub layer: Layer,
    pub status: HealthStatus,
    pub circuit_state: CircuitState,
    pub last_check: Option<HealthCheckResult>,
    pub consecutive_failures: u32,
    pub uptime_percent: f32,
}

/// Health monitor for all layers
pub struct HealthMonitor {
    config: Arc<RwLock<HealthMonitorConfig>>,
    layer_health: Arc<RwLock<HashMap<Layer, HealthStatus>>>,
    circuit_breakers: Arc<RwLock<HashMap<Layer, CircuitBreaker>>>,
    check_results: Arc<RwLock<HashMap<Layer, Vec<HealthCheckResult>>>>,
    running: Arc<RwLock<bool>>,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HealthMonitorConfig::default())),
            layer_health: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            check_results: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: HealthMonitorConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            layer_health: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            check_results: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        // Initialize circuit breakers for all layers
        let config = self.config.read().await;
        let mut breakers = self.circuit_breakers.write().await;
        for layer in Layer::all() {
            breakers.insert(layer.clone(), CircuitBreaker::new(config.circuit_breaker.clone()));
        }
        drop(breakers);
        drop(config);
        
        // Start monitoring loop
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });
        
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }
    
    /// Main monitoring loop
    async fn monitoring_loop(&self) {
        while *self.running.read().await {
            let config = self.config.read().await;
            let interval = config.check_interval;
            drop(config);
            
            // Check all layers
            for layer in Layer::all() {
                if !*self.running.read().await {
                    break;
                }
                
                self.check_layer_health(&layer).await;
            }
            
            tokio::time::sleep(interval).await;
        }
    }
    
    /// Check health of a specific layer
    async fn check_layer_health(&self, layer: &Layer) {
        // Check if circuit breaker allows the check
        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers.get_mut(layer).unwrap();
        
        if !breaker.is_allowed() {
            // Circuit is open, mark as failed
            let mut health = self.layer_health.write().await;
            health.insert(layer.clone(), HealthStatus::Failed);
            return;
        }
        drop(breakers);
        
        // Perform health check
        let start = Instant::now();
        let result = self.perform_health_check(layer).await;
        let response_time_ms = start.elapsed().as_millis() as u64;
        
        let check_result = HealthCheckResult {
            layer: layer.clone(),
            status: result.0,
            response_time_ms,
            error: result.1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Update health status
        let mut health = self.layer_health.write().await;
        health.insert(layer.clone(), result.0);
        drop(health);
        
        // Update circuit breaker
        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers.get_mut(layer).unwrap();
        
        match result.0 {
            HealthStatus::Healthy | HealthStatus::Degraded => {
                breaker.record_success();
            }
            HealthStatus::Unhealthy | HealthStatus::Failed => {
                breaker.record_failure();
            }
            HealthStatus::Unknown => {}
        }
        drop(breakers);
        
        // Store check result
        let mut results = self.check_results.write().await;
        let history = results.entry(layer.clone()).or_insert_with(Vec::new);
        history.push(check_result);
        
        // Keep only last 100 results
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }
    
    /// Perform actual health check for a layer
    async fn perform_health_check(&self, layer: &Layer) -> (HealthStatus, Option<String>) {
        // In a real implementation, this would perform layer-specific checks
        // For now, simulate with placeholder logic
        
        match layer {
            Layer::Hardware => {
                // Check camera, sensors, display
                (HealthStatus::Healthy, None)
            }
            Layer::P2P => {
                // Check network connectivity
                (HealthStatus::Healthy, None)
            }
            Layer::Ledger => {
                // Check blockchain sync status
                (HealthStatus::Healthy, None)
            }
            Layer::Oracle => {
                // Check oracle connectivity
                (HealthStatus::Healthy, None)
            }
            Layer::Intelligence => {
                // Check intelligence services
                (HealthStatus::Healthy, None)
            }
            Layer::AI => {
                // Check AI models loaded and responsive
                (HealthStatus::Healthy, None)
            }
            Layer::Interface => {
                // Check UI rendering
                (HealthStatus::Healthy, None)
            }
            Layer::Apps => {
                // Check app ecosystem
                (HealthStatus::Healthy, None)
            }
            Layer::System => {
                // Check system services
                (HealthStatus::Healthy, None)
            }
        }
    }
    
    /// Get health status of a layer
    pub async fn get_layer_health(&self, layer: &Layer) -> HealthStatus {
        self.layer_health.read().await
            .get(layer)
            .copied()
            .unwrap_or(HealthStatus::Unknown)
    }
    
    /// Get circuit breaker state for a layer
    pub async fn get_circuit_state(&self, layer: &Layer) -> Option<CircuitState> {
        self.circuit_breakers.read().await
            .get(layer)
            .map(|b| b.get_state())
    }
    
    /// Get detailed health information for all layers
    pub async fn get_all_health(&self) -> Vec<LayerHealth> {
        let health = self.layer_health.read().await;
        let breakers = self.circuit_breakers.read().await;
        let results = self.check_results.read().await;
        
        Layer::all().iter().map(|layer| {
            let status = health.get(layer).copied().unwrap_or(HealthStatus::Unknown);
            let circuit_state = breakers.get(layer).map(|b| b.get_state()).unwrap_or(CircuitState::Closed);
            let last_check = results.get(layer).and_then(|r| r.last().cloned());
            
            let history = results.get(layer);
            let consecutive_failures = breakers.get(layer).map(|b| b.failure_count).unwrap_or(0);
            
            let uptime_percent = if let Some(history) = history {
                if history.is_empty() {
                    100.0
                } else {
                    let healthy_count = history.iter()
                        .filter(|r| matches!(r.status, HealthStatus::Healthy | HealthStatus::Degraded))
                        .count();
                    (healthy_count as f32 / history.len() as f32) * 100.0
                }
            } else {
                100.0
            };
            
            LayerHealth {
                layer: layer.clone(),
                status,
                circuit_state,
                last_check,
                consecutive_failures,
                uptime_percent,
            }
        }).collect()
    }
    
    /// Get list of unhealthy layers
    pub async fn get_unhealthy_layers(&self) -> Vec<Layer> {
        let health = self.layer_health.read().await;
        Layer::all().iter()
            .filter(|layer| {
                matches!(
                    health.get(layer).copied().unwrap_or(HealthStatus::Unknown),
                    HealthStatus::Unhealthy | HealthStatus::Failed
                )
            })
            .cloned()
            .collect()
    }
    
    /// Manually trigger health check for a layer
    pub async fn check_now(&self, layer: &Layer) -> HealthCheckResult {
        let start = Instant::now();
        let result = self.perform_health_check(layer).await;
        let response_time_ms = start.elapsed().as_millis() as u64;
        
        HealthCheckResult {
            layer: layer.clone(),
            status: result.0,
            response_time_ms,
            error: result.1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl Clone for HealthMonitor {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            layer_health: Arc::clone(&self.layer_health),
            circuit_breakers: Arc::clone(&self.circuit_breakers),
            check_results: Arc::clone(&self.check_results),
            running: Arc::clone(&self.running),
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        let health = monitor.get_all_health().await;
        
        // All layers should have unknown status initially
        for layer_health in health {
            assert_eq!(layer_health.status, HealthStatus::Unknown);
        }
    }
    
    #[tokio::test]
    async fn test_health_monitor_start() {
        let monitor = HealthMonitor::new();
        monitor.start().await.unwrap();
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        monitor.stop().await;
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            half_open_successes: 1,
            timeout: Duration::from_millis(100),
        };
        
        let mut breaker = CircuitBreaker::new(config);
        
        // Initially closed
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.is_allowed());
        
        // Record failures to open
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);
        assert!(!breaker.is_allowed());
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be half-open now
        assert!(breaker.is_allowed());
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);
        
        // Success closes circuit
        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }
    
    #[tokio::test]
    async fn test_layer_health_tracking() {
        let monitor = HealthMonitor::new();
        monitor.start().await.unwrap();
        
        // Wait for some checks
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let health = monitor.get_all_health().await;
        assert_eq!(health.len(), 9); // All layers
        
        monitor.stop().await;
    }
    
    #[tokio::test]
    async fn test_manual_health_check() {
        let monitor = HealthMonitor::new();
        
        let result = monitor.check_now(&Layer::Hardware).await;
        assert_eq!(result.layer, Layer::Hardware);
        assert!(result.response_time_ms < 1000);
    }
    
    #[tokio::test]
    async fn test_unhealthy_layers() {
        let monitor = HealthMonitor::new();
        
        // Initially no unhealthy layers
        let unhealthy = monitor.get_unhealthy_layers().await;
        assert_eq!(unhealthy.len(), 0);
    }
}
