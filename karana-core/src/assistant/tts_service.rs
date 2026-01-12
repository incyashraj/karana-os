// Kāraṇa OS - Text-to-Speech Service
// Natural voice synthesis using system TTS or external engines

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// TTS voice profile
#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: VoiceGender,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}

/// TTS configuration
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// Voice to use
    pub voice_id: String,
    /// Speech rate (0.5-2.0, 1.0 = normal)
    pub rate: f32,
    /// Pitch (-1.0 to 1.0, 0.0 = normal)
    pub pitch: f32,
    /// Volume (0.0-1.0)
    pub volume: f32,
    /// Sample rate for output
    pub sample_rate: u32,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            voice_id: "default".to_string(),
            rate: 1.0,
            pitch: 0.0,
            volume: 1.0,
            sample_rate: 22050,
        }
    }
}

/// TTS Engine trait
pub trait TtsEngine: Send + Sync {
    /// Get available voices
    fn available_voices(&self) -> Vec<VoiceProfile>;
    
    /// Synthesize text to audio samples
    fn synthesize(&self, text: &str, config: &TtsConfig) -> Result<Vec<f32>>;
    
    /// Check if engine is available
    fn is_available(&self) -> bool;
}

/// System TTS Engine (uses OS TTS - fallback)
pub struct SystemTtsEngine {
    available: bool,
}

impl SystemTtsEngine {
    pub fn new() -> Self {
        Self {
            available: true, // Always available as fallback
        }
    }
}

impl TtsEngine for SystemTtsEngine {
    fn available_voices(&self) -> Vec<VoiceProfile> {
        vec![
            VoiceProfile {
                id: "system_default".to_string(),
                name: "System Default".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Neutral,
                sample_rate: 22050,
            }
        ]
    }

    fn synthesize(&self, text: &str, _config: &TtsConfig) -> Result<Vec<f32>> {
        log::warn!("[TTS] System TTS requires external process, returning empty audio");
        log::info!("[TTS] Would speak: '{}'", text);
        
        // In a real implementation, this would:
        // 1. Call system TTS (espeak, say, etc.)
        // 2. Save to temp file
        // 3. Load and return audio samples
        
        // For now, return silence as placeholder
        Ok(vec![0.0; 22050]) // 1 second of silence
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

impl Default for SystemTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock TTS Engine for testing
pub struct MockTtsEngine;

impl MockTtsEngine {
    pub fn new() -> Self {
        Self
    }
}

impl TtsEngine for MockTtsEngine {
    fn available_voices(&self) -> Vec<VoiceProfile> {
        vec![
            VoiceProfile {
                id: "mock_voice".to_string(),
                name: "Mock Voice".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Neutral,
                sample_rate: 16000,
            }
        ]
    }

    fn synthesize(&self, text: &str, config: &TtsConfig) -> Result<Vec<f32>> {
        log::info!("[TTS] Mock synthesis: '{}' (rate: {}, pitch: {})", 
            text, config.rate, config.pitch);
        
        // Generate simple tone as placeholder
        let duration_secs = 1.0;
        let num_samples = (config.sample_rate as f32 * duration_secs) as usize;
        let frequency = 440.0; // A4 note
        
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / config.sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.1 * config.volume
            })
            .collect();
        
        Ok(samples)
    }

    fn is_available(&self) -> bool {
        true
    }
}

impl Default for MockTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// TTS Service - manages TTS engines and synthesis
pub struct TtsService {
    engine: Arc<Mutex<Box<dyn TtsEngine>>>,
    config: Arc<Mutex<TtsConfig>>,
    cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
}

impl TtsService {
    /// Create new TTS service with default engine
    pub fn new() -> Self {
        Self::with_engine(Box::new(SystemTtsEngine::new()))
    }

    /// Create TTS service with specific engine
    pub fn with_engine(engine: Box<dyn TtsEngine>) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            config: Arc::new(Mutex::new(TtsConfig::default())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create TTS service with mock engine (for testing)
    pub fn mock() -> Self {
        Self::with_engine(Box::new(MockTtsEngine::new()))
    }

    /// Get available voices
    pub async fn get_voices(&self) -> Vec<VoiceProfile> {
        let engine = self.engine.lock().await;
        engine.available_voices()
    }

    /// Set voice
    pub async fn set_voice(&self, voice_id: &str) {
        let mut config = self.config.lock().await;
        config.voice_id = voice_id.to_string();
    }

    /// Set speech rate
    pub async fn set_rate(&self, rate: f32) {
        let mut config = self.config.lock().await;
        config.rate = rate.clamp(0.5, 2.0);
    }

    /// Set pitch
    pub async fn set_pitch(&self, pitch: f32) {
        let mut config = self.config.lock().await;
        config.pitch = pitch.clamp(-1.0, 1.0);
    }

    /// Set volume
    pub async fn set_volume(&self, volume: f32) {
        let mut config = self.config.lock().await;
        config.volume = volume.clamp(0.0, 1.0);
    }

    /// Synthesize text to audio
    pub async fn speak(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(text) {
                log::debug!("[TTS] Using cached audio for: '{}'", text);
                return Ok(cached.clone());
            }
        }

        // Synthesize
        let engine = self.engine.lock().await;
        let config = self.config.lock().await;
        
        log::info!("[TTS] Synthesizing: '{}'", text);
        let audio = engine.synthesize(text, &config)?;

        // Cache result
        {
            let mut cache = self.cache.lock().await;
            cache.insert(text.to_string(), audio.clone());
            
            // Limit cache size
            if cache.len() > 50 {
                // Remove oldest (random) entry
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }
        }

        Ok(audio)
    }

    /// Speak text without caching (for dynamic content)
    pub async fn speak_uncached(&self, text: &str) -> Result<Vec<f32>> {
        let engine = self.engine.lock().await;
        let config = self.config.lock().await;
        engine.synthesize(text, &config)
    }

    /// Check if TTS is available
    pub async fn is_available(&self) -> bool {
        let engine = self.engine.lock().await;
        engine.is_available()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Get current config
    pub async fn get_config(&self) -> TtsConfig {
        self.config.lock().await.clone()
    }
}

impl Default for TtsService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to format text for better TTS
pub fn format_for_tts(text: &str) -> String {
    let mut formatted = text.to_string();
    
    // Add pauses after punctuation
    formatted = formatted.replace(". ", "... ");
    formatted = formatted.replace("! ", "... ");
    formatted = formatted.replace("? ", "... ");
    
    // Expand common abbreviations
    formatted = formatted.replace("etc.", "et cetera");
    formatted = formatted.replace("e.g.", "for example");
    formatted = formatted.replace("i.e.", "that is");
    
    // Handle URLs (don't read them)
    if formatted.contains("http://") || formatted.contains("https://") {
        formatted = formatted
            .split_whitespace()
            .filter(|word| !word.contains("http://") && !word.contains("https://"))
            .collect::<Vec<_>>()
            .join(" ");
    }
    
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tts_service() {
        let tts = TtsService::mock();
        assert!(tts.is_available().await);
        
        let voices = tts.get_voices().await;
        assert!(!voices.is_empty());
    }

    #[tokio::test]
    async fn test_speak() {
        let tts = TtsService::mock();
        let audio = tts.speak("Hello world").await;
        assert!(audio.is_ok());
        
        let samples = audio.unwrap();
        assert!(!samples.is_empty());
    }

    #[tokio::test]
    async fn test_config() {
        let tts = TtsService::mock();
        
        tts.set_rate(1.5).await;
        tts.set_pitch(0.5).await;
        tts.set_volume(0.8).await;
        
        let config = tts.get_config().await;
        assert_eq!(config.rate, 1.5);
        assert_eq!(config.pitch, 0.5);
        assert_eq!(config.volume, 0.8);
    }

    #[tokio::test]
    async fn test_cache() {
        let tts = TtsService::mock();
        
        // First call - not cached
        let audio1 = tts.speak("Test message").await.unwrap();
        
        // Second call - should be cached
        let audio2 = tts.speak("Test message").await.unwrap();
        
        assert_eq!(audio1.len(), audio2.len());
    }

    #[test]
    fn test_format_for_tts() {
        let text = "Hello. This is a test! Visit http://example.com for more info.";
        let formatted = format_for_tts(text);
        
        assert!(formatted.contains("..."));
        assert!(!formatted.contains("http://"));
    }

    #[test]
    fn test_voice_gender() {
        let voice = VoiceProfile {
            id: "test".to_string(),
            name: "Test Voice".to_string(),
            language: "en-US".to_string(),
            gender: VoiceGender::Female,
            sample_rate: 22050,
        };
        
        assert_eq!(voice.gender, VoiceGender::Female);
    }
}
