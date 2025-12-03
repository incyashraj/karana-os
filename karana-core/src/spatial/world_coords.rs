//! World Coordinate System
//!
//! Provides a unified coordinate system that bridges:
//! - Local SLAM coordinates (relative to session origin)
//! - Room-relative coordinates (persist across sessions)
//! - GPS coordinates (for outdoor/global positioning)

use serde::{Deserialize, Serialize};

// ============================================================================
// LOCAL COORDINATES
// ============================================================================

/// Local 3D coordinates relative to SLAM session origin
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LocalCoord {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl LocalCoord {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Distance to another point
    pub fn distance_to(&self, other: &LocalCoord) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    
    /// Length/magnitude of vector
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    /// Normalize to unit vector
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len > f32::EPSILON {
            Some(Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            })
        } else {
            None
        }
    }
}

impl std::ops::Add for LocalCoord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub for LocalCoord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// ============================================================================
// GPS COORDINATES
// ============================================================================

/// GPS coordinates for outdoor/global positioning
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GpsCoord {
    /// Latitude in degrees
    pub latitude: f64,
    /// Longitude in degrees
    pub longitude: f64,
    /// Altitude in meters (MSL)
    pub altitude: f32,
    /// Horizontal accuracy in meters
    pub accuracy: f32,
}

impl Default for GpsCoord {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
            accuracy: 100.0, // Unknown
        }
    }
}

impl GpsCoord {
    pub fn new(latitude: f64, longitude: f64, altitude: f32) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
            accuracy: 10.0,
        }
    }
    
    /// Haversine distance to another GPS coordinate (meters)
    pub fn distance_to(&self, other: &GpsCoord) -> f64 {
        const EARTH_RADIUS: f64 = 6_371_000.0; // meters
        
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();
        
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        EARTH_RADIUS * c
    }
    
    /// Check if we have good accuracy
    pub fn is_accurate(&self) -> bool {
        self.accuracy < 20.0
    }
}

// ============================================================================
// ROOM IDENTIFIER
// ============================================================================

/// Unique identifier for a mapped room/space
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub String);

impl RoomId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Generate from visual signature hash
    pub fn from_signature(hash: &[u8; 32]) -> Self {
        Self(hex::encode(&hash[..8]))
    }
}

impl Default for RoomId {
    fn default() -> Self {
        Self("unknown".to_string())
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// WORLD POSITION
// ============================================================================

/// Complete world position combining all coordinate systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPosition {
    /// Local SLAM coordinates (always present)
    pub local: LocalCoord,
    /// Room this position belongs to
    pub room_id: Option<RoomId>,
    /// GPS coordinates (for outdoor or coarse indoor)
    pub gps: Option<GpsCoord>,
    /// Floor level (0 = ground floor)
    pub floor: i8,
    /// Coordinate space version (for migrations)
    pub version: u8,
}

impl Default for WorldPosition {
    fn default() -> Self {
        Self {
            local: LocalCoord::default(),
            room_id: None,
            gps: None,
            floor: 0,
            version: 1,
        }
    }
}

impl WorldPosition {
    /// Create position with local coordinates only
    pub fn from_local(x: f32, y: f32, z: f32) -> Self {
        Self {
            local: LocalCoord::new(x, y, z),
            ..Default::default()
        }
    }
    
    /// Create position with room
    pub fn in_room(local: LocalCoord, room: RoomId) -> Self {
        Self {
            local,
            room_id: Some(room),
            ..Default::default()
        }
    }
    
    /// Create outdoor position with GPS
    pub fn outdoor(gps: GpsCoord) -> Self {
        Self {
            local: LocalCoord::default(),
            gps: Some(gps),
            ..Default::default()
        }
    }
    
    /// Distance to another position
    /// Uses local coords if same room, GPS if available, else infinity
    pub fn distance_to(&self, other: &WorldPosition) -> f32 {
        // Same room - use local coords (including both being None)
        if self.room_id == other.room_id {
            return self.local.distance_to(&other.local);
        }
        
        // Different rooms but have GPS
        if let (Some(gps1), Some(gps2)) = (&self.gps, &other.gps) {
            return gps1.distance_to(gps2) as f32;
        }
        
        // Can't compute distance
        f32::INFINITY
    }
    
    /// Check if this position is outdoors (has GPS but no room)
    pub fn is_outdoor(&self) -> bool {
        self.room_id.is_none() && self.gps.is_some()
    }
    
    /// Check if this position is mapped (has room)
    pub fn is_mapped(&self) -> bool {
        self.room_id.is_some()
    }
}

// ============================================================================
// COORDINATE TRANSFORMS
// ============================================================================

/// Transform from one coordinate space to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateTransform {
    /// Translation offset
    pub translation: LocalCoord,
    /// Rotation as quaternion (x, y, z, w)
    pub rotation: [f32; 4],
    /// Scale factor
    pub scale: f32,
}

impl Default for CoordinateTransform {
    fn default() -> Self {
        Self {
            translation: LocalCoord::default(),
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity
            scale: 1.0,
        }
    }
}

impl CoordinateTransform {
    /// Apply transform to a local coordinate
    pub fn apply(&self, coord: &LocalCoord) -> LocalCoord {
        // Scale
        let scaled = LocalCoord {
            x: coord.x * self.scale,
            y: coord.y * self.scale,
            z: coord.z * self.scale,
        };
        
        // Rotate (quaternion rotation)
        let [qx, qy, qz, qw] = self.rotation;
        let rotated = LocalCoord {
            x: (1.0 - 2.0 * (qy * qy + qz * qz)) * scaled.x
                + 2.0 * (qx * qy - qz * qw) * scaled.y
                + 2.0 * (qx * qz + qy * qw) * scaled.z,
            y: 2.0 * (qx * qy + qz * qw) * scaled.x
                + (1.0 - 2.0 * (qx * qx + qz * qz)) * scaled.y
                + 2.0 * (qy * qz - qx * qw) * scaled.z,
            z: 2.0 * (qx * qz - qy * qw) * scaled.x
                + 2.0 * (qy * qz + qx * qw) * scaled.y
                + (1.0 - 2.0 * (qx * qx + qy * qy)) * scaled.z,
        };
        
        // Translate
        rotated + self.translation
    }
    
    /// Create inverse transform
    pub fn inverse(&self) -> Self {
        let inv_scale = 1.0 / self.scale;
        let [qx, qy, qz, qw] = self.rotation;
        let inv_rotation = [-qx, -qy, -qz, qw]; // Quaternion conjugate
        
        // Inverse translation in rotated space
        let inv = Self {
            translation: LocalCoord::default(),
            rotation: inv_rotation,
            scale: inv_scale,
        };
        let inv_translation = inv.apply(&LocalCoord {
            x: -self.translation.x,
            y: -self.translation.y,
            z: -self.translation.z,
        });
        
        Self {
            translation: inv_translation,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }
}

// ============================================================================
// REFERENCE FRAME
// ============================================================================

/// Visual signature of a location for relocalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceFrame {
    /// Unique frame ID
    pub id: u64,
    /// Visual feature descriptors
    pub features: Vec<[u8; 32]>,
    /// Position where captured
    pub position: LocalCoord,
    /// Orientation when captured (quaternion)
    pub orientation: [f32; 4],
    /// Timestamp
    pub timestamp: u64,
}

impl Default for ReferenceFrame {
    fn default() -> Self {
        Self {
            id: 0,
            features: Vec::new(),
            position: LocalCoord::default(),
            orientation: [0.0, 0.0, 0.0, 1.0],
            timestamp: 0,
        }
    }
}

// ============================================================================
// COORDINATE FUSION
// ============================================================================

/// Configuration for coordinate fusion
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// GPS weight (0.0-1.0)
    pub gps_weight: f32,
    /// SLAM weight (0.0-1.0)
    pub slam_weight: f32,
    /// IMU weight (0.0-1.0)
    pub imu_weight: f32,
    /// Minimum GPS accuracy to use (meters)
    pub min_gps_accuracy: f32,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            gps_weight: 0.3,
            slam_weight: 0.6,
            imu_weight: 0.1,
            min_gps_accuracy: 20.0,
        }
    }
}

/// Fuses multiple coordinate sources into unified position
pub struct CoordinateFusion {
    /// Configuration
    config: FusionConfig,
    /// Last GPS reading
    last_gps: Option<GpsCoord>,
    /// Last SLAM position
    last_slam: Option<LocalCoord>,
}

impl CoordinateFusion {
    /// Create new coordinate fusion
    pub fn new(config: FusionConfig) -> Self {
        Self {
            config,
            last_gps: None,
            last_slam: None,
        }
    }
    
    /// Update GPS reading
    pub fn update_gps(&mut self, gps: GpsCoord) {
        if gps.accuracy <= self.config.min_gps_accuracy {
            self.last_gps = Some(gps);
        }
    }
    
    /// Fuse position from SLAM pose
    pub fn fuse_position(&self, pose: &super::slam::Pose) -> WorldPosition {
        WorldPosition {
            local: pose.position,
            room_id: None,
            gps: self.last_gps,
            floor: 0,
            version: 1,
        }
    }
    
    /// Get fused world position
    pub fn get_fused_position(&self) -> WorldPosition {
        WorldPosition {
            local: self.last_slam.unwrap_or_default(),
            room_id: None,
            gps: self.last_gps,
            floor: 0,
            version: 1,
        }
    }
}

impl Default for CoordinateFusion {
    fn default() -> Self {
        Self::new(FusionConfig::default())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_local_coord_distance() {
        let a = LocalCoord::new(0.0, 0.0, 0.0);
        let b = LocalCoord::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_local_coord_ops() {
        let a = LocalCoord::new(1.0, 2.0, 3.0);
        let b = LocalCoord::new(1.0, 1.0, 1.0);
        
        let sum = a + b;
        assert_eq!(sum.x, 2.0);
        assert_eq!(sum.y, 3.0);
        assert_eq!(sum.z, 4.0);
        
        let diff = a - b;
        assert_eq!(diff.x, 0.0);
        assert_eq!(diff.y, 1.0);
        assert_eq!(diff.z, 2.0);
    }
    
    #[test]
    fn test_gps_distance() {
        // San Francisco to Oakland ~13km
        let sf = GpsCoord::new(37.7749, -122.4194, 0.0);
        let oakland = GpsCoord::new(37.8044, -122.2712, 0.0);
        
        let dist = sf.distance_to(&oakland);
        assert!(dist > 10_000.0 && dist < 20_000.0);
    }
    
    #[test]
    fn test_world_position_same_room() {
        let room = RoomId::new("living_room");
        
        let pos1 = WorldPosition::in_room(LocalCoord::new(0.0, 0.0, 0.0), room.clone());
        let pos2 = WorldPosition::in_room(LocalCoord::new(3.0, 4.0, 0.0), room);
        
        assert!((pos1.distance_to(&pos2) - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_world_position_different_rooms() {
        let pos1 = WorldPosition::in_room(
            LocalCoord::new(0.0, 0.0, 0.0),
            RoomId::new("room_a")
        );
        let pos2 = WorldPosition::in_room(
            LocalCoord::new(0.0, 0.0, 0.0),
            RoomId::new("room_b")
        );
        
        // No GPS, can't compute distance
        assert!(pos1.distance_to(&pos2).is_infinite());
    }
    
    #[test]
    fn test_transform_identity() {
        let transform = CoordinateTransform::default();
        let coord = LocalCoord::new(1.0, 2.0, 3.0);
        
        let result = transform.apply(&coord);
        assert!((result.x - 1.0).abs() < 0.001);
        assert!((result.y - 2.0).abs() < 0.001);
        assert!((result.z - 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_transform_translation() {
        let transform = CoordinateTransform {
            translation: LocalCoord::new(10.0, 20.0, 30.0),
            ..Default::default()
        };
        let coord = LocalCoord::new(1.0, 2.0, 3.0);
        
        let result = transform.apply(&coord);
        assert!((result.x - 11.0).abs() < 0.001);
        assert!((result.y - 22.0).abs() < 0.001);
        assert!((result.z - 33.0).abs() < 0.001);
    }
    
    #[test]
    fn test_room_id_from_signature() {
        let hash = [1u8; 32];
        let room = RoomId::from_signature(&hash);
        assert_eq!(room.0, "0101010101010101");
    }
}
