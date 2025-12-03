//! Multimodal Sense - Input Processing for Oracle Veil
//!
//! This module handles input from multiple modalities:
//! - Voice (primary): Whisper transcription + MFCC analysis
//! - Gaze (secondary): Eye tracking for context
//! - Gesture (tertiary): IMU-based gesture recognition
//!
//! All input flows through this module before reaching the Oracle for mediation.
//!
//! ## Power-Aware Sensing (v1.1)
//!
//! The sense module integrates with PowerManager to optimize battery usage:
//! - Adaptive polling rates based on power profile
//! - Doze mode: minimal sensing (wake word + peripheral gaze only)
//! - Frame dropping in low-power modes

use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use crate::hardware::power::{PowerProfile, SensorPollingConfig, GazeState};

// ============================================================================
// MULTIMODAL SENSE
// ============================================================================

/// Multimodal input processor that fuses voice, gaze, and gesture
pub struct MultimodalSense {
    /// Voice processor
    voice: Arc<Mutex<VoiceProcessor>>,
    
    /// Gaze tracker
    gaze: Arc<Mutex<GazeTracker>>,
    
    /// Gesture detector
    gesture: Arc<Mutex<GestureDetector>>,
    
    /// Fused input channel
    output_tx: mpsc::Sender<FusedInput>,
    
    /// Configuration
    config: SenseConfig,
    
    /// Current state
    state: Arc<RwLock<SenseState>>,
    
    /// Power-aware polling config
    power_config: Arc<RwLock<SensorPollingConfig>>,
    
    /// Frame counter for power-aware frame dropping
    frame_counter: Arc<std::sync::atomic::AtomicU64>,
}

/// Configuration for multimodal sensing
#[derive(Debug, Clone)]
pub struct SenseConfig {
    /// Enable voice input
    pub voice_enabled: bool,
    
    /// Enable gaze tracking
    pub gaze_enabled: bool,
    
    /// Enable gesture detection
    pub gesture_enabled: bool,
    
    /// Voice activation keyword (e.g., "Hey Kāraṇa")
    pub wake_word: Option<String>,
    
    /// Gaze dwell time threshold (ms)
    pub gaze_dwell_threshold_ms: u64,
    
    /// Minimum voice confidence threshold
    pub voice_confidence_threshold: f32,
    
    /// Enable continuous listening (vs wake word activation)
    pub continuous_listening: bool,
    
    /// Power-aware: Max frames to process per second
    pub max_fps: u8,
    
    /// Power-aware: Downsample factor for camera frames
    pub downsample_factor: u8,
    
    /// Power-aware: Enable wake-word-only mode in doze
    pub doze_wake_word_only: bool,
}

impl Default for SenseConfig {
    fn default() -> Self {
        Self {
            voice_enabled: true,
            gaze_enabled: true,
            gesture_enabled: true,
            wake_word: Some("hey karana".to_string()),
            gaze_dwell_threshold_ms: 500,
            voice_confidence_threshold: 0.6,
            continuous_listening: false,
            max_fps: 30,
            downsample_factor: 1,
            doze_wake_word_only: true,
        }
    }
}

impl SenseConfig {
    /// Create power-optimized config for a given power profile
    pub fn for_power_profile(profile: PowerProfile) -> Self {
        let mut config = Self::default();
        
        match profile {
            PowerProfile::Performance => {
                config.max_fps = 60;
                config.downsample_factor = 1;
                config.continuous_listening = true;
            }
            PowerProfile::Balanced => {
                config.max_fps = 30;
                config.downsample_factor = 1;
                config.continuous_listening = false;
            }
            PowerProfile::LowPower => {
                config.max_fps = 15;
                config.downsample_factor = 2;
                config.gaze_enabled = false; // Disable full gaze tracking
            }
            PowerProfile::Critical => {
                config.max_fps = 5;
                config.downsample_factor = 4;
                config.gaze_enabled = false;
                config.gesture_enabled = false;
            }
            PowerProfile::Doze => {
                config.max_fps = 0; // No active processing
                config.gaze_enabled = false;
                config.gesture_enabled = false;
                config.doze_wake_word_only = true;
            }
        }
        
        config
    }
}

/// Current state of the sensing system
#[derive(Debug, Clone, Default)]
pub struct SenseState {
    /// Is voice currently active
    pub voice_active: bool,
    
    /// Current gaze target
    pub gaze_target: Option<GazeTarget>,
    
    /// Last detected gesture
    pub last_gesture: Option<GestureType>,
    
    /// Wake word detected timestamp
    pub wake_detected_at: Option<u64>,
    
    /// Is in listening mode
    pub listening: bool,
}

/// Fused input from all modalities
#[derive(Debug, Clone)]
pub struct FusedInput {
    /// Primary content (usually transcribed voice)
    pub content: String,
    
    /// Input source
    pub source: InputModality,
    
    /// Confidence score
    pub confidence: f32,
    
    /// Gaze context (what user is looking at)
    pub gaze_context: Option<GazeTarget>,
    
    /// Gesture context (if gesture triggered this)
    pub gesture_context: Option<GestureType>,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Audio features for emotion/urgency detection
    pub audio_features: Option<AudioFeatures>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputModality {
    Voice,
    Gaze,
    Gesture,
    Combined,
}

// ============================================================================
// VOICE PROCESSING
// ============================================================================

/// Voice input processor using Whisper
pub struct VoiceProcessor {
    /// Audio buffer
    buffer: Vec<f32>,
    
    /// Sample rate
    sample_rate: u32,
    
    /// Is currently recording
    recording: bool,
    
    /// Voice activity detection threshold
    vad_threshold: f32,
    
    /// Silence duration to trigger end of utterance
    silence_duration: Duration,
    
    /// Last voice activity timestamp
    last_voice_activity: Instant,
}

impl VoiceProcessor {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            buffer: Vec::new(),
            sample_rate,
            recording: false,
            vad_threshold: 0.02,
            silence_duration: Duration::from_millis(800),
            last_voice_activity: Instant::now(),
        }
    }
    
    /// Process incoming audio samples
    pub fn process_audio(&mut self, samples: &[f32]) -> Option<VoiceEvent> {
        // Calculate RMS for voice activity detection
        let rms = self.calculate_rms(samples);
        
        if rms > self.vad_threshold {
            self.last_voice_activity = Instant::now();
            
            if !self.recording {
                self.recording = true;
                self.buffer.clear();
            }
            
            self.buffer.extend_from_slice(samples);
            None
        } else if self.recording {
            // Check if silence duration exceeded
            if self.last_voice_activity.elapsed() > self.silence_duration {
                self.recording = false;
                let audio = std::mem::take(&mut self.buffer);
                Some(VoiceEvent::UtteranceComplete { audio })
            } else {
                self.buffer.extend_from_slice(samples);
                None
            }
        } else {
            None
        }
    }
    
    /// Calculate RMS of audio samples
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }
    
    /// Extract MFCC features for emotion/intent analysis
    pub fn extract_mfcc(&self, audio: &[f32]) -> AudioFeatures {
        // Simplified MFCC extraction
        // In production, use a proper DSP library
        
        let energy = self.calculate_rms(audio);
        let pitch = self.estimate_pitch(audio);
        let tempo = self.estimate_tempo(audio);
        
        AudioFeatures {
            energy,
            pitch,
            tempo,
            mfcc_coefficients: vec![0.0; 13], // Placeholder
        }
    }
    
    /// Estimate fundamental frequency (pitch)
    fn estimate_pitch(&self, audio: &[f32]) -> f32 {
        // Simplified autocorrelation-based pitch detection
        if audio.len() < 512 {
            return 0.0;
        }
        
        // Look for periodicity in typical voice range (80-400 Hz)
        let min_period = (self.sample_rate as f32 / 400.0) as usize;
        let max_period = (self.sample_rate as f32 / 80.0) as usize;
        
        let mut best_correlation = 0.0f32;
        let mut best_period = 0;
        
        for period in min_period..max_period.min(audio.len() / 2) {
            let mut correlation = 0.0f32;
            for i in 0..(audio.len() - period) {
                correlation += audio[i] * audio[i + period];
            }
            correlation /= (audio.len() - period) as f32;
            
            if correlation > best_correlation {
                best_correlation = correlation;
                best_period = period;
            }
        }
        
        if best_period > 0 {
            self.sample_rate as f32 / best_period as f32
        } else {
            0.0
        }
    }
    
    /// Estimate speaking tempo (syllables per second estimate)
    fn estimate_tempo(&self, audio: &[f32]) -> f32 {
        // Count energy peaks as proxy for syllables
        let window_size = (self.sample_rate as usize) / 20; // 50ms windows
        let mut peak_count = 0;
        let mut last_was_peak = false;
        
        for chunk in audio.chunks(window_size) {
            let energy = self.calculate_rms(chunk);
            let is_peak = energy > self.vad_threshold * 1.5;
            
            if is_peak && !last_was_peak {
                peak_count += 1;
            }
            last_was_peak = is_peak;
        }
        
        let duration_secs = audio.len() as f32 / self.sample_rate as f32;
        if duration_secs > 0.0 {
            peak_count as f32 / duration_secs
        } else {
            0.0
        }
    }
    
    /// Check for wake word in transcription
    pub fn check_wake_word(&self, transcription: &str, wake_word: &str) -> bool {
        let transcription_lower = transcription.to_lowercase();
        let wake_lower = wake_word.to_lowercase();
        
        transcription_lower.contains(&wake_lower) ||
        // Also check for common mishearings
        transcription_lower.contains("hey corona") ||
        transcription_lower.contains("hey kirana") ||
        transcription_lower.contains("karana")
    }
}

#[derive(Debug, Clone)]
pub enum VoiceEvent {
    UtteranceComplete { audio: Vec<f32> },
    WakeWordDetected,
    Silence,
}

#[derive(Debug, Clone, Default)]
pub struct AudioFeatures {
    /// Energy level (RMS)
    pub energy: f32,
    
    /// Estimated pitch (Hz)
    pub pitch: f32,
    
    /// Speaking tempo (syllables/sec estimate)
    pub tempo: f32,
    
    /// MFCC coefficients
    pub mfcc_coefficients: Vec<f32>,
}

// ============================================================================
// GAZE TRACKING
// ============================================================================

/// Gaze tracking and fixation detection
pub struct GazeTracker {
    /// Current gaze point (normalized 0-1)
    current_point: Option<(f32, f32)>,
    
    /// Gaze history for smoothing
    history: Vec<GazePoint>,
    
    /// Current fixation
    current_fixation: Option<Fixation>,
    
    /// Dwell threshold (ms)
    dwell_threshold: u64,
    
    /// Movement threshold for fixation detection
    movement_threshold: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GazePoint {
    pub x: f32,
    pub y: f32,
    pub timestamp: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct Fixation {
    pub center: (f32, f32),
    pub start_time: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GazeTarget {
    /// What the user is looking at
    pub target_type: GazeTargetType,
    
    /// Screen position (normalized)
    pub position: (f32, f32),
    
    /// How long they've been looking
    pub dwell_ms: u64,
    
    /// Confidence in target identification
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GazeTargetType {
    /// Looking at UI element
    UIElement { element_id: u32 },
    
    /// Looking at AR object
    ARObject { object_id: u32 },
    
    /// Looking at real-world object (via camera)
    RealWorldObject,
    
    /// Looking at text
    Text,
    
    /// General area
    Area,
    
    /// Unknown/unfocused
    Unknown,
}

impl GazeTracker {
    pub fn new(dwell_threshold_ms: u64) -> Self {
        Self {
            current_point: None,
            history: Vec::with_capacity(30),
            current_fixation: None,
            dwell_threshold: dwell_threshold_ms,
            movement_threshold: 0.05, // 5% of screen
        }
    }
    
    /// Update gaze with new point
    pub fn update(&mut self, point: GazePoint) -> Option<GazeEvent> {
        // Add to history
        self.history.push(point);
        if self.history.len() > 30 {
            self.history.remove(0);
        }
        
        // Smooth current point
        let smoothed = self.smooth_gaze();
        self.current_point = Some(smoothed);
        
        // Check for fixation
        self.detect_fixation(point.timestamp)
    }
    
    /// Apply smoothing to gaze points
    fn smooth_gaze(&self) -> (f32, f32) {
        if self.history.is_empty() {
            return (0.5, 0.5);
        }
        
        // Weighted average favoring recent points
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut sum_weight = 0.0f32;
        
        for (i, point) in self.history.iter().enumerate() {
            let weight = (i + 1) as f32 * point.confidence;
            sum_x += point.x * weight;
            sum_y += point.y * weight;
            sum_weight += weight;
        }
        
        if sum_weight > 0.0 {
            (sum_x / sum_weight, sum_y / sum_weight)
        } else {
            (0.5, 0.5)
        }
    }
    
    /// Detect fixation (stable gaze)
    fn detect_fixation(&mut self, current_time: u64) -> Option<GazeEvent> {
        let current = self.current_point?;
        
        if let Some(ref mut fixation) = self.current_fixation {
            // Check if still within fixation area
            let distance = ((current.0 - fixation.center.0).powi(2) + 
                           (current.1 - fixation.center.1).powi(2)).sqrt();
            
            if distance < self.movement_threshold {
                // Update fixation duration
                fixation.duration_ms = current_time.saturating_sub(fixation.start_time);
                
                // Check for dwell threshold
                if fixation.duration_ms >= self.dwell_threshold {
                    return Some(GazeEvent::Dwell {
                        position: fixation.center,
                        duration_ms: fixation.duration_ms,
                    });
                }
            } else {
                // Fixation broken, start new one
                let old_fixation = self.current_fixation.take();
                self.current_fixation = Some(Fixation {
                    center: current,
                    start_time: current_time,
                    duration_ms: 0,
                });
                
                if let Some(f) = old_fixation {
                    if f.duration_ms >= self.dwell_threshold {
                        return Some(GazeEvent::FixationEnd {
                            position: f.center,
                            duration_ms: f.duration_ms,
                        });
                    }
                }
            }
        } else {
            // Start new fixation
            self.current_fixation = Some(Fixation {
                center: current,
                start_time: current_time,
                duration_ms: 0,
            });
        }
        
        None
    }
    
    /// Get current gaze target with context
    pub fn get_target(&self) -> Option<GazeTarget> {
        let (x, y) = self.current_point?;
        let fixation = self.current_fixation.as_ref()?;
        
        Some(GazeTarget {
            target_type: GazeTargetType::Area, // Would be enhanced with scene understanding
            position: (x, y),
            dwell_ms: fixation.duration_ms,
            confidence: 0.8,
        })
    }
}

#[derive(Debug, Clone)]
pub enum GazeEvent {
    /// User dwelling on a point
    Dwell { position: (f32, f32), duration_ms: u64 },
    
    /// Fixation ended
    FixationEnd { position: (f32, f32), duration_ms: u64 },
    
    /// Quick glance detected
    Glance { from: (f32, f32), to: (f32, f32) },
}

// ============================================================================
// GESTURE DETECTION
// ============================================================================

/// IMU-based gesture detection
pub struct GestureDetector {
    /// Accelerometer history
    accel_history: Vec<AccelSample>,
    
    /// Gyroscope history
    gyro_history: Vec<GyroSample>,
    
    /// Currently detected gesture
    current_gesture: Option<GestureType>,
    
    /// Gesture templates for matching
    templates: Vec<GestureTemplate>,
}

#[derive(Debug, Clone, Copy)]
pub struct AccelSample {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct GyroSample {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    /// Nod (up-down)
    Nod,
    
    /// Shake (left-right)
    Shake,
    
    /// Tilt left
    TiltLeft,
    
    /// Tilt right
    TiltRight,
    
    /// Look up
    LookUp,
    
    /// Look down
    LookDown,
    
    /// Double tap (detected via accelerometer spike)
    DoubleTap,
    
    /// Long press (detected via steady state)
    LongPress,
}

struct GestureTemplate {
    gesture_type: GestureType,
    pattern: Vec<(f32, f32, f32)>, // Simplified pattern
    threshold: f32,
}

impl GestureDetector {
    pub fn new() -> Self {
        Self {
            accel_history: Vec::with_capacity(60),
            gyro_history: Vec::with_capacity(60),
            current_gesture: None,
            templates: Self::default_templates(),
        }
    }
    
    fn default_templates() -> Vec<GestureTemplate> {
        vec![
            GestureTemplate {
                gesture_type: GestureType::Nod,
                pattern: vec![(0.0, 0.5, 0.0), (0.0, -0.5, 0.0), (0.0, 0.5, 0.0)],
                threshold: 0.3,
            },
            GestureTemplate {
                gesture_type: GestureType::Shake,
                pattern: vec![(0.5, 0.0, 0.0), (-0.5, 0.0, 0.0), (0.5, 0.0, 0.0)],
                threshold: 0.3,
            },
        ]
    }
    
    /// Update with new accelerometer sample
    pub fn update_accel(&mut self, sample: AccelSample) -> Option<GestureType> {
        self.accel_history.push(sample);
        if self.accel_history.len() > 60 {
            self.accel_history.remove(0);
        }
        
        // Simple gesture detection based on acceleration magnitude
        self.detect_gesture()
    }
    
    /// Update with new gyroscope sample
    pub fn update_gyro(&mut self, sample: GyroSample) {
        self.gyro_history.push(sample);
        if self.gyro_history.len() > 60 {
            self.gyro_history.remove(0);
        }
    }
    
    /// Detect gesture from IMU data
    fn detect_gesture(&mut self) -> Option<GestureType> {
        if self.accel_history.len() < 10 {
            return None;
        }
        
        let recent: Vec<_> = self.accel_history.iter().rev().take(10).collect();
        
        // Detect nod (Y-axis movement)
        let y_variance: f32 = recent.iter().map(|s| s.y * s.y).sum::<f32>() / 10.0;
        if y_variance > 0.3 {
            return Some(GestureType::Nod);
        }
        
        // Detect shake (X-axis movement)
        let x_variance: f32 = recent.iter().map(|s| s.x * s.x).sum::<f32>() / 10.0;
        if x_variance > 0.3 {
            return Some(GestureType::Shake);
        }
        
        // Detect double tap (Z-axis spike)
        let z_recent: Vec<f32> = recent.iter().map(|s| s.z).collect();
        let z_max = z_recent.iter().cloned().fold(f32::MIN, f32::max);
        if z_max > 2.0 {
            return Some(GestureType::DoubleTap);
        }
        
        None
    }
    
    /// Get current tilt angle
    pub fn get_tilt(&self) -> Option<(f32, f32)> {
        let sample = self.accel_history.last()?;
        
        // Calculate pitch and roll from accelerometer
        let pitch = sample.x.atan2((sample.y.powi(2) + sample.z.powi(2)).sqrt());
        let roll = sample.y.atan2((sample.x.powi(2) + sample.z.powi(2)).sqrt());
        
        Some((pitch, roll))
    }
}

// ============================================================================
// MULTIMODAL SENSE IMPLEMENTATION
// ============================================================================

impl MultimodalSense {
    /// Create new multimodal sense with default config
    pub fn new(output_tx: mpsc::Sender<FusedInput>) -> Self {
        Self::with_config(output_tx, SenseConfig::default())
    }
    
    /// Create with custom config
    pub fn with_config(output_tx: mpsc::Sender<FusedInput>, config: SenseConfig) -> Self {
        let power_config = SensorPollingConfig::for_profile(PowerProfile::Balanced);
        
        Self {
            voice: Arc::new(Mutex::new(VoiceProcessor::new(power_config.mic_sample_rate))),
            gaze: Arc::new(Mutex::new(GazeTracker::new(config.gaze_dwell_threshold_ms))),
            gesture: Arc::new(Mutex::new(GestureDetector::new())),
            output_tx,
            config,
            state: Arc::new(RwLock::new(SenseState::default())),
            power_config: Arc::new(RwLock::new(power_config)),
            frame_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Create with specific power profile
    pub fn with_power_profile(output_tx: mpsc::Sender<FusedInput>, profile: PowerProfile) -> Self {
        let config = SenseConfig::for_power_profile(profile);
        let power_config = SensorPollingConfig::for_profile(profile);
        
        Self {
            voice: Arc::new(Mutex::new(VoiceProcessor::new(power_config.mic_sample_rate))),
            gaze: Arc::new(Mutex::new(GazeTracker::new(config.gaze_dwell_threshold_ms))),
            gesture: Arc::new(Mutex::new(GestureDetector::new())),
            output_tx,
            config,
            state: Arc::new(RwLock::new(SenseState::default())),
            power_config: Arc::new(RwLock::new(power_config)),
            frame_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Update power profile (called by PowerManager on profile change)
    pub async fn set_power_profile(&self, profile: PowerProfile) {
        let new_config = SensorPollingConfig::for_profile(profile);
        let camera_fps = new_config.camera_fps;
        let eye_hz = new_config.eye_tracking_hz;
        
        let mut power_config = self.power_config.write().await;
        *power_config = new_config;
        
        log::info!("[SENSE] Power profile updated: {:?} (camera={}fps, eye={}Hz)", 
            profile, camera_fps, eye_hz);
    }
    
    /// Check if frame should be processed (power-aware frame dropping)
    pub fn should_process_frame(&self) -> bool {
        use std::sync::atomic::Ordering;
        let frame_idx = self.frame_counter.fetch_add(1, Ordering::SeqCst);
        
        // Drop frames based on config max_fps
        let max_fps = self.config.max_fps;
        if max_fps == 0 {
            return false; // Doze mode - no processing
        }
        
        // Assuming 60fps input, calculate drop ratio
        let keep_ratio = max_fps as u64 * 100 / 60;
        (frame_idx * 100 % 100) < keep_ratio
    }
    
    /// Get current gaze state for power management
    pub async fn get_gaze_state(&self) -> GazeState {
        let state = self.state.read().await;
        
        // Convert gaze target to power GazeState
        match &state.gaze_target {
            Some(target) if target.dwell_ms > 500 => GazeState::Active,
            Some(_) => GazeState::Peripheral,
            None if state.listening => GazeState::Peripheral,
            None => GazeState::Away,
        }
    }
    
    /// Process voice audio
    pub async fn process_voice(&self, audio: Vec<f32>, transcription: &str) -> Result<()> {
        // In doze mode, only process wake word
        if self.config.doze_wake_word_only {
            let power_config = self.power_config.read().await;
            if power_config.camera_fps == 0 {
                // Doze mode - only check wake word
                if let Some(wake_word) = &self.config.wake_word {
                    let voice = self.voice.lock().await;
                    if voice.check_wake_word(transcription, wake_word) {
                        log::info!("[SENSE] Wake word detected in doze mode!");
                        let mut state = self.state.write().await;
                        state.wake_detected_at = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        state.listening = true;
                    }
                }
                return Ok(());
            }
        }
        
        let mut voice = self.voice.lock().await;
        let features = voice.extract_mfcc(&audio);
        
        // Check wake word if not in continuous mode
        let mut state = self.state.write().await;
        
        if !self.config.continuous_listening {
            if let Some(wake_word) = &self.config.wake_word {
                if voice.check_wake_word(transcription, wake_word) {
                    state.wake_detected_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );
                    state.listening = true;
                    return Ok(());
                }
            }
            
            // If not listening and no wake word, ignore
            if !state.listening {
                return Ok(());
            }
        }
        
        // Get gaze context
        let gaze_context = {
            let gaze = self.gaze.lock().await;
            gaze.get_target()
        };
        
        // Get gesture context
        let gesture_context = {
            let gesture = self.gesture.lock().await;
            gesture.current_gesture
        };
        
        // Create fused input
        let fused = FusedInput {
            content: transcription.to_string(),
            source: InputModality::Voice,
            confidence: 0.9, // Would come from Whisper
            gaze_context,
            gesture_context,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            audio_features: Some(features),
        };
        
        // Send to output
        self.output_tx.send(fused).await
            .map_err(|_| anyhow!("Failed to send fused input"))?;
        
        Ok(())
    }
    
    /// Process gaze point
    pub async fn process_gaze(&self, x: f32, y: f32, confidence: f32) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let point = GazePoint { x, y, timestamp, confidence };
        
        let event = {
            let mut gaze = self.gaze.lock().await;
            gaze.update(point)
        };
        
        // If dwell detected, could trigger gaze-based input
        if let Some(GazeEvent::Dwell { position, duration_ms }) = event {
            log::info!("[SENSE] Gaze dwell at ({:.2}, {:.2}) for {}ms", 
                position.0, position.1, duration_ms);
            
            // Update state
            let mut state = self.state.write().await;
            state.gaze_target = Some(GazeTarget {
                target_type: GazeTargetType::Area,
                position,
                dwell_ms: duration_ms,
                confidence,
            });
        }
        
        Ok(())
    }
    
    /// Process IMU data
    pub async fn process_imu(&self, accel: (f32, f32, f32), gyro: (f32, f32, f32)) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let accel_sample = AccelSample {
            x: accel.0, y: accel.1, z: accel.2, timestamp,
        };
        let gyro_sample = GyroSample {
            x: gyro.0, y: gyro.1, z: gyro.2, timestamp,
        };
        
        let gesture = {
            let mut detector = self.gesture.lock().await;
            detector.update_gyro(gyro_sample);
            detector.update_accel(accel_sample)
        };
        
        if let Some(gesture_type) = gesture {
            log::info!("[SENSE] Gesture detected: {:?}", gesture_type);
            
            // Update state
            let mut state = self.state.write().await;
            state.last_gesture = Some(gesture_type);
            
            // Nod = confirm, Shake = cancel
            match gesture_type {
                GestureType::Nod => {
                    // Could send confirmation input
                    let fused = FusedInput {
                        content: "confirm".to_string(),
                        source: InputModality::Gesture,
                        confidence: 0.8,
                        gaze_context: None,
                        gesture_context: Some(gesture_type),
                        timestamp,
                        audio_features: None,
                    };
                    let _ = self.output_tx.send(fused).await;
                }
                GestureType::Shake => {
                    // Could send cancel input
                    let fused = FusedInput {
                        content: "cancel".to_string(),
                        source: InputModality::Gesture,
                        confidence: 0.8,
                        gaze_context: None,
                        gesture_context: Some(gesture_type),
                        timestamp,
                        audio_features: None,
                    };
                    let _ = self.output_tx.send(fused).await;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Get current sense state
    pub async fn get_state(&self) -> SenseState {
        self.state.read().await.clone()
    }
    
    /// Enable/disable listening mode
    pub async fn set_listening(&self, listening: bool) {
        let mut state = self.state.write().await;
        state.listening = listening;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_rms() {
        let processor = VoiceProcessor::new(16000);
        
        // Silent audio
        let silent = vec![0.0f32; 100];
        assert_eq!(processor.calculate_rms(&silent), 0.0);
        
        // Some noise
        let noise: Vec<f32> = (0..100).map(|i| (i as f32 / 10.0).sin() * 0.1).collect();
        let rms = processor.calculate_rms(&noise);
        assert!(rms > 0.0 && rms < 0.1);
    }
    
    #[test]
    fn test_wake_word_detection() {
        let processor = VoiceProcessor::new(16000);
        
        assert!(processor.check_wake_word("hey karana what's up", "hey karana"));
        assert!(processor.check_wake_word("Hey Karana, check balance", "hey karana"));
        assert!(!processor.check_wake_word("hello there", "hey karana"));
    }
    
    #[test]
    fn test_gaze_fixation() {
        let mut tracker = GazeTracker::new(500);
        
        // Add stable gaze points
        for i in 0..20 {
            let point = GazePoint {
                x: 0.5,
                y: 0.5,
                timestamp: i * 50,
                confidence: 0.9,
            };
            tracker.update(point);
        }
        
        let target = tracker.get_target();
        assert!(target.is_some());
    }
    
    #[test]
    fn test_gesture_detection() {
        let mut detector = GestureDetector::new();
        
        // Simulate tap
        for i in 0..5 {
            let sample = AccelSample {
                x: 0.0,
                y: 0.0,
                z: if i == 2 { 3.0 } else { 1.0 },
                timestamp: i * 10,
            };
            let _ = detector.update_accel(sample);
        }
        
        // Would need more sophisticated test for actual gesture detection
    }
    
    #[test]
    fn test_power_aware_config() {
        // Test that power profiles create appropriate configs
        let perf_config = SenseConfig::for_power_profile(PowerProfile::Performance);
        assert_eq!(perf_config.max_fps, 60);
        assert!(perf_config.continuous_listening);
        
        let doze_config = SenseConfig::for_power_profile(PowerProfile::Doze);
        assert_eq!(doze_config.max_fps, 0);
        assert!(!doze_config.gaze_enabled);
        assert!(doze_config.doze_wake_word_only);
        
        let low_config = SenseConfig::for_power_profile(PowerProfile::LowPower);
        assert_eq!(low_config.max_fps, 15);
        assert_eq!(low_config.downsample_factor, 2);
    }
    
    #[test]
    fn test_frame_dropping() {
        let (tx, _rx) = mpsc::channel(10);
        
        // Test performance mode - should process most frames
        let mut config = SenseConfig::default();
        config.max_fps = 60;
        let sense_perf = MultimodalSense::with_config(tx.clone(), config);
        
        let processed: Vec<bool> = (0..60).map(|_| sense_perf.should_process_frame()).collect();
        let processed_count = processed.iter().filter(|&&x| x).count();
        assert!(processed_count >= 55); // Most frames should be processed
        
        // Test doze mode - should process no frames
        let (tx2, _rx2) = mpsc::channel(10);
        let mut config_doze = SenseConfig::default();
        config_doze.max_fps = 0;
        let sense_doze = MultimodalSense::with_config(tx2, config_doze);
        
        let doze_processed: Vec<bool> = (0..60).map(|_| sense_doze.should_process_frame()).collect();
        let doze_count = doze_processed.iter().filter(|&&x| x).count();
        assert_eq!(doze_count, 0); // No frames in doze
    }
}
