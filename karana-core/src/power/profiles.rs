//! Power profiles and presets

use std::collections::HashMap;

/// Power profile preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilePreset {
    /// Maximum performance
    Performance,
    /// Balanced performance and battery
    Balanced,
    /// Power saving mode
    PowerSaver,
    /// Extended battery mode
    UltraSaver,
    /// Custom profile
    Custom,
}

/// Power profile configuration
#[derive(Debug, Clone)]
pub struct PowerProfile {
    /// Profile name
    pub name: String,
    /// Preset type
    pub preset: ProfilePreset,
    /// Display brightness (0.0 - 1.0)
    pub display_brightness: f32,
    /// Audio volume limit (0.0 - 1.0)
    pub audio_volume_limit: f32,
    /// Haptic intensity (0.0 - 1.0)
    pub haptic_intensity: f32,
    /// Camera quality level (0.0 - 1.0)
    pub camera_quality: f32,
    /// Network aggressiveness (0.0 - 1.0)
    pub network_aggressiveness: f32,
    /// CPU frequency limit (0.0 - 1.0)
    pub cpu_frequency: f32,
    /// GPU frequency limit (0.0 - 1.0)
    pub gpu_frequency: f32,
    /// Location update interval (seconds)
    pub location_interval: u32,
    /// Background sync enabled
    pub background_sync: bool,
    /// Voice assistant always listening
    pub voice_always_on: bool,
    /// Hand tracking enabled
    pub hand_tracking: bool,
    /// Eye tracking enabled
    pub eye_tracking: bool,
    /// Spatial mapping quality
    pub spatial_quality: f32,
}

impl PowerProfile {
    /// Create profile from preset
    pub fn from_preset(preset: ProfilePreset) -> Self {
        match preset {
            ProfilePreset::Performance => Self {
                name: "Performance".to_string(),
                preset,
                display_brightness: 1.0,
                audio_volume_limit: 1.0,
                haptic_intensity: 1.0,
                camera_quality: 1.0,
                network_aggressiveness: 1.0,
                cpu_frequency: 1.0,
                gpu_frequency: 1.0,
                location_interval: 5,
                background_sync: true,
                voice_always_on: true,
                hand_tracking: true,
                eye_tracking: true,
                spatial_quality: 1.0,
            },
            ProfilePreset::Balanced => Self {
                name: "Balanced".to_string(),
                preset,
                display_brightness: 0.7,
                audio_volume_limit: 0.8,
                haptic_intensity: 0.7,
                camera_quality: 0.8,
                network_aggressiveness: 0.7,
                cpu_frequency: 0.8,
                gpu_frequency: 0.8,
                location_interval: 30,
                background_sync: true,
                voice_always_on: true,
                hand_tracking: true,
                eye_tracking: true,
                spatial_quality: 0.8,
            },
            ProfilePreset::PowerSaver => Self {
                name: "Power Saver".to_string(),
                preset,
                display_brightness: 0.5,
                audio_volume_limit: 0.6,
                haptic_intensity: 0.4,
                camera_quality: 0.5,
                network_aggressiveness: 0.4,
                cpu_frequency: 0.6,
                gpu_frequency: 0.5,
                location_interval: 120,
                background_sync: false,
                voice_always_on: false,
                hand_tracking: true,
                eye_tracking: true,
                spatial_quality: 0.5,
            },
            ProfilePreset::UltraSaver => Self {
                name: "Ultra Saver".to_string(),
                preset,
                display_brightness: 0.3,
                audio_volume_limit: 0.4,
                haptic_intensity: 0.2,
                camera_quality: 0.3,
                network_aggressiveness: 0.2,
                cpu_frequency: 0.4,
                gpu_frequency: 0.3,
                location_interval: 300,
                background_sync: false,
                voice_always_on: false,
                hand_tracking: false,
                eye_tracking: false,
                spatial_quality: 0.3,
            },
            ProfilePreset::Custom => Self::default(),
        }
    }
    
    /// Get overall power multiplier for profile
    pub fn power_multiplier(&self) -> f32 {
        let factors = [
            self.display_brightness,
            self.cpu_frequency,
            self.gpu_frequency,
            self.camera_quality,
            self.network_aggressiveness,
        ];
        
        let avg: f32 = factors.iter().sum::<f32>() / factors.len() as f32;
        
        // Add bonus for disabled features
        let feature_savings = 
            if !self.background_sync { 0.05 } else { 0.0 } +
            if !self.voice_always_on { 0.08 } else { 0.0 } +
            if !self.hand_tracking { 0.1 } else { 0.0 } +
            if !self.eye_tracking { 0.1 } else { 0.0 };
        
        (avg - feature_savings).clamp(0.1, 1.0)
    }
    
    /// Estimate battery life multiplier
    pub fn battery_life_multiplier(&self) -> f32 {
        1.0 / self.power_multiplier()
    }
    
    /// Set display brightness
    pub fn set_brightness(&mut self, brightness: f32) {
        self.display_brightness = brightness.clamp(0.1, 1.0);
        self.preset = ProfilePreset::Custom;
    }
    
    /// Adjust for low battery
    pub fn adjust_for_low_battery(&mut self) {
        self.display_brightness *= 0.7;
        self.haptic_intensity *= 0.5;
        self.camera_quality *= 0.6;
        self.cpu_frequency *= 0.7;
        self.gpu_frequency *= 0.6;
        self.background_sync = false;
        self.preset = ProfilePreset::Custom;
    }
    
    /// Adjust for thermal throttling
    pub fn adjust_for_thermal(&mut self, throttle_factor: f32) {
        let factor = throttle_factor.clamp(0.3, 1.0);
        self.cpu_frequency *= factor;
        self.gpu_frequency *= factor;
        self.camera_quality *= factor;
        self.preset = ProfilePreset::Custom;
    }
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self::from_preset(ProfilePreset::Balanced)
    }
}

/// Profile scheduler for automatic switching
#[derive(Debug)]
pub struct ProfileScheduler {
    /// Scheduled profiles
    schedules: Vec<ScheduleEntry>,
    /// Active schedule index
    active_schedule: Option<usize>,
    /// Override profile (temporary)
    override_profile: Option<PowerProfile>,
}

/// Schedule entry
#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    /// Schedule name
    pub name: String,
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Days of week (0 = Sunday)
    pub days: Vec<u8>,
    /// Profile to use
    pub profile: ProfilePreset,
}

impl ProfileScheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        Self {
            schedules: Vec::new(),
            active_schedule: None,
            override_profile: None,
        }
    }
    
    /// Add schedule
    pub fn add_schedule(&mut self, entry: ScheduleEntry) {
        self.schedules.push(entry);
    }
    
    /// Remove schedule by name
    pub fn remove_schedule(&mut self, name: &str) {
        self.schedules.retain(|s| s.name != name);
    }
    
    /// Set temporary override
    pub fn set_override(&mut self, profile: Option<PowerProfile>) {
        self.override_profile = profile;
    }
    
    /// Get current profile based on time
    pub fn current_profile(&self, hour: u8, day_of_week: u8) -> ProfilePreset {
        // Override takes precedence
        if let Some(ref override_profile) = self.override_profile {
            return override_profile.preset;
        }
        
        // Find matching schedule
        for entry in &self.schedules {
            if entry.days.contains(&day_of_week) {
                let in_range = if entry.start_hour <= entry.end_hour {
                    hour >= entry.start_hour && hour < entry.end_hour
                } else {
                    // Overnight schedule
                    hour >= entry.start_hour || hour < entry.end_hour
                };
                
                if in_range {
                    return entry.profile;
                }
            }
        }
        
        // Default
        ProfilePreset::Balanced
    }
    
    /// List all schedules
    pub fn list_schedules(&self) -> &[ScheduleEntry] {
        &self.schedules
    }
}

impl Default for ProfileScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_creation() {
        let profile = PowerProfile::from_preset(ProfilePreset::Performance);
        assert_eq!(profile.display_brightness, 1.0);
        assert!(profile.voice_always_on);
    }
    
    #[test]
    fn test_power_multiplier() {
        let performance = PowerProfile::from_preset(ProfilePreset::Performance);
        let saver = PowerProfile::from_preset(ProfilePreset::PowerSaver);
        
        assert!(performance.power_multiplier() > saver.power_multiplier());
    }
    
    #[test]
    fn test_battery_life_multiplier() {
        let performance = PowerProfile::from_preset(ProfilePreset::Performance);
        let saver = PowerProfile::from_preset(ProfilePreset::PowerSaver);
        
        // Power saver should give longer battery life
        assert!(saver.battery_life_multiplier() > performance.battery_life_multiplier());
    }
    
    #[test]
    fn test_low_battery_adjustment() {
        let mut profile = PowerProfile::from_preset(ProfilePreset::Balanced);
        let original_brightness = profile.display_brightness;
        
        profile.adjust_for_low_battery();
        
        assert!(profile.display_brightness < original_brightness);
        assert!(!profile.background_sync);
    }
    
    #[test]
    fn test_thermal_adjustment() {
        let mut profile = PowerProfile::from_preset(ProfilePreset::Balanced);
        let original_cpu = profile.cpu_frequency;
        
        profile.adjust_for_thermal(0.7);
        
        assert!(profile.cpu_frequency < original_cpu);
    }
    
    #[test]
    fn test_scheduler() {
        let mut scheduler = ProfileScheduler::new();
        
        scheduler.add_schedule(ScheduleEntry {
            name: "Night Mode".to_string(),
            start_hour: 22,
            end_hour: 6,
            days: vec![0, 1, 2, 3, 4, 5, 6], // All days
            profile: ProfilePreset::PowerSaver,
        });
        
        // 23:00 should use power saver
        assert_eq!(scheduler.current_profile(23, 1), ProfilePreset::PowerSaver);
        
        // 14:00 should use default (Balanced)
        assert_eq!(scheduler.current_profile(14, 1), ProfilePreset::Balanced);
    }
    
    #[test]
    fn test_scheduler_override() {
        let mut scheduler = ProfileScheduler::new();
        scheduler.set_override(Some(PowerProfile::from_preset(ProfilePreset::Performance)));
        
        // Should always return override
        assert_eq!(scheduler.current_profile(0, 0), ProfilePreset::Performance);
        assert_eq!(scheduler.current_profile(12, 3), ProfilePreset::Performance);
    }
}
