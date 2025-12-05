//! AR Session Management for Kāraṇa OS
//! 
//! Manages AR session lifecycle, configuration, and state.

use super::*;
use super::tracking::{WorldTracker, CameraIntrinsics, TrackingState, PoseEstimate, CameraFrame, ImuMeasurement};
use super::plane::{PlaneDetector, PlaneDetectionConfig, SurfacePlane};
use super::mesh::{MeshReconstructor, MeshConfig, MeshChunk};
use super::lighting::{LightEstimator, LightEstimationConfig, LightEstimate};
use super::occlusion::{OcclusionHandler, OcclusionConfig, OcclusionResult, DepthFrame, SegmentationFrame};
use nalgebra::{Point3, Vector3};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// AR session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Session not initialized
    NotInitialized,
    /// Session initializing
    Initializing,
    /// Session running normally
    Running,
    /// Session paused
    Paused,
    /// Session stopped
    Stopped,
}

/// AR session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Enable world tracking
    pub world_tracking: bool,
    /// Enable plane detection
    pub plane_detection: bool,
    /// Plane detection config
    pub plane_config: PlaneDetectionConfig,
    /// Enable mesh reconstruction
    pub mesh_reconstruction: bool,
    /// Mesh config
    pub mesh_config: MeshConfig,
    /// Enable light estimation
    pub light_estimation: bool,
    /// Light estimation config
    pub light_config: LightEstimationConfig,
    /// Enable occlusion
    pub occlusion: bool,
    /// Occlusion config
    pub occlusion_config: OcclusionConfig,
    /// Target frame rate
    pub target_fps: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            world_tracking: true,
            plane_detection: true,
            plane_config: PlaneDetectionConfig::default(),
            mesh_reconstruction: false,
            mesh_config: MeshConfig::default(),
            light_estimation: true,
            light_config: LightEstimationConfig::default(),
            occlusion: true,
            occlusion_config: OcclusionConfig::default(),
            target_fps: 60,
        }
    }
}

/// AR frame data
#[derive(Debug, Clone)]
pub struct ArFrame {
    /// Frame ID
    pub id: u64,
    /// Timestamp
    pub timestamp_ns: u64,
    /// Camera pose
    pub pose: PoseEstimate,
    /// Tracking state
    pub tracking_state: TrackingState,
    /// Updated plane IDs
    pub updated_planes: Vec<u64>,
    /// Light estimate
    pub light_estimate: LightEstimate,
    /// Frame processing time
    pub processing_time: Duration,
}

/// Hit test result
#[derive(Debug, Clone)]
pub struct HitTestResult {
    /// Hit position
    pub position: Point3<f32>,
    /// Hit normal
    pub normal: Vector3<f32>,
    /// Hit distance
    pub distance: f32,
    /// Hit type
    pub hit_type: HitType,
    /// Associated plane ID (if any)
    pub plane_id: Option<u64>,
    /// Associated anchor ID (if any)
    pub anchor_id: Option<u64>,
}

/// Hit test type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitType {
    /// Hit a plane
    Plane,
    /// Hit a mesh
    Mesh,
    /// Hit a feature point
    FeaturePoint,
    /// Hit an anchor
    Anchor,
}

/// AR Session
#[derive(Debug)]
pub struct ArSession {
    /// Session ID
    pub id: u64,
    /// Session state
    state: SessionState,
    /// Configuration
    config: SessionConfig,
    /// Camera intrinsics
    intrinsics: CameraIntrinsics,
    /// World tracker
    world_tracker: WorldTracker,
    /// Plane detector
    plane_detector: PlaneDetector,
    /// Mesh reconstructor
    mesh_reconstructor: MeshReconstructor,
    /// Light estimator
    light_estimator: LightEstimator,
    /// Occlusion handler
    occlusion_handler: OcclusionHandler,
    /// Frame counter
    frame_count: u64,
    /// Start time
    start_time: Option<Instant>,
    /// Last frame time
    last_frame_time: Option<Instant>,
    /// Current pose
    current_pose: PoseEstimate,
}

impl ArSession {
    /// Create new AR session
    pub fn new(config: SessionConfig, intrinsics: CameraIntrinsics) -> Self {
        let id = super::next_content_id();
        
        Self {
            id,
            state: SessionState::NotInitialized,
            config: config.clone(),
            intrinsics,
            world_tracker: WorldTracker::new(intrinsics),
            plane_detector: PlaneDetector::new(config.plane_config.clone()),
            mesh_reconstructor: MeshReconstructor::new(config.mesh_config.clone()),
            light_estimator: LightEstimator::new(config.light_config.clone()),
            occlusion_handler: OcclusionHandler::new(config.occlusion_config.clone()),
            frame_count: 0,
            start_time: None,
            last_frame_time: None,
            current_pose: PoseEstimate::identity(),
        }
    }

    /// Start AR session
    pub fn start(&mut self) -> Result<(), String> {
        if self.state == SessionState::Running {
            return Ok(());
        }

        self.state = SessionState::Initializing;
        self.start_time = Some(Instant::now());
        self.frame_count = 0;

        // Start subsystems
        if self.config.plane_detection {
            self.plane_detector.start();
        }
        if self.config.mesh_reconstruction {
            self.mesh_reconstructor.start();
        }
        if self.config.light_estimation {
            self.light_estimator.start();
        }

        self.occlusion_handler.set_intrinsics(self.intrinsics);

        self.state = SessionState::Running;
        Ok(())
    }

    /// Pause AR session
    pub fn pause(&mut self) {
        if self.state == SessionState::Running {
            self.state = SessionState::Paused;
            self.plane_detector.stop();
            self.mesh_reconstructor.stop();
            self.light_estimator.stop();
        }
    }

    /// Resume AR session
    pub fn resume(&mut self) {
        if self.state == SessionState::Paused {
            self.state = SessionState::Running;
            
            if self.config.plane_detection {
                self.plane_detector.start();
            }
            if self.config.mesh_reconstruction {
                self.mesh_reconstructor.start();
            }
            if self.config.light_estimation {
                self.light_estimator.start();
            }
        }
    }

    /// Stop AR session
    pub fn stop(&mut self) {
        self.state = SessionState::Stopped;
        self.plane_detector.stop();
        self.mesh_reconstructor.stop();
        self.light_estimator.stop();
    }

    /// Get session state
    pub fn get_state(&self) -> SessionState {
        self.state
    }

    /// Process camera frame
    pub fn process_camera_frame(&mut self, frame: &CameraFrame) -> ArFrame {
        let start = Instant::now();
        self.frame_count += 1;

        // Update world tracking
        let tracking_state = if self.config.world_tracking {
            self.world_tracker.process_frame(frame)
        } else {
            TrackingState::NotAvailable
        };

        self.current_pose = self.world_tracker.get_pose();

        // Update plane detection
        let updated_planes = if self.config.plane_detection {
            // Add feature points for plane detection
            let features = self.world_tracker.get_features();
            let points: Vec<Point3<f32>> = features.iter()
                .filter_map(|f| f.position_3d)
                .collect();
            let normals: Vec<Vector3<f32>> = vec![Vector3::y(); points.len()];
            self.plane_detector.add_points(&points, &normals);
            self.plane_detector.update()
        } else {
            Vec::new()
        };

        // Update light estimation
        if self.config.light_estimation {
            self.light_estimator.process_frame(&frame.image, frame.width, frame.height);
        }

        // Update session state based on tracking
        if self.state == SessionState::Initializing {
            if matches!(tracking_state, TrackingState::Normal | TrackingState::Excellent) {
                self.state = SessionState::Running;
            }
        }

        let processing_time = start.elapsed();
        self.last_frame_time = Some(Instant::now());

        ArFrame {
            id: self.frame_count,
            timestamp_ns: frame.timestamp_ns,
            pose: self.current_pose,
            tracking_state,
            updated_planes,
            light_estimate: self.light_estimator.get_estimate().clone(),
            processing_time,
        }
    }

    /// Process IMU measurement
    pub fn process_imu(&mut self, measurement: ImuMeasurement) {
        if self.config.world_tracking && self.state == SessionState::Running {
            self.world_tracker.process_imu(measurement);
        }
    }

    /// Process depth frame
    pub fn process_depth_frame(&mut self, frame: DepthFrame) {
        if self.config.occlusion {
            self.occlusion_handler.update_depth(frame.clone());
        }

        if self.config.mesh_reconstruction {
            let pose = Transform {
                position: self.current_pose.position,
                rotation: self.current_pose.orientation,
                scale: Vector3::new(1.0, 1.0, 1.0),
            };
            self.mesh_reconstructor.process_depth_frame(
                &frame.data,
                frame.width,
                frame.height,
                &pose,
                &self.intrinsics,
            );
        }
    }

    /// Process segmentation frame
    pub fn process_segmentation_frame(&mut self, frame: SegmentationFrame) {
        if self.config.occlusion {
            self.occlusion_handler.update_segmentation(frame);
        }
    }

    /// Get current pose
    pub fn get_pose(&self) -> PoseEstimate {
        self.current_pose
    }

    /// Get tracking state
    pub fn get_tracking_state(&self) -> TrackingState {
        if self.config.world_tracking {
            self.world_tracker.state
        } else {
            TrackingState::NotAvailable
        }
    }

    /// Get tracking quality
    pub fn get_tracking_quality(&self) -> f32 {
        self.world_tracker.get_tracking_quality()
    }

    /// Get all detected planes
    pub fn get_planes(&self) -> Vec<&SurfacePlane> {
        self.plane_detector.get_planes()
    }

    /// Get plane by ID
    pub fn get_plane(&self, id: u64) -> Option<&SurfacePlane> {
        self.plane_detector.get_plane(id)
    }

    /// Get floor plane
    pub fn get_floor(&self) -> Option<&SurfacePlane> {
        self.plane_detector.get_floor()
    }

    /// Get mesh chunks
    pub fn get_mesh_chunks(&self) -> Vec<&MeshChunk> {
        self.mesh_reconstructor.get_chunks()
    }

    /// Get light estimate
    pub fn get_light_estimate(&self) -> &LightEstimate {
        self.light_estimator.get_estimate()
    }

    /// Hit test at screen point
    pub fn hit_test_screen(&self, x: f32, y: f32) -> Vec<HitTestResult> {
        // Unproject screen point to ray
        let ray_dir = self.intrinsics.unproject(x, y);
        let ray_origin = self.current_pose.position;
        let ray_direction = self.current_pose.orientation * ray_dir;

        self.hit_test_ray(ray_origin, ray_direction)
    }

    /// Hit test with ray
    pub fn hit_test_ray(&self, origin: Point3<f32>, direction: Vector3<f32>) -> Vec<HitTestResult> {
        let mut results = Vec::new();

        // Test against planes
        if let Some((plane_id, hit_point)) = self.plane_detector.hit_test(origin, direction) {
            if let Some(plane) = self.plane_detector.get_plane(plane_id) {
                results.push(HitTestResult {
                    position: hit_point,
                    normal: plane.normal,
                    distance: (hit_point - origin).norm(),
                    hit_type: HitType::Plane,
                    plane_id: Some(plane_id),
                    anchor_id: None,
                });
            }
        }

        // Test against mesh
        if self.config.mesh_reconstruction {
            if let Some((hit_point, normal)) = self.mesh_reconstructor.raycast(origin, direction) {
                results.push(HitTestResult {
                    position: hit_point,
                    normal,
                    distance: (hit_point - origin).norm(),
                    hit_type: HitType::Mesh,
                    plane_id: None,
                    anchor_id: None,
                });
            }
        }

        // Sort by distance
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        results
    }

    /// Test occlusion for point
    pub fn test_occlusion(&self, point: Point3<f32>) -> OcclusionResult {
        let pose = Transform {
            position: self.current_pose.position,
            rotation: self.current_pose.orientation,
            scale: Vector3::new(1.0, 1.0, 1.0),
        };
        self.occlusion_handler.test_point(point, &pose)
    }

    /// Get frame rate
    pub fn get_fps(&self) -> f32 {
        if let (Some(start), Some(_last)) = (self.start_time, self.last_frame_time) {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed > 0.0 {
                self.frame_count as f32 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get frame count
    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get session duration
    pub fn get_duration(&self) -> Duration {
        self.start_time.map(|s| s.elapsed()).unwrap_or_default()
    }

    /// Reset session
    pub fn reset(&mut self) {
        self.world_tracker.reset();
        self.plane_detector.clear();
        self.mesh_reconstructor.clear();
        self.frame_count = 0;
        self.start_time = Some(Instant::now());
        self.current_pose = PoseEstimate::identity();
        self.state = SessionState::Initializing;
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SessionConfig) {
        self.config = config.clone();

        // Update subsystem configs
        if config.plane_detection && !self.plane_detector.is_running() {
            self.plane_detector.start();
        } else if !config.plane_detection && self.plane_detector.is_running() {
            self.plane_detector.stop();
        }

        if config.mesh_reconstruction && !self.mesh_reconstructor.is_running() {
            self.mesh_reconstructor.start();
        } else if !config.mesh_reconstruction && self.mesh_reconstructor.is_running() {
            self.mesh_reconstructor.stop();
        }

        if config.light_estimation && !self.light_estimator.is_running() {
            self.light_estimator.start();
        } else if !config.light_estimation && self.light_estimator.is_running() {
            self.light_estimator.stop();
        }

        if config.occlusion {
            self.occlusion_handler.enable();
        } else {
            self.occlusion_handler.disable();
        }
    }
}

/// AR Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Frame count
    pub frame_count: u64,
    /// Average FPS
    pub avg_fps: f32,
    /// Tracking quality
    pub tracking_quality: f32,
    /// Plane count
    pub plane_count: usize,
    /// Mesh vertex count
    pub mesh_vertex_count: usize,
    /// Feature count
    pub feature_count: usize,
    /// Session duration
    pub duration: Duration,
}

impl ArSession {
    /// Get session statistics
    pub fn get_stats(&self) -> SessionStats {
        SessionStats {
            frame_count: self.frame_count,
            avg_fps: self.get_fps(),
            tracking_quality: self.get_tracking_quality(),
            plane_count: self.plane_detector.plane_count(),
            mesh_vertex_count: self.mesh_reconstructor.total_vertex_count(),
            feature_count: self.world_tracker.get_feature_count(),
            duration: self.get_duration(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_intrinsics() -> CameraIntrinsics {
        CameraIntrinsics::new(500.0, 500.0, 320.0, 240.0, 640, 480)
    }

    #[test]
    fn test_session_creation() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let session = ArSession::new(config, intrinsics);
        assert_eq!(session.get_state(), SessionState::NotInitialized);
    }

    #[test]
    fn test_session_start() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let mut session = ArSession::new(config, intrinsics);
        
        let result = session.start();
        assert!(result.is_ok());
        assert_eq!(session.get_state(), SessionState::Running);
    }

    #[test]
    fn test_session_pause_resume() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let mut session = ArSession::new(config, intrinsics);
        
        session.start().unwrap();
        session.pause();
        assert_eq!(session.get_state(), SessionState::Paused);
        
        session.resume();
        assert_eq!(session.get_state(), SessionState::Running);
    }

    #[test]
    fn test_session_stop() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let mut session = ArSession::new(config, intrinsics);
        
        session.start().unwrap();
        session.stop();
        assert_eq!(session.get_state(), SessionState::Stopped);
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert!(config.world_tracking);
        assert!(config.plane_detection);
        assert!(config.light_estimation);
    }

    #[test]
    fn test_hit_test_result() {
        let result = HitTestResult {
            position: Point3::origin(),
            normal: Vector3::y(),
            distance: 1.0,
            hit_type: HitType::Plane,
            plane_id: Some(1),
            anchor_id: None,
        };
        assert_eq!(result.hit_type, HitType::Plane);
    }

    #[test]
    fn test_session_stats() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let session = ArSession::new(config, intrinsics);
        
        let stats = session.get_stats();
        assert_eq!(stats.frame_count, 0);
    }

    #[test]
    fn test_session_reset() {
        let config = SessionConfig::default();
        let intrinsics = create_test_intrinsics();
        let mut session = ArSession::new(config, intrinsics);
        
        session.start().unwrap();
        session.reset();
        assert_eq!(session.get_frame_count(), 0);
    }
}
