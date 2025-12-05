// Kāraṇa OS - Gaze and Face Tracking Module
// Eye gaze estimation, face detection, and expression recognition

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{MLError, MLConfig};
use super::inference::ImageInput;
use super::vision::BoundingBox;

/// Gaze tracking system
#[derive(Debug)]
pub struct GazeTracker {
    /// Configuration
    config: GazeConfig,
    /// Gaze history for smoothing
    gaze_history: VecDeque<GazePoint>,
    /// Calibration data
    calibration: Option<GazeCalibration>,
    /// Last valid gaze
    last_gaze: Option<GazePoint>,
    /// Blink detector
    blink_detector: BlinkDetector,
}

/// Gaze configuration
#[derive(Debug, Clone)]
pub struct GazeConfig {
    /// Enable pupil tracking
    pub pupil_tracking: bool,
    /// Enable head pose compensation
    pub head_pose_compensation: bool,
    /// Smoothing window size
    pub smoothing_window: usize,
    /// Fixation threshold (pixels)
    pub fixation_threshold: f32,
    /// Fixation duration (ms)
    pub fixation_duration_ms: u32,
    /// Saccade velocity threshold
    pub saccade_threshold: f32,
    /// Model type
    pub model: GazeModel,
}

impl Default for GazeConfig {
    fn default() -> Self {
        Self {
            pupil_tracking: true,
            head_pose_compensation: true,
            smoothing_window: 5,
            fixation_threshold: 50.0,
            fixation_duration_ms: 100,
            saccade_threshold: 500.0,
            model: GazeModel::LiteGaze,
        }
    }
}

/// Gaze model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GazeModel {
    /// Lightweight gaze estimation
    LiteGaze,
    /// Full gaze with calibration
    FullGaze,
    /// MPIIGaze
    MpiiGaze,
    /// Gaze360
    Gaze360,
}

impl GazeModel {
    /// Get model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            GazeModel::LiteGaze => "lite-gaze",
            GazeModel::FullGaze => "full-gaze",
            GazeModel::MpiiGaze => "mpii-gaze",
            GazeModel::Gaze360 => "gaze360",
        }
    }
}

/// Gaze point
#[derive(Debug, Clone, Copy)]
pub struct GazePoint {
    /// X coordinate (normalized 0-1 or screen pixels)
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Confidence
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
    /// Pupil diameter (if available)
    pub pupil_diameter: Option<f32>,
}

impl GazePoint {
    /// Distance to another point
    pub fn distance(&self, other: &GazePoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Gaze calibration
#[derive(Debug, Clone)]
pub struct GazeCalibration {
    /// Calibration points
    pub points: Vec<CalibrationPoint>,
    /// Calibration accuracy
    pub accuracy: f32,
    /// Is calibrated
    pub is_calibrated: bool,
}

/// Calibration point
#[derive(Debug, Clone)]
pub struct CalibrationPoint {
    /// Target screen position
    pub target: (f32, f32),
    /// Measured gaze positions
    pub samples: Vec<(f32, f32)>,
    /// Error after calibration
    pub error: f32,
}

impl GazeTracker {
    /// Create new tracker
    pub fn new(config: GazeConfig) -> Self {
        let window_size = config.smoothing_window;
        Self {
            config,
            gaze_history: VecDeque::with_capacity(window_size),
            calibration: None,
            last_gaze: None,
            blink_detector: BlinkDetector::new(),
        }
    }

    /// Process frame and estimate gaze
    pub fn process_frame(&mut self, image: &ImageInput) -> Result<GazeResult, MLError> {
        let start = Instant::now();

        // Detect face and eyes (simulated)
        let face = self.detect_face(image)?;

        // Estimate gaze
        let raw_gaze = self.estimate_gaze(image, &face)?;

        // Apply calibration if available
        let calibrated_gaze = if let Some(ref cal) = self.calibration {
            self.apply_calibration(raw_gaze, cal)
        } else {
            raw_gaze
        };

        // Smooth gaze
        self.gaze_history.push_back(calibrated_gaze);
        if self.gaze_history.len() > self.config.smoothing_window {
            self.gaze_history.pop_front();
        }

        let smoothed = self.smooth_gaze();

        // Detect blink
        let blink = self.blink_detector.update(&face);

        // Detect fixation or saccade
        let gaze_event = self.detect_gaze_event(&smoothed);

        self.last_gaze = Some(smoothed);

        Ok(GazeResult {
            gaze: smoothed,
            raw_gaze,
            face: Some(face),
            is_blinking: blink.is_some(),
            blink: blink,
            gaze_event,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    /// Detect face (simulated)
    fn detect_face(&self, image: &ImageInput) -> Result<FaceData, MLError> {
        let cx = image.width as f32 / 2.0;
        let cy = image.height as f32 / 2.0;

        Ok(FaceData {
            bbox: BoundingBox::new(cx - 100.0, cy - 120.0, 200.0, 240.0),
            landmarks: FaceLandmarks {
                left_eye: (cx - 40.0, cy - 30.0),
                right_eye: (cx + 40.0, cy - 30.0),
                nose: (cx, cy + 10.0),
                mouth_left: (cx - 30.0, cy + 60.0),
                mouth_right: (cx + 30.0, cy + 60.0),
            },
            head_pose: HeadPose {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
            eye_openness: EyeOpenness {
                left: 0.85,
                right: 0.85,
            },
            confidence: 0.95,
        })
    }

    /// Estimate gaze (simulated)
    fn estimate_gaze(&self, image: &ImageInput, face: &FaceData) -> Result<GazePoint, MLError> {
        // Simulated gaze near center with some noise
        let cx = image.width as f32 / 2.0;
        let cy = image.height as f32 / 2.0;

        // Add slight variation based on head pose
        let x = cx + face.head_pose.yaw * 10.0;
        let y = cy + face.head_pose.pitch * 10.0;

        Ok(GazePoint {
            x,
            y,
            confidence: 0.88,
            timestamp: Instant::now(),
            pupil_diameter: Some(4.5),
        })
    }

    /// Apply calibration
    fn apply_calibration(&self, gaze: GazePoint, _cal: &GazeCalibration) -> GazePoint {
        // In real implementation, would transform coordinates
        gaze
    }

    /// Smooth gaze using history
    fn smooth_gaze(&self) -> GazePoint {
        if self.gaze_history.is_empty() {
            return GazePoint {
                x: 0.0,
                y: 0.0,
                confidence: 0.0,
                timestamp: Instant::now(),
                pupil_diameter: None,
            };
        }

        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        let mut conf_sum = 0.0;

        for gaze in &self.gaze_history {
            x_sum += gaze.x * gaze.confidence;
            y_sum += gaze.y * gaze.confidence;
            conf_sum += gaze.confidence;
        }

        GazePoint {
            x: x_sum / conf_sum,
            y: y_sum / conf_sum,
            confidence: conf_sum / self.gaze_history.len() as f32,
            timestamp: Instant::now(),
            pupil_diameter: self.gaze_history.back().and_then(|g| g.pupil_diameter),
        }
    }

    /// Detect gaze event (fixation or saccade)
    fn detect_gaze_event(&self, current: &GazePoint) -> Option<GazeEvent> {
        let Some(ref last) = self.last_gaze else {
            return None;
        };

        let distance = current.distance(last);
        let dt = current.timestamp.duration_since(last.timestamp).as_secs_f32();

        if dt <= 0.0 {
            return None;
        }

        let velocity = distance / dt;

        if velocity > self.config.saccade_threshold {
            Some(GazeEvent::Saccade {
                start: (last.x, last.y),
                end: (current.x, current.y),
                velocity,
            })
        } else if distance < self.config.fixation_threshold {
            Some(GazeEvent::Fixation {
                x: current.x,
                y: current.y,
                duration: Duration::from_millis(self.config.fixation_duration_ms as u64),
            })
        } else {
            None
        }
    }

    /// Start calibration
    pub fn start_calibration(&mut self, num_points: usize) -> Vec<(f32, f32)> {
        // Generate calibration points in grid pattern
        let mut points = Vec::new();

        let cols = ((num_points as f32).sqrt().ceil() as usize).max(2);
        let rows = (num_points + cols - 1) / cols;

        for row in 0..rows {
            for col in 0..cols {
                if points.len() >= num_points {
                    break;
                }
                let x = (col as f32 + 0.5) / cols as f32;
                let y = (row as f32 + 0.5) / rows as f32;
                points.push((x, y));
            }
        }

        self.calibration = Some(GazeCalibration {
            points: points.iter().map(|&(x, y)| CalibrationPoint {
                target: (x, y),
                samples: Vec::new(),
                error: 0.0,
            }).collect(),
            accuracy: 0.0,
            is_calibrated: false,
        });

        points
    }

    /// Add calibration sample
    pub fn add_calibration_sample(&mut self, point_index: usize, gaze: (f32, f32)) -> bool {
        if let Some(ref mut cal) = self.calibration {
            if point_index < cal.points.len() {
                cal.points[point_index].samples.push(gaze);
                return true;
            }
        }
        false
    }

    /// Finish calibration
    pub fn finish_calibration(&mut self) -> Option<f32> {
        let cal = self.calibration.as_mut()?;

        // Calculate average error for each point
        let mut total_error = 0.0;
        for point in &mut cal.points {
            if point.samples.is_empty() {
                continue;
            }
            let avg_x: f32 = point.samples.iter().map(|(x, _)| x).sum::<f32>() / point.samples.len() as f32;
            let avg_y: f32 = point.samples.iter().map(|(_, y)| y).sum::<f32>() / point.samples.len() as f32;

            let dx = avg_x - point.target.0;
            let dy = avg_y - point.target.1;
            point.error = (dx * dx + dy * dy).sqrt();
            total_error += point.error;
        }

        cal.accuracy = total_error / cal.points.len() as f32;
        cal.is_calibrated = true;

        Some(cal.accuracy)
    }

    /// Check if calibrated
    pub fn is_calibrated(&self) -> bool {
        self.calibration.as_ref().map(|c| c.is_calibrated).unwrap_or(false)
    }
}

/// Gaze result
#[derive(Debug)]
pub struct GazeResult {
    /// Smoothed gaze point
    pub gaze: GazePoint,
    /// Raw gaze point
    pub raw_gaze: GazePoint,
    /// Face data
    pub face: Option<FaceData>,
    /// Is currently blinking
    pub is_blinking: bool,
    /// Blink event
    pub blink: Option<BlinkEvent>,
    /// Gaze event
    pub gaze_event: Option<GazeEvent>,
    /// Processing latency
    pub latency_ms: f64,
}

/// Gaze event
#[derive(Debug, Clone)]
pub enum GazeEvent {
    /// Fixation (dwelling on a point)
    Fixation {
        x: f32,
        y: f32,
        duration: Duration,
    },
    /// Saccade (rapid eye movement)
    Saccade {
        start: (f32, f32),
        end: (f32, f32),
        velocity: f32,
    },
}

/// Face data
#[derive(Debug, Clone)]
pub struct FaceData {
    /// Face bounding box
    pub bbox: BoundingBox,
    /// Facial landmarks
    pub landmarks: FaceLandmarks,
    /// Head pose
    pub head_pose: HeadPose,
    /// Eye openness
    pub eye_openness: EyeOpenness,
    /// Detection confidence
    pub confidence: f32,
}

/// Face landmarks
#[derive(Debug, Clone, Copy)]
pub struct FaceLandmarks {
    /// Left eye center
    pub left_eye: (f32, f32),
    /// Right eye center
    pub right_eye: (f32, f32),
    /// Nose tip
    pub nose: (f32, f32),
    /// Mouth left corner
    pub mouth_left: (f32, f32),
    /// Mouth right corner
    pub mouth_right: (f32, f32),
}

/// Head pose (Euler angles)
#[derive(Debug, Clone, Copy)]
pub struct HeadPose {
    /// Pitch (up/down)
    pub pitch: f32,
    /// Yaw (left/right)
    pub yaw: f32,
    /// Roll (tilt)
    pub roll: f32,
}

/// Eye openness
#[derive(Debug, Clone, Copy)]
pub struct EyeOpenness {
    /// Left eye (0 = closed, 1 = open)
    pub left: f32,
    /// Right eye
    pub right: f32,
}

/// Blink detector
#[derive(Debug)]
pub struct BlinkDetector {
    /// Openness threshold for closed
    closed_threshold: f32,
    /// Openness threshold for open
    open_threshold: f32,
    /// Was eyes closed
    was_closed: bool,
    /// Closed start time
    closed_at: Option<Instant>,
    /// Recent blinks
    recent_blinks: VecDeque<BlinkEvent>,
}

impl BlinkDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            closed_threshold: 0.3,
            open_threshold: 0.6,
            was_closed: false,
            closed_at: None,
            recent_blinks: VecDeque::with_capacity(10),
        }
    }

    /// Update with face data
    pub fn update(&mut self, face: &FaceData) -> Option<BlinkEvent> {
        let avg_openness = (face.eye_openness.left + face.eye_openness.right) / 2.0;

        // Detect close
        if !self.was_closed && avg_openness < self.closed_threshold {
            self.was_closed = true;
            self.closed_at = Some(Instant::now());
        }

        // Detect open (blink complete)
        if self.was_closed && avg_openness > self.open_threshold {
            self.was_closed = false;

            if let Some(closed_at) = self.closed_at.take() {
                let duration = closed_at.elapsed();

                // Filter out too short or too long
                if duration >= Duration::from_millis(50) && duration <= Duration::from_millis(500) {
                    let blink = BlinkEvent {
                        timestamp: Instant::now(),
                        duration,
                        is_double: self.is_double_blink(duration),
                    };

                    self.recent_blinks.push_back(blink.clone());
                    if self.recent_blinks.len() > 10 {
                        self.recent_blinks.pop_front();
                    }

                    return Some(blink);
                }
            }
        }

        None
    }

    /// Check if this is a double blink
    fn is_double_blink(&self, _duration: Duration) -> bool {
        if self.recent_blinks.is_empty() {
            return false;
        }

        if let Some(last) = self.recent_blinks.back() {
            // Double blink if within 500ms of last blink
            last.timestamp.elapsed() < Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Get blinks per minute
    pub fn blink_rate(&self) -> f32 {
        let cutoff = Instant::now() - Duration::from_secs(60);
        let recent: Vec<_> = self.recent_blinks.iter()
            .filter(|b| b.timestamp > cutoff)
            .collect();

        recent.len() as f32
    }
}

impl Default for BlinkDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Blink event
#[derive(Debug, Clone)]
pub struct BlinkEvent {
    /// When the blink occurred
    pub timestamp: Instant,
    /// How long eyes were closed
    pub duration: Duration,
    /// Is this a double blink
    pub is_double: bool,
}

/// Expression recognizer
#[derive(Debug)]
pub struct ExpressionRecognizer {
    /// Model type
    model: ExpressionModel,
}

/// Expression model variants
#[derive(Debug, Clone, Copy)]
pub enum ExpressionModel {
    FerPlus,
    EmotionNet,
}

impl ExpressionRecognizer {
    /// Create new recognizer
    pub fn new(model: ExpressionModel) -> Self {
        Self { model }
    }

    /// Recognize expression
    pub fn recognize(&self, face: &FaceData) -> Expression {
        // Simulated expression detection
        Expression {
            emotion: Emotion::Neutral,
            confidence: 0.85,
            scores: vec![
                (Emotion::Neutral, 0.85),
                (Emotion::Happy, 0.10),
                (Emotion::Surprise, 0.03),
                (Emotion::Sad, 0.02),
            ],
        }
    }
}

/// Expression result
#[derive(Debug, Clone)]
pub struct Expression {
    /// Primary emotion
    pub emotion: Emotion,
    /// Confidence
    pub confidence: f32,
    /// All emotion scores
    pub scores: Vec<(Emotion, f32)>,
}

/// Emotion categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fear,
    Surprise,
    Disgust,
    Contempt,
}

/// Face mesh for AR effects
#[derive(Debug)]
pub struct FaceMesh {
    /// 468 face mesh landmarks
    pub landmarks: Vec<(f32, f32, f32)>,
    /// Face contour indices
    pub contour: Vec<usize>,
    /// Left eye contour
    pub left_eye_contour: Vec<usize>,
    /// Right eye contour
    pub right_eye_contour: Vec<usize>,
    /// Lips contour
    pub lips_contour: Vec<usize>,
}

impl FaceMesh {
    /// Create empty mesh
    pub fn new() -> Self {
        Self {
            landmarks: vec![(0.0, 0.0, 0.0); 468],
            contour: Vec::new(),
            left_eye_contour: Vec::new(),
            right_eye_contour: Vec::new(),
            lips_contour: Vec::new(),
        }
    }

    /// Get landmark
    pub fn landmark(&self, idx: usize) -> Option<(f32, f32, f32)> {
        self.landmarks.get(idx).copied()
    }

    /// Get 2D projection
    pub fn landmark_2d(&self, idx: usize) -> Option<(f32, f32)> {
        self.landmarks.get(idx).map(|(x, y, _)| (*x, *y))
    }
}

impl Default for FaceMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Full face tracker combining everything
#[derive(Debug)]
pub struct FaceTracker {
    /// Gaze tracker
    gaze: GazeTracker,
    /// Expression recognizer
    expression: ExpressionRecognizer,
    /// Face mesh (optional)
    mesh_enabled: bool,
}

impl FaceTracker {
    /// Create new tracker
    pub fn new(gaze_config: GazeConfig, mesh_enabled: bool) -> Self {
        Self {
            gaze: GazeTracker::new(gaze_config),
            expression: ExpressionRecognizer::new(ExpressionModel::FerPlus),
            mesh_enabled,
        }
    }

    /// Process frame
    pub fn process_frame(&mut self, image: &ImageInput) -> Result<FaceTrackingResult, MLError> {
        let gaze_result = self.gaze.process_frame(image)?;

        let expression = gaze_result.face.as_ref()
            .map(|f| self.expression.recognize(f));

        let mesh = if self.mesh_enabled {
            Some(FaceMesh::new())
        } else {
            None
        };

        Ok(FaceTrackingResult {
            gaze: gaze_result.gaze,
            face: gaze_result.face,
            expression,
            mesh,
            blink: gaze_result.blink,
            gaze_event: gaze_result.gaze_event,
            latency_ms: gaze_result.latency_ms,
        })
    }

    /// Get gaze tracker
    pub fn gaze_tracker(&mut self) -> &mut GazeTracker {
        &mut self.gaze
    }
}

/// Full face tracking result
#[derive(Debug)]
pub struct FaceTrackingResult {
    /// Gaze point
    pub gaze: GazePoint,
    /// Face data
    pub face: Option<FaceData>,
    /// Expression
    pub expression: Option<Expression>,
    /// Face mesh
    pub mesh: Option<FaceMesh>,
    /// Blink event
    pub blink: Option<BlinkEvent>,
    /// Gaze event
    pub gaze_event: Option<GazeEvent>,
    /// Processing latency
    pub latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::inference::ImageFormat;

    fn test_image() -> ImageInput {
        ImageInput {
            data: vec![128u8; 640 * 480 * 3],
            width: 640,
            height: 480,
            format: ImageFormat::RGB,
        }
    }

    #[test]
    fn test_gaze_config_default() {
        let config = GazeConfig::default();
        assert!(config.pupil_tracking);
        assert_eq!(config.smoothing_window, 5);
    }

    #[test]
    fn test_gaze_point_distance() {
        let p1 = GazePoint {
            x: 0.0,
            y: 0.0,
            confidence: 1.0,
            timestamp: Instant::now(),
            pupil_diameter: None,
        };
        let p2 = GazePoint {
            x: 3.0,
            y: 4.0,
            confidence: 1.0,
            timestamp: Instant::now(),
            pupil_diameter: None,
        };

        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn test_gaze_tracker() {
        let config = GazeConfig::default();
        let mut tracker = GazeTracker::new(config);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        assert!(result.gaze.confidence > 0.0);
        assert!(result.latency_ms > 0.0);
    }

    #[test]
    fn test_gaze_smoothing() {
        let config = GazeConfig::default();
        let mut tracker = GazeTracker::new(config);
        let image = test_image();

        // Process multiple frames
        for _ in 0..10 {
            let _ = tracker.process_frame(&image).unwrap();
        }

        let result = tracker.process_frame(&image).unwrap();
        assert!(result.gaze.confidence > 0.0);
    }

    #[test]
    fn test_calibration() {
        let config = GazeConfig::default();
        let mut tracker = GazeTracker::new(config);

        let points = tracker.start_calibration(9);
        assert_eq!(points.len(), 9);

        // Add samples
        for i in 0..9 {
            tracker.add_calibration_sample(i, (points[i].0 + 0.01, points[i].1 + 0.01));
        }

        let accuracy = tracker.finish_calibration();
        assert!(accuracy.is_some());
        assert!(tracker.is_calibrated());
    }

    #[test]
    fn test_blink_detector() {
        let mut detector = BlinkDetector::new();

        // Simulate open eyes
        let face_open = FaceData {
            bbox: BoundingBox::new(0.0, 0.0, 100.0, 100.0),
            landmarks: FaceLandmarks {
                left_eye: (30.0, 30.0),
                right_eye: (70.0, 30.0),
                nose: (50.0, 50.0),
                mouth_left: (35.0, 70.0),
                mouth_right: (65.0, 70.0),
            },
            head_pose: HeadPose { pitch: 0.0, yaw: 0.0, roll: 0.0 },
            eye_openness: EyeOpenness { left: 0.85, right: 0.85 },
            confidence: 0.95,
        };

        let result = detector.update(&face_open);
        assert!(result.is_none());

        // Simulate closed eyes
        let face_closed = FaceData {
            eye_openness: EyeOpenness { left: 0.1, right: 0.1 },
            ..face_open.clone()
        };

        let _ = detector.update(&face_closed);

        // Wait a bit and open
        std::thread::sleep(Duration::from_millis(100));

        let result = detector.update(&face_open);
        assert!(result.is_some());
    }

    #[test]
    fn test_expression_recognition() {
        let recognizer = ExpressionRecognizer::new(ExpressionModel::FerPlus);

        let face = FaceData {
            bbox: BoundingBox::new(0.0, 0.0, 100.0, 100.0),
            landmarks: FaceLandmarks {
                left_eye: (30.0, 30.0),
                right_eye: (70.0, 30.0),
                nose: (50.0, 50.0),
                mouth_left: (35.0, 70.0),
                mouth_right: (65.0, 70.0),
            },
            head_pose: HeadPose { pitch: 0.0, yaw: 0.0, roll: 0.0 },
            eye_openness: EyeOpenness { left: 0.85, right: 0.85 },
            confidence: 0.95,
        };

        let expression = recognizer.recognize(&face);
        assert!(expression.confidence > 0.0);
    }

    #[test]
    fn test_face_mesh() {
        let mesh = FaceMesh::new();
        assert_eq!(mesh.landmarks.len(), 468);

        let landmark = mesh.landmark(0);
        assert!(landmark.is_some());

        let landmark_2d = mesh.landmark_2d(0);
        assert!(landmark_2d.is_some());
    }

    #[test]
    fn test_face_tracker() {
        let config = GazeConfig::default();
        let mut tracker = FaceTracker::new(config, true);
        let image = test_image();

        let result = tracker.process_frame(&image).unwrap();
        assert!(result.gaze.confidence > 0.0);
        assert!(result.face.is_some());
        assert!(result.mesh.is_some());
    }

    #[test]
    fn test_head_pose() {
        let pose = HeadPose {
            pitch: 10.0,
            yaw: -5.0,
            roll: 2.0,
        };

        assert_eq!(pose.pitch, 10.0);
        assert_eq!(pose.yaw, -5.0);
        assert_eq!(pose.roll, 2.0);
    }
}
