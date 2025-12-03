//! Haptic controller for device communication

use std::time::{Duration, Instant};
use crate::haptics::actuator::{ActuatorPosition, ActuatorType};

/// Haptic controller for communicating with hardware
#[derive(Debug)]
pub struct HapticController {
    /// Connection state
    connected: bool,
    /// Feedback mode
    mode: FeedbackMode,
    /// Device capabilities
    capabilities: DeviceCapabilities,
    /// Output buffer for commands
    command_buffer: Vec<HapticCommand>,
    /// Last command sent time
    last_command: Option<Instant>,
    /// Command rate limit (commands per second)
    rate_limit: u32,
    /// Error count
    error_count: u32,
}

/// Feedback modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackMode {
    /// Normal operation
    Normal,
    /// Reduced feedback (quiet mode)
    Reduced,
    /// Enhanced feedback (accessibility)
    Enhanced,
    /// Silent (no haptics)
    Silent,
    /// Test mode
    Test,
}

impl FeedbackMode {
    /// Get intensity scale for mode
    pub fn intensity_scale(&self) -> f32 {
        match self {
            FeedbackMode::Normal => 1.0,
            FeedbackMode::Reduced => 0.5,
            FeedbackMode::Enhanced => 1.2,
            FeedbackMode::Silent => 0.0,
            FeedbackMode::Test => 1.0,
        }
    }
}

/// Device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    /// Number of actuators
    pub actuator_count: usize,
    /// Actuator positions
    pub positions: Vec<ActuatorPosition>,
    /// Actuator types
    pub actuator_types: Vec<ActuatorType>,
    /// Maximum frequency (Hz)
    pub max_frequency: f32,
    /// Minimum frequency (Hz)
    pub min_frequency: f32,
    /// Intensity resolution (bits)
    pub intensity_bits: u8,
    /// Maximum simultaneous patterns
    pub max_patterns: usize,
    /// Supports waveform streaming
    pub waveform_streaming: bool,
    /// Battery powered
    pub battery_powered: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            actuator_count: 2,
            positions: vec![ActuatorPosition::LeftTemple, ActuatorPosition::RightTemple],
            actuator_types: vec![ActuatorType::LinearResonant, ActuatorType::LinearResonant],
            max_frequency: 300.0,
            min_frequency: 150.0,
            intensity_bits: 8,
            max_patterns: 4,
            waveform_streaming: true,
            battery_powered: true,
        }
    }
}

/// Haptic command to send to device
#[derive(Debug, Clone)]
pub struct HapticCommand {
    /// Target actuator
    pub target: ActuatorPosition,
    /// Command type
    pub command_type: CommandType,
    /// Timestamp
    pub timestamp: Instant,
}

/// Types of haptic commands
#[derive(Debug, Clone)]
pub enum CommandType {
    /// Set intensity (0-255)
    SetIntensity(u8),
    /// Set frequency (Hz)
    SetFrequency(f32),
    /// Play named pattern
    PlayPattern(String),
    /// Stop playback
    Stop,
    /// Play waveform data
    PlayWaveform(Vec<u8>),
    /// Calibrate actuator
    Calibrate,
}

impl HapticController {
    /// Create new haptic controller
    pub fn new() -> Self {
        Self {
            connected: false,
            mode: FeedbackMode::Normal,
            capabilities: DeviceCapabilities::default(),
            command_buffer: Vec::new(),
            last_command: None,
            rate_limit: 100, // 100 commands per second max
            error_count: 0,
        }
    }
    
    /// Connect to haptic device
    pub fn connect(&mut self) -> Result<(), HapticError> {
        // Simulated connection
        self.connected = true;
        Ok(())
    }
    
    /// Disconnect from device
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.command_buffer.clear();
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Set feedback mode
    pub fn set_mode(&mut self, mode: FeedbackMode) {
        self.mode = mode;
    }
    
    /// Get current mode
    pub fn mode(&self) -> FeedbackMode {
        self.mode
    }
    
    /// Send command to actuator
    pub fn send_command(&mut self, target: ActuatorPosition, command: CommandType) -> Result<(), HapticError> {
        if !self.connected {
            return Err(HapticError::NotConnected);
        }
        
        if self.mode == FeedbackMode::Silent {
            return Ok(()); // Silently ignore
        }
        
        // Check rate limit (skip in test mode)
        if self.mode != FeedbackMode::Test {
            if let Some(last) = self.last_command {
                let min_interval = Duration::from_secs_f32(1.0 / self.rate_limit as f32);
                if last.elapsed() < min_interval {
                    return Err(HapticError::RateLimited);
                }
            }
        }
        
        // Apply mode scaling to intensity commands
        let command = match command {
            CommandType::SetIntensity(intensity) => {
                let scaled = (intensity as f32 * self.mode.intensity_scale()) as u8;
                CommandType::SetIntensity(scaled.min(255))
            }
            other => other,
        };
        
        let haptic_cmd = HapticCommand {
            target,
            command_type: command,
            timestamp: Instant::now(),
        };
        
        self.command_buffer.push(haptic_cmd);
        self.last_command = Some(Instant::now());
        
        // Simulate sending to device
        self.flush_commands()?;
        
        Ok(())
    }
    
    /// Flush command buffer to device
    fn flush_commands(&mut self) -> Result<(), HapticError> {
        if !self.connected {
            return Err(HapticError::NotConnected);
        }
        
        // Simulated transmission
        self.command_buffer.clear();
        Ok(())
    }
    
    /// Get device capabilities
    pub fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }
    
    /// Run self-test
    pub fn self_test(&mut self) -> Result<TestResult, HapticError> {
        if !self.connected {
            return Err(HapticError::NotConnected);
        }
        
        let old_mode = self.mode;
        self.mode = FeedbackMode::Test;
        
        // Test each actuator
        let mut results = Vec::new();
        for position in &self.capabilities.positions.clone() {
            let result = self.test_actuator(*position);
            results.push((*position, result));
        }
        
        self.mode = old_mode;
        
        let passed = results.iter().all(|(_, r)| r.is_ok());
        
        Ok(TestResult {
            passed,
            actuator_results: results,
            firmware_version: "1.0.0".to_string(),
        })
    }
    
    /// Test individual actuator
    fn test_actuator(&mut self, position: ActuatorPosition) -> Result<(), HapticError> {
        // Simulated test
        self.send_command(position, CommandType::SetIntensity(128))?;
        self.send_command(position, CommandType::Stop)?;
        Ok(())
    }
    
    /// Get error count
    pub fn error_count(&self) -> u32 {
        self.error_count
    }
    
    /// Reset error count
    pub fn reset_errors(&mut self) {
        self.error_count = 0;
    }
    
    /// Set intensity for position (convenience method)
    pub fn set_intensity(&mut self, position: ActuatorPosition, intensity: f32) -> Result<(), HapticError> {
        let intensity_u8 = (intensity.clamp(0.0, 1.0) * 255.0) as u8;
        self.send_command(position, CommandType::SetIntensity(intensity_u8))
    }
    
    /// Stop all actuators
    pub fn stop_all(&mut self) -> Result<(), HapticError> {
        for position in self.capabilities.positions.clone() {
            self.send_command(position, CommandType::Stop)?;
        }
        Ok(())
    }
}

impl Default for HapticController {
    fn default() -> Self {
        Self::new()
    }
}

/// Haptic errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HapticError {
    /// Device not connected
    NotConnected,
    /// Command rate limited
    RateLimited,
    /// Invalid command
    InvalidCommand,
    /// Device error
    DeviceError(String),
    /// Timeout
    Timeout,
}

impl std::fmt::Display for HapticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HapticError::NotConnected => write!(f, "Haptic device not connected"),
            HapticError::RateLimited => write!(f, "Command rate limited"),
            HapticError::InvalidCommand => write!(f, "Invalid haptic command"),
            HapticError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            HapticError::Timeout => write!(f, "Command timeout"),
        }
    }
}

impl std::error::Error for HapticError {}

/// Self-test result
#[derive(Debug)]
pub struct TestResult {
    /// Overall test passed
    pub passed: bool,
    /// Per-actuator results
    pub actuator_results: Vec<(ActuatorPosition, Result<(), HapticError>)>,
    /// Firmware version
    pub firmware_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_controller_creation() {
        let controller = HapticController::new();
        assert!(!controller.is_connected());
        assert_eq!(controller.mode(), FeedbackMode::Normal);
    }
    
    #[test]
    fn test_connect_disconnect() {
        let mut controller = HapticController::new();
        
        assert!(controller.connect().is_ok());
        assert!(controller.is_connected());
        
        controller.disconnect();
        assert!(!controller.is_connected());
    }
    
    #[test]
    fn test_send_command_not_connected() {
        let mut controller = HapticController::new();
        
        let result = controller.send_command(
            ActuatorPosition::LeftTemple,
            CommandType::SetIntensity(128)
        );
        
        assert_eq!(result, Err(HapticError::NotConnected));
    }
    
    #[test]
    fn test_send_command_connected() {
        let mut controller = HapticController::new();
        controller.connect().unwrap();
        
        let result = controller.send_command(
            ActuatorPosition::LeftTemple,
            CommandType::SetIntensity(128)
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_silent_mode() {
        let mut controller = HapticController::new();
        controller.connect().unwrap();
        controller.set_mode(FeedbackMode::Silent);
        
        // Should succeed but not actually send
        let result = controller.send_command(
            ActuatorPosition::LeftTemple,
            CommandType::SetIntensity(255)
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_intensity_scaling() {
        let reduced = FeedbackMode::Reduced.intensity_scale();
        let enhanced = FeedbackMode::Enhanced.intensity_scale();
        
        assert!(reduced < 1.0);
        assert!(enhanced > 1.0);
    }
    
    #[test]
    fn test_self_test() {
        let mut controller = HapticController::new();
        controller.connect().unwrap();
        
        let result = controller.self_test();
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }
    
    #[test]
    fn test_capabilities() {
        let controller = HapticController::new();
        let caps = controller.capabilities();
        
        assert_eq!(caps.actuator_count, 2);
        assert!(caps.max_frequency > caps.min_frequency);
    }
    
    #[test]
    fn test_stop_all() {
        let mut controller = HapticController::new();
        controller.connect().unwrap();
        controller.set_mode(FeedbackMode::Test); // Use test mode to bypass rate limit
        
        // Set some intensities
        controller.set_intensity(ActuatorPosition::LeftTemple, 0.8).unwrap();
        controller.set_intensity(ActuatorPosition::RightTemple, 0.8).unwrap();
        
        // Stop all
        assert!(controller.stop_all().is_ok());
    }
}
