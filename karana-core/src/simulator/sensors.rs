//! Virtual Sensors Module
//!
//! Simulates all the sensors found in smart glasses:
//! - IMU (accelerometer, gyroscope, magnetometer)
//! - GPS/Location
//! - Ambient light sensor
//! - Proximity sensor
//! - Touch/gesture input
//! - Eye tracking (simulated)

use std::time::{Duration, Instant};
use std::f32::consts::PI;

/// 3D Vector for sensor data
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
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

impl Default for Quaternion {
    fn default() -> Self {
        Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Quaternion {
    /// Convert to euler angles (pitch, yaw, roll) in degrees
    pub fn to_euler(&self) -> Vec3 {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            (PI / 2.0).copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vec3 {
            x: pitch.to_degrees(),
            y: yaw.to_degrees(),
            z: roll.to_degrees(),
        }
    }
}

/// GPS/Location data
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub accuracy: f32,
    pub speed: f32,         // m/s
    pub heading: f32,       // degrees from north
}

impl Default for Location {
    fn default() -> Self {
        // Default to San Francisco
        Self {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: 10.0,
            accuracy: 5.0,
            speed: 0.0,
            heading: 0.0,
        }
    }
}

/// Gesture types that can be detected
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    None,
    Tap,
    DoubleTap,
    LongPress,
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    Pinch,
    Spread,
    // Head gestures
    NodYes,
    ShakeNo,
    LookUp,
    LookDown,
    LookLeft,
    LookRight,
}

/// Eye tracking data (simulated)
#[derive(Debug, Clone, Copy, Default)]
pub struct EyeTracking {
    pub gaze_point: (f32, f32),  // Normalized screen coordinates
    pub left_eye_open: f32,      // 0.0 = closed, 1.0 = open
    pub right_eye_open: f32,
    pub pupil_dilation: f32,     // Relative to baseline
    pub is_blinking: bool,
}

/// Individual sensor reading with timestamp
#[derive(Debug, Clone)]
pub struct SensorReading<T: Clone> {
    pub value: T,
    pub timestamp: Instant,
    pub confidence: f32,
}

impl<T: Clone> SensorReading<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            confidence: 1.0,
        }
    }
}

/// Activity/Motion state detected from sensors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityState {
    Stationary,
    Walking,
    Running,
    Cycling,
    Driving,
    Unknown,
}

/// Virtual sensor system
pub struct VirtualSensors {
    // IMU
    pub accelerometer: SensorReading<Vec3>,
    pub gyroscope: SensorReading<Vec3>,
    pub magnetometer: SensorReading<Vec3>,
    pub orientation: SensorReading<Quaternion>,
    
    // Location
    pub location: SensorReading<Location>,
    
    // Environment
    pub ambient_light: SensorReading<f32>,  // Lux
    pub proximity: SensorReading<f32>,      // cm
    pub temperature: SensorReading<f32>,    // °C (ambient)
    
    // Eye tracking
    pub eye_tracking: SensorReading<EyeTracking>,
    
    // Derived
    pub activity: ActivityState,
    pub step_count: u32,
    
    // Gesture
    pub last_gesture: GestureType,
    gesture_time: Instant,
    
    // State
    active: bool,
    simulation_time: f32,
    
    // Noise simulation
    noise_level: f32,
}

impl VirtualSensors {
    pub fn new() -> Self {
        Self {
            accelerometer: SensorReading::new(Vec3::new(0.0, -9.81, 0.0)),
            gyroscope: SensorReading::new(Vec3::default()),
            magnetometer: SensorReading::new(Vec3::new(0.0, 25.0, -40.0)),
            orientation: SensorReading::new(Quaternion::default()),
            location: SensorReading::new(Location::default()),
            ambient_light: SensorReading::new(500.0), // Indoor lighting
            proximity: SensorReading::new(100.0),     // Nothing nearby
            temperature: SensorReading::new(22.0),
            eye_tracking: SensorReading::new(EyeTracking::default()),
            activity: ActivityState::Stationary,
            step_count: 0,
            last_gesture: GestureType::None,
            gesture_time: Instant::now(),
            active: true,
            simulation_time: 0.0,
            noise_level: 0.1,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Update sensors with simulated noise and movement
    pub fn tick(&mut self, delta: Duration) {
        if !self.active {
            return;
        }

        self.simulation_time += delta.as_secs_f32();
        
        // Add noise to accelerometer
        let noise = self.noise_level;
        self.accelerometer.value.x += (rand_simple() - 0.5) * noise;
        self.accelerometer.value.y += (rand_simple() - 0.5) * noise;
        self.accelerometer.value.z += (rand_simple() - 0.5) * noise;
        self.accelerometer.timestamp = Instant::now();

        // Simulate small gyroscope drift
        self.gyroscope.value.x = (rand_simple() - 0.5) * 0.01;
        self.gyroscope.value.y = (rand_simple() - 0.5) * 0.01;
        self.gyroscope.value.z = (rand_simple() - 0.5) * 0.01;
        self.gyroscope.timestamp = Instant::now();

        // Simulate eye tracking
        self.eye_tracking.value.left_eye_open = 1.0;
        self.eye_tracking.value.right_eye_open = 1.0;
        self.eye_tracking.value.gaze_point = (0.5, 0.5); // Center
        self.eye_tracking.timestamp = Instant::now();

        // Update activity based on accelerometer magnitude
        let accel_mag = self.accelerometer.value.magnitude();
        self.activity = if accel_mag < 10.0 {
            ActivityState::Stationary
        } else if accel_mag < 12.0 {
            ActivityState::Walking
        } else if accel_mag < 20.0 {
            ActivityState::Running
        } else {
            ActivityState::Unknown
        };
    }

    /// Simulate walking motion
    pub fn simulate_walking(&mut self) {
        self.activity = ActivityState::Walking;
        let t = self.simulation_time;
        
        // Simulate step pattern
        self.accelerometer.value = Vec3::new(
            (t * 4.0).sin() * 0.5,
            -9.81 + (t * 8.0).sin().abs() * 2.0,
            (t * 4.0).cos() * 0.3,
        );
        
        // Count steps (simplified)
        if (t * 4.0).sin() > 0.9 {
            self.step_count += 1;
        }
        
        // Update location based on movement
        let speed = 1.4; // m/s walking speed
        let heading_rad = self.location.value.heading.to_radians();
        let delta_lat = (speed * heading_rad.cos() * 0.00001) as f64;
        let delta_lon = (speed * heading_rad.sin() * 0.00001) as f64;
        
        self.location.value.latitude += delta_lat;
        self.location.value.longitude += delta_lon;
        self.location.value.speed = speed;
    }

    /// Simulate head turning left
    pub fn simulate_look_left(&mut self) {
        self.gyroscope.value = Vec3::new(0.0, 2.0, 0.0); // Yaw rotation
        self.last_gesture = GestureType::LookLeft;
        self.gesture_time = Instant::now();
    }

    /// Simulate head turning right
    pub fn simulate_look_right(&mut self) {
        self.gyroscope.value = Vec3::new(0.0, -2.0, 0.0);
        self.last_gesture = GestureType::LookRight;
        self.gesture_time = Instant::now();
    }

    /// Simulate nodding yes
    pub fn simulate_nod_yes(&mut self) {
        self.gyroscope.value = Vec3::new(1.5, 0.0, 0.0); // Pitch rotation
        self.last_gesture = GestureType::NodYes;
        self.gesture_time = Instant::now();
    }

    /// Simulate shaking head no
    pub fn simulate_shake_no(&mut self) {
        let t = self.simulation_time;
        self.gyroscope.value = Vec3::new(0.0, (t * 10.0).sin() * 3.0, 0.0);
        self.last_gesture = GestureType::ShakeNo;
        self.gesture_time = Instant::now();
    }

    /// Simulate a tap gesture
    pub fn simulate_tap(&mut self) {
        self.last_gesture = GestureType::Tap;
        self.gesture_time = Instant::now();
    }

    /// Simulate swipe gesture
    pub fn simulate_swipe(&mut self, direction: GestureType) {
        self.last_gesture = direction;
        self.gesture_time = Instant::now();
    }

    /// Set ambient light level
    pub fn set_ambient_light(&mut self, lux: f32) {
        self.ambient_light = SensorReading::new(lux);
    }

    /// Set location
    pub fn set_location(&mut self, lat: f64, lon: f64) {
        self.location.value.latitude = lat;
        self.location.value.longitude = lon;
        self.location.timestamp = Instant::now();
    }

    /// Check if a gesture was just detected
    pub fn get_recent_gesture(&self) -> Option<GestureType> {
        if self.gesture_time.elapsed() < Duration::from_millis(500) {
            Some(self.last_gesture)
        } else {
            None
        }
    }

    /// Get head orientation as euler angles
    pub fn get_head_orientation(&self) -> Vec3 {
        self.orientation.value.to_euler()
    }

    /// Get current gaze point on screen
    pub fn get_gaze_point(&self) -> (f32, f32) {
        self.eye_tracking.value.gaze_point
    }

    /// Check if user is looking at a specific area
    pub fn is_looking_at(&self, x: f32, y: f32, radius: f32) -> bool {
        let (gx, gy) = self.eye_tracking.value.gaze_point;
        let dx = gx - x;
        let dy = gy - y;
        (dx * dx + dy * dy).sqrt() < radius
    }

    /// Get a status summary
    pub fn status_summary(&self) -> String {
        format!(
            "Activity: {:?} | Steps: {} | Light: {:.0} lux | Gesture: {:?}",
            self.activity,
            self.step_count,
            self.ambient_light.value,
            self.last_gesture
        )
    }
}

impl Default for VirtualSensors {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple pseudo-random number (0.0 to 1.0)
fn rand_simple() -> f32 {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((seed as f32 * 0.0000001) % 1.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensors_creation() {
        let sensors = VirtualSensors::new();
        assert!(sensors.is_active());
        assert_eq!(sensors.activity, ActivityState::Stationary);
    }

    #[test]
    fn test_walking_simulation() {
        let mut sensors = VirtualSensors::new();
        sensors.simulate_walking();
        assert_eq!(sensors.activity, ActivityState::Walking);
    }

    #[test]
    fn test_gesture_detection() {
        let mut sensors = VirtualSensors::new();
        sensors.simulate_tap();
        assert_eq!(sensors.get_recent_gesture(), Some(GestureType::Tap));
    }

    #[test]
    fn test_location() {
        let mut sensors = VirtualSensors::new();
        sensors.set_location(40.7128, -74.0060); // NYC
        assert!((sensors.location.value.latitude - 40.7128).abs() < 0.001);
    }

    #[test]
    fn test_vec3_magnitude() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 0.001);
    }
}
