// Kāraṇa OS - Sensor Driver
// Low-level driver for smart glasses sensor hardware (IMU, etc.)

use super::{Driver, DriverError, DriverInfo, DriverState, DriverStats, I2cDevice, SpiDevice};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Sensor driver configuration
#[derive(Debug, Clone)]
pub struct SensorDriverConfig {
    /// Bus type (I2C or SPI)
    pub bus_type: SensorBusType,
    /// I2C bus number
    pub i2c_bus: u8,
    /// I2C address
    pub i2c_addr: u8,
    /// SPI bus number
    pub spi_bus: u8,
    /// SPI chip select
    pub spi_cs: u8,
    /// Sensor type
    pub sensor_type: SensorType,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// FIFO enabled
    pub fifo_enabled: bool,
    /// FIFO threshold
    pub fifo_threshold: u16,
    /// Interrupt enabled
    pub interrupt_enabled: bool,
    /// Interrupt GPIO pin
    pub int_gpio: Option<u32>,
}

impl Default for SensorDriverConfig {
    fn default() -> Self {
        Self {
            bus_type: SensorBusType::I2c,
            i2c_bus: 1,
            i2c_addr: 0x68,
            spi_bus: 0,
            spi_cs: 0,
            sensor_type: SensorType::Imu6Dof,
            sample_rate: 100,
            fifo_enabled: true,
            fifo_threshold: 64,
            interrupt_enabled: true,
            int_gpio: Some(17),
        }
    }
}

/// Sensor bus type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorBusType {
    I2c,
    Spi,
}

/// Sensor type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    /// 6-DOF IMU (accel + gyro)
    Imu6Dof,
    /// 9-DOF IMU (accel + gyro + mag)
    Imu9Dof,
    /// Accelerometer only
    Accelerometer,
    /// Gyroscope only
    Gyroscope,
    /// Magnetometer
    Magnetometer,
    /// Barometer
    Barometer,
    /// Ambient light sensor
    Als,
    /// Proximity sensor
    Proximity,
    /// Temperature sensor
    Temperature,
    /// Heart rate sensor
    HeartRate,
    /// Time-of-flight sensor
    Tof,
}

/// Accelerometer range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelRange {
    G2,
    G4,
    G8,
    G16,
}

/// Gyroscope range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GyroRange {
    Dps125,
    Dps250,
    Dps500,
    Dps1000,
    Dps2000,
}

/// IMU data packet
#[derive(Debug, Clone, Copy, Default)]
pub struct ImuData {
    /// Acceleration X (m/s²)
    pub accel_x: f32,
    /// Acceleration Y (m/s²)
    pub accel_y: f32,
    /// Acceleration Z (m/s²)
    pub accel_z: f32,
    /// Angular velocity X (rad/s)
    pub gyro_x: f32,
    /// Angular velocity Y (rad/s)
    pub gyro_y: f32,
    /// Angular velocity Z (rad/s)
    pub gyro_z: f32,
    /// Magnetometer X (uT)
    pub mag_x: f32,
    /// Magnetometer Y (uT)
    pub mag_y: f32,
    /// Magnetometer Z (uT)
    pub mag_z: f32,
    /// Temperature (°C)
    pub temperature: f32,
    /// Timestamp (microseconds)
    pub timestamp: u64,
}

/// Barometer data
#[derive(Debug, Clone, Copy, Default)]
pub struct BarometerData {
    /// Pressure (Pa)
    pub pressure: f32,
    /// Temperature (°C)
    pub temperature: f32,
    /// Altitude estimate (m)
    pub altitude: f32,
    /// Timestamp (microseconds)
    pub timestamp: u64,
}

/// Ambient light data
#[derive(Debug, Clone, Copy, Default)]
pub struct AlsData {
    /// Visible light (lux)
    pub lux: f32,
    /// IR level
    pub ir: f32,
    /// UV index (if available)
    pub uv_index: f32,
    /// Timestamp (microseconds)
    pub timestamp: u64,
}

/// Common sensor registers (ICM-42688-P style)
mod registers {
    pub const WHO_AM_I: u8 = 0x75;
    pub const PWR_MGMT0: u8 = 0x4E;
    pub const GYRO_CONFIG0: u8 = 0x4F;
    pub const ACCEL_CONFIG0: u8 = 0x50;
    pub const GYRO_CONFIG1: u8 = 0x51;
    pub const ACCEL_CONFIG1: u8 = 0x53;
    pub const FIFO_CONFIG: u8 = 0x16;
    pub const FIFO_COUNT_H: u8 = 0x2E;
    pub const FIFO_COUNT_L: u8 = 0x2F;
    pub const FIFO_DATA: u8 = 0x30;
    pub const TEMP_DATA1: u8 = 0x1D;
    pub const TEMP_DATA0: u8 = 0x1E;
    pub const ACCEL_DATA_X1: u8 = 0x1F;
    pub const GYRO_DATA_X1: u8 = 0x25;
    pub const INT_SOURCE0: u8 = 0x65;
    pub const INT_STATUS: u8 = 0x2D;
}

/// Sensor driver
#[derive(Debug)]
pub struct SensorDriver {
    /// Configuration
    config: SensorDriverConfig,
    /// Current state
    state: DriverState,
    /// I2C device (if using I2C)
    i2c: Option<I2cDevice>,
    /// SPI device (if using SPI)
    spi: Option<SpiDevice>,
    /// FIFO buffer
    fifo: VecDeque<ImuData>,
    /// Accelerometer range
    accel_range: AccelRange,
    /// Gyroscope range
    gyro_range: GyroRange,
    /// Calibration offsets
    cal_offsets: ImuData,
    /// Sample counter
    sample_count: AtomicU64,
    /// Statistics
    stats: DriverStats,
    /// Is sampling
    sampling: AtomicBool,
    /// Last sample time
    last_sample: Option<Instant>,
}

impl SensorDriver {
    /// Create new sensor driver
    pub fn new(config: SensorDriverConfig) -> Self {
        Self {
            config,
            state: DriverState::Unloaded,
            i2c: None,
            spi: None,
            fifo: VecDeque::with_capacity(256),
            accel_range: AccelRange::G4,
            gyro_range: GyroRange::Dps500,
            cal_offsets: ImuData::default(),
            sample_count: AtomicU64::new(0),
            stats: DriverStats::default(),
            sampling: AtomicBool::new(false),
            last_sample: None,
        }
    }

    /// Read register
    fn read_reg(&self, reg: u8) -> Result<u8, DriverError> {
        match self.config.bus_type {
            SensorBusType::I2c => {
                self.i2c.as_ref()
                    .ok_or(DriverError::NotLoaded)?
                    .read_reg(reg)
            }
            SensorBusType::Spi => {
                let spi = self.spi.as_ref().ok_or(DriverError::NotLoaded)?;
                let tx = [reg | 0x80, 0];
                let mut rx = [0u8; 2];
                spi.transfer(&tx, &mut rx)?;
                Ok(rx[1])
            }
        }
    }

    /// Write register
    fn write_reg(&self, reg: u8, value: u8) -> Result<(), DriverError> {
        match self.config.bus_type {
            SensorBusType::I2c => {
                self.i2c.as_ref()
                    .ok_or(DriverError::NotLoaded)?
                    .write_reg(reg, value)
            }
            SensorBusType::Spi => {
                let spi = self.spi.as_ref().ok_or(DriverError::NotLoaded)?;
                let tx = [reg & 0x7F, value];
                let mut rx = [0u8; 2];
                spi.transfer(&tx, &mut rx)?;
                Ok(())
            }
        }
    }

    /// Read multiple bytes
    fn read_bytes(&self, reg: u8, buf: &mut [u8]) -> Result<(), DriverError> {
        match self.config.bus_type {
            SensorBusType::I2c => {
                self.i2c.as_ref()
                    .ok_or(DriverError::NotLoaded)?
                    .read_bytes(reg, buf)?;
                Ok(())
            }
            SensorBusType::Spi => {
                let spi = self.spi.as_ref().ok_or(DriverError::NotLoaded)?;
                let mut tx = vec![reg | 0x80];
                tx.extend(std::iter::repeat(0).take(buf.len()));
                let mut rx = vec![0u8; tx.len()];
                spi.transfer(&tx, &mut rx)?;
                buf.copy_from_slice(&rx[1..]);
                Ok(())
            }
        }
    }

    /// Set accelerometer range
    pub fn set_accel_range(&mut self, range: AccelRange) -> Result<(), DriverError> {
        let config = match range {
            AccelRange::G2 => 0x03,
            AccelRange::G4 => 0x02,
            AccelRange::G8 => 0x01,
            AccelRange::G16 => 0x00,
        };
        self.write_reg(registers::ACCEL_CONFIG0, config)?;
        self.accel_range = range;
        Ok(())
    }

    /// Set gyroscope range
    pub fn set_gyro_range(&mut self, range: GyroRange) -> Result<(), DriverError> {
        let config = match range {
            GyroRange::Dps125 => 0x04,
            GyroRange::Dps250 => 0x03,
            GyroRange::Dps500 => 0x02,
            GyroRange::Dps1000 => 0x01,
            GyroRange::Dps2000 => 0x00,
        };
        self.write_reg(registers::GYRO_CONFIG0, config)?;
        self.gyro_range = range;
        Ok(())
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, hz: u32) -> Result<(), DriverError> {
        // Would configure ODR (Output Data Rate)
        Ok(())
    }

    /// Read IMU data
    pub fn read_imu(&mut self) -> Result<ImuData, DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        let mut buf = [0u8; 14]; // 6 accel + 6 gyro + 2 temp
        self.read_bytes(registers::ACCEL_DATA_X1, &mut buf)?;

        // Convert raw values (16-bit signed)
        let accel_scale = match self.accel_range {
            AccelRange::G2 => 16384.0,
            AccelRange::G4 => 8192.0,
            AccelRange::G8 => 4096.0,
            AccelRange::G16 => 2048.0,
        };

        let gyro_scale = match self.gyro_range {
            GyroRange::Dps125 => 262.0,
            GyroRange::Dps250 => 131.0,
            GyroRange::Dps500 => 65.5,
            GyroRange::Dps1000 => 32.8,
            GyroRange::Dps2000 => 16.4,
        };

        let accel_x_raw = i16::from_be_bytes([buf[0], buf[1]]);
        let accel_y_raw = i16::from_be_bytes([buf[2], buf[3]]);
        let accel_z_raw = i16::from_be_bytes([buf[4], buf[5]]);
        let gyro_x_raw = i16::from_be_bytes([buf[6], buf[7]]);
        let gyro_y_raw = i16::from_be_bytes([buf[8], buf[9]]);
        let gyro_z_raw = i16::from_be_bytes([buf[10], buf[11]]);
        let temp_raw = i16::from_be_bytes([buf[12], buf[13]]);

        self.sample_count.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_read += 14;
        self.last_sample = Some(Instant::now());

        Ok(ImuData {
            accel_x: (accel_x_raw as f32 / accel_scale) * 9.81 - self.cal_offsets.accel_x,
            accel_y: (accel_y_raw as f32 / accel_scale) * 9.81 - self.cal_offsets.accel_y,
            accel_z: (accel_z_raw as f32 / accel_scale) * 9.81 - self.cal_offsets.accel_z,
            gyro_x: (gyro_x_raw as f32 / gyro_scale).to_radians() - self.cal_offsets.gyro_x,
            gyro_y: (gyro_y_raw as f32 / gyro_scale).to_radians() - self.cal_offsets.gyro_y,
            gyro_z: (gyro_z_raw as f32 / gyro_scale).to_radians() - self.cal_offsets.gyro_z,
            temperature: (temp_raw as f32 / 132.48) + 25.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
            ..Default::default()
        })
    }

    /// Read FIFO
    pub fn read_fifo(&mut self) -> Result<Vec<ImuData>, DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        // Read FIFO count
        let count_h = self.read_reg(registers::FIFO_COUNT_H)? as u16;
        let count_l = self.read_reg(registers::FIFO_COUNT_L)? as u16;
        let count = (count_h << 8) | count_l;

        let samples = (count / 16) as usize; // 16 bytes per sample
        let mut data = Vec::with_capacity(samples);

        for _ in 0..samples {
            if let Ok(sample) = self.read_imu() {
                data.push(sample);
            }
        }

        Ok(data)
    }

    /// Calibrate sensor (find static offsets)
    pub fn calibrate(&mut self, samples: u32) -> Result<ImuData, DriverError> {
        if self.state != DriverState::Running {
            return Err(DriverError::NotLoaded);
        }

        let mut sum = ImuData::default();
        
        for _ in 0..samples {
            let data = self.read_imu()?;
            sum.accel_x += data.accel_x;
            sum.accel_y += data.accel_y;
            sum.accel_z += data.accel_z - 9.81; // Remove gravity
            sum.gyro_x += data.gyro_x;
            sum.gyro_y += data.gyro_y;
            sum.gyro_z += data.gyro_z;
        }

        let n = samples as f32;
        self.cal_offsets = ImuData {
            accel_x: sum.accel_x / n,
            accel_y: sum.accel_y / n,
            accel_z: sum.accel_z / n,
            gyro_x: sum.gyro_x / n,
            gyro_y: sum.gyro_y / n,
            gyro_z: sum.gyro_z / n,
            ..Default::default()
        };

        Ok(self.cal_offsets)
    }

    /// Get calibration offsets
    pub fn calibration(&self) -> &ImuData {
        &self.cal_offsets
    }

    /// Get sample count
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }
}

impl Driver for SensorDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "karana-sensor".into(),
            version: "1.0.0".into(),
            vendor: "KaranaOS".into(),
            device_ids: vec!["sensor:imu".into(), "sensor:icm42688".into()],
            loaded: self.state != DriverState::Unloaded,
            state: self.state,
        }
    }

    fn state(&self) -> DriverState {
        self.state
    }

    fn load(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Loading;

        // Open bus device
        match self.config.bus_type {
            SensorBusType::I2c => {
                let mut dev = I2cDevice::new(self.config.i2c_bus, self.config.i2c_addr);
                dev.open()?;
                self.i2c = Some(dev);
            }
            SensorBusType::Spi => {
                let mut dev = SpiDevice::new(self.config.spi_bus, self.config.spi_cs);
                dev.open()?;
                dev.set_mode(3)?; // SPI mode 3 for IMU
                dev.set_speed(10_000_000)?;
                self.spi = Some(dev);
            }
        }

        self.state = DriverState::Loaded;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), DriverError> {
        if let Some(ref mut i2c) = self.i2c {
            i2c.close();
        }
        if let Some(ref mut spi) = self.spi {
            spi.close();
        }
        self.i2c = None;
        self.spi = None;
        self.state = DriverState::Unloaded;
        Ok(())
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Loaded {
            return Err(DriverError::NotLoaded);
        }

        // Verify device ID
        let who_am_i = self.read_reg(registers::WHO_AM_I)?;
        // Would check expected ID

        // Configure sensor
        self.write_reg(registers::PWR_MGMT0, 0x0F)?; // Enable accel + gyro
        self.set_accel_range(self.accel_range)?;
        self.set_gyro_range(self.gyro_range)?;

        if self.config.fifo_enabled {
            self.write_reg(registers::FIFO_CONFIG, 0x40)?; // Enable FIFO
        }

        self.state = DriverState::Ready;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Ready {
            return Err(DriverError::NotLoaded);
        }
        self.sampling.store(true, Ordering::Relaxed);
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.sampling.store(false, Ordering::Relaxed);
        self.state = DriverState::Ready;
        Ok(())
    }

    fn suspend(&mut self) -> Result<(), DriverError> {
        self.write_reg(registers::PWR_MGMT0, 0x00)?; // Disable sensors
        self.state = DriverState::Suspended;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), DriverError> {
        self.write_reg(registers::PWR_MGMT0, 0x0F)?; // Re-enable sensors
        self.state = DriverState::Running;
        Ok(())
    }

    fn stats(&self) -> DriverStats {
        DriverStats {
            bytes_read: self.stats.bytes_read,
            interrupts: self.sample_count.load(Ordering::Relaxed),
            ..self.stats.clone()
        }
    }

    fn test(&self) -> Result<(), DriverError> {
        if self.state == DriverState::Unloaded {
            return Err(DriverError::NotLoaded);
        }
        // Would perform self-test
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_driver_creation() {
        let driver = SensorDriver::new(SensorDriverConfig::default());
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_sensor_driver_lifecycle() {
        let mut driver = SensorDriver::new(SensorDriverConfig::default());
        
        driver.load().unwrap();
        assert_eq!(driver.state(), DriverState::Loaded);
        
        driver.init().unwrap();
        assert_eq!(driver.state(), DriverState::Ready);
        
        driver.start().unwrap();
        assert_eq!(driver.state(), DriverState::Running);
        
        driver.stop().unwrap();
        driver.unload().unwrap();
    }

    #[test]
    fn test_accel_range() {
        let mut driver = SensorDriver::new(SensorDriverConfig::default());
        driver.load().unwrap();
        driver.init().unwrap();
        
        driver.set_accel_range(AccelRange::G8).unwrap();
        assert_eq!(driver.accel_range, AccelRange::G8);
    }

    #[test]
    fn test_gyro_range() {
        let mut driver = SensorDriver::new(SensorDriverConfig::default());
        driver.load().unwrap();
        driver.init().unwrap();
        
        driver.set_gyro_range(GyroRange::Dps1000).unwrap();
        assert_eq!(driver.gyro_range, GyroRange::Dps1000);
    }

    #[test]
    fn test_imu_data_default() {
        let data = ImuData::default();
        assert_eq!(data.accel_x, 0.0);
        assert_eq!(data.gyro_x, 0.0);
    }

    #[test]
    fn test_driver_info() {
        let driver = SensorDriver::new(SensorDriverConfig::default());
        let info = driver.info();
        
        assert_eq!(info.name, "karana-sensor");
        assert!(!info.loaded);
    }
}
