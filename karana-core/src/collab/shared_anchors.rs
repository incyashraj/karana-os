//! Shared Anchors
//!
//! Enables sharing spatial anchors between users for collaborative AR experiences.
//! Shared anchors allow multiple users to see the same virtual content at the same
//! real-world location.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::WorldPosition;
use super::ParticipantId;

/// Shared anchor with ownership and permission info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedAnchor {
    /// Unique ID
    pub id: SharedAnchorId,
    /// Underlying spatial anchor ID (local)
    pub local_anchor_id: Option<String>,
    /// Position in world space
    pub position: WorldPosition,
    /// Anchor orientation (quaternion)
    pub orientation: [f32; 4],
    /// Owner participant
    pub owner_id: ParticipantId,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Content attached to anchor
    pub content: SharedAnchorContent,
    /// Permissions
    pub permissions: AnchorPermissions,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Version for conflict resolution
    pub version: u64,
}

/// Unique shared anchor identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SharedAnchorId(pub String);

impl SharedAnchorId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for SharedAnchorId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SharedAnchorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Content that can be attached to a shared anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharedAnchorContent {
    /// No content, just a marker
    Marker,
    /// 3D model reference
    Model {
        url: String,
        scale: f32,
    },
    /// Text label
    Label {
        text: String,
        style: LabelStyle,
    },
    /// Image or video
    Media {
        url: String,
        width: f32,
        height: f32,
    },
    /// Link to a shared tab
    TabLink {
        tab_id: String,
    },
    /// Custom content
    Custom {
        content_type: String,
        data: String,
    },
}

/// Label display style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStyle {
    /// Font size
    pub font_size: f32,
    /// Text color (hex)
    pub color: String,
    /// Background color (hex, with alpha)
    pub background: Option<String>,
    /// Billboard (always face user)
    pub billboard: bool,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            font_size: 0.05, // 5cm text
            color: "#FFFFFF".to_string(),
            background: Some("#00000080".to_string()),
            billboard: true,
        }
    }
}

/// Anchor permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPermissions {
    /// Who can view the anchor
    pub view: PermissionLevel,
    /// Who can edit the anchor
    pub edit: PermissionLevel,
    /// Who can delete the anchor
    pub delete: PermissionLevel,
    /// Who can move/reposition
    pub move_anchor: PermissionLevel,
}

impl Default for AnchorPermissions {
    fn default() -> Self {
        Self {
            view: PermissionLevel::Everyone,
            edit: PermissionLevel::Owner,
            delete: PermissionLevel::Owner,
            move_anchor: PermissionLevel::Collaborators,
        }
    }
}

/// Permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Only the owner
    Owner,
    /// Owner and collaborators
    Collaborators,
    /// Everyone in session
    Everyone,
    /// Nobody (disabled)
    Nobody,
}

impl SharedAnchor {
    /// Create a new shared anchor
    pub fn new(
        owner_id: ParticipantId,
        name: String,
        position: WorldPosition,
        content: SharedAnchorContent,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id: SharedAnchorId::new(),
            local_anchor_id: None,
            position,
            orientation: [0.0, 0.0, 0.0, 1.0],
            owner_id,
            name,
            description: None,
            content,
            permissions: AnchorPermissions::default(),
            created_at: now,
            modified_at: now,
            version: 1,
        }
    }
    
    /// Create a marker anchor
    pub fn marker(owner_id: ParticipantId, name: String, position: WorldPosition) -> Self {
        Self::new(owner_id, name, position, SharedAnchorContent::Marker)
    }
    
    /// Create a label anchor
    pub fn label(owner_id: ParticipantId, name: String, position: WorldPosition, text: String) -> Self {
        Self::new(
            owner_id,
            name,
            position,
            SharedAnchorContent::Label {
                text,
                style: LabelStyle::default(),
            },
        )
    }
    
    /// Check if user can view
    pub fn can_view(&self, participant: &ParticipantId, is_collaborator: bool) -> bool {
        match self.permissions.view {
            PermissionLevel::Owner => &self.owner_id == participant,
            PermissionLevel::Collaborators => &self.owner_id == participant || is_collaborator,
            PermissionLevel::Everyone => true,
            PermissionLevel::Nobody => false,
        }
    }
    
    /// Check if user can edit
    pub fn can_edit(&self, participant: &ParticipantId, is_collaborator: bool) -> bool {
        match self.permissions.edit {
            PermissionLevel::Owner => &self.owner_id == participant,
            PermissionLevel::Collaborators => &self.owner_id == participant || is_collaborator,
            PermissionLevel::Everyone => true,
            PermissionLevel::Nobody => false,
        }
    }
    
    /// Check if user can move
    pub fn can_move(&self, participant: &ParticipantId, is_collaborator: bool) -> bool {
        match self.permissions.move_anchor {
            PermissionLevel::Owner => &self.owner_id == participant,
            PermissionLevel::Collaborators => &self.owner_id == participant || is_collaborator,
            PermissionLevel::Everyone => true,
            PermissionLevel::Nobody => false,
        }
    }
    
    /// Update position
    pub fn set_position(&mut self, position: WorldPosition) {
        self.position = position;
        self.touch();
    }
    
    /// Update orientation
    pub fn set_orientation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.orientation = [x, y, z, w];
        self.touch();
    }
    
    /// Update content
    pub fn set_content(&mut self, content: SharedAnchorContent) {
        self.content = content;
        self.touch();
    }
    
    /// Mark as modified
    fn touch(&mut self) {
        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.version += 1;
    }
}

/// Shared anchor manager
pub struct SharedAnchorManager {
    /// Anchors we've shared
    shared: HashMap<SharedAnchorId, SharedAnchor>,
    /// Anchors shared with us by others
    received: HashMap<SharedAnchorId, SharedAnchor>,
    /// Mapping from shared ID to local anchor ID
    local_mapping: HashMap<SharedAnchorId, String>,
}

impl SharedAnchorManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            shared: HashMap::new(),
            received: HashMap::new(),
            local_mapping: HashMap::new(),
        }
    }
    
    /// Share an anchor
    pub fn share(&mut self, anchor: SharedAnchor) -> SharedAnchorId {
        let id = anchor.id.clone();
        self.shared.insert(id.clone(), anchor);
        id
    }
    
    /// Unshare an anchor
    pub fn unshare(&mut self, id: &SharedAnchorId) -> Option<SharedAnchor> {
        self.shared.remove(id)
    }
    
    /// Get a shared anchor
    pub fn get_shared(&self, id: &SharedAnchorId) -> Option<&SharedAnchor> {
        self.shared.get(id)
    }
    
    /// Get mutable shared anchor
    pub fn get_shared_mut(&mut self, id: &SharedAnchorId) -> Option<&mut SharedAnchor> {
        self.shared.get_mut(id)
    }
    
    /// List all shared anchors
    pub fn list_shared(&self) -> impl Iterator<Item = &SharedAnchor> {
        self.shared.values()
    }
    
    /// Receive an anchor from another user
    pub fn receive(&mut self, anchor: SharedAnchor) {
        self.received.insert(anchor.id.clone(), anchor);
    }
    
    /// Update a received anchor
    pub fn update_received(&mut self, anchor: SharedAnchor) {
        if let Some(existing) = self.received.get_mut(&anchor.id) {
            // Only update if newer version
            if anchor.version > existing.version {
                *existing = anchor;
            }
        } else {
            self.received.insert(anchor.id.clone(), anchor);
        }
    }
    
    /// Remove a received anchor
    pub fn remove_received(&mut self, id: &SharedAnchorId) {
        self.received.remove(id);
        self.local_mapping.remove(id);
    }
    
    /// Get a received anchor
    pub fn get_received(&self, id: &SharedAnchorId) -> Option<&SharedAnchor> {
        self.received.get(id)
    }
    
    /// List all received anchors
    pub fn list_received(&self) -> impl Iterator<Item = &SharedAnchor> {
        self.received.values()
    }
    
    /// All visible anchors (shared + received)
    pub fn all_anchors(&self) -> impl Iterator<Item = &SharedAnchor> {
        self.shared.values().chain(self.received.values())
    }
    
    /// Set local anchor mapping
    pub fn set_local_mapping(&mut self, shared_id: SharedAnchorId, local_id: String) {
        self.local_mapping.insert(shared_id, local_id);
    }
    
    /// Get local anchor ID for a shared anchor
    pub fn get_local_id(&self, shared_id: &SharedAnchorId) -> Option<&String> {
        self.local_mapping.get(shared_id)
    }
    
    /// Find anchors near a position
    pub fn find_nearby(&self, position: &WorldPosition, radius: f32) -> Vec<&SharedAnchor> {
        self.all_anchors()
            .filter(|anchor| {
                if anchor.position.room_id != position.room_id {
                    return false;
                }
                
                let dx = anchor.position.local.x - position.local.x;
                let dy = anchor.position.local.y - position.local.y;
                let dz = anchor.position.local.z - position.local.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                
                dist <= radius
            })
            .collect()
    }
    
    /// Get anchors owned by a participant
    pub fn anchors_by_owner(&self, owner: &ParticipantId) -> Vec<&SharedAnchor> {
        self.all_anchors()
            .filter(|anchor| &anchor.owner_id == owner)
            .collect()
    }
}

impl Default for SharedAnchorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::world_coords::{LocalCoord, RoomId};
    
    fn test_position() -> WorldPosition {
        WorldPosition {
            room_id: Some(RoomId("test-room".to_string())),
            local: LocalCoord { x: 0.0, y: 0.0, z: 0.0 },
            gps: None,
            floor: 0,
            version: 1,
        }
    }
    
    #[test]
    fn test_shared_anchor_creation() {
        let owner = ParticipantId::new();
        let anchor = SharedAnchor::marker(owner.clone(), "Test".to_string(), test_position());
        
        assert_eq!(anchor.owner_id, owner);
        assert!(matches!(anchor.content, SharedAnchorContent::Marker));
    }
    
    #[test]
    fn test_permissions() {
        let owner = ParticipantId::new();
        let other = ParticipantId::new();
        
        let anchor = SharedAnchor::marker(owner.clone(), "Test".to_string(), test_position());
        
        // Owner can do everything
        assert!(anchor.can_view(&owner, false));
        assert!(anchor.can_edit(&owner, false));
        assert!(anchor.can_move(&owner, false));
        
        // Non-owner viewer
        assert!(anchor.can_view(&other, false)); // view is Everyone by default
        assert!(!anchor.can_edit(&other, false)); // edit is Owner only
        
        // Collaborator
        assert!(anchor.can_move(&other, true)); // move is Collaborators
    }
    
    #[test]
    fn test_version_increment() {
        let owner = ParticipantId::new();
        let mut anchor = SharedAnchor::marker(owner, "Test".to_string(), test_position());
        
        let v1 = anchor.version;
        anchor.set_position(test_position());
        
        assert!(anchor.version > v1);
    }
    
    #[test]
    fn test_manager_share() {
        let mut manager = SharedAnchorManager::new();
        let owner = ParticipantId::new();
        
        let anchor = SharedAnchor::marker(owner, "Test".to_string(), test_position());
        let id = manager.share(anchor);
        
        assert!(manager.get_shared(&id).is_some());
    }
    
    #[test]
    fn test_manager_receive() {
        let mut manager = SharedAnchorManager::new();
        let owner = ParticipantId::new();
        
        let anchor = SharedAnchor::marker(owner, "Test".to_string(), test_position());
        let id = anchor.id.clone();
        
        manager.receive(anchor);
        assert!(manager.get_received(&id).is_some());
    }
    
    #[test]
    fn test_version_conflict() {
        let mut manager = SharedAnchorManager::new();
        let owner = ParticipantId::new();
        
        let mut anchor = SharedAnchor::marker(owner.clone(), "Test".to_string(), test_position());
        anchor.version = 5;
        let id = anchor.id.clone();
        
        manager.receive(anchor);
        
        // Try to update with older version
        let mut old_anchor = SharedAnchor::marker(owner, "Old".to_string(), test_position());
        old_anchor.id = id.clone();
        old_anchor.version = 3;
        
        manager.update_received(old_anchor);
        
        // Should still have version 5
        assert_eq!(manager.get_received(&id).unwrap().version, 5);
    }
    
    #[test]
    fn test_find_nearby() {
        let mut manager = SharedAnchorManager::new();
        let owner = ParticipantId::new();
        
        let pos1 = WorldPosition {
            room_id: Some(RoomId("room".to_string())),
            local: LocalCoord { x: 0.0, y: 0.0, z: 0.0 },
            gps: None,
            floor: 0,
            version: 1,
        };
        let pos2 = WorldPosition {
            room_id: Some(RoomId("room".to_string())),
            local: LocalCoord { x: 1.0, y: 0.0, z: 0.0 },
            gps: None,
            floor: 0,
            version: 1,
        };
        let pos3 = WorldPosition {
            room_id: Some(RoomId("room".to_string())),
            local: LocalCoord { x: 10.0, y: 0.0, z: 0.0 },
            gps: None,
            floor: 0,
            version: 1,
        };
        
        manager.share(SharedAnchor::marker(owner.clone(), "A".to_string(), pos1.clone()));
        manager.share(SharedAnchor::marker(owner.clone(), "B".to_string(), pos2));
        manager.share(SharedAnchor::marker(owner, "C".to_string(), pos3));
        
        let nearby = manager.find_nearby(&pos1, 2.0);
        assert_eq!(nearby.len(), 2); // A and B are within 2m
    }
}
