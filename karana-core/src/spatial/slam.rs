//! SLAM Integration Module
//!
//! Integrates with visual SLAM systems (ORB-SLAM3, ARCore, ARKit)
//! to provide spatial tracking and relocalization.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::world_coords::{CoordinateTransform, LocalCoord, RoomId, WorldPosition};

// ============================================================================
// SLAM STATES
// ============================================================================

/// Current state of the SLAM system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlamState {
    /// System is initializing
    Initializing,
    /// Actively tracking with good quality
    Tracking,
    /// Tracking but with reduced quality
    TrackingWeak,
    /// Lost tracking, searching for known features
    Relocalization,
    /// System is not running
    NotRunning,
    /// Error state
    Error,
}

impl Default for SlamState {
    fn default() -> Self {
        Self::NotRunning
    }
}

// ============================================================================
// VISUAL FEATURES
// ============================================================================

/// A visual feature detected in a camera frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualFeature {
    /// 2D position in image (normalized 0-1)
    pub image_pos: (f32, f32),
    /// 3D position if triangulated
    pub world_pos: Option<LocalCoord>,
    /// Feature descriptor (ORB-style)
    pub descriptor: FeatureDescriptor,
    /// Number of frames this feature has been tracked
    pub track_length: u32,
    /// Is this a reliable landmark?
    pub is_landmark: bool,
}

/// Feature descriptor (256-bit ORB-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDescriptor {
    pub data: [u8; 32],
}

impl FeatureDescriptor {
    /// Hamming distance to another descriptor
    pub fn distance_to(&self, other: &FeatureDescriptor) -> u32 {
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }
}

/// Collection of features forming a room signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSignature {
    /// Room identifier
    pub room_id: RoomId,
    /// Landmark features for relocalization
    pub landmarks: Vec<VisualFeature>,
    /// Centroid of landmarks
    pub centroid: LocalCoord,
    /// Bounding box (min, max)
    pub bounds: (LocalCoord, LocalCoord),
    /// When this signature was created
    pub created_at: u64,
    /// How many times successfully relocalized
    pub reloc_count: u32,
}

impl RoomSignature {
    /// Match quality against a set of observed features
    /// Returns (matched_count, total_landmarks, confidence)
    pub fn match_features(&self, observed: &[VisualFeature]) -> (usize, usize, f32) {
        const MATCH_THRESHOLD: u32 = 64; // Max hamming distance for match
        
        let mut matched = 0;
        for landmark in &self.landmarks {
            for obs in observed {
                if landmark.descriptor.distance_to(&obs.descriptor) < MATCH_THRESHOLD {
                    matched += 1;
                    break;
                }
            }
        }
        
        let total = self.landmarks.len();
        let confidence = if total > 0 {
            matched as f32 / total as f32
        } else {
            0.0
        };
        
        (matched, total, confidence)
    }
}

// ============================================================================
// KEYFRAME
// ============================================================================

/// A keyframe in the SLAM map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    /// Unique keyframe ID
    pub id: u64,
    /// Camera pose when captured
    pub pose: CameraPose,
    /// Features observed in this keyframe
    pub features: Vec<VisualFeature>,
    /// Timestamp
    pub timestamp: u64,
    /// Room this keyframe belongs to
    pub room_id: Option<RoomId>,
}

/// Camera pose (position + orientation)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CameraPose {
    /// Position in world coordinates
    pub position: LocalCoord,
    /// Orientation as quaternion (x, y, z, w)
    pub orientation: [f32; 4],
}

impl CameraPose {
    pub fn new(position: LocalCoord, orientation: [f32; 4]) -> Self {
        Self { position, orientation }
    }
    
    /// Identity pose at origin
    pub fn identity() -> Self {
        Self {
            position: LocalCoord::default(),
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Alias for CameraPose for API compatibility
pub type Pose = CameraPose;

/// Alias for Keyframe for API compatibility  
pub type KeyFrame = Keyframe;

/// Visual features extracted from a frame
#[derive(Debug, Clone, Default)]
pub struct VisualFeatures {
    /// List of detected features
    pub features: Vec<VisualFeature>,
    /// Frame timestamp
    pub timestamp: u64,
}

/// SLAM map containing all keyframes and map points
#[derive(Debug, Clone, Default)]
pub struct SlamMap {
    /// All keyframes
    pub keyframes: Vec<Keyframe>,
    /// Map points (landmarks)
    pub map_points: HashMap<u64, LocalCoord>,
    /// Room signatures
    pub room_signatures: HashMap<RoomId, RoomSignature>,
}

// ============================================================================
// SLAM CONFIG
// ============================================================================

/// Configuration for the SLAM engine
#[derive(Debug, Clone)]
pub struct SlamConfig {
    /// Maximum keyframes to keep
    pub max_keyframes: usize,
    /// Feature extraction threshold
    pub feature_threshold: f32,
    /// Minimum features for initialization
    pub min_init_features: usize,
    /// Keyframe distance threshold (meters)
    pub keyframe_distance: f32,
    /// Enable loop closure detection
    pub enable_loop_closure: bool,
}

impl Default for SlamConfig {
    fn default() -> Self {
        Self {
            max_keyframes: 500,
            feature_threshold: 20.0,
            min_init_features: 50,
            keyframe_distance: 0.5,
            enable_loop_closure: true,
        }
    }
}

// ============================================================================
// SLAM ENGINE
// ============================================================================

/// Main SLAM engine that wraps SlamSession with additional features
pub struct SlamEngine {
    /// Configuration
    config: SlamConfig,
    /// Internal session
    session: SlamSession,
    /// Current SLAM map
    map: SlamMap,
}

impl SlamEngine {
    /// Create a new SLAM engine
    pub fn new(config: SlamConfig) -> Self {
        Self {
            config,
            session: SlamSession::new(),
            map: SlamMap::default(),
        }
    }
    
    /// Get current pose
    pub fn get_current_pose(&self) -> Pose {
        self.session.current_pose
    }
    
    /// Track a camera frame
    pub fn track(&mut self, frame: &super::CameraFrame) -> Result<Pose> {
        // Extract features from frame (simplified)
        let features = self.extract_features(frame);
        
        // Process through session
        self.session.process_frame(features)?;
        
        Ok(self.session.current_pose)
    }
    
    /// Extract features from a camera frame
    fn extract_features(&self, frame: &super::CameraFrame) -> Vec<VisualFeature> {
        // Simplified feature extraction
        // Real implementation would use ORB-SLAM3 or similar
        let num_features = (frame.width * frame.height / 1000) as usize;
        let mut features = Vec::with_capacity(num_features.min(500));
        
        for i in 0..num_features.min(500) {
            let x = (i % frame.width as usize) as f32 / frame.width as f32;
            let y = (i / frame.width as usize) as f32 / frame.height as f32;
            
            let mut descriptor = [0u8; 32];
            let idx = i * 3;
            if idx + 32 <= frame.data.len() {
                descriptor.copy_from_slice(&frame.data[idx..idx + 32]);
            }
            
            features.push(VisualFeature {
                image_pos: (x, y),
                world_pos: None,
                descriptor: FeatureDescriptor { data: descriptor },
                track_length: 1,
                is_landmark: i % 10 == 0, // Every 10th is a landmark
            });
        }
        
        features
    }
    
    /// Get SLAM state
    pub fn state(&self) -> SlamState {
        self.session.state
    }
    
    /// Get the map
    pub fn get_map(&self) -> &SlamMap {
        &self.map
    }
    
    /// Reset the engine
    pub fn reset(&mut self) {
        self.session.reset();
        self.map = SlamMap::default();
    }
}

impl Default for SlamEngine {
    fn default() -> Self {
        Self::new(SlamConfig::default())
    }
}

// ============================================================================
// SLAM SESSION
// ============================================================================

/// A SLAM mapping session
pub struct SlamSession {
    /// Current state
    pub state: SlamState,
    /// Current camera pose
    pub current_pose: CameraPose,
    /// Keyframes in this session
    keyframes: Vec<Keyframe>,
    /// Next keyframe ID
    next_keyframe_id: u64,
    /// Map points (3D landmarks)
    map_points: HashMap<u64, LocalCoord>,
    /// Current room (if identified)
    current_room: Option<RoomId>,
    /// Tracking quality (0-100)
    tracking_quality: u8,
    /// Known room signatures for relocalization
    room_signatures: HashMap<RoomId, RoomSignature>,
    /// Session start time
    started_at: u64,
}

impl Default for SlamSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SlamSession {
    /// Create a new SLAM session
    pub fn new() -> Self {
        Self {
            state: SlamState::Initializing,
            current_pose: CameraPose::identity(),
            keyframes: Vec::new(),
            next_keyframe_id: 1,
            map_points: HashMap::new(),
            current_room: None,
            tracking_quality: 0,
            room_signatures: HashMap::new(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Process a camera frame
    /// In real implementation, this would call into ORB-SLAM3 or similar
    pub fn process_frame(&mut self, features: Vec<VisualFeature>) -> Result<()> {
        match self.state {
            SlamState::NotRunning => {
                return Err(anyhow!("SLAM not running"));
            }
            SlamState::Initializing => {
                // Need enough features to initialize
                if features.len() >= 50 {
                    self.state = SlamState::Tracking;
                    self.tracking_quality = 80;
                    self.add_keyframe(features)?;
                }
            }
            SlamState::Tracking | SlamState::TrackingWeak => {
                // Track existing features and add new ones
                self.update_tracking(&features);
                
                // Check if we need a new keyframe
                if self.should_add_keyframe(&features) {
                    self.add_keyframe(features)?;
                }
            }
            SlamState::Relocalization => {
                // Try to match against known room signatures
                if let Some(room_id) = self.try_relocalize(&features) {
                    self.current_room = Some(room_id);
                    self.state = SlamState::Tracking;
                    self.tracking_quality = 70;
                }
            }
            SlamState::Error => {
                return Err(anyhow!("SLAM in error state"));
            }
        }
        
        Ok(())
    }
    
    /// Update tracking with new features
    fn update_tracking(&mut self, features: &[VisualFeature]) {
        // Simplified tracking quality based on feature count
        let quality = match features.len() {
            0..=10 => 10,
            11..=30 => 40,
            31..=50 => 60,
            51..=100 => 80,
            _ => 100,
        };
        
        self.tracking_quality = quality;
        
        // Update state based on quality
        self.state = if quality > 50 {
            SlamState::Tracking
        } else if quality > 20 {
            SlamState::TrackingWeak
        } else {
            SlamState::Relocalization
        };
    }
    
    /// Check if we should add a new keyframe
    fn should_add_keyframe(&self, _features: &[VisualFeature]) -> bool {
        // Simplified: add keyframe every 30 frames
        // Real implementation would check feature overlap, distance traveled, etc.
        if let Some(last) = self.keyframes.last() {
            let dist = self.current_pose.position.distance_to(&last.pose.position);
            dist > 0.5 // New keyframe every 0.5 meters
        } else {
            true
        }
    }
    
    /// Add a new keyframe
    fn add_keyframe(&mut self, features: Vec<VisualFeature>) -> Result<()> {
        let keyframe = Keyframe {
            id: self.next_keyframe_id,
            pose: self.current_pose,
            features,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            room_id: self.current_room.clone(),
        };
        
        self.keyframes.push(keyframe);
        self.next_keyframe_id += 1;
        
        Ok(())
    }
    
    /// Try to relocalize using known room signatures
    fn try_relocalize(&mut self, features: &[VisualFeature]) -> Option<RoomId> {
        let mut best_match: Option<(RoomId, f32)> = None;
        
        for (room_id, signature) in &mut self.room_signatures {
            let (_, _, confidence) = signature.match_features(features);
            
            if confidence > 0.5 {
                if let Some((_, best_conf)) = &best_match {
                    if confidence > *best_conf {
                        best_match = Some((room_id.clone(), confidence));
                    }
                } else {
                    best_match = Some((room_id.clone(), confidence));
                }
            }
        }
        
        if let Some((room_id, _)) = best_match {
            // Increment reloc count
            if let Some(sig) = self.room_signatures.get_mut(&room_id) {
                sig.reloc_count += 1;
            }
            Some(room_id)
        } else {
            None
        }
    }
    
    /// Get current world position
    pub fn get_world_position(&self) -> WorldPosition {
        WorldPosition {
            local: self.current_pose.position,
            room_id: self.current_room.clone(),
            gps: None,
            floor: 0,
            version: 1,
        }
    }
    
    /// Register a new room signature
    pub fn register_room(&mut self, room_id: RoomId, signature: RoomSignature) {
        self.room_signatures.insert(room_id, signature);
    }
    
    /// Get keyframe count
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }
    
    /// Get map point count
    pub fn map_point_count(&self) -> usize {
        self.map_points.len()
    }
    
    /// Get tracking quality (0-100)
    pub fn tracking_quality(&self) -> u8 {
        self.tracking_quality
    }
    
    /// Reset the session
    pub fn reset(&mut self) {
        self.state = SlamState::Initializing;
        self.current_pose = CameraPose::identity();
        self.keyframes.clear();
        self.map_points.clear();
        self.current_room = None;
        self.tracking_quality = 0;
        self.started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Build room signature from current keyframes
    pub fn build_room_signature(&self, room_id: RoomId) -> Option<RoomSignature> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        // Collect all landmark features
        let mut landmarks: Vec<VisualFeature> = Vec::new();
        for kf in &self.keyframes {
            for feat in &kf.features {
                if feat.is_landmark {
                    landmarks.push(feat.clone());
                }
            }
        }
        
        // Limit to top 200 most reliable landmarks
        landmarks.truncate(200);
        
        if landmarks.is_empty() {
            return None;
        }
        
        // Compute centroid
        let mut centroid = LocalCoord::default();
        let mut count = 0;
        for kf in &self.keyframes {
            centroid = centroid + kf.pose.position;
            count += 1;
        }
        centroid.x /= count as f32;
        centroid.y /= count as f32;
        centroid.z /= count as f32;
        
        // Compute bounds
        let mut min = LocalCoord::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = LocalCoord::new(f32::MIN, f32::MIN, f32::MIN);
        for kf in &self.keyframes {
            min.x = min.x.min(kf.pose.position.x);
            min.y = min.y.min(kf.pose.position.y);
            min.z = min.z.min(kf.pose.position.z);
            max.x = max.x.max(kf.pose.position.x);
            max.y = max.y.max(kf.pose.position.y);
            max.z = max.z.max(kf.pose.position.z);
        }
        
        Some(RoomSignature {
            room_id,
            landmarks,
            centroid,
            bounds: (min, max),
            created_at: self.started_at,
            reloc_count: 0,
        })
    }
}

// ============================================================================
// SLAM MANAGER
// ============================================================================

/// Manages SLAM sessions and room transitions
pub struct SlamManager {
    /// Active SLAM session
    session: SlamSession,
    /// All known rooms
    known_rooms: HashMap<RoomId, RoomSignature>,
    /// Transform between rooms (room_a, room_b) -> transform
    room_transforms: HashMap<(RoomId, RoomId), CoordinateTransform>,
}

impl Default for SlamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SlamManager {
    /// Create a new SLAM manager
    pub fn new() -> Self {
        Self {
            session: SlamSession::new(),
            known_rooms: HashMap::new(),
            room_transforms: HashMap::new(),
        }
    }
    
    /// Get current session state
    pub fn state(&self) -> SlamState {
        self.session.state
    }
    
    /// Process camera frame
    pub fn process_frame(&mut self, features: Vec<VisualFeature>) -> Result<()> {
        self.session.process_frame(features)
    }
    
    /// Get current world position
    pub fn current_position(&self) -> WorldPosition {
        self.session.get_world_position()
    }
    
    /// Save current room
    pub fn save_room(&mut self, room_id: RoomId) -> Result<()> {
        let signature = self.session.build_room_signature(room_id.clone())
            .ok_or_else(|| anyhow!("Cannot build room signature"))?;
        
        self.known_rooms.insert(room_id.clone(), signature.clone());
        self.session.register_room(room_id, signature);
        
        Ok(())
    }
    
    /// Get known room count
    pub fn known_room_count(&self) -> usize {
        self.known_rooms.len()
    }
    
    /// Reset session (e.g., when entering a new area)
    pub fn reset_session(&mut self) {
        self.session.reset();
    }
    
    /// Get transform between two rooms
    pub fn get_room_transform(&self, from: &RoomId, to: &RoomId) -> Option<&CoordinateTransform> {
        self.room_transforms.get(&(from.clone(), to.clone()))
    }
    
    /// Register transform between rooms
    pub fn register_room_transform(&mut self, from: RoomId, to: RoomId, transform: CoordinateTransform) {
        self.room_transforms.insert((from, to), transform);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_feature(x: f32, y: f32, landmark: bool) -> VisualFeature {
        VisualFeature {
            image_pos: (x, y),
            world_pos: Some(LocalCoord::new(x, y, 0.0)),
            descriptor: FeatureDescriptor { data: [0u8; 32] },
            track_length: 10,
            is_landmark: landmark,
        }
    }
    
    #[test]
    fn test_slam_initialization() {
        let mut session = SlamSession::new();
        assert_eq!(session.state, SlamState::Initializing);
        
        // Not enough features
        let features: Vec<VisualFeature> = (0..10)
            .map(|i| make_feature(i as f32 / 10.0, 0.0, true))
            .collect();
        session.process_frame(features).unwrap();
        assert_eq!(session.state, SlamState::Initializing);
        
        // Enough features
        let features: Vec<VisualFeature> = (0..60)
            .map(|i| make_feature(i as f32 / 60.0, 0.0, true))
            .collect();
        session.process_frame(features).unwrap();
        assert_eq!(session.state, SlamState::Tracking);
    }
    
    #[test]
    fn test_tracking_quality() {
        let mut session = SlamSession::new();
        
        // Initialize
        let features: Vec<VisualFeature> = (0..60)
            .map(|i| make_feature(i as f32 / 60.0, 0.0, true))
            .collect();
        session.process_frame(features).unwrap();
        assert!(session.tracking_quality() >= 80);
        
        // Low features degrades quality
        let features: Vec<VisualFeature> = (0..5)
            .map(|i| make_feature(i as f32 / 5.0, 0.0, true))
            .collect();
        session.process_frame(features).unwrap();
        assert!(session.tracking_quality() < 50);
    }
    
    #[test]
    fn test_feature_descriptor_distance() {
        let desc1 = FeatureDescriptor { data: [0u8; 32] };
        let desc2 = FeatureDescriptor { data: [0u8; 32] };
        let desc3 = FeatureDescriptor { data: [0xFF; 32] };
        
        assert_eq!(desc1.distance_to(&desc2), 0);
        assert_eq!(desc1.distance_to(&desc3), 256); // All bits different
    }
    
    #[test]
    fn test_room_signature_matching() {
        let landmarks: Vec<VisualFeature> = (0..10)
            .map(|i| {
                let mut feat = make_feature(i as f32 / 10.0, 0.0, true);
                feat.descriptor.data[0] = i;
                feat
            })
            .collect();
        
        let signature = RoomSignature {
            room_id: RoomId::new("test_room"),
            landmarks,
            centroid: LocalCoord::default(),
            bounds: (LocalCoord::default(), LocalCoord::default()),
            created_at: 0,
            reloc_count: 0,
        };
        
        // Perfect match
        let observed: Vec<VisualFeature> = (0..10)
            .map(|i| {
                let mut feat = make_feature(i as f32 / 10.0, 0.0, true);
                feat.descriptor.data[0] = i;
                feat
            })
            .collect();
        
        let (matched, total, confidence) = signature.match_features(&observed);
        assert_eq!(matched, 10);
        assert_eq!(total, 10);
        assert!((confidence - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_slam_manager() {
        let mut manager = SlamManager::new();
        
        assert_eq!(manager.state(), SlamState::Initializing);
        assert_eq!(manager.known_room_count(), 0);
        
        // Initialize tracking
        let features: Vec<VisualFeature> = (0..60)
            .map(|i| make_feature(i as f32 / 60.0, 0.0, true))
            .collect();
        manager.process_frame(features).unwrap();
        
        assert_eq!(manager.state(), SlamState::Tracking);
    }
}
