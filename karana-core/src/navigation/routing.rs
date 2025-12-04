//! Routing for Kāraṇa OS AR Glasses
//!
//! Route planning and turn-by-turn directions.

use std::time::Duration;
use super::location::Location;

/// Navigation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationMode {
    /// Walking navigation
    Walking,
    /// Cycling navigation
    Cycling,
    /// Driving navigation
    Driving,
    /// Public transit
    Transit,
}

impl NavigationMode {
    /// Get average speed for mode (m/s)
    pub fn average_speed(&self) -> f32 {
        match self {
            Self::Walking => 1.4,   // ~5 km/h
            Self::Cycling => 4.5,   // ~16 km/h
            Self::Driving => 13.9,  // ~50 km/h
            Self::Transit => 10.0,  // ~36 km/h
        }
    }
    
    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Walking => "Walking",
            Self::Cycling => "Cycling",
            Self::Driving => "Driving",
            Self::Transit => "Transit",
        }
    }
}

/// Turn direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnDirection {
    /// Continue straight
    Straight,
    /// Slight left turn
    SlightLeft,
    /// Left turn
    Left,
    /// Sharp left turn
    SharpLeft,
    /// Slight right turn
    SlightRight,
    /// Right turn
    Right,
    /// Sharp right turn
    SharpRight,
    /// U-turn
    UTurn,
    /// Merge
    Merge,
    /// Fork left
    ForkLeft,
    /// Fork right
    ForkRight,
    /// Enter roundabout
    RoundaboutEnter,
    /// Exit roundabout
    RoundaboutExit,
    /// Arrive at destination
    Arrive,
}

impl TurnDirection {
    /// Get instruction text
    pub fn instruction(&self) -> &str {
        match self {
            Self::Straight => "Continue straight",
            Self::SlightLeft => "Turn slight left",
            Self::Left => "Turn left",
            Self::SharpLeft => "Turn sharp left",
            Self::SlightRight => "Turn slight right",
            Self::Right => "Turn right",
            Self::SharpRight => "Turn sharp right",
            Self::UTurn => "Make a U-turn",
            Self::Merge => "Merge",
            Self::ForkLeft => "Keep left at fork",
            Self::ForkRight => "Keep right at fork",
            Self::RoundaboutEnter => "Enter the roundabout",
            Self::RoundaboutExit => "Exit the roundabout",
            Self::Arrive => "You have arrived",
        }
    }
    
    /// Get short instruction
    pub fn short(&self) -> &str {
        match self {
            Self::Straight => "Straight",
            Self::SlightLeft => "Slight L",
            Self::Left => "Left",
            Self::SharpLeft => "Sharp L",
            Self::SlightRight => "Slight R",
            Self::Right => "Right",
            Self::SharpRight => "Sharp R",
            Self::UTurn => "U-turn",
            Self::Merge => "Merge",
            Self::ForkLeft => "Fork L",
            Self::ForkRight => "Fork R",
            Self::RoundaboutEnter => "Roundabout",
            Self::RoundaboutExit => "Exit",
            Self::Arrive => "Arrive",
        }
    }
    
    /// Get angle for AR rendering (degrees, 0 = forward)
    pub fn angle(&self) -> f32 {
        match self {
            Self::Straight => 0.0,
            Self::SlightLeft => -30.0,
            Self::Left => -90.0,
            Self::SharpLeft => -135.0,
            Self::SlightRight => 30.0,
            Self::Right => 90.0,
            Self::SharpRight => 135.0,
            Self::UTurn => 180.0,
            Self::Merge => 0.0,
            Self::ForkLeft => -20.0,
            Self::ForkRight => 20.0,
            Self::RoundaboutEnter => 0.0,
            Self::RoundaboutExit => 0.0,
            Self::Arrive => 0.0,
        }
    }
}

/// Route step (single instruction)
#[derive(Debug, Clone)]
pub struct RouteStep {
    /// Turn direction
    pub turn_direction: TurnDirection,
    /// Street name
    pub street_name: String,
    /// Distance of this step (meters)
    pub distance: f32,
    /// Duration of this step
    pub duration: Duration,
    /// Location of the turn
    pub location: Location,
    /// Instruction text
    pub instruction: String,
    /// Maneuver modifier (e.g., "keep right on exit")
    pub modifier: Option<String>,
}

impl RouteStep {
    /// Create new step
    pub fn new(
        turn_direction: TurnDirection,
        street_name: String,
        distance: f32,
        location: Location,
    ) -> Self {
        let instruction = format!(
            "{} onto {}",
            turn_direction.instruction(),
            street_name
        );
        
        // Estimate duration based on walking speed
        let duration = Duration::from_secs_f32(distance / NavigationMode::Walking.average_speed());
        
        Self {
            turn_direction,
            street_name,
            distance,
            duration,
            location,
            instruction,
            modifier: None,
        }
    }
}

/// Route summary
#[derive(Debug, Clone)]
pub struct RouteSummary {
    /// Total distance (meters)
    pub total_distance: f32,
    /// Total duration estimate
    pub total_duration: Duration,
    /// Number of steps
    pub step_count: usize,
    /// Start location name
    pub start_name: String,
    /// End location name
    pub end_name: String,
}

/// Complete route
#[derive(Debug, Clone)]
pub struct Route {
    /// Navigation mode
    pub mode: NavigationMode,
    /// Route steps
    pub steps: Vec<RouteStep>,
    /// Waypoints (intermediate destinations)
    pub waypoints: Vec<Location>,
    /// Total distance
    pub total_distance: f32,
    /// Total duration
    pub total_duration: Duration,
    /// Route geometry (polyline points)
    pub geometry: Vec<Location>,
    /// Alternative routes available
    pub alternatives: Vec<Route>,
}

impl Route {
    /// Create empty route
    pub fn new(mode: NavigationMode) -> Self {
        Self {
            mode,
            steps: Vec::new(),
            waypoints: Vec::new(),
            total_distance: 0.0,
            total_duration: Duration::ZERO,
            geometry: Vec::new(),
            alternatives: Vec::new(),
        }
    }
    
    /// Create simple direct route (for testing)
    pub fn simple(
        start: Option<Location>,
        end: Location,
        mode: NavigationMode,
    ) -> Self {
        let start = start.unwrap_or(Location::new(0.0, 0.0));
        let distance = start.distance_to(&end);
        let duration = Duration::from_secs_f32(distance / mode.average_speed());
        
        let step = RouteStep::new(
            TurnDirection::Arrive,
            "Destination".to_string(),
            distance,
            end,
        );
        
        Self {
            mode,
            steps: vec![step],
            waypoints: Vec::new(),
            total_distance: distance,
            total_duration: duration,
            geometry: vec![start, end],
            alternatives: Vec::new(),
        }
    }
    
    /// Add step to route
    pub fn add_step(&mut self, step: RouteStep) {
        self.total_distance += step.distance;
        self.total_duration += step.duration;
        self.steps.push(step);
    }
    
    /// Get summary
    pub fn summary(&self) -> RouteSummary {
        RouteSummary {
            total_distance: self.total_distance,
            total_duration: self.total_duration,
            step_count: self.steps.len(),
            start_name: "Current Location".to_string(),
            end_name: "Destination".to_string(),
        }
    }
    
    /// Get remaining distance from step
    pub fn remaining_distance(&self, from_step: usize) -> f32 {
        self.steps.iter()
            .skip(from_step)
            .map(|s| s.distance)
            .sum()
    }
    
    /// Get remaining time from step
    pub fn remaining_time(&self, from_step: usize) -> Duration {
        self.steps.iter()
            .skip(from_step)
            .map(|s| s.duration)
            .sum()
    }
    
    /// Find closest point on route to location
    pub fn distance_to_route(&self, location: &Location) -> f32 {
        if self.geometry.is_empty() {
            return f32::MAX;
        }
        
        self.geometry.iter()
            .map(|p| location.distance_to(p))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_navigation_mode() {
        assert_eq!(NavigationMode::Walking.name(), "Walking");
        assert!(NavigationMode::Driving.average_speed() > NavigationMode::Walking.average_speed());
    }
    
    #[test]
    fn test_turn_direction() {
        assert_eq!(TurnDirection::Left.angle(), -90.0);
        assert_eq!(TurnDirection::Right.angle(), 90.0);
        assert!(TurnDirection::Left.instruction().contains("left"));
    }
    
    #[test]
    fn test_route_step() {
        let location = Location::new(0.0, 0.0);
        let step = RouteStep::new(
            TurnDirection::Left,
            "Main Street".to_string(),
            100.0,
            location,
        );
        
        assert_eq!(step.distance, 100.0);
        assert!(step.instruction.contains("Main Street"));
    }
    
    #[test]
    fn test_route_creation() {
        let route = Route::new(NavigationMode::Walking);
        assert!(route.steps.is_empty());
    }
    
    #[test]
    fn test_simple_route() {
        let start = Some(Location::new(0.0, 0.0));
        let end = Location::new(0.1, 0.1);
        
        let route = Route::simple(start, end, NavigationMode::Walking);
        
        assert_eq!(route.steps.len(), 1);
        assert!(route.total_distance > 0.0);
    }
    
    #[test]
    fn test_remaining_distance() {
        let mut route = Route::new(NavigationMode::Walking);
        
        let loc1 = Location::new(0.0, 0.0);
        let step1 = RouteStep::new(TurnDirection::Straight, "A".to_string(), 100.0, loc1);
        route.add_step(step1);
        
        let loc2 = Location::new(0.0, 0.001);
        let step2 = RouteStep::new(TurnDirection::Left, "B".to_string(), 200.0, loc2);
        route.add_step(step2);
        
        assert_eq!(route.remaining_distance(0), 300.0);
        assert_eq!(route.remaining_distance(1), 200.0);
    }
}
