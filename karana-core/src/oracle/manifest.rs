//! Minimal Manifest - AR Whispers & Haptic Output
//!
//! This module handles OUTPUT from the Oracle to the user via:
//! - AR text overlays ("whispers") - minimal, non-intrusive
//! - Haptic patterns - tactile feedback
//! - Audio cues - subtle sounds
//!
//! Design Philosophy: "The Oracle whispers, it doesn't shout"
//! - No cluttered UI panels
//! - Subtle, context-aware feedback
//! - Haptic-first for confirmations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::oracle::command::{AROverlay, AROverlayType, HapticPattern, HapticPulse, WhisperStyle};

// ============================================================================
// MINIMAL MANIFEST
// ============================================================================

/// Output renderer for Oracle responses
pub struct MinimalManifest {
    /// Current AR overlays
    active_overlays: Vec<ActiveOverlay>,
    
    /// Haptic driver
    haptic: HapticDriver,
    
    /// Audio driver
    audio: AudioDriver,
    
    /// Configuration
    config: ManifestConfig,
    
    /// Output mode
    mode: OutputMode,
}

/// Configuration for manifest output
#[derive(Debug, Clone)]
pub struct ManifestConfig {
    /// Default whisper duration (ms)
    pub default_whisper_duration_ms: u64,
    
    /// Maximum concurrent overlays
    pub max_overlays: usize,
    
    /// Default whisper position
    pub default_position: (f32, f32),
    
    /// Haptic enabled
    pub haptic_enabled: bool,
    
    /// Audio enabled
    pub audio_enabled: bool,
    
    /// Privacy mode (minimal/no visual output)
    pub privacy_mode: bool,
}

impl Default for ManifestConfig {
    fn default() -> Self {
        Self {
            default_whisper_duration_ms: 3000,
            max_overlays: 3,
            default_position: (0.5, 0.85), // Bottom center
            haptic_enabled: true,
            audio_enabled: true,
            privacy_mode: false,
        }
    }
}

/// Output mode determines how responses are rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Full AR + haptic + audio
    Full,
    
    /// Minimal AR (small text only) + haptic
    Minimal,
    
    /// Haptic only (stealth mode)
    HapticOnly,
    
    /// Silent (log only)
    Silent,
}

/// Active overlay being displayed
struct ActiveOverlay {
    overlay: AROverlay,
    started_at: Instant,
    expires_at: Option<Instant>,
}

// ============================================================================
// HAPTIC DRIVER
// ============================================================================

/// Haptic feedback driver
pub struct HapticDriver {
    /// Whether hardware is available
    available: bool,
    
    /// Pattern library
    patterns: PatternLibrary,
    
    /// Last pattern played
    last_played: Option<(HapticPattern, Instant)>,
}

/// Pre-defined haptic patterns
pub struct PatternLibrary {
    pub success: Vec<HapticPulse>,
    pub confirm: Vec<HapticPulse>,
    pub error: Vec<HapticPulse>,
    pub attention: Vec<HapticPulse>,
    pub thinking: Vec<HapticPulse>,
    pub navigation_left: Vec<HapticPulse>,
    pub navigation_right: Vec<HapticPulse>,
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self {
            // Single short pulse - action completed
            success: vec![
                HapticPulse { duration_ms: 50, intensity: 0.8, pause_after_ms: 0 },
            ],
            
            // Double tap - confirmation needed
            confirm: vec![
                HapticPulse { duration_ms: 30, intensity: 0.6, pause_after_ms: 50 },
                HapticPulse { duration_ms: 30, intensity: 0.6, pause_after_ms: 0 },
            ],
            
            // Triple harsh - error
            error: vec![
                HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 40 },
                HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 40 },
                HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 0 },
            ],
            
            // Escalating pulse - get attention
            attention: vec![
                HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 100 },
                HapticPulse { duration_ms: 30, intensity: 0.5, pause_after_ms: 100 },
                HapticPulse { duration_ms: 40, intensity: 0.8, pause_after_ms: 0 },
            ],
            
            // Gentle repeating - processing
            thinking: vec![
                HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 200 },
                HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 200 },
                HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 0 },
            ],
            
            // Single tick - navigation left
            navigation_left: vec![
                HapticPulse { duration_ms: 15, intensity: 0.5, pause_after_ms: 0 },
            ],
            
            // Single tick - navigation right
            navigation_right: vec![
                HapticPulse { duration_ms: 15, intensity: 0.5, pause_after_ms: 0 },
            ],
        }
    }
}

impl HapticDriver {
    pub fn new() -> Self {
        Self {
            available: true, // Would probe hardware
            patterns: PatternLibrary::default(),
            last_played: None,
        }
    }
    
    /// Play a haptic pattern
    pub async fn play(&mut self, pattern: &HapticPattern) -> Result<()> {
        if !self.available {
            return Ok(());
        }
        
        let pulses = match pattern {
            HapticPattern::Success => &self.patterns.success,
            HapticPattern::Confirm => &self.patterns.confirm,
            HapticPattern::Error => &self.patterns.error,
            HapticPattern::Attention => &self.patterns.attention,
            HapticPattern::Thinking => &self.patterns.thinking,
            HapticPattern::Navigation { direction } => {
                match direction {
                    crate::oracle::command::NavigationDirection::Left => &self.patterns.navigation_left,
                    crate::oracle::command::NavigationDirection::Right => &self.patterns.navigation_right,
                    _ => &self.patterns.navigation_left,
                }
            }
            HapticPattern::Custom { pulses } => pulses,
        };
        
        // Execute pulses
        for pulse in pulses {
            self.vibrate(pulse.duration_ms, pulse.intensity)?;
            if pulse.pause_after_ms > 0 {
                tokio::time::sleep(Duration::from_millis(pulse.pause_after_ms as u64)).await;
            }
        }
        
        self.last_played = Some((pattern.clone(), Instant::now()));
        Ok(())
    }
    
    /// Low-level vibrate command
    fn vibrate(&self, duration_ms: u32, intensity: f32) -> Result<()> {
        // In production, this would call the hardware API
        log::debug!("[HAPTIC] Vibrate {}ms @ {:.0}%", duration_ms, intensity * 100.0);
        
        #[cfg(target_os = "android")]
        {
            // android_hal::haptic::vibrate(duration_ms, (intensity * 255.0) as u8)?;
        }
        
        Ok(())
    }
}

// ============================================================================
// AUDIO DRIVER
// ============================================================================

/// Audio feedback driver
pub struct AudioDriver {
    /// Whether audio is available
    available: bool,
    
    /// Volume level (0.0 - 1.0)
    volume: f32,
}

impl AudioDriver {
    pub fn new() -> Self {
        Self {
            available: true,
            volume: 0.3, // Low by default for subtlety
        }
    }
    
    /// Play a subtle audio cue
    pub fn play_cue(&self, cue: AudioCue) -> Result<()> {
        if !self.available {
            return Ok(());
        }
        
        log::debug!("[AUDIO] Playing cue: {:?} @ volume {:.0}%", cue, self.volume * 100.0);
        
        // In production, would play actual audio
        // For now, just log
        
        Ok(())
    }
    
    /// Set volume level
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
}

/// Audio cue types
#[derive(Debug, Clone, Copy)]
pub enum AudioCue {
    /// Soft ding for success
    Success,
    
    /// Double tone for confirmation needed
    Confirm,
    
    /// Buzz for error
    Error,
    
    /// Rising tone for attention
    Attention,
    
    /// Soft tick for navigation
    Tick,
}

// ============================================================================
// WHISPER RENDERER
// ============================================================================

/// Renders text as AR whisper overlay
pub struct WhisperRenderer {
    /// Current whisper queue
    queue: Vec<WhisperItem>,
    
    /// Maximum items in queue
    max_queue: usize,
}

struct WhisperItem {
    text: String,
    style: WhisperStyle,
    position: (f32, f32),
    duration_ms: u64,
    created_at: Instant,
}

impl WhisperRenderer {
    pub fn new(max_queue: usize) -> Self {
        Self {
            queue: Vec::new(),
            max_queue,
        }
    }
    
    /// Add a whisper to the queue
    pub fn whisper(&mut self, text: &str, style: WhisperStyle, duration_ms: u64) {
        // Clean up expired whispers
        self.queue.retain(|w| w.created_at.elapsed().as_millis() < w.duration_ms as u128);
        
        // Enforce max queue
        if self.queue.len() >= self.max_queue {
            self.queue.remove(0);
        }
        
        // Add new whisper
        self.queue.push(WhisperItem {
            text: text.to_string(),
            style,
            position: (0.5, 0.85), // Bottom center default
            duration_ms,
            created_at: Instant::now(),
        });
        
        log::info!("[WHISPER] {} ({:?})", text, style);
    }
    
    /// Get current whispers for rendering
    pub fn get_active_whispers(&self) -> Vec<AROverlay> {
        self.queue.iter()
            .filter(|w| w.created_at.elapsed().as_millis() < w.duration_ms as u128)
            .enumerate()
            .map(|(i, w)| {
                AROverlay {
                    overlay_type: AROverlayType::Whisper,
                    content: w.text.clone(),
                    position: (w.position.0, w.position.1 - (i as f32 * 0.08)), // Stack upward
                    duration_ms: w.duration_ms.saturating_sub(w.created_at.elapsed().as_millis() as u64),
                    style: w.style,
                }
            })
            .collect()
    }
    
    /// Clear all whispers
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

// ============================================================================
// MINIMAL MANIFEST IMPLEMENTATION
// ============================================================================

impl MinimalManifest {
    /// Create new manifest with default config
    pub fn new() -> Self {
        Self::with_config(ManifestConfig::default())
    }
    
    /// Create with custom config
    pub fn with_config(config: ManifestConfig) -> Self {
        Self {
            active_overlays: Vec::new(),
            haptic: HapticDriver::new(),
            audio: AudioDriver::new(),
            config,
            mode: OutputMode::Full,
        }
    }
    
    /// Set output mode
    pub fn set_mode(&mut self, mode: OutputMode) {
        self.mode = mode;
        log::info!("[MANIFEST] Output mode: {:?}", mode);
    }
    
    /// Render a complete Oracle response
    pub async fn render(&mut self, response: &crate::oracle::veil::OracleResponse) -> Result<()> {
        // 1. Play haptic feedback
        if self.config.haptic_enabled && self.mode != OutputMode::Silent {
            self.haptic.play(&response.haptic).await?;
        }
        
        // 2. Show whisper (if not haptic-only mode)
        if self.mode == OutputMode::Full || self.mode == OutputMode::Minimal {
            if !response.whisper.is_empty() {
                self.show_whisper(&response.whisper, self.determine_style(response))?;
            }
        }
        
        // 3. Show overlay if provided
        if let Some(overlay) = &response.overlay {
            if self.mode == OutputMode::Full {
                self.add_overlay(overlay.clone())?;
            }
        }
        
        // 4. Play audio cue
        if self.config.audio_enabled && self.mode == OutputMode::Full {
            let cue = if response.needs_confirmation {
                AudioCue::Confirm
            } else {
                AudioCue::Success
            };
            self.audio.play_cue(cue)?;
        }
        
        Ok(())
    }
    
    /// Show a text whisper
    pub fn show_whisper(&mut self, text: &str, style: WhisperStyle) -> Result<()> {
        if self.config.privacy_mode {
            log::info!("[MANIFEST] Privacy mode - whisper suppressed: {}", text);
            return Ok(());
        }
        
        // Clean up expired overlays
        self.cleanup_expired();
        
        // Enforce max overlays
        while self.active_overlays.len() >= self.config.max_overlays {
            self.active_overlays.remove(0);
        }
        
        // Create whisper overlay
        let overlay = AROverlay {
            overlay_type: AROverlayType::Whisper,
            content: text.to_string(),
            position: self.config.default_position,
            duration_ms: self.config.default_whisper_duration_ms,
            style,
        };
        
        self.add_overlay(overlay)?;
        
        log::info!("[WHISPER] {:?}: {}", style, text);
        Ok(())
    }
    
    /// Add an AR overlay
    pub fn add_overlay(&mut self, overlay: AROverlay) -> Result<()> {
        let duration = if overlay.duration_ms > 0 {
            Some(Instant::now() + Duration::from_millis(overlay.duration_ms))
        } else {
            None
        };
        
        self.active_overlays.push(ActiveOverlay {
            overlay,
            started_at: Instant::now(),
            expires_at: duration,
        });
        
        Ok(())
    }
    
    /// Play haptic pattern directly
    pub async fn play_haptic(&mut self, pattern: HapticPattern) -> Result<()> {
        self.haptic.play(&pattern).await
    }
    
    /// Get current active overlays for rendering
    pub fn get_overlays(&mut self) -> Vec<AROverlay> {
        self.cleanup_expired();
        
        self.active_overlays.iter()
            .map(|ao| ao.overlay.clone())
            .collect()
    }
    
    /// Clear all overlays
    pub fn clear_overlays(&mut self) {
        self.active_overlays.clear();
    }
    
    /// Remove expired overlays
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.active_overlays.retain(|ao| {
            ao.expires_at.map(|exp| exp > now).unwrap_or(true)
        });
    }
    
    /// Determine whisper style based on response
    fn determine_style(&self, response: &crate::oracle::veil::OracleResponse) -> WhisperStyle {
        if response.needs_confirmation {
            WhisperStyle::Emphasized
        } else if response.confidence < 0.5 {
            WhisperStyle::Subtle
        } else {
            WhisperStyle::Normal
        }
    }
}

impl Default for MinimalManifest {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RENDER HELPERS
// ============================================================================

/// Format a whisper for display
pub fn format_whisper(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    lines
}

/// Get emoji for whisper style
pub fn style_emoji(style: WhisperStyle) -> &'static str {
    match style {
        WhisperStyle::Subtle => "💭",
        WhisperStyle::Normal => "💬",
        WhisperStyle::Emphasized => "📢",
        WhisperStyle::Alert => "⚠️",
    }
}

/// Format balance for display
pub fn format_balance(amount: u128) -> String {
    let kara = amount / 1_000_000; // Assuming 6 decimal places
    let remainder = (amount % 1_000_000) / 10_000; // 2 decimal places
    format!("{}.{:02} KARA", kara, remainder)
}

/// Format duration for display
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

// ============================================================================
// MANIFEST BUILDER
// ============================================================================

/// Builder for creating rich manifests from intent results
pub struct ManifestBuilder {
    config: ManifestConfig,
}

impl ManifestBuilder {
    pub fn new() -> Self {
        Self {
            config: ManifestConfig::default(),
        }
    }
    
    /// Build manifest for a balance check result
    pub fn balance(&self, balance: u128, max: u128) -> ManifestOutput {
        let ratio = balance as f32 / max.max(1) as f32;
        let color = if ratio > 0.5 { "green" } else { "amber" };
        
        ManifestOutput {
            whisper: format_balance(balance),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Progress { percent: ratio * 100.0 },
                content: format!("Balance: {}", format_balance(balance)),
                position: (0.5, 0.3),
                duration_ms: 3000,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for a transfer result
    pub fn transfer(&self, amount: u128, recipient: &str, success: bool) -> ManifestOutput {
        let short_recipient = if recipient.len() > 8 { 
            &recipient[..8] 
        } else { 
            recipient 
        };
        
        if success {
            ManifestOutput {
                whisper: format!("✓ {} to {}", format_balance(amount), short_recipient),
                haptic: HapticPattern::Success,
                overlay: Some(AROverlay {
                    overlay_type: AROverlayType::Confirmation,
                    content: format!("Sent {} KARA", amount / 1_000_000),
                    position: (0.5, 0.5),
                    duration_ms: 2000,
                    style: WhisperStyle::Emphasized,
                }),
                needs_confirmation: false,
                confidence: 1.0,
            }
        } else {
            ManifestOutput {
                whisper: "✗ Transfer failed".to_string(),
                haptic: HapticPattern::Error,
                overlay: Some(AROverlay {
                    overlay_type: AROverlayType::Warning,
                    content: "Transfer failed".to_string(),
                    position: (0.5, 0.5),
                    duration_ms: 4000,
                    style: WhisperStyle::Alert,
                }),
                needs_confirmation: false,
                confidence: 1.0,
            }
        }
    }
    
    /// Build manifest for a timer
    pub fn timer(&self, remaining_secs: u64, label: &str) -> ManifestOutput {
        let urgency = if remaining_secs <= 10 {
            WhisperStyle::Alert
        } else {
            WhisperStyle::Normal
        };
        
        let haptic = if remaining_secs == 0 {
            HapticPattern::Attention
        } else if remaining_secs <= 10 {
            HapticPattern::Confirm
        } else {
            HapticPattern::Success
        };
        
        ManifestOutput {
            whisper: format!("⏱️ {} {}", label, format_duration(remaining_secs)),
            haptic,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Timer,
                content: format!("{}: {}", label, format_duration(remaining_secs)),
                position: (0.9, 0.1),
                duration_ms: 2000,
                style: urgency,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for general success
    pub fn success(&self, message: &str) -> ManifestOutput {
        let truncated = truncate(message, 47);
        ManifestOutput {
            whisper: format!("✓ {}", truncated),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Confirmation,
                content: message.to_string(),
                position: (0.5, 0.85),
                duration_ms: 2000,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for general error
    pub fn error(&self, message: &str) -> ManifestOutput {
        let truncated = truncate(message, 47);
        ManifestOutput {
            whisper: format!("✗ {}", truncated),
            haptic: HapticPattern::Error,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Warning,
                content: message.to_string(),
                position: (0.5, 0.5),
                duration_ms: 4000,
                style: WhisperStyle::Alert,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for navigation
    pub fn navigation(&self, direction: &str, distance: f32, instruction: &str) -> ManifestOutput {
        let arrow = match direction {
            "left" => "←",
            "right" => "→",
            "forward" | "straight" => "↑",
            "back" => "↓",
            _ => "●",
        };
        
        ManifestOutput {
            whisper: format!("{} {:.0}m", arrow, distance),
            haptic: HapticPattern::Navigation { 
                direction: match direction {
                    "left" => crate::oracle::command::NavigationDirection::Left,
                    "right" => crate::oracle::command::NavigationDirection::Right,
                    _ => crate::oracle::command::NavigationDirection::Forward,
                }
            },
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Navigation,
                content: instruction.to_string(),
                position: (0.5, 0.2),
                duration_ms: 5000,
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for object identification
    pub fn identify(&self, object: &str, confidence: f32) -> ManifestOutput {
        ManifestOutput {
            whisper: object.to_string(),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Highlight { bounds: (0.4, 0.4, 0.2, 0.2) },
                content: format!("{} ({:.0}%)", object, confidence * 100.0),
                position: (0.5, 0.5),
                duration_ms: 3000,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence,
        }
    }
    
    /// Build manifest for conversation response (whisper only)
    pub fn conversation(&self, response: &str) -> ManifestOutput {
        ManifestOutput {
            whisper: truncate(response, 50).to_string(),
            haptic: HapticPattern::Success, // Subtle, no haptic for conversation
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Whisper,
                content: truncate(response, 100).to_string(),
                position: (0.5, 0.85),
                duration_ms: 5000,
                style: WhisperStyle::Subtle,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for confirmation request
    pub fn confirmation(&self, action: &str, details: &str) -> ManifestOutput {
        ManifestOutput {
            whisper: format!("Confirm: {}?", truncate(action, 35)),
            haptic: HapticPattern::Confirm,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Confirmation,
                content: details.to_string(),
                position: (0.5, 0.5),
                duration_ms: 10000,
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: true,
            confidence: 1.0,
        }
    }
    
    /// Build manifest for thinking/processing state
    pub fn thinking(&self, status: &str) -> ManifestOutput {
        ManifestOutput {
            whisper: format!("💭 {}", truncate(status, 45)),
            haptic: HapticPattern::Thinking,
            overlay: Some(AROverlay {
                overlay_type: AROverlayType::Progress { percent: 0.0 }, // Indeterminate
                content: status.to_string(),
                position: (0.5, 0.9),
                duration_ms: 0, // Persistent until cleared
                style: WhisperStyle::Subtle,
            }),
            needs_confirmation: false,
            confidence: 0.5,
        }
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from ManifestBuilder (can be converted to OracleResponse)
#[derive(Debug, Clone)]
pub struct ManifestOutput {
    pub whisper: String,
    pub haptic: HapticPattern,
    pub overlay: Option<AROverlay>,
    pub needs_confirmation: bool,
    pub confidence: f32,
}

impl ManifestOutput {
    /// Convert to OracleResponse for API
    pub fn to_oracle_response(self) -> crate::oracle::veil::OracleResponse {
        crate::oracle::veil::OracleResponse {
            whisper: self.whisper,
            haptic: self.haptic,
            overlay: self.overlay,
            needs_confirmation: self.needs_confirmation,
            data: None,
            confidence: self.confidence,
        }
    }
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        // Find a safe UTF-8 boundary
        let mut end = max_len.saturating_sub(3);
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_whisper() {
        let text = "This is a long whisper that should be wrapped across multiple lines";
        let lines = format_whisper(text, 30);
        
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.len() <= 30));
    }
    
    #[test]
    fn test_format_balance() {
        assert_eq!(format_balance(1_000_000), "1.00 KARA");
        assert_eq!(format_balance(123_456_789), "123.45 KARA");
        assert_eq!(format_balance(0), "0.00 KARA");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }
    
    #[tokio::test]
    async fn test_haptic_driver() {
        let mut driver = HapticDriver::new();
        
        // Should not panic
        driver.play(&HapticPattern::Success).await.unwrap();
        driver.play(&HapticPattern::Error).await.unwrap();
    }
    
    #[test]
    fn test_manifest_overlay_cleanup() {
        let mut manifest = MinimalManifest::new();
        
        // Add an overlay with 1ms duration
        manifest.add_overlay(AROverlay {
            overlay_type: AROverlayType::Whisper,
            content: "test".to_string(),
            position: (0.5, 0.5),
            duration_ms: 1,
            style: WhisperStyle::Normal,
        }).unwrap();
        
        // Wait for expiry
        std::thread::sleep(Duration::from_millis(10));
        
        // Should be cleaned up
        let overlays = manifest.get_overlays();
        assert!(overlays.is_empty());
    }
}
