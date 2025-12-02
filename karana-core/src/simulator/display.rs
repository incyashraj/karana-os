//! Virtual Display System
//!
//! Simulates the AR display with layers, transparency, and HUD elements.

use std::collections::HashMap;

/// A single AR element that can be displayed
#[derive(Debug, Clone)]
pub struct ARElement {
    pub id: String,
    pub element_type: ARElementType,
    pub position: Position,
    pub size: Size,
    pub content: String,
    pub opacity: f32,
    pub z_index: i32,
    pub visible: bool,
    pub animation: Option<Animation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ARElementType {
    Text,
    Icon,
    Panel,
    Notification,
    Timer,
    Navigation,
    ObjectLabel,
    StatusBar,
    VoiceIndicator,
    ContextMenu,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// X position (0.0 = left edge, 1.0 = right edge)
    pub x: f32,
    /// Y position (0.0 = top, 1.0 = bottom)
    pub y: f32,
    /// Anchor point
    pub anchor: Anchor,
}

#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,  // Relative to display width
    pub height: f32, // Relative to display height
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub animation_type: AnimationType,
    pub duration_ms: u32,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    FadeIn,
    FadeOut,
    SlideIn { from: Direction },
    SlideOut { to: Direction },
    Pulse,
    Bounce,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

/// A display layer containing multiple elements
#[derive(Debug, Clone)]
pub struct DisplayLayer {
    pub name: String,
    pub elements: HashMap<String, ARElement>,
    pub visible: bool,
    pub opacity: f32,
}

impl DisplayLayer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            elements: HashMap::new(),
            visible: true,
            opacity: 1.0,
        }
    }

    pub fn add_element(&mut self, element: ARElement) {
        self.elements.insert(element.id.clone(), element);
    }

    pub fn remove_element(&mut self, id: &str) -> Option<ARElement> {
        self.elements.remove(id)
    }

    pub fn get_element_mut(&mut self, id: &str) -> Option<&mut ARElement> {
        self.elements.get_mut(id)
    }
}

/// The virtual AR display
pub struct VirtualDisplay {
    pub width: u32,
    pub height: u32,
    pub is_on: bool,
    pub brightness: f32,
    pub layers: Vec<DisplayLayer>,
    /// Current frame buffer (for rendering to terminal)
    frame_buffer: Vec<Vec<char>>,
}

impl VirtualDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        let mut display = Self {
            width,
            height,
            is_on: false,
            brightness: 0.8,
            layers: Vec::new(),
            frame_buffer: vec![vec![' '; width as usize / 10]; height as usize / 20],
        };

        // Create default layers
        display.layers.push(DisplayLayer::new("background"));
        display.layers.push(DisplayLayer::new("world"));  // World-locked AR
        display.layers.push(DisplayLayer::new("hud"));    // Head-locked HUD
        display.layers.push(DisplayLayer::new("notifications"));
        display.layers.push(DisplayLayer::new("system"));

        display
    }

    /// Turn on the display
    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    /// Turn off the display
    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    /// Set brightness (0.0 - 1.0)
    pub fn dim(&mut self, level: f32) {
        self.brightness = level.clamp(0.0, 1.0);
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        for layer in &mut self.layers {
            layer.elements.clear();
        }
    }

    /// Show boot screen
    pub fn show_boot_screen(&mut self) {
        self.is_on = true;
        
        let boot_text = ARElement {
            id: "boot_logo".to_string(),
            element_type: ARElementType::Text,
            position: Position { x: 0.5, y: 0.5, anchor: Anchor::Center },
            size: Size { width: 0.5, height: 0.1 },
            content: "KĀRAṆA OS".to_string(),
            opacity: 1.0,
            z_index: 100,
            visible: true,
            animation: Some(Animation {
                animation_type: AnimationType::FadeIn,
                duration_ms: 1000,
                progress: 0.0,
            }),
        };

        if let Some(layer) = self.get_layer_mut("system") {
            layer.add_element(boot_text);
        }
    }

    /// Get a layer by name
    pub fn get_layer(&self, name: &str) -> Option<&DisplayLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Get a mutable layer by name
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut DisplayLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Add a text notification
    pub fn show_notification(&mut self, id: &str, text: &str, duration_ms: u32) {
        let notification = ARElement {
            id: id.to_string(),
            element_type: ARElementType::Notification,
            position: Position { x: 0.5, y: 0.1, anchor: Anchor::TopCenter },
            size: Size { width: 0.6, height: 0.08 },
            content: text.to_string(),
            opacity: 0.9,
            z_index: 90,
            visible: true,
            animation: Some(Animation {
                animation_type: AnimationType::SlideIn { from: Direction::Top },
                duration_ms,
                progress: 0.0,
            }),
        };

        if let Some(layer) = self.get_layer_mut("notifications") {
            layer.add_element(notification);
        }
    }

    /// Show an object label in the world
    pub fn show_object_label(&mut self, id: &str, label: &str, x: f32, y: f32) {
        let object_label = ARElement {
            id: id.to_string(),
            element_type: ARElementType::ObjectLabel,
            position: Position { x, y, anchor: Anchor::BottomCenter },
            size: Size { width: 0.2, height: 0.05 },
            content: label.to_string(),
            opacity: 0.85,
            z_index: 50,
            visible: true,
            animation: None,
        };

        if let Some(layer) = self.get_layer_mut("world") {
            layer.add_element(object_label);
        }
    }

    /// Show a timer
    pub fn show_timer(&mut self, label: &str, remaining: &str) {
        let timer = ARElement {
            id: "active_timer".to_string(),
            element_type: ARElementType::Timer,
            position: Position { x: 0.95, y: 0.05, anchor: Anchor::TopRight },
            size: Size { width: 0.15, height: 0.08 },
            content: format!("{}: {}", label, remaining),
            opacity: 0.9,
            z_index: 80,
            visible: true,
            animation: None,
        };

        if let Some(layer) = self.get_layer_mut("hud") {
            layer.add_element(timer);
        }
    }

    /// Update timer display
    pub fn update_timer(&mut self, remaining: &str) {
        if let Some(layer) = self.get_layer_mut("hud") {
            if let Some(timer) = layer.get_element_mut("active_timer") {
                // Update just the time portion
                if let Some(colon_pos) = timer.content.find(':') {
                    timer.content = format!("{}: {}", &timer.content[..colon_pos], remaining);
                }
            }
        }
    }

    /// Show voice listening indicator
    pub fn show_voice_indicator(&mut self, listening: bool) {
        let indicator = ARElement {
            id: "voice_indicator".to_string(),
            element_type: ARElementType::VoiceIndicator,
            position: Position { x: 0.5, y: 0.9, anchor: Anchor::BottomCenter },
            size: Size { width: 0.1, height: 0.05 },
            content: if listening { "🎤 Listening..." } else { "" }.to_string(),
            opacity: 0.9,
            z_index: 95,
            visible: listening,
            animation: if listening {
                Some(Animation {
                    animation_type: AnimationType::Pulse,
                    duration_ms: 500,
                    progress: 0.0,
                })
            } else {
                None
            },
        };

        if let Some(layer) = self.get_layer_mut("hud") {
            layer.add_element(indicator);
        }
    }

    /// Show navigation arrow
    pub fn show_navigation(&mut self, direction: &str, distance: &str) {
        let nav = ARElement {
            id: "navigation".to_string(),
            element_type: ARElementType::Navigation,
            position: Position { x: 0.5, y: 0.3, anchor: Anchor::Center },
            size: Size { width: 0.3, height: 0.1 },
            content: format!("{} {}", direction, distance),
            opacity: 0.85,
            z_index: 70,
            visible: true,
            animation: None,
        };

        if let Some(layer) = self.get_layer_mut("world") {
            layer.add_element(nav);
        }
    }

    /// Show status bar
    pub fn show_status_bar(&mut self, time: &str, battery: &str, connectivity: &str) {
        let status = ARElement {
            id: "status_bar".to_string(),
            element_type: ARElementType::StatusBar,
            position: Position { x: 0.5, y: 0.02, anchor: Anchor::TopCenter },
            size: Size { width: 0.9, height: 0.04 },
            content: format!("{}  |  {} |  {}", time, battery, connectivity),
            opacity: 0.7,
            z_index: 100,
            visible: true,
            animation: None,
        };

        if let Some(layer) = self.get_layer_mut("system") {
            layer.add_element(status);
        }
    }

    /// Render the display to a text representation
    pub fn render_to_text(&self, width: usize, height: usize) -> Vec<String> {
        if !self.is_on {
            return vec!["[Display Off]".to_string()];
        }

        let mut output = vec![String::new(); height];
        
        // Create a simple frame
        let border_h = "─".repeat(width - 2);
        output[0] = format!("┌{}┐", border_h);
        output[height - 1] = format!("└{}┘", border_h);
        
        for i in 1..height - 1 {
            output[i] = format!("│{}│", " ".repeat(width - 2));
        }

        // Render elements from each visible layer
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }

            for element in layer.elements.values() {
                if !element.visible {
                    continue;
                }

                // Calculate position in text grid
                let x = ((element.position.x * (width - 4) as f32) as usize).min(width - 4);
                let y = ((element.position.y * (height - 2) as f32) as usize + 1).min(height - 2);

                // Render based on element type
                let rendered = match element.element_type {
                    ARElementType::Notification => format!("🔔 {}", element.content),
                    ARElementType::Timer => format!("⏱️  {}", element.content),
                    ARElementType::ObjectLabel => format!("📍 {}", element.content),
                    ARElementType::VoiceIndicator => format!("{}", element.content),
                    ARElementType::Navigation => format!("🧭 {}", element.content),
                    ARElementType::StatusBar => element.content.clone(),
                    _ => element.content.clone(),
                };

                // Apply brightness
                let dimmed = if self.brightness < 0.5 {
                    format!("\x1b[2m{}\x1b[0m", rendered) // Dim text
                } else {
                    rendered
                };

                // Insert into output
                if y < height - 1 && x + 2 < width {
                    let line = &mut output[y];
                    let content_start = x + 1;
                    let available = width - content_start - 2;
                    let truncated: String = dimmed.chars().take(available).collect();
                    
                    // Replace characters in the line
                    let mut chars: Vec<char> = line.chars().collect();
                    for (i, c) in truncated.chars().enumerate() {
                        if content_start + i < chars.len() {
                            chars[content_start + i] = c;
                        }
                    }
                    *line = chars.into_iter().collect();
                }
            }
        }

        output
    }

    /// Get all visible elements sorted by z-index
    pub fn get_visible_elements(&self) -> Vec<&ARElement> {
        let mut elements: Vec<&ARElement> = self.layers
            .iter()
            .filter(|l| l.visible)
            .flat_map(|l| l.elements.values())
            .filter(|e| e.visible)
            .collect();
        
        elements.sort_by(|a, b| a.z_index.cmp(&b.z_index));
        elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_creation() {
        let display = VirtualDisplay::new(1920, 1080);
        assert_eq!(display.width, 1920);
        assert_eq!(display.layers.len(), 5);
    }

    #[test]
    fn test_notification() {
        let mut display = VirtualDisplay::new(1920, 1080);
        display.turn_on();
        display.show_notification("test", "Hello World", 3000);
        
        let layer = display.get_layer("notifications").unwrap();
        assert!(layer.elements.contains_key("test"));
    }

    #[test]
    fn test_render() {
        let mut display = VirtualDisplay::new(1920, 1080);
        display.turn_on();
        display.show_notification("test", "Test", 1000);
        
        let rendered = display.render_to_text(80, 24);
        assert!(!rendered.is_empty());
    }
}
