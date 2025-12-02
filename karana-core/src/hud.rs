//! # Kāraṇa Smart Glasses UI Simulator
//!
//! A realistic terminal-based simulation of AR smart glasses with:
//! - Transparent HUD overlay simulation
//! - Voice command input (simulated as text)
//! - Real-time status displays
//! - Animated transitions and visual effects
//! - AI conversation interface
//!
//! This creates an immersive experience that mirrors what users would
//! see through actual smart glasses hardware.

use std::io::{self, Write, stdout};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::VecDeque;

// ANSI escape codes for terminal control
pub mod ansi {
    pub const CLEAR: &str = "\x1b[2J";
    pub const HOME: &str = "\x1b[H";
    pub const HIDE_CURSOR: &str = "\x1b[?25l";
    pub const SHOW_CURSOR: &str = "\x1b[?25h";
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    
    // Colors
    pub const FG_BLACK: &str = "\x1b[30m";
    pub const FG_RED: &str = "\x1b[31m";
    pub const FG_GREEN: &str = "\x1b[32m";
    pub const FG_YELLOW: &str = "\x1b[33m";
    pub const FG_BLUE: &str = "\x1b[34m";
    pub const FG_MAGENTA: &str = "\x1b[35m";
    pub const FG_CYAN: &str = "\x1b[36m";
    pub const FG_WHITE: &str = "\x1b[37m";
    
    // Bright colors (better for HUD effect)
    pub const FG_BRIGHT_CYAN: &str = "\x1b[96m";
    pub const FG_BRIGHT_GREEN: &str = "\x1b[92m";
    pub const FG_BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const FG_BRIGHT_WHITE: &str = "\x1b[97m";
    pub const FG_BRIGHT_BLUE: &str = "\x1b[94m";
    pub const FG_BRIGHT_MAGENTA: &str = "\x1b[95m";
    
    // Background colors
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_CYAN: &str = "\x1b[46m";
    
    // RGB colors for more control
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }
    
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
    
    // Move cursor
    pub fn goto(x: u16, y: u16) -> String {
        format!("\x1b[{};{}H", y, x)
    }
    
    // Clear line
    pub const CLEAR_LINE: &str = "\x1b[2K";
}

/// HUD color scheme (cyberpunk/AR aesthetic)
pub struct HudColors {
    pub primary: &'static str,      // Main text/elements
    pub secondary: &'static str,    // Secondary elements
    pub accent: &'static str,       // Highlights
    pub success: &'static str,      // Success states
    pub warning: &'static str,      // Warnings
    pub error: &'static str,        // Errors
    pub dim: &'static str,          // Dimmed elements
    pub border: &'static str,       // Borders
}

impl Default for HudColors {
    fn default() -> Self {
        Self {
            primary: "\x1b[38;2;0;255;255m",    // Cyan
            secondary: "\x1b[38;2;100;200;255m", // Light blue
            accent: "\x1b[38;2;255;100;255m",   // Magenta
            success: "\x1b[38;2;100;255;100m",  // Green
            warning: "\x1b[38;2;255;200;50m",   // Yellow
            error: "\x1b[38;2;255;80;80m",      // Red
            dim: "\x1b[38;2;100;100;120m",      // Gray
            border: "\x1b[38;2;50;150;150m",    // Dark cyan
        }
    }
}

/// Notification in the HUD
#[derive(Clone)]
pub struct HudNotification {
    pub id: u64,
    pub icon: String,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub created_at: Instant,
    pub ttl: Duration,
    pub dismissed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl HudNotification {
    pub fn new(icon: &str, title: &str, body: &str) -> Self {
        Self {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            icon: icon.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            priority: NotificationPriority::Normal,
            created_at: Instant::now(),
            ttl: Duration::from_secs(5),
            dismissed: false,
        }
    }
    
    pub fn urgent(icon: &str, title: &str, body: &str) -> Self {
        let mut n = Self::new(icon, title, body);
        n.priority = NotificationPriority::Urgent;
        n.ttl = Duration::from_secs(10);
        n
    }
    
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl || self.dismissed
    }
}

/// A message in the AI conversation
#[derive(Clone)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Instant,
    pub typing_complete: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// System status information
#[derive(Clone)]
pub struct SystemStatus {
    pub battery: u8,
    pub is_charging: bool,
    pub wifi_connected: bool,
    pub wifi_strength: u8,  // 0-4
    pub bluetooth: bool,
    pub gps: bool,
    pub recording: bool,
    pub time: String,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            battery: 85,
            is_charging: false,
            wifi_connected: true,
            wifi_strength: 3,
            bluetooth: true,
            gps: false,
            recording: false,
            time: "12:00".to_string(),
        }
    }
}

impl SystemStatus {
    /// Update time to current
    pub fn update_time(&mut self) {
        let now = chrono::Local::now();
        self.time = now.format("%H:%M").to_string();
    }
    
    /// Get battery icon based on level
    pub fn battery_icon(&self) -> &str {
        if self.is_charging {
            "🔌"
        } else if self.battery > 75 {
            "🔋"
        } else if self.battery > 50 {
            "🔋"
        } else if self.battery > 25 {
            "🪫"
        } else {
            "🪫"
        }
    }
    
    /// Get WiFi icon based on strength
    pub fn wifi_icon(&self) -> &str {
        if !self.wifi_connected {
            "📡"
        } else {
            match self.wifi_strength {
                0 => "▂",
                1 => "▂▄",
                2 => "▂▄▆",
                3 | 4 => "▂▄▆█",
                _ => "▂▄▆█",
            }
        }
    }
}

/// Gaze cursor position
#[derive(Clone, Default)]
pub struct GazeCursor {
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub dwell_progress: f32,  // 0.0 to 1.0
}

/// The Smart Glasses HUD Simulator
pub struct GlassesHUD {
    // Terminal dimensions
    width: u16,
    height: u16,
    
    // Colors
    colors: HudColors,
    
    // State
    status: SystemStatus,
    notifications: VecDeque<HudNotification>,
    conversation: Vec<ConversationMessage>,
    gaze: GazeCursor,
    
    // UI state
    listening: bool,
    processing: bool,
    input_buffer: String,
    
    // Animation state
    frame_count: u64,
    boot_sequence_complete: bool,
    
    // Typing animation
    typing_char_index: usize,
    last_type_time: Instant,
    
    // Frame buffer for smooth rendering
    frame_buffer: Vec<Vec<char>>,
    color_buffer: Vec<Vec<String>>,
}

impl GlassesHUD {
    pub fn new() -> Self {
        // Get terminal size
        let (width, height) = term_size::dimensions()
            .map(|(w, h)| (w as u16, h as u16))
            .unwrap_or((120, 40));
        
        Self {
            width,
            height,
            colors: HudColors::default(),
            status: SystemStatus::default(),
            notifications: VecDeque::new(),
            conversation: Vec::new(),
            gaze: GazeCursor::default(),
            listening: false,
            processing: false,
            input_buffer: String::new(),
            frame_count: 0,
            boot_sequence_complete: false,
            typing_char_index: 0,
            last_type_time: Instant::now(),
            frame_buffer: vec![vec![' '; width as usize]; height as usize],
            color_buffer: vec![vec![String::new(); width as usize]; height as usize],
        }
    }
    
    /// Initialize the terminal for HUD display
    pub fn init(&self) {
        print!("{}{}{}", ansi::CLEAR, ansi::HOME, ansi::HIDE_CURSOR);
        stdout().flush().unwrap();
    }
    
    /// Clean up terminal on exit
    pub fn cleanup(&self) {
        print!("{}{}{}", ansi::SHOW_CURSOR, ansi::RESET, ansi::CLEAR);
        stdout().flush().unwrap();
    }
    
    /// Update terminal dimensions
    pub fn refresh_dimensions(&mut self) {
        if let Some((w, h)) = term_size::dimensions() {
            self.width = w as u16;
            self.height = h as u16;
        }
    }
    
    /// Add a notification
    pub fn notify(&mut self, notification: HudNotification) {
        self.notifications.push_front(notification);
        // Keep only last 5 notifications
        while self.notifications.len() > 5 {
            self.notifications.pop_back();
        }
    }
    
    /// Add a user message
    pub fn add_user_message(&mut self, content: &str) {
        self.conversation.push(ConversationMessage {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Instant::now(),
            typing_complete: true,
        });
        // Keep last 10 messages
        while self.conversation.len() > 10 {
            self.conversation.remove(0);
        }
    }
    
    /// Add an assistant message (with typing animation)
    pub fn add_assistant_message(&mut self, content: &str) {
        self.conversation.push(ConversationMessage {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: Instant::now(),
            typing_complete: false,
        });
        self.typing_char_index = 0;
        // Keep last 10 messages
        while self.conversation.len() > 10 {
            self.conversation.remove(0);
        }
    }
    
    /// Add a system message (no typing animation)
    pub fn add_system_message(&mut self, content: &str) {
        self.conversation.push(ConversationMessage {
            role: MessageRole::System,
            content: content.to_string(),
            timestamp: Instant::now(),
            typing_complete: true,
        });
        // Keep last 10 messages
        while self.conversation.len() > 10 {
            self.conversation.remove(0);
        }
    }
    
    /// Clear all conversation messages
    pub fn clear_messages(&mut self) {
        self.conversation.clear();
    }
    
    /// Set listening state
    pub fn set_listening(&mut self, listening: bool) {
        self.listening = listening;
    }
    
    /// Set processing state
    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing;
    }
    
    /// Update input buffer (for display)
    pub fn set_input(&mut self, input: &str) {
        self.input_buffer = input.to_string();
    }
    
    /// Get current input
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }
    
    /// Clear input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }
    
    /// Append to input buffer
    pub fn push_input(&mut self, c: char) {
        self.input_buffer.push(c);
    }
    
    /// Remove last character from input
    pub fn pop_input(&mut self) {
        self.input_buffer.pop();
    }
    
    /// Update typing animation
    fn update_typing(&mut self) {
        if let Some(last) = self.conversation.last_mut() {
            if !last.typing_complete && last.role == MessageRole::Assistant {
                if self.last_type_time.elapsed() > Duration::from_millis(15) {
                    self.typing_char_index += 1;
                    if self.typing_char_index >= last.content.chars().count() {
                        last.typing_complete = true;
                    }
                    self.last_type_time = Instant::now();
                }
            }
        }
    }
    
    /// Remove expired notifications
    fn cleanup_notifications(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }
    
    /// Render the full HUD frame
    pub fn render(&mut self) {
        self.frame_count += 1;
        self.status.update_time();
        self.update_typing();
        self.cleanup_notifications();
        self.refresh_dimensions();
        
        let mut output = String::new();
        output.push_str(ansi::HOME);
        
        // Render each component
        self.render_background(&mut output);
        self.render_status_bar(&mut output);
        self.render_side_indicators(&mut output);
        self.render_conversation(&mut output);
        self.render_notifications(&mut output);
        self.render_input_area(&mut output);
        self.render_gaze_cursor(&mut output);
        
        // Final output
        print!("{}", output);
        stdout().flush().unwrap();
    }
    
    /// Render the semi-transparent background
    fn render_background(&self, output: &mut String) {
        // Create a subtle grid pattern to simulate AR overlay
        let grid_char = if self.frame_count % 60 < 30 { '·' } else { '·' };
        
        for y in 0..self.height {
            output.push_str(&ansi::goto(1, y + 1));
            for x in 0..self.width {
                if (x + y) % 8 == 0 {
                    output.push_str(&format!("{}{}{}", self.colors.dim, grid_char, ansi::RESET));
                } else {
                    output.push(' ');
                }
            }
        }
    }
    
    /// Render the top status bar
    fn render_status_bar(&self, output: &mut String) {
        let bar_width = self.width as usize;
        
        // Top border
        output.push_str(&ansi::goto(1, 1));
        output.push_str(&format!("{}╭{}╮{}", 
            self.colors.border,
            "─".repeat(bar_width.saturating_sub(2)),
            ansi::RESET
        ));
        
        // Status bar content
        output.push_str(&ansi::goto(1, 2));
        output.push_str(&format!("{}│{}", self.colors.border, ansi::RESET));
        
        // Left side: KARANA logo
        output.push_str(&format!(" {}{}◈ KARANA{} ", 
            self.colors.primary, ansi::BOLD, ansi::RESET));
        
        // Center: Time
        let time_pos = bar_width / 2 - 3;
        output.push_str(&ansi::goto(time_pos as u16, 2));
        output.push_str(&format!("{}🕐 {}{}", 
            self.colors.secondary, self.status.time, ansi::RESET));
        
        // Right side: Status icons
        let right_content = format!(
            "{} {} {}{}%  {}",
            self.status.wifi_icon(),
            if self.status.bluetooth { "󰂯" } else { "󰂲" },
            self.status.battery_icon(),
            self.status.battery,
            if self.status.recording { "🔴 REC" } else { "" }
        );
        let right_pos = bar_width.saturating_sub(right_content.chars().count() + 2);
        output.push_str(&ansi::goto(right_pos as u16, 2));
        output.push_str(&format!("{}{}{}", self.colors.secondary, right_content, ansi::RESET));
        
        output.push_str(&ansi::goto(bar_width as u16, 2));
        output.push_str(&format!("{}│{}", self.colors.border, ansi::RESET));
        
        // Bottom border
        output.push_str(&ansi::goto(1, 3));
        output.push_str(&format!("{}╰{}╯{}", 
            self.colors.border,
            "─".repeat(bar_width.saturating_sub(2)),
            ansi::RESET
        ));
    }
    
    /// Render side indicators (minimal UI elements)
    fn render_side_indicators(&self, output: &mut String) {
        // Left side: Voice/listening indicator
        let left_x = 3;
        let left_y = self.height / 2;
        
        output.push_str(&ansi::goto(left_x, left_y - 1));
        if self.listening {
            // Animated listening indicator
            let pulse = ((self.frame_count % 20) as f32 / 20.0 * std::f32::consts::PI * 2.0).sin();
            let intensity = ((pulse + 1.0) / 2.0 * 155.0 + 100.0) as u8;
            output.push_str(&format!("{}┌───┐{}", ansi::fg_rgb(intensity, 50, 50), ansi::RESET));
            output.push_str(&ansi::goto(left_x, left_y));
            output.push_str(&format!("{}│🎤 │{}", ansi::fg_rgb(intensity, 50, 50), ansi::RESET));
            output.push_str(&ansi::goto(left_x, left_y + 1));
            output.push_str(&format!("{}└───┘{}", ansi::fg_rgb(intensity, 50, 50), ansi::RESET));
        } else if self.processing {
            // Processing spinner
            let spinner = ['◐', '◓', '◑', '◒'][(self.frame_count / 5 % 4) as usize];
            output.push_str(&format!("{}┌───┐{}", self.colors.accent, ansi::RESET));
            output.push_str(&ansi::goto(left_x, left_y));
            output.push_str(&format!("{}│ {} │{}", self.colors.accent, spinner, ansi::RESET));
            output.push_str(&ansi::goto(left_x, left_y + 1));
            output.push_str(&format!("{}└───┘{}", self.colors.accent, ansi::RESET));
        }
        
        // Right side: Quick stats or context
        let right_x = self.width - 12;
        let right_y = self.height / 2 - 2;
        
        output.push_str(&ansi::goto(right_x, right_y));
        output.push_str(&format!("{}┌─────────┐{}", self.colors.dim, ansi::RESET));
        output.push_str(&ansi::goto(right_x, right_y + 1));
        output.push_str(&format!("{}│ {} MSGS  │{}", self.colors.dim, self.conversation.len(), ansi::RESET));
        output.push_str(&ansi::goto(right_x, right_y + 2));
        output.push_str(&format!("{}│ {} NOTIF │{}", self.colors.dim, self.notifications.len(), ansi::RESET));
        output.push_str(&ansi::goto(right_x, right_y + 3));
        output.push_str(&format!("{}└─────────┘{}", self.colors.dim, ansi::RESET));
    }
    
    /// Render the conversation area
    fn render_conversation(&self, output: &mut String) {
        let conv_start_y = 5;
        let conv_height = self.height.saturating_sub(12);
        let conv_start_x = 10;
        let conv_width = self.width.saturating_sub(24);
        
        // Conversation box
        output.push_str(&ansi::goto(conv_start_x, conv_start_y));
        output.push_str(&format!("{}╭─ Conversation {}╮{}", 
            self.colors.border,
            "─".repeat(conv_width.saturating_sub(17) as usize),
            ansi::RESET
        ));
        
        // Draw side borders
        for y in 1..conv_height {
            output.push_str(&ansi::goto(conv_start_x, conv_start_y + y));
            output.push_str(&format!("{}│{}", self.colors.border, ansi::RESET));
            output.push_str(&ansi::goto(conv_start_x + conv_width - 1, conv_start_y + y));
            output.push_str(&format!("{}│{}", self.colors.border, ansi::RESET));
        }
        
        // Bottom border
        output.push_str(&ansi::goto(conv_start_x, conv_start_y + conv_height));
        output.push_str(&format!("{}╰{}╯{}", 
            self.colors.border,
            "─".repeat(conv_width.saturating_sub(2) as usize),
            ansi::RESET
        ));
        
        // Render messages
        let mut line_y = conv_start_y + 1;
        let max_line_width = conv_width.saturating_sub(4) as usize;
        
        for msg in &self.conversation {
            if line_y >= conv_start_y + conv_height - 1 {
                break;
            }
            
            let (prefix, color) = match msg.role {
                MessageRole::User => ("▶ You: ", self.colors.accent),
                MessageRole::Assistant => ("◀ AI: ", self.colors.primary),
                MessageRole::System => ("● Sys: ", self.colors.dim),
            };
            
            // Get content to display (with typing animation for assistant)
            let display_content = if msg.role == MessageRole::Assistant && !msg.typing_complete {
                let chars: String = msg.content.chars().take(self.typing_char_index).collect();
                format!("{}▌", chars)  // Add cursor
            } else {
                msg.content.clone()
            };
            
            // Word wrap
            let full_text = format!("{}{}", prefix, display_content);
            let lines = word_wrap(&full_text, max_line_width);
            
            for (i, line) in lines.iter().enumerate() {
                if line_y >= conv_start_y + conv_height - 1 {
                    break;
                }
                output.push_str(&ansi::goto(conv_start_x + 2, line_y));
                if i == 0 {
                    output.push_str(&format!("{}{}{}", color, line, ansi::RESET));
                } else {
                    output.push_str(&format!("{}  {}{}", color, line, ansi::RESET));
                }
                line_y += 1;
            }
            line_y += 1; // Space between messages
        }
        
        // Show empty state
        if self.conversation.is_empty() {
            output.push_str(&ansi::goto(conv_start_x + 4, conv_start_y + conv_height / 2));
            output.push_str(&format!(
                "{}Say \"Hey Karana\" or type to begin...{}",
                self.colors.dim, ansi::RESET
            ));
        }
    }
    
    /// Render notifications panel
    fn render_notifications(&self, output: &mut String) {
        if self.notifications.is_empty() {
            return;
        }
        
        let notif_x = self.width - 35;
        let mut notif_y = 5;
        
        for notif in self.notifications.iter().take(3) {
            let age = notif.created_at.elapsed();
            let fade = if age > Duration::from_secs(3) {
                0.5
            } else {
                1.0
            };
            
            let border_color = match notif.priority {
                NotificationPriority::Urgent => self.colors.error,
                NotificationPriority::High => self.colors.warning,
                _ => self.colors.secondary,
            };
            
            // Notification box
            output.push_str(&ansi::goto(notif_x, notif_y));
            output.push_str(&format!("{}╭─ {} {} ─────────────╮{}", 
                border_color, notif.icon, truncate(&notif.title, 15), ansi::RESET));
            
            notif_y += 1;
            output.push_str(&ansi::goto(notif_x, notif_y));
            output.push_str(&format!("{}│ {:<28} │{}", 
                border_color, truncate(&notif.body, 28), ansi::RESET));
            
            notif_y += 1;
            output.push_str(&ansi::goto(notif_x, notif_y));
            output.push_str(&format!("{}╰──────────────────────────────╯{}", 
                border_color, ansi::RESET));
            
            notif_y += 2;
        }
    }
    
    /// Render the input area
    fn render_input_area(&self, output: &mut String) {
        let input_y = self.height - 4;
        let input_x = 10;
        let input_width = self.width.saturating_sub(24);
        
        // Input box
        output.push_str(&ansi::goto(input_x, input_y));
        let title = if self.listening {
            "🎤 Listening..."
        } else {
            "💬 Type or speak"
        };
        output.push_str(&format!("{}╭─ {} {}╮{}", 
            self.colors.primary,
            title,
            "─".repeat(input_width.saturating_sub(title.len() as u16 + 5) as usize),
            ansi::RESET
        ));
        
        output.push_str(&ansi::goto(input_x, input_y + 1));
        output.push_str(&format!("{}│{}", self.colors.primary, ansi::RESET));
        
        // Show input buffer or prompt
        output.push_str(&ansi::goto(input_x + 2, input_y + 1));
        if self.input_buffer.is_empty() {
            if self.listening {
                // Animated waveform
                let wave: String = (0..20).map(|i| {
                    let phase = (self.frame_count as f32 / 5.0 + i as f32 * 0.5).sin();
                    if phase > 0.3 { '█' } else if phase > -0.3 { '▄' } else { '▁' }
                }).collect();
                output.push_str(&format!("{}{}{}", self.colors.accent, wave, ansi::RESET));
            } else {
                output.push_str(&format!("{}>{}", self.colors.dim, ansi::RESET));
            }
        } else {
            let cursor = if self.frame_count % 20 < 10 { "▌" } else { " " };
            output.push_str(&format!("{}> {}{}{}", 
                self.colors.primary, 
                &self.input_buffer,
                cursor,
                ansi::RESET
            ));
        }
        
        output.push_str(&ansi::goto(input_x + input_width - 1, input_y + 1));
        output.push_str(&format!("{}│{}", self.colors.primary, ansi::RESET));
        
        output.push_str(&ansi::goto(input_x, input_y + 2));
        output.push_str(&format!("{}╰{}╯{}", 
            self.colors.primary,
            "─".repeat(input_width.saturating_sub(2) as usize),
            ansi::RESET
        ));
        
        // Help text
        output.push_str(&ansi::goto(input_x, input_y + 3));
        output.push_str(&format!(
            "{}[Enter] Send  [Ctrl+L] Listen  [Ctrl+C] Exit{}",
            self.colors.dim, ansi::RESET
        ));
    }
    
    /// Render gaze cursor (if enabled)
    fn render_gaze_cursor(&self, output: &mut String) {
        if !self.gaze.visible {
            return;
        }
        
        let x = (self.gaze.x * self.width as f32) as u16;
        let y = (self.gaze.y * self.height as f32) as u16;
        
        output.push_str(&ansi::goto(x.max(1), y.max(1)));
        
        // Gaze cursor with dwell progress
        if self.gaze.dwell_progress > 0.0 {
            let segments = (self.gaze.dwell_progress * 8.0) as usize;
            let ring = ["○", "◔", "◐", "◕", "●", "◉", "⦿", "⊛", "✦"];
            output.push_str(&format!("{}{}{}", self.colors.accent, ring[segments.min(8)], ansi::RESET));
        } else {
            output.push_str(&format!("{}◎{}", self.colors.accent, ansi::RESET));
        }
    }
    
    /// Play boot sequence animation
    pub fn boot_sequence(&mut self) {
        self.init();
        
        let messages = [
            ("Initializing neural interface...", 200),
            ("Loading KARANA-OS v0.8.0...", 150),
            ("Connecting to blockchain network...", 250),
            ("AI core online...", 100),
            ("Calibrating AR display...", 150),
            ("Ready.", 300),
        ];
        
        for (msg, delay_ms) in messages {
            print!("{}", ansi::CLEAR);
            print!("{}", ansi::HOME);
            
            // Center the message
            let x = (self.width / 2).saturating_sub(msg.len() as u16 / 2);
            let y = self.height / 2;
            
            // Animated typing effect
            for (i, ch) in msg.chars().enumerate() {
                print!("{}", ansi::goto(x + i as u16, y));
                print!("{}{}{}", self.colors.primary, ch, ansi::RESET);
                stdout().flush().unwrap();
                thread::sleep(Duration::from_millis(30));
            }
            
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
        self.boot_sequence_complete = true;
    }
}

impl Default for GlassesHUD {
    fn default() -> Self {
        Self::new()
    }
}

/// Word wrap helper
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_word_wrap() {
        let text = "Hello world this is a test";
        let wrapped = word_wrap(text, 10);
        assert!(wrapped.len() > 1);
    }
    
    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World", 8), "Hello...");
    }
    
    #[test]
    fn test_notification() {
        let notif = HudNotification::new("🔔", "Test", "Body");
        assert!(!notif.is_expired());
    }
}
