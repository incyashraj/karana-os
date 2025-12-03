//! Reality Capture APIs
//!
//! Provides access to environmental understanding data for WebXR sessions.
//! This includes camera passthrough, depth data, and scene geometry.

use serde::{Deserialize, Serialize};

use super::XRVector3;

/// Depth sensing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthSensingConfig {
    /// Usage intent
    pub usage_preference: DepthUsage,
    /// Data format preference  
    pub data_format_preference: DepthDataFormat,
}

impl Default for DepthSensingConfig {
    fn default() -> Self {
        Self {
            usage_preference: DepthUsage::CpuOptimized,
            data_format_preference: DepthDataFormat::Luminance,
        }
    }
}

/// Depth data usage intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthUsage {
    /// Optimized for CPU access
    CpuOptimized,
    /// Optimized for GPU access
    GpuOptimized,
}

/// Depth data format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthDataFormat {
    /// 8-bit luminance
    Luminance,
    /// 32-bit float
    Float32,
    /// 16-bit unsigned int
    Uint16,
}

/// Depth information for a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRDepthInfo {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Normalization factor for depth values
    pub norm_depth_buffer_from_norm_view: [f32; 16], // 4x4 transform
    /// Raw depth data as meters (linearized)
    pub data: Vec<f32>,
}

impl XRDepthInfo {
    /// Create empty depth info
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            norm_depth_buffer_from_norm_view: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
            data: vec![],
        }
    }
    
    /// Get depth at a normalized coordinate (0-1)
    pub fn get_depth_meters(&self, norm_x: f32, norm_y: f32) -> Option<f32> {
        if self.data.is_empty() {
            return None;
        }
        
        let x = (norm_x * self.width as f32) as usize;
        let y = (norm_y * self.height as f32) as usize;
        
        if x >= self.width as usize || y >= self.height as usize {
            return None;
        }
        
        let idx = y * self.width as usize + x;
        self.data.get(idx).copied()
    }
}

/// Camera access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAccessConfig {
    /// Texture size requested
    pub texture_size: Option<(u32, u32)>,
    /// Whether to allow raw pixel access
    pub allow_raw_access: bool,
}

impl Default for CameraAccessConfig {
    fn default() -> Self {
        Self {
            texture_size: None,
            allow_raw_access: false,
        }
    }
}

/// Camera frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRCameraFrame {
    /// Frame timestamp
    pub timestamp: f64,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Pixel format
    pub format: CameraPixelFormat,
    /// Raw pixel data (if raw access enabled)
    pub pixels: Option<Vec<u8>>,
    /// Intrinsic camera matrix
    pub intrinsics: CameraIntrinsics,
}

/// Camera pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraPixelFormat {
    /// RGBA 8-bit per channel
    Rgba8,
    /// RGB 8-bit per channel
    Rgb8,
    /// YUV 4:2:0 (common on mobile)
    Yuv420,
    /// Grayscale 8-bit
    Gray8,
}

/// Camera intrinsic parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraIntrinsics {
    /// Focal length X
    pub fx: f64,
    /// Focal length Y
    pub fy: f64,
    /// Principal point X
    pub cx: f64,
    /// Principal point Y
    pub cy: f64,
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        // Typical AR glasses camera intrinsics
        Self {
            fx: 500.0,
            fy: 500.0,
            cx: 320.0,
            cy: 240.0,
        }
    }
}

impl CameraIntrinsics {
    /// Project a 3D point to 2D
    pub fn project(&self, point: &XRVector3) -> Option<(f64, f64)> {
        if point.z <= 0.0 {
            return None;
        }
        
        let u = self.fx * point.x / point.z + self.cx;
        let v = self.fy * point.y / point.z + self.cy;
        
        Some((u, v))
    }
    
    /// Unproject a 2D point to ray direction
    pub fn unproject(&self, u: f64, v: f64) -> XRVector3 {
        let x = (u - self.cx) / self.fx;
        let y = (v - self.cy) / self.fy;
        
        XRVector3::new(x, y, 1.0).normalize()
    }
}

/// Mesh data from scene understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRMesh {
    /// Unique mesh ID
    pub id: String,
    /// Vertices (x, y, z triplets)
    pub vertices: Vec<f32>,
    /// Indices (triangle list)
    pub indices: Vec<u32>,
    /// Normals (x, y, z triplets)
    pub normals: Option<Vec<f32>>,
    /// Semantic labels per face
    pub semantics: Option<Vec<MeshSemanticLabel>>,
    /// Last update time
    pub last_updated: f64,
}

/// Mesh semantic labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshSemanticLabel {
    /// Unknown surface
    Unknown,
    /// Wall
    Wall,
    /// Floor
    Floor,
    /// Ceiling
    Ceiling,
    /// Table
    Table,
    /// Seat (chair, sofa)
    Seat,
    /// Window
    Window,
    /// Door
    Door,
    /// Storage (cabinet, shelf)
    Storage,
    /// Bed
    Bed,
    /// Screen (TV, monitor)
    Screen,
    /// Lamp
    Lamp,
    /// Plant
    Plant,
    /// Stairs
    Stairs,
    /// Other
    Other,
}

/// Reality capture manager
pub struct RealityCaptureManager {
    /// Depth sensing config
    depth_config: Option<DepthSensingConfig>,
    /// Camera config
    camera_config: Option<CameraAccessConfig>,
    /// Last depth frame
    last_depth: Option<XRDepthInfo>,
    /// Last camera frame
    last_camera: Option<XRCameraFrame>,
    /// Detected meshes
    meshes: Vec<XRMesh>,
}

impl RealityCaptureManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            depth_config: None,
            camera_config: None,
            last_depth: None,
            last_camera: None,
            meshes: vec![],
        }
    }
    
    /// Enable depth sensing
    pub fn enable_depth_sensing(&mut self, config: DepthSensingConfig) {
        self.depth_config = Some(config);
    }
    
    /// Disable depth sensing
    pub fn disable_depth_sensing(&mut self) {
        self.depth_config = None;
        self.last_depth = None;
    }
    
    /// Enable camera access
    pub fn enable_camera(&mut self, config: CameraAccessConfig) {
        self.camera_config = Some(config);
    }
    
    /// Disable camera access
    pub fn disable_camera(&mut self) {
        self.camera_config = None;
        self.last_camera = None;
    }
    
    /// Get latest depth info
    pub fn get_depth_info(&self) -> Option<&XRDepthInfo> {
        self.last_depth.as_ref()
    }
    
    /// Get latest camera frame
    pub fn get_camera_frame(&self) -> Option<&XRCameraFrame> {
        self.last_camera.as_ref()
    }
    
    /// Get all detected meshes
    pub fn get_meshes(&self) -> &[XRMesh] {
        &self.meshes
    }
    
    /// Update with new depth data (called by system)
    pub fn update_depth(&mut self, depth: XRDepthInfo) {
        if self.depth_config.is_some() {
            self.last_depth = Some(depth);
        }
    }
    
    /// Update with new camera frame (called by system)
    pub fn update_camera(&mut self, frame: XRCameraFrame) {
        if self.camera_config.is_some() {
            self.last_camera = Some(frame);
        }
    }
    
    /// Update meshes (called by system)
    pub fn update_meshes(&mut self, meshes: Vec<XRMesh>) {
        self.meshes = meshes;
    }
}

impl Default for RealityCaptureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_depth_info() {
        let mut depth = XRDepthInfo::empty();
        depth.width = 10;
        depth.height = 10;
        depth.data = vec![1.5; 100];
        
        let d = depth.get_depth_meters(0.5, 0.5);
        assert!(d.is_some());
        assert!((d.unwrap() - 1.5).abs() < 0.001);
    }
    
    #[test]
    fn test_camera_intrinsics() {
        let intrinsics = CameraIntrinsics::default();
        
        // Project a point
        let point = XRVector3::new(0.0, 0.0, 1.0);
        let (u, v) = intrinsics.project(&point).unwrap();
        
        assert!((u - intrinsics.cx).abs() < 0.001);
        assert!((v - intrinsics.cy).abs() < 0.001);
    }
    
    #[test]
    fn test_unproject() {
        let intrinsics = CameraIntrinsics::default();
        
        // Unproject center pixel
        let ray = intrinsics.unproject(intrinsics.cx, intrinsics.cy);
        
        assert!(ray.x.abs() < 0.001);
        assert!(ray.y.abs() < 0.001);
        assert!((ray.z - 1.0).abs() < 0.1);
    }
    
    #[test]
    fn test_reality_capture_manager() {
        let mut manager = RealityCaptureManager::new();
        
        // Initially no data
        assert!(manager.get_depth_info().is_none());
        assert!(manager.get_camera_frame().is_none());
        
        // Enable and update
        manager.enable_depth_sensing(DepthSensingConfig::default());
        manager.update_depth(XRDepthInfo::empty());
        
        assert!(manager.get_depth_info().is_some());
    }
}
