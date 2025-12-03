//! Scene Understanding Module
//!
//! Context-aware environmental perception for AR experiences.
//! Analyzes spatial layout, surfaces, lighting, and semantic content.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use nalgebra::{Point3, Vector3, Matrix4, UnitQuaternion};
use uuid::Uuid;

pub mod analyzer;
pub mod surfaces;
pub mod lighting;
pub mod semantic;
pub mod anchors;

pub use analyzer::SceneAnalyzer;
pub use surfaces::{SurfaceDetector, Surface, SurfaceType, Plane};
pub use lighting::{LightingEstimator, LightProbe, AmbientLight, DirectionalLight};
pub use semantic::{SemanticLabeler, SemanticLabel, SceneObject, ObjectCategory};
pub use anchors::{AnchorManager, SpatialAnchor, AnchorState};

/// Unique identifier for scene elements
pub type SceneId = Uuid;

/// Scene understanding engine configuration
#[derive(Debug, Clone)]
pub struct SceneConfig {
    /// Minimum surface area for detection (m²)
    pub min_surface_area: f32,
    /// Maximum surfaces to track simultaneously
    pub max_surfaces: usize,
    /// Surface detection confidence threshold
    pub surface_confidence: f32,
    /// Lighting estimation update rate
    pub lighting_update_hz: f32,
    /// Semantic labeling enabled
    pub semantic_enabled: bool,
    /// Object detection confidence threshold
    pub object_confidence: f32,
    /// Maximum objects to track
    pub max_objects: usize,
    /// Anchor persistence enabled
    pub anchor_persistence: bool,
    /// Scene mesh reconstruction enabled
    pub mesh_reconstruction: bool,
    /// Mesh resolution (vertices per m³)
    pub mesh_resolution: f32,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            min_surface_area: 0.1,
            max_surfaces: 100,
            surface_confidence: 0.7,
            lighting_update_hz: 10.0,
            semantic_enabled: true,
            object_confidence: 0.6,
            max_objects: 50,
            anchor_persistence: true,
            mesh_reconstruction: true,
            mesh_resolution: 1000.0,
        }
    }
}

/// Main scene understanding engine
#[derive(Debug)]
pub struct SceneEngine {
    config: SceneConfig,
    state: SceneState,
    surface_detector: SurfaceDetector,
    lighting_estimator: LightingEstimator,
    semantic_labeler: SemanticLabeler,
    anchor_manager: AnchorManager,
    scene_graph: SceneGraph,
    last_update: Instant,
}

/// Current scene state
#[derive(Debug, Clone)]
pub struct SceneState {
    /// Detected surfaces
    pub surfaces: Vec<Surface>,
    /// Current lighting conditions
    pub lighting: SceneLighting,
    /// Detected objects with semantic labels
    pub objects: Vec<SceneObject>,
    /// Active spatial anchors
    pub anchors: Vec<SpatialAnchor>,
    /// Scene mesh (if reconstruction enabled)
    pub mesh: Option<SceneMesh>,
    /// Scene boundaries (AABB)
    pub bounds: Option<SceneBounds>,
    /// Processing statistics
    pub stats: SceneStats,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            surfaces: Vec::new(),
            lighting: SceneLighting::default(),
            objects: Vec::new(),
            anchors: Vec::new(),
            mesh: None,
            bounds: None,
            stats: SceneStats::default(),
        }
    }
}

/// Scene lighting conditions
#[derive(Debug, Clone)]
pub struct SceneLighting {
    /// Ambient light estimate
    pub ambient: AmbientLight,
    /// Primary directional light (sun/main source)
    pub main_light: Option<DirectionalLight>,
    /// Additional light sources
    pub lights: Vec<LightProbe>,
    /// Shadow direction estimate
    pub shadow_direction: Option<Vector3<f32>>,
    /// Overall brightness (0-1)
    pub brightness: f32,
    /// Color temperature (Kelvin)
    pub color_temperature: f32,
}

impl Default for SceneLighting {
    fn default() -> Self {
        Self {
            ambient: AmbientLight::default(),
            main_light: None,
            lights: Vec::new(),
            shadow_direction: None,
            brightness: 0.5,
            color_temperature: 5500.0, // Daylight
        }
    }
}

/// Scene mesh for spatial reconstruction
#[derive(Debug, Clone)]
pub struct SceneMesh {
    /// Mesh vertices
    pub vertices: Vec<Point3<f32>>,
    /// Vertex normals
    pub normals: Vec<Vector3<f32>>,
    /// Triangle indices
    pub indices: Vec<u32>,
    /// Vertex colors (if available)
    pub colors: Option<Vec<[f32; 4]>>,
    /// Last mesh update time
    pub updated_at: Instant,
    /// Mesh completeness (0-1)
    pub completeness: f32,
}

impl SceneMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            colors: None,
            updated_at: Instant::now(),
            completeness: 0.0,
        }
    }
    
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
    
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

impl Default for SceneMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Axis-aligned bounding box for scene
#[derive(Debug, Clone, Copy)]
pub struct SceneBounds {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl SceneBounds {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }
    
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
    
    pub fn center(&self) -> Point3<f32> {
        Point3::from((self.min.coords + self.max.coords) / 2.0)
    }
    
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }
    
    pub fn contains(&self, point: &Point3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn expand(&mut self, point: &Point3<f32>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }
}

/// Scene processing statistics
#[derive(Debug, Clone, Default)]
pub struct SceneStats {
    /// Total surfaces detected
    pub surface_count: usize,
    /// Total objects detected
    pub object_count: usize,
    /// Total anchors active
    pub anchor_count: usize,
    /// Mesh vertices
    pub mesh_vertices: usize,
    /// Mesh triangles
    pub mesh_triangles: usize,
    /// Processing time (last frame)
    pub process_time_ms: f32,
    /// Updates per second
    pub updates_per_second: f32,
}

/// Hierarchical scene representation
#[derive(Debug)]
pub struct SceneGraph {
    /// Root nodes
    nodes: HashMap<SceneId, SceneNode>,
    /// Parent-child relationships
    hierarchy: HashMap<SceneId, Vec<SceneId>>,
    /// Node transforms
    transforms: HashMap<SceneId, Matrix4<f32>>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            hierarchy: HashMap::new(),
            transforms: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: SceneNode) -> SceneId {
        let id = node.id;
        self.transforms.insert(id, node.transform);
        self.nodes.insert(id, node);
        id
    }
    
    pub fn remove_node(&mut self, id: SceneId) -> Option<SceneNode> {
        self.transforms.remove(&id);
        self.hierarchy.remove(&id);
        self.nodes.remove(&id)
    }
    
    pub fn get_node(&self, id: SceneId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }
    
    pub fn set_parent(&mut self, child: SceneId, parent: SceneId) {
        self.hierarchy.entry(parent).or_default().push(child);
    }
    
    pub fn get_children(&self, parent: SceneId) -> Option<&Vec<SceneId>> {
        self.hierarchy.get(&parent)
    }
    
    pub fn get_world_transform(&self, id: SceneId) -> Option<Matrix4<f32>> {
        self.transforms.get(&id).copied()
    }
    
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A node in the scene graph
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: SceneId,
    pub name: String,
    pub node_type: SceneNodeType,
    pub transform: Matrix4<f32>,
    pub visible: bool,
    pub metadata: HashMap<String, String>,
}

impl SceneNode {
    pub fn new(name: impl Into<String>, node_type: SceneNodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_type,
            transform: Matrix4::identity(),
            visible: true,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_transform(mut self, transform: Matrix4<f32>) -> Self {
        self.transform = transform;
        self
    }
    
    pub fn position(&self) -> Point3<f32> {
        Point3::new(
            self.transform[(0, 3)],
            self.transform[(1, 3)],
            self.transform[(2, 3)],
        )
    }
}

/// Types of scene nodes
#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeType {
    /// Empty transform node
    Empty,
    /// Surface reference
    Surface(SceneId),
    /// Object reference
    Object(SceneId),
    /// Anchor reference
    Anchor(SceneId),
    /// Light source
    Light(SceneId),
    /// Virtual content
    Virtual(String),
    /// Group container
    Group,
}

impl SceneEngine {
    pub fn new(config: SceneConfig) -> Self {
        Self {
            surface_detector: SurfaceDetector::new(
                config.min_surface_area,
                config.surface_confidence,
            ),
            lighting_estimator: LightingEstimator::new(config.lighting_update_hz),
            semantic_labeler: SemanticLabeler::new(config.object_confidence),
            anchor_manager: AnchorManager::new(config.anchor_persistence),
            scene_graph: SceneGraph::new(),
            state: SceneState::default(),
            last_update: Instant::now(),
            config,
        }
    }
    
    /// Process a depth frame for scene understanding
    pub fn process_depth_frame(&mut self, depth_data: &DepthFrame) -> &SceneState {
        let start = Instant::now();
        
        // Surface detection
        let surfaces = self.surface_detector.detect(&depth_data.points);
        self.state.surfaces = surfaces.into_iter()
            .take(self.config.max_surfaces)
            .collect();
        
        // Update scene bounds
        self.update_bounds(&depth_data.points);
        
        // Mesh reconstruction
        if self.config.mesh_reconstruction {
            self.update_mesh(&depth_data.points);
        }
        
        // Update statistics
        self.state.stats.surface_count = self.state.surfaces.len();
        self.state.stats.process_time_ms = start.elapsed().as_secs_f32() * 1000.0;
        
        let elapsed = self.last_update.elapsed();
        if elapsed.as_secs_f32() > 0.0 {
            self.state.stats.updates_per_second = 1.0 / elapsed.as_secs_f32();
        }
        self.last_update = Instant::now();
        
        &self.state
    }
    
    /// Process a color frame for semantic understanding
    pub fn process_color_frame(&mut self, color_data: &ColorFrame) -> &SceneState {
        // Lighting estimation
        self.state.lighting = SceneLighting {
            ambient: self.lighting_estimator.estimate_ambient(&color_data.pixels),
            main_light: self.lighting_estimator.estimate_main_light(&color_data.pixels),
            lights: Vec::new(),
            shadow_direction: None,
            brightness: self.lighting_estimator.estimate_brightness(&color_data.pixels),
            color_temperature: self.lighting_estimator.estimate_temperature(&color_data.pixels),
        };
        
        // Semantic labeling
        if self.config.semantic_enabled {
            let objects = self.semantic_labeler.detect_objects(&color_data.pixels, color_data.width, color_data.height);
            self.state.objects = objects.into_iter()
                .take(self.config.max_objects)
                .collect();
            self.state.stats.object_count = self.state.objects.len();
        }
        
        &self.state
    }
    
    /// Create a spatial anchor at a location
    pub fn create_anchor(&mut self, position: Point3<f32>, name: Option<String>) -> SceneId {
        let anchor = self.anchor_manager.create_anchor(position, name);
        let id = anchor.id;
        self.state.anchors.push(anchor);
        self.state.stats.anchor_count = self.state.anchors.len();
        id
    }
    
    /// Remove a spatial anchor
    pub fn remove_anchor(&mut self, id: SceneId) -> bool {
        if self.anchor_manager.remove_anchor(id) {
            self.state.anchors.retain(|a| a.id != id);
            self.state.stats.anchor_count = self.state.anchors.len();
            true
        } else {
            false
        }
    }
    
    /// Find the nearest surface to a point
    pub fn find_nearest_surface(&self, point: &Point3<f32>) -> Option<&Surface> {
        self.state.surfaces.iter()
            .min_by(|a, b| {
                let dist_a = a.distance_to_point(point);
                let dist_b = b.distance_to_point(point);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
    
    /// Raycast against scene surfaces
    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>) -> Option<RaycastHit> {
        let ray = Ray { origin, direction: direction.normalize() };
        
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = f32::MAX;
        
        for surface in &self.state.surfaces {
            if let Some(hit) = surface.intersect_ray(&ray) {
                let distance = (hit.point - origin).norm();
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_hit = Some(hit);
                }
            }
        }
        
        closest_hit
    }
    
    /// Find surfaces suitable for content placement
    pub fn find_placement_surfaces(&self, content_size: Vector3<f32>) -> Vec<PlacementCandidate> {
        let required_area = content_size.x * content_size.z;
        
        self.state.surfaces.iter()
            .filter(|s| s.surface_type == SurfaceType::Horizontal && s.area >= required_area)
            .map(|s| PlacementCandidate {
                surface_id: s.id,
                position: s.center,
                normal: s.plane.normal,
                score: self.calculate_placement_score(s, content_size),
            })
            .filter(|c| c.score > 0.5)
            .collect()
    }
    
    fn calculate_placement_score(&self, surface: &Surface, content_size: Vector3<f32>) -> f32 {
        let required_area = content_size.x * content_size.z;
        let area_ratio = (surface.area / required_area).min(2.0) / 2.0;
        let confidence = surface.confidence;
        let stability = if surface.surface_type == SurfaceType::Horizontal { 1.0 } else { 0.5 };
        
        area_ratio * 0.4 + confidence * 0.4 + stability * 0.2
    }
    
    fn update_bounds(&mut self, points: &[Point3<f32>]) {
        if points.is_empty() {
            return;
        }
        
        let mut bounds = self.state.bounds.unwrap_or_else(|| {
            SceneBounds::new(points[0], points[0])
        });
        
        for point in points {
            bounds.expand(point);
        }
        
        self.state.bounds = Some(bounds);
    }
    
    fn update_mesh(&mut self, points: &[Point3<f32>]) {
        let mesh = self.state.mesh.get_or_insert_with(SceneMesh::new);
        
        // Simple point cloud to mesh (placeholder - real impl would use marching cubes)
        mesh.vertices = points.to_vec();
        mesh.normals = points.iter().map(|_| Vector3::new(0.0, 1.0, 0.0)).collect();
        mesh.updated_at = Instant::now();
        mesh.completeness = (points.len() as f32 / 10000.0).min(1.0);
        
        self.state.stats.mesh_vertices = mesh.vertex_count();
        self.state.stats.mesh_triangles = mesh.triangle_count();
    }
    
    /// Get current scene state
    pub fn state(&self) -> &SceneState {
        &self.state
    }
    
    /// Get scene graph
    pub fn scene_graph(&self) -> &SceneGraph {
        &self.scene_graph
    }
    
    /// Get mutable scene graph
    pub fn scene_graph_mut(&mut self) -> &mut SceneGraph {
        &mut self.scene_graph
    }
}

/// Depth frame data
#[derive(Debug, Clone)]
pub struct DepthFrame {
    /// Point cloud from depth sensor
    pub points: Vec<Point3<f32>>,
    /// Frame timestamp
    pub timestamp: Instant,
    /// Sensor intrinsics
    pub intrinsics: Option<CameraIntrinsics>,
}

/// Color frame data
#[derive(Debug, Clone)]
pub struct ColorFrame {
    /// RGBA pixels
    pub pixels: Vec<[u8; 4]>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Frame timestamp
    pub timestamp: Instant,
}

/// Camera intrinsic parameters
#[derive(Debug, Clone, Copy)]
pub struct CameraIntrinsics {
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub width: u32,
    pub height: u32,
}

/// 3D ray for raycasting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self { origin, direction: direction.normalize() }
    }
    
    pub fn point_at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }
}

/// Result of a raycast
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// Hit point in world space
    pub point: Point3<f32>,
    /// Surface normal at hit point
    pub normal: Vector3<f32>,
    /// Distance from ray origin
    pub distance: f32,
    /// ID of hit surface
    pub surface_id: Option<SceneId>,
    /// ID of hit object
    pub object_id: Option<SceneId>,
}

/// Candidate for content placement
#[derive(Debug, Clone)]
pub struct PlacementCandidate {
    /// ID of the surface
    pub surface_id: SceneId,
    /// Suggested placement position
    pub position: Point3<f32>,
    /// Surface normal
    pub normal: Vector3<f32>,
    /// Suitability score (0-1)
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scene_config_default() {
        let config = SceneConfig::default();
        assert_eq!(config.min_surface_area, 0.1);
        assert_eq!(config.max_surfaces, 100);
        assert!(config.semantic_enabled);
    }
    
    #[test]
    fn test_scene_engine_creation() {
        let engine = SceneEngine::new(SceneConfig::default());
        assert!(engine.state.surfaces.is_empty());
        assert!(engine.state.objects.is_empty());
    }
    
    #[test]
    fn test_scene_bounds() {
        let mut bounds = SceneBounds::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        
        assert_eq!(bounds.volume(), 1.0);
        assert!(bounds.contains(&Point3::new(0.5, 0.5, 0.5)));
        assert!(!bounds.contains(&Point3::new(2.0, 0.5, 0.5)));
        
        bounds.expand(&Point3::new(2.0, 2.0, 2.0));
        assert_eq!(bounds.volume(), 8.0);
    }
    
    #[test]
    fn test_scene_graph() {
        let mut graph = SceneGraph::new();
        
        let node1 = SceneNode::new("Node1", SceneNodeType::Empty);
        let id1 = node1.id;
        graph.add_node(node1);
        
        let node2 = SceneNode::new("Node2", SceneNodeType::Empty);
        let id2 = node2.id;
        graph.add_node(node2);
        
        graph.set_parent(id2, id1);
        
        assert_eq!(graph.node_count(), 2);
        assert!(graph.get_node(id1).is_some());
        assert_eq!(graph.get_children(id1).unwrap().len(), 1);
    }
    
    #[test]
    fn test_scene_node() {
        let node = SceneNode::new("TestNode", SceneNodeType::Group);
        assert_eq!(node.name, "TestNode");
        assert_eq!(node.node_type, SceneNodeType::Group);
        assert!(node.visible);
        
        let pos = node.position();
        assert_eq!(pos, Point3::new(0.0, 0.0, 0.0));
    }
    
    #[test]
    fn test_ray() {
        let ray = Ray::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        );
        
        let point = ray.point_at(5.0);
        assert!((point.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_scene_mesh() {
        let mut mesh = SceneMesh::new();
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        mesh.indices = vec![0, 1, 2];
        
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }
    
    #[test]
    fn test_scene_lighting_default() {
        let lighting = SceneLighting::default();
        assert_eq!(lighting.brightness, 0.5);
        assert_eq!(lighting.color_temperature, 5500.0);
    }
    
    #[test]
    fn test_anchor_creation() {
        let mut engine = SceneEngine::new(SceneConfig::default());
        
        let id = engine.create_anchor(Point3::new(1.0, 0.5, 2.0), Some("Test".to_string()));
        assert_eq!(engine.state.anchors.len(), 1);
        assert_eq!(engine.state.stats.anchor_count, 1);
        
        assert!(engine.remove_anchor(id));
        assert_eq!(engine.state.anchors.len(), 0);
    }
}
