//! Navigation & Maps System for Kāraṇa OS AR Glasses
//!
//! Provides turn-by-turn navigation with AR overlays,
//! points of interest, and spatial mapping.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use nalgebra::{Vector2, Vector3};

pub mod routing;
pub mod poi;
pub mod ar_nav;
pub mod location;
pub mod maps;

pub use routing::{Route, RouteStep, NavigationMode, TurnDirection};
pub use poi::{PointOfInterest, POICategory, POIManager};
pub use ar_nav::{ARNavOverlay, NavIndicator, ARNavigator};
pub use location::{Location, LocationProvider, GeoCoordinate};
pub use maps::{MapTile, MapLayer, MapRenderer};

/// Navigation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationState {
    /// Idle, no active navigation
    Idle,
    /// Planning route
    Planning,
    /// Actively navigating
    Navigating,
    /// Arrived at destination
    Arrived,
    /// Route recalculating
    Recalculating,
    /// Navigation paused
    Paused,
    /// Error state
    Error,
}

/// Navigation event
#[derive(Debug, Clone)]
pub enum NavigationEvent {
    /// Route started
    RouteStarted { destination: String },
    /// Turn approaching
    TurnApproaching { direction: TurnDirection, distance: f32, street: String },
    /// Turn now
    TurnNow { direction: TurnDirection, street: String },
    /// Route recalculated
    RouteRecalculated { reason: String },
    /// Arrived at waypoint
    WaypointReached { index: usize, name: String },
    /// Arrived at destination
    DestinationReached,
    /// Route cancelled
    RouteCancelled,
    /// Off route
    OffRoute,
    /// POI nearby
    POINearby { poi: String, distance: f32 },
}

/// Navigation preferences
#[derive(Debug, Clone)]
pub struct NavigationPreferences {
    /// Preferred navigation mode
    pub mode: NavigationMode,
    /// Avoid highways
    pub avoid_highways: bool,
    /// Avoid tolls
    pub avoid_tolls: bool,
    /// Prefer scenic routes
    pub prefer_scenic: bool,
    /// Voice guidance enabled
    pub voice_guidance: bool,
    /// Voice guidance volume (0.0-1.0)
    pub voice_volume: f32,
    /// AR overlay enabled
    pub ar_overlay: bool,
    /// AR overlay opacity (0.0-1.0)
    pub ar_opacity: f32,
    /// Distance units (true = metric)
    pub metric_units: bool,
    /// Show POIs along route
    pub show_pois: bool,
    /// POI categories to show
    pub poi_categories: Vec<POICategory>,
    /// Turn alert distance (meters)
    pub turn_alert_distance: f32,
    /// Off-route threshold (meters)
    pub off_route_threshold: f32,
}

impl Default for NavigationPreferences {
    fn default() -> Self {
        Self {
            mode: NavigationMode::Walking,
            avoid_highways: false,
            avoid_tolls: false,
            prefer_scenic: false,
            voice_guidance: true,
            voice_volume: 0.7,
            ar_overlay: true,
            ar_opacity: 0.8,
            metric_units: true,
            show_pois: true,
            poi_categories: vec![
                POICategory::Restaurant,
                POICategory::CoffeeShop,
                POICategory::GasStation,
            ],
            turn_alert_distance: 50.0,
            off_route_threshold: 30.0,
        }
    }
}

/// Navigation statistics
#[derive(Debug, Clone)]
pub struct NavigationStats {
    /// Total distance traveled
    pub distance_traveled: f32,
    /// Distance remaining
    pub distance_remaining: f32,
    /// Estimated time remaining
    pub time_remaining: Duration,
    /// Time elapsed
    pub time_elapsed: Duration,
    /// Current speed (m/s)
    pub current_speed: f32,
    /// Average speed (m/s)
    pub average_speed: f32,
    /// Number of turns remaining
    pub turns_remaining: usize,
    /// POIs passed
    pub pois_passed: usize,
}

/// Navigation manager
#[derive(Debug)]
pub struct NavigationManager {
    /// Current state
    state: NavigationState,
    /// Current route
    current_route: Option<Route>,
    /// Current step index
    current_step: usize,
    /// User location
    user_location: Option<Location>,
    /// Destination
    destination: Option<Location>,
    /// Navigation preferences
    preferences: NavigationPreferences,
    /// POI manager
    poi_manager: POIManager,
    /// AR navigator
    ar_navigator: ARNavigator,
    /// Event history
    event_history: VecDeque<NavigationEvent>,
    /// Max event history
    max_history: usize,
    /// Navigation start time
    nav_start_time: Option<Instant>,
    /// Distance traveled
    distance_traveled: f32,
    /// Last location for distance calc
    last_location: Option<Location>,
    /// Off-route count
    off_route_count: u32,
}

impl NavigationManager {
    /// Create new navigation manager
    pub fn new() -> Self {
        Self {
            state: NavigationState::Idle,
            current_route: None,
            current_step: 0,
            user_location: None,
            destination: None,
            preferences: NavigationPreferences::default(),
            poi_manager: POIManager::new(),
            ar_navigator: ARNavigator::new(),
            event_history: VecDeque::with_capacity(100),
            max_history: 100,
            nav_start_time: None,
            distance_traveled: 0.0,
            last_location: None,
            off_route_count: 0,
        }
    }
    
    /// Update navigation preferences
    pub fn update_preferences(&mut self, prefs: NavigationPreferences) {
        self.preferences = prefs;
        self.ar_navigator.set_opacity(self.preferences.ar_opacity);
    }
    
    /// Get preferences
    pub fn preferences(&self) -> &NavigationPreferences {
        &self.preferences
    }
    
    /// Start navigation to destination
    pub fn start_navigation(&mut self, destination: Location, name: String) {
        self.destination = Some(destination);
        self.state = NavigationState::Planning;
        
        // In real implementation, would call routing service
        // For now, create a simple route
        let route = Route::simple(
            self.user_location.clone(),
            destination,
            self.preferences.mode,
        );
        
        self.current_route = Some(route);
        self.current_step = 0;
        self.state = NavigationState::Navigating;
        self.nav_start_time = Some(Instant::now());
        self.distance_traveled = 0.0;
        self.off_route_count = 0;
        
        self.add_event(NavigationEvent::RouteStarted { destination: name });
    }
    
    /// Stop navigation
    pub fn stop_navigation(&mut self) {
        self.current_route = None;
        self.current_step = 0;
        self.destination = None;
        self.state = NavigationState::Idle;
        self.nav_start_time = None;
        
        self.add_event(NavigationEvent::RouteCancelled);
    }
    
    /// Pause navigation
    pub fn pause_navigation(&mut self) {
        if self.state == NavigationState::Navigating {
            self.state = NavigationState::Paused;
        }
    }
    
    /// Resume navigation
    pub fn resume_navigation(&mut self) {
        if self.state == NavigationState::Paused {
            self.state = NavigationState::Navigating;
        }
    }
    
    /// Update user location
    pub fn update_location(&mut self, location: Location) {
        // Update distance traveled
        if let Some(ref last) = self.last_location {
            self.distance_traveled += location.distance_to(last);
        }
        self.last_location = Some(location.clone());
        
        self.user_location = Some(location.clone());
        
        if self.state != NavigationState::Navigating {
            return;
        }
        
        // Collect events to add later (avoids borrow issues)
        let mut events_to_add: Vec<NavigationEvent> = Vec::new();
        let mut should_recalculate = false;
        let mut arrived = false;
        let mut next_step = self.current_step;
        
        // Check if on route
        if let Some(ref route) = self.current_route {
            let distance_to_route = route.distance_to_route(&location);
            
            if distance_to_route > self.preferences.off_route_threshold {
                self.off_route_count += 1;
                
                if self.off_route_count >= 3 {
                    events_to_add.push(NavigationEvent::OffRoute);
                    should_recalculate = true;
                }
            } else {
                self.off_route_count = 0;
            }
            
            // Check for turn alerts
            if let Some(step) = route.steps.get(self.current_step) {
                let distance_to_turn = location.distance_to(&step.location);
                
                if distance_to_turn <= self.preferences.turn_alert_distance {
                    events_to_add.push(NavigationEvent::TurnApproaching {
                        direction: step.turn_direction,
                        distance: distance_to_turn,
                        street: step.street_name.clone(),
                    });
                }
                
                // Check if completed step
                if distance_to_turn < 10.0 {
                    events_to_add.push(NavigationEvent::TurnNow {
                        direction: step.turn_direction,
                        street: step.street_name.clone(),
                    });
                    next_step = self.current_step + 1;
                    
                    // Check if arrived
                    if next_step >= route.steps.len() {
                        arrived = true;
                        events_to_add.push(NavigationEvent::DestinationReached);
                    }
                }
            }
        }
        
        // Check for nearby POIs
        if self.preferences.show_pois {
            let nearby_pois: Vec<_> = self.poi_manager.find_nearby(&location, 100.0)
                .into_iter()
                .filter(|poi| self.preferences.poi_categories.contains(&poi.category))
                .map(|poi| (poi.name.clone(), location.distance_to(&poi.location)))
                .collect();
            
            for (name, distance) in nearby_pois {
                events_to_add.push(NavigationEvent::POINearby {
                    poi: name,
                    distance,
                });
            }
        }
        
        // Apply state changes
        self.current_step = next_step;
        if arrived {
            self.state = NavigationState::Arrived;
        }
        
        // Add collected events
        for event in events_to_add {
            self.add_event(event);
        }
        
        // Recalculate if needed
        if should_recalculate {
            self.recalculate_route();
        }
        
        // Update AR navigator
        if self.preferences.ar_overlay {
            self.ar_navigator.update(&location, self.current_route.as_ref());
        }
    }
    
    /// Recalculate route
    fn recalculate_route(&mut self) {
        self.state = NavigationState::Recalculating;
        
        if let (Some(current), Some(dest)) = (&self.user_location, &self.destination) {
            let route = Route::simple(
                Some(current.clone()),
                *dest,
                self.preferences.mode,
            );
            
            self.current_route = Some(route);
            self.current_step = 0;
            self.state = NavigationState::Navigating;
            self.off_route_count = 0;
            
            self.add_event(NavigationEvent::RouteRecalculated {
                reason: "Off route".to_string(),
            });
        }
    }
    
    /// Get current state
    pub fn state(&self) -> NavigationState {
        self.state
    }
    
    /// Get current route
    pub fn current_route(&self) -> Option<&Route> {
        self.current_route.as_ref()
    }
    
    /// Get current step
    pub fn current_step(&self) -> Option<&RouteStep> {
        self.current_route
            .as_ref()
            .and_then(|r| r.steps.get(self.current_step))
    }
    
    /// Get next turn description
    pub fn next_turn(&self) -> Option<String> {
        self.current_step().map(|step| {
            format!(
                "{} onto {}",
                step.turn_direction.instruction(),
                step.street_name
            )
        })
    }
    
    /// Get distance to next turn
    pub fn distance_to_next_turn(&self) -> Option<f32> {
        let step = self.current_step()?;
        let location = self.user_location.as_ref()?;
        Some(location.distance_to(&step.location))
    }
    
    /// Get navigation statistics
    pub fn stats(&self) -> NavigationStats {
        let time_elapsed = self.nav_start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);
        
        let distance_remaining = self.current_route
            .as_ref()
            .map(|r| r.remaining_distance(self.current_step))
            .unwrap_or(0.0);
        
        let time_remaining = self.current_route
            .as_ref()
            .map(|r| r.remaining_time(self.current_step))
            .unwrap_or(Duration::ZERO);
        
        let average_speed = if time_elapsed.as_secs() > 0 {
            self.distance_traveled / time_elapsed.as_secs_f32()
        } else {
            0.0
        };
        
        let current_speed = self.user_location
            .as_ref()
            .map(|l| l.speed)
            .unwrap_or(0.0);
        
        let turns_remaining = self.current_route
            .as_ref()
            .map(|r| r.steps.len().saturating_sub(self.current_step))
            .unwrap_or(0);
        
        NavigationStats {
            distance_traveled: self.distance_traveled,
            distance_remaining,
            time_remaining,
            time_elapsed,
            current_speed,
            average_speed,
            turns_remaining,
            pois_passed: 0,
        }
    }
    
    /// Get AR overlays for current navigation
    pub fn ar_overlays(&self) -> Vec<ARNavOverlay> {
        self.ar_navigator.get_overlays()
    }
    
    /// Get POI manager
    pub fn poi_manager(&self) -> &POIManager {
        &self.poi_manager
    }
    
    /// Get mutable POI manager
    pub fn poi_manager_mut(&mut self) -> &mut POIManager {
        &mut self.poi_manager
    }
    
    /// Add event to history
    fn add_event(&mut self, event: NavigationEvent) {
        if self.event_history.len() >= self.max_history {
            self.event_history.pop_front();
        }
        self.event_history.push_back(event);
    }
    
    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<&NavigationEvent> {
        self.event_history.iter().rev().take(count).collect()
    }
    
    /// Clear event history
    pub fn clear_events(&mut self) {
        self.event_history.clear();
    }
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_navigation_manager_creation() {
        let manager = NavigationManager::new();
        assert_eq!(manager.state(), NavigationState::Idle);
    }
    
    #[test]
    fn test_default_preferences() {
        let prefs = NavigationPreferences::default();
        assert!(prefs.voice_guidance);
        assert!(prefs.ar_overlay);
        assert!(prefs.metric_units);
    }
    
    #[test]
    fn test_navigation_state() {
        let mut manager = NavigationManager::new();
        
        // Start navigation
        let dest = Location::new(0.0, 0.0);
        manager.start_navigation(dest, "Test".to_string());
        
        assert_eq!(manager.state(), NavigationState::Navigating);
    }
    
    #[test]
    fn test_pause_resume() {
        let mut manager = NavigationManager::new();
        
        let dest = Location::new(0.0, 0.0);
        manager.start_navigation(dest, "Test".to_string());
        
        manager.pause_navigation();
        assert_eq!(manager.state(), NavigationState::Paused);
        
        manager.resume_navigation();
        assert_eq!(manager.state(), NavigationState::Navigating);
    }
    
    #[test]
    fn test_stop_navigation() {
        let mut manager = NavigationManager::new();
        
        let dest = Location::new(0.0, 0.0);
        manager.start_navigation(dest, "Test".to_string());
        manager.stop_navigation();
        
        assert_eq!(manager.state(), NavigationState::Idle);
        assert!(manager.current_route().is_none());
    }
    
    #[test]
    fn test_stats() {
        let manager = NavigationManager::new();
        let stats = manager.stats();
        
        assert_eq!(stats.distance_traveled, 0.0);
    }
}
