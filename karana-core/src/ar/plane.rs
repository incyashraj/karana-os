//! Plane Detection for Kāraṇa OS
//! 
//! Detects horizontal and vertical surfaces for AR content placement.

use super::*;
use nalgebra::{Point3, UnitQuaternion, Vector3};
use std::collections::HashMap;
use std::time::Instant;

/// Plane classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneClassification {
    /// Unknown plane type
    Unknown,
    /// Horizontal plane facing up (floor, table)
    HorizontalUp,
    /// Horizontal plane facing down (ceiling)
    HorizontalDown,
    /// Vertical plane (wall)
    Vertical,
    /// Sloped surface
    Sloped,
}

/// Plane detection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneDetectionMode {
    /// Detect horizontal planes only
    Horizontal,
    /// Detect vertical planes only
    Vertical,
    /// Detect both horizontal and vertical
    Both,
}

/// Detected plane boundary vertex
#[derive(Debug, Clone, Copy)]
pub struct PlaneVertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

/// Detected plane in AR
#[derive(Debug, Clone)]
pub struct SurfacePlane {
    /// Unique plane ID
    pub id: u64,
    /// Plane center position
    pub center: Point3<f32>,
    /// Plane normal
    pub normal: Vector3<f32>,
    /// Plane extent (width, height)
    pub extent: (f32, f32),
    /// Plane boundary vertices
    pub boundary: Vec<PlaneVertex>,
    /// Plane classification
    pub classification: PlaneClassification,
    /// Plane orientation
    pub orientation: UnitQuaternion<f32>,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Is plane updated
    pub updated: bool,
    /// Last update timestamp
    pub last_updated: Instant,
    /// Anchor points on this plane
    pub anchor_count: u32,
}

impl SurfacePlane {
    /// Create new detected plane
    pub fn new(id: u64, center: Point3<f32>, normal: Vector3<f32>) -> Self {
        let classification = Self::classify_normal(&normal);
        let orientation = Self::compute_orientation(&normal);

        Self {
            id,
            center,
            normal: normal.normalize(),
            extent: (0.0, 0.0),
            boundary: Vec::new(),
            classification,
            orientation,
            confidence: 0.5,
            updated: true,
            last_updated: Instant::now(),
            anchor_count: 0,
        }
    }

    /// Classify plane based on normal
    fn classify_normal(normal: &Vector3<f32>) -> PlaneClassification {
        let up_dot = normal.dot(&Vector3::y());
        let abs_up_dot = up_dot.abs();

        if abs_up_dot > 0.9 {
            if up_dot > 0.0 {
                PlaneClassification::HorizontalUp
            } else {
                PlaneClassification::HorizontalDown
            }
        } else if abs_up_dot < 0.1 {
            PlaneClassification::Vertical
        } else {
            PlaneClassification::Sloped
        }
    }

    /// Compute orientation quaternion from normal
    fn compute_orientation(normal: &Vector3<f32>) -> UnitQuaternion<f32> {
        let up = Vector3::y();
        let dot = normal.dot(&up);
        
        if dot.abs() > 0.9999 {
            if dot > 0.0 {
                UnitQuaternion::identity()
            } else {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), std::f32::consts::PI)
            }
        } else {
            let axis = up.cross(normal).normalize();
            let angle = dot.acos();
            UnitQuaternion::from_axis_angle(&nalgebra::Unit::new_normalize(axis), angle)
        }
    }

    /// Get plane equation coefficients (ax + by + cz + d = 0)
    pub fn get_equation(&self) -> (f32, f32, f32, f32) {
        let d = -self.normal.dot(&self.center.coords);
        (self.normal.x, self.normal.y, self.normal.z, d)
    }

    /// Distance from point to plane
    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        let (a, b, c, d) = self.get_equation();
        (a * point.x + b * point.y + c * point.z + d).abs()
    }

    /// Project point onto plane
    pub fn project_point(&self, point: Point3<f32>) -> Point3<f32> {
        let d = self.distance_to_point(point);
        let direction = (self.center - point).normalize();
        let sign = if self.normal.dot(&direction) > 0.0 { 1.0 } else { -1.0 };
        point + self.normal * (d * sign)
    }

    /// Check if point is within plane boundary
    pub fn contains_point(&self, point: Point3<f32>) -> bool {
        // Project to plane first
        let projected = self.project_point(point);
        
        // Check if within extent
        let local = self.orientation.inverse() * (projected - self.center);
        local.x.abs() <= self.extent.0 / 2.0 && local.z.abs() <= self.extent.1 / 2.0
    }

    /// Compute area
    pub fn area(&self) -> f32 {
        self.extent.0 * self.extent.1
    }

    /// Get world transform matrix
    pub fn get_transform(&self) -> Transform {
        Transform {
            position: self.center,
            rotation: self.orientation,
            scale: Vector3::new(self.extent.0, 1.0, self.extent.1),
        }
    }

    /// Is this a floor plane
    pub fn is_floor(&self) -> bool {
        self.classification == PlaneClassification::HorizontalUp && self.center.y < 0.5
    }

    /// Is this a wall plane
    pub fn is_wall(&self) -> bool {
        self.classification == PlaneClassification::Vertical
    }

    /// Is this a ceiling plane
    pub fn is_ceiling(&self) -> bool {
        self.classification == PlaneClassification::HorizontalDown
    }

    /// Is this a table/surface plane
    pub fn is_surface(&self) -> bool {
        self.classification == PlaneClassification::HorizontalUp && self.center.y >= 0.5
    }
}

/// Plane detection configuration
#[derive(Debug, Clone)]
pub struct PlaneDetectionConfig {
    /// Detection mode
    pub mode: PlaneDetectionMode,
    /// Minimum plane area (m²)
    pub min_area: f32,
    /// Maximum plane count
    pub max_planes: usize,
    /// Merge threshold (m)
    pub merge_threshold: f32,
    /// Update interval (seconds)
    pub update_interval: f32,
}

impl Default for PlaneDetectionConfig {
    fn default() -> Self {
        Self {
            mode: PlaneDetectionMode::Both,
            min_area: 0.1,
            max_planes: 50,
            merge_threshold: 0.1,
            update_interval: 0.1,
        }
    }
}

/// Plane detector
#[derive(Debug)]
pub struct PlaneDetector {
    /// Configuration
    config: PlaneDetectionConfig,
    /// Detected planes
    planes: HashMap<u64, SurfacePlane>,
    /// Next plane ID
    next_plane_id: u64,
    /// Point cloud buffer
    point_buffer: Vec<Point3<f32>>,
    /// Normal buffer
    normal_buffer: Vec<Vector3<f32>>,
    /// Last update time
    last_update: Instant,
    /// Is running
    running: bool,
}

impl PlaneDetector {
    /// Create new plane detector
    pub fn new(config: PlaneDetectionConfig) -> Self {
        Self {
            config,
            planes: HashMap::new(),
            next_plane_id: 1,
            point_buffer: Vec::new(),
            normal_buffer: Vec::new(),
            last_update: Instant::now(),
            running: false,
        }
    }

    /// Start plane detection
    pub fn start(&mut self) {
        self.running = true;
        self.last_update = Instant::now();
    }

    /// Stop plane detection
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Add point cloud data
    pub fn add_points(&mut self, points: &[Point3<f32>], normals: &[Vector3<f32>]) {
        self.point_buffer.extend(points);
        self.normal_buffer.extend(normals);

        // Limit buffer size
        const MAX_POINTS: usize = 10000;
        if self.point_buffer.len() > MAX_POINTS {
            self.point_buffer.drain(0..self.point_buffer.len() - MAX_POINTS);
            self.normal_buffer.drain(0..self.normal_buffer.len() - MAX_POINTS);
        }
    }

    /// Update plane detection
    pub fn update(&mut self) -> Vec<u64> {
        if !self.running {
            return Vec::new();
        }

        let elapsed = self.last_update.elapsed().as_secs_f32();
        if elapsed < self.config.update_interval {
            return Vec::new();
        }
        self.last_update = Instant::now();

        let mut updated_ids = Vec::new();

        // Detect planes from point cloud using RANSAC
        let new_planes = self.ransac_detect();

        for plane_params in new_planes {
            let (center, normal, extent, boundary) = plane_params;

            // Check if this plane should be merged with existing
            if let Some(existing_id) = self.find_mergeable_plane(&center, &normal) {
                // Update existing plane
                if let Some(plane) = self.planes.get_mut(&existing_id) {
                    plane.center = center;
                    plane.normal = normal;
                    plane.extent = extent;
                    plane.boundary = boundary;
                    plane.confidence = (plane.confidence + 0.1).min(1.0);
                    plane.updated = true;
                    plane.last_updated = Instant::now();
                    updated_ids.push(existing_id);
                }
            } else if self.planes.len() < self.config.max_planes {
                // Create new plane
                let mut plane = SurfacePlane::new(self.next_plane_id, center, normal);
                plane.extent = extent;
                plane.boundary = boundary;
                
                self.planes.insert(self.next_plane_id, plane);
                updated_ids.push(self.next_plane_id);
                self.next_plane_id += 1;
            }
        }

        // Remove low confidence planes
        self.planes.retain(|_, plane| {
            let age = plane.last_updated.elapsed().as_secs_f32();
            if age > 5.0 {
                plane.confidence -= 0.1;
            }
            plane.confidence > 0.0
        });

        updated_ids
    }

    /// Get all planes
    pub fn get_planes(&self) -> Vec<&SurfacePlane> {
        self.planes.values().collect()
    }

    /// Get plane by ID
    pub fn get_plane(&self, id: u64) -> Option<&SurfacePlane> {
        self.planes.get(&id)
    }

    /// Get horizontal planes
    pub fn get_horizontal_planes(&self) -> Vec<&SurfacePlane> {
        self.planes.values()
            .filter(|p| matches!(p.classification, 
                PlaneClassification::HorizontalUp | PlaneClassification::HorizontalDown))
            .collect()
    }

    /// Get vertical planes
    pub fn get_vertical_planes(&self) -> Vec<&SurfacePlane> {
        self.planes.values()
            .filter(|p| p.classification == PlaneClassification::Vertical)
            .collect()
    }

    /// Get floor plane
    pub fn get_floor(&self) -> Option<&SurfacePlane> {
        self.planes.values().find(|p| p.is_floor())
    }

    /// Hit test ray against planes
    pub fn hit_test(&self, origin: Point3<f32>, direction: Vector3<f32>) -> Option<(u64, Point3<f32>)> {
        let mut closest: Option<(u64, Point3<f32>, f32)> = None;

        for (id, plane) in &self.planes {
            // Ray-plane intersection
            let denom = plane.normal.dot(&direction);
            if denom.abs() < 1e-6 {
                continue;
            }

            let t = (plane.center - origin).dot(&plane.normal) / denom;
            if t < 0.0 {
                continue;
            }

            let hit_point = origin + direction * t;

            // Check if within plane bounds
            if plane.contains_point(hit_point) {
                if closest.is_none() || t < closest.as_ref().unwrap().2 {
                    closest = Some((*id, hit_point, t));
                }
            }
        }

        closest.map(|(id, point, _)| (id, point))
    }

    /// Get plane count
    pub fn plane_count(&self) -> usize {
        self.planes.len()
    }

    /// Clear all planes
    pub fn clear(&mut self) {
        self.planes.clear();
        self.point_buffer.clear();
        self.normal_buffer.clear();
    }

    // Internal methods

    fn ransac_detect(&self) -> Vec<(Point3<f32>, Vector3<f32>, (f32, f32), Vec<PlaneVertex>)> {
        let mut planes = Vec::new();

        if self.point_buffer.len() < 100 {
            return planes;
        }

        // Simple plane detection from point clusters
        let grid_size = 0.5;
        let mut cells: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();

        for (i, point) in self.point_buffer.iter().enumerate() {
            let cell = (
                (point.x / grid_size) as i32,
                (point.y / grid_size) as i32,
                (point.z / grid_size) as i32,
            );
            cells.entry(cell).or_default().push(i);
        }

        for (_cell, indices) in cells {
            if indices.len() < 20 {
                continue;
            }

            // Compute centroid
            let mut centroid = Point3::origin();
            for &i in &indices {
                centroid.coords += self.point_buffer[i].coords;
            }
            centroid.coords /= indices.len() as f32;

            // Compute average normal
            let mut avg_normal = Vector3::zeros();
            for &i in &indices {
                if i < self.normal_buffer.len() {
                    avg_normal += self.normal_buffer[i];
                }
            }
            if avg_normal.norm() < 0.1 {
                avg_normal = Vector3::y(); // Default to up
            }
            avg_normal = avg_normal.normalize();

            // Check if matches detection mode
            let classification = SurfacePlane::classify_normal(&avg_normal);
            let should_include = match self.config.mode {
                PlaneDetectionMode::Horizontal => {
                    matches!(classification, 
                        PlaneClassification::HorizontalUp | PlaneClassification::HorizontalDown)
                }
                PlaneDetectionMode::Vertical => classification == PlaneClassification::Vertical,
                PlaneDetectionMode::Both => true,
            };

            if !should_include {
                continue;
            }

            // Compute extent
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;

            for &i in &indices {
                let p = self.point_buffer[i];
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_z = min_z.min(p.z);
                max_z = max_z.max(p.z);
            }

            let extent = (max_x - min_x, max_z - min_z);
            if extent.0 * extent.1 < self.config.min_area {
                continue;
            }

            // Create boundary
            let boundary = vec![
                PlaneVertex { position: Point3::new(min_x, centroid.y, min_z), normal: avg_normal },
                PlaneVertex { position: Point3::new(max_x, centroid.y, min_z), normal: avg_normal },
                PlaneVertex { position: Point3::new(max_x, centroid.y, max_z), normal: avg_normal },
                PlaneVertex { position: Point3::new(min_x, centroid.y, max_z), normal: avg_normal },
            ];

            planes.push((centroid, avg_normal, extent, boundary));
        }

        planes
    }

    fn find_mergeable_plane(&self, center: &Point3<f32>, normal: &Vector3<f32>) -> Option<u64> {
        for (id, plane) in &self.planes {
            let center_dist = (plane.center - center).norm();
            let normal_dot = plane.normal.dot(normal).abs();

            if center_dist < self.config.merge_threshold && normal_dot > 0.9 {
                return Some(*id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_classification() {
        let up_plane = SurfacePlane::new(1, Point3::origin(), Vector3::y());
        assert_eq!(up_plane.classification, PlaneClassification::HorizontalUp);

        let down_plane = SurfacePlane::new(2, Point3::origin(), -Vector3::y());
        assert_eq!(down_plane.classification, PlaneClassification::HorizontalDown);

        let wall_plane = SurfacePlane::new(3, Point3::origin(), Vector3::x());
        assert_eq!(wall_plane.classification, PlaneClassification::Vertical);
    }

    #[test]
    fn test_plane_distance_to_point() {
        let plane = SurfacePlane::new(1, Point3::origin(), Vector3::y());
        let point = Point3::new(0.0, 5.0, 0.0);
        assert!((plane.distance_to_point(point) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_plane_project_point() {
        let plane = SurfacePlane::new(1, Point3::origin(), Vector3::y());
        let point = Point3::new(1.0, 5.0, 2.0);
        let projected = plane.project_point(point);
        assert!(projected.y.abs() < 0.01);
    }

    #[test]
    fn test_plane_detector_creation() {
        let config = PlaneDetectionConfig::default();
        let detector = PlaneDetector::new(config);
        assert!(!detector.is_running());
    }

    #[test]
    fn test_plane_detector_start_stop() {
        let config = PlaneDetectionConfig::default();
        let mut detector = PlaneDetector::new(config);
        
        detector.start();
        assert!(detector.is_running());
        
        detector.stop();
        assert!(!detector.is_running());
    }

    #[test]
    fn test_plane_area() {
        let mut plane = SurfacePlane::new(1, Point3::origin(), Vector3::y());
        plane.extent = (2.0, 3.0);
        assert!((plane.area() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_plane_is_floor() {
        let mut plane = SurfacePlane::new(1, Point3::new(0.0, 0.0, 0.0), Vector3::y());
        plane.classification = PlaneClassification::HorizontalUp;
        assert!(plane.is_floor());
    }

    #[test]
    fn test_plane_is_wall() {
        let mut plane = SurfacePlane::new(1, Point3::origin(), Vector3::x());
        plane.classification = PlaneClassification::Vertical;
        assert!(plane.is_wall());
    }

    #[test]
    fn test_plane_detection_config_default() {
        let config = PlaneDetectionConfig::default();
        assert_eq!(config.mode, PlaneDetectionMode::Both);
        assert!((config.min_area - 0.1).abs() < 0.01);
    }
}
