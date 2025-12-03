//! Lighting Estimation
//!
//! Estimates environmental lighting from camera images for realistic AR rendering.

use nalgebra::Vector3;
use std::time::Instant;

/// Lighting estimation engine
#[derive(Debug)]
pub struct LightingEstimator {
    update_rate_hz: f32,
    last_update: Instant,
    history: Vec<LightingSample>,
    history_size: usize,
}

impl LightingEstimator {
    pub fn new(update_rate_hz: f32) -> Self {
        Self {
            update_rate_hz,
            last_update: Instant::now(),
            history: Vec::new(),
            history_size: 10,
        }
    }
    
    /// Estimate ambient light from image
    pub fn estimate_ambient(&mut self, pixels: &[[u8; 4]]) -> AmbientLight {
        if pixels.is_empty() {
            return AmbientLight::default();
        }
        
        // Calculate average color
        let (mut r_sum, mut g_sum, mut b_sum) = (0u64, 0u64, 0u64);
        
        for pixel in pixels {
            r_sum += pixel[0] as u64;
            g_sum += pixel[1] as u64;
            b_sum += pixel[2] as u64;
        }
        
        let count = pixels.len() as f32;
        let r = (r_sum as f32 / count) / 255.0;
        let g = (g_sum as f32 / count) / 255.0;
        let b = (b_sum as f32 / count) / 255.0;
        
        // Apply gamma correction approximation
        let intensity = (r * 0.299 + g * 0.587 + b * 0.114).powf(0.8);
        
        AmbientLight {
            color: [r, g, b],
            intensity: intensity.clamp(0.0, 1.0),
        }
    }
    
    /// Estimate main directional light
    pub fn estimate_main_light(&self, pixels: &[[u8; 4]]) -> Option<DirectionalLight> {
        if pixels.is_empty() {
            return None;
        }
        
        // Simplified: estimate from brightest region
        // Real implementation would use spherical harmonics or shadow analysis
        let (mut max_brightness, mut _max_idx) = (0u32, 0usize);
        
        for (i, pixel) in pixels.iter().enumerate() {
            let brightness = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
            if brightness > max_brightness {
                max_brightness = brightness;
                _max_idx = i;
            }
        }
        
        if max_brightness < 384 { // Below threshold for main light
            return None;
        }
        
        // Default sun-like light from above
        Some(DirectionalLight {
            direction: Vector3::new(-0.3, -1.0, 0.2).normalize(),
            color: [1.0, 0.98, 0.95], // Slightly warm
            intensity: (max_brightness as f32 / 765.0).clamp(0.0, 1.0),
        })
    }
    
    /// Estimate overall brightness
    pub fn estimate_brightness(&self, pixels: &[[u8; 4]]) -> f32 {
        if pixels.is_empty() {
            return 0.5;
        }
        
        let total_luminance: f32 = pixels.iter()
            .map(|p| {
                let r = p[0] as f32 / 255.0;
                let g = p[1] as f32 / 255.0;
                let b = p[2] as f32 / 255.0;
                r * 0.299 + g * 0.587 + b * 0.114
            })
            .sum();
        
        (total_luminance / pixels.len() as f32).clamp(0.0, 1.0)
    }
    
    /// Estimate color temperature in Kelvin
    pub fn estimate_temperature(&self, pixels: &[[u8; 4]]) -> f32 {
        if pixels.is_empty() {
            return 5500.0; // Daylight default
        }
        
        // Calculate average R/B ratio
        let (mut r_sum, mut b_sum) = (0f32, 0f32);
        
        for pixel in pixels {
            r_sum += pixel[0] as f32;
            b_sum += pixel[2] as f32;
        }
        
        if b_sum < 1.0 {
            return 10000.0; // Very warm
        }
        
        let ratio = r_sum / b_sum;
        
        // Map ratio to temperature (rough approximation)
        // Low ratio (blue) -> high temp, high ratio (red) -> low temp
        let temp = if ratio > 1.5 {
            2700.0 // Warm incandescent
        } else if ratio > 1.2 {
            3500.0 // Warm white
        } else if ratio > 0.9 {
            5500.0 // Daylight
        } else if ratio > 0.7 {
            7500.0 // Cool daylight
        } else {
            10000.0 // Blue sky
        };
        
        temp
    }
    
    /// Add a lighting sample to history for smoothing
    pub fn add_sample(&mut self, sample: LightingSample) {
        self.history.push(sample);
        if self.history.len() > self.history_size {
            self.history.remove(0);
        }
        self.last_update = Instant::now();
    }
    
    /// Get smoothed lighting values
    pub fn get_smoothed(&self) -> Option<LightingSample> {
        if self.history.is_empty() {
            return None;
        }
        
        let count = self.history.len() as f32;
        
        let brightness: f32 = self.history.iter().map(|s| s.brightness).sum::<f32>() / count;
        let temperature: f32 = self.history.iter().map(|s| s.temperature).sum::<f32>() / count;
        
        let mut color = [0.0f32; 3];
        for sample in &self.history {
            color[0] += sample.ambient_color[0];
            color[1] += sample.ambient_color[1];
            color[2] += sample.ambient_color[2];
        }
        color[0] /= count;
        color[1] /= count;
        color[2] /= count;
        
        Some(LightingSample {
            brightness,
            temperature,
            ambient_color: color,
            timestamp: Instant::now(),
        })
    }
}

/// Ambient light properties
#[derive(Debug, Clone, Copy)]
pub struct AmbientLight {
    /// RGB color (normalized)
    pub color: [f32; 3],
    /// Light intensity (0-1)
    pub intensity: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            intensity: 0.3,
        }
    }
}

/// Directional light (sun-like)
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// Light direction (normalized)
    pub direction: Vector3<f32>,
    /// RGB color (normalized)
    pub color: [f32; 3],
    /// Light intensity (0-1)
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, -1.0, 0.0),
            color: [1.0, 1.0, 1.0],
            intensity: 0.7,
        }
    }
}

/// Point or spot light probe
#[derive(Debug, Clone)]
pub struct LightProbe {
    /// Position in world space
    pub position: Vector3<f32>,
    /// RGB color
    pub color: [f32; 3],
    /// Light intensity
    pub intensity: f32,
    /// Light type
    pub light_type: LightType,
    /// Influence radius (for point/spot)
    pub radius: f32,
}

/// Types of light sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Point,
    Spot,
    Area,
}

/// Lighting sample for history tracking
#[derive(Debug, Clone)]
pub struct LightingSample {
    pub brightness: f32,
    pub temperature: f32,
    pub ambient_color: [f32; 3],
    pub timestamp: Instant,
}

/// Spherical harmonics coefficients for indirect lighting
#[derive(Debug, Clone)]
pub struct SphericalHarmonics {
    /// L0 (constant)
    pub l0: [f32; 3],
    /// L1 coefficients (linear)
    pub l1: [[f32; 3]; 3],
    /// L2 coefficients (quadratic)
    pub l2: [[f32; 3]; 5],
}

impl Default for SphericalHarmonics {
    fn default() -> Self {
        Self {
            l0: [0.5, 0.5, 0.5],
            l1: [[0.0; 3]; 3],
            l2: [[0.0; 3]; 5],
        }
    }
}

impl SphericalHarmonics {
    /// Evaluate SH at direction
    pub fn evaluate(&self, direction: Vector3<f32>) -> [f32; 3] {
        let d = direction.normalize();
        
        let mut result = self.l0;
        
        // L1 contribution
        result[0] += self.l1[0][0] * d.y + self.l1[1][0] * d.z + self.l1[2][0] * d.x;
        result[1] += self.l1[0][1] * d.y + self.l1[1][1] * d.z + self.l1[2][1] * d.x;
        result[2] += self.l1[0][2] * d.y + self.l1[1][2] * d.z + self.l1[2][2] * d.x;
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lighting_estimator_creation() {
        let estimator = LightingEstimator::new(10.0);
        assert_eq!(estimator.update_rate_hz, 10.0);
    }
    
    #[test]
    fn test_ambient_estimation() {
        let mut estimator = LightingEstimator::new(10.0);
        
        // Bright white pixels
        let pixels: Vec<[u8; 4]> = vec![[255, 255, 255, 255]; 100];
        let ambient = estimator.estimate_ambient(&pixels);
        
        assert!(ambient.intensity > 0.8);
        assert!(ambient.color[0] > 0.9);
    }
    
    #[test]
    fn test_ambient_dark() {
        let mut estimator = LightingEstimator::new(10.0);
        
        // Dark pixels
        let pixels: Vec<[u8; 4]> = vec![[20, 20, 20, 255]; 100];
        let ambient = estimator.estimate_ambient(&pixels);
        
        assert!(ambient.intensity < 0.2);
    }
    
    #[test]
    fn test_brightness_estimation() {
        let estimator = LightingEstimator::new(10.0);
        
        let bright_pixels: Vec<[u8; 4]> = vec![[200, 200, 200, 255]; 100];
        let brightness = estimator.estimate_brightness(&bright_pixels);
        assert!(brightness > 0.7);
        
        let dark_pixels: Vec<[u8; 4]> = vec![[50, 50, 50, 255]; 100];
        let brightness = estimator.estimate_brightness(&dark_pixels);
        assert!(brightness < 0.3);
    }
    
    #[test]
    fn test_temperature_estimation() {
        let estimator = LightingEstimator::new(10.0);
        
        // Warm (red-ish)
        let warm_pixels: Vec<[u8; 4]> = vec![[255, 200, 150, 255]; 100];
        let temp = estimator.estimate_temperature(&warm_pixels);
        assert!(temp < 4000.0);
        
        // Cool (blue-ish)
        let cool_pixels: Vec<[u8; 4]> = vec![[150, 200, 255, 255]; 100];
        let temp = estimator.estimate_temperature(&cool_pixels);
        assert!(temp > 6000.0);
    }
    
    #[test]
    fn test_main_light_detection() {
        let estimator = LightingEstimator::new(10.0);
        
        // Bright scene should detect main light
        let bright_pixels: Vec<[u8; 4]> = vec![[200, 200, 200, 255]; 100];
        let light = estimator.estimate_main_light(&bright_pixels);
        assert!(light.is_some());
        
        // Dark scene should not detect main light
        let dark_pixels: Vec<[u8; 4]> = vec![[30, 30, 30, 255]; 100];
        let light = estimator.estimate_main_light(&dark_pixels);
        assert!(light.is_none());
    }
    
    #[test]
    fn test_lighting_history() {
        let mut estimator = LightingEstimator::new(10.0);
        
        for i in 0..5 {
            estimator.add_sample(LightingSample {
                brightness: i as f32 * 0.2,
                temperature: 5000.0 + i as f32 * 100.0,
                ambient_color: [0.5, 0.5, 0.5],
                timestamp: Instant::now(),
            });
        }
        
        let smoothed = estimator.get_smoothed().unwrap();
        assert!((smoothed.brightness - 0.4).abs() < 0.01);
    }
    
    #[test]
    fn test_spherical_harmonics() {
        let sh = SphericalHarmonics::default();
        let color = sh.evaluate(Vector3::new(0.0, 1.0, 0.0));
        
        assert!(color[0] > 0.0);
        assert!(color[1] > 0.0);
        assert!(color[2] > 0.0);
    }
    
    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight::default();
        assert!((light.direction.norm() - 1.0).abs() < 0.001);
        assert_eq!(light.intensity, 0.7);
    }
}
