//! Collaborative Tabs
//!
//! Enables sharing AR tabs between users for collaborative work.
//! Multiple users can view and interact with the same floating tabs.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::WorldPosition;
use super::ParticipantId;

/// Collaborative tab wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeTab {
    /// Shared tab ID
    pub id: CollabTabId,
    /// Underlying tab info
    pub tab_info: SharedTabMetadata,
    /// Owner
    pub owner_id: ParticipantId,
    /// Collaborators with edit access
    pub collaborators: Vec<ParticipantId>,
    /// Current viewers
    pub viewers: Vec<ParticipantId>,
    /// Interaction mode
    pub mode: CollabMode,
    /// Edit lock
    pub edit_lock: Option<EditLock>,
    /// Cursor positions
    pub cursors: HashMap<ParticipantId, CursorState>,
    /// Annotations
    pub annotations: Vec<Annotation>,
    /// Version
    pub version: u64,
    /// Created at
    pub created_at: u64,
    /// Modified at
    pub modified_at: u64,
}

/// Collaborative tab ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollabTabId(pub String);

impl CollabTabId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for CollabTabId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CollabTabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Shared tab metadata (serializable version of Tab info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedTabMetadata {
    /// Original tab ID (local)
    pub local_tab_id: String,
    /// Tab title
    pub title: String,
    /// Tab type
    pub tab_type: String,
    /// URL if browser tab
    pub url: Option<String>,
    /// Position in world space
    pub position: WorldPosition,
    /// Size in meters
    pub width: f32,
    pub height: f32,
    /// Visibility
    pub visible: bool,
}

/// Collaboration mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollabMode {
    /// Everyone can view, owner controls
    Presentation,
    /// Everyone can interact independently
    FreeInteract,
    /// Turn-based editing
    TurnBased,
    /// Synchronized scrolling/viewing
    SyncView,
    /// Real-time collaborative editing
    CoEdit,
}

/// Edit lock for turn-based or exclusive editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditLock {
    /// Who holds the lock
    pub holder: ParticipantId,
    /// When acquired
    pub acquired_at: u64,
    /// Auto-release after (ms)
    pub expires_after: Option<u64>,
}

impl EditLock {
    pub fn new(holder: ParticipantId) -> Self {
        Self {
            holder,
            acquired_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            expires_after: Some(60000), // 1 minute default
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_after {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            now.saturating_sub(self.acquired_at) > expires
        } else {
            false
        }
    }
}

/// Cursor state for a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    /// Cursor position (normalized 0-1)
    pub x: f32,
    pub y: f32,
    /// Is clicking/selecting
    pub active: bool,
    /// Selection range (if any)
    pub selection: Option<SelectionRange>,
    /// Last update
    pub updated_at: u64,
}

/// Text selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: usize,
    pub end: usize,
}

/// Annotation on a shared tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation ID
    pub id: String,
    /// Creator
    pub author_id: ParticipantId,
    /// Author name
    pub author_name: String,
    /// Annotation type
    pub annotation_type: AnnotationType,
    /// Position (normalized 0-1)
    pub x: f32,
    pub y: f32,
    /// Content
    pub content: String,
    /// Color
    pub color: String,
    /// Created at
    pub created_at: u64,
}

/// Annotation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    /// Simple comment
    Comment,
    /// Highlight region
    Highlight { width: f32, height: f32 },
    /// Drawing/sketch
    Drawing { points: Vec<(f32, f32)> },
    /// Pin marker
    Pin,
    /// Arrow pointing somewhere
    Arrow { end_x: f32, end_y: f32 },
}

impl CollaborativeTab {
    /// Create a new collaborative tab
    pub fn new(owner_id: ParticipantId, tab_info: SharedTabMetadata) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id: CollabTabId::new(),
            tab_info,
            owner_id,
            collaborators: vec![],
            viewers: vec![],
            mode: CollabMode::FreeInteract,
            edit_lock: None,
            cursors: HashMap::new(),
            annotations: vec![],
            version: 1,
            created_at: now,
            modified_at: now,
        }
    }
    
    /// Add a collaborator
    pub fn add_collaborator(&mut self, participant: ParticipantId) {
        if !self.collaborators.contains(&participant) {
            self.collaborators.push(participant);
            self.touch();
        }
    }
    
    /// Remove a collaborator
    pub fn remove_collaborator(&mut self, participant: &ParticipantId) {
        self.collaborators.retain(|p| p != participant);
        self.touch();
    }
    
    /// Add a viewer
    pub fn add_viewer(&mut self, participant: ParticipantId) {
        if !self.viewers.contains(&participant) {
            self.viewers.push(participant);
        }
    }
    
    /// Remove a viewer
    pub fn remove_viewer(&mut self, participant: &ParticipantId) {
        self.viewers.retain(|p| p != participant);
        self.cursors.remove(participant);
    }
    
    /// Check if participant can edit
    pub fn can_edit(&self, participant: &ParticipantId) -> bool {
        if &self.owner_id == participant {
            return true;
        }
        
        if self.collaborators.contains(participant) {
            // Check edit lock
            if let Some(lock) = &self.edit_lock {
                if lock.is_expired() {
                    return true;
                }
                return &lock.holder == participant;
            }
            return true;
        }
        
        false
    }
    
    /// Try to acquire edit lock
    pub fn acquire_lock(&mut self, participant: ParticipantId) -> bool {
        if let Some(lock) = &self.edit_lock {
            if !lock.is_expired() && lock.holder != participant {
                return false;
            }
        }
        
        self.edit_lock = Some(EditLock::new(participant));
        self.touch();
        true
    }
    
    /// Release edit lock
    pub fn release_lock(&mut self, participant: &ParticipantId) -> bool {
        if let Some(lock) = &self.edit_lock {
            if &lock.holder == participant {
                self.edit_lock = None;
                self.touch();
                return true;
            }
        }
        false
    }
    
    /// Update cursor position
    pub fn update_cursor(&mut self, participant: ParticipantId, x: f32, y: f32, active: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let cursor = self.cursors.entry(participant).or_insert(CursorState {
            x: 0.0,
            y: 0.0,
            active: false,
            selection: None,
            updated_at: now,
        });
        
        cursor.x = x;
        cursor.y = y;
        cursor.active = active;
        cursor.updated_at = now;
    }
    
    /// Add an annotation
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
        self.touch();
    }
    
    /// Remove an annotation
    pub fn remove_annotation(&mut self, id: &str, participant: &ParticipantId) -> bool {
        if let Some(idx) = self.annotations.iter().position(|a| a.id == id) {
            let annotation = &self.annotations[idx];
            // Only owner or author can remove
            if &annotation.author_id == participant || &self.owner_id == participant {
                self.annotations.remove(idx);
                self.touch();
                return true;
            }
        }
        false
    }
    
    /// Get all active cursors
    pub fn active_cursors(&self) -> impl Iterator<Item = (&ParticipantId, &CursorState)> {
        self.cursors.iter()
    }
    
    /// Total participant count
    pub fn participant_count(&self) -> usize {
        1 + self.collaborators.len() + self.viewers.len()
    }
    
    fn touch(&mut self) {
        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.version += 1;
    }
}

/// Collaborative tab manager
pub struct CollaborativeTabManager {
    /// Tabs we're sharing
    shared: HashMap<CollabTabId, CollaborativeTab>,
    /// Tabs others are sharing with us
    received: HashMap<CollabTabId, CollaborativeTab>,
}

impl CollaborativeTabManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            shared: HashMap::new(),
            received: HashMap::new(),
        }
    }
    
    /// Share a tab
    pub fn share(&mut self, tab: CollaborativeTab) -> CollabTabId {
        let id = tab.id.clone();
        self.shared.insert(id.clone(), tab);
        id
    }
    
    /// Unshare a tab
    pub fn unshare(&mut self, id: &CollabTabId) -> Option<CollaborativeTab> {
        self.shared.remove(id)
    }
    
    /// Get shared tab
    pub fn get_shared(&self, id: &CollabTabId) -> Option<&CollaborativeTab> {
        self.shared.get(id)
    }
    
    /// Get shared tab mutably
    pub fn get_shared_mut(&mut self, id: &CollabTabId) -> Option<&mut CollaborativeTab> {
        self.shared.get_mut(id)
    }
    
    /// List shared tabs
    pub fn list_shared(&self) -> impl Iterator<Item = &CollaborativeTab> {
        self.shared.values()
    }
    
    /// Receive a shared tab
    pub fn receive(&mut self, tab: CollaborativeTab) {
        self.received.insert(tab.id.clone(), tab);
    }
    
    /// Update received tab
    pub fn update_received(&mut self, tab: CollaborativeTab) {
        if let Some(existing) = self.received.get_mut(&tab.id) {
            if tab.version > existing.version {
                *existing = tab;
            }
        } else {
            self.received.insert(tab.id.clone(), tab);
        }
    }
    
    /// Remove received tab
    pub fn remove_received(&mut self, id: &CollabTabId) {
        self.received.remove(id);
    }
    
    /// Get received tab
    pub fn get_received(&self, id: &CollabTabId) -> Option<&CollaborativeTab> {
        self.received.get(id)
    }
    
    /// Get received tab mutably
    pub fn get_received_mut(&mut self, id: &CollabTabId) -> Option<&mut CollaborativeTab> {
        self.received.get_mut(id)
    }
    
    /// List received tabs
    pub fn list_received(&self) -> impl Iterator<Item = &CollaborativeTab> {
        self.received.values()
    }
    
    /// All visible tabs
    pub fn all_tabs(&self) -> impl Iterator<Item = &CollaborativeTab> {
        self.shared.values().chain(self.received.values())
    }
}

impl Default for CollaborativeTabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::world_coords::{LocalCoord, RoomId};
    
    fn test_metadata() -> SharedTabMetadata {
        SharedTabMetadata {
            local_tab_id: "tab-123".to_string(),
            title: "Test Tab".to_string(),
            tab_type: "browser".to_string(),
            url: Some("https://example.com".to_string()),
            position: WorldPosition {
                room_id: Some(RoomId("room".to_string())),
                local: LocalCoord::default(),
                gps: None,
                floor: 0,
                version: 1,
            },
            width: 1.0,
            height: 0.75,
            visible: true,
        }
    }
    
    #[test]
    fn test_collab_tab_creation() {
        let owner = ParticipantId::new();
        let tab = CollaborativeTab::new(owner.clone(), test_metadata());
        
        assert_eq!(tab.owner_id, owner);
        assert!(tab.can_edit(&owner));
    }
    
    #[test]
    fn test_collaborators() {
        let owner = ParticipantId::new();
        let collaborator = ParticipantId::new();
        let viewer = ParticipantId::new();
        
        let mut tab = CollaborativeTab::new(owner.clone(), test_metadata());
        tab.add_collaborator(collaborator.clone());
        tab.add_viewer(viewer.clone());
        
        assert!(tab.can_edit(&owner));
        assert!(tab.can_edit(&collaborator));
        assert!(!tab.can_edit(&viewer));
    }
    
    #[test]
    fn test_edit_lock() {
        let owner = ParticipantId::new();
        let collab = ParticipantId::new();
        
        let mut tab = CollaborativeTab::new(owner.clone(), test_metadata());
        tab.add_collaborator(collab.clone());
        
        // Both can edit initially
        assert!(tab.can_edit(&collab));
        
        // Owner acquires lock
        assert!(tab.acquire_lock(owner.clone()));
        
        // Collaborator can't edit while locked
        assert!(!tab.can_edit(&collab));
        
        // Owner releases lock
        assert!(tab.release_lock(&owner));
        assert!(tab.can_edit(&collab));
    }
    
    #[test]
    fn test_cursors() {
        let owner = ParticipantId::new();
        let mut tab = CollaborativeTab::new(owner.clone(), test_metadata());
        
        tab.update_cursor(owner.clone(), 0.5, 0.5, true);
        
        let cursor = tab.cursors.get(&owner).unwrap();
        assert_eq!(cursor.x, 0.5);
        assert!(cursor.active);
    }
    
    #[test]
    fn test_annotations() {
        let owner = ParticipantId::new();
        let mut tab = CollaborativeTab::new(owner.clone(), test_metadata());
        
        let annotation = Annotation {
            id: "ann-1".to_string(),
            author_id: owner.clone(),
            author_name: "Owner".to_string(),
            annotation_type: AnnotationType::Comment,
            x: 0.5,
            y: 0.5,
            content: "Test comment".to_string(),
            color: "#FF0000".to_string(),
            created_at: 0,
        };
        
        tab.add_annotation(annotation);
        assert_eq!(tab.annotations.len(), 1);
        
        // Owner can remove their own annotation
        assert!(tab.remove_annotation("ann-1", &owner));
        assert!(tab.annotations.is_empty());
    }
    
    #[test]
    fn test_manager() {
        let mut manager = CollaborativeTabManager::new();
        let owner = ParticipantId::new();
        
        let tab = CollaborativeTab::new(owner, test_metadata());
        let id = manager.share(tab);
        
        assert!(manager.get_shared(&id).is_some());
    }
}
