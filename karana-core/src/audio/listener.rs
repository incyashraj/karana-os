//! Audio Listener
//!
//! Represents the listener (user) position and orientation for spatial audio.

use nalgebra::{Point3, Vector3, UnitQuaternion, Matrix3, Rotation3};
use std::time::Instant;

/// Audio listener (the user's ears)
#[derive(Debug, Clone)]
pub struct AudioListener {
    /// Position in world space
    pub position: Point3<f32>,
    /// Orientation
    pub orientation: UnitQuaternion<f32>,
    /// Velocity for Doppler effect
    pub velocity: Vector3<f32>,
    /// Master gain
    pub gain: f32,
    /// State tracking
    pub state: ListenerState,
    /// Last update time
    last_position: Point3<f32>,
    last_update: Instant,
}

impl AudioListener {
    pub fn new() -> Self {
        Self {
            position: Point3::origin(),
            orientation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            gain: 1.0,
            state: ListenerState::Active,
            last_position: Point3::origin(),
            last_update: Instant::now(),
        }
    }
    
    /// Set position and auto-calculate velocity
    pub fn set_position(&mut self, position: Point3<f32>) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        
        if dt > 0.001 {
            self.velocity = (position - self.last_position) / dt;
        }
        
        self.last_position = self.position;
        self.position = position;
        self.last_update = now;
    }
    
    /// Set orientation
    pub fn set_orientation(&mut self, orientation: UnitQuaternion<f32>) {
        self.orientation = orientation;
    }
    
    /// Set orientation from forward/up vectors
    pub fn set_orientation_from_vectors(&mut self, forward: Vector3<f32>, up: Vector3<f32>) {
        let forward = forward.normalize();
        let up = up.normalize();
        let right = forward.cross(&up).normalize();
        let up = right.cross(&forward).normalize();
        
        let rotation_matrix = Rotation3::from_matrix_unchecked(
            Matrix3::from_columns(&[right, up, -forward])
        );
        self.orientation = UnitQuaternion::from_rotation_matrix(&rotation_matrix);
    }
    
    /// Get forward direction
    pub fn forward(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(0.0, 0.0, -1.0)
    }
    
    /// Get up direction
    pub fn up(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(0.0, 1.0, 0.0)
    }
    
    /// Get right direction
    pub fn right(&self) -> Vector3<f32> {
        self.orientation * Vector3::new(1.0, 0.0, 0.0)
    }
    
    /// Transform world position to listener-local space
    pub fn world_to_local(&self, world_pos: Point3<f32>) -> Point3<f32> {
        let relative = world_pos - self.position;
        Point3::from(self.orientation.inverse() * relative)
    }
    
    /// Transform local position to world space
    pub fn local_to_world(&self, local_pos: Point3<f32>) -> Point3<f32> {
        Point3::from(self.orientation * local_pos.coords + self.position.coords)
    }
    
    /// Calculate direction to a point in listener space
    pub fn direction_to(&self, world_pos: Point3<f32>) -> Vector3<f32> {
        let local = self.world_to_local(world_pos);
        if local.coords.norm() > 0.001 {
            local.coords.normalize()
        } else {
            Vector3::new(0.0, 0.0, -1.0)
        }
    }
    
    /// Calculate distance to a point
    pub fn distance_to(&self, world_pos: Point3<f32>) -> f32 {
        (world_pos - self.position).norm()
    }
    
    /// Set master gain
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 2.0);
    }
    
    /// Mute listener
    pub fn mute(&mut self) {
        self.state = ListenerState::Muted;
    }
    
    /// Unmute listener
    pub fn unmute(&mut self) {
        self.state = ListenerState::Active;
    }
    
    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.state == ListenerState::Muted
    }
    
    /// Get effective gain (considering state)
    pub fn effective_gain(&self) -> f32 {
        match self.state {
            ListenerState::Active => self.gain,
            ListenerState::Muted => 0.0,
            ListenerState::Suspended => 0.0,
        }
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Listener state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerState {
    /// Normal operation
    Active,
    /// Muted
    Muted,
    /// Suspended (audio context suspended)
    Suspended,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_listener_creation() {
        let listener = AudioListener::new();
        assert_eq!(listener.position, Point3::origin());
        assert!((listener.gain - 1.0).abs() < 0.001);
        assert_eq!(listener.state, ListenerState::Active);
    }
    
    #[test]
    fn test_listener_position() {
        let mut listener = AudioListener::new();
        
        listener.set_position(Point3::new(5.0, 0.0, 0.0));
        assert!((listener.position.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_listener_directions() {
        let listener = AudioListener::new();
        
        // Default orientation: looking down -Z
        let forward = listener.forward();
        assert!((forward.z + 1.0).abs() < 0.001);
        
        let up = listener.up();
        assert!((up.y - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_world_to_local() {
        let mut listener = AudioListener::new();
        listener.set_position(Point3::new(10.0, 0.0, 0.0));
        
        // Point in front of listener (in world space)
        let world_pos = Point3::new(10.0, 0.0, -5.0);
        let local = listener.world_to_local(world_pos);
        
        // In local space, should be directly in front (negative Z)
        assert!((local.x).abs() < 0.001);
        assert!((local.z + 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_direction_to() {
        let listener = AudioListener::new();
        
        // Point directly in front
        let dir = listener.direction_to(Point3::new(0.0, 0.0, -10.0));
        assert!((dir.z + 1.0).abs() < 0.01);
        
        // Point to the right
        let dir_right = listener.direction_to(Point3::new(10.0, 0.0, 0.0));
        assert!(dir_right.x > 0.9);
    }
    
    #[test]
    fn test_distance() {
        let listener = AudioListener::new();
        
        let dist = listener.distance_to(Point3::new(3.0, 4.0, 0.0));
        assert!((dist - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_mute() {
        let mut listener = AudioListener::new();
        
        assert!(!listener.is_muted());
        assert!((listener.effective_gain() - 1.0).abs() < 0.001);
        
        listener.mute();
        assert!(listener.is_muted());
        assert!((listener.effective_gain()).abs() < 0.001);
        
        listener.unmute();
        assert!(!listener.is_muted());
    }
    
    #[test]
    fn test_set_gain() {
        let mut listener = AudioListener::new();
        
        listener.set_gain(0.5);
        assert!((listener.gain - 0.5).abs() < 0.001);
        
        // Should clamp to max
        listener.set_gain(10.0);
        assert!((listener.gain - 2.0).abs() < 0.001);
    }
}
