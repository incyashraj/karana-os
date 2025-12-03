//! Spatial Anchor System
//!
//! A spatial anchor is a point in the real world where AR content is pinned.
//! Anchors persist across sessions and can be re-localized when returning
//! to a location.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::world_coords::WorldPosition;

// ============================================================================
// ANCHOR TYPES
// ============================================================================

/// Unique identifier for a spatial anchor
pub type AnchorId = u64;

/// A point in the real world where AR content is pinned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAnchor {
    /// Unique identifier
    pub id: AnchorId,
    /// World position (GPS + local SLAM coordinates)
    pub position: WorldPosition,
    /// Orientation as quaternion (x, y, z, w)
    pub orientation: Quaternion,
    /// Visual signature for relocalization
    pub visual_signature: VisualHash,
    /// Content hash for integrity verification
    pub content_hash: ContentHash,
    /// What's pinned at this anchor
    pub content: AnchorContent,
    /// Current state
    pub state: AnchorState,
    /// Tracking confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Creation timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// Owner DID (for ZK-proofs)
    pub owner_did: Option<String>,
    /// Human-readable label
    pub label: Option<String>,
}

/// Quaternion for 3D rotation
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    /// Identity quaternion (no rotation)
    pub fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
    
    /// Create from euler angles (radians)
    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        
        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }
}

/// Visual signature for relocalization matching
pub type VisualHash = [u8; 32];

/// Content hash for integrity verification
pub type ContentHash = [u8; 32];

/// State of a spatial anchor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorState {
    /// Anchor is active and being tracked
    Active,
    /// Anchor is visible but tracking quality is low
    Degraded,
    /// Anchor is not currently visible (needs relocalization)
    Lost,
    /// Anchor has been deleted/archived
    Archived,
}

/// Types of content that can be anchored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorContent {
    /// Simple text note
    Text { text: String },
    
    /// Web browser tab
    Browser { 
        url: String,
        title: Option<String>,
        scroll_position: f32,
    },
    
    /// Video player
    Video {
        url: String,
        position_secs: f32,
        is_playing: bool,
    },
    
    /// Code editor
    CodeEditor {
        file_path: String,
        cursor_line: u32,
        language: String,
    },
    
    /// Game world
    Game {
        game_id: String,
        state_hash: [u8; 32],
    },
    
    /// 3D model/object
    Model3D {
        model_url: String,
        scale: f32,
    },
    
    /// Navigation waypoint
    Waypoint {
        destination: String,
        step_number: u32,
    },
    
    /// Custom AR app
    Custom {
        app_id: String,
        state: Vec<u8>,
    },
}

impl AnchorContent {
    /// Get the type name for this content
    pub fn type_name(&self) -> &'static str {
        match self {
            AnchorContent::Text { .. } => "text",
            AnchorContent::Browser { .. } => "browser",
            AnchorContent::Video { .. } => "video",
            AnchorContent::CodeEditor { .. } => "code",
            AnchorContent::Game { .. } => "game",
            AnchorContent::Model3D { .. } => "model",
            AnchorContent::Waypoint { .. } => "waypoint",
            AnchorContent::Custom { .. } => "custom",
        }
    }
    
    /// Compute content hash for integrity
    pub fn hash(&self) -> ContentHash {
        use sha2::{Sha256, Digest};
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        let result = Sha256::digest(&serialized);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

// ============================================================================
// ANCHOR REGISTRY
// ============================================================================

/// Registry for managing spatial anchors
pub struct SpatialAnchorRegistry {
    /// All anchors indexed by ID
    anchors: HashMap<AnchorId, SpatialAnchor>,
    /// Next anchor ID
    next_id: AnchorId,
    /// Spatial index for fast lookups
    spatial_index: SpatialIndex,
}

/// Simple spatial index using grid cells
struct SpatialIndex {
    /// Grid cell size in meters
    cell_size: f32,
    /// Anchors in each cell
    cells: HashMap<(i32, i32, i32), Vec<AnchorId>>,
}

impl SpatialIndex {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }
    
    fn cell_key(&self, pos: &WorldPosition) -> (i32, i32, i32) {
        (
            (pos.local.x / self.cell_size).floor() as i32,
            (pos.local.y / self.cell_size).floor() as i32,
            (pos.local.z / self.cell_size).floor() as i32,
        )
    }
    
    fn insert(&mut self, id: AnchorId, pos: &WorldPosition) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_default().push(id);
    }
    
    fn remove(&mut self, id: AnchorId, pos: &WorldPosition) {
        let key = self.cell_key(pos);
        if let Some(cell) = self.cells.get_mut(&key) {
            cell.retain(|&i| i != id);
        }
    }
    
    fn find_nearby(&self, pos: &WorldPosition, radius: f32) -> Vec<AnchorId> {
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;
        let center = self.cell_key(pos);
        
        let mut results = Vec::new();
        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                for dz in -cells_to_check..=cells_to_check {
                    let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                    if let Some(cell) = self.cells.get(&key) {
                        results.extend(cell.iter().copied());
                    }
                }
            }
        }
        results
    }
}

/// Request to create a new anchor
#[derive(Debug, Clone, Default)]
pub struct CreateAnchorRequest {
    /// Position in the world
    pub position: WorldPosition,
    /// Orientation (defaults to identity)
    pub orientation: Option<Quaternion>,
    /// Content to anchor
    pub content: AnchorContent,
    /// Visual signature (computed if not provided)
    pub visual_signature: Option<VisualHash>,
    /// Human-readable label
    pub label: Option<String>,
    /// Owner DID
    pub owner_did: Option<String>,
}

impl Default for AnchorContent {
    fn default() -> Self {
        AnchorContent::Text { text: String::new() }
    }
}

impl SpatialAnchorRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
            next_id: 1,
            spatial_index: SpatialIndex::new(1.0), // 1 meter grid
        }
    }
    
    /// Create a new anchor
    pub fn create(&mut self, request: CreateAnchorRequest) -> Result<SpatialAnchor> {
        let id = self.next_id;
        self.next_id += 1;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let anchor = SpatialAnchor {
            id,
            position: request.position.clone(),
            orientation: request.orientation.unwrap_or(Quaternion::identity()),
            visual_signature: request.visual_signature.unwrap_or([0u8; 32]),
            content_hash: request.content.hash(),
            content: request.content,
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: now,
            updated_at: now,
            owner_did: request.owner_did,
            label: request.label,
        };
        
        self.spatial_index.insert(id, &anchor.position);
        self.anchors.insert(id, anchor.clone());
        
        Ok(anchor)
    }
    
    /// Get an anchor by ID
    pub fn get(&self, id: AnchorId) -> Option<&SpatialAnchor> {
        self.anchors.get(&id)
    }
    
    /// Get mutable anchor by ID
    pub fn get_mut(&mut self, id: AnchorId) -> Option<&mut SpatialAnchor> {
        self.anchors.get_mut(&id)
    }
    
    /// Find anchors near a position
    pub fn find_nearby(&self, position: &WorldPosition, radius_m: f32) -> Vec<SpatialAnchor> {
        let candidate_ids = self.spatial_index.find_nearby(position, radius_m);
        
        candidate_ids
            .iter()
            .filter_map(|&id| self.anchors.get(&id))
            .filter(|anchor| {
                let dist = anchor.position.distance_to(position);
                dist <= radius_m
            })
            .cloned()
            .collect()
    }
    
    /// Update anchor confidence
    pub fn update_confidence(&mut self, id: AnchorId, confidence: f32) {
        if let Some(anchor) = self.anchors.get_mut(&id) {
            anchor.confidence = confidence.clamp(0.0, 1.0);
            anchor.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Update state based on confidence
            anchor.state = if confidence > 0.7 {
                AnchorState::Active
            } else if confidence > 0.3 {
                AnchorState::Degraded
            } else {
                AnchorState::Lost
            };
        }
    }
    
    /// Update anchor position (for drift correction)
    pub fn update_position(&mut self, id: AnchorId, new_position: WorldPosition) -> Result<()> {
        let anchor = self.anchors.get_mut(&id).ok_or(anyhow!("Anchor not found"))?;
        
        // Update spatial index
        self.spatial_index.remove(id, &anchor.position);
        self.spatial_index.insert(id, &new_position);
        
        anchor.position = new_position;
        anchor.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(())
    }
    
    /// Remove an anchor
    pub fn remove(&mut self, id: AnchorId) -> Result<SpatialAnchor> {
        let anchor = self.anchors.remove(&id).ok_or(anyhow!("Anchor not found"))?;
        self.spatial_index.remove(id, &anchor.position);
        Ok(anchor)
    }
    
    /// Get all active anchors
    pub fn get_active(&self) -> Vec<&SpatialAnchor> {
        self.anchors
            .values()
            .filter(|a| a.state == AnchorState::Active || a.state == AnchorState::Degraded)
            .collect()
    }
    
    /// Get anchor count
    pub fn count(&self) -> usize {
        self.anchors.len()
    }
    
    /// Get all anchors
    pub fn get_all(&self) -> Vec<&SpatialAnchor> {
        self.anchors.values().collect()
    }
}

impl Default for SpatialAnchorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::world_coords::LocalCoord;
    
    #[test]
    fn test_anchor_creation() {
        let mut registry = SpatialAnchorRegistry::new();
        
        let request = CreateAnchorRequest {
            position: WorldPosition::default(),
            content: AnchorContent::Text { text: "Test".to_string() },
            label: Some("Test Note".to_string()),
            ..Default::default()
        };
        
        let anchor = registry.create(request).unwrap();
        
        assert_eq!(anchor.id, 1);
        assert_eq!(anchor.state, AnchorState::Active);
        assert_eq!(anchor.confidence, 1.0);
        assert!(anchor.label.as_ref().unwrap().contains("Test"));
    }
    
    #[test]
    fn test_find_nearby() {
        let mut registry = SpatialAnchorRegistry::new();
        
        // Create anchors at different positions
        for i in 0..5 {
            let request = CreateAnchorRequest {
                position: WorldPosition {
                    local: LocalCoord { x: i as f32, y: 0.0, z: 0.0 },
                    ..Default::default()
                },
                content: AnchorContent::Text { text: format!("Note {}", i) },
                ..Default::default()
            };
            registry.create(request).unwrap();
        }
        
        // Find anchors near origin
        let origin = WorldPosition::default();
        let nearby = registry.find_nearby(&origin, 2.5);
        
        // Should find anchors at x=0, 1, 2
        assert_eq!(nearby.len(), 3);
    }
    
    #[test]
    fn test_confidence_update() {
        let mut registry = SpatialAnchorRegistry::new();
        
        let request = CreateAnchorRequest {
            position: WorldPosition::default(),
            content: AnchorContent::Text { text: "Test".to_string() },
            ..Default::default()
        };
        
        let anchor = registry.create(request).unwrap();
        assert_eq!(anchor.state, AnchorState::Active);
        
        // Degrade confidence
        registry.update_confidence(1, 0.5);
        assert_eq!(registry.get(1).unwrap().state, AnchorState::Degraded);
        
        // Lose tracking
        registry.update_confidence(1, 0.1);
        assert_eq!(registry.get(1).unwrap().state, AnchorState::Lost);
    }
    
    #[test]
    fn test_anchor_removal() {
        let mut registry = SpatialAnchorRegistry::new();
        
        let request = CreateAnchorRequest {
            position: WorldPosition::default(),
            content: AnchorContent::Text { text: "Test".to_string() },
            ..Default::default()
        };
        
        registry.create(request).unwrap();
        assert_eq!(registry.count(), 1);
        
        registry.remove(1).unwrap();
        assert_eq!(registry.count(), 0);
    }
    
    #[test]
    fn test_quaternion() {
        let identity = Quaternion::identity();
        assert_eq!(identity.w, 1.0);
        assert_eq!(identity.x, 0.0);
        
        let rotated = Quaternion::from_euler(0.0, 0.0, std::f32::consts::PI / 2.0);
        assert!(rotated.w < 1.0); // Should have changed
    }
    
    #[test]
    fn test_content_hash() {
        let content1 = AnchorContent::Text { text: "Hello".to_string() };
        let content2 = AnchorContent::Text { text: "Hello".to_string() };
        let content3 = AnchorContent::Text { text: "World".to_string() };
        
        assert_eq!(content1.hash(), content2.hash());
        assert_ne!(content1.hash(), content3.hash());
    }
}
