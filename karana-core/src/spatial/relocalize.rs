//! Relocalization Engine
//!
//! Finds known anchors when returning to a previously visited location.
//! Uses visual feature matching and spatial constraints.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::anchor::{AnchorId, SpatialAnchor};
use super::slam::VisualFeature;
use super::world_coords::{LocalCoord, WorldPosition};
use super::CameraFrame;

// ============================================================================
// RELOCALIZATION CONFIG
// ============================================================================

/// Configuration for relocalization
#[derive(Debug, Clone)]
pub struct RelocalizeConfig {
    /// Minimum confidence to consider a match
    pub min_confidence: f32,
    /// Maximum distance for spatial constraints (meters)
    pub max_distance: f32,
    /// Number of features to extract
    pub num_features: usize,
    /// Matching ratio threshold (Lowe's ratio test)
    pub ratio_threshold: f32,
}

impl Default for RelocalizeConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_distance: 20.0,
            num_features: 500,
            ratio_threshold: 0.75,
        }
    }
}

// ============================================================================
// RELOCALIZATION RESULT
// ============================================================================

/// Result of a relocalization attempt
#[derive(Debug, Clone)]
pub struct RelocalizeResult {
    /// Anchors that were successfully matched
    pub matched_anchors: Vec<RelocatedAnchor>,
    /// Total anchors attempted
    pub total_attempted: usize,
    /// Overall confidence
    pub overall_confidence: f32,
    /// Estimated camera pose
    pub estimated_pose: Option<WorldPosition>,
}

/// An anchor that was successfully relocated
#[derive(Debug, Clone)]
pub struct RelocatedAnchor {
    /// Anchor ID
    pub id: AnchorId,
    /// Confidence of the match (0.0 - 1.0)
    pub confidence: f32,
    /// Position error estimate (meters)
    pub position_error: f32,
    /// Number of feature matches
    pub feature_matches: usize,
}

// ============================================================================
// RELOCALIZATION ENGINE
// ============================================================================

/// Engine for relocalization
pub struct RelocalizeEngine {
    /// Configuration
    config: RelocalizeConfig,
    /// Cached feature descriptors for known anchors
    anchor_features: HashMap<AnchorId, Vec<VisualFeature>>,
    /// Last successful relocalization
    last_reloc: Option<RelocalizeResult>,
}

impl RelocalizeEngine {
    /// Create a new relocalization engine
    pub fn new(config: RelocalizeConfig) -> Self {
        Self {
            config,
            anchor_features: HashMap::new(),
            last_reloc: None,
        }
    }
    
    /// Register features for an anchor
    pub fn register_anchor(&mut self, id: AnchorId, features: Vec<VisualFeature>) {
        self.anchor_features.insert(id, features);
    }
    
    /// Unregister an anchor
    pub fn unregister_anchor(&mut self, id: AnchorId) {
        self.anchor_features.remove(&id);
    }
    
    /// Attempt relocalization with current frame
    pub async fn attempt(
        &mut self,
        frame: &CameraFrame,
        stored_anchors: &[SpatialAnchor],
    ) -> Result<RelocalizeResult> {
        // Extract features from current frame
        let current_features = self.extract_features(frame)?;
        
        let mut matched = Vec::new();
        let mut total_confidence = 0.0;
        
        for anchor in stored_anchors {
            // Get stored features for this anchor
            let stored_features = match self.anchor_features.get(&anchor.id) {
                Some(f) => f,
                None => continue,
            };
            
            // Match features
            let matches = self.match_features(&current_features, stored_features);
            
            if matches.len() < 10 {
                continue; // Not enough matches
            }
            
            // Compute confidence based on match quality
            let confidence = self.compute_confidence(&matches, stored_features.len());
            
            if confidence >= self.config.min_confidence {
                // Estimate position error from match distribution
                let position_error = self.estimate_position_error(&matches);
                
                matched.push(RelocatedAnchor {
                    id: anchor.id,
                    confidence,
                    position_error,
                    feature_matches: matches.len(),
                });
                
                total_confidence += confidence;
            }
        }
        
        let overall_confidence = if !matched.is_empty() {
            total_confidence / matched.len() as f32
        } else {
            0.0
        };
        
        // Estimate camera pose from matches
        let estimated_pose = if matched.len() >= 3 {
            self.estimate_pose(&matched, stored_anchors)
        } else {
            None
        };
        
        let result = RelocalizeResult {
            matched_anchors: matched,
            total_attempted: stored_anchors.len(),
            overall_confidence,
            estimated_pose,
        };
        
        self.last_reloc = Some(result.clone());
        
        Ok(result)
    }
    
    /// Extract visual features from a camera frame
    fn extract_features(&self, frame: &CameraFrame) -> Result<Vec<VisualFeature>> {
        // In production, this would use ORB, SIFT, or similar
        // For now, return mock features based on frame data
        
        let mut features = Vec::new();
        let step = (frame.width * frame.height) as usize / self.config.num_features;
        
        for i in 0..self.config.num_features {
            let idx = (i * step).min(frame.data.len().saturating_sub(32));
            let x = (i % frame.width as usize) as f32 / frame.width as f32;
            let y = (i / frame.width as usize) as f32 / frame.height as f32;
            
            let mut descriptor = [0u8; 32];
            if idx + 32 <= frame.data.len() {
                descriptor.copy_from_slice(&frame.data[idx..idx + 32]);
            }
            
            features.push(VisualFeature {
                image_pos: (x, y),
                world_pos: None,
                descriptor: super::slam::FeatureDescriptor { data: descriptor },
                track_length: 1,
                is_landmark: false,
            });
        }
        
        Ok(features)
    }
    
    /// Match features between current and stored
    fn match_features(
        &self,
        current: &[VisualFeature],
        stored: &[VisualFeature],
    ) -> Vec<FeatureMatch> {
        let mut matches = Vec::new();
        
        for (curr_idx, curr) in current.iter().enumerate() {
            let mut best_dist = u32::MAX;
            let mut second_dist = u32::MAX;
            let mut best_idx = 0;
            
            for (stor_idx, stor) in stored.iter().enumerate() {
                let dist = curr.descriptor.distance_to(&stor.descriptor);
                
                if dist < best_dist {
                    second_dist = best_dist;
                    best_dist = dist;
                    best_idx = stor_idx;
                } else if dist < second_dist {
                    second_dist = dist;
                }
            }
            
            // Lowe's ratio test
            if second_dist > 0 {
                let ratio = best_dist as f32 / second_dist as f32;
                if ratio < self.config.ratio_threshold {
                    matches.push(FeatureMatch {
                        current_idx: curr_idx,
                        stored_idx: best_idx,
                        distance: best_dist,
                    });
                }
            }
        }
        
        matches
    }
    
    /// Compute confidence from matches
    fn compute_confidence(&self, matches: &[FeatureMatch], total_stored: usize) -> f32 {
        if total_stored == 0 {
            return 0.0;
        }
        
        // Inlier ratio
        let inlier_ratio = matches.len() as f32 / total_stored as f32;
        
        // Average match quality
        let avg_distance: f32 = matches.iter().map(|m| m.distance as f32).sum::<f32>()
            / matches.len().max(1) as f32;
        
        // Quality score (lower distance = better)
        let quality_score = 1.0 - (avg_distance / 256.0).min(1.0);
        
        // Combined confidence
        (inlier_ratio * 0.5 + quality_score * 0.5).clamp(0.0, 1.0)
    }
    
    /// Estimate position error from match distribution
    fn estimate_position_error(&self, matches: &[FeatureMatch]) -> f32 {
        // Simple heuristic: more matches = lower error
        let base_error = 0.5; // 0.5 meters base error
        let match_factor = 1.0 / (1.0 + matches.len() as f32 / 50.0);
        base_error * match_factor
    }
    
    /// Estimate camera pose from matched anchors
    fn estimate_pose(
        &self,
        matched: &[RelocatedAnchor],
        stored: &[SpatialAnchor],
    ) -> Option<WorldPosition> {
        if matched.is_empty() {
            return None;
        }
        
        // Weighted average of anchor positions
        let mut total_weight = 0.0;
        let mut avg_pos = LocalCoord::default();
        
        for reloc in matched {
            if let Some(anchor) = stored.iter().find(|a| a.id == reloc.id) {
                let weight = reloc.confidence;
                avg_pos.x += anchor.position.local.x * weight;
                avg_pos.y += anchor.position.local.y * weight;
                avg_pos.z += anchor.position.local.z * weight;
                total_weight += weight;
            }
        }
        
        if total_weight > 0.0 {
            avg_pos.x /= total_weight;
            avg_pos.y /= total_weight;
            avg_pos.z /= total_weight;
            
            Some(WorldPosition {
                local: avg_pos,
                room_id: None,
                gps: None,
                floor: 0,
                version: 1,
            })
        } else {
            None
        }
    }
    
    /// Get last relocalization result
    pub fn get_last_result(&self) -> Option<&RelocalizeResult> {
        self.last_reloc.as_ref()
    }
}

impl Default for RelocalizeEngine {
    fn default() -> Self {
        Self::new(RelocalizeConfig::default())
    }
}

/// A feature match between current and stored frames
#[derive(Debug, Clone)]
struct FeatureMatch {
    current_idx: usize,
    stored_idx: usize,
    distance: u32,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::anchor::{AnchorContent, AnchorState, Quaternion};
    
    fn make_test_frame(seed: u8) -> CameraFrame {
        CameraFrame {
            data: vec![seed; 640 * 480 * 3],
            width: 640,
            height: 480,
            timestamp: 0,
            intrinsics: None,
        }
    }
    
    fn make_test_anchor(id: AnchorId) -> SpatialAnchor {
        SpatialAnchor {
            id,
            position: WorldPosition::from_local(id as f32, 0.0, 0.0),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: format!("Anchor {}", id) },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        }
    }
    
    #[tokio::test]
    async fn test_relocalize_engine_creation() {
        let engine = RelocalizeEngine::new(RelocalizeConfig::default());
        assert!(engine.get_last_result().is_none());
    }
    
    #[tokio::test]
    async fn test_feature_extraction() {
        let engine = RelocalizeEngine::new(RelocalizeConfig::default());
        let frame = make_test_frame(42);
        
        let features = engine.extract_features(&frame).unwrap();
        assert!(!features.is_empty());
    }
    
    #[tokio::test]
    async fn test_relocalize_attempt() {
        let mut engine = RelocalizeEngine::new(RelocalizeConfig::default());
        let frame = make_test_frame(42);
        let anchors = vec![make_test_anchor(1), make_test_anchor(2)];
        
        // Register features for anchors
        for anchor in &anchors {
            let fake_features = vec![VisualFeature {
                image_pos: (0.5, 0.5),
                world_pos: None,
                descriptor: super::super::slam::FeatureDescriptor { data: [42u8; 32] },
                track_length: 10,
                is_landmark: true,
            }];
            engine.register_anchor(anchor.id, fake_features);
        }
        
        let result = engine.attempt(&frame, &anchors).await.unwrap();
        
        assert_eq!(result.total_attempted, 2);
    }
}
