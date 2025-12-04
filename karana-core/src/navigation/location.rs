//! Location Services for Kāraṇa OS AR Glasses
//!
//! GPS and location tracking with coordinate systems.

use std::time::{Duration, Instant};
use nalgebra::Vector3;

/// Geographic coordinate
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoCoordinate {
    /// Latitude in degrees
    pub latitude: f64,
    /// Longitude in degrees
    pub longitude: f64,
    /// Altitude in meters (optional)
    pub altitude: Option<f64>,
}

impl GeoCoordinate {
    /// Create new coordinate
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: None,
        }
    }
    
    /// Create with altitude
    pub fn with_altitude(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: Some(altitude),
        }
    }
    
    /// Calculate distance to another coordinate (meters)
    /// Uses Haversine formula
    pub fn distance_to(&self, other: &GeoCoordinate) -> f64 {
        const EARTH_RADIUS: f64 = 6_371_000.0; // meters
        
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();
        
        let a = (dlat / 2.0).sin().powi(2) +
                lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        EARTH_RADIUS * c
    }
    
    /// Calculate bearing to another coordinate (degrees)
    pub fn bearing_to(&self, other: &GeoCoordinate) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();
        
        let y = dlon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
        
        y.atan2(x).to_degrees().rem_euclid(360.0)
    }
    
    /// Get coordinate at distance and bearing
    pub fn destination(&self, distance: f64, bearing: f64) -> GeoCoordinate {
        const EARTH_RADIUS: f64 = 6_371_000.0;
        
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let brng = bearing.to_radians();
        let d = distance / EARTH_RADIUS;
        
        let lat2 = (lat1.sin() * d.cos() + lat1.cos() * d.sin() * brng.cos()).asin();
        let lon2 = lon1 + (brng.sin() * d.sin() * lat1.cos())
            .atan2(d.cos() - lat1.sin() * lat2.sin());
        
        GeoCoordinate::new(lat2.to_degrees(), lon2.to_degrees())
    }
}

/// Location with additional metadata
#[derive(Debug, Clone, Copy)]
pub struct Location {
    /// Geographic coordinate
    pub coordinate: GeoCoordinate,
    /// Accuracy in meters
    pub accuracy: f32,
    /// Speed in m/s
    pub speed: f32,
    /// Heading in degrees
    pub heading: f32,
    /// Timestamp
    pub timestamp: Instant,
}

impl Location {
    /// Create new location
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            coordinate: GeoCoordinate::new(latitude, longitude),
            accuracy: 0.0,
            speed: 0.0,
            heading: 0.0,
            timestamp: Instant::now(),
        }
    }
    
    /// Create with full data
    pub fn full(
        latitude: f64,
        longitude: f64,
        accuracy: f32,
        speed: f32,
        heading: f32,
    ) -> Self {
        Self {
            coordinate: GeoCoordinate::new(latitude, longitude),
            accuracy,
            speed,
            heading,
            timestamp: Instant::now(),
        }
    }
    
    /// Distance to another location
    pub fn distance_to(&self, other: &Location) -> f32 {
        self.coordinate.distance_to(&other.coordinate) as f32
    }
    
    /// Bearing to another location
    pub fn bearing_to(&self, other: &Location) -> f32 {
        self.coordinate.bearing_to(&other.coordinate) as f32
    }
    
    /// Get latitude
    pub fn latitude(&self) -> f64 {
        self.coordinate.latitude
    }
    
    /// Get longitude
    pub fn longitude(&self) -> f64 {
        self.coordinate.longitude
    }
}

/// Location provider source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationSource {
    /// GPS
    GPS,
    /// Network-based
    Network,
    /// Fused (combined)
    Fused,
    /// Manual/simulated
    Manual,
}

/// Location provider state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationState {
    /// Unknown/uninitialized
    Unknown,
    /// Acquiring fix
    Acquiring,
    /// Active and tracking
    Active,
    /// Disabled
    Disabled,
    /// Error
    Error,
}

/// Location provider settings
#[derive(Debug, Clone)]
pub struct LocationSettings {
    /// Minimum update interval
    pub min_interval: Duration,
    /// Minimum distance change for update (meters)
    pub min_distance: f32,
    /// Desired accuracy level
    pub desired_accuracy: LocationAccuracy,
    /// Power mode
    pub power_mode: LocationPowerMode,
}

impl Default for LocationSettings {
    fn default() -> Self {
        Self {
            min_interval: Duration::from_secs(1),
            min_distance: 1.0,
            desired_accuracy: LocationAccuracy::High,
            power_mode: LocationPowerMode::Balanced,
        }
    }
}

/// Location accuracy level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationAccuracy {
    /// Best available accuracy
    Best,
    /// High accuracy (GPS)
    High,
    /// Medium accuracy (network)
    Medium,
    /// Low accuracy (coarse)
    Low,
}

/// Location power mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationPowerMode {
    /// Highest accuracy, highest power
    HighAccuracy,
    /// Balanced accuracy and power
    Balanced,
    /// Power saving, lower accuracy
    LowPower,
}

/// Location provider
#[derive(Debug)]
pub struct LocationProvider {
    /// Current state
    state: LocationState,
    /// Primary source
    source: LocationSource,
    /// Last known location
    last_location: Option<Location>,
    /// Settings
    settings: LocationSettings,
    /// Last update time
    last_update: Option<Instant>,
    /// Location history
    history: Vec<Location>,
    /// Max history size
    max_history: usize,
}

impl LocationProvider {
    /// Create new location provider
    pub fn new() -> Self {
        Self {
            state: LocationState::Unknown,
            source: LocationSource::Fused,
            last_location: None,
            settings: LocationSettings::default(),
            last_update: None,
            history: Vec::new(),
            max_history: 100,
        }
    }
    
    /// Start location updates
    pub fn start(&mut self) {
        self.state = LocationState::Acquiring;
    }
    
    /// Stop location updates
    pub fn stop(&mut self) {
        self.state = LocationState::Disabled;
    }
    
    /// Update settings
    pub fn update_settings(&mut self, settings: LocationSettings) {
        self.settings = settings;
    }
    
    /// Get state
    pub fn state(&self) -> LocationState {
        self.state
    }
    
    /// Get last location
    pub fn last_location(&self) -> Option<&Location> {
        self.last_location.as_ref()
    }
    
    /// Simulate location update (for testing)
    pub fn simulate_location(&mut self, location: Location) {
        self.state = LocationState::Active;
        
        // Check if should update
        let should_update = match &self.last_location {
            Some(last) => {
                let time_ok = self.last_update
                    .map(|t| t.elapsed() >= self.settings.min_interval)
                    .unwrap_or(true);
                let distance_ok = location.distance_to(last) >= self.settings.min_distance;
                time_ok && distance_ok
            }
            None => true,
        };
        
        if should_update {
            if self.history.len() >= self.max_history {
                self.history.remove(0);
            }
            if let Some(loc) = self.last_location.take() {
                self.history.push(loc);
            }
            
            self.last_location = Some(location);
            self.last_update = Some(Instant::now());
        }
    }
    
    /// Get location history
    pub fn history(&self) -> &[Location] {
        &self.history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Calculate average speed from history
    pub fn average_speed(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }
        
        let total_speed: f32 = self.history.iter().map(|l| l.speed).sum();
        total_speed / self.history.len() as f32
    }
}

impl Default for LocationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_geo_coordinate() {
        let coord1 = GeoCoordinate::new(40.7128, -74.0060); // NYC
        let coord2 = GeoCoordinate::new(51.5074, -0.1278);  // London
        
        let distance = coord1.distance_to(&coord2);
        // Should be approximately 5570 km
        assert!(distance > 5_500_000.0 && distance < 5_600_000.0);
    }
    
    #[test]
    fn test_bearing() {
        let coord1 = GeoCoordinate::new(0.0, 0.0);
        let coord2 = GeoCoordinate::new(0.0, 1.0);
        
        let bearing = coord1.bearing_to(&coord2);
        // Should be approximately 90 degrees (east)
        assert!(bearing > 89.0 && bearing < 91.0);
    }
    
    #[test]
    fn test_location_creation() {
        let location = Location::new(40.7128, -74.0060);
        assert_eq!(location.latitude(), 40.7128);
        assert_eq!(location.longitude(), -74.0060);
    }
    
    #[test]
    fn test_location_distance() {
        let loc1 = Location::new(40.7128, -74.0060);
        let loc2 = Location::new(40.7150, -74.0080);
        
        let distance = loc1.distance_to(&loc2);
        assert!(distance > 0.0);
    }
    
    #[test]
    fn test_location_provider_creation() {
        let provider = LocationProvider::new();
        assert_eq!(provider.state(), LocationState::Unknown);
    }
    
    #[test]
    fn test_location_provider_start() {
        let mut provider = LocationProvider::new();
        provider.start();
        assert_eq!(provider.state(), LocationState::Acquiring);
    }
    
    #[test]
    fn test_simulate_location() {
        let mut provider = LocationProvider::new();
        provider.start();
        
        let location = Location::new(40.7128, -74.0060);
        provider.simulate_location(location);
        
        assert_eq!(provider.state(), LocationState::Active);
        assert!(provider.last_location().is_some());
    }
}
