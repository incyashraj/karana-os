// Kāraṇa OS - Device Drivers
// Low-level hardware drivers for smart glasses

pub mod display_driver;
pub mod camera_driver;
pub mod sensor_driver;
pub mod audio_driver;
pub mod connectivity_driver;
pub mod input_driver;

pub use display_driver::{DisplayDriver, DisplayDriverConfig};
pub use camera_driver::{CameraDriver, CameraDriverConfig};
pub use sensor_driver::{SensorDriver, SensorDriverConfig};
pub use audio_driver::{AudioDriver, AudioDriverConfig};
pub use connectivity_driver::{ConnectivityDriver, ConnectivityDriverConfig};
pub use input_driver::{InputDriver, InputDriverConfig};

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Driver error types
#[derive(Debug, Clone)]
pub enum DriverError {
    /// Device not found
    DeviceNotFound(String),
    /// Failed to open device
    OpenFailed(String),
    /// I/O error
    IoError(String),
    /// Device disconnected
    Disconnected,
    /// Timeout
    Timeout,
    /// Invalid parameter
    InvalidParameter(String),
    /// Resource busy
    Busy,
    /// Permission denied
    PermissionDenied,
    /// Driver not loaded
    NotLoaded,
    /// Hardware error
    HardwareError(String),
    /// Buffer overflow
    BufferOverflow,
    /// Protocol error
    ProtocolError(String),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            DriverError::OpenFailed(s) => write!(f, "Failed to open device: {}", s),
            DriverError::IoError(s) => write!(f, "I/O error: {}", s),
            DriverError::Disconnected => write!(f, "Device disconnected"),
            DriverError::Timeout => write!(f, "Operation timed out"),
            DriverError::InvalidParameter(s) => write!(f, "Invalid parameter: {}", s),
            DriverError::Busy => write!(f, "Device busy"),
            DriverError::PermissionDenied => write!(f, "Permission denied"),
            DriverError::NotLoaded => write!(f, "Driver not loaded"),
            DriverError::HardwareError(s) => write!(f, "Hardware error: {}", s),
            DriverError::BufferOverflow => write!(f, "Buffer overflow"),
            DriverError::ProtocolError(s) => write!(f, "Protocol error: {}", s),
        }
    }
}

impl std::error::Error for DriverError {}

/// Driver state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    /// Not loaded
    Unloaded,
    /// Loading
    Loading,
    /// Loaded but not initialized
    Loaded,
    /// Initialized and ready
    Ready,
    /// Running
    Running,
    /// Suspended
    Suspended,
    /// Error state
    Error,
}

/// Driver info
#[derive(Debug, Clone)]
pub struct DriverInfo {
    /// Driver name
    pub name: String,
    /// Driver version
    pub version: String,
    /// Vendor name
    pub vendor: String,
    /// Supported device IDs
    pub device_ids: Vec<String>,
    /// Is loaded
    pub loaded: bool,
    /// Current state
    pub state: DriverState,
}

/// Driver statistics
#[derive(Debug, Clone, Default)]
pub struct DriverStats {
    /// Bytes read
    pub bytes_read: u64,
    /// Bytes written
    pub bytes_written: u64,
    /// Interrupts handled
    pub interrupts: u64,
    /// Errors
    pub errors: u64,
    /// DMA transfers
    pub dma_transfers: u64,
    /// Average latency (us)
    pub avg_latency_us: f64,
}

/// Driver trait
pub trait Driver: Send + Sync {
    /// Get driver info
    fn info(&self) -> DriverInfo;
    
    /// Get driver state
    fn state(&self) -> DriverState;
    
    /// Load driver
    fn load(&mut self) -> Result<(), DriverError>;
    
    /// Unload driver
    fn unload(&mut self) -> Result<(), DriverError>;
    
    /// Initialize driver
    fn init(&mut self) -> Result<(), DriverError>;
    
    /// Start driver
    fn start(&mut self) -> Result<(), DriverError>;
    
    /// Stop driver
    fn stop(&mut self) -> Result<(), DriverError>;
    
    /// Suspend driver (low power)
    fn suspend(&mut self) -> Result<(), DriverError>;
    
    /// Resume driver
    fn resume(&mut self) -> Result<(), DriverError>;
    
    /// Get statistics
    fn stats(&self) -> DriverStats;
    
    /// Self-test
    fn test(&self) -> Result<(), DriverError>;
}

/// Device descriptor
#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    /// Device path (e.g., /dev/video0)
    pub path: String,
    /// Device type
    pub device_type: String,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Serial number
    pub serial: Option<String>,
    /// Bus info (USB, I2C, SPI, etc.)
    pub bus_type: BusType,
    /// Is available
    pub available: bool,
}

/// Bus type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    /// USB
    Usb,
    /// I2C
    I2c,
    /// SPI
    Spi,
    /// PCIe
    Pcie,
    /// MIPI CSI
    MipiCsi,
    /// MIPI DSI
    MipiDsi,
    /// I2S
    I2s,
    /// GPIO
    Gpio,
    /// Platform
    Platform,
    /// Virtual
    Virtual,
}

/// Register map entry
#[derive(Debug, Clone)]
pub struct RegisterEntry {
    /// Register address
    pub address: u32,
    /// Register name
    pub name: String,
    /// Register size (bytes)
    pub size: u8,
    /// Is read-only
    pub read_only: bool,
    /// Default value
    pub default: u32,
}

/// DMA buffer
#[derive(Debug)]
pub struct DmaBuffer {
    /// Buffer data
    pub data: Vec<u8>,
    /// Physical address (for real DMA)
    pub phys_addr: u64,
    /// Size
    pub size: usize,
    /// Is coherent
    pub coherent: bool,
}

impl DmaBuffer {
    /// Create new DMA buffer
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            phys_addr: 0, // Would be allocated by kernel
            size,
            coherent: true,
        }
    }

    /// Get buffer slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Interrupt handler
pub type InterruptHandler = Box<dyn Fn(u32) + Send + Sync>;

/// Device Manager
pub struct DeviceManager {
    /// Loaded drivers
    drivers: HashMap<String, Box<dyn Driver>>,
    /// Detected devices
    devices: Vec<DeviceDescriptor>,
    /// Driver-device bindings
    bindings: HashMap<String, String>, // device path -> driver name
    /// Statistics
    stats: DriverStats,
    /// Is running
    running: AtomicBool,
}

impl std::fmt::Debug for DeviceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceManager")
            .field("drivers", &self.drivers.keys().collect::<Vec<_>>())
            .field("devices", &self.devices)
            .field("bindings", &self.bindings)
            .field("stats", &self.stats)
            .field("running", &self.running)
            .finish()
    }
}

impl DeviceManager {
    /// Create new device manager
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            devices: Vec::new(),
            bindings: HashMap::new(),
            stats: DriverStats::default(),
            running: AtomicBool::new(false),
        }
    }

    /// Scan for devices
    pub fn scan_devices(&mut self) -> Result<Vec<DeviceDescriptor>, DriverError> {
        // In real implementation, would scan /dev, /sys, etc.
        let devices = vec![
            DeviceDescriptor {
                path: "/dev/video0".into(),
                device_type: "camera".into(),
                vendor_id: 0x1234,
                product_id: 0x5678,
                serial: Some("CAM001".into()),
                bus_type: BusType::MipiCsi,
                available: true,
            },
            DeviceDescriptor {
                path: "/dev/fb0".into(),
                device_type: "display".into(),
                vendor_id: 0x2345,
                product_id: 0x6789,
                serial: None,
                bus_type: BusType::MipiDsi,
                available: true,
            },
            DeviceDescriptor {
                path: "/dev/iio:device0".into(),
                device_type: "imu".into(),
                vendor_id: 0x0461, // ST Micro
                product_id: 0x6A20,
                serial: None,
                bus_type: BusType::I2c,
                available: true,
            },
            DeviceDescriptor {
                path: "/dev/snd/pcmC0D0p".into(),
                device_type: "audio_playback".into(),
                vendor_id: 0x1234,
                product_id: 0xABCD,
                serial: None,
                bus_type: BusType::I2s,
                available: true,
            },
        ];

        self.devices = devices.clone();
        Ok(devices)
    }

    /// Load a driver
    pub fn load_driver(&mut self, name: &str, driver: Box<dyn Driver>) -> Result<(), DriverError> {
        self.drivers.insert(name.to_string(), driver);
        if let Some(drv) = self.drivers.get_mut(name) {
            drv.load()?;
        }
        Ok(())
    }

    /// Bind device to driver
    pub fn bind(&mut self, device_path: &str, driver_name: &str) -> Result<(), DriverError> {
        if !self.drivers.contains_key(driver_name) {
            return Err(DriverError::NotLoaded);
        }

        self.bindings.insert(device_path.to_string(), driver_name.to_string());
        Ok(())
    }

    /// Get driver for device
    pub fn get_driver(&self, device_path: &str) -> Option<&str> {
        self.bindings.get(device_path).map(|s| s.as_str())
    }

    /// Initialize all loaded drivers
    pub fn init_all(&mut self) -> Result<(), DriverError> {
        for driver in self.drivers.values_mut() {
            driver.init()?;
        }
        Ok(())
    }

    /// Start all drivers
    pub fn start_all(&mut self) -> Result<(), DriverError> {
        self.running.store(true, Ordering::Relaxed);
        for driver in self.drivers.values_mut() {
            driver.start()?;
        }
        Ok(())
    }

    /// Stop all drivers
    pub fn stop_all(&mut self) -> Result<(), DriverError> {
        self.running.store(false, Ordering::Relaxed);
        for driver in self.drivers.values_mut() {
            driver.stop()?;
        }
        Ok(())
    }

    /// Get all devices
    pub fn devices(&self) -> &[DeviceDescriptor] {
        &self.devices
    }

    /// Get driver names
    pub fn driver_names(&self) -> Vec<&str> {
        self.drivers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// I2C device abstraction
#[derive(Debug)]
pub struct I2cDevice {
    /// Device path
    path: String,
    /// Device address
    address: u8,
    /// Is open
    open: bool,
}

impl I2cDevice {
    /// Create new I2C device
    pub fn new(bus: u8, address: u8) -> Self {
        Self {
            path: format!("/dev/i2c-{}", bus),
            address,
            open: false,
        }
    }

    /// Open device
    pub fn open(&mut self) -> Result<(), DriverError> {
        // Would use ioctl to set I2C_SLAVE address
        self.open = true;
        Ok(())
    }

    /// Close device
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Read register
    pub fn read_reg(&self, reg: u8) -> Result<u8, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        // Simulated read
        Ok(0)
    }

    /// Write register
    pub fn write_reg(&self, reg: u8, value: u8) -> Result<(), DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        // Simulated write
        Ok(())
    }

    /// Read multiple bytes
    pub fn read_bytes(&self, reg: u8, buf: &mut [u8]) -> Result<usize, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        Ok(buf.len())
    }

    /// Write multiple bytes
    pub fn write_bytes(&self, reg: u8, data: &[u8]) -> Result<usize, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        Ok(data.len())
    }
}

/// SPI device abstraction
#[derive(Debug)]
pub struct SpiDevice {
    /// Device path
    path: String,
    /// SPI mode
    mode: u8,
    /// Speed (Hz)
    speed: u32,
    /// Bits per word
    bits_per_word: u8,
    /// Is open
    open: bool,
}

impl SpiDevice {
    /// Create new SPI device
    pub fn new(bus: u8, cs: u8) -> Self {
        Self {
            path: format!("/dev/spidev{}.{}", bus, cs),
            mode: 0,
            speed: 1_000_000,
            bits_per_word: 8,
            open: false,
        }
    }

    /// Open device
    pub fn open(&mut self) -> Result<(), DriverError> {
        self.open = true;
        Ok(())
    }

    /// Close device
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Set SPI mode
    pub fn set_mode(&mut self, mode: u8) -> Result<(), DriverError> {
        if mode > 3 {
            return Err(DriverError::InvalidParameter("Mode must be 0-3".into()));
        }
        self.mode = mode;
        Ok(())
    }

    /// Set speed
    pub fn set_speed(&mut self, hz: u32) -> Result<(), DriverError> {
        self.speed = hz;
        Ok(())
    }

    /// Transfer (simultaneous read/write)
    pub fn transfer(&self, tx: &[u8], rx: &mut [u8]) -> Result<usize, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        let len = tx.len().min(rx.len());
        Ok(len)
    }

    /// Write only
    pub fn write(&self, data: &[u8]) -> Result<usize, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        Ok(data.len())
    }

    /// Read only
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, DriverError> {
        if !self.open {
            return Err(DriverError::NotLoaded);
        }
        Ok(buf.len())
    }
}

/// GPIO abstraction
#[derive(Debug)]
pub struct GpioPin {
    /// Pin number
    pin: u32,
    /// Direction (true = output)
    output: bool,
    /// Current value
    value: bool,
    /// Is exported
    exported: bool,
}

impl GpioPin {
    /// Create new GPIO pin
    pub fn new(pin: u32) -> Self {
        Self {
            pin,
            output: false,
            value: false,
            exported: false,
        }
    }

    /// Export pin
    pub fn export(&mut self) -> Result<(), DriverError> {
        self.exported = true;
        Ok(())
    }

    /// Unexport pin
    pub fn unexport(&mut self) -> Result<(), DriverError> {
        self.exported = false;
        Ok(())
    }

    /// Set direction
    pub fn set_direction(&mut self, output: bool) -> Result<(), DriverError> {
        if !self.exported {
            return Err(DriverError::NotLoaded);
        }
        self.output = output;
        Ok(())
    }

    /// Set value (for output)
    pub fn set_value(&mut self, high: bool) -> Result<(), DriverError> {
        if !self.exported || !self.output {
            return Err(DriverError::InvalidParameter("Not an output".into()));
        }
        self.value = high;
        Ok(())
    }

    /// Get value
    pub fn get_value(&self) -> Result<bool, DriverError> {
        if !self.exported {
            return Err(DriverError::NotLoaded);
        }
        Ok(self.value)
    }

    /// Wait for edge
    pub fn wait_for_edge(&self, _timeout_ms: u32) -> Result<bool, DriverError> {
        if !self.exported {
            return Err(DriverError::NotLoaded);
        }
        Ok(true) // Would use poll/epoll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_manager_creation() {
        let manager = DeviceManager::new();
        assert!(manager.devices().is_empty());
    }

    #[test]
    fn test_device_scan() {
        let mut manager = DeviceManager::new();
        let devices = manager.scan_devices().unwrap();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_i2c_device() {
        let mut dev = I2cDevice::new(1, 0x50);
        dev.open().unwrap();
        
        let val = dev.read_reg(0x00).unwrap();
        assert_eq!(val, 0);
        
        dev.write_reg(0x00, 0xFF).unwrap();
        dev.close();
    }

    #[test]
    fn test_spi_device() {
        let mut dev = SpiDevice::new(0, 0);
        dev.open().unwrap();
        
        dev.set_mode(0).unwrap();
        dev.set_speed(10_000_000).unwrap();
        
        let tx = [0x00, 0x01, 0x02];
        let mut rx = [0u8; 3];
        let len = dev.transfer(&tx, &mut rx).unwrap();
        assert_eq!(len, 3);
        
        dev.close();
    }

    #[test]
    fn test_gpio_pin() {
        let mut pin = GpioPin::new(17);
        pin.export().unwrap();
        pin.set_direction(true).unwrap();
        
        pin.set_value(true).unwrap();
        assert!(pin.get_value().unwrap());
        
        pin.set_value(false).unwrap();
        assert!(!pin.get_value().unwrap());
        
        pin.unexport().unwrap();
    }

    #[test]
    fn test_dma_buffer() {
        let mut buf = DmaBuffer::new(4096);
        assert_eq!(buf.size, 4096);
        assert_eq!(buf.as_slice().len(), 4096);
        
        buf.as_mut_slice()[0] = 0xFF;
        assert_eq!(buf.as_slice()[0], 0xFF);
    }

    #[test]
    fn test_driver_error_display() {
        let err = DriverError::DeviceNotFound("test".into());
        assert!(err.to_string().contains("test"));
    }
}
