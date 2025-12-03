//! # Browser Wrapper
//!
//! Wraps a browser engine (Chromium/Servo) for AR tab web content.
//! Handles navigation, scrolling, input, and rendering.

use std::collections::HashMap;

/// Browser instance for AR tabs
#[derive(Debug, Clone)]
pub struct BrowserInstance {
    /// Configuration
    pub config: BrowserConfig,
    /// Current URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Favicon data
    pub favicon: Option<Vec<u8>>,
    /// Loading state
    pub is_loading: bool,
    /// Current scroll position (0.0 - 1.0)
    pub scroll_position: f32,
    /// Can go back in history
    pub can_go_back: bool,
    /// Can go forward in history
    pub can_go_forward: bool,
    /// Navigation history
    history: Vec<String>,
    /// Current history index
    history_index: usize,
    /// Cursor position within page (normalized 0.0 - 1.0)
    cursor: Option<(f32, f32)>,
    /// Form input values
    form_data: HashMap<String, String>,
    /// Page zoom level
    zoom: f32,
    /// Last rendered frame (placeholder - would be actual pixels)
    frame_buffer: FrameBuffer,
}

impl BrowserInstance {
    /// Create a new browser instance
    pub fn new(url: &str) -> Self {
        Self {
            config: BrowserConfig::default(),
            url: url.to_string(),
            title: "Loading...".to_string(),
            favicon: None,
            is_loading: true,
            scroll_position: 0.0,
            can_go_back: false,
            can_go_forward: false,
            history: vec![url.to_string()],
            history_index: 0,
            cursor: None,
            form_data: HashMap::new(),
            zoom: 1.0,
            frame_buffer: FrameBuffer::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(url: &str, config: BrowserConfig) -> Self {
        Self {
            config,
            ..Self::new(url)
        }
    }
    
    /// Navigate to a URL
    pub fn navigate(&mut self, url: &str) -> Result<(), BrowserError> {
        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            // Try adding https
            return self.navigate(&format!("https://{}", url));
        }
        
        // Truncate forward history
        self.history.truncate(self.history_index + 1);
        
        // Add to history
        self.history.push(url.to_string());
        self.history_index = self.history.len() - 1;
        
        // Update state
        self.url = url.to_string();
        self.title = "Loading...".to_string();
        self.is_loading = true;
        self.scroll_position = 0.0;
        self.can_go_back = self.history_index > 0;
        self.can_go_forward = false;
        
        // In real implementation, this would trigger actual page load
        Ok(())
    }
    
    /// Go back in history
    pub fn go_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.url = self.history[self.history_index].clone();
            self.can_go_back = self.history_index > 0;
            self.can_go_forward = true;
            self.scroll_position = 0.0;
            self.is_loading = true;
            true
        } else {
            false
        }
    }
    
    /// Go forward in history
    pub fn go_forward(&mut self) -> bool {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.url = self.history[self.history_index].clone();
            self.can_go_back = true;
            self.can_go_forward = self.history_index < self.history.len() - 1;
            self.scroll_position = 0.0;
            self.is_loading = true;
            true
        } else {
            false
        }
    }
    
    /// Reload the current page
    pub fn reload(&mut self) {
        self.is_loading = true;
        self.scroll_position = 0.0;
    }
    
    /// Stop loading
    pub fn stop(&mut self) {
        self.is_loading = false;
    }
    
    /// Scroll the page
    pub fn scroll(&mut self, delta: f32) {
        self.scroll_position = (self.scroll_position + delta).clamp(0.0, 1.0);
    }
    
    /// Scroll to specific position
    pub fn scroll_to(&mut self, position: f32) {
        self.scroll_position = position.clamp(0.0, 1.0);
    }
    
    /// Voice-controlled scrolling
    pub fn voice_scroll(&mut self, direction: ScrollDirection, amount: ScrollAmount) {
        let delta = match amount {
            ScrollAmount::Line => 0.02,
            ScrollAmount::HalfPage => 0.25,
            ScrollAmount::Page => 0.5,
            ScrollAmount::ToEnd => 1.0,
        };
        
        match direction {
            ScrollDirection::Up => self.scroll(-delta),
            ScrollDirection::Down => self.scroll(delta),
            ScrollDirection::Left | ScrollDirection::Right => {
                // Horizontal scroll - not implemented for simplicity
            }
        }
    }
    
    /// Click at position (normalized coordinates)
    pub fn click(&mut self, x: f32, y: f32) -> ClickResult {
        self.cursor = Some((x, y));
        
        // In real implementation, this would:
        // 1. Hit test to find element at position
        // 2. Trigger click event
        // 3. Handle navigation if link clicked
        
        ClickResult::Clicked { x, y }
    }
    
    /// Double click
    pub fn double_click(&mut self, x: f32, y: f32) -> ClickResult {
        self.cursor = Some((x, y));
        
        // Would select word at position
        ClickResult::Selected { 
            start: (x - 0.05, y),
            end: (x + 0.05, y),
        }
    }
    
    /// Type text (for form input)
    pub fn type_text(&mut self, text: &str) {
        // Would type into focused element
        // For now, just append to current form field
        if let Some(current) = self.form_data.get_mut("_focused") {
            current.push_str(text);
        }
    }
    
    /// Handle keyboard input
    pub fn key_press(&mut self, key: KeyCode, modifiers: Modifiers) {
        match (key, modifiers) {
            (KeyCode::Backspace, _) => {
                if let Some(current) = self.form_data.get_mut("_focused") {
                    current.pop();
                }
            }
            (KeyCode::Enter, _) => {
                // Submit form or navigate
            }
            (KeyCode::Escape, _) => {
                // Cancel/unfocus
            }
            (KeyCode::Tab, _) => {
                // Move to next element
            }
            (KeyCode::F5, _) | (KeyCode::R, Modifiers { ctrl: true, .. }) => {
                self.reload();
            }
            (KeyCode::Left, Modifiers { alt: true, .. }) => {
                self.go_back();
            }
            (KeyCode::Right, Modifiers { alt: true, .. }) => {
                self.go_forward();
            }
            _ => {}
        }
    }
    
    /// Update cursor position (for gaze tracking)
    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.cursor = Some((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)));
    }
    
    /// Get cursor position
    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.cursor
    }
    
    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }
    
    /// Zoom in
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * 1.25);
    }
    
    /// Zoom out
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / 1.25);
    }
    
    /// Reset zoom
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }
    
    /// Get current zoom level
    pub fn zoom_level(&self) -> f32 {
        self.zoom
    }
    
    /// Find text on page
    pub fn find(&mut self, query: &str) -> FindResult {
        // Would search page content
        FindResult {
            query: query.to_string(),
            matches: 0,
            current: 0,
        }
    }
    
    /// Find next match
    pub fn find_next(&mut self) {
        // Would highlight next match
    }
    
    /// Find previous match
    pub fn find_previous(&mut self) {
        // Would highlight previous match
    }
    
    /// Clear find highlights
    pub fn clear_find(&mut self) {
        // Would clear highlights
    }
    
    /// Execute JavaScript
    pub fn execute_js(&mut self, script: &str) -> Result<String, BrowserError> {
        // Would execute JS and return result
        // For security, might want to restrict this
        Ok(format!("Executed: {}", script))
    }
    
    /// Get page HTML
    pub fn get_html(&self) -> String {
        // Would return current page HTML
        format!("<html><body>Page at {}</body></html>", self.url)
    }
    
    /// Get selected text
    pub fn get_selection(&self) -> Option<String> {
        // Would return currently selected text
        None
    }
    
    /// Copy selection to clipboard
    pub fn copy(&self) -> Option<String> {
        self.get_selection()
    }
    
    /// Paste from clipboard
    pub fn paste(&mut self, text: &str) {
        self.type_text(text);
    }
    
    /// Render the current frame
    pub fn render_frame(&mut self) -> &FrameBuffer {
        // In real implementation, this would:
        // 1. Layout the page
        // 2. Render to texture
        // 3. Return frame buffer
        
        // Update frame metadata
        self.frame_buffer.width = self.config.viewport_width;
        self.frame_buffer.height = self.config.viewport_height;
        self.frame_buffer.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        &self.frame_buffer
    }
    
    /// Mark page as loaded (for testing/simulation)
    pub fn mark_loaded(&mut self, title: &str) {
        self.is_loading = false;
        self.title = title.to_string();
    }
}

/// Browser configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Viewport width in pixels
    pub viewport_width: u32,
    /// Viewport height in pixels
    pub viewport_height: u32,
    /// User agent string
    pub user_agent: String,
    /// Enable JavaScript
    pub javascript_enabled: bool,
    /// Enable cookies
    pub cookies_enabled: bool,
    /// Enable local storage
    pub local_storage_enabled: bool,
    /// Enable WebGL
    pub webgl_enabled: bool,
    /// Enable audio
    pub audio_enabled: bool,
    /// Enable video
    pub video_enabled: bool,
    /// Privacy mode (no history, clear on close)
    pub private_mode: bool,
    /// Block ads/trackers
    pub content_blocking: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: "KaranaOS/1.0 (AR Glasses; Spatial Browser)".to_string(),
            javascript_enabled: true,
            cookies_enabled: true,
            local_storage_enabled: true,
            webgl_enabled: true,
            audio_enabled: true,
            video_enabled: true,
            private_mode: false,
            content_blocking: true,
        }
    }
}

impl BrowserConfig {
    /// Create minimal config (for low power/performance)
    pub fn minimal() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            webgl_enabled: false,
            video_enabled: false,
            content_blocking: true,
            ..Self::default()
        }
    }
    
    /// Create privacy-focused config
    pub fn private() -> Self {
        Self {
            private_mode: true,
            cookies_enabled: false,
            local_storage_enabled: false,
            content_blocking: true,
            ..Self::default()
        }
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Scroll amount
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollAmount {
    Line,
    HalfPage,
    Page,
    ToEnd,
}

/// Click result
#[derive(Debug, Clone)]
pub enum ClickResult {
    /// Clicked at position
    Clicked { x: f32, y: f32 },
    /// Clicked a link
    NavigatedTo(String),
    /// Selected text
    Selected { start: (f32, f32), end: (f32, f32) },
    /// Focused input element
    FocusedInput { element_id: String },
    /// No element at position
    Miss,
}

/// Find result
#[derive(Debug, Clone)]
pub struct FindResult {
    pub query: String,
    pub matches: usize,
    pub current: usize,
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, 
    Num5, Num6, Num7, Num8, Num9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Enter, Escape, Backspace, Tab, Space,
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    Insert, Delete,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Frame buffer for rendered content
#[derive(Debug, Clone, Default)]
pub struct FrameBuffer {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Pixel data (RGBA)
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
    /// Format
    pub format: PixelFormat,
}

impl FrameBuffer {
    /// Create a new frame buffer
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
            timestamp: 0,
            format: PixelFormat::RGBA8,
        }
    }
    
    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}

/// Pixel format
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PixelFormat {
    #[default]
    RGBA8,
    RGB8,
    BGRA8,
    BGR8,
}

/// Browser errors
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserError {
    /// Invalid URL
    InvalidUrl(String),
    /// Navigation failed
    NavigationFailed(String),
    /// Page load timeout
    Timeout,
    /// Script error
    ScriptError(String),
    /// Resource not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
}

impl std::fmt::Display for BrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            BrowserError::NavigationFailed(reason) => write!(f, "Navigation failed: {}", reason),
            BrowserError::Timeout => write!(f, "Page load timeout"),
            BrowserError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            BrowserError::NotFound(resource) => write!(f, "Not found: {}", resource),
            BrowserError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for BrowserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_creation() {
        let browser = BrowserInstance::new("https://example.com");
        assert_eq!(browser.url, "https://example.com");
        assert!(browser.is_loading);
        assert!(!browser.can_go_back);
    }

    #[test]
    fn test_navigation() {
        let mut browser = BrowserInstance::new("https://page1.com");
        
        browser.navigate("https://page2.com").unwrap();
        assert_eq!(browser.url, "https://page2.com");
        assert!(browser.can_go_back);
        
        browser.navigate("https://page3.com").unwrap();
        assert_eq!(browser.url, "https://page3.com");
        assert_eq!(browser.history.len(), 3);
    }

    #[test]
    fn test_navigation_history() {
        let mut browser = BrowserInstance::new("https://page1.com");
        browser.navigate("https://page2.com").unwrap();
        browser.navigate("https://page3.com").unwrap();
        
        // Go back
        assert!(browser.go_back());
        assert_eq!(browser.url, "https://page2.com");
        assert!(browser.can_go_forward);
        
        // Go forward
        assert!(browser.go_forward());
        assert_eq!(browser.url, "https://page3.com");
        assert!(!browser.can_go_forward);
        
        // Go back twice
        browser.go_back();
        browser.go_back();
        assert_eq!(browser.url, "https://page1.com");
        assert!(!browser.can_go_back);
    }

    #[test]
    fn test_url_normalization() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        // Should add https://
        browser.navigate("example.com").unwrap();
        assert_eq!(browser.url, "https://example.com");
    }

    #[test]
    fn test_scrolling() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        assert_eq!(browser.scroll_position, 0.0);
        
        browser.scroll(0.5);
        assert!((browser.scroll_position - 0.5).abs() < 0.001);
        
        browser.scroll(0.8);
        assert!((browser.scroll_position - 1.0).abs() < 0.001); // Clamped to 1.0
        
        browser.scroll_to(0.25);
        assert!((browser.scroll_position - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_voice_scroll() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        browser.voice_scroll(ScrollDirection::Down, ScrollAmount::Page);
        assert!((browser.scroll_position - 0.5).abs() < 0.001);
        
        browser.voice_scroll(ScrollDirection::Up, ScrollAmount::HalfPage);
        assert!((browser.scroll_position - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_zoom() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        assert!((browser.zoom - 1.0).abs() < 0.001);
        
        browser.zoom_in();
        assert!(browser.zoom > 1.0);
        
        browser.zoom_out();
        browser.zoom_out();
        assert!(browser.zoom < 1.0);
        
        browser.reset_zoom();
        assert!((browser.zoom - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cursor() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        assert!(browser.cursor_position().is_none());
        
        browser.update_cursor(0.5, 0.5);
        assert_eq!(browser.cursor_position(), Some((0.5, 0.5)));
        
        // Test clamping
        browser.update_cursor(1.5, -0.5);
        assert_eq!(browser.cursor_position(), Some((1.0, 0.0)));
    }

    #[test]
    fn test_click() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        let result = browser.click(0.5, 0.5);
        assert!(matches!(result, ClickResult::Clicked { .. }));
        assert_eq!(browser.cursor_position(), Some((0.5, 0.5)));
    }

    #[test]
    fn test_page_load() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        assert!(browser.is_loading);
        
        browser.mark_loaded("Test Page");
        assert!(!browser.is_loading);
        assert_eq!(browser.title, "Test Page");
    }

    #[test]
    fn test_config() {
        let default = BrowserConfig::default();
        assert!(default.javascript_enabled);
        assert!(!default.private_mode);
        
        let private = BrowserConfig::private();
        assert!(private.private_mode);
        assert!(!private.cookies_enabled);
        
        let minimal = BrowserConfig::minimal();
        assert!(!minimal.webgl_enabled);
        assert_eq!(minimal.viewport_width, 1280);
    }

    #[test]
    fn test_frame_buffer() {
        let mut buffer = FrameBuffer::new(1920, 1080);
        
        assert_eq!(buffer.width, 1920);
        assert_eq!(buffer.height, 1080);
        assert_eq!(buffer.size_bytes(), 1920 * 1080 * 4);
        
        buffer.clear();
        assert!(buffer.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_keyboard_input() {
        let mut browser = BrowserInstance::new("https://test.com");
        
        browser.form_data.insert("_focused".to_string(), "Hello".to_string());
        browser.type_text(" World");
        assert_eq!(browser.form_data.get("_focused").unwrap(), "Hello World");
        
        browser.key_press(KeyCode::Backspace, Modifiers::default());
        assert_eq!(browser.form_data.get("_focused").unwrap(), "Hello Worl");
    }

    #[test]
    fn test_history_truncation() {
        let mut browser = BrowserInstance::new("https://page1.com");
        browser.navigate("https://page2.com").unwrap();
        browser.navigate("https://page3.com").unwrap();
        
        // Go back and navigate to new page
        browser.go_back();
        browser.navigate("https://page4.com").unwrap();
        
        // Forward history should be truncated
        assert_eq!(browser.history.len(), 3);
        assert_eq!(browser.history[2], "https://page4.com");
        assert!(!browser.can_go_forward);
    }
}
