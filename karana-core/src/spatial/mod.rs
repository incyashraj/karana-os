//! Spatial Persistence System for AR Glasses
//!
//! This module provides the foundation for persistent AR content that "sticks"
//! to real-world locations. Users can pin tabs, games, and apps to surfaces,
//! and they will reappear when returning to that location.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    SPATIAL PERSISTENCE SYSTEM                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
//! │  │ World Coord  │───→│ SpatialAnchor│───→│  ZK-Attested    │  │
//! │  │ (GPS+IMU+    │    │   Registry   │    │  Anchor Store   │  │
//! │  │  VisualSLAM) │    │              │    │  (On-Chain)     │  │
//! │  └──────────────┘    └──────────────┘    └──────────────────┘  │
//! │         │                   │                     │             │
//! │         │            ┌──────▼──────┐             │             │
//! │         │            │ Relocalize  │◄────────────┘             │
//! │         └───────────→│   Engine    │                           │
//! │                      │ (On Return) │                           │
//! │                      └─────────────┘                           │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Concepts
//!
//! - **SpatialAnchor**: A point in the real world where AR content is pinned
//! - **WorldPosition**: Combines GPS (outdoor) and SLAM (indoor) coordinates
//! - **ReferenceFrame**: Visual signature of a location for relocalization
//! - **Relocalization**: Finding anchors when returning to a location

pub mod anchor;
pub mod world_coords;
pub mod slam;
pub mod relocalize;
pub mod persistence;

pub use anchor::{
    SpatialAnchor, AnchorId, AnchorState, AnchorContent,
    SpatialAnchorRegistry, CreateAnchorRequest, Quaternion,
};
pub use world_coords::{
    WorldPosition, GpsCoord, LocalCoord, ReferenceFrame,
    CoordinateFusion, FusionConfig, RoomId, CoordinateTransform,
};
pub use slam::{
    SlamEngine, SlamConfig, SlamState, Pose, KeyFrame,
    VisualFeatures, SlamMap, VisualFeature, CameraPose,
    SlamSession, SlamManager, FeatureDescriptor,
};
pub use relocalize::{
    RelocalizeEngine, RelocatedAnchor, RelocalizeResult,
    RelocalizeConfig,
};
pub use persistence::{
    AnchorStore, AnchorProof, StoredAnchor,
    AnchorIntegrityProof, PersistenceMode, SyncStatus,
};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// SPATIAL SYSTEM MANAGER
// ============================================================================

/// Main manager for the spatial persistence system
pub struct SpatialSystem {
    /// Anchor registry
    pub anchors: Arc<RwLock<SpatialAnchorRegistry>>,
    /// SLAM engine for tracking
    pub slam: Arc<RwLock<SlamEngine>>,
    /// Coordinate fusion
    pub fusion: Arc<RwLock<CoordinateFusion>>,
    /// Relocalization engine
    pub relocalize: Arc<RwLock<RelocalizeEngine>>,
    /// Persistent anchor store
    pub store: Arc<RwLock<AnchorStore>>,
    /// Configuration
    config: SpatialConfig,
}

/// Configuration for the spatial system
#[derive(Debug, Clone)]
pub struct SpatialConfig {
    /// Maximum anchors per location
    pub max_anchors_per_location: usize,
    /// Relocalization confidence threshold
    pub relocalize_threshold: f32,
    /// Enable GPS for outdoor positioning
    pub enable_gps: bool,
    /// Enable visual SLAM
    pub enable_slam: bool,
    /// Anchor persistence path
    pub persistence_path: String,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            max_anchors_per_location: 50,
            relocalize_threshold: 0.7,
            enable_gps: true,
            enable_slam: true,
            persistence_path: "/tmp/karana/spatial".to_string(),
        }
    }
}

impl SpatialSystem {
    /// Create a new spatial system
    pub fn new(config: SpatialConfig) -> Self {
        Self {
            anchors: Arc::new(RwLock::new(SpatialAnchorRegistry::new())),
            slam: Arc::new(RwLock::new(SlamEngine::new(SlamConfig::default()))),
            fusion: Arc::new(RwLock::new(CoordinateFusion::new(FusionConfig::default()))),
            relocalize: Arc::new(RwLock::new(RelocalizeEngine::new(RelocalizeConfig::default()))),
            store: Arc::new(RwLock::new(AnchorStore::new(&config.persistence_path))),
            config,
        }
    }
    
    /// Create a new anchor at the current position
    pub async fn create_anchor(&self, content: AnchorContent) -> Result<SpatialAnchor> {
        // Get current position from SLAM
        let pose = {
            let slam = self.slam.read().await;
            slam.get_current_pose()
        };
        
        // Fuse with GPS if available
        let position = {
            let fusion = self.fusion.read().await;
            fusion.fuse_position(&pose)
        };
        
        // Create anchor
        let anchor = {
            let mut registry = self.anchors.write().await;
            registry.create(CreateAnchorRequest {
                position,
                content,
                ..Default::default()
            })?
        };
        
        // Store persistently
        {
            let mut store = self.store.write().await;
            store.save_async(&anchor).await?;
        }
        
        log::info!("[SPATIAL] Created anchor {} at {:?}", anchor.id, anchor.position);
        Ok(anchor)
    }
    
    /// Get all anchors at a location
    pub async fn get_anchors_at(&self, position: &WorldPosition, radius_m: f32) -> Vec<SpatialAnchor> {
        let registry = self.anchors.read().await;
        registry.find_nearby(position, radius_m)
    }
    
    /// Update SLAM with a new camera frame
    pub async fn process_frame(&self, frame: &CameraFrame) -> Result<Pose> {
        let mut slam = self.slam.write().await;
        slam.track(frame)
    }
    
    /// Try to relocalize (find known anchors in current view)
    pub async fn relocalize(&self, frame: &CameraFrame) -> Result<RelocalizeResult> {
        // Get all stored anchors
        let stored = {
            let store = self.store.read().await;
            store.get_all().await?
        };
        
        // Try to match current view with known anchors
        let mut relocalize = self.relocalize.write().await;
        let result = relocalize.attempt(frame, &stored).await?;
        
        // Update registry with relocated anchors
        if !result.matched_anchors.is_empty() {
            let mut registry = self.anchors.write().await;
            for anchor in &result.matched_anchors {
                registry.update_confidence(anchor.id, anchor.confidence);
            }
        }
        
        log::info!("[SPATIAL] Relocalized {} anchors", result.matched_anchors.len());
        Ok(result)
    }
    
    /// Get the current world position
    pub async fn get_current_position(&self) -> WorldPosition {
        let slam = self.slam.read().await;
        let pose = slam.get_current_pose();
        
        let fusion = self.fusion.read().await;
        fusion.fuse_position(&pose)
    }
    
    /// Delete an anchor
    pub async fn delete_anchor(&self, id: AnchorId) -> Result<()> {
        {
            let mut registry = self.anchors.write().await;
            registry.remove(id)?;
        }
        {
            let mut store = self.store.write().await;
            store.delete_async(id).await?;
        }
        log::info!("[SPATIAL] Deleted anchor {}", id);
        Ok(())
    }
}

/// Camera frame for SLAM processing
#[derive(Debug, Clone)]
pub struct CameraFrame {
    /// Raw image data (RGB)
    pub data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Camera intrinsics
    pub intrinsics: Option<CameraIntrinsics>,
}

/// Camera intrinsic parameters
#[derive(Debug, Clone, Copy)]
pub struct CameraIntrinsics {
    /// Focal length X
    pub fx: f32,
    /// Focal length Y
    pub fy: f32,
    /// Principal point X
    pub cx: f32,
    /// Principal point Y
    pub cy: f32,
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        // Typical smartphone/glasses camera
        Self {
            fx: 500.0,
            fy: 500.0,
            cx: 320.0,
            cy: 240.0,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spatial_system_creation() {
        let system = SpatialSystem::new(SpatialConfig::default());
        let position = system.get_current_position().await;
        
        // Default position should be origin
        assert_eq!(position.local.x, 0.0);
        assert_eq!(position.local.y, 0.0);
        assert_eq!(position.local.z, 0.0);
    }
    
    #[tokio::test]
    async fn test_anchor_creation() {
        let system = SpatialSystem::new(SpatialConfig::default());
        
        let content = AnchorContent::Text {
            text: "Test Note".to_string(),
        };
        
        let anchor = system.create_anchor(content).await.unwrap();
        
        assert!(anchor.confidence > 0.0);
        assert_eq!(anchor.state, AnchorState::Active);
    }
    
    #[tokio::test]
    async fn test_find_nearby_anchors() {
        let system = SpatialSystem::new(SpatialConfig::default());
        
        // Create a few anchors
        for i in 0..3 {
            let content = AnchorContent::Text {
                text: format!("Note {}", i),
            };
            system.create_anchor(content).await.unwrap();
        }
        
        // Find anchors near origin
        let position = WorldPosition::default();
        let nearby = system.get_anchors_at(&position, 10.0).await;
        
        assert_eq!(nearby.len(), 3);
    }
}
