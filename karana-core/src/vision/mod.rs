// Advanced Vision Processing System for Kāraṇa OS
// Handles advanced image processing, object detection, OCR, and QR scanning

pub mod capture;
pub mod processing;
pub mod detection;
pub mod ocr;
pub mod qr;

pub use capture::*;
pub use processing::*;
pub use detection::*;
pub use ocr::*;
pub use qr::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Camera identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraId {
    /// Main forward-facing camera
    Main,
    /// Wide-angle camera for peripheral vision
    Wide,
    /// Depth sensor camera
    Depth,
    /// Eye-tracking camera (internal)
    EyeTracking,
    /// Custom camera by index
    Custom(u8),
}

/// Camera resolution preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// 640x480
    VGA,
    /// 1280x720
    HD,
    /// 1920x1080
    FullHD,
    /// 3840x2160
    UHD4K,
    /// Custom resolution
    Custom(u32, u32),
}

impl Resolution {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Resolution::VGA => (640, 480),
            Resolution::HD => (1280, 720),
            Resolution::FullHD => (1920, 1080),
            Resolution::UHD4K => (3840, 2160),
            Resolution::Custom(w, h) => (*w, *h),
        }
    }
    
    pub fn pixel_count(&self) -> u32 {
        let (w, h) = self.dimensions();
        w * h
    }
}

/// Camera frame format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    /// RGB 24-bit
    RGB,
    /// RGBA 32-bit
    RGBA,
    /// YUV 4:2:0
    YUV420,
    /// Grayscale 8-bit
    Grayscale,
    /// Depth map 16-bit
    Depth16,
    /// Raw Bayer pattern
    RawBayer,
}

impl FrameFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            FrameFormat::RGB => 3,
            FrameFormat::RGBA => 4,
            FrameFormat::YUV420 => 2, // Average
            FrameFormat::Grayscale => 1,
            FrameFormat::Depth16 => 2,
            FrameFormat::RawBayer => 2,
        }
    }
}

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub camera_id: CameraId,
    pub resolution: Resolution,
    pub format: FrameFormat,
    pub fps: u32,
    pub auto_exposure: bool,
    pub auto_focus: bool,
    pub auto_white_balance: bool,
    pub exposure_ms: Option<f32>,
    pub iso: Option<u32>,
    pub focus_distance: Option<f32>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            camera_id: CameraId::Main,
            resolution: Resolution::HD,
            format: FrameFormat::RGB,
            fps: 30,
            auto_exposure: true,
            auto_focus: true,
            auto_white_balance: true,
            exposure_ms: None,
            iso: None,
            focus_distance: None,
        }
    }
}

/// A captured camera frame
#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub camera_id: CameraId,
    pub width: u32,
    pub height: u32,
    pub format: FrameFormat,
    pub data: Vec<u8>,
    pub timestamp: Instant,
    pub frame_number: u64,
    pub exposure_ms: f32,
    pub iso: u32,
}

impl CameraFrame {
    pub fn new(
        camera_id: CameraId,
        width: u32,
        height: u32,
        format: FrameFormat,
        data: Vec<u8>,
    ) -> Self {
        Self {
            camera_id,
            width,
            height,
            format,
            data,
            timestamp: Instant::now(),
            frame_number: 0,
            exposure_ms: 16.0,
            iso: 100,
        }
    }
    
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }
    
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    
    /// Get pixel value at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let bpp = self.format.bytes_per_pixel();
        let offset = ((y * self.width + x) as usize) * bpp;
        
        if offset + bpp <= self.data.len() {
            Some(&self.data[offset..offset + bpp])
        } else {
            None
        }
    }
    
    /// Convert frame to grayscale
    pub fn to_grayscale(&self) -> Option<CameraFrame> {
        match self.format {
            FrameFormat::RGB => {
                let mut gray_data = Vec::with_capacity((self.width * self.height) as usize);
                for chunk in self.data.chunks(3) {
                    if chunk.len() == 3 {
                        // Luminance formula: 0.299R + 0.587G + 0.114B
                        let gray = (0.299 * chunk[0] as f32 +
                                   0.587 * chunk[1] as f32 +
                                   0.114 * chunk[2] as f32) as u8;
                        gray_data.push(gray);
                    }
                }
                Some(CameraFrame {
                    camera_id: self.camera_id,
                    width: self.width,
                    height: self.height,
                    format: FrameFormat::Grayscale,
                    data: gray_data,
                    timestamp: self.timestamp,
                    frame_number: self.frame_number,
                    exposure_ms: self.exposure_ms,
                    iso: self.iso,
                })
            }
            FrameFormat::RGBA => {
                let mut gray_data = Vec::with_capacity((self.width * self.height) as usize);
                for chunk in self.data.chunks(4) {
                    if chunk.len() == 4 {
                        let gray = (0.299 * chunk[0] as f32 +
                                   0.587 * chunk[1] as f32 +
                                   0.114 * chunk[2] as f32) as u8;
                        gray_data.push(gray);
                    }
                }
                Some(CameraFrame {
                    camera_id: self.camera_id,
                    width: self.width,
                    height: self.height,
                    format: FrameFormat::Grayscale,
                    data: gray_data,
                    timestamp: self.timestamp,
                    frame_number: self.frame_number,
                    exposure_ms: self.exposure_ms,
                    iso: self.iso,
                })
            }
            FrameFormat::Grayscale => Some(self.clone()),
            _ => None,
        }
    }
}

/// Vision processing pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Raw frame capture
    Capture,
    /// Preprocessing (denoise, stabilize)
    Preprocess,
    /// Feature detection
    FeatureDetection,
    /// Object detection
    ObjectDetection,
    /// OCR processing
    OCR,
    /// Scene understanding
    SceneAnalysis,
    /// Output/display
    Output,
}

/// Vision pipeline statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub frames_processed: u64,
    pub average_latency_ms: f32,
    pub detections_per_frame: f32,
    pub stage_timings: HashMap<String, f32>,
}

/// Main camera system manager
pub struct CameraSystem {
    cameras: HashMap<CameraId, CameraConfig>,
    active_cameras: Vec<CameraId>,
    pipeline_stats: PipelineStats,
    frame_callbacks: Vec<Box<dyn Fn(&CameraFrame) + Send + Sync>>,
    privacy_mode: bool,
    recording: bool,
}

impl CameraSystem {
    pub fn new() -> Self {
        Self {
            cameras: HashMap::new(),
            active_cameras: Vec::new(),
            pipeline_stats: PipelineStats::default(),
            frame_callbacks: Vec::new(),
            privacy_mode: false,
            recording: false,
        }
    }
    
    /// Register a camera with configuration
    pub fn register_camera(&mut self, config: CameraConfig) {
        self.cameras.insert(config.camera_id, config);
    }
    
    /// Start capturing from a camera
    pub fn start_capture(&mut self, camera_id: CameraId) -> Result<(), CameraError> {
        if self.privacy_mode {
            return Err(CameraError::PrivacyModeActive);
        }
        
        if !self.cameras.contains_key(&camera_id) {
            return Err(CameraError::CameraNotFound(camera_id));
        }
        
        if !self.active_cameras.contains(&camera_id) {
            self.active_cameras.push(camera_id);
        }
        
        Ok(())
    }
    
    /// Stop capturing from a camera
    pub fn stop_capture(&mut self, camera_id: CameraId) {
        self.active_cameras.retain(|&id| id != camera_id);
    }
    
    /// Check if camera is active
    pub fn is_active(&self, camera_id: CameraId) -> bool {
        self.active_cameras.contains(&camera_id)
    }
    
    /// Enable privacy mode (disables all cameras)
    pub fn set_privacy_mode(&mut self, enabled: bool) {
        self.privacy_mode = enabled;
        if enabled {
            self.active_cameras.clear();
            self.recording = false;
        }
    }
    
    /// Check if privacy mode is active
    pub fn is_privacy_mode(&self) -> bool {
        self.privacy_mode
    }
    
    /// Start recording
    pub fn start_recording(&mut self) -> Result<(), CameraError> {
        if self.privacy_mode {
            return Err(CameraError::PrivacyModeActive);
        }
        if self.active_cameras.is_empty() {
            return Err(CameraError::NoCameraActive);
        }
        self.recording = true;
        Ok(())
    }
    
    /// Stop recording
    pub fn stop_recording(&mut self) {
        self.recording = false;
    }
    
    /// Check if recording
    pub fn is_recording(&self) -> bool {
        self.recording
    }
    
    /// Get pipeline statistics
    pub fn get_stats(&self) -> &PipelineStats {
        &self.pipeline_stats
    }
    
    /// Update pipeline statistics
    pub fn update_stats(&mut self, latency_ms: f32, detections: u32) {
        self.pipeline_stats.frames_processed += 1;
        let n = self.pipeline_stats.frames_processed as f32;
        
        // Running average
        self.pipeline_stats.average_latency_ms = 
            self.pipeline_stats.average_latency_ms * (n - 1.0) / n + latency_ms / n;
        self.pipeline_stats.detections_per_frame =
            self.pipeline_stats.detections_per_frame * (n - 1.0) / n + detections as f32 / n;
    }
    
    /// Get active camera count
    pub fn active_camera_count(&self) -> usize {
        self.active_cameras.len()
    }
    
    /// Get registered cameras
    pub fn registered_cameras(&self) -> Vec<CameraId> {
        self.cameras.keys().copied().collect()
    }
}

impl Default for CameraSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera system errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraError {
    CameraNotFound(CameraId),
    PrivacyModeActive,
    NoCameraActive,
    InvalidResolution,
    CaptureError(String),
    ProcessingError(String),
}

impl std::fmt::Display for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraError::CameraNotFound(id) => write!(f, "Camera not found: {:?}", id),
            CameraError::PrivacyModeActive => write!(f, "Privacy mode is active"),
            CameraError::NoCameraActive => write!(f, "No camera is active"),
            CameraError::InvalidResolution => write!(f, "Invalid resolution"),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            CameraError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for CameraError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolution_dimensions() {
        assert_eq!(Resolution::VGA.dimensions(), (640, 480));
        assert_eq!(Resolution::HD.dimensions(), (1280, 720));
        assert_eq!(Resolution::FullHD.dimensions(), (1920, 1080));
        assert_eq!(Resolution::UHD4K.dimensions(), (3840, 2160));
        assert_eq!(Resolution::Custom(800, 600).dimensions(), (800, 600));
    }
    
    #[test]
    fn test_resolution_pixel_count() {
        assert_eq!(Resolution::VGA.pixel_count(), 307200);
        assert_eq!(Resolution::HD.pixel_count(), 921600);
    }
    
    #[test]
    fn test_frame_format_bpp() {
        assert_eq!(FrameFormat::RGB.bytes_per_pixel(), 3);
        assert_eq!(FrameFormat::RGBA.bytes_per_pixel(), 4);
        assert_eq!(FrameFormat::Grayscale.bytes_per_pixel(), 1);
    }
    
    #[test]
    fn test_camera_frame_creation() {
        let data = vec![255, 128, 64, 32, 16, 8]; // 2 RGB pixels
        let frame = CameraFrame::new(
            CameraId::Main,
            2,
            1,
            FrameFormat::RGB,
            data,
        );
        
        assert_eq!(frame.pixel_count(), 2);
        assert_eq!(frame.aspect_ratio(), 2.0);
    }
    
    #[test]
    fn test_get_pixel() {
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 RGB pixels
        let frame = CameraFrame::new(
            CameraId::Main,
            3,
            1,
            FrameFormat::RGB,
            data,
        );
        
        assert_eq!(frame.get_pixel(0, 0), Some(&[255u8, 0, 0][..]));
        assert_eq!(frame.get_pixel(1, 0), Some(&[0u8, 255, 0][..]));
        assert_eq!(frame.get_pixel(2, 0), Some(&[0u8, 0, 255][..]));
        assert_eq!(frame.get_pixel(3, 0), None); // Out of bounds
    }
    
    #[test]
    fn test_to_grayscale_rgb() {
        // Create a 2x1 RGB image: red and green pixels
        let data = vec![255, 0, 0, 0, 255, 0];
        let frame = CameraFrame::new(
            CameraId::Main,
            2,
            1,
            FrameFormat::RGB,
            data,
        );
        
        let gray = frame.to_grayscale().unwrap();
        assert_eq!(gray.format, FrameFormat::Grayscale);
        assert_eq!(gray.data.len(), 2);
        
        // Red: 0.299 * 255 ≈ 76
        assert!((gray.data[0] as i32 - 76).abs() <= 1);
        // Green: 0.587 * 255 ≈ 150
        assert!((gray.data[1] as i32 - 150).abs() <= 1);
    }
    
    #[test]
    fn test_camera_system_registration() {
        let mut system = CameraSystem::new();
        
        let config = CameraConfig {
            camera_id: CameraId::Main,
            ..Default::default()
        };
        
        system.register_camera(config);
        
        assert!(system.registered_cameras().contains(&CameraId::Main));
    }
    
    #[test]
    fn test_camera_capture_lifecycle() {
        let mut system = CameraSystem::new();
        
        let config = CameraConfig {
            camera_id: CameraId::Main,
            ..Default::default()
        };
        system.register_camera(config);
        
        assert!(!system.is_active(CameraId::Main));
        
        system.start_capture(CameraId::Main).unwrap();
        assert!(system.is_active(CameraId::Main));
        assert_eq!(system.active_camera_count(), 1);
        
        system.stop_capture(CameraId::Main);
        assert!(!system.is_active(CameraId::Main));
    }
    
    #[test]
    fn test_privacy_mode() {
        let mut system = CameraSystem::new();
        
        let config = CameraConfig::default();
        system.register_camera(config);
        system.start_capture(CameraId::Main).unwrap();
        
        system.set_privacy_mode(true);
        assert!(system.is_privacy_mode());
        assert_eq!(system.active_camera_count(), 0);
        
        // Cannot start capture in privacy mode
        let result = system.start_capture(CameraId::Main);
        assert_eq!(result, Err(CameraError::PrivacyModeActive));
    }
    
    #[test]
    fn test_recording() {
        let mut system = CameraSystem::new();
        
        let config = CameraConfig::default();
        system.register_camera(config);
        
        // Cannot record without active camera
        let result = system.start_recording();
        assert_eq!(result, Err(CameraError::NoCameraActive));
        
        system.start_capture(CameraId::Main).unwrap();
        system.start_recording().unwrap();
        assert!(system.is_recording());
        
        system.stop_recording();
        assert!(!system.is_recording());
    }
    
    #[test]
    fn test_pipeline_stats() {
        let mut system = CameraSystem::new();
        
        system.update_stats(10.0, 5);
        assert_eq!(system.get_stats().frames_processed, 1);
        assert_eq!(system.get_stats().average_latency_ms, 10.0);
        assert_eq!(system.get_stats().detections_per_frame, 5.0);
        
        system.update_stats(20.0, 10);
        assert_eq!(system.get_stats().frames_processed, 2);
        assert_eq!(system.get_stats().average_latency_ms, 15.0);
    }
    
    #[test]
    fn test_camera_not_found() {
        let mut system = CameraSystem::new();
        
        let result = system.start_capture(CameraId::Depth);
        assert!(matches!(result, Err(CameraError::CameraNotFound(_))));
    }
}
