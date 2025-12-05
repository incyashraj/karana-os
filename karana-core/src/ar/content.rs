//! AR Content Management for Kāraṇa OS
//! 
//! Manages AR content rendering and interaction.

use super::*;
use nalgebra::{Point3, UnitQuaternion, Vector3};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// AR content type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// 3D model
    Model3D,
    /// 2D image/billboard
    Billboard,
    /// Text label
    Text,
    /// UI panel
    Panel,
    /// Video
    Video,
    /// Particle system
    Particles,
    /// Custom content
    Custom,
}

/// Content visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentVisibility {
    /// Always visible
    Visible,
    /// Hidden
    Hidden,
    /// Visible only when in range
    DistanceBased,
    /// Visible only when looking at it
    GazeBased,
}

/// AR content item
#[derive(Debug, Clone)]
pub struct ArContent {
    /// Unique ID
    pub id: u64,
    /// Content type
    pub content_type: ContentType,
    /// Transform
    pub transform: Transform,
    /// Content name
    pub name: String,
    /// Visibility mode
    pub visibility: ContentVisibility,
    /// Is visible
    pub visible: bool,
    /// Is interactive
    pub interactive: bool,
    /// Associated anchor ID
    pub anchor_id: Option<u64>,
    /// Creation time
    pub created: Instant,
    /// Last update time
    pub updated: Instant,
    /// Custom data
    pub data: HashMap<String, String>,
    /// Bounding box (local space)
    pub bounds: Option<BoundingBox>,
}

impl ArContent {
    /// Create new AR content
    pub fn new(id: u64, content_type: ContentType, name: String) -> Self {
        Self {
            id,
            content_type,
            transform: Transform::new(),
            name,
            visibility: ContentVisibility::Visible,
            visible: true,
            interactive: true,
            anchor_id: None,
            created: Instant::now(),
            updated: Instant::now(),
            data: HashMap::new(),
            bounds: None,
        }
    }

    /// Set position
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.transform.position = position;
        self.updated = Instant::now();
    }

    /// Set rotation
    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.transform.rotation = rotation;
        self.updated = Instant::now();
    }

    /// Set scale
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.transform.scale = scale;
        self.updated = Instant::now();
    }

    /// Attach to anchor
    pub fn attach_to_anchor(&mut self, anchor_id: u64) {
        self.anchor_id = Some(anchor_id);
        self.updated = Instant::now();
    }

    /// Detach from anchor
    pub fn detach_from_anchor(&mut self) {
        self.anchor_id = None;
        self.updated = Instant::now();
    }

    /// Set visibility
    pub fn set_visibility(&mut self, visibility: ContentVisibility) {
        self.visibility = visibility;
        self.updated = Instant::now();
    }

    /// Get world bounding box
    pub fn get_world_bounds(&self) -> Option<BoundingBox> {
        self.bounds.map(|local| {
            let matrix = self.transform.to_matrix();
            let corners = [
                Point3::new(local.min.x, local.min.y, local.min.z),
                Point3::new(local.max.x, local.min.y, local.min.z),
                Point3::new(local.min.x, local.max.y, local.min.z),
                Point3::new(local.max.x, local.max.y, local.min.z),
                Point3::new(local.min.x, local.min.y, local.max.z),
                Point3::new(local.max.x, local.min.y, local.max.z),
                Point3::new(local.min.x, local.max.y, local.max.z),
                Point3::new(local.max.x, local.max.y, local.max.z),
            ];

            let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

            for corner in corners {
                let world = matrix.transform_point(&corner);
                min.x = min.x.min(world.x);
                min.y = min.y.min(world.y);
                min.z = min.z.min(world.z);
                max.x = max.x.max(world.x);
                max.y = max.y.max(world.y);
                max.z = max.z.max(world.z);
            }

            BoundingBox { min, max }
        })
    }

    /// Distance to camera
    pub fn distance_to(&self, camera_pos: Point3<f32>) -> f32 {
        (self.transform.position - camera_pos).norm()
    }

    /// Set custom data
    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
        self.updated = Instant::now();
    }

    /// Get custom data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
}

/// AR content manager
#[derive(Debug)]
pub struct ContentManager {
    /// All content
    content: HashMap<u64, ArContent>,
    /// Next content ID
    next_id: u64,
    /// Visible content IDs
    visible_content: Vec<u64>,
    /// Interactive content IDs
    interactive_content: Vec<u64>,
    /// Max content count
    max_content: usize,
    /// Visibility distance
    visibility_distance: f32,
}

impl ContentManager {
    /// Create new content manager
    pub fn new() -> Self {
        Self {
            content: HashMap::new(),
            next_id: 1,
            visible_content: Vec::new(),
            interactive_content: Vec::new(),
            max_content: 1000,
            visibility_distance: 50.0,
        }
    }

    /// Create content
    pub fn create(&mut self, content_type: ContentType, name: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let content = ArContent::new(id, content_type, name.to_string());
        self.content.insert(id, content);

        id
    }

    /// Create content at position
    pub fn create_at(&mut self, content_type: ContentType, name: &str, position: Point3<f32>) -> u64 {
        let id = self.create(content_type, name);
        if let Some(content) = self.content.get_mut(&id) {
            content.set_position(position);
        }
        id
    }

    /// Get content
    pub fn get(&self, id: u64) -> Option<&ArContent> {
        self.content.get(&id)
    }

    /// Get mutable content
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ArContent> {
        self.content.get_mut(&id)
    }

    /// Remove content
    pub fn remove(&mut self, id: u64) -> Option<ArContent> {
        let content = self.content.remove(&id);
        self.visible_content.retain(|&x| x != id);
        self.interactive_content.retain(|&x| x != id);
        content
    }

    /// Update visibility based on camera position
    pub fn update_visibility(&mut self, camera_pos: Point3<f32>, camera_dir: Vector3<f32>) {
        self.visible_content.clear();
        self.interactive_content.clear();

        for (id, content) in &mut self.content {
            let distance = content.distance_to(camera_pos);

            let visible = match content.visibility {
                ContentVisibility::Visible => true,
                ContentVisibility::Hidden => false,
                ContentVisibility::DistanceBased => distance <= self.visibility_distance,
                ContentVisibility::GazeBased => {
                    let dir_to_content = (content.transform.position - camera_pos).normalize();
                    let dot = camera_dir.dot(&dir_to_content);
                    dot > 0.7 && distance <= self.visibility_distance
                }
            };

            content.visible = visible;

            if visible {
                self.visible_content.push(*id);
                if content.interactive {
                    self.interactive_content.push(*id);
                }
            }
        }
    }

    /// Get all visible content
    pub fn get_visible(&self) -> Vec<&ArContent> {
        self.visible_content.iter()
            .filter_map(|id| self.content.get(id))
            .collect()
    }

    /// Get all interactive content
    pub fn get_interactive(&self) -> Vec<&ArContent> {
        self.interactive_content.iter()
            .filter_map(|id| self.content.get(id))
            .collect()
    }

    /// Get content by type
    pub fn get_by_type(&self, content_type: ContentType) -> Vec<&ArContent> {
        self.content.values()
            .filter(|c| c.content_type == content_type)
            .collect()
    }

    /// Get content at anchor
    pub fn get_at_anchor(&self, anchor_id: u64) -> Vec<&ArContent> {
        self.content.values()
            .filter(|c| c.anchor_id == Some(anchor_id))
            .collect()
    }

    /// Hit test content with ray
    pub fn hit_test(&self, origin: Point3<f32>, direction: Vector3<f32>) -> Option<(u64, Point3<f32>)> {
        let direction = direction.normalize();
        let mut closest: Option<(u64, Point3<f32>, f32)> = None;

        for content in self.get_interactive() {
            if let Some(bounds) = content.get_world_bounds() {
                if let Some(t) = ray_box_intersect(origin, direction, bounds.min, bounds.max) {
                    if closest.is_none() || t < closest.as_ref().unwrap().2 {
                        let hit_point = Point3::from(origin.coords + direction * t);
                        closest = Some((content.id, hit_point, t));
                    }
                }
            }
        }

        closest.map(|(id, point, _)| (id, point))
    }

    /// Get content count
    pub fn count(&self) -> usize {
        self.content.len()
    }

    /// Clear all content
    pub fn clear(&mut self) {
        self.content.clear();
        self.visible_content.clear();
        self.interactive_content.clear();
    }

    /// Set visibility distance
    pub fn set_visibility_distance(&mut self, distance: f32) {
        self.visibility_distance = distance;
    }
}

impl Default for ContentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Ray-box intersection test
fn ray_box_intersect(origin: Point3<f32>, direction: Vector3<f32>, min: Point3<f32>, max: Point3<f32>) -> Option<f32> {
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
                return None;
            }
        } else {
            let t1 = (bmin - orig) / dir;
            let t2 = (bmax - orig) / dir;
            let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
        }
    }

    if tmax >= tmin && tmax >= 0.0 {
        Some(tmin.max(0.0))
    } else {
        None
    }
}

/// Billboard content that always faces camera
#[derive(Debug, Clone)]
pub struct BillboardContent {
    /// Base content
    pub content: ArContent,
    /// Billboard mode
    pub mode: BillboardMode,
}

/// Billboard mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillboardMode {
    /// Face camera on all axes
    Full,
    /// Face camera only on Y axis
    YAxisOnly,
    /// No billboarding
    None,
}

impl BillboardContent {
    /// Create new billboard
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            content: ArContent::new(id, ContentType::Billboard, name.to_string()),
            mode: BillboardMode::Full,
        }
    }

    /// Update rotation to face camera
    pub fn face_camera(&mut self, camera_pos: Point3<f32>) {
        let direction = camera_pos - self.content.transform.position;
        if direction.norm() < 0.001 {
            return;
        }

        match self.mode {
            BillboardMode::Full => {
                let forward = direction.normalize();
                let right = Vector3::y().cross(&forward).normalize();
                let up = forward.cross(&right);
                
                // Create rotation from basis vectors
                let rotation = UnitQuaternion::face_towards(&forward, &up);
                self.content.transform.rotation = rotation;
            }
            BillboardMode::YAxisOnly => {
                let forward = Vector3::new(direction.x, 0.0, direction.z).normalize();
                if forward.norm() > 0.001 {
                    let rotation = UnitQuaternion::face_towards(&forward, &Vector3::y());
                    self.content.transform.rotation = rotation;
                }
            }
            BillboardMode::None => {}
        }
    }
}

/// Text label content
#[derive(Debug, Clone)]
pub struct TextLabelContent {
    /// Base content
    pub content: ArContent,
    /// Text string
    pub text: String,
    /// Font size
    pub font_size: f32,
    /// Text color (RGBA)
    pub color: [u8; 4],
    /// Background color (RGBA)
    pub background: [u8; 4],
    /// Max width (0 = unlimited)
    pub max_width: f32,
    /// Alignment
    pub alignment: TextAlignment,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl TextLabelContent {
    /// Create new text label
    pub fn new(id: u64, text: &str) -> Self {
        Self {
            content: ArContent::new(id, ContentType::Text, text.to_string()),
            text: text.to_string(),
            font_size: 24.0,
            color: [255, 255, 255, 255],
            background: [0, 0, 0, 128],
            max_width: 0.0,
            alignment: TextAlignment::Center,
        }
    }

    /// Set text
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.content.name = text.to_string();
        self.content.updated = Instant::now();
    }

    /// Set font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
        self.content.updated = Instant::now();
    }

    /// Set color
    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
        self.content.updated = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ar_content_creation() {
        let content = ArContent::new(1, ContentType::Model3D, "Test".to_string());
        assert_eq!(content.id, 1);
        assert_eq!(content.content_type, ContentType::Model3D);
        assert!(content.visible);
    }

    #[test]
    fn test_ar_content_position() {
        let mut content = ArContent::new(1, ContentType::Model3D, "Test".to_string());
        content.set_position(Point3::new(1.0, 2.0, 3.0));
        assert_eq!(content.transform.position, Point3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_ar_content_anchor() {
        let mut content = ArContent::new(1, ContentType::Model3D, "Test".to_string());
        content.attach_to_anchor(42);
        assert_eq!(content.anchor_id, Some(42));
        
        content.detach_from_anchor();
        assert_eq!(content.anchor_id, None);
    }

    #[test]
    fn test_content_manager_create() {
        let mut manager = ContentManager::new();
        let id = manager.create(ContentType::Model3D, "Test");
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_content_manager_remove() {
        let mut manager = ContentManager::new();
        let id = manager.create(ContentType::Model3D, "Test");
        let removed = manager.remove(id);
        assert!(removed.is_some());
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_content_manager_count() {
        let mut manager = ContentManager::new();
        assert_eq!(manager.count(), 0);
        
        manager.create(ContentType::Model3D, "Test1");
        manager.create(ContentType::Model3D, "Test2");
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        let center = bbox.center();
        assert!((center.x).abs() < 0.01);
        assert!((center.y).abs() < 0.01);
        assert!((center.z).abs() < 0.01);
    }

    #[test]
    fn test_billboard_creation() {
        let billboard = BillboardContent::new(1, "Test");
        assert_eq!(billboard.mode, BillboardMode::Full);
    }

    #[test]
    fn test_text_label_creation() {
        let label = TextLabelContent::new(1, "Hello");
        assert_eq!(label.text, "Hello");
        assert_eq!(label.font_size, 24.0);
    }

    #[test]
    fn test_text_label_set_text() {
        let mut label = TextLabelContent::new(1, "Hello");
        label.set_text("World");
        assert_eq!(label.text, "World");
    }
}
