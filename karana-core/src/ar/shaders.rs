// AR Shaders for Kāraṇa OS
// Shader definitions and uniform management

use super::*;
use std::collections::HashMap;

/// Shader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    Compute,
}

/// Built-in shader programs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinShader {
    /// Standard PBR shader
    Standard,
    /// Unlit solid color
    Unlit,
    /// Transparent/translucent
    Transparent,
    /// Billboard/sprite shader
    Billboard,
    /// Text rendering
    Text,
    /// Line rendering
    Line,
    /// Point cloud
    PointCloud,
    /// UI panel
    UI,
    /// Depth only (shadow pass)
    DepthOnly,
    /// Skybox
    Skybox,
    /// Post-process bloom
    Bloom,
    /// Post-process FXAA
    FXAA,
}

/// Shader uniform value
#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
    Int(i32),
    Texture(ContentId),
}

/// Shader program
#[derive(Debug, Clone)]
pub struct ShaderProgram {
    pub id: ContentId,
    pub name: String,
    pub builtin: Option<BuiltinShader>,
    pub uniforms: HashMap<String, UniformValue>,
    pub vertex_source: Option<String>,
    pub fragment_source: Option<String>,
}

impl ShaderProgram {
    pub fn new(name: &str) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            builtin: None,
            uniforms: HashMap::new(),
            vertex_source: None,
            fragment_source: None,
        }
    }
    
    /// Create from builtin shader
    pub fn from_builtin(shader: BuiltinShader) -> Self {
        let name = format!("{:?}", shader);
        Self {
            id: next_content_id(),
            name,
            builtin: Some(shader),
            uniforms: HashMap::new(),
            vertex_source: None,
            fragment_source: None,
        }
    }
    
    /// Set uniform value
    pub fn set_uniform(&mut self, name: &str, value: UniformValue) {
        self.uniforms.insert(name.to_string(), value);
    }
    
    /// Get uniform value
    pub fn get_uniform(&self, name: &str) -> Option<&UniformValue> {
        self.uniforms.get(name)
    }
    
    /// Check if shader is builtin
    pub fn is_builtin(&self) -> bool {
        self.builtin.is_some()
    }
}

/// Shader uniform block for common data
#[derive(Debug, Clone, Default)]
pub struct CommonUniforms {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
    pub view_projection_matrix: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
    pub time: f32,
    pub delta_time: f32,
    pub screen_size: [f32; 2],
}

impl CommonUniforms {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update from camera
    pub fn update_from_camera(&mut self, camera: &Camera) {
        let view = camera.view_matrix();
        let proj = camera.projection_matrix();
        let vp = camera.view_projection_matrix();
        
        self.view_matrix = matrix_to_array(&view);
        self.projection_matrix = matrix_to_array(&proj);
        self.view_projection_matrix = matrix_to_array(&vp);
        self.camera_position = [
            camera.position.x,
            camera.position.y,
            camera.position.z,
        ];
    }
}

/// Convert nalgebra matrix to array
fn matrix_to_array(m: &Matrix4<f32>) -> [[f32; 4]; 4] {
    [
        [m[(0, 0)], m[(1, 0)], m[(2, 0)], m[(3, 0)]],
        [m[(0, 1)], m[(1, 1)], m[(2, 1)], m[(3, 1)]],
        [m[(0, 2)], m[(1, 2)], m[(2, 2)], m[(3, 2)]],
        [m[(0, 3)], m[(1, 3)], m[(2, 3)], m[(3, 3)]],
    ]
}

/// Per-object uniform block
#[derive(Debug, Clone, Default)]
pub struct ObjectUniforms {
    pub model_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 3]; 3],
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
}

impl ObjectUniforms {
    pub fn new() -> Self {
        Self {
            model_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            normal_matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }
    }
    
    /// Update from transform
    pub fn update_from_transform(&mut self, transform: &Transform) {
        let model = transform.to_matrix();
        self.model_matrix = matrix_to_array(&model);
        
        // Normal matrix is inverse transpose of upper-left 3x3
        let inv = model.try_inverse().unwrap_or(Matrix4::identity());
        self.normal_matrix = [
            [inv[(0, 0)], inv[(0, 1)], inv[(0, 2)]],
            [inv[(1, 0)], inv[(1, 1)], inv[(1, 2)]],
            [inv[(2, 0)], inv[(2, 1)], inv[(2, 2)]],
        ];
    }
    
    /// Update from material
    pub fn update_from_material(&mut self, material: &Material) {
        self.color = material.color;
        self.metallic = material.metallic;
        self.roughness = material.roughness;
        self.emissive = material.emissive;
    }
}

/// Lighting uniform block
#[derive(Debug, Clone)]
pub struct LightingUniforms {
    pub ambient_color: [f32; 3],
    pub directional_light_dir: [f32; 3],
    pub directional_light_color: [f32; 3],
    pub directional_light_intensity: f32,
    pub point_lights: Vec<PointLightData>,
    pub spot_lights: Vec<SpotLightData>,
}

impl Default for LightingUniforms {
    fn default() -> Self {
        Self {
            ambient_color: [0.1, 0.1, 0.1],
            directional_light_dir: [0.0, -1.0, 0.0],
            directional_light_color: [1.0, 1.0, 1.0],
            directional_light_intensity: 1.0,
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        }
    }
}

/// Point light data for uniform buffer
#[derive(Debug, Clone, Copy)]
pub struct PointLightData {
    pub position: [f32; 3],
    pub range: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

/// Spot light data for uniform buffer
#[derive(Debug, Clone, Copy)]
pub struct SpotLightData {
    pub position: [f32; 3],
    pub range: f32,
    pub direction: [f32; 3],
    pub inner_cone: f32,
    pub color: [f32; 3],
    pub outer_cone: f32,
    pub intensity: f32,
}

/// Shader manager
pub struct ShaderManager {
    shaders: HashMap<ContentId, ShaderProgram>,
    builtin_cache: HashMap<BuiltinShader, ContentId>,
    common_uniforms: CommonUniforms,
}

impl ShaderManager {
    pub fn new() -> Self {
        let mut manager = Self {
            shaders: HashMap::new(),
            builtin_cache: HashMap::new(),
            common_uniforms: CommonUniforms::new(),
        };
        
        // Pre-create builtin shaders
        for builtin in [
            BuiltinShader::Standard,
            BuiltinShader::Unlit,
            BuiltinShader::Billboard,
            BuiltinShader::Text,
            BuiltinShader::UI,
        ] {
            let shader = ShaderProgram::from_builtin(builtin);
            let id = shader.id;
            manager.shaders.insert(id, shader);
            manager.builtin_cache.insert(builtin, id);
        }
        
        manager
    }
    
    /// Get builtin shader
    pub fn get_builtin(&self, builtin: BuiltinShader) -> Option<&ShaderProgram> {
        self.builtin_cache.get(&builtin)
            .and_then(|id| self.shaders.get(id))
    }
    
    /// Get shader by ID
    pub fn get_shader(&self, id: ContentId) -> Option<&ShaderProgram> {
        self.shaders.get(&id)
    }
    
    /// Add custom shader
    pub fn add_shader(&mut self, shader: ShaderProgram) -> ContentId {
        let id = shader.id;
        self.shaders.insert(id, shader);
        id
    }
    
    /// Remove shader
    pub fn remove_shader(&mut self, id: ContentId) {
        self.shaders.remove(&id);
    }
    
    /// Update common uniforms
    pub fn update_common(&mut self, camera: &Camera, time: f32, delta_time: f32) {
        self.common_uniforms.update_from_camera(camera);
        self.common_uniforms.time = time;
        self.common_uniforms.delta_time = delta_time;
    }
    
    /// Get common uniforms
    pub fn common_uniforms(&self) -> &CommonUniforms {
        &self.common_uniforms
    }
    
    /// Get shader count
    pub fn shader_count(&self) -> usize {
        self.shaders.len()
    }
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Render pass type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPass {
    /// Shadow map generation
    Shadow,
    /// Depth pre-pass
    DepthPrepass,
    /// Main opaque geometry
    Opaque,
    /// Transparent geometry
    Transparent,
    /// Post-processing
    PostProcess,
    /// UI overlay
    UI,
}

impl RenderPass {
    /// Get pass order (lower = earlier)
    pub fn order(&self) -> u8 {
        match self {
            RenderPass::Shadow => 0,
            RenderPass::DepthPrepass => 1,
            RenderPass::Opaque => 2,
            RenderPass::Transparent => 3,
            RenderPass::PostProcess => 4,
            RenderPass::UI => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_program_builtin() {
        let shader = ShaderProgram::from_builtin(BuiltinShader::Standard);
        
        assert!(shader.is_builtin());
        assert_eq!(shader.builtin, Some(BuiltinShader::Standard));
    }
    
    #[test]
    fn test_shader_uniforms() {
        let mut shader = ShaderProgram::new("custom");
        
        shader.set_uniform("color", UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]));
        shader.set_uniform("intensity", UniformValue::Float(0.5));
        
        assert!(shader.get_uniform("color").is_some());
        assert!(shader.get_uniform("nonexistent").is_none());
    }
    
    #[test]
    fn test_common_uniforms() {
        let mut uniforms = CommonUniforms::new();
        let camera = Camera::new();
        
        uniforms.update_from_camera(&camera);
        
        // Camera at origin, so position should be zeros
        assert_eq!(uniforms.camera_position, [0.0, 0.0, 0.0]);
    }
    
    #[test]
    fn test_object_uniforms() {
        let mut uniforms = ObjectUniforms::new();
        let transform = Transform::at_position(1.0, 2.0, 3.0);
        
        uniforms.update_from_transform(&transform);
        
        // Translation should be in model matrix
        assert!((uniforms.model_matrix[3][0] - 1.0).abs() < 0.001);
        assert!((uniforms.model_matrix[3][1] - 2.0).abs() < 0.001);
        assert!((uniforms.model_matrix[3][2] - 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_shader_manager() {
        let manager = ShaderManager::new();
        
        assert!(manager.get_builtin(BuiltinShader::Standard).is_some());
        assert!(manager.get_builtin(BuiltinShader::UI).is_some());
        assert!(manager.shader_count() >= 5); // At least the builtins
    }
    
    #[test]
    fn test_shader_manager_custom() {
        let mut manager = ShaderManager::new();
        
        let shader = ShaderProgram::new("my_shader");
        let id = manager.add_shader(shader);
        
        assert!(manager.get_shader(id).is_some());
        
        manager.remove_shader(id);
        assert!(manager.get_shader(id).is_none());
    }
    
    #[test]
    fn test_render_pass_order() {
        assert!(RenderPass::Shadow.order() < RenderPass::Opaque.order());
        assert!(RenderPass::Opaque.order() < RenderPass::Transparent.order());
        assert!(RenderPass::PostProcess.order() < RenderPass::UI.order());
    }
    
    #[test]
    fn test_lighting_uniforms() {
        let lighting = LightingUniforms::default();
        
        assert_eq!(lighting.directional_light_intensity, 1.0);
        assert!(lighting.point_lights.is_empty());
    }
}
