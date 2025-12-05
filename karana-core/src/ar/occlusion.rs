//! Occlusion Handling for Kāraṇa OS
//! 
//! Handles real-world occlusion of virtual objects for realistic AR.

use super::*;
use nalgebra::{Point3, Vector3};
use std::time::Instant;

/// Occlusion mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcclusionMode {
    /// No occlusion
    None,
    /// Environment depth occlusion
    EnvironmentDepth,
    /// Human segmentation occlusion
    HumanSegmentation,
    /// Combined occlusion
    Combined,
}

/// Segmentation type for human occlusion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentationType {
    /// No segmentation
    None,
    /// Person (full body)
    Person,
    /// Hair
    Hair,
    /// Skin
    Skin,
    /// Glass (eyewear)
    Glass,
}

/// Depth frame for occlusion
#[derive(Debug, Clone)]
pub struct DepthFrame {
    /// Frame ID
    pub id: u64,
    /// Timestamp
    pub timestamp_ns: u64,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Depth data (meters)
    pub data: Vec<f32>,
    /// Confidence data (0.0-1.0)
    pub confidence: Vec<f32>,
}

impl DepthFrame {
    /// Create new depth frame
    pub fn new(id: u64, width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            id,
            timestamp_ns: 0,
            width,
            height,
            data: vec![0.0; size],
            confidence: vec![0.0; size],
        }
    }

    /// Get depth at pixel
    pub fn get_depth(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.data.len() {
            self.data[idx]
        } else {
            0.0
        }
    }

    /// Get confidence at pixel
    pub fn get_confidence(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.confidence.len() {
            self.confidence[idx]
        } else {
            0.0
        }
    }

    /// Sample depth with bilinear interpolation
    pub fn sample_bilinear(&self, u: f32, v: f32) -> f32 {
        let x = u * (self.width - 1) as f32;
        let y = v * (self.height - 1) as f32;

        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let d00 = self.get_depth(x0, y0);
        let d10 = self.get_depth(x1, y0);
        let d01 = self.get_depth(x0, y1);
        let d11 = self.get_depth(x1, y1);

        let d0 = d00 * (1.0 - fx) + d10 * fx;
        let d1 = d01 * (1.0 - fx) + d11 * fx;

        d0 * (1.0 - fy) + d1 * fy
    }
}

/// Segmentation mask frame
#[derive(Debug, Clone)]
pub struct SegmentationFrame {
    /// Frame ID
    pub id: u64,
    /// Timestamp
    pub timestamp_ns: u64,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Mask data (0-255)
    pub mask: Vec<u8>,
    /// Segmentation type
    pub segmentation_type: SegmentationType,
}

impl SegmentationFrame {
    /// Create new segmentation frame
    pub fn new(id: u64, width: u32, height: u32, seg_type: SegmentationType) -> Self {
        let size = (width * height) as usize;
        Self {
            id,
            timestamp_ns: 0,
            width,
            height,
            mask: vec![0; size],
            segmentation_type: seg_type,
        }
    }

    /// Get mask value at pixel
    pub fn get_mask(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.mask.len() {
            self.mask[idx]
        } else {
            0
        }
    }

    /// Check if pixel is occluded
    pub fn is_occluded(&self, x: u32, y: u32, threshold: u8) -> bool {
        self.get_mask(x, y) > threshold
    }
}

/// Occlusion test result
#[derive(Debug, Clone, Copy)]
pub struct OcclusionResult {
    /// Is point occluded
    pub occluded: bool,
    /// Occlusion factor (0.0 = fully visible, 1.0 = fully occluded)
    pub factor: f32,
    /// Distance to occluder (meters)
    pub occluder_distance: f32,
    /// Type of occlusion
    pub occlusion_type: OcclusionType,
}

/// Type of occlusion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcclusionType {
    /// Not occluded
    None,
    /// Occluded by environment (depth)
    Environment,
    /// Occluded by human
    Human,
    /// Partially occluded
    Partial,
}

impl OcclusionResult {
    /// No occlusion
    pub fn visible() -> Self {
        Self {
            occluded: false,
            factor: 0.0,
            occluder_distance: f32::MAX,
            occlusion_type: OcclusionType::None,
        }
    }

    /// Full occlusion
    pub fn occluded(distance: f32, occ_type: OcclusionType) -> Self {
        Self {
            occluded: true,
            factor: 1.0,
            occluder_distance: distance,
            occlusion_type: occ_type,
        }
    }

    /// Partial occlusion
    pub fn partial(factor: f32, distance: f32) -> Self {
        Self {
            occluded: factor > 0.5,
            factor,
            occluder_distance: distance,
            occlusion_type: OcclusionType::Partial,
        }
    }
}

/// Occlusion handler configuration
#[derive(Debug, Clone)]
pub struct OcclusionConfig {
    /// Occlusion mode
    pub mode: OcclusionMode,
    /// Depth tolerance (meters)
    pub depth_tolerance: f32,
    /// Segmentation threshold (0-255)
    pub segmentation_threshold: u8,
    /// Edge smoothing
    pub edge_smoothing: bool,
    /// Smooth kernel size
    pub smooth_kernel_size: u32,
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            mode: OcclusionMode::EnvironmentDepth,
            depth_tolerance: 0.02,
            segmentation_threshold: 128,
            edge_smoothing: true,
            smooth_kernel_size: 3,
        }
    }
}

/// Occlusion handler
#[derive(Debug)]
pub struct OcclusionHandler {
    /// Configuration
    config: OcclusionConfig,
    /// Current depth frame
    depth_frame: Option<DepthFrame>,
    /// Current segmentation frame
    segmentation_frame: Option<SegmentationFrame>,
    /// Camera intrinsics
    intrinsics: Option<super::tracking::CameraIntrinsics>,
    /// Last update time
    last_update: Instant,
    /// Is enabled
    enabled: bool,
}

impl OcclusionHandler {
    /// Create new occlusion handler
    pub fn new(config: OcclusionConfig) -> Self {
        Self {
            config,
            depth_frame: None,
            segmentation_frame: None,
            intrinsics: None,
            last_update: Instant::now(),
            enabled: true,
        }
    }

    /// Enable occlusion
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable occlusion
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set camera intrinsics
    pub fn set_intrinsics(&mut self, intrinsics: super::tracking::CameraIntrinsics) {
        self.intrinsics = Some(intrinsics);
    }

    /// Update depth frame
    pub fn update_depth(&mut self, frame: DepthFrame) {
        self.depth_frame = Some(frame);
        self.last_update = Instant::now();
    }

    /// Update segmentation frame
    pub fn update_segmentation(&mut self, frame: SegmentationFrame) {
        self.segmentation_frame = Some(frame);
        self.last_update = Instant::now();
    }

    /// Test occlusion for a 3D point
    pub fn test_point(&self, point: Point3<f32>, camera_pose: &Transform) -> OcclusionResult {
        if !self.enabled || self.config.mode == OcclusionMode::None {
            return OcclusionResult::visible();
        }

        // Transform point to camera space
        let camera_vec = camera_pose.rotation.inverse() * (point - camera_pose.position);
        let camera_point = Point3::new(camera_vec.x, camera_vec.y, camera_vec.z);

        // Point behind camera
        if camera_point.z <= 0.0 {
            return OcclusionResult::visible();
        }

        // Project to image space
        let intrinsics = match &self.intrinsics {
            Some(i) => i,
            None => return OcclusionResult::visible(),
        };

        let projected = match intrinsics.project(camera_point) {
            Some(p) => p,
            None => return OcclusionResult::visible(),
        };

        // Check bounds
        let (u, v) = projected;
        if u < 0.0 || v < 0.0 || u >= intrinsics.width as f32 || v >= intrinsics.height as f32 {
            return OcclusionResult::visible();
        }

        let point_depth = camera_point.z;

        // Test based on mode
        match self.config.mode {
            OcclusionMode::None => OcclusionResult::visible(),
            OcclusionMode::EnvironmentDepth => self.test_depth_occlusion(u, v, point_depth),
            OcclusionMode::HumanSegmentation => self.test_human_occlusion(u as u32, v as u32),
            OcclusionMode::Combined => self.test_combined_occlusion(u, v, point_depth),
        }
    }

    /// Test occlusion for multiple points
    pub fn test_points(&self, points: &[Point3<f32>], camera_pose: &Transform) -> Vec<OcclusionResult> {
        points.iter().map(|p| self.test_point(*p, camera_pose)).collect()
    }

    /// Get depth at normalized coordinates
    pub fn get_depth(&self, u: f32, v: f32) -> Option<f32> {
        self.depth_frame.as_ref().map(|f| f.sample_bilinear(u, v))
    }

    /// Get segmentation at pixel
    pub fn get_segmentation(&self, x: u32, y: u32) -> Option<u8> {
        self.segmentation_frame.as_ref().map(|f| f.get_mask(x, y))
    }

    /// Get current mode
    pub fn get_mode(&self) -> OcclusionMode {
        self.config.mode
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: OcclusionMode) {
        self.config.mode = mode;
    }

    // Internal methods

    fn test_depth_occlusion(&self, u: f32, v: f32, point_depth: f32) -> OcclusionResult {
        let depth_frame = match &self.depth_frame {
            Some(f) => f,
            None => return OcclusionResult::visible(),
        };

        let nu = u / self.intrinsics.as_ref().unwrap().width as f32;
        let nv = v / self.intrinsics.as_ref().unwrap().height as f32;
        let scene_depth = depth_frame.sample_bilinear(nu, nv);

        if scene_depth <= 0.0 {
            return OcclusionResult::visible();
        }

        let depth_diff = point_depth - scene_depth;

        if depth_diff > self.config.depth_tolerance {
            OcclusionResult::occluded(scene_depth, OcclusionType::Environment)
        } else if depth_diff > 0.0 {
            let factor = depth_diff / self.config.depth_tolerance;
            OcclusionResult::partial(factor, scene_depth)
        } else {
            OcclusionResult::visible()
        }
    }

    fn test_human_occlusion(&self, x: u32, y: u32) -> OcclusionResult {
        let seg_frame = match &self.segmentation_frame {
            Some(f) => f,
            None => return OcclusionResult::visible(),
        };

        if seg_frame.is_occluded(x, y, self.config.segmentation_threshold) {
            OcclusionResult::occluded(0.5, OcclusionType::Human)
        } else {
            OcclusionResult::visible()
        }
    }

    fn test_combined_occlusion(&self, u: f32, v: f32, point_depth: f32) -> OcclusionResult {
        // Test human first (higher priority)
        let human_result = self.test_human_occlusion(u as u32, v as u32);
        if human_result.occluded {
            return human_result;
        }

        // Then test depth
        self.test_depth_occlusion(u, v, point_depth)
    }
}

/// Occlusion shader data
#[derive(Debug, Clone, Copy)]
pub struct OcclusionShaderData {
    /// Depth texture enable
    pub depth_enabled: bool,
    /// Segmentation texture enable
    pub segmentation_enabled: bool,
    /// Depth tolerance
    pub depth_tolerance: f32,
    /// Segmentation threshold
    pub segmentation_threshold: f32,
    /// Near clip
    pub near_clip: f32,
    /// Far clip
    pub far_clip: f32,
}

impl Default for OcclusionShaderData {
    fn default() -> Self {
        Self {
            depth_enabled: true,
            segmentation_enabled: false,
            depth_tolerance: 0.02,
            segmentation_threshold: 0.5,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_frame_creation() {
        let frame = DepthFrame::new(1, 640, 480);
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert_eq!(frame.data.len(), 640 * 480);
    }

    #[test]
    fn test_depth_frame_get_depth() {
        let mut frame = DepthFrame::new(1, 10, 10);
        frame.data[55] = 2.5; // (5, 5)
        assert!((frame.get_depth(5, 5) - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_segmentation_frame_creation() {
        let frame = SegmentationFrame::new(1, 640, 480, SegmentationType::Person);
        assert_eq!(frame.width, 640);
        assert_eq!(frame.segmentation_type, SegmentationType::Person);
    }

    #[test]
    fn test_segmentation_is_occluded() {
        let mut frame = SegmentationFrame::new(1, 10, 10, SegmentationType::Person);
        frame.mask[55] = 200;
        assert!(frame.is_occluded(5, 5, 128));
        assert!(!frame.is_occluded(5, 5, 250));
    }

    #[test]
    fn test_occlusion_result_visible() {
        let result = OcclusionResult::visible();
        assert!(!result.occluded);
        assert_eq!(result.factor, 0.0);
    }

    #[test]
    fn test_occlusion_result_occluded() {
        let result = OcclusionResult::occluded(1.5, OcclusionType::Environment);
        assert!(result.occluded);
        assert_eq!(result.factor, 1.0);
        assert!((result.occluder_distance - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_occlusion_config_default() {
        let config = OcclusionConfig::default();
        assert_eq!(config.mode, OcclusionMode::EnvironmentDepth);
        assert!(config.edge_smoothing);
    }

    #[test]
    fn test_occlusion_handler_creation() {
        let config = OcclusionConfig::default();
        let handler = OcclusionHandler::new(config);
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_occlusion_handler_enable_disable() {
        let config = OcclusionConfig::default();
        let mut handler = OcclusionHandler::new(config);
        
        handler.disable();
        assert!(!handler.is_enabled());
        
        handler.enable();
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_occlusion_mode_none() {
        let mut config = OcclusionConfig::default();
        config.mode = OcclusionMode::None;
        let handler = OcclusionHandler::new(config);
        
        let result = handler.test_point(Point3::origin(), &Transform::new());
        assert!(!result.occluded);
    }
}
