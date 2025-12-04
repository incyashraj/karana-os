// Frame Rate Controller for Kāraṇa OS
// Manages display refresh and rendering frame rate for AR glasses

use std::collections::VecDeque;

/// Target frame rate modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameRateMode {
    /// Fixed frame rate
    Fixed(u32),
    /// Variable rate between min and max
    Variable { min: u32, max: u32 },
    /// Adaptive based on content
    Adaptive,
    /// Match display refresh
    VSync,
}

/// Frame timing statistics
#[derive(Debug, Clone)]
pub struct FrameStats {
    pub frame_number: u64,
    pub frame_time_ms: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub was_dropped: bool,
    pub timestamp: u64,
}

/// V-Sync mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VSyncMode {
    Off,
    On,
    Adaptive,
    Triple,
}

/// Frame pacing strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FramePacing {
    /// Render as fast as possible
    Immediate,
    /// Even frame distribution
    Smooth,
    /// Prioritize low latency
    LowLatency,
}

/// Frame rate controller
pub struct FrameRateController {
    mode: FrameRateMode,
    target_fps: u32,
    current_fps: f32,
    display_refresh_hz: u32,
    vsync: VSyncMode,
    pacing: FramePacing,
    frame_history: VecDeque<FrameStats>,
    history_size: usize,
    dropped_frames: u32,
    total_frames: u64,
    frame_budget_ms: f32,
    adaptive_threshold_high: f32,
    adaptive_threshold_low: f32,
}

impl FrameRateController {
    pub fn new() -> Self {
        Self {
            mode: FrameRateMode::Adaptive,
            target_fps: 60,
            current_fps: 60.0,
            display_refresh_hz: 90, // Typical for AR glasses
            vsync: VSyncMode::Adaptive,
            pacing: FramePacing::Smooth,
            frame_history: VecDeque::with_capacity(120),
            history_size: 120,
            dropped_frames: 0,
            total_frames: 0,
            frame_budget_ms: 16.67,
            adaptive_threshold_high: 0.9,
            adaptive_threshold_low: 0.7,
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps.clamp(15, self.display_refresh_hz);
        self.frame_budget_ms = 1000.0 / self.target_fps as f32;
        self.mode = FrameRateMode::Fixed(fps);
    }

    pub fn get_target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn decrease_target_fps(&mut self) {
        let new_fps = match self.target_fps {
            fps if fps > 60 => 60,
            fps if fps > 45 => 45,
            fps if fps > 30 => 30,
            _ => 15,
        };
        self.set_target_fps(new_fps);
    }

    pub fn increase_target_fps(&mut self) {
        let new_fps = match self.target_fps {
            fps if fps < 30 => 30,
            fps if fps < 45 => 45,
            fps if fps < 60 => 60,
            fps if fps < 90 => 90,
            _ => self.display_refresh_hz,
        };
        self.set_target_fps(new_fps);
    }

    pub fn set_mode(&mut self, mode: FrameRateMode) {
        self.mode = mode;
        match mode {
            FrameRateMode::Fixed(fps) => self.set_target_fps(fps),
            FrameRateMode::VSync => self.set_target_fps(self.display_refresh_hz),
            _ => {}
        }
    }

    pub fn get_mode(&self) -> FrameRateMode {
        self.mode
    }

    pub fn set_display_refresh(&mut self, hz: u32) {
        self.display_refresh_hz = hz;
    }

    pub fn get_display_refresh(&self) -> u32 {
        self.display_refresh_hz
    }

    pub fn set_vsync(&mut self, mode: VSyncMode) {
        self.vsync = mode;
    }

    pub fn get_vsync(&self) -> VSyncMode {
        self.vsync
    }

    pub fn set_pacing(&mut self, pacing: FramePacing) {
        self.pacing = pacing;
    }

    pub fn get_pacing(&self) -> FramePacing {
        self.pacing
    }

    pub fn record_frame(&mut self, stats: FrameStats) {
        self.total_frames += 1;
        if stats.was_dropped {
            self.dropped_frames += 1;
        }

        if self.frame_history.len() >= self.history_size {
            self.frame_history.pop_front();
        }
        self.frame_history.push_back(stats);

        self.update_current_fps();
        self.adapt_frame_rate();
    }

    fn update_current_fps(&mut self) {
        if self.frame_history.len() < 2 {
            return;
        }

        let recent: Vec<_> = self.frame_history.iter()
            .rev()
            .take(30)
            .collect();
        
        if !recent.is_empty() {
            let avg_frame_time: f32 = recent.iter()
                .map(|s| s.frame_time_ms)
                .sum::<f32>() / recent.len() as f32;
            
            if avg_frame_time > 0.0 {
                self.current_fps = 1000.0 / avg_frame_time;
            }
        }
    }

    fn adapt_frame_rate(&mut self) {
        if self.mode != FrameRateMode::Adaptive {
            return;
        }

        let frame_time_ratio = self.get_average_frame_time() / self.frame_budget_ms;

        if frame_time_ratio > self.adaptive_threshold_high {
            self.decrease_target_fps();
        } else if frame_time_ratio < self.adaptive_threshold_low && self.dropped_frames == 0 {
            self.increase_target_fps();
        }
    }

    pub fn get_current_fps(&self) -> f32 {
        self.current_fps
    }

    pub fn get_frame_budget_ms(&self) -> f32 {
        self.frame_budget_ms
    }

    pub fn get_average_frame_time(&self) -> f32 {
        if self.frame_history.is_empty() {
            return self.frame_budget_ms;
        }
        
        let sum: f32 = self.frame_history.iter()
            .map(|s| s.frame_time_ms)
            .sum();
        sum / self.frame_history.len() as f32
    }

    pub fn get_frame_time_variance(&self) -> f32 {
        if self.frame_history.len() < 2 {
            return 0.0;
        }
        
        let avg = self.get_average_frame_time();
        let variance: f32 = self.frame_history.iter()
            .map(|s| (s.frame_time_ms - avg).powi(2))
            .sum::<f32>() / self.frame_history.len() as f32;
        variance.sqrt()
    }

    pub fn get_dropped_frame_count(&self) -> u32 {
        self.dropped_frames
    }

    pub fn get_total_frames(&self) -> u64 {
        self.total_frames
    }

    pub fn get_dropped_frame_ratio(&self) -> f32 {
        if self.total_frames == 0 {
            return 0.0;
        }
        self.dropped_frames as f32 / self.total_frames as f32
    }

    pub fn reset_stats(&mut self) {
        self.frame_history.clear();
        self.dropped_frames = 0;
        self.total_frames = 0;
    }

    pub fn get_percentile_frame_time(&self, percentile: f32) -> f32 {
        if self.frame_history.is_empty() {
            return self.frame_budget_ms;
        }

        let mut times: Vec<f32> = self.frame_history.iter()
            .map(|s| s.frame_time_ms)
            .collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((percentile / 100.0) * (times.len() - 1) as f32) as usize;
        times[index.min(times.len() - 1)]
    }

    pub fn should_skip_frame(&self) -> bool {
        // Skip frame if we're way behind
        self.current_fps < (self.target_fps as f32 * 0.5)
    }

    pub fn get_frame_history(&self) -> &VecDeque<FrameStats> {
        &self.frame_history
    }
}

impl Default for FrameRateController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_controller_creation() {
        let controller = FrameRateController::new();
        assert_eq!(controller.get_target_fps(), 60);
        assert_eq!(controller.get_display_refresh(), 90);
    }

    #[test]
    fn test_set_target_fps() {
        let mut controller = FrameRateController::new();
        controller.set_target_fps(30);
        assert_eq!(controller.get_target_fps(), 30);
        assert!((controller.get_frame_budget_ms() - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_fps_clamping() {
        let mut controller = FrameRateController::new();
        controller.set_target_fps(200); // Above display refresh
        assert_eq!(controller.get_target_fps(), 90);
        
        controller.set_target_fps(5); // Below minimum
        assert_eq!(controller.get_target_fps(), 15);
    }

    #[test]
    fn test_decrease_fps() {
        let mut controller = FrameRateController::new();
        controller.set_target_fps(90);
        controller.decrease_target_fps();
        assert_eq!(controller.get_target_fps(), 60);
        
        controller.decrease_target_fps();
        assert_eq!(controller.get_target_fps(), 45);
    }

    #[test]
    fn test_increase_fps() {
        let mut controller = FrameRateController::new();
        controller.set_target_fps(30);
        controller.increase_target_fps();
        assert_eq!(controller.get_target_fps(), 45);
    }

    #[test]
    fn test_record_frame() {
        let mut controller = FrameRateController::new();
        
        for i in 0..10 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: 16.0,
                cpu_time_ms: 8.0,
                gpu_time_ms: 6.0,
                was_dropped: false,
                timestamp: i * 16,
            };
            controller.record_frame(stats);
        }
        
        assert_eq!(controller.get_total_frames(), 10);
        assert_eq!(controller.get_dropped_frame_count(), 0);
    }

    #[test]
    fn test_dropped_frames() {
        let mut controller = FrameRateController::new();
        
        for i in 0..10 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: 16.0,
                cpu_time_ms: 8.0,
                gpu_time_ms: 6.0,
                was_dropped: i % 3 == 0,
                timestamp: i * 16,
            };
            controller.record_frame(stats);
        }
        
        assert_eq!(controller.get_dropped_frame_count(), 4);
        assert!(controller.get_dropped_frame_ratio() > 0.0);
    }

    #[test]
    fn test_average_frame_time() {
        let mut controller = FrameRateController::new();
        
        for i in 0..5 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: 20.0,
                cpu_time_ms: 10.0,
                gpu_time_ms: 8.0,
                was_dropped: false,
                timestamp: i * 20,
            };
            controller.record_frame(stats);
        }
        
        assert!((controller.get_average_frame_time() - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile_frame_time() {
        let mut controller = FrameRateController::new();
        
        for i in 0..100 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: i as f32 + 1.0,
                cpu_time_ms: 10.0,
                gpu_time_ms: 8.0,
                was_dropped: false,
                timestamp: i * 16,
            };
            controller.record_frame(stats);
        }
        
        let p99 = controller.get_percentile_frame_time(99.0);
        assert!(p99 >= 99.0);
    }

    #[test]
    fn test_vsync_modes() {
        let mut controller = FrameRateController::new();
        controller.set_vsync(VSyncMode::On);
        assert_eq!(controller.get_vsync(), VSyncMode::On);
    }

    #[test]
    fn test_frame_pacing() {
        let mut controller = FrameRateController::new();
        controller.set_pacing(FramePacing::LowLatency);
        assert_eq!(controller.get_pacing(), FramePacing::LowLatency);
    }

    #[test]
    fn test_reset_stats() {
        let mut controller = FrameRateController::new();
        
        for i in 0..10 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: 16.0,
                cpu_time_ms: 8.0,
                gpu_time_ms: 6.0,
                was_dropped: true,
                timestamp: i * 16,
            };
            controller.record_frame(stats);
        }
        
        controller.reset_stats();
        assert_eq!(controller.get_total_frames(), 0);
        assert_eq!(controller.get_dropped_frame_count(), 0);
    }

    #[test]
    fn test_frame_time_variance() {
        let mut controller = FrameRateController::new();
        
        // Consistent frame times should have low variance
        for i in 0..10 {
            let stats = FrameStats {
                frame_number: i,
                frame_time_ms: 16.67,
                cpu_time_ms: 8.0,
                gpu_time_ms: 6.0,
                was_dropped: false,
                timestamp: i * 16,
            };
            controller.record_frame(stats);
        }
        
        let variance = controller.get_frame_time_variance();
        assert!(variance < 1.0);
    }
}
