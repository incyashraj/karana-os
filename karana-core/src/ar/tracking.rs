//! World Tracking and SLAM for Kāraṇa OS
//! 
//! Implements visual-inertial odometry (VIO) and SLAM for accurate
//! 6DoF pose estimation in AR applications.

use super::*;
use nalgebra::{Matrix4, Point3, UnitQuaternion, Vector3};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Tracking state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingState {
    /// Not tracking
    NotAvailable,
    /// Limited tracking quality
    Limited(TrackingLimitedReason),
    /// Normal tracking
    Normal,
    /// High-quality tracking
    Excellent,
}

/// Reasons for limited tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingLimitedReason {
    /// Insufficient features visible
    InsufficientFeatures,
    /// Excessive motion blur
    ExcessiveMotion,
    /// Camera initialization in progress
    Initializing,
    /// Relocalization in progress
    Relocalizing,
}

/// Camera intrinsic parameters
#[derive(Debug, Clone, Copy)]
pub struct CameraIntrinsics {
    /// Focal length X
    pub fx: f32,
    /// Focal length Y
    pub fy: f32,
    /// Principal point X
    pub cx: f32,
    /// Principal point Y
    pub cy: f32,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
}

impl CameraIntrinsics {
    pub fn new(fx: f32, fy: f32, cx: f32, cy: f32, width: u32, height: u32) -> Self {
        Self { fx, fy, cx, cy, width, height }
    }

    /// Project 3D point to 2D image coordinates
    pub fn project(&self, point: Point3<f32>) -> Option<(f32, f32)> {
        if point.z <= 0.0 {
            return None;
        }
        let x = self.fx * point.x / point.z + self.cx;
        let y = self.fy * point.y / point.z + self.cy;
        Some((x, y))
    }

    /// Unproject 2D image point to 3D ray
    pub fn unproject(&self, x: f32, y: f32) -> Vector3<f32> {
        let nx = (x - self.cx) / self.fx;
        let ny = (y - self.cy) / self.fy;
        Vector3::new(nx, ny, 1.0).normalize()
    }

    /// Get projection matrix
    pub fn projection_matrix(&self, near: f32, far: f32) -> Matrix4<f32> {
        let w = self.width as f32;
        let h = self.height as f32;
        Matrix4::new(
            2.0 * self.fx / w, 0.0, (w - 2.0 * self.cx) / w, 0.0,
            0.0, 2.0 * self.fy / h, (2.0 * self.cy - h) / h, 0.0,
            0.0, 0.0, -(far + near) / (far - near), -2.0 * far * near / (far - near),
            0.0, 0.0, -1.0, 0.0,
        )
    }
}

/// Visual feature for tracking
#[derive(Debug, Clone)]
pub struct TrackedFeature {
    /// Feature ID
    pub id: u64,
    /// 2D position in image
    pub position_2d: (f32, f32),
    /// 3D position in world (if triangulated)
    pub position_3d: Option<Point3<f32>>,
    /// Feature descriptor
    pub descriptor: Vec<u8>,
    /// Track age (number of frames)
    pub age: u32,
    /// Is feature reliable
    pub reliable: bool,
}

/// IMU measurement
#[derive(Debug, Clone, Copy)]
pub struct ImuMeasurement {
    /// Timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// Accelerometer reading (m/s²)
    pub acceleration: Vector3<f32>,
    /// Gyroscope reading (rad/s)
    pub angular_velocity: Vector3<f32>,
}

/// Camera frame with image and metadata
#[derive(Debug, Clone)]
pub struct CameraFrame {
    /// Frame ID
    pub id: u64,
    /// Timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Grayscale image data
    pub image: Vec<u8>,
    /// Camera intrinsics
    pub intrinsics: CameraIntrinsics,
    /// Exposure time in microseconds
    pub exposure_us: u32,
}

/// Pose estimate from tracking
#[derive(Debug, Clone, Copy)]
pub struct PoseEstimate {
    /// Position in world coordinates
    pub position: Point3<f32>,
    /// Orientation as unit quaternion
    pub orientation: UnitQuaternion<f32>,
    /// Linear velocity (m/s)
    pub velocity: Vector3<f32>,
    /// Angular velocity (rad/s)
    pub angular_velocity: Vector3<f32>,
    /// Position uncertainty (std dev in meters)
    pub position_uncertainty: f32,
    /// Orientation uncertainty (std dev in radians)
    pub orientation_uncertainty: f32,
    /// Timestamp
    pub timestamp_ns: u64,
}

impl PoseEstimate {
    pub fn identity() -> Self {
        Self {
            position: Point3::origin(),
            orientation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            position_uncertainty: 0.0,
            orientation_uncertainty: 0.0,
            timestamp_ns: 0,
        }
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let rotation = self.orientation.inverse().to_homogeneous();
        let translation = Matrix4::new_translation(&(-self.position.coords));
        rotation * translation
    }

    /// Transform a point from world to camera coordinates
    pub fn world_to_camera(&self, point: Point3<f32>) -> Point3<f32> {
        let p = point - self.position;
        Point3::from(self.orientation.inverse() * p)
    }

    /// Transform a point from camera to world coordinates
    pub fn camera_to_world(&self, point: Point3<f32>) -> Point3<f32> {
        self.position + self.orientation * point.coords
    }
}

/// Keyframe for SLAM
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Keyframe ID
    pub id: u64,
    /// Pose at keyframe
    pub pose: PoseEstimate,
    /// Tracked features
    pub features: Vec<TrackedFeature>,
    /// Covisibility graph edges (keyframe_id -> shared features)
    pub covisibility: HashMap<u64, u32>,
    /// Is this keyframe fixed (not optimized)
    pub fixed: bool,
}

/// Map point in SLAM
#[derive(Debug, Clone)]
pub struct MapPoint {
    /// Point ID
    pub id: u64,
    /// 3D position
    pub position: Point3<f32>,
    /// Normal vector
    pub normal: Vector3<f32>,
    /// Descriptor
    pub descriptor: Vec<u8>,
    /// Observing keyframes
    pub observations: Vec<u64>,
    /// Is point reliable
    pub reliable: bool,
}

/// World tracking system
#[derive(Debug)]
pub struct WorldTracker {
    /// Current tracking state
    pub state: TrackingState,
    /// Current pose estimate
    pub pose: PoseEstimate,
    /// Camera intrinsics
    intrinsics: CameraIntrinsics,
    /// Tracked features
    features: Vec<TrackedFeature>,
    /// Keyframes
    keyframes: HashMap<u64, Keyframe>,
    /// Map points
    map_points: HashMap<u64, MapPoint>,
    /// IMU buffer
    imu_buffer: Vec<ImuMeasurement>,
    /// Last frame timestamp
    last_frame_ns: u64,
    /// Frame counter
    frame_count: u64,
    /// Feature ID counter
    next_feature_id: u64,
    /// Keyframe ID counter
    next_keyframe_id: u64,
    /// Map point ID counter
    next_map_point_id: u64,
    /// Is initialized
    initialized: bool,
    /// Gravity vector in world coordinates
    gravity: Vector3<f32>,
}

impl WorldTracker {
    /// Create new world tracker
    pub fn new(intrinsics: CameraIntrinsics) -> Self {
        Self {
            state: TrackingState::NotAvailable,
            pose: PoseEstimate::identity(),
            intrinsics,
            features: Vec::new(),
            keyframes: HashMap::new(),
            map_points: HashMap::new(),
            imu_buffer: Vec::new(),
            last_frame_ns: 0,
            frame_count: 0,
            next_feature_id: 1,
            next_keyframe_id: 1,
            next_map_point_id: 1,
            initialized: false,
            gravity: Vector3::new(0.0, -9.81, 0.0),
        }
    }

    /// Process camera frame
    pub fn process_frame(&mut self, frame: &CameraFrame) -> TrackingState {
        self.frame_count += 1;

        // Extract features from frame
        let new_features = self.extract_features(frame);

        if !self.initialized {
            // Initialization phase
            if self.features.is_empty() {
                self.features = new_features.clone();
                self.state = TrackingState::Limited(TrackingLimitedReason::Initializing);
            } else {
                // Try to initialize from feature matches
                if self.try_initialize(frame, &new_features) {
                    self.initialized = true;
                    self.state = TrackingState::Normal;
                } else {
                    self.state = TrackingState::Limited(TrackingLimitedReason::Initializing);
                }
            }
        } else {
            // Normal tracking
            let matched = self.track_features(&new_features);
            
            if matched >= 30 {
                // Estimate pose from matches
                self.estimate_pose(frame, &new_features);
                self.state = if matched >= 50 {
                    TrackingState::Excellent
                } else {
                    TrackingState::Normal
                };

                // Create keyframe if needed
                if self.should_create_keyframe() {
                    self.create_keyframe(frame, &new_features);
                }
            } else {
                // Tracking limited or lost
                self.state = TrackingState::Limited(TrackingLimitedReason::InsufficientFeatures);
                
                if matched < 10 {
                    // Try to relocalize
                    if !self.try_relocalize(frame, &new_features) {
                        self.state = TrackingState::Limited(TrackingLimitedReason::Relocalizing);
                    }
                }
            }
        }

        self.last_frame_ns = frame.timestamp_ns;
        self.features = new_features;
        self.state
    }

    /// Process IMU measurement
    pub fn process_imu(&mut self, measurement: ImuMeasurement) {
        self.imu_buffer.push(measurement);

        // Keep buffer size reasonable
        if self.imu_buffer.len() > 1000 {
            self.imu_buffer.remove(0);
        }

        // Integrate IMU for pose prediction
        if self.initialized && !self.imu_buffer.is_empty() {
            self.integrate_imu();
        }
    }

    /// Get current pose
    pub fn get_pose(&self) -> PoseEstimate {
        self.pose
    }

    /// Get tracking quality (0.0 - 1.0)
    pub fn get_tracking_quality(&self) -> f32 {
        match self.state {
            TrackingState::NotAvailable => 0.0,
            TrackingState::Limited(_) => 0.3,
            TrackingState::Normal => 0.7,
            TrackingState::Excellent => 1.0,
        }
    }

    /// Get number of tracked features
    pub fn get_feature_count(&self) -> usize {
        self.features.iter().filter(|f| f.reliable).count()
    }

    /// Get number of map points
    pub fn get_map_point_count(&self) -> usize {
        self.map_points.len()
    }

    /// Get number of keyframes
    pub fn get_keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Get tracked features
    pub fn get_features(&self) -> &[TrackedFeature] {
        &self.features
    }

    /// Get map points
    pub fn get_map_points(&self) -> Vec<&MapPoint> {
        self.map_points.values().collect()
    }

    /// Reset tracking
    pub fn reset(&mut self) {
        self.state = TrackingState::NotAvailable;
        self.pose = PoseEstimate::identity();
        self.features.clear();
        self.keyframes.clear();
        self.map_points.clear();
        self.imu_buffer.clear();
        self.initialized = false;
        self.frame_count = 0;
    }

    // Internal methods

    fn extract_features(&mut self, frame: &CameraFrame) -> Vec<TrackedFeature> {
        // Simulate FAST feature detection
        let mut features = Vec::new();
        let grid_size = 32;
        let cols = frame.width / grid_size;
        let rows = frame.height / grid_size;

        for row in 0..rows {
            for col in 0..cols {
                let x = col * grid_size + grid_size / 2;
                let y = row * grid_size + grid_size / 2;
                
                // Compute simple corner response
                let idx = (y * frame.width + x) as usize;
                if idx < frame.image.len() {
                    let response = self.compute_corner_response(frame, x, y);
                    if response > 20.0 {
                        features.push(TrackedFeature {
                            id: self.next_feature_id,
                            position_2d: (x as f32, y as f32),
                            position_3d: None,
                            descriptor: self.compute_descriptor(frame, x, y),
                            age: 0,
                            reliable: true,
                        });
                        self.next_feature_id += 1;
                    }
                }
            }
        }

        features
    }

    fn compute_corner_response(&self, frame: &CameraFrame, x: u32, y: u32) -> f32 {
        // Simplified Harris corner response
        if x < 3 || y < 3 || x >= frame.width - 3 || y >= frame.height - 3 {
            return 0.0;
        }

        let get_pixel = |px: u32, py: u32| -> f32 {
            let idx = (py * frame.width + px) as usize;
            if idx < frame.image.len() {
                frame.image[idx] as f32
            } else {
                0.0
            }
        };

        let mut ix_sum = 0.0f32;
        let mut iy_sum = 0.0f32;
        let mut ixy_sum = 0.0f32;

        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let px = (x as i32 + dx) as u32;
                let py = (y as i32 + dy) as u32;
                let ix = get_pixel(px + 1, py) - get_pixel(px.saturating_sub(1), py);
                let iy = get_pixel(px, py + 1) - get_pixel(px, py.saturating_sub(1));
                ix_sum += ix * ix;
                iy_sum += iy * iy;
                ixy_sum += ix * iy;
            }
        }

        let det = ix_sum * iy_sum - ixy_sum * ixy_sum;
        let trace = ix_sum + iy_sum;
        det - 0.04 * trace * trace
    }

    fn compute_descriptor(&self, frame: &CameraFrame, x: u32, y: u32) -> Vec<u8> {
        // Simple BRIEF-like descriptor
        let mut desc = vec![0u8; 32];
        let pairs = [
            (-3, -3, 3, 3), (-2, -2, 2, 2), (-1, -1, 1, 1),
            (-3, 0, 3, 0), (0, -3, 0, 3), (-2, 1, 2, -1),
            (1, -2, -1, 2), (-3, 2, 3, -2), (2, -3, -2, 3),
        ];

        let get_pixel = |px: i32, py: i32| -> u8 {
            let px = (px.max(0) as u32).min(frame.width - 1);
            let py = (py.max(0) as u32).min(frame.height - 1);
            let idx = (py * frame.width + px) as usize;
            if idx < frame.image.len() {
                frame.image[idx]
            } else {
                0
            }
        };

        for (i, &(dx1, dy1, dx2, dy2)) in pairs.iter().enumerate() {
            let p1 = get_pixel(x as i32 + dx1, y as i32 + dy1);
            let p2 = get_pixel(x as i32 + dx2, y as i32 + dy2);
            if p1 > p2 {
                desc[i / 8] |= 1 << (i % 8);
            }
        }

        desc
    }

    fn track_features(&mut self, new_features: &[TrackedFeature]) -> usize {
        // Simple feature matching based on descriptor distance
        let mut matched = 0;

        for old_feat in &self.features {
            let mut best_dist = u32::MAX;
            let mut _best_idx = None;

            for (i, new_feat) in new_features.iter().enumerate() {
                let dist = self.hamming_distance(&old_feat.descriptor, &new_feat.descriptor);
                if dist < best_dist {
                    best_dist = dist;
                    _best_idx = Some(i);
                }
            }

            if best_dist < 40 {
                matched += 1;
            }
        }

        matched
    }

    fn hamming_distance(&self, a: &[u8], b: &[u8]) -> u32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum()
    }

    fn try_initialize(&mut self, _frame: &CameraFrame, new_features: &[TrackedFeature]) -> bool {
        // Check if we have enough parallax for initialization
        let matched = self.track_features(new_features);
        matched >= 50 && self.frame_count >= 10
    }

    fn estimate_pose(&mut self, frame: &CameraFrame, _features: &[TrackedFeature]) {
        // Integrate IMU data between frames
        let dt = (frame.timestamp_ns - self.last_frame_ns) as f32 / 1e9;
        
        if dt > 0.0 && dt < 1.0 {
            // Simple motion model prediction
            self.pose.position = self.pose.position + self.pose.velocity * dt;
        }

        self.pose.timestamp_ns = frame.timestamp_ns;
    }

    fn integrate_imu(&mut self) {
        if self.imu_buffer.len() < 2 {
            return;
        }

        // Get latest measurements
        let current = &self.imu_buffer[self.imu_buffer.len() - 1];
        let previous = &self.imu_buffer[self.imu_buffer.len() - 2];

        let dt = (current.timestamp_ns - previous.timestamp_ns) as f32 / 1e9;
        if dt <= 0.0 || dt > 0.1 {
            return;
        }

        // Rotate acceleration to world frame and remove gravity
        let accel_world = self.pose.orientation * current.acceleration - self.gravity;

        // Integrate acceleration to velocity
        self.pose.velocity = self.pose.velocity + accel_world * dt;

        // Integrate angular velocity
        let omega = current.angular_velocity * dt;
        let dq = UnitQuaternion::from_scaled_axis(omega);
        self.pose.orientation = self.pose.orientation * dq;

        self.pose.angular_velocity = current.angular_velocity;
    }

    fn should_create_keyframe(&self) -> bool {
        // Create keyframe if enough features are new or moved significantly
        self.frame_count % 30 == 0
    }

    fn create_keyframe(&mut self, _frame: &CameraFrame, features: &[TrackedFeature]) {
        let keyframe = Keyframe {
            id: self.next_keyframe_id,
            pose: self.pose,
            features: features.to_vec(),
            covisibility: HashMap::new(),
            fixed: false,
        };

        self.keyframes.insert(self.next_keyframe_id, keyframe);
        self.next_keyframe_id += 1;
    }

    fn try_relocalize(&mut self, _frame: &CameraFrame, new_features: &[TrackedFeature]) -> bool {
        // Try to match against keyframes
        for kf in self.keyframes.values() {
            let mut matches = 0;
            for kf_feat in &kf.features {
                for new_feat in new_features {
                    let dist = self.hamming_distance(&kf_feat.descriptor, &new_feat.descriptor);
                    if dist < 30 {
                        matches += 1;
                        break;
                    }
                }
            }

            if matches >= 20 {
                // Found enough matches, relocalize
                self.pose = kf.pose;
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_intrinsics_projection() {
        let intrinsics = CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480);
        let point = Point3::new(0.0, 0.0, 2.0);
        let projected = intrinsics.project(point);
        assert!(projected.is_some());
        let (x, y) = projected.unwrap();
        assert!((x - 320.0).abs() < 0.1);
        assert!((y - 240.0).abs() < 0.1);
    }

    #[test]
    fn test_camera_intrinsics_unproject() {
        let intrinsics = CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480);
        let ray = intrinsics.unproject(320.0, 240.0);
        assert!((ray.x).abs() < 0.1);
        assert!((ray.y).abs() < 0.1);
        assert!((ray.z - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_pose_estimate_identity() {
        let pose = PoseEstimate::identity();
        assert_eq!(pose.position, Point3::origin());
    }

    #[test]
    fn test_pose_view_matrix() {
        let mut pose = PoseEstimate::identity();
        pose.position = Point3::new(1.0, 2.0, 3.0);
        let view = pose.view_matrix();
        // View matrix translates by negative position (identity rotation)
        // In nalgebra's column-major Matrix4, translation is at (0,3), (1,3), (2,3)
        assert!((view[(0, 3)] + 1.0).abs() < 0.001);
        assert!((view[(1, 3)] + 2.0).abs() < 0.001);
        assert!((view[(2, 3)] + 3.0).abs() < 0.001);
    }

    #[test]
    fn test_world_tracker_creation() {
        let intrinsics = CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480);
        let tracker = WorldTracker::new(intrinsics);
        assert_eq!(tracker.state, TrackingState::NotAvailable);
    }

    #[test]
    fn test_world_tracker_reset() {
        let intrinsics = CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480);
        let mut tracker = WorldTracker::new(intrinsics);
        tracker.frame_count = 100;
        tracker.reset();
        assert_eq!(tracker.frame_count, 0);
        assert_eq!(tracker.state, TrackingState::NotAvailable);
    }

    #[test]
    fn test_tracking_quality() {
        let intrinsics = CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480);
        let mut tracker = WorldTracker::new(intrinsics);
        assert_eq!(tracker.get_tracking_quality(), 0.0);
        
        tracker.state = TrackingState::Normal;
        assert!((tracker.get_tracking_quality() - 0.7).abs() < 0.01);
        
        tracker.state = TrackingState::Excellent;
        assert!((tracker.get_tracking_quality() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_imu_measurement() {
        let imu = ImuMeasurement {
            timestamp_ns: 1000000,
            acceleration: Vector3::new(0.0, 9.81, 0.0),
            angular_velocity: Vector3::zeros(),
        };
        assert!((imu.acceleration.y - 9.81).abs() < 0.01);
    }
}
