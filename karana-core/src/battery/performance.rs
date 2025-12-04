//! Performance Profiles for Kāraṇa OS AR Glasses
//!
//! Dynamic performance scaling based on workload and power state.

use std::time::Instant;

/// Performance profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceProfile {
    /// Maximum performance
    Performance,
    /// Balanced performance and power
    Balanced,
    /// Power saving
    PowerSave,
    /// Ultra power save
    UltraPowerSave,
    /// Custom
    Custom,
}

impl PerformanceProfile {
    /// Get CPU scaling factor (0.0-1.0)
    pub fn cpu_scale(&self) -> f32 {
        match self {
            Self::Performance => 1.0,
            Self::Balanced => 0.8,
            Self::PowerSave => 0.5,
            Self::UltraPowerSave => 0.3,
            Self::Custom => 0.7,
        }
    }
    
    /// Get GPU scaling factor (0.0-1.0)
    pub fn gpu_scale(&self) -> f32 {
        match self {
            Self::Performance => 1.0,
            Self::Balanced => 0.8,
            Self::PowerSave => 0.4,
            Self::UltraPowerSave => 0.2,
            Self::Custom => 0.6,
        }
    }
    
    /// Get display refresh rate
    pub fn refresh_rate(&self) -> u32 {
        match self {
            Self::Performance => 120,
            Self::Balanced => 90,
            Self::PowerSave => 60,
            Self::UltraPowerSave => 30,
            Self::Custom => 72,
        }
    }
    
    /// Get render resolution scale
    pub fn resolution_scale(&self) -> f32 {
        match self {
            Self::Performance => 1.0,
            Self::Balanced => 0.9,
            Self::PowerSave => 0.7,
            Self::UltraPowerSave => 0.5,
            Self::Custom => 0.8,
        }
    }
    
    /// Allow background tasks
    pub fn allow_background(&self) -> bool {
        match self {
            Self::Performance => true,
            Self::Balanced => true,
            Self::PowerSave => false,
            Self::UltraPowerSave => false,
            Self::Custom => true,
        }
    }
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Performance settings
#[derive(Debug, Clone)]
pub struct PerformanceSettings {
    /// CPU frequency scale (0.0-1.0)
    pub cpu_scale: f32,
    /// GPU frequency scale (0.0-1.0)
    pub gpu_scale: f32,
    /// Display refresh rate
    pub refresh_rate: u32,
    /// Render resolution scale
    pub resolution_scale: f32,
    /// Background tasks allowed
    pub background_allowed: bool,
    /// Animation quality (0-2)
    pub animation_quality: u8,
    /// AR tracking quality (0-2)
    pub tracking_quality: u8,
}

impl PerformanceSettings {
    /// From profile
    pub fn from_profile(profile: PerformanceProfile) -> Self {
        Self {
            cpu_scale: profile.cpu_scale(),
            gpu_scale: profile.gpu_scale(),
            refresh_rate: profile.refresh_rate(),
            resolution_scale: profile.resolution_scale(),
            background_allowed: profile.allow_background(),
            animation_quality: match profile {
                PerformanceProfile::Performance => 2,
                PerformanceProfile::Balanced => 1,
                _ => 0,
            },
            tracking_quality: match profile {
                PerformanceProfile::Performance => 2,
                PerformanceProfile::Balanced => 1,
                PerformanceProfile::PowerSave => 1,
                _ => 0,
            },
        }
    }
    
    /// High quality
    pub fn high_quality() -> Self {
        Self {
            cpu_scale: 1.0,
            gpu_scale: 1.0,
            refresh_rate: 120,
            resolution_scale: 1.0,
            background_allowed: true,
            animation_quality: 2,
            tracking_quality: 2,
        }
    }
    
    /// Low power
    pub fn low_power() -> Self {
        Self {
            cpu_scale: 0.3,
            gpu_scale: 0.2,
            refresh_rate: 30,
            resolution_scale: 0.5,
            background_allowed: false,
            animation_quality: 0,
            tracking_quality: 0,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self::from_profile(PerformanceProfile::Balanced)
    }
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Frame rate (FPS)
    pub frame_rate: f32,
    /// Frame time (ms)
    pub frame_time: f32,
    /// CPU usage (%)
    pub cpu_usage: f32,
    /// GPU usage (%)
    pub gpu_usage: f32,
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// Dropped frames
    pub dropped_frames: u32,
    /// Timestamp
    pub timestamp: Instant,
}

impl PerformanceMetrics {
    /// Create new
    pub fn new() -> Self {
        Self {
            frame_rate: 60.0,
            frame_time: 16.67,
            cpu_usage: 0.0,
            gpu_usage: 0.0,
            memory_usage: 0,
            dropped_frames: 0,
            timestamp: Instant::now(),
        }
    }
    
    /// Is smooth (>55 FPS target for 60Hz)
    pub fn is_smooth(&self, target_fps: f32) -> bool {
        self.frame_rate >= target_fps * 0.9
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance manager
#[derive(Debug)]
pub struct PerformanceManager {
    /// Current profile
    profile: PerformanceProfile,
    /// Settings
    settings: PerformanceSettings,
    /// Current metrics
    metrics: PerformanceMetrics,
    /// Metrics history
    metrics_history: Vec<PerformanceMetrics>,
    /// Max history
    max_history: usize,
    /// Auto adjust enabled
    auto_adjust: bool,
    /// Target frame rate
    target_fps: f32,
}

impl PerformanceManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            profile: PerformanceProfile::Balanced,
            settings: PerformanceSettings::default(),
            metrics: PerformanceMetrics::new(),
            metrics_history: Vec::new(),
            max_history: 100,
            auto_adjust: true,
            target_fps: 60.0,
        }
    }
    
    /// Get current profile
    pub fn profile(&self) -> PerformanceProfile {
        self.profile
    }
    
    /// Set profile
    pub fn set_profile(&mut self, profile: PerformanceProfile) {
        self.profile = profile;
        self.settings = PerformanceSettings::from_profile(profile);
        self.target_fps = self.settings.refresh_rate as f32;
    }
    
    /// Get settings
    pub fn settings(&self) -> &PerformanceSettings {
        &self.settings
    }
    
    /// Get mutable settings
    pub fn settings_mut(&mut self) -> &mut PerformanceSettings {
        self.profile = PerformanceProfile::Custom;
        &mut self.settings
    }
    
    /// Get current metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Update metrics
    pub fn update_metrics(&mut self, metrics: PerformanceMetrics) {
        // Store history
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(self.metrics.clone());
        
        self.metrics = metrics;
        
        // Auto adjust if enabled
        if self.auto_adjust {
            self.auto_adjust_quality();
        }
    }
    
    /// Auto adjust quality based on metrics
    fn auto_adjust_quality(&mut self) {
        if !self.metrics.is_smooth(self.target_fps) {
            // Performance is poor, reduce quality
            if self.settings.resolution_scale > 0.5 {
                self.settings.resolution_scale -= 0.1;
            }
        } else if self.metrics.frame_rate > self.target_fps * 1.1 {
            // Performance is good, can increase quality
            if self.settings.resolution_scale < 1.0 {
                self.settings.resolution_scale += 0.05;
            }
        }
    }
    
    /// Get average frame rate
    pub fn average_fps(&self) -> f32 {
        if self.metrics_history.is_empty() {
            return self.metrics.frame_rate;
        }
        
        let sum: f32 = self.metrics_history.iter()
            .map(|m| m.frame_rate)
            .sum();
        sum / self.metrics_history.len() as f32
    }
    
    /// Get dropped frame count
    pub fn total_dropped_frames(&self) -> u32 {
        self.metrics_history.iter()
            .map(|m| m.dropped_frames)
            .sum()
    }
    
    /// Set auto adjust
    pub fn set_auto_adjust(&mut self, enabled: bool) {
        self.auto_adjust = enabled;
    }
    
    /// Is auto adjust enabled
    pub fn is_auto_adjust(&self) -> bool {
        self.auto_adjust
    }
    
    /// Apply thermal throttle
    pub fn apply_thermal_throttle(&mut self, throttle_percent: u8) {
        let factor = 1.0 - (throttle_percent as f32 / 100.0);
        self.settings.cpu_scale = (self.settings.cpu_scale * factor).max(0.2);
        self.settings.gpu_scale = (self.settings.gpu_scale * factor).max(0.2);
    }
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_profile_scales() {
        assert!(PerformanceProfile::Performance.cpu_scale() > PerformanceProfile::PowerSave.cpu_scale());
        assert!(PerformanceProfile::Performance.refresh_rate() > PerformanceProfile::UltraPowerSave.refresh_rate());
    }
    
    #[test]
    fn test_performance_settings() {
        let settings = PerformanceSettings::from_profile(PerformanceProfile::Balanced);
        
        assert!(settings.cpu_scale > 0.0);
        assert!(settings.refresh_rate > 0);
    }
    
    #[test]
    fn test_performance_manager_creation() {
        let manager = PerformanceManager::new();
        
        assert_eq!(manager.profile(), PerformanceProfile::Balanced);
    }
    
    #[test]
    fn test_set_profile() {
        let mut manager = PerformanceManager::new();
        
        manager.set_profile(PerformanceProfile::PowerSave);
        
        assert_eq!(manager.profile(), PerformanceProfile::PowerSave);
        assert_eq!(manager.settings().refresh_rate, 60);
    }
    
    #[test]
    fn test_thermal_throttle() {
        let mut manager = PerformanceManager::new();
        let original_cpu = manager.settings().cpu_scale;
        
        manager.apply_thermal_throttle(50);
        
        assert!(manager.settings().cpu_scale < original_cpu);
    }
    
    #[test]
    fn test_metrics_smooth() {
        let mut metrics = PerformanceMetrics::new();
        metrics.frame_rate = 55.0;
        
        assert!(metrics.is_smooth(60.0));
        
        metrics.frame_rate = 30.0;
        assert!(!metrics.is_smooth(60.0));
    }
}
