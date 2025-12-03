//! Power governor for automatic power management

use std::collections::HashMap;
use super::{
    BatteryInfo, ThermalState, PowerProfile, PowerEvent,
    ComponentPowerState, ChargingState,
    thermal::ThermalPolicy,
    profiles::ProfilePreset,
};

/// Governor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernorMode {
    /// Manual control only
    Manual,
    /// Automatic power management
    Automatic,
    /// Adaptive based on usage
    Adaptive,
    /// Emergency power saving
    Emergency,
}

/// Power governor for automatic decisions
#[derive(Debug)]
pub struct PowerGovernor {
    /// Governor mode
    mode: GovernorMode,
    /// Low battery actions taken
    low_battery_actions: bool,
    /// Critical battery actions taken
    critical_battery_actions: bool,
    /// Thermal actions taken
    thermal_actions: bool,
    /// Saved profile before emergency changes
    saved_profile: Option<PowerProfile>,
    /// Component priority for throttling
    component_priority: HashMap<String, u8>,
}

impl PowerGovernor {
    /// Create new governor
    pub fn new() -> Self {
        Self {
            mode: GovernorMode::Automatic,
            low_battery_actions: false,
            critical_battery_actions: false,
            thermal_actions: false,
            saved_profile: None,
            component_priority: Self::default_priorities(),
        }
    }
    
    /// Default component priorities (higher = more important)
    fn default_priorities() -> HashMap<String, u8> {
        let mut priorities = HashMap::new();
        
        // Critical (don't disable)
        priorities.insert("display".to_string(), 10);
        priorities.insert("compute".to_string(), 9);
        priorities.insert("imu".to_string(), 9);
        
        // Important (reduce but don't disable)
        priorities.insert("audio".to_string(), 8);
        priorities.insert("eye_tracker".to_string(), 7);
        priorities.insert("wifi".to_string(), 7);
        
        // Nice to have (can disable)
        priorities.insert("bluetooth".to_string(), 5);
        priorities.insert("haptics".to_string(), 5);
        priorities.insert("camera".to_string(), 4);
        
        // Non-essential (disable first)
        priorities.insert("gps".to_string(), 2);
        
        priorities
    }
    
    /// Apply power policy based on current state
    pub fn apply_policy(
        &mut self,
        battery: &BatteryInfo,
        thermal: &ThermalState,
        profile: &mut PowerProfile,
        components: &mut HashMap<String, ComponentPowerState>,
    ) -> Vec<PowerEvent> {
        if self.mode == GovernorMode::Manual {
            return Vec::new();
        }
        
        let mut events = Vec::new();
        
        // Handle thermal first (safety critical)
        events.extend(self.handle_thermal(thermal, profile, components));
        
        // Handle battery state
        events.extend(self.handle_battery(battery, profile, components));
        
        events
    }
    
    /// Handle thermal state
    fn handle_thermal(
        &mut self,
        thermal: &ThermalState,
        profile: &mut PowerProfile,
        components: &mut HashMap<String, ComponentPowerState>,
    ) -> Vec<PowerEvent> {
        let mut events = Vec::new();
        
        match thermal.policy() {
            ThermalPolicy::None => {
                if self.thermal_actions {
                    // Restore from thermal throttling
                    self.thermal_actions = false;
                    if let Some(saved) = self.saved_profile.take() {
                        *profile = saved;
                        events.push(PowerEvent::ProfileChanged("Restored".to_string()));
                    }
                }
            }
            ThermalPolicy::Light => {
                if !self.thermal_actions {
                    self.thermal_actions = true;
                    self.saved_profile = Some(profile.clone());
                }
                profile.adjust_for_thermal(0.85);
                self.reduce_low_priority_components(components, 3);
            }
            ThermalPolicy::Moderate => {
                if !self.thermal_actions {
                    self.thermal_actions = true;
                    self.saved_profile = Some(profile.clone());
                }
                profile.adjust_for_thermal(0.7);
                self.reduce_low_priority_components(components, 5);
            }
            ThermalPolicy::Aggressive => {
                if !self.thermal_actions {
                    self.thermal_actions = true;
                    self.saved_profile = Some(profile.clone());
                }
                profile.adjust_for_thermal(0.5);
                self.reduce_low_priority_components(components, 7);
            }
            ThermalPolicy::Emergency => {
                self.mode = GovernorMode::Emergency;
                profile.adjust_for_thermal(0.3);
                self.reduce_all_components(components);
                events.push(PowerEvent::ThermalThrottle);
            }
        }
        
        events
    }
    
    /// Handle battery state
    fn handle_battery(
        &mut self,
        battery: &BatteryInfo,
        profile: &mut PowerProfile,
        components: &mut HashMap<String, ComponentPowerState>,
    ) -> Vec<PowerEvent> {
        let mut events = Vec::new();
        let level = battery.level();
        
        // Skip if charging
        if battery.charging_state() == ChargingState::Charging {
            if self.low_battery_actions || self.critical_battery_actions {
                // Restore from power saving
                self.low_battery_actions = false;
                self.critical_battery_actions = false;
                if let Some(saved) = self.saved_profile.take() {
                    *profile = saved;
                    events.push(PowerEvent::ProfileChanged("Restored".to_string()));
                }
            }
            return events;
        }
        
        // Critical battery
        if level <= 5.0 && !self.critical_battery_actions {
            self.critical_battery_actions = true;
            if !self.low_battery_actions {
                self.saved_profile = Some(profile.clone());
            }
            
            // Apply ultra saver
            *profile = PowerProfile::from_preset(ProfilePreset::UltraSaver);
            self.reduce_all_components(components);
            
            events.push(PowerEvent::CriticalBatteryWarning);
            events.push(PowerEvent::ProfileChanged("Ultra Saver".to_string()));
        }
        // Low battery
        else if level <= 20.0 && !self.low_battery_actions {
            self.low_battery_actions = true;
            self.saved_profile = Some(profile.clone());
            
            // Apply power saver
            *profile = PowerProfile::from_preset(ProfilePreset::PowerSaver);
            self.reduce_low_priority_components(components, 4);
            
            events.push(PowerEvent::LowBatteryWarning);
            events.push(PowerEvent::ProfileChanged("Power Saver".to_string()));
        }
        // Recovered from low battery
        else if level > 25.0 && self.low_battery_actions && !self.critical_battery_actions {
            self.low_battery_actions = false;
            if let Some(saved) = self.saved_profile.take() {
                *profile = saved;
                events.push(PowerEvent::ProfileChanged("Restored".to_string()));
            }
        }
        
        events
    }
    
    /// Reduce low priority components
    fn reduce_low_priority_components(
        &self,
        components: &mut HashMap<String, ComponentPowerState>,
        threshold: u8,
    ) {
        for (name, state) in components.iter_mut() {
            let priority = self.component_priority.get(name).copied().unwrap_or(5);
            if priority < threshold {
                *state = ComponentPowerState::LowPower;
            }
        }
    }
    
    /// Reduce all non-critical components
    fn reduce_all_components(&self, components: &mut HashMap<String, ComponentPowerState>) {
        for (name, state) in components.iter_mut() {
            let priority = self.component_priority.get(name).copied().unwrap_or(5);
            if priority < 8 {
                *state = ComponentPowerState::Off;
            } else if priority < 10 {
                *state = ComponentPowerState::LowPower;
            }
        }
    }
    
    /// Set governor mode
    pub fn set_mode(&mut self, mode: GovernorMode) {
        self.mode = mode;
    }
    
    /// Get current mode
    pub fn mode(&self) -> GovernorMode {
        self.mode
    }
    
    /// Set component priority
    pub fn set_component_priority(&mut self, component: &str, priority: u8) {
        self.component_priority.insert(component.to_string(), priority);
    }
    
    /// Reset governor state
    pub fn reset(&mut self) {
        self.low_battery_actions = false;
        self.critical_battery_actions = false;
        self.thermal_actions = false;
        self.saved_profile = None;
        self.mode = GovernorMode::Automatic;
    }
}

impl Default for PowerGovernor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_governor_creation() {
        let gov = PowerGovernor::new();
        assert_eq!(gov.mode(), GovernorMode::Automatic);
    }
    
    #[test]
    fn test_governor_mode_setting() {
        let mut gov = PowerGovernor::new();
        
        gov.set_mode(GovernorMode::Manual);
        assert_eq!(gov.mode(), GovernorMode::Manual);
    }
    
    #[test]
    fn test_manual_mode_no_action() {
        let mut gov = PowerGovernor::new();
        gov.set_mode(GovernorMode::Manual);
        
        let battery = BatteryInfo::new();
        let thermal = ThermalState::new();
        let mut profile = PowerProfile::default();
        let mut components = HashMap::new();
        
        let events = gov.apply_policy(&battery, &thermal, &mut profile, &mut components);
        
        assert!(events.is_empty());
    }
    
    #[test]
    fn test_component_priorities() {
        let gov = PowerGovernor::new();
        
        // Display should have high priority
        assert!(gov.component_priority.get("display").copied().unwrap_or(0) > 
                gov.component_priority.get("gps").copied().unwrap_or(10));
    }
    
    #[test]
    fn test_reset() {
        let mut gov = PowerGovernor::new();
        gov.low_battery_actions = true;
        gov.mode = GovernorMode::Emergency;
        
        gov.reset();
        
        assert!(!gov.low_battery_actions);
        assert_eq!(gov.mode(), GovernorMode::Automatic);
    }
}
