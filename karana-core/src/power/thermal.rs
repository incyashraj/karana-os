//! Thermal management and monitoring

use std::time::{Duration, Instant};

/// Thermal zone identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalZone {
    /// Main processor
    Processor,
    /// GPU/display processor
    Gpu,
    /// Display
    Display,
    /// Battery
    Battery,
    /// Camera module
    Camera,
    /// Wireless radios
    Wireless,
    /// Ambient/skin temperature
    Skin,
}

impl ThermalZone {
    /// Get thermal limit for zone
    pub fn thermal_limit(&self) -> f32 {
        match self {
            ThermalZone::Processor => 85.0,
            ThermalZone::Gpu => 85.0,
            ThermalZone::Display => 50.0,
            ThermalZone::Battery => 45.0,
            ThermalZone::Camera => 60.0,
            ThermalZone::Wireless => 70.0,
            ThermalZone::Skin => 43.0, // Skin contact limit
        }
    }
    
    /// Get throttle threshold for zone
    pub fn throttle_threshold(&self) -> f32 {
        self.thermal_limit() - 10.0
    }
}

/// Thermal policy for managing temperature
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalPolicy {
    /// No throttling
    None,
    /// Light throttling (reduce non-essential)
    Light,
    /// Moderate throttling (reduce all systems)
    Moderate,
    /// Aggressive throttling (minimum power)
    Aggressive,
    /// Emergency (preparing for shutdown)
    Emergency,
}

impl ThermalPolicy {
    /// Get power limit multiplier
    pub fn power_limit(&self) -> f32 {
        match self {
            ThermalPolicy::None => 1.0,
            ThermalPolicy::Light => 0.85,
            ThermalPolicy::Moderate => 0.7,
            ThermalPolicy::Aggressive => 0.5,
            ThermalPolicy::Emergency => 0.3,
        }
    }
    
    /// Get CPU/GPU frequency limit
    pub fn frequency_limit(&self) -> f32 {
        match self {
            ThermalPolicy::None => 1.0,
            ThermalPolicy::Light => 0.9,
            ThermalPolicy::Moderate => 0.75,
            ThermalPolicy::Aggressive => 0.5,
            ThermalPolicy::Emergency => 0.25,
        }
    }
}

/// Per-zone thermal reading
#[derive(Debug, Clone)]
pub struct ZoneReading {
    /// Current temperature (Celsius)
    pub temperature: f32,
    /// Temperature trend (degrees per minute)
    pub trend: f32,
    /// Active policy for this zone
    pub policy: ThermalPolicy,
    /// Last update time
    pub last_update: Instant,
}

impl Default for ZoneReading {
    fn default() -> Self {
        Self {
            temperature: 30.0,
            trend: 0.0,
            policy: ThermalPolicy::None,
            last_update: Instant::now(),
        }
    }
}

/// Overall thermal state
#[derive(Debug)]
pub struct ThermalState {
    /// Per-zone readings
    zones: std::collections::HashMap<ThermalZone, ZoneReading>,
    /// Overall thermal policy
    policy: ThermalPolicy,
    /// Is thermal throttling active
    throttling: bool,
    /// Ambient temperature
    ambient_temp: f32,
    /// Thermal headroom (degrees below limit)
    headroom: f32,
    /// Cooling available (e.g., fan)
    active_cooling: bool,
    /// History for trend calculation
    temp_history: Vec<(Instant, f32)>,
}

impl ThermalState {
    /// Create new thermal state
    pub fn new() -> Self {
        let mut zones = std::collections::HashMap::new();
        
        // Initialize all zones
        zones.insert(ThermalZone::Processor, ZoneReading::default());
        zones.insert(ThermalZone::Gpu, ZoneReading::default());
        zones.insert(ThermalZone::Display, ZoneReading::default());
        zones.insert(ThermalZone::Battery, ZoneReading::default());
        zones.insert(ThermalZone::Camera, ZoneReading::default());
        zones.insert(ThermalZone::Wireless, ZoneReading::default());
        zones.insert(ThermalZone::Skin, ZoneReading::default());
        
        Self {
            zones,
            policy: ThermalPolicy::None,
            throttling: false,
            ambient_temp: 25.0,
            headroom: 20.0,
            active_cooling: false,
            temp_history: Vec::new(),
        }
    }
    
    /// Update thermal state based on power draw
    pub fn update(&mut self, power_mw: f32) -> f32 {
        // Simulate temperature based on power draw
        // Simplified thermal model: higher power = higher temp
        let thermal_resistance = 0.02; // °C per mW
        let thermal_capacitance = 0.1; // Smoothing factor
        
        let steady_state_temp = self.ambient_temp + power_mw * thermal_resistance;
        
        // Get current average temperature
        let current_temp = self.average_temperature();
        
        // Move towards steady state
        let new_temp = current_temp + (steady_state_temp - current_temp) * thermal_capacitance;
        
        // Calculate policy first before mutating
        let processor_policy = self.calculate_zone_policy(ThermalZone::Processor, new_temp);
        
        // Update processor zone (primary zone)
        if let Some(reading) = self.zones.get_mut(&ThermalZone::Processor) {
            let old_temp = reading.temperature;
            reading.temperature = new_temp;
            reading.trend = (new_temp - old_temp) * 60.0; // per minute
            reading.last_update = Instant::now();
            reading.policy = processor_policy;
        }
        
        // Store ambient temp locally to avoid borrow issues
        let ambient = self.ambient_temp;
        
        // Update other zones (simplified)
        let zone_factors = [
            (ThermalZone::Gpu, 0.9),
            (ThermalZone::Display, 0.6),
            (ThermalZone::Battery, 0.5),
            (ThermalZone::Camera, 0.4),
            (ThermalZone::Wireless, 0.3),
            (ThermalZone::Skin, 0.5),
        ];
        
        for (zone, factor) in &zone_factors {
            if let Some(reading) = self.zones.get_mut(zone) {
                reading.temperature = ambient + (new_temp - ambient) * factor;
                reading.last_update = Instant::now();
            }
        }
        
        // Record history
        self.temp_history.push((Instant::now(), new_temp));
        if self.temp_history.len() > 60 {
            self.temp_history.remove(0);
        }
        
        // Update overall policy
        self.update_policy();
        
        // Calculate headroom
        let hottest = self.hottest_zone().map(|(_, r)| r.temperature).unwrap_or(30.0);
        self.headroom = 85.0 - hottest; // Assuming 85°C as general limit
        
        new_temp
    }
    
    /// Calculate policy for a zone
    fn calculate_zone_policy(&self, zone: ThermalZone, temp: f32) -> ThermalPolicy {
        let limit = zone.thermal_limit();
        let throttle = zone.throttle_threshold();
        
        if temp >= limit {
            ThermalPolicy::Emergency
        } else if temp >= throttle + 5.0 {
            ThermalPolicy::Aggressive
        } else if temp >= throttle {
            ThermalPolicy::Moderate
        } else if temp >= throttle - 5.0 {
            ThermalPolicy::Light
        } else {
            ThermalPolicy::None
        }
    }
    
    /// Update overall thermal policy
    fn update_policy(&mut self) {
        // Use most aggressive policy from any zone
        let worst_policy = self.zones.values()
            .map(|r| r.policy)
            .max_by_key(|p| *p as u8)
            .unwrap_or(ThermalPolicy::None);
        
        self.policy = worst_policy;
        self.throttling = worst_policy != ThermalPolicy::None;
    }
    
    /// Get average temperature across zones
    pub fn average_temperature(&self) -> f32 {
        if self.zones.is_empty() {
            return self.ambient_temp;
        }
        
        let sum: f32 = self.zones.values().map(|r| r.temperature).sum();
        sum / self.zones.len() as f32
    }
    
    /// Get hottest zone
    pub fn hottest_zone(&self) -> Option<(ThermalZone, &ZoneReading)> {
        self.zones.iter()
            .max_by(|a, b| a.1.temperature.partial_cmp(&b.1.temperature).unwrap())
            .map(|(z, r)| (*z, r))
    }
    
    /// Get zone reading
    pub fn get_zone(&self, zone: ThermalZone) -> Option<&ZoneReading> {
        self.zones.get(&zone)
    }
    
    /// Check if throttling
    pub fn is_throttling(&self) -> bool {
        self.throttling
    }
    
    /// Set throttling state
    pub fn set_throttling(&mut self, throttling: bool) {
        self.throttling = throttling;
    }
    
    /// Get current policy
    pub fn policy(&self) -> ThermalPolicy {
        self.policy
    }
    
    /// Get thermal headroom
    pub fn headroom(&self) -> f32 {
        self.headroom
    }
    
    /// Set ambient temperature
    pub fn set_ambient(&mut self, temp: f32) {
        self.ambient_temp = temp.clamp(-20.0, 50.0);
    }
    
    /// Get temperature trend (degrees per minute)
    pub fn temperature_trend(&self) -> f32 {
        if self.temp_history.len() < 2 {
            return 0.0;
        }
        
        let first = &self.temp_history[0];
        let last = &self.temp_history[self.temp_history.len() - 1];
        
        let temp_diff = last.1 - first.1;
        let time_diff = last.0.duration_since(first.0).as_secs_f32() / 60.0;
        
        if time_diff > 0.0 {
            temp_diff / time_diff
        } else {
            0.0
        }
    }
}

impl Default for ThermalState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thermal_state_creation() {
        let thermal = ThermalState::new();
        assert!(!thermal.is_throttling());
        assert_eq!(thermal.policy(), ThermalPolicy::None);
    }
    
    #[test]
    fn test_thermal_update() {
        let mut thermal = ThermalState::new();
        
        // Simulate some power draw
        let temp = thermal.update(1000.0); // 1W
        
        assert!(temp > thermal.ambient_temp);
    }
    
    #[test]
    fn test_thermal_zones() {
        let thermal = ThermalState::new();
        
        assert!(thermal.get_zone(ThermalZone::Processor).is_some());
        assert!(thermal.get_zone(ThermalZone::Battery).is_some());
    }
    
    #[test]
    fn test_thermal_policy_power_limits() {
        let none = ThermalPolicy::None.power_limit();
        let aggressive = ThermalPolicy::Aggressive.power_limit();
        
        assert!(none > aggressive);
    }
    
    #[test]
    fn test_zone_limits() {
        let processor = ThermalZone::Processor;
        let skin = ThermalZone::Skin;
        
        // Skin should have lower limit than processor
        assert!(skin.thermal_limit() < processor.thermal_limit());
    }
    
    #[test]
    fn test_hottest_zone() {
        let mut thermal = ThermalState::new();
        thermal.update(2000.0); // High power
        
        let hottest = thermal.hottest_zone();
        assert!(hottest.is_some());
    }
    
    #[test]
    fn test_thermal_headroom() {
        let mut thermal = ThermalState::new();
        
        // Update with low power to calculate real headroom
        thermal.update(500.0);
        
        // Should have headroom (default 30°C temp vs 85°C limit = 55°C headroom)
        assert!(thermal.headroom() > 0.0);
    }
}
