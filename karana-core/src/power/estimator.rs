//! Power consumption estimation

use std::collections::HashMap;

/// Power consumer identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PowerConsumer {
    /// Display subsystem
    Display,
    /// Main processor
    Cpu,
    /// Graphics processor
    Gpu,
    /// Camera(s)
    Camera,
    /// Audio system
    Audio,
    /// Haptic feedback
    Haptics,
    /// WiFi radio
    Wifi,
    /// Bluetooth radio
    Bluetooth,
    /// GPS receiver
    Gps,
    /// IMU/sensors
    Imu,
    /// Eye tracking
    EyeTracker,
    /// Hand tracking
    HandTracker,
    /// SLAM/spatial mapping
    SpatialMapping,
    /// Voice processing
    VoiceProcessing,
    /// Custom component
    Custom(String),
}

impl PowerConsumer {
    /// Get base power consumption (mW)
    pub fn base_power(&self) -> f32 {
        match self {
            PowerConsumer::Display => 400.0,
            PowerConsumer::Cpu => 800.0,
            PowerConsumer::Gpu => 600.0,
            PowerConsumer::Camera => 300.0,
            PowerConsumer::Audio => 100.0,
            PowerConsumer::Haptics => 50.0,
            PowerConsumer::Wifi => 150.0,
            PowerConsumer::Bluetooth => 50.0,
            PowerConsumer::Gps => 100.0,
            PowerConsumer::Imu => 30.0,
            PowerConsumer::EyeTracker => 200.0,
            PowerConsumer::HandTracker => 250.0,
            PowerConsumer::SpatialMapping => 300.0,
            PowerConsumer::VoiceProcessing => 150.0,
            PowerConsumer::Custom(_) => 100.0,
        }
    }
    
    /// Get idle power consumption (mW)
    pub fn idle_power(&self) -> f32 {
        match self {
            PowerConsumer::Display => 50.0,
            PowerConsumer::Cpu => 100.0,
            PowerConsumer::Gpu => 50.0,
            PowerConsumer::Camera => 10.0,
            PowerConsumer::Audio => 10.0,
            PowerConsumer::Haptics => 0.0,
            PowerConsumer::Wifi => 30.0,
            PowerConsumer::Bluetooth => 10.0,
            PowerConsumer::Gps => 5.0,
            PowerConsumer::Imu => 10.0,
            PowerConsumer::EyeTracker => 20.0,
            PowerConsumer::HandTracker => 20.0,
            PowerConsumer::SpatialMapping => 30.0,
            PowerConsumer::VoiceProcessing => 20.0,
            PowerConsumer::Custom(_) => 10.0,
        }
    }
}

/// Power consumption estimate
#[derive(Debug, Clone)]
pub struct ConsumptionEstimate {
    /// Total estimated power (mW)
    pub total_power: f32,
    /// Per-component breakdown
    pub breakdown: HashMap<String, f32>,
    /// Estimated battery life (hours)
    pub estimated_hours: f32,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Power estimator
#[derive(Debug)]
pub struct PowerEstimator {
    /// Component power values
    component_power: HashMap<String, f32>,
    /// Component utilization
    utilization: HashMap<String, f32>,
    /// Historical measurements
    history: Vec<PowerMeasurement>,
    /// Battery capacity (mWh)
    battery_capacity: f32,
    /// Learning enabled
    learning_enabled: bool,
}

/// Historical power measurement
#[derive(Debug, Clone)]
struct PowerMeasurement {
    /// Timestamp (Unix seconds)
    timestamp: u64,
    /// Measured power (mW)
    power: f32,
    /// Active components
    components: Vec<String>,
}

impl PowerEstimator {
    /// Create new power estimator
    pub fn new() -> Self {
        Self {
            component_power: Self::default_component_power(),
            utilization: HashMap::new(),
            history: Vec::new(),
            battery_capacity: 5700.0, // 5.7 Wh typical
            learning_enabled: true,
        }
    }
    
    /// Default component power consumption
    fn default_component_power() -> HashMap<String, f32> {
        let mut power = HashMap::new();
        
        power.insert("display".to_string(), 400.0);
        power.insert("compute".to_string(), 1000.0); // CPU + GPU
        power.insert("camera".to_string(), 300.0);
        power.insert("audio".to_string(), 100.0);
        power.insert("haptics".to_string(), 50.0);
        power.insert("wifi".to_string(), 150.0);
        power.insert("bluetooth".to_string(), 50.0);
        power.insert("gps".to_string(), 100.0);
        power.insert("imu".to_string(), 30.0);
        power.insert("eye_tracker".to_string(), 200.0);
        
        power
    }
    
    /// Get component power
    pub fn get_component_power(&self, component: &str) -> f32 {
        self.component_power.get(component).copied().unwrap_or(50.0)
    }
    
    /// Set component power
    pub fn set_component_power(&mut self, component: &str, power: f32) {
        self.component_power.insert(component.to_string(), power);
    }
    
    /// Set component utilization
    pub fn set_utilization(&mut self, component: &str, utilization: f32) {
        self.utilization.insert(component.to_string(), utilization.clamp(0.0, 1.0));
    }
    
    /// Estimate total power consumption
    pub fn estimate(&self) -> ConsumptionEstimate {
        let mut total = 0.0;
        let mut breakdown = HashMap::new();
        
        for (component, base_power) in &self.component_power {
            let utilization = self.utilization.get(component).copied().unwrap_or(0.5);
            let power = base_power * utilization;
            breakdown.insert(component.clone(), power);
            total += power;
        }
        
        // Add base system power
        let system_power = 200.0;
        breakdown.insert("system".to_string(), system_power);
        total += system_power;
        
        let estimated_hours = if total > 0.0 {
            self.battery_capacity / total
        } else {
            24.0 // Assume 24h if no consumption
        };
        
        ConsumptionEstimate {
            total_power: total,
            breakdown,
            estimated_hours,
            confidence: self.calculate_confidence(),
        }
    }
    
    /// Calculate confidence in estimate
    fn calculate_confidence(&self) -> f32 {
        if self.history.is_empty() {
            return 0.5; // Default confidence without history
        }
        
        // More history = higher confidence
        let history_factor = (self.history.len() as f32 / 100.0).min(1.0);
        
        // More utilization data = higher confidence
        let utilization_factor = if self.utilization.is_empty() {
            0.5
        } else {
            0.8
        };
        
        (history_factor + utilization_factor) / 2.0
    }
    
    /// Record actual measurement for learning
    pub fn record_measurement(&mut self, power: f32, components: Vec<String>) {
        if !self.learning_enabled {
            return;
        }
        
        let measurement = PowerMeasurement {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            power,
            components,
        };
        
        self.history.push(measurement);
        
        // Keep only last 1000 measurements
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        
        // Update estimates based on measurements
        self.update_from_history();
    }
    
    /// Update component estimates from history
    fn update_from_history(&mut self) {
        if self.history.len() < 10 {
            return;
        }
        
        // Simple averaging for now
        // A more sophisticated approach would use regression
        
        // Group measurements by active components
        let mut component_totals: HashMap<String, (f32, u32)> = HashMap::new();
        
        for measurement in &self.history {
            let per_component = measurement.power / measurement.components.len().max(1) as f32;
            
            for component in &measurement.components {
                let entry = component_totals.entry(component.clone()).or_insert((0.0, 0));
                entry.0 += per_component;
                entry.1 += 1;
            }
        }
        
        // Update power estimates with weighted average
        for (component, (total, count)) in component_totals {
            if count > 5 {
                let measured_avg = total / count as f32;
                if let Some(current) = self.component_power.get_mut(&component) {
                    // Blend with existing estimate
                    *current = *current * 0.7 + measured_avg * 0.3;
                }
            }
        }
    }
    
    /// Estimate power for specific scenario
    pub fn estimate_scenario(&self, components: &[String], utilizations: &[f32]) -> f32 {
        let mut total = 200.0; // Base system power
        
        for (i, component) in components.iter().enumerate() {
            let base_power = self.component_power.get(component).copied().unwrap_or(50.0);
            let utilization = utilizations.get(i).copied().unwrap_or(0.5);
            total += base_power * utilization;
        }
        
        total
    }
    
    /// Get top power consumers
    pub fn top_consumers(&self, count: usize) -> Vec<(String, f32)> {
        let mut consumers: Vec<_> = self.component_power
            .iter()
            .map(|(name, &power)| {
                let utilization = self.utilization.get(name).copied().unwrap_or(0.5);
                (name.clone(), power * utilization)
            })
            .collect();
        
        consumers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        consumers.truncate(count);
        consumers
    }
    
    /// Set battery capacity
    pub fn set_battery_capacity(&mut self, capacity_mwh: f32) {
        self.battery_capacity = capacity_mwh;
    }
    
    /// Enable/disable learning
    pub fn set_learning(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
    }
}

impl Default for PowerEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_estimator_creation() {
        let estimator = PowerEstimator::new();
        assert!(estimator.get_component_power("display") > 0.0);
    }
    
    #[test]
    fn test_power_estimate() {
        let estimator = PowerEstimator::new();
        let estimate = estimator.estimate();
        
        assert!(estimate.total_power > 0.0);
        assert!(estimate.estimated_hours > 0.0);
        assert!(!estimate.breakdown.is_empty());
    }
    
    #[test]
    fn test_utilization() {
        let mut estimator = PowerEstimator::new();
        
        // Low utilization
        estimator.set_utilization("display", 0.1);
        estimator.set_utilization("compute", 0.1);
        let low_estimate = estimator.estimate();
        
        // High utilization
        estimator.set_utilization("display", 1.0);
        estimator.set_utilization("compute", 1.0);
        let high_estimate = estimator.estimate();
        
        assert!(high_estimate.total_power > low_estimate.total_power);
    }
    
    #[test]
    fn test_scenario_estimation() {
        let estimator = PowerEstimator::new();
        
        let light = estimator.estimate_scenario(
            &["display".to_string()],
            &[0.3]
        );
        
        let heavy = estimator.estimate_scenario(
            &["display".to_string(), "compute".to_string(), "camera".to_string()],
            &[1.0, 1.0, 1.0]
        );
        
        assert!(heavy > light);
    }
    
    #[test]
    fn test_top_consumers() {
        let estimator = PowerEstimator::new();
        let top = estimator.top_consumers(3);
        
        assert_eq!(top.len(), 3);
        
        // First should be highest power
        assert!(top[0].1 >= top[1].1);
        assert!(top[1].1 >= top[2].1);
    }
    
    #[test]
    fn test_consumer_power() {
        let display = PowerConsumer::Display;
        let haptics = PowerConsumer::Haptics;
        
        // Display should use more power than haptics
        assert!(display.base_power() > haptics.base_power());
    }
    
    #[test]
    fn test_record_measurement() {
        let mut estimator = PowerEstimator::new();
        
        estimator.record_measurement(1500.0, vec!["display".to_string(), "compute".to_string()]);
        
        // Should have one measurement
        assert_eq!(estimator.history.len(), 1);
    }
}
