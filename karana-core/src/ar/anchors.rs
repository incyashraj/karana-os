// AR Spatial Anchors for Kāraṇa OS
// Handles world-locked content anchoring and persistence

use super::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Anchor tracking state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorState {
    /// Anchor is being created/initialized
    Pending,
    /// Anchor is actively tracked
    Tracking,
    /// Anchor lost tracking temporarily
    Limited,
    /// Anchor tracking failed
    Lost,
    /// Anchor is paused (not actively tracking)
    Paused,
}

/// Type of spatial anchor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorType {
    /// Anchored to a detected plane
    Plane,
    /// Anchored to a point in space
    Point,
    /// Anchored to a detected image marker
    Image,
    /// Anchored to a detected object
    Object,
    /// Anchored to QR/marker code
    Marker,
    /// Anchored relative to user position
    UserRelative,
}

/// A spatial anchor for world-locked content
#[derive(Debug, Clone)]
pub struct SpatialAnchor {
    pub id: ContentId,
    pub name: String,
    pub anchor_type: AnchorType,
    pub transform: Transform,
    pub state: AnchorState,
    pub confidence: f32,
    pub created_at: Instant,
    pub last_updated: Instant,
    pub persistent: bool,
    pub cloud_id: Option<String>,
}

impl SpatialAnchor {
    pub fn new(name: &str, anchor_type: AnchorType, position: Point3<f32>) -> Self {
        let now = Instant::now();
        Self {
            id: next_content_id(),
            name: name.to_string(),
            anchor_type,
            transform: Transform {
                position,
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            state: AnchorState::Pending,
            confidence: 0.0,
            created_at: now,
            last_updated: now,
            persistent: false,
            cloud_id: None,
        }
    }
    
    /// Create point anchor
    pub fn point(name: &str, x: f32, y: f32, z: f32) -> Self {
        Self::new(name, AnchorType::Point, Point3::new(x, y, z))
    }
    
    /// Create plane anchor
    pub fn plane(name: &str, position: Point3<f32>, normal: Vector3<f32>) -> Self {
        let mut anchor = Self::new(name, AnchorType::Plane, position);
        
        // Orient anchor to face along normal
        if normal.norm() > 0.0 {
            let up = if normal.y.abs() < 0.9 {
                Vector3::y()
            } else {
                Vector3::z()
            };
            anchor.transform.rotation = UnitQuaternion::face_towards(&normal, &up);
        }
        
        anchor
    }
    
    /// Mark as persistent (saved across sessions)
    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }
    
    /// Update tracking state
    pub fn update_tracking(&mut self, new_transform: Transform, confidence: f32) {
        self.transform = new_transform;
        self.confidence = confidence.clamp(0.0, 1.0);
        self.last_updated = Instant::now();
        
        self.state = if confidence > 0.8 {
            AnchorState::Tracking
        } else if confidence > 0.3 {
            AnchorState::Limited
        } else {
            AnchorState::Lost
        };
    }
    
    /// Check if anchor is usable
    pub fn is_tracking(&self) -> bool {
        matches!(self.state, AnchorState::Tracking | AnchorState::Limited)
    }
    
    /// Get age of anchor
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
    
    /// Time since last update
    pub fn time_since_update(&self) -> Duration {
        self.last_updated.elapsed()
    }
}

/// Anchor manager for handling multiple anchors
pub struct AnchorManager {
    anchors: HashMap<ContentId, SpatialAnchor>,
    attached_content: HashMap<ContentId, Vec<ContentId>>,
    max_anchors: usize,
    auto_cleanup: bool,
    stale_threshold: Duration,
}

impl AnchorManager {
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
            attached_content: HashMap::new(),
            max_anchors: 100,
            auto_cleanup: true,
            stale_threshold: Duration::from_secs(60),
        }
    }
    
    /// Set maximum anchor count
    pub fn set_max_anchors(&mut self, max: usize) {
        self.max_anchors = max;
    }
    
    /// Add a new anchor
    pub fn add_anchor(&mut self, anchor: SpatialAnchor) -> Option<ContentId> {
        if self.anchors.len() >= self.max_anchors {
            if self.auto_cleanup {
                self.cleanup_stale();
            }
            if self.anchors.len() >= self.max_anchors {
                return None;
            }
        }
        
        let id = anchor.id;
        self.anchors.insert(id, anchor);
        self.attached_content.insert(id, Vec::new());
        Some(id)
    }
    
    /// Remove an anchor
    pub fn remove_anchor(&mut self, anchor_id: ContentId) -> Option<SpatialAnchor> {
        self.attached_content.remove(&anchor_id);
        self.anchors.remove(&anchor_id)
    }
    
    /// Get anchor by ID
    pub fn get_anchor(&self, anchor_id: ContentId) -> Option<&SpatialAnchor> {
        self.anchors.get(&anchor_id)
    }
    
    /// Get mutable anchor by ID
    pub fn get_anchor_mut(&mut self, anchor_id: ContentId) -> Option<&mut SpatialAnchor> {
        self.anchors.get_mut(&anchor_id)
    }
    
    /// Get all anchors
    pub fn all_anchors(&self) -> impl Iterator<Item = &SpatialAnchor> {
        self.anchors.values()
    }
    
    /// Get tracking anchors
    pub fn tracking_anchors(&self) -> impl Iterator<Item = &SpatialAnchor> {
        self.anchors.values().filter(|a| a.is_tracking())
    }
    
    /// Attach content to an anchor
    pub fn attach_content(&mut self, anchor_id: ContentId, content_id: ContentId) -> bool {
        if let Some(content_list) = self.attached_content.get_mut(&anchor_id) {
            if !content_list.contains(&content_id) {
                content_list.push(content_id);
            }
            true
        } else {
            false
        }
    }
    
    /// Detach content from an anchor
    pub fn detach_content(&mut self, anchor_id: ContentId, content_id: ContentId) {
        if let Some(content_list) = self.attached_content.get_mut(&anchor_id) {
            content_list.retain(|&id| id != content_id);
        }
    }
    
    /// Get content attached to an anchor
    pub fn get_attached_content(&self, anchor_id: ContentId) -> Option<&Vec<ContentId>> {
        self.attached_content.get(&anchor_id)
    }
    
    /// Find anchor by name
    pub fn find_by_name(&self, name: &str) -> Option<&SpatialAnchor> {
        self.anchors.values().find(|a| a.name == name)
    }
    
    /// Find nearest anchor to a point
    pub fn find_nearest(&self, position: &Point3<f32>) -> Option<&SpatialAnchor> {
        self.anchors.values()
            .filter(|a| a.is_tracking())
            .min_by(|a, b| {
                let dist_a = (a.transform.position - position).norm();
                let dist_b = (b.transform.position - position).norm();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }
    
    /// Find anchors within radius
    pub fn find_within_radius(&self, position: &Point3<f32>, radius: f32) -> Vec<&SpatialAnchor> {
        self.anchors.values()
            .filter(|a| {
                a.is_tracking() && 
                (a.transform.position - position).norm() <= radius
            })
            .collect()
    }
    
    /// Cleanup stale anchors
    pub fn cleanup_stale(&mut self) {
        let threshold = self.stale_threshold;
        let stale_ids: Vec<ContentId> = self.anchors.iter()
            .filter(|(_, a)| {
                !a.persistent && 
                a.state == AnchorState::Lost &&
                a.time_since_update() > threshold
            })
            .map(|(&id, _)| id)
            .collect();
        
        for id in stale_ids {
            self.remove_anchor(id);
        }
    }
    
    /// Get anchor count
    pub fn anchor_count(&self) -> usize {
        self.anchors.len()
    }
    
    /// Get tracking anchor count
    pub fn tracking_count(&self) -> usize {
        self.anchors.values().filter(|a| a.is_tracking()).count()
    }
    
    /// Clear all non-persistent anchors
    pub fn clear_non_persistent(&mut self) {
        let non_persistent: Vec<ContentId> = self.anchors.iter()
            .filter(|(_, a)| !a.persistent)
            .map(|(&id, _)| id)
            .collect();
        
        for id in non_persistent {
            self.remove_anchor(id);
        }
    }
}

impl Default for AnchorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Detected plane for anchoring
#[derive(Debug, Clone)]
pub struct DetectedPlane {
    pub id: ContentId,
    pub plane_type: PlaneType,
    pub center: Point3<f32>,
    pub normal: Vector3<f32>,
    pub extents: (f32, f32),
    pub boundary: Vec<Point3<f32>>,
    pub confidence: f32,
}

/// Type of detected plane
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneType {
    HorizontalUp,   // Floor, table
    HorizontalDown, // Ceiling
    Vertical,       // Wall
    Unknown,
}

impl DetectedPlane {
    pub fn new(plane_type: PlaneType, center: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            id: next_content_id(),
            plane_type,
            center,
            normal: normal.normalize(),
            extents: (1.0, 1.0),
            boundary: Vec::new(),
            confidence: 1.0,
        }
    }
    
    /// Get area of the plane
    pub fn area(&self) -> f32 {
        self.extents.0 * self.extents.1
    }
    
    /// Check if point is on plane (within tolerance)
    pub fn is_point_on_plane(&self, point: &Point3<f32>, tolerance: f32) -> bool {
        let to_point = point - self.center;
        let distance = to_point.dot(&self.normal).abs();
        distance <= tolerance
    }
    
    /// Project point onto plane
    pub fn project_point(&self, point: &Point3<f32>) -> Point3<f32> {
        let to_point = point - self.center;
        let distance = to_point.dot(&self.normal);
        point - self.normal * distance
    }
    
    /// Create anchor on this plane
    pub fn create_anchor(&self, name: &str, position: Point3<f32>) -> SpatialAnchor {
        let projected = self.project_point(&position);
        SpatialAnchor::plane(name, projected, self.normal)
    }
}

/// Image marker for image-based anchoring
#[derive(Debug, Clone)]
pub struct ImageMarker {
    pub id: ContentId,
    pub name: String,
    pub physical_width: f32,
    pub image_data: Vec<u8>,
    pub detected: bool,
    pub transform: Option<Transform>,
}

impl ImageMarker {
    pub fn new(name: &str, physical_width: f32) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            physical_width,
            image_data: Vec::new(),
            detected: false,
            transform: None,
        }
    }
    
    /// Set image data
    pub fn with_image_data(mut self, data: Vec<u8>) -> Self {
        self.image_data = data;
        self
    }
    
    /// Update detection
    pub fn update_detection(&mut self, transform: Option<Transform>) {
        self.transform = transform;
        self.detected = transform.is_some();
    }
    
    /// Create anchor from detected marker
    pub fn create_anchor(&self) -> Option<SpatialAnchor> {
        self.transform.map(|t| {
            let mut anchor = SpatialAnchor::new(&self.name, AnchorType::Image, t.position);
            anchor.transform = t;
            anchor.state = AnchorState::Tracking;
            anchor.confidence = 1.0;
            anchor
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_anchor_creation() {
        let anchor = SpatialAnchor::point("test", 1.0, 2.0, 3.0);
        
        assert_eq!(anchor.name, "test");
        assert_eq!(anchor.anchor_type, AnchorType::Point);
        assert_eq!(anchor.transform.position.x, 1.0);
    }
    
    #[test]
    fn test_spatial_anchor_plane() {
        let anchor = SpatialAnchor::plane(
            "floor",
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        
        assert_eq!(anchor.anchor_type, AnchorType::Plane);
    }
    
    #[test]
    fn test_anchor_tracking_update() {
        let mut anchor = SpatialAnchor::point("test", 0.0, 0.0, 0.0);
        
        let new_transform = Transform::at_position(1.0, 1.0, 1.0);
        anchor.update_tracking(new_transform, 0.95);
        
        assert_eq!(anchor.state, AnchorState::Tracking);
        assert!(anchor.is_tracking());
        
        anchor.update_tracking(new_transform, 0.1);
        assert_eq!(anchor.state, AnchorState::Lost);
        assert!(!anchor.is_tracking());
    }
    
    #[test]
    fn test_anchor_manager() {
        let mut manager = AnchorManager::new();
        
        let anchor1 = SpatialAnchor::point("a1", 0.0, 0.0, 0.0);
        let anchor2 = SpatialAnchor::point("a2", 5.0, 0.0, 0.0);
        
        let id1 = manager.add_anchor(anchor1).unwrap();
        let id2 = manager.add_anchor(anchor2).unwrap();
        
        assert_eq!(manager.anchor_count(), 2);
        
        // Update tracking
        if let Some(anchor) = manager.get_anchor_mut(id1) {
            anchor.update_tracking(anchor.transform, 0.9);
        }
        
        assert_eq!(manager.tracking_count(), 1);
    }
    
    #[test]
    fn test_anchor_manager_attach_content() {
        let mut manager = AnchorManager::new();
        
        let anchor = SpatialAnchor::point("test", 0.0, 0.0, 0.0);
        let anchor_id = manager.add_anchor(anchor).unwrap();
        
        manager.attach_content(anchor_id, 100);
        manager.attach_content(anchor_id, 101);
        
        let attached = manager.get_attached_content(anchor_id).unwrap();
        assert_eq!(attached.len(), 2);
        
        manager.detach_content(anchor_id, 100);
        assert_eq!(manager.get_attached_content(anchor_id).unwrap().len(), 1);
    }
    
    #[test]
    fn test_anchor_manager_find_nearest() {
        let mut manager = AnchorManager::new();
        
        let mut anchor1 = SpatialAnchor::point("near", 1.0, 0.0, 0.0);
        let mut anchor2 = SpatialAnchor::point("far", 10.0, 0.0, 0.0);
        
        anchor1.state = AnchorState::Tracking;
        anchor2.state = AnchorState::Tracking;
        
        manager.add_anchor(anchor1);
        manager.add_anchor(anchor2);
        
        let nearest = manager.find_nearest(&Point3::origin()).unwrap();
        assert_eq!(nearest.name, "near");
    }
    
    #[test]
    fn test_anchor_manager_find_by_name() {
        let mut manager = AnchorManager::new();
        
        let anchor = SpatialAnchor::point("my_anchor", 0.0, 0.0, 0.0);
        manager.add_anchor(anchor);
        
        assert!(manager.find_by_name("my_anchor").is_some());
        assert!(manager.find_by_name("nonexistent").is_none());
    }
    
    #[test]
    fn test_detected_plane() {
        let plane = DetectedPlane::new(
            PlaneType::HorizontalUp,
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        
        assert!(plane.is_point_on_plane(&Point3::new(5.0, 0.0, 5.0), 0.01));
        assert!(!plane.is_point_on_plane(&Point3::new(0.0, 1.0, 0.0), 0.01));
    }
    
    #[test]
    fn test_detected_plane_project() {
        let plane = DetectedPlane::new(
            PlaneType::HorizontalUp,
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        
        let projected = plane.project_point(&Point3::new(3.0, 2.0, 4.0));
        
        assert!((projected.x - 3.0).abs() < 0.001);
        assert!(projected.y.abs() < 0.001);
        assert!((projected.z - 4.0).abs() < 0.001);
    }
    
    #[test]
    fn test_image_marker() {
        let mut marker = ImageMarker::new("poster", 0.5);
        
        assert!(!marker.detected);
        
        marker.update_detection(Some(Transform::at_position(1.0, 1.0, 0.0)));
        
        assert!(marker.detected);
        assert!(marker.create_anchor().is_some());
    }
    
    #[test]
    fn test_anchor_persistent() {
        let anchor = SpatialAnchor::point("test", 0.0, 0.0, 0.0)
            .with_persistent(true);
        
        assert!(anchor.persistent);
    }
}
