//! Battery & Performance Optimization for Kāraṇa OS AR Glasses
//!
//! Comprehensive power and performance management for extended battery life.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub mod battery;
pub mod thermal;
pub mod performance;
pub mod optimization;

pub use battery::{BatteryInfo, BatteryState, BatteryManager};
pub use thermal::{ThermalState, ThermalZone, ThermalManager};
pub use performance::{PerformanceProfile, PerformanceManager};
pub use optimization::{OptimizationHint, OptimizationManager};

/// Power event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEventType {
    /// Battery level changed
    BatteryChanged,
    /// Charging started
    ChargingStarted,
    /// Charging stopped
    ChargingStopped,
    /// Low battery warning
    LowBattery,
    /// Critical battery
    CriticalBattery,
    /// Thermal throttling
    ThermalThrottle,
    /// Performance profile changed
    ProfileChanged,
    /// Power save mode
    PowerSave,
}

/// Power event
#[derive(Debug, Clone)]
pub struct PowerEvent {
    /// Event type
    pub event_type: PowerEventType,
    /// Timestamp
    pub timestamp: Instant,
    /// Details
    pub details: String,
    /// Battery level at event
    pub battery_level: u8,
}

impl PowerEvent {
    /// Create new event
    pub fn new(event_type: PowerEventType, battery_level: u8) -> Self {
        Self {
            event_type,
            timestamp: Instant::now(),
            details: String::new(),
            battery_level,
        }
    }
    
    /// With details
    pub fn with_details(mut self, details: String) -> Self {
        self.details = details;
        self
    }
}

/// Component power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    /// Active
    Active,
    /// Idle
    Idle,
    /// Low power
    LowPower,
    /// Off
    Off,
}

/// Hardware component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    /// Display
    Display,
    /// CPU
    CPU,
    /// GPU
    GPU,
    /// Camera
    Camera,
    /// WiFi
    WiFi,
    /// Bluetooth
    Bluetooth,
    /// GPS
    GPS,
    /// Sensors
    Sensors,
    /// Audio
    Audio,
    /// DepthSensor
    DepthSensor,
}

impl Component {
    /// All components
    pub fn all() -> Vec<Component> {
        vec![
            Self::Display, Self::CPU, Self::GPU, Self::Camera,
            Self::WiFi, Self::Bluetooth, Self::GPS, Self::Sensors,
            Self::Audio, Self::DepthSensor,
        ]
    }
    
    /// Typical power draw (mW)
    pub fn typical_power_mw(&self) -> u32 {
        match self {
            Self::Display => 400,
            Self::CPU => 800,
            Self::GPU => 600,
            Self::Camera => 300,
            Self::WiFi => 200,
            Self::Bluetooth => 50,
            Self::GPS => 100,
            Self::Sensors => 30,
            Self::Audio => 100,
            Self::DepthSensor => 250,
        }
    }
}

/// Component info
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component
    pub component: Component,
    /// Current state
    pub state: ComponentState,
    /// Current power draw (mW)
    pub power_mw: u32,
    /// Temperature (Celsius)
    pub temperature: Option<f32>,
    /// Last active
    pub last_active: Instant,
}

impl ComponentInfo {
    /// Create new
    pub fn new(component: Component) -> Self {
        Self {
            component,
            state: ComponentState::Off,
            power_mw: 0,
            temperature: None,
            last_active: Instant::now(),
        }
    }
    
    /// Set state
    pub fn set_state(&mut self, state: ComponentState) {
        self.state = state;
        self.power_mw = match state {
            ComponentState::Active => self.component.typical_power_mw(),
            ComponentState::Idle => self.component.typical_power_mw() / 4,
            ComponentState::LowPower => self.component.typical_power_mw() / 10,
            ComponentState::Off => 0,
        };
        if state == ComponentState::Active {
            self.last_active = Instant::now();
        }
    }
}

/// Power manager configuration
#[derive(Debug, Clone)]
pub struct PowerConfig {
    /// Low battery threshold (%)
    pub low_battery_threshold: u8,
    /// Critical battery threshold (%)
    pub critical_battery_threshold: u8,
    /// Auto power save threshold (%)
    pub auto_power_save_threshold: u8,
    /// Screen timeout (seconds)
    pub screen_timeout: u64,
    /// Idle timeout for components (seconds)
    pub idle_timeout: u64,
    /// Enable thermal management
    pub thermal_management: bool,
    /// Enable adaptive brightness
    pub adaptive_brightness: bool,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            low_battery_threshold: 20,
            critical_battery_threshold: 5,
            auto_power_save_threshold: 15,
            screen_timeout: 30,
            idle_timeout: 60,
            thermal_management: true,
            adaptive_brightness: true,
        }
    }
}

/// Main power manager
#[derive(Debug)]
pub struct PowerManager {
    /// Battery manager
    battery: BatteryManager,
    /// Thermal manager
    thermal: ThermalManager,
    /// Performance manager
    performance: PerformanceManager,
    /// Optimization manager
    optimization: OptimizationManager,
    /// Components
    components: HashMap<Component, ComponentInfo>,
    /// Configuration
    config: PowerConfig,
    /// Power save mode
    power_save_mode: bool,
    /// Events
    events: VecDeque<PowerEvent>,
    /// Max events
    max_events: usize,
    /// Total power draw estimation (mW)
    total_power_mw: u32,
}

impl PowerManager {
    /// Create new power manager
    pub fn new() -> Self {
        Self::with_config(PowerConfig::default())
    }
    
    /// Create with configuration
    pub fn with_config(config: PowerConfig) -> Self {
        let mut components = HashMap::new();
        for comp in Component::all() {
            components.insert(comp, ComponentInfo::new(comp));
        }
        
        Self {
            battery: BatteryManager::new(),
            thermal: ThermalManager::new(),
            performance: PerformanceManager::new(),
            optimization: OptimizationManager::new(),
            components,
            config,
            power_save_mode: false,
            events: VecDeque::new(),
            max_events: 100,
            total_power_mw: 0,
        }
    }
    
    /// Get battery manager
    pub fn battery(&self) -> &BatteryManager {
        &self.battery
    }
    
    /// Get mutable battery manager
    pub fn battery_mut(&mut self) -> &mut BatteryManager {
        &mut self.battery
    }
    
    /// Get thermal manager
    pub fn thermal(&self) -> &ThermalManager {
        &self.thermal
    }
    
    /// Get mutable thermal manager
    pub fn thermal_mut(&mut self) -> &mut ThermalManager {
        &mut self.thermal
    }
    
    /// Get performance manager
    pub fn performance(&self) -> &PerformanceManager {
        &self.performance
    }
    
    /// Get mutable performance manager
    pub fn performance_mut(&mut self) -> &mut PerformanceManager {
        &mut self.performance
    }
    
    /// Get optimization manager
    pub fn optimization(&self) -> &OptimizationManager {
        &self.optimization
    }
    
    /// Get mutable optimization manager
    pub fn optimization_mut(&mut self) -> &mut OptimizationManager {
        &mut self.optimization
    }
    
    /// Set component state
    pub fn set_component_state(&mut self, component: Component, state: ComponentState) {
        if let Some(info) = self.components.get_mut(&component) {
            info.set_state(state);
        }
        self.update_total_power();
    }
    
    /// Get component info
    pub fn get_component(&self, component: &Component) -> Option<&ComponentInfo> {
        self.components.get(component)
    }
    
    /// Update total power estimation
    fn update_total_power(&mut self) {
        self.total_power_mw = self.components.values()
            .map(|c| c.power_mw)
            .sum();
    }
    
    /// Get total power draw
    pub fn total_power_mw(&self) -> u32 {
        self.total_power_mw
    }
    
    /// Estimate remaining battery time
    pub fn estimate_remaining_time(&self) -> Option<Duration> {
        let level = self.battery.level();
        if self.total_power_mw == 0 {
            return None;
        }
        
        // Assume 3000mAh battery at 3.7V = 11.1Wh = 11100mWh
        let capacity_mwh = 11100.0;
        let remaining_mwh = capacity_mwh * (level as f64 / 100.0);
        let hours = remaining_mwh / self.total_power_mw as f64;
        
        Some(Duration::from_secs((hours * 3600.0) as u64))
    }
    
    /// Is power save mode
    pub fn is_power_save_mode(&self) -> bool {
        self.power_save_mode
    }
    
    /// Enable power save mode
    pub fn enable_power_save(&mut self) {
        if !self.power_save_mode {
            self.power_save_mode = true;
            self.performance.set_profile(PerformanceProfile::PowerSave);
            self.add_event(PowerEvent::new(
                PowerEventType::PowerSave,
                self.battery.level(),
            ).with_details("Power save enabled".to_string()));
        }
    }
    
    /// Disable power save mode
    pub fn disable_power_save(&mut self) {
        if self.power_save_mode {
            self.power_save_mode = false;
            self.performance.set_profile(PerformanceProfile::Balanced);
            self.add_event(PowerEvent::new(
                PowerEventType::PowerSave,
                self.battery.level(),
            ).with_details("Power save disabled".to_string()));
        }
    }
    
    /// Update (called periodically)
    pub fn update(&mut self) {
        // Update battery
        self.battery.update();
        
        // Check battery thresholds
        let level = self.battery.level();
        
        if level <= self.config.critical_battery_threshold {
            self.add_event(PowerEvent::new(
                PowerEventType::CriticalBattery,
                level,
            ));
        } else if level <= self.config.low_battery_threshold {
            self.add_event(PowerEvent::new(
                PowerEventType::LowBattery,
                level,
            ));
        }
        
        // Auto power save
        if level <= self.config.auto_power_save_threshold && !self.power_save_mode {
            self.enable_power_save();
        }
        
        // Update thermal
        if self.config.thermal_management {
            self.thermal.update();
            
            if self.thermal.is_throttling() {
                self.add_event(PowerEvent::new(
                    PowerEventType::ThermalThrottle,
                    level,
                ));
            }
        }
        
        // Check idle components
        let timeout = Duration::from_secs(self.config.idle_timeout);
        let now = Instant::now();
        
        for info in self.components.values_mut() {
            if info.state == ComponentState::Active &&
               now.duration_since(info.last_active) > timeout {
                info.set_state(ComponentState::Idle);
            }
        }
        
        self.update_total_power();
    }
    
    /// Add event
    fn add_event(&mut self, event: PowerEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_back();
        }
        self.events.push_front(event);
    }
    
    /// Get recent events
    pub fn events(&self) -> &VecDeque<PowerEvent> {
        &self.events
    }
    
    /// Get power summary
    pub fn summary(&self) -> PowerSummary {
        PowerSummary {
            battery_level: self.battery.level(),
            is_charging: self.battery.is_charging(),
            power_save_mode: self.power_save_mode,
            total_power_mw: self.total_power_mw,
            estimated_remaining: self.estimate_remaining_time(),
            thermal_state: self.thermal.state(),
            performance_profile: self.performance.profile(),
        }
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Power summary
#[derive(Debug, Clone)]
pub struct PowerSummary {
    /// Battery level
    pub battery_level: u8,
    /// Is charging
    pub is_charging: bool,
    /// Power save mode
    pub power_save_mode: bool,
    /// Total power draw (mW)
    pub total_power_mw: u32,
    /// Estimated remaining time
    pub estimated_remaining: Option<Duration>,
    /// Thermal state
    pub thermal_state: ThermalState,
    /// Performance profile
    pub performance_profile: PerformanceProfile,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_power() {
        assert!(Component::Display.typical_power_mw() > 0);
        assert!(Component::CPU.typical_power_mw() > Component::Bluetooth.typical_power_mw());
    }
    
    #[test]
    fn test_component_info() {
        let mut info = ComponentInfo::new(Component::Display);
        
        assert_eq!(info.state, ComponentState::Off);
        assert_eq!(info.power_mw, 0);
        
        info.set_state(ComponentState::Active);
        assert!(info.power_mw > 0);
    }
    
    #[test]
    fn test_power_manager_creation() {
        let manager = PowerManager::new();
        
        assert!(!manager.is_power_save_mode());
        assert!(manager.get_component(&Component::Display).is_some());
    }
    
    #[test]
    fn test_set_component_state() {
        let mut manager = PowerManager::new();
        
        manager.set_component_state(Component::Display, ComponentState::Active);
        
        let comp = manager.get_component(&Component::Display).unwrap();
        assert_eq!(comp.state, ComponentState::Active);
        assert!(manager.total_power_mw() > 0);
    }
    
    #[test]
    fn test_power_save_mode() {
        let mut manager = PowerManager::new();
        
        manager.enable_power_save();
        assert!(manager.is_power_save_mode());
        
        manager.disable_power_save();
        assert!(!manager.is_power_save_mode());
    }
    
    #[test]
    fn test_power_summary() {
        let manager = PowerManager::new();
        
        let summary = manager.summary();
        assert!(summary.battery_level <= 100);
    }
}
