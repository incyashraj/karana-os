// AR Renderer for Kāraṇa OS
// Manages rendering pipeline and scene management

use super::*;
use std::collections::HashMap;

/// Render quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl RenderQuality {
    pub fn shadow_resolution(&self) -> u32 {
        match self {
            RenderQuality::Low => 512,
            RenderQuality::Medium => 1024,
            RenderQuality::High => 2048,
            RenderQuality::Ultra => 4096,
        }
    }
    
    pub fn msaa_samples(&self) -> u32 {
        match self {
            RenderQuality::Low => 1,
            RenderQuality::Medium => 2,
            RenderQuality::High => 4,
            RenderQuality::Ultra => 8,
        }
    }
}

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub quality: RenderQuality,
    pub enable_shadows: bool,
    pub enable_reflections: bool,
    pub enable_bloom: bool,
    pub enable_fxaa: bool,
    pub ambient_occlusion: bool,
    pub max_lights: usize,
    pub render_scale: f32,
    pub fov_degrees: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            quality: RenderQuality::Medium,
            enable_shadows: true,
            enable_reflections: false,
            enable_bloom: false,
            enable_fxaa: true,
            ambient_occlusion: false,
            max_lights: 4,
            render_scale: 1.0,
            fov_degrees: 90.0,
            near_plane: 0.01,
            far_plane: 100.0,
        }
    }
}

/// Light types for AR scene
#[derive(Debug, Clone)]
pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
    Spot(SpotLight),
    Ambient(AmbientLight),
}

impl Light {
    pub fn id(&self) -> ContentId {
        match self {
            Light::Directional(l) => l.id,
            Light::Point(l) => l.id,
            Light::Spot(l) => l.id,
            Light::Ambient(l) => l.id,
        }
    }
}

/// Directional light (sun)
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub id: ContentId,
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
}

impl DirectionalLight {
    pub fn new(direction: Vector3<f32>) -> Self {
        Self {
            id: next_content_id(),
            direction: direction.normalize(),
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            cast_shadows: true,
        }
    }
}

/// Point light
#[derive(Debug, Clone)]
pub struct PointLight {
    pub id: ContentId,
    pub position: Point3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
}

impl PointLight {
    pub fn new(position: Point3<f32>) -> Self {
        Self {
            id: next_content_id(),
            position,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            cast_shadows: false,
        }
    }
}

/// Spot light
#[derive(Debug, Clone)]
pub struct SpotLight {
    pub id: ContentId,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
}

impl SpotLight {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            id: next_content_id(),
            position,
            direction: direction.normalize(),
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            inner_cone_angle: 30.0,
            outer_cone_angle: 45.0,
            cast_shadows: true,
        }
    }
}

/// Ambient light
#[derive(Debug, Clone)]
pub struct AmbientLight {
    pub id: ContentId,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl AmbientLight {
    pub fn new(intensity: f32) -> Self {
        Self {
            id: next_content_id(),
            color: [1.0, 1.0, 1.0],
            intensity,
        }
    }
}

/// Camera for AR rendering
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.01,
            far: 100.0,
        }
    }
    
    /// Get view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let rotation_matrix = self.rotation.to_homogeneous();
        let translation = Matrix4::new_translation(&(-self.position.coords));
        rotation_matrix.transpose() * translation
    }
    
    /// Get projection matrix
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(
            self.aspect_ratio,
            self.fov.to_radians(),
            self.near,
            self.far,
        )
    }
    
    /// Get view-projection matrix
    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
    
    /// Get frustum for culling
    pub fn frustum(&self) -> Frustum {
        Frustum::from_matrix(&self.view_projection_matrix())
    }
    
    /// Get forward direction
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(0.0, 0.0, -1.0)
    }
    
    /// Get up direction
    pub fn up(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(0.0, 1.0, 0.0)
    }
    
    /// Get right direction
    pub fn right(&self) -> Vector3<f32> {
        self.rotation * Vector3::new(1.0, 0.0, 0.0)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// AR scene containing all renderable content
pub struct ARScene {
    pub meshes: HashMap<ContentId, Mesh>,
    pub materials: HashMap<ContentId, Material>,
    pub objects: HashMap<ContentId, Object3D>,
    pub billboards: HashMap<ContentId, Billboard>,
    pub lines: HashMap<ContentId, LineRenderer>,
    pub point_clouds: HashMap<ContentId, PointCloud>,
    pub labels: HashMap<ContentId, TextLabel>,
    pub panels: HashMap<ContentId, UIPanel>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub ambient_color: [f32; 3],
}

impl ARScene {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            objects: HashMap::new(),
            billboards: HashMap::new(),
            lines: HashMap::new(),
            point_clouds: HashMap::new(),
            labels: HashMap::new(),
            panels: HashMap::new(),
            lights: Vec::new(),
            camera: Camera::new(),
            ambient_color: [0.1, 0.1, 0.1],
        }
    }
    
    /// Add a mesh to the scene
    pub fn add_mesh(&mut self, mesh: Mesh) -> ContentId {
        let id = mesh.id;
        self.meshes.insert(id, mesh);
        id
    }
    
    /// Add a material to the scene
    pub fn add_material(&mut self, material: Material) -> ContentId {
        let id = material.id;
        self.materials.insert(id, material);
        id
    }
    
    /// Add a 3D object to the scene
    pub fn add_object(&mut self, object: Object3D) -> ContentId {
        let id = object.id;
        self.objects.insert(id, object);
        id
    }
    
    /// Add a billboard
    pub fn add_billboard(&mut self, billboard: Billboard) -> ContentId {
        let id = billboard.id;
        self.billboards.insert(id, billboard);
        id
    }
    
    /// Add a line renderer
    pub fn add_line(&mut self, line: LineRenderer) -> ContentId {
        let id = line.id;
        self.lines.insert(id, line);
        id
    }
    
    /// Add a point cloud
    pub fn add_point_cloud(&mut self, cloud: PointCloud) -> ContentId {
        let id = cloud.id;
        self.point_clouds.insert(id, cloud);
        id
    }
    
    /// Add a text label
    pub fn add_label(&mut self, label: TextLabel) -> ContentId {
        let id = label.id;
        self.labels.insert(id, label);
        id
    }
    
    /// Add a UI panel
    pub fn add_panel(&mut self, panel: UIPanel) -> ContentId {
        let id = panel.id;
        self.panels.insert(id, panel);
        id
    }
    
    /// Add a light
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }
    
    /// Remove an object by ID
    pub fn remove_object(&mut self, id: ContentId) -> bool {
        self.objects.remove(&id).is_some()
    }
    
    /// Get object count
    pub fn object_count(&self) -> usize {
        self.objects.len() + self.billboards.len() + 
        self.labels.len() + self.panels.len()
    }
    
    /// Clear all content
    pub fn clear(&mut self) {
        self.meshes.clear();
        self.materials.clear();
        self.objects.clear();
        self.billboards.clear();
        self.lines.clear();
        self.point_clouds.clear();
        self.labels.clear();
        self.panels.clear();
        self.lights.clear();
    }
}

impl Default for ARScene {
    fn default() -> Self {
        Self::new()
    }
}

/// AR Renderer
pub struct ARRenderer {
    config: RendererConfig,
    scene: ARScene,
    stats: RenderStats,
    frame_count: u64,
}

impl ARRenderer {
    pub fn new(config: RendererConfig) -> Self {
        let mut scene = ARScene::new();
        
        // Add default ambient light
        scene.add_light(Light::Ambient(AmbientLight::new(0.3)));
        
        Self {
            config,
            scene,
            stats: RenderStats::default(),
            frame_count: 0,
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
    }
    
    /// Get scene reference
    pub fn scene(&self) -> &ARScene {
        &self.scene
    }
    
    /// Get mutable scene reference
    pub fn scene_mut(&mut self) -> &mut ARScene {
        &mut self.scene
    }
    
    /// Get render statistics
    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }
    
    /// Update camera pose
    pub fn update_camera(&mut self, position: Point3<f32>, rotation: UnitQuaternion<f32>) {
        self.scene.camera.position = position;
        self.scene.camera.rotation = rotation;
    }
    
    /// Perform frustum culling and prepare render list
    pub fn cull_and_prepare(&mut self) -> Vec<ContentId> {
        let frustum = self.scene.camera.frustum();
        let mut visible = Vec::new();
        
        self.stats.reset();
        
        for (id, obj) in &self.scene.objects {
            if obj.visibility != Visibility::Visible {
                continue;
            }
            
            // Get mesh bounds and transform
            if let Some(mesh) = self.scene.meshes.get(&obj.mesh_id) {
                let world_bounds = self.transform_bounds(&mesh.bounds, &obj.transform);
                
                if frustum.contains_box(&world_bounds) {
                    visible.push(*id);
                    self.stats.objects_rendered += 1;
                    self.stats.triangles_rendered += mesh.triangle_count() as u64;
                } else {
                    self.stats.objects_culled += 1;
                }
            }
        }
        
        // Sort by layer
        visible.sort_by(|a, b| {
            let layer_a = self.scene.objects.get(a).map(|o| o.layer.render_order()).unwrap_or(0);
            let layer_b = self.scene.objects.get(b).map(|o| o.layer.render_order()).unwrap_or(0);
            layer_a.cmp(&layer_b)
        });
        
        visible
    }
    
    /// Transform bounding box to world space
    fn transform_bounds(&self, bounds: &BoundingBox3D, transform: &Transform) -> BoundingBox3D {
        let corners = [
            Point3::new(bounds.min.x, bounds.min.y, bounds.min.z),
            Point3::new(bounds.max.x, bounds.min.y, bounds.min.z),
            Point3::new(bounds.min.x, bounds.max.y, bounds.min.z),
            Point3::new(bounds.max.x, bounds.max.y, bounds.min.z),
            Point3::new(bounds.min.x, bounds.min.y, bounds.max.z),
            Point3::new(bounds.max.x, bounds.min.y, bounds.max.z),
            Point3::new(bounds.min.x, bounds.max.y, bounds.max.z),
            Point3::new(bounds.max.x, bounds.max.y, bounds.max.z),
        ];
        
        let matrix = transform.to_matrix();
        let mut new_min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut new_max = Point3::new(f32::MIN, f32::MIN, f32::MIN);
        
        for corner in &corners {
            let transformed = matrix.transform_point(corner);
            new_min.x = new_min.x.min(transformed.x);
            new_min.y = new_min.y.min(transformed.y);
            new_min.z = new_min.z.min(transformed.z);
            new_max.x = new_max.x.max(transformed.x);
            new_max.y = new_max.y.max(transformed.y);
            new_max.z = new_max.z.max(transformed.z);
        }
        
        BoundingBox3D::new(new_min, new_max)
    }
    
    /// Simulate render frame (actual GPU work would go here)
    pub fn render_frame(&mut self) {
        let _visible = self.cull_and_prepare();
        self.frame_count += 1;
        self.stats.draw_calls = self.stats.objects_rendered;
    }
    
    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_quality() {
        assert_eq!(RenderQuality::Low.shadow_resolution(), 512);
        assert_eq!(RenderQuality::Ultra.msaa_samples(), 8);
    }
    
    #[test]
    fn test_camera() {
        let camera = Camera::new();
        
        let forward = camera.forward();
        assert!((forward.z - (-1.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_camera_matrices() {
        let camera = Camera::new();
        
        let view = camera.view_matrix();
        let proj = camera.projection_matrix();
        let vp = camera.view_projection_matrix();
        
        // VP should be proj * view
        let expected = proj * view;
        assert!((vp[(0, 0)] - expected[(0, 0)]).abs() < 0.001);
    }
    
    #[test]
    fn test_directional_light() {
        let light = DirectionalLight::new(Vector3::new(1.0, -1.0, 0.0));
        
        // Direction should be normalized
        let len = light.direction.norm();
        assert!((len - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_ar_scene() {
        let mut scene = ARScene::new();
        
        let mesh = Mesh::cube(1.0);
        let mesh_id = scene.add_mesh(mesh);
        
        let mat = Material::solid_color(1.0, 0.0, 0.0);
        let mat_id = scene.add_material(mat);
        
        let obj = Object3D::new("cube", mesh_id, mat_id);
        scene.add_object(obj);
        
        assert_eq!(scene.meshes.len(), 1);
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scene.objects.len(), 1);
    }
    
    #[test]
    fn test_ar_renderer() {
        let config = RendererConfig::default();
        let mut renderer = ARRenderer::new(config);
        
        let mesh = Mesh::cube(1.0);
        let mesh_id = renderer.scene_mut().add_mesh(mesh);
        
        let mat = Material::solid_color(1.0, 1.0, 1.0);
        let mat_id = renderer.scene_mut().add_material(mat);
        
        let obj = Object3D::new("test", mesh_id, mat_id)
            .with_position(0.0, 0.0, -5.0);
        renderer.scene_mut().add_object(obj);
        
        renderer.render_frame();
        
        assert!(renderer.stats().objects_rendered > 0 || renderer.stats().objects_culled > 0);
    }
    
    #[test]
    fn test_scene_clear() {
        let mut scene = ARScene::new();
        
        scene.add_mesh(Mesh::cube(1.0));
        scene.add_label(TextLabel::new("test", Point3::origin()));
        
        scene.clear();
        
        assert_eq!(scene.meshes.len(), 0);
        assert_eq!(scene.labels.len(), 0);
    }
    
    #[test]
    fn test_renderer_camera_update() {
        let mut renderer = ARRenderer::new(RendererConfig::default());
        
        let new_pos = Point3::new(1.0, 2.0, 3.0);
        let new_rot = UnitQuaternion::identity();
        
        renderer.update_camera(new_pos, new_rot);
        
        assert_eq!(renderer.scene().camera.position, new_pos);
    }
}
