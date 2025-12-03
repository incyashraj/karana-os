//! Power Management System for Kāraṇa OS AR Glasses
//! 
//! Manages battery, thermal throttling, and power optimization
//! for extended usage on mobile AR hardware.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod battery;
pub mod thermal;
pub mod profiles;
pub mod governor;
pub mod estimator;

pub use battery::{BatteryState, BatteryInfo, ChargingState};
pub use thermal::{ThermalZone, ThermalState, ThermalPolicy};
pub use profiles::{PowerProfile, ProfilePreset};
pub use governor::{PowerGovernor, GovernorMode};
pub use estimator::{PowerEstimator, PowerConsumer, ConsumptionEstimate};

/// Power management engine
#[derive(Debug)]
pub struct PowerManager {
    /// Current battery state
    battery: BatteryInfo,
    /// Thermal state
    thermal: ThermalState,
    /// Active power profile
    profile: PowerProfile,
    /// Power governor
    governor: PowerGovernor,
    /// Power estimator
    estimator: PowerEstimator,
    /// Component power states
    component_states: HashMap<String, ComponentPowerState>,
    /// Configuration
    config: PowerConfig,
    /// Last update time
    last_update: Instant,
    /// Statistics
    stats: PowerStats,
}

/// Power management configuration
#[derive(Debug, Clone)]
pub struct PowerConfig {
    /// Battery low threshold (percentage)
    pub low_battery_threshold: f32,
    /// Battery critical threshold (percentage)
    pub critical_battery_threshold: f32,
    /// Thermal throttle temperature (Celsius)
    pub thermal_throttle_temp: f32,
    /// Thermal shutdown temperature (Celsius)
    pub thermal_shutdown_temp: f32,
    /// Update interval
    pub update_interval: Duration,
    /// Enable adaptive power management
    pub adaptive_enabled: bool,
    /// Enable predictive battery estimation
    pub predictive_battery: bool,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            low_battery_threshold: 20.0,
            critical_battery_threshold: 5.0,
            thermal_throttle_temp: 45.0,
            thermal_shutdown_temp: 60.0,
            update_interval: Duration::from_secs(5),
            adaptive_enabled: true,
            predictive_battery: true,
        }
    }
}

/// Component power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPowerState {
    /// Fully active
    Active,
    /// Reduced power mode
    LowPower,
    /// Suspended/sleep mode
    Suspended,
    /// Completely off
    Off,
}

impl ComponentPowerState {
    /// Get power multiplier
    pub fn power_multiplier(&self) -> f32 {
        match self {
            ComponentPowerState::Active => 1.0,
            ComponentPowerState::LowPower => 0.5,
            ComponentPowerState::Suspended => 0.1,
            ComponentPowerState::Off => 0.0,
        }
    }
}

/// Power management statistics
#[derive(Debug, Default)]
pub struct PowerStats {
    /// Total energy consumed (mWh)
    pub total_energy_consumed: f32,
    /// Average power draw (mW)
    pub avg_power_draw: f32,
    /// Peak power draw (mW)
    pub peak_power_draw: f32,
    /// Time in each profile
    pub profile_time: HashMap<String, Duration>,
    /// Thermal throttle events
    pub throttle_events: u32,
    /// Low battery events
    pub low_battery_events: u32,
}

/// Power event types
#[derive(Debug, Clone)]
pub enum PowerEvent {
    /// Battery level changed
    BatteryLevelChanged(f32),
    /// Charging state changed
    ChargingStateChanged(ChargingState),
    /// Low battery warning
    LowBatteryWarning,
    /// Critical battery warning
    CriticalBatteryWarning,
    /// Thermal throttling started
    ThermalThrottle,
    /// Thermal warning
    ThermalWarning(f32),
    /// Profile changed
    ProfileChanged(String),
    /// Component state changed
    ComponentStateChanged(String, ComponentPowerState),
}

impl PowerManager {
    /// Create new power manager
    pub fn new() -> Self {
        Self::with_config(PowerConfig::default())
    }
    
    /// Create power manager with custom config
    pub fn with_config(config: PowerConfig) -> Self {
        Self {
            battery: BatteryInfo::new(),
            thermal: ThermalState::new(),
            profile: PowerProfile::default(),
            governor: PowerGovernor::new(),
            estimator: PowerEstimator::new(),
            component_states: Self::init_components(),
            config,
            last_update: Instant::now(),
            stats: PowerStats::default(),
        }
    }
    
    /// Initialize default component states
    fn init_components() -> HashMap<String, ComponentPowerState> {
        let mut components = HashMap::new();
        
        // Core components
        components.insert("display".to_string(), ComponentPowerState::Active);
        components.insert("camera".to_string(), ComponentPowerState::Suspended);
        components.insert("audio".to_string(), ComponentPowerState::Active);
        components.insert("haptics".to_string(), ComponentPowerState::Active);
        components.insert("wifi".to_string(), ComponentPowerState::Active);
        components.insert("bluetooth".to_string(), ComponentPowerState::Active);
        components.insert("gps".to_string(), ComponentPowerState::Suspended);
        components.insert("imu".to_string(), ComponentPowerState::Active);
        components.insert("eye_tracker".to_string(), ComponentPowerState::Active);
        components.insert("compute".to_string(), ComponentPowerState::Active);
        
        components
    }
    
    /// Update power state (call periodically)
    pub fn update(&mut self) -> Vec<PowerEvent> {
        if self.last_update.elapsed() < self.config.update_interval {
            return Vec::new();
        }
        self.last_update = Instant::now();
        
        let mut events = Vec::new();
        
        // Update battery
        let battery_events = self.update_battery();
        events.extend(battery_events);
        
        // Update thermal
        let thermal_events = self.update_thermal();
        events.extend(thermal_events);
        
        // Update power estimation
        self.update_estimation();
        
        // Apply governor decisions
        if self.config.adaptive_enabled {
            let gov_events = self.governor.apply_policy(
                &self.battery,
                &self.thermal,
                &mut self.profile,
                &mut self.component_states,
            );
            events.extend(gov_events);
        }
        
        events
    }
    
    /// Update battery state
    fn update_battery(&mut self) -> Vec<PowerEvent> {
        let mut events = Vec::new();
        
        // Simulate battery drain (in real implementation, read from hardware)
        let old_level = self.battery.level();
        self.battery.simulate_drain(self.config.update_interval, self.current_power_draw());
        let new_level = self.battery.level();
        
        if (old_level - new_level).abs() > 0.1 {
            events.push(PowerEvent::BatteryLevelChanged(new_level));
        }
        
        // Check thresholds
        if new_level <= self.config.critical_battery_threshold && old_level > self.config.critical_battery_threshold {
            events.push(PowerEvent::CriticalBatteryWarning);
            self.stats.low_battery_events += 1;
        } else if new_level <= self.config.low_battery_threshold && old_level > self.config.low_battery_threshold {
            events.push(PowerEvent::LowBatteryWarning);
            self.stats.low_battery_events += 1;
        }
        
        events
    }
    
    /// Update thermal state
    fn update_thermal(&mut self) -> Vec<PowerEvent> {
        let mut events = Vec::new();
        
        // Simulate thermal update (in real implementation, read from sensors)
        let temp = self.thermal.update(self.current_power_draw());
        
        if temp >= self.config.thermal_throttle_temp {
            if !self.thermal.is_throttling() {
                self.thermal.set_throttling(true);
                events.push(PowerEvent::ThermalThrottle);
                self.stats.throttle_events += 1;
            }
        } else if self.thermal.is_throttling() && temp < self.config.thermal_throttle_temp - 5.0 {
            self.thermal.set_throttling(false);
        }
        
        if temp >= self.config.thermal_throttle_temp - 5.0 {
            events.push(PowerEvent::ThermalWarning(temp));
        }
        
        events
    }
    
    /// Update power estimation
    fn update_estimation(&mut self) {
        let power = self.current_power_draw();
        
        // Update stats
        self.stats.avg_power_draw = (self.stats.avg_power_draw + power) / 2.0;
        if power > self.stats.peak_power_draw {
            self.stats.peak_power_draw = power;
        }
        
        let energy = power * self.config.update_interval.as_secs_f32() / 3600.0;
        self.stats.total_energy_consumed += energy;
    }
    
    /// Calculate current power draw
    pub fn current_power_draw(&self) -> f32 {
        let mut total = 0.0;
        
        for (component, state) in &self.component_states {
            let base_power = self.estimator.get_component_power(component);
            total += base_power * state.power_multiplier();
        }
        
        // Apply profile multiplier
        total *= self.profile.power_multiplier();
        
        // Apply thermal throttling
        if self.thermal.is_throttling() {
            total *= 0.7; // Reduce to 70% under throttling
        }
        
        total
    }
    
    /// Set component power state
    pub fn set_component_state(&mut self, component: &str, state: ComponentPowerState) {
        if let Some(current) = self.component_states.get_mut(component) {
            *current = state;
        }
    }
    
    /// Get component power state
    pub fn get_component_state(&self, component: &str) -> Option<ComponentPowerState> {
        self.component_states.get(component).copied()
    }
    
    /// Set power profile
    pub fn set_profile(&mut self, profile: PowerProfile) {
        self.profile = profile;
    }
    
    /// Set profile from preset
    pub fn set_profile_preset(&mut self, preset: ProfilePreset) {
        self.profile = PowerProfile::from_preset(preset);
    }
    
    /// Get current profile
    pub fn profile(&self) -> &PowerProfile {
        &self.profile
    }
    
    /// Get battery info
    pub fn battery(&self) -> &BatteryInfo {
        &self.battery
    }
    
    /// Get thermal state
    pub fn thermal(&self) -> &ThermalState {
        &self.thermal
    }
    
    /// Get estimated remaining time
    pub fn estimated_remaining_time(&self) -> Duration {
        if self.battery.charging_state() == ChargingState::Charging {
            return Duration::MAX;
        }
        
        let power_draw = self.current_power_draw();
        if power_draw <= 0.0 {
            return Duration::MAX;
        }
        
        let remaining_energy = self.battery.remaining_capacity();
        let hours = remaining_energy / power_draw;
        
        Duration::from_secs_f32(hours * 3600.0)
    }
    
    /// Get power statistics
    pub fn stats(&self) -> &PowerStats {
        &self.stats
    }
    
    /// Enable power saver mode
    pub fn enable_power_saver(&mut self) {
        self.set_profile_preset(ProfilePreset::PowerSaver);
        
        // Reduce non-essential components
        self.set_component_state("camera", ComponentPowerState::Off);
        self.set_component_state("gps", ComponentPowerState::Off);
        self.set_component_state("haptics", ComponentPowerState::LowPower);
    }
    
    /// Check if in critical battery state
    pub fn is_critical_battery(&self) -> bool {
        self.battery.level() <= self.config.critical_battery_threshold
    }
    
    /// Check if in low battery state
    pub fn is_low_battery(&self) -> bool {
        self.battery.level() <= self.config.low_battery_threshold
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_manager_creation() {
        let pm = PowerManager::new();
        assert!(pm.battery().level() > 0.0);
    }
    
    #[test]
    fn test_component_states() {
        let mut pm = PowerManager::new();
        
        assert_eq!(pm.get_component_state("display"), Some(ComponentPowerState::Active));
        
        pm.set_component_state("display", ComponentPowerState::LowPower);
        assert_eq!(pm.get_component_state("display"), Some(ComponentPowerState::LowPower));
    }
    
    #[test]
    fn test_power_draw_calculation() {
        let pm = PowerManager::new();
        let power = pm.current_power_draw();
        
        assert!(power > 0.0);
    }
    
    #[test]
    fn test_profile_setting() {
        let mut pm = PowerManager::new();
        
        pm.set_profile_preset(ProfilePreset::PowerSaver);
        assert!(pm.profile().power_multiplier() < 1.0);
        
        pm.set_profile_preset(ProfilePreset::Performance);
        assert!(pm.profile().power_multiplier() >= 1.0);
    }
    
    #[test]
    fn test_remaining_time_estimation() {
        let pm = PowerManager::new();
        let remaining = pm.estimated_remaining_time();
        
        // Should be finite and positive
        assert!(remaining < Duration::MAX);
        assert!(remaining > Duration::ZERO);
    }
    
    #[test]
    fn test_power_saver_mode() {
        let mut pm = PowerManager::new();
        let normal_power = pm.current_power_draw();
        
        pm.enable_power_saver();
        let saver_power = pm.current_power_draw();
        
        // Power saver should reduce power draw
        assert!(saver_power < normal_power);
    }
    
    #[test]
    fn test_battery_thresholds() {
        let config = PowerConfig {
            low_battery_threshold: 20.0,
            critical_battery_threshold: 5.0,
            ..Default::default()
        };
        
        let pm = PowerManager::with_config(config);
        
        // Fresh battery shouldn't be low or critical
        assert!(!pm.is_low_battery());
        assert!(!pm.is_critical_battery());
    }
}
