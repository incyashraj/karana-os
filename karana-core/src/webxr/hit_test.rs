//! WebXR Hit Testing
//!
//! Enables ray-world intersection testing for placing virtual content
//! on real surfaces detected by the spatial system.

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

use super::{XRRigidTransform, XRVector3, XRHitTestResult};

/// Hit test source identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HitTestSourceId(u64);

impl HitTestSourceId {
    /// Create a new unique ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for HitTestSourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Hit test source - persistent ray cast configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitTestSource {
    /// Unique identifier
    pub id: HitTestSourceId,
    /// Ray definition
    pub ray: HitTestRay,
    /// Entity types to test against
    pub entity_types: Vec<HitTestEntityType>,
}

impl HitTestSource {
    /// Create a new hit test source
    pub fn new(ray: HitTestRay, entity_types: Vec<HitTestEntityType>) -> Self {
        Self {
            id: HitTestSourceId::new(),
            ray,
            entity_types,
        }
    }
    
    /// Create for viewer-centered testing
    pub fn from_viewer() -> Self {
        Self::new(
            HitTestRay::Viewer {
                offset: XRRigidTransform::identity(),
            },
            vec![HitTestEntityType::Plane],
        )
    }
    
    /// Create for transient input (tap/click)
    pub fn for_transient_input(input_source: u32) -> Self {
        Self::new(
            HitTestRay::TransientInput { source_id: input_source },
            vec![HitTestEntityType::Plane, HitTestEntityType::Point],
        )
    }
}

/// Hit test ray definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitTestRay {
    /// Ray from viewer (gaze)
    Viewer {
        /// Offset from viewer origin
        offset: XRRigidTransform,
    },
    /// Ray from controller/hand
    Controller {
        /// Controller profile
        profile: String,
        /// Ray offset from controller
        offset: XRRigidTransform,
    },
    /// Ray from transient input (touch/click)
    TransientInput {
        /// Input source identifier
        source_id: u32,
    },
    /// Arbitrary ray in space
    Space {
        /// Origin point
        origin: XRVector3,
        /// Direction vector
        direction: XRVector3,
    },
}

/// Entity types that can be hit tested
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitTestEntityType {
    /// Test against detected planes
    Plane,
    /// Test against point cloud
    Point,
    /// Test against mesh geometry
    Mesh,
}

/// Hit test result with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHitTestResult {
    /// Basic result
    pub result: XRHitTestResult,
    /// Distance from ray origin
    pub distance: f64,
    /// Type of entity hit
    pub entity_type: HitTestEntityType,
    /// Plane ID if hit a plane
    pub plane_id: Option<String>,
    /// Normal at hit point
    pub normal: XRVector3,
    /// Confidence (0-1)
    pub confidence: f64,
}

/// Hit test engine - integrates with spatial system for ray casting
pub struct HitTestEngine {
    /// Active hit test sources
    sources: Vec<HitTestSource>,
    /// Cached results from last frame
    results: Vec<(HitTestSourceId, Vec<DetailedHitTestResult>)>,
}

impl HitTestEngine {
    /// Create a new hit test engine
    pub fn new() -> Self {
        Self {
            sources: vec![],
            results: vec![],
        }
    }
    
    /// Add a hit test source
    pub fn add_source(&mut self, source: HitTestSource) -> HitTestSourceId {
        let id = source.id;
        self.sources.push(source);
        id
    }
    
    /// Remove a hit test source
    pub fn remove_source(&mut self, id: HitTestSourceId) -> bool {
        if let Some(idx) = self.sources.iter().position(|s| s.id == id) {
            self.sources.remove(idx);
            self.results.retain(|(rid, _)| *rid != id);
            true
        } else {
            false
        }
    }
    
    /// Get all active sources
    pub fn sources(&self) -> &[HitTestSource] {
        &self.sources
    }
    
    /// Get results for a source
    pub fn get_results(&self, source_id: HitTestSourceId) -> Option<&Vec<DetailedHitTestResult>> {
        self.results.iter()
            .find(|(id, _)| *id == source_id)
            .map(|(_, results)| results)
    }
    
    /// Update hit test results (called by system each frame)
    pub fn update_results(&mut self, results: Vec<(HitTestSourceId, Vec<DetailedHitTestResult>)>) {
        self.results = results;
    }
    
    /// Perform a single ray cast (synchronous)
    pub fn ray_cast(
        &self,
        origin: XRVector3,
        direction: XRVector3,
        max_distance: f64,
        entity_types: &[HitTestEntityType],
    ) -> Vec<DetailedHitTestResult> {
        // This would integrate with the spatial system
        // For now, return simulated results for testing
        
        // Simulate hitting a horizontal plane at y=0
        let dir_norm = direction.normalize();
        
        if dir_norm.y.abs() < 0.001 {
            // Ray parallel to ground
            return vec![];
        }
        
        // Calculate intersection with y=0 plane
        let t = -origin.y / dir_norm.y;
        
        if t <= 0.0 || t > max_distance {
            // Behind or too far
            return vec![];
        }
        
        if !entity_types.contains(&HitTestEntityType::Plane) {
            return vec![];
        }
        
        let hit_point = XRVector3 {
            x: origin.x + dir_norm.x * t,
            y: 0.0,
            z: origin.z + dir_norm.z * t,
        };
        
        vec![DetailedHitTestResult {
            result: XRHitTestResult {
                transform: XRRigidTransform::from_position(hit_point.x, hit_point.y, hit_point.z),
                pose: XRRigidTransform::from_position(hit_point.x, hit_point.y, hit_point.z),
            },
            distance: t,
            entity_type: HitTestEntityType::Plane,
            plane_id: Some("floor-plane".to_string()),
            normal: XRVector3::new(0.0, 1.0, 0.0),
            confidence: 0.95,
        }]
    }
    
    /// Clear all sources and results
    pub fn clear(&mut self) {
        self.sources.clear();
        self.results.clear();
    }
}

impl Default for HitTestEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hit_test_source_creation() {
        let source = HitTestSource::from_viewer();
        assert!(!source.entity_types.is_empty());
    }
    
    #[test]
    fn test_hit_test_engine() {
        let mut engine = HitTestEngine::new();
        
        let source = HitTestSource::from_viewer();
        let id = engine.add_source(source);
        
        assert_eq!(engine.sources().len(), 1);
        
        engine.remove_source(id);
        assert_eq!(engine.sources().len(), 0);
    }
    
    #[test]
    fn test_ray_cast_hit() {
        let engine = HitTestEngine::new();
        
        // Ray pointing down from above floor
        let results = engine.ray_cast(
            XRVector3::new(0.0, 1.5, 0.0),
            XRVector3::new(0.0, -1.0, 0.0),
            10.0,
            &[HitTestEntityType::Plane],
        );
        
        assert!(!results.is_empty());
        assert!((results[0].distance - 1.5).abs() < 0.01);
        assert!((results[0].result.transform.position.y).abs() < 0.01);
    }
    
    #[test]
    fn test_ray_cast_miss() {
        let engine = HitTestEngine::new();
        
        // Ray pointing away from floor
        let results = engine.ray_cast(
            XRVector3::new(0.0, 1.5, 0.0),
            XRVector3::new(0.0, 1.0, 0.0),
            10.0,
            &[HitTestEntityType::Plane],
        );
        
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_ray_cast_distance_limit() {
        let engine = HitTestEngine::new();
        
        // Ray that would hit but too far
        let results = engine.ray_cast(
            XRVector3::new(0.0, 100.0, 0.0),
            XRVector3::new(0.0, -1.0, 0.0),
            10.0, // Max 10m, but floor is 100m away
            &[HitTestEntityType::Plane],
        );
        
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_transient_input_source() {
        let source = HitTestSource::for_transient_input(0);
        assert!(source.entity_types.contains(&HitTestEntityType::Point));
    }
}
