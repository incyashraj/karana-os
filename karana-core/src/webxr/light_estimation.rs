//! WebXR Light Estimation
//!
//! Provides environmental lighting information for realistic AR rendering.
//! Enables virtual objects to match the lighting of the real world.

use serde::{Deserialize, Serialize};

use super::XRVector3;

/// Light estimation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightEstimationConfig {
    /// Reflection cube map preference
    pub reflection_format: ReflectionFormat,
}

impl Default for LightEstimationConfig {
    fn default() -> Self {
        Self {
            reflection_format: ReflectionFormat::SRgba8,
        }
    }
}

/// Reflection format for environment maps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionFormat {
    /// Standard RGBA 8-bit
    SRgba8,
    /// Linear RGBA 16-bit float
    Rgba16f,
}

/// Complete light estimate for a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightEstimation {
    /// Primary light direction (normalized, pointing toward light)
    pub primary_light_direction: XRVector3,
    /// Primary light intensity (RGB)
    pub primary_light_intensity: XRVector3,
    /// Ambient light spherical harmonics (9 RGB coefficients = 27 values)
    /// Order: L00, L1-1, L10, L11, L2-2, L2-1, L20, L21, L22
    pub spherical_harmonics: [f32; 27],
    /// Ambient intensity estimate (simple average)
    pub ambient_intensity: f32,
    /// Color temperature in Kelvin
    pub color_temperature: Option<f32>,
    /// Confidence (0-1)
    pub confidence: f32,
}

impl Default for LightEstimation {
    fn default() -> Self {
        // Default to neutral daylight-ish lighting
        Self {
            primary_light_direction: XRVector3::new(0.5, 0.8, 0.3).normalize(),
            primary_light_intensity: XRVector3::new(1.0, 1.0, 1.0),
            spherical_harmonics: [
                // L00 (ambient)
                0.5, 0.5, 0.5,
                // L1-1, L10, L11
                0.0, 0.0, 0.0,
                0.2, 0.2, 0.2,
                0.0, 0.0, 0.0,
                // L2-2, L2-1, L20, L21, L22
                0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
                0.1, 0.1, 0.1,
                0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
            ],
            ambient_intensity: 0.5,
            color_temperature: Some(6500.0), // Daylight
            confidence: 0.8,
        }
    }
}

impl LightEstimation {
    /// Create from a simple directional light
    pub fn from_directional(direction: XRVector3, intensity: f32, color: XRVector3) -> Self {
        let dir = direction.normalize();
        let intensity_f64 = intensity as f64;
        let scaled_color = XRVector3::new(
            color.x * intensity_f64,
            color.y * intensity_f64,
            color.z * intensity_f64,
        );
        
        let sh = Self::compute_sh_from_directional(&dir, intensity, &color);
        
        Self {
            primary_light_direction: dir,
            primary_light_intensity: scaled_color,
            spherical_harmonics: sh,
            ambient_intensity: intensity * 0.3, // Assume some ambient
            color_temperature: None,
            confidence: 0.7,
        }
    }
    
    /// Compute spherical harmonics from a directional light
    fn compute_sh_from_directional(dir: &XRVector3, intensity: f32, color: &XRVector3) -> [f32; 27] {
        let mut sh = [0.0f32; 27];
        
        // SH basis functions evaluated at light direction
        let y00 = 0.282095f32;
        let y1_1 = 0.488603 * dir.y as f32;
        let y10 = 0.488603 * dir.z as f32;
        let y11 = 0.488603 * dir.x as f32;
        
        // Scale by intensity
        let scale = intensity * 0.5;
        
        // L00 - ambient term
        sh[0] = color.x as f32 * y00 * scale;
        sh[1] = color.y as f32 * y00 * scale;
        sh[2] = color.z as f32 * y00 * scale;
        
        // L1 terms - directional
        sh[3] = color.x as f32 * y1_1 * scale;
        sh[4] = color.y as f32 * y1_1 * scale;
        sh[5] = color.z as f32 * y1_1 * scale;
        
        sh[6] = color.x as f32 * y10 * scale;
        sh[7] = color.y as f32 * y10 * scale;
        sh[8] = color.z as f32 * y10 * scale;
        
        sh[9] = color.x as f32 * y11 * scale;
        sh[10] = color.y as f32 * y11 * scale;
        sh[11] = color.z as f32 * y11 * scale;
        
        // L2 terms - set to small values for softer lighting
        for i in 12..27 {
            sh[i] = 0.02;
        }
        
        sh
    }
    
    /// Sample the SH environment at a direction
    pub fn sample_sh(&self, direction: &XRVector3) -> XRVector3 {
        let dir = direction.normalize();
        
        // Evaluate SH basis functions (all f32 for consistency with sh array)
        let y00 = 0.282095f32;
        let y1_1 = 0.488603f32 * dir.y as f32;
        let y10 = 0.488603f32 * dir.z as f32;
        let y11 = 0.488603f32 * dir.x as f32;
        
        let y2_2 = 1.092548f32 * dir.x as f32 * dir.y as f32;
        let y2_1 = 1.092548f32 * dir.y as f32 * dir.z as f32;
        let y20 = 0.315392f32 * (3.0f32 * (dir.z as f32) * (dir.z as f32) - 1.0f32);
        let y21 = 1.092548f32 * dir.x as f32 * dir.z as f32;
        let y22 = 0.546274f32 * ((dir.x as f32) * (dir.x as f32) - (dir.y as f32) * (dir.y as f32));
        
        let sh = &self.spherical_harmonics;
        
        XRVector3 {
            x: (sh[0] * y00 + sh[3] * y1_1 + sh[6] * y10 + sh[9] * y11 +
               sh[12] * y2_2 + sh[15] * y2_1 + sh[18] * y20 + sh[21] * y21 + sh[24] * y22) as f64,
            y: (sh[1] * y00 + sh[4] * y1_1 + sh[7] * y10 + sh[10] * y11 +
               sh[13] * y2_2 + sh[16] * y2_1 + sh[19] * y20 + sh[22] * y21 + sh[25] * y22) as f64,
            z: (sh[2] * y00 + sh[5] * y1_1 + sh[8] * y10 + sh[11] * y11 +
               sh[14] * y2_2 + sh[17] * y2_1 + sh[20] * y20 + sh[23] * y21 + sh[26] * y22) as f64,
        }
    }
    
    /// Get estimated shadow intensity (0 = full shadow, 1 = no shadow)
    pub fn shadow_intensity(&self, surface_normal: &XRVector3) -> f32 {
        let normal = surface_normal.normalize();
        let n_dot_l = normal.dot(&self.primary_light_direction);
        
        // Remap from [-1, 1] to [0, 1] with bias toward lit
        ((n_dot_l as f32 + 0.2).max(0.0).min(1.0))
    }
}

/// Light probe for IBL (Image-Based Lighting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightProbe {
    /// Position in world space
    pub position: XRVector3,
    /// Spherical harmonics at this position
    pub spherical_harmonics: [f32; 27],
    /// Reflection cube map ID (if available)
    pub reflection_map_id: Option<u32>,
    /// Capture radius
    pub radius: f32,
}

impl LightProbe {
    /// Create a probe at a position
    pub fn new(position: XRVector3) -> Self {
        Self {
            position,
            spherical_harmonics: [0.5; 27], // Neutral
            reflection_map_id: None,
            radius: 5.0, // 5 meter radius
        }
    }
    
    /// Get weight for blending based on distance
    pub fn weight(&self, point: &XRVector3) -> f32 {
        let dx = point.x - self.position.x;
        let dy = point.y - self.position.y;
        let dz = point.z - self.position.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() as f32;
        
        if dist >= self.radius {
            0.0
        } else {
            1.0 - (dist / self.radius)
        }
    }
}

/// Light estimation engine - integrates with camera for real-time estimation
pub struct LightEstimationEngine {
    /// Current estimate
    current: LightEstimation,
    /// Light probes
    probes: Vec<LightProbe>,
    /// Smoothing factor
    smoothing: f32,
    /// Enabled state
    enabled: bool,
}

impl LightEstimationEngine {
    /// Create a new engine
    pub fn new() -> Self {
        Self {
            current: LightEstimation::default(),
            probes: vec![],
            smoothing: 0.1,
            enabled: true,
        }
    }
    
    /// Enable/disable estimation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Get current estimate
    pub fn estimate(&self) -> &LightEstimation {
        &self.current
    }
    
    /// Add a light probe
    pub fn add_probe(&mut self, probe: LightProbe) {
        self.probes.push(probe);
    }
    
    /// Get interpolated estimate at a position
    pub fn estimate_at(&self, position: &XRVector3) -> LightEstimation {
        if self.probes.is_empty() {
            return self.current.clone();
        }
        
        let mut total_weight = 0.0f32;
        let mut blended_sh = [0.0f32; 27];
        
        for probe in &self.probes {
            let weight = probe.weight(position);
            if weight > 0.0 {
                total_weight += weight;
                for i in 0..27 {
                    blended_sh[i] += probe.spherical_harmonics[i] * weight;
                }
            }
        }
        
        if total_weight > 0.0 {
            for i in 0..27 {
                blended_sh[i] /= total_weight;
            }
        }
        
        LightEstimation {
            spherical_harmonics: blended_sh,
            ..self.current.clone()
        }
    }
    
    /// Update from camera frame (called by system)
    pub fn update_from_camera(&mut self, _brightness: f32, _color_temp: Option<f32>) {
        if !self.enabled {
            return;
        }
        
        // Would analyze camera image to estimate lighting
        // For now, use simulated values
    }
    
    /// Set manual light direction (for testing/override)
    pub fn set_light_direction(&mut self, direction: XRVector3) {
        self.current.primary_light_direction = direction.normalize();
    }
}

impl Default for LightEstimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_light_estimation() {
        let est = LightEstimation::default();
        assert!(est.confidence > 0.0);
        assert!(est.ambient_intensity > 0.0);
    }
    
    #[test]
    fn test_directional_light() {
        let est = LightEstimation::from_directional(
            XRVector3::new(0.0, -1.0, 0.0),
            1.0,
            XRVector3::new(1.0, 1.0, 1.0),
        );
        
        // Light pointing down
        assert!(est.primary_light_direction.y < 0.0);
    }
    
    #[test]
    fn test_sh_sampling() {
        let est = LightEstimation::default();
        
        // Sample up direction
        let up = est.sample_sh(&XRVector3::new(0.0, 1.0, 0.0));
        assert!(up.x >= 0.0);
        assert!(up.y >= 0.0);
        assert!(up.z >= 0.0);
    }
    
    #[test]
    fn test_shadow_intensity() {
        let est = LightEstimation::from_directional(
            XRVector3::new(0.0, 1.0, 0.0), // Light from above
            1.0,
            XRVector3::new(1.0, 1.0, 1.0),
        );
        
        // Surface facing up should be fully lit
        let lit = est.shadow_intensity(&XRVector3::new(0.0, 1.0, 0.0));
        assert!(lit > 0.5);
        
        // Surface facing down should be in shadow
        let shadow = est.shadow_intensity(&XRVector3::new(0.0, -1.0, 0.0));
        assert!(shadow < 0.5);
    }
    
    #[test]
    fn test_light_probe_weight() {
        let probe = LightProbe::new(XRVector3::new(0.0, 0.0, 0.0));
        
        // At probe position
        let w1 = probe.weight(&XRVector3::new(0.0, 0.0, 0.0));
        assert!((w1 - 1.0).abs() < 0.01);
        
        // At edge of radius
        let w2 = probe.weight(&XRVector3::new(probe.radius as f64, 0.0, 0.0));
        assert!(w2.abs() < 0.01);
        
        // Outside radius
        let w3 = probe.weight(&XRVector3::new(10.0, 0.0, 0.0));
        assert_eq!(w3, 0.0);
    }
    
    #[test]
    fn test_light_estimation_engine() {
        let mut engine = LightEstimationEngine::new();
        
        engine.set_light_direction(XRVector3::new(1.0, 0.0, 0.0));
        
        let est = engine.estimate();
        assert!((est.primary_light_direction.x - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_probe_interpolation() {
        let mut engine = LightEstimationEngine::new();
        
        // Place probes within their default 5m radius from origin
        let mut probe1 = LightProbe::new(XRVector3::new(-3.0, 0.0, 0.0));
        probe1.spherical_harmonics[0] = 0.2;
        
        let mut probe2 = LightProbe::new(XRVector3::new(3.0, 0.0, 0.0));
        probe2.spherical_harmonics[0] = 0.8;
        
        engine.add_probe(probe1);
        engine.add_probe(probe2);
        
        // Estimate at origin should blend both probes
        let est = engine.estimate_at(&XRVector3::new(0.0, 0.0, 0.0));
        assert!(est.spherical_harmonics[0] > 0.4);
        assert!(est.spherical_harmonics[0] < 0.6);
    }
}
