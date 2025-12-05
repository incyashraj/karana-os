// Kāraṇa OS - Audio Processing Pipeline
// Real-time audio capture, processing, and spatial audio for smart glasses

pub mod analysis;
pub mod capture;
pub mod effects;
pub mod output;
pub mod spatial;

pub use analysis::{AudioAnalyzer, AnalyzerConfig, AnalysisResult, LevelMeter, SpectrumAnalyzer, WindowType};
pub use capture::{AudioCapture, CaptureConfig, CaptureSource, AudioFrame, AudioDevice, list_devices};
pub use effects::{AudioEffect, GainEffect, HighPassFilter, LowPassFilter, ParametricEQ, Compressor, DelayEffect, NoiseGate, EffectsChain, EQBand, EQBandType};
pub use output::{AudioOutput, OutputConfig, OutputDevice, LatencyMode, AudioMixer, Crossfader, CrossfadeCurve};
pub use spatial::{SpatialAudio, SpatialConfig, AudioSource3D, Listener, Position3D, Orientation3D, AttenuationModel};

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Audio pipeline configuration
#[derive(Debug, Clone)]
pub struct AudioPipelineConfig {
    /// Sample rate (typically 16000 for speech, 48000 for audio)
    pub sample_rate: u32,
    /// Buffer size in samples
    pub buffer_size: usize,
    /// Number of input channels
    pub input_channels: u8,
    /// Number of output channels  
    pub output_channels: u8,
    /// Enable noise cancellation
    pub noise_cancellation: bool,
    /// Enable echo cancellation
    pub echo_cancellation: bool,
    /// Enable automatic gain control
    pub agc: bool,
    /// Enable spatial audio
    pub spatial_audio: bool,
    /// Latency target (ms)
    pub target_latency_ms: u32,
}

impl Default for AudioPipelineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 960, // 20ms at 48kHz
            input_channels: 2, // Stereo mic
            output_channels: 2, // Stereo output
            noise_cancellation: true,
            echo_cancellation: true,
            agc: true,
            spatial_audio: true,
            target_latency_ms: 20,
        }
    }
}

/// Main audio pipeline manager
#[derive(Debug)]
pub struct AudioPipeline {
    /// Configuration
    config: AudioPipelineConfig,
    /// Audio capture
    capture: AudioCapture,
    /// Audio output
    output: AudioOutput,
    /// Effects chain
    effects: EffectsChain,
    /// Spatial audio engine
    spatial: SpatialAudio,
    /// Audio mixer
    mixer: AudioMixer,
    /// Audio analyzer
    analyzer: AudioAnalyzer,
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
    /// Buffer underruns
    pub underruns: u64,
    /// Buffer overruns
    pub overruns: u64,
    /// Current CPU usage (%)
    pub cpu_usage: f32,
}

impl AudioPipeline {
    /// Create new audio pipeline
    pub fn new(config: AudioPipelineConfig) -> Self {
        let capture_config = CaptureConfig {
            sample_rate: config.sample_rate,
            channels: config.input_channels,
            buffer_size: config.buffer_size,
            source: CaptureSource::Default,
        };

        let output_config = OutputConfig {
            sample_rate: config.sample_rate,
            channels: config.output_channels,
            buffer_size: config.buffer_size,
            device: OutputDevice::Default,
            latency: LatencyMode::Normal,
        };

        let spatial_config = SpatialConfig::default();
        let analyzer_config = AnalyzerConfig::default();

        Self {
            capture: AudioCapture::new(capture_config),
            output: AudioOutput::new(output_config),
            effects: EffectsChain::new(),
            spatial: SpatialAudio::new(spatial_config, config.sample_rate),
            mixer: AudioMixer::new(config.output_channels, config.sample_rate),
            analyzer: AudioAnalyzer::new(analyzer_config, config.sample_rate),
            running: Arc::new(AtomicBool::new(false)),
            frame_count: AtomicU64::new(0),
            metrics: PipelineMetrics::default(),
            state: PipelineState::Idle,
            config,
        }
    }

    /// Start the audio pipeline
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.state == PipelineState::Running {
            return Ok(());
        }

        self.state = PipelineState::Starting;
        self.running.store(true, Ordering::SeqCst);

        // Initialize capture and output
        self.capture.start()?;
        self.output.start()?;

        // Setup processing chain
        self.setup_effects_chain()?;

        self.state = PipelineState::Running;
        Ok(())
    }

    /// Stop the audio pipeline
    pub fn stop(&mut self) -> Result<(), AudioError> {
        if self.state != PipelineState::Running {
            return Ok(());
        }

        self.state = PipelineState::Stopping;
        self.running.store(false, Ordering::SeqCst);

        self.capture.stop()?;
        self.output.stop()?;
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

    /// Process one frame of audio
    pub fn process_frame(&mut self, input: &AudioFrame) -> Result<AudioFrame, AudioError> {
        if self.state != PipelineState::Running {
            return Err(AudioError::NotRunning);
        }

        let start = Instant::now();

        // Run through effects chain
        let processed = self.effects.process(input);

        // Analyze audio
        let _analysis = self.analyzer.analyze(&processed);

        // Apply spatial audio if enabled
        let output = if self.config.spatial_audio {
            // For now, just pass through since we need source IDs
            processed
        } else {
            processed
        };

        // Update metrics
        let latency_us = start.elapsed().as_micros() as u64;
        self.update_metrics(latency_us);

        self.frame_count.fetch_add(1, Ordering::Relaxed);
        Ok(output)
    }

    /// Setup the effects chain based on config
    fn setup_effects_chain(&mut self) -> Result<(), AudioError> {
        // Clear existing chain
        while self.effects.len() > 0 {
            self.effects.remove(0);
        }

        if self.config.noise_cancellation {
            // Add noise gate
            let mut gate = NoiseGate::new(self.config.sample_rate);
            gate.set_parameter("threshold", -40.0);
            self.effects.add(Box::new(gate));
        }

        if self.config.agc {
            // Add compressor for AGC
            let mut comp = Compressor::new(self.config.sample_rate);
            comp.set_parameter("threshold", -20.0);
            comp.set_parameter("ratio", 4.0);
            self.effects.add(Box::new(comp));
        }

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

    /// Get spatial audio engine
    pub fn spatial(&mut self) -> &mut SpatialAudio {
        &mut self.spatial
    }

    /// Get mixer
    pub fn mixer(&mut self) -> &mut AudioMixer {
        &mut self.mixer
    }

    /// Get effects chain
    pub fn effects(&mut self) -> &mut EffectsChain {
        &mut self.effects
    }

    /// Get analyzer
    pub fn analyzer(&mut self) -> &mut AudioAnalyzer {
        &mut self.analyzer
    }

    /// Get capture device
    pub fn capture(&mut self) -> &mut AudioCapture {
        &mut self.capture
    }

    /// Get output device
    pub fn output(&mut self) -> &mut AudioOutput {
        &mut self.output
    }
}

/// Audio error types
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Pipeline not running
    NotRunning,
    /// Device not found
    DeviceNotFound,
    /// Device open failed
    DeviceOpenFailed(String),
    /// Buffer overflow
    BufferOverflow,
    /// Buffer underflow
    BufferUnderflow,
    /// Processing error
    ProcessingError(String),
    /// Configuration error
    ConfigError(String),
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::NotRunning => write!(f, "Audio pipeline not running"),
            AudioError::DeviceNotFound => write!(f, "Audio device not found"),
            AudioError::DeviceOpenFailed(s) => write!(f, "Failed to open device: {}", s),
            AudioError::BufferOverflow => write!(f, "Buffer overflow"),
            AudioError::BufferUnderflow => write!(f, "Buffer underflow"),
            AudioError::ProcessingError(s) => write!(f, "Processing error: {}", s),
            AudioError::ConfigError(s) => write!(f, "Configuration error: {}", s),
            AudioError::IoError(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = AudioPipelineConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 960);
        assert!(config.noise_cancellation);
        assert!(config.spatial_audio);
    }

    #[test]
    fn test_pipeline_creation() {
        let config = AudioPipelineConfig::default();
        let pipeline = AudioPipeline::new(config);
        assert_eq!(pipeline.state(), PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_start_stop() {
        let config = AudioPipelineConfig::default();
        let mut pipeline = AudioPipeline::new(config);

        pipeline.start().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Running);
        assert!(pipeline.is_running());

        pipeline.stop().unwrap();
        assert_eq!(pipeline.state(), PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_pause_resume() {
        let config = AudioPipelineConfig::default();
        let mut pipeline = AudioPipeline::new(config);

        pipeline.start().unwrap();
        pipeline.pause();
        assert_eq!(pipeline.state(), PipelineState::Paused);

        pipeline.resume();
        assert_eq!(pipeline.state(), PipelineState::Running);
    }

    #[test]
    fn test_metrics_initial() {
        let config = AudioPipelineConfig::default();
        let pipeline = AudioPipeline::new(config);
        let metrics = pipeline.metrics();

        assert_eq!(metrics.frames_processed, 0);
        assert_eq!(metrics.frames_dropped, 0);
    }
}
