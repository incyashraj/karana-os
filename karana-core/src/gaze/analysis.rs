//! Gaze analysis for fixations, saccades, and patterns
//!
//! Implements velocity-based event detection algorithms.

use super::{GazePoint, GazeConfig, GazeEvent, Fixation, Saccade, EyeState};
use std::collections::VecDeque;
use std::time::Instant;

/// Number of samples for velocity calculation
const VELOCITY_WINDOW: usize = 5;

/// Conversion factor from normalized to degrees (approximate)
/// Assumes 90 degree FOV, so 1.0 normalized = 45 degrees from center
const DEG_PER_UNIT: f32 = 90.0;

/// Gaze analysis engine
pub struct GazeAnalyzer {
    config: GazeConfig,
    
    /// Recent gaze samples for analysis
    samples: VecDeque<(GazePoint, u64)>,
    
    /// Current fixation being built
    current_fixation: Option<FixationBuilder>,
    
    /// Last known point (for saccade detection)
    last_point: Option<GazePoint>,
    
    /// Current gaze velocity (degrees/sec)
    velocity: f32,
    
    /// In saccade state
    in_saccade: bool,
    
    /// Saccade start point
    saccade_start: Option<GazePoint>,
    
    /// Saccade start time
    saccade_start_time: Option<u64>,
    
    /// Peak velocity during current saccade
    peak_velocity: f32,
    
    /// Eye state (for blink detection)
    pub eye_state: EyeState,
    
    /// Blink start time
    pub blink_start: Option<Instant>,
}

/// Helper for building fixations
struct FixationBuilder {
    points: Vec<GazePoint>,
    start_ms: u64,
    sum_x: f32,
    sum_y: f32,
}

impl FixationBuilder {
    fn new(point: GazePoint, timestamp: u64) -> Self {
        Self {
            points: vec![point],
            start_ms: timestamp,
            sum_x: point.x,
            sum_y: point.y,
        }
    }
    
    fn add(&mut self, point: GazePoint) {
        self.points.push(point);
        self.sum_x += point.x;
        self.sum_y += point.y;
    }
    
    fn center(&self) -> GazePoint {
        let n = self.points.len() as f32;
        GazePoint::new(self.sum_x / n, self.sum_y / n, 1.0)
    }
    
    fn dispersion(&self) -> f32 {
        let center = self.center();
        let mut max_dist: f32 = 0.0;
        
        for p in &self.points {
            let dist = center.distance_to(p);
            if dist > max_dist {
                max_dist = dist;
            }
        }
        
        max_dist
    }
    
    fn duration_ms(&self, current_time: u64) -> u32 {
        (current_time - self.start_ms) as u32
    }
    
    fn to_fixation(&self, end_time: u64) -> Fixation {
        Fixation {
            center: self.center(),
            start_ms: self.start_ms,
            duration_ms: self.duration_ms(end_time),
            sample_count: self.points.len() as u32,
            dispersion: self.dispersion(),
        }
    }
}

impl GazeAnalyzer {
    pub fn new(config: GazeConfig) -> Self {
        Self {
            config,
            samples: VecDeque::with_capacity(VELOCITY_WINDOW * 2),
            current_fixation: None,
            last_point: None,
            velocity: 0.0,
            in_saccade: false,
            saccade_start: None,
            saccade_start_time: None,
            peak_velocity: 0.0,
            eye_state: EyeState::Open,
            blink_start: None,
        }
    }
    
    /// Analyze new gaze point and return event if detected
    pub fn analyze(&mut self, point: &GazePoint, timestamp: u64) -> Option<GazeEvent> {
        // Add to sample buffer
        self.samples.push_back((*point, timestamp));
        if self.samples.len() > VELOCITY_WINDOW * 2 {
            self.samples.pop_front();
        }
        
        // Calculate velocity
        self.velocity = self.calculate_velocity();
        
        // Track peak velocity during saccade
        if self.in_saccade && self.velocity > self.peak_velocity {
            self.peak_velocity = self.velocity;
        }
        
        // State machine for fixation/saccade detection
        let event = if self.velocity > self.config.saccade_velocity_threshold {
            // High velocity - saccade
            self.handle_saccade(point, timestamp)
        } else {
            // Low velocity - potential fixation
            self.handle_fixation(point, timestamp)
        };
        
        self.last_point = Some(*point);
        event
    }
    
    /// Calculate current gaze velocity in degrees per second
    fn calculate_velocity(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        
        let (newest, newest_time) = self.samples.back().unwrap();
        let (oldest, oldest_time) = self.samples.front().unwrap();
        
        let dt_ms = newest_time.saturating_sub(*oldest_time);
        if dt_ms == 0 {
            return 0.0;
        }
        
        let distance = newest.distance_to(oldest);
        let distance_deg = distance * DEG_PER_UNIT;
        let dt_sec = dt_ms as f32 / 1000.0;
        
        distance_deg / dt_sec
    }
    
    /// Handle high-velocity (saccade) state
    fn handle_saccade(&mut self, point: &GazePoint, timestamp: u64) -> Option<GazeEvent> {
        let mut event = None;
        
        // End current fixation if we had one
        if let Some(builder) = self.current_fixation.take() {
            if builder.points.len() >= 3 {
                let fixation = builder.to_fixation(timestamp);
                if fixation.duration_ms >= self.config.min_fixation_ms {
                    event = Some(GazeEvent::FixationEnd(fixation));
                }
            }
        }
        
        // Start saccade if not already in one
        if !self.in_saccade {
            self.in_saccade = true;
            self.saccade_start = self.last_point;
            self.saccade_start_time = Some(timestamp);
            self.peak_velocity = self.velocity;
        }
        
        event
    }
    
    /// Handle low-velocity (fixation) state
    fn handle_fixation(&mut self, point: &GazePoint, timestamp: u64) -> Option<GazeEvent> {
        // End saccade if we were in one
        if self.in_saccade {
            self.in_saccade = false;
            
            if let (Some(start), Some(start_time)) = (self.saccade_start.take(), self.saccade_start_time.take()) {
                let duration_ms = timestamp.saturating_sub(start_time) as u32;
                let amplitude = start.distance_to(point) * DEG_PER_UNIT;
                
                // Only report meaningful saccades
                if amplitude > 1.0 {
                    let saccade = Saccade {
                        start,
                        end: *point,
                        duration_ms,
                        peak_velocity: self.peak_velocity,
                        amplitude,
                    };
                    
                    self.peak_velocity = 0.0;
                    return Some(GazeEvent::Saccade(saccade));
                }
            }
        }
        
        // Fixation handling
        match &mut self.current_fixation {
            Some(builder) => {
                // Check if point is within dispersion threshold
                let center = builder.center();
                if center.distance_to(point) <= self.config.fixation_dispersion {
                    builder.add(*point);
                    
                    let duration = builder.duration_ms(timestamp);
                    if duration >= self.config.min_fixation_ms {
                        return Some(GazeEvent::FixationUpdate(builder.to_fixation(timestamp)));
                    }
                } else {
                    // Point outside dispersion - end current fixation, start new one
                    let old_fixation = builder.to_fixation(timestamp);
                    self.current_fixation = Some(FixationBuilder::new(*point, timestamp));
                    
                    if old_fixation.duration_ms >= self.config.min_fixation_ms {
                        return Some(GazeEvent::FixationEnd(old_fixation));
                    }
                }
            }
            None => {
                // Start new fixation
                self.current_fixation = Some(FixationBuilder::new(*point, timestamp));
                return Some(GazeEvent::FixationStart(Fixation {
                    center: *point,
                    start_ms: timestamp,
                    duration_ms: 0,
                    sample_count: 1,
                    dispersion: 0.0,
                }));
            }
        }
        
        None
    }
    
    /// Get current velocity
    pub fn velocity(&self) -> f32 {
        self.velocity
    }
    
    /// Check if currently in a fixation
    pub fn is_fixating(&self) -> bool {
        self.current_fixation.is_some()
    }
    
    /// Check if currently in a saccade
    pub fn is_in_saccade(&self) -> bool {
        self.in_saccade
    }
    
    /// Get current fixation info
    pub fn current_fixation(&self) -> Option<Fixation> {
        self.current_fixation.as_ref().map(|b| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            b.to_fixation(now)
        })
    }
}

/// Attention heatmap for visualization
pub struct AttentionHeatmap {
    /// Grid resolution
    pub resolution: (usize, usize),
    /// Accumulated attention values
    cells: Vec<f32>,
    /// Total samples
    sample_count: u64,
}

impl AttentionHeatmap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            resolution: (width, height),
            cells: vec![0.0; width * height],
            sample_count: 0,
        }
    }
    
    /// Add a gaze sample to the heatmap
    pub fn add_sample(&mut self, point: &GazePoint, weight: f32) {
        let (w, h) = self.resolution;
        let col = ((point.x * w as f32).floor() as usize).min(w - 1);
        let row = ((point.y * h as f32).floor() as usize).min(h - 1);
        
        let idx = row * w + col;
        self.cells[idx] += weight;
        self.sample_count += 1;
    }
    
    /// Add fixation (weighted by duration)
    pub fn add_fixation(&mut self, fixation: &Fixation) {
        let weight = fixation.duration_ms as f32 / 100.0;
        self.add_sample(&fixation.center, weight);
    }
    
    /// Get normalized heatmap values (0-1)
    pub fn normalized(&self) -> Vec<f32> {
        let max = self.cells.iter().cloned().fold(0.0f32, f32::max);
        if max <= 0.0 {
            return self.cells.clone();
        }
        
        self.cells.iter().map(|&v| v / max).collect()
    }
    
    /// Get value at position
    pub fn get(&self, x: f32, y: f32) -> f32 {
        let (w, h) = self.resolution;
        let col = ((x * w as f32).floor() as usize).min(w - 1);
        let row = ((y * h as f32).floor() as usize).min(h - 1);
        
        self.cells[row * w + col]
    }
    
    /// Reset heatmap
    pub fn clear(&mut self) {
        self.cells.fill(0.0);
        self.sample_count = 0;
    }
    
    /// Get sample count
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyzer_creation() {
        let analyzer = GazeAnalyzer::new(GazeConfig::default());
        assert!(!analyzer.is_fixating());
        assert!(!analyzer.is_in_saccade());
    }
    
    #[test]
    fn test_fixation_detection() {
        let mut analyzer = GazeAnalyzer::new(GazeConfig::default());
        
        // Send stable gaze points
        let base_point = GazePoint::new(0.5, 0.5, 1.0);
        let mut events = Vec::new();
        
        for i in 0..20 {
            let noise = 0.001 * (i as f32).sin();
            let point = GazePoint::new(0.5 + noise, 0.5 + noise, 1.0);
            if let Some(event) = analyzer.analyze(&point, i * 16) {
                events.push(event);
            }
        }
        
        // Should detect fixation start
        assert!(events.iter().any(|e| matches!(e, GazeEvent::FixationStart(_))));
    }
    
    #[test]
    fn test_saccade_detection() {
        let mut analyzer = GazeAnalyzer::new(GazeConfig::default());
        
        // Start with fixation
        for i in 0..5 {
            analyzer.analyze(&GazePoint::new(0.2, 0.2, 1.0), i * 16);
        }
        
        // Make a large jump
        let event = analyzer.analyze(&GazePoint::new(0.8, 0.8, 1.0), 100);
        
        // Should end fixation or detect saccade
        assert!(event.is_some());
    }
    
    #[test]
    fn test_fixation_builder() {
        let p1 = GazePoint::new(0.5, 0.5, 1.0);
        let p2 = GazePoint::new(0.51, 0.49, 1.0);
        
        let mut builder = FixationBuilder::new(p1, 0);
        builder.add(p2);
        
        let center = builder.center();
        assert!((center.x - 0.505).abs() < 0.01);
        
        let fixation = builder.to_fixation(100);
        assert_eq!(fixation.sample_count, 2);
        assert_eq!(fixation.duration_ms, 100);
    }
    
    #[test]
    fn test_heatmap() {
        let mut heatmap = AttentionHeatmap::new(10, 10);
        
        // Add samples in center
        for _ in 0..10 {
            heatmap.add_sample(&GazePoint::new(0.5, 0.5, 1.0), 1.0);
        }
        
        // Center should have highest value
        let center_val = heatmap.get(0.5, 0.5);
        let corner_val = heatmap.get(0.0, 0.0);
        
        assert!(center_val > corner_val);
    }
    
    #[test]
    fn test_heatmap_normalization() {
        let mut heatmap = AttentionHeatmap::new(4, 4);
        
        heatmap.add_sample(&GazePoint::new(0.5, 0.5, 1.0), 10.0);
        heatmap.add_sample(&GazePoint::new(0.0, 0.0, 1.0), 5.0);
        
        let normalized = heatmap.normalized();
        
        // Max should be 1.0
        let max = normalized.iter().cloned().fold(0.0f32, f32::max);
        assert!((max - 1.0).abs() < 0.001);
    }
}
