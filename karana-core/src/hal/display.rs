// Kāraṇa OS - Display HAL
// Hardware abstraction for smart glasses displays

use super::HalError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Display configuration
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    /// Resolution width
    pub width: u32,
    /// Resolution height
    pub height: u32,
    /// Refresh rate (Hz)
    pub refresh_rate: u32,
    /// Color depth (bits)
    pub color_depth: u32,
    /// Target brightness (0-100)
    pub brightness: u8,
    /// Enable HDR
    pub hdr: bool,
    /// Enable variable refresh rate
    pub vrr: bool,
    /// Display mode
    pub mode: DisplayMode,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            refresh_rate: 90,
            color_depth: 24,
            brightness: 70,
            hdr: true,
            vrr: true,
            mode: DisplayMode::Stereo,
        }
    }
}

/// Display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Single display (monocular)
    Mono,
    /// Stereo display (one per eye)
    Stereo,
    /// See-through AR mode
    SeeThrough,
    /// Passthrough video mode
    Passthrough,
}

/// Display capabilities
#[derive(Debug, Clone)]
pub struct DisplayCapabilities {
    /// Maximum width
    pub max_width: u32,
    /// Maximum height
    pub max_height: u32,
    /// Maximum refresh rate
    pub max_refresh_rate: u32,
    /// Supported color depths
    pub color_depths: Vec<u32>,
    /// Supports HDR
    pub hdr_capable: bool,
    /// Supports variable refresh rate
    pub vrr_capable: bool,
    /// Display type
    pub display_type: DisplayType,
    /// Field of view (degrees)
    pub fov_horizontal: f32,
    /// Vertical FOV
    pub fov_vertical: f32,
    /// Inter-pupillary distance range (mm)
    pub ipd_range: (f32, f32),
}

impl Default for DisplayCapabilities {
    fn default() -> Self {
        Self {
            max_width: 2560,
            max_height: 1440,
            max_refresh_rate: 120,
            color_depths: vec![24, 30, 36],
            hdr_capable: true,
            vrr_capable: true,
            display_type: DisplayType::MicroOled,
            fov_horizontal: 52.0,
            fov_vertical: 45.0,
            ipd_range: (58.0, 72.0),
        }
    }
}

/// Display technology type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayType {
    /// LCD display
    Lcd,
    /// OLED display
    Oled,
    /// MicroOLED
    MicroOled,
    /// LCoS (Liquid Crystal on Silicon)
    Lcos,
    /// DLP (Digital Light Processing)
    Dlp,
    /// Waveguide
    Waveguide,
    /// Holographic
    Holographic,
}

/// Display state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayState {
    /// Display off
    Off,
    /// Standby mode
    Standby,
    /// Active
    Active,
    /// Low power mode
    LowPower,
    /// Dimmed
    Dimmed,
}

/// Display HAL
#[derive(Debug)]
pub struct Display {
    /// Configuration
    config: DisplayConfig,
    /// Capabilities
    capabilities: DisplayCapabilities,
    /// Current state
    state: DisplayState,
    /// Current brightness
    brightness: u8,
    /// Current IPD setting
    ipd_mm: f32,
    /// Frame counter
    frame_counter: AtomicU64,
    /// Statistics
    stats: DisplayStats,
    /// Is initialized
    initialized: bool,
}

/// Display statistics
#[derive(Debug, Default, Clone)]
pub struct DisplayStats {
    /// Frames rendered
    pub frames_rendered: u64,
    /// Frames dropped
    pub frames_dropped: u64,
    /// Average frame time (ms)
    pub avg_frame_time_ms: f64,
    /// Current refresh rate
    pub current_refresh_rate: u32,
    /// Display on time (seconds)
    pub display_on_time_secs: f64,
}

impl Display {
    /// Create new display HAL
    pub fn new(config: DisplayConfig) -> Result<Self, HalError> {
        Ok(Self {
            config,
            capabilities: DisplayCapabilities::default(),
            state: DisplayState::Off,
            brightness: 70,
            ipd_mm: 64.0,
            frame_counter: AtomicU64::new(0),
            stats: DisplayStats::default(),
            initialized: false,
        })
    }

    /// Initialize display
    pub fn initialize(&mut self) -> Result<(), HalError> {
        // Detect display capabilities
        self.detect_capabilities()?;

        // Validate config against capabilities
        self.validate_config()?;

        self.initialized = true;
        Ok(())
    }

    /// Start display
    pub fn start(&mut self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Display not initialized".into()));
        }

        self.state = DisplayState::Active;
        self.set_brightness(self.config.brightness)?;

        Ok(())
    }

    /// Stop display
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.state = DisplayState::Off;
        Ok(())
    }

    /// Suspend display
    pub fn suspend(&mut self) -> Result<(), HalError> {
        self.state = DisplayState::Standby;
        Ok(())
    }

    /// Resume display
    pub fn resume(&mut self) -> Result<(), HalError> {
        self.state = DisplayState::Active;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> DisplayState {
        self.state
    }

    /// Get capabilities
    pub fn capabilities(&self) -> DisplayCapabilities {
        self.capabilities.clone()
    }

    /// Get statistics
    pub fn stats(&self) -> DisplayStats {
        DisplayStats {
            frames_rendered: self.frame_counter.load(Ordering::Relaxed),
            ..self.stats.clone()
        }
    }

    /// Set brightness (0-100)
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), HalError> {
        if brightness > 100 {
            return Err(HalError::ConfigError("Brightness must be 0-100".into()));
        }

        self.brightness = brightness;

        // Update dimmed state
        if brightness < 20 {
            self.state = DisplayState::Dimmed;
        } else if self.state == DisplayState::Dimmed {
            self.state = DisplayState::Active;
        }

        Ok(())
    }

    /// Get current brightness
    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    /// Set IPD (inter-pupillary distance)
    pub fn set_ipd(&mut self, ipd_mm: f32) -> Result<(), HalError> {
        let (min, max) = self.capabilities.ipd_range;
        if ipd_mm < min || ipd_mm > max {
            return Err(HalError::ConfigError(format!(
                "IPD must be between {} and {} mm",
                min, max
            )));
        }

        self.ipd_mm = ipd_mm;
        Ok(())
    }

    /// Get current IPD
    pub fn ipd(&self) -> f32 {
        self.ipd_mm
    }

    /// Set display mode
    pub fn set_mode(&mut self, mode: DisplayMode) -> Result<(), HalError> {
        self.config.mode = mode;
        Ok(())
    }

    /// Get display mode
    pub fn mode(&self) -> DisplayMode {
        self.config.mode
    }

    /// Present frame
    pub fn present(&mut self) -> Result<(), HalError> {
        if self.state != DisplayState::Active {
            return Err(HalError::DeviceBusy);
        }

        self.frame_counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Set resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), HalError> {
        if width > self.capabilities.max_width || height > self.capabilities.max_height {
            return Err(HalError::ConfigError("Resolution exceeds capabilities".into()));
        }

        self.config.width = width;
        self.config.height = height;
        Ok(())
    }

    /// Get resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Set refresh rate
    pub fn set_refresh_rate(&mut self, rate: u32) -> Result<(), HalError> {
        if rate > self.capabilities.max_refresh_rate {
            return Err(HalError::ConfigError("Refresh rate exceeds capabilities".into()));
        }

        self.config.refresh_rate = rate;
        self.stats.current_refresh_rate = rate;
        Ok(())
    }

    /// Get refresh rate
    pub fn refresh_rate(&self) -> u32 {
        self.config.refresh_rate
    }

    /// Enable/disable HDR
    pub fn set_hdr(&mut self, enabled: bool) -> Result<(), HalError> {
        if enabled && !self.capabilities.hdr_capable {
            return Err(HalError::NotSupported);
        }

        self.config.hdr = enabled;
        Ok(())
    }

    /// Self-test
    pub fn test(&self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Not initialized".into()));
        }
        Ok(())
    }

    /// Detect capabilities
    fn detect_capabilities(&mut self) -> Result<(), HalError> {
        // Would query actual hardware in real implementation
        self.capabilities = DisplayCapabilities::default();
        Ok(())
    }

    /// Validate configuration
    fn validate_config(&self) -> Result<(), HalError> {
        if self.config.width > self.capabilities.max_width {
            return Err(HalError::ConfigError("Width exceeds maximum".into()));
        }
        if self.config.height > self.capabilities.max_height {
            return Err(HalError::ConfigError("Height exceeds maximum".into()));
        }
        if self.config.refresh_rate > self.capabilities.max_refresh_rate {
            return Err(HalError::ConfigError("Refresh rate exceeds maximum".into()));
        }
        Ok(())
    }
}

/// Display buffer for double/triple buffering
#[derive(Debug)]
pub struct DisplayBuffer {
    /// Buffer width
    pub width: u32,
    /// Buffer height  
    pub height: u32,
    /// Pixel data
    pub data: Vec<u8>,
    /// Is dirty (needs flip)
    pub dirty: bool,
}

impl DisplayBuffer {
    /// Create new display buffer
    pub fn new(width: u32, height: u32, color_depth: u32) -> Self {
        let bytes_per_pixel = color_depth / 8;
        let size = (width * height * bytes_per_pixel) as usize;
        
        Self {
            width,
            height,
            data: vec![0; size],
            dirty: false,
        }
    }

    /// Clear buffer
    pub fn clear(&mut self, color: u32) {
        let bytes = color.to_le_bytes();
        for chunk in self.data.chunks_mut(3) {
            chunk[0] = bytes[0];
            chunk[1] = bytes[1];
            chunk[2] = bytes[2];
        }
        self.dirty = true;
    }

    /// Mark as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.refresh_rate, 90);
    }

    #[test]
    fn test_display_capabilities() {
        let caps = DisplayCapabilities::default();
        assert!(caps.hdr_capable);
        assert!(caps.vrr_capable);
        assert_eq!(caps.display_type, DisplayType::MicroOled);
    }

    #[test]
    fn test_display_creation() {
        let config = DisplayConfig::default();
        let display = Display::new(config);
        assert!(display.is_ok());
    }

    #[test]
    fn test_display_initialization() {
        let mut display = Display::new(DisplayConfig::default()).unwrap();
        assert!(display.initialize().is_ok());
    }

    #[test]
    fn test_display_brightness() {
        let mut display = Display::new(DisplayConfig::default()).unwrap();
        display.initialize().unwrap();

        display.set_brightness(50).unwrap();
        assert_eq!(display.brightness(), 50);

        assert!(display.set_brightness(101).is_err());
    }

    #[test]
    fn test_display_ipd() {
        let mut display = Display::new(DisplayConfig::default()).unwrap();
        display.initialize().unwrap();

        display.set_ipd(65.0).unwrap();
        assert_eq!(display.ipd(), 65.0);

        // Out of range
        assert!(display.set_ipd(50.0).is_err());
    }

    #[test]
    fn test_display_start_stop() {
        let mut display = Display::new(DisplayConfig::default()).unwrap();
        display.initialize().unwrap();

        display.start().unwrap();
        assert_eq!(display.state(), DisplayState::Active);

        display.stop().unwrap();
        assert_eq!(display.state(), DisplayState::Off);
    }

    #[test]
    fn test_display_buffer() {
        let mut buffer = DisplayBuffer::new(100, 100, 24);
        assert_eq!(buffer.data.len(), 100 * 100 * 3);

        buffer.clear(0xFF0000); // Red
        assert!(buffer.dirty);
    }

    #[test]
    fn test_display_present() {
        let mut display = Display::new(DisplayConfig::default()).unwrap();
        display.initialize().unwrap();
        display.start().unwrap();

        display.present().unwrap();
        display.present().unwrap();

        assert_eq!(display.stats().frames_rendered, 2);
    }
}
