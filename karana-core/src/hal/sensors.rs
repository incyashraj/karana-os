// Kāraṇa OS - Sensors HAL
// Hardware abstraction for smart glasses sensors

use super::HalError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Sensor power mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorPowerMode {
    /// Sensor off
    Off,
    /// Low power mode
    Low,
    /// Normal operation
    Normal,
    /// High performance
    High,
}

/// Sensor configuration
#[derive(Debug, Clone)]
pub struct SensorConfig {
    /// Polling interval (ms)
    pub poll_interval_ms: u32,
    /// Batch size
    pub batch_size: u32,
    /// Enable sensor fusion
    pub fusion_enabled: bool,
    /// Calibration enabled
    pub calibration_enabled: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 10,
            batch_size: 1,
            fusion_enabled: true,
            calibration_enabled: true,
        }
    }
}

/// Sensor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    /// 3-axis accelerometer
    Accelerometer,
    /// 3-axis gyroscope
    Gyroscope,
    /// 3-axis magnetometer
    Magnetometer,
    /// Barometer (pressure)
    Barometer,
    /// Ambient light sensor
    AmbientLight,
    /// Proximity sensor
    Proximity,
    /// Temperature sensor
    Temperature,
    /// Humidity sensor
    Humidity,
    /// Heart rate sensor
    HeartRate,
    /// GPS receiver
    Gps,
    /// Gravity sensor (fused)
    Gravity,
    /// Linear acceleration (fused)
    LinearAcceleration,
    /// Rotation vector (fused)
    RotationVector,
    /// Game rotation vector
    GameRotationVector,
    /// Step counter
    StepCounter,
    /// Significant motion
    SignificantMotion,
}

/// Sensor data container
#[derive(Debug, Clone)]
pub struct SensorData {
    /// Sensor type
    pub sensor_type: SensorType,
    /// Timestamp (nanoseconds)
    pub timestamp_ns: u64,
    /// Values (interpretation depends on sensor type)
    pub values: Vec<f32>,
    /// Accuracy
    pub accuracy: SensorAccuracy,
}

/// Sensor accuracy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorAccuracy {
    /// Unreliable
    Unreliable,
    /// Low accuracy
    Low,
    /// Medium accuracy
    Medium,
    /// High accuracy
    High,
}

/// Individual sensor instance
#[derive(Debug)]
pub struct Sensor {
    /// Sensor type
    sensor_type: SensorType,
    /// Sensor name
    name: String,
    /// Is enabled
    enabled: bool,
    /// Sampling rate (Hz)
    sample_rate: u32,
    /// Maximum range
    max_range: f32,
    /// Resolution
    resolution: f32,
    /// Power consumption (mA)
    power_ma: f32,
    /// Minimum delay (us)
    min_delay_us: u32,
    /// Last reading
    last_data: Option<SensorData>,
    /// Read count
    read_count: AtomicU64,
}

impl Sensor {
    /// Create new sensor
    pub fn new(sensor_type: SensorType) -> Self {
        let (name, max_range, resolution, power, min_delay) = match sensor_type {
            SensorType::Accelerometer => ("Accelerometer", 39.2, 0.001, 0.5, 1000),
            SensorType::Gyroscope => ("Gyroscope", 34.9, 0.0001, 0.8, 1000),
            SensorType::Magnetometer => ("Magnetometer", 2000.0, 0.1, 0.3, 10000),
            SensorType::Barometer => ("Barometer", 1100.0, 0.01, 0.1, 20000),
            SensorType::AmbientLight => ("Ambient Light", 100000.0, 1.0, 0.2, 200000),
            SensorType::Proximity => ("Proximity", 5.0, 1.0, 0.1, 100000),
            SensorType::Temperature => ("Temperature", 85.0, 0.1, 0.1, 100000),
            SensorType::Humidity => ("Humidity", 100.0, 0.1, 0.1, 100000),
            SensorType::HeartRate => ("Heart Rate", 250.0, 1.0, 1.0, 100000),
            SensorType::Gps => ("GPS", 0.0, 0.0, 50.0, 1000000),
            SensorType::Gravity => ("Gravity", 9.81, 0.001, 0.0, 1000),
            SensorType::LinearAcceleration => ("Linear Accel", 39.2, 0.001, 0.0, 1000),
            SensorType::RotationVector => ("Rotation", 1.0, 0.0001, 0.0, 1000),
            SensorType::GameRotationVector => ("Game Rotation", 1.0, 0.0001, 0.0, 1000),
            SensorType::StepCounter => ("Step Counter", 1000000.0, 1.0, 0.1, 0),
            SensorType::SignificantMotion => ("Motion", 1.0, 1.0, 0.1, 0),
        };

        Self {
            sensor_type,
            name: name.into(),
            enabled: false,
            sample_rate: 100,
            max_range,
            resolution,
            power_ma: power,
            min_delay_us: min_delay,
            last_data: None,
            read_count: AtomicU64::new(0),
        }
    }

    /// Enable sensor
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable sensor
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get sensor type
    pub fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate;
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Read sensor
    pub fn read(&mut self) -> Option<SensorData> {
        if !self.enabled {
            return None;
        }

        self.read_count.fetch_add(1, Ordering::Relaxed);

        // Generate simulated data
        let values = self.generate_data();
        let data = SensorData {
            sensor_type: self.sensor_type,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            values,
            accuracy: SensorAccuracy::High,
        };

        self.last_data = Some(data.clone());
        Some(data)
    }

    /// Generate simulated sensor data
    fn generate_data(&self) -> Vec<f32> {
        match self.sensor_type {
            SensorType::Accelerometer => vec![0.0, 0.0, 9.81], // Standing still
            SensorType::Gyroscope => vec![0.0, 0.0, 0.0],
            SensorType::Magnetometer => vec![25.0, 0.0, 45.0],
            SensorType::Barometer => vec![1013.25], // Sea level
            SensorType::AmbientLight => vec![500.0], // Indoor lighting
            SensorType::Proximity => vec![5.0], // Nothing close
            SensorType::Temperature => vec![25.0],
            SensorType::Humidity => vec![50.0],
            SensorType::HeartRate => vec![72.0],
            SensorType::Gps => vec![37.7749, -122.4194, 10.0], // lat, lon, alt
            SensorType::Gravity => vec![0.0, 0.0, 9.81],
            SensorType::LinearAcceleration => vec![0.0, 0.0, 0.0],
            SensorType::RotationVector => vec![0.0, 0.0, 0.0, 1.0], // Quaternion
            SensorType::GameRotationVector => vec![0.0, 0.0, 0.0, 1.0],
            SensorType::StepCounter => vec![0.0],
            SensorType::SignificantMotion => vec![0.0],
        }
    }

    /// Get read count
    pub fn read_count(&self) -> u64 {
        self.read_count.load(Ordering::Relaxed)
    }
}

/// Sensor hub for managing all sensors
#[derive(Debug)]
pub struct SensorHub {
    /// Configuration
    config: SensorConfig,
    /// Registered sensors
    sensors: HashMap<SensorType, Sensor>,
    /// Sensor fusion
    fusion: SensorFusion,
    /// Statistics
    stats: SensorHubStats,
    /// Is initialized
    initialized: bool,
    /// Is running
    running: bool,
}

/// Sensor hub statistics
#[derive(Debug, Default, Clone)]
pub struct SensorHubStats {
    /// Total sensor reads
    pub reads: u64,
    /// Fusion updates
    pub fusion_updates: u64,
    /// Errors
    pub errors: u64,
}

impl SensorHub {
    /// Create new sensor hub
    pub fn new(config: SensorConfig) -> Result<Self, HalError> {
        let mut sensors = HashMap::new();

        // Register default sensors
        for sensor_type in &[
            SensorType::Accelerometer,
            SensorType::Gyroscope,
            SensorType::Magnetometer,
            SensorType::Barometer,
            SensorType::AmbientLight,
            SensorType::Proximity,
            SensorType::Temperature,
        ] {
            sensors.insert(*sensor_type, Sensor::new(*sensor_type));
        }

        Ok(Self {
            config,
            sensors,
            fusion: SensorFusion::new(),
            stats: SensorHubStats::default(),
            initialized: false,
            running: false,
        })
    }

    /// Initialize sensor hub
    pub fn initialize(&mut self) -> Result<(), HalError> {
        // Initialize all sensors
        for sensor in self.sensors.values_mut() {
            sensor.set_sample_rate(1000 / self.config.poll_interval_ms);
        }

        self.initialized = true;
        Ok(())
    }

    /// Start sensor polling
    pub fn start(&mut self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Sensor hub not initialized".into()));
        }

        // Enable default sensors
        self.enable_sensor(SensorType::Accelerometer)?;
        self.enable_sensor(SensorType::Gyroscope)?;

        self.running = true;
        Ok(())
    }

    /// Stop sensor polling
    pub fn stop(&mut self) -> Result<(), HalError> {
        // Disable all sensors
        for sensor in self.sensors.values_mut() {
            sensor.disable();
        }

        self.running = false;
        Ok(())
    }

    /// Enable a sensor
    pub fn enable_sensor(&mut self, sensor_type: SensorType) -> Result<(), HalError> {
        if let Some(sensor) = self.sensors.get_mut(&sensor_type) {
            sensor.enable();
            Ok(())
        } else {
            Err(HalError::DeviceNotFound(format!("{:?}", sensor_type)))
        }
    }

    /// Disable a sensor
    pub fn disable_sensor(&mut self, sensor_type: SensorType) -> Result<(), HalError> {
        if let Some(sensor) = self.sensors.get_mut(&sensor_type) {
            sensor.disable();
            Ok(())
        } else {
            Err(HalError::DeviceNotFound(format!("{:?}", sensor_type)))
        }
    }

    /// Read sensor data
    pub fn read(&mut self, sensor_type: SensorType) -> Result<SensorData, HalError> {
        if let Some(sensor) = self.sensors.get_mut(&sensor_type) {
            if let Some(data) = sensor.read() {
                self.stats.reads += 1;
                Ok(data)
            } else {
                Err(HalError::ConfigError("Sensor not enabled".into()))
            }
        } else {
            Err(HalError::DeviceNotFound(format!("{:?}", sensor_type)))
        }
    }

    /// Read all enabled sensors
    pub fn read_all(&mut self) -> Vec<SensorData> {
        let mut results = Vec::new();

        for sensor in self.sensors.values_mut() {
            if let Some(data) = sensor.read() {
                self.stats.reads += 1;
                results.push(data);
            }
        }

        // Update fusion
        if self.config.fusion_enabled {
            self.fusion.update(&results);
            self.stats.fusion_updates += 1;
        }

        results
    }

    /// Get fused orientation
    pub fn orientation(&self) -> Quaternion {
        self.fusion.orientation()
    }

    /// Get fused position (relative)
    pub fn position(&self) -> Vector3 {
        self.fusion.position()
    }

    /// Get statistics
    pub fn stats(&self) -> SensorHubStats {
        self.stats.clone()
    }

    /// Set power mode
    pub fn set_power_mode(&mut self, mode: SensorPowerMode) -> Result<(), HalError> {
        let rate = match mode {
            SensorPowerMode::Off => 0,
            SensorPowerMode::Low => 10,
            SensorPowerMode::Normal => 100,
            SensorPowerMode::High => 1000,
        };

        for sensor in self.sensors.values_mut() {
            if rate == 0 {
                sensor.disable();
            } else {
                sensor.set_sample_rate(rate);
            }
        }

        Ok(())
    }

    /// Self-test
    pub fn test(&self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Not initialized".into()));
        }
        Ok(())
    }

    /// Get available sensors
    pub fn available_sensors(&self) -> Vec<SensorType> {
        self.sensors.keys().copied().collect()
    }
}

/// Sensor fusion (complementary filter)
#[derive(Debug)]
pub struct SensorFusion {
    /// Current orientation (quaternion)
    orientation: Quaternion,
    /// Current position (relative)
    position: Vector3,
    /// Velocity
    velocity: Vector3,
    /// Filter alpha
    alpha: f32,
    /// Last update time
    last_update: Option<Instant>,
}

impl SensorFusion {
    /// Create new fusion filter
    pub fn new() -> Self {
        Self {
            orientation: Quaternion::identity(),
            position: Vector3::zero(),
            velocity: Vector3::zero(),
            alpha: 0.98,
            last_update: None,
        }
    }

    /// Update with sensor data
    pub fn update(&mut self, data: &[SensorData]) {
        let now = Instant::now();
        let dt = self.last_update
            .map(|t| now.duration_since(t).as_secs_f32())
            .unwrap_or(0.01);
        self.last_update = Some(now);

        // Extract sensor readings
        let mut accel = None;
        let mut gyro = None;

        for reading in data {
            match reading.sensor_type {
                SensorType::Accelerometer => {
                    if reading.values.len() >= 3 {
                        accel = Some(Vector3 {
                            x: reading.values[0],
                            y: reading.values[1],
                            z: reading.values[2],
                        });
                    }
                }
                SensorType::Gyroscope => {
                    if reading.values.len() >= 3 {
                        gyro = Some(Vector3 {
                            x: reading.values[0],
                            y: reading.values[1],
                            z: reading.values[2],
                        });
                    }
                }
                _ => {}
            }
        }

        // Complementary filter
        if let (Some(a), Some(g)) = (accel, gyro) {
            // Integrate gyroscope
            let gyro_quat = Quaternion::from_angular_velocity(&g, dt);
            let gyro_orientation = self.orientation.multiply(&gyro_quat);

            // Calculate accelerometer orientation
            let accel_orientation = Quaternion::from_gravity(&a);

            // Blend
            self.orientation = gyro_orientation.slerp(&accel_orientation, 1.0 - self.alpha);
            self.orientation = self.orientation.normalize();

            // Update position (double integration - very noisy, just for demo)
            let gravity = Vector3 { x: 0.0, y: 0.0, z: 9.81 };
            let linear_accel = a.subtract(&gravity);
            self.velocity = self.velocity.add(&linear_accel.scale(dt));
            self.position = self.position.add(&self.velocity.scale(dt));
        }
    }

    /// Get current orientation
    pub fn orientation(&self) -> Quaternion {
        self.orientation
    }

    /// Get current position
    pub fn position(&self) -> Vector3 {
        self.position
    }

    /// Reset fusion state
    pub fn reset(&mut self) {
        self.orientation = Quaternion::identity();
        self.position = Vector3::zero();
        self.velocity = Vector3::zero();
        self.last_update = None;
    }
}

impl Default for SensorFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// 3D vector
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let m = self.magnitude();
        if m > 0.0001 {
            self.scale(1.0 / m)
        } else {
            *self
        }
    }
}

/// Quaternion for rotation
#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    pub fn identity() -> Self {
        Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn from_gravity(gravity: &Vector3) -> Self {
        let g = gravity.normalize();
        // Calculate rotation from [0,0,1] to gravity direction
        let cross = Vector3 {
            x: -g.y,
            y: g.x,
            z: 0.0,
        };
        let dot = g.z;
        let s = ((1.0 + dot) * 2.0).sqrt();
        
        if s > 0.0001 {
            Self {
                w: s / 2.0,
                x: cross.x / s,
                y: cross.y / s,
                z: cross.z / s,
            }.normalize()
        } else {
            Self::identity()
        }
    }

    pub fn from_angular_velocity(omega: &Vector3, dt: f32) -> Self {
        let half_angle = omega.magnitude() * dt / 2.0;
        if half_angle > 0.0001 {
            let axis = omega.normalize();
            let sin_half = half_angle.sin();
            Self {
                w: half_angle.cos(),
                x: axis.x * sin_half,
                y: axis.y * sin_half,
                z: axis.z * sin_half,
            }
        } else {
            Self::identity()
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    pub fn normalize(&self) -> Self {
        let m = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if m > 0.0001 {
            Self {
                w: self.w / m,
                x: self.x / m,
                y: self.y / m,
                z: self.z / m,
            }
        } else {
            Self::identity()
        }
    }

    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        let mut dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;
        
        let other = if dot < 0.0 {
            dot = -dot;
            Self { w: -other.w, x: -other.x, y: -other.y, z: -other.z }
        } else {
            *other
        };

        if dot > 0.9995 {
            // Linear interpolation for nearly parallel quaternions
            return Self {
                w: self.w + t * (other.w - self.w),
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
            }.normalize();
        }

        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let w1 = ((1.0 - t) * theta).sin() / sin_theta;
        let w2 = (t * theta).sin() / sin_theta;

        Self {
            w: self.w * w1 + other.w * w2,
            x: self.x * w1 + other.x * w2,
            y: self.y * w1 + other.y * w2,
            z: self.z * w1 + other.z * w2,
        }
    }

    pub fn to_euler(&self) -> (f32, f32, f32) {
        // Returns (roll, pitch, yaw) in radians
        let sinr = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr.atan2(cosr);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny.atan2(cosy);

        (roll, pitch, yaw)
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_config_default() {
        let config = SensorConfig::default();
        assert_eq!(config.poll_interval_ms, 10);
        assert!(config.fusion_enabled);
    }

    #[test]
    fn test_sensor_creation() {
        let sensor = Sensor::new(SensorType::Accelerometer);
        assert_eq!(sensor.sensor_type(), SensorType::Accelerometer);
        assert!(!sensor.is_enabled());
    }

    #[test]
    fn test_sensor_enable_disable() {
        let mut sensor = Sensor::new(SensorType::Accelerometer);
        
        sensor.enable();
        assert!(sensor.is_enabled());

        sensor.disable();
        assert!(!sensor.is_enabled());
    }

    #[test]
    fn test_sensor_read() {
        let mut sensor = Sensor::new(SensorType::Accelerometer);
        sensor.enable();

        let data = sensor.read().unwrap();
        assert_eq!(data.sensor_type, SensorType::Accelerometer);
        assert_eq!(data.values.len(), 3);
    }

    #[test]
    fn test_sensor_hub_creation() {
        let hub = SensorHub::new(SensorConfig::default());
        assert!(hub.is_ok());
    }

    #[test]
    fn test_sensor_hub_enable_disable() {
        let mut hub = SensorHub::new(SensorConfig::default()).unwrap();
        hub.initialize().unwrap();

        hub.enable_sensor(SensorType::Accelerometer).unwrap();
        let data = hub.read(SensorType::Accelerometer).unwrap();
        assert_eq!(data.values.len(), 3);

        hub.disable_sensor(SensorType::Accelerometer).unwrap();
        assert!(hub.read(SensorType::Accelerometer).is_err());
    }

    #[test]
    fn test_sensor_hub_read_all() {
        let mut hub = SensorHub::new(SensorConfig::default()).unwrap();
        hub.initialize().unwrap();
        hub.start().unwrap();

        let data = hub.read_all();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_quaternion_identity() {
        let q = Quaternion::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
    }

    #[test]
    fn test_quaternion_normalize() {
        let q = Quaternion { w: 2.0, x: 0.0, y: 0.0, z: 0.0 };
        let n = q.normalize();
        assert!((n.w - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vector3_operations() {
        let a = Vector3 { x: 1.0, y: 2.0, z: 3.0 };
        let b = Vector3 { x: 4.0, y: 5.0, z: 6.0 };

        let sum = a.add(&b);
        assert_eq!(sum.x, 5.0);

        let diff = b.subtract(&a);
        assert_eq!(diff.x, 3.0);

        let scaled = a.scale(2.0);
        assert_eq!(scaled.x, 2.0);
    }

    #[test]
    fn test_sensor_fusion() {
        let mut fusion = SensorFusion::new();

        let data = vec![
            SensorData {
                sensor_type: SensorType::Accelerometer,
                timestamp_ns: 0,
                values: vec![0.0, 0.0, 9.81],
                accuracy: SensorAccuracy::High,
            },
            SensorData {
                sensor_type: SensorType::Gyroscope,
                timestamp_ns: 0,
                values: vec![0.0, 0.0, 0.0],
                accuracy: SensorAccuracy::High,
            },
        ];

        fusion.update(&data);
        
        let orientation = fusion.orientation();
        // Should be close to identity for stationary device
        assert!((orientation.w - 1.0).abs() < 0.5);
    }
}
