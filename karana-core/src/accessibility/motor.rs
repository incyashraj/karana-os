//! Motor accessibility for users with motor impairments

use std::time::{Duration, Instant};

/// Input method types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    /// Standard input (gaze + gesture)
    Standard,
    /// Voice control only
    VoiceOnly,
    /// Switch access (external buttons)
    SwitchAccess,
    /// Eye tracking only
    EyeTracking,
    /// Head movement only
    HeadTracking,
    /// External controller
    ExternalController,
}

/// Dwell selection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwellMode {
    /// Disabled
    Disabled,
    /// Simple dwell (look to select)
    Simple,
    /// Confirmed dwell (look + confirm)
    Confirmed,
    /// Progressive (fill indicator)
    Progressive,
}

/// Switch scanning mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanningMode {
    /// No scanning
    None,
    /// Row-column scanning
    RowColumn,
    /// Linear scanning
    Linear,
    /// Group scanning
    Group,
}

/// Motor assist configuration
#[derive(Debug, Clone)]
pub struct MotorConfig {
    /// Primary input method
    pub primary_input: InputMethod,
    /// Secondary input method
    pub secondary_input: Option<InputMethod>,
    /// Dwell selection mode
    pub dwell_mode: DwellMode,
    /// Dwell time required
    pub dwell_time: Duration,
    /// Touch/selection target size multiplier
    pub target_size: f32,
    /// Sticky keys enabled
    pub sticky_keys: bool,
    /// Auto-repeat delay
    pub repeat_delay: Duration,
    /// Auto-repeat rate
    pub repeat_rate: Duration,
    /// Filter tremor/jitter
    pub tremor_filter: bool,
    /// Tremor filter strength
    pub tremor_strength: f32,
    /// Gesture hold time
    pub gesture_hold_time: Duration,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            primary_input: InputMethod::Standard,
            secondary_input: None,
            dwell_mode: DwellMode::Disabled,
            dwell_time: Duration::from_millis(1000),
            target_size: 1.0,
            sticky_keys: false,
            repeat_delay: Duration::from_millis(500),
            repeat_rate: Duration::from_millis(100),
            tremor_filter: false,
            tremor_strength: 0.5,
            gesture_hold_time: Duration::from_millis(300),
        }
    }
}

/// Switch access configuration
#[derive(Debug, Clone)]
pub struct SwitchConfig {
    /// Scanning mode
    pub scanning_mode: ScanningMode,
    /// Scan speed (time per item)
    pub scan_speed: Duration,
    /// Number of switches (1-3)
    pub switch_count: u8,
    /// Auto-scan enabled
    pub auto_scan: bool,
    /// Scan restart delay
    pub restart_delay: Duration,
}

impl Default for SwitchConfig {
    fn default() -> Self {
        Self {
            scanning_mode: ScanningMode::RowColumn,
            scan_speed: Duration::from_millis(1000),
            switch_count: 2,
            auto_scan: true,
            restart_delay: Duration::from_millis(500),
        }
    }
}

/// Dwell indicator state
#[derive(Debug, Clone)]
pub struct DwellIndicator {
    /// Whether dwell is active
    pub active: bool,
    /// Progress (0.0 - 1.0)
    pub progress: f32,
    /// Target position (x, y) in screen coords
    pub position: (f32, f32),
    /// When dwell started
    pub start_time: Option<Instant>,
}

impl Default for DwellIndicator {
    fn default() -> Self {
        Self {
            active: false,
            progress: 0.0,
            position: (0.0, 0.0),
            start_time: None,
        }
    }
}

/// Motor accessibility system
#[derive(Debug)]
pub struct MotorAssist {
    /// Motor configuration
    config: MotorConfig,
    /// Switch configuration
    switch_config: SwitchConfig,
    /// Current dwell state
    dwell_state: DwellIndicator,
    /// Tremor filter buffer
    position_buffer: Vec<(f32, f32)>,
    /// Buffer size for tremor filtering
    buffer_size: usize,
    /// Last interaction time
    last_interaction: Instant,
    /// Voice commands enabled
    voice_commands: bool,
    /// Current scan index
    scan_index: usize,
    /// Last scan advance
    last_scan: Instant,
    /// Timeout for auto-dismiss
    interaction_timeout: Duration,
}

impl MotorAssist {
    /// Create new motor assist system
    pub fn new() -> Self {
        Self {
            config: MotorConfig::default(),
            switch_config: SwitchConfig::default(),
            dwell_state: DwellIndicator::default(),
            position_buffer: Vec::new(),
            buffer_size: 10,
            last_interaction: Instant::now(),
            voice_commands: false,
            scan_index: 0,
            last_scan: Instant::now(),
            interaction_timeout: Duration::from_secs(30),
        }
    }
    
    /// Set motor configuration
    pub fn set_config(&mut self, config: MotorConfig) {
        self.config = config;
    }
    
    /// Get motor configuration
    pub fn config(&self) -> &MotorConfig {
        &self.config
    }
    
    /// Set primary input method
    pub fn set_primary_input(&mut self, method: InputMethod) {
        self.config.primary_input = method;
    }
    
    /// Get primary input method
    pub fn primary_input(&self) -> InputMethod {
        self.config.primary_input
    }
    
    /// Set dwell mode
    pub fn set_dwell_mode(&mut self, mode: DwellMode) {
        self.config.dwell_mode = mode;
    }
    
    /// Get dwell mode
    pub fn dwell_mode(&self) -> DwellMode {
        self.config.dwell_mode
    }
    
    /// Set dwell time
    pub fn set_dwell_time(&mut self, duration: Duration) {
        self.config.dwell_time = duration;
    }
    
    /// Get dwell time
    pub fn dwell_time(&self) -> Duration {
        self.config.dwell_time
    }
    
    /// Set target size multiplier (1.0 - 3.0)
    pub fn set_target_size(&mut self, size: f32) {
        self.config.target_size = size.clamp(1.0, 3.0);
    }
    
    /// Get target size multiplier
    pub fn target_size(&self) -> f32 {
        self.config.target_size
    }
    
    /// Enable/disable tremor filter
    pub fn set_tremor_filter(&mut self, enabled: bool, strength: f32) {
        self.config.tremor_filter = enabled;
        self.config.tremor_strength = strength.clamp(0.0, 1.0);
    }
    
    /// Check if tremor filter is enabled
    pub fn tremor_filter_enabled(&self) -> bool {
        self.config.tremor_filter
    }
    
    /// Enable/disable sticky keys
    pub fn set_sticky_keys(&mut self, enabled: bool) {
        self.config.sticky_keys = enabled;
    }
    
    /// Check if sticky keys enabled
    pub fn sticky_keys_enabled(&self) -> bool {
        self.config.sticky_keys
    }
    
    /// Set switch configuration
    pub fn set_switch_config(&mut self, config: SwitchConfig) {
        self.switch_config = config;
    }
    
    /// Get switch configuration
    pub fn switch_config(&self) -> &SwitchConfig {
        &self.switch_config
    }
    
    /// Enable/disable voice commands
    pub fn set_voice_commands(&mut self, enabled: bool) {
        self.voice_commands = enabled;
    }
    
    /// Check if voice commands enabled
    pub fn voice_commands_enabled(&self) -> bool {
        self.voice_commands
    }
    
    /// Start dwell at position
    pub fn start_dwell(&mut self, x: f32, y: f32) {
        self.dwell_state.active = true;
        self.dwell_state.progress = 0.0;
        self.dwell_state.position = (x, y);
        self.dwell_state.start_time = Some(Instant::now());
    }
    
    /// Update dwell progress
    pub fn update_dwell(&mut self) -> Option<bool> {
        if !self.dwell_state.active {
            return None;
        }
        
        if self.config.dwell_mode == DwellMode::Disabled {
            return None;
        }
        
        if let Some(start) = self.dwell_state.start_time {
            let elapsed = start.elapsed();
            self.dwell_state.progress = 
                elapsed.as_secs_f32() / self.config.dwell_time.as_secs_f32();
            
            if self.dwell_state.progress >= 1.0 {
                self.dwell_state.progress = 1.0;
                return Some(true); // Selection triggered
            }
        }
        
        Some(false) // Still dwelling
    }
    
    /// Cancel dwell
    pub fn cancel_dwell(&mut self) {
        self.dwell_state.active = false;
        self.dwell_state.progress = 0.0;
        self.dwell_state.start_time = None;
    }
    
    /// Get dwell state
    pub fn dwell_state(&self) -> &DwellIndicator {
        &self.dwell_state
    }
    
    /// Filter input position (for tremor)
    pub fn filter_position(&mut self, x: f32, y: f32) -> (f32, f32) {
        if !self.config.tremor_filter {
            return (x, y);
        }
        
        self.position_buffer.push((x, y));
        if self.position_buffer.len() > self.buffer_size {
            self.position_buffer.remove(0);
        }
        
        if self.position_buffer.is_empty() {
            return (x, y);
        }
        
        // Weighted moving average (recent positions weighted more)
        let total_weight: f32 = (1..=self.position_buffer.len())
            .map(|i| i as f32)
            .sum();
        
        let (sum_x, sum_y) = self.position_buffer
            .iter()
            .enumerate()
            .fold((0.0f32, 0.0f32), |(sx, sy), (i, (px, py))| {
                let weight = (i + 1) as f32;
                (sx + px * weight, sy + py * weight)
            });
        
        // Blend between filtered and raw based on strength
        let filtered_x = sum_x / total_weight;
        let filtered_y = sum_y / total_weight;
        
        let strength = self.config.tremor_strength;
        let final_x = x * (1.0 - strength) + filtered_x * strength;
        let final_y = y * (1.0 - strength) + filtered_y * strength;
        
        (final_x, final_y)
    }
    
    /// Advance scan to next item
    pub fn advance_scan(&mut self, item_count: usize) -> usize {
        if item_count == 0 {
            return 0;
        }
        
        self.scan_index = (self.scan_index + 1) % item_count;
        self.last_scan = Instant::now();
        self.scan_index
    }
    
    /// Get current scan index
    pub fn scan_index(&self) -> usize {
        self.scan_index
    }
    
    /// Reset scan to beginning
    pub fn reset_scan(&mut self) {
        self.scan_index = 0;
        self.last_scan = Instant::now();
    }
    
    /// Check if should auto-advance scan
    pub fn should_auto_advance(&self) -> bool {
        if !self.switch_config.auto_scan {
            return false;
        }
        
        self.last_scan.elapsed() >= self.switch_config.scan_speed
    }
    
    /// Record interaction (resets timeout)
    pub fn record_interaction(&mut self) {
        self.last_interaction = Instant::now();
    }
    
    /// Check if interaction has timed out
    pub fn is_timed_out(&self) -> bool {
        self.last_interaction.elapsed() > self.interaction_timeout
    }
    
    /// Update motor assist state
    pub fn update(&mut self, item_count: usize) {
        // Auto-advance scanning if enabled
        if self.should_auto_advance() && item_count > 0 {
            self.advance_scan(item_count);
        }
        
        // Update dwell
        if let Some(true) = self.update_dwell() {
            self.cancel_dwell();
            self.record_interaction();
        }
    }
}

impl Default for MotorAssist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_motor_assist_creation() {
        let ma = MotorAssist::new();
        assert_eq!(ma.primary_input(), InputMethod::Standard);
        assert_eq!(ma.dwell_mode(), DwellMode::Disabled);
    }
    
    #[test]
    fn test_target_size() {
        let mut ma = MotorAssist::new();
        
        ma.set_target_size(2.0);
        assert_eq!(ma.target_size(), 2.0);
        
        // Test clamping
        ma.set_target_size(5.0);
        assert_eq!(ma.target_size(), 3.0);
    }
    
    #[test]
    fn test_dwell_selection() {
        let mut ma = MotorAssist::new();
        ma.set_dwell_mode(DwellMode::Simple);
        ma.set_dwell_time(Duration::from_millis(100));
        
        ma.start_dwell(100.0, 100.0);
        assert!(ma.dwell_state().active);
        
        // Should not trigger immediately
        let result = ma.update_dwell();
        assert_eq!(result, Some(false));
        
        // Wait for dwell time
        std::thread::sleep(Duration::from_millis(150));
        let result = ma.update_dwell();
        assert_eq!(result, Some(true));
    }
    
    #[test]
    fn test_tremor_filter() {
        let mut ma = MotorAssist::new();
        ma.set_tremor_filter(true, 0.5);
        
        // Add some positions
        ma.filter_position(100.0, 100.0);
        ma.filter_position(102.0, 98.0);
        ma.filter_position(99.0, 101.0);
        
        // Filtered position should be smoothed
        let (x, y) = ma.filter_position(101.0, 99.0);
        
        // Should be close to average but weighted toward recent
        assert!((x - 100.0).abs() < 5.0);
        assert!((y - 100.0).abs() < 5.0);
    }
    
    #[test]
    fn test_scanning() {
        let mut ma = MotorAssist::new();
        
        assert_eq!(ma.scan_index(), 0);
        
        ma.advance_scan(5);
        assert_eq!(ma.scan_index(), 1);
        
        ma.advance_scan(5);
        assert_eq!(ma.scan_index(), 2);
        
        // Should wrap around
        ma.advance_scan(5);
        ma.advance_scan(5);
        ma.advance_scan(5);
        assert_eq!(ma.scan_index(), 0);
    }
    
    #[test]
    fn test_voice_commands() {
        let mut ma = MotorAssist::new();
        
        assert!(!ma.voice_commands_enabled());
        ma.set_voice_commands(true);
        assert!(ma.voice_commands_enabled());
    }
    
    #[test]
    fn test_input_methods() {
        let mut ma = MotorAssist::new();
        
        ma.set_primary_input(InputMethod::VoiceOnly);
        assert_eq!(ma.primary_input(), InputMethod::VoiceOnly);
        
        ma.set_primary_input(InputMethod::EyeTracking);
        assert_eq!(ma.primary_input(), InputMethod::EyeTracking);
    }
}
