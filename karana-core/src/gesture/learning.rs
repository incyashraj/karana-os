//! Gesture Learning and Customization for Kāraṇa OS
//!
//! Allows users to define and train custom gestures.
//! Uses pattern matching and machine learning for recognition.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{Handedness, Finger, HandLandmark};

/// Custom gesture learning system
pub struct GestureLearner {
    /// Learned gestures
    gestures: HashMap<String, LearnedGesture>,
    /// Training mode state
    training: Option<TrainingSession>,
    /// Configuration
    config: LearningConfig,
    /// Recognition state
    recognition_buffer: RecognitionBuffer,
    /// Statistics
    stats: LearningStats,
}

/// A learned custom gesture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedGesture {
    /// Gesture name/identifier
    pub name: String,
    /// Description
    pub description: String,
    /// Training samples
    pub samples: Vec<GestureSample>,
    /// Template after DTW processing
    pub template: Option<GestureTemplate>,
    /// Associated action
    pub action: CustomGestureAction,
    /// Required hand
    pub required_hand: Option<Handedness>,
    /// Minimum confidence for recognition
    pub min_confidence: f32,
    /// Creation timestamp
    pub created_at: u64,
    /// Usage count
    pub usage_count: u64,
    /// Average recognition confidence
    pub avg_confidence: f32,
}

/// Single training sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureSample {
    /// Frames of hand positions
    pub frames: Vec<HandFrame>,
    /// Sample quality score
    pub quality: f32,
    /// Duration
    pub duration_ms: u64,
    /// Timestamp recorded
    pub recorded_at: u64,
}

/// Single frame of hand data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandFrame {
    /// Landmark positions (21 points)
    pub landmarks: Vec<LandmarkPoint>,
    /// Finger states
    pub finger_states: [FingerFrame; 5],
    /// Palm position
    pub palm_position: Point3,
    /// Palm normal
    pub palm_normal: Point3,
    /// Timestamp offset from start
    pub time_offset_ms: u64,
}

/// Single landmark point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LandmarkPoint {
    /// Landmark index
    pub index: u8,
    /// Position relative to palm
    pub position: Point3,
}

/// 3D point for serialization
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, other: &Point3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Single finger frame
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FingerFrame {
    /// Finger type
    pub finger: u8,
    /// Curl amount (0-1)
    pub curl: f32,
    /// Spread angle
    pub spread: f32,
    /// Is extended
    pub extended: bool,
}

/// Gesture template for recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureTemplate {
    /// Normalized key frames
    pub key_frames: Vec<KeyFrame>,
    /// Velocity profile
    pub velocity_profile: Vec<f32>,
    /// Duration range (min, max)
    pub duration_range: (u64, u64),
    /// Key pose indices
    pub key_poses: Vec<usize>,
}

/// Key frame in template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFrame {
    /// Normalized time (0-1)
    pub time: f32,
    /// Finger curl values
    pub finger_curls: [f32; 5],
    /// Key landmark positions (normalized)
    pub landmarks: Vec<Point3>,
    /// Variance allowed
    pub variance: f32,
}

/// Custom gesture action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomGestureAction {
    /// System command
    SystemCommand(String),
    /// App launch
    LaunchApp(String),
    /// Keyboard shortcut
    KeyboardShortcut(Vec<String>),
    /// Voice command trigger
    VoiceCommand(String),
    /// Custom callback ID
    Callback(u64),
    /// Chain of actions
    ActionChain(Vec<CustomGestureAction>),
}

/// Training session state
pub struct TrainingSession {
    /// Gesture being trained
    gesture_name: String,
    /// Collected samples
    samples: Vec<GestureSample>,
    /// Current sample being recorded
    current_sample: Option<SampleRecording>,
    /// Required samples
    required_samples: usize,
    /// Session start time
    started_at: Instant,
}

/// Sample being recorded
struct SampleRecording {
    frames: Vec<HandFrame>,
    started_at: Instant,
}

/// Learning configuration
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Minimum samples required
    pub min_samples: usize,
    /// Maximum samples to keep
    pub max_samples: usize,
    /// Minimum sample quality
    pub min_quality: f32,
    /// DTW window size
    pub dtw_window: usize,
    /// Recognition threshold
    pub recognition_threshold: f32,
    /// Maximum gesture duration (ms)
    pub max_duration_ms: u64,
    /// Minimum gesture duration (ms)
    pub min_duration_ms: u64,
    /// Frame sample rate
    pub sample_rate_hz: u32,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            min_samples: 3,
            max_samples: 10,
            min_quality: 0.5,
            dtw_window: 10,
            recognition_threshold: 0.7,
            max_duration_ms: 3000,
            min_duration_ms: 200,
            sample_rate_hz: 30,
        }
    }
}

/// Recognition buffer for continuous matching
struct RecognitionBuffer {
    frames: VecDeque<HandFrame>,
    max_frames: usize,
}

impl RecognitionBuffer {
    fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_frames),
            max_frames,
        }
    }

    fn add_frame(&mut self, frame: HandFrame) {
        self.frames.push_back(frame);
        while self.frames.len() > self.max_frames {
            self.frames.pop_front();
        }
    }

    fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Learning statistics
#[derive(Debug, Default)]
pub struct LearningStats {
    /// Gestures learned
    pub gestures_learned: u64,
    /// Total training samples
    pub total_samples: u64,
    /// Recognition attempts
    pub recognition_attempts: u64,
    /// Successful recognitions
    pub successful_recognitions: u64,
    /// Average recognition confidence
    pub avg_recognition_confidence: f32,
}

/// Recognition result
#[derive(Debug, Clone)]
pub struct RecognitionResult {
    /// Recognized gesture name
    pub gesture_name: String,
    /// Confidence (0-1)
    pub confidence: f32,
    /// DTW distance
    pub dtw_distance: f32,
    /// Matched template
    pub matched: bool,
}

impl GestureLearner {
    /// Create new gesture learner
    pub fn new() -> Self {
        Self {
            gestures: HashMap::new(),
            training: None,
            config: LearningConfig::default(),
            recognition_buffer: RecognitionBuffer::new(90), // 3 seconds at 30fps
            stats: LearningStats::default(),
        }
    }

    /// Create with configuration
    pub fn with_config(config: LearningConfig) -> Self {
        let max_frames = (config.max_duration_ms * config.sample_rate_hz as u64 / 1000) as usize;
        Self {
            gestures: HashMap::new(),
            training: None,
            config,
            recognition_buffer: RecognitionBuffer::new(max_frames),
            stats: LearningStats::default(),
        }
    }

    /// Start training a new gesture
    pub fn start_training(&mut self, name: &str) -> Result<(), LearningError> {
        if self.training.is_some() {
            return Err(LearningError::AlreadyTraining);
        }

        self.training = Some(TrainingSession {
            gesture_name: name.to_string(),
            samples: Vec::new(),
            current_sample: None,
            required_samples: self.config.min_samples,
            started_at: Instant::now(),
        });

        Ok(())
    }

    /// Start recording a training sample
    pub fn start_sample(&mut self) -> Result<(), LearningError> {
        let training = self.training.as_mut()
            .ok_or(LearningError::NotTraining)?;

        if training.current_sample.is_some() {
            return Err(LearningError::SampleInProgress);
        }

        training.current_sample = Some(SampleRecording {
            frames: Vec::new(),
            started_at: Instant::now(),
        });

        Ok(())
    }

    /// Add frame to current training sample
    pub fn add_training_frame(&mut self, frame: HandFrame) -> Result<(), LearningError> {
        let training = self.training.as_mut()
            .ok_or(LearningError::NotTraining)?;

        let sample = training.current_sample.as_mut()
            .ok_or(LearningError::NoSampleInProgress)?;

        sample.frames.push(frame);

        // Check max duration
        if sample.started_at.elapsed().as_millis() > self.config.max_duration_ms as u128 {
            // Auto-complete
            self.complete_sample()?;
        }

        Ok(())
    }

    /// Complete current training sample
    pub fn complete_sample(&mut self) -> Result<SampleQuality, LearningError> {
        // First extract the sample frames to calculate quality
        let (sample_frames, duration_ms) = {
            let training = self.training.as_mut()
                .ok_or(LearningError::NotTraining)?;

            let sample = training.current_sample.take()
                .ok_or(LearningError::NoSampleInProgress)?;

            let duration_ms = sample.started_at.elapsed().as_millis() as u64;
            (sample.frames, duration_ms)
        };

        // Check minimum duration
        if duration_ms < self.config.min_duration_ms {
            return Err(LearningError::SampleTooShort);
        }

        // Calculate quality (now self is not borrowed mutably)
        let quality = self.calculate_sample_quality(&sample_frames);

        if quality < self.config.min_quality {
            return Ok(SampleQuality::Poor { quality });
        }

        // Create sample
        let gesture_sample = GestureSample {
            frames: sample_frames,
            quality,
            duration_ms,
            recorded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        // Now get training again to push the sample
        let training = self.training.as_mut()
            .ok_or(LearningError::NotTraining)?;
        
        training.samples.push(gesture_sample);
        self.stats.total_samples += 1;

        let samples_remaining = training.required_samples.saturating_sub(training.samples.len());

        Ok(SampleQuality::Good { 
            quality, 
            samples_remaining,
        })
    }

    /// Calculate sample quality
    fn calculate_sample_quality(&self, frames: &[HandFrame]) -> f32 {
        if frames.is_empty() {
            return 0.0;
        }

        // Check for:
        // 1. Consistent tracking
        // 2. Reasonable velocity
        // 3. No large jumps

        let mut quality: f32 = 1.0;

        // Check for jumps
        for window in frames.windows(2) {
            let dist = window[0].palm_position.distance(&window[1].palm_position);
            if dist > 0.1 {  // Large jump
                quality -= 0.2;
            }
        }

        // Check for movement (should have some motion)
        if frames.len() > 5 {
            let first = &frames[0].palm_position;
            let last = &frames[frames.len() - 1].palm_position;
            let total_motion = first.distance(last);
            if total_motion < 0.01 {
                quality -= 0.3;  // Too little motion
            }
        }

        quality.max(0.0)
    }

    /// Complete training and create gesture
    pub fn complete_training(&mut self, action: CustomGestureAction) -> Result<String, LearningError> {
        let training = self.training.take()
            .ok_or(LearningError::NotTraining)?;

        if training.samples.len() < self.config.min_samples {
            self.training = Some(training);
            return Err(LearningError::NotEnoughSamples);
        }

        // Build template from samples
        let template = self.build_template(&training.samples);

        let gesture = LearnedGesture {
            name: training.gesture_name.clone(),
            description: String::new(),
            samples: training.samples,
            template: Some(template),
            action,
            required_hand: None,
            min_confidence: self.config.recognition_threshold,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            usage_count: 0,
            avg_confidence: 0.0,
        };

        self.gestures.insert(gesture.name.clone(), gesture);
        self.stats.gestures_learned += 1;

        Ok(training.gesture_name)
    }

    /// Build template from training samples using DTW
    fn build_template(&self, samples: &[GestureSample]) -> GestureTemplate {
        // Find average duration
        let avg_duration: u64 = samples.iter().map(|s| s.duration_ms).sum::<u64>() / samples.len() as u64;
        let min_duration = samples.iter().map(|s| s.duration_ms).min().unwrap_or(0);
        let max_duration = samples.iter().map(|s| s.duration_ms).max().unwrap_or(0);

        // Normalize to key frames
        let num_key_frames = 10;
        let mut key_frames = Vec::with_capacity(num_key_frames);

        for i in 0..num_key_frames {
            let t = i as f32 / (num_key_frames - 1) as f32;
            
            // Average finger curls at this time point
            let mut avg_curls = [0.0f32; 5];
            let mut count = 0;

            for sample in samples {
                let frame_idx = (t * (sample.frames.len() - 1) as f32) as usize;
                if frame_idx < sample.frames.len() {
                    let frame = &sample.frames[frame_idx];
                    for (j, finger) in frame.finger_states.iter().enumerate() {
                        avg_curls[j] += finger.curl;
                    }
                    count += 1;
                }
            }

            if count > 0 {
                for curl in &mut avg_curls {
                    *curl /= count as f32;
                }
            }

            key_frames.push(KeyFrame {
                time: t,
                finger_curls: avg_curls,
                landmarks: Vec::new(),
                variance: 0.2,
            });
        }

        // Calculate velocity profile
        let velocity_profile: Vec<f32> = (0..num_key_frames - 1)
            .map(|i| {
                let mut avg_vel = 0.0;
                for sample in samples {
                    let idx1 = (i as f32 / num_key_frames as f32 * sample.frames.len() as f32) as usize;
                    let idx2 = ((i + 1) as f32 / num_key_frames as f32 * sample.frames.len() as f32) as usize;
                    if idx1 < sample.frames.len() && idx2 < sample.frames.len() {
                        avg_vel += sample.frames[idx1].palm_position
                            .distance(&sample.frames[idx2].palm_position);
                    }
                }
                avg_vel / samples.len() as f32
            })
            .collect();

        GestureTemplate {
            key_frames,
            velocity_profile,
            duration_range: (min_duration, max_duration),
            key_poses: vec![0, num_key_frames / 2, num_key_frames - 1],
        }
    }

    /// Cancel current training
    pub fn cancel_training(&mut self) {
        self.training = None;
    }

    /// Process frame for recognition
    pub fn process_frame(&mut self, frame: HandFrame) -> Option<RecognitionResult> {
        self.recognition_buffer.add_frame(frame);
        self.stats.recognition_attempts += 1;

        // Try to match against all gestures
        let mut best_match: Option<RecognitionResult> = None;

        for (name, gesture) in &self.gestures {
            if let Some(template) = &gesture.template {
                let result = self.match_template(name, template);
                
                if result.matched && result.confidence > gesture.min_confidence {
                    if best_match.as_ref().map(|b| result.confidence > b.confidence).unwrap_or(true) {
                        best_match = Some(result);
                    }
                }
            }
        }

        if best_match.is_some() {
            self.stats.successful_recognitions += 1;
            self.recognition_buffer.clear();
        }

        best_match
    }

    /// Match buffer against template using DTW
    fn match_template(&self, name: &str, template: &GestureTemplate) -> RecognitionResult {
        let frames: Vec<_> = self.recognition_buffer.frames.iter().collect();
        
        if frames.is_empty() {
            return RecognitionResult {
                gesture_name: name.to_string(),
                confidence: 0.0,
                dtw_distance: f32::MAX,
                matched: false,
            };
        }

        // Simple DTW implementation
        let n = frames.len();
        let m = template.key_frames.len();
        
        let mut dtw = vec![vec![f32::MAX; m + 1]; n + 1];
        dtw[0][0] = 0.0;

        for i in 1..=n {
            for j in 1..=m {
                let cost = self.frame_distance(&frames[i - 1], &template.key_frames[j - 1]);
                dtw[i][j] = cost + dtw[i - 1][j].min(dtw[i][j - 1]).min(dtw[i - 1][j - 1]);
            }
        }

        let dtw_distance = dtw[n][m];
        let max_distance = 5.0;  // Normalize
        let confidence = (1.0 - (dtw_distance / max_distance)).max(0.0).min(1.0);

        RecognitionResult {
            gesture_name: name.to_string(),
            confidence,
            dtw_distance,
            matched: confidence > 0.5,
        }
    }

    /// Calculate distance between frame and key frame
    fn frame_distance(&self, frame: &HandFrame, key_frame: &KeyFrame) -> f32 {
        let mut distance = 0.0;

        // Compare finger curls
        for (i, finger) in frame.finger_states.iter().enumerate() {
            let diff = (finger.curl - key_frame.finger_curls[i]).abs();
            distance += diff * diff;
        }

        (distance / 5.0).sqrt()
    }

    /// Delete a learned gesture
    pub fn delete_gesture(&mut self, name: &str) -> bool {
        self.gestures.remove(name).is_some()
    }

    /// Get learned gesture by name
    pub fn get_gesture(&self, name: &str) -> Option<&LearnedGesture> {
        self.gestures.get(name)
    }

    /// Get all gesture names
    pub fn gesture_names(&self) -> Vec<&str> {
        self.gestures.keys().map(|s| s.as_str()).collect()
    }

    /// Export gestures to bytes (for persistence)
    pub fn export_gestures(&self) -> Result<Vec<u8>, LearningError> {
        let gestures: Vec<&LearnedGesture> = self.gestures.values().collect();
        serde_json::to_vec(&gestures)
            .map_err(|_| LearningError::SerializationFailed)
    }

    /// Import gestures from bytes
    pub fn import_gestures(&mut self, data: &[u8]) -> Result<usize, LearningError> {
        let gestures: Vec<LearnedGesture> = serde_json::from_slice(data)
            .map_err(|_| LearningError::DeserializationFailed)?;
        
        let count = gestures.len();
        for gesture in gestures {
            self.gestures.insert(gesture.name.clone(), gesture);
        }

        Ok(count)
    }

    /// Get statistics
    pub fn stats(&self) -> &LearningStats {
        &self.stats
    }

    /// Is currently training
    pub fn is_training(&self) -> bool {
        self.training.is_some()
    }

    /// Get training progress
    pub fn training_progress(&self) -> Option<TrainingProgress> {
        let training = self.training.as_ref()?;
        Some(TrainingProgress {
            gesture_name: training.gesture_name.clone(),
            samples_collected: training.samples.len(),
            samples_required: training.required_samples,
            recording: training.current_sample.is_some(),
        })
    }
}

impl Default for GestureLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Training progress info
#[derive(Debug, Clone)]
pub struct TrainingProgress {
    pub gesture_name: String,
    pub samples_collected: usize,
    pub samples_required: usize,
    pub recording: bool,
}

/// Sample quality result
#[derive(Debug)]
pub enum SampleQuality {
    Good { quality: f32, samples_remaining: usize },
    Poor { quality: f32 },
}

/// Learning errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningError {
    AlreadyTraining,
    NotTraining,
    SampleInProgress,
    NoSampleInProgress,
    SampleTooShort,
    NotEnoughSamples,
    GestureNotFound,
    SerializationFailed,
    DeserializationFailed,
}

impl std::fmt::Display for LearningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyTraining => write!(f, "Already training a gesture"),
            Self::NotTraining => write!(f, "Not in training mode"),
            Self::SampleInProgress => write!(f, "Sample recording already in progress"),
            Self::NoSampleInProgress => write!(f, "No sample recording in progress"),
            Self::SampleTooShort => write!(f, "Sample too short"),
            Self::NotEnoughSamples => write!(f, "Not enough training samples"),
            Self::GestureNotFound => write!(f, "Gesture not found"),
            Self::SerializationFailed => write!(f, "Failed to serialize gestures"),
            Self::DeserializationFailed => write!(f, "Failed to deserialize gestures"),
        }
    }
}

impl std::error::Error for LearningError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learner_creation() {
        let learner = GestureLearner::new();
        assert!(!learner.is_training());
        assert_eq!(learner.gesture_names().len(), 0);
    }

    #[test]
    fn test_start_training() {
        let mut learner = GestureLearner::new();
        assert!(learner.start_training("test").is_ok());
        assert!(learner.is_training());
    }

    #[test]
    fn test_double_training_error() {
        let mut learner = GestureLearner::new();
        learner.start_training("test").unwrap();
        assert_eq!(
            learner.start_training("test2"),
            Err(LearningError::AlreadyTraining)
        );
    }

    #[test]
    fn test_cancel_training() {
        let mut learner = GestureLearner::new();
        learner.start_training("test").unwrap();
        learner.cancel_training();
        assert!(!learner.is_training());
    }

    #[test]
    fn test_point3_distance() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 0.0, 0.0);
        assert!((p1.distance(&p2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_training_progress() {
        let mut learner = GestureLearner::new();
        learner.start_training("test").unwrap();
        
        let progress = learner.training_progress().unwrap();
        assert_eq!(progress.gesture_name, "test");
        assert_eq!(progress.samples_collected, 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = LearningConfig::default();
        assert!(config.min_samples > 0);
        assert!(config.recognition_threshold > 0.0);
    }
}
