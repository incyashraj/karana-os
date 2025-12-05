//! Haptic Feedback Integration for Kāraṇa OS
//!
//! Provides tactile feedback for gesture recognition.
//! Supports various haptic patterns for different gesture events.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use super::{GestureType, DynamicGestureType};

/// Haptic feedback manager
pub struct HapticFeedback {
    /// Haptic device state
    device: HapticDevice,
    /// Configuration
    config: HapticConfig,
    /// Registered patterns
    patterns: HashMap<String, HapticPattern>,
    /// Gesture-to-pattern mapping
    gesture_mappings: HashMap<GestureFeedbackKey, String>,
    /// Queue of pending effects
    effect_queue: Vec<QueuedEffect>,
    /// Currently playing effect
    current_effect: Option<PlayingEffect>,
    /// Statistics
    stats: HapticStats,
    /// Last feedback time
    last_feedback: Option<Instant>,
}

/// Haptic device abstraction
#[derive(Debug)]
pub struct HapticDevice {
    /// Device type
    pub device_type: HapticDeviceType,
    /// Is device available
    pub available: bool,
    /// Maximum intensity (0-1)
    pub max_intensity: f32,
    /// Minimum pulse duration (ms)
    pub min_pulse_ms: u32,
    /// Supports waveforms
    pub supports_waveforms: bool,
    /// Number of actuators
    pub actuator_count: u8,
}

/// Haptic device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticDeviceType {
    /// Linear resonant actuator
    Lra,
    /// Eccentric rotating mass
    Erm,
    /// Piezoelectric actuator
    Piezo,
    /// Voice coil motor
    VoiceCoil,
    /// Software simulation (no haptics)
    Simulated,
}

/// Haptic configuration
#[derive(Debug, Clone)]
pub struct HapticConfig {
    /// Master enable
    pub enabled: bool,
    /// Global intensity multiplier (0-1)
    pub intensity: f32,
    /// Minimum interval between feedback (ms)
    pub min_interval_ms: u64,
    /// Enable feedback on gesture start
    pub feedback_on_start: bool,
    /// Enable feedback on gesture recognized
    pub feedback_on_recognized: bool,
    /// Enable feedback on gesture end
    pub feedback_on_end: bool,
    /// Enable feedback on errors
    pub feedback_on_error: bool,
    /// Adaptive intensity based on gesture confidence
    pub adaptive_intensity: bool,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.7,
            min_interval_ms: 50,
            feedback_on_start: false,
            feedback_on_recognized: true,
            feedback_on_end: false,
            feedback_on_error: true,
            adaptive_intensity: true,
        }
    }
}

/// Haptic pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticPattern {
    /// Pattern name
    pub name: String,
    /// Description
    pub description: String,
    /// Pattern elements
    pub elements: Vec<HapticElement>,
    /// Total duration
    pub duration_ms: u32,
    /// Default intensity
    pub default_intensity: f32,
}

/// Single element in a haptic pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticElement {
    /// Element type
    pub element_type: HapticElementType,
    /// Start time offset (ms)
    pub start_ms: u32,
    /// Duration (ms)
    pub duration_ms: u32,
    /// Intensity (0-1)
    pub intensity: f32,
    /// Frequency (Hz) for supported devices
    pub frequency_hz: Option<f32>,
}

/// Haptic element types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HapticElementType {
    /// Simple pulse
    Pulse,
    /// Continuous vibration
    Vibrate,
    /// Ramp up intensity
    RampUp,
    /// Ramp down intensity
    RampDown,
    /// Click (very short pulse)
    Click,
    /// Double click
    DoubleClick,
    /// Tick (subtle feedback)
    Tick,
    /// Buzz (error-like)
    Buzz,
    /// Custom waveform
    Waveform,
}

/// Key for gesture feedback mapping
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GestureFeedbackKey {
    /// Gesture type
    pub gesture: GestureFeedbackType,
    /// Event type
    pub event: GestureFeedbackEvent,
}

/// Gesture types for feedback
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum GestureFeedbackType {
    /// Pinch gesture
    Pinch,
    /// Point gesture
    Point,
    /// Swipe gesture
    Swipe,
    /// Grab gesture
    Grab,
    /// Tap gesture
    Tap,
    /// Hold gesture
    Hold,
    /// Rotate gesture
    Rotate,
    /// Zoom gesture
    Zoom,
    /// Custom gesture by name
    Custom(String),
    /// Any gesture
    Any,
}

/// Gesture events for feedback
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum GestureFeedbackEvent {
    /// Gesture started
    Started,
    /// Gesture recognized
    Recognized,
    /// Gesture ended
    Ended,
    /// Gesture cancelled
    Cancelled,
    /// Gesture in progress
    Progress,
    /// Error during gesture
    Error,
}

/// Queued haptic effect
#[derive(Debug)]
struct QueuedEffect {
    pattern: HapticPattern,
    intensity: f32,
    queued_at: Instant,
    priority: u8,
}

/// Currently playing effect
#[derive(Debug)]
struct PlayingEffect {
    pattern: HapticPattern,
    started_at: Instant,
    intensity: f32,
    current_element: usize,
}

/// Haptic statistics
#[derive(Debug, Default)]
pub struct HapticStats {
    /// Total effects played
    pub effects_played: u64,
    /// Effects dropped due to rate limiting
    pub effects_dropped: u64,
    /// Total duration played (ms)
    pub total_duration_ms: u64,
    /// Average intensity
    pub avg_intensity: f32,
}

impl HapticFeedback {
    /// Create new haptic feedback manager
    pub fn new() -> Self {
        let mut manager = Self {
            device: HapticDevice::simulated(),
            config: HapticConfig::default(),
            patterns: HashMap::new(),
            gesture_mappings: HashMap::new(),
            effect_queue: Vec::new(),
            current_effect: None,
            stats: HapticStats::default(),
            last_feedback: None,
        };

        // Register default patterns
        manager.register_default_patterns();
        manager.setup_default_mappings();

        manager
    }

    /// Create with device
    pub fn with_device(device: HapticDevice) -> Self {
        let mut manager = Self::new();
        manager.device = device;
        manager
    }

    /// Set configuration
    pub fn set_config(&mut self, config: HapticConfig) {
        self.config = config;
    }

    /// Check if haptics are enabled and available
    pub fn is_available(&self) -> bool {
        self.config.enabled && self.device.available
    }

    /// Trigger haptic feedback for gesture event
    pub fn on_gesture(&mut self, gesture_type: GestureFeedbackType, event: GestureFeedbackEvent, confidence: f32) {
        if !self.config.enabled {
            return;
        }

        // Check event type settings
        let should_trigger = match event {
            GestureFeedbackEvent::Started => self.config.feedback_on_start,
            GestureFeedbackEvent::Recognized => self.config.feedback_on_recognized,
            GestureFeedbackEvent::Ended => self.config.feedback_on_end,
            GestureFeedbackEvent::Error => self.config.feedback_on_error,
            _ => true,
        };

        if !should_trigger {
            return;
        }

        // Find pattern for this gesture/event
        let key = GestureFeedbackKey {
            gesture: gesture_type.clone(),
            event: event.clone(),
        };

        let pattern_name = self.gesture_mappings.get(&key)
            .or_else(|| {
                // Try "any" gesture
                let any_key = GestureFeedbackKey {
                    gesture: GestureFeedbackType::Any,
                    event,
                };
                self.gesture_mappings.get(&any_key)
            })
            .cloned();

        if let Some(name) = pattern_name {
            let intensity = if self.config.adaptive_intensity {
                self.config.intensity * confidence
            } else {
                self.config.intensity
            };

            self.play_pattern(&name, intensity);
        }
    }

    /// Play a pattern by name
    pub fn play_pattern(&mut self, name: &str, intensity: f32) {
        // Check rate limiting
        if let Some(last) = self.last_feedback {
            if last.elapsed().as_millis() < self.config.min_interval_ms as u128 {
                self.stats.effects_dropped += 1;
                return;
            }
        }

        if let Some(pattern) = self.patterns.get(name).cloned() {
            self.play_effect(pattern, intensity);
        }
    }

    /// Play a haptic effect
    fn play_effect(&mut self, pattern: HapticPattern, intensity: f32) {
        if !self.device.available {
            return;
        }

        let final_intensity = (intensity * self.device.max_intensity).min(1.0);

        // For now, simulate by setting current effect
        self.current_effect = Some(PlayingEffect {
            pattern: pattern.clone(),
            started_at: Instant::now(),
            intensity: final_intensity,
            current_element: 0,
        });

        self.last_feedback = Some(Instant::now());
        self.stats.effects_played += 1;
        self.stats.total_duration_ms += pattern.duration_ms as u64;

        // Update average intensity
        let n = self.stats.effects_played as f32;
        self.stats.avg_intensity = (self.stats.avg_intensity * (n - 1.0) + final_intensity) / n;
    }

    /// Update haptic state (call from main loop)
    pub fn update(&mut self) -> Option<HapticOutput> {
        let effect = self.current_effect.as_mut()?;
        let elapsed = effect.started_at.elapsed().as_millis() as u32;

        if elapsed >= effect.pattern.duration_ms {
            self.current_effect = None;
            return Some(HapticOutput::Stop);
        }

        // Find current element
        for (i, element) in effect.pattern.elements.iter().enumerate() {
            if elapsed >= element.start_ms && elapsed < element.start_ms + element.duration_ms {
                let element_progress = (elapsed - element.start_ms) as f32 / element.duration_ms as f32;
                let element_intensity = match element.element_type {
                    HapticElementType::RampUp => element.intensity * element_progress,
                    HapticElementType::RampDown => element.intensity * (1.0 - element_progress),
                    _ => element.intensity,
                } * effect.intensity;

                return Some(HapticOutput::Vibrate {
                    intensity: element_intensity,
                    frequency_hz: element.frequency_hz,
                });
            }
        }

        Some(HapticOutput::Idle)
    }

    /// Register a haptic pattern
    pub fn register_pattern(&mut self, pattern: HapticPattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }

    /// Map gesture to pattern
    pub fn map_gesture(&mut self, gesture: GestureFeedbackType, event: GestureFeedbackEvent, pattern_name: &str) {
        let key = GestureFeedbackKey { gesture, event };
        self.gesture_mappings.insert(key, pattern_name.to_string());
    }

    fn register_default_patterns(&mut self) {
        // Click pattern - very short pulse
        self.patterns.insert("click".to_string(), HapticPattern {
            name: "click".to_string(),
            description: "Short click feedback".to_string(),
            elements: vec![HapticElement {
                element_type: HapticElementType::Click,
                start_ms: 0,
                duration_ms: 10,
                intensity: 0.8,
                frequency_hz: Some(150.0),
            }],
            duration_ms: 10,
            default_intensity: 0.8,
        });

        // Double click pattern
        self.patterns.insert("double_click".to_string(), HapticPattern {
            name: "double_click".to_string(),
            description: "Double click feedback".to_string(),
            elements: vec![
                HapticElement {
                    element_type: HapticElementType::Click,
                    start_ms: 0,
                    duration_ms: 10,
                    intensity: 0.8,
                    frequency_hz: Some(150.0),
                },
                HapticElement {
                    element_type: HapticElementType::Click,
                    start_ms: 50,
                    duration_ms: 10,
                    intensity: 0.8,
                    frequency_hz: Some(150.0),
                },
            ],
            duration_ms: 60,
            default_intensity: 0.8,
        });

        // Success pattern
        self.patterns.insert("success".to_string(), HapticPattern {
            name: "success".to_string(),
            description: "Success/confirm feedback".to_string(),
            elements: vec![
                HapticElement {
                    element_type: HapticElementType::Tick,
                    start_ms: 0,
                    duration_ms: 15,
                    intensity: 0.5,
                    frequency_hz: Some(100.0),
                },
                HapticElement {
                    element_type: HapticElementType::Pulse,
                    start_ms: 30,
                    duration_ms: 20,
                    intensity: 0.7,
                    frequency_hz: Some(200.0),
                },
            ],
            duration_ms: 50,
            default_intensity: 0.7,
        });

        // Error pattern
        self.patterns.insert("error".to_string(), HapticPattern {
            name: "error".to_string(),
            description: "Error feedback".to_string(),
            elements: vec![
                HapticElement {
                    element_type: HapticElementType::Buzz,
                    start_ms: 0,
                    duration_ms: 50,
                    intensity: 0.9,
                    frequency_hz: Some(50.0),
                },
                HapticElement {
                    element_type: HapticElementType::Buzz,
                    start_ms: 70,
                    duration_ms: 50,
                    intensity: 0.9,
                    frequency_hz: Some(50.0),
                },
            ],
            duration_ms: 120,
            default_intensity: 0.9,
        });

        // Light tick
        self.patterns.insert("tick".to_string(), HapticPattern {
            name: "tick".to_string(),
            description: "Light tick feedback".to_string(),
            elements: vec![HapticElement {
                element_type: HapticElementType::Tick,
                start_ms: 0,
                duration_ms: 5,
                intensity: 0.3,
                frequency_hz: Some(200.0),
            }],
            duration_ms: 5,
            default_intensity: 0.3,
        });

        // Selection pattern
        self.patterns.insert("select".to_string(), HapticPattern {
            name: "select".to_string(),
            description: "Selection feedback".to_string(),
            elements: vec![HapticElement {
                element_type: HapticElementType::Pulse,
                start_ms: 0,
                duration_ms: 15,
                intensity: 0.6,
                frequency_hz: Some(150.0),
            }],
            duration_ms: 15,
            default_intensity: 0.6,
        });

        // Long press pattern
        self.patterns.insert("long_press".to_string(), HapticPattern {
            name: "long_press".to_string(),
            description: "Long press hold feedback".to_string(),
            elements: vec![HapticElement {
                element_type: HapticElementType::RampUp,
                start_ms: 0,
                duration_ms: 100,
                intensity: 0.6,
                frequency_hz: Some(120.0),
            }],
            duration_ms: 100,
            default_intensity: 0.6,
        });

        // Release pattern
        self.patterns.insert("release".to_string(), HapticPattern {
            name: "release".to_string(),
            description: "Release feedback".to_string(),
            elements: vec![HapticElement {
                element_type: HapticElementType::RampDown,
                start_ms: 0,
                duration_ms: 30,
                intensity: 0.5,
                frequency_hz: Some(100.0),
            }],
            duration_ms: 30,
            default_intensity: 0.5,
        });
    }

    fn setup_default_mappings(&mut self) {
        // Pinch gestures
        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Pinch,
                event: GestureFeedbackEvent::Recognized,
            },
            "click".to_string(),
        );

        // Tap gestures
        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Tap,
                event: GestureFeedbackEvent::Recognized,
            },
            "tick".to_string(),
        );

        // Hold gestures
        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Hold,
                event: GestureFeedbackEvent::Recognized,
            },
            "long_press".to_string(),
        );

        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Hold,
                event: GestureFeedbackEvent::Ended,
            },
            "release".to_string(),
        );

        // Grab gestures
        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Grab,
                event: GestureFeedbackEvent::Recognized,
            },
            "select".to_string(),
        );

        // Error - any gesture
        self.gesture_mappings.insert(
            GestureFeedbackKey {
                gesture: GestureFeedbackType::Any,
                event: GestureFeedbackEvent::Error,
            },
            "error".to_string(),
        );
    }

    /// Get statistics
    pub fn stats(&self) -> &HapticStats {
        &self.stats
    }

    /// Get device info
    pub fn device(&self) -> &HapticDevice {
        &self.device
    }

    /// Set device
    pub fn set_device(&mut self, device: HapticDevice) {
        self.device = device;
    }

    /// Get pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&HapticPattern> {
        self.patterns.get(name)
    }

    /// List all pattern names
    pub fn pattern_names(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for HapticFeedback {
    fn default() -> Self {
        Self::new()
    }
}

impl HapticDevice {
    /// Create simulated device (no actual haptics)
    pub fn simulated() -> Self {
        Self {
            device_type: HapticDeviceType::Simulated,
            available: true,
            max_intensity: 1.0,
            min_pulse_ms: 5,
            supports_waveforms: true,
            actuator_count: 1,
        }
    }

    /// Create LRA device
    pub fn lra() -> Self {
        Self {
            device_type: HapticDeviceType::Lra,
            available: true,
            max_intensity: 1.0,
            min_pulse_ms: 5,
            supports_waveforms: true,
            actuator_count: 1,
        }
    }

    /// Create ERM device
    pub fn erm() -> Self {
        Self {
            device_type: HapticDeviceType::Erm,
            available: true,
            max_intensity: 1.0,
            min_pulse_ms: 20,
            supports_waveforms: false,
            actuator_count: 1,
        }
    }
}

/// Haptic output command
#[derive(Debug, Clone)]
pub enum HapticOutput {
    /// Vibrate at intensity
    Vibrate {
        intensity: f32,
        frequency_hz: Option<f32>,
    },
    /// Stop vibration
    Stop,
    /// Idle (no change)
    Idle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haptic_feedback_creation() {
        let feedback = HapticFeedback::new();
        assert!(feedback.is_available());
    }

    #[test]
    fn test_default_patterns() {
        let feedback = HapticFeedback::new();
        assert!(feedback.get_pattern("click").is_some());
        assert!(feedback.get_pattern("error").is_some());
        assert!(feedback.get_pattern("success").is_some());
    }

    #[test]
    fn test_config_enable_disable() {
        let mut feedback = HapticFeedback::new();
        
        let mut config = HapticConfig::default();
        config.enabled = false;
        feedback.set_config(config);
        
        assert!(!feedback.is_available());
    }

    #[test]
    fn test_on_gesture() {
        let mut feedback = HapticFeedback::new();
        feedback.on_gesture(
            GestureFeedbackType::Pinch,
            GestureFeedbackEvent::Recognized,
            0.9,
        );
        
        assert_eq!(feedback.stats.effects_played, 1);
    }

    #[test]
    fn test_play_pattern() {
        let mut feedback = HapticFeedback::new();
        feedback.play_pattern("click", 1.0);
        
        assert!(feedback.current_effect.is_some());
    }

    #[test]
    fn test_rate_limiting() {
        let mut feedback = HapticFeedback::new();
        
        feedback.play_pattern("click", 1.0);
        feedback.play_pattern("click", 1.0); // Should be dropped
        
        assert_eq!(feedback.stats.effects_dropped, 1);
    }

    #[test]
    fn test_device_types() {
        let lra = HapticDevice::lra();
        assert_eq!(lra.device_type, HapticDeviceType::Lra);
        
        let erm = HapticDevice::erm();
        assert_eq!(erm.device_type, HapticDeviceType::Erm);
    }

    #[test]
    fn test_register_pattern() {
        let mut feedback = HapticFeedback::new();
        
        let pattern = HapticPattern {
            name: "custom".to_string(),
            description: "Custom pattern".to_string(),
            elements: vec![],
            duration_ms: 100,
            default_intensity: 0.5,
        };
        
        feedback.register_pattern(pattern);
        assert!(feedback.get_pattern("custom").is_some());
    }

    #[test]
    fn test_update_cycle() {
        let mut feedback = HapticFeedback::new();
        feedback.play_pattern("click", 1.0);
        
        let output = feedback.update();
        assert!(output.is_some());
    }

    #[test]
    fn test_pattern_names() {
        let feedback = HapticFeedback::new();
        let names = feedback.pattern_names();
        assert!(names.len() >= 5);
    }
}
