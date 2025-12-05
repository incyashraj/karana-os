// Speech Recognition for Kāraṇa OS
// Real-time speech-to-text processing

use std::collections::VecDeque;

/// Speech recognition model type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecognitionModel {
    /// Fast, lightweight model for simple commands
    Fast,
    /// Accurate model for general speech
    Accurate,
    /// Offline model for privacy/connectivity
    Offline,
    /// Domain-specific model
    Domain(DomainType),
}

/// Domain types for specialized recognition
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DomainType {
    Navigation,
    Messaging,
    SmartHome,
    Medical,
    Technical,
}

/// Recognition result
#[derive(Debug, Clone)]
pub struct RecognitionResult {
    pub transcript: String,
    pub confidence: f32,
    pub is_final: bool,
    pub alternatives: Vec<(String, f32)>,
    pub timestamp: u64,
    pub duration_ms: u64,
}

/// Word timing information
#[derive(Debug, Clone)]
pub struct WordTiming {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
}

/// Speech segment
#[derive(Debug, Clone)]
pub struct SpeechSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker_id: Option<String>,
    pub words: Vec<WordTiming>,
}

/// Voice activity detection state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceActivity {
    Silence,
    Speech,
    Noise,
    Music,
}

/// Speech recognizer
pub struct SpeechRecognizer {
    language: String,
    model: RecognitionModel,
    audio_buffer: VecDeque<f32>,
    sample_rate: u32,
    partial_result: String,
    is_processing: bool,
    vad_enabled: bool,
    noise_threshold: f32,
    silence_timeout_ms: u64,
    last_speech_time: u64,
}

impl SpeechRecognizer {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            model: RecognitionModel::Accurate,
            audio_buffer: VecDeque::with_capacity(16000 * 5), // 5 seconds at 16kHz
            sample_rate: 16000,
            partial_result: String::new(),
            is_processing: false,
            vad_enabled: true,
            noise_threshold: 0.02,
            silence_timeout_ms: 1500,
            last_speech_time: 0,
        }
    }

    pub fn with_model(language: &str, model: RecognitionModel) -> Self {
        let mut recognizer = Self::new(language);
        recognizer.model = model;
        recognizer
    }

    pub fn process(&mut self, samples: &[f32]) -> Option<String> {
        // Add samples to buffer
        for &sample in samples {
            if self.audio_buffer.len() >= self.audio_buffer.capacity() {
                self.audio_buffer.pop_front();
            }
            self.audio_buffer.push_back(sample);
        }

        // Check for voice activity
        if self.vad_enabled && !self.detect_speech(samples) {
            return None;
        }

        // Simulate recognition (in real implementation, would use ML model)
        self.is_processing = true;
        let result = self.simulate_recognition();
        self.is_processing = false;

        result
    }

    fn detect_speech(&self, samples: &[f32]) -> bool {
        if samples.is_empty() {
            return false;
        }
        
        // Simple energy-based VAD
        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        energy.sqrt() > self.noise_threshold
    }

    fn simulate_recognition(&mut self) -> Option<String> {
        // Simulated recognition - in real implementation would use Whisper, etc.
        if self.audio_buffer.len() > 8000 {
            // Simulate detecting some speech
            Some("simulated transcript".to_string())
        } else {
            None
        }
    }

    pub fn get_partial_result(&self) -> &str {
        &self.partial_result
    }

    pub fn reset(&mut self) {
        self.audio_buffer.clear();
        self.partial_result.clear();
        self.is_processing = false;
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_model(&mut self, model: RecognitionModel) {
        self.model = model;
    }

    pub fn get_model(&self) -> RecognitionModel {
        self.model
    }

    pub fn set_vad_enabled(&mut self, enabled: bool) {
        self.vad_enabled = enabled;
    }

    pub fn set_noise_threshold(&mut self, threshold: f32) {
        self.noise_threshold = threshold.clamp(0.001, 0.5);
    }

    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    pub fn get_voice_activity(&self, samples: &[f32]) -> VoiceActivity {
        if samples.is_empty() {
            return VoiceActivity::Silence;
        }

        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        let rms = energy.sqrt();

        if rms < self.noise_threshold * 0.5 {
            VoiceActivity::Silence
        } else if rms < self.noise_threshold {
            VoiceActivity::Noise
        } else {
            VoiceActivity::Speech
        }
    }
}

/// Keyword spotter for wake words and commands
pub struct KeywordSpotter {
    keywords: Vec<String>,
    threshold: f32,
    detected_keyword: Option<String>,
}

impl KeywordSpotter {
    pub fn new() -> Self {
        Self {
            keywords: Vec::new(),
            threshold: 0.8,
            detected_keyword: None,
        }
    }

    pub fn add_keyword(&mut self, keyword: &str) {
        self.keywords.push(keyword.to_lowercase());
    }

    pub fn remove_keyword(&mut self, keyword: &str) {
        self.keywords.retain(|k| k != &keyword.to_lowercase());
    }

    pub fn process(&mut self, _samples: &[f32]) -> Option<&str> {
        // Simulated keyword detection
        self.detected_keyword = None;
        self.detected_keyword.as_deref()
    }

    pub fn check_text(&self, text: &str) -> Option<&str> {
        let lower = text.to_lowercase();
        for keyword in &self.keywords {
            if lower.contains(keyword) {
                return Some(keyword);
            }
        }
        None
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.5, 1.0);
    }

    pub fn get_keywords(&self) -> &[String] {
        &self.keywords
    }
}

impl Default for KeywordSpotter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognizer_creation() {
        let recognizer = SpeechRecognizer::new("en-US");
        assert_eq!(recognizer.get_language(), "en-US");
        assert_eq!(recognizer.get_model(), RecognitionModel::Accurate);
    }

    #[test]
    fn test_recognizer_with_model() {
        let recognizer = SpeechRecognizer::with_model("en-US", RecognitionModel::Fast);
        assert_eq!(recognizer.get_model(), RecognitionModel::Fast);
    }

    #[test]
    fn test_process_empty() {
        let mut recognizer = SpeechRecognizer::new("en-US");
        let result = recognizer.process(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_vad_silence() {
        let recognizer = SpeechRecognizer::new("en-US");
        let samples = vec![0.001; 1000];
        let activity = recognizer.get_voice_activity(&samples);
        assert_eq!(activity, VoiceActivity::Silence);
    }

    #[test]
    fn test_vad_speech() {
        let recognizer = SpeechRecognizer::new("en-US");
        let samples = vec![0.1; 1000];
        let activity = recognizer.get_voice_activity(&samples);
        assert_eq!(activity, VoiceActivity::Speech);
    }

    #[test]
    fn test_reset() {
        let mut recognizer = SpeechRecognizer::new("en-US");
        recognizer.process(&[0.1; 1000]);
        recognizer.reset();
        assert!(recognizer.get_partial_result().is_empty());
    }

    #[test]
    fn test_language_change() {
        let mut recognizer = SpeechRecognizer::new("en-US");
        recognizer.set_language("es-ES");
        assert_eq!(recognizer.get_language(), "es-ES");
    }

    #[test]
    fn test_keyword_spotter() {
        let mut spotter = KeywordSpotter::new();
        spotter.add_keyword("hey karana");
        
        let found = spotter.check_text("Hey Karana, what's the weather?");
        assert!(found.is_some());
    }

    #[test]
    fn test_keyword_spotter_not_found() {
        let mut spotter = KeywordSpotter::new();
        spotter.add_keyword("hey karana");
        
        let found = spotter.check_text("Hello there");
        assert!(found.is_none());
    }

    #[test]
    fn test_remove_keyword() {
        let mut spotter = KeywordSpotter::new();
        spotter.add_keyword("hey karana");
        spotter.remove_keyword("hey karana");
        
        assert!(spotter.get_keywords().is_empty());
    }

    #[test]
    fn test_noise_threshold() {
        let mut recognizer = SpeechRecognizer::new("en-US");
        recognizer.set_noise_threshold(0.1);
        
        let samples = vec![0.05; 1000];
        assert!(!recognizer.detect_speech(&samples));
        
        let loud_samples = vec![0.2; 1000];
        assert!(recognizer.detect_speech(&loud_samples));
    }
}
