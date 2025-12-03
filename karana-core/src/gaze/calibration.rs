//! Gaze calibration system
//!
//! Handles calibration of eye tracker to improve accuracy.

use super::GazePoint;
use std::collections::HashMap;

/// Number of calibration points
pub const CALIBRATION_POINTS: usize = 9;

/// Calibration point targets (3x3 grid)
pub const CALIBRATION_TARGETS: [(f32, f32); CALIBRATION_POINTS] = [
    (0.1, 0.1), (0.5, 0.1), (0.9, 0.1),  // Top row
    (0.1, 0.5), (0.5, 0.5), (0.9, 0.5),  // Middle row
    (0.1, 0.9), (0.5, 0.9), (0.9, 0.9),  // Bottom row
];

/// Calibration status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationStatus {
    NotStarted,
    InProgress { current_point: usize },
    Complete,
    Failed,
}

/// Calibration quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationQuality {
    Excellent,  // < 0.5 degree error
    Good,       // < 1.0 degree
    Fair,       // < 1.5 degree
    Poor,       // >= 1.5 degree
}

/// Single calibration sample
#[derive(Debug, Clone)]
pub struct CalibrationSample {
    /// Target point (where user should look)
    pub target: GazePoint,
    /// Measured gaze point
    pub measured: GazePoint,
    /// Timestamp
    pub timestamp: u64,
}

/// Calibration data for computing correction
#[derive(Debug, Clone, Default)]
pub struct CalibrationData {
    /// Collected samples
    pub samples: Vec<CalibrationSample>,
    /// Average error per calibration point
    pub point_errors: HashMap<usize, f32>,
}

/// Gaze calibration system
#[derive(Debug, Clone)]
pub struct GazeCalibration {
    status: CalibrationStatus,
    data: CalibrationData,
    
    /// Correction offsets (per quadrant)
    offset_x: [[f32; 3]; 3],
    offset_y: [[f32; 3]; 3],
    
    /// Correction scale
    scale_x: f32,
    scale_y: f32,
    
    /// Quality metrics
    average_error: f32,
    quality: CalibrationQuality,
}

impl Default for GazeCalibration {
    fn default() -> Self {
        Self {
            status: CalibrationStatus::NotStarted,
            data: CalibrationData::default(),
            offset_x: [[0.0; 3]; 3],
            offset_y: [[0.0; 3]; 3],
            scale_x: 1.0,
            scale_y: 1.0,
            average_error: 0.0,
            quality: CalibrationQuality::Poor,
        }
    }
}

impl GazeCalibration {
    /// Start calibration process
    pub fn start(&mut self) {
        self.status = CalibrationStatus::InProgress { current_point: 0 };
        self.data = CalibrationData::default();
    }
    
    /// Get current calibration status
    pub fn status(&self) -> CalibrationStatus {
        self.status
    }
    
    /// Check if calibration is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, CalibrationStatus::Complete)
    }
    
    /// Get current target point (during calibration)
    pub fn current_target(&self) -> Option<GazePoint> {
        match self.status {
            CalibrationStatus::InProgress { current_point } if current_point < CALIBRATION_POINTS => {
                let (x, y) = CALIBRATION_TARGETS[current_point];
                Some(GazePoint::new(x, y, 1.0))
            }
            _ => None,
        }
    }
    
    /// Add a calibration sample
    pub fn add_point(&mut self, target: GazePoint, measured: GazePoint) {
        if let CalibrationStatus::InProgress { current_point } = &mut self.status {
            let sample = CalibrationSample {
                target,
                measured,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            };
            
            self.data.samples.push(sample);
            
            // Move to next point after enough samples
            if self.data.samples.len() >= (*current_point + 1) * 10 {
                *current_point += 1;
            }
            
            if *current_point >= CALIBRATION_POINTS {
                self.status = CalibrationStatus::InProgress { 
                    current_point: CALIBRATION_POINTS 
                };
            }
        }
    }
    
    /// Add multiple samples for current point
    pub fn add_samples(&mut self, samples: &[(GazePoint, GazePoint)]) {
        for (target, measured) in samples {
            self.add_point(*target, *measured);
        }
    }
    
    /// Compute calibration from collected data
    pub fn compute(&mut self) -> bool {
        if self.data.samples.len() < CALIBRATION_POINTS * 5 {
            self.status = CalibrationStatus::Failed;
            return false;
        }
        
        // Group samples by target point
        let mut point_samples: Vec<Vec<&CalibrationSample>> = vec![Vec::new(); CALIBRATION_POINTS];
        
        for sample in &self.data.samples {
            // Find closest target
            let mut min_dist = f32::MAX;
            let mut closest_idx = 0;
            
            for (i, (tx, ty)) in CALIBRATION_TARGETS.iter().enumerate() {
                let dx = sample.target.x - tx;
                let dy = sample.target.y - ty;
                let dist = dx * dx + dy * dy;
                if dist < min_dist {
                    min_dist = dist;
                    closest_idx = i;
                }
            }
            
            point_samples[closest_idx].push(sample);
        }
        
        // Compute offsets for each grid cell
        let mut total_error = 0.0;
        let mut total_count = 0;
        
        for (i, samples) in point_samples.iter().enumerate() {
            if samples.is_empty() {
                continue;
            }
            
            let row = i / 3;
            let col = i % 3;
            
            let (tx, ty) = CALIBRATION_TARGETS[i];
            
            // Average measured position for this target
            let mut avg_x = 0.0;
            let mut avg_y = 0.0;
            
            for s in samples {
                avg_x += s.measured.x;
                avg_y += s.measured.y;
            }
            
            avg_x /= samples.len() as f32;
            avg_y /= samples.len() as f32;
            
            // Offset is difference between target and average measured
            self.offset_x[row][col] = tx - avg_x;
            self.offset_y[row][col] = ty - avg_y;
            
            // Calculate error for this point
            let error = ((tx - avg_x).powi(2) + (ty - avg_y).powi(2)).sqrt();
            self.data.point_errors.insert(i, error);
            
            total_error += error;
            total_count += 1;
        }
        
        if total_count > 0 {
            self.average_error = total_error / total_count as f32;
        }
        
        // Determine quality based on average error
        // Using approximate conversion: 0.01 normalized ≈ 0.5 degrees
        self.quality = match self.average_error {
            e if e < 0.01 => CalibrationQuality::Excellent,
            e if e < 0.02 => CalibrationQuality::Good,
            e if e < 0.03 => CalibrationQuality::Fair,
            _ => CalibrationQuality::Poor,
        };
        
        // Compute overall scale correction
        self.compute_scale();
        
        self.status = CalibrationStatus::Complete;
        true
    }
    
    /// Compute scale correction factors
    fn compute_scale(&mut self) {
        if self.data.samples.is_empty() {
            return;
        }
        
        // Calculate variance in measured vs target
        let mut var_measured_x = 0.0;
        let mut var_target_x = 0.0;
        let mut var_measured_y = 0.0;
        let mut var_target_y = 0.0;
        
        let n = self.data.samples.len() as f32;
        
        // First pass: means
        let mut mean_mx = 0.0;
        let mut mean_my = 0.0;
        let mut mean_tx = 0.0;
        let mut mean_ty = 0.0;
        
        for s in &self.data.samples {
            mean_mx += s.measured.x;
            mean_my += s.measured.y;
            mean_tx += s.target.x;
            mean_ty += s.target.y;
        }
        
        mean_mx /= n;
        mean_my /= n;
        mean_tx /= n;
        mean_ty /= n;
        
        // Second pass: variance
        for s in &self.data.samples {
            var_measured_x += (s.measured.x - mean_mx).powi(2);
            var_measured_y += (s.measured.y - mean_my).powi(2);
            var_target_x += (s.target.x - mean_tx).powi(2);
            var_target_y += (s.target.y - mean_ty).powi(2);
        }
        
        // Scale is ratio of standard deviations
        if var_measured_x > 0.0 {
            self.scale_x = (var_target_x / var_measured_x).sqrt().clamp(0.5, 2.0);
        }
        if var_measured_y > 0.0 {
            self.scale_y = (var_target_y / var_measured_y).sqrt().clamp(0.5, 2.0);
        }
    }
    
    /// Apply calibration correction to a gaze point
    pub fn apply(&self, point: &GazePoint) -> GazePoint {
        if !self.is_complete() {
            return *point;
        }
        
        // Find which grid cell this point is in
        let col = ((point.x * 3.0).floor() as usize).min(2);
        let row = ((point.y * 3.0).floor() as usize).min(2);
        
        // Bilinear interpolation of offsets
        let offset_x = self.interpolate_offset(point.x, point.y, true);
        let offset_y = self.interpolate_offset(point.x, point.y, false);
        
        // Apply scale around center (0.5, 0.5)
        let scaled_x = 0.5 + (point.x - 0.5) * self.scale_x;
        let scaled_y = 0.5 + (point.y - 0.5) * self.scale_y;
        
        // Apply offset
        GazePoint::new(
            (scaled_x + offset_x).clamp(0.0, 1.0),
            (scaled_y + offset_y).clamp(0.0, 1.0),
            point.confidence,
        )
    }
    
    /// Interpolate offset value at given point
    fn interpolate_offset(&self, x: f32, y: f32, is_x: bool) -> f32 {
        let offsets = if is_x { &self.offset_x } else { &self.offset_y };
        
        // Map to grid coordinates (0-2)
        let gx = (x * 2.0).clamp(0.0, 2.0);
        let gy = (y * 2.0).clamp(0.0, 2.0);
        
        let col = gx.floor() as usize;
        let row = gy.floor() as usize;
        
        // Fractional part for interpolation
        let fx = gx - col as f32;
        let fy = gy - row as f32;
        
        // Get corner offsets
        let o00 = offsets[row.min(2)][col.min(2)];
        let o10 = offsets[row.min(2)][(col + 1).min(2)];
        let o01 = offsets[(row + 1).min(2)][col.min(2)];
        let o11 = offsets[(row + 1).min(2)][(col + 1).min(2)];
        
        // Bilinear interpolation
        let top = o00 * (1.0 - fx) + o10 * fx;
        let bottom = o01 * (1.0 - fx) + o11 * fx;
        
        top * (1.0 - fy) + bottom * fy
    }
    
    /// Get calibration quality
    pub fn quality(&self) -> CalibrationQuality {
        self.quality
    }
    
    /// Get average error
    pub fn average_error(&self) -> f32 {
        self.average_error
    }
    
    /// Get error for specific calibration point
    pub fn point_error(&self, index: usize) -> Option<f32> {
        self.data.point_errors.get(&index).copied()
    }
    
    /// Reset calibration
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Create pre-configured calibration (for testing)
    pub fn preconfigured() -> Self {
        let mut cal = Self::default();
        cal.status = CalibrationStatus::Complete;
        cal.quality = CalibrationQuality::Good;
        cal.average_error = 0.015;
        cal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calibration_default() {
        let cal = GazeCalibration::default();
        
        assert!(!cal.is_complete());
        assert_eq!(cal.status(), CalibrationStatus::NotStarted);
    }
    
    #[test]
    fn test_calibration_start() {
        let mut cal = GazeCalibration::default();
        cal.start();
        
        assert_eq!(cal.status(), CalibrationStatus::InProgress { current_point: 0 });
        assert!(cal.current_target().is_some());
    }
    
    #[test]
    fn test_calibration_targets() {
        assert_eq!(CALIBRATION_POINTS, 9);
        assert_eq!(CALIBRATION_TARGETS[0], (0.1, 0.1));  // Top-left
        assert_eq!(CALIBRATION_TARGETS[4], (0.5, 0.5));  // Center
        assert_eq!(CALIBRATION_TARGETS[8], (0.9, 0.9));  // Bottom-right
    }
    
    #[test]
    fn test_calibration_add_samples() {
        let mut cal = GazeCalibration::default();
        cal.start();
        
        // Add samples for all points
        for i in 0..CALIBRATION_POINTS {
            let (tx, ty) = CALIBRATION_TARGETS[i];
            let target = GazePoint::new(tx, ty, 1.0);
            
            // Simulate some measurement error
            for j in 0..12 {
                let noise = 0.005 * (j as f32).sin();
                let measured = GazePoint::new(tx + noise, ty + noise, 0.95);
                cal.add_point(target, measured);
            }
        }
        
        // Compute calibration
        assert!(cal.compute());
        assert!(cal.is_complete());
    }
    
    #[test]
    fn test_calibration_apply() {
        let cal = GazeCalibration::preconfigured();
        
        let input = GazePoint::new(0.5, 0.5, 1.0);
        let output = cal.apply(&input);
        
        // Should be close to input for center point with default offsets
        assert!((output.x - input.x).abs() < 0.1);
        assert!((output.y - input.y).abs() < 0.1);
    }
    
    #[test]
    fn test_calibration_quality() {
        let mut cal = GazeCalibration::default();
        cal.average_error = 0.005;
        cal.quality = CalibrationQuality::Excellent;
        
        assert_eq!(cal.quality(), CalibrationQuality::Excellent);
    }
    
    #[test]
    fn test_calibration_reset() {
        let mut cal = GazeCalibration::preconfigured();
        assert!(cal.is_complete());
        
        cal.reset();
        assert!(!cal.is_complete());
        assert_eq!(cal.status(), CalibrationStatus::NotStarted);
    }
}
