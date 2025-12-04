//! Ambient Light Monitoring for Kāraṇa OS AR Glasses
//!
//! Monitors ambient lighting conditions for optimal AR display.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Lighting level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightingLevel {
    /// Very dark (< 10 lux)
    VeryDark,
    /// Dark (10-50 lux)
    Dark,
    /// Dim (50-200 lux)
    Dim,
    /// Normal indoor (200-500 lux)
    Normal,
    /// Bright indoor (500-1000 lux)
    Bright,
    /// Very bright / outdoor (> 1000 lux)
    VeryBright,
}

impl LightingLevel {
    /// Get from lux value
    pub fn from_lux(lux: f32) -> Self {
        match lux {
            x if x < 10.0 => Self::VeryDark,
            x if x < 50.0 => Self::Dark,
            x if x < 200.0 => Self::Dim,
            x if x < 500.0 => Self::Normal,
            x if x < 1000.0 => Self::Bright,
            _ => Self::VeryBright,
        }
    }
    
    /// Is comfortable for extended use
    pub fn is_comfortable(&self) -> bool {
        matches!(self, Self::Normal | Self::Bright)
    }
}

/// Lighting conditions
#[derive(Debug, Clone)]
pub struct LightingConditions {
    /// Ambient light level in lux
    pub ambient_lux: f32,
    /// Lighting level category
    pub level: LightingLevel,
    /// Is direct sunlight detected
    pub direct_sunlight: bool,
    /// Color temperature estimate (Kelvin)
    pub color_temperature: f32,
    /// Is artificial light detected
    pub artificial_light: bool,
    /// Time in current conditions
    pub duration: Duration,
}

impl LightingConditions {
    /// Create new lighting conditions
    pub fn new(lux: f32) -> Self {
        Self {
            ambient_lux: lux,
            level: LightingLevel::from_lux(lux),
            direct_sunlight: lux > 10000.0,
            color_temperature: 5500.0,  // Default daylight
            artificial_light: false,
            duration: Duration::ZERO,
        }
    }
    
    /// Recommended display brightness (0.0-1.0)
    pub fn recommended_brightness(&self) -> f32 {
        match self.level {
            LightingLevel::VeryDark => 0.1,
            LightingLevel::Dark => 0.2,
            LightingLevel::Dim => 0.4,
            LightingLevel::Normal => 0.6,
            LightingLevel::Bright => 0.8,
            LightingLevel::VeryBright => 1.0,
        }
    }
    
    /// Should blue light filter be enabled
    pub fn should_filter_blue_light(&self) -> bool {
        // Filter in dark conditions or warm artificial light
        self.level == LightingLevel::VeryDark 
            || self.level == LightingLevel::Dark
            || self.color_temperature < 4000.0
    }
}

/// Ambient alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbientAlert {
    /// Too dark
    TooDark,
    /// Too bright
    TooBright,
    /// Direct sunlight
    DirectSunlight,
    /// High contrast (dark room with bright display)
    HighContrast,
    /// Blue light concern
    BlueLightConcern,
}

impl AmbientAlert {
    /// Get message
    pub fn message(&self) -> &str {
        match self {
            Self::TooDark => "Low ambient light may cause eye strain",
            Self::TooBright => "Very bright environment - display may be hard to see",
            Self::DirectSunlight => "Direct sunlight detected - find shade",
            Self::HighContrast => "High contrast between display and environment",
            Self::BlueLightConcern => "Consider enabling blue light filter",
        }
    }
}

/// Ambient light monitor
#[derive(Debug)]
pub struct AmbientMonitor {
    /// Current conditions
    current: LightingConditions,
    /// Conditions history
    history: VecDeque<LightingConditions>,
    /// Active alerts
    alerts: Vec<AmbientAlert>,
    /// Last update
    last_update: Instant,
    /// Time in uncomfortable lighting
    uncomfortable_duration: Duration,
    /// Brightness adaptation enabled
    auto_brightness: bool,
    /// Blue light filter enabled
    blue_filter_enabled: bool,
    /// Maximum history entries
    max_history: usize,
}

impl AmbientMonitor {
    /// Create new ambient monitor
    pub fn new() -> Self {
        Self {
            current: LightingConditions::new(300.0),  // Normal indoor
            history: VecDeque::with_capacity(100),
            alerts: Vec::new(),
            last_update: Instant::now(),
            uncomfortable_duration: Duration::ZERO,
            auto_brightness: true,
            blue_filter_enabled: false,
            max_history: 100,
        }
    }
    
    /// Update with new sensor reading
    pub fn update(&mut self, lux: f32, delta: Duration) {
        let new_conditions = LightingConditions::new(lux);
        
        // Track duration in current level
        if new_conditions.level == self.current.level {
            self.current.duration += delta;
        } else {
            // Level changed - store in history
            if self.history.len() >= self.max_history {
                self.history.pop_front();
            }
            self.history.push_back(self.current.clone());
            self.current = new_conditions;
        }
        
        // Track uncomfortable duration
        if !self.current.level.is_comfortable() {
            self.uncomfortable_duration += delta;
        } else {
            self.uncomfortable_duration = Duration::ZERO;
        }
        
        // Check for alerts
        self.check_alerts();
        
        self.last_update = Instant::now();
    }
    
    /// Check and update alerts
    fn check_alerts(&mut self) {
        self.alerts.clear();
        
        // Too dark
        if matches!(self.current.level, LightingLevel::VeryDark | LightingLevel::Dark) {
            self.alerts.push(AmbientAlert::TooDark);
        }
        
        // Too bright
        if self.current.level == LightingLevel::VeryBright {
            self.alerts.push(AmbientAlert::TooBright);
        }
        
        // Direct sunlight
        if self.current.direct_sunlight {
            self.alerts.push(AmbientAlert::DirectSunlight);
        }
        
        // High contrast (dark room)
        if self.current.level == LightingLevel::VeryDark {
            self.alerts.push(AmbientAlert::HighContrast);
        }
        
        // Blue light in evening (would check time in real impl)
        if self.current.level == LightingLevel::Dark && !self.blue_filter_enabled {
            self.alerts.push(AmbientAlert::BlueLightConcern);
        }
    }
    
    /// Get current conditions
    pub fn current_conditions(&self) -> &LightingConditions {
        &self.current
    }
    
    /// Get current lux
    pub fn current_lux(&self) -> f32 {
        self.current.ambient_lux
    }
    
    /// Get current level
    pub fn current_level(&self) -> LightingLevel {
        self.current.level
    }
    
    /// Get active alerts
    pub fn alerts(&self) -> &[AmbientAlert] {
        &self.alerts
    }
    
    /// Has any alerts
    pub fn has_alerts(&self) -> bool {
        !self.alerts.is_empty()
    }
    
    /// Get recommended display brightness
    pub fn recommended_brightness(&self) -> f32 {
        self.current.recommended_brightness()
    }
    
    /// Should enable blue light filter
    pub fn should_enable_blue_filter(&self) -> bool {
        self.current.should_filter_blue_light()
    }
    
    /// Get uncomfortable duration
    pub fn uncomfortable_duration(&self) -> Duration {
        self.uncomfortable_duration
    }
    
    /// Enable/disable auto brightness
    pub fn set_auto_brightness(&mut self, enabled: bool) {
        self.auto_brightness = enabled;
    }
    
    /// Is auto brightness enabled
    pub fn auto_brightness_enabled(&self) -> bool {
        self.auto_brightness
    }
    
    /// Set blue filter state
    pub fn set_blue_filter(&mut self, enabled: bool) {
        self.blue_filter_enabled = enabled;
    }
    
    /// Is blue filter enabled
    pub fn blue_filter_enabled(&self) -> bool {
        self.blue_filter_enabled
    }
    
    /// Get average lux over history
    pub fn average_lux(&self) -> f32 {
        if self.history.is_empty() {
            return self.current.ambient_lux;
        }
        
        let sum: f32 = self.history.iter().map(|c| c.ambient_lux).sum();
        sum / self.history.len() as f32
    }
    
    /// Get history
    pub fn history(&self) -> &VecDeque<LightingConditions> {
        &self.history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for AmbientMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ambient_monitor_creation() {
        let monitor = AmbientMonitor::new();
        assert_eq!(monitor.current_level(), LightingLevel::Normal);
    }
    
    #[test]
    fn test_lighting_level_from_lux() {
        assert_eq!(LightingLevel::from_lux(5.0), LightingLevel::VeryDark);
        assert_eq!(LightingLevel::from_lux(30.0), LightingLevel::Dark);
        assert_eq!(LightingLevel::from_lux(100.0), LightingLevel::Dim);
        assert_eq!(LightingLevel::from_lux(300.0), LightingLevel::Normal);
        assert_eq!(LightingLevel::from_lux(700.0), LightingLevel::Bright);
        assert_eq!(LightingLevel::from_lux(5000.0), LightingLevel::VeryBright);
    }
    
    #[test]
    fn test_update_conditions() {
        let mut monitor = AmbientMonitor::new();
        monitor.update(30.0, Duration::from_secs(1));
        
        assert_eq!(monitor.current_level(), LightingLevel::Dark);
    }
    
    #[test]
    fn test_alerts_dark() {
        let mut monitor = AmbientMonitor::new();
        monitor.update(5.0, Duration::from_secs(1));
        
        assert!(monitor.has_alerts());
        assert!(monitor.alerts().contains(&AmbientAlert::TooDark));
    }
    
    #[test]
    fn test_alerts_bright() {
        let mut monitor = AmbientMonitor::new();
        monitor.update(15000.0, Duration::from_secs(1));
        
        assert!(monitor.has_alerts());
        assert!(monitor.alerts().contains(&AmbientAlert::DirectSunlight));
    }
    
    #[test]
    fn test_recommended_brightness() {
        let mut monitor = AmbientMonitor::new();
        
        monitor.update(5.0, Duration::from_secs(1));
        assert!(monitor.recommended_brightness() < 0.2);
        
        monitor.update(5000.0, Duration::from_secs(1));
        assert!(monitor.recommended_brightness() > 0.9);
    }
    
    #[test]
    fn test_blue_filter_recommendation() {
        let mut monitor = AmbientMonitor::new();
        
        monitor.update(5.0, Duration::from_secs(1));
        assert!(monitor.should_enable_blue_filter());
    }
    
    #[test]
    fn test_auto_brightness_toggle() {
        let mut monitor = AmbientMonitor::new();
        
        assert!(monitor.auto_brightness_enabled());
        monitor.set_auto_brightness(false);
        assert!(!monitor.auto_brightness_enabled());
    }
}
