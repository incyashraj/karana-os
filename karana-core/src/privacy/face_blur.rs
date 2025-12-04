//! Face Blur System for Kāraṇa OS AR Glasses
//!
//! Automatic face detection and blurring to protect bystander privacy
//! in recordings and displays.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use nalgebra::Vector2;

/// Blur mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurMode {
    /// No blur
    Off,
    /// Gaussian blur
    Gaussian,
    /// Pixelation
    Pixelate,
    /// Solid color overlay
    SolidMask,
    /// Silhouette (outline only)
    Silhouette,
    /// Dynamic (automatically adjust to context)
    Dynamic,
}

impl BlurMode {
    /// Get blur intensity description
    pub fn description(&self) -> &str {
        match self {
            BlurMode::Off => "Face blur disabled",
            BlurMode::Gaussian => "Gaussian blur applied to faces",
            BlurMode::Pixelate => "Pixelation applied to faces",
            BlurMode::SolidMask => "Solid mask covering faces",
            BlurMode::Silhouette => "Only outline visible",
            BlurMode::Dynamic => "Automatically adjusts blur method",
        }
    }
    
    /// Processing cost (relative)
    pub fn processing_cost(&self) -> f32 {
        match self {
            BlurMode::Off => 0.0,
            BlurMode::Pixelate => 0.2,
            BlurMode::SolidMask => 0.1,
            BlurMode::Silhouette => 0.3,
            BlurMode::Gaussian => 0.5,
            BlurMode::Dynamic => 0.6,
        }
    }
}

/// Face detection result
#[derive(Debug, Clone)]
pub struct FaceDetection {
    /// Unique face ID for tracking
    pub id: u64,
    /// Bounding box (x, y, width, height) normalized 0-1
    pub bounds: (f32, f32, f32, f32),
    /// Confidence score 0-1
    pub confidence: f32,
    /// Is recognized (known face)
    pub is_recognized: bool,
    /// Person ID if recognized
    pub person_id: Option<String>,
    /// Landmarks (eyes, nose, mouth)
    pub landmarks: Option<FaceLandmarks>,
    /// First seen timestamp
    pub first_seen: Instant,
    /// Last seen timestamp
    pub last_seen: Instant,
    /// Is face of device owner
    pub is_owner: bool,
}

impl FaceDetection {
    /// Get center of face
    pub fn center(&self) -> Vector2<f32> {
        Vector2::new(
            self.bounds.0 + self.bounds.2 / 2.0,
            self.bounds.1 + self.bounds.3 / 2.0,
        )
    }
    
    /// Get face area
    pub fn area(&self) -> f32 {
        self.bounds.2 * self.bounds.3
    }
    
    /// Duration face has been visible
    pub fn visible_duration(&self) -> Duration {
        self.last_seen.duration_since(self.first_seen)
    }
    
    /// Should this face be blurred?
    pub fn should_blur(&self, always_blur_unknown: bool) -> bool {
        // Never blur owner
        if self.is_owner {
            return false;
        }
        
        // Always blur unknown faces if setting enabled
        if always_blur_unknown && !self.is_recognized {
            return true;
        }
        
        // Otherwise, don't blur recognized faces
        !self.is_recognized
    }
}

/// Face landmarks for precise blurring
#[derive(Debug, Clone)]
pub struct FaceLandmarks {
    /// Left eye position (normalized)
    pub left_eye: Vector2<f32>,
    /// Right eye position (normalized)
    pub right_eye: Vector2<f32>,
    /// Nose tip position
    pub nose: Vector2<f32>,
    /// Left mouth corner
    pub mouth_left: Vector2<f32>,
    /// Right mouth corner
    pub mouth_right: Vector2<f32>,
}

/// Privacy zone in the view
#[derive(Debug, Clone)]
pub struct PrivacyZone {
    /// Zone ID
    pub id: String,
    /// Zone name
    pub name: String,
    /// Zone bounds (x, y, width, height) normalized
    pub bounds: (f32, f32, f32, f32),
    /// Blur mode for this zone
    pub blur_mode: BlurMode,
    /// Is zone active
    pub active: bool,
    /// Priority (higher = processed first)
    pub priority: i32,
}

impl PrivacyZone {
    /// Check if point is in zone
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.bounds.0 &&
        x <= self.bounds.0 + self.bounds.2 &&
        y >= self.bounds.1 &&
        y <= self.bounds.1 + self.bounds.3
    }
}

/// Exception for face blur (allowed faces)
#[derive(Debug, Clone)]
pub struct BlurException {
    /// Person ID
    pub person_id: String,
    /// Person name
    pub name: String,
    /// Exception reason
    pub reason: String,
    /// Created at
    pub created_at: Instant,
    /// Expiration (optional)
    pub expires_at: Option<Instant>,
}

/// Face blur engine
#[derive(Debug)]
pub struct FaceBlurEngine {
    /// Is engine enabled
    enabled: bool,
    /// Current blur mode
    blur_mode: BlurMode,
    /// Blur intensity (0-1)
    blur_intensity: f32,
    /// Currently tracked faces
    tracked_faces: HashMap<u64, FaceDetection>,
    /// Next face ID
    next_face_id: u64,
    /// Privacy zones
    privacy_zones: Vec<PrivacyZone>,
    /// Blur exceptions (allowed faces)
    exceptions: HashMap<String, BlurException>,
    /// Always blur unknown faces
    blur_unknown: bool,
    /// Blur in recordings
    blur_recordings: bool,
    /// Blur in live view
    blur_live: bool,
    /// Detection confidence threshold
    confidence_threshold: f32,
    /// Face tracking timeout
    tracking_timeout: Duration,
    /// Processing frame count
    frames_processed: u64,
}

impl FaceBlurEngine {
    /// Create new face blur engine
    pub fn new() -> Self {
        Self {
            enabled: true,
            blur_mode: BlurMode::Gaussian,
            blur_intensity: 0.8,
            tracked_faces: HashMap::new(),
            next_face_id: 1,
            privacy_zones: Vec::new(),
            exceptions: HashMap::new(),
            blur_unknown: true,
            blur_recordings: true,
            blur_live: false,
            confidence_threshold: 0.7,
            tracking_timeout: Duration::from_secs(5),
            frames_processed: 0,
        }
    }
    
    /// Enable/disable engine
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set blur mode
    pub fn set_blur_mode(&mut self, mode: BlurMode) {
        self.blur_mode = mode;
    }
    
    /// Get blur mode
    pub fn blur_mode(&self) -> BlurMode {
        self.blur_mode
    }
    
    /// Set blur intensity
    pub fn set_blur_intensity(&mut self, intensity: f32) {
        self.blur_intensity = intensity.clamp(0.0, 1.0);
    }
    
    /// Get blur intensity
    pub fn blur_intensity(&self) -> f32 {
        self.blur_intensity
    }
    
    /// Process frame and detect faces (simulated)
    pub fn process_frame(&mut self, _frame_data: &[u8]) -> Vec<BlurRegion> {
        if !self.enabled {
            return Vec::new();
        }
        
        self.frames_processed += 1;
        
        // Clean up stale faces
        let now = Instant::now();
        self.tracked_faces.retain(|_, face| {
            now.duration_since(face.last_seen) < self.tracking_timeout
        });
        
        // Generate blur regions for tracked faces
        let mut regions = Vec::new();
        
        for face in self.tracked_faces.values() {
            if face.should_blur(self.blur_unknown) && face.confidence >= self.confidence_threshold {
                // Check if face is in any privacy zone
                let center = face.center();
                let zone_mode = self.get_zone_mode(center.x, center.y);
                let mode = zone_mode.unwrap_or(self.blur_mode);
                
                if mode != BlurMode::Off {
                    regions.push(BlurRegion {
                        bounds: face.bounds,
                        mode,
                        intensity: self.blur_intensity,
                        face_id: Some(face.id),
                    });
                }
            }
        }
        
        // Add privacy zone blur regions
        for zone in &self.privacy_zones {
            if zone.active && zone.blur_mode != BlurMode::Off {
                regions.push(BlurRegion {
                    bounds: zone.bounds,
                    mode: zone.blur_mode,
                    intensity: self.blur_intensity,
                    face_id: None,
                });
            }
        }
        
        regions
    }
    
    /// Get zone mode for position
    fn get_zone_mode(&self, x: f32, y: f32) -> Option<BlurMode> {
        self.privacy_zones
            .iter()
            .filter(|z| z.active && z.contains(x, y))
            .max_by_key(|z| z.priority)
            .map(|z| z.blur_mode)
    }
    
    /// Add detected face (from ML model)
    pub fn add_detection(
        &mut self,
        bounds: (f32, f32, f32, f32),
        confidence: f32,
        person_id: Option<String>,
    ) -> u64 {
        let now = Instant::now();
        let is_recognized = person_id.is_some();
        let is_owner = person_id.as_ref().map(|id| id == "owner").unwrap_or(false);
        
        // Check for exception
        let is_exception = person_id.as_ref()
            .map(|id| self.has_valid_exception(id))
            .unwrap_or(false);
        
        let id = self.next_face_id;
        self.next_face_id += 1;
        
        let face = FaceDetection {
            id,
            bounds,
            confidence,
            is_recognized: is_recognized || is_exception,
            person_id,
            landmarks: None,
            first_seen: now,
            last_seen: now,
            is_owner,
        };
        
        self.tracked_faces.insert(id, face);
        id
    }
    
    /// Update tracked face
    pub fn update_face(&mut self, face_id: u64, bounds: (f32, f32, f32, f32), confidence: f32) {
        if let Some(face) = self.tracked_faces.get_mut(&face_id) {
            face.bounds = bounds;
            face.confidence = confidence;
            face.last_seen = Instant::now();
        }
    }
    
    /// Remove tracked face
    pub fn remove_face(&mut self, face_id: u64) {
        self.tracked_faces.remove(&face_id);
    }
    
    /// Add blur exception
    pub fn add_exception(&mut self, person_id: &str, name: &str, reason: &str, duration: Option<Duration>) {
        let now = Instant::now();
        let exception = BlurException {
            person_id: person_id.to_string(),
            name: name.to_string(),
            reason: reason.to_string(),
            created_at: now,
            expires_at: duration.map(|d| now + d),
        };
        
        self.exceptions.insert(person_id.to_string(), exception);
    }
    
    /// Remove blur exception
    pub fn remove_exception(&mut self, person_id: &str) {
        self.exceptions.remove(person_id);
    }
    
    /// Check if valid exception exists
    pub fn has_valid_exception(&self, person_id: &str) -> bool {
        if let Some(exception) = self.exceptions.get(person_id) {
            if let Some(expires) = exception.expires_at {
                return Instant::now() < expires;
            }
            return true;
        }
        false
    }
    
    /// Add privacy zone
    pub fn add_privacy_zone(&mut self, zone: PrivacyZone) {
        self.privacy_zones.push(zone);
    }
    
    /// Remove privacy zone
    pub fn remove_privacy_zone(&mut self, zone_id: &str) {
        self.privacy_zones.retain(|z| z.id != zone_id);
    }
    
    /// Get tracked faces
    pub fn tracked_faces(&self) -> impl Iterator<Item = &FaceDetection> {
        self.tracked_faces.values()
    }
    
    /// Get face count
    pub fn face_count(&self) -> usize {
        self.tracked_faces.len()
    }
    
    /// Get statistics
    pub fn stats(&self) -> FaceBlurStats {
        let faces_blurred = self.tracked_faces.values()
            .filter(|f| f.should_blur(self.blur_unknown))
            .count();
        
        FaceBlurStats {
            enabled: self.enabled,
            blur_mode: self.blur_mode,
            tracked_faces: self.tracked_faces.len(),
            faces_blurred,
            exceptions: self.exceptions.len(),
            privacy_zones: self.privacy_zones.len(),
            frames_processed: self.frames_processed,
        }
    }
    
    /// Set blur in recordings
    pub fn set_blur_recordings(&mut self, enabled: bool) {
        self.blur_recordings = enabled;
    }
    
    /// Set blur in live view
    pub fn set_blur_live(&mut self, enabled: bool) {
        self.blur_live = enabled;
    }
    
    /// Should blur for recording?
    pub fn should_blur_recording(&self) -> bool {
        self.enabled && self.blur_recordings
    }
    
    /// Should blur for live view?
    pub fn should_blur_live(&self) -> bool {
        self.enabled && self.blur_live
    }
}

impl Default for FaceBlurEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Blur region to be applied
#[derive(Debug, Clone)]
pub struct BlurRegion {
    /// Region bounds (x, y, width, height) normalized
    pub bounds: (f32, f32, f32, f32),
    /// Blur mode
    pub mode: BlurMode,
    /// Blur intensity
    pub intensity: f32,
    /// Associated face ID (if any)
    pub face_id: Option<u64>,
}

/// Face blur statistics
#[derive(Debug, Clone)]
pub struct FaceBlurStats {
    /// Engine enabled
    pub enabled: bool,
    /// Current blur mode
    pub blur_mode: BlurMode,
    /// Currently tracked faces
    pub tracked_faces: usize,
    /// Faces being blurred
    pub faces_blurred: usize,
    /// Number of exceptions
    pub exceptions: usize,
    /// Privacy zones defined
    pub privacy_zones: usize,
    /// Total frames processed
    pub frames_processed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_face_blur_engine_creation() {
        let engine = FaceBlurEngine::new();
        assert!(engine.is_enabled());
        assert_eq!(engine.blur_mode(), BlurMode::Gaussian);
    }
    
    #[test]
    fn test_face_detection() {
        let mut engine = FaceBlurEngine::new();
        
        let id = engine.add_detection((0.3, 0.2, 0.2, 0.3), 0.9, None);
        
        assert_eq!(engine.face_count(), 1);
        assert!(engine.tracked_faces().any(|f| f.id == id));
    }
    
    #[test]
    fn test_face_should_blur() {
        let face = FaceDetection {
            id: 1,
            bounds: (0.3, 0.2, 0.2, 0.3),
            confidence: 0.9,
            is_recognized: false,
            person_id: None,
            landmarks: None,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            is_owner: false,
        };
        
        // Unknown face should blur
        assert!(face.should_blur(true));
    }
    
    #[test]
    fn test_owner_never_blurred() {
        let face = FaceDetection {
            id: 1,
            bounds: (0.3, 0.2, 0.2, 0.3),
            confidence: 0.9,
            is_recognized: true,
            person_id: Some("owner".to_string()),
            landmarks: None,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            is_owner: true,
        };
        
        // Owner never blurred
        assert!(!face.should_blur(true));
    }
    
    #[test]
    fn test_blur_exception() {
        let mut engine = FaceBlurEngine::new();
        
        engine.add_exception("friend1", "Alice", "Friend", None);
        
        assert!(engine.has_valid_exception("friend1"));
        assert!(!engine.has_valid_exception("stranger"));
    }
    
    #[test]
    fn test_privacy_zone() {
        let mut engine = FaceBlurEngine::new();
        
        engine.add_privacy_zone(PrivacyZone {
            id: "zone1".to_string(),
            name: "Private Area".to_string(),
            bounds: (0.0, 0.0, 0.5, 0.5),
            blur_mode: BlurMode::Pixelate,
            active: true,
            priority: 1,
        });
        
        let mode = engine.get_zone_mode(0.25, 0.25);
        assert_eq!(mode, Some(BlurMode::Pixelate));
    }
    
    #[test]
    fn test_process_frame() {
        let mut engine = FaceBlurEngine::new();
        
        engine.add_detection((0.3, 0.2, 0.2, 0.3), 0.9, None);
        
        let regions = engine.process_frame(&[]);
        
        assert!(!regions.is_empty());
    }
    
    #[test]
    fn test_blur_mode_cost() {
        assert!(BlurMode::Gaussian.processing_cost() > BlurMode::SolidMask.processing_cost());
    }
    
    #[test]
    fn test_face_center() {
        let face = FaceDetection {
            id: 1,
            bounds: (0.2, 0.3, 0.4, 0.4),
            confidence: 0.9,
            is_recognized: false,
            person_id: None,
            landmarks: None,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            is_owner: false,
        };
        
        let center = face.center();
        assert!((center.x - 0.4).abs() < 0.001);
        assert!((center.y - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_blur_stats() {
        let mut engine = FaceBlurEngine::new();
        
        engine.add_detection((0.3, 0.2, 0.2, 0.3), 0.9, None);
        engine.add_detection((0.6, 0.4, 0.2, 0.3), 0.85, Some("friend".to_string()));
        
        let stats = engine.stats();
        assert_eq!(stats.tracked_faces, 2);
        assert_eq!(stats.faces_blurred, 1); // Only unknown face
    }
}
