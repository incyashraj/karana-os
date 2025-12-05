// Camera Capture Management for Kāraṇa OS
// Handles frame capture, buffering, and streaming

use super::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Frame capture mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Continuous capture at specified FPS
    Continuous,
    /// Single frame on demand
    SingleFrame,
    /// Burst capture (multiple frames quickly)
    Burst(u32),
    /// Time-lapse capture
    TimeLapse(Duration),
}

/// Frame buffer for storing captured frames
pub struct FrameBuffer {
    frames: VecDeque<CameraFrame>,
    max_size: usize,
    total_captured: AtomicU64,
    dropped_frames: AtomicU64,
}

impl FrameBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_size),
            max_size,
            total_captured: AtomicU64::new(0),
            dropped_frames: AtomicU64::new(0),
        }
    }
    
    /// Push a new frame to the buffer
    pub fn push(&mut self, frame: CameraFrame) {
        self.total_captured.fetch_add(1, Ordering::SeqCst);
        
        if self.frames.len() >= self.max_size {
            self.frames.pop_front();
            self.dropped_frames.fetch_add(1, Ordering::SeqCst);
        }
        
        self.frames.push_back(frame);
    }
    
    /// Pop the oldest frame from the buffer
    pub fn pop(&mut self) -> Option<CameraFrame> {
        self.frames.pop_front()
    }
    
    /// Get the most recent frame without removing it
    pub fn latest(&self) -> Option<&CameraFrame> {
        self.frames.back()
    }
    
    /// Get frame at specific index
    pub fn get(&self, index: usize) -> Option<&CameraFrame> {
        self.frames.get(index)
    }
    
    /// Current buffer size
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    
    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.frames.len() >= self.max_size
    }
    
    /// Total frames captured
    pub fn total_captured(&self) -> u64 {
        self.total_captured.load(Ordering::SeqCst)
    }
    
    /// Number of dropped frames
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames.load(Ordering::SeqCst)
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.frames.clear();
    }
    
    /// Get all frames as a slice
    pub fn frames(&self) -> Vec<&CameraFrame> {
        self.frames.iter().collect()
    }
}

/// Capture session configuration
#[derive(Debug, Clone)]
pub struct CaptureSession {
    pub camera_id: CameraId,
    pub mode: CaptureMode,
    pub config: CameraConfig,
    pub started_at: Option<Instant>,
    pub frames_captured: u64,
    pub target_fps: u32,
}

impl CaptureSession {
    pub fn new(camera_id: CameraId, config: CameraConfig) -> Self {
        let target_fps = config.fps;
        Self {
            camera_id,
            mode: CaptureMode::Continuous,
            config,
            started_at: None,
            frames_captured: 0,
            target_fps,
        }
    }
    
    /// Start the capture session
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.frames_captured = 0;
    }
    
    /// Stop the capture session
    pub fn stop(&mut self) {
        self.started_at = None;
    }
    
    /// Check if session is running
    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }
    
    /// Get session duration
    pub fn duration(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }
    
    /// Calculate actual FPS
    pub fn actual_fps(&self) -> f32 {
        match self.duration() {
            Some(duration) if duration.as_secs_f32() > 0.0 => {
                self.frames_captured as f32 / duration.as_secs_f32()
            }
            _ => 0.0,
        }
    }
    
    /// Record a captured frame
    pub fn record_capture(&mut self) {
        self.frames_captured += 1;
    }
    
    /// Calculate frame interval for target FPS
    pub fn frame_interval(&self) -> Duration {
        if self.target_fps > 0 {
            Duration::from_secs_f32(1.0 / self.target_fps as f32)
        } else {
            Duration::from_millis(33) // Default to ~30 FPS
        }
    }
}

/// Frame synchronization for multi-camera setups
pub struct FrameSynchronizer {
    cameras: Vec<CameraId>,
    sync_tolerance: Duration,
    frame_groups: VecDeque<Vec<CameraFrame>>,
    pending_frames: Vec<(CameraId, CameraFrame)>,
    max_groups: usize,
}

impl FrameSynchronizer {
    pub fn new(cameras: Vec<CameraId>, sync_tolerance: Duration) -> Self {
        Self {
            cameras,
            sync_tolerance,
            frame_groups: VecDeque::new(),
            pending_frames: Vec::new(),
            max_groups: 10,
        }
    }
    
    /// Add a frame for synchronization
    pub fn add_frame(&mut self, frame: CameraFrame) {
        self.pending_frames.push((frame.camera_id, frame));
        self.try_synchronize();
    }
    
    /// Try to form synchronized frame groups
    fn try_synchronize(&mut self) {
        if self.pending_frames.len() < self.cameras.len() {
            return;
        }
        
        // Find frames within tolerance
        let reference_time = self.pending_frames[0].1.timestamp;
        let mut group = Vec::new();
        let mut used_indices = Vec::new();
        
        for camera_id in &self.cameras {
            for (i, (cid, frame)) in self.pending_frames.iter().enumerate() {
                if cid == camera_id && !used_indices.contains(&i) {
                    let time_diff = if frame.timestamp > reference_time {
                        frame.timestamp.duration_since(reference_time)
                    } else {
                        reference_time.duration_since(frame.timestamp)
                    };
                    
                    if time_diff <= self.sync_tolerance {
                        group.push(frame.clone());
                        used_indices.push(i);
                        break;
                    }
                }
            }
        }
        
        // If we have frames from all cameras
        if group.len() == self.cameras.len() {
            // Remove used frames
            used_indices.sort_by(|a, b| b.cmp(a));
            for i in used_indices {
                self.pending_frames.remove(i);
            }
            
            // Add to groups
            if self.frame_groups.len() >= self.max_groups {
                self.frame_groups.pop_front();
            }
            self.frame_groups.push_back(group);
        }
    }
    
    /// Get the next synchronized frame group
    pub fn pop_group(&mut self) -> Option<Vec<CameraFrame>> {
        self.frame_groups.pop_front()
    }
    
    /// Number of available synchronized groups
    pub fn available_groups(&self) -> usize {
        self.frame_groups.len()
    }
    
    /// Number of pending unsynchronized frames
    pub fn pending_count(&self) -> usize {
        self.pending_frames.len()
    }
}

/// Frame rate controller
pub struct FrameRateController {
    target_fps: u32,
    last_frame_time: Option<Instant>,
    frame_times: VecDeque<Duration>,
    max_samples: usize,
}

impl FrameRateController {
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            last_frame_time: None,
            frame_times: VecDeque::with_capacity(60),
            max_samples: 60,
        }
    }
    
    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
    }
    
    /// Get target FPS
    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }
    
    /// Check if it's time for the next frame
    pub fn should_capture(&mut self) -> bool {
        let now = Instant::now();
        let interval = Duration::from_secs_f32(1.0 / self.target_fps as f32);
        
        match self.last_frame_time {
            Some(last) if now.duration_since(last) < interval => false,
            _ => true,
        }
    }
    
    /// Record a frame capture
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        
        if let Some(last) = self.last_frame_time {
            let frame_time = now.duration_since(last);
            
            if self.frame_times.len() >= self.max_samples {
                self.frame_times.pop_front();
            }
            self.frame_times.push_back(frame_time);
        }
        
        self.last_frame_time = Some(now);
    }
    
    /// Calculate actual average FPS
    pub fn actual_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total.as_secs_f32() / self.frame_times.len() as f32;
        
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }
    
    /// Get frame time variance (for stability analysis)
    pub fn frame_time_variance(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }
        
        let times: Vec<f32> = self.frame_times.iter()
            .map(|d| d.as_secs_f32() * 1000.0)
            .collect();
        
        let mean = times.iter().sum::<f32>() / times.len() as f32;
        let variance = times.iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f32>() / times.len() as f32;
        
        variance.sqrt()
    }
}

/// Image stabilization
pub struct ImageStabilizer {
    enabled: bool,
    motion_threshold: f32,
    smoothing_factor: f32,
    offset_x: f32,
    offset_y: f32,
    history: VecDeque<(f32, f32)>,
}

impl ImageStabilizer {
    pub fn new() -> Self {
        Self {
            enabled: true,
            motion_threshold: 2.0,
            smoothing_factor: 0.8,
            offset_x: 0.0,
            offset_y: 0.0,
            history: VecDeque::with_capacity(30),
        }
    }
    
    /// Enable/disable stabilization
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.offset_x = 0.0;
            self.offset_y = 0.0;
            self.history.clear();
        }
    }
    
    /// Check if stabilization is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set motion detection threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.motion_threshold = threshold.max(0.1);
    }
    
    /// Set smoothing factor (0.0 = no smoothing, 1.0 = max smoothing)
    pub fn set_smoothing(&mut self, factor: f32) {
        self.smoothing_factor = factor.clamp(0.0, 0.99);
    }
    
    /// Process motion vector and calculate stabilization offset
    pub fn process_motion(&mut self, motion_x: f32, motion_y: f32) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }
        
        // Store motion history
        if self.history.len() >= 30 {
            self.history.pop_front();
        }
        self.history.push_back((motion_x, motion_y));
        
        // Calculate smoothed motion
        let (avg_x, avg_y) = if !self.history.is_empty() {
            let sum: (f32, f32) = self.history.iter()
                .fold((0.0, 0.0), |acc, &(x, y)| (acc.0 + x, acc.1 + y));
            (sum.0 / self.history.len() as f32, sum.1 / self.history.len() as f32)
        } else {
            (motion_x, motion_y)
        };
        
        // Apply threshold
        let filtered_x = if avg_x.abs() > self.motion_threshold { avg_x } else { 0.0 };
        let filtered_y = if avg_y.abs() > self.motion_threshold { avg_y } else { 0.0 };
        
        // Smooth the offset
        self.offset_x = self.offset_x * self.smoothing_factor + 
                        filtered_x * (1.0 - self.smoothing_factor);
        self.offset_y = self.offset_y * self.smoothing_factor + 
                        filtered_y * (1.0 - self.smoothing_factor);
        
        (-self.offset_x, -self.offset_y)
    }
    
    /// Get current stabilization offset
    pub fn get_offset(&self) -> (f32, f32) {
        (-self.offset_x, -self.offset_y)
    }
    
    /// Reset stabilization state
    pub fn reset(&mut self) {
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.history.clear();
    }
}

impl Default for ImageStabilizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame(camera_id: CameraId, frame_number: u64) -> CameraFrame {
        let mut frame = CameraFrame::new(
            camera_id,
            640, 480,
            FrameFormat::RGB,
            vec![0; 640 * 480 * 3],
        );
        frame.frame_number = frame_number;
        frame
    }
    
    #[test]
    fn test_frame_buffer_push_pop() {
        let mut buffer = FrameBuffer::new(3);
        
        buffer.push(create_test_frame(CameraId::Main, 1));
        buffer.push(create_test_frame(CameraId::Main, 2));
        
        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_full());
        
        let frame = buffer.pop().unwrap();
        assert_eq!(frame.frame_number, 1);
    }
    
    #[test]
    fn test_frame_buffer_overflow() {
        let mut buffer = FrameBuffer::new(2);
        
        buffer.push(create_test_frame(CameraId::Main, 1));
        buffer.push(create_test_frame(CameraId::Main, 2));
        buffer.push(create_test_frame(CameraId::Main, 3));
        
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.dropped_frames(), 1);
        
        // Oldest should be frame 2 now
        let frame = buffer.pop().unwrap();
        assert_eq!(frame.frame_number, 2);
    }
    
    #[test]
    fn test_frame_buffer_latest() {
        let mut buffer = FrameBuffer::new(5);
        
        buffer.push(create_test_frame(CameraId::Main, 1));
        buffer.push(create_test_frame(CameraId::Main, 2));
        buffer.push(create_test_frame(CameraId::Main, 3));
        
        let latest = buffer.latest().unwrap();
        assert_eq!(latest.frame_number, 3);
    }
    
    #[test]
    fn test_capture_session() {
        let config = CameraConfig::default();
        let mut session = CaptureSession::new(CameraId::Main, config);
        
        assert!(!session.is_running());
        
        session.start();
        assert!(session.is_running());
        
        session.record_capture();
        session.record_capture();
        assert_eq!(session.frames_captured, 2);
        
        session.stop();
        assert!(!session.is_running());
    }
    
    #[test]
    fn test_capture_session_fps() {
        let mut config = CameraConfig::default();
        config.fps = 60;
        
        let session = CaptureSession::new(CameraId::Main, config);
        
        let interval = session.frame_interval();
        assert!((interval.as_secs_f32() - 1.0/60.0).abs() < 0.001);
    }
    
    #[test]
    fn test_frame_rate_controller() {
        let mut controller = FrameRateController::new(30);
        
        assert_eq!(controller.target_fps(), 30);
        
        // First frame should always be captured
        assert!(controller.should_capture());
        controller.record_frame();
        
        // Immediately after, should not capture
        assert!(!controller.should_capture());
    }
    
    #[test]
    fn test_frame_rate_controller_actual_fps() {
        let mut controller = FrameRateController::new(30);
        
        // Simulate 10 frames at exactly 30 FPS (33.33ms apart)
        let interval = Duration::from_secs_f32(1.0 / 30.0);
        
        controller.record_frame();
        
        // Manually add frame times
        for _ in 0..10 {
            controller.frame_times.push_back(interval);
        }
        
        let fps = controller.actual_fps();
        assert!((fps - 30.0).abs() < 1.0);
    }
    
    #[test]
    fn test_image_stabilizer() {
        let mut stabilizer = ImageStabilizer::new();
        
        assert!(stabilizer.is_enabled());
        
        // Small motion below threshold should be filtered
        stabilizer.set_threshold(5.0);
        let offset = stabilizer.process_motion(2.0, 1.0);
        
        // Motion is below threshold, so offset should be minimal
        assert!(offset.0.abs() < 1.0);
        assert!(offset.1.abs() < 1.0);
    }
    
    #[test]
    fn test_image_stabilizer_large_motion() {
        let mut stabilizer = ImageStabilizer::new();
        stabilizer.set_threshold(1.0);
        stabilizer.set_smoothing(0.0); // No smoothing for predictable test
        
        // Apply large motion multiple times to build up history
        for _ in 0..10 {
            stabilizer.process_motion(10.0, 5.0);
        }
        
        let offset = stabilizer.get_offset();
        
        // Offset should counter the motion
        assert!(offset.0 < 0.0); // Counter positive X motion
        assert!(offset.1 < 0.0); // Counter positive Y motion
    }
    
    #[test]
    fn test_image_stabilizer_disabled() {
        let mut stabilizer = ImageStabilizer::new();
        stabilizer.set_enabled(false);
        
        let offset = stabilizer.process_motion(100.0, 100.0);
        assert_eq!(offset, (0.0, 0.0));
    }
    
    #[test]
    fn test_image_stabilizer_reset() {
        let mut stabilizer = ImageStabilizer::new();
        
        stabilizer.process_motion(10.0, 10.0);
        stabilizer.reset();
        
        assert_eq!(stabilizer.get_offset(), (0.0, 0.0));
    }
    
    #[test]
    fn test_frame_synchronizer_creation() {
        let cameras = vec![CameraId::Main, CameraId::Wide];
        let sync = FrameSynchronizer::new(cameras, Duration::from_millis(10));
        
        assert_eq!(sync.available_groups(), 0);
        assert_eq!(sync.pending_count(), 0);
    }
    
    #[test]
    fn test_frame_buffer_total_captured() {
        let mut buffer = FrameBuffer::new(2);
        
        buffer.push(create_test_frame(CameraId::Main, 1));
        buffer.push(create_test_frame(CameraId::Main, 2));
        buffer.push(create_test_frame(CameraId::Main, 3));
        
        assert_eq!(buffer.total_captured(), 3);
        assert_eq!(buffer.len(), 2);
    }
}
