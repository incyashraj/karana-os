//! Spatial Anchor Management
//!
//! Persistent spatial anchors for AR content placement.

use nalgebra::{Point3, UnitQuaternion, Matrix4, Vector3};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::SceneId;

/// Manages spatial anchors in the scene
#[derive(Debug)]
pub struct AnchorManager {
    anchors: HashMap<SceneId, SpatialAnchor>,
    persistence_enabled: bool,
    last_save: Instant,
    save_interval: Duration,
}

impl AnchorManager {
    pub fn new(persistence_enabled: bool) -> Self {
        Self {
            anchors: HashMap::new(),
            persistence_enabled,
            last_save: Instant::now(),
            save_interval: Duration::from_secs(30),
        }
    }
    
    /// Create a new anchor at position
    pub fn create_anchor(&mut self, position: Point3<f32>, name: Option<String>) -> SpatialAnchor {
        let anchor = SpatialAnchor::new(position, name);
        let id = anchor.id;
        self.anchors.insert(id, anchor.clone());
        anchor
    }
    
    /// Create anchor with full transform
    pub fn create_anchor_with_transform(
        &mut self,
        position: Point3<f32>,
        rotation: UnitQuaternion<f32>,
        name: Option<String>,
    ) -> SpatialAnchor {
        let mut anchor = SpatialAnchor::new(position, name);
        anchor.rotation = rotation;
        let id = anchor.id;
        self.anchors.insert(id, anchor.clone());
        anchor
    }
    
    /// Get anchor by ID
    pub fn get_anchor(&self, id: SceneId) -> Option<&SpatialAnchor> {
        self.anchors.get(&id)
    }
    
    /// Get mutable anchor
    pub fn get_anchor_mut(&mut self, id: SceneId) -> Option<&mut SpatialAnchor> {
        self.anchors.get_mut(&id)
    }
    
    /// Update anchor position
    pub fn update_anchor_position(&mut self, id: SceneId, position: Point3<f32>) -> bool {
        if let Some(anchor) = self.anchors.get_mut(&id) {
            anchor.position = position;
            anchor.last_updated = Instant::now();
            true
        } else {
            false
        }
    }
    
    /// Remove an anchor
    pub fn remove_anchor(&mut self, id: SceneId) -> bool {
        self.anchors.remove(&id).is_some()
    }
    
    /// Get all anchors
    pub fn all_anchors(&self) -> Vec<&SpatialAnchor> {
        self.anchors.values().collect()
    }
    
    /// Get anchors by state
    pub fn anchors_by_state(&self, state: AnchorState) -> Vec<&SpatialAnchor> {
        self.anchors.values()
            .filter(|a| a.state == state)
            .collect()
    }
    
    /// Get anchors in radius
    pub fn anchors_in_radius(&self, center: Point3<f32>, radius: f32) -> Vec<&SpatialAnchor> {
        self.anchors.values()
            .filter(|a| (a.position - center).norm() <= radius)
            .collect()
    }
    
    /// Find nearest anchor to point
    pub fn find_nearest(&self, point: &Point3<f32>) -> Option<&SpatialAnchor> {
        self.anchors.values()
            .min_by(|a, b| {
                let dist_a = (a.position - point).norm();
                let dist_b = (b.position - point).norm();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
    
    /// Update tracking state for all anchors
    pub fn update_tracking(&mut self) {
        let now = Instant::now();
        
        for anchor in self.anchors.values_mut() {
            // Mark as limited if not updated recently
            let age = now.duration_since(anchor.last_updated);
            if age > Duration::from_secs(5) {
                anchor.state = AnchorState::Limited;
            }
            if age > Duration::from_secs(30) {
                anchor.state = AnchorState::NotTracking;
            }
        }
    }
    
    /// Relocalize anchors (after tracking loss)
    pub fn relocalize(&mut self, _relocalization_data: &[u8]) -> usize {
        let mut count = 0;
        
        for anchor in self.anchors.values_mut() {
            if anchor.state == AnchorState::NotTracking {
                // Placeholder: Real implementation would use relocalization data
                anchor.state = AnchorState::Limited;
                count += 1;
            }
        }
        
        count
    }
    
    /// Serialize anchors for persistence
    pub fn serialize(&self) -> Vec<AnchorData> {
        self.anchors.values()
            .map(|a| AnchorData {
                id: a.id,
                name: a.name.clone(),
                position: [a.position.x, a.position.y, a.position.z],
                rotation: [
                    a.rotation.quaternion().i,
                    a.rotation.quaternion().j,
                    a.rotation.quaternion().k,
                    a.rotation.quaternion().w,
                ],
                cloud_id: a.cloud_id.clone(),
            })
            .collect()
    }
    
    /// Deserialize and restore anchors
    pub fn deserialize(&mut self, data: &[AnchorData]) {
        for anchor_data in data {
            let position = Point3::new(
                anchor_data.position[0],
                anchor_data.position[1],
                anchor_data.position[2],
            );
            
            let rotation = UnitQuaternion::from_quaternion(
                nalgebra::Quaternion::new(
                    anchor_data.rotation[3],
                    anchor_data.rotation[0],
                    anchor_data.rotation[1],
                    anchor_data.rotation[2],
                )
            );
            
            let mut anchor = SpatialAnchor {
                id: anchor_data.id,
                name: anchor_data.name.clone(),
                position,
                rotation,
                state: AnchorState::NotTracking,
                confidence: 0.0,
                cloud_id: anchor_data.cloud_id.clone(),
                created_at: Instant::now(),
                last_updated: Instant::now(),
            };
            
            self.anchors.insert(anchor.id, anchor);
        }
    }
    
    /// Count of anchors
    pub fn count(&self) -> usize {
        self.anchors.len()
    }
    
    /// Clear all anchors
    pub fn clear(&mut self) {
        self.anchors.clear();
    }
}

/// A spatial anchor for AR content placement
#[derive(Debug, Clone)]
pub struct SpatialAnchor {
    /// Unique identifier
    pub id: SceneId,
    /// Optional name
    pub name: Option<String>,
    /// Position in world space
    pub position: Point3<f32>,
    /// Rotation
    pub rotation: UnitQuaternion<f32>,
    /// Tracking state
    pub state: AnchorState,
    /// Tracking confidence (0-1)
    pub confidence: f32,
    /// Cloud anchor ID (for cross-device sharing)
    pub cloud_id: Option<String>,
    /// Creation time
    pub created_at: Instant,
    /// Last update time
    pub last_updated: Instant,
}

impl SpatialAnchor {
    pub fn new(position: Point3<f32>, name: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            position,
            rotation: UnitQuaternion::identity(),
            state: AnchorState::Tracking,
            confidence: 1.0,
            cloud_id: None,
            created_at: Instant::now(),
            last_updated: Instant::now(),
        }
    }
    
    /// Get transformation matrix
    pub fn transform(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.position.coords);
        let rotation = self.rotation.to_homogeneous();
        translation * rotation
    }
    
    /// Transform a point from anchor-local to world space
    pub fn transform_point(&self, local_point: Point3<f32>) -> Point3<f32> {
        let rotated = self.rotation * local_point.coords;
        Point3::from(rotated + self.position.coords)
    }
    
    /// Transform direction from anchor-local to world space
    pub fn transform_direction(&self, local_dir: Vector3<f32>) -> Vector3<f32> {
        self.rotation * local_dir
    }
    
    /// Inverse transform (world to anchor-local)
    pub fn inverse_transform_point(&self, world_point: Point3<f32>) -> Point3<f32> {
        let relative = world_point - self.position;
        Point3::from(self.rotation.inverse() * relative)
    }
    
    /// Check if anchor is being tracked
    pub fn is_tracking(&self) -> bool {
        self.state == AnchorState::Tracking
    }
    
    /// Check if anchor is usable (tracking or limited)
    pub fn is_usable(&self) -> bool {
        matches!(self.state, AnchorState::Tracking | AnchorState::Limited)
    }
    
    /// Distance to a point
    pub fn distance_to(&self, point: &Point3<f32>) -> f32 {
        (self.position - point).norm()
    }
    
    /// Age since creation
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Tracking state of an anchor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorState {
    /// Fully tracked with high confidence
    Tracking,
    /// Tracking with reduced confidence
    Limited,
    /// Not currently tracked
    NotTracking,
    /// Pending resolution (e.g., cloud anchor)
    Pending,
}

/// Serializable anchor data
#[derive(Debug, Clone)]
pub struct AnchorData {
    pub id: SceneId,
    pub name: Option<String>,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion [x, y, z, w]
    pub cloud_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anchor_manager_creation() {
        let manager = AnchorManager::new(true);
        assert_eq!(manager.count(), 0);
    }
    
    #[test]
    fn test_create_anchor() {
        let mut manager = AnchorManager::new(false);
        
        let anchor = manager.create_anchor(Point3::new(1.0, 2.0, 3.0), Some("Test".to_string()));
        
        assert_eq!(manager.count(), 1);
        assert_eq!(anchor.name, Some("Test".to_string()));
        assert!(anchor.is_tracking());
    }
    
    #[test]
    fn test_get_anchor() {
        let mut manager = AnchorManager::new(false);
        
        let anchor = manager.create_anchor(Point3::new(0.0, 0.0, 0.0), None);
        let id = anchor.id;
        
        let retrieved = manager.get_anchor(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }
    
    #[test]
    fn test_remove_anchor() {
        let mut manager = AnchorManager::new(false);
        
        let anchor = manager.create_anchor(Point3::new(0.0, 0.0, 0.0), None);
        let id = anchor.id;
        
        assert!(manager.remove_anchor(id));
        assert_eq!(manager.count(), 0);
        assert!(!manager.remove_anchor(id)); // Already removed
    }
    
    #[test]
    fn test_update_anchor_position() {
        let mut manager = AnchorManager::new(false);
        
        let anchor = manager.create_anchor(Point3::new(0.0, 0.0, 0.0), None);
        let id = anchor.id;
        
        assert!(manager.update_anchor_position(id, Point3::new(5.0, 5.0, 5.0)));
        
        let updated = manager.get_anchor(id).unwrap();
        assert!((updated.position.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_anchors_in_radius() {
        let mut manager = AnchorManager::new(false);
        
        manager.create_anchor(Point3::new(0.0, 0.0, 0.0), None);
        manager.create_anchor(Point3::new(1.0, 0.0, 0.0), None);
        manager.create_anchor(Point3::new(10.0, 0.0, 0.0), None);
        
        let nearby = manager.anchors_in_radius(Point3::origin(), 2.0);
        assert_eq!(nearby.len(), 2);
    }
    
    #[test]
    fn test_find_nearest() {
        let mut manager = AnchorManager::new(false);
        
        manager.create_anchor(Point3::new(0.0, 0.0, 0.0), Some("A".to_string()));
        manager.create_anchor(Point3::new(5.0, 0.0, 0.0), Some("B".to_string()));
        
        let nearest = manager.find_nearest(&Point3::new(1.0, 0.0, 0.0));
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().name, Some("A".to_string()));
    }
    
    #[test]
    fn test_anchor_transform() {
        let anchor = SpatialAnchor::new(Point3::new(10.0, 0.0, 0.0), None);
        
        let local_point = Point3::new(1.0, 0.0, 0.0);
        let world_point = anchor.transform_point(local_point);
        
        assert!((world_point.x - 11.0).abs() < 0.001);
    }
    
    #[test]
    fn test_anchor_inverse_transform() {
        let anchor = SpatialAnchor::new(Point3::new(10.0, 0.0, 0.0), None);
        
        let world_point = Point3::new(15.0, 0.0, 0.0);
        let local_point = anchor.inverse_transform_point(world_point);
        
        assert!((local_point.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_anchor_state() {
        let mut anchor = SpatialAnchor::new(Point3::origin(), None);
        
        assert!(anchor.is_tracking());
        assert!(anchor.is_usable());
        
        anchor.state = AnchorState::Limited;
        assert!(!anchor.is_tracking());
        assert!(anchor.is_usable());
        
        anchor.state = AnchorState::NotTracking;
        assert!(!anchor.is_usable());
    }
    
    #[test]
    fn test_serialize_deserialize() {
        let mut manager = AnchorManager::new(false);
        
        manager.create_anchor(Point3::new(1.0, 2.0, 3.0), Some("Test".to_string()));
        manager.create_anchor(Point3::new(4.0, 5.0, 6.0), None);
        
        let data = manager.serialize();
        assert_eq!(data.len(), 2);
        
        let mut new_manager = AnchorManager::new(false);
        new_manager.deserialize(&data);
        
        assert_eq!(new_manager.count(), 2);
    }
    
    #[test]
    fn test_anchor_distance() {
        let anchor = SpatialAnchor::new(Point3::new(0.0, 0.0, 0.0), None);
        
        let distance = anchor.distance_to(&Point3::new(3.0, 4.0, 0.0));
        assert!((distance - 5.0).abs() < 0.001);
    }
}
