//! Battery Information for Kāraṇa OS AR Glasses
//!
//! Track battery state, health, and charging.

use std::time::{Duration, Instant};

/// Battery state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    /// Discharging
    Discharging,
    /// Charging
    Charging,
    /// Full
    Full,
    /// Not charging (plugged but not charging)
    NotCharging,
    /// Unknown
    Unknown,
}

impl BatteryState {
    /// Is plugged in
    pub fn is_plugged(&self) -> bool {
        matches!(self, Self::Charging | Self::Full | Self::NotCharging)
    }
}

/// Battery health
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryHealth {
    /// Good
    Good,
    /// Overheat
    Overheat,
    /// Cold
    Cold,
    /// Dead
    Dead,
    /// Degraded
    Degraded,
    /// Unknown
    Unknown,
}

/// Charging type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargingType {
    /// Not charging
    None,
    /// Standard USB charging
    USB,
    /// Fast charging
    Fast,
    /// Wireless charging
    Wireless,
}

/// Battery info
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    /// Level (0-100)
    pub level: u8,
    /// State
    pub state: BatteryState,
    /// Health
    pub health: BatteryHealth,
    /// Temperature (Celsius)
    pub temperature: f32,
    /// Voltage (mV)
    pub voltage: u32,
    /// Current (mA, positive = charging)
    pub current: i32,
    /// Design capacity (mAh)
    pub design_capacity: u32,
    /// Current capacity (mAh)
    pub current_capacity: u32,
    /// Charging type
    pub charging_type: ChargingType,
    /// Cycle count
    pub cycle_count: u32,
    /// Last updated
    pub updated: Instant,
}

impl BatteryInfo {
    /// Create new battery info
    pub fn new() -> Self {
        Self {
            level: 100,
            state: BatteryState::Full,
            health: BatteryHealth::Good,
            temperature: 25.0,
            voltage: 4200,
            current: 0,
            design_capacity: 3000,
            current_capacity: 3000,
            charging_type: ChargingType::None,
            cycle_count: 0,
            updated: Instant::now(),
        }
    }
    
    /// Health percentage
    pub fn health_percent(&self) -> f32 {
        if self.design_capacity == 0 {
            return 0.0;
        }
        (self.current_capacity as f32 / self.design_capacity as f32) * 100.0
    }
    
    /// Is charging
    pub fn is_charging(&self) -> bool {
        self.state == BatteryState::Charging
    }
    
    /// Is low
    pub fn is_low(&self) -> bool {
        self.level <= 20
    }
    
    /// Is critical
    pub fn is_critical(&self) -> bool {
        self.level <= 5
    }
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Battery history entry
#[derive(Debug, Clone)]
pub struct BatteryHistoryEntry {
    /// Timestamp
    pub timestamp: Instant,
    /// Level
    pub level: u8,
    /// State
    pub state: BatteryState,
    /// Power draw (mW)
    pub power_mw: Option<u32>,
}

/// Battery manager
#[derive(Debug)]
pub struct BatteryManager {
    /// Current info
    info: BatteryInfo,
    /// History
    history: Vec<BatteryHistoryEntry>,
    /// Max history
    max_history: usize,
    /// History interval
    history_interval: Duration,
    /// Last history record
    last_history: Instant,
    /// Simulated drain rate (for testing)
    drain_rate: f32,
}

impl BatteryManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            info: BatteryInfo::new(),
            history: Vec::new(),
            max_history: 1000,
            history_interval: Duration::from_secs(60),
            last_history: Instant::now(),
            drain_rate: 0.1, // % per minute when active
        }
    }
    
    /// Get battery level
    pub fn level(&self) -> u8 {
        self.info.level
    }
    
    /// Get battery state
    pub fn state(&self) -> BatteryState {
        self.info.state
    }
    
    /// Is charging
    pub fn is_charging(&self) -> bool {
        self.info.is_charging()
    }
    
    /// Get battery info
    pub fn info(&self) -> &BatteryInfo {
        &self.info
    }
    
    /// Update battery (called periodically)
    pub fn update(&mut self) {
        self.info.updated = Instant::now();
        
        // Record history
        if self.last_history.elapsed() >= self.history_interval {
            self.record_history(None);
        }
    }
    
    /// Record history entry
    fn record_history(&mut self, power_mw: Option<u32>) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        
        self.history.push(BatteryHistoryEntry {
            timestamp: Instant::now(),
            level: self.info.level,
            state: self.info.state,
            power_mw,
        });
        
        self.last_history = Instant::now();
    }
    
    /// Get history
    pub fn history(&self) -> &[BatteryHistoryEntry] {
        &self.history
    }
    
    /// Set battery level (for simulation/testing)
    pub fn set_level(&mut self, level: u8) {
        self.info.level = level.min(100);
        
        if self.info.level == 100 && self.info.state.is_plugged() {
            self.info.state = BatteryState::Full;
        }
    }
    
    /// Set charging state
    pub fn set_charging(&mut self, charging_type: ChargingType) {
        self.info.charging_type = charging_type;
        
        match charging_type {
            ChargingType::None => {
                self.info.state = BatteryState::Discharging;
                self.info.current = -500; // Discharging
            }
            ChargingType::USB => {
                self.info.state = BatteryState::Charging;
                self.info.current = 500;
            }
            ChargingType::Fast => {
                self.info.state = BatteryState::Charging;
                self.info.current = 2000;
            }
            ChargingType::Wireless => {
                self.info.state = BatteryState::Charging;
                self.info.current = 1000;
            }
        }
    }
    
    /// Set temperature
    pub fn set_temperature(&mut self, temp: f32) {
        self.info.temperature = temp;
        
        // Update health based on temperature
        if temp > 45.0 {
            self.info.health = BatteryHealth::Overheat;
        } else if temp < 0.0 {
            self.info.health = BatteryHealth::Cold;
        } else {
            self.info.health = BatteryHealth::Good;
        }
    }
    
    /// Estimate time to full charge
    pub fn time_to_full(&self) -> Option<Duration> {
        if !self.info.is_charging() || self.info.level == 100 {
            return None;
        }
        
        if self.info.current <= 0 {
            return None;
        }
        
        let remaining_mah = self.info.current_capacity * (100 - self.info.level) as u32 / 100;
        let hours = remaining_mah as f32 / self.info.current as f32;
        
        Some(Duration::from_secs((hours * 3600.0) as u64))
    }
    
    /// Get average drain rate (% per hour) from history
    pub fn average_drain_rate(&self) -> Option<f32> {
        if self.history.len() < 2 {
            return None;
        }
        
        let first = self.history.first()?;
        let last = self.history.last()?;
        
        let level_diff = first.level as i32 - last.level as i32;
        let time_diff = last.timestamp.duration_since(first.timestamp);
        
        if time_diff.as_secs() == 0 {
            return None;
        }
        
        let hours = time_diff.as_secs_f32() / 3600.0;
        Some(level_diff as f32 / hours)
    }
    
    /// Simulate battery drain
    pub fn simulate_drain(&mut self, power_mw: u32, duration: Duration) {
        // Calculate drain based on power consumption
        // Assuming 3000mAh @ 3.7V = 11100mWh
        let capacity_mwh = 11100.0;
        let drain_mwh = power_mw as f64 * duration.as_secs_f64() / 3600.0;
        let drain_percent = (drain_mwh / capacity_mwh * 100.0) as u8;
        
        if drain_percent > 0 && !self.info.is_charging() {
            self.info.level = self.info.level.saturating_sub(drain_percent);
        }
    }
}

impl Default for BatteryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_state() {
        assert!(BatteryState::Charging.is_plugged());
        assert!(!BatteryState::Discharging.is_plugged());
    }
    
    #[test]
    fn test_battery_info() {
        let info = BatteryInfo::new();
        
        assert_eq!(info.level, 100);
        assert_eq!(info.health_percent(), 100.0);
    }
    
    #[test]
    fn test_battery_manager_creation() {
        let manager = BatteryManager::new();
        
        assert_eq!(manager.level(), 100);
        assert!(!manager.is_charging());
    }
    
    #[test]
    fn test_set_level() {
        let mut manager = BatteryManager::new();
        
        manager.set_level(50);
        assert_eq!(manager.level(), 50);
    }
    
    #[test]
    fn test_charging() {
        let mut manager = BatteryManager::new();
        
        manager.set_charging(ChargingType::Fast);
        assert!(manager.is_charging());
        
        manager.set_charging(ChargingType::None);
        assert!(!manager.is_charging());
    }
    
    #[test]
    fn test_temperature_health() {
        let mut manager = BatteryManager::new();
        
        manager.set_temperature(50.0);
        assert_eq!(manager.info().health, BatteryHealth::Overheat);
        
        manager.set_temperature(25.0);
        assert_eq!(manager.info().health, BatteryHealth::Good);
    }
    
    #[test]
    fn test_simulate_drain() {
        let mut manager = BatteryManager::new();
        manager.set_level(100);
        
        // High power consumption
        manager.simulate_drain(5000, Duration::from_secs(60));
        
        assert!(manager.level() < 100);
    }
}
