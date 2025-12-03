//! Haptic Feedback System for Kāraṇa OS AR Glasses
//! 
//! Provides tactile feedback through temple-mounted actuators for
//! notifications, UI interactions, and spatial awareness.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use nalgebra::Vector3;

pub mod actuator;
pub mod patterns;
pub mod spatial;
pub mod notification;
pub mod controller;

pub use actuator::{HapticActuator, ActuatorType, ActuatorPosition};
pub use patterns::{HapticPattern, PatternType, WaveformType};
pub use spatial::{SpatialHaptics, HapticCue, CueDirection};
pub use notification::{HapticNotification, NotificationPriority};
pub use controller::{HapticController, FeedbackMode};

/// Haptic feedback engine
#[derive(Debug)]
pub struct HapticEngine {
    /// Available actuators
    actuators: HashMap<ActuatorPosition, HapticActuator>,
    /// Pattern library
    patterns: HashMap<String, HapticPattern>,
    /// Active playback instances
    active_patterns: Vec<PatternPlayback>,
    /// Global intensity scale (0.0 - 1.0)
    intensity_scale: f32,
    /// Whether engine is enabled
    enabled: bool,
    /// Engine configuration
    config: HapticConfig,
    /// Spatial haptics processor
    spatial: SpatialHaptics,
    /// Controller for device communication
    controller: HapticController,
    /// Last update time
    last_update: Instant,
    /// Statistics
    stats: HapticStats,
}

/// Haptic engine configuration
#[derive(Debug, Clone)]
pub struct HapticConfig {
    /// Maximum simultaneous patterns
    pub max_concurrent_patterns: usize,
    /// Default intensity (0.0 - 1.0)
    pub default_intensity: f32,
    /// Minimum intensity threshold
    pub min_intensity: f32,
    /// Pattern update rate in Hz
    pub update_rate: u32,
    /// Enable spatial haptics
    pub spatial_enabled: bool,
    /// Adaptive intensity based on context
    pub adaptive_intensity: bool,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            max_concurrent_patterns: 4,
            default_intensity: 0.7,
            min_intensity: 0.1,
            update_rate: 100,
            spatial_enabled: true,
            adaptive_intensity: true,
        }
    }
}

/// Active pattern playback state
#[derive(Debug)]
struct PatternPlayback {
    /// Pattern being played
    pattern: HapticPattern,
    /// Start time
    started_at: Instant,
    /// Current position in pattern
    position: f32,
    /// Playback speed multiplier
    speed: f32,
    /// Intensity override
    intensity: f32,
    /// Target actuator(s)
    targets: Vec<ActuatorPosition>,
    /// Loop count (0 = infinite)
    loops: u32,
    /// Current loop iteration
    current_loop: u32,
    /// Priority for conflict resolution
    priority: u8,
}

/// Haptic engine statistics
#[derive(Debug, Default)]
pub struct HapticStats {
    /// Total patterns played
    pub patterns_played: u64,
    /// Total playback time
    pub total_playback_ms: u64,
    /// Patterns currently active
    pub active_count: usize,
    /// Average intensity
    pub avg_intensity: f32,
    /// Spatial cues triggered
    pub spatial_cues: u64,
}

/// Result of haptic playback
#[derive(Debug, Clone)]
pub struct PlaybackHandle {
    /// Unique identifier
    pub id: u64,
    /// Pattern name
    pub pattern_name: String,
    /// Start time
    pub started_at: Instant,
    /// Expected duration
    pub duration: Duration,
}

impl HapticEngine {
    /// Create new haptic engine with default configuration
    pub fn new() -> Self {
        Self::with_config(HapticConfig::default())
    }
    
    /// Create haptic engine with custom configuration
    pub fn with_config(config: HapticConfig) -> Self {
        let mut actuators = HashMap::new();
        
        // Initialize default actuators (temple-mounted)
        actuators.insert(
            ActuatorPosition::LeftTemple,
            HapticActuator::new(ActuatorType::LinearResonant, ActuatorPosition::LeftTemple)
        );
        actuators.insert(
            ActuatorPosition::RightTemple,
            HapticActuator::new(ActuatorType::LinearResonant, ActuatorPosition::RightTemple)
        );
        
        Self {
            actuators,
            patterns: Self::create_default_patterns(),
            active_patterns: Vec::new(),
            intensity_scale: config.default_intensity,
            enabled: true,
            config,
            spatial: SpatialHaptics::new(),
            controller: HapticController::new(),
            last_update: Instant::now(),
            stats: HapticStats::default(),
        }
    }
    
    /// Create default pattern library
    fn create_default_patterns() -> HashMap<String, HapticPattern> {
        let mut patterns = HashMap::new();
        
        // Short tap
        patterns.insert(
            "tap".to_string(),
            HapticPattern::new("tap")
                .add_segment(WaveformType::Sharp, 0.0, 0.8, Duration::from_millis(30))
        );
        
        // Double tap
        patterns.insert(
            "double_tap".to_string(),
            HapticPattern::new("double_tap")
                .add_segment(WaveformType::Sharp, 0.0, 0.8, Duration::from_millis(30))
                .add_pause(Duration::from_millis(80))
                .add_segment(WaveformType::Sharp, 0.0, 0.8, Duration::from_millis(30))
        );
        
        // Long press
        patterns.insert(
            "long_press".to_string(),
            HapticPattern::new("long_press")
                .add_segment(WaveformType::Smooth, 0.0, 0.6, Duration::from_millis(200))
        );
        
        // Success
        patterns.insert(
            "success".to_string(),
            HapticPattern::new("success")
                .add_segment(WaveformType::Sharp, 0.0, 0.7, Duration::from_millis(40))
                .add_pause(Duration::from_millis(60))
                .add_segment(WaveformType::Smooth, 0.0, 0.9, Duration::from_millis(100))
        );
        
        // Error
        patterns.insert(
            "error".to_string(),
            HapticPattern::new("error")
                .add_segment(WaveformType::Buzz, 0.0, 0.9, Duration::from_millis(100))
                .add_pause(Duration::from_millis(50))
                .add_segment(WaveformType::Buzz, 0.0, 0.9, Duration::from_millis(100))
                .add_pause(Duration::from_millis(50))
                .add_segment(WaveformType::Buzz, 0.0, 0.9, Duration::from_millis(100))
        );
        
        // Warning
        patterns.insert(
            "warning".to_string(),
            HapticPattern::new("warning")
                .add_segment(WaveformType::Pulse, 0.0, 0.7, Duration::from_millis(150))
                .add_pause(Duration::from_millis(100))
                .add_segment(WaveformType::Pulse, 0.0, 0.7, Duration::from_millis(150))
        );
        
        // Notification
        patterns.insert(
            "notification".to_string(),
            HapticPattern::new("notification")
                .add_segment(WaveformType::Smooth, 0.2, 0.6, Duration::from_millis(80))
                .add_segment(WaveformType::Smooth, 0.6, 0.3, Duration::from_millis(120))
        );
        
        // Selection
        patterns.insert(
            "selection".to_string(),
            HapticPattern::new("selection")
                .add_segment(WaveformType::Click, 0.0, 0.5, Duration::from_millis(20))
        );
        
        // Scroll tick
        patterns.insert(
            "scroll_tick".to_string(),
            HapticPattern::new("scroll_tick")
                .add_segment(WaveformType::Click, 0.0, 0.3, Duration::from_millis(10))
        );
        
        // Navigation arrival
        patterns.insert(
            "nav_arrival".to_string(),
            HapticPattern::new("nav_arrival")
                .add_segment(WaveformType::Smooth, 0.0, 0.5, Duration::from_millis(100))
                .add_pause(Duration::from_millis(100))
                .add_segment(WaveformType::Smooth, 0.0, 0.7, Duration::from_millis(150))
                .add_pause(Duration::from_millis(100))
                .add_segment(WaveformType::Smooth, 0.0, 0.9, Duration::from_millis(200))
        );
        
        // Heartbeat (for presence/health)
        patterns.insert(
            "heartbeat".to_string(),
            HapticPattern::new("heartbeat")
                .add_segment(WaveformType::Smooth, 0.0, 0.6, Duration::from_millis(80))
                .add_segment(WaveformType::Smooth, 0.6, 0.3, Duration::from_millis(80))
                .add_pause(Duration::from_millis(200))
                .add_segment(WaveformType::Smooth, 0.0, 0.4, Duration::from_millis(60))
                .add_segment(WaveformType::Smooth, 0.4, 0.2, Duration::from_millis(60))
        );
        
        patterns
    }
    
    /// Play a named pattern
    pub fn play(&mut self, pattern_name: &str) -> Option<PlaybackHandle> {
        self.play_with_options(pattern_name, 1.0, Vec::new(), 1, 0)
    }
    
    /// Play pattern with custom options
    pub fn play_with_options(
        &mut self,
        pattern_name: &str,
        intensity: f32,
        targets: Vec<ActuatorPosition>,
        loops: u32,
        priority: u8,
    ) -> Option<PlaybackHandle> {
        if !self.enabled {
            return None;
        }
        
        let pattern = self.patterns.get(pattern_name)?.clone();
        let duration = pattern.duration();
        
        // Determine target actuators
        let targets = if targets.is_empty() {
            vec![ActuatorPosition::LeftTemple, ActuatorPosition::RightTemple]
        } else {
            targets
        };
        
        // Check concurrent pattern limit
        if self.active_patterns.len() >= self.config.max_concurrent_patterns {
            // Remove lowest priority pattern if new one is higher
            if let Some(idx) = self.find_lowest_priority_pattern() {
                if self.active_patterns[idx].priority < priority {
                    self.active_patterns.remove(idx);
                } else {
                    return None;
                }
            }
        }
        
        let playback = PatternPlayback {
            pattern: pattern.clone(),
            started_at: Instant::now(),
            position: 0.0,
            speed: 1.0,
            intensity: intensity.clamp(0.0, 1.0),
            targets,
            loops,
            current_loop: 0,
            priority,
        };
        
        self.active_patterns.push(playback);
        self.stats.patterns_played += 1;
        
        Some(PlaybackHandle {
            id: self.stats.patterns_played,
            pattern_name: pattern_name.to_string(),
            started_at: Instant::now(),
            duration,
        })
    }
    
    /// Play custom pattern
    pub fn play_custom(&mut self, pattern: HapticPattern) -> Option<PlaybackHandle> {
        if !self.enabled {
            return None;
        }
        
        let duration = pattern.duration();
        let name = pattern.name.clone();
        
        let playback = PatternPlayback {
            pattern,
            started_at: Instant::now(),
            position: 0.0,
            speed: 1.0,
            intensity: 1.0,
            targets: vec![ActuatorPosition::LeftTemple, ActuatorPosition::RightTemple],
            loops: 1,
            current_loop: 0,
            priority: 5,
        };
        
        self.active_patterns.push(playback);
        self.stats.patterns_played += 1;
        
        Some(PlaybackHandle {
            id: self.stats.patterns_played,
            pattern_name: name,
            started_at: Instant::now(),
            duration,
        })
    }
    
    /// Find lowest priority pattern index
    fn find_lowest_priority_pattern(&self) -> Option<usize> {
        self.active_patterns
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| p.priority)
            .map(|(i, _)| i)
    }
    
    /// Stop all active patterns
    pub fn stop_all(&mut self) {
        self.active_patterns.clear();
        for actuator in self.actuators.values_mut() {
            actuator.stop();
        }
    }
    
    /// Stop specific pattern by handle ID
    pub fn stop(&mut self, handle_id: u64) {
        // Since we don't store handle IDs, stop by position
        if let Some(idx) = self.active_patterns.iter().position(|p| {
            p.started_at.elapsed().as_millis() as u64 == handle_id % 1000
        }) {
            self.active_patterns.remove(idx);
        }
    }
    
    /// Update haptic engine (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }
        
        let update_interval = Duration::from_secs_f32(1.0 / self.config.update_rate as f32);
        if self.last_update.elapsed() < update_interval {
            return;
        }
        self.last_update = Instant::now();
        
        // Update active patterns
        let mut completed = Vec::new();
        let mut actuator_values: HashMap<ActuatorPosition, f32> = HashMap::new();
        
        for (idx, playback) in self.active_patterns.iter_mut().enumerate() {
            let elapsed = playback.started_at.elapsed();
            let pattern_duration = playback.pattern.duration();
            
            // Check if pattern completed
            if elapsed >= pattern_duration {
                playback.current_loop += 1;
                if playback.loops > 0 && playback.current_loop >= playback.loops {
                    completed.push(idx);
                    continue;
                }
                // Reset for next loop
                playback.started_at = Instant::now();
                playback.position = 0.0;
            }
            
            // Calculate current intensity
            let position = elapsed.as_secs_f32() / pattern_duration.as_secs_f32().max(0.001);
            let base_intensity = playback.pattern.sample(position);
            let intensity = base_intensity * playback.intensity * self.intensity_scale;
            
            // Apply to target actuators
            for target in &playback.targets {
                let entry = actuator_values.entry(*target).or_insert(0.0);
                *entry = (*entry + intensity).min(1.0);
            }
        }
        
        // Remove completed patterns (in reverse order to preserve indices)
        for idx in completed.into_iter().rev() {
            self.active_patterns.remove(idx);
        }
        
        // Apply values to actuators
        for (position, intensity) in actuator_values {
            if let Some(actuator) = self.actuators.get_mut(&position) {
                actuator.set_intensity(intensity);
            }
        }
        
        // Update stats
        self.stats.active_count = self.active_patterns.len();
    }
    
    /// Set global intensity scale
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity_scale = intensity.clamp(0.0, 1.0);
    }
    
    /// Get global intensity scale
    pub fn intensity(&self) -> f32 {
        self.intensity_scale
    }
    
    /// Enable/disable haptics
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.stop_all();
        }
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Add custom pattern to library
    pub fn add_pattern(&mut self, pattern: HapticPattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }
    
    /// Get pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&HapticPattern> {
        self.patterns.get(name)
    }
    
    /// List available patterns
    pub fn list_patterns(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
    
    /// Play directional haptic cue
    pub fn play_directional_cue(&mut self, direction: Vector3<f32>, intensity: f32) {
        if !self.enabled || !self.config.spatial_enabled {
            return;
        }
        
        let (left_intensity, right_intensity) = self.spatial.calculate_stereo_intensity(
            direction,
            intensity * self.intensity_scale,
        );
        
        if let Some(left) = self.actuators.get_mut(&ActuatorPosition::LeftTemple) {
            left.set_intensity(left_intensity);
        }
        if let Some(right) = self.actuators.get_mut(&ActuatorPosition::RightTemple) {
            right.set_intensity(right_intensity);
        }
        
        self.stats.spatial_cues += 1;
    }
    
    /// Get engine statistics
    pub fn stats(&self) -> &HapticStats {
        &self.stats
    }
    
    /// Get active pattern count
    pub fn active_pattern_count(&self) -> usize {
        self.active_patterns.len()
    }
}

impl Default for HapticEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = HapticEngine::new();
        assert!(engine.is_enabled());
        assert!(engine.list_patterns().len() > 0);
    }
    
    #[test]
    fn test_play_pattern() {
        let mut engine = HapticEngine::new();
        let handle = engine.play("tap");
        assert!(handle.is_some());
        assert_eq!(handle.unwrap().pattern_name, "tap");
    }
    
    #[test]
    fn test_play_unknown_pattern() {
        let mut engine = HapticEngine::new();
        let handle = engine.play("nonexistent");
        assert!(handle.is_none());
    }
    
    #[test]
    fn test_intensity_control() {
        let mut engine = HapticEngine::new();
        engine.set_intensity(0.5);
        assert!((engine.intensity() - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_enable_disable() {
        let mut engine = HapticEngine::new();
        engine.set_enabled(false);
        assert!(!engine.is_enabled());
        
        let handle = engine.play("tap");
        assert!(handle.is_none());
    }
    
    #[test]
    fn test_custom_pattern() {
        let mut engine = HapticEngine::new();
        let pattern = HapticPattern::new("custom")
            .add_segment(WaveformType::Smooth, 0.0, 1.0, Duration::from_millis(100));
        
        let handle = engine.play_custom(pattern);
        assert!(handle.is_some());
    }
    
    #[test]
    fn test_add_pattern() {
        let mut engine = HapticEngine::new();
        let pattern = HapticPattern::new("my_pattern")
            .add_segment(WaveformType::Sharp, 0.0, 0.8, Duration::from_millis(50));
        
        engine.add_pattern(pattern);
        assert!(engine.get_pattern("my_pattern").is_some());
    }
    
    #[test]
    fn test_stop_all() {
        let mut engine = HapticEngine::new();
        engine.play("tap");
        engine.play("success");
        
        assert!(engine.active_pattern_count() > 0);
        engine.stop_all();
        assert_eq!(engine.active_pattern_count(), 0);
    }
    
    #[test]
    fn test_default_patterns() {
        let engine = HapticEngine::new();
        let patterns = engine.list_patterns();
        
        assert!(patterns.contains(&"tap"));
        assert!(patterns.contains(&"double_tap"));
        assert!(patterns.contains(&"success"));
        assert!(patterns.contains(&"error"));
        assert!(patterns.contains(&"notification"));
    }
    
    #[test]
    fn test_directional_cue() {
        let mut engine = HapticEngine::new();
        engine.play_directional_cue(Vector3::new(1.0, 0.0, 0.0), 0.8);
        assert!(engine.stats().spatial_cues > 0);
    }
}
