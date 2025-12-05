// Kāraṇa OS - Camera Driver
// Low-level driver for smart glasses camera hardware

use super::{Driver, DriverError, DriverInfo, DriverState, DriverStats, BusType, DmaBuffer};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Camera driver configuration
#[derive(Debug, Clone)]
pub struct CameraDriverConfig {
    /// Device path
    pub device_path: String,
    /// Sensor width
    pub width: u32,
    /// Sensor height
    pub height: u32,
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Frame rate
    pub fps: u32,
    /// Number of buffers
    pub buffer_count: u32,
    /// Enable streaming
    pub streaming: bool,
    /// Camera index
    pub camera_index: u8,
}

impl Default for CameraDriverConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/video0".into(),
            width: 1920,
            height: 1080,
            pixel_format: PixelFormat::Nv12,
            fps: 30,
            buffer_count: 4,
            streaming: true,
            camera_index: 0,
        }
    }
}

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB 24-bit
    Rgb24,
    /// RGBA 32-bit
    Rgba32,
    /// BGR 24-bit
    Bgr24,
    /// BGRA 32-bit
    Bgra32,
    /// YUV 4:2:2 (YUYV)
    Yuyv,
    /// YUV 4:2:0 planar (NV12)
    Nv12,
    /// YUV 4:2:0 planar (NV21)
    Nv21,
    /// YUV 4:2:0 planar (I420)
    I420,
    /// RAW8 (Bayer)
    Raw8,
    /// RAW10 (packed)
    Raw10,
    /// RAW12 (packed)
    Raw12,
    /// Grayscale 8-bit
    Gray8,
    /// Grayscale 16-bit
    Gray16,
    /// MJPEG
    Mjpeg,
    /// H.264
    H264,
    /// Depth 16-bit
    Depth16,
}

impl PixelFormat {
    /// Get bytes per pixel (for packed formats)
    pub fn bytes_per_pixel(&self) -> Option<f32> {
        match self {
            PixelFormat::Rgb24 | PixelFormat::Bgr24 => Some(3.0),
            PixelFormat::Rgba32 | PixelFormat::Bgra32 => Some(4.0),
            PixelFormat::Yuyv => Some(2.0),
            PixelFormat::Nv12 | PixelFormat::Nv21 | PixelFormat::I420 => Some(1.5),
            PixelFormat::Gray8 | PixelFormat::Raw8 => Some(1.0),
            PixelFormat::Gray16 | PixelFormat::Depth16 => Some(2.0),
            PixelFormat::Raw10 => Some(1.25),
            PixelFormat::Raw12 => Some(1.5),
            _ => None, // Compressed formats
        }
    }
}

/// V4L2 buffer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V4l2BufType {
    /// Video capture
    VideoCapture,
    /// Video capture multiplanar
    VideoCaptureMplane,
    /// Video output
    VideoOutput,
    /// Metadata capture
    MetaCapture,
}

/// V4L2 memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V4l2Memory {
    /// Memory mapped
    Mmap,
    /// User pointer
    UserPtr,
    /// DMA buffer
    DmaBuf,
}

/// Camera buffer
#[derive(Debug)]
pub struct CameraBuffer {
    /// Buffer index
    pub index: u32,
    /// Buffer data
    pub data: DmaBuffer,
    /// Timestamp (microseconds)
    pub timestamp: u64,
    /// Sequence number
    pub sequence: u32,
    /// Is queued
    pub queued: bool,
    /// Bytes used
    pub bytes_used: usize,
}

/// Camera controls
#[derive(Debug, Clone)]
pub struct CameraControls {
    /// Brightness (-100 to 100)
    pub brightness: i32,
    /// Contrast (0 to 100)
    pub contrast: i32,
    /// Saturation (0 to 100)
    pub saturation: i32,
    /// Hue (-180 to 180)
    pub hue: i32,
    /// Sharpness (0 to 100)
    pub sharpness: i32,
    /// Exposure mode
    pub exposure_mode: ExposureMode,
    /// Manual exposure (us)
    pub exposure_us: u32,
    /// ISO/gain
    pub iso: u32,
    /// White balance mode
    pub white_balance: WhiteBalanceMode,
    /// Manual color temperature (K)
    pub color_temp: u32,
    /// Auto focus mode
    pub focus_mode: FocusMode,
    /// Manual focus position
    pub focus_pos: u32,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            brightness: 0,
            contrast: 50,
            saturation: 50,
            hue: 0,
            sharpness: 50,
            exposure_mode: ExposureMode::Auto,
            exposure_us: 16666,
            iso: 100,
            white_balance: WhiteBalanceMode::Auto,
            color_temp: 5500,
            focus_mode: FocusMode::Auto,
            focus_pos: 0,
        }
    }
}

/// Exposure mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureMode {
    Auto,
    Manual,
    ShutterPriority,
    AperturePriority,
}

/// White balance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalanceMode {
    Auto,
    Daylight,
    Cloudy,
    Shade,
    Tungsten,
    Fluorescent,
    Flash,
    Manual,
}

/// Focus mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    Auto,
    ContinuousAuto,
    Manual,
    Infinity,
    Macro,
}

/// Camera driver
#[derive(Debug)]
pub struct CameraDriver {
    /// Configuration
    config: CameraDriverConfig,
    /// Current state
    state: DriverState,
    /// Capture buffers
    buffers: Vec<CameraBuffer>,
    /// Ready buffer queue
    ready_queue: VecDeque<u32>,
    /// Controls
    controls: CameraControls,
    /// Frame counter
    frame_count: AtomicU64,
    /// Dropped frames
    dropped_frames: AtomicU64,
    /// Statistics
    stats: DriverStats,
    /// Is streaming
    streaming: AtomicBool,
    /// Last frame time
    last_frame_time: Option<Instant>,
}

impl CameraDriver {
    /// Create new camera driver
    pub fn new(config: CameraDriverConfig) -> Self {
        Self {
            config,
            state: DriverState::Unloaded,
            buffers: Vec::new(),
            ready_queue: VecDeque::new(),
            controls: CameraControls::default(),
            frame_count: AtomicU64::new(0),
            dropped_frames: AtomicU64::new(0),
            stats: DriverStats::default(),
            streaming: AtomicBool::new(false),
            last_frame_time: None,
        }
    }

    /// Get controls
    pub fn controls(&self) -> &CameraControls {
        &self.controls
    }

    /// Set control
    pub fn set_control(&mut self, control: &str, value: i32) -> Result<(), DriverError> {
        match control {
            "brightness" => self.controls.brightness = value.clamp(-100, 100),
            "contrast" => self.controls.contrast = value.clamp(0, 100),
            "saturation" => self.controls.saturation = value.clamp(0, 100),
            "hue" => self.controls.hue = value.clamp(-180, 180),
            "sharpness" => self.controls.sharpness = value.clamp(0, 100),
            "exposure_us" => self.controls.exposure_us = value as u32,
            "iso" => self.controls.iso = value as u32,
            _ => return Err(DriverError::InvalidParameter(format!("Unknown control: {}", control))),
        }
        Ok(())
    }

    /// Set exposure mode
    pub fn set_exposure_mode(&mut self, mode: ExposureMode) -> Result<(), DriverError> {
        self.controls.exposure_mode = mode;
        Ok(())
    }

    /// Set white balance mode
    pub fn set_white_balance(&mut self, mode: WhiteBalanceMode) -> Result<(), DriverError> {
        self.controls.white_balance = mode;
        Ok(())
    }

    /// Set focus mode
    pub fn set_focus_mode(&mut self, mode: FocusMode) -> Result<(), DriverError> {
        self.controls.focus_mode = mode;
        Ok(())
    }

    /// Start streaming
    pub fn start_streaming(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Ready && self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        // Queue all buffers
        for buf in &mut self.buffers {
            buf.queued = true;
            self.ready_queue.push_back(buf.index);
        }

        self.streaming.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Stop streaming
    pub fn stop_streaming(&mut self) -> Result<(), DriverError> {
        self.streaming.store(false, Ordering::Relaxed);
        self.ready_queue.clear();
        for buf in &mut self.buffers {
            buf.queued = false;
        }
        Ok(())
    }

    /// Dequeue buffer (get captured frame)
    pub fn dequeue_buffer(&mut self) -> Result<&CameraBuffer, DriverError> {
        if !self.streaming.load(Ordering::Relaxed) {
            return Err(DriverError::NotLoaded);
        }

        if let Some(idx) = self.ready_queue.pop_front() {
            let buffer = &mut self.buffers[idx as usize];
            buffer.queued = false;
            buffer.sequence = self.frame_count.fetch_add(1, Ordering::Relaxed) as u32;
            buffer.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;

            // Calculate bytes used
            if let Some(bpp) = self.config.pixel_format.bytes_per_pixel() {
                buffer.bytes_used = (self.config.width as f32 * self.config.height as f32 * bpp) as usize;
            }

            self.stats.bytes_read += buffer.bytes_used as u64;
            self.last_frame_time = Some(Instant::now());

            Ok(&self.buffers[idx as usize])
        } else {
            Err(DriverError::Timeout)
        }
    }

    /// Queue buffer (return to driver)
    pub fn queue_buffer(&mut self, index: u32) -> Result<(), DriverError> {
        if index as usize >= self.buffers.len() {
            return Err(DriverError::InvalidParameter("Invalid buffer index".into()));
        }

        self.buffers[index as usize].queued = true;
        self.ready_queue.push_back(index);
        Ok(())
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::Relaxed)
    }

    /// Get dropped frame count
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames.load(Ordering::Relaxed)
    }

    /// Is streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming.load(Ordering::Relaxed)
    }

    /// Get buffer data
    pub fn get_buffer_data(&self, index: u32) -> Option<&[u8]> {
        self.buffers.get(index as usize).map(|b| b.data.as_slice())
    }
}

impl Driver for CameraDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "karana-camera".into(),
            version: "1.0.0".into(),
            vendor: "KaranaOS".into(),
            device_ids: vec!["camera:v4l2".into(), "camera:mipi-csi".into()],
            loaded: self.state != DriverState::Unloaded,
            state: self.state,
        }
    }

    fn state(&self) -> DriverState {
        self.state
    }

    fn load(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Loading;
        // Would open /dev/videoX
        self.state = DriverState::Loaded;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), DriverError> {
        self.stop_streaming()?;
        self.buffers.clear();
        self.state = DriverState::Unloaded;
        Ok(())
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Loaded {
            return Err(DriverError::NotLoaded);
        }

        // Calculate buffer size
        let buffer_size = if let Some(bpp) = self.config.pixel_format.bytes_per_pixel() {
            (self.config.width as f32 * self.config.height as f32 * bpp) as usize
        } else {
            // For compressed formats, allocate generous buffer
            (self.config.width * self.config.height) as usize
        };

        // Allocate buffers
        for i in 0..self.config.buffer_count {
            self.buffers.push(CameraBuffer {
                index: i,
                data: DmaBuffer::new(buffer_size),
                timestamp: 0,
                sequence: 0,
                queued: false,
                bytes_used: 0,
            });
        }

        self.state = DriverState::Ready;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Ready {
            return Err(DriverError::NotLoaded);
        }
        self.state = DriverState::Running;
        if self.config.streaming {
            self.start_streaming()?;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.stop_streaming()?;
        self.state = DriverState::Ready;
        Ok(())
    }

    fn suspend(&mut self) -> Result<(), DriverError> {
        self.stop_streaming()?;
        self.state = DriverState::Suspended;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Running;
        if self.config.streaming {
            self.start_streaming()?;
        }
        Ok(())
    }

    fn stats(&self) -> DriverStats {
        DriverStats {
            bytes_read: self.stats.bytes_read,
            dma_transfers: self.frame_count.load(Ordering::Relaxed),
            errors: self.dropped_frames.load(Ordering::Relaxed),
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
    fn test_camera_driver_creation() {
        let driver = CameraDriver::new(CameraDriverConfig::default());
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_camera_driver_lifecycle() {
        let mut driver = CameraDriver::new(CameraDriverConfig {
            buffer_count: 2,
            streaming: false,
            ..Default::default()
        });
        
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        assert_eq!(driver.buffers.len(), 2);
        
        driver.stop().unwrap();
        driver.unload().unwrap();
    }

    #[test]
    fn test_streaming() {
        let mut driver = CameraDriver::new(CameraDriverConfig {
            buffer_count: 4,
            streaming: false,
            ..Default::default()
        });
        
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        driver.start_streaming().unwrap();
        assert!(driver.is_streaming());
        
        // Dequeue and queue buffers
        let _buf = driver.dequeue_buffer().unwrap();
        driver.queue_buffer(0).unwrap();
        
        driver.stop_streaming().unwrap();
        assert!(!driver.is_streaming());
    }

    #[test]
    fn test_controls() {
        let mut driver = CameraDriver::new(CameraDriverConfig::default());
        
        driver.set_control("brightness", 50).unwrap();
        assert_eq!(driver.controls().brightness, 50);
        
        driver.set_control("contrast", 75).unwrap();
        assert_eq!(driver.controls().contrast, 75);
        
        driver.set_exposure_mode(ExposureMode::Manual).unwrap();
        assert_eq!(driver.controls().exposure_mode, ExposureMode::Manual);
    }

    #[test]
    fn test_pixel_format() {
        assert_eq!(PixelFormat::Rgb24.bytes_per_pixel(), Some(3.0));
        assert_eq!(PixelFormat::Nv12.bytes_per_pixel(), Some(1.5));
        assert!(PixelFormat::Mjpeg.bytes_per_pixel().is_none());
    }

    #[test]
    fn test_driver_info() {
        let driver = CameraDriver::new(CameraDriverConfig::default());
        let info = driver.info();
        
        assert_eq!(info.name, "karana-camera");
        assert!(!info.loaded);
    }
}
