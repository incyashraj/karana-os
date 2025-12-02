use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Represents a 2D/3D object in the AR world
#[derive(Clone, Debug)]
pub struct ARNode {
    pub id: String,
    pub content: String, // Text content or texture ID
    pub x: f32,
    pub y: f32,
    pub z: f32, // Depth for layering
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
}

/// The Scene Graph containing all active AR elements
#[derive(Default)]
pub struct ARScene {
    pub hud_nodes: Vec<ARNode>,
    pub app_nodes: Vec<ARNode>,
    pub background_opacity: f32,
}

impl ARScene {
    pub fn new() -> Self {
        Self {
            hud_nodes: Vec::new(),
            app_nodes: Vec::new(),
            background_opacity: 0.0, // Transparent by default (AR)
        }
    }

    pub fn get_all_nodes(&self) -> Vec<&ARNode> {
        let mut all: Vec<&ARNode> = self.hud_nodes.iter().chain(self.app_nodes.iter()).collect();
        // Sort by Z-index (painter's algorithm)
        all.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
        all
    }

    pub fn clear_hud(&mut self) {
        self.hud_nodes.clear();
    }
    
    pub fn clear_apps(&mut self) {
        self.app_nodes.clear();
    }
}

/// Trait for rendering the AR Scene to a display backend
pub trait ARRenderer {
    fn render(&self, scene: &ARScene, width: usize, height: usize) -> Result<String>;
}

/// Renders the AR Scene to an ASCII grid for TUI display
pub struct AsciiRenderer;

impl ARRenderer for AsciiRenderer {
    fn render(&self, scene: &ARScene, width: usize, height: usize) -> Result<String> {
        // 1. Create empty buffer
        let mut buffer = vec![vec![' '; width]; height];

        // 2. Draw Border (Glass Frame)
        // Top/Bottom
        for x in 0..width {
            buffer[0][x] = '═';
            buffer[height - 1][x] = '═';
        }
        // Sides
        for y in 0..height {
            buffer[y][0] = '║';
            buffer[y][width - 1] = '║';
        }
        // Corners
        buffer[0][0] = '╔';
        buffer[0][width - 1] = '╗';
        buffer[height - 1][0] = '╚';
        buffer[height - 1][width - 1] = '╝';

        // 3. Render Nodes
        for node in scene.get_all_nodes() {
            // Map normalized coords (0.0-1.0) to screen coords
            let screen_x = (node.x * width as f32) as usize;
            let screen_y = (node.y * height as f32) as usize;
            let screen_w = (node.width * width as f32).max(4.0) as usize;
            let screen_h = (node.height * height as f32).max(3.0) as usize;

            // Clamp to bounds
            let end_x = (screen_x + screen_w).min(width - 2);
            let end_y = (screen_y + screen_h).min(height - 2);
            let start_x = screen_x.max(1);
            let start_y = screen_y.max(1);

            if start_x >= end_x || start_y >= end_y { continue; }

            // Draw Box with Border
            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    let ch = if y == start_y && x == start_x { '┌' }
                    else if y == start_y && x == end_x { '┐' }
                    else if y == end_y && x == start_x { '└' }
                    else if y == end_y && x == end_x { '┘' }
                    else if y == start_y || y == end_y { '─' }
                    else if x == start_x || x == end_x { '│' }
                    else { ' ' }; // Clear content inside
                    
                    buffer[y][x] = ch;
                }
            }

            // Draw Content (Wrapped)
            let _content_width = end_x - start_x - 1;
            let content_chars: Vec<char> = node.content.chars().collect();
            
            let mut cx = start_x + 1;
            let mut cy = start_y + 1;

            for char in content_chars {
                if cy < end_y && cx < end_x {
                    buffer[cy][cx] = char;
                    cx += 1;
                    if cx >= end_x {
                        cx = start_x + 1;
                        cy += 1;
                    }
                }
            }
        }

        // 4. Serialize to String
        let mut output = String::new();
        for row in buffer {
            output.push_str(&row.into_iter().collect::<String>());
            output.push('\n');
        }

        Ok(output)
    }
}

/// The Compositor manages the Scene and the active Renderer
pub struct ARCompositor {
    pub scene: Arc<Mutex<ARScene>>,
    renderer: Box<dyn ARRenderer + Send + Sync>,
}

impl ARCompositor {
    pub fn new() -> Self {
        Self {
            scene: Arc::new(Mutex::new(ARScene::new())),
            renderer: Box::new(AsciiRenderer),
        }
    }

    pub fn update_hud(&self, time: &str, battery: &str, status: &str, gaze: (f32, f32)) {
        let mut scene = self.scene.lock().unwrap();
        scene.clear_hud();

        // Top Right: Time
        scene.hud_nodes.push(ARNode {
            id: "clock".to_string(),
            content: time.to_string(),
            x: 0.75, y: 0.05, z: 1.0,
            width: 0.2, height: 0.1,
            opacity: 1.0,
        });

        // Top Left: Battery
        scene.hud_nodes.push(ARNode {
            id: "battery".to_string(),
            content: battery.to_string(),
            x: 0.05, y: 0.05, z: 1.0,
            width: 0.2, height: 0.1,
            opacity: 1.0,
        });

        // Bottom Center: Status
        scene.hud_nodes.push(ARNode {
            id: "status".to_string(),
            content: status.to_string(),
            x: 0.2, y: 0.85, z: 1.0,
            width: 0.6, height: 0.1,
            opacity: 0.8,
        });

        // Gaze Cursor
        scene.hud_nodes.push(ARNode {
            id: "cursor".to_string(),
            content: "(+)".to_string(),
            x: gaze.0, y: gaze.1, z: 2.0,
            width: 0.05, height: 0.05,
            opacity: 1.0,
        });
    }

    pub fn add_widget(&self, id: &str, content: &str, x: f32, y: f32) {
        let mut scene = self.scene.lock().unwrap();
        // Remove existing if same ID
        scene.app_nodes.retain(|n| n.id != id);
        
        scene.app_nodes.push(ARNode {
            id: id.to_string(),
            content: content.to_string(),
            x, y, z: 0.5,
            width: 0.3, height: 0.2,
            opacity: 0.9,
        });
    }

    pub fn add_widget_sized(&self, id: &str, content: &str, x: f32, y: f32, w: f32, h: f32) {
        let mut scene = self.scene.lock().unwrap();
        // Remove existing if same ID
        scene.app_nodes.retain(|n| n.id != id);
        
        scene.app_nodes.push(ARNode {
            id: id.to_string(),
            content: content.to_string(),
            x, y, z: 0.5,
            width: w, height: h,
            opacity: 0.9,
        });
    }

    pub fn render(&self, width: usize, height: usize) -> Result<String> {
        let scene = self.scene.lock().unwrap();
        self.renderer.render(&scene, width, height)
    }
}
