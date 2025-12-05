// Kāraṇa OS - Video Processing Pipeline
// Real-time video capture, processing, and analysis for smart glasses

pub mod capture;
pub mod frame;
pub mod processing;
pub mod encoder;
pub mod streaming;

pub use capture::{VideoCapture, CaptureConfig, CaptureSource, CameraInfo};
pub use frame::{VideoFrame, PixelFormat, FrameBuffer, ColorSpace};
pub use processing::{VideoProcessor, ProcessingPipeline, VideoFilter};
pub use encoder::{VideoEncoder, EncoderConfig, VideoCodec, EncoderProfile};
pub use streaming::{VideoStreamer, StreamConfig, StreamProtocol};

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Video pipeline configuration
#[derive(Debug, Clone)]
pub struct VideoPipelineConfig {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Target frame rate (fps)
    pub framerate: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Enable image stabilization
    pub stabilization: bool,
    /// Enable auto exposure
    pub auto_exposure: bool,
    /// Enable auto focus
    pub auto_focus: bool,
    /// Enable HDR
    pub hdr: bool,
    /// Target latency (ms)
    pub target_latency_ms: u32,
    /// Recording enabled
    pub recording: bool,
}

impl Default for VideoPipelineConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            framerate: 30,
            format: PixelFormat::NV12,
            stabilization: true,
            auto_exposure: true,
            auto_focus: true,
            hdr: false,
            target_latency_ms: 33, // ~1 frame at 30fps
            recording: false,
        }
    }
}

/// Main video pipeline manager
#[derive(Debug)]
pub struct VideoPipeline {
    /// Configuration
    config: VideoPipelineConfig,
    /// Video capture
    capture: VideoCapture,
    /// Processing pipeline
    processor: VideoProcessor,
    /// Video encoder (optional)
    encoder: Option<VideoEncoder>,
    /// Video streamer (optional)
    streamer: Option<VideoStreamer>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Processed frame count
    frame_count: AtomicU64,
    /// Pipeline metrics
    metrics: PipelineMetrics,
    /// Pipeline state
    state: PipelineState,
}

/// Pipeline state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle,
    Starting,
    Running,
    Paused,
    Stopping,
    Error,
}

/// Pipeline metrics
#[derive(Debug, Default)]
pub struct PipelineMetrics {
    /// Total frames processed
    pub frames_processed: u64,
    /// Dropped frames
    pub frames_dropped: u64,
    /// Average processing latency (us)
    pub avg_latency_us: f64,
    /// Peak latency (us)
    pub peak_latency_us: u64,
    /// Average frame rate
    pub avg_fps: f32,
    /// Current bitrate (if encoding)
    pub bitrate_bps: u64,
    /// Encoder queue depth
    pub encoder_queue_depth: usize,
}

impl VideoPipeline {
    /// Create new video pipeline
    pub fn new(config: VideoPipelineConfig) -> Self {
        let capture_config = CaptureConfig {
            width: config.width,
            height: config.height,
            framerate: config.framerate,
            format: config.format,
            source: CaptureSource::Default,
        };

        Self {
            capture: VideoCapture::new(capture_config),
            processor: VideoProcessor::new(config.width, config.height),
            encoder: None,
            streamer: None,
            running: Arc::new(AtomicBool::new(false)),
            frame_count: AtomicU64::new(0),
            metrics: PipelineMetrics::default(),
            state: PipelineState::Idle,
            config,
        }
    }

    /// Start the video pipeline
    pub fn start(&mut self) -> Result<(), VideoError> {
        if self.state == PipelineState::Running {
            return Ok(());
        }

        self.state = PipelineState::Starting;
        self.running.store(true, Ordering::SeqCst);

        // Initialize capture
        self.capture.start()?;

        // Setup processing pipeline
        self.setup_processing_pipeline()?;

        // Initialize encoder if recording
        if self.config.recording {
            self.setup_encoder()?;
        }

        self.state = PipelineState::Running;
        Ok(())
    }

    /// Stop the video pipeline
    pub fn stop(&mut self) -> Result<(), VideoError> {
        if self.state != PipelineState::Running {
            return Ok(());
        }

        self.state = PipelineState::Stopping;
        self.running.store(false, Ordering::SeqCst);

        // Stop encoder first
        if let Some(ref mut encoder) = self.encoder {
            encoder.stop()?;
        }

        // Stop capture
        self.capture.stop()?;
        
        self.state = PipelineState::Idle;
        Ok(())
    }

    /// Pause processing
    pub fn pause(&mut self) {
        if self.state == PipelineState::Running {
            self.state = PipelineState::Paused;
        }
    }

    /// Resume processing
    pub fn resume(&mut self) {
        if self.state == PipelineState::Paused {
            self.state = PipelineState::Running;
        }
    }

    /// Process one frame of video
    pub fn process_frame(&mut self, input: &VideoFrame) -> Result<VideoFrame, VideoError> {
        if self.state != PipelineState::Running {
            return Err(VideoError::NotRunning);
        }

        let start = Instant::now();

        // Run through processing pipeline
        let processed = self.processor.process(input)?;

        // Encode if recording
        if let Some(ref mut encoder) = self.encoder {
            encoder.encode(&processed)?;
        }

        // Stream if enabled
        if let Some(ref mut streamer) = self.streamer {
            streamer.send_frame(&processed)?;
        }

        // Update metrics
        let latency_us = start.elapsed().as_micros() as u64;
        self.update_metrics(latency_us);

        self.frame_count.fetch_add(1, Ordering::Relaxed);
        Ok(processed)
    }

    /// Capture and process next frame
    pub fn capture_and_process(&mut self) -> Result<VideoFrame, VideoError> {
        let frame = self.capture.capture()?;
        self.process_frame(&frame)
    }

    /// Setup processing pipeline based on config
    fn setup_processing_pipeline(&mut self) -> Result<(), VideoError> {
        self.processor.clear_filters();

        if self.config.stabilization {
            self.processor.add_filter(VideoFilter::Stabilization {
                strength: 0.8,
            });
        }

        if self.config.auto_exposure {
            self.processor.add_filter(VideoFilter::AutoExposure {
                target_brightness: 0.5,
            });
        }

        if self.config.hdr {
            self.processor.add_filter(VideoFilter::ToneMapping {
                exposure: 1.0,
            });
        }

        Ok(())
    }

    /// Setup encoder for recording
    fn setup_encoder(&mut self) -> Result<(), VideoError> {
        let encoder_config = EncoderConfig {
            width: self.config.width,
            height: self.config.height,
            framerate: self.config.framerate as f32,
            bitrate: 8_000_000, // 8 Mbps
            codec: VideoCodec::H264,
            profile: EncoderProfile::Main,
            ..Default::default()
        };

        let encoder = VideoEncoder::new(encoder_config)?;
        self.encoder = Some(encoder);
        Ok(())
    }

    fn update_metrics(&mut self, latency_us: u64) {
        self.metrics.frames_processed += 1;
        
        // Update average latency (exponential moving average)
        let alpha = 0.1;
        self.metrics.avg_latency_us = 
            self.metrics.avg_latency_us * (1.0 - alpha) + latency_us as f64 * alpha;

        if latency_us > self.metrics.peak_latency_us {
            self.metrics.peak_latency_us = latency_us;
        }

        // Update fps estimate
        if self.metrics.frames_processed > 10 {
            let frame_time_ms = self.metrics.avg_latency_us / 1000.0;
            if frame_time_ms > 0.0 {
                self.metrics.avg_fps = 1000.0 / frame_time_ms as f32;
            }
        }
    }

    /// Get pipeline metrics
    pub fn metrics(&self) -> &PipelineMetrics {
        &self.metrics
    }

    /// Get pipeline state
    pub fn state(&self) -> PipelineState {
        self.state
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.state == PipelineState::Running
    }

    /// Get capture device
    pub fn capture(&mut self) -> &mut VideoCapture {
        &mut self.capture
    }

    /// Get processor
    pub fn processor(&mut self) -> &mut VideoProcessor {
        &mut self.processor
    }

    /// Get encoder (if any)
    pub fn encoder(&mut self) -> Option<&mut VideoEncoder> {
        self.encoder.as_mut()
    }

    /// Enable streaming
    pub fn enable_streaming(&mut self, config: StreamConfig) -> Result<(), VideoError> {
        let streamer = VideoStreamer::new(config);
        self.streamer = Some(streamer);
        Ok(())
    }

    /// Disable streaming
    pub fn disable_streaming(&mut self) {
        self.streamer = None;
    }

    /// Start recording
    pub fn start_recording(&mut self, path: &str) -> Result<(), VideoError> {
        if self.encoder.is_none() {
            self.setup_encoder()?;
        }

        if let Some(ref mut encoder) = self.encoder {
            encoder.start_recording(path)?;
        }

        self.config.recording = true;
        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<Option<String>, VideoError> {
        self.config.recording = false;
        
        if let Some(ref mut encoder) = self.encoder {
            let path = encoder.stop_recording()?;
            return Ok(Some(path));
        }

        Ok(None)
    }

    /// Take a snapshot
    pub fn take_snapshot(&mut self) -> Result<VideoFrame, VideoError> {
        self.capture.capture()
    }
}

/// Video error types
#[derive(Debug, Clone)]
pub enum VideoError {
    /// Pipeline not running
    NotRunning,
    /// Not initialized
    NotInitialized,
    /// Camera not found
    CameraNotFound,
    /// Camera open failed
    CameraOpenFailed(String),
    /// Capture error
    CaptureError(String),
    /// Processing error
    ProcessingError(String),
    /// Encoding error
    EncodingError(String),
    /// Streaming error
    StreamingError(String),
    /// Configuration error
    ConfigError(String),
    /// I/O error
    IoError(String),
    /// Format not supported
    UnsupportedFormat,
}

impl std::fmt::Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoError::NotRunning => write!(f, "Video pipeline not running"),
            VideoError::NotInitialized => write!(f, "Video component not initialized"),
            VideoError::CameraNotFound => write!(f, "Camera not found"),
            VideoError::CameraOpenFailed(s) => write!(f, "Failed to open camera: {}", s),
            VideoError::CaptureError(s) => write!(f, "Capture error: {}", s),
            VideoError::ProcessingError(s) => write!(f, "Processing error: {}", s),
            VideoError::EncodingError(s) => write!(f, "Encoding error: {}", s),
            VideoError::StreamingError(s) => write!(f, "Streaming error: {}", s),
            VideoError::ConfigError(s) => write!(f, "Configuration error: {}", s),
            VideoError::IoError(s) => write!(f, "I/O error: {}", s),
            VideoError::UnsupportedFormat => write!(f, "Unsupported format"),
        }
    }
}

impl std::error::Error for VideoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = VideoPipelineConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.framerate, 30);
        assert!(config.stabilization);
    }

    #[test]
    fn test_pipeline_creation() {
        let config = VideoPipelineConfig::default();
        let pipeline = VideoPipeline::new(config);
        assert_eq!(pipeline.state(), PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_start_stop() {
        let config = VideoPipelineConfig::default();
        let mut pipeline = VideoPipeline::new(config);

        pipeline.start().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Running);
        assert!(pipeline.is_running());

        pipeline.stop().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_pause_resume() {
        let config = VideoPipelineConfig::default();
        let mut pipeline = VideoPipeline::new(config);

        pipeline.start().unwrap();
        pipeline.pause();
        assert_eq!(pipeline.state(), PipelineState::Paused);

        pipeline.resume();
        assert_eq!(pipeline.state(), PipelineState::Running);
    }

    #[test]
    fn test_metrics_initial() {
        let config = VideoPipelineConfig::default();
        let pipeline = VideoPipeline::new(config);
        let metrics = pipeline.metrics();

        assert_eq!(metrics.frames_processed, 0);
        assert_eq!(metrics.frames_dropped, 0);
    }
}
