//! Mesh Reconstruction for Kāraṇa OS
//! 
//! Real-time 3D mesh reconstruction from depth data for AR occlusion
//! and physics interactions.

use super::*;
use nalgebra::{Point3, Vector3};
use std::collections::HashMap;
use std::time::Instant;

/// Mesh vertex
#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    /// Position
    pub position: Point3<f32>,
    /// Normal
    pub normal: Vector3<f32>,
    /// Texture coordinate
    pub uv: (f32, f32),
    /// Color (RGBA)
    pub color: [u8; 4],
}

impl MeshVertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            position,
            normal,
            uv: (0.0, 0.0),
            color: [255, 255, 255, 255],
        }
    }
}

/// Triangle face
#[derive(Debug, Clone, Copy)]
pub struct MeshFace {
    /// Vertex indices
    pub indices: [u32; 3],
    /// Face normal
    pub normal: Vector3<f32>,
}

/// Mesh classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshClassification {
    /// Unknown/unclassified
    None,
    /// Wall surface
    Wall,
    /// Floor surface
    Floor,
    /// Ceiling surface
    Ceiling,
    /// Table/surface
    Table,
    /// Seat
    Seat,
    /// Door
    Door,
    /// Window
    Window,
}

/// 3D mesh chunk (for spatial hashing)
#[derive(Debug, Clone)]
pub struct MeshChunk {
    /// Chunk ID
    pub id: u64,
    /// Chunk position (center)
    pub position: Point3<f32>,
    /// Chunk size
    pub size: f32,
    /// Vertices
    pub vertices: Vec<MeshVertex>,
    /// Faces
    pub faces: Vec<MeshFace>,
    /// Classification
    pub classification: MeshClassification,
    /// Is dirty (needs upload)
    pub dirty: bool,
    /// Last update time
    pub last_updated: Instant,
}

impl MeshChunk {
    pub fn new(id: u64, position: Point3<f32>, size: f32) -> Self {
        Self {
            id,
            position,
            size,
            vertices: Vec::new(),
            faces: Vec::new(),
            classification: MeshClassification::None,
            dirty: true,
            last_updated: Instant::now(),
        }
    }

    /// Get bounding box
    pub fn get_bounds(&self) -> (Point3<f32>, Point3<f32>) {
        let half = self.size / 2.0;
        let min = Point3::new(
            self.position.x - half,
            self.position.y - half,
            self.position.z - half,
        );
        let max = Point3::new(
            self.position.x + half,
            self.position.y + half,
            self.position.z + half,
        );
        (min, max)
    }

    /// Check if point is inside chunk bounds
    pub fn contains(&self, point: Point3<f32>) -> bool {
        let half = self.size / 2.0;
        (point.x - self.position.x).abs() <= half
            && (point.y - self.position.y).abs() <= half
            && (point.z - self.position.z).abs() <= half
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get face count
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Clear mesh data
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.faces.clear();
        self.dirty = true;
    }

    /// Compute face normals
    pub fn compute_normals(&mut self) {
        // First, reset all vertex normals
        for vertex in &mut self.vertices {
            vertex.normal = Vector3::zeros();
        }

        // Compute face normals and accumulate to vertices
        for face in &mut self.faces {
            if face.indices[0] as usize >= self.vertices.len() 
                || face.indices[1] as usize >= self.vertices.len()
                || face.indices[2] as usize >= self.vertices.len() {
                continue;
            }

            let v0 = self.vertices[face.indices[0] as usize].position;
            let v1 = self.vertices[face.indices[1] as usize].position;
            let v2 = self.vertices[face.indices[2] as usize].position;

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2);

            if normal.norm() > 0.0 {
                let normalized = normal.normalize();
                face.normal = normalized;

                // Add to vertex normals
                self.vertices[face.indices[0] as usize].normal += normalized;
                self.vertices[face.indices[1] as usize].normal += normalized;
                self.vertices[face.indices[2] as usize].normal += normalized;
            }
        }

        // Normalize vertex normals
        for vertex in &mut self.vertices {
            if vertex.normal.norm() > 0.0 {
                vertex.normal = vertex.normal.normalize();
            }
        }
    }
}

/// Mesh reconstruction configuration
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// Chunk size (meters)
    pub chunk_size: f32,
    /// Max vertex count per chunk
    pub max_vertices_per_chunk: usize,
    /// Voxel resolution (meters)
    pub voxel_size: f32,
    /// Max distance from camera (meters)
    pub max_distance: f32,
    /// Enable mesh classification
    pub classify_mesh: bool,
    /// Update interval (seconds)
    pub update_interval: f32,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1.0,
            max_vertices_per_chunk: 10000,
            voxel_size: 0.02,
            max_distance: 5.0,
            classify_mesh: true,
            update_interval: 0.1,
        }
    }
}

/// Mesh reconstructor using TSDF (Truncated Signed Distance Function)
#[derive(Debug)]
pub struct MeshReconstructor {
    /// Configuration
    config: MeshConfig,
    /// Mesh chunks (spatial hash)
    chunks: HashMap<(i32, i32, i32), MeshChunk>,
    /// Next chunk ID
    next_chunk_id: u64,
    /// Last update time
    last_update: Instant,
    /// Is running
    running: bool,
    /// Total vertex count
    total_vertices: usize,
    /// Total face count
    total_faces: usize,
}

impl MeshReconstructor {
    /// Create new mesh reconstructor
    pub fn new(config: MeshConfig) -> Self {
        Self {
            config,
            chunks: HashMap::new(),
            next_chunk_id: 1,
            last_update: Instant::now(),
            running: false,
            total_vertices: 0,
            total_faces: 0,
        }
    }

    /// Start mesh reconstruction
    pub fn start(&mut self) {
        self.running = true;
        self.last_update = Instant::now();
    }

    /// Stop mesh reconstruction
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Process depth frame
    pub fn process_depth_frame(
        &mut self,
        depth_data: &[f32],
        width: u32,
        height: u32,
        camera_pose: &Transform,
        intrinsics: &super::tracking::CameraIntrinsics,
    ) {
        if !self.running {
            return;
        }

        let elapsed = self.last_update.elapsed().as_secs_f32();
        if elapsed < self.config.update_interval {
            return;
        }
        self.last_update = Instant::now();

        // Unproject depth points to 3D
        let points = self.unproject_depth(depth_data, width, height, camera_pose, intrinsics);

        // Update affected chunks
        for point in points {
            let chunk_key = self.get_chunk_key(point);
            
            if !self.chunks.contains_key(&chunk_key) {
                let chunk_center = Point3::new(
                    (chunk_key.0 as f32 + 0.5) * self.config.chunk_size,
                    (chunk_key.1 as f32 + 0.5) * self.config.chunk_size,
                    (chunk_key.2 as f32 + 0.5) * self.config.chunk_size,
                );
                let chunk = MeshChunk::new(self.next_chunk_id, chunk_center, self.config.chunk_size);
                self.chunks.insert(chunk_key, chunk);
                self.next_chunk_id += 1;
            }

            if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
                if chunk.vertices.len() < self.config.max_vertices_per_chunk {
                    chunk.vertices.push(MeshVertex::new(point, Vector3::y()));
                    chunk.dirty = true;
                    chunk.last_updated = Instant::now();
                }
            }
        }

        // Generate mesh for dirty chunks
        self.generate_meshes();

        // Update statistics
        self.update_statistics();
    }

    /// Get chunk at position
    pub fn get_chunk_at(&self, position: Point3<f32>) -> Option<&MeshChunk> {
        let key = self.get_chunk_key(position);
        self.chunks.get(&key)
    }

    /// Get all chunks
    pub fn get_chunks(&self) -> Vec<&MeshChunk> {
        self.chunks.values().collect()
    }

    /// Get dirty chunks
    pub fn get_dirty_chunks(&self) -> Vec<&MeshChunk> {
        self.chunks.values().filter(|c| c.dirty).collect()
    }

    /// Mark chunk as clean
    pub fn mark_clean(&mut self, chunk_id: u64) {
        for chunk in self.chunks.values_mut() {
            if chunk.id == chunk_id {
                chunk.dirty = false;
                break;
            }
        }
    }

    /// Get total vertex count
    pub fn total_vertex_count(&self) -> usize {
        self.total_vertices
    }

    /// Get total face count
    pub fn total_face_count(&self) -> usize {
        self.total_faces
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Clear all mesh data
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.total_vertices = 0;
        self.total_faces = 0;
    }

    /// Raycast against mesh
    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>) -> Option<(Point3<f32>, Vector3<f32>)> {
        let direction = direction.normalize();
        let mut closest_hit: Option<(f32, Point3<f32>, Vector3<f32>)> = None;

        for chunk in self.chunks.values() {
            // Quick bounds check
            let (min, max) = chunk.get_bounds();
            if !self.ray_box_intersect(origin, direction, min, max) {
                continue;
            }

            // Test individual faces
            for face in &chunk.faces {
                if let Some((t, point)) = self.ray_triangle_intersect(
                    origin,
                    direction,
                    &chunk.vertices,
                    face,
                ) {
                    if closest_hit.is_none() || t < closest_hit.as_ref().unwrap().0 {
                        closest_hit = Some((t, point, face.normal));
                    }
                }
            }
        }

        closest_hit.map(|(_, point, normal)| (point, normal))
    }

    // Internal methods

    fn get_chunk_key(&self, position: Point3<f32>) -> (i32, i32, i32) {
        (
            (position.x / self.config.chunk_size).floor() as i32,
            (position.y / self.config.chunk_size).floor() as i32,
            (position.z / self.config.chunk_size).floor() as i32,
        )
    }

    fn unproject_depth(
        &self,
        depth_data: &[f32],
        width: u32,
        height: u32,
        camera_pose: &Transform,
        intrinsics: &super::tracking::CameraIntrinsics,
    ) -> Vec<Point3<f32>> {
        let mut points = Vec::new();
        let step = 4; // Downsample

        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                let idx = (y * width + x) as usize;
                if idx >= depth_data.len() {
                    continue;
                }

                let depth = depth_data[idx];
                if depth <= 0.0 || depth > self.config.max_distance {
                    continue;
                }

                // Unproject to camera space
                let ray = intrinsics.unproject(x as f32, y as f32);
                let point_camera = Point3::from(ray * depth);

                // Transform to world space
                let point_world = camera_pose.position + camera_pose.rotation * point_camera.coords;

                points.push(Point3::from(point_world));
            }
        }

        points
    }

    fn generate_meshes(&mut self) {
        for chunk in self.chunks.values_mut() {
            if !chunk.dirty || chunk.vertices.len() < 10 {
                continue;
            }

            // Simple Delaunay-like triangulation
            chunk.faces.clear();

            // Sort vertices by position for consistent triangulation
            let mut indices: Vec<usize> = (0..chunk.vertices.len()).collect();
            indices.sort_by(|&a, &b| {
                let va = &chunk.vertices[a];
                let vb = &chunk.vertices[b];
                va.position.x.partial_cmp(&vb.position.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(va.position.z.partial_cmp(&vb.position.z)
                        .unwrap_or(std::cmp::Ordering::Equal))
            });

            // Create triangles from vertex grid
            let grid_step = self.config.voxel_size * 2.0;
            let mut vertex_grid: HashMap<(i32, i32), usize> = HashMap::new();

            for (new_idx, &orig_idx) in indices.iter().enumerate() {
                let v = &chunk.vertices[orig_idx];
                let gx = (v.position.x / grid_step) as i32;
                let gz = (v.position.z / grid_step) as i32;
                vertex_grid.insert((gx, gz), new_idx);
            }

            for (&(gx, gz), &idx) in &vertex_grid {
                // Try to form triangles with neighbors
                if let (Some(&right), Some(&up), Some(&diag)) = (
                    vertex_grid.get(&(gx + 1, gz)),
                    vertex_grid.get(&(gx, gz + 1)),
                    vertex_grid.get(&(gx + 1, gz + 1)),
                ) {
                    chunk.faces.push(MeshFace {
                        indices: [idx as u32, right as u32, up as u32],
                        normal: Vector3::y(),
                    });
                    chunk.faces.push(MeshFace {
                        indices: [right as u32, diag as u32, up as u32],
                        normal: Vector3::y(),
                    });
                }
            }

            // Compute proper normals
            chunk.compute_normals();

            // Classify mesh if enabled
            if self.config.classify_mesh {
                Self::classify_chunk(chunk);
            }
        }
    }

    fn classify_chunk(chunk: &mut MeshChunk) {
        if chunk.faces.is_empty() {
            return;
        }

        // Compute average normal
        let mut avg_normal = Vector3::zeros();
        for face in &chunk.faces {
            avg_normal += face.normal;
        }
        avg_normal = avg_normal.normalize();

        let up_dot = avg_normal.dot(&Vector3::y()).abs();

        chunk.classification = if up_dot > 0.9 {
            if chunk.position.y < 0.3 {
                MeshClassification::Floor
            } else if chunk.position.y > 2.0 {
                MeshClassification::Ceiling
            } else {
                MeshClassification::Table
            }
        } else if up_dot < 0.1 {
            MeshClassification::Wall
        } else {
            MeshClassification::None
        };
    }

    fn update_statistics(&mut self) {
        self.total_vertices = 0;
        self.total_faces = 0;

        for chunk in self.chunks.values() {
            self.total_vertices += chunk.vertices.len();
            self.total_faces += chunk.faces.len();
        }
    }

    fn ray_box_intersect(&self, origin: Point3<f32>, direction: Vector3<f32>, min: Point3<f32>, max: Point3<f32>) -> bool {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..3 {
            let (orig, dir, bmin, bmax) = match i {
                0 => (origin.x, direction.x, min.x, max.x),
                1 => (origin.y, direction.y, min.y, max.y),
                _ => (origin.z, direction.z, min.z, max.z),
            };

            if dir.abs() < 1e-6 {
                if orig < bmin || orig > bmax {
                    return false;
                }
            } else {
                let t1 = (bmin - orig) / dir;
                let t2 = (bmax - orig) / dir;
                let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
            }
        }

        tmax >= tmin && tmax >= 0.0
    }

    fn ray_triangle_intersect(
        &self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        vertices: &[MeshVertex],
        face: &MeshFace,
    ) -> Option<(f32, Point3<f32>)> {
        let v0 = vertices[face.indices[0] as usize].position;
        let v1 = vertices[face.indices[1] as usize].position;
        let v2 = vertices[face.indices[2] as usize].position;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = direction.cross(&edge2);
        let a = edge1.dot(&h);

        if a.abs() < 1e-6 {
            return None;
        }

        let f = 1.0 / a;
        let s = origin - v0;
        let u = f * s.dot(&h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(&edge1);
        let v = f * direction.dot(&q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(&q);
        if t > 1e-6 {
            let point = Point3::from(origin.coords + direction * t);
            Some((t, point))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_vertex() {
        let vertex = MeshVertex::new(Point3::origin(), Vector3::y());
        assert_eq!(vertex.position, Point3::origin());
        assert_eq!(vertex.normal, Vector3::y());
    }

    #[test]
    fn test_mesh_chunk_creation() {
        let chunk = MeshChunk::new(1, Point3::origin(), 1.0);
        assert_eq!(chunk.id, 1);
        assert_eq!(chunk.size, 1.0);
        assert!(chunk.dirty);
    }

    #[test]
    fn test_mesh_chunk_bounds() {
        let chunk = MeshChunk::new(1, Point3::new(1.0, 1.0, 1.0), 2.0);
        let (min, max) = chunk.get_bounds();
        assert!((min.x - 0.0).abs() < 0.01);
        assert!((max.x - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_mesh_chunk_contains() {
        let chunk = MeshChunk::new(1, Point3::origin(), 2.0);
        assert!(chunk.contains(Point3::origin()));
        assert!(!chunk.contains(Point3::new(5.0, 0.0, 0.0)));
    }

    #[test]
    fn test_mesh_config_default() {
        let config = MeshConfig::default();
        assert!((config.chunk_size - 1.0).abs() < 0.01);
        assert!(config.classify_mesh);
    }

    #[test]
    fn test_mesh_reconstructor_creation() {
        let config = MeshConfig::default();
        let reconstructor = MeshReconstructor::new(config);
        assert!(!reconstructor.is_running());
    }

    #[test]
    fn test_mesh_reconstructor_start_stop() {
        let config = MeshConfig::default();
        let mut reconstructor = MeshReconstructor::new(config);
        
        reconstructor.start();
        assert!(reconstructor.is_running());
        
        reconstructor.stop();
        assert!(!reconstructor.is_running());
    }

    #[test]
    fn test_mesh_classification() {
        assert_eq!(MeshClassification::Floor, MeshClassification::Floor);
        assert_ne!(MeshClassification::Floor, MeshClassification::Wall);
    }
}
