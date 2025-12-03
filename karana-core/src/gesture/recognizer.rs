//! Gesture Recognition
//!
//! Classifies hand poses into gestures using geometric rules
//! and machine learning.

use super::{HandPose, GestureType, Finger, GestureConfig, HandLandmark};

/// Gesture recognizer
pub struct GestureRecognizer {
    /// Configuration
    config: GestureConfig,
}

impl GestureRecognizer {
    /// Create new recognizer
    pub fn new(config: GestureConfig) -> Self {
        Self { config }
    }
    
    /// Classify a hand pose into a gesture
    pub fn classify(&self, pose: &HandPose) -> Option<GestureType> {
        if pose.confidence < self.config.confidence_threshold {
            return None;
        }
        
        // Check gestures in priority order
        // More specific gestures first
        
        if self.is_ok_sign(pose) {
            return Some(GestureType::OkSign);
        }
        
        // Check fist BEFORE pinch (in a fist, thumb/index can appear close)
        if self.is_fist(pose) {
            return Some(GestureType::Fist);
        }
        
        if self.is_pinch(pose) {
            return Some(GestureType::Pinch);
        }
        
        if self.is_thumbs_up(pose) {
            return Some(GestureType::ThumbsUp);
        }
        
        if self.is_thumbs_down(pose) {
            return Some(GestureType::ThumbsDown);
        }
        
        if self.is_peace(pose) {
            return Some(GestureType::Peace);
        }
        
        if self.is_rock(pose) {
            return Some(GestureType::Rock);
        }
        
        if self.is_point(pose) {
            return Some(GestureType::Point);
        }
        
        if self.is_fist(pose) {
            return Some(GestureType::Fist);
        }
        
        if self.is_open_palm(pose) {
            return Some(GestureType::OpenPalm);
        }
        
        None
    }
    
    /// Check for pinch gesture (thumb + index touching)
    fn is_pinch(&self, pose: &HandPose) -> bool {
        let distance = pose.pinch_distance();
        distance < self.config.pinch_threshold && 
            !pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            !pose.is_finger_extended(Finger::Pinky)
    }
    
    /// Check for OK sign (thumb + index circle, others extended)
    fn is_ok_sign(&self, pose: &HandPose) -> bool {
        let distance = pose.pinch_distance();
        distance < self.config.pinch_threshold * 1.5 && // Slightly larger threshold
            pose.is_finger_extended(Finger::Middle) &&
            pose.is_finger_extended(Finger::Ring) &&
            pose.is_finger_extended(Finger::Pinky)
    }
    
    /// Check for point gesture (index extended only)
    fn is_point(&self, pose: &HandPose) -> bool {
        pose.is_finger_extended(Finger::Index) &&
            !pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            !pose.is_finger_extended(Finger::Pinky) &&
            !pose.is_finger_extended(Finger::Thumb)
    }
    
    /// Check for fist (no fingers extended)
    fn is_fist(&self, pose: &HandPose) -> bool {
        pose.extended_count() == 0
    }
    
    /// Check for open palm (all fingers extended)
    fn is_open_palm(&self, pose: &HandPose) -> bool {
        pose.extended_count() >= 4 // At least 4 fingers
    }
    
    /// Check for thumbs up
    fn is_thumbs_up(&self, pose: &HandPose) -> bool {
        pose.is_finger_extended(Finger::Thumb) &&
            !pose.is_finger_extended(Finger::Index) &&
            !pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            !pose.is_finger_extended(Finger::Pinky) &&
            self.is_thumb_up(pose)
    }
    
    /// Check for thumbs down  
    fn is_thumbs_down(&self, pose: &HandPose) -> bool {
        pose.is_finger_extended(Finger::Thumb) &&
            !pose.is_finger_extended(Finger::Index) &&
            !pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            !pose.is_finger_extended(Finger::Pinky) &&
            self.is_thumb_down(pose)
    }
    
    /// Check thumb is pointing up
    fn is_thumb_up(&self, pose: &HandPose) -> bool {
        let thumb_tip = pose.landmark(HandLandmark::ThumbTip);
        let thumb_mcp = pose.landmark(HandLandmark::ThumbMcp);
        thumb_tip.position.y > thumb_mcp.position.y + 0.02
    }
    
    /// Check thumb is pointing down
    fn is_thumb_down(&self, pose: &HandPose) -> bool {
        let thumb_tip = pose.landmark(HandLandmark::ThumbTip);
        let thumb_mcp = pose.landmark(HandLandmark::ThumbMcp);
        thumb_tip.position.y < thumb_mcp.position.y - 0.02
    }
    
    /// Check for peace sign (index + middle extended)
    fn is_peace(&self, pose: &HandPose) -> bool {
        pose.is_finger_extended(Finger::Index) &&
            pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            !pose.is_finger_extended(Finger::Pinky)
    }
    
    /// Check for rock gesture (index + pinky extended)
    fn is_rock(&self, pose: &HandPose) -> bool {
        pose.is_finger_extended(Finger::Index) &&
            !pose.is_finger_extended(Finger::Middle) &&
            !pose.is_finger_extended(Finger::Ring) &&
            pose.is_finger_extended(Finger::Pinky)
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new(GestureConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gesture::Landmark3D;
    
    fn create_fist_pose() -> HandPose {
        let mut landmarks = [Landmark3D::default(); 21];
        
        // Wrist
        landmarks[HandLandmark::Wrist as usize] = Landmark3D::new(0.0, 0.0, 0.0, 1.0);
        
        // All fingertips curled into palm
        landmarks[HandLandmark::IndexMcp as usize] = Landmark3D::new(0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::IndexTip as usize] = Landmark3D::new(0.02, -0.01, -0.02, 1.0);
        
        landmarks[HandLandmark::MiddleMcp as usize] = Landmark3D::new(0.0, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::MiddleTip as usize] = Landmark3D::new(0.0, -0.01, -0.02, 1.0);
        
        landmarks[HandLandmark::RingMcp as usize] = Landmark3D::new(-0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::RingTip as usize] = Landmark3D::new(-0.02, -0.01, -0.02, 1.0);
        
        landmarks[HandLandmark::PinkyMcp as usize] = Landmark3D::new(-0.04, 0.0, -0.04, 1.0);
        landmarks[HandLandmark::PinkyTip as usize] = Landmark3D::new(-0.04, -0.01, -0.01, 1.0);
        
        // Thumb wrapping around fist - curled but neutral (not pointing up or down)
        landmarks[HandLandmark::ThumbMcp as usize] = Landmark3D::new(0.04, 0.0, -0.02, 1.0);
        landmarks[HandLandmark::ThumbTip as usize] = Landmark3D::new(0.01, 0.0, -0.02, 1.0);
        
        HandPose::new(super::super::Handedness::Right, landmarks, 0.95)
    }
    
    fn create_open_palm_pose() -> HandPose {
        let mut landmarks = [Landmark3D::default(); 21];
        
        // Wrist at origin
        landmarks[HandLandmark::Wrist as usize] = Landmark3D::new(0.0, 0.0, 0.0, 1.0);
        
        // Extended fingers (tips far from wrist)
        landmarks[HandLandmark::IndexMcp as usize] = Landmark3D::new(0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::IndexTip as usize] = Landmark3D::new(0.02, 0.0, -0.12, 1.0);
        
        landmarks[HandLandmark::MiddleMcp as usize] = Landmark3D::new(0.0, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::MiddleTip as usize] = Landmark3D::new(0.0, 0.0, -0.13, 1.0);
        
        landmarks[HandLandmark::RingMcp as usize] = Landmark3D::new(-0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::RingTip as usize] = Landmark3D::new(-0.02, 0.0, -0.12, 1.0);
        
        landmarks[HandLandmark::PinkyMcp as usize] = Landmark3D::new(-0.04, 0.0, -0.04, 1.0);
        landmarks[HandLandmark::PinkyTip as usize] = Landmark3D::new(-0.04, 0.0, -0.10, 1.0);
        
        landmarks[HandLandmark::ThumbMcp as usize] = Landmark3D::new(0.04, 0.0, -0.02, 1.0);
        landmarks[HandLandmark::ThumbTip as usize] = Landmark3D::new(0.06, 0.0, -0.06, 1.0);
        
        HandPose::new(super::super::Handedness::Right, landmarks, 0.95)
    }
    
    fn create_point_pose() -> HandPose {
        let mut landmarks = [Landmark3D::default(); 21];
        
        // Wrist
        landmarks[HandLandmark::Wrist as usize] = Landmark3D::new(0.0, 0.0, 0.0, 1.0);
        
        // Only index extended
        landmarks[HandLandmark::IndexMcp as usize] = Landmark3D::new(0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::IndexTip as usize] = Landmark3D::new(0.02, 0.0, -0.12, 1.0);
        
        // Others curled
        landmarks[HandLandmark::MiddleMcp as usize] = Landmark3D::new(0.0, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::MiddleTip as usize] = Landmark3D::new(0.0, 0.0, -0.03, 1.0);
        
        landmarks[HandLandmark::RingMcp as usize] = Landmark3D::new(-0.02, 0.0, -0.05, 1.0);
        landmarks[HandLandmark::RingTip as usize] = Landmark3D::new(-0.02, 0.0, -0.03, 1.0);
        
        landmarks[HandLandmark::PinkyMcp as usize] = Landmark3D::new(-0.04, 0.0, -0.04, 1.0);
        landmarks[HandLandmark::PinkyTip as usize] = Landmark3D::new(-0.04, 0.0, -0.02, 1.0);
        
        landmarks[HandLandmark::ThumbMcp as usize] = Landmark3D::new(0.04, 0.0, -0.02, 1.0);
        landmarks[HandLandmark::ThumbTip as usize] = Landmark3D::new(0.03, 0.0, -0.01, 1.0);
        
        HandPose::new(super::super::Handedness::Right, landmarks, 0.95)
    }
    
    #[test]
    fn test_recognizer_creation() {
        let recognizer = GestureRecognizer::default();
        assert!(recognizer.config.confidence_threshold > 0.0);
    }
    
    #[test]
    fn test_fist_detection() {
        let recognizer = GestureRecognizer::default();
        let pose = create_fist_pose();
        
        let gesture = recognizer.classify(&pose);
        assert_eq!(gesture, Some(GestureType::Fist));
    }
    
    #[test]
    fn test_open_palm_detection() {
        let recognizer = GestureRecognizer::default();
        let pose = create_open_palm_pose();
        
        let gesture = recognizer.classify(&pose);
        assert_eq!(gesture, Some(GestureType::OpenPalm));
    }
    
    #[test]
    fn test_point_detection() {
        let recognizer = GestureRecognizer::default();
        let pose = create_point_pose();
        
        let gesture = recognizer.classify(&pose);
        assert_eq!(gesture, Some(GestureType::Point));
    }
    
    #[test]
    fn test_low_confidence_rejected() {
        let recognizer = GestureRecognizer::default();
        let mut pose = create_open_palm_pose();
        pose.confidence = 0.1; // Below threshold
        
        let gesture = recognizer.classify(&pose);
        assert_eq!(gesture, None);
    }
}
