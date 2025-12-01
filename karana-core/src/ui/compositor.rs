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
    pub nodes: Vec<ARNode>,
    pub background_opacity: f32,
}

impl ARScene {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            background_opacity: 0.0, // Transparent by default (AR)
        }
    }

    pub fn add_node(&mut self, node: ARNode) {
        self.nodes.push(node);
        // Sort by Z-index (painter's algorithm)
        self.nodes.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
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
        for x in 0..width {
            buffer[0][x] = '-';
            buffer[height - 1][x] = '-';
        }
        for y in 0..height {
            buffer[y][0] = '|';
            buffer[y][width - 1] = '|';
        }

        // 3. Render Nodes
        for node in &scene.nodes {
            // Map normalized coords (0.0-1.0) to screen coords
            let screen_x = (node.x * width as f32) as usize;
            let screen_y = (node.y * height as f32) as usize;
            let screen_w = (node.width * width as f32) as usize;
            let screen_h = (node.height * height as f32) as usize;

            // Draw Box
            for y in screen_y..(screen_y + screen_h).min(height - 1) {
                for x in screen_x..(screen_x + screen_w).min(width - 1) {
                    if y > 0 && x > 0 {
                        buffer[y][x] = '.'; // Background fill
                    }
                }
            }

            // Draw Content (Centered)
            let content_chars: Vec<char> = node.content.chars().collect();
            let start_x = screen_x + 1;
            let start_y = screen_y + 1;
            
            let mut cx = start_x;
            let mut cy = start_y;

            for char in content_chars {
                if cy < height - 1 && cx < width - 1 {
                    buffer[cy][cx] = char;
                    cx += 1;
                    if cx >= screen_x + screen_w - 1 {
                        cx = start_x;
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

    pub fn update_hud(&self, time: &str, battery: &str, status: &str) {
        let mut scene = self.scene.lock().unwrap();
        scene.clear();

        // Top Right: Time
        scene.add_node(ARNode {
            id: "clock".to_string(),
            content: time.to_string(),
            x: 0.8, y: 0.05, z: 1.0,
            width: 0.15, height: 0.05,
            opacity: 1.0,
        });

        // Top Left: Battery
        scene.add_node(ARNode {
            id: "battery".to_string(),
            content: battery.to_string(),
            x: 0.05, y: 0.05, z: 1.0,
            width: 0.15, height: 0.05,
            opacity: 1.0,
        });

        // Bottom Center: Status
        scene.add_node(ARNode {
            id: "status".to_string(),
            content: status.to_string(),
            x: 0.2, y: 0.85, z: 1.0,
            width: 0.6, height: 0.1,
            opacity: 0.8,
        });
    }

    pub fn add_widget(&self, id: &str, content: &str, x: f32, y: f32) {
        let mut scene = self.scene.lock().unwrap();
        scene.add_node(ARNode {
            id: id.to_string(),
            content: content.to_string(),
            x, y, z: 0.5,
            width: 0.3, height: 0.2,
            opacity: 0.9,
        });
    }

    pub fn render(&self, width: usize, height: usize) -> Result<String> {
        let scene = self.scene.lock().unwrap();
        self.renderer.render(&scene, width, height)
    }
}
