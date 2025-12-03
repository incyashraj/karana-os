//! Haptic actuator types and control

use std::time::{Duration, Instant};

/// Types of haptic actuators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActuatorType {
    /// Linear Resonant Actuator (LRA) - precise, low latency
    LinearResonant,
    /// Eccentric Rotating Mass (ERM) - traditional vibration motor
    EccentricRotatingMass,
    /// Piezoelectric - ultra-precise, thin profile
    Piezoelectric,
    /// Voice coil - wide frequency range
    VoiceCoil,
}

impl ActuatorType {
    /// Get typical response time
    pub fn response_time(&self) -> Duration {
        match self {
            ActuatorType::LinearResonant => Duration::from_millis(5),
            ActuatorType::EccentricRotatingMass => Duration::from_millis(30),
            ActuatorType::Piezoelectric => Duration::from_millis(1),
            ActuatorType::VoiceCoil => Duration::from_millis(3),
        }
    }
    
    /// Get typical frequency range (Hz)
    pub fn frequency_range(&self) -> (f32, f32) {
        match self {
            ActuatorType::LinearResonant => (150.0, 300.0),
            ActuatorType::EccentricRotatingMass => (50.0, 200.0),
            ActuatorType::Piezoelectric => (100.0, 500.0),
            ActuatorType::VoiceCoil => (20.0, 400.0),
        }
    }
    
    /// Get power efficiency (0-1)
    pub fn efficiency(&self) -> f32 {
        match self {
            ActuatorType::LinearResonant => 0.85,
            ActuatorType::EccentricRotatingMass => 0.60,
            ActuatorType::Piezoelectric => 0.95,
            ActuatorType::VoiceCoil => 0.75,
        }
    }
}

/// Position of actuator on glasses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActuatorPosition {
    /// Left temple arm
    LeftTemple,
    /// Right temple arm
    RightTemple,
    /// Bridge of nose
    Bridge,
    /// Left earpiece
    LeftEar,
    /// Right earpiece
    RightEar,
    /// Frame top
    TopFrame,
}

/// Individual haptic actuator
#[derive(Debug)]
pub struct HapticActuator {
    /// Actuator type
    actuator_type: ActuatorType,
    /// Position on device
    position: ActuatorPosition,
    /// Current intensity (0.0 - 1.0)
    intensity: f32,
    /// Target intensity for ramping
    target_intensity: f32,
    /// Ramp speed (intensity change per second)
    ramp_speed: f32,
    /// Operating frequency (Hz)
    frequency: f32,
    /// Whether actuator is active
    active: bool,
    /// Last update time
    last_update: Instant,
    /// Calibration offset
    calibration_offset: f32,
    /// Maximum safe intensity
    max_intensity: f32,
    /// Minimum perceptible intensity
    min_intensity: f32,
}

impl HapticActuator {
    /// Create new actuator
    pub fn new(actuator_type: ActuatorType, position: ActuatorPosition) -> Self {
        let (min_freq, max_freq) = actuator_type.frequency_range();
        let default_freq = (min_freq + max_freq) / 2.0;
        
        Self {
            actuator_type,
            position,
            intensity: 0.0,
            target_intensity: 0.0,
            ramp_speed: 10.0, // Full ramp in 0.1 seconds
            frequency: default_freq,
            active: false,
            last_update: Instant::now(),
            calibration_offset: 0.0,
            max_intensity: 1.0,
            min_intensity: 0.05,
        }
    }
    
    /// Set intensity immediately
    pub fn set_intensity(&mut self, intensity: f32) {
        let clamped = intensity.clamp(0.0, self.max_intensity) + self.calibration_offset;
        self.intensity = clamped.clamp(0.0, 1.0);
        self.target_intensity = self.intensity;
        self.active = self.intensity > self.min_intensity;
    }
    
    /// Set target intensity for smooth ramping
    pub fn set_target_intensity(&mut self, intensity: f32) {
        self.target_intensity = intensity.clamp(0.0, self.max_intensity);
    }
    
    /// Update actuator (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        if (self.intensity - self.target_intensity).abs() < 0.001 {
            self.intensity = self.target_intensity;
        } else {
            let direction = if self.target_intensity > self.intensity { 1.0 } else { -1.0 };
            let change = self.ramp_speed * delta_time * direction;
            
            if direction > 0.0 {
                self.intensity = (self.intensity + change).min(self.target_intensity);
            } else {
                self.intensity = (self.intensity + change).max(self.target_intensity);
            }
        }
        
        self.active = self.intensity > self.min_intensity;
        self.last_update = Instant::now();
    }
    
    /// Stop actuator immediately
    pub fn stop(&mut self) {
        self.intensity = 0.0;
        self.target_intensity = 0.0;
        self.active = false;
    }
    
    /// Set operating frequency
    pub fn set_frequency(&mut self, frequency: f32) {
        let (min_freq, max_freq) = self.actuator_type.frequency_range();
        self.frequency = frequency.clamp(min_freq, max_freq);
    }
    
    /// Get current intensity
    pub fn intensity(&self) -> f32 {
        self.intensity
    }
    
    /// Get actuator type
    pub fn actuator_type(&self) -> ActuatorType {
        self.actuator_type
    }
    
    /// Get position
    pub fn position(&self) -> ActuatorPosition {
        self.position
    }
    
    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Set calibration offset
    pub fn set_calibration(&mut self, offset: f32) {
        self.calibration_offset = offset.clamp(-0.2, 0.2);
    }
    
    /// Set ramp speed
    pub fn set_ramp_speed(&mut self, speed: f32) {
        self.ramp_speed = speed.clamp(1.0, 100.0);
    }
    
    /// Pulse once at intensity
    pub fn pulse(&mut self, intensity: f32, duration_ms: u32) {
        self.set_intensity(intensity);
        // Note: actual duration control would need async/timer
    }
    
    /// Get estimated power consumption (mW)
    pub fn power_consumption(&self) -> f32 {
        // Rough estimate based on type and intensity
        let base_power = match self.actuator_type {
            ActuatorType::LinearResonant => 50.0,
            ActuatorType::EccentricRotatingMass => 80.0,
            ActuatorType::Piezoelectric => 20.0,
            ActuatorType::VoiceCoil => 100.0,
        };
        
        base_power * self.intensity * self.intensity // Power scales with intensity squared
    }
}

/// Calibration data for actuator
#[derive(Debug, Clone)]
pub struct ActuatorCalibration {
    /// Position being calibrated
    pub position: ActuatorPosition,
    /// Minimum perceptible intensity (user-specific)
    pub perception_threshold: f32,
    /// Comfortable maximum intensity
    pub comfort_max: f32,
    /// Offset to match actuator pair
    pub balance_offset: f32,
    /// Frequency preference
    pub preferred_frequency: f32,
}

impl Default for ActuatorCalibration {
    fn default() -> Self {
        Self {
            position: ActuatorPosition::LeftTemple,
            perception_threshold: 0.05,
            comfort_max: 0.9,
            balance_offset: 0.0,
            preferred_frequency: 200.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_actuator_creation() {
        let actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        assert!(!actuator.is_active());
        assert_eq!(actuator.intensity(), 0.0);
    }
    
    #[test]
    fn test_set_intensity() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        actuator.set_intensity(0.5);
        assert!((actuator.intensity() - 0.5).abs() < 0.001);
        assert!(actuator.is_active());
    }
    
    #[test]
    fn test_intensity_clamping() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        actuator.set_intensity(1.5);
        assert!(actuator.intensity() <= 1.0);
        
        actuator.set_intensity(-0.5);
        assert!(actuator.intensity() >= 0.0);
    }
    
    #[test]
    fn test_stop() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        actuator.set_intensity(0.8);
        assert!(actuator.is_active());
        
        actuator.stop();
        assert!(!actuator.is_active());
        assert_eq!(actuator.intensity(), 0.0);
    }
    
    #[test]
    fn test_actuator_types() {
        let lra = ActuatorType::LinearResonant;
        let erm = ActuatorType::EccentricRotatingMass;
        
        // LRA should have faster response
        assert!(lra.response_time() < erm.response_time());
        
        // LRA should be more efficient
        assert!(lra.efficiency() > erm.efficiency());
    }
    
    #[test]
    fn test_frequency_range() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        let (min_freq, max_freq) = ActuatorType::LinearResonant.frequency_range();
        
        // Should clamp to valid range
        actuator.set_frequency(1000.0);
        assert!(actuator.frequency <= max_freq);
        
        actuator.set_frequency(10.0);
        assert!(actuator.frequency >= min_freq);
    }
    
    #[test]
    fn test_power_consumption() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        let power_idle = actuator.power_consumption();
        
        actuator.set_intensity(1.0);
        let power_full = actuator.power_consumption();
        
        assert!(power_full > power_idle);
    }
    
    #[test]
    fn test_ramping() {
        let mut actuator = HapticActuator::new(
            ActuatorType::LinearResonant,
            ActuatorPosition::LeftTemple
        );
        
        actuator.set_target_intensity(1.0);
        actuator.update(0.05); // 50ms
        
        // Should have started ramping but not reached target
        assert!(actuator.intensity() > 0.0);
        assert!(actuator.intensity() < 1.0);
    }
}
