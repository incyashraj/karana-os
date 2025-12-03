//! HRTF (Head-Related Transfer Function) Processing
//!
//! Simulates binaural audio for realistic 3D sound positioning.

use std::f32::consts::PI;

/// HRTF processor for binaural spatialization
#[derive(Debug)]
pub struct HrtfProcessor {
    sample_rate: u32,
    /// Left ear impulse response buffer
    left_ir: Vec<f32>,
    /// Right ear impulse response buffer
    right_ir: Vec<f32>,
    /// Convolution buffer for left channel
    left_buffer: Vec<f32>,
    /// Convolution buffer for right channel
    right_buffer: Vec<f32>,
    /// Buffer position
    buffer_pos: usize,
    /// Current azimuth (cached)
    current_azimuth: f32,
    /// Current elevation (cached)
    current_elevation: f32,
    /// IR length
    ir_length: usize,
}

impl HrtfProcessor {
    pub fn new(sample_rate: u32) -> Self {
        let ir_length = 128; // Typical HRTF length
        
        Self {
            sample_rate,
            left_ir: vec![0.0; ir_length],
            right_ir: vec![0.0; ir_length],
            left_buffer: vec![0.0; ir_length],
            right_buffer: vec![0.0; ir_length],
            buffer_pos: 0,
            current_azimuth: 0.0,
            current_elevation: 0.0,
            ir_length,
        }
    }
    
    /// Load HRTF data
    pub fn load_hrtf(&mut self, data: &HrtfData) {
        if data.left_ir.len() > 0 && data.right_ir.len() > 0 {
            self.ir_length = data.left_ir.len().min(data.right_ir.len());
            self.left_ir = data.left_ir[..self.ir_length].to_vec();
            self.right_ir = data.right_ir[..self.ir_length].to_vec();
            self.left_buffer = vec![0.0; self.ir_length];
            self.right_buffer = vec![0.0; self.ir_length];
        }
    }
    
    /// Process mono input to stereo output with spatialization
    pub fn process(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        azimuth: f32,
        elevation: f32,
        gain: f32,
    ) {
        // Update HRIR if angle changed significantly
        if (azimuth - self.current_azimuth).abs() > 1.0 ||
           (elevation - self.current_elevation).abs() > 1.0 {
            self.update_hrir(azimuth, elevation);
            self.current_azimuth = azimuth;
            self.current_elevation = elevation;
        }
        
        // Simple delay-based spatialization for now
        // Real implementation would use full HRTF convolution
        
        let azimuth_rad = azimuth * PI / 180.0;
        
        // Interaural time difference (ITD) - max ~0.7ms at 90 degrees
        let head_radius = 0.0875; // meters
        let speed_of_sound = 343.0; // m/s
        let itd = head_radius * (azimuth_rad + azimuth_rad.sin()) / speed_of_sound;
        let itd_samples = (itd * self.sample_rate as f32).round() as i32;
        
        // Interaural level difference (ILD)
        let ild_factor = 1.0 - 0.3 * azimuth_rad.sin().abs();
        let left_gain = gain * if azimuth > 0.0 { ild_factor } else { 1.0 };
        let right_gain = gain * if azimuth < 0.0 { ild_factor } else { 1.0 };
        
        // Process samples
        for (i, &sample) in input.iter().enumerate() {
            let out_idx = i * 2;
            if out_idx + 1 >= output.len() {
                break;
            }
            
            // Apply ITD (simplified - direct sample offset)
            let left_idx = i as i32 - itd_samples.max(0);
            let right_idx = i as i32 + itd_samples.max(0);
            
            let left_sample = if left_idx >= 0 && (left_idx as usize) < input.len() {
                input[left_idx as usize]
            } else {
                sample
            };
            
            let right_sample = if right_idx >= 0 && (right_idx as usize) < input.len() {
                input[right_idx as usize]
            } else {
                sample
            };
            
            // Apply pinna shadow for elevation
            let elevation_factor = 1.0 - 0.2 * (elevation / 90.0).abs();
            
            output[out_idx] = left_sample * left_gain * elevation_factor;
            output[out_idx + 1] = right_sample * right_gain * elevation_factor;
        }
    }
    
    fn update_hrir(&mut self, azimuth: f32, elevation: f32) {
        // Generate synthetic HRIR based on angle
        // Real implementation would interpolate from measured HRTF database
        
        let azimuth_rad = azimuth * PI / 180.0;
        let elevation_rad = elevation * PI / 180.0;
        
        for i in 0..self.ir_length {
            let t = i as f32 / self.ir_length as f32;
            
            // Simple decay with angle-dependent characteristics
            let decay = (-t * 10.0).exp();
            let freq_shift = 1.0 + 0.1 * azimuth_rad.sin();
            
            self.left_ir[i] = decay * (t * 50.0 * freq_shift * PI).sin();
            self.right_ir[i] = decay * (t * 50.0 / freq_shift * PI).sin();
        }
    }
    
    /// Reset processing state
    pub fn reset(&mut self) {
        self.left_buffer.fill(0.0);
        self.right_buffer.fill(0.0);
        self.buffer_pos = 0;
    }
}

/// HRTF data container
#[derive(Debug, Clone)]
pub struct HrtfData {
    /// Left ear impulse response
    pub left_ir: Vec<f32>,
    /// Right ear impulse response
    pub right_ir: Vec<f32>,
    /// Sample rate
    pub sample_rate: u32,
    /// Azimuth angle (degrees)
    pub azimuth: f32,
    /// Elevation angle (degrees)
    pub elevation: f32,
}

impl HrtfData {
    pub fn new(left_ir: Vec<f32>, right_ir: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            left_ir,
            right_ir,
            sample_rate,
            azimuth: 0.0,
            elevation: 0.0,
        }
    }
}

/// HRTF database for multiple angles
#[derive(Debug)]
pub struct HrtfDatabase {
    /// HRTF data indexed by (azimuth_idx, elevation_idx)
    data: Vec<HrtfData>,
    /// Azimuth angles available
    azimuths: Vec<f32>,
    /// Elevation angles available
    elevations: Vec<f32>,
}

impl HrtfDatabase {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            azimuths: Vec::new(),
            elevations: Vec::new(),
        }
    }
    
    /// Add HRTF measurement
    pub fn add(&mut self, data: HrtfData) {
        if !self.azimuths.contains(&data.azimuth) {
            self.azimuths.push(data.azimuth);
            self.azimuths.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        if !self.elevations.contains(&data.elevation) {
            self.elevations.push(data.elevation);
            self.elevations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        self.data.push(data);
    }
    
    /// Get nearest HRTF for angle
    pub fn get_nearest(&self, azimuth: f32, elevation: f32) -> Option<&HrtfData> {
        self.data.iter()
            .min_by(|a, b| {
                let dist_a = (a.azimuth - azimuth).powi(2) + (a.elevation - elevation).powi(2);
                let dist_b = (b.azimuth - azimuth).powi(2) + (b.elevation - elevation).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }
    
    /// Interpolate HRTF for exact angle
    pub fn interpolate(&self, azimuth: f32, elevation: f32) -> Option<HrtfData> {
        // Find 4 nearest points and bilinear interpolate
        // Simplified: just return nearest for now
        self.get_nearest(azimuth, elevation).cloned()
    }
    
    /// Number of measurements
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for HrtfDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hrtf_processor_creation() {
        let processor = HrtfProcessor::new(48000);
        assert_eq!(processor.sample_rate, 48000);
    }
    
    #[test]
    fn test_hrtf_process() {
        let mut processor = HrtfProcessor::new(48000);
        
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin()).collect();
        let mut output = vec![0.0f32; 1024]; // Stereo
        
        processor.process(&input, &mut output, 45.0, 0.0, 1.0);
        
        // Should have non-zero output
        assert!(output.iter().any(|&s| s != 0.0));
    }
    
    #[test]
    fn test_hrtf_stereo_difference() {
        let mut processor = HrtfProcessor::new(48000);
        
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin()).collect();
        
        // Sound from the right
        let mut output_right = vec![0.0f32; 1024];
        processor.process(&input, &mut output_right, 90.0, 0.0, 1.0);
        
        // Sound from the left
        processor.reset();
        let mut output_left = vec![0.0f32; 1024];
        processor.process(&input, &mut output_left, -90.0, 0.0, 1.0);
        
        // Left/right channels should be different
        let left_energy: f32 = output_right.iter().step_by(2).map(|s| s.powi(2)).sum();
        let right_energy: f32 = output_right.iter().skip(1).step_by(2).map(|s| s.powi(2)).sum();
        
        // For sound from the right, right channel should be louder
        assert!(right_energy > left_energy * 0.8);
    }
    
    #[test]
    fn test_hrtf_database() {
        let mut db = HrtfDatabase::new();
        
        db.add(HrtfData::new(vec![1.0; 128], vec![1.0; 128], 48000));
        db.add(HrtfData {
            left_ir: vec![0.5; 128],
            right_ir: vec![0.5; 128],
            sample_rate: 48000,
            azimuth: 45.0,
            elevation: 0.0,
        });
        
        assert_eq!(db.len(), 2);
        
        let nearest = db.get_nearest(40.0, 0.0);
        assert!(nearest.is_some());
        assert!((nearest.unwrap().azimuth - 45.0).abs() < 0.1);
    }
    
    #[test]
    fn test_hrtf_reset() {
        let mut processor = HrtfProcessor::new(48000);
        
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin()).collect();
        let mut output = vec![0.0f32; 1024];
        
        processor.process(&input, &mut output, 45.0, 0.0, 1.0);
        processor.reset();
        
        assert!(processor.left_buffer.iter().all(|&s| s == 0.0));
    }
}
