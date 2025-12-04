//! Thermal Management for Kāraṇa OS AR Glasses
//!
//! Monitor and manage device temperature.

use std::collections::HashMap;
use std::time::Instant;

/// Thermal state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    /// Normal temperature
    Normal,
    /// Warm (performance may be reduced)
    Warm,
    /// Hot (throttling active)
    Hot,
    /// Critical (emergency shutdown)
    Critical,
}

impl ThermalState {
    /// Get warning message
    pub fn warning(&self) -> Option<&str> {
        match self {
            Self::Normal => None,
            Self::Warm => Some("Device is warm"),
            Self::Hot => Some("Device is hot - performance reduced"),
            Self::Critical => Some("Device critically hot - shutting down"),
        }
    }
    
    /// Is throttling
    pub fn is_throttling(&self) -> bool {
        matches!(self, Self::Hot | Self::Critical)
    }
}

/// Thermal zone type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalZone {
    /// CPU
    CPU,
    /// GPU
    GPU,
    /// Battery
    Battery,
    /// Display
    Display,
    /// Ambient
    Ambient,
    /// Skin (user-facing surface)
    Skin,
}

impl ThermalZone {
    /// All zones
    pub fn all() -> Vec<ThermalZone> {
        vec![
            Self::CPU, Self::GPU, Self::Battery,
            Self::Display, Self::Ambient, Self::Skin,
        ]
    }
    
    /// Threshold temperatures (Normal, Warm, Hot, Critical)
    pub fn thresholds(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::CPU => (60.0, 75.0, 90.0, 100.0),
            Self::GPU => (60.0, 75.0, 90.0, 100.0),
            Self::Battery => (35.0, 40.0, 45.0, 55.0),
            Self::Display => (45.0, 55.0, 65.0, 75.0),
            Self::Ambient => (30.0, 35.0, 40.0, 45.0),
            Self::Skin => (35.0, 38.0, 42.0, 45.0),
        }
    }
    
    /// Get state from temperature
    pub fn state_from_temp(&self, temp: f32) -> ThermalState {
        let (normal, warm, hot, critical) = self.thresholds();
        
        if temp >= critical {
            ThermalState::Critical
        } else if temp >= hot {
            ThermalState::Hot
        } else if temp >= warm {
            ThermalState::Warm
        } else {
            ThermalState::Normal
        }
    }
}

/// Thermal zone info
#[derive(Debug, Clone)]
pub struct ThermalZoneInfo {
    /// Zone
    pub zone: ThermalZone,
    /// Current temperature (Celsius)
    pub temperature: f32,
    /// State
    pub state: ThermalState,
    /// Temperature trend (positive = heating)
    pub trend: f32,
    /// Last updated
    pub updated: Instant,
}

impl ThermalZoneInfo {
    /// Create new
    pub fn new(zone: ThermalZone) -> Self {
        Self {
            zone,
            temperature: 25.0,
            state: ThermalState::Normal,
            trend: 0.0,
            updated: Instant::now(),
        }
    }
    
    /// Update temperature
    pub fn update(&mut self, temp: f32) {
        self.trend = temp - self.temperature;
        self.temperature = temp;
        self.state = self.zone.state_from_temp(temp);
        self.updated = Instant::now();
    }
}

/// Cooling action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolingAction {
    /// No action
    None,
    /// Reduce CPU frequency
    ReduceCPU,
    /// Reduce GPU frequency
    ReduceGPU,
    /// Reduce display brightness
    ReduceBrightness,
    /// Disable background tasks
    DisableBackground,
    /// Emergency shutdown
    Shutdown,
}

/// Thermal manager
#[derive(Debug)]
pub struct ThermalManager {
    /// Zones
    zones: HashMap<ThermalZone, ThermalZoneInfo>,
    /// Overall state
    overall_state: ThermalState,
    /// Active cooling actions
    cooling_actions: Vec<CoolingAction>,
    /// Throttle level (0-100)
    throttle_level: u8,
    /// Temperature history sampling
    history: Vec<(Instant, f32)>,
    /// Max history entries
    max_history: usize,
}

impl ThermalManager {
    /// Create new manager
    pub fn new() -> Self {
        let mut zones = HashMap::new();
        for zone in ThermalZone::all() {
            zones.insert(zone, ThermalZoneInfo::new(zone));
        }
        
        Self {
            zones,
            overall_state: ThermalState::Normal,
            cooling_actions: Vec::new(),
            throttle_level: 0,
            history: Vec::new(),
            max_history: 100,
        }
    }
    
    /// Get overall thermal state
    pub fn state(&self) -> ThermalState {
        self.overall_state
    }
    
    /// Is throttling
    pub fn is_throttling(&self) -> bool {
        self.overall_state.is_throttling()
    }
    
    /// Get throttle level
    pub fn throttle_level(&self) -> u8 {
        self.throttle_level
    }
    
    /// Get zone info
    pub fn zone(&self, zone: &ThermalZone) -> Option<&ThermalZoneInfo> {
        self.zones.get(zone)
    }
    
    /// Get all zones
    pub fn all_zones(&self) -> Vec<&ThermalZoneInfo> {
        self.zones.values().collect()
    }
    
    /// Update zone temperature
    pub fn update_zone(&mut self, zone: ThermalZone, temp: f32) {
        if let Some(info) = self.zones.get_mut(&zone) {
            info.update(temp);
        }
        
        self.update_overall_state();
    }
    
    /// Update all zones (simulated)
    pub fn update(&mut self) {
        self.update_overall_state();
    }
    
    /// Update overall state
    fn update_overall_state(&mut self) {
        // Find worst state
        self.overall_state = self.zones.values()
            .map(|z| z.state)
            .max_by_key(|s| match s {
                ThermalState::Normal => 0,
                ThermalState::Warm => 1,
                ThermalState::Hot => 2,
                ThermalState::Critical => 3,
            })
            .unwrap_or(ThermalState::Normal);
        
        // Update throttle level
        self.throttle_level = match self.overall_state {
            ThermalState::Normal => 0,
            ThermalState::Warm => 25,
            ThermalState::Hot => 50,
            ThermalState::Critical => 100,
        };
        
        // Determine cooling actions
        self.cooling_actions = self.determine_cooling_actions();
        
        // Record average temperature
        let avg_temp = self.average_temperature();
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push((Instant::now(), avg_temp));
    }
    
    /// Determine cooling actions
    fn determine_cooling_actions(&self) -> Vec<CoolingAction> {
        let mut actions = Vec::new();
        
        match self.overall_state {
            ThermalState::Normal => {}
            ThermalState::Warm => {
                actions.push(CoolingAction::DisableBackground);
            }
            ThermalState::Hot => {
                actions.push(CoolingAction::DisableBackground);
                actions.push(CoolingAction::ReduceCPU);
                actions.push(CoolingAction::ReduceGPU);
                actions.push(CoolingAction::ReduceBrightness);
            }
            ThermalState::Critical => {
                actions.push(CoolingAction::Shutdown);
            }
        }
        
        actions
    }
    
    /// Get active cooling actions
    pub fn cooling_actions(&self) -> &[CoolingAction] {
        &self.cooling_actions
    }
    
    /// Get average temperature across all zones
    pub fn average_temperature(&self) -> f32 {
        if self.zones.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.zones.values().map(|z| z.temperature).sum();
        sum / self.zones.len() as f32
    }
    
    /// Get hottest zone
    pub fn hottest_zone(&self) -> Option<&ThermalZoneInfo> {
        self.zones.values()
            .max_by(|a, b| a.temperature.partial_cmp(&b.temperature).unwrap())
    }
    
    /// Get temperature trend
    pub fn temperature_trend(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }
        
        let recent = &self.history[self.history.len().saturating_sub(5)..];
        if recent.len() < 2 {
            return 0.0;
        }
        
        let first_temp = recent.first().map(|(_, t)| *t).unwrap_or(0.0);
        let last_temp = recent.last().map(|(_, t)| *t).unwrap_or(0.0);
        
        last_temp - first_temp
    }
    
    /// Set zone temperature (for simulation)
    pub fn set_temperature(&mut self, zone: ThermalZone, temp: f32) {
        self.update_zone(zone, temp);
    }
}

impl Default for ThermalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thermal_state() {
        assert!(!ThermalState::Normal.is_throttling());
        assert!(ThermalState::Hot.is_throttling());
        assert!(ThermalState::Critical.is_throttling());
    }
    
    #[test]
    fn test_zone_thresholds() {
        let zone = ThermalZone::CPU;
        
        assert_eq!(zone.state_from_temp(50.0), ThermalState::Normal);
        assert_eq!(zone.state_from_temp(80.0), ThermalState::Warm);
        assert_eq!(zone.state_from_temp(95.0), ThermalState::Hot);
        assert_eq!(zone.state_from_temp(105.0), ThermalState::Critical);
    }
    
    #[test]
    fn test_thermal_manager_creation() {
        let manager = ThermalManager::new();
        
        assert_eq!(manager.state(), ThermalState::Normal);
        assert_eq!(manager.throttle_level(), 0);
    }
    
    #[test]
    fn test_update_zone() {
        let mut manager = ThermalManager::new();
        
        manager.update_zone(ThermalZone::CPU, 95.0);
        
        assert_eq!(manager.state(), ThermalState::Hot);
        assert!(manager.is_throttling());
    }
    
    #[test]
    fn test_cooling_actions() {
        let mut manager = ThermalManager::new();
        
        manager.update_zone(ThermalZone::CPU, 95.0);
        
        let actions = manager.cooling_actions();
        assert!(!actions.is_empty());
        assert!(actions.contains(&CoolingAction::ReduceCPU));
    }
    
    #[test]
    fn test_hottest_zone() {
        let mut manager = ThermalManager::new();
        
        manager.update_zone(ThermalZone::CPU, 80.0);
        manager.update_zone(ThermalZone::GPU, 60.0);
        
        let hottest = manager.hottest_zone().unwrap();
        assert_eq!(hottest.zone, ThermalZone::CPU);
    }
}
