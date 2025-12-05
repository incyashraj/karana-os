// AR 3D Object Rendering for Kāraṇa OS
// Handles 3D meshes, primitives, and model loading

use super::*;
use std::collections::HashMap;

/// Vertex data for 3D meshes
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
    
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// 3D mesh data
#[derive(Debug, Clone)]
pub struct Mesh {
    pub id: ContentId,
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounds: BoundingBox3D,
}

impl Mesh {
    pub fn new(name: &str) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
            bounds: BoundingBox3D::default(),
        }
    }
    
    /// Create with vertices and indices
    pub fn with_data(mut self, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        self.vertices = vertices;
        self.indices = indices;
        self.calculate_bounds();
        self
    }
    
    /// Calculate bounding box from vertices
    pub fn calculate_bounds(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        
        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);
        
        for v in &self.vertices {
            min.x = min.x.min(v.position[0]);
            min.y = min.y.min(v.position[1]);
            min.z = min.z.min(v.position[2]);
            max.x = max.x.max(v.position[0]);
            max.y = max.y.max(v.position[1]);
            max.z = max.z.max(v.position[2]);
        }
        
        self.bounds = BoundingBox3D::new(min, max);
    }
    
    /// Get triangle count
    pub fn triangle_count(&self) -> u32 {
        (self.indices.len() / 3) as u32
    }
    
    /// Get vertex count
    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }
    
    /// Create a cube primitive
    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            // Front face
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back face
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0], [0.0, 1.0]),
            // Top face
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0], [0.0, 1.0]),
            // Bottom face
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0], [0.0, 1.0]),
            // Right face
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left face
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        ];
        
        let indices: Vec<u32> = (0..6).flat_map(|face| {
            let base = face * 4;
            vec![base, base + 1, base + 2, base, base + 2, base + 3]
        }).collect();
        
        Self::new("cube").with_data(vertices, indices)
    }
    
    /// Create a quad/plane primitive
    pub fn quad(width: f32, height: f32) -> Self {
        let w = width / 2.0;
        let h = height / 2.0;
        
        let vertices = vec![
            Vertex::new([-w, -h, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([ w, -h, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([ w,  h, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-w,  h, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        Self::new("quad").with_data(vertices, indices)
    }
    
    /// Create a sphere primitive
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            
            for seg in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
                
                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();
                
                let u = seg as f32 / segments as f32;
                let v = ring as f32 / rings as f32;
                
                vertices.push(Vertex::new(
                    [x * radius, y * radius, z * radius],
                    [x, y, z],
                    [u, v],
                ));
            }
        }
        
        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;
                
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);
                
                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }
        
        Self::new("sphere").with_data(vertices, indices)
    }
}

/// AR 3D object instance
#[derive(Debug, Clone)]
pub struct Object3D {
    pub id: ContentId,
    pub name: String,
    pub mesh_id: ContentId,
    pub material_id: ContentId,
    pub transform: Transform,
    pub visibility: Visibility,
    pub layer: ContentLayer,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub interactive: bool,
    pub user_data: HashMap<String, String>,
}

impl Object3D {
    pub fn new(name: &str, mesh_id: ContentId, material_id: ContentId) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            mesh_id,
            material_id,
            transform: Transform::new(),
            visibility: Visibility::Visible,
            layer: ContentLayer::World,
            cast_shadows: true,
            receive_shadows: true,
            interactive: false,
            user_data: HashMap::new(),
        }
    }
    
    /// Set transform
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    
    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.position = Point3::new(x, y, z);
        self
    }
    
    /// Set layer
    pub fn with_layer(mut self, layer: ContentLayer) -> Self {
        self.layer = layer;
        self
    }
    
    /// Set interactive flag
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
    
    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visibility == Visibility::Visible || 
        self.visibility == Visibility::FadingIn
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
    }
}

/// Billboard that always faces the camera
#[derive(Debug, Clone)]
pub struct Billboard {
    pub id: ContentId,
    pub name: String,
    pub texture_id: ContentId,
    pub position: Point3<f32>,
    pub size: (f32, f32),
    pub color: [f32; 4],
    pub visibility: Visibility,
    pub layer: ContentLayer,
    pub lock_y_axis: bool,
}

impl Billboard {
    pub fn new(name: &str, texture_id: ContentId) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            texture_id,
            position: Point3::origin(),
            size: (1.0, 1.0),
            color: [1.0, 1.0, 1.0, 1.0],
            visibility: Visibility::Visible,
            layer: ContentLayer::World,
            lock_y_axis: false,
        }
    }
    
    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Point3::new(x, y, z);
        self
    }
    
    /// Set size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }
    
    /// Set color tint
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
}

/// Line renderer for paths and guides
#[derive(Debug, Clone)]
pub struct LineRenderer {
    pub id: ContentId,
    pub points: Vec<Point3<f32>>,
    pub color: [f32; 4],
    pub width: f32,
    pub dashed: bool,
    pub dash_length: f32,
    pub visibility: Visibility,
    pub layer: ContentLayer,
}

impl LineRenderer {
    pub fn new() -> Self {
        Self {
            id: next_content_id(),
            points: Vec::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            width: 0.01,
            dashed: false,
            dash_length: 0.1,
            visibility: Visibility::Visible,
            layer: ContentLayer::World,
        }
    }
    
    /// Add a point
    pub fn add_point(&mut self, x: f32, y: f32, z: f32) {
        self.points.push(Point3::new(x, y, z));
    }
    
    /// Set color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
    
    /// Set width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width.max(0.001);
        self
    }
    
    /// Enable dashed line
    pub fn with_dashed(mut self, dash_length: f32) -> Self {
        self.dashed = true;
        self.dash_length = dash_length.max(0.01);
        self
    }
    
    /// Calculate total length
    pub fn total_length(&self) -> f32 {
        let mut length = 0.0;
        for i in 1..self.points.len() {
            length += (self.points[i] - self.points[i - 1]).norm();
        }
        length
    }
    
    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
    }
}

impl Default for LineRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Point cloud renderer
#[derive(Debug, Clone)]
pub struct PointCloud {
    pub id: ContentId,
    pub points: Vec<Point3<f32>>,
    pub colors: Vec<[f32; 4]>,
    pub point_size: f32,
    pub visibility: Visibility,
    pub layer: ContentLayer,
}

impl PointCloud {
    pub fn new() -> Self {
        Self {
            id: next_content_id(),
            points: Vec::new(),
            colors: Vec::new(),
            point_size: 0.01,
            visibility: Visibility::Visible,
            layer: ContentLayer::World,
        }
    }
    
    /// Add a point with color
    pub fn add_point(&mut self, position: Point3<f32>, color: [f32; 4]) {
        self.points.push(position);
        self.colors.push(color);
    }
    
    /// Add points from slice
    pub fn add_points(&mut self, positions: &[Point3<f32>], default_color: [f32; 4]) {
        for &pos in positions {
            self.add_point(pos, default_color);
        }
    }
    
    /// Get point count
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
    
    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
        self.colors.clear();
    }
}

impl Default for PointCloud {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vertex_creation() {
        let v = Vertex::new([1.0, 2.0, 3.0], [0.0, 1.0, 0.0], [0.5, 0.5]);
        
        assert_eq!(v.position, [1.0, 2.0, 3.0]);
        assert_eq!(v.normal, [0.0, 1.0, 0.0]);
        assert_eq!(v.uv, [0.5, 0.5]);
    }
    
    #[test]
    fn test_mesh_cube() {
        let cube = Mesh::cube(2.0);
        
        assert_eq!(cube.vertex_count(), 24); // 6 faces * 4 vertices
        assert_eq!(cube.triangle_count(), 12); // 6 faces * 2 triangles
    }
    
    #[test]
    fn test_mesh_quad() {
        let quad = Mesh::quad(1.0, 1.0);
        
        assert_eq!(quad.vertex_count(), 4);
        assert_eq!(quad.triangle_count(), 2);
    }
    
    #[test]
    fn test_mesh_sphere() {
        let sphere = Mesh::sphere(1.0, 16, 8);
        
        assert!(sphere.vertex_count() > 0);
        assert!(sphere.triangle_count() > 0);
    }
    
    #[test]
    fn test_object3d_creation() {
        let obj = Object3D::new("test", 1, 2)
            .with_position(1.0, 2.0, 3.0)
            .with_interactive(true);
        
        assert_eq!(obj.transform.position.x, 1.0);
        assert!(obj.interactive);
        assert!(obj.is_visible());
    }
    
    #[test]
    fn test_object3d_visibility() {
        let mut obj = Object3D::new("test", 1, 2);
        
        assert!(obj.is_visible());
        
        obj.set_visible(false);
        assert!(!obj.is_visible());
    }
    
    #[test]
    fn test_billboard() {
        let billboard = Billboard::new("label", 1)
            .with_position(5.0, 2.0, 0.0)
            .with_size(2.0, 1.0);
        
        assert_eq!(billboard.position.x, 5.0);
        assert_eq!(billboard.size, (2.0, 1.0));
    }
    
    #[test]
    fn test_line_renderer() {
        let mut line = LineRenderer::new()
            .with_color(1.0, 0.0, 0.0, 1.0)
            .with_width(0.02);
        
        line.add_point(0.0, 0.0, 0.0);
        line.add_point(1.0, 0.0, 0.0);
        line.add_point(1.0, 1.0, 0.0);
        
        assert_eq!(line.points.len(), 3);
        assert!((line.total_length() - 2.0).abs() < 0.001);
    }
    
    #[test]
    fn test_line_renderer_dashed() {
        let line = LineRenderer::new()
            .with_dashed(0.05);
        
        assert!(line.dashed);
        assert_eq!(line.dash_length, 0.05);
    }
    
    #[test]
    fn test_point_cloud() {
        let mut cloud = PointCloud::new();
        
        cloud.add_point(Point3::new(0.0, 0.0, 0.0), [1.0, 1.0, 1.0, 1.0]);
        cloud.add_point(Point3::new(1.0, 1.0, 1.0), [1.0, 0.0, 0.0, 1.0]);
        
        assert_eq!(cloud.point_count(), 2);
        
        cloud.clear();
        assert_eq!(cloud.point_count(), 0);
    }
    
    #[test]
    fn test_mesh_bounds() {
        let cube = Mesh::cube(2.0);
        
        assert!((cube.bounds.min.x - (-1.0)).abs() < 0.001);
        assert!((cube.bounds.max.x - 1.0).abs() < 0.001);
    }
}
