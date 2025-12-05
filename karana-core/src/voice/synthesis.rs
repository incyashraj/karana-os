//! Voice Synthesis and Text-to-Speech
//!
//! Speech synthesis for voice feedback and responses.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Text-to-speech synthesizer
pub struct VoiceSynthesizer {
    /// Voice configuration
    config: SynthConfig,
    /// Available voices
    voices: Vec<Voice>,
    /// Current voice
    current_voice: usize,
    /// Speech queue
    queue: Vec<SpeechItem>,
    /// Pronunciation dictionary
    pronunciations: HashMap<String, String>,
    /// State
    state: SynthState,
}

impl VoiceSynthesizer {
    /// Create new synthesizer
    pub fn new(config: SynthConfig) -> Self {
        Self {
            config,
            voices: Self::default_voices(),
            current_voice: 0,
            queue: Vec::new(),
            pronunciations: Self::default_pronunciations(),
            state: SynthState::Idle,
        }
    }

    /// Speak text
    pub fn speak(&mut self, text: &str) -> SpeechHandle {
        let item = SpeechItem {
            id: self.generate_id(),
            text: text.to_string(),
            ssml: None,
            priority: SpeechPriority::Normal,
            voice: self.current_voice,
        };

        let handle = SpeechHandle { id: item.id };
        self.queue.push(item);
        self.process_queue();

        handle
    }

    /// Speak with priority
    pub fn speak_priority(&mut self, text: &str, priority: SpeechPriority) -> SpeechHandle {
        let item = SpeechItem {
            id: self.generate_id(),
            text: text.to_string(),
            ssml: None,
            priority,
            voice: self.current_voice,
        };

        let handle = SpeechHandle { id: item.id };

        // Insert based on priority
        let pos = self.queue.iter().position(|i| i.priority < priority)
            .unwrap_or(self.queue.len());
        self.queue.insert(pos, item);

        if priority == SpeechPriority::Interrupt {
            self.interrupt();
        }

        self.process_queue();
        handle
    }

    /// Speak with SSML
    pub fn speak_ssml(&mut self, ssml: &str) -> SpeechHandle {
        let item = SpeechItem {
            id: self.generate_id(),
            text: String::new(),
            ssml: Some(ssml.to_string()),
            priority: SpeechPriority::Normal,
            voice: self.current_voice,
        };

        let handle = SpeechHandle { id: item.id };
        self.queue.push(item);
        self.process_queue();

        handle
    }

    /// Stop speaking
    pub fn stop(&mut self) {
        self.queue.clear();
        self.state = SynthState::Idle;
    }

    /// Pause speaking
    pub fn pause(&mut self) {
        if self.state == SynthState::Speaking {
            self.state = SynthState::Paused;
        }
    }

    /// Resume speaking
    pub fn resume(&mut self) {
        if self.state == SynthState::Paused {
            self.state = SynthState::Speaking;
            self.process_queue();
        }
    }

    /// Interrupt current speech
    fn interrupt(&mut self) {
        if self.state == SynthState::Speaking {
            // Would stop audio here
            self.state = SynthState::Idle;
        }
    }

    /// Set voice by name
    pub fn set_voice(&mut self, name: &str) -> bool {
        if let Some(idx) = self.voices.iter().position(|v| v.name == name) {
            self.current_voice = idx;
            true
        } else {
            false
        }
    }

    /// Set speech rate
    pub fn set_rate(&mut self, rate: f32) {
        self.config.rate = rate.clamp(0.5, 2.0);
    }

    /// Set pitch
    pub fn set_pitch(&mut self, pitch: f32) {
        self.config.pitch = pitch.clamp(0.5, 2.0);
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 1.0);
    }

    /// Get available voices
    pub fn voices(&self) -> &[Voice] {
        &self.voices
    }

    /// Get current state
    pub fn state(&self) -> SynthState {
        self.state
    }

    /// Process speech queue
    fn process_queue(&mut self) {
        if self.state != SynthState::Idle || self.queue.is_empty() {
            return;
        }

        if let Some(item) = self.queue.first() {
            self.state = SynthState::Speaking;

            // Get text to synthesize
            let text = if let Some(ssml) = &item.ssml {
                self.parse_ssml(ssml)
            } else {
                self.preprocess_text(&item.text)
            };

            // Generate speech (simulated)
            let _audio = self.synthesize(&text, item.voice);

            // In real implementation, this would play audio
            // For now, just move to next item
        }
    }

    /// Preprocess text for synthesis
    fn preprocess_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Expand abbreviations
        result = result.replace("Dr.", "Doctor");
        result = result.replace("Mr.", "Mister");
        result = result.replace("Mrs.", "Missus");
        result = result.replace("Ms.", "Miss");
        result = result.replace("vs.", "versus");
        result = result.replace("etc.", "etcetera");

        // Apply custom pronunciations
        for (word, pronunciation) in &self.pronunciations {
            result = result.replace(word, pronunciation);
        }

        // Expand numbers
        result = self.expand_numbers(&result);

        result
    }

    /// Expand numbers to words
    fn expand_numbers(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                let mut num = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                result.push_str(&self.number_to_words(&num));
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Convert number to words
    fn number_to_words(&self, num_str: &str) -> String {
        let ones = ["", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
        let teens = ["ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
                     "sixteen", "seventeen", "eighteen", "nineteen"];
        let tens = ["", "", "twenty", "thirty", "forty", "fifty",
                    "sixty", "seventy", "eighty", "ninety"];

        if let Ok(num) = num_str.parse::<i32>() {
            if num == 0 {
                return "zero".to_string();
            }
            if num < 10 {
                return ones[num as usize].to_string();
            }
            if num < 20 {
                return teens[(num - 10) as usize].to_string();
            }
            if num < 100 {
                let t = tens[(num / 10) as usize];
                let o = if num % 10 > 0 { format!("-{}", ones[(num % 10) as usize]) } else { String::new() };
                return format!("{}{}", t, o);
            }
            // For larger numbers, just return digits
            return num_str.to_string();
        }

        num_str.to_string()
    }

    /// Parse SSML markup
    fn parse_ssml(&self, ssml: &str) -> String {
        // Simple SSML parsing - strip tags and extract text
        let mut result = String::new();
        let mut in_tag = false;

        for c in ssml.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag {
                result.push(c);
            }
        }

        result
    }

    /// Synthesize speech
    fn synthesize(&self, text: &str, voice_idx: usize) -> SynthesizedAudio {
        let voice = &self.voices[voice_idx.min(self.voices.len() - 1)];

        // Calculate approximate duration (words per minute)
        let word_count = text.split_whitespace().count();
        let wpm = 150.0 * self.config.rate;
        let duration = (word_count as f32 / wpm) * 60.0;

        SynthesizedAudio {
            sample_rate: 22050,
            channels: 1,
            duration,
            format: AudioFormat::PCM16,
            voice_id: voice.id.clone(),
            samples: Vec::new(), // Would contain actual audio
        }
    }

    /// Generate unique ID
    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    /// Default voices
    fn default_voices() -> Vec<Voice> {
        vec![
            Voice {
                id: "system-default".to_string(),
                name: "System Default".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Neutral,
                quality: VoiceQuality::High,
            },
            Voice {
                id: "assistant-female".to_string(),
                name: "Assistant (Female)".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Female,
                quality: VoiceQuality::High,
            },
            Voice {
                id: "assistant-male".to_string(),
                name: "Assistant (Male)".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Male,
                quality: VoiceQuality::High,
            },
            Voice {
                id: "compact".to_string(),
                name: "Compact".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Neutral,
                quality: VoiceQuality::Low,
            },
        ]
    }

    /// Default pronunciations
    fn default_pronunciations() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("Karana".to_string(), "kah-rah-nah".to_string());
        map.insert("AR".to_string(), "A R".to_string());
        map.insert("UI".to_string(), "U I".to_string());
        map.insert("API".to_string(), "A P I".to_string());
        map.insert("WiFi".to_string(), "why-fye".to_string());
        map.insert("iOS".to_string(), "eye O S".to_string());
        map
    }
}

impl Default for VoiceSynthesizer {
    fn default() -> Self {
        Self::new(SynthConfig::default())
    }
}

/// Synthesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthConfig {
    /// Speech rate (0.5 - 2.0)
    pub rate: f32,
    /// Pitch (0.5 - 2.0)
    pub pitch: f32,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Use SSML
    pub use_ssml: bool,
    /// Default voice
    pub default_voice: String,
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self {
            rate: 1.0,
            pitch: 1.0,
            volume: 0.8,
            use_ssml: false,
            default_voice: "system-default".to_string(),
        }
    }
}

/// Voice definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Language code
    pub language: String,
    /// Voice gender
    pub gender: VoiceGender,
    /// Voice quality
    pub quality: VoiceQuality,
}

/// Voice gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}

/// Voice quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceQuality {
    Low,
    Medium,
    High,
    Premium,
}

/// Speech item in queue
struct SpeechItem {
    /// Unique ID
    id: u64,
    /// Text to speak
    text: String,
    /// SSML markup
    ssml: Option<String>,
    /// Priority
    priority: SpeechPriority,
    /// Voice index
    voice: usize,
}

/// Speech priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpeechPriority {
    /// Low priority, queued
    Low,
    /// Normal priority
    Normal,
    /// High priority, moved to front
    High,
    /// Interrupt current speech
    Interrupt,
}

/// Speech handle for tracking
#[derive(Debug, Clone)]
pub struct SpeechHandle {
    /// Speech ID
    pub id: u64,
}

/// Synthesized audio
pub struct SynthesizedAudio {
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Duration in seconds
    pub duration: f32,
    /// Audio format
    pub format: AudioFormat,
    /// Voice used
    pub voice_id: String,
    /// Audio samples
    pub samples: Vec<i16>,
}

/// Audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PCM16,
    PCM32,
    Float32,
}

/// Synthesis state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthState {
    /// Not speaking
    Idle,
    /// Currently speaking
    Speaking,
    /// Paused
    Paused,
}

/// SSML builder for generating speech markup
pub struct SsmlBuilder {
    content: String,
}

impl SsmlBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            content: String::from("<speak>"),
        }
    }

    /// Add text
    pub fn text(mut self, text: &str) -> Self {
        self.content.push_str(text);
        self
    }

    /// Add break/pause
    pub fn pause(mut self, ms: u32) -> Self {
        self.content.push_str(&format!("<break time=\"{}ms\"/>", ms));
        self
    }

    /// Add emphasis
    pub fn emphasis(mut self, text: &str, level: EmphasisLevel) -> Self {
        let level_str = match level {
            EmphasisLevel::Strong => "strong",
            EmphasisLevel::Moderate => "moderate",
            EmphasisLevel::Reduced => "reduced",
        };
        self.content.push_str(&format!("<emphasis level=\"{}\">{}</emphasis>", level_str, text));
        self
    }

    /// Add prosody (rate, pitch, volume)
    pub fn prosody(mut self, text: &str, rate: Option<f32>, pitch: Option<f32>) -> Self {
        let mut attrs = String::new();
        if let Some(r) = rate {
            attrs.push_str(&format!(" rate=\"{}%\"", (r * 100.0) as i32));
        }
        if let Some(p) = pitch {
            let st = ((p - 1.0) * 12.0) as i32;
            attrs.push_str(&format!(" pitch=\"{}st\"", st));
        }
        self.content.push_str(&format!("<prosody{}>{}</prosody>", attrs, text));
        self
    }

    /// Say as (number, date, etc.)
    pub fn say_as(mut self, text: &str, interpret_as: &str) -> Self {
        self.content.push_str(&format!("<say-as interpret-as=\"{}\">{}</say-as>", interpret_as, text));
        self
    }

    /// Phoneme (custom pronunciation)
    pub fn phoneme(mut self, text: &str, pronunciation: &str) -> Self {
        self.content.push_str(&format!("<phoneme ph=\"{}\">{}</phoneme>", pronunciation, text));
        self
    }

    /// Build final SSML
    pub fn build(mut self) -> String {
        self.content.push_str("</speak>");
        self.content
    }
}

impl Default for SsmlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Emphasis level for SSML
#[derive(Debug, Clone, Copy)]
pub enum EmphasisLevel {
    Strong,
    Moderate,
    Reduced,
}

/// Response generator for natural voice feedback
pub struct ResponseGenerator {
    templates: HashMap<ResponseType, Vec<String>>,
}

impl ResponseGenerator {
    /// Create new generator
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert(ResponseType::Acknowledgment, vec![
            "Got it.".to_string(),
            "Okay.".to_string(),
            "Sure.".to_string(),
            "Done.".to_string(),
        ]);

        templates.insert(ResponseType::Confirmation, vec![
            "Do you want me to proceed?".to_string(),
            "Should I continue?".to_string(),
            "Is that correct?".to_string(),
        ]);

        templates.insert(ResponseType::Error, vec![
            "Sorry, I couldn't do that.".to_string(),
            "Something went wrong.".to_string(),
            "I encountered an error.".to_string(),
        ]);

        templates.insert(ResponseType::NotUnderstood, vec![
            "I didn't understand that.".to_string(),
            "Could you repeat that?".to_string(),
            "I'm not sure what you mean.".to_string(),
        ]);

        templates.insert(ResponseType::Greeting, vec![
            "Hello!".to_string(),
            "Hi there!".to_string(),
            "Hey!".to_string(),
        ]);

        templates.insert(ResponseType::Farewell, vec![
            "Goodbye!".to_string(),
            "See you!".to_string(),
            "Take care!".to_string(),
        ]);

        Self { templates }
    }

    /// Generate response
    pub fn generate(&self, response_type: ResponseType) -> String {
        if let Some(templates) = self.templates.get(&response_type) {
            let idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as usize)
                .unwrap_or(0)) % templates.len();
            templates[idx].clone()
        } else {
            "...".to_string()
        }
    }

    /// Generate with context
    pub fn generate_with_context(&self, response_type: ResponseType, context: &str) -> String {
        let base = self.generate(response_type);
        if context.is_empty() {
            base
        } else {
            format!("{} {}", base, context)
        }
    }
}

impl Default for ResponseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Response types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseType {
    Acknowledgment,
    Confirmation,
    Error,
    NotUnderstood,
    Greeting,
    Farewell,
    Progress,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_creation() {
        let synth = VoiceSynthesizer::default();
        assert_eq!(synth.state(), SynthState::Idle);
    }

    #[test]
    fn test_speak() {
        let mut synth = VoiceSynthesizer::default();
        let handle = synth.speak("Hello world");
        assert!(handle.id > 0);
    }

    #[test]
    fn test_number_to_words() {
        let synth = VoiceSynthesizer::default();
        assert_eq!(synth.number_to_words("5"), "five");
        assert_eq!(synth.number_to_words("15"), "fifteen");
        assert_eq!(synth.number_to_words("42"), "forty-two");
    }

    #[test]
    fn test_ssml_builder() {
        let ssml = SsmlBuilder::new()
            .text("Hello ")
            .emphasis("world", EmphasisLevel::Strong)
            .pause(500)
            .build();
        
        assert!(ssml.contains("<speak>"));
        assert!(ssml.contains("</speak>"));
        assert!(ssml.contains("<emphasis"));
    }

    #[test]
    fn test_voice_selection() {
        let mut synth = VoiceSynthesizer::default();
        assert!(synth.set_voice("Assistant (Female)"));
        assert!(!synth.set_voice("Nonexistent Voice"));
    }

    #[test]
    fn test_response_generator() {
        let generator = ResponseGenerator::default();
        let response = generator.generate(ResponseType::Acknowledgment);
        assert!(!response.is_empty());
    }

    #[test]
    fn test_synthesis_config() {
        let config = SynthConfig::default();
        assert_eq!(config.rate, 1.0);
        assert_eq!(config.pitch, 1.0);
    }
}
