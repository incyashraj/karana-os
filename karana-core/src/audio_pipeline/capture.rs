// Kāraṇa OS - Audio Capture Module
// Real-time audio input from microphones

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::AudioError;

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Buffer size in samples per channel
    pub buffer_size: usize,
    /// Capture source
    pub source: CaptureSource,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 960,
            source: CaptureSource::Default,
        }
    }
}

/// Audio capture source
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureSource {
    /// System default input device
    Default,
    /// Specific device by name
    Device(String),
    /// Specific device by index
    DeviceIndex(usize),
    /// Virtual/loopback device
    Loopback,
    /// Test tone generator
    TestTone,
}

/// Single audio frame
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Interleaved audio samples
    pub data: Vec<f32>,
    /// Number of channels
    pub channels: u8,
    /// Sample rate
    pub sample_rate: u32,
    /// Timestamp (monotonic)
    pub timestamp: u64,
    /// Frame sequence number
    pub sequence: u64,
}

impl AudioFrame {
    /// Create new audio frame
    pub fn new(data: Vec<f32>, channels: u8, sample_rate: u32) -> Self {
        Self {
            data,
            channels,
            sample_rate,
            timestamp: 0,
            sequence: 0,
        }
    }

    /// Create silence frame
    pub fn silence(samples: usize, channels: u8, sample_rate: u32) -> Self {
        Self {
            data: vec![0.0; samples * channels as usize],
            channels,
            sample_rate,
            timestamp: 0,
            sequence: 0,
        }
    }

    /// Number of samples per channel
    pub fn samples_per_channel(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.data.len() / self.channels as usize
        }
    }

    /// Duration in seconds
    pub fn duration(&self) -> f32 {
        self.samples_per_channel() as f32 / self.sample_rate as f32
    }

    /// Get channel data
    pub fn channel(&self, ch: u8) -> Vec<f32> {
        if ch >= self.channels {
            return Vec::new();
        }

        self.data
            .iter()
            .skip(ch as usize)
            .step_by(self.channels as usize)
            .copied()
            .collect()
    }

    /// Mix to mono
    pub fn to_mono(&self) -> AudioFrame {
        if self.channels == 1 {
            return self.clone();
        }

        let samples = self.samples_per_channel();
        let mut mono = vec![0.0f32; samples];

        for i in 0..samples {
            let mut sum = 0.0f32;
            for ch in 0..self.channels as usize {
                sum += self.data[i * self.channels as usize + ch];
            }
            mono[i] = sum / self.channels as f32;
        }

        AudioFrame {
            data: mono,
            channels: 1,
            sample_rate: self.sample_rate,
            timestamp: self.timestamp,
            sequence: self.sequence,
        }
    }

    /// Calculate RMS level
    pub fn rms(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.data.iter().map(|s| s * s).sum();
        (sum / self.data.len() as f32).sqrt()
    }

    /// Calculate peak level
    pub fn peak(&self) -> f32 {
        self.data.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    /// Calculate dB level
    pub fn db(&self) -> f32 {
        let rms = self.rms();
        if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            -100.0
        }
    }

    /// Apply gain
    pub fn apply_gain(&mut self, gain: f32) {
        for sample in &mut self.data {
            *sample *= gain;
        }
    }

    /// Resample to target rate (simple linear interpolation)
    pub fn resample(&self, target_rate: u32) -> AudioFrame {
        if target_rate == self.sample_rate {
            return self.clone();
        }

        let ratio = self.sample_rate as f64 / target_rate as f64;
        let new_samples = (self.samples_per_channel() as f64 / ratio) as usize;
        let mut new_data = Vec::with_capacity(new_samples * self.channels as usize);

        for i in 0..new_samples {
            let src_pos = i as f64 * ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;

            for ch in 0..self.channels as usize {
                let idx1 = src_idx * self.channels as usize + ch;
                let idx2 = (src_idx + 1) * self.channels as usize + ch;

                let sample = if idx2 < self.data.len() {
                    let s1 = self.data[idx1] as f64;
                    let s2 = self.data[idx2] as f64;
                    (s1 * (1.0 - frac) + s2 * frac) as f32
                } else if idx1 < self.data.len() {
                    self.data[idx1]
                } else {
                    0.0
                };

                new_data.push(sample);
            }
        }

        AudioFrame {
            data: new_data,
            channels: self.channels,
            sample_rate: target_rate,
            timestamp: self.timestamp,
            sequence: self.sequence,
        }
    }
}

/// Audio capture manager
#[derive(Debug)]
pub struct AudioCapture {
    /// Configuration
    config: CaptureConfig,
    /// Running state
    running: Arc<AtomicBool>,
    /// Frame buffer
    buffer: VecDeque<AudioFrame>,
    /// Buffer capacity
    buffer_capacity: usize,
    /// Frame sequence counter
    sequence: u64,
    /// Start time
    start_time: Option<Instant>,
    /// Test tone phase (for TestTone source)
    test_phase: f32,
}

impl AudioCapture {
    /// Create new audio capture
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: VecDeque::with_capacity(100),
            buffer_capacity: 100,
            sequence: 0,
            start_time: None,
            test_phase: 0.0,
        }
    }

    /// Start capturing
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // In real implementation, would open audio device here
        match &self.config.source {
            CaptureSource::Default | CaptureSource::Device(_) | CaptureSource::DeviceIndex(_) => {
                // Would open ALSA/PulseAudio/CoreAudio device
            }
            CaptureSource::Loopback => {
                // Would open loopback device
            }
            CaptureSource::TestTone => {
                // Test mode - generates sine wave
            }
        }

        self.running.store(true, Ordering::SeqCst);
        self.start_time = Some(Instant::now());
        self.sequence = 0;
        Ok(())
    }

    /// Stop capturing
    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.running.store(false, Ordering::SeqCst);
        self.buffer.clear();
        Ok(())
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Read next audio frame
    pub fn read(&mut self) -> Result<AudioFrame, AudioError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(AudioError::NotRunning);
        }

        // Check buffer first
        if let Some(frame) = self.buffer.pop_front() {
            return Ok(frame);
        }

        // Generate frame based on source
        let frame = match &self.config.source {
            CaptureSource::TestTone => self.generate_test_tone(),
            _ => self.capture_from_device()?,
        };

        Ok(frame)
    }

    /// Generate test tone (440Hz sine wave)
    fn generate_test_tone(&mut self) -> AudioFrame {
        let samples = self.config.buffer_size;
        let channels = self.config.channels;
        let sample_rate = self.config.sample_rate;

        let mut data = Vec::with_capacity(samples * channels as usize);
        let freq = 440.0; // A4
        let phase_inc = 2.0 * std::f32::consts::PI * freq / sample_rate as f32;

        for _ in 0..samples {
            let sample = (self.test_phase).sin() * 0.3; // -10dB
            for _ in 0..channels {
                data.push(sample);
            }
            self.test_phase += phase_inc;
            if self.test_phase > 2.0 * std::f32::consts::PI {
                self.test_phase -= 2.0 * std::f32::consts::PI;
            }
        }

        let timestamp = self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0);

        self.sequence += 1;

        AudioFrame {
            data,
            channels,
            sample_rate,
            timestamp,
            sequence: self.sequence,
        }
    }

    /// Capture from actual device (simulated)
    fn capture_from_device(&mut self) -> Result<AudioFrame, AudioError> {
        // In real implementation, would read from device
        // For now, return silence
        let samples = self.config.buffer_size;
        let channels = self.config.channels;

        let timestamp = self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0);

        self.sequence += 1;

        Ok(AudioFrame {
            data: vec![0.0; samples * channels as usize],
            channels,
            sample_rate: self.config.sample_rate,
            timestamp,
            sequence: self.sequence,
        })
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

    /// Set new configuration (must be stopped)
    pub fn set_config(&mut self, config: CaptureConfig) -> Result<(), AudioError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(AudioError::ConfigError("Cannot change config while running".into()));
        }
        self.config = config;
        Ok(())
    }
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device name
    pub name: String,
    /// Device ID
    pub id: String,
    /// Is default input
    pub is_default_input: bool,
    /// Is default output
    pub is_default_output: bool,
    /// Max input channels
    pub max_input_channels: u8,
    /// Max output channels
    pub max_output_channels: u8,
    /// Supported sample rates
    pub sample_rates: Vec<u32>,
}

/// List available audio devices
pub fn list_devices() -> Vec<AudioDevice> {
    // In real implementation, would query system
    // For now, return simulated devices
    vec![
        AudioDevice {
            name: "Built-in Microphone".to_string(),
            id: "builtin_mic".to_string(),
            is_default_input: true,
            is_default_output: false,
            max_input_channels: 2,
            max_output_channels: 0,
            sample_rates: vec![16000, 44100, 48000],
        },
        AudioDevice {
            name: "Built-in Speakers".to_string(),
            id: "builtin_spk".to_string(),
            is_default_input: false,
            is_default_output: true,
            max_input_channels: 0,
            max_output_channels: 2,
            sample_rates: vec![44100, 48000, 96000],
        },
    ]
}

/// Get default input device
pub fn default_input_device() -> Option<AudioDevice> {
    list_devices().into_iter().find(|d| d.is_default_input)
}

/// Get default output device
pub fn default_output_device() -> Option<AudioDevice> {
    list_devices().into_iter().find(|d| d.is_default_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_audio_frame() {
        let data = vec![0.5f32; 960 * 2]; // Stereo
        let frame = AudioFrame::new(data, 2, 48000);

        assert_eq!(frame.samples_per_channel(), 960);
        assert_eq!(frame.duration(), 0.02); // 20ms
    }

    #[test]
    fn test_audio_frame_silence() {
        let frame = AudioFrame::silence(480, 2, 48000);
        assert_eq!(frame.data.len(), 960);
        assert_eq!(frame.rms(), 0.0);
    }

    #[test]
    fn test_audio_frame_mono() {
        let mut data = Vec::new();
        for i in 0..100 {
            data.push(0.5); // Left
            data.push(-0.5); // Right
        }
        let frame = AudioFrame::new(data, 2, 48000);
        let mono = frame.to_mono();

        assert_eq!(mono.channels, 1);
        assert_eq!(mono.data.len(), 100);
        assert!(mono.data[0].abs() < 0.001); // Should be ~0 (average of 0.5 and -0.5)
    }

    #[test]
    fn test_audio_frame_channel() {
        let mut data = Vec::new();
        for i in 0..100 {
            data.push(1.0); // Left
            data.push(2.0); // Right
        }
        let frame = AudioFrame::new(data, 2, 48000);

        let left = frame.channel(0);
        let right = frame.channel(1);

        assert_eq!(left.len(), 100);
        assert_eq!(right.len(), 100);
        assert_eq!(left[0], 1.0);
        assert_eq!(right[0], 2.0);
    }

    #[test]
    fn test_audio_frame_rms() {
        let data = vec![0.5f32; 100];
        let frame = AudioFrame::new(data, 1, 48000);
        assert!((frame.rms() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_audio_frame_db() {
        let data = vec![1.0f32; 100]; // Full scale
        let frame = AudioFrame::new(data, 1, 48000);
        assert!((frame.db() - 0.0).abs() < 0.1);

        let data2 = vec![0.1f32; 100]; // -20dB
        let frame2 = AudioFrame::new(data2, 1, 48000);
        assert!((frame2.db() - (-20.0)).abs() < 1.0);
    }

    #[test]
    fn test_audio_frame_gain() {
        let data = vec![0.5f32; 100];
        let mut frame = AudioFrame::new(data, 1, 48000);
        frame.apply_gain(2.0);
        assert_eq!(frame.data[0], 1.0);
    }

    #[test]
    fn test_capture_start_stop() {
        let config = CaptureConfig::default();
        let mut capture = AudioCapture::new(config);

        capture.start().unwrap();
        assert!(capture.is_running());

        capture.stop().unwrap();
        assert!(!capture.is_running());
    }

    #[test]
    fn test_capture_test_tone() {
        let config = CaptureConfig {
            source: CaptureSource::TestTone,
            ..Default::default()
        };
        let mut capture = AudioCapture::new(config);

        capture.start().unwrap();
        let frame = capture.read().unwrap();

        assert!(!frame.data.is_empty());
        assert!(frame.rms() > 0.0); // Should have audio
        assert!(frame.sequence > 0);
    }

    #[test]
    fn test_list_devices() {
        let devices = list_devices();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_default_devices() {
        let input = default_input_device();
        let output = default_output_device();

        assert!(input.is_some());
        assert!(output.is_some());
    }

    #[test]
    fn test_resample() {
        let data = vec![0.5f32; 480 * 2]; // 480 stereo samples at 48kHz
        let frame = AudioFrame::new(data, 2, 48000);

        let resampled = frame.resample(16000);
        assert_eq!(resampled.sample_rate, 16000);
        assert_eq!(resampled.channels, 2);
        // 480 * 16000/48000 = 160 samples
        assert_eq!(resampled.samples_per_channel(), 160);
    }
}
