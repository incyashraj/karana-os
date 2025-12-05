// Kāraṇa OS - Hardware Abstraction Layer (HAL)
// Unified interface for smart glasses hardware

pub mod display;
pub mod camera;
pub mod sensors;
pub mod audio;
pub mod power;
pub mod connectivity;
pub mod input;

pub use display::{Display, DisplayConfig, DisplayMode, DisplayCapabilities};
pub use camera::{Camera, CameraConfig, CameraPosition};
pub use sensors::{SensorHub, SensorConfig, SensorType, SensorData, SensorPowerMode};
pub use audio::{AudioHal, AudioConfig, AudioRoute};
pub use power::{PowerHal, PowerState, BatteryStatus};
pub use connectivity::{ConnectivityHal, WifiState, BluetoothState, CellularState};
pub use input::{InputHal, InputConfig, GestureType, GestureEvent, TouchEvent};

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// HAL error types
#[derive(Debug, Clone)]
pub enum HalError {
    /// Device not found
    DeviceNotFound(String),
    /// Device open failed
    DeviceOpenFailed(String),
    /// Device busy
    DeviceBusy,
    /// Operation not supported
    NotSupported,
    /// Permission denied
    PermissionDenied,
    /// I/O error
    IoError(String),
    /// Configuration error
    ConfigError(String),
    /// Timeout
    Timeout,
    /// Hardware fault
    HardwareFault(String),
    /// Calibration needed
    CalibrationNeeded,
    /// Resource exhausted
    ResourceExhausted,
}

impl std::fmt::Display for HalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HalError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            HalError::DeviceOpenFailed(s) => write!(f, "Failed to open device: {}", s),
            HalError::DeviceBusy => write!(f, "Device is busy"),
            HalError::NotSupported => write!(f, "Operation not supported"),
            HalError::PermissionDenied => write!(f, "Permission denied"),
            HalError::IoError(s) => write!(f, "I/O error: {}", s),
            HalError::ConfigError(s) => write!(f, "Configuration error: {}", s),
            HalError::Timeout => write!(f, "Operation timed out"),
            HalError::HardwareFault(s) => write!(f, "Hardware fault: {}", s),
            HalError::CalibrationNeeded => write!(f, "Calibration needed"),
            HalError::ResourceExhausted => write!(f, "Resource exhausted"),
        }
    }
}

impl std::error::Error for HalError {}

/// HAL version information
#[derive(Debug, Clone)]
pub struct HalVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Build info
    pub build: String,
}

impl HalVersion {
    pub fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
            build: "karana-hal-2024".into(),
        }
    }
}

/// Device capabilities
#[derive(Debug, Clone, Default)]
pub struct DeviceCapabilities {
    /// Display capabilities
    pub display: Option<DisplayCapabilities>,
    /// Number of cameras
    pub cameras: u32,
    /// Available sensors
    pub sensors: Vec<SensorType>,
    /// Has audio output
    pub has_audio_output: bool,
    /// Has audio input (microphone)
    pub has_audio_input: bool,
    /// Has haptic feedback
    pub has_haptics: bool,
    /// Has GPS
    pub has_gps: bool,
    /// Has cellular
    pub has_cellular: bool,
    /// Has WiFi
    pub has_wifi: bool,
    /// Has Bluetooth
    pub has_bluetooth: bool,
    /// Has NFC
    pub has_nfc: bool,
    /// Has eye tracking
    pub has_eye_tracking: bool,
    /// Has hand tracking
    pub has_hand_tracking: bool,
}

/// HAL configuration
#[derive(Debug, Clone)]
pub struct HalConfig {
    /// Enable power management
    pub power_management: bool,
    /// Low power threshold (%)
    pub low_power_threshold: u8,
    /// Sensor polling interval (ms)
    pub sensor_poll_ms: u32,
    /// Enable hardware watchdog
    pub watchdog_enabled: bool,
    /// Watchdog timeout (ms)
    pub watchdog_timeout_ms: u32,
    /// Enable thermal management
    pub thermal_management: bool,
    /// Thermal throttle temperature (°C)
    pub thermal_throttle_temp: f32,
}

impl Default for HalConfig {
    fn default() -> Self {
        Self {
            power_management: true,
            low_power_threshold: 20,
            sensor_poll_ms: 10,
            watchdog_enabled: true,
            watchdog_timeout_ms: 5000,
            thermal_management: true,
            thermal_throttle_temp: 40.0,
        }
    }
}

/// HAL state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalState {
    /// Not initialized
    Uninitialized,
    /// Initializing
    Initializing,
    /// Ready
    Ready,
    /// Running
    Running,
    /// Suspended
    Suspended,
    /// Error state
    Error,
}

/// Main Hardware Abstraction Layer
#[derive(Debug)]
pub struct Hal {
    /// Configuration
    config: HalConfig,
    /// Current state
    state: HalState,
    /// Device capabilities
    capabilities: DeviceCapabilities,
    /// Display subsystem
    pub display: Display,
    /// Camera subsystem
    pub camera: Camera,
    /// Sensor hub
    pub sensors: SensorHub,
    /// Audio HAL
    pub audio: AudioHal,
    /// Power management
    pub power: PowerHal,
    /// Input HAL
    pub input: InputHal,
    /// Statistics
    stats: HalStats,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Start time
    start_time: Option<Instant>,
}

/// HAL statistics
#[derive(Debug, Default)]
pub struct HalStats {
    /// Total uptime (seconds)
    pub uptime_secs: f64,
    /// Sensor reads
    pub sensor_reads: u64,
    /// Display frames
    pub display_frames: u64,
    /// Audio samples
    pub audio_samples: u64,
    /// Input events
    pub input_events: u64,
    /// Errors encountered
    pub errors: u64,
}

impl Hal {
    /// Create new HAL instance
    pub fn new(config: HalConfig) -> Result<Self, HalError> {
        Ok(Self {
            config: config.clone(),
            state: HalState::Uninitialized,
            capabilities: DeviceCapabilities::default(),
            display: Display::new(DisplayConfig::default())?,
            camera: Camera::new(CameraConfig::default())?,
            sensors: SensorHub::new(SensorConfig::default())?,
            audio: AudioHal::new(AudioConfig::default())?,
            power: PowerHal::new(power::PowerConfig::default())?,
            input: InputHal::new(input::InputConfig::default())?,
            stats: HalStats::default(),
            running: Arc::new(AtomicBool::new(false)),
            start_time: None,
        })
    }

    /// Initialize HAL
    pub fn initialize(&mut self) -> Result<(), HalError> {
        self.state = HalState::Initializing;

        // Detect device capabilities
        self.detect_capabilities()?;

        // Initialize subsystems
        self.display.initialize()?;
        self.camera.initialize()?;
        self.sensors.initialize()?;
        self.audio.initialize()?;
        self.power.initialize()?;
        self.input.initialize()?;

        self.state = HalState::Ready;
        Ok(())
    }

    /// Start HAL
    pub fn start(&mut self) -> Result<(), HalError> {
        if self.state != HalState::Ready && self.state != HalState::Suspended {
            return Err(HalError::ConfigError("HAL not ready".into()));
        }

        self.running.store(true, Ordering::SeqCst);
        self.start_time = Some(Instant::now());
        self.state = HalState::Running;

        // Start subsystems
        self.display.start()?;
        self.sensors.start()?;
        self.audio.start()?;
        self.input.start()?;

        Ok(())
    }

    /// Stop HAL
    pub fn stop(&mut self) -> Result<(), HalError> {
        self.running.store(false, Ordering::SeqCst);

        // Stop subsystems
        self.input.stop()?;
        self.audio.stop()?;
        self.sensors.stop()?;
        self.display.stop()?;

        self.state = HalState::Ready;
        Ok(())
    }

    /// Suspend HAL (low power)
    pub fn suspend(&mut self) -> Result<(), HalError> {
        self.state = HalState::Suspended;
        
        // Put subsystems in low power mode
        self.display.suspend()?;
        self.sensors.set_power_mode(sensors::SensorPowerMode::Low)?;
        self.audio.suspend()?;

        Ok(())
    }

    /// Resume from suspend
    pub fn resume(&mut self) -> Result<(), HalError> {
        // Resume subsystems
        self.display.resume()?;
        self.sensors.set_power_mode(sensors::SensorPowerMode::Normal)?;
        self.audio.resume()?;

        self.state = HalState::Running;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> HalState {
        self.state
    }

    /// Get device capabilities
    pub fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    /// Get statistics
    pub fn stats(&self) -> HalStats {
        let mut stats = HalStats {
            sensor_reads: self.sensors.stats().reads,
            display_frames: self.display.stats().frames_rendered,
            audio_samples: self.audio.stats().samples_processed,
            input_events: self.input.stats().touch_events + self.input.stats().button_presses,
            errors: self.stats.errors,
            ..Default::default()
        };

        if let Some(start) = self.start_time {
            stats.uptime_secs = start.elapsed().as_secs_f64();
        }

        stats
    }

    /// Detect device capabilities
    fn detect_capabilities(&mut self) -> Result<(), HalError> {
        self.capabilities = DeviceCapabilities {
            display: Some(self.display.capabilities()),
            cameras: 2, // Front + World cameras
            sensors: vec![
                SensorType::Accelerometer,
                SensorType::Gyroscope,
                SensorType::Magnetometer,
                SensorType::Barometer,
                SensorType::AmbientLight,
                SensorType::Proximity,
                SensorType::Temperature,
            ],
            has_audio_output: true,
            has_audio_input: true,
            has_haptics: true,
            has_gps: true,
            has_cellular: false,
            has_wifi: true,
            has_bluetooth: true,
            has_nfc: false,
            has_eye_tracking: true,
            has_hand_tracking: true,
        };

        Ok(())
    }

    /// Get HAL version
    pub fn version(&self) -> HalVersion {
        HalVersion::current()
    }

    /// Perform self-test
    pub fn self_test(&mut self) -> Result<SelfTestResult, HalError> {
        let mut result = SelfTestResult::default();

        // Test display
        result.display = self.display.test().is_ok();

        // Test camera
        result.camera = self.camera.test().is_ok();

        // Test sensors
        result.sensors = self.sensors.test().is_ok();

        // Test audio
        result.audio = self.audio.test().is_ok();

        // Test power
        result.power = self.power.test().is_ok();

        result.passed = result.display && result.camera && result.sensors 
            && result.audio && result.power;

        Ok(result)
    }

    /// Reset HAL
    pub fn reset(&mut self) -> Result<(), HalError> {
        self.stop()?;
        self.state = HalState::Uninitialized;
        self.stats = HalStats::default();
        self.start_time = None;
        self.initialize()?;
        Ok(())
    }
}

/// Self-test results
#[derive(Debug, Default)]
pub struct SelfTestResult {
    /// Overall pass
    pub passed: bool,
    /// Display test
    pub display: bool,
    /// Camera test
    pub camera: bool,
    /// Sensors test
    pub sensors: bool,
    /// Audio test
    pub audio: bool,
    /// Power test
    pub power: bool,
    /// Detailed messages
    pub messages: Vec<String>,
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Manufacturer
    pub manufacturer: String,
    /// Model name
    pub model: String,
    /// Serial number
    pub serial: String,
    /// Hardware revision
    pub hw_revision: String,
    /// Firmware version
    pub fw_version: String,
}

impl DeviceInfo {
    pub fn detect() -> Self {
        Self {
            manufacturer: "Kāraṇa".into(),
            model: "Smart Glasses v1".into(),
            serial: format!("SG{:08X}", rand_u32()),
            hw_revision: "1.0".into(),
            fw_version: "1.0.0".into(),
        }
    }
}

/// Interrupt handler trait
pub trait InterruptHandler: Send + Sync {
    fn handle(&self, irq: u32, data: &[u8]);
}

/// DMA buffer for hardware transfers
#[derive(Debug)]
pub struct DmaBuffer {
    /// Physical address
    pub phys_addr: u64,
    /// Virtual address
    pub virt_addr: *mut u8,
    /// Buffer size
    pub size: usize,
    /// Is coherent (no cache flushing needed)
    pub coherent: bool,
}

impl DmaBuffer {
    /// Create new DMA buffer
    pub fn new(size: usize, coherent: bool) -> Result<Self, HalError> {
        Ok(Self {
            phys_addr: rand_u64() & 0xFFFF_FFFF_FFF0, // Page aligned
            virt_addr: std::ptr::null_mut(),
            size,
            coherent,
        })
    }

    /// Sync for device access
    pub fn sync_for_device(&self) {
        // Would flush CPU cache in real implementation
    }

    /// Sync for CPU access
    pub fn sync_for_cpu(&self) {
        // Would invalidate CPU cache in real implementation
    }
}

/// Simple random number helpers
fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u32;
    seed.wrapping_mul(1103515245).wrapping_add(12345)
}

fn rand_u64() -> u64 {
    let lo = rand_u32() as u64;
    let hi = rand_u32() as u64;
    (hi << 32) | lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hal_version() {
        let version = HalVersion::current();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
    }

    #[test]
    fn test_hal_config_default() {
        let config = HalConfig::default();
        assert!(config.power_management);
        assert_eq!(config.low_power_threshold, 20);
        assert!(config.watchdog_enabled);
    }

    #[test]
    fn test_hal_error_display() {
        let error = HalError::DeviceNotFound("test".into());
        assert!(error.to_string().contains("test"));
    }

    #[test]
    fn test_device_info() {
        let info = DeviceInfo::detect();
        assert_eq!(info.manufacturer, "Kāraṇa");
        assert!(info.serial.starts_with("SG"));
    }

    #[test]
    fn test_dma_buffer() {
        let buffer = DmaBuffer::new(4096, true).unwrap();
        assert_eq!(buffer.size, 4096);
        assert!(buffer.coherent);
    }
}
