// Kāraṇa OS - Audio Output Module
// Real-time audio playback to speakers/headphones

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::capture::AudioFrame;
use super::AudioError;

/// Audio output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Buffer size in samples per channel
    pub buffer_size: usize,
    /// Output device
    pub device: OutputDevice,
    /// Latency mode
    pub latency: LatencyMode,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 960,
            device: OutputDevice::Default,
            latency: LatencyMode::Normal,
        }
    }
}

/// Output device selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDevice {
    /// System default output
    Default,
    /// Specific device by name
    Device(String),
    /// Specific device by index
    DeviceIndex(usize),
    /// Null output (discards audio)
    Null,
}

/// Latency preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyMode {
    /// Ultra-low latency (<5ms) - may have glitches
    UltraLow,
    /// Low latency (~10ms)
    Low,
    /// Normal latency (~20ms) - good balance
    Normal,
    /// High latency (~50ms) - very stable
    High,
}

impl LatencyMode {
    /// Get target latency in milliseconds
    pub fn target_ms(&self) -> u32 {
        match self {
            LatencyMode::UltraLow => 5,
            LatencyMode::Low => 10,
            LatencyMode::Normal => 20,
            LatencyMode::High => 50,
        }
    }

    /// Get buffer count for this latency
    pub fn buffer_count(&self) -> usize {
        match self {
            LatencyMode::UltraLow => 2,
            LatencyMode::Low => 3,
            LatencyMode::Normal => 4,
            LatencyMode::High => 8,
        }
    }
}

/// Audio output manager
#[derive(Debug)]
pub struct AudioOutput {
    /// Configuration
    config: OutputConfig,
    /// Running state
    running: Arc<AtomicBool>,
    /// Output buffer queue
    buffer: VecDeque<AudioFrame>,
    /// Buffer capacity
    buffer_capacity: usize,
    /// Total frames written
    frames_written: Arc<AtomicU64>,
    /// Total frames played
    frames_played: Arc<AtomicU64>,
    /// Start time
    start_time: Option<Instant>,
    /// Volume (0.0 to 1.0)
    volume: f32,
    /// Muted state
    muted: bool,
}

impl AudioOutput {
    /// Create new audio output
    pub fn new(config: OutputConfig) -> Self {
        let capacity = config.latency.buffer_count() * 2;
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: VecDeque::with_capacity(capacity),
            buffer_capacity: capacity,
            frames_written: Arc::new(AtomicU64::new(0)),
            frames_played: Arc::new(AtomicU64::new(0)),
            start_time: None,
            volume: 1.0,
            muted: false,
        }
    }

    /// Start playback
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // In real implementation, would open audio device here
        match &self.config.device {
            OutputDevice::Default | OutputDevice::Device(_) | OutputDevice::DeviceIndex(_) => {
                // Would open ALSA/PulseAudio/CoreAudio device
            }
            OutputDevice::Null => {
                // Null output - just discard
            }
        }

        self.running.store(true, Ordering::SeqCst);
        self.start_time = Some(Instant::now());
        self.frames_written.store(0, Ordering::SeqCst);
        self.frames_played.store(0, Ordering::SeqCst);
        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.running.store(false, Ordering::SeqCst);
        self.buffer.clear();
        Ok(())
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Write audio frame to output buffer
    pub fn write(&mut self, frame: AudioFrame) -> Result<(), AudioError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(AudioError::NotRunning);
        }

        // Convert if needed
        let mut frame = if frame.sample_rate != self.config.sample_rate {
            frame.resample(self.config.sample_rate)
        } else {
            frame
        };

        // Apply volume and mute
        if self.muted {
            frame.data.iter_mut().for_each(|s| *s = 0.0);
        } else if (self.volume - 1.0).abs() > 0.001 {
            frame.apply_gain(self.volume);
        }

        // Check buffer space
        if self.buffer.len() >= self.buffer_capacity {
            // Buffer full - drop oldest
            self.buffer.pop_front();
        }

        self.buffer.push_back(frame);
        self.frames_written.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Write raw samples
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<(), AudioError> {
        let frame = AudioFrame::new(
            samples.to_vec(),
            self.config.channels,
            self.config.sample_rate,
        );
        self.write(frame)
    }

    /// Flush buffer (wait for playback to complete)
    pub fn flush(&mut self) -> Result<(), AudioError> {
        // In real implementation, would wait for device to drain
        // For now, just clear buffer
        while !self.buffer.is_empty() {
            self.buffer.pop_front();
            self.frames_played.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Get buffered duration
    pub fn buffered_duration(&self) -> Duration {
        let samples: usize = self.buffer.iter()
            .map(|f| f.samples_per_channel())
            .sum();
        let seconds = samples as f64 / self.config.sample_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    /// Get current latency estimate
    pub fn latency(&self) -> Duration {
        self.buffered_duration()
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get current volume
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Set muted state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Get frames written count
    pub fn frames_written(&self) -> u64 {
        self.frames_written.load(Ordering::SeqCst)
    }

    /// Get frames played count
    pub fn frames_played(&self) -> u64 {
        self.frames_played.load(Ordering::SeqCst)
    }

    /// Get configuration
    pub fn config(&self) -> &OutputConfig {
        &self.config
    }

    /// Get buffer fill level (0.0 to 1.0)
    pub fn buffer_level(&self) -> f32 {
        self.buffer.len() as f32 / self.buffer_capacity as f32
    }

    /// Check if buffer is underrun
    pub fn is_underrun(&self) -> bool {
        self.buffer.is_empty() && self.running.load(Ordering::SeqCst)
    }

    /// Simulate playback (for testing)
    pub fn simulate_playback(&mut self) -> Option<AudioFrame> {
        if let Some(frame) = self.buffer.pop_front() {
            self.frames_played.fetch_add(1, Ordering::SeqCst);
            Some(frame)
        } else {
            None
        }
    }
}

/// Audio mixer for combining multiple streams
#[derive(Debug)]
pub struct AudioMixer {
    /// Number of channels
    channels: u8,
    /// Sample rate
    sample_rate: u32,
    /// Input sources with gains
    sources: Vec<MixerSource>,
    /// Master volume
    master_volume: f32,
    /// Limiter enabled
    limiter_enabled: bool,
    /// Limiter threshold
    limiter_threshold: f32,
}

/// Mixer source
#[derive(Debug)]
struct MixerSource {
    /// Source ID
    id: u32,
    /// Gain (0.0 to 1.0+)
    gain: f32,
    /// Pan (-1.0 left to 1.0 right)
    pan: f32,
    /// Muted
    muted: bool,
    /// Solo
    solo: bool,
}

impl AudioMixer {
    /// Create new mixer
    pub fn new(channels: u8, sample_rate: u32) -> Self {
        Self {
            channels,
            sample_rate,
            sources: Vec::new(),
            master_volume: 1.0,
            limiter_enabled: true,
            limiter_threshold: 0.95,
        }
    }

    /// Add source to mixer
    pub fn add_source(&mut self, id: u32) {
        self.sources.push(MixerSource {
            id,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
        });
    }

    /// Remove source from mixer
    pub fn remove_source(&mut self, id: u32) {
        self.sources.retain(|s| s.id != id);
    }

    /// Set source gain
    pub fn set_source_gain(&mut self, id: u32, gain: f32) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.id == id) {
            source.gain = gain.max(0.0);
        }
    }

    /// Set source pan
    pub fn set_source_pan(&mut self, id: u32, pan: f32) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.id == id) {
            source.pan = pan.clamp(-1.0, 1.0);
        }
    }

    /// Set source muted
    pub fn set_source_muted(&mut self, id: u32, muted: bool) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.id == id) {
            source.muted = muted;
        }
    }

    /// Set source solo
    pub fn set_source_solo(&mut self, id: u32, solo: bool) {
        if let Some(source) = self.sources.iter_mut().find(|s| s.id == id) {
            source.solo = solo;
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
    }

    /// Mix multiple frames together
    pub fn mix(&self, frames: &[(u32, AudioFrame)]) -> AudioFrame {
        let mut result = AudioFrame::silence(
            frames.first().map(|(_, f)| f.samples_per_channel()).unwrap_or(960),
            self.channels,
            self.sample_rate,
        );

        // Check for solo sources
        let has_solo = self.sources.iter().any(|s| s.solo);

        for (id, frame) in frames {
            if let Some(source) = self.sources.iter().find(|s| s.id == *id) {
                // Skip if muted or not solo when solo exists
                if source.muted || (has_solo && !source.solo) {
                    continue;
                }

                // Resample if needed
                let frame = if frame.sample_rate != self.sample_rate {
                    frame.resample(self.sample_rate)
                } else {
                    frame.clone()
                };

                // Mix into result
                self.mix_frame(&frame, source, &mut result);
            }
        }

        // Apply master volume
        if (self.master_volume - 1.0).abs() > 0.001 {
            result.apply_gain(self.master_volume);
        }

        // Apply limiter
        if self.limiter_enabled {
            self.apply_limiter(&mut result);
        }

        result
    }

    /// Mix single frame into result
    fn mix_frame(&self, frame: &AudioFrame, source: &MixerSource, result: &mut AudioFrame) {
        let samples = frame.samples_per_channel().min(result.samples_per_channel());

        for i in 0..samples {
            // Get mono or stereo sample
            let (left, right) = if frame.channels >= 2 {
                let l = frame.data[i * frame.channels as usize];
                let r = frame.data[i * frame.channels as usize + 1];
                (l, r)
            } else {
                let m = frame.data[i * frame.channels as usize];
                (m, m)
            };

            // Apply gain and pan
            let gain = source.gain;
            let pan = source.pan;

            // Constant power pan law
            let pan_angle = (pan + 1.0) * 0.25 * std::f32::consts::PI;
            let left_gain = gain * pan_angle.cos();
            let right_gain = gain * pan_angle.sin();

            // Mix into result
            if self.channels >= 2 {
                result.data[i * self.channels as usize] += left * left_gain;
                result.data[i * self.channels as usize + 1] += right * right_gain;
            } else {
                result.data[i] += (left + right) * 0.5 * gain;
            }
        }
    }

    /// Apply soft limiter
    fn apply_limiter(&self, frame: &mut AudioFrame) {
        for sample in &mut frame.data {
            let abs = sample.abs();
            if abs > self.limiter_threshold {
                // Soft knee compression
                let excess = abs - self.limiter_threshold;
                let compressed = self.limiter_threshold + excess / (1.0 + excess);
                *sample = compressed * sample.signum();
            }
        }
    }
}

/// Crossfader for smooth transitions
#[derive(Debug)]
pub struct Crossfader {
    /// Current position (0.0 = A, 1.0 = B)
    position: f32,
    /// Crossfade curve
    curve: CrossfadeCurve,
}

/// Crossfade curve type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossfadeCurve {
    /// Linear crossfade
    Linear,
    /// Equal power (constant energy)
    EqualPower,
    /// S-curve (smooth)
    SCurve,
    /// Hard cut at 0.5
    HardCut,
}

impl Crossfader {
    /// Create new crossfader
    pub fn new(curve: CrossfadeCurve) -> Self {
        Self {
            position: 0.5,
            curve,
        }
    }

    /// Set position (0.0 = A, 1.0 = B)
    pub fn set_position(&mut self, position: f32) {
        self.position = position.clamp(0.0, 1.0);
    }

    /// Get current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Get gains for A and B
    pub fn gains(&self) -> (f32, f32) {
        match self.curve {
            CrossfadeCurve::Linear => {
                (1.0 - self.position, self.position)
            }
            CrossfadeCurve::EqualPower => {
                let angle = self.position * std::f32::consts::FRAC_PI_2;
                (angle.cos(), angle.sin())
            }
            CrossfadeCurve::SCurve => {
                // Smoothstep
                let t = self.position;
                let s = t * t * (3.0 - 2.0 * t);
                (1.0 - s, s)
            }
            CrossfadeCurve::HardCut => {
                if self.position < 0.5 {
                    (1.0, 0.0)
                } else {
                    (0.0, 1.0)
                }
            }
        }
    }

    /// Crossfade two frames
    pub fn crossfade(&self, frame_a: &AudioFrame, frame_b: &AudioFrame) -> AudioFrame {
        let (gain_a, gain_b) = self.gains();

        let samples = frame_a.data.len().min(frame_b.data.len());
        let mut result = vec![0.0f32; samples];

        for i in 0..samples {
            result[i] = frame_a.data[i] * gain_a + frame_b.data[i] * gain_b;
        }

        AudioFrame {
            data: result,
            channels: frame_a.channels,
            sample_rate: frame_a.sample_rate,
            timestamp: frame_a.timestamp,
            sequence: frame_a.sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.latency, LatencyMode::Normal);
    }

    #[test]
    fn test_latency_mode() {
        assert_eq!(LatencyMode::UltraLow.target_ms(), 5);
        assert_eq!(LatencyMode::Normal.target_ms(), 20);
        assert_eq!(LatencyMode::High.buffer_count(), 8);
    }

    #[test]
    fn test_output_start_stop() {
        let config = OutputConfig::default();
        let mut output = AudioOutput::new(config);

        output.start().unwrap();
        assert!(output.is_running());

        output.stop().unwrap();
        assert!(!output.is_running());
    }

    #[test]
    fn test_output_write() {
        let config = OutputConfig::default();
        let mut output = AudioOutput::new(config);
        output.start().unwrap();

        let frame = AudioFrame::silence(960, 2, 48000);
        output.write(frame).unwrap();

        assert_eq!(output.frames_written(), 1);
    }

    #[test]
    fn test_output_volume() {
        let config = OutputConfig::default();
        let mut output = AudioOutput::new(config);

        output.set_volume(0.5);
        assert_eq!(output.volume(), 0.5);

        output.set_volume(1.5); // Clamped
        assert_eq!(output.volume(), 1.0);
    }

    #[test]
    fn test_output_mute() {
        let config = OutputConfig::default();
        let mut output = AudioOutput::new(config);

        assert!(!output.is_muted());
        output.set_muted(true);
        assert!(output.is_muted());
    }

    #[test]
    fn test_mixer_basic() {
        let mut mixer = AudioMixer::new(2, 48000);
        mixer.add_source(1);
        mixer.add_source(2);

        let frame1 = AudioFrame::new(vec![0.5; 960 * 2], 2, 48000);
        let frame2 = AudioFrame::new(vec![0.3; 960 * 2], 2, 48000);

        let result = mixer.mix(&[(1, frame1), (2, frame2)]);
        assert_eq!(result.channels, 2);
        assert_eq!(result.sample_rate, 48000);
    }

    #[test]
    fn test_mixer_mute() {
        let mut mixer = AudioMixer::new(2, 48000);
        mixer.add_source(1);
        mixer.add_source(2);
        mixer.set_source_muted(1, true);

        let frame1 = AudioFrame::new(vec![1.0; 960 * 2], 2, 48000);
        let frame2 = AudioFrame::new(vec![0.5; 960 * 2], 2, 48000);

        let result = mixer.mix(&[(1, frame1), (2, frame2)]);
        // Only frame2 should be mixed (source 1 is muted)
        assert!(result.data[0] < 0.8); // Should be around 0.5 (from frame2 only)
    }

    #[test]
    fn test_mixer_solo() {
        let mut mixer = AudioMixer::new(2, 48000);
        mixer.add_source(1);
        mixer.add_source(2);
        mixer.set_source_solo(1, true);

        let frame1 = AudioFrame::new(vec![1.0; 960 * 2], 2, 48000);
        let frame2 = AudioFrame::new(vec![0.5; 960 * 2], 2, 48000);

        let result = mixer.mix(&[(1, frame1), (2, frame2)]);
        // Only source 1 (solo) should be heard
        assert!(result.data[0] > 0.7);
    }

    #[test]
    fn test_crossfader_linear() {
        let crossfader = Crossfader::new(CrossfadeCurve::Linear);
        
        let mut cf = crossfader;
        cf.set_position(0.0);
        let (a, b) = cf.gains();
        assert!((a - 1.0).abs() < 0.001);
        assert!(b < 0.001);

        cf.set_position(1.0);
        let (a, b) = cf.gains();
        assert!(a < 0.001);
        assert!((b - 1.0).abs() < 0.001);

        cf.set_position(0.5);
        let (a, b) = cf.gains();
        assert!((a - 0.5).abs() < 0.001);
        assert!((b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_crossfader_equal_power() {
        let mut crossfader = Crossfader::new(CrossfadeCurve::EqualPower);
        crossfader.set_position(0.5);

        let (a, b) = crossfader.gains();
        // At midpoint, both should be ~0.707 for constant power
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!((a - expected).abs() < 0.01);
        assert!((b - expected).abs() < 0.01);
    }

    #[test]
    fn test_crossfader_apply() {
        let mut crossfader = Crossfader::new(CrossfadeCurve::Linear);
        crossfader.set_position(0.5);

        let frame_a = AudioFrame::new(vec![1.0; 100], 1, 48000);
        let frame_b = AudioFrame::new(vec![0.0; 100], 1, 48000);

        let result = crossfader.crossfade(&frame_a, &frame_b);
        assert!((result.data[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_limiter() {
        let mixer = AudioMixer::new(2, 48000);
        let mut frame = AudioFrame::new(vec![1.5; 100], 1, 48000);
        
        mixer.apply_limiter(&mut frame);
        
        // All samples should be reduced after limiting (soft limiter)
        // The formula: threshold + excess/(1+excess) with threshold=0.95
        // For input 1.5: excess = 0.55, compressed = 0.95 + 0.55/1.55 = 0.95 + 0.355 = 1.305
        // So soft limiter reduces but doesn't hard clip to <1.0
        for sample in &frame.data {
            // Should be less than original 1.5
            assert!(*sample < 1.5, "Sample {} not reduced", sample);
            // Soft limiter asymptotically approaches threshold + 1.0
            assert!(*sample < 1.95, "Sample {} exceeds soft limit", sample);
        }
    }
}
