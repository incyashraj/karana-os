// Kāraṇa OS - Video Capture Module
// Camera capture and device management

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::frame::{VideoFrame, PixelFormat};
use super::VideoError;

/// Video capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Target framerate
    pub framerate: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Capture source
    pub source: CaptureSource,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            framerate: 30,
            format: PixelFormat::NV12,
            source: CaptureSource::Default,
        }
    }
}

/// Capture source selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureSource {
    /// System default camera
    Default,
    /// Specific camera by name
    Camera(String),
    /// Specific camera by index
    CameraIndex(usize),
    /// Front-facing camera
    FrontCamera,
    /// Rear camera
    RearCamera,
    /// Screen capture
    Screen,
    /// Window capture by ID
    Window(u64),
    /// Test pattern generator
    TestPattern,
}

/// Camera information
#[derive(Debug, Clone)]
pub struct CameraInfo {
    /// Camera name
    pub name: String,
    /// Camera ID
    pub id: String,
    /// Device path
    pub device_path: String,
    /// Is front-facing
    pub front_facing: bool,
    /// Supported resolutions
    pub resolutions: Vec<(u32, u32)>,
    /// Supported framerates
    pub framerates: Vec<u32>,
    /// Supported formats
    pub formats: Vec<PixelFormat>,
    /// Has autofocus
    pub has_autofocus: bool,
    /// Has flash
    pub has_flash: bool,
    /// Has zoom
    pub has_zoom: bool,
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
}

/// Camera controls
#[derive(Debug, Clone)]
pub struct CameraControls {
    /// Brightness (-1.0 to 1.0)
    pub brightness: f32,
    /// Contrast (0.0 to 2.0)
    pub contrast: f32,
    /// Saturation (0.0 to 2.0)
    pub saturation: f32,
    /// Hue (-180.0 to 180.0)
    pub hue: f32,
    /// Gamma (0.0 to 2.0)
    pub gamma: f32,
    /// Sharpness (0.0 to 1.0)
    pub sharpness: f32,
    /// White balance temperature (2000 to 10000K)
    pub white_balance: u32,
    /// Auto white balance
    pub auto_white_balance: bool,
    /// Exposure compensation (-4.0 to 4.0 EV)
    pub exposure: f32,
    /// Auto exposure
    pub auto_exposure: bool,
    /// Focus distance (0.0 = infinity, 1.0 = macro)
    pub focus: f32,
    /// Auto focus
    pub auto_focus: bool,
    /// Zoom level
    pub zoom: f32,
    /// Flash mode
    pub flash: FlashMode,
    /// Anti-flicker mode
    pub anti_flicker: AntiFlickerMode,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue: 0.0,
            gamma: 1.0,
            sharpness: 0.5,
            white_balance: 5500,
            auto_white_balance: true,
            exposure: 0.0,
            auto_exposure: true,
            focus: 0.0,
            auto_focus: true,
            zoom: 1.0,
            flash: FlashMode::Off,
            anti_flicker: AntiFlickerMode::Auto,
        }
    }
}

/// Flash mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashMode {
    Off,
    On,
    Auto,
    Torch,
    RedEye,
}

/// Anti-flicker mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiFlickerMode {
    Off,
    Auto,
    Hz50,
    Hz60,
}

/// Video capture manager
#[derive(Debug)]
pub struct VideoCapture {
    /// Configuration
    config: CaptureConfig,
    /// Camera controls
    controls: CameraControls,
    /// Running state
    running: Arc<AtomicBool>,
    /// Frame buffer
    buffer: VecDeque<VideoFrame>,
    /// Buffer capacity
    buffer_capacity: usize,
    /// Frame sequence counter
    sequence: u64,
    /// Start time
    start_time: Option<Instant>,
    /// Test pattern phase
    test_phase: u32,
    /// Current camera info
    camera_info: Option<CameraInfo>,
}

impl VideoCapture {
    /// Create new video capture
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            controls: CameraControls::default(),
            running: Arc::new(AtomicBool::new(false)),
            buffer: VecDeque::with_capacity(10),
            buffer_capacity: 10,
            sequence: 0,
            start_time: None,
            test_phase: 0,
            camera_info: None,
        }
    }

    /// Start capturing
    pub fn start(&mut self) -> Result<(), VideoError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // In real implementation, would open camera device here
        match &self.config.source {
            CaptureSource::Default | CaptureSource::Camera(_) | 
            CaptureSource::CameraIndex(_) | CaptureSource::FrontCamera |
            CaptureSource::RearCamera => {
                // Would open V4L2/AVFoundation/MediaFoundation device
                self.camera_info = Some(CameraInfo {
                    name: "Simulated Camera".to_string(),
                    id: "sim_cam_0".to_string(),
                    device_path: "/dev/video0".to_string(),
                    front_facing: matches!(&self.config.source, CaptureSource::FrontCamera),
                    resolutions: vec![(1920, 1080), (1280, 720), (640, 480)],
                    framerates: vec![30, 60, 120],
                    formats: vec![PixelFormat::NV12, PixelFormat::YUYV, PixelFormat::RGB24],
                    has_autofocus: true,
                    has_flash: true,
                    has_zoom: true,
                    min_zoom: 1.0,
                    max_zoom: 8.0,
                });
            }
            CaptureSource::Screen | CaptureSource::Window(_) => {
                // Would setup screen capture
            }
            CaptureSource::TestPattern => {
                // Test mode - generates pattern
            }
        }

        self.running.store(true, Ordering::SeqCst);
        self.start_time = Some(Instant::now());
        self.sequence = 0;
        Ok(())
    }

    /// Stop capturing
    pub fn stop(&mut self) -> Result<(), VideoError> {
        self.running.store(false, Ordering::SeqCst);
        self.buffer.clear();
        Ok(())
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Capture next frame
    pub fn capture(&mut self) -> Result<VideoFrame, VideoError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(VideoError::NotRunning);
        }

        // Check buffer first
        if let Some(frame) = self.buffer.pop_front() {
            return Ok(frame);
        }

        // Generate frame based on source
        let frame = match &self.config.source {
            CaptureSource::TestPattern => self.generate_test_pattern(),
            _ => self.capture_from_device()?,
        };

        Ok(frame)
    }

    /// Generate test pattern
    fn generate_test_pattern(&mut self) -> VideoFrame {
        let width = self.config.width;
        let height = self.config.height;
        let mut frame = VideoFrame::new(width, height, PixelFormat::RGB24);

        // Generate color bars with moving element
        let bar_width = width / 8;

        for y in 0..height {
            for x in 0..width {
                let bar = x / bar_width;
                let (r, g, b) = match bar {
                    0 => (255, 255, 255), // White
                    1 => (255, 255, 0),   // Yellow
                    2 => (0, 255, 255),   // Cyan
                    3 => (0, 255, 0),     // Green
                    4 => (255, 0, 255),   // Magenta
                    5 => (255, 0, 0),     // Red
                    6 => (0, 0, 255),     // Blue
                    _ => (0, 0, 0),       // Black
                };

                // Add moving element
                let center_x = ((self.test_phase as f32 / 100.0).sin() * 0.4 + 0.5) * width as f32;
                let center_y = ((self.test_phase as f32 / 80.0).cos() * 0.4 + 0.5) * height as f32;
                let dist = (((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)) as f32).sqrt();

                let (r, g, b) = if dist < 50.0 {
                    (255, 0, 0) // Red circle
                } else {
                    (r, g, b)
                };

                frame.set_pixel(x, y, r, g, b, 255);
            }
        }

        self.test_phase = self.test_phase.wrapping_add(1);

        let timestamp = self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0);

        self.sequence += 1;
        frame.timestamp = timestamp;
        frame.sequence = self.sequence;
        frame.pts = timestamp;
        frame.duration = 1_000_000 / self.config.framerate as u64;

        frame
    }

    /// Capture from actual device (simulated)
    fn capture_from_device(&mut self) -> Result<VideoFrame, VideoError> {
        // In real implementation, would read from device
        // For now, return a gray frame
        let width = self.config.width;
        let height = self.config.height;
        let mut frame = VideoFrame::new(width, height, self.config.format);

        // Fill with gray
        match self.config.format {
            PixelFormat::RGB24 => {
                for chunk in frame.data.chunks_mut(3) {
                    chunk[0] = 128;
                    chunk[1] = 128;
                    chunk[2] = 128;
                }
            }
            PixelFormat::NV12 => {
                // Y = 128, UV = 128
                let y_size = (width * height) as usize;
                for i in 0..y_size {
                    frame.data[i] = 128;
                }
                for i in y_size..frame.data.len() {
                    frame.data[i] = 128;
                }
            }
            _ => {
                frame.data.fill(128);
            }
        }

        let timestamp = self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0);

        self.sequence += 1;
        frame.timestamp = timestamp;
        frame.sequence = self.sequence;
        frame.pts = timestamp;
        frame.duration = 1_000_000 / self.config.framerate as u64;

        Ok(frame)
    }

    /// Get available frames in buffer
    pub fn available(&self) -> usize {
        self.buffer.len()
    }

    /// Clear buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &CaptureConfig {
        &self.config
    }

    /// Set configuration (must be stopped)
    pub fn set_config(&mut self, config: CaptureConfig) -> Result<(), VideoError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(VideoError::ConfigError("Cannot change config while running".into()));
        }
        self.config = config;
        Ok(())
    }

    /// Get camera controls
    pub fn controls(&self) -> &CameraControls {
        &self.controls
    }

    /// Set camera controls
    pub fn set_controls(&mut self, controls: CameraControls) {
        self.controls = controls;
        // In real implementation, would apply to device
    }

    /// Set brightness
    pub fn set_brightness(&mut self, value: f32) {
        self.controls.brightness = value.clamp(-1.0, 1.0);
    }

    /// Set contrast
    pub fn set_contrast(&mut self, value: f32) {
        self.controls.contrast = value.clamp(0.0, 2.0);
    }

    /// Set exposure
    pub fn set_exposure(&mut self, value: f32) {
        self.controls.exposure = value.clamp(-4.0, 4.0);
        self.controls.auto_exposure = false;
    }

    /// Enable auto exposure
    pub fn set_auto_exposure(&mut self, enabled: bool) {
        self.controls.auto_exposure = enabled;
    }

    /// Set focus
    pub fn set_focus(&mut self, value: f32) {
        self.controls.focus = value.clamp(0.0, 1.0);
        self.controls.auto_focus = false;
    }

    /// Enable auto focus
    pub fn set_auto_focus(&mut self, enabled: bool) {
        self.controls.auto_focus = enabled;
    }

    /// Trigger autofocus (one-shot)
    pub fn trigger_autofocus(&mut self) -> Result<(), VideoError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(VideoError::NotRunning);
        }
        // In real implementation, would trigger device autofocus
        Ok(())
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, value: f32) {
        if let Some(ref info) = self.camera_info {
            self.controls.zoom = value.clamp(info.min_zoom, info.max_zoom);
        }
    }

    /// Set flash mode
    pub fn set_flash(&mut self, mode: FlashMode) {
        self.controls.flash = mode;
    }

    /// Get camera info
    pub fn camera_info(&self) -> Option<&CameraInfo> {
        self.camera_info.as_ref()
    }
}

/// List available cameras
pub fn list_cameras() -> Vec<CameraInfo> {
    // In real implementation, would enumerate devices
    vec![
        CameraInfo {
            name: "Front Camera".to_string(),
            id: "front_cam".to_string(),
            device_path: "/dev/video0".to_string(),
            front_facing: true,
            resolutions: vec![(1920, 1080), (1280, 720), (640, 480)],
            framerates: vec![30, 60],
            formats: vec![PixelFormat::NV12, PixelFormat::YUYV],
            has_autofocus: true,
            has_flash: false,
            has_zoom: true,
            min_zoom: 1.0,
            max_zoom: 4.0,
        },
        CameraInfo {
            name: "Rear Camera".to_string(),
            id: "rear_cam".to_string(),
            device_path: "/dev/video1".to_string(),
            front_facing: false,
            resolutions: vec![(4032, 3024), (1920, 1080), (1280, 720)],
            framerates: vec![30, 60, 120, 240],
            formats: vec![PixelFormat::NV12, PixelFormat::YUYV],
            has_autofocus: true,
            has_flash: true,
            has_zoom: true,
            min_zoom: 1.0,
            max_zoom: 10.0,
        },
    ]
}

/// Get default camera
pub fn default_camera() -> Option<CameraInfo> {
    list_cameras().into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.framerate, 30);
    }

    #[test]
    fn test_capture_start_stop() {
        let config = CaptureConfig::default();
        let mut capture = VideoCapture::new(config);

        capture.start().unwrap();
        assert!(capture.is_running());

        capture.stop().unwrap();
        assert!(!capture.is_running());
    }

    #[test]
    fn test_capture_test_pattern() {
        let config = CaptureConfig {
            source: CaptureSource::TestPattern,
            width: 640,
            height: 480,
            ..Default::default()
        };
        let mut capture = VideoCapture::new(config);

        capture.start().unwrap();
        let frame = capture.capture().unwrap();

        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert!(frame.sequence > 0);
    }

    #[test]
    fn test_camera_controls_default() {
        let controls = CameraControls::default();
        assert_eq!(controls.brightness, 0.0);
        assert_eq!(controls.contrast, 1.0);
        assert!(controls.auto_exposure);
        assert!(controls.auto_focus);
    }

    #[test]
    fn test_list_cameras() {
        let cameras = list_cameras();
        assert!(!cameras.is_empty());
        assert!(cameras.iter().any(|c| c.front_facing));
        assert!(cameras.iter().any(|c| !c.front_facing));
    }

    #[test]
    fn test_camera_zoom() {
        let config = CaptureConfig::default();
        let mut capture = VideoCapture::new(config);
        capture.start().unwrap();

        capture.set_zoom(2.0);
        assert_eq!(capture.controls().zoom, 2.0);

        // Test clamping
        capture.set_zoom(100.0);
        assert!(capture.controls().zoom <= 10.0);
    }

    #[test]
    fn test_exposure_settings() {
        let config = CaptureConfig::default();
        let mut capture = VideoCapture::new(config);

        capture.set_auto_exposure(true);
        assert!(capture.controls().auto_exposure);

        capture.set_exposure(-2.0);
        assert!(!capture.controls().auto_exposure);
        assert_eq!(capture.controls().exposure, -2.0);
    }

    #[test]
    fn test_focus_settings() {
        let config = CaptureConfig::default();
        let mut capture = VideoCapture::new(config);

        capture.set_auto_focus(true);
        assert!(capture.controls().auto_focus);

        capture.set_focus(0.5);
        assert!(!capture.controls().auto_focus);
        assert_eq!(capture.controls().focus, 0.5);
    }
}
