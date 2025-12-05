// Kāraṇa OS - Speech Processing Module
// Whisper-based speech recognition and voice activity detection

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{MLSystem, MLError, MLConfig};
use super::inference::{InferenceRequest, InputData, OutputData, AudioInput};
use super::models::ModelType;

/// Speech recognition system using Whisper
#[derive(Debug)]
pub struct SpeechRecognizer {
    /// Configuration
    config: SpeechConfig,
    /// Audio buffer for streaming
    audio_buffer: VecDeque<f32>,
    /// Voice activity detector
    vad: VoiceActivityDetector,
    /// Current transcription session
    session: Option<TranscriptionSession>,
    /// Language detection results
    detected_language: Option<String>,
    /// Processing state
    state: SpeechState,
}

/// Speech configuration
#[derive(Debug, Clone)]
pub struct SpeechConfig {
    /// Model size (tiny, base, small, medium, large)
    pub model_size: WhisperModel,
    /// Language (None for auto-detect)
    pub language: Option<String>,
    /// Enable word timestamps
    pub word_timestamps: bool,
    /// Sample rate (typically 16000)
    pub sample_rate: u32,
    /// Chunk size for streaming (in samples)
    pub chunk_size: usize,
    /// Maximum audio length (in seconds)
    pub max_audio_length: f32,
    /// VAD threshold
    pub vad_threshold: f32,
    /// Enable speaker diarization
    pub speaker_diarization: bool,
}

impl Default for SpeechConfig {
    fn default() -> Self {
        Self {
            model_size: WhisperModel::Tiny,
            language: None,
            word_timestamps: true,
            sample_rate: 16000,
            chunk_size: 4800, // 300ms at 16kHz
            max_audio_length: 30.0,
            vad_threshold: 0.5,
            speaker_diarization: false,
        }
    }
}

/// Whisper model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhisperModel {
    Tiny,    // 39M params
    Base,    // 74M params
    Small,   // 244M params
    Medium,  // 769M params
    Large,   // 1550M params
    LargeV3, // Latest large model
}

impl WhisperModel {
    /// Get model ID string
    pub fn model_id(&self) -> &'static str {
        match self {
            WhisperModel::Tiny => "whisper-tiny",
            WhisperModel::Base => "whisper-base",
            WhisperModel::Small => "whisper-small",
            WhisperModel::Medium => "whisper-medium",
            WhisperModel::Large => "whisper-large",
            WhisperModel::LargeV3 => "whisper-large-v3",
        }
    }

    /// Get parameters count
    pub fn params(&self) -> usize {
        match self {
            WhisperModel::Tiny => 39_000_000,
            WhisperModel::Base => 74_000_000,
            WhisperModel::Small => 244_000_000,
            WhisperModel::Medium => 769_000_000,
            WhisperModel::Large => 1_550_000_000,
            WhisperModel::LargeV3 => 1_550_000_000,
        }
    }

    /// Get memory requirement (MB)
    pub fn memory_mb(&self) -> usize {
        match self {
            WhisperModel::Tiny => 75,
            WhisperModel::Base => 150,
            WhisperModel::Small => 500,
            WhisperModel::Medium => 1500,
            WhisperModel::Large => 3000,
            WhisperModel::LargeV3 => 3200,
        }
    }
}

/// Speech processing state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechState {
    Idle,
    Listening,
    Processing,
    Transcribing,
}

/// Transcription session
#[derive(Debug)]
struct TranscriptionSession {
    /// Session start time
    start_time: Instant,
    /// Accumulated audio samples
    samples: Vec<f32>,
    /// Partial transcriptions
    partials: Vec<String>,
    /// Is currently speaking
    is_speaking: bool,
}

/// Voice Activity Detector
#[derive(Debug)]
pub struct VoiceActivityDetector {
    /// Threshold for speech detection
    threshold: f32,
    /// Minimum speech duration (ms)
    min_speech_duration_ms: u32,
    /// Minimum silence duration (ms)
    min_silence_duration_ms: u32,
    /// Current state
    is_speech: bool,
    /// Energy history
    energy_history: VecDeque<f32>,
    /// Speech frame count
    speech_frames: u32,
    /// Silence frame count
    silence_frames: u32,
}

impl VoiceActivityDetector {
    /// Create new VAD
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            min_speech_duration_ms: 250,
            min_silence_duration_ms: 500,
            is_speech: false,
            energy_history: VecDeque::with_capacity(50),
            speech_frames: 0,
            silence_frames: 0,
        }
    }

    /// Process audio frame
    pub fn process_frame(&mut self, samples: &[f32]) -> VadResult {
        // Calculate frame energy (RMS)
        let energy: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        // Update energy history
        self.energy_history.push_back(energy);
        if self.energy_history.len() > 50 {
            self.energy_history.pop_front();
        }

        // Calculate adaptive threshold
        let mean_energy: f32 = self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;
        let adaptive_threshold = mean_energy + self.threshold;

        // Update state
        let is_speech_frame = energy > adaptive_threshold;

        if is_speech_frame {
            self.speech_frames += 1;
            self.silence_frames = 0;
        } else {
            self.silence_frames += 1;
            self.speech_frames = 0;
        }

        // State transitions
        let prev_speech = self.is_speech;

        // Speech start detection
        if !self.is_speech && self.speech_frames * 30 >= self.min_speech_duration_ms {
            self.is_speech = true;
        }

        // Speech end detection
        if self.is_speech && self.silence_frames * 30 >= self.min_silence_duration_ms {
            self.is_speech = false;
        }

        VadResult {
            is_speech: self.is_speech,
            speech_started: !prev_speech && self.is_speech,
            speech_ended: prev_speech && !self.is_speech,
            energy,
        }
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.is_speech = false;
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.energy_history.clear();
    }
}

/// VAD processing result
#[derive(Debug, Clone)]
pub struct VadResult {
    /// Currently detecting speech
    pub is_speech: bool,
    /// Speech just started
    pub speech_started: bool,
    /// Speech just ended
    pub speech_ended: bool,
    /// Frame energy level
    pub energy: f32,
}

impl SpeechRecognizer {
    /// Create new speech recognizer
    pub fn new(config: SpeechConfig) -> Self {
        let vad_threshold = config.vad_threshold;
        Self {
            config,
            audio_buffer: VecDeque::with_capacity(16000 * 30), // 30 seconds
            vad: VoiceActivityDetector::new(vad_threshold),
            session: None,
            detected_language: None,
            state: SpeechState::Idle,
        }
    }

    /// Start listening
    pub fn start_listening(&mut self) {
        self.state = SpeechState::Listening;
        self.audio_buffer.clear();
        self.vad.reset();
        self.session = Some(TranscriptionSession {
            start_time: Instant::now(),
            samples: Vec::new(),
            partials: Vec::new(),
            is_speaking: false,
        });
    }

    /// Stop listening
    pub fn stop_listening(&mut self) -> Option<TranscriptionResult> {
        let session = self.session.take()?;
        self.state = SpeechState::Idle;

        if session.samples.is_empty() {
            return None;
        }

        // Final transcription
        Some(self.transcribe_samples(&session.samples))
    }

    /// Feed audio samples
    pub fn feed_audio(&mut self, samples: &[f32]) -> Option<PartialTranscription> {
        if self.state != SpeechState::Listening {
            return None;
        }

        // VAD processing
        let vad_result = self.vad.process_frame(samples);

        // Update session
        let partial_result = if let Some(ref mut session) = self.session {
            if vad_result.is_speech {
                session.samples.extend_from_slice(samples);
                session.is_speaking = true;

                // Add to buffer
                for &s in samples {
                    self.audio_buffer.push_back(s);
                }
            }

            // When speech ends, prepare for partial transcription
            if vad_result.speech_ended && !session.samples.is_empty() {
                let samples_clone = session.samples.clone();
                Some(samples_clone)
            } else {
                None
            }
        } else {
            None
        };

        // Do partial transcription outside of the mutable borrow
        if let Some(samples_for_partial) = partial_result {
            let partial = self.transcribe_partial(&samples_for_partial);
            if let Some(ref mut session) = self.session {
                session.partials.push(partial.text.clone());
            }
            return Some(partial);
        }

        None
    }

    /// Transcribe audio samples (simulated)
    fn transcribe_samples(&self, samples: &[f32]) -> TranscriptionResult {
        let duration = samples.len() as f32 / self.config.sample_rate as f32;

        // Simulated transcription
        TranscriptionResult {
            text: "Simulated transcription of audio input.".to_string(),
            language: self.config.language.clone().unwrap_or_else(|| "en".to_string()),
            confidence: 0.95,
            duration_seconds: duration,
            words: if self.config.word_timestamps {
                vec![
                    WordTimestamp { word: "Simulated".to_string(), start: 0.0, end: 0.5, confidence: 0.95 },
                    WordTimestamp { word: "transcription".to_string(), start: 0.5, end: 1.0, confidence: 0.94 },
                    WordTimestamp { word: "of".to_string(), start: 1.0, end: 1.1, confidence: 0.99 },
                    WordTimestamp { word: "audio".to_string(), start: 1.1, end: 1.4, confidence: 0.96 },
                    WordTimestamp { word: "input".to_string(), start: 1.4, end: 1.8, confidence: 0.93 },
                ]
            } else {
                Vec::new()
            },
            segments: vec![
                TranscriptionSegment {
                    id: 0,
                    start: 0.0,
                    end: duration,
                    text: "Simulated transcription of audio input.".to_string(),
                    confidence: 0.95,
                    speaker: None,
                }
            ],
        }
    }

    /// Transcribe partial audio
    fn transcribe_partial(&self, samples: &[f32]) -> PartialTranscription {
        let duration = samples.len() as f32 / self.config.sample_rate as f32;

        PartialTranscription {
            text: "Partial transcription...".to_string(),
            confidence: 0.8,
            is_final: false,
            duration_seconds: duration,
        }
    }

    /// Get current state
    pub fn state(&self) -> SpeechState {
        self.state
    }

    /// Transcribe audio file (one-shot)
    pub fn transcribe_audio(&self, audio: AudioInput) -> Result<TranscriptionResult, MLError> {
        // Convert to samples
        let samples = audio.data.clone();
        Ok(self.transcribe_samples(&samples))
    }

    /// Detect language from audio
    pub fn detect_language(&self, samples: &[f32]) -> LanguageDetection {
        // Simulated language detection
        LanguageDetection {
            language: "en".to_string(),
            confidence: 0.92,
            alternatives: vec![
                ("es".to_string(), 0.03),
                ("fr".to_string(), 0.02),
                ("de".to_string(), 0.015),
            ],
        }
    }
}

/// Transcription result
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    /// Full transcribed text
    pub text: String,
    /// Detected/specified language
    pub language: String,
    /// Overall confidence
    pub confidence: f32,
    /// Audio duration
    pub duration_seconds: f32,
    /// Word-level timestamps
    pub words: Vec<WordTimestamp>,
    /// Segment-level results
    pub segments: Vec<TranscriptionSegment>,
}

/// Partial transcription
#[derive(Debug, Clone)]
pub struct PartialTranscription {
    /// Partial text
    pub text: String,
    /// Current confidence
    pub confidence: f32,
    /// Is this the final result
    pub is_final: bool,
    /// Duration so far
    pub duration_seconds: f32,
}

/// Word with timestamp
#[derive(Debug, Clone)]
pub struct WordTimestamp {
    /// The word
    pub word: String,
    /// Start time (seconds)
    pub start: f32,
    /// End time (seconds)
    pub end: f32,
    /// Word confidence
    pub confidence: f32,
}

/// Transcription segment
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    /// Segment ID
    pub id: usize,
    /// Start time
    pub start: f32,
    /// End time
    pub end: f32,
    /// Segment text
    pub text: String,
    /// Segment confidence
    pub confidence: f32,
    /// Speaker ID (if diarization enabled)
    pub speaker: Option<String>,
}

/// Language detection result
#[derive(Debug, Clone)]
pub struct LanguageDetection {
    /// Most likely language
    pub language: String,
    /// Confidence
    pub confidence: f32,
    /// Alternative languages with scores
    pub alternatives: Vec<(String, f32)>,
}

/// Command recognizer for wake words and commands
#[derive(Debug)]
pub struct CommandRecognizer {
    /// Wake words to listen for
    wake_words: Vec<String>,
    /// Command patterns
    commands: Vec<CommandPattern>,
    /// Is active (after wake word)
    active: bool,
    /// Activation time
    activated_at: Option<Instant>,
    /// Active timeout
    active_timeout: Duration,
}

/// Command pattern
#[derive(Debug, Clone)]
pub struct CommandPattern {
    /// Pattern name
    pub name: String,
    /// Keywords to match
    pub keywords: Vec<String>,
    /// Fuzzy match threshold
    pub threshold: f32,
}

impl CommandRecognizer {
    /// Create new command recognizer
    pub fn new() -> Self {
        Self {
            wake_words: vec!["hey karana".to_string(), "karana".to_string()],
            commands: Vec::new(),
            active: false,
            activated_at: None,
            active_timeout: Duration::from_secs(10),
        }
    }

    /// Add command pattern
    pub fn add_command(&mut self, pattern: CommandPattern) {
        self.commands.push(pattern);
    }

    /// Check if text contains wake word
    pub fn check_wake_word(&mut self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        for wake_word in &self.wake_words {
            if text_lower.contains(wake_word) {
                self.active = true;
                self.activated_at = Some(Instant::now());
                return true;
            }
        }
        false
    }

    /// Match command from text
    pub fn match_command(&self, text: &str) -> Option<CommandMatch> {
        if !self.active {
            return None;
        }

        let text_lower = text.to_lowercase();

        for command in &self.commands {
            let mut matched_keywords = 0;
            for keyword in &command.keywords {
                if text_lower.contains(&keyword.to_lowercase()) {
                    matched_keywords += 1;
                }
            }

            let score = matched_keywords as f32 / command.keywords.len() as f32;
            if score >= command.threshold {
                return Some(CommandMatch {
                    command: command.name.clone(),
                    confidence: score,
                    text: text.to_string(),
                });
            }
        }

        None
    }

    /// Update state (check timeout)
    pub fn update(&mut self) {
        if let Some(activated_at) = self.activated_at {
            if activated_at.elapsed() > self.active_timeout {
                self.active = false;
                self.activated_at = None;
            }
        }
    }

    /// Deactivate manually
    pub fn deactivate(&mut self) {
        self.active = false;
        self.activated_at = None;
    }

    /// Is currently active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for CommandRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Matched command
#[derive(Debug, Clone)]
pub struct CommandMatch {
    /// Command name
    pub command: String,
    /// Match confidence
    pub confidence: f32,
    /// Original text
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speech_config_default() {
        let config = SpeechConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert!(config.word_timestamps);
    }

    #[test]
    fn test_whisper_model_info() {
        assert_eq!(WhisperModel::Tiny.model_id(), "whisper-tiny");
        assert_eq!(WhisperModel::Tiny.memory_mb(), 75);
        assert!(WhisperModel::Large.params() > WhisperModel::Small.params());
    }

    #[test]
    fn test_vad() {
        let mut vad = VoiceActivityDetector::new(0.01); // Low threshold for testing

        // First establish a baseline with silent frames
        let silent = vec![0.0f32; 480];
        for _ in 0..10 {
            vad.process_frame(&silent);
        }

        // Verify not detecting speech during silence
        let result = vad.process_frame(&silent);
        assert!(!result.is_speech);

        // Now send loud frames - energy 0.5 should be well above the baseline + 0.01
        let loud = vec![0.5f32; 480];
        for _ in 0..20 {
            vad.process_frame(&loud);
        }
        let result = vad.process_frame(&loud);
        assert!(result.is_speech);
    }

    #[test]
    fn test_speech_recognizer() {
        let config = SpeechConfig::default();
        let mut recognizer = SpeechRecognizer::new(config);

        assert_eq!(recognizer.state(), SpeechState::Idle);

        recognizer.start_listening();
        assert_eq!(recognizer.state(), SpeechState::Listening);

        // Feed some audio
        let samples = vec![0.1f32; 4800];
        let _ = recognizer.feed_audio(&samples);

        let result = recognizer.stop_listening();
        assert_eq!(recognizer.state(), SpeechState::Idle);
    }

    #[test]
    fn test_transcribe_audio() {
        let config = SpeechConfig::default();
        let recognizer = SpeechRecognizer::new(config);

        let audio = AudioInput {
            data: vec![0.1; 16000],
            sample_rate: 16000,
            channels: 1,
        };

        let result = recognizer.transcribe_audio(audio);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.text.is_empty());
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_language_detection() {
        let config = SpeechConfig::default();
        let recognizer = SpeechRecognizer::new(config);

        let samples = vec![0.1; 16000];
        let detection = recognizer.detect_language(&samples);

        assert!(!detection.language.is_empty());
        assert!(detection.confidence > 0.0);
    }

    #[test]
    fn test_command_recognizer() {
        let mut recognizer = CommandRecognizer::new();

        recognizer.add_command(CommandPattern {
            name: "open_app".to_string(),
            keywords: vec!["open".to_string(), "app".to_string()],
            threshold: 0.5,
        });

        // Not active yet
        assert!(recognizer.match_command("open the app").is_none());

        // Activate with wake word
        assert!(recognizer.check_wake_word("Hey Karana"));
        assert!(recognizer.is_active());

        // Now match command
        let result = recognizer.match_command("please open the app");
        assert!(result.is_some());
        assert_eq!(result.unwrap().command, "open_app");
    }

    #[test]
    fn test_command_timeout() {
        let mut recognizer = CommandRecognizer::new();
        recognizer.active_timeout = Duration::from_millis(1);

        recognizer.check_wake_word("karana");
        assert!(recognizer.is_active());

        std::thread::sleep(Duration::from_millis(10));
        recognizer.update();
        assert!(!recognizer.is_active());
    }

    #[test]
    fn test_word_timestamps() {
        let mut config = SpeechConfig::default();
        config.word_timestamps = true;

        let recognizer = SpeechRecognizer::new(config);
        let samples = vec![0.1; 16000];
        let result = recognizer.transcribe_samples(&samples);

        assert!(!result.words.is_empty());
        for word in &result.words {
            assert!(word.start <= word.end);
            assert!(word.confidence > 0.0);
        }
    }
}
