// Object Detection for Kāraṇa OS
// Handles detecting objects, faces, hands, and scene elements

use super::*;
use std::collections::HashMap;
use std::time::Instant;

/// Detected object category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectCategory {
    Person,
    Face,
    Hand,
    Vehicle,
    Animal,
    Text,
    QRCode,
    Barcode,
    Food,
    Furniture,
    Electronics,
    Plant,
    Sign,
    Door,
    Window,
    Stairs,
    TrafficLight,
    Crosswalk,
    Obstacle,
    Unknown,
}

impl ObjectCategory {
    /// Check if category is important for navigation
    pub fn is_navigation_relevant(&self) -> bool {
        matches!(self, 
            ObjectCategory::Door |
            ObjectCategory::Stairs |
            ObjectCategory::TrafficLight |
            ObjectCategory::Crosswalk |
            ObjectCategory::Obstacle |
            ObjectCategory::Sign
        )
    }
    
    /// Check if category is a person or part of person
    pub fn is_person_related(&self) -> bool {
        matches!(self,
            ObjectCategory::Person |
            ObjectCategory::Face |
            ObjectCategory::Hand
        )
    }
}

/// Bounding box for detected objects
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    /// Get center point
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    
    /// Get area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
    
    /// Check if point is inside
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }
    
    /// Calculate IoU (Intersection over Union) with another box
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        
        let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
        let union = self.area() + other.area() - intersection;
        
        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }
    
    /// Expand box by percentage
    pub fn expand(&self, percent: f32) -> BoundingBox {
        let dx = self.width * percent / 2.0;
        let dy = self.height * percent / 2.0;
        
        BoundingBox {
            x: self.x - dx,
            y: self.y - dy,
            width: self.width + dx * 2.0,
            height: self.height + dy * 2.0,
        }
    }
    
    /// Convert to normalized coordinates (0-1)
    pub fn normalize(&self, frame_width: u32, frame_height: u32) -> BoundingBox {
        BoundingBox {
            x: self.x / frame_width as f32,
            y: self.y / frame_height as f32,
            width: self.width / frame_width as f32,
            height: self.height / frame_height as f32,
        }
    }
}

/// A detected object with metadata
#[derive(Debug, Clone)]
pub struct Detection {
    pub id: u64,
    pub category: ObjectCategory,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub label: Option<String>,
    pub timestamp: Instant,
    pub tracking_id: Option<u64>,
    pub attributes: HashMap<String, String>,
}

impl Detection {
    pub fn new(category: ObjectCategory, bounding_box: BoundingBox, confidence: f32) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            category,
            bounding_box,
            confidence,
            label: None,
            timestamp: Instant::now(),
            tracking_id: None,
            attributes: HashMap::new(),
        }
    }
    
    /// Set label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    /// Set tracking ID
    pub fn with_tracking_id(mut self, id: u64) -> Self {
        self.tracking_id = Some(id);
        self
    }
    
    /// Add attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
    
    /// Get age of detection
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

/// Face detection with landmarks
#[derive(Debug, Clone)]
pub struct FaceDetection {
    pub detection: Detection,
    pub landmarks: Option<FaceLandmarks>,
    pub emotion: Option<Emotion>,
    pub age_estimate: Option<u8>,
    pub gender_estimate: Option<Gender>,
    pub gaze_direction: Option<(f32, f32)>,
    pub is_looking_at_camera: bool,
}

/// Face landmark points
#[derive(Debug, Clone)]
pub struct FaceLandmarks {
    pub left_eye: (f32, f32),
    pub right_eye: (f32, f32),
    pub nose_tip: (f32, f32),
    pub mouth_left: (f32, f32),
    pub mouth_right: (f32, f32),
    pub mouth_center: (f32, f32),
    pub chin: (f32, f32),
}

impl FaceLandmarks {
    /// Calculate eye distance
    pub fn eye_distance(&self) -> f32 {
        let dx = self.right_eye.0 - self.left_eye.0;
        let dy = self.right_eye.1 - self.left_eye.1;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Estimate face rotation angle
    pub fn rotation_angle(&self) -> f32 {
        let dy = self.right_eye.1 - self.left_eye.1;
        let dx = self.right_eye.0 - self.left_eye.0;
        dy.atan2(dx).to_degrees()
    }
}

/// Detected emotion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Fearful,
    Disgusted,
}

/// Estimated gender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

/// Hand detection with gesture recognition
#[derive(Debug, Clone)]
pub struct HandDetection {
    pub detection: Detection,
    pub is_left: bool,
    pub landmarks: Option<Vec<(f32, f32)>>,
    pub gesture: Option<HandGesture>,
    pub confidence: f32,
}

/// Recognized hand gestures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandGesture {
    Open,
    Closed,
    Pointing,
    Thumbs,
    Peace,
    Wave,
    Pinch,
    Grab,
    Swipe,
    Unknown,
}

/// Object tracker for persistent tracking across frames
pub struct ObjectTracker {
    tracked_objects: HashMap<u64, TrackedObject>,
    next_tracking_id: u64,
    max_age: std::time::Duration,
    iou_threshold: f32,
}

/// A tracked object with history
struct TrackedObject {
    tracking_id: u64,
    category: ObjectCategory,
    last_detection: Detection,
    positions: Vec<(Instant, BoundingBox)>,
    frames_tracked: u32,
    frames_missing: u32,
}

impl ObjectTracker {
    pub fn new() -> Self {
        Self {
            tracked_objects: HashMap::new(),
            next_tracking_id: 1,
            max_age: std::time::Duration::from_secs(2),
            iou_threshold: 0.3,
        }
    }
    
    /// Update tracker with new detections
    pub fn update(&mut self, detections: Vec<Detection>) -> Vec<Detection> {
        let mut matched_detections = Vec::new();
        let mut unmatched_detections = detections;
        let mut matched_tracking_ids = Vec::new();
        
        // Match detections to existing tracks
        for (&tracking_id, tracked) in &self.tracked_objects {
            let mut best_match: Option<(usize, f32)> = None;
            
            for (i, det) in unmatched_detections.iter().enumerate() {
                if det.category == tracked.category {
                    let iou = det.bounding_box.iou(&tracked.last_detection.bounding_box);
                    if iou > self.iou_threshold {
                        if best_match.map_or(true, |(_, best_iou)| iou > best_iou) {
                            best_match = Some((i, iou));
                        }
                    }
                }
            }
            
            if let Some((idx, _)) = best_match {
                let mut det = unmatched_detections.remove(idx);
                det.tracking_id = Some(tracking_id);
                matched_detections.push(det);
                matched_tracking_ids.push(tracking_id);
            }
        }
        
        // Update matched tracks
        for det in &matched_detections {
            if let Some(tracking_id) = det.tracking_id {
                if let Some(tracked) = self.tracked_objects.get_mut(&tracking_id) {
                    tracked.last_detection = det.clone();
                    tracked.frames_tracked += 1;
                    tracked.frames_missing = 0;
                    tracked.positions.push((Instant::now(), det.bounding_box));
                    
                    // Keep only recent positions
                    if tracked.positions.len() > 30 {
                        tracked.positions.remove(0);
                    }
                }
            }
        }
        
        // Create new tracks for unmatched detections
        for mut det in unmatched_detections {
            let tracking_id = self.next_tracking_id;
            self.next_tracking_id += 1;
            
            det.tracking_id = Some(tracking_id);
            
            let tracked = TrackedObject {
                tracking_id,
                category: det.category,
                last_detection: det.clone(),
                positions: vec![(Instant::now(), det.bounding_box)],
                frames_tracked: 1,
                frames_missing: 0,
            };
            
            self.tracked_objects.insert(tracking_id, tracked);
            matched_detections.push(det);
        }
        
        // Age out old tracks
        let now = Instant::now();
        self.tracked_objects.retain(|id, tracked| {
            if !matched_tracking_ids.contains(id) {
                tracked.frames_missing += 1;
            }
            
            tracked.last_detection.timestamp.elapsed() < self.max_age &&
            tracked.frames_missing < 10
        });
        
        matched_detections
    }
    
    /// Get tracked object count
    pub fn tracked_count(&self) -> usize {
        self.tracked_objects.len()
    }
    
    /// Get velocity of tracked object
    pub fn get_velocity(&self, tracking_id: u64) -> Option<(f32, f32)> {
        let tracked = self.tracked_objects.get(&tracking_id)?;
        
        if tracked.positions.len() < 2 {
            return None;
        }
        
        let recent = &tracked.positions[tracked.positions.len() - 1];
        let previous = &tracked.positions[tracked.positions.len() - 2];
        
        let dt = recent.0.duration_since(previous.0).as_secs_f32();
        if dt > 0.0 {
            let (cx1, cy1) = previous.1.center();
            let (cx2, cy2) = recent.1.center();
            
            Some(((cx2 - cx1) / dt, (cy2 - cy1) / dt))
        } else {
            None
        }
    }
    
    /// Clear all tracks
    pub fn clear(&mut self) {
        self.tracked_objects.clear();
    }
}

impl Default for ObjectTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Detection configuration
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    pub min_confidence: f32,
    pub categories: Vec<ObjectCategory>,
    pub max_detections: usize,
    pub enable_tracking: bool,
    pub nms_threshold: f32,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            categories: vec![
                ObjectCategory::Person,
                ObjectCategory::Face,
                ObjectCategory::Hand,
                ObjectCategory::QRCode,
                ObjectCategory::Text,
            ],
            max_detections: 50,
            enable_tracking: true,
            nms_threshold: 0.4,
        }
    }
}

/// Object detector manager
pub struct ObjectDetector {
    config: DetectionConfig,
    tracker: ObjectTracker,
    detection_count: u64,
}

impl ObjectDetector {
    pub fn new(config: DetectionConfig) -> Self {
        Self {
            config,
            tracker: ObjectTracker::new(),
            detection_count: 0,
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &DetectionConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: DetectionConfig) {
        self.config = config;
    }
    
    /// Process detections (apply NMS and tracking)
    pub fn process(&mut self, mut detections: Vec<Detection>) -> Vec<Detection> {
        // Filter by confidence
        detections.retain(|d| d.confidence >= self.config.min_confidence);
        
        // Filter by category
        if !self.config.categories.is_empty() {
            detections.retain(|d| self.config.categories.contains(&d.category));
        }
        
        // Apply Non-Maximum Suppression
        detections = self.apply_nms(detections);
        
        // Limit count
        detections.truncate(self.config.max_detections);
        
        // Apply tracking
        if self.config.enable_tracking {
            detections = self.tracker.update(detections);
        }
        
        self.detection_count += detections.len() as u64;
        
        detections
    }
    
    /// Apply Non-Maximum Suppression
    fn apply_nms(&self, mut detections: Vec<Detection>) -> Vec<Detection> {
        // Sort by confidence (highest first)
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        let mut keep = Vec::new();
        
        while !detections.is_empty() {
            let best = detections.remove(0);
            
            // Remove overlapping detections of same category
            detections.retain(|d| {
                d.category != best.category ||
                best.bounding_box.iou(&d.bounding_box) < self.config.nms_threshold
            });
            
            keep.push(best);
        }
        
        keep
    }
    
    /// Get total detection count
    pub fn total_detections(&self) -> u64 {
        self.detection_count
    }
    
    /// Get tracked object count
    pub fn tracked_count(&self) -> usize {
        self.tracker.tracked_count()
    }
    
    /// Clear tracker
    pub fn reset_tracking(&mut self) {
        self.tracker.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bounding_box_center() {
        let bb = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        assert_eq!(bb.center(), (125.0, 125.0));
    }
    
    #[test]
    fn test_bounding_box_contains() {
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        
        assert!(bb.contains(50.0, 50.0));
        assert!(bb.contains(0.0, 0.0));
        assert!(!bb.contains(101.0, 50.0));
    }
    
    #[test]
    fn test_bounding_box_iou() {
        let bb1 = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let bb2 = BoundingBox::new(50.0, 50.0, 100.0, 100.0);
        
        // 50x50 intersection / (10000 + 10000 - 2500) union
        let iou = bb1.iou(&bb2);
        assert!((iou - 0.142857).abs() < 0.01);
        
        // Same box = IoU of 1
        assert!((bb1.iou(&bb1) - 1.0).abs() < 0.001);
        
        // Non-overlapping
        let bb3 = BoundingBox::new(200.0, 200.0, 50.0, 50.0);
        assert_eq!(bb1.iou(&bb3), 0.0);
    }
    
    #[test]
    fn test_bounding_box_normalize() {
        let bb = BoundingBox::new(320.0, 240.0, 64.0, 48.0);
        let normalized = bb.normalize(640, 480);
        
        assert_eq!(normalized.x, 0.5);
        assert_eq!(normalized.y, 0.5);
        assert_eq!(normalized.width, 0.1);
        assert_eq!(normalized.height, 0.1);
    }
    
    #[test]
    fn test_detection_creation() {
        let bb = BoundingBox::new(10.0, 20.0, 100.0, 100.0);
        let det = Detection::new(ObjectCategory::Person, bb, 0.95)
            .with_label("person_1")
            .with_attribute("pose", "standing");
        
        assert_eq!(det.category, ObjectCategory::Person);
        assert_eq!(det.confidence, 0.95);
        assert_eq!(det.label, Some("person_1".to_string()));
        assert_eq!(det.attributes.get("pose"), Some(&"standing".to_string()));
    }
    
    #[test]
    fn test_object_category_navigation() {
        assert!(ObjectCategory::Door.is_navigation_relevant());
        assert!(ObjectCategory::Stairs.is_navigation_relevant());
        assert!(!ObjectCategory::Person.is_navigation_relevant());
    }
    
    #[test]
    fn test_object_category_person() {
        assert!(ObjectCategory::Person.is_person_related());
        assert!(ObjectCategory::Face.is_person_related());
        assert!(!ObjectCategory::Vehicle.is_person_related());
    }
    
    #[test]
    fn test_face_landmarks_distance() {
        let landmarks = FaceLandmarks {
            left_eye: (100.0, 100.0),
            right_eye: (200.0, 100.0),
            nose_tip: (150.0, 150.0),
            mouth_left: (120.0, 180.0),
            mouth_right: (180.0, 180.0),
            mouth_center: (150.0, 180.0),
            chin: (150.0, 220.0),
        };
        
        assert_eq!(landmarks.eye_distance(), 100.0);
        assert_eq!(landmarks.rotation_angle(), 0.0);
    }
    
    #[test]
    fn test_object_tracker() {
        let mut tracker = ObjectTracker::new();
        
        let bb = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let det = Detection::new(ObjectCategory::Person, bb, 0.9);
        
        let tracked = tracker.update(vec![det]);
        
        assert_eq!(tracked.len(), 1);
        assert!(tracked[0].tracking_id.is_some());
        assert_eq!(tracker.tracked_count(), 1);
    }
    
    #[test]
    fn test_tracker_continuity() {
        let mut tracker = ObjectTracker::new();
        
        // Frame 1
        let bb1 = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let det1 = Detection::new(ObjectCategory::Person, bb1, 0.9);
        let tracked1 = tracker.update(vec![det1]);
        let id1 = tracked1[0].tracking_id;
        
        // Frame 2 - slightly moved
        let bb2 = BoundingBox::new(105.0, 105.0, 50.0, 50.0);
        let det2 = Detection::new(ObjectCategory::Person, bb2, 0.9);
        let tracked2 = tracker.update(vec![det2]);
        
        // Should maintain same tracking ID
        assert_eq!(tracked2[0].tracking_id, id1);
    }
    
    #[test]
    fn test_object_detector_nms() {
        let config = DetectionConfig {
            min_confidence: 0.5,
            nms_threshold: 0.4, // Lower threshold to ensure suppression
            enable_tracking: false,
            ..Default::default()
        };
        let mut detector = ObjectDetector::new(config);
        
        // Two highly overlapping detections (IoU > 0.4)
        let bb1 = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let bb2 = BoundingBox::new(105.0, 105.0, 50.0, 50.0); // More overlap
        
        let det1 = Detection::new(ObjectCategory::Person, bb1, 0.9);
        let det2 = Detection::new(ObjectCategory::Person, bb2, 0.8);
        
        let result = detector.process(vec![det1, det2]);
        
        // NMS should keep only the higher confidence one
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence, 0.9);
    }
    
    #[test]
    fn test_detector_confidence_filter() {
        let config = DetectionConfig {
            min_confidence: 0.7,
            enable_tracking: false,
            ..Default::default()
        };
        let mut detector = ObjectDetector::new(config);
        
        let bb = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let det_high = Detection::new(ObjectCategory::Person, bb, 0.9);
        let det_low = Detection::new(ObjectCategory::Person, bb, 0.5);
        
        let result = detector.process(vec![det_high, det_low]);
        
        assert_eq!(result.len(), 1);
    }
    
    #[test]
    fn test_bounding_box_expand() {
        let bb = BoundingBox::new(100.0, 100.0, 100.0, 100.0);
        let expanded = bb.expand(0.2);
        
        assert_eq!(expanded.x, 90.0);
        assert_eq!(expanded.y, 90.0);
        assert_eq!(expanded.width, 120.0);
        assert_eq!(expanded.height, 120.0);
    }
}
