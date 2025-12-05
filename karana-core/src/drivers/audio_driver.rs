// Kāraṇa OS - Audio Driver
// Low-level driver for smart glasses audio hardware

use super::{Driver, DriverError, DriverInfo, DriverState, DriverStats, I2cDevice, DmaBuffer};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::Instant;
use std::collections::VecDeque;

/// Audio driver configuration
#[derive(Debug, Clone)]
pub struct AudioDriverConfig {
    /// Device path
    pub device_path: String,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Channels
    pub channels: u8,
    /// Bits per sample
    pub bits_per_sample: u8,
    /// Period size (frames)
    pub period_size: u32,
    /// Number of periods
    pub periods: u32,
    /// Codec I2C address
    pub codec_i2c_addr: u8,
    /// Codec I2C bus
    pub codec_i2c_bus: u8,
    /// Enable playback
    pub playback_enabled: bool,
    /// Enable capture
    pub capture_enabled: bool,
}

impl Default for AudioDriverConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/snd/pcmC0D0p".into(),
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 16,
            period_size: 256,
            periods: 4,
            codec_i2c_addr: 0x1A,
            codec_i2c_bus: 1,
            playback_enabled: true,
            capture_enabled: true,
        }
    }
}

/// Audio stream state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Not opened
    Closed,
    /// Setup complete
    Setup,
    /// Prepared
    Prepared,
    /// Running
    Running,
    /// Paused
    Paused,
    /// Buffer underrun (xrun)
    Xrun,
}

/// Audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// Signed 16-bit little-endian
    S16Le,
    /// Signed 24-bit little-endian
    S24Le,
    /// Signed 32-bit little-endian
    S32Le,
    /// Float 32-bit little-endian
    FloatLe,
}

impl AudioFormat {
    pub fn bytes_per_sample(&self) -> u8 {
        match self {
            AudioFormat::S16Le => 2,
            AudioFormat::S24Le => 3,
            AudioFormat::S32Le | AudioFormat::FloatLe => 4,
        }
    }
}

/// Audio buffer
#[derive(Debug)]
pub struct AudioBuffer {
    /// Buffer data
    pub data: DmaBuffer,
    /// Frames in buffer
    pub frames: u32,
    /// Is queued
    pub queued: bool,
}

/// Codec register map (generic)
mod codec_regs {
    pub const CHIP_ID: u8 = 0x00;
    pub const POWER_MGMT: u8 = 0x02;
    pub const DAC_VOL_L: u8 = 0x10;
    pub const DAC_VOL_R: u8 = 0x11;
    pub const ADC_VOL_L: u8 = 0x12;
    pub const ADC_VOL_R: u8 = 0x13;
    pub const HEADPHONE_VOL: u8 = 0x20;
    pub const MIC_GAIN: u8 = 0x21;
    pub const MIXER_CTRL: u8 = 0x30;
    pub const SAMPLE_RATE: u8 = 0x40;
    pub const INTERFACE_CTRL: u8 = 0x41;
}

/// Audio driver
#[derive(Debug)]
pub struct AudioDriver {
    /// Configuration
    config: AudioDriverConfig,
    /// Current state
    state: DriverState,
    /// Playback stream state
    playback_state: StreamState,
    /// Capture stream state
    capture_state: StreamState,
    /// Codec I2C interface
    codec: Option<I2cDevice>,
    /// Playback buffers
    playback_buffers: Vec<AudioBuffer>,
    /// Capture buffers
    capture_buffers: Vec<AudioBuffer>,
    /// Playback queue
    playback_queue: VecDeque<usize>,
    /// Capture queue
    capture_queue: VecDeque<usize>,
    /// Volume (0-100)
    volume: u8,
    /// Is muted
    muted: bool,
    /// Statistics
    stats: DriverStats,
    /// Samples played
    samples_played: AtomicU64,
    /// Samples captured
    samples_captured: AtomicU64,
    /// Buffer underruns
    underruns: AtomicU64,
    /// Buffer overruns
    overruns: AtomicU64,
}

impl AudioDriver {
    /// Create new audio driver
    pub fn new(config: AudioDriverConfig) -> Self {
        Self {
            config,
            state: DriverState::Unloaded,
            playback_state: StreamState::Closed,
            capture_state: StreamState::Closed,
            codec: None,
            playback_buffers: Vec::new(),
            capture_buffers: Vec::new(),
            playback_queue: VecDeque::new(),
            capture_queue: VecDeque::new(),
            volume: 70,
            muted: false,
            stats: DriverStats::default(),
            samples_played: AtomicU64::new(0),
            samples_captured: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
        }
    }

    /// Initialize codec
    fn init_codec(&mut self) -> Result<(), DriverError> {
        let codec = self.codec.as_ref().ok_or(DriverError::NotLoaded)?;

        // Reset codec
        codec.write_reg(codec_regs::POWER_MGMT, 0x00)?;
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Power up
        codec.write_reg(codec_regs::POWER_MGMT, 0xFF)?;

        // Set sample rate
        let sr_reg = match self.config.sample_rate {
            8000 => 0x00,
            16000 => 0x01,
            32000 => 0x02,
            44100 => 0x03,
            48000 => 0x04,
            96000 => 0x05,
            _ => 0x04, // Default to 48kHz
        };
        codec.write_reg(codec_regs::SAMPLE_RATE, sr_reg)?;

        // Set interface format (I2S, 16/24/32 bit)
        let fmt_reg = match self.config.bits_per_sample {
            16 => 0x00,
            24 => 0x01,
            32 => 0x02,
            _ => 0x00,
        };
        codec.write_reg(codec_regs::INTERFACE_CTRL, fmt_reg)?;

        // Set initial volume
        self.set_hw_volume(self.volume)?;

        Ok(())
    }

    /// Set hardware volume
    fn set_hw_volume(&self, volume: u8) -> Result<(), DriverError> {
        let codec = self.codec.as_ref().ok_or(DriverError::NotLoaded)?;
        let hw_vol = if self.muted { 0 } else { (volume as u16 * 255 / 100) as u8 };
        codec.write_reg(codec_regs::DAC_VOL_L, hw_vol)?;
        codec.write_reg(codec_regs::DAC_VOL_R, hw_vol)?;
        codec.write_reg(codec_regs::HEADPHONE_VOL, hw_vol)?;
        Ok(())
    }

    /// Set volume (0-100)
    pub fn set_volume(&mut self, volume: u8) -> Result<(), DriverError> {
        self.volume = volume.min(100);
        if self.state == DriverState::Running {
            self.set_hw_volume(self.volume)?;
        }
        Ok(())
    }

    /// Get volume
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Set mute
    pub fn set_mute(&mut self, muted: bool) -> Result<(), DriverError> {
        self.muted = muted;
        if self.state == DriverState::Running {
            self.set_hw_volume(self.volume)?;
        }
        Ok(())
    }

    /// Is muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Start playback
    pub fn start_playback(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        // Queue all buffers
        for i in 0..self.playback_buffers.len() {
            self.playback_buffers[i].queued = true;
            self.playback_queue.push_back(i);
        }

        self.playback_state = StreamState::Running;
        Ok(())
    }

    /// Stop playback
    pub fn stop_playback(&mut self) -> Result<(), DriverError> {
        self.playback_state = StreamState::Setup;
        self.playback_queue.clear();
        for buf in &mut self.playback_buffers {
            buf.queued = false;
        }
        Ok(())
    }

    /// Start capture
    pub fn start_capture(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        for i in 0..self.capture_buffers.len() {
            self.capture_buffers[i].queued = true;
            self.capture_queue.push_back(i);
        }

        self.capture_state = StreamState::Running;
        Ok(())
    }

    /// Stop capture
    pub fn stop_capture(&mut self) -> Result<(), DriverError> {
        self.capture_state = StreamState::Setup;
        self.capture_queue.clear();
        for buf in &mut self.capture_buffers {
            buf.queued = false;
        }
        Ok(())
    }

    /// Write audio data (playback)
    pub fn write(&mut self, data: &[u8]) -> Result<usize, DriverError> {
        if self.playback_state != StreamState::Running {
            return Err(DriverError::NotLoaded);
        }

        if let Some(idx) = self.playback_queue.pop_front() {
            let buffer = &mut self.playback_buffers[idx];
            let copy_len = data.len().min(buffer.data.size);
            buffer.data.as_mut_slice()[..copy_len].copy_from_slice(&data[..copy_len]);
            buffer.frames = (copy_len as u32) / (self.config.channels as u32 * (self.config.bits_per_sample as u32 / 8));
            buffer.queued = true;
            self.playback_queue.push_back(idx);

            let samples = copy_len / (self.config.bits_per_sample as usize / 8);
            self.samples_played.fetch_add(samples as u64, Ordering::Relaxed);
            self.stats.bytes_written += copy_len as u64;

            Ok(copy_len)
        } else {
            self.underruns.fetch_add(1, Ordering::Relaxed);
            Err(DriverError::BufferOverflow)
        }
    }

    /// Read audio data (capture)
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        if self.capture_state != StreamState::Running {
            return Err(DriverError::NotLoaded);
        }

        if let Some(idx) = self.capture_queue.pop_front() {
            let cap_buf = &mut self.capture_buffers[idx];
            let copy_len = buffer.len().min(cap_buf.data.size);
            buffer[..copy_len].copy_from_slice(&cap_buf.data.as_slice()[..copy_len]);
            cap_buf.queued = true;
            self.capture_queue.push_back(idx);

            let samples = copy_len / (self.config.bits_per_sample as usize / 8);
            self.samples_captured.fetch_add(samples as u64, Ordering::Relaxed);
            self.stats.bytes_read += copy_len as u64;

            Ok(copy_len)
        } else {
            self.overruns.fetch_add(1, Ordering::Relaxed);
            Err(DriverError::BufferOverflow)
        }
    }

    /// Get playback state
    pub fn playback_state(&self) -> StreamState {
        self.playback_state
    }

    /// Get capture state
    pub fn capture_state(&self) -> StreamState {
        self.capture_state
    }

    /// Get samples played
    pub fn samples_played(&self) -> u64 {
        self.samples_played.load(Ordering::Relaxed)
    }

    /// Get samples captured
    pub fn samples_captured(&self) -> u64 {
        self.samples_captured.load(Ordering::Relaxed)
    }

    /// Get underrun count
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }

    /// Get overrun count
    pub fn overruns(&self) -> u64 {
        self.overruns.load(Ordering::Relaxed)
    }
}

impl Driver for AudioDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "karana-audio".into(),
            version: "1.0.0".into(),
            vendor: "KaranaOS".into(),
            device_ids: vec!["audio:alsa".into(), "audio:i2s".into()],
            loaded: self.state != DriverState::Unloaded,
            state: self.state,
        }
    }

    fn state(&self) -> DriverState {
        self.state
    }

    fn load(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Loading;

        // Open codec I2C
        let mut codec = I2cDevice::new(self.config.codec_i2c_bus, self.config.codec_i2c_addr);
        codec.open()?;
        self.codec = Some(codec);

        self.state = DriverState::Loaded;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), DriverError> {
        self.stop_playback()?;
        self.stop_capture()?;

        if let Some(ref mut codec) = self.codec {
            codec.close();
        }
        self.codec = None;
        self.playback_buffers.clear();
        self.capture_buffers.clear();
        self.state = DriverState::Unloaded;
        Ok(())
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Loaded {
            return Err(DriverError::NotLoaded);
        }

        // Initialize codec
        self.init_codec()?;

        // Allocate buffers
        let frame_size = self.config.channels as usize * (self.config.bits_per_sample as usize / 8);
        let buffer_size = self.config.period_size as usize * frame_size;

        for _ in 0..self.config.periods {
            if self.config.playback_enabled {
                self.playback_buffers.push(AudioBuffer {
                    data: DmaBuffer::new(buffer_size),
                    frames: 0,
                    queued: false,
                });
            }
            if self.config.capture_enabled {
                self.capture_buffers.push(AudioBuffer {
                    data: DmaBuffer::new(buffer_size),
                    frames: 0,
                    queued: false,
                });
            }
        }

        self.playback_state = StreamState::Setup;
        self.capture_state = StreamState::Setup;
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
        self.stop_playback()?;
        self.stop_capture()?;
        self.state = DriverState::Ready;
        Ok(())
    }

    fn suspend(&mut self) -> Result<(), DriverError> {
        self.stop_playback()?;
        self.stop_capture()?;
        // Put codec in low power mode
        if let Some(ref codec) = self.codec {
            codec.write_reg(codec_regs::POWER_MGMT, 0x00)?;
        }
        self.state = DriverState::Suspended;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), DriverError> {
        // Wake codec
        if let Some(ref codec) = self.codec {
            codec.write_reg(codec_regs::POWER_MGMT, 0xFF)?;
        }
        self.state = DriverState::Running;
        Ok(())
    }

    fn stats(&self) -> DriverStats {
        DriverStats {
            bytes_read: self.stats.bytes_read,
            bytes_written: self.stats.bytes_written,
            errors: self.underruns.load(Ordering::Relaxed) + self.overruns.load(Ordering::Relaxed),
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
    fn test_audio_driver_creation() {
        let driver = AudioDriver::new(AudioDriverConfig::default());
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_audio_driver_lifecycle() {
        let mut driver = AudioDriver::new(AudioDriverConfig::default());
        
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        assert!(!driver.playback_buffers.is_empty());
        
        driver.stop().unwrap();
        driver.unload().unwrap();
    }

    #[test]
    fn test_volume() {
        let mut driver = AudioDriver::new(AudioDriverConfig::default());
        
        driver.set_volume(50).unwrap();
        assert_eq!(driver.volume(), 50);
        
        driver.set_volume(150).unwrap();
        assert_eq!(driver.volume(), 100); // Clamped
    }

    #[test]
    fn test_mute() {
        let mut driver = AudioDriver::new(AudioDriverConfig::default());
        
        driver.set_mute(true).unwrap();
        assert!(driver.is_muted());
        
        driver.set_mute(false).unwrap();
        assert!(!driver.is_muted());
    }

    #[test]
    fn test_playback_lifecycle() {
        let mut driver = AudioDriver::new(AudioDriverConfig::default());
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        driver.start_playback().unwrap();
        assert_eq!(driver.playback_state(), StreamState::Running);
        
        driver.stop_playback().unwrap();
        assert_eq!(driver.playback_state(), StreamState::Setup);
    }

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioFormat::S16Le.bytes_per_sample(), 2);
        assert_eq!(AudioFormat::S24Le.bytes_per_sample(), 3);
        assert_eq!(AudioFormat::S32Le.bytes_per_sample(), 4);
    }

    #[test]
    fn test_driver_info() {
        let driver = AudioDriver::new(AudioDriverConfig::default());
        let info = driver.info();
        
        assert_eq!(info.name, "karana-audio");
        assert!(!info.loaded);
    }
}
