//! Hand Pose Data Structures
//!
//! Contains hand pose representation with 21 skeletal landmarks.

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::world_coords::LocalCoord;
use super::{Handedness, HandLandmark, Landmark3D, Finger, GestureType, GestureState};

/// Complete hand pose with all 21 landmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandPose {
    /// Which hand
    pub handedness: Handedness,
    /// 21 landmarks (MediaPipe compatible indices)
    pub landmarks: [Landmark3D; 21],
    /// Overall detection confidence
    pub confidence: f32,
    /// Timestamp (ms since epoch)
    pub timestamp: u64,
}

impl Default for HandPose {
    fn default() -> Self {
        Self {
            handedness: Handedness::Unknown,
            landmarks: [Landmark3D::default(); 21],
            confidence: 0.0,
            timestamp: 0,
        }
    }
}

impl HandPose {
    /// Create new hand pose
    pub fn new(handedness: Handedness, landmarks: [Landmark3D; 21], confidence: f32) -> Self {
        Self {
            handedness,
            landmarks,
            confidence,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Get specific landmark
    pub fn landmark(&self, lm: HandLandmark) -> &Landmark3D {
        &self.landmarks[lm as usize]
    }
    
    /// Get wrist position
    pub fn wrist(&self) -> LocalCoord {
        self.landmarks[HandLandmark::Wrist as usize].position
    }
    
    /// Get palm center (average of MCP joints)
    pub fn palm_center(&self) -> LocalCoord {
        let mcp_indices = [
            HandLandmark::IndexMcp as usize,
            HandLandmark::MiddleMcp as usize,
            HandLandmark::RingMcp as usize,
            HandLandmark::PinkyMcp as usize,
        ];
        
        let mut sum = LocalCoord::new(0.0, 0.0, 0.0);
        for idx in mcp_indices {
            sum.x += self.landmarks[idx].position.x;
            sum.y += self.landmarks[idx].position.y;
            sum.z += self.landmarks[idx].position.z;
        }
        
        LocalCoord::new(sum.x / 4.0, sum.y / 4.0, sum.z / 4.0)
    }
    
    /// Get fingertip position
    pub fn fingertip(&self, finger: Finger) -> LocalCoord {
        self.landmarks[finger.tip() as usize].position
    }
    
    /// Distance between thumb and index tips (for pinch detection)
    pub fn pinch_distance(&self) -> f32 {
        let thumb = &self.landmarks[HandLandmark::ThumbTip as usize];
        let index = &self.landmarks[HandLandmark::IndexTip as usize];
        thumb.distance_to(index)
    }
    
    /// Check if finger is extended (straightened)
    pub fn is_finger_extended(&self, finger: Finger) -> bool {
        let mcp = &self.landmarks[finger.mcp() as usize].position;
        let tip = &self.landmarks[finger.tip() as usize].position;
        let wrist = &self.landmarks[HandLandmark::Wrist as usize].position;
        
        // Simple check: tip is further from wrist than MCP
        let mcp_dist = mcp.distance_to(wrist);
        let tip_dist = tip.distance_to(wrist);
        
        tip_dist > mcp_dist * 1.2  // 20% threshold
    }
    
    /// Get extended fingers
    pub fn extended_fingers(&self) -> Vec<Finger> {
        let fingers = [Finger::Thumb, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky];
        fingers.into_iter()
            .filter(|f| self.is_finger_extended(*f))
            .collect()
    }
    
    /// Count extended fingers
    pub fn extended_count(&self) -> usize {
        [Finger::Thumb, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky]
            .iter()
            .filter(|f| self.is_finger_extended(**f))
            .count()
    }
    
    /// Get palm normal direction (approximated)
    pub fn palm_normal(&self) -> LocalCoord {
        // Use cross product of two palm vectors
        let wrist = self.wrist();
        let index_mcp = self.landmarks[HandLandmark::IndexMcp as usize].position;
        let pinky_mcp = self.landmarks[HandLandmark::PinkyMcp as usize].position;
        
        // Vectors along palm
        let v1 = LocalCoord::new(
            index_mcp.x - wrist.x,
            index_mcp.y - wrist.y,
            index_mcp.z - wrist.z,
        );
        let v2 = LocalCoord::new(
            pinky_mcp.x - wrist.x,
            pinky_mcp.y - wrist.y,
            pinky_mcp.z - wrist.z,
        );
        
        // Cross product
        let normal = LocalCoord::new(
            v1.y * v2.z - v1.z * v2.y,
            v1.z * v2.x - v1.x * v2.z,
            v1.x * v2.y - v1.y * v2.x,
        );
        
        // Normalize
        normal.normalize().unwrap_or(LocalCoord::new(0.0, 0.0, -1.0))
    }
    
    /// Get index finger pointing ray
    pub fn index_ray(&self) -> (LocalCoord, LocalCoord) {
        let mcp = self.landmarks[HandLandmark::IndexMcp as usize].position;
        let tip = self.landmarks[HandLandmark::IndexTip as usize].position;
        
        let direction = LocalCoord::new(
            tip.x - mcp.x,
            tip.y - mcp.y,
            tip.z - mcp.z,
        );
        
        (tip, direction.normalize().unwrap_or(LocalCoord::new(0.0, 0.0, -1.0)))
    }
    
    /// Hand span (thumb tip to pinky tip)
    pub fn hand_span(&self) -> f32 {
        let thumb = &self.landmarks[HandLandmark::ThumbTip as usize];
        let pinky = &self.landmarks[HandLandmark::PinkyTip as usize];
        thumb.distance_to(pinky)
    }
}

/// Runtime hand state with gesture tracking
#[derive(Debug, Clone)]
pub struct HandState {
    /// Current pose
    pub pose: HandPose,
    /// Current detected gesture
    pub current_gesture: Option<GestureType>,
    /// Gesture state machine
    pub gesture_state: GestureState,
    /// When gesture started
    pub gesture_start_time: Option<u64>,
    /// Previous pose (for motion detection)
    pub prev_pose: Option<HandPose>,
    /// Velocity estimation
    pub velocity: LocalCoord,
    /// Last update timestamp
    pub last_update: u64,
}

impl HandState {
    /// Create new hand state
    pub fn new(pose: HandPose, gesture: Option<GestureType>, now: u64) -> Self {
        Self {
            pose,
            current_gesture: gesture,
            gesture_state: if gesture.is_some() { 
                GestureState::Starting 
            } else { 
                GestureState::None 
            },
            gesture_start_time: gesture.map(|_| now),
            prev_pose: None,
            velocity: LocalCoord::default(),
            last_update: now,
        }
    }
    
    /// Update with new pose
    pub fn update(&mut self, pose: HandPose, gesture: Option<GestureType>, now: u64) {
        // Calculate velocity
        let dt = (now - self.last_update) as f32 / 1000.0;
        if dt > 0.0 {
            let palm = pose.palm_center();
            let prev_palm = self.pose.palm_center();
            self.velocity = LocalCoord::new(
                (palm.x - prev_palm.x) / dt,
                (palm.y - prev_palm.y) / dt,
                (palm.z - prev_palm.z) / dt,
            );
        }
        
        // Update gesture state
        match (&self.current_gesture, &gesture) {
            (None, Some(_)) => {
                // New gesture starting
                self.gesture_state = GestureState::Starting;
                self.gesture_start_time = Some(now);
            }
            (Some(old), Some(new)) if old == new => {
                // Same gesture continuing
                if self.gesture_state == GestureState::Starting {
                    // Check hold time
                    if let Some(start) = self.gesture_start_time {
                        if now - start > 100 {  // 100ms hold time
                            self.gesture_state = GestureState::Active;
                        }
                    }
                }
            }
            (Some(_), None) => {
                // Gesture ended
                self.gesture_state = GestureState::Ending;
            }
            (Some(_), Some(_)) => {
                // Gesture changed - new one starting
                self.gesture_state = GestureState::Starting;
                self.gesture_start_time = Some(now);
            }
            (None, None) => {
                self.gesture_state = GestureState::None;
            }
        }
        
        self.prev_pose = Some(std::mem::replace(&mut self.pose, pose));
        self.current_gesture = gesture;
        self.last_update = now;
    }
    
    /// Get pinch info (strength 0-1, position)
    pub fn pinch_info(&self) -> Option<(f32, LocalCoord)> {
        let distance = self.pose.pinch_distance();
        // Map distance to strength (closer = stronger)
        // Full pinch at ~1cm, no pinch at ~5cm
        let strength = (1.0 - (distance - 0.01) / 0.04).clamp(0.0, 1.0);
        
        if strength > 0.1 {
            // Pinch position is midpoint
            let thumb = self.pose.landmarks[HandLandmark::ThumbTip as usize].position;
            let index = self.pose.landmarks[HandLandmark::IndexTip as usize].position;
            let pos = LocalCoord::new(
                (thumb.x + index.x) / 2.0,
                (thumb.y + index.y) / 2.0,
                (thumb.z + index.z) / 2.0,
            );
            Some((strength, pos))
        } else {
            None
        }
    }
    
    /// Get pointing ray
    pub fn point_ray(&self) -> Option<(LocalCoord, LocalCoord)> {
        if self.pose.is_finger_extended(Finger::Index) {
            Some(self.pose.index_ray())
        } else {
            None
        }
    }
    
    /// Get gesture duration
    pub fn gesture_duration(&self) -> Option<u64> {
        self.gesture_start_time.map(|start| self.last_update - start)
    }
    
    /// Check if hand is moving fast
    pub fn is_moving_fast(&self, threshold: f32) -> bool {
        self.velocity.length() > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_landmarks() -> [Landmark3D; 21] {
        let mut landmarks = [Landmark3D::default(); 21];
        
        // Set up a basic hand pose with wrist at origin
        // and fingers pointing forward (negative Z)
        landmarks[HandLandmark::Wrist as usize] = Landmark3D::new(0.0, 0.0, 0.0, 1.0);
        
        // MCP joints
        landmarks[HandLandmark::IndexMcp as usize] = Landmark3D::new(0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::MiddleMcp as usize] = Landmark3D::new(0.0, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::RingMcp as usize] = Landmark3D::new(-0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::PinkyMcp as usize] = Landmark3D::new(-0.04, 0.0, -0.04, 1.0);
        landmarks[HandLandmark::ThumbMcp as usize] = Landmark3D::new(0.04, 0.0, -0.02, 1.0);
        
        // Fingertips (extended)
        landmarks[HandLandmark::IndexTip as usize] = Landmark3D::new(0.02, 0.0, -0.12, 1.0);
        landmarks[HandLandmark::MiddleTip as usize] = Landmark3D::new(0.0, 0.0, -0.13, 1.0);
        landmarks[HandLandmark::RingTip as usize] = Landmark3D::new(-0.02, 0.0, -0.12, 1.0);
        landmarks[HandLandmark::PinkyTip as usize] = Landmark3D::new(-0.04, 0.0, -0.10, 1.0);
        landmarks[HandLandmark::ThumbTip as usize] = Landmark3D::new(0.06, 0.0, -0.06, 1.0);
        
        landmarks
    }
    
    #[test]
    fn test_hand_pose_creation() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks, 0.95);
        
        assert_eq!(pose.handedness, Handedness::Right);
        assert!(pose.confidence > 0.9);
    }
    
    #[test]
    fn test_palm_center() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks, 1.0);
        
        let center = pose.palm_center();
        // Should be roughly at the average of MCP joints
        assert!(center.z < 0.0); // In front of wrist
    }
    
    #[test]
    fn test_pinch_distance() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks, 1.0);
        
        let distance = pose.pinch_distance();
        assert!(distance > 0.0);
    }
    
    #[test]
    fn test_finger_extended() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks, 1.0);
        
        // With our test landmarks, fingers should be extended
        assert!(pose.is_finger_extended(Finger::Index));
        assert!(pose.is_finger_extended(Finger::Middle));
    }
    
    #[test]
    fn test_extended_count() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks, 1.0);
        
        let count = pose.extended_count();
        assert!(count >= 2); // At least index and middle extended
    }
    
    #[test]
    fn test_hand_state_velocity() {
        let landmarks = test_landmarks();
        let pose = HandPose::new(Handedness::Right, landmarks.clone(), 1.0);
        
        let mut state = HandState::new(pose, None, 0);
        
        // Move hand forward
        let mut new_landmarks = landmarks;
        for lm in &mut new_landmarks {
            lm.position.z -= 0.1; // Move 10cm forward
        }
        let new_pose = HandPose::new(Handedness::Right, new_landmarks, 1.0);
        
        state.update(new_pose, None, 100); // 100ms later
        
        // Velocity should be negative Z (moving forward)
        assert!(state.velocity.z < 0.0);
    }
}
