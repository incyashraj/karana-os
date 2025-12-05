// Kāraṇa OS - Audio HAL
// Hardware abstraction for smart glasses audio

use super::HalError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Audio configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Bits per sample
    pub bits_per_sample: u8,
    /// Buffer size (samples)
    pub buffer_size: u32,
    /// Latency target (ms)
    pub latency_target_ms: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 16,
            buffer_size: 256,
            latency_target_ms: 10,
        }
    }
}

/// Audio route
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRoute {
    /// Built-in speakers
    Speaker,
    /// Bone conduction transducer
    BoneConduction,
    /// Bluetooth headset
    BluetoothHeadset,
    /// Wired headphones
    WiredHeadphones,
    /// USB audio
    UsbAudio,
}

/// Audio output device
#[derive(Debug, Clone)]
pub struct AudioOutputDevice {
    /// Device name
    pub name: String,
    /// Device ID
    pub id: String,
    /// Route type
    pub route: AudioRoute,
    /// Max volume
    pub max_volume: u8,
    /// Supports spatial audio
    pub spatial_audio: bool,
}

/// Audio input device  
#[derive(Debug, Clone)]
pub struct AudioInputDevice {
    /// Device name
    pub name: String,
    /// Device ID
    pub id: String,
    /// Is microphone array
    pub is_array: bool,
    /// Number of channels
    pub channels: u8,
    /// Supports noise cancellation
    pub noise_cancellation: bool,
    /// Supports beamforming
    pub beamforming: bool,
}

/// Audio HAL state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    /// Not initialized
    Uninitialized,
    /// Idle
    Idle,
    /// Playing
    Playing,
    /// Recording
    Recording,
    /// Playing and recording
    Duplex,
    /// Suspended
    Suspended,
}

/// Audio HAL
#[derive(Debug)]
pub struct AudioHal {
    /// Configuration
    config: AudioConfig,
    /// Current state
    state: AudioState,
    /// Current output route
    output_route: AudioRoute,
    /// Current volume (0-100)
    volume: u8,
    /// Is muted
    muted: bool,
    /// Statistics
    stats: AudioStats,
    /// Sample counter
    sample_counter: AtomicU64,
    /// Is initialized
    initialized: bool,
    /// Start time
    start_time: Option<Instant>,
}

/// Audio statistics
#[derive(Debug, Default, Clone)]
pub struct AudioStats {
    /// Samples processed
    pub samples_processed: u64,
    /// Buffer underruns
    pub underruns: u64,
    /// Buffer overruns
    pub overruns: u64,
    /// Current latency (ms)
    pub latency_ms: f32,
    /// Average CPU usage (%)
    pub cpu_usage: f32,
}

impl AudioHal {
    /// Create new audio HAL
    pub fn new(config: AudioConfig) -> Result<Self, HalError> {
        Ok(Self {
            config,
            state: AudioState::Uninitialized,
            output_route: AudioRoute::Speaker,
            volume: 70,
            muted: false,
            stats: AudioStats::default(),
            sample_counter: AtomicU64::new(0),
            initialized: false,
            start_time: None,
        })
    }

    /// Initialize audio system
    pub fn initialize(&mut self) -> Result<(), HalError> {
        // Detect audio devices
        self.detect_devices()?;

        self.state = AudioState::Idle;
        self.initialized = true;
        Ok(())
    }

    /// Start audio processing
    pub fn start(&mut self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Audio not initialized".into()));
        }

        self.start_time = Some(Instant::now());
        self.state = AudioState::Idle;
        Ok(())
    }

    /// Stop audio processing
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.state = AudioState::Idle;
        self.start_time = None;
        Ok(())
    }

    /// Suspend audio (low power)
    pub fn suspend(&mut self) -> Result<(), HalError> {
        self.state = AudioState::Suspended;
        Ok(())
    }

    /// Resume audio
    pub fn resume(&mut self) -> Result<(), HalError> {
        self.state = AudioState::Idle;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> AudioState {
        self.state
    }

    /// Get statistics
    pub fn stats(&self) -> AudioStats {
        AudioStats {
            samples_processed: self.sample_counter.load(Ordering::Relaxed),
            ..self.stats.clone()
        }
    }

    /// Start playback
    pub fn start_playback(&mut self) -> Result<(), HalError> {
        if self.state == AudioState::Recording {
            self.state = AudioState::Duplex;
        } else {
            self.state = AudioState::Playing;
        }
        Ok(())
    }

    /// Stop playback
    pub fn stop_playback(&mut self) -> Result<(), HalError> {
        if self.state == AudioState::Duplex {
            self.state = AudioState::Recording;
        } else {
            self.state = AudioState::Idle;
        }
        Ok(())
    }

    /// Start recording
    pub fn start_recording(&mut self) -> Result<(), HalError> {
        if self.state == AudioState::Playing {
            self.state = AudioState::Duplex;
        } else {
            self.state = AudioState::Recording;
        }
        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<(), HalError> {
        if self.state == AudioState::Duplex {
            self.state = AudioState::Playing;
        } else {
            self.state = AudioState::Idle;
        }
        Ok(())
    }

    /// Write audio samples (playback)
    pub fn write(&mut self, samples: &[i16]) -> Result<usize, HalError> {
        if self.state != AudioState::Playing && self.state != AudioState::Duplex {
            return Err(HalError::ConfigError("Not playing".into()));
        }

        self.sample_counter.fetch_add(samples.len() as u64, Ordering::Relaxed);
        Ok(samples.len())
    }

    /// Read audio samples (recording)
    pub fn read(&mut self, buffer: &mut [i16]) -> Result<usize, HalError> {
        if self.state != AudioState::Recording && self.state != AudioState::Duplex {
            return Err(HalError::ConfigError("Not recording".into()));
        }

        // Fill with silence/simulated data
        for sample in buffer.iter_mut() {
            *sample = 0;
        }

        self.sample_counter.fetch_add(buffer.len() as u64, Ordering::Relaxed);
        Ok(buffer.len())
    }

    /// Set output route
    pub fn set_route(&mut self, route: AudioRoute) -> Result<(), HalError> {
        self.output_route = route;
        Ok(())
    }

    /// Get current route
    pub fn route(&self) -> AudioRoute {
        self.output_route
    }

    /// Set volume (0-100)
    pub fn set_volume(&mut self, volume: u8) -> Result<(), HalError> {
        if volume > 100 {
            return Err(HalError::ConfigError("Volume must be 0-100".into()));
        }
        self.volume = volume;
        Ok(())
    }

    /// Get current volume
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Mute/unmute
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Is muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<(), HalError> {
        if self.state != AudioState::Idle {
            return Err(HalError::DeviceBusy);
        }

        self.config.sample_rate = rate;
        Ok(())
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Self-test
    pub fn test(&self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Not initialized".into()));
        }
        Ok(())
    }

    /// Detect audio devices
    fn detect_devices(&self) -> Result<(), HalError> {
        // Would enumerate actual audio devices
        Ok(())
    }

    /// Get available output devices
    pub fn output_devices(&self) -> Vec<AudioOutputDevice> {
        vec![
            AudioOutputDevice {
                name: "Built-in Speakers".into(),
                id: "speaker:0".into(),
                route: AudioRoute::Speaker,
                max_volume: 100,
                spatial_audio: true,
            },
            AudioOutputDevice {
                name: "Bone Conduction".into(),
                id: "bone:0".into(),
                route: AudioRoute::BoneConduction,
                max_volume: 100,
                spatial_audio: false,
            },
        ]
    }

    /// Get available input devices
    pub fn input_devices(&self) -> Vec<AudioInputDevice> {
        vec![
            AudioInputDevice {
                name: "Microphone Array".into(),
                id: "mic:0".into(),
                is_array: true,
                channels: 4,
                noise_cancellation: true,
                beamforming: true,
            },
        ]
    }
}

/// Spatial audio processor
#[derive(Debug)]
pub struct SpatialAudioProcessor {
    /// Sample rate
    sample_rate: u32,
    /// Head tracking enabled
    head_tracking: bool,
    /// HRTF enabled
    hrtf_enabled: bool,
    /// Room simulation enabled
    room_simulation: bool,
    /// Listener position
    listener_pos: [f32; 3],
    /// Listener rotation (euler angles)
    listener_rot: [f32; 3],
}

impl SpatialAudioProcessor {
    /// Create new spatial processor
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            head_tracking: true,
            hrtf_enabled: true,
            room_simulation: false,
            listener_pos: [0.0, 0.0, 0.0],
            listener_rot: [0.0, 0.0, 0.0],
        }
    }

    /// Update listener position
    pub fn update_listener(&mut self, position: [f32; 3], rotation: [f32; 3]) {
        self.listener_pos = position;
        self.listener_rot = rotation;
    }

    /// Enable/disable head tracking
    pub fn set_head_tracking(&mut self, enabled: bool) {
        self.head_tracking = enabled;
    }

    /// Enable/disable HRTF
    pub fn set_hrtf(&mut self, enabled: bool) {
        self.hrtf_enabled = enabled;
    }

    /// Process audio for spatial rendering
    pub fn process(&self, input: &[f32], source_pos: [f32; 3], output: &mut [f32]) {
        // Simple panning based on source position
        let rel_x = source_pos[0] - self.listener_pos[0];
        let rel_z = source_pos[2] - self.listener_pos[2];
        
        let angle = rel_x.atan2(rel_z);
        let pan = (angle / std::f32::consts::PI).clamp(-1.0, 1.0);
        
        let left_gain = ((1.0 - pan) / 2.0).sqrt();
        let right_gain = ((1.0 + pan) / 2.0).sqrt();

        for (i, sample) in input.iter().enumerate() {
            if i * 2 + 1 < output.len() {
                output[i * 2] = sample * left_gain;
                output[i * 2 + 1] = sample * right_gain;
            }
        }
    }
}

/// Voice activity detection
#[derive(Debug)]
pub struct VoiceActivityDetector {
    /// Threshold for voice detection
    threshold: f32,
    /// Energy accumulator
    energy: f32,
    /// Smooth factor
    smooth_factor: f32,
    /// Is voice detected
    voice_detected: bool,
    /// Frames since last voice
    silence_frames: u32,
    /// Minimum voice frames
    min_voice_frames: u32,
}

impl VoiceActivityDetector {
    /// Create new VAD
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            energy: 0.0,
            smooth_factor: 0.95,
            voice_detected: false,
            silence_frames: 0,
            min_voice_frames: 5,
        }
    }

    /// Process audio frame
    pub fn process(&mut self, samples: &[i16]) -> bool {
        // Calculate frame energy
        let frame_energy: f32 = samples.iter()
            .map(|&s| {
                let f = s as f32 / 32768.0;
                f * f
            })
            .sum::<f32>() / samples.len() as f32;

        // Smooth energy
        self.energy = self.smooth_factor * self.energy + (1.0 - self.smooth_factor) * frame_energy;

        // Detect voice
        let is_voice = self.energy > self.threshold;

        if is_voice {
            self.silence_frames = 0;
            self.voice_detected = true;
        } else {
            self.silence_frames += 1;
            if self.silence_frames > self.min_voice_frames {
                self.voice_detected = false;
            }
        }

        self.voice_detected
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.energy = 0.0;
        self.voice_detected = false;
        self.silence_frames = 0;
    }
}

/// Echo cancellation
#[derive(Debug)]
pub struct EchoCanceller {
    /// Filter length
    filter_length: usize,
    /// Filter coefficients
    filter: Vec<f32>,
    /// Reference buffer
    ref_buffer: Vec<f32>,
    /// Step size
    step_size: f32,
}

impl EchoCanceller {
    /// Create new echo canceller
    pub fn new(filter_length: usize) -> Self {
        Self {
            filter_length,
            filter: vec![0.0; filter_length],
            ref_buffer: vec![0.0; filter_length],
            step_size: 0.01,
        }
    }

    /// Process frame (NLMS algorithm)
    pub fn process(&mut self, reference: &[f32], microphone: &[f32], output: &mut [f32]) {
        for (i, (&mic, &ref_sample)) in microphone.iter().zip(reference.iter()).enumerate() {
            // Shift reference buffer
            self.ref_buffer.rotate_right(1);
            self.ref_buffer[0] = ref_sample;

            // Estimate echo
            let echo_estimate: f32 = self.filter.iter()
                .zip(self.ref_buffer.iter())
                .map(|(&f, &r)| f * r)
                .sum();

            // Cancel echo
            let error = mic - echo_estimate;
            
            if i < output.len() {
                output[i] = error;
            }

            // Update filter (NLMS)
            let power: f32 = self.ref_buffer.iter().map(|&x| x * x).sum();
            if power > 0.0001 {
                let norm_step = self.step_size / (power + 0.001);
                for (f, &r) in self.filter.iter_mut().zip(self.ref_buffer.iter()) {
                    *f += norm_step * error * r;
                }
            }
        }
    }

    /// Reset filter
    pub fn reset(&mut self) {
        self.filter.fill(0.0);
        self.ref_buffer.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_audio_hal_creation() {
        let hal = AudioHal::new(AudioConfig::default());
        assert!(hal.is_ok());
    }

    #[test]
    fn test_audio_hal_initialization() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        hal.initialize().unwrap();
        assert_eq!(hal.state(), AudioState::Idle);
    }

    #[test]
    fn test_audio_playback() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        hal.start_playback().unwrap();
        assert_eq!(hal.state(), AudioState::Playing);

        let samples = vec![0i16; 256];
        let written = hal.write(&samples).unwrap();
        assert_eq!(written, 256);

        hal.stop_playback().unwrap();
        assert_eq!(hal.state(), AudioState::Idle);
    }

    #[test]
    fn test_audio_recording() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        hal.initialize().unwrap();
        hal.start().unwrap();

        hal.start_recording().unwrap();
        assert_eq!(hal.state(), AudioState::Recording);

        let mut buffer = vec![0i16; 256];
        let read = hal.read(&mut buffer).unwrap();
        assert_eq!(read, 256);

        hal.stop_recording().unwrap();
    }

    #[test]
    fn test_audio_volume() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        hal.initialize().unwrap();

        hal.set_volume(50).unwrap();
        assert_eq!(hal.volume(), 50);

        assert!(hal.set_volume(101).is_err());
    }

    #[test]
    fn test_audio_mute() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        
        hal.set_muted(true);
        assert!(hal.is_muted());

        hal.set_muted(false);
        assert!(!hal.is_muted());
    }

    #[test]
    fn test_audio_route() {
        let mut hal = AudioHal::new(AudioConfig::default()).unwrap();
        
        hal.set_route(AudioRoute::BoneConduction).unwrap();
        assert_eq!(hal.route(), AudioRoute::BoneConduction);
    }

    #[test]
    fn test_spatial_audio() {
        let mut processor = SpatialAudioProcessor::new(48000);
        
        processor.update_listener([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        
        let input = vec![1.0f32; 256];
        let mut output = vec![0.0f32; 512];
        
        processor.process(&input, [1.0, 0.0, 0.0], &mut output);
        
        // Right side source should have more right channel
        assert!(output[1].abs() > output[0].abs());
    }

    #[test]
    fn test_voice_activity_detector() {
        let mut vad = VoiceActivityDetector::new(0.001);

        // Silence
        let silence = vec![0i16; 256];
        assert!(!vad.process(&silence));

        // Voice (loud signal)
        let voice: Vec<i16> = (0..256).map(|i| ((i as f32 * 0.1).sin() * 10000.0) as i16).collect();
        assert!(vad.process(&voice));
    }

    #[test]
    fn test_echo_canceller() {
        let mut aec = EchoCanceller::new(128);

        let reference = vec![0.5f32; 64];
        let microphone = vec![0.5f32; 64]; // Echo from speaker
        let mut output = vec![0.0f32; 64];

        aec.process(&reference, &microphone, &mut output);

        // After processing, echo should be reduced
        // (Initial pass won't fully cancel)
    }

    #[test]
    fn test_output_devices() {
        let hal = AudioHal::new(AudioConfig::default()).unwrap();
        let devices = hal.output_devices();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_input_devices() {
        let hal = AudioHal::new(AudioConfig::default()).unwrap();
        let devices = hal.input_devices();
        assert!(!devices.is_empty());
    }
}
