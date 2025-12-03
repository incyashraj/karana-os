//! Hearing accessibility features for deaf and hard-of-hearing users

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Caption display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionStyle {
    /// Standard subtitles
    Standard,
    /// Large text
    Large,
    /// Outlined text
    Outlined,
    /// Boxed with background
    Boxed,
    /// High contrast
    HighContrast,
}

/// Caption position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionPosition {
    /// Bottom center
    BottomCenter,
    /// Top center
    TopCenter,
    /// Bottom left
    BottomLeft,
    /// Bottom right
    BottomRight,
}

/// Audio visualization type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioVisualization {
    /// No visualization
    None,
    /// Waveform display
    Waveform,
    /// Spectrum analyzer
    Spectrum,
    /// Volume meter
    VolumeMeter,
    /// Direction indicator
    DirectionIndicator,
}

/// Sound type for visual alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    /// Alert/notification
    Alert,
    /// Speech/voice
    Speech,
    /// Music
    Music,
    /// Environment sound
    Environment,
    /// Alarm/emergency
    Alarm,
    /// Unknown
    Unknown,
}

/// Visual sound indicator
#[derive(Debug, Clone)]
pub struct SoundIndicator {
    /// Sound type
    pub sound_type: SoundType,
    /// Direction in degrees (0 = forward, 90 = right)
    pub direction: f32,
    /// Intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Description
    pub description: String,
    /// When detected
    pub timestamp: Instant,
}

/// Caption entry
#[derive(Debug, Clone)]
pub struct Caption {
    /// Speaker identifier (if known)
    pub speaker: Option<String>,
    /// Caption text
    pub text: String,
    /// When the caption starts
    pub start_time: Instant,
    /// Duration to display
    pub duration: Duration,
    /// Whether this is final or being refined
    pub is_final: bool,
}

/// Hearing assistance system
#[derive(Debug)]
pub struct HearingAssist {
    /// Captions enabled
    captions_enabled: bool,
    /// Caption style
    caption_style: CaptionStyle,
    /// Caption position
    caption_position: CaptionPosition,
    /// Caption text size (1.0 = normal)
    caption_size: f32,
    /// Caption background opacity
    caption_opacity: f32,
    /// Show speaker labels
    show_speakers: bool,
    /// Audio visualization mode
    visualization: AudioVisualization,
    /// Visual flash for alerts
    flash_alerts: bool,
    /// Sound direction indicators
    sound_indicators_enabled: bool,
    /// Current captions
    captions: VecDeque<Caption>,
    /// Current sound indicators
    sound_indicators: VecDeque<SoundIndicator>,
    /// Maximum captions to show
    max_captions: usize,
    /// Maximum sound indicators
    max_indicators: usize,
    /// Mono audio enabled
    mono_audio: bool,
    /// Audio balance (-1.0 = left, 1.0 = right)
    audio_balance: f32,
}

impl HearingAssist {
    /// Create new hearing assist system
    pub fn new() -> Self {
        Self {
            captions_enabled: false,
            caption_style: CaptionStyle::Standard,
            caption_position: CaptionPosition::BottomCenter,
            caption_size: 1.0,
            caption_opacity: 0.75,
            show_speakers: true,
            visualization: AudioVisualization::None,
            flash_alerts: false,
            sound_indicators_enabled: false,
            captions: VecDeque::new(),
            sound_indicators: VecDeque::new(),
            max_captions: 3,
            max_indicators: 5,
            mono_audio: false,
            audio_balance: 0.0,
        }
    }
    
    /// Enable/disable captions
    pub fn set_captions_enabled(&mut self, enabled: bool) {
        self.captions_enabled = enabled;
    }
    
    /// Check if captions are enabled
    pub fn captions_enabled(&self) -> bool {
        self.captions_enabled
    }
    
    /// Set caption style
    pub fn set_caption_style(&mut self, style: CaptionStyle) {
        self.caption_style = style;
    }
    
    /// Get caption style
    pub fn caption_style(&self) -> CaptionStyle {
        self.caption_style
    }
    
    /// Set caption position
    pub fn set_caption_position(&mut self, position: CaptionPosition) {
        self.caption_position = position;
    }
    
    /// Get caption position
    pub fn caption_position(&self) -> CaptionPosition {
        self.caption_position
    }
    
    /// Set caption size (0.5 - 3.0)
    pub fn set_caption_size(&mut self, size: f32) {
        self.caption_size = size.clamp(0.5, 3.0);
    }
    
    /// Get caption size
    pub fn caption_size(&self) -> f32 {
        self.caption_size
    }
    
    /// Set caption background opacity
    pub fn set_caption_opacity(&mut self, opacity: f32) {
        self.caption_opacity = opacity.clamp(0.0, 1.0);
    }
    
    /// Set show speaker labels
    pub fn set_show_speakers(&mut self, show: bool) {
        self.show_speakers = show;
    }
    
    /// Check if speaker labels shown
    pub fn show_speakers(&self) -> bool {
        self.show_speakers
    }
    
    /// Set audio visualization mode
    pub fn set_visualization(&mut self, mode: AudioVisualization) {
        self.visualization = mode;
    }
    
    /// Get visualization mode
    pub fn visualization(&self) -> AudioVisualization {
        self.visualization
    }
    
    /// Enable/disable flash alerts
    pub fn set_flash_alerts(&mut self, enabled: bool) {
        self.flash_alerts = enabled;
    }
    
    /// Check if flash alerts enabled
    pub fn flash_alerts_enabled(&self) -> bool {
        self.flash_alerts
    }
    
    /// Enable/disable sound direction indicators
    pub fn set_sound_indicators(&mut self, enabled: bool) {
        self.sound_indicators_enabled = enabled;
    }
    
    /// Check if sound indicators enabled
    pub fn sound_indicators_enabled(&self) -> bool {
        self.sound_indicators_enabled
    }
    
    /// Add a caption
    pub fn add_caption(&mut self, speaker: Option<String>, text: String, duration: Duration, is_final: bool) {
        let caption = Caption {
            speaker,
            text,
            start_time: Instant::now(),
            duration,
            is_final,
        };
        
        // Remove old captions if at limit
        while self.captions.len() >= self.max_captions {
            self.captions.pop_front();
        }
        
        self.captions.push_back(caption);
    }
    
    /// Get current captions
    pub fn current_captions(&self) -> Vec<&Caption> {
        let now = Instant::now();
        self.captions
            .iter()
            .filter(|c| now.duration_since(c.start_time) < c.duration)
            .collect()
    }
    
    /// Clear all captions
    pub fn clear_captions(&mut self) {
        self.captions.clear();
    }
    
    /// Add sound indicator
    pub fn add_sound_indicator(&mut self, sound_type: SoundType, direction: f32, intensity: f32, description: String) {
        let indicator = SoundIndicator {
            sound_type,
            direction,
            intensity: intensity.clamp(0.0, 1.0),
            description,
            timestamp: Instant::now(),
        };
        
        while self.sound_indicators.len() >= self.max_indicators {
            self.sound_indicators.pop_front();
        }
        
        self.sound_indicators.push_back(indicator);
    }
    
    /// Get recent sound indicators
    pub fn recent_sound_indicators(&self, max_age: Duration) -> Vec<&SoundIndicator> {
        let now = Instant::now();
        self.sound_indicators
            .iter()
            .filter(|s| now.duration_since(s.timestamp) < max_age)
            .collect()
    }
    
    /// Enable/disable mono audio
    pub fn set_mono_audio(&mut self, enabled: bool) {
        self.mono_audio = enabled;
    }
    
    /// Check if mono audio enabled
    pub fn mono_audio_enabled(&self) -> bool {
        self.mono_audio
    }
    
    /// Set audio balance (-1.0 = left, 1.0 = right)
    pub fn set_audio_balance(&mut self, balance: f32) {
        self.audio_balance = balance.clamp(-1.0, 1.0);
    }
    
    /// Get audio balance
    pub fn audio_balance(&self) -> f32 {
        self.audio_balance
    }
    
    /// Calculate left/right channel gains
    pub fn channel_gains(&self) -> (f32, f32) {
        if self.mono_audio {
            (1.0, 1.0)
        } else {
            let left = if self.audio_balance <= 0.0 {
                1.0
            } else {
                1.0 - self.audio_balance
            };
            let right = if self.audio_balance >= 0.0 {
                1.0
            } else {
                1.0 + self.audio_balance
            };
            (left, right)
        }
    }
    
    /// Update - called every frame
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // Remove expired captions
        self.captions.retain(|c| now.duration_since(c.start_time) < c.duration);
        
        // Remove old sound indicators (older than 5 seconds)
        let max_age = Duration::from_secs(5);
        self.sound_indicators.retain(|s| now.duration_since(s.timestamp) < max_age);
    }
}

impl Default for HearingAssist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hearing_assist_creation() {
        let ha = HearingAssist::new();
        assert!(!ha.captions_enabled());
        assert!(!ha.mono_audio_enabled());
    }
    
    #[test]
    fn test_captions() {
        let mut ha = HearingAssist::new();
        ha.set_captions_enabled(true);
        
        ha.add_caption(
            Some("Speaker".to_string()),
            "Hello world".to_string(),
            Duration::from_secs(5),
            true,
        );
        
        let captions = ha.current_captions();
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].text, "Hello world");
    }
    
    #[test]
    fn test_caption_style() {
        let mut ha = HearingAssist::new();
        
        ha.set_caption_style(CaptionStyle::Boxed);
        assert_eq!(ha.caption_style(), CaptionStyle::Boxed);
        
        ha.set_caption_size(2.0);
        assert_eq!(ha.caption_size(), 2.0);
    }
    
    #[test]
    fn test_sound_indicators() {
        let mut ha = HearingAssist::new();
        ha.set_sound_indicators(true);
        
        ha.add_sound_indicator(
            SoundType::Alert,
            90.0,
            0.8,
            "Doorbell".to_string(),
        );
        
        let indicators = ha.recent_sound_indicators(Duration::from_secs(10));
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].description, "Doorbell");
    }
    
    #[test]
    fn test_audio_balance() {
        let mut ha = HearingAssist::new();
        
        // Center
        let (l, r) = ha.channel_gains();
        assert!((l - 1.0).abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
        
        // Pan right
        ha.set_audio_balance(0.5);
        let (l, r) = ha.channel_gains();
        assert!((l - 0.5).abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
        
        // Pan left
        ha.set_audio_balance(-0.5);
        let (l, r) = ha.channel_gains();
        assert!((l - 1.0).abs() < 0.01);
        assert!((r - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_mono_audio() {
        let mut ha = HearingAssist::new();
        
        ha.set_mono_audio(true);
        ha.set_audio_balance(-0.5);
        
        // Mono should ignore balance
        let (l, r) = ha.channel_gains();
        assert!((l - 1.0).abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
    }
}
