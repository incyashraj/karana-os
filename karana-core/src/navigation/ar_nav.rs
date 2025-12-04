//! AR Navigation Overlays for Kāraṇa OS AR Glasses
//!
//! Visual AR indicators for navigation.

use std::time::{Duration, Instant};
use nalgebra::Vector3;
use super::location::Location;
use super::routing::{Route, TurnDirection};

/// AR navigation indicator type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavIndicator {
    /// Arrow pointing to direction
    Arrow,
    /// Path line on ground
    PathLine,
    /// Turn sign
    TurnSign,
    /// Destination marker
    DestinationMarker,
    /// Distance indicator
    Distance,
    /// Street name label
    StreetLabel,
    /// POI marker
    POIMarker,
    /// Waypoint marker
    WaypointMarker,
}

/// AR overlay display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayStyle {
    /// Minimal (arrows only)
    Minimal,
    /// Standard (arrows + path)
    Standard,
    /// Full (all indicators)
    Full,
    /// Immersive (3D path)
    Immersive,
}

/// Color for AR overlay
#[derive(Debug, Clone, Copy)]
pub struct ARColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ARColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
    
    pub fn blue() -> Self {
        Self::new(0.2, 0.6, 1.0, 1.0)
    }
    
    pub fn green() -> Self {
        Self::new(0.2, 1.0, 0.4, 1.0)
    }
    
    pub fn yellow() -> Self {
        Self::new(1.0, 0.9, 0.2, 1.0)
    }
    
    pub fn red() -> Self {
        Self::new(1.0, 0.3, 0.3, 1.0)
    }
}

/// Single AR navigation overlay
#[derive(Debug, Clone)]
pub struct ARNavOverlay {
    /// Indicator type
    pub indicator: NavIndicator,
    /// World position
    pub position: Vector3<f32>,
    /// Rotation (euler angles)
    pub rotation: Vector3<f32>,
    /// Scale
    pub scale: f32,
    /// Color
    pub color: ARColor,
    /// Text label (if any)
    pub label: Option<String>,
    /// Time to live
    pub ttl: Option<Duration>,
    /// Created at
    pub created: Instant,
    /// Is animated
    pub animated: bool,
}

impl ARNavOverlay {
    /// Create new overlay
    pub fn new(indicator: NavIndicator, position: Vector3<f32>) -> Self {
        Self {
            indicator,
            position,
            rotation: Vector3::zeros(),
            scale: 1.0,
            color: ARColor::blue(),
            label: None,
            ttl: None,
            created: Instant::now(),
            animated: false,
        }
    }
    
    /// Create turn arrow
    pub fn turn_arrow(direction: TurnDirection, distance: f32) -> Self {
        let angle = direction.angle();
        let mut overlay = Self::new(
            NavIndicator::Arrow,
            Vector3::new(0.0, 1.5, distance.min(20.0)),
        );
        overlay.rotation = Vector3::new(0.0, angle.to_radians(), 0.0);
        overlay.color = if distance < 20.0 { ARColor::green() } else { ARColor::blue() };
        overlay.animated = distance < 50.0;
        overlay
    }
    
    /// Create destination marker
    pub fn destination(name: &str, position: Vector3<f32>) -> Self {
        let mut overlay = Self::new(NavIndicator::DestinationMarker, position);
        overlay.label = Some(name.to_string());
        overlay.color = ARColor::green();
        overlay
    }
    
    /// Create street label
    pub fn street_label(name: &str, position: Vector3<f32>) -> Self {
        let mut overlay = Self::new(NavIndicator::StreetLabel, position);
        overlay.label = Some(name.to_string());
        overlay.color = ARColor::white();
        overlay
    }
    
    /// Create distance indicator
    pub fn distance_indicator(distance: f32, position: Vector3<f32>) -> Self {
        let mut overlay = Self::new(NavIndicator::Distance, position);
        overlay.label = Some(format_distance(distance));
        overlay
    }
    
    /// Is expired
    pub fn is_expired(&self) -> bool {
        self.ttl.map(|ttl| self.created.elapsed() > ttl).unwrap_or(false)
    }
    
    /// With color
    pub fn with_color(mut self, color: ARColor) -> Self {
        self.color = color;
        self
    }
    
    /// With label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
    
    /// With ttl
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// Format distance for display
fn format_distance(meters: f32) -> String {
    if meters >= 1000.0 {
        format!("{:.1} km", meters / 1000.0)
    } else {
        format!("{:.0} m", meters)
    }
}

/// AR Navigator - generates overlays for navigation
#[derive(Debug)]
pub struct ARNavigator {
    /// Active overlays
    overlays: Vec<ARNavOverlay>,
    /// Display style
    style: OverlayStyle,
    /// Opacity
    opacity: f32,
    /// Show path
    show_path: bool,
    /// Path points
    path_points: Vec<Vector3<f32>>,
    /// Max overlays
    max_overlays: usize,
    /// Last update
    last_update: Instant,
}

impl ARNavigator {
    /// Create new AR navigator
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
            style: OverlayStyle::Standard,
            opacity: 0.8,
            show_path: true,
            path_points: Vec::new(),
            max_overlays: 20,
            last_update: Instant::now(),
        }
    }
    
    /// Set display style
    pub fn set_style(&mut self, style: OverlayStyle) {
        self.style = style;
    }
    
    /// Set opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    
    /// Get opacity
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    
    /// Enable/disable path
    pub fn set_show_path(&mut self, show: bool) {
        self.show_path = show;
    }
    
    /// Update navigation overlays
    pub fn update(&mut self, location: &Location, route: Option<&Route>) {
        self.overlays.clear();
        
        let Some(route) = route else { return };
        
        // Find current step
        let current_step = route.steps.first();
        
        if let Some(step) = current_step {
            let distance = location.distance_to(&step.location);
            
            // Add turn arrow
            let arrow = ARNavOverlay::turn_arrow(step.turn_direction, distance);
            self.overlays.push(arrow);
            
            // Add distance indicator
            if matches!(self.style, OverlayStyle::Standard | OverlayStyle::Full) {
                let dist_overlay = ARNavOverlay::distance_indicator(
                    distance,
                    Vector3::new(0.0, 1.8, 2.0),
                );
                self.overlays.push(dist_overlay);
            }
            
            // Add street label
            if matches!(self.style, OverlayStyle::Full) {
                let street_overlay = ARNavOverlay::street_label(
                    &step.street_name,
                    Vector3::new(0.0, 2.2, 3.0),
                );
                self.overlays.push(street_overlay);
            }
        }
        
        // Generate path line
        if self.show_path && !route.geometry.is_empty() {
            self.generate_path_points(&route.geometry, location);
        }
        
        // Remove expired overlays
        self.overlays.retain(|o| !o.is_expired());
        
        // Cap overlays
        while self.overlays.len() > self.max_overlays {
            self.overlays.remove(0);
        }
        
        self.last_update = Instant::now();
    }
    
    /// Generate path points for AR path visualization
    fn generate_path_points(&mut self, geometry: &[Location], _user_location: &Location) {
        self.path_points.clear();
        
        // Convert geo coordinates to local 3D space
        // In a real implementation this would use proper coordinate transformation
        for (i, point) in geometry.iter().enumerate().take(10) {
            let z = (i as f32) * 5.0;
            let point_3d = Vector3::new(0.0, 0.0, z);
            self.path_points.push(point_3d);
        }
    }
    
    /// Get active overlays
    pub fn get_overlays(&self) -> Vec<ARNavOverlay> {
        self.overlays.clone()
    }
    
    /// Get path points
    pub fn get_path_points(&self) -> &[Vector3<f32>] {
        &self.path_points
    }
    
    /// Add custom overlay
    pub fn add_overlay(&mut self, overlay: ARNavOverlay) {
        if self.overlays.len() < self.max_overlays {
            self.overlays.push(overlay);
        }
    }
    
    /// Clear overlays
    pub fn clear(&mut self) {
        self.overlays.clear();
        self.path_points.clear();
    }
}

impl Default for ARNavigator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ar_color() {
        let blue = ARColor::blue();
        assert!(blue.b > blue.r);
    }
    
    #[test]
    fn test_ar_overlay_creation() {
        let overlay = ARNavOverlay::new(
            NavIndicator::Arrow,
            Vector3::new(0.0, 1.5, 5.0),
        );
        
        assert_eq!(overlay.indicator, NavIndicator::Arrow);
        assert!(!overlay.is_expired());
    }
    
    #[test]
    fn test_turn_arrow() {
        let arrow = ARNavOverlay::turn_arrow(TurnDirection::Left, 30.0);
        
        assert_eq!(arrow.indicator, NavIndicator::Arrow);
        assert!(arrow.rotation.y < 0.0); // Left turn = negative angle
    }
    
    #[test]
    fn test_destination_marker() {
        let marker = ARNavOverlay::destination("Home", Vector3::new(0.0, 0.0, 100.0));
        
        assert_eq!(marker.indicator, NavIndicator::DestinationMarker);
        assert_eq!(marker.label, Some("Home".to_string()));
    }
    
    #[test]
    fn test_format_distance() {
        assert_eq!(format_distance(500.0), "500 m");
        assert_eq!(format_distance(1500.0), "1.5 km");
    }
    
    #[test]
    fn test_ar_navigator_creation() {
        let nav = ARNavigator::new();
        assert!(nav.get_overlays().is_empty());
    }
    
    #[test]
    fn test_set_style() {
        let mut nav = ARNavigator::new();
        nav.set_style(OverlayStyle::Full);
        // Style change is internal
    }
    
    #[test]
    fn test_set_opacity() {
        let mut nav = ARNavigator::new();
        nav.set_opacity(0.5);
        assert_eq!(nav.opacity(), 0.5);
        
        // Test clamping
        nav.set_opacity(1.5);
        assert_eq!(nav.opacity(), 1.0);
    }
    
    #[test]
    fn test_add_custom_overlay() {
        let mut nav = ARNavigator::new();
        let overlay = ARNavOverlay::new(
            NavIndicator::POIMarker,
            Vector3::new(10.0, 0.0, 20.0),
        );
        
        nav.add_overlay(overlay);
        assert_eq!(nav.get_overlays().len(), 1);
    }
}
