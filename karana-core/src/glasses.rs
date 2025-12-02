//! # Kāraṇa Smart Glasses Integration
//!
//! This module provides the complete smart glasses experience:
//! - Voice command pipeline (Whisper → AI → Blockchain → UI)
//! - AR HUD rendering optimized for transparent displays
//! - Gaze tracking and eye gesture recognition
//! - Ambient context awareness
//!
//! ## Architecture
//! ```
//! Microphone → Whisper → Intent Parsing → Oracle → Blockchain → AR HUD
//!                           ↑                          ↓
//!                    Gaze Tracking ←←←←←←←←←←← Response Formatting
//! ```

use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::ai::KaranaAI;
use crate::oracle::KaranaOracle;

/// Configuration for the smart glasses display
#[derive(Debug, Clone)]
pub struct GlassesConfig {
    /// Display opacity (0.0 = fully transparent, 1.0 = opaque)
    pub display_opacity: f32,
    /// Enable eye tracking
    pub eye_tracking: bool,
    /// Voice activation keyword
    pub wake_word: String,
    /// Auto-dismiss notifications after N seconds
    pub notification_timeout: u32,
    /// Gaze-to-dismiss enabled
    pub gaze_dismiss: bool,
    /// Minimal HUD mode (for driving/walking)
    pub minimal_mode: bool,
    /// Font scale for readability
    pub font_scale: f32,
}

impl Default for GlassesConfig {
    fn default() -> Self {
        Self {
            display_opacity: 0.85,
            eye_tracking: true,
            wake_word: "karana".to_string(),
            notification_timeout: 5,
            gaze_dismiss: true,
            minimal_mode: false,
            font_scale: 1.0,
        }
    }
}

/// Represents the current gaze state
#[derive(Debug, Clone, Default)]
pub struct GazeState {
    /// Normalized X position (0.0 = left, 1.0 = right)
    pub x: f32,
    /// Normalized Y position (0.0 = top, 1.0 = bottom)
    pub y: f32,
    /// Dwell time on current position (ms)
    pub dwell_ms: u32,
    /// Currently focused element ID
    pub focused_element: Option<String>,
    /// Blink detected
    pub blink_detected: bool,
    /// Double blink (confirmation gesture)
    pub double_blink: bool,
}

/// AR element with gaze-aware positioning
#[derive(Debug, Clone)]
pub struct ARElement {
    pub id: String,
    pub content: String,
    /// Position (normalized 0.0-1.0)
    pub x: f32,
    pub y: f32,
    /// Size (normalized)
    pub width: f32,
    pub height: f32,
    /// Priority for occlusion (higher = on top)
    pub z_order: u8,
    /// Element type for styling
    pub element_type: ARElementType,
    /// Time-to-live in ms (0 = permanent)
    pub ttl_ms: u32,
    /// Created timestamp
    pub created_at: std::time::Instant,
    /// Is this element dismissible by gaze
    pub gaze_dismissible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ARElementType {
    /// Top HUD bar (battery, time, network)
    HudBar,
    /// Notification toast
    Notification,
    /// Wallet/balance display
    WalletWidget,
    /// Governance alert
    GovernanceAlert,
    /// AI response panel
    AiResponse,
    /// Transaction confirmation
    TransactionConfirm,
    /// File/storage indicator
    FileWidget,
    /// Error/warning
    ErrorAlert,
    /// Minimal mode indicator
    MinimalIndicator,
}

impl ARElement {
    /// Check if element should be visible based on TTL
    pub fn is_expired(&self) -> bool {
        if self.ttl_ms == 0 {
            return false;
        }
        self.created_at.elapsed().as_millis() as u32 >= self.ttl_ms
    }

    /// Check if gaze is within element bounds
    pub fn contains_gaze(&self, gaze: &GazeState) -> bool {
        gaze.x >= self.x && gaze.x <= (self.x + self.width) &&
        gaze.y >= self.y && gaze.y <= (self.y + self.height)
    }
}

/// The main smart glasses controller
pub struct SmartGlasses {
    config: GlassesConfig,
    ai: Arc<Mutex<KaranaAI>>,
    gaze: GazeState,
    elements: Vec<ARElement>,
    /// Voice buffer for streaming transcription
    voice_buffer: Vec<f32>,
    /// Is voice recording active
    voice_active: bool,
    /// Last transcribed text
    last_transcription: Option<String>,
    /// User DID for signing transactions
    user_did: String,
}

impl SmartGlasses {
    pub fn new(ai: Arc<Mutex<KaranaAI>>, user_did: &str) -> Self {
        Self {
            config: GlassesConfig::default(),
            ai,
            gaze: GazeState::default(),
            elements: Vec::new(),
            voice_buffer: Vec::new(),
            voice_active: false,
            last_transcription: None,
            user_did: user_did.to_string(),
        }
    }

    pub fn with_config(mut self, config: GlassesConfig) -> Self {
        self.config = config;
        self
    }

    /// Update gaze tracking state
    pub fn update_gaze(&mut self, x: f32, y: f32) {
        let moved = (self.gaze.x - x).abs() > 0.02 || (self.gaze.y - y).abs() > 0.02;
        
        if moved {
            self.gaze.dwell_ms = 0;
        } else {
            self.gaze.dwell_ms += 100; // Assuming 100ms update interval
        }

        self.gaze.x = x.clamp(0.0, 1.0);
        self.gaze.y = y.clamp(0.0, 1.0);

        // Update focused element
        self.gaze.focused_element = self.elements.iter()
            .filter(|e| e.contains_gaze(&self.gaze))
            .max_by_key(|e| e.z_order)
            .map(|e| e.id.clone());

        // Gaze dismiss after 2 seconds of dwell on dismissible element
        if self.config.gaze_dismiss && self.gaze.dwell_ms > 2000 {
            if let Some(focused_id) = &self.gaze.focused_element {
                self.elements.retain(|e| {
                    !(e.id == *focused_id && e.gaze_dismissible)
                });
                self.gaze.dwell_ms = 0;
            }
        }
    }

    /// Handle blink gesture
    pub fn handle_blink(&mut self, is_double: bool) {
        self.gaze.blink_detected = true;
        self.gaze.double_blink = is_double;

        if is_double {
            // Double blink = confirm action
            if let Some(focused_id) = &self.gaze.focused_element.clone() {
                log::info!("[GLASSES] Double blink confirm on: {}", focused_id);
                // Could trigger confirmation on focused element
            }
        }
    }

    /// Start voice recording
    pub fn start_voice_recording(&mut self) {
        self.voice_active = true;
        self.voice_buffer.clear();
        log::info!("[GLASSES] 🎤 Voice recording started");
        
        // Show recording indicator
        self.add_element(ARElement {
            id: "voice_indicator".to_string(),
            content: "🎤 Listening...".to_string(),
            x: 0.4,
            y: 0.02,
            width: 0.2,
            height: 0.06,
            z_order: 200,
            element_type: ARElementType::MinimalIndicator,
            ttl_ms: 0,
            created_at: std::time::Instant::now(),
            gaze_dismissible: false,
        });
    }

    /// Add audio samples to buffer
    pub fn add_voice_samples(&mut self, samples: &[f32]) {
        if self.voice_active {
            self.voice_buffer.extend_from_slice(samples);
        }
    }

    /// Stop recording and transcribe
    pub fn stop_voice_recording(&mut self) -> Result<Option<String>> {
        if !self.voice_active {
            return Ok(None);
        }

        self.voice_active = false;
        self.remove_element("voice_indicator");

        if self.voice_buffer.is_empty() {
            return Ok(None);
        }

        // Transcribe using Whisper
        let samples = std::mem::take(&mut self.voice_buffer);
        let transcription = {
            let mut ai = self.ai.lock().unwrap();
            ai.transcribe(samples).ok()
        };

        if let Some(ref text) = transcription {
            log::info!("[GLASSES] 📝 Transcribed: '{}'", text);
            self.last_transcription = Some(text.clone());
            
            // Show transcription briefly
            self.show_notification(&format!("\"{}\"", text), 2000);
        }

        Ok(transcription)
    }

    /// Process voice command through the Oracle
    pub fn process_voice_command(&mut self, oracle: &KaranaOracle, transcription: &str) -> Result<String> {
        log::info!("[GLASSES] 🗣️ Processing: '{}'", transcription);
        
        // Show processing indicator
        self.add_element(ARElement {
            id: "processing".to_string(),
            content: "⏳ Processing...".to_string(),
            x: 0.35,
            y: 0.45,
            width: 0.3,
            height: 0.1,
            z_order: 150,
            element_type: ARElementType::AiResponse,
            ttl_ms: 0,
            created_at: std::time::Instant::now(),
            gaze_dismissible: false,
        });

        // Execute through Oracle
        let result = oracle.process_query(transcription, &self.user_did)?;
        
        // Remove processing indicator
        self.remove_element("processing");
        
        // Display result
        self.display_oracle_result(&result);
        
        Ok(result)
    }

    /// Display Oracle result in AR format
    fn display_oracle_result(&mut self, result: &str) {
        // Parse result type from content
        let element_type = if result.contains("Wallet") || result.contains("KARA") {
            ARElementType::WalletWidget
        } else if result.contains("Proposal") || result.contains("Governance") {
            ARElementType::GovernanceAlert
        } else if result.contains("Transfer") || result.contains("Sent") {
            ARElementType::TransactionConfirm
        } else if result.contains("File") || result.contains("stored") {
            ARElementType::FileWidget
        } else if result.contains("Error") || result.contains("Failed") {
            ARElementType::ErrorAlert
        } else {
            ARElementType::AiResponse
        };

        // Position based on type
        let (x, y, w, h) = match element_type {
            ARElementType::WalletWidget => (0.7, 0.15, 0.25, 0.15),
            ARElementType::TransactionConfirm => (0.3, 0.35, 0.4, 0.2),
            ARElementType::GovernanceAlert => (0.25, 0.3, 0.5, 0.3),
            ARElementType::ErrorAlert => (0.3, 0.4, 0.4, 0.15),
            _ => (0.2, 0.25, 0.6, 0.4),
        };

        self.add_element(ARElement {
            id: "oracle_result".to_string(),
            content: result.to_string(),
            x,
            y,
            width: w,
            height: h,
            z_order: 100,
            element_type,
            ttl_ms: 8000, // Auto-dismiss after 8s
            created_at: std::time::Instant::now(),
            gaze_dismissible: true,
        });
    }

    /// Show a notification toast
    pub fn show_notification(&mut self, message: &str, duration_ms: u32) {
        self.add_element(ARElement {
            id: format!("notif_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            content: message.to_string(),
            x: 0.25,
            y: 0.85,
            width: 0.5,
            height: 0.08,
            z_order: 180,
            element_type: ARElementType::Notification,
            ttl_ms: duration_ms,
            created_at: std::time::Instant::now(),
            gaze_dismissible: true,
        });
    }

    /// Update the HUD bar (battery, network, etc.)
    pub fn update_hud(&mut self, battery_pct: u8, network_status: &str, block_height: u64) {
        // Remove old HUD
        self.elements.retain(|e| e.element_type != ARElementType::HudBar);

        if self.config.minimal_mode {
            // Minimal mode: just show essential info
            self.add_element(ARElement {
                id: "hud_minimal".to_string(),
                content: format!("{}% | H#{}", battery_pct, block_height),
                x: 0.8,
                y: 0.02,
                width: 0.18,
                height: 0.04,
                z_order: 255,
                element_type: ARElementType::HudBar,
                ttl_ms: 0,
                created_at: std::time::Instant::now(),
                gaze_dismissible: false,
            });
        } else {
            // Full HUD
            // Battery (top left)
            let battery_icon = if battery_pct > 80 { "🔋" } 
                else if battery_pct > 40 { "🔋" } 
                else if battery_pct > 20 { "🪫" } 
                else { "⚠️" };
            
            self.add_element(ARElement {
                id: "hud_battery".to_string(),
                content: format!("{} {}%", battery_icon, battery_pct),
                x: 0.02,
                y: 0.02,
                width: 0.15,
                height: 0.05,
                z_order: 255,
                element_type: ARElementType::HudBar,
                ttl_ms: 0,
                created_at: std::time::Instant::now(),
                gaze_dismissible: false,
            });

            // Network (top center)
            let net_icon = if network_status.contains("Sync") { "🌐" } else { "📡" };
            self.add_element(ARElement {
                id: "hud_network".to_string(),
                content: format!("{} {} | H#{}", net_icon, network_status, block_height),
                x: 0.35,
                y: 0.02,
                width: 0.3,
                height: 0.05,
                z_order: 255,
                element_type: ARElementType::HudBar,
                ttl_ms: 0,
                created_at: std::time::Instant::now(),
                gaze_dismissible: false,
            });

            // Time (top right)
            let time = chrono::Local::now().format("%H:%M").to_string();
            self.add_element(ARElement {
                id: "hud_time".to_string(),
                content: time,
                x: 0.88,
                y: 0.02,
                width: 0.1,
                height: 0.05,
                z_order: 255,
                element_type: ARElementType::HudBar,
                ttl_ms: 0,
                created_at: std::time::Instant::now(),
                gaze_dismissible: false,
            });
        }
    }

    /// Add an element to the display
    pub fn add_element(&mut self, element: ARElement) {
        // Remove existing element with same ID
        self.elements.retain(|e| e.id != element.id);
        self.elements.push(element);
    }

    /// Remove an element by ID
    pub fn remove_element(&mut self, id: &str) {
        self.elements.retain(|e| e.id != id);
    }

    /// Clean up expired elements
    pub fn cleanup_expired(&mut self) {
        self.elements.retain(|e| !e.is_expired());
    }

    /// Render the AR view to ASCII for TUI simulation
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let mut buffer = vec![vec![' '; width]; height];

        // Sort elements by z-order
        let mut sorted_elements: Vec<&ARElement> = self.elements.iter().collect();
        sorted_elements.sort_by_key(|e| e.z_order);

        for element in sorted_elements {
            // Map normalized coords to screen
            let sx = (element.x * width as f32) as usize;
            let sy = (element.y * height as f32) as usize;
            let sw = (element.width * width as f32).max(4.0) as usize;
            let sh = (element.height * height as f32).max(3.0) as usize;

            let ex = (sx + sw).min(width - 1);
            let ey = (sy + sh).min(height - 1);

            if sx >= ex || sy >= ey {
                continue;
            }

            // Draw border based on element type
            let (tl, tr, bl, br, h, v) = match element.element_type {
                ARElementType::HudBar => ('┌', '┐', '└', '┘', '─', '│'),
                ARElementType::Notification => ('╭', '╮', '╰', '╯', '─', '│'),
                ARElementType::ErrorAlert => ('╔', '╗', '╚', '╝', '═', '║'),
                ARElementType::TransactionConfirm => ('╔', '╗', '╚', '╝', '═', '║'),
                _ => ('╭', '╮', '╰', '╯', '─', '│'),
            };

            // Corners
            if sy < height && sx < width { buffer[sy][sx] = tl; }
            if sy < height && ex < width { buffer[sy][ex] = tr; }
            if ey < height && sx < width { buffer[ey][sx] = bl; }
            if ey < height && ex < width { buffer[ey][ex] = br; }

            // Horizontal edges
            for x in (sx + 1)..ex {
                if sy < height && x < width { buffer[sy][x] = h; }
                if ey < height && x < width { buffer[ey][x] = h; }
            }

            // Vertical edges
            for y in (sy + 1)..ey {
                if y < height && sx < width { buffer[y][sx] = v; }
                if y < height && ex < width { buffer[y][ex] = v; }
            }

            // Content
            let content_width = ex - sx - 2;
            let mut cx = sx + 1;
            let mut cy = sy + 1;

            for ch in element.content.chars() {
                if ch == '\n' {
                    cy += 1;
                    cx = sx + 1;
                    continue;
                }
                if cy < ey && cx < ex && cy < height && cx < width {
                    buffer[cy][cx] = ch;
                    cx += 1;
                    if cx >= ex - 1 {
                        cx = sx + 1;
                        cy += 1;
                    }
                }
            }
        }

        // Render gaze cursor
        let gx = (self.gaze.x * width as f32) as usize;
        let gy = (self.gaze.y * height as f32) as usize;
        if gy < height && gx < width {
            buffer[gy][gx] = '◉';
        }

        // Serialize
        let mut output = String::new();
        for row in buffer {
            output.push_str(&row.into_iter().collect::<String>());
            output.push('\n');
        }
        output
    }

    /// Get current config
    pub fn config(&self) -> &GlassesConfig {
        &self.config
    }

    /// Toggle minimal mode
    pub fn toggle_minimal_mode(&mut self) {
        self.config.minimal_mode = !self.config.minimal_mode;
        log::info!("[GLASSES] Minimal mode: {}", self.config.minimal_mode);
    }

    /// Set display opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.config.display_opacity = opacity.clamp(0.0, 1.0);
    }
}

/// Voice command pipeline for glasses
pub struct VoiceCommandPipeline {
    glasses: Arc<Mutex<SmartGlasses>>,
    oracle: Arc<KaranaOracle>,
    wake_word_detected: bool,
}

impl VoiceCommandPipeline {
    pub fn new(glasses: Arc<Mutex<SmartGlasses>>, oracle: Arc<KaranaOracle>) -> Self {
        Self {
            glasses,
            oracle,
            wake_word_detected: false,
        }
    }

    /// Process incoming audio chunk
    pub fn process_audio_chunk(&mut self, samples: &[f32]) -> Result<Option<String>> {
        let mut glasses = self.glasses.lock().unwrap();
        
        // Check for wake word in audio (simplified - would use keyword spotting in production)
        // For now, we assume voice is always active after "karana" detection
        if !self.wake_word_detected && !glasses.voice_active {
            // In real implementation: run keyword spotting
            // For demo, we'll start on any audio above threshold
            let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
            if rms > 0.1 {
                self.wake_word_detected = true;
                glasses.start_voice_recording();
            }
        }

        if glasses.voice_active {
            glasses.add_voice_samples(samples);
            
            // Check for silence to end recording
            let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
            if rms < 0.02 {
                // Silence detected - transcribe and process
                if let Some(transcription) = glasses.stop_voice_recording()? {
                    self.wake_word_detected = false;
                    
                    // Process through oracle
                    let result = glasses.process_voice_command(&self.oracle, &transcription)?;
                    return Ok(Some(result));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaze_tracking() {
        let ai = Arc::new(Mutex::new(KaranaAI::new().unwrap()));
        let mut glasses = SmartGlasses::new(ai, "test_user");
        
        glasses.update_gaze(0.5, 0.5);
        assert_eq!(glasses.gaze.x, 0.5);
        assert_eq!(glasses.gaze.y, 0.5);
    }

    #[test]
    fn test_element_expiry() {
        let element = ARElement {
            id: "test".to_string(),
            content: "Test".to_string(),
            x: 0.0, y: 0.0,
            width: 0.1, height: 0.1,
            z_order: 1,
            element_type: ARElementType::Notification,
            ttl_ms: 1, // 1ms TTL
            created_at: std::time::Instant::now(),
            gaze_dismissible: true,
        };

        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(element.is_expired());
    }
}
