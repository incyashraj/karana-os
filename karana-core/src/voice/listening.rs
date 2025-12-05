//! Continuous Listening and Wake Word Detection
//!
//! Always-on voice detection for smart glasses.

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Continuous listening system
pub struct ContinuousListener {
    /// Configuration
    config: ListenerConfig,
    /// Wake word detector
    wake_detector: WakeWordDetector,
    /// Voice activity detector
    vad: VoiceActivityDetector,
    /// Audio buffer
    audio_buffer: AudioRingBuffer,
    /// State
    state: ListenerState,
    /// Noise estimator
    noise_estimator: NoiseEstimator,
    /// Echo canceller
    echo_canceller: EchoCanceller,
}

impl ContinuousListener {
    /// Create new listener
    pub fn new(config: ListenerConfig) -> Self {
        let buffer_size = (config.sample_rate as f32 * config.buffer_duration) as usize;

        Self {
            wake_detector: WakeWordDetector::new(&config.wake_words),
            vad: VoiceActivityDetector::new(config.vad_config.clone()),
            audio_buffer: AudioRingBuffer::new(buffer_size),
            state: ListenerState::Idle,
            noise_estimator: NoiseEstimator::new(config.sample_rate),
            echo_canceller: EchoCanceller::new(),
            config,
        }
    }

    /// Process audio frame
    pub fn process_frame(&mut self, samples: &[f32]) -> ListenerEvent {
        // Add to buffer
        self.audio_buffer.push_samples(samples);

        // Update noise estimate
        let noise_level = self.noise_estimator.update(samples);

        // Apply echo cancellation if needed
        let processed = if self.config.echo_cancellation {
            self.echo_canceller.process(samples)
        } else {
            samples.to_vec()
        };

        // Check state-specific processing
        match self.state {
            ListenerState::Idle => {
                self.process_idle(&processed, noise_level)
            }
            ListenerState::WakeWordDetected => {
                self.process_wake_detected(&processed)
            }
            ListenerState::Listening => {
                self.process_listening(&processed)
            }
            ListenerState::Processing => {
                ListenerEvent::None
            }
        }
    }

    /// Process in idle state
    fn process_idle(&mut self, samples: &[f32], noise_level: f32) -> ListenerEvent {
        // Check for wake word
        if let Some(wake_result) = self.wake_detector.detect(samples) {
            if wake_result.confidence >= self.config.wake_word_threshold {
                self.state = ListenerState::WakeWordDetected;
                return ListenerEvent::WakeWordDetected {
                    word: wake_result.word,
                    confidence: wake_result.confidence,
                };
            }
        }

        // Monitor for continuous listening mode
        if self.config.continuous_mode {
            let vad_result = self.vad.process(samples);
            if vad_result.is_speech && noise_level < self.config.noise_threshold {
                return ListenerEvent::SpeechDetected;
            }
        }

        ListenerEvent::None
    }

    /// Process after wake word detected
    fn process_wake_detected(&mut self, samples: &[f32]) -> ListenerEvent {
        let vad_result = self.vad.process(samples);

        if vad_result.is_speech {
            self.state = ListenerState::Listening;
            return ListenerEvent::ListeningStarted;
        }

        ListenerEvent::None
    }

    /// Process while listening
    fn process_listening(&mut self, samples: &[f32]) -> ListenerEvent {
        let vad_result = self.vad.process(samples);

        // Check for end of speech
        if !vad_result.is_speech {
            if self.vad.silence_duration() >= self.config.end_of_speech_timeout {
                self.state = ListenerState::Processing;

                // Get buffered audio
                let audio = self.audio_buffer.get_recent(
                    (self.config.sample_rate as f32 * 10.0) as usize
                );

                return ListenerEvent::SpeechEnded { audio };
            }
        }

        ListenerEvent::Listening {
            level: vad_result.energy,
        }
    }

    /// Set playback reference for echo cancellation
    pub fn set_playback_reference(&mut self, samples: &[f32]) {
        self.echo_canceller.set_reference(samples);
    }

    /// Reset to idle
    pub fn reset(&mut self) {
        self.state = ListenerState::Idle;
        self.vad.reset();
        self.audio_buffer.clear();
    }

    /// Get current state
    pub fn state(&self) -> ListenerState {
        self.state
    }

    /// Get noise level
    pub fn noise_level(&self) -> f32 {
        self.noise_estimator.current_level()
    }
}

impl Default for ContinuousListener {
    fn default() -> Self {
        Self::new(ListenerConfig::default())
    }
}

/// Listener configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    /// Sample rate
    pub sample_rate: u32,
    /// Wake words
    pub wake_words: Vec<String>,
    /// Wake word detection threshold
    pub wake_word_threshold: f32,
    /// Buffer duration in seconds
    pub buffer_duration: f32,
    /// End of speech timeout (seconds)
    pub end_of_speech_timeout: f32,
    /// Noise threshold
    pub noise_threshold: f32,
    /// Enable continuous listening
    pub continuous_mode: bool,
    /// Enable echo cancellation
    pub echo_cancellation: bool,
    /// VAD configuration
    pub vad_config: VadConfig,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            wake_words: vec!["hey karana".to_string(), "karana".to_string()],
            wake_word_threshold: 0.8,
            buffer_duration: 30.0,
            end_of_speech_timeout: 1.5,
            noise_threshold: 0.7,
            continuous_mode: false,
            echo_cancellation: true,
            vad_config: VadConfig::default(),
        }
    }
}

/// Listener state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerState {
    /// Waiting for wake word
    Idle,
    /// Wake word detected, waiting for speech
    WakeWordDetected,
    /// Actively listening
    Listening,
    /// Processing speech
    Processing,
}

/// Listener event
#[derive(Debug, Clone)]
pub enum ListenerEvent {
    /// No event
    None,
    /// Wake word detected
    WakeWordDetected {
        word: String,
        confidence: f32,
    },
    /// Speech detected (continuous mode)
    SpeechDetected,
    /// Listening started
    ListeningStarted,
    /// Currently listening
    Listening {
        level: f32,
    },
    /// Speech ended
    SpeechEnded {
        audio: Vec<f32>,
    },
}

/// Wake word detector
pub struct WakeWordDetector {
    /// Wake words
    words: Vec<String>,
    /// Detection models (simplified)
    models: Vec<WakeWordModel>,
    /// Feature extractor
    extractor: MfccExtractor,
    /// Frame buffer
    frame_buffer: VecDeque<Vec<f32>>,
}

impl WakeWordDetector {
    /// Create new detector
    pub fn new(words: &[String]) -> Self {
        let models = words.iter()
            .map(|w| WakeWordModel::new(w))
            .collect();

        Self {
            words: words.to_vec(),
            models,
            extractor: MfccExtractor::new(16000, 13),
            frame_buffer: VecDeque::with_capacity(100),
        }
    }

    /// Detect wake word in samples
    pub fn detect(&mut self, samples: &[f32]) -> Option<WakeWordResult> {
        // Extract features
        let features = self.extractor.extract(samples);
        self.frame_buffer.push_back(features);

        // Limit buffer
        while self.frame_buffer.len() > 100 {
            self.frame_buffer.pop_front();
        }

        // Check each model
        for (i, model) in self.models.iter().enumerate() {
            let confidence = model.score(&self.frame_buffer);
            if confidence > 0.5 {
                return Some(WakeWordResult {
                    word: self.words[i].clone(),
                    confidence,
                });
            }
        }

        None
    }
}

/// Wake word detection result
#[derive(Debug, Clone)]
pub struct WakeWordResult {
    /// Detected word
    pub word: String,
    /// Confidence score
    pub confidence: f32,
}

/// Wake word model (simplified)
struct WakeWordModel {
    /// Word this model detects
    word: String,
    /// Template features
    templates: Vec<Vec<f32>>,
}

impl WakeWordModel {
    /// Create new model
    fn new(word: &str) -> Self {
        Self {
            word: word.to_string(),
            templates: Vec::new(),
        }
    }

    /// Score against frame buffer
    fn score(&self, _frames: &VecDeque<Vec<f32>>) -> f32 {
        // Simplified scoring - real implementation would use DTW or neural network
        0.0
    }
}

/// Voice Activity Detector
pub struct VoiceActivityDetector {
    /// Configuration
    config: VadConfig,
    /// Energy history
    energy_history: VecDeque<f32>,
    /// Speech state
    is_speech: bool,
    /// Frames since state change
    frames_since_change: usize,
    /// Silence duration counter
    silence_frames: usize,
    /// Sample rate
    sample_rate: u32,
}

impl VoiceActivityDetector {
    /// Create new VAD
    pub fn new(config: VadConfig) -> Self {
        Self {
            energy_history: VecDeque::with_capacity(100),
            is_speech: false,
            frames_since_change: 0,
            silence_frames: 0,
            sample_rate: 16000,
            config,
        }
    }

    /// Process audio frame
    pub fn process(&mut self, samples: &[f32]) -> VadResult {
        // Calculate frame energy
        let energy = self.calculate_energy(samples);
        self.energy_history.push_back(energy);
        if self.energy_history.len() > 100 {
            self.energy_history.pop_front();
        }

        // Calculate adaptive threshold
        let threshold = self.adaptive_threshold();

        // Detect speech
        let speech_detected = energy > threshold * self.config.threshold_multiplier;

        // Hysteresis
        if speech_detected != self.is_speech {
            self.frames_since_change += 1;
            let hold_frames = if self.is_speech {
                self.config.hangover_frames
            } else {
                self.config.activation_frames
            };

            if self.frames_since_change >= hold_frames {
                self.is_speech = speech_detected;
                self.frames_since_change = 0;
            }
        } else {
            self.frames_since_change = 0;
        }

        // Track silence
        if self.is_speech {
            self.silence_frames = 0;
        } else {
            self.silence_frames += 1;
        }

        VadResult {
            is_speech: self.is_speech,
            energy,
            threshold,
        }
    }

    /// Calculate frame energy
    fn calculate_energy(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    /// Calculate adaptive threshold
    fn adaptive_threshold(&self) -> f32 {
        if self.energy_history.is_empty() {
            return self.config.min_threshold;
        }

        // Use lower percentile as noise floor estimate
        let mut sorted: Vec<f32> = self.energy_history.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let percentile_idx = (sorted.len() as f32 * 0.2) as usize;
        let noise_floor = sorted[percentile_idx.min(sorted.len() - 1)];

        (noise_floor * 2.0).max(self.config.min_threshold)
    }

    /// Get silence duration in seconds
    pub fn silence_duration(&self) -> f32 {
        let frame_duration = 0.02; // 20ms frames assumed
        self.silence_frames as f32 * frame_duration
    }

    /// Reset VAD state
    pub fn reset(&mut self) {
        self.is_speech = false;
        self.frames_since_change = 0;
        self.silence_frames = 0;
        self.energy_history.clear();
    }
}

/// VAD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Minimum energy threshold
    pub min_threshold: f32,
    /// Threshold multiplier
    pub threshold_multiplier: f32,
    /// Frames to activate speech
    pub activation_frames: usize,
    /// Frames to deactivate (hangover)
    pub hangover_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            min_threshold: 0.01,
            threshold_multiplier: 3.0,
            activation_frames: 3,
            hangover_frames: 15,
        }
    }
}

/// VAD result
#[derive(Debug, Clone)]
pub struct VadResult {
    /// Is speech detected
    pub is_speech: bool,
    /// Frame energy
    pub energy: f32,
    /// Current threshold
    pub threshold: f32,
}

/// Audio ring buffer
pub struct AudioRingBuffer {
    /// Buffer
    buffer: VecDeque<f32>,
    /// Maximum size
    max_size: usize,
}

impl AudioRingBuffer {
    /// Create new buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push samples
    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer.push_back(sample);
        }

        while self.buffer.len() > self.max_size {
            self.buffer.pop_front();
        }
    }

    /// Get recent samples
    pub fn get_recent(&self, count: usize) -> Vec<f32> {
        let start = self.buffer.len().saturating_sub(count);
        self.buffer.iter().skip(start).copied().collect()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Noise estimator
pub struct NoiseEstimator {
    /// Noise floor estimate
    noise_floor: f32,
    /// Update rate
    alpha: f32,
    /// Sample rate
    _sample_rate: u32,
}

impl NoiseEstimator {
    /// Create new estimator
    pub fn new(sample_rate: u32) -> Self {
        Self {
            noise_floor: 0.01,
            alpha: 0.05,
            _sample_rate: sample_rate,
        }
    }

    /// Update with new samples
    pub fn update(&mut self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return self.noise_floor;
        }

        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        let energy = energy.sqrt();

        // Only update if energy is low (likely noise)
        if energy < self.noise_floor * 2.0 {
            self.noise_floor = self.noise_floor * (1.0 - self.alpha) + energy * self.alpha;
        }

        energy
    }

    /// Get current noise level
    pub fn current_level(&self) -> f32 {
        self.noise_floor
    }
}

/// Echo canceller (simplified)
pub struct EchoCanceller {
    /// Reference buffer
    reference: VecDeque<f32>,
    /// Filter coefficients
    filter: Vec<f32>,
    /// Adaptation rate
    mu: f32,
}

impl EchoCanceller {
    /// Create new canceller
    pub fn new() -> Self {
        Self {
            reference: VecDeque::with_capacity(1600),
            filter: vec![0.0; 160],
            mu: 0.01,
        }
    }

    /// Set playback reference
    pub fn set_reference(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.reference.push_back(sample);
        }
        while self.reference.len() > 1600 {
            self.reference.pop_front();
        }
    }

    /// Process samples (remove echo)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());

        for &sample in samples {
            // Estimate echo
            let mut echo_estimate = 0.0;
            for (i, &coef) in self.filter.iter().enumerate() {
                if i < self.reference.len() {
                    echo_estimate += coef * self.reference[self.reference.len() - 1 - i];
                }
            }

            // Remove echo
            let cleaned = sample - echo_estimate;
            output.push(cleaned);

            // Adapt filter (NLMS-like)
            let error = cleaned;
            let norm: f32 = self.reference.iter().take(self.filter.len())
                .map(|x| x * x).sum::<f32>() + 1e-6;

            for (i, coef) in self.filter.iter_mut().enumerate() {
                if i < self.reference.len() {
                    *coef += self.mu * error * self.reference[self.reference.len() - 1 - i] / norm;
                }
            }
        }

        output
    }
}

impl Default for EchoCanceller {
    fn default() -> Self {
        Self::new()
    }
}

/// MFCC feature extractor (simplified)
pub struct MfccExtractor {
    /// Number of coefficients
    num_coeffs: usize,
    /// Sample rate
    _sample_rate: u32,
}

impl MfccExtractor {
    /// Create new extractor
    pub fn new(sample_rate: u32, num_coeffs: usize) -> Self {
        Self {
            num_coeffs,
            _sample_rate: sample_rate,
        }
    }

    /// Extract MFCC features
    pub fn extract(&self, samples: &[f32]) -> Vec<f32> {
        // Simplified - real implementation would do:
        // 1. Windowing
        // 2. FFT
        // 3. Mel filterbank
        // 4. Log
        // 5. DCT

        let mut features = vec![0.0; self.num_coeffs];

        // Simple energy-based features as placeholder
        if !samples.is_empty() {
            let energy: f32 = samples.iter().map(|s| s * s).sum();
            features[0] = (energy / samples.len() as f32).sqrt();

            // Zero crossing rate
            let zcr = samples.windows(2)
                .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
                .count();
            features[1] = zcr as f32 / samples.len() as f32;
        }

        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_listener() {
        let listener = ContinuousListener::default();
        assert_eq!(listener.state(), ListenerState::Idle);
    }

    #[test]
    fn test_vad() {
        let mut vad = VoiceActivityDetector::new(VadConfig::default());

        // Silent samples
        let silent = vec![0.001; 320];
        let result = vad.process(&silent);
        assert!(!result.is_speech);

        // Loud samples
        let loud = vec![0.5; 320];
        for _ in 0..5 {
            vad.process(&loud);
        }
        let result = vad.process(&loud);
        assert!(result.is_speech);
    }

    #[test]
    fn test_audio_ring_buffer() {
        let mut buffer = AudioRingBuffer::new(100);
        buffer.push_samples(&[1.0, 2.0, 3.0]);

        let recent = buffer.get_recent(2);
        assert_eq!(recent, vec![2.0, 3.0]);
    }

    #[test]
    fn test_noise_estimator() {
        let mut estimator = NoiseEstimator::new(16000);

        let samples = vec![0.01; 320];
        estimator.update(&samples);

        let level = estimator.current_level();
        assert!(level > 0.0);
    }

    #[test]
    fn test_echo_canceller() {
        let mut canceller = EchoCanceller::new();

        let reference = vec![0.5; 320];
        canceller.set_reference(&reference);

        let input = vec![0.3; 320];
        let output = canceller.process(&input);

        assert_eq!(output.len(), input.len());
    }

    #[test]
    fn test_mfcc_extractor() {
        let extractor = MfccExtractor::new(16000, 13);

        let samples = vec![0.1; 320];
        let features = extractor.extract(&samples);

        assert_eq!(features.len(), 13);
    }
}
