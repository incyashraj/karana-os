//! Spatial haptic feedback for directional cues

use nalgebra::Vector3;
use std::f32::consts::PI;

/// Spatial haptics processor
#[derive(Debug)]
pub struct SpatialHaptics {
    /// Listener forward direction
    forward: Vector3<f32>,
    /// Listener up direction
    up: Vector3<f32>,
    /// Head width for stereo separation (meters)
    head_width: f32,
    /// Intensity falloff model
    falloff: FalloffModel,
    /// Enable cross-fade for smooth transitions
    crossfade_enabled: bool,
    /// Crossfade width in degrees
    crossfade_width: f32,
}

/// Intensity falloff models
#[derive(Debug, Clone, Copy)]
pub enum FalloffModel {
    /// Linear falloff
    Linear,
    /// Cosine-based (natural)
    Cosine,
    /// Sharp cutoff
    StepFunction,
}

impl SpatialHaptics {
    /// Create new spatial haptics processor
    pub fn new() -> Self {
        Self {
            forward: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            head_width: 0.15, // 15cm typical head width
            falloff: FalloffModel::Cosine,
            crossfade_enabled: true,
            crossfade_width: 30.0,
        }
    }
    
    /// Update listener orientation
    pub fn update_orientation(&mut self, forward: Vector3<f32>, up: Vector3<f32>) {
        self.forward = forward.normalize();
        self.up = up.normalize();
    }
    
    /// Calculate stereo intensity for direction
    pub fn calculate_stereo_intensity(
        &self,
        direction: Vector3<f32>,
        base_intensity: f32,
    ) -> (f32, f32) {
        let dir = direction.normalize();
        
        // Calculate angle from forward in horizontal plane
        let right = self.forward.cross(&self.up).normalize();
        
        // Project direction onto horizontal plane
        let horizontal = Vector3::new(dir.x, 0.0, dir.z).normalize();
        
        // Calculate horizontal angle (-180 to 180 degrees)
        let forward_component = horizontal.dot(&self.forward);
        let right_component = horizontal.dot(&right);
        let angle = right_component.atan2(forward_component) * 180.0 / PI;
        
        // Calculate left/right intensity based on angle
        let (left, right) = match self.falloff {
            FalloffModel::Linear => {
                let normalized_angle = angle / 180.0; // -1 to 1
                let left = (0.5 - normalized_angle * 0.5).clamp(0.0, 1.0);
                let right = (0.5 + normalized_angle * 0.5).clamp(0.0, 1.0);
                (left, right)
            }
            FalloffModel::Cosine => {
                let angle_rad = angle * PI / 180.0;
                // Left actuator: stronger for negative angles (left side)
                // When angle = -90 (left), left should be max
                let left = ((-angle_rad).sin() + 1.0) / 2.0;
                // Right actuator: stronger for positive angles (right side)
                // When angle = +90 (right), right should be max
                let right = ((angle_rad).sin() + 1.0) / 2.0;
                (left.clamp(0.0, 1.0), right.clamp(0.0, 1.0))
            }
            FalloffModel::StepFunction => {
                if angle < -self.crossfade_width / 2.0 {
                    (1.0, 0.0) // Full left
                } else if angle > self.crossfade_width / 2.0 {
                    (0.0, 1.0) // Full right
                } else {
                    (0.5, 0.5) // Both for center
                }
            }
        };
        
        // Apply crossfade if enabled
        let (left, right) = if self.crossfade_enabled {
            self.apply_crossfade(left, right, angle)
        } else {
            (left, right)
        };
        
        (left * base_intensity, right * base_intensity)
    }
    
    /// Apply crossfade for smoother transitions
    fn apply_crossfade(&self, left: f32, right: f32, _angle: f32) -> (f32, f32) {
        // Ensure minimum intensity when one side is active
        let min_crossfade = 0.1;
        
        let left = if right > 0.5 { left.max(min_crossfade * right) } else { left };
        let right = if left > 0.5 { right.max(min_crossfade * left) } else { right };
        
        (left, right)
    }
    
    /// Calculate direction cue from world position
    pub fn direction_from_position(
        &self,
        source_position: Vector3<f32>,
        listener_position: Vector3<f32>,
    ) -> Vector3<f32> {
        (source_position - listener_position).normalize()
    }
    
    /// Set falloff model
    pub fn set_falloff(&mut self, model: FalloffModel) {
        self.falloff = model;
    }
    
    /// Enable/disable crossfade
    pub fn set_crossfade(&mut self, enabled: bool) {
        self.crossfade_enabled = enabled;
    }
}

impl Default for SpatialHaptics {
    fn default() -> Self {
        Self::new()
    }
}

/// Directional haptic cue
#[derive(Debug, Clone)]
pub struct HapticCue {
    /// Direction in world space
    pub direction: CueDirection,
    /// Intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Pattern to use
    pub pattern: Option<String>,
    /// Priority
    pub priority: u8,
}

impl HapticCue {
    /// Create new cue
    pub fn new(direction: CueDirection, intensity: f32) -> Self {
        Self {
            direction,
            intensity: intensity.clamp(0.0, 1.0),
            duration_ms: 200,
            pattern: None,
            priority: 5,
        }
    }
    
    /// Set duration
    pub fn with_duration(mut self, ms: u32) -> Self {
        self.duration_ms = ms;
        self
    }
    
    /// Set pattern
    pub fn with_pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    /// Convert to world direction
    pub fn to_world_direction(&self) -> Vector3<f32> {
        self.direction.to_vector()
    }
}

/// Simplified cue directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CueDirection {
    /// Directly ahead
    Front,
    /// Behind
    Back,
    /// Left side
    Left,
    /// Right side
    Right,
    /// Front-left
    FrontLeft,
    /// Front-right
    FrontRight,
    /// Back-left
    BackLeft,
    /// Back-right
    BackRight,
    /// Custom direction vector
    Custom(f32, f32, f32),
}

impl CueDirection {
    /// Convert to unit vector
    pub fn to_vector(&self) -> Vector3<f32> {
        let v = match self {
            CueDirection::Front => Vector3::new(0.0, 0.0, -1.0),
            CueDirection::Back => Vector3::new(0.0, 0.0, 1.0),
            CueDirection::Left => Vector3::new(-1.0, 0.0, 0.0),
            CueDirection::Right => Vector3::new(1.0, 0.0, 0.0),
            CueDirection::FrontLeft => Vector3::new(-0.707, 0.0, -0.707),
            CueDirection::FrontRight => Vector3::new(0.707, 0.0, -0.707),
            CueDirection::BackLeft => Vector3::new(-0.707, 0.0, 0.707),
            CueDirection::BackRight => Vector3::new(0.707, 0.0, 0.707),
            CueDirection::Custom(x, y, z) => Vector3::new(*x, *y, *z),
        };
        v.normalize()
    }
    
    /// Create from angle (0 = front, positive = right)
    pub fn from_angle(degrees: f32) -> Self {
        let rad = degrees * PI / 180.0;
        CueDirection::Custom(rad.sin(), 0.0, -rad.cos())
    }
    
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            CueDirection::Front => "ahead",
            CueDirection::Back => "behind",
            CueDirection::Left => "to your left",
            CueDirection::Right => "to your right",
            CueDirection::FrontLeft => "ahead and left",
            CueDirection::FrontRight => "ahead and right",
            CueDirection::BackLeft => "behind and left",
            CueDirection::BackRight => "behind and right",
            CueDirection::Custom(_, _, _) => "nearby",
        }
    }
}

/// Navigation cue generator
#[derive(Debug)]
pub struct NavigationHaptics {
    /// Current heading (degrees, 0 = north)
    heading: f32,
    /// Target bearing (degrees)
    target_bearing: f32,
    /// Distance to target (meters)
    distance: f32,
    /// Enable proximity intensity scaling
    proximity_scaling: bool,
}

impl NavigationHaptics {
    /// Create new navigation haptics
    pub fn new() -> Self {
        Self {
            heading: 0.0,
            target_bearing: 0.0,
            distance: 0.0,
            proximity_scaling: true,
        }
    }
    
    /// Update current heading
    pub fn set_heading(&mut self, heading: f32) {
        self.heading = heading % 360.0;
    }
    
    /// Set navigation target
    pub fn set_target(&mut self, bearing: f32, distance: f32) {
        self.target_bearing = bearing % 360.0;
        self.distance = distance;
    }
    
    /// Generate navigation cue
    pub fn generate_cue(&self, base_intensity: f32) -> HapticCue {
        // Calculate relative bearing
        let mut relative = self.target_bearing - self.heading;
        if relative > 180.0 {
            relative -= 360.0;
        } else if relative < -180.0 {
            relative += 360.0;
        }
        
        // Determine direction
        let direction = CueDirection::from_angle(relative);
        
        // Scale intensity by proximity if enabled
        let intensity = if self.proximity_scaling {
            let proximity_factor = (50.0 / self.distance.max(1.0)).min(1.0);
            base_intensity * (0.5 + proximity_factor * 0.5)
        } else {
            base_intensity
        };
        
        HapticCue::new(direction, intensity)
    }
    
    /// Check if approaching target
    pub fn is_approaching(&self, threshold_meters: f32) -> bool {
        self.distance < threshold_meters
    }
}

impl Default for NavigationHaptics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_haptics_creation() {
        let spatial = SpatialHaptics::new();
        assert!(spatial.crossfade_enabled);
    }
    
    #[test]
    fn test_stereo_intensity_left() {
        let spatial = SpatialHaptics::new();
        let direction = Vector3::new(-1.0, 0.0, 0.0); // Left
        let (left, right) = spatial.calculate_stereo_intensity(direction, 1.0);
        
        // Left should be stronger
        assert!(left > right);
    }
    
    #[test]
    fn test_stereo_intensity_right() {
        let spatial = SpatialHaptics::new();
        let direction = Vector3::new(1.0, 0.0, 0.0); // Right
        let (left, right) = spatial.calculate_stereo_intensity(direction, 1.0);
        
        // Right should be stronger
        assert!(right > left);
    }
    
    #[test]
    fn test_stereo_intensity_front() {
        let spatial = SpatialHaptics::new();
        let direction = Vector3::new(0.0, 0.0, -1.0); // Front
        let (left, right) = spatial.calculate_stereo_intensity(direction, 1.0);
        
        // Should be roughly equal
        assert!((left - right).abs() < 0.2);
    }
    
    #[test]
    fn test_cue_direction_vectors() {
        let front = CueDirection::Front.to_vector();
        let back = CueDirection::Back.to_vector();
        
        // Should be opposite
        let dot = front.dot(&back);
        assert!((dot - (-1.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_cue_from_angle() {
        let right = CueDirection::from_angle(90.0);
        let vec = right.to_vector();
        
        // Should point right
        assert!(vec.x > 0.9);
        assert!(vec.z.abs() < 0.1);
    }
    
    #[test]
    fn test_navigation_haptics() {
        let mut nav = NavigationHaptics::new();
        nav.set_heading(0.0); // Facing north
        nav.set_target(90.0, 100.0); // Target east
        
        let cue = nav.generate_cue(0.8);
        let direction = cue.to_world_direction();
        
        // Should point right (east relative to north)
        assert!(direction.x > 0.5);
    }
    
    #[test]
    fn test_navigation_approaching() {
        let mut nav = NavigationHaptics::new();
        nav.set_target(0.0, 30.0);
        
        assert!(nav.is_approaching(50.0));
        assert!(!nav.is_approaching(20.0));
    }
    
    #[test]
    fn test_haptic_cue_builder() {
        let cue = HapticCue::new(CueDirection::Left, 0.7)
            .with_duration(300)
            .with_pattern("pulse")
            .with_priority(8);
        
        assert_eq!(cue.duration_ms, 300);
        assert_eq!(cue.pattern, Some("pulse".to_string()));
        assert_eq!(cue.priority, 8);
    }
}
