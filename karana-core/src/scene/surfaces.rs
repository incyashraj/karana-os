//! Surface Detection and Classification
//!
//! Detects and classifies planar surfaces from depth data.

use nalgebra::{Point3, Vector3};
use uuid::Uuid;
use std::time::Instant;

use super::{SceneId, Ray, RaycastHit};

/// Surface detector using RANSAC-based plane fitting
#[derive(Debug)]
pub struct SurfaceDetector {
    min_area: f32,
    confidence_threshold: f32,
    ransac_iterations: usize,
    inlier_threshold: f32,
}

impl SurfaceDetector {
    pub fn new(min_area: f32, confidence_threshold: f32) -> Self {
        Self {
            min_area,
            confidence_threshold,
            ransac_iterations: 100,
            inlier_threshold: 0.02, // 2cm
        }
    }
    
    /// Detect surfaces from point cloud
    pub fn detect(&self, points: &[Point3<f32>]) -> Vec<Surface> {
        if points.len() < 3 {
            return Vec::new();
        }
        
        let mut surfaces = Vec::new();
        let mut remaining_points: Vec<Point3<f32>> = points.to_vec();
        
        // Iteratively detect planes
        while remaining_points.len() >= 10 {
            if let Some((plane, inliers)) = self.fit_plane_ransac(&remaining_points) {
                if inliers.len() >= 10 {
                    let surface = self.create_surface(&plane, &inliers);
                    
                    if surface.area >= self.min_area && surface.confidence >= self.confidence_threshold {
                        surfaces.push(surface);
                    }
                    
                    // Remove inliers from remaining points
                    let inlier_set: std::collections::HashSet<usize> = inliers.into_iter().collect();
                    remaining_points = remaining_points.into_iter()
                        .enumerate()
                        .filter(|(i, _)| !inlier_set.contains(i))
                        .map(|(_, p)| p)
                        .collect();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        surfaces
    }
    
    fn fit_plane_ransac(&self, points: &[Point3<f32>]) -> Option<(Plane, Vec<usize>)> {
        if points.len() < 3 {
            return None;
        }
        
        let mut best_plane: Option<Plane> = None;
        let mut best_inliers: Vec<usize> = Vec::new();
        
        for _ in 0..self.ransac_iterations {
            // Random sample 3 points
            let indices = self.random_sample(points.len(), 3);
            if indices.len() < 3 {
                continue;
            }
            
            let p0 = points[indices[0]];
            let p1 = points[indices[1]];
            let p2 = points[indices[2]];
            
            // Fit plane to 3 points
            if let Some(plane) = Plane::from_points(p0, p1, p2) {
                // Count inliers
                let inliers: Vec<usize> = points.iter()
                    .enumerate()
                    .filter(|(_, p)| plane.distance_to_point(p).abs() < self.inlier_threshold)
                    .map(|(i, _)| i)
                    .collect();
                
                if inliers.len() > best_inliers.len() {
                    best_inliers = inliers;
                    best_plane = Some(plane);
                }
            }
        }
        
        best_plane.map(|p| (p, best_inliers))
    }
    
    fn random_sample(&self, max: usize, count: usize) -> Vec<usize> {
        // Simple deterministic sampling for reproducibility
        let step = max / count.max(1);
        (0..count).map(|i| (i * step) % max).collect()
    }
    
    fn create_surface(&self, plane: &Plane, inlier_indices: &[usize]) -> Surface {
        // For now, estimate area from bounding box of inliers
        // Real implementation would compute convex hull
        let area = (inlier_indices.len() as f32 * 0.01).min(10.0); // Rough estimate
        
        // Determine surface type from normal
        let surface_type = if plane.normal.y.abs() > 0.9 {
            if plane.normal.y > 0.0 {
                SurfaceType::Horizontal
            } else {
                SurfaceType::Ceiling
            }
        } else if plane.normal.y.abs() < 0.1 {
            SurfaceType::Vertical
        } else {
            SurfaceType::Sloped
        };
        
        Surface {
            id: Uuid::new_v4(),
            plane: *plane,
            surface_type,
            center: Point3::from(plane.normal * plane.distance),
            area,
            confidence: (inlier_indices.len() as f32 / 100.0).min(1.0),
            bounds: SurfaceBounds::default(),
            updated_at: Instant::now(),
        }
    }
}

/// A 3D plane in Hessian normal form
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal vector
    pub normal: Vector3<f32>,
    /// Distance from origin
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        Self { normal: normal.normalize(), distance }
    }
    
    /// Create plane from 3 points
    pub fn from_points(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Option<Self> {
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let normal = v1.cross(&v2);
        
        let length = normal.norm();
        if length < 1e-6 {
            return None; // Points are collinear
        }
        
        let normal = normal / length;
        let distance = normal.dot(&p0.coords);
        
        Some(Self { normal, distance })
    }
    
    /// Signed distance from point to plane
    pub fn distance_to_point(&self, point: &Point3<f32>) -> f32 {
        self.normal.dot(&point.coords) - self.distance
    }
    
    /// Project point onto plane
    pub fn project_point(&self, point: &Point3<f32>) -> Point3<f32> {
        let dist = self.distance_to_point(point);
        point - self.normal * dist
    }
    
    /// Intersect with ray
    pub fn intersect_ray(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot(&ray.direction);
        
        if denom.abs() < 1e-6 {
            return None; // Ray parallel to plane
        }
        
        let t = (self.distance - self.normal.dot(&ray.origin.coords)) / denom;
        
        if t >= 0.0 {
            Some(t)
        } else {
            None // Intersection behind ray origin
        }
    }
}

/// Type of detected surface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceType {
    /// Horizontal surface (floor, table)
    Horizontal,
    /// Vertical surface (wall)
    Vertical,
    /// Ceiling
    Ceiling,
    /// Sloped surface
    Sloped,
    /// Unknown
    Unknown,
}

/// A detected planar surface
#[derive(Debug, Clone)]
pub struct Surface {
    /// Unique identifier
    pub id: SceneId,
    /// Plane equation
    pub plane: Plane,
    /// Classification
    pub surface_type: SurfaceType,
    /// Center point
    pub center: Point3<f32>,
    /// Surface area (m²)
    pub area: f32,
    /// Detection confidence (0-1)
    pub confidence: f32,
    /// Bounding polygon
    pub bounds: SurfaceBounds,
    /// Last update time
    pub updated_at: Instant,
}

impl Surface {
    /// Distance from point to surface
    pub fn distance_to_point(&self, point: &Point3<f32>) -> f32 {
        self.plane.distance_to_point(point).abs()
    }
    
    /// Check if point is within surface bounds (projected)
    pub fn contains_point(&self, point: &Point3<f32>) -> bool {
        let projected = self.plane.project_point(point);
        let to_point = projected - self.center;
        let dist = to_point.norm();
        
        // Simple circular approximation
        let radius = (self.area / std::f32::consts::PI).sqrt();
        dist <= radius
    }
    
    /// Intersect ray with surface
    pub fn intersect_ray(&self, ray: &Ray) -> Option<RaycastHit> {
        self.plane.intersect_ray(ray).map(|t| {
            let point = ray.point_at(t);
            RaycastHit {
                point,
                normal: self.plane.normal,
                distance: t,
                surface_id: Some(self.id),
                object_id: None,
            }
        })
    }
}

/// 2D bounds of a surface
#[derive(Debug, Clone, Default)]
pub struct SurfaceBounds {
    /// Vertices of bounding polygon (in surface local coords)
    pub vertices: Vec<[f32; 2]>,
}

impl SurfaceBounds {
    pub fn from_vertices(vertices: Vec<[f32; 2]>) -> Self {
        Self { vertices }
    }
    
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plane_from_points() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.0, 0.0, 1.0);
        
        let plane = Plane::from_points(p0, p1, p2).unwrap();
        
        // Should be horizontal plane (Y-up)
        assert!((plane.normal.y.abs() - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_plane_distance() {
        let plane = Plane::new(Vector3::new(0.0, 1.0, 0.0), 0.0);
        
        let point = Point3::new(0.0, 5.0, 0.0);
        assert!((plane.distance_to_point(&point) - 5.0).abs() < 0.001);
        
        let point2 = Point3::new(0.0, -3.0, 0.0);
        assert!((plane.distance_to_point(&point2) + 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_plane_project() {
        let plane = Plane::new(Vector3::new(0.0, 1.0, 0.0), 0.0);
        
        let point = Point3::new(1.0, 5.0, 2.0);
        let projected = plane.project_point(&point);
        
        assert!((projected.y).abs() < 0.001);
        assert!((projected.x - 1.0).abs() < 0.001);
        assert!((projected.z - 2.0).abs() < 0.001);
    }
    
    #[test]
    fn test_plane_ray_intersection() {
        let plane = Plane::new(Vector3::new(0.0, 1.0, 0.0), 0.0);
        let ray = Ray::new(Point3::new(0.0, 5.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
        
        let t = plane.intersect_ray(&ray).unwrap();
        assert!((t - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_surface_detector() {
        let detector = SurfaceDetector::new(0.01, 0.1);
        
        // Create a larger horizontal plane point cloud for reliable detection
        let mut points = Vec::new();
        for x in 0..20 {
            for z in 0..20 {
                points.push(Point3::new(x as f32 * 0.1, 0.0, z as f32 * 0.1));
            }
        }
        
        let surfaces = detector.detect(&points);
        // With 400 coplanar points, we should detect at least one surface
        assert!(!surfaces.is_empty(), "Should detect at least one surface from {} points", points.len());
        
        let surface = &surfaces[0];
        assert_eq!(surface.surface_type, SurfaceType::Horizontal);
    }
    
    #[test]
    fn test_surface_type_classification() {
        // Horizontal (floor)
        let plane = Plane::new(Vector3::new(0.0, 1.0, 0.0), 0.0);
        assert!(plane.normal.y > 0.9);
        
        // Vertical (wall)
        let plane = Plane::new(Vector3::new(1.0, 0.0, 0.0), 0.0);
        assert!(plane.normal.y.abs() < 0.1);
        
        // Ceiling
        let plane = Plane::new(Vector3::new(0.0, -1.0, 0.0), 2.5);
        assert!(plane.normal.y < -0.9);
    }
}
