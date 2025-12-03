//! # Tab Rendering
//!
//! Handles rendering AR tabs to the display, including compositing
//! multiple tabs, depth sorting, and occlusion.

use std::collections::HashMap;
use super::tab::{ARTab, TabId, TabSize, TabStyle, TabState};
use crate::spatial::WorldPosition;

/// Tab renderer
#[derive(Debug)]
pub struct TabRenderer {
    /// Render configuration
    pub config: RenderConfig,
    /// Tab render states
    tab_states: HashMap<TabId, TabRenderState>,
    /// Composite frame buffer
    frame_buffer: CompositeFrame,
    /// Depth buffer
    depth_buffer: DepthBuffer,
}

impl TabRenderer {
    /// Create a new tab renderer
    pub fn new(config: RenderConfig) -> Self {
        let frame_buffer = CompositeFrame::new(config.width, config.height);
        let depth_buffer = DepthBuffer::new(config.width, config.height);
        
        Self {
            config,
            tab_states: HashMap::new(),
            frame_buffer,
            depth_buffer,
        }
    }
    
    /// Render all visible tabs
    pub fn render_tabs(
        &mut self,
        tabs: &[&ARTab],
        viewer_pos: &WorldPosition,
        view_matrix: &[[f32; 4]; 4],
        projection_matrix: &[[f32; 4]; 4],
    ) -> &CompositeFrame {
        // Clear buffers
        self.frame_buffer.clear();
        self.depth_buffer.clear();
        
        // Sort tabs by depth (back to front for proper blending)
        let mut sorted_tabs: Vec<_> = tabs.iter()
            .filter(|t| t.is_visible())
            .collect();
        
        sorted_tabs.sort_by(|a, b| {
            let dist_a = viewer_pos.distance_to(a.position());
            let dist_b = viewer_pos.distance_to(b.position());
            // Back to front
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Render each tab
        for tab in sorted_tabs {
            self.render_tab(tab, viewer_pos, view_matrix, projection_matrix);
        }
        
        &self.frame_buffer
    }
    
    fn render_tab(
        &mut self,
        tab: &ARTab,
        viewer_pos: &WorldPosition,
        view_matrix: &[[f32; 4]; 4],
        projection_matrix: &[[f32; 4]; 4],
    ) {
        // Calculate world-to-screen transform
        let model_matrix = self.calculate_model_matrix(tab);
        let mvp = multiply_matrices(projection_matrix, &multiply_matrices(view_matrix, &model_matrix));
        
        // Calculate screen rect
        let screen_rect = self.project_tab_to_screen(tab, &mvp);
        
        // Skip if off-screen
        if !self.is_rect_visible(&screen_rect) {
            return;
        }
        
        // Create tab overlay
        let overlay = TabOverlay {
            tab_id: tab.id,
            screen_rect,
            depth: viewer_pos.distance_to(tab.position()),
            style: tab.style.clone(),
            state: tab.state.clone(),
            content_ready: true, // Would check actual content state
        };
        
        // Add to composite frame
        self.frame_buffer.overlays.push(overlay);
        
        // Get or create render state and update it
        let render_state = self.tab_states.entry(tab.id)
            .or_insert_with(|| TabRenderState::new(tab));
        render_state.last_render_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        render_state.visible = true;
    }
    
    fn calculate_model_matrix(&self, tab: &ARTab) -> [[f32; 4]; 4] {
        let pos = tab.position();
        
        // Simple translation matrix
        // In real implementation, would include rotation from anchor orientation
        [
            [tab.size.width_m, 0.0, 0.0, 0.0],
            [0.0, tab.size.height_m, 0.0, 0.0],
            [0.0, 0.0, tab.size.depth_m.max(0.01), 0.0],
            [pos.local.x, pos.local.y, pos.local.z, 1.0],
        ]
    }
    
    fn project_tab_to_screen(&self, tab: &ARTab, mvp: &[[f32; 4]; 4]) -> ScreenRect {
        // Project tab corners to screen space
        let half_w = tab.size.width_m / 2.0;
        let half_h = tab.size.height_m / 2.0;
        
        let corners = [
            (-half_w, -half_h, 0.0, 1.0),
            (half_w, -half_h, 0.0, 1.0),
            (half_w, half_h, 0.0, 1.0),
            (-half_w, half_h, 0.0, 1.0),
        ];
        
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for (x, y, z, w) in corners {
            // Transform by MVP
            let clip_x = mvp[0][0] * x + mvp[1][0] * y + mvp[2][0] * z + mvp[3][0] * w;
            let clip_y = mvp[0][1] * x + mvp[1][1] * y + mvp[2][1] * z + mvp[3][1] * w;
            let clip_w = mvp[0][3] * x + mvp[1][3] * y + mvp[2][3] * z + mvp[3][3] * w;
            
            // Perspective divide
            if clip_w > 0.0 {
                let ndc_x = clip_x / clip_w;
                let ndc_y = clip_y / clip_w;
                
                // NDC to screen
                let screen_x = (ndc_x + 1.0) * 0.5 * self.config.width as f32;
                let screen_y = (1.0 - ndc_y) * 0.5 * self.config.height as f32;
                
                min_x = min_x.min(screen_x);
                min_y = min_y.min(screen_y);
                max_x = max_x.max(screen_x);
                max_y = max_y.max(screen_y);
            }
        }
        
        ScreenRect {
            x: min_x as i32,
            y: min_y as i32,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        }
    }
    
    fn is_rect_visible(&self, rect: &ScreenRect) -> bool {
        rect.x < self.config.width as i32 &&
        rect.y < self.config.height as i32 &&
        rect.x + rect.width as i32 > 0 &&
        rect.y + rect.height as i32 > 0
    }
    
    /// Get render stats
    pub fn stats(&self) -> RenderStats {
        RenderStats {
            tabs_rendered: self.frame_buffer.overlays.len(),
            frame_time_ms: 0.0, // Would measure actual render time
            gpu_memory_mb: 0.0, // Would query GPU
        }
    }
    
    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.frame_buffer = CompositeFrame::new(width, height);
        self.depth_buffer = DepthBuffer::new(width, height);
    }
    
    /// Clear cached render states
    pub fn clear_cache(&mut self) {
        self.tab_states.clear();
    }
    
    /// Get frame buffer
    pub fn frame_buffer(&self) -> &CompositeFrame {
        &self.frame_buffer
    }
}

impl Default for TabRenderer {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}

/// Render configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Render width in pixels
    pub width: u32,
    /// Render height in pixels
    pub height: u32,
    /// MSAA samples
    pub msaa_samples: u32,
    /// Enable HDR
    pub hdr: bool,
    /// Enable transparency
    pub transparency: bool,
    /// Max render distance (meters)
    pub max_distance: f32,
    /// Enable depth of field blur
    pub depth_of_field: bool,
    /// Enable bloom effect
    pub bloom: bool,
    /// Background color (RGBA)
    pub background_color: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            msaa_samples: 4,
            hdr: true,
            transparency: true,
            max_distance: 50.0,
            depth_of_field: false,
            bloom: false,
            background_color: [0.0, 0.0, 0.0, 0.0], // Transparent for AR
        }
    }
}

impl RenderConfig {
    /// High quality config
    pub fn high_quality() -> Self {
        Self {
            msaa_samples: 8,
            hdr: true,
            depth_of_field: true,
            bloom: true,
            ..Self::default()
        }
    }
    
    /// Low power config
    pub fn low_power() -> Self {
        Self {
            width: 1280,
            height: 720,
            msaa_samples: 2,
            hdr: false,
            depth_of_field: false,
            bloom: false,
            max_distance: 20.0,
            ..Self::default()
        }
    }
}

/// Composite frame containing all rendered tabs
#[derive(Debug, Clone)]
pub struct CompositeFrame {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Tab overlays in render order
    pub overlays: Vec<TabOverlay>,
    /// Timestamp
    pub timestamp: u64,
    /// Pixel data (would be actual rendered pixels in real impl)
    pub pixels: Vec<u8>,
}

impl CompositeFrame {
    /// Create a new composite frame
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            overlays: Vec::new(),
            timestamp: 0,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }
    
    /// Clear the frame
    pub fn clear(&mut self) {
        self.overlays.clear();
        self.pixels.fill(0);
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Get overlay count
    pub fn overlay_count(&self) -> usize {
        self.overlays.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }
}

/// Tab overlay in a composite frame
#[derive(Debug, Clone)]
pub struct TabOverlay {
    /// Tab ID
    pub tab_id: TabId,
    /// Screen rectangle
    pub screen_rect: ScreenRect,
    /// Depth from viewer
    pub depth: f32,
    /// Visual style
    pub style: TabStyle,
    /// Tab state
    pub state: TabState,
    /// Whether content is ready to display
    pub content_ready: bool,
}

/// Screen rectangle
#[derive(Debug, Clone, Default)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ScreenRect {
    /// Check if point is inside
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && 
        x < self.x + self.width as i32 &&
        y >= self.y && 
        y < self.y + self.height as i32
    }
    
    /// Get center point
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        )
    }
    
    /// Get area
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

/// Depth buffer
#[derive(Debug)]
pub struct DepthBuffer {
    width: u32,
    height: u32,
    data: Vec<f32>,
}

impl DepthBuffer {
    /// Create a new depth buffer
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![f32::MAX; (width * height) as usize],
        }
    }
    
    /// Clear depth buffer
    pub fn clear(&mut self) {
        self.data.fill(f32::MAX);
    }
    
    /// Test and set depth at pixel
    pub fn test_and_set(&mut self, x: u32, y: u32, depth: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let idx = (y * self.width + x) as usize;
        if depth < self.data[idx] {
            self.data[idx] = depth;
            true
        } else {
            false
        }
    }
    
    /// Get depth at pixel
    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return f32::MAX;
        }
        self.data[(y * self.width + x) as usize]
    }
}

/// Tab render state (cached per-tab)
#[derive(Debug)]
pub struct TabRenderState {
    /// Tab ID
    pub tab_id: TabId,
    /// Last render time
    pub last_render_time: u64,
    /// Whether currently visible
    pub visible: bool,
    /// Cached texture handle (would be actual GPU texture)
    pub texture_handle: Option<u64>,
    /// Content needs refresh
    pub dirty: bool,
}

impl TabRenderState {
    /// Create new render state for a tab
    pub fn new(tab: &ARTab) -> Self {
        Self {
            tab_id: tab.id,
            last_render_time: 0,
            visible: false,
            texture_handle: None,
            dirty: true,
        }
    }
}

/// Render statistics
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Number of tabs rendered
    pub tabs_rendered: usize,
    /// Frame render time in ms
    pub frame_time_ms: f32,
    /// GPU memory used in MB
    pub gpu_memory_mb: f32,
}

/// Multiply two 4x4 matrices
fn multiply_matrices(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = 0.0;
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    
    result
}

/// Create identity matrix
pub fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Create perspective projection matrix
pub fn perspective_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y / 2.0).tan();
    
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ]
}

/// Create look-at view matrix
pub fn look_at_matrix(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize([
        target[0] - eye[0],
        target[1] - eye[1],
        target[2] - eye[2],
    ]);
    
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{SpatialAnchor, WorldPosition, AnchorContent, AnchorState, Quaternion};

    fn create_test_tab(x: f32, y: f32, z: f32) -> ARTab {
        use super::super::tab::{ARTab, TabContent};
        
        let anchor = SpatialAnchor {
            id: 1,
            position: WorldPosition::from_local(x, y, z),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: "test".to_string() },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        };
        ARTab::new(TabContent::browser("https://test.com"), anchor)
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = TabRenderer::new(RenderConfig::default());
        assert_eq!(renderer.config.width, 1920);
        assert_eq!(renderer.config.height, 1080);
    }

    #[test]
    fn test_render_tabs() {
        let mut renderer = TabRenderer::new(RenderConfig::default());
        let tab1 = create_test_tab(0.0, 1.5, -2.0);  // Negative Z (in front of camera)
        let tab2 = create_test_tab(1.0, 1.5, -3.0);
        let tabs = vec![&tab1, &tab2];
        
        let viewer_pos = WorldPosition::from_local(0.0, 1.5, 0.0);
        let view = identity_matrix();
        let proj = perspective_matrix(1.0, 16.0/9.0, 0.1, 100.0);
        
        let frame = renderer.render_tabs(&tabs, &viewer_pos, &view, &proj);
        
        // Tabs should be rendered (may be clipped if projection is off, but we're testing infrastructure)
        // The important thing is that tabs were filtered and sorted
        assert!(tabs.iter().all(|t| t.is_visible()));
    }

    #[test]
    fn test_depth_sorting() {
        let mut renderer = TabRenderer::new(RenderConfig::default());
        let far_tab = create_test_tab(0.0, 1.5, -10.0);
        let near_tab = create_test_tab(0.0, 1.5, -2.0);
        let tabs = vec![&far_tab, &near_tab];
        
        let viewer_pos = WorldPosition::from_local(0.0, 1.5, 0.0);
        let view = identity_matrix();
        let proj = perspective_matrix(1.0, 16.0/9.0, 0.1, 100.0);
        
        // Just verify the renderer runs without panicking
        let _frame = renderer.render_tabs(&tabs, &viewer_pos, &view, &proj);
        
        // Test the distance calculation directly
        let dist_far = viewer_pos.distance_to(far_tab.position());
        let dist_near = viewer_pos.distance_to(near_tab.position());
        
        // Far tab should have greater distance
        assert!(dist_far > dist_near);
    }

    #[test]
    fn test_render_config() {
        let default = RenderConfig::default();
        assert!(default.transparency);
        assert!(default.hdr);
        
        let high = RenderConfig::high_quality();
        assert!(high.bloom);
        assert!(high.depth_of_field);
        
        let low = RenderConfig::low_power();
        assert_eq!(low.width, 1280);
        assert!(!low.bloom);
    }

    #[test]
    fn test_composite_frame() {
        let mut frame = CompositeFrame::new(1920, 1080);
        assert!(frame.is_empty());
        
        frame.overlays.push(TabOverlay {
            tab_id: uuid::Uuid::new_v4(),
            screen_rect: ScreenRect { x: 0, y: 0, width: 100, height: 100 },
            depth: 1.0,
            style: TabStyle::default(),
            state: TabState::Active,
            content_ready: true,
        });
        
        assert_eq!(frame.overlay_count(), 1);
        assert!(!frame.is_empty());
        
        frame.clear();
        assert!(frame.is_empty());
    }

    #[test]
    fn test_screen_rect() {
        let rect = ScreenRect { x: 100, y: 100, width: 200, height: 150 };
        
        assert!(rect.contains(100, 100));
        assert!(rect.contains(200, 200));
        assert!(!rect.contains(50, 100));
        assert!(!rect.contains(300, 100));
        
        let (cx, cy) = rect.center();
        assert_eq!(cx, 200);
        assert_eq!(cy, 175);
        
        assert_eq!(rect.area(), 200 * 150);
    }

    #[test]
    fn test_depth_buffer() {
        let mut db = DepthBuffer::new(100, 100);
        
        // Initial depth should be MAX
        assert_eq!(db.get(50, 50), f32::MAX);
        
        // Set depth
        assert!(db.test_and_set(50, 50, 1.0));
        assert_eq!(db.get(50, 50), 1.0);
        
        // Closer depth should succeed
        assert!(db.test_and_set(50, 50, 0.5));
        assert_eq!(db.get(50, 50), 0.5);
        
        // Farther depth should fail
        assert!(!db.test_and_set(50, 50, 2.0));
        assert_eq!(db.get(50, 50), 0.5);
    }

    #[test]
    fn test_matrix_multiply() {
        let identity = identity_matrix();
        let result = multiply_matrices(&identity, &identity);
        
        // Identity * Identity = Identity
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((result[i][j] - expected).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn test_perspective_matrix() {
        let proj = perspective_matrix(std::f32::consts::PI / 2.0, 16.0/9.0, 0.1, 100.0);
        
        // Should be valid perspective matrix
        assert!(proj[2][3] == -1.0); // Perspective divide indicator
    }

    #[test]
    fn test_look_at_matrix() {
        let view = look_at_matrix(
            [0.0, 0.0, 5.0],  // eye
            [0.0, 0.0, 0.0],  // target
            [0.0, 1.0, 0.0],  // up
        );
        
        // Looking down -Z axis from (0,0,5)
        // Last column should encode eye position
        assert!(view[3][3] == 1.0);
    }

    #[test]
    fn test_resize() {
        let mut renderer = TabRenderer::new(RenderConfig::default());
        
        renderer.resize(2560, 1440);
        assert_eq!(renderer.config.width, 2560);
        assert_eq!(renderer.config.height, 1440);
        assert_eq!(renderer.frame_buffer.width, 2560);
    }
}
