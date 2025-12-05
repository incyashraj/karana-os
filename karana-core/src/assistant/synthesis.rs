// Speech Synthesis for Kāraṇa OS
// Text-to-speech generation for voice responses

use std::collections::HashMap;

/// Voice characteristics
#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub voice_id: String,
    pub name: String,
    pub language: String,
    pub gender: VoiceGender,
    pub age: VoiceAge,
    pub style: VoiceStyle,
    pub pitch_shift: f32,
    pub speed_multiplier: f32,
}

impl Default for VoiceProfile {
    fn default() -> Self {
        Self {
            voice_id: "default".to_string(),
            name: "Karana".to_string(),
            language: "en-US".to_string(),
            gender: VoiceGender::Neutral,
            age: VoiceAge::Adult,
            style: VoiceStyle::Conversational,
            pitch_shift: 0.0,
            speed_multiplier: 1.0,
        }
    }
}

/// Voice gender
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}

/// Voice age characteristic
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceAge {
    Child,
    Young,
    Adult,
    Senior,
}

/// Voice speaking style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceStyle {
    Conversational,
    Formal,
    Friendly,
    Calm,
    Energetic,
    Whisper,
}

/// Speech synthesis parameters
#[derive(Debug, Clone)]
pub struct SynthesisParams {
    pub pitch: f32,      // -1.0 to 1.0
    pub rate: f32,       // 0.5 to 2.0
    pub volume: f32,     // 0.0 to 1.0
    pub emphasis: EmphasisLevel,
}

impl Default for SynthesisParams {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            rate: 1.0,
            volume: 1.0,
            emphasis: EmphasisLevel::Normal,
        }
    }
}

/// Emphasis level for speech
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmphasisLevel {
    None,
    Reduced,
    Normal,
    Moderate,
    Strong,
}

/// SSML tag for speech markup
#[derive(Debug, Clone)]
pub enum SsmlTag {
    Break { time_ms: u32 },
    Emphasis(EmphasisLevel),
    Prosody { pitch: Option<f32>, rate: Option<f32>, volume: Option<f32> },
    SayAs { interpret_as: String },
    Phoneme { alphabet: String, ph: String },
}

/// Speech synthesizer
pub struct SpeechSynthesizer {
    voice: VoiceProfile,
    params: SynthesisParams,
    sample_rate: u32,
    available_voices: HashMap<String, VoiceProfile>,
    cache_enabled: bool,
    cache: HashMap<String, Vec<f32>>,
    max_cache_entries: usize,
}

impl SpeechSynthesizer {
    pub fn new(voice_id: &str) -> Self {
        let mut available_voices = HashMap::new();
        
        // Add default voices
        let default_voice = VoiceProfile::default();
        available_voices.insert("default".to_string(), default_voice.clone());
        
        let formal_voice = VoiceProfile {
            voice_id: "formal".to_string(),
            name: "Professional".to_string(),
            style: VoiceStyle::Formal,
            ..Default::default()
        };
        available_voices.insert("formal".to_string(), formal_voice);

        let voice = available_voices.get(voice_id)
            .cloned()
            .unwrap_or_else(VoiceProfile::default);

        Self {
            voice,
            params: SynthesisParams::default(),
            sample_rate: 22050,
            available_voices,
            cache_enabled: true,
            cache: HashMap::new(),
            max_cache_entries: 100,
        }
    }

    pub fn synthesize(&mut self, text: &str) -> Option<Vec<f32>> {
        if text.is_empty() {
            return None;
        }

        // Check cache
        let cache_key = format!("{}:{}", self.voice.voice_id, text);
        if self.cache_enabled {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Some(cached.clone());
            }
        }

        // Generate audio (simulated)
        let audio = self.generate_audio(text);

        // Cache result
        if self.cache_enabled && self.cache.len() < self.max_cache_entries {
            self.cache.insert(cache_key, audio.clone());
        }

        Some(audio)
    }

    fn generate_audio(&self, text: &str) -> Vec<f32> {
        // Simulated audio generation
        // In real implementation, would use TTS model
        let duration_samples = (text.len() as f32 * 0.1 * self.sample_rate as f32) as usize;
        let duration_samples = duration_samples.max(self.sample_rate as usize / 4);
        
        vec![0.0; duration_samples]
    }

    pub fn synthesize_ssml(&mut self, ssml: &str) -> Option<Vec<f32>> {
        // Parse SSML and synthesize
        // For now, strip tags and synthesize plain text
        let text = self.strip_ssml_tags(ssml);
        self.synthesize(&text)
    }

    fn strip_ssml_tags(&self, ssml: &str) -> String {
        // Simple tag stripping
        let mut result = String::new();
        let mut in_tag = false;
        
        for c in ssml.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        
        result
    }

    pub fn set_voice(&mut self, voice_id: &str) -> bool {
        if let Some(voice) = self.available_voices.get(voice_id) {
            self.voice = voice.clone();
            true
        } else {
            false
        }
    }

    pub fn get_voice(&self) -> &VoiceProfile {
        &self.voice
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.params.pitch = pitch.clamp(-1.0, 1.0);
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.params.rate = rate.clamp(0.5, 2.0);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.params.volume = volume.clamp(0.0, 1.0);
    }

    pub fn get_params(&self) -> &SynthesisParams {
        &self.params
    }

    pub fn add_voice(&mut self, voice: VoiceProfile) {
        self.available_voices.insert(voice.voice_id.clone(), voice);
    }

    pub fn get_available_voices(&self) -> Vec<&VoiceProfile> {
        self.available_voices.values().collect()
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if !enabled {
            self.cache.clear();
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn estimate_duration(&self, text: &str) -> f32 {
        // Estimate speech duration in seconds
        let words = text.split_whitespace().count();
        let words_per_second = 2.5 * self.params.rate;
        words as f32 / words_per_second
    }
}

impl Default for SpeechSynthesizer {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_creation() {
        let synth = SpeechSynthesizer::new("default");
        assert_eq!(synth.get_voice().voice_id, "default");
    }

    #[test]
    fn test_synthesize_text() {
        let mut synth = SpeechSynthesizer::new("default");
        let audio = synth.synthesize("Hello, world!");
        assert!(audio.is_some());
        assert!(!audio.unwrap().is_empty());
    }

    #[test]
    fn test_synthesize_empty() {
        let mut synth = SpeechSynthesizer::new("default");
        let audio = synth.synthesize("");
        assert!(audio.is_none());
    }

    #[test]
    fn test_set_voice() {
        let mut synth = SpeechSynthesizer::new("default");
        assert!(synth.set_voice("formal"));
        assert_eq!(synth.get_voice().voice_id, "formal");
    }

    #[test]
    fn test_invalid_voice() {
        let mut synth = SpeechSynthesizer::new("default");
        assert!(!synth.set_voice("nonexistent"));
    }

    #[test]
    fn test_pitch_clamping() {
        let mut synth = SpeechSynthesizer::new("default");
        synth.set_pitch(5.0);
        assert_eq!(synth.get_params().pitch, 1.0);
        
        synth.set_pitch(-5.0);
        assert_eq!(synth.get_params().pitch, -1.0);
    }

    #[test]
    fn test_rate_clamping() {
        let mut synth = SpeechSynthesizer::new("default");
        synth.set_rate(0.1);
        assert_eq!(synth.get_params().rate, 0.5);
        
        synth.set_rate(5.0);
        assert_eq!(synth.get_params().rate, 2.0);
    }

    #[test]
    fn test_cache() {
        let mut synth = SpeechSynthesizer::new("default");
        
        // First call
        let audio1 = synth.synthesize("Hello");
        // Second call should use cache
        let audio2 = synth.synthesize("Hello");
        
        assert_eq!(audio1, audio2);
    }

    #[test]
    fn test_cache_disabled() {
        let mut synth = SpeechSynthesizer::new("default");
        synth.set_cache_enabled(false);
        
        let audio = synth.synthesize("Hello");
        assert!(audio.is_some());
    }

    #[test]
    fn test_ssml_synthesis() {
        let mut synth = SpeechSynthesizer::new("default");
        let ssml = "<speak>Hello <break time='500ms'/> world</speak>";
        let audio = synth.synthesize_ssml(ssml);
        assert!(audio.is_some());
    }

    #[test]
    fn test_duration_estimate() {
        let synth = SpeechSynthesizer::new("default");
        let duration = synth.estimate_duration("This is a test sentence with several words");
        assert!(duration > 0.0);
    }

    #[test]
    fn test_add_voice() {
        let mut synth = SpeechSynthesizer::new("default");
        let custom_voice = VoiceProfile {
            voice_id: "custom".to_string(),
            name: "Custom Voice".to_string(),
            ..Default::default()
        };
        
        synth.add_voice(custom_voice);
        assert!(synth.set_voice("custom"));
    }

    #[test]
    fn test_available_voices() {
        let synth = SpeechSynthesizer::new("default");
        let voices = synth.get_available_voices();
        assert!(voices.len() >= 2);
    }
}
