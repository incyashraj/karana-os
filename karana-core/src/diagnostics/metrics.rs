// Kāraṇa OS - Metrics Collector
// System metrics collection and history

use std::collections::VecDeque;
use std::time::Instant;

/// System metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory usage percentage (0-100)
    pub memory_usage: f32,
    /// Memory used in bytes
    pub memory_used_bytes: u64,
    /// Total memory in bytes
    pub memory_total_bytes: u64,
    /// Device temperature (Celsius)
    pub temperature: f32,
    /// Battery level percentage (0-100)
    pub battery_level: f32,
    /// Is battery charging
    pub battery_charging: bool,
    /// Current frame rate
    pub frame_rate: f32,
    /// Frame drop rate percentage
    pub frame_drop_rate: f32,
    /// Storage used in bytes
    pub storage_used: u64,
    /// Total storage in bytes
    pub storage_total: u64,
    /// Network connected
    pub network_connected: bool,
    /// Network latency in ms
    pub network_latency_ms: f32,
    /// Snapshot timestamp
    pub timestamp: Instant,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            temperature: 25.0,
            battery_level: 100.0,
            battery_charging: false,
            frame_rate: 60.0,
            frame_drop_rate: 0.0,
            storage_used: 0,
            storage_total: 0,
            network_connected: true,
            network_latency_ms: 0.0,
            timestamp: Instant::now(),
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    /// Metrics history
    history: VecDeque<MetricsSnapshot>,
    /// Max history size
    max_history: usize,
    /// Metric sources
    sources: MetricSources,
    /// Aggregated stats
    aggregated: AggregatedMetrics,
}

/// Sources for metric collection
struct MetricSources {
    /// CPU metrics provider
    cpu_provider: Box<dyn CpuMetricsProvider + Send + Sync>,
    /// Memory metrics provider
    memory_provider: Box<dyn MemoryMetricsProvider + Send + Sync>,
    /// Battery metrics provider
    battery_provider: Box<dyn BatteryMetricsProvider + Send + Sync>,
    /// Frame metrics provider
    frame_provider: Box<dyn FrameMetricsProvider + Send + Sync>,
}

/// Aggregated metrics over time
#[derive(Debug, Clone, Default)]
pub struct AggregatedMetrics {
    /// Average CPU usage
    pub avg_cpu: f32,
    /// Peak CPU usage
    pub peak_cpu: f32,
    /// Average memory usage
    pub avg_memory: f32,
    /// Peak memory usage
    pub peak_memory: f32,
    /// Average temperature
    pub avg_temperature: f32,
    /// Peak temperature
    pub peak_temperature: f32,
    /// Average frame rate
    pub avg_frame_rate: f32,
    /// Min frame rate
    pub min_frame_rate: f32,
    /// Total frame drops
    pub total_frame_drops: u64,
    /// Sample count
    pub sample_count: u64,
}

// Metric provider traits

pub trait CpuMetricsProvider: Send + Sync {
    fn get_usage(&self) -> f32;
    fn get_temperature(&self) -> f32;
}

pub trait MemoryMetricsProvider: Send + Sync {
    fn get_used(&self) -> u64;
    fn get_total(&self) -> u64;
    fn get_usage_percent(&self) -> f32;
}

pub trait BatteryMetricsProvider: Send + Sync {
    fn get_level(&self) -> f32;
    fn is_charging(&self) -> bool;
}

pub trait FrameMetricsProvider: Send + Sync {
    fn get_frame_rate(&self) -> f32;
    fn get_drop_rate(&self) -> f32;
}

// Default implementations

struct DefaultCpuProvider;
impl CpuMetricsProvider for DefaultCpuProvider {
    fn get_usage(&self) -> f32 {
        // In real implementation, read from /proc/stat or similar
        // For now, return simulated value
        15.0 + (rand_simple() * 10.0)
    }
    fn get_temperature(&self) -> f32 {
        35.0 + (rand_simple() * 5.0)
    }
}

struct DefaultMemoryProvider;
impl MemoryMetricsProvider for DefaultMemoryProvider {
    fn get_used(&self) -> u64 {
        // Simulated: 500MB
        500_000_000
    }
    fn get_total(&self) -> u64 {
        // Simulated: 2GB
        2_000_000_000
    }
    fn get_usage_percent(&self) -> f32 {
        (self.get_used() as f32 / self.get_total() as f32) * 100.0
    }
}

struct DefaultBatteryProvider;
impl BatteryMetricsProvider for DefaultBatteryProvider {
    fn get_level(&self) -> f32 {
        85.0
    }
    fn is_charging(&self) -> bool {
        false
    }
}

struct DefaultFrameProvider;
impl FrameMetricsProvider for DefaultFrameProvider {
    fn get_frame_rate(&self) -> f32 {
        59.0 + rand_simple()
    }
    fn get_drop_rate(&self) -> f32 {
        rand_simple() * 2.0
    }
}

fn rand_simple() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}

impl MetricsCollector {
    /// Create new collector
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            sources: MetricSources {
                cpu_provider: Box::new(DefaultCpuProvider),
                memory_provider: Box::new(DefaultMemoryProvider),
                battery_provider: Box::new(DefaultBatteryProvider),
                frame_provider: Box::new(DefaultFrameProvider),
            },
            aggregated: AggregatedMetrics::default(),
        }
    }

    /// Collect current metrics snapshot
    pub fn collect_snapshot(&mut self) -> MetricsSnapshot {
        let snapshot = MetricsSnapshot {
            cpu_usage: self.sources.cpu_provider.get_usage(),
            memory_usage: self.sources.memory_provider.get_usage_percent(),
            memory_used_bytes: self.sources.memory_provider.get_used(),
            memory_total_bytes: self.sources.memory_provider.get_total(),
            temperature: self.sources.cpu_provider.get_temperature(),
            battery_level: self.sources.battery_provider.get_level(),
            battery_charging: self.sources.battery_provider.is_charging(),
            frame_rate: self.sources.frame_provider.get_frame_rate(),
            frame_drop_rate: self.sources.frame_provider.get_drop_rate(),
            storage_used: 50_000_000_000, // 50GB simulated
            storage_total: 128_000_000_000, // 128GB simulated
            network_connected: true,
            network_latency_ms: 20.0 + rand_simple() * 30.0,
            timestamp: Instant::now(),
        };

        self.add_snapshot(snapshot.clone());
        snapshot
    }

    fn add_snapshot(&mut self, snapshot: MetricsSnapshot) {
        // Update aggregated metrics
        self.update_aggregated(&snapshot);

        // Add to history
        self.history.push_back(snapshot);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    fn update_aggregated(&mut self, snapshot: &MetricsSnapshot) {
        let n = self.aggregated.sample_count as f32;
        let new_n = n + 1.0;

        // Running averages
        self.aggregated.avg_cpu = (self.aggregated.avg_cpu * n + snapshot.cpu_usage) / new_n;
        self.aggregated.avg_memory = (self.aggregated.avg_memory * n + snapshot.memory_usage) / new_n;
        self.aggregated.avg_temperature = (self.aggregated.avg_temperature * n + snapshot.temperature) / new_n;
        self.aggregated.avg_frame_rate = (self.aggregated.avg_frame_rate * n + snapshot.frame_rate) / new_n;

        // Peaks
        self.aggregated.peak_cpu = self.aggregated.peak_cpu.max(snapshot.cpu_usage);
        self.aggregated.peak_memory = self.aggregated.peak_memory.max(snapshot.memory_usage);
        self.aggregated.peak_temperature = self.aggregated.peak_temperature.max(snapshot.temperature);

        // Mins
        if self.aggregated.sample_count == 0 {
            self.aggregated.min_frame_rate = snapshot.frame_rate;
        } else {
            self.aggregated.min_frame_rate = self.aggregated.min_frame_rate.min(snapshot.frame_rate);
        }

        // Totals
        if snapshot.frame_drop_rate > 1.0 {
            self.aggregated.total_frame_drops += 1;
        }

        self.aggregated.sample_count += 1;
    }

    /// Get metrics history
    pub fn history(&self) -> &[MetricsSnapshot] {
        // Convert VecDeque to slice via make_contiguous is not available with immutable ref
        // Return empty slice if no contiguous representation available
        self.history.as_slices().0
    }

    /// Get full history as vec
    pub fn history_vec(&self) -> Vec<MetricsSnapshot> {
        self.history.iter().cloned().collect()
    }

    /// Get aggregated metrics
    pub fn aggregated(&self) -> &AggregatedMetrics {
        &self.aggregated
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
        self.aggregated = AggregatedMetrics::default();
    }

    /// Get last N snapshots
    pub fn last_n(&self, n: usize) -> Vec<&MetricsSnapshot> {
        self.history.iter().rev().take(n).collect()
    }

    /// Register custom CPU provider
    pub fn set_cpu_provider<P: CpuMetricsProvider + 'static>(&mut self, provider: P) {
        self.sources.cpu_provider = Box::new(provider);
    }

    /// Register custom memory provider
    pub fn set_memory_provider<P: MemoryMetricsProvider + 'static>(&mut self, provider: P) {
        self.sources.memory_provider = Box::new(provider);
    }

    /// Register custom battery provider
    pub fn set_battery_provider<P: BatteryMetricsProvider + 'static>(&mut self, provider: P) {
        self.sources.battery_provider = Box::new(provider);
    }

    /// Register custom frame provider
    pub fn set_frame_provider<P: FrameMetricsProvider + 'static>(&mut self, provider: P) {
        self.sources.frame_provider = Box::new(provider);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(100);
        assert!(collector.history().is_empty());
    }

    #[test]
    fn test_collect_snapshot() {
        let mut collector = MetricsCollector::new(100);
        let snapshot = collector.collect_snapshot();
        
        assert!(snapshot.cpu_usage >= 0.0);
        assert!(snapshot.memory_usage >= 0.0);
        assert!(snapshot.battery_level >= 0.0);
    }

    #[test]
    fn test_history_limit() {
        let mut collector = MetricsCollector::new(5);
        
        for _ in 0..10 {
            collector.collect_snapshot();
        }
        
        assert!(collector.history_vec().len() <= 5);
    }

    #[test]
    fn test_aggregated_metrics() {
        let mut collector = MetricsCollector::new(100);
        
        for _ in 0..5 {
            collector.collect_snapshot();
        }
        
        let agg = collector.aggregated();
        assert_eq!(agg.sample_count, 5);
        assert!(agg.avg_cpu > 0.0);
    }

    #[test]
    fn test_metrics_snapshot_default() {
        let snapshot = MetricsSnapshot::default();
        assert_eq!(snapshot.battery_level, 100.0);
        assert_eq!(snapshot.frame_rate, 60.0);
    }

    #[test]
    fn test_clear_history() {
        let mut collector = MetricsCollector::new(100);
        collector.collect_snapshot();
        collector.collect_snapshot();
        
        assert!(!collector.history_vec().is_empty());
        
        collector.clear();
        assert!(collector.history_vec().is_empty());
        assert_eq!(collector.aggregated().sample_count, 0);
    }

    #[test]
    fn test_last_n() {
        let mut collector = MetricsCollector::new(100);
        
        for _ in 0..10 {
            collector.collect_snapshot();
        }
        
        let last_3 = collector.last_n(3);
        assert_eq!(last_3.len(), 3);
    }
}
