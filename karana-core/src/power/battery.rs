//! Battery state and monitoring

use std::time::{Duration, Instant};

/// Battery charging state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargingState {
    /// Not charging
    Discharging,
    /// Actively charging
    Charging,
    /// Fully charged
    Full,
    /// Not charging (e.g., suspended)
    NotCharging,
    /// Unknown state
    Unknown,
}

/// Battery health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryHealth {
    /// Battery is healthy
    Good,
    /// Battery is degraded
    Degraded,
    /// Battery needs replacement
    Poor,
    /// Battery status unknown
    Unknown,
}

impl BatteryHealth {
    /// Get capacity retention factor
    pub fn capacity_factor(&self) -> f32 {
        match self {
            BatteryHealth::Good => 1.0,
            BatteryHealth::Degraded => 0.85,
            BatteryHealth::Poor => 0.7,
            BatteryHealth::Unknown => 0.9,
        }
    }
}

/// Battery state information
#[derive(Debug, Clone)]
pub struct BatteryState {
    /// Current capacity (mAh)
    pub current_capacity: f32,
    /// Design capacity (mAh)
    pub design_capacity: f32,
    /// Full charge capacity (mAh)
    pub full_charge_capacity: f32,
    /// Current voltage (mV)
    pub voltage: f32,
    /// Current draw (mA, negative = charging)
    pub current: f32,
    /// Temperature (Celsius)
    pub temperature: f32,
    /// Cycle count
    pub cycle_count: u32,
}

impl Default for BatteryState {
    fn default() -> Self {
        Self {
            current_capacity: 1500.0, // 1500 mAh typical
            design_capacity: 1500.0,
            full_charge_capacity: 1500.0,
            voltage: 3800.0, // 3.8V typical LiPo
            current: 500.0, // 500mA draw
            temperature: 25.0,
            cycle_count: 0,
        }
    }
}

/// Battery information
#[derive(Debug)]
pub struct BatteryInfo {
    /// Current state
    state: BatteryState,
    /// Charging state
    charging_state: ChargingState,
    /// Battery health
    health: BatteryHealth,
    /// Last update time
    last_update: Instant,
    /// Charge rate (mA when charging)
    charge_rate: f32,
    /// Discharge history for estimation
    discharge_history: Vec<(Instant, f32)>,
}

impl BatteryInfo {
    /// Create new battery info with full charge
    pub fn new() -> Self {
        Self {
            state: BatteryState::default(),
            charging_state: ChargingState::Discharging,
            health: BatteryHealth::Good,
            last_update: Instant::now(),
            charge_rate: 1000.0, // 1A charge rate
            discharge_history: Vec::new(),
        }
    }
    
    /// Get battery level as percentage
    pub fn level(&self) -> f32 {
        (self.state.current_capacity / self.state.full_charge_capacity * 100.0)
            .clamp(0.0, 100.0)
    }
    
    /// Get remaining capacity in mWh
    pub fn remaining_capacity(&self) -> f32 {
        self.state.current_capacity * self.state.voltage / 1000.0 / 1000.0
    }
    
    /// Get charging state
    pub fn charging_state(&self) -> ChargingState {
        self.charging_state
    }
    
    /// Get battery health
    pub fn health(&self) -> BatteryHealth {
        self.health
    }
    
    /// Get temperature
    pub fn temperature(&self) -> f32 {
        self.state.temperature
    }
    
    /// Get voltage
    pub fn voltage(&self) -> f32 {
        self.state.voltage
    }
    
    /// Get current draw
    pub fn current(&self) -> f32 {
        self.state.current
    }
    
    /// Get cycle count
    pub fn cycle_count(&self) -> u32 {
        self.state.cycle_count
    }
    
    /// Simulate battery drain
    pub fn simulate_drain(&mut self, duration: Duration, power_mw: f32) {
        if self.charging_state == ChargingState::Charging {
            // Simulate charging
            let charge = self.charge_rate * duration.as_secs_f32() / 3600.0;
            self.state.current_capacity = (self.state.current_capacity + charge)
                .min(self.state.full_charge_capacity);
            
            if self.state.current_capacity >= self.state.full_charge_capacity {
                self.charging_state = ChargingState::Full;
            }
        } else if self.charging_state != ChargingState::Full {
            // Simulate discharging
            let current_ma = power_mw / (self.state.voltage / 1000.0);
            let drain = current_ma * duration.as_secs_f32() / 3600.0;
            self.state.current_capacity = (self.state.current_capacity - drain).max(0.0);
            self.state.current = current_ma;
            
            // Record discharge point
            self.discharge_history.push((Instant::now(), self.level()));
            
            // Keep only last 100 points
            if self.discharge_history.len() > 100 {
                self.discharge_history.remove(0);
            }
        }
        
        self.last_update = Instant::now();
    }
    
    /// Set charging state
    pub fn set_charging(&mut self, charging: bool) {
        if charging {
            self.charging_state = ChargingState::Charging;
        } else {
            self.charging_state = ChargingState::Discharging;
        }
    }
    
    /// Get estimated time to full charge
    pub fn time_to_full(&self) -> Option<Duration> {
        if self.charging_state != ChargingState::Charging {
            return None;
        }
        
        let remaining = self.state.full_charge_capacity - self.state.current_capacity;
        let hours = remaining / self.charge_rate;
        
        Some(Duration::from_secs_f32(hours * 3600.0))
    }
    
    /// Get average discharge rate from history
    pub fn average_discharge_rate(&self) -> Option<f32> {
        if self.discharge_history.len() < 2 {
            return None;
        }
        
        let first = &self.discharge_history[0];
        let last = &self.discharge_history[self.discharge_history.len() - 1];
        
        let level_diff = first.1 - last.1;
        let time_diff = last.0.duration_since(first.0).as_secs_f32() / 3600.0;
        
        if time_diff > 0.0 {
            Some(level_diff / time_diff) // % per hour
        } else {
            None
        }
    }
    
    /// Update battery health based on capacity
    pub fn update_health(&mut self) {
        let capacity_ratio = self.state.full_charge_capacity / self.state.design_capacity;
        
        self.health = if capacity_ratio > 0.9 {
            BatteryHealth::Good
        } else if capacity_ratio > 0.75 {
            BatteryHealth::Degraded
        } else {
            BatteryHealth::Poor
        };
    }
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_creation() {
        let battery = BatteryInfo::new();
        assert_eq!(battery.level(), 100.0);
        assert_eq!(battery.health(), BatteryHealth::Good);
    }
    
    #[test]
    fn test_battery_drain() {
        let mut battery = BatteryInfo::new();
        let initial = battery.level();
        
        battery.simulate_drain(Duration::from_secs(60), 1000.0); // 1W for 1 minute
        
        assert!(battery.level() < initial);
    }
    
    #[test]
    fn test_battery_charging() {
        let mut battery = BatteryInfo::new();
        battery.state.current_capacity = battery.state.full_charge_capacity * 0.5; // 50%
        battery.set_charging(true);
        
        let initial = battery.level();
        battery.simulate_drain(Duration::from_secs(60), 1000.0); // Power draw ignored when charging
        
        assert!(battery.level() > initial);
    }
    
    #[test]
    fn test_charging_state() {
        let mut battery = BatteryInfo::new();
        
        assert_eq!(battery.charging_state(), ChargingState::Discharging);
        
        battery.set_charging(true);
        assert_eq!(battery.charging_state(), ChargingState::Charging);
    }
    
    #[test]
    fn test_time_to_full() {
        let mut battery = BatteryInfo::new();
        battery.state.current_capacity = battery.state.full_charge_capacity * 0.5;
        
        // Not charging
        assert!(battery.time_to_full().is_none());
        
        // Charging
        battery.set_charging(true);
        let ttf = battery.time_to_full();
        assert!(ttf.is_some());
        assert!(ttf.unwrap() > Duration::ZERO);
    }
    
    #[test]
    fn test_health_update() {
        let mut battery = BatteryInfo::new();
        
        // Good health
        battery.update_health();
        assert_eq!(battery.health(), BatteryHealth::Good);
        
        // Degraded
        battery.state.full_charge_capacity = battery.state.design_capacity * 0.8;
        battery.update_health();
        assert_eq!(battery.health(), BatteryHealth::Degraded);
        
        // Poor
        battery.state.full_charge_capacity = battery.state.design_capacity * 0.6;
        battery.update_health();
        assert_eq!(battery.health(), BatteryHealth::Poor);
    }
}
