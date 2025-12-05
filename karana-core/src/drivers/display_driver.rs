// Kāraṇa OS - Display Driver
// Low-level driver for smart glasses display hardware

use super::{Driver, DriverError, DriverInfo, DriverState, DriverStats, BusType, DmaBuffer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Display driver configuration
#[derive(Debug, Clone)]
pub struct DisplayDriverConfig {
    /// Frame buffer device path
    pub device_path: String,
    /// Display width
    pub width: u32,
    /// Display height
    pub height: u32,
    /// Bits per pixel
    pub bpp: u8,
    /// Refresh rate
    pub refresh_hz: u32,
    /// Enable DMA
    pub dma_enabled: bool,
    /// Enable double buffering
    pub double_buffer: bool,
    /// Panel type
    pub panel_type: PanelType,
}

impl Default for DisplayDriverConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/fb0".into(),
            width: 1920,
            height: 1080,
            bpp: 32,
            refresh_hz: 60,
            dma_enabled: true,
            double_buffer: true,
            panel_type: PanelType::MicroOled,
        }
    }
}

/// Panel types for smart glasses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    /// Micro OLED (Sony, etc.)
    MicroOled,
    /// LCoS (Liquid Crystal on Silicon)
    Lcos,
    /// DLP (Digital Light Processing)
    Dlp,
    /// Micro LED
    MicroLed,
    /// Waveguide combiner
    Waveguide,
    /// Birdbath combiner
    Birdbath,
    /// Laser scanning (Bosch, etc.)
    LaserScanning,
}

/// Display timing parameters
#[derive(Debug, Clone)]
pub struct DisplayTiming {
    /// Horizontal active pixels
    pub h_active: u32,
    /// Horizontal front porch
    pub h_front_porch: u32,
    /// Horizontal sync width
    pub h_sync: u32,
    /// Horizontal back porch
    pub h_back_porch: u32,
    /// Vertical active lines
    pub v_active: u32,
    /// Vertical front porch
    pub v_front_porch: u32,
    /// Vertical sync width
    pub v_sync: u32,
    /// Vertical back porch
    pub v_back_porch: u32,
    /// Pixel clock (Hz)
    pub pixel_clock: u64,
    /// Is interlaced
    pub interlaced: bool,
}

impl Default for DisplayTiming {
    fn default() -> Self {
        // Standard 1080p60 timing
        Self {
            h_active: 1920,
            h_front_porch: 88,
            h_sync: 44,
            h_back_porch: 148,
            v_active: 1080,
            v_front_porch: 4,
            v_sync: 5,
            v_back_porch: 36,
            pixel_clock: 148_500_000,
            interlaced: false,
        }
    }
}

/// Frame buffer info
#[derive(Debug, Clone)]
pub struct FrameBufferInfo {
    /// Virtual resolution X
    pub virt_x: u32,
    /// Virtual resolution Y  
    pub virt_y: u32,
    /// X offset
    pub x_offset: u32,
    /// Y offset
    pub y_offset: u32,
    /// Bits per pixel
    pub bpp: u8,
    /// Stride (bytes per line)
    pub stride: u32,
    /// Total size
    pub size: usize,
    /// Memory type
    pub mem_type: MemoryType,
}

/// Memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// System memory
    System,
    /// DMA-able memory
    Dma,
    /// Video memory
    Video,
    /// Contiguous
    Contiguous,
}

/// Display driver
#[derive(Debug)]
pub struct DisplayDriver {
    /// Configuration
    config: DisplayDriverConfig,
    /// Current state
    state: DriverState,
    /// Display timing
    timing: DisplayTiming,
    /// Frame buffer info
    fb_info: FrameBufferInfo,
    /// Front buffer
    front_buffer: Option<DmaBuffer>,
    /// Back buffer
    back_buffer: Option<DmaBuffer>,
    /// Current buffer index
    current_buffer: u8,
    /// Frames rendered
    frames_rendered: AtomicU64,
    /// VSync count
    vsync_count: AtomicU64,
    /// Statistics
    stats: DriverStats,
    /// Last vsync time
    last_vsync: Option<Instant>,
    /// Is blanked
    blanked: bool,
}

impl DisplayDriver {
    /// Create new display driver
    pub fn new(config: DisplayDriverConfig) -> Self {
        let stride = config.width * (config.bpp as u32 / 8);
        let size = (stride * config.height) as usize;

        Self {
            fb_info: FrameBufferInfo {
                virt_x: config.width,
                virt_y: if config.double_buffer { config.height * 2 } else { config.height },
                x_offset: 0,
                y_offset: 0,
                bpp: config.bpp,
                stride,
                size,
                mem_type: if config.dma_enabled { MemoryType::Dma } else { MemoryType::System },
            },
            config,
            state: DriverState::Unloaded,
            timing: DisplayTiming::default(),
            front_buffer: None,
            back_buffer: None,
            current_buffer: 0,
            frames_rendered: AtomicU64::new(0),
            vsync_count: AtomicU64::new(0),
            stats: DriverStats::default(),
            last_vsync: None,
            blanked: false,
        }
    }

    /// Get frame buffer info
    pub fn fb_info(&self) -> &FrameBufferInfo {
        &self.fb_info
    }

    /// Get timing info
    pub fn timing(&self) -> &DisplayTiming {
        &self.timing
    }

    /// Set timing
    pub fn set_timing(&mut self, timing: DisplayTiming) -> Result<(), DriverError> {
        if self.state == DriverState::Running {
            return Err(DriverError::Busy);
        }
        self.timing = timing;
        Ok(())
    }

    /// Get current buffer
    pub fn get_buffer(&mut self) -> Option<&mut [u8]> {
        if self.current_buffer == 0 {
            self.front_buffer.as_mut().map(|b| b.as_mut_slice())
        } else {
            self.back_buffer.as_mut().map(|b| b.as_mut_slice())
        }
    }

    /// Swap buffers
    pub fn swap_buffers(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        if self.config.double_buffer {
            self.current_buffer = 1 - self.current_buffer;
            self.fb_info.y_offset = if self.current_buffer == 0 { 
                0 
            } else { 
                self.config.height 
            };
        }

        self.frames_rendered.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_written += self.fb_info.size as u64;
        
        Ok(())
    }

    /// Wait for vsync
    pub fn wait_vsync(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        // Would use ioctl FBIO_WAITFORVSYNC
        self.vsync_count.fetch_add(1, Ordering::Relaxed);
        self.last_vsync = Some(Instant::now());
        
        Ok(())
    }

    /// Blank display
    pub fn blank(&mut self, blank: bool) -> Result<(), DriverError> {
        if self.state != DriverState::Running && self.state != DriverState::Ready {
            return Err(DriverError::NotLoaded);
        }
        self.blanked = blank;
        Ok(())
    }

    /// Is blanked
    pub fn is_blanked(&self) -> bool {
        self.blanked
    }

    /// Set brightness (0-255)
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }
        // Would write to backlight sysfs or use I2C
        Ok(())
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frames_rendered.load(Ordering::Relaxed)
    }

    /// Get vsync count
    pub fn vsync_count(&self) -> u64 {
        self.vsync_count.load(Ordering::Relaxed)
    }

    /// Direct memory write (for optimized rendering)
    pub fn write_region(&mut self, x: u32, y: u32, width: u32, height: u32, data: &[u8]) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        let bpp = self.fb_info.bpp as u32 / 8;
        let stride = self.fb_info.stride;
        let buffer = self.get_buffer().ok_or(DriverError::BufferOverflow)?;

        for row in 0..height {
            let dst_offset = ((y + row) * stride + x * bpp) as usize;
            let src_offset = (row * width * bpp) as usize;
            let copy_len = (width * bpp) as usize;

            if dst_offset + copy_len <= buffer.len() && src_offset + copy_len <= data.len() {
                buffer[dst_offset..dst_offset + copy_len]
                    .copy_from_slice(&data[src_offset..src_offset + copy_len]);
            }
        }

        self.stats.bytes_written += (width * height * bpp) as u64;
        Ok(())
    }
}

impl Driver for DisplayDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "karana-display".into(),
            version: "1.0.0".into(),
            vendor: "KaranaOS".into(),
            device_ids: vec!["display:mipi-dsi".into(), "display:fb".into()],
            loaded: self.state != DriverState::Unloaded,
            state: self.state,
        }
    }

    fn state(&self) -> DriverState {
        self.state
    }

    fn load(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Loading;
        // Would open /dev/fb0 or DRM device
        self.state = DriverState::Loaded;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), DriverError> {
        self.front_buffer = None;
        self.back_buffer = None;
        self.state = DriverState::Unloaded;
        Ok(())
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Loaded {
            return Err(DriverError::NotLoaded);
        }

        // Allocate frame buffers
        let size = self.fb_info.size;
        self.front_buffer = Some(DmaBuffer::new(size));
        if self.config.double_buffer {
            self.back_buffer = Some(DmaBuffer::new(size));
        }

        self.state = DriverState::Ready;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Ready {
            return Err(DriverError::NotLoaded);
        }
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Ready;
        Ok(())
    }

    fn suspend(&mut self) -> Result<(), DriverError> {
        self.blank(true)?;
        self.state = DriverState::Suspended;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), DriverError> {
        self.blank(false)?;
        self.state = DriverState::Running;
        Ok(())
    }

    fn stats(&self) -> DriverStats {
        DriverStats {
            bytes_written: self.stats.bytes_written,
            dma_transfers: self.frames_rendered.load(Ordering::Relaxed),
            ..self.stats.clone()
        }
    }

    fn test(&self) -> Result<(), DriverError> {
        if self.state == DriverState::Unloaded {
            return Err(DriverError::NotLoaded);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_driver_creation() {
        let driver = DisplayDriver::new(DisplayDriverConfig::default());
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_display_driver_lifecycle() {
        let mut driver = DisplayDriver::new(DisplayDriverConfig::default());
        
        driver.load().unwrap();
        assert_eq!(driver.state(), DriverState::Loaded);
        
        driver.init().unwrap();
        assert_eq!(driver.state(), DriverState::Ready);
        
        driver.start().unwrap();
        assert_eq!(driver.state(), DriverState::Running);
        
        driver.stop().unwrap();
        assert_eq!(driver.state(), DriverState::Ready);
        
        driver.unload().unwrap();
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_double_buffering() {
        let mut driver = DisplayDriver::new(DisplayDriverConfig {
            double_buffer: true,
            ..Default::default()
        });
        
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        assert!(driver.get_buffer().is_some());
        driver.swap_buffers().unwrap();
        assert_eq!(driver.frame_count(), 1);
    }

    #[test]
    fn test_blanking() {
        let mut driver = DisplayDriver::new(DisplayDriverConfig::default());
        driver.load().unwrap();
        driver.init().unwrap();
        
        driver.blank(true).unwrap();
        assert!(driver.is_blanked());
        
        driver.blank(false).unwrap();
        assert!(!driver.is_blanked());
    }

    #[test]
    fn test_fb_info() {
        let config = DisplayDriverConfig {
            width: 1280,
            height: 720,
            bpp: 32,
            ..Default::default()
        };
        let driver = DisplayDriver::new(config);
        let info = driver.fb_info();
        
        assert_eq!(info.virt_x, 1280);
        assert_eq!(info.stride, 1280 * 4);
    }

    #[test]
    fn test_driver_info() {
        let driver = DisplayDriver::new(DisplayDriverConfig::default());
        let info = driver.info();
        
        assert_eq!(info.name, "karana-display");
        assert!(!info.loaded);
    }
}
