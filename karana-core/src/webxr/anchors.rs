//! WebXR Anchors API
//!
//! Enables web content to create persistent spatial anchors that survive
//! session restarts and can be shared between sessions.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{XRRigidTransform, XRTrackingState};

/// WebXR Anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRAnchor {
    /// Unique identifier
    pub id: String,
    /// Current pose in reference space
    pub pose: XRRigidTransform,
    /// Tracking state
    pub tracking_state: XRTrackingState,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub last_updated: u64,
    /// Persistence status
    pub persistence: AnchorPersistence,
}

impl XRAnchor {
    /// Create a new anchor
    pub fn new(pose: XRRigidTransform) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id: Uuid::new_v4().to_string(),
            pose,
            tracking_state: XRTrackingState::Tracking,
            created_at: now,
            last_updated: now,
            persistence: AnchorPersistence::Session,
        }
    }
    
    /// Create a persistent anchor
    pub fn new_persistent(pose: XRRigidTransform) -> Self {
        let mut anchor = Self::new(pose);
        anchor.persistence = AnchorPersistence::Persistent;
        anchor
    }
    
    /// Update the anchor's pose
    pub fn update_pose(&mut self, pose: XRRigidTransform) {
        self.pose = pose;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Set tracking state
    pub fn set_tracking_state(&mut self, state: XRTrackingState) {
        self.tracking_state = state;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Check if anchor is currently tracking
    pub fn is_tracking(&self) -> bool {
        matches!(self.tracking_state, 
            XRTrackingState::Tracking | XRTrackingState::Emulated)
    }
    
    /// Request persistence
    pub fn request_persistence(&mut self) -> bool {
        if self.persistence == AnchorPersistence::Session {
            self.persistence = AnchorPersistence::Persistent;
            true
        } else {
            false
        }
    }
    
    /// Delete anchor
    pub fn delete(&mut self) {
        self.tracking_state = XRTrackingState::NotTracking;
    }
}

/// Anchor persistence mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorPersistence {
    /// Anchor exists only for current session
    Session,
    /// Anchor is stored persistently
    Persistent,
}

/// Persistent anchor store - integrates with Kāraṇa OS spatial system
pub struct PersistentAnchorStore {
    /// Stored anchors by ID
    anchors: Vec<StoredAnchor>,
}

/// Stored anchor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAnchor {
    /// Anchor ID
    pub id: String,
    /// Pose when stored
    pub pose: XRRigidTransform,
    /// Storage timestamp
    pub stored_at: u64,
    /// Domain that created this anchor
    pub origin_domain: String,
    /// Application-provided name
    pub name: Option<String>,
}

impl PersistentAnchorStore {
    /// Create a new store
    pub fn new() -> Self {
        Self {
            anchors: vec![],
        }
    }
    
    /// Store an anchor
    pub fn store(&mut self, anchor: &XRAnchor, domain: &str, name: Option<String>) -> bool {
        // Check if already stored
        if self.anchors.iter().any(|a| a.id == anchor.id) {
            return false;
        }
        
        let stored = StoredAnchor {
            id: anchor.id.clone(),
            pose: anchor.pose.clone(),
            stored_at: anchor.created_at,
            origin_domain: domain.to_string(),
            name,
        };
        
        self.anchors.push(stored);
        true
    }
    
    /// Retrieve an anchor
    pub fn get(&self, id: &str) -> Option<&StoredAnchor> {
        self.anchors.iter().find(|a| a.id == id)
    }
    
    /// Get all anchors for a domain
    pub fn get_for_domain(&self, domain: &str) -> Vec<&StoredAnchor> {
        self.anchors.iter()
            .filter(|a| a.origin_domain == domain)
            .collect()
    }
    
    /// Delete a stored anchor
    pub fn delete(&mut self, id: &str) -> bool {
        if let Some(idx) = self.anchors.iter().position(|a| a.id == id) {
            self.anchors.remove(idx);
            true
        } else {
            false
        }
    }
    
    /// Clear all anchors for a domain
    pub fn clear_domain(&mut self, domain: &str) {
        self.anchors.retain(|a| a.origin_domain != domain);
    }
    
    /// Get anchor count
    pub fn count(&self) -> usize {
        self.anchors.len()
    }
}

impl Default for PersistentAnchorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Anchor creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorCreateOptions {
    /// Make anchor persistent immediately
    pub persistent: bool,
    /// Human-readable name
    pub name: Option<String>,
    /// Additional metadata
    pub metadata: Option<String>,
}

impl Default for AnchorCreateOptions {
    fn default() -> Self {
        Self {
            persistent: false,
            name: None,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anchor_creation() {
        let pose = XRRigidTransform::from_position(1.0, 0.5, -2.0);
        let anchor = XRAnchor::new(pose);
        
        assert!(!anchor.id.is_empty());
        assert!(anchor.is_tracking());
        assert_eq!(anchor.persistence, AnchorPersistence::Session);
    }
    
    #[test]
    fn test_anchor_persistence() {
        let pose = XRRigidTransform::from_position(0.0, 0.0, -1.0);
        let mut anchor = XRAnchor::new(pose);
        
        assert!(anchor.request_persistence());
        assert_eq!(anchor.persistence, AnchorPersistence::Persistent);
        
        // Can't request again
        assert!(!anchor.request_persistence());
    }
    
    #[test]
    fn test_anchor_update() {
        let mut anchor = XRAnchor::new(XRRigidTransform::identity());
        let initial_update = anchor.last_updated;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        anchor.update_pose(XRRigidTransform::from_position(1.0, 0.0, 0.0));
        assert!(anchor.last_updated >= initial_update);
    }
    
    #[test]
    fn test_persistent_store() {
        let mut store = PersistentAnchorStore::new();
        
        let anchor = XRAnchor::new(XRRigidTransform::identity());
        let id = anchor.id.clone();
        
        assert!(store.store(&anchor, "example.com", Some("test".to_string())));
        assert!(!store.store(&anchor, "example.com", None)); // Duplicate
        
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.origin_domain, "example.com");
        assert_eq!(retrieved.name, Some("test".to_string()));
    }
    
    #[test]
    fn test_store_domain_filtering() {
        let mut store = PersistentAnchorStore::new();
        
        let a1 = XRAnchor::new(XRRigidTransform::identity());
        let a2 = XRAnchor::new(XRRigidTransform::identity());
        let a3 = XRAnchor::new(XRRigidTransform::identity());
        
        store.store(&a1, "example.com", None);
        store.store(&a2, "example.com", None);
        store.store(&a3, "other.com", None);
        
        assert_eq!(store.get_for_domain("example.com").len(), 2);
        assert_eq!(store.get_for_domain("other.com").len(), 1);
        
        store.clear_domain("example.com");
        assert_eq!(store.count(), 1);
    }
    
    #[test]
    fn test_tracking_state() {
        let mut anchor = XRAnchor::new(XRRigidTransform::identity());
        assert!(anchor.is_tracking());
        
        anchor.set_tracking_state(XRTrackingState::NotTracking);
        assert!(!anchor.is_tracking());
        
        anchor.set_tracking_state(XRTrackingState::Emulated);
        assert!(anchor.is_tracking());
    }
}
