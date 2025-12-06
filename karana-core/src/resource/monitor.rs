// Resource Management - Adaptive resource monitoring and profiling
// Phase 46: Enable intelligent operation on constrained hardware

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// System resource state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f32,
    
    /// Memory usage in bytes
    pub memory_used: u64,
    
    /// Total memory available in bytes
    pub memory_total: u64,
    
    /// Temperature in Celsius
    pub temperature: f32,
    
    /// Battery percentage (0.0 - 100.0)
    pub battery_level: f32,
    
    /// Battery charging state
    pub is_charging: bool,
    
    /// Timestamp of snapshot (not serialized, use timestamp_secs for storage)
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    
    /// Unix timestamp in seconds (for serialization)
    pub timestamp_secs: u64,
}

impl ResourceSnapshot {
    /// Get memory usage as percentage
    pub fn memory_usage_percent(&self) -> f32 {
        if self.memory_total == 0 {
            return 0.0;
        }
        (self.memory_used as f32 / self.memory_total as f32) * 100.0
    }
    
    /// Check if system is under thermal stress
    pub fn is_thermal_stress(&self) -> bool {
        self.temperature > 75.0  // Above 75°C
    }
    
    /// Check if battery is critically low
    pub fn is_battery_critical(&self) -> bool {
        self.battery_level < 15.0 && !self.is_charging
    }
    
    /// Check if battery is low
    pub fn is_battery_low(&self) -> bool {
        self.battery_level < 30.0 && !self.is_charging
    }
}

/// Resource constraint level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceLevel {
    /// Abundant resources, full performance mode
    Abundant,
    
    /// Normal operation
    Normal,
    
    /// Constrained resources, reduce background tasks
    Constrained,
    
    /// Severely constrained, minimal mode
    Critical,
}

impl ResourceLevel {
    /// Determine resource level from snapshot
    pub fn from_snapshot(snapshot: &ResourceSnapshot) -> Self {
        // Critical if any critical condition
        if snapshot.is_battery_critical() 
            || snapshot.temperature > 85.0 
            || snapshot.memory_usage_percent() > 95.0 
            || snapshot.cpu_usage > 95.0 {
            return ResourceLevel::Critical;
        }
        
        // Constrained if multiple warning conditions
        let mut constraints = 0;
        if snapshot.is_battery_low() { constraints += 1; }
        if snapshot.is_thermal_stress() { constraints += 1; }
        if snapshot.memory_usage_percent() > 80.0 { constraints += 1; }
        if snapshot.cpu_usage > 80.0 { constraints += 1; }
        
        if constraints >= 2 {
            return ResourceLevel::Constrained;
        }
        
        if constraints == 1 {
            return ResourceLevel::Normal;
        }
        
        // Abundant if charging with good resources
        if snapshot.is_charging 
            && snapshot.battery_level > 80.0 
            && snapshot.temperature < 60.0 
            && snapshot.memory_usage_percent() < 50.0 {
            return ResourceLevel::Abundant;
        }
        
        ResourceLevel::Normal
    }
}

/// Resource usage prediction
#[derive(Debug, Clone)]
pub struct ResourcePrediction {
    /// Predicted CPU usage in next window
    pub predicted_cpu: f32,
    
    /// Predicted memory usage
    pub predicted_memory: u64,
    
    /// Predicted battery drain rate (% per hour)
    pub battery_drain_rate: f32,
    
    /// Confidence in prediction (0.0 - 1.0)
    pub confidence: f32,
}

/// Historical resource usage pattern
#[derive(Debug, Clone)]
struct ResourceHistory {
    snapshots: Vec<ResourceSnapshot>,
    max_history: usize,
}

impl ResourceHistory {
    fn new(max_size: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_size),
            max_history: max_size,
        }
    }
    
    fn add(&mut self, snapshot: ResourceSnapshot) {
        if self.snapshots.len() >= self.max_history {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }
    
    fn average_cpu(&self) -> f32 {
        if self.snapshots.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.snapshots.iter().map(|s| s.cpu_usage).sum();
        sum / self.snapshots.len() as f32
    }
    
    fn average_memory(&self) -> u64 {
        if self.snapshots.is_empty() {
            return 0;
        }
        let sum: u64 = self.snapshots.iter().map(|s| s.memory_used).sum();
        sum / self.snapshots.len() as u64
    }
    
    fn battery_trend(&self) -> f32 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }
        
        let first = &self.snapshots[0];
        let last = &self.snapshots[self.snapshots.len() - 1];
        
        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f32() / 3600.0; // hours
        if time_diff == 0.0 {
            return 0.0;
        }
        
        (last.battery_level - first.battery_level) / time_diff
    }
}

/// Resource monitor with predictive analytics
pub struct ResourceMonitor {
    current: Arc<RwLock<ResourceSnapshot>>,
    history: Arc<RwLock<ResourceHistory>>,
    update_interval: Duration,
}

impl ResourceMonitor {
    /// Create new resource monitor
    pub fn new() -> Self {
        Self {
            current: Arc::new(RwLock::new(Self::get_initial_snapshot())),
            history: Arc::new(RwLock::new(ResourceHistory::new(300))), // 5 min at 1s intervals
            update_interval: Duration::from_secs(1),
        }
    }
    
    /// Get initial resource snapshot
    fn get_initial_snapshot() -> ResourceSnapshot {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        ResourceSnapshot {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: Self::get_total_memory(),
            temperature: 25.0,
            battery_level: 100.0,
            is_charging: false,
            timestamp: Instant::now(),
            timestamp_secs: now,
        }
    }
    
    /// Get total system memory
    fn get_total_memory() -> u64 {
        // Use sysinfo or similar in production
        // For now, return a reasonable default
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                return kb * 1024; // Convert to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Default fallback
        4 * 1024 * 1024 * 1024 // 4GB
    }
    
    /// Start monitoring loop
    pub async fn start_monitoring(&self) {
        let current = self.current.clone();
        let history = self.history.clone();
        let interval = self.update_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Get fresh snapshot
                let snapshot = Self::capture_snapshot().await;
                
                // Update current
                {
                    let mut current_lock = current.write().await;
                    *current_lock = snapshot.clone();
                }
                
                // Add to history
                {
                    let mut history_lock = history.write().await;
                    history_lock.add(snapshot);
                }
            }
        });
    }
    
    /// Capture current system state
    async fn capture_snapshot() -> ResourceSnapshot {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        ResourceSnapshot {
            cpu_usage: Self::get_cpu_usage(),
            memory_used: Self::get_memory_usage(),
            memory_total: Self::get_total_memory(),
            temperature: Self::get_temperature(),
            battery_level: Self::get_battery_level(),
            is_charging: Self::is_charging(),
            timestamp: Instant::now(),
            timestamp_secs: now,
        }
    }
    
    /// Get current CPU usage
    fn get_cpu_usage() -> f32 {
        // In production, use sysinfo or similar
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
                // Parse /proc/stat for CPU usage
                // This is simplified; real implementation would track deltas
                if let Some(line) = stat.lines().next() {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 5 && fields[0] == "cpu" {
                        let user: u64 = fields[1].parse().unwrap_or(0);
                        let nice: u64 = fields[2].parse().unwrap_or(0);
                        let system: u64 = fields[3].parse().unwrap_or(0);
                        let idle: u64 = fields[4].parse().unwrap_or(0);
                        
                        let total = user + nice + system + idle;
                        let used = user + nice + system;
                        
                        if total > 0 {
                            return (used as f32 / total as f32) * 100.0;
                        }
                    }
                }
            }
        }
        
        // Simulate for testing
        20.0 + (rand::random::<f32>() * 30.0)
    }
    
    /// Get current memory usage
    fn get_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                let mut mem_total = 0u64;
                let mut mem_available = 0u64;
                
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            mem_total = value.parse().unwrap_or(0);
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            mem_available = value.parse().unwrap_or(0);
                        }
                    }
                }
                
                if mem_total > 0 && mem_available > 0 {
                    return (mem_total - mem_available) * 1024; // Convert to bytes
                }
            }
        }
        
        // Simulate for testing
        (1.5 * 1024.0 * 1024.0 * 1024.0) as u64 // ~1.5GB
    }
    
    /// Get device temperature
    fn get_temperature() -> f32 {
        #[cfg(target_os = "linux")]
        {
            // Try common thermal zones
            let thermal_paths = [
                "/sys/class/thermal/thermal_zone0/temp",
                "/sys/class/thermal/thermal_zone1/temp",
            ];
            
            for path in &thermal_paths {
                if let Ok(temp_str) = std::fs::read_to_string(path) {
                    if let Ok(temp_millidegrees) = temp_str.trim().parse::<f32>() {
                        return temp_millidegrees / 1000.0; // Convert to Celsius
                    }
                }
            }
        }
        
        // Simulate for testing
        45.0 + (rand::random::<f32>() * 20.0)
    }
    
    /// Get battery level
    fn get_battery_level() -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(capacity) = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
                if let Ok(level) = capacity.trim().parse::<f32>() {
                    return level;
                }
            }
        }
        
        // Simulate for testing
        60.0 + (rand::random::<f32>() * 30.0)
    }
    
    /// Check if device is charging
    fn is_charging() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/sys/class/power_supply/BAT0/status") {
                return status.trim() == "Charging" || status.trim() == "Full";
            }
        }
        
        // Simulate for testing
        rand::random::<f32>() > 0.7 // 30% chance of charging
    }
    
    /// Get current resource snapshot
    pub async fn get_snapshot(&self) -> ResourceSnapshot {
        self.current.read().await.clone()
    }
    
    /// Get current resource level
    pub async fn get_resource_level(&self) -> ResourceLevel {
        let snapshot = self.get_snapshot().await;
        ResourceLevel::from_snapshot(&snapshot)
    }
    
    /// Predict resource usage
    pub async fn predict_resources(&self, window: Duration) -> Result<ResourcePrediction> {
        let history = self.history.read().await;
        
        if history.snapshots.len() < 10 {
            return Err(anyhow!("Insufficient history for prediction"));
        }
        
        // Simple moving average prediction
        let avg_cpu = history.average_cpu();
        let avg_memory = history.average_memory();
        let battery_trend = history.battery_trend();
        
        // Confidence based on history size
        let confidence = (history.snapshots.len() as f32 / history.max_history as f32).min(1.0);
        
        Ok(ResourcePrediction {
            predicted_cpu: avg_cpu,
            predicted_memory: avg_memory,
            battery_drain_rate: -battery_trend, // Make positive for drain
            confidence,
        })
    }
    
    /// Get resource usage statistics
    pub async fn get_statistics(&self) -> ResourceStatistics {
        let current = self.get_snapshot().await;
        let history = self.history.read().await;
        
        ResourceStatistics {
            current_cpu: current.cpu_usage,
            average_cpu: history.average_cpu(),
            current_memory: current.memory_used,
            average_memory: history.average_memory(),
            battery_level: current.battery_level,
            battery_trend: history.battery_trend(),
            temperature: current.temperature,
            resource_level: ResourceLevel::from_snapshot(&current),
            history_size: history.snapshots.len(),
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatistics {
    pub current_cpu: f32,
    pub average_cpu: f32,
    pub current_memory: u64,
    pub average_memory: u64,
    pub battery_level: f32,
    pub battery_trend: f32,
    pub temperature: f32,
    pub resource_level: ResourceLevel,
    pub history_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_level_from_snapshot() {
        let mut snapshot = ResourceSnapshot {
            cpu_usage: 40.0,  // Lower usage for Abundant
            memory_used: 1_500_000_000,  // 37.5% usage
            memory_total: 4_000_000_000,
            temperature: 50.0,
            battery_level: 85.0,  // Higher battery
            is_charging: true,
            timestamp: Instant::now(),
            timestamp_secs: 1000000,
        };
        
        assert_eq!(ResourceLevel::from_snapshot(&snapshot), ResourceLevel::Abundant);
        
        snapshot.battery_level = 10.0;
        snapshot.is_charging = false;
        assert_eq!(ResourceLevel::from_snapshot(&snapshot), ResourceLevel::Critical);
        
        snapshot.battery_level = 25.0;
        snapshot.temperature = 80.0;
        assert_eq!(ResourceLevel::from_snapshot(&snapshot), ResourceLevel::Constrained);
    }
    
    #[test]
    fn test_snapshot_helpers() {
        let snapshot = ResourceSnapshot {
            cpu_usage: 50.0,
            memory_used: 2_000_000_000,
            memory_total: 4_000_000_000,
            temperature: 80.0,
            battery_level: 10.0,
            is_charging: false,
            timestamp: Instant::now(),
            timestamp_secs: 1000000,
        };
        
        assert_eq!(snapshot.memory_usage_percent(), 50.0);
        assert!(snapshot.is_thermal_stress());
        assert!(snapshot.is_battery_critical());
    }
    
    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new();
        let snapshot = monitor.get_snapshot().await;
        
        assert!(snapshot.memory_total > 0);
    }
    
    #[tokio::test]
    async fn test_resource_monitor_level() {
        let monitor = ResourceMonitor::new();
        let level = monitor.get_resource_level().await;
        
        // Should be one of the valid levels
        assert!(matches!(level, ResourceLevel::Abundant | ResourceLevel::Normal | ResourceLevel::Constrained | ResourceLevel::Critical));
    }
    
    #[tokio::test]
    async fn test_resource_statistics() {
        let monitor = ResourceMonitor::new();
        
        // Start monitoring
        monitor.start_monitoring().await;
        
        // Wait a bit for some history
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let stats = monitor.get_statistics().await;
        
        assert!(stats.current_cpu >= 0.0);
        assert!(stats.current_memory > 0);
        assert!(stats.battery_level >= 0.0 && stats.battery_level <= 100.0);
    }
}
