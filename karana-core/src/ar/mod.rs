// AR Content Rendering System for Kāraṇa OS
// Handles 3D object rendering, overlays, anchoring, and world-locked content

pub mod objects;
pub mod anchors;
pub mod overlays;
pub mod renderer;
pub mod shaders;
pub mod tracking;
pub mod plane;
pub mod mesh;
pub mod lighting;
pub mod occlusion;
pub mod session;
pub mod content;

pub use objects::*;
pub use anchors::*;
pub use overlays::*;
pub use renderer::*;
pub use shaders::*;
pub use tracking::*;
pub use plane::*;
pub use mesh::*;
pub use lighting::*;
pub use occlusion::*;
pub use session::*;
pub use content::*;

use nalgebra::{Matrix4, Point3, Quaternion, UnitQuaternion, Vector3, Vector4};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Unique identifier for AR content
pub type ContentId = u64;

/// Generate unique content ID
fn next_content_id() -> ContentId {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 3D transform for AR content
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
    
    /// Create transform at position
    pub fn at_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Point3::new(x, y, z),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
    
    /// Convert to 4x4 transformation matrix
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.position.coords);
        let rotation = self.rotation.to_homogeneous();
        let scale = Matrix4::new_nonuniform_scaling(&self.scale);
        
        translation * rotation * scale
    }
    
    /// Interpolate between two transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        let t = t.clamp(0.0, 1.0);
        
        Transform {
            position: Point3::from(self.position.coords.lerp(&other.position.coords, t)),
            rotation: self.rotation.slerp(&other.rotation, t),
            scale: self.scale.lerp(&other.scale, t),
        }
    }
    
    /// Get forward vector
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(0.0, 0.0, -1.0)
    }
    
    /// Get up vector
    pub fn up(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(0.0, 1.0, 0.0)
    }
    
    /// Get right vector
    pub fn right(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(1.0, 0.0, 0.0)
    }
    
    /// Look at a point
    pub fn look_at(&mut self, target: &Point3<f32>) {
        let direction = (target - self.position).normalize();
        if direction.norm() > 0.0 {
            self.rotation = UnitQuaternion::face_towards(&direction, &Vector3::y());
        }
    }
    
    /// Distance to another transform
    pub fn distance_to(&self, other: &Transform) -> f32 {
        (self.position - other.position).norm()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

/// AR content visibility state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    FadingIn,
    FadingOut,
    Occluded,
}

/// AR content type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// 3D model/mesh
    Model3D,
    /// 2D sprite/billboard
    Sprite,
    /// Text label
    Text,
    /// UI panel
    UIPanel,
    /// Video surface
    Video,
    /// Particle system
    Particles,
    /// Line/path
    Line,
    /// Point cloud
    PointCloud,
}

/// Material properties for rendering
#[derive(Debug, Clone)]
pub struct Material {
    pub id: ContentId,
    pub name: String,
    pub color: [f32; 4],
    pub emissive: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub opacity: f32,
    pub double_sided: bool,
    pub texture_id: Option<ContentId>,
    pub normal_map_id: Option<ContentId>,
}

impl Material {
    pub fn new(name: &str) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: [0.0, 0.0, 0.0],
            metallic: 0.0,
            roughness: 0.5,
            opacity: 1.0,
            double_sided: false,
            texture_id: None,
            normal_map_id: None,
        }
    }
    
    /// Create solid color material
    pub fn solid_color(r: f32, g: f32, b: f32) -> Self {
        let mut mat = Self::new("solid_color");
        mat.color = [r, g, b, 1.0];
        mat
    }
    
    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self.color[3] = self.opacity;
        self
    }
    
    /// Set metallic/roughness
    pub fn with_pbr(mut self, metallic: f32, roughness: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }
    
    /// Check if transparent
    pub fn is_transparent(&self) -> bool {
        self.opacity < 1.0
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Bounding box for content
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox3D {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox3D {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }
    
    /// Create from center and half extents
    pub fn from_center_extents(center: Point3<f32>, extents: Vector3<f32>) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }
    
    /// Get center point
    pub fn center(&self) -> Point3<f32> {
        Point3::from((self.min.coords + self.max.coords) * 0.5)
    }
    
    /// Get size
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
    
    /// Get half extents
    pub fn extents(&self) -> Vector3<f32> {
        self.size() * 0.5
    }
    
    /// Check if point is inside
    pub fn contains(&self, point: &Point3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    /// Check intersection with another box
    pub fn intersects(&self, other: &BoundingBox3D) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
    
    /// Expand to include a point
    pub fn expand_to_include(&mut self, point: &Point3<f32>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }
    
    /// Get volume
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s.x * s.y * s.z
    }
}

impl Default for BoundingBox3D {
    fn default() -> Self {
        Self {
            min: Point3::new(-0.5, -0.5, -0.5),
            max: Point3::new(0.5, 0.5, 0.5),
        }
    }
}

/// Camera frustum for culling
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Vector4<f32>; 6],
}

impl Frustum {
    /// Create from view-projection matrix
    pub fn from_matrix(vp: &Matrix4<f32>) -> Self {
        let mut planes = [Vector4::zeros(); 6];
        
        // Extract rows as Vector4
        let row0 = Vector4::new(vp[(0, 0)], vp[(0, 1)], vp[(0, 2)], vp[(0, 3)]);
        let row1 = Vector4::new(vp[(1, 0)], vp[(1, 1)], vp[(1, 2)], vp[(1, 3)]);
        let row2 = Vector4::new(vp[(2, 0)], vp[(2, 1)], vp[(2, 2)], vp[(2, 3)]);
        let row3 = Vector4::new(vp[(3, 0)], vp[(3, 1)], vp[(3, 2)], vp[(3, 3)]);
        
        // Left plane
        planes[0] = row3 + row0;
        // Right plane
        planes[1] = row3 - row0;
        // Bottom plane
        planes[2] = row3 + row1;
        // Top plane
        planes[3] = row3 - row1;
        // Near plane
        planes[4] = row3 + row2;
        // Far plane
        planes[5] = row3 - row2;
        
        // Normalize planes
        for plane in &mut planes {
            let len = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
            if len > 0.0 {
                *plane /= len;
            }
        }
        
        Self { planes }
    }
    
    /// Test if bounding box is inside or intersects frustum
    pub fn contains_box(&self, bbox: &BoundingBox3D) -> bool {
        for plane in &self.planes {
            // Get positive vertex (furthest along plane normal)
            let px = if plane.x >= 0.0 { bbox.max.x } else { bbox.min.x };
            let py = if plane.y >= 0.0 { bbox.max.y } else { bbox.min.y };
            let pz = if plane.z >= 0.0 { bbox.max.z } else { bbox.min.z };
            
            // If positive vertex is outside, box is outside
            if plane.x * px + plane.y * py + plane.z * pz + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
    
    /// Test if point is inside frustum
    pub fn contains_point(&self, point: &Point3<f32>) -> bool {
        for plane in &self.planes {
            if plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Render statistics
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub objects_rendered: u32,
    pub objects_culled: u32,
    pub triangles_rendered: u64,
    pub draw_calls: u32,
    pub frame_time_ms: f32,
    pub gpu_time_ms: f32,
}

impl RenderStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Calculate culling efficiency
    pub fn culling_efficiency(&self) -> f32 {
        let total = self.objects_rendered + self.objects_culled;
        if total > 0 {
            self.objects_culled as f32 / total as f32
        } else {
            0.0
        }
    }
}

/// Content layer for organizing AR content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentLayer {
    /// Background layer (skybox, far objects)
    Background,
    /// World-locked content
    World,
    /// Head-locked content (always visible)
    HeadLocked,
    /// UI overlay
    UI,
    /// Debug visualization
    Debug,
}

impl ContentLayer {
    /// Get render order (lower = rendered first)
    pub fn render_order(&self) -> u8 {
        match self {
            ContentLayer::Background => 0,
            ContentLayer::World => 1,
            ContentLayer::HeadLocked => 2,
            ContentLayer::UI => 3,
            ContentLayer::Debug => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    
    #[test]
    fn test_transform_new() {
        let t = Transform::new();
        
        assert_eq!(t.position, Point3::origin());
        assert_eq!(t.scale, Vector3::new(1.0, 1.0, 1.0));
    }
    
    #[test]
    fn test_transform_at_position() {
        let t = Transform::at_position(1.0, 2.0, 3.0);
        
        assert_eq!(t.position.x, 1.0);
        assert_eq!(t.position.y, 2.0);
        assert_eq!(t.position.z, 3.0);
    }
    
    #[test]
    fn test_transform_forward() {
        let t = Transform::new();
        let forward = t.forward();
        
        // Default forward is -Z
        assert!((forward.z - (-1.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_transform_distance() {
        let t1 = Transform::at_position(0.0, 0.0, 0.0);
        let t2 = Transform::at_position(3.0, 4.0, 0.0);
        
        assert!((t1.distance_to(&t2) - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_transform_lerp() {
        let t1 = Transform::at_position(0.0, 0.0, 0.0);
        let t2 = Transform::at_position(10.0, 0.0, 0.0);
        
        let mid = t1.lerp(&t2, 0.5);
        
        assert!((mid.position.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_material_solid_color() {
        let mat = Material::solid_color(1.0, 0.0, 0.0);
        
        assert_eq!(mat.color[0], 1.0);
        assert_eq!(mat.color[1], 0.0);
        assert_eq!(mat.color[2], 0.0);
    }
    
    #[test]
    fn test_material_opacity() {
        let mat = Material::solid_color(1.0, 1.0, 1.0).with_opacity(0.5);
        
        assert!(mat.is_transparent());
        assert_eq!(mat.opacity, 0.5);
    }
    
    #[test]
    fn test_bounding_box_center() {
        let bbox = BoundingBox3D::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
        );
        
        let center = bbox.center();
        assert_eq!(center, Point3::new(5.0, 5.0, 5.0));
    }
    
    #[test]
    fn test_bounding_box_contains() {
        let bbox = BoundingBox3D::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
        );
        
        assert!(bbox.contains(&Point3::new(5.0, 5.0, 5.0)));
        assert!(!bbox.contains(&Point3::new(15.0, 5.0, 5.0)));
    }
    
    #[test]
    fn test_bounding_box_intersects() {
        let bbox1 = BoundingBox3D::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
        );
        let bbox2 = BoundingBox3D::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(15.0, 15.0, 15.0),
        );
        let bbox3 = BoundingBox3D::new(
            Point3::new(20.0, 20.0, 20.0),
            Point3::new(30.0, 30.0, 30.0),
        );
        
        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
    }
    
    #[test]
    fn test_bounding_box_volume() {
        let bbox = BoundingBox3D::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 3.0, 4.0),
        );
        
        assert_eq!(bbox.volume(), 24.0);
    }
    
    #[test]
    fn test_content_layer_order() {
        assert!(ContentLayer::Background.render_order() < ContentLayer::World.render_order());
        assert!(ContentLayer::World.render_order() < ContentLayer::UI.render_order());
    }
    
    #[test]
    fn test_render_stats() {
        let mut stats = RenderStats::default();
        
        stats.objects_rendered = 80;
        stats.objects_culled = 20;
        
        assert!((stats.culling_efficiency() - 0.2).abs() < 0.001);
    }
    
    #[test]
    fn test_transform_matrix() {
        let t = Transform::at_position(1.0, 2.0, 3.0);
        let matrix = t.to_matrix();
        
        // Translation should be in the last column
        assert!((matrix[(0, 3)] - 1.0).abs() < 0.001);
        assert!((matrix[(1, 3)] - 2.0).abs() < 0.001);
        assert!((matrix[(2, 3)] - 3.0).abs() < 0.001);
    }
}
