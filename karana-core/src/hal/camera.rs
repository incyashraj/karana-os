// Kāraṇa OS - Camera HAL
// Hardware abstraction for smart glasses cameras

use super::HalError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// Resolution width
    pub width: u32,
    /// Resolution height
    pub height: u32,
    /// Frame rate
    pub framerate: u32,
    /// Pixel format
    pub format: CameraFormat,
    /// Auto exposure
    pub auto_exposure: bool,
    /// Auto focus
    pub auto_focus: bool,
    /// Auto white balance
    pub auto_white_balance: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            framerate: 30,
            format: CameraFormat::Nv12,
            auto_exposure: true,
            auto_focus: true,
            auto_white_balance: true,
        }
    }
}

/// Camera pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraFormat {
    /// RGB 24-bit
    Rgb24,
    /// RGBA 32-bit
    Rgba32,
    /// YUV 4:2:0
    Yuv420,
    /// NV12 (Y plane + interleaved UV)
    Nv12,
    /// MJPEG
    Mjpeg,
    /// Raw Bayer
    Raw,
}

/// Camera position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPosition {
    /// World-facing camera
    World,
    /// Eye-tracking camera (per eye)
    EyeLeft,
    EyeRight,
    /// Front-facing user camera
    Front,
    /// Depth camera
    Depth,
}

/// Camera capabilities
#[derive(Debug, Clone)]
pub struct CameraCapabilities {
    /// Camera position
    pub position: CameraPosition,
    /// Maximum resolution
    pub max_resolution: (u32, u32),
    /// Supported resolutions
    pub resolutions: Vec<(u32, u32)>,
    /// Maximum framerate
    pub max_framerate: u32,
    /// Supported formats
    pub formats: Vec<CameraFormat>,
    /// Has flash
    pub has_flash: bool,
    /// Has optical zoom
    pub has_zoom: bool,
    /// Zoom range (if available)
    pub zoom_range: Option<(f32, f32)>,
    /// Has optical image stabilization
    pub has_ois: bool,
    /// Field of view (degrees)
    pub fov: f32,
    /// Sensor size (mm)
    pub sensor_size: (f32, f32),
}

impl Default for CameraCapabilities {
    fn default() -> Self {
        Self {
            position: CameraPosition::World,
            max_resolution: (4032, 3024),
            resolutions: vec![
                (4032, 3024),
                (1920, 1080),
                (1280, 720),
                (640, 480),
            ],
            max_framerate: 120,
            formats: vec![
                CameraFormat::Nv12,
                CameraFormat::Yuv420,
                CameraFormat::Mjpeg,
            ],
            has_flash: false,
            has_zoom: true,
            zoom_range: Some((1.0, 8.0)),
            has_ois: true,
            fov: 120.0,
            sensor_size: (6.4, 4.8),
        }
    }
}

/// Camera state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraState {
    /// Closed
    Closed,
    /// Opening
    Opening,
    /// Ready
    Ready,
    /// Streaming
    Streaming,
    /// Error
    Error,
}

/// Camera controls
#[derive(Debug, Clone)]
pub struct CameraControls {
    /// Exposure time (microseconds)
    pub exposure_time_us: u32,
    /// ISO sensitivity
    pub iso: u32,
    /// White balance temperature (Kelvin)
    pub white_balance_k: u32,
    /// Focus distance (0.0 = infinity, 1.0 = minimum)
    pub focus_distance: f32,
    /// Zoom level
    pub zoom: f32,
    /// Flash mode
    pub flash_mode: FlashMode,
    /// Stabilization enabled
    pub stabilization: bool,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            exposure_time_us: 16667, // ~60fps
            iso: 100,
            white_balance_k: 5500,
            focus_distance: 0.0, // Infinity
            zoom: 1.0,
            flash_mode: FlashMode::Off,
            stabilization: true,
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
}

/// Camera frame
#[derive(Debug, Clone)]
pub struct CameraFrame {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Pixel format
    pub format: CameraFormat,
    /// Frame data
    pub data: Vec<u8>,
    /// Timestamp (monotonic nanoseconds)
    pub timestamp_ns: u64,
    /// Frame sequence number
    pub sequence: u64,
    /// Exposure time used (us)
    pub exposure_us: u32,
    /// ISO used
    pub iso: u32,
    /// Focus state
    pub focus_state: FocusState,
}

/// Focus state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// Not focused
    Unfocused,
    /// Focusing in progress
    Focusing,
    /// Focused
    Focused,
    /// Focus locked
    Locked,
}

/// Camera HAL
#[derive(Debug)]
pub struct Camera {
    /// Configuration
    config: CameraConfig,
    /// Capabilities
    capabilities: CameraCapabilities,
    /// Current state
    state: CameraState,
    /// Camera controls
    controls: CameraControls,
    /// Frame counter
    frame_counter: AtomicU64,
    /// Statistics
    stats: CameraStats,
    /// Is initialized
    initialized: bool,
    /// Stream start time
    stream_start: Option<Instant>,
}

/// Camera statistics
#[derive(Debug, Default, Clone)]
pub struct CameraStats {
    /// Frames captured
    pub frames_captured: u64,
    /// Frames dropped
    pub frames_dropped: u64,
    /// Average frame time (ms)
    pub avg_frame_time_ms: f64,
    /// Current framerate
    pub current_fps: f32,
}

impl Camera {
    /// Create new camera HAL
    pub fn new(config: CameraConfig) -> Result<Self, HalError> {
        Ok(Self {
            config,
            capabilities: CameraCapabilities::default(),
            state: CameraState::Closed,
            controls: CameraControls::default(),
            frame_counter: AtomicU64::new(0),
            stats: CameraStats::default(),
            initialized: false,
            stream_start: None,
        })
    }

    /// Initialize camera
    pub fn initialize(&mut self) -> Result<(), HalError> {
        self.detect_capabilities()?;
        self.validate_config()?;
        self.initialized = true;
        Ok(())
    }

    /// Open camera
    pub fn open(&mut self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Camera not initialized".into()));
        }

        self.state = CameraState::Opening;
        
        // Simulate camera open
        self.state = CameraState::Ready;
        Ok(())
    }

    /// Close camera
    pub fn close(&mut self) -> Result<(), HalError> {
        self.state = CameraState::Closed;
        self.stream_start = None;
        Ok(())
    }

    /// Start streaming
    pub fn start_streaming(&mut self) -> Result<(), HalError> {
        if self.state != CameraState::Ready {
            return Err(HalError::ConfigError("Camera not ready".into()));
        }

        self.state = CameraState::Streaming;
        self.stream_start = Some(Instant::now());
        Ok(())
    }

    /// Stop streaming
    pub fn stop_streaming(&mut self) -> Result<(), HalError> {
        self.state = CameraState::Ready;
        self.stream_start = None;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> CameraState {
        self.state
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &CameraCapabilities {
        &self.capabilities
    }

    /// Get statistics
    pub fn stats(&self) -> CameraStats {
        let mut stats = CameraStats {
            frames_captured: self.frame_counter.load(Ordering::Relaxed),
            ..self.stats.clone()
        };

        // Calculate current FPS
        if let Some(start) = self.stream_start {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed > 0.0 {
                stats.current_fps = stats.frames_captured as f32 / elapsed;
            }
        }

        stats
    }

    /// Capture frame
    pub fn capture(&mut self) -> Result<CameraFrame, HalError> {
        if self.state != CameraState::Streaming {
            return Err(HalError::ConfigError("Camera not streaming".into()));
        }

        let seq = self.frame_counter.fetch_add(1, Ordering::Relaxed);

        // Create simulated frame
        let frame = CameraFrame {
            width: self.config.width,
            height: self.config.height,
            format: self.config.format,
            data: vec![0; (self.config.width * self.config.height * 3 / 2) as usize],
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            sequence: seq,
            exposure_us: self.controls.exposure_time_us,
            iso: self.controls.iso,
            focus_state: FocusState::Focused,
        };

        Ok(frame)
    }

    /// Set camera controls
    pub fn set_controls(&mut self, controls: CameraControls) -> Result<(), HalError> {
        // Validate zoom
        if let Some((min, max)) = self.capabilities.zoom_range {
            if controls.zoom < min || controls.zoom > max {
                return Err(HalError::ConfigError(format!(
                    "Zoom must be between {} and {}",
                    min, max
                )));
            }
        }

        self.controls = controls;
        Ok(())
    }

    /// Get current controls
    pub fn controls(&self) -> &CameraControls {
        &self.controls
    }

    /// Set resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), HalError> {
        if self.state == CameraState::Streaming {
            return Err(HalError::DeviceBusy);
        }

        // Check if resolution is supported
        if !self.capabilities.resolutions.contains(&(width, height)) {
            return Err(HalError::ConfigError("Resolution not supported".into()));
        }

        self.config.width = width;
        self.config.height = height;
        Ok(())
    }

    /// Get resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Set framerate
    pub fn set_framerate(&mut self, fps: u32) -> Result<(), HalError> {
        if fps > self.capabilities.max_framerate {
            return Err(HalError::ConfigError("Framerate exceeds maximum".into()));
        }

        self.config.framerate = fps;
        Ok(())
    }

    /// Get framerate
    pub fn framerate(&self) -> u32 {
        self.config.framerate
    }

    /// Set format
    pub fn set_format(&mut self, format: CameraFormat) -> Result<(), HalError> {
        if !self.capabilities.formats.contains(&format) {
            return Err(HalError::ConfigError("Format not supported".into()));
        }

        self.config.format = format;
        Ok(())
    }

    /// Auto focus
    pub fn autofocus(&mut self) -> Result<(), HalError> {
        if !self.config.auto_focus {
            return Err(HalError::NotSupported);
        }
        Ok(())
    }

    /// Set zoom
    pub fn set_zoom(&mut self, zoom: f32) -> Result<(), HalError> {
        if !self.capabilities.has_zoom {
            return Err(HalError::NotSupported);
        }

        if let Some((min, max)) = self.capabilities.zoom_range {
            if zoom < min || zoom > max {
                return Err(HalError::ConfigError(format!(
                    "Zoom {} out of range [{}, {}]",
                    zoom, min, max
                )));
            }
        }

        self.controls.zoom = zoom;
        Ok(())
    }

    /// Get zoom
    pub fn zoom(&self) -> f32 {
        self.controls.zoom
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
        // Would query actual hardware
        self.capabilities = CameraCapabilities::default();
        Ok(())
    }

    /// Validate configuration
    fn validate_config(&self) -> Result<(), HalError> {
        let (max_w, max_h) = self.capabilities.max_resolution;
        if self.config.width > max_w || self.config.height > max_h {
            return Err(HalError::ConfigError("Resolution exceeds maximum".into()));
        }
        if self.config.framerate > self.capabilities.max_framerate {
            return Err(HalError::ConfigError("Framerate exceeds maximum".into()));
        }
        Ok(())
    }
}

/// Multi-camera manager
#[derive(Debug)]
pub struct CameraManager {
    /// World-facing camera
    world_camera: Option<Camera>,
    /// Eye-tracking cameras
    eye_cameras: Option<(Camera, Camera)>,
    /// Depth camera
    depth_camera: Option<Camera>,
}

impl CameraManager {
    /// Create camera manager
    pub fn new() -> Self {
        Self {
            world_camera: None,
            eye_cameras: None,
            depth_camera: None,
        }
    }

    /// Initialize world camera
    pub fn init_world_camera(&mut self, config: CameraConfig) -> Result<(), HalError> {
        let mut camera = Camera::new(config)?;
        camera.capabilities.position = CameraPosition::World;
        camera.initialize()?;
        self.world_camera = Some(camera);
        Ok(())
    }

    /// Initialize eye tracking cameras
    pub fn init_eye_cameras(&mut self, config: CameraConfig) -> Result<(), HalError> {
        let mut left = Camera::new(config.clone())?;
        left.capabilities.position = CameraPosition::EyeLeft;
        left.capabilities.max_resolution = (640, 480);
        left.capabilities.fov = 60.0;
        left.initialize()?;

        let mut right = Camera::new(config)?;
        right.capabilities.position = CameraPosition::EyeRight;
        right.capabilities.max_resolution = (640, 480);
        right.capabilities.fov = 60.0;
        right.initialize()?;

        self.eye_cameras = Some((left, right));
        Ok(())
    }

    /// Get world camera
    pub fn world_camera(&mut self) -> Option<&mut Camera> {
        self.world_camera.as_mut()
    }

    /// Get eye cameras
    pub fn eye_cameras(&mut self) -> Option<(&mut Camera, &mut Camera)> {
        self.eye_cameras.as_mut().map(|(l, r)| (l, r))
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_config_default() {
        let config = CameraConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.auto_exposure);
    }

    #[test]
    fn test_camera_capabilities() {
        let caps = CameraCapabilities::default();
        assert!(caps.has_ois);
        assert_eq!(caps.fov, 120.0);
    }

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(CameraConfig::default());
        assert!(camera.is_ok());
    }

    #[test]
    fn test_camera_open_close() {
        let mut camera = Camera::new(CameraConfig::default()).unwrap();
        camera.initialize().unwrap();

        camera.open().unwrap();
        assert_eq!(camera.state(), CameraState::Ready);

        camera.close().unwrap();
        assert_eq!(camera.state(), CameraState::Closed);
    }

    #[test]
    fn test_camera_streaming() {
        let mut camera = Camera::new(CameraConfig::default()).unwrap();
        camera.initialize().unwrap();
        camera.open().unwrap();

        camera.start_streaming().unwrap();
        assert_eq!(camera.state(), CameraState::Streaming);

        let frame = camera.capture().unwrap();
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);

        camera.stop_streaming().unwrap();
        assert_eq!(camera.state(), CameraState::Ready);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new(CameraConfig::default()).unwrap();
        camera.initialize().unwrap();

        camera.set_zoom(2.0).unwrap();
        assert_eq!(camera.zoom(), 2.0);

        // Out of range
        assert!(camera.set_zoom(10.0).is_err());
    }

    #[test]
    fn test_camera_resolution() {
        let mut camera = Camera::new(CameraConfig::default()).unwrap();
        camera.initialize().unwrap();

        camera.set_resolution(1920, 1080).unwrap();
        assert_eq!(camera.resolution(), (1920, 1080));

        // Unsupported resolution
        assert!(camera.set_resolution(1234, 567).is_err());
    }

    #[test]
    fn test_camera_manager() {
        let mut manager = CameraManager::new();
        manager.init_world_camera(CameraConfig::default()).unwrap();

        assert!(manager.world_camera().is_some());
    }
}
