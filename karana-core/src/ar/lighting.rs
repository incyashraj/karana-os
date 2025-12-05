//! Light Estimation for Kāraṇa OS
//! 
//! Estimates environmental lighting for realistic AR rendering.

use super::*;
use nalgebra::{Point3, Vector3};
use std::time::Instant;

/// Light type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    /// Ambient light
    Ambient,
    /// Directional light (sun)
    Directional,
    /// Point light
    Point,
    /// Spot light
    Spot,
    /// Area light
    Area,
}

/// Estimated light source
#[derive(Debug, Clone, Copy)]
pub struct EstimatedLight {
    /// Light type
    pub light_type: LightType,
    /// Light position (for point/spot lights)
    pub position: Point3<f32>,
    /// Light direction (for directional/spot lights)
    pub direction: Vector3<f32>,
    /// Light color (RGB, 0.0-1.0)
    pub color: [f32; 3],
    /// Light intensity
    pub intensity: f32,
    /// Spot angle (radians, for spot lights)
    pub spot_angle: f32,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
}

impl EstimatedLight {
    /// Create ambient light
    pub fn ambient(color: [f32; 3], intensity: f32) -> Self {
        Self {
            light_type: LightType::Ambient,
            position: Point3::origin(),
            direction: Vector3::zeros(),
            color,
            intensity,
            spot_angle: 0.0,
            confidence: 1.0,
        }
    }

    /// Create directional light
    pub fn directional(direction: Vector3<f32>, color: [f32; 3], intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Point3::origin(),
            direction: direction.normalize(),
            color,
            intensity,
            spot_angle: 0.0,
            confidence: 1.0,
        }
    }

    /// Create point light
    pub fn point(position: Point3<f32>, color: [f32; 3], intensity: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vector3::zeros(),
            color,
            intensity,
            spot_angle: 0.0,
            confidence: 1.0,
        }
    }

    /// Create spot light
    pub fn spot(position: Point3<f32>, direction: Vector3<f32>, color: [f32; 3], intensity: f32, angle: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            spot_angle: angle,
            confidence: 1.0,
        }
    }
}

/// Spherical harmonics coefficients (L2, 9 coefficients per channel)
#[derive(Debug, Clone, Copy, Default)]
pub struct SphericalHarmonics {
    /// Red channel coefficients
    pub red: [f32; 9],
    /// Green channel coefficients
    pub green: [f32; 9],
    /// Blue channel coefficients
    pub blue: [f32; 9],
}

impl SphericalHarmonics {
    /// Create from uniform color
    pub fn from_uniform(color: [f32; 3]) -> Self {
        let mut sh = Self::default();
        // L0 coefficient for uniform light
        let l0_scale = 0.282095; // Y_0^0 = 1/(2*sqrt(pi))
        sh.red[0] = color[0] / l0_scale;
        sh.green[0] = color[1] / l0_scale;
        sh.blue[0] = color[2] / l0_scale;
        sh
    }

    /// Evaluate SH at direction
    pub fn evaluate(&self, direction: Vector3<f32>) -> [f32; 3] {
        let d = direction.normalize();
        
        // SH basis functions
        let y00 = 0.282095; // 1/(2*sqrt(pi))
        let y1m1 = 0.488603 * d.y; // sqrt(3/(4*pi)) * y
        let y10 = 0.488603 * d.z; // sqrt(3/(4*pi)) * z
        let y1p1 = 0.488603 * d.x; // sqrt(3/(4*pi)) * x
        let y2m2 = 1.092548 * d.x * d.y; // sqrt(15/(4*pi)) * xy
        let y2m1 = 1.092548 * d.y * d.z; // sqrt(15/(4*pi)) * yz
        let y20 = 0.315392 * (3.0 * d.z * d.z - 1.0); // sqrt(5/(16*pi)) * (3z² - 1)
        let y2p1 = 1.092548 * d.x * d.z; // sqrt(15/(4*pi)) * xz
        let y2p2 = 0.546274 * (d.x * d.x - d.y * d.y); // sqrt(15/(16*pi)) * (x² - y²)

        let basis = [y00, y1m1, y10, y1p1, y2m2, y2m1, y20, y2p1, y2p2];

        let mut result = [0.0f32; 3];
        for i in 0..9 {
            result[0] += self.red[i] * basis[i];
            result[1] += self.green[i] * basis[i];
            result[2] += self.blue[i] * basis[i];
        }

        result
    }

    /// Blend with another SH
    pub fn blend(&self, other: &SphericalHarmonics, t: f32) -> SphericalHarmonics {
        let mut result = SphericalHarmonics::default();
        for i in 0..9 {
            result.red[i] = self.red[i] * (1.0 - t) + other.red[i] * t;
            result.green[i] = self.green[i] * (1.0 - t) + other.green[i] * t;
            result.blue[i] = self.blue[i] * (1.0 - t) + other.blue[i] * t;
        }
        result
    }
}

/// Light estimation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightEstimationMode {
    /// Disabled
    Disabled,
    /// Ambient intensity only
    AmbientIntensity,
    /// Environment spherical harmonics
    EnvironmentSH,
    /// HDR environment map
    EnvironmentHDR,
    /// Full light probe
    LightProbe,
}

/// Light estimation result
#[derive(Debug, Clone)]
pub struct LightEstimate {
    /// Estimated ambient intensity (0.0-1.0)
    pub ambient_intensity: f32,
    /// Ambient color temperature (Kelvin)
    pub color_temperature: f32,
    /// Primary light direction
    pub primary_light_direction: Vector3<f32>,
    /// Primary light intensity
    pub primary_light_intensity: f32,
    /// Spherical harmonics
    pub spherical_harmonics: SphericalHarmonics,
    /// Estimated lights
    pub lights: Vec<EstimatedLight>,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl Default for LightEstimate {
    fn default() -> Self {
        Self {
            ambient_intensity: 0.5,
            color_temperature: 6500.0, // Daylight
            primary_light_direction: Vector3::new(0.0, -1.0, 0.0),
            primary_light_intensity: 1.0,
            spherical_harmonics: SphericalHarmonics::default(),
            lights: vec![
                EstimatedLight::ambient([0.3, 0.3, 0.3], 1.0),
                EstimatedLight::directional(Vector3::new(0.0, -1.0, 0.0), [1.0, 1.0, 1.0], 1.0),
            ],
            confidence: 0.5,
            timestamp: Instant::now(),
        }
    }
}

impl LightEstimate {
    /// Get ambient color from temperature
    pub fn get_ambient_color(&self) -> [f32; 3] {
        temperature_to_rgb(self.color_temperature)
    }

    /// Is estimate valid
    pub fn is_valid(&self) -> bool {
        self.confidence > 0.3
    }
}

/// Light estimation configuration
#[derive(Debug, Clone)]
pub struct LightEstimationConfig {
    /// Estimation mode
    pub mode: LightEstimationMode,
    /// Update interval (seconds)
    pub update_interval: f32,
    /// Smooth factor (0.0-1.0)
    pub smooth_factor: f32,
    /// Max detected lights
    pub max_lights: usize,
}

impl Default for LightEstimationConfig {
    fn default() -> Self {
        Self {
            mode: LightEstimationMode::EnvironmentSH,
            update_interval: 0.1,
            smooth_factor: 0.3,
            max_lights: 4,
        }
    }
}

/// Light estimator
#[derive(Debug)]
pub struct LightEstimator {
    /// Configuration
    config: LightEstimationConfig,
    /// Current estimate
    current_estimate: LightEstimate,
    /// Previous estimate (for smoothing)
    previous_estimate: Option<LightEstimate>,
    /// Last update time
    last_update: Instant,
    /// Is running
    running: bool,
    /// Frame buffer for averaging
    intensity_buffer: Vec<f32>,
}

impl LightEstimator {
    /// Create new light estimator
    pub fn new(config: LightEstimationConfig) -> Self {
        Self {
            config,
            current_estimate: LightEstimate::default(),
            previous_estimate: None,
            last_update: Instant::now(),
            running: false,
            intensity_buffer: Vec::new(),
        }
    }

    /// Start estimation
    pub fn start(&mut self) {
        self.running = true;
        self.last_update = Instant::now();
    }

    /// Stop estimation
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Process camera frame for light estimation
    pub fn process_frame(&mut self, image_data: &[u8], width: u32, height: u32) {
        if !self.running || self.config.mode == LightEstimationMode::Disabled {
            return;
        }

        let elapsed = self.last_update.elapsed().as_secs_f32();
        if elapsed < self.config.update_interval {
            return;
        }
        self.last_update = Instant::now();

        // Save previous estimate
        self.previous_estimate = Some(self.current_estimate.clone());

        // Estimate ambient intensity from image luminance
        let luminance = self.compute_average_luminance(image_data, width, height);
        self.intensity_buffer.push(luminance);
        if self.intensity_buffer.len() > 10 {
            self.intensity_buffer.remove(0);
        }

        let avg_luminance: f32 = self.intensity_buffer.iter().sum::<f32>() / self.intensity_buffer.len() as f32;
        self.current_estimate.ambient_intensity = avg_luminance;

        // Estimate color temperature
        self.current_estimate.color_temperature = self.estimate_color_temperature(image_data, width, height);

        // Estimate primary light direction from highlights
        self.current_estimate.primary_light_direction = self.estimate_light_direction(image_data, width, height);

        // Update spherical harmonics
        if self.config.mode == LightEstimationMode::EnvironmentSH {
            self.update_spherical_harmonics(image_data, width, height);
        }

        // Update confidence
        self.current_estimate.confidence = self.compute_confidence();
        self.current_estimate.timestamp = Instant::now();

        // Apply smoothing
        if let Some(prev) = &self.previous_estimate {
            let t = self.config.smooth_factor;
            self.current_estimate.ambient_intensity = 
                prev.ambient_intensity * (1.0 - t) + self.current_estimate.ambient_intensity * t;
            self.current_estimate.color_temperature = 
                prev.color_temperature * (1.0 - t) + self.current_estimate.color_temperature * t;
            self.current_estimate.spherical_harmonics = 
                prev.spherical_harmonics.blend(&self.current_estimate.spherical_harmonics, t);
        }

        // Update light list
        self.update_lights();
    }

    /// Get current estimate
    pub fn get_estimate(&self) -> &LightEstimate {
        &self.current_estimate
    }

    /// Get ambient intensity
    pub fn get_ambient_intensity(&self) -> f32 {
        self.current_estimate.ambient_intensity
    }

    /// Get color temperature
    pub fn get_color_temperature(&self) -> f32 {
        self.current_estimate.color_temperature
    }

    /// Get spherical harmonics
    pub fn get_spherical_harmonics(&self) -> &SphericalHarmonics {
        &self.current_estimate.spherical_harmonics
    }

    /// Get primary light direction
    pub fn get_primary_light_direction(&self) -> Vector3<f32> {
        self.current_estimate.primary_light_direction
    }

    // Internal methods

    fn compute_average_luminance(&self, image_data: &[u8], width: u32, height: u32) -> f32 {
        if image_data.is_empty() {
            return 0.5;
        }

        let step = 16; // Sample every 16th pixel
        let mut total_luminance = 0.0f32;
        let mut count = 0;

        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 2 < image_data.len() {
                    let r = image_data[idx] as f32 / 255.0;
                    let g = image_data[idx + 1] as f32 / 255.0;
                    let b = image_data[idx + 2] as f32 / 255.0;
                    // ITU BT.709 luminance
                    total_luminance += 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_luminance / count as f32
        } else {
            0.5
        }
    }

    fn estimate_color_temperature(&self, image_data: &[u8], width: u32, height: u32) -> f32 {
        if image_data.is_empty() {
            return 6500.0;
        }

        let step = 32;
        let mut r_sum = 0.0f32;
        let mut b_sum = 0.0f32;
        let mut count = 0;

        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 2 < image_data.len() {
                    r_sum += image_data[idx] as f32;
                    b_sum += image_data[idx + 2] as f32;
                    count += 1;
                }
            }
        }

        if count > 0 && b_sum > 0.0 {
            let rb_ratio = r_sum / b_sum;
            // Approximate color temperature from R/B ratio
            let temp = 7000.0 / (rb_ratio + 0.1);
            temp.clamp(2000.0, 10000.0)
        } else {
            6500.0
        }
    }

    fn estimate_light_direction(&self, image_data: &[u8], width: u32, height: u32) -> Vector3<f32> {
        if image_data.is_empty() {
            return Vector3::new(0.0, -1.0, 0.0);
        }

        // Find brightest regions to estimate light direction
        let step = 16;
        let mut weighted_x = 0.0f32;
        let mut weighted_y = 0.0f32;
        let mut total_weight = 0.0f32;

        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 2 < image_data.len() {
                    let luminance = (image_data[idx] as f32 + 
                                    image_data[idx + 1] as f32 + 
                                    image_data[idx + 2] as f32) / (255.0 * 3.0);
                    
                    // Weight by luminance squared to emphasize bright areas
                    let weight = luminance * luminance;
                    weighted_x += (x as f32 - width as f32 / 2.0) * weight;
                    weighted_y += (y as f32 - height as f32 / 2.0) * weight;
                    total_weight += weight;
                }
            }
        }

        if total_weight > 0.0 {
            let cx = weighted_x / total_weight / (width as f32 / 2.0);
            let cy = weighted_y / total_weight / (height as f32 / 2.0);
            
            // Convert to direction (assumes overhead lighting)
            Vector3::new(-cx, -1.0, -cy).normalize()
        } else {
            Vector3::new(0.0, -1.0, 0.0)
        }
    }

    fn update_spherical_harmonics(&mut self, image_data: &[u8], width: u32, height: u32) {
        if image_data.is_empty() {
            return;
        }

        // Simple approximation: compute SH from image assuming equirectangular projection
        let mut sh = SphericalHarmonics::default();
        let step = 8;

        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 2 >= image_data.len() {
                    continue;
                }

                let r = image_data[idx] as f32 / 255.0;
                let g = image_data[idx + 1] as f32 / 255.0;
                let b = image_data[idx + 2] as f32 / 255.0;

                // Map pixel to direction
                let u = x as f32 / width as f32;
                let v = y as f32 / height as f32;
                let theta = u * 2.0 * std::f32::consts::PI;
                let phi = v * std::f32::consts::PI;

                let dx = phi.sin() * theta.cos();
                let dy = phi.cos();
                let dz = phi.sin() * theta.sin();

                // Compute SH basis values
                let basis = compute_sh_basis(dx, dy, dz);

                // Accumulate
                for i in 0..9 {
                    sh.red[i] += r * basis[i];
                    sh.green[i] += g * basis[i];
                    sh.blue[i] += b * basis[i];
                }
            }
        }

        // Normalize
        let count = ((width / step as u32) * (height / step as u32)) as f32;
        for i in 0..9 {
            sh.red[i] /= count;
            sh.green[i] /= count;
            sh.blue[i] /= count;
        }

        self.current_estimate.spherical_harmonics = sh;
    }

    fn compute_confidence(&self) -> f32 {
        // Confidence based on intensity variance
        if self.intensity_buffer.len() < 3 {
            return 0.5;
        }

        let mean: f32 = self.intensity_buffer.iter().sum::<f32>() / self.intensity_buffer.len() as f32;
        let variance: f32 = self.intensity_buffer.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / self.intensity_buffer.len() as f32;

        // Lower variance = higher confidence
        (1.0 - variance.sqrt().min(1.0)).max(0.3)
    }

    fn update_lights(&mut self) {
        self.current_estimate.lights.clear();

        // Add ambient light
        let ambient_color = self.current_estimate.get_ambient_color();
        self.current_estimate.lights.push(
            EstimatedLight::ambient(ambient_color, self.current_estimate.ambient_intensity)
        );

        // Add primary directional light
        self.current_estimate.lights.push(
            EstimatedLight::directional(
                self.current_estimate.primary_light_direction,
                [1.0, 1.0, 1.0],
                self.current_estimate.primary_light_intensity,
            )
        );
    }
}

/// Convert color temperature to RGB
fn temperature_to_rgb(temp: f32) -> [f32; 3] {
    let temp = temp.clamp(1000.0, 40000.0) / 100.0;

    let r = if temp <= 66.0 {
        255.0
    } else {
        let r = 329.698727446 * (temp - 60.0).powf(-0.1332047592);
        r.clamp(0.0, 255.0)
    };

    let g = if temp <= 66.0 {
        let g = 99.4708025861 * temp.ln() - 161.1195681661;
        g.clamp(0.0, 255.0)
    } else {
        let g = 288.1221695283 * (temp - 60.0).powf(-0.0755148492);
        g.clamp(0.0, 255.0)
    };

    let b = if temp >= 66.0 {
        255.0
    } else if temp <= 19.0 {
        0.0
    } else {
        let b = 138.5177312231 * (temp - 10.0).ln() - 305.0447927307;
        b.clamp(0.0, 255.0)
    };

    [r / 255.0, g / 255.0, b / 255.0]
}

/// Compute SH basis values at direction
fn compute_sh_basis(x: f32, y: f32, z: f32) -> [f32; 9] {
    [
        0.282095,                           // Y_0^0
        0.488603 * y,                       // Y_1^-1
        0.488603 * z,                       // Y_1^0
        0.488603 * x,                       // Y_1^1
        1.092548 * x * y,                   // Y_2^-2
        1.092548 * y * z,                   // Y_2^-1
        0.315392 * (3.0 * z * z - 1.0),     // Y_2^0
        1.092548 * x * z,                   // Y_2^1
        0.546274 * (x * x - y * y),         // Y_2^2
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimated_light_ambient() {
        let light = EstimatedLight::ambient([0.5, 0.5, 0.5], 1.0);
        assert_eq!(light.light_type, LightType::Ambient);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_estimated_light_directional() {
        let light = EstimatedLight::directional(Vector3::new(0.0, -1.0, 0.0), [1.0, 1.0, 1.0], 1.0);
        assert_eq!(light.light_type, LightType::Directional);
    }

    #[test]
    fn test_spherical_harmonics_uniform() {
        let sh = SphericalHarmonics::from_uniform([1.0, 1.0, 1.0]);
        assert!(sh.red[0] > 0.0);
    }

    #[test]
    fn test_spherical_harmonics_blend() {
        let sh1 = SphericalHarmonics::from_uniform([1.0, 0.0, 0.0]);
        let sh2 = SphericalHarmonics::from_uniform([0.0, 0.0, 1.0]);
        let blended = sh1.blend(&sh2, 0.5);
        assert!((blended.red[0] - sh1.red[0] / 2.0).abs() < 0.1);
    }

    #[test]
    fn test_light_estimate_default() {
        let estimate = LightEstimate::default();
        assert!(estimate.ambient_intensity > 0.0);
        assert!(!estimate.lights.is_empty());
    }

    #[test]
    fn test_light_estimator_creation() {
        let config = LightEstimationConfig::default();
        let estimator = LightEstimator::new(config);
        assert!(!estimator.is_running());
    }

    #[test]
    fn test_light_estimator_start_stop() {
        let config = LightEstimationConfig::default();
        let mut estimator = LightEstimator::new(config);
        
        estimator.start();
        assert!(estimator.is_running());
        
        estimator.stop();
        assert!(!estimator.is_running());
    }

    #[test]
    fn test_temperature_to_rgb() {
        let warm = temperature_to_rgb(3000.0);
        let cold = temperature_to_rgb(10000.0);
        // Warm light should be more red
        assert!(warm[0] > cold[0] || (warm[0] - cold[0]).abs() < 0.1);
    }

    #[test]
    fn test_compute_sh_basis() {
        let basis = compute_sh_basis(0.0, 1.0, 0.0);
        assert!((basis[0] - 0.282095).abs() < 0.001);
    }
}
