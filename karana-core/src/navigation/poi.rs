//! Points of Interest for Kāraṇa OS AR Glasses
//!
//! POI management and nearby search.

use std::collections::HashMap;
use super::location::Location;

/// POI category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum POICategory {
    /// Restaurant
    Restaurant,
    /// Coffee shop
    CoffeeShop,
    /// Bar/Pub
    Bar,
    /// Gas station
    GasStation,
    /// Parking
    Parking,
    /// Hotel
    Hotel,
    /// Shopping
    Shopping,
    /// Grocery store
    Grocery,
    /// Bank/ATM
    Bank,
    /// Hospital
    Hospital,
    /// Pharmacy
    Pharmacy,
    /// Park
    Park,
    /// Museum
    Museum,
    /// Landmark
    Landmark,
    /// Public transit
    Transit,
    /// Other
    Other,
}

impl POICategory {
    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Restaurant => "Restaurant",
            Self::CoffeeShop => "Coffee",
            Self::Bar => "Bar",
            Self::GasStation => "Gas Station",
            Self::Parking => "Parking",
            Self::Hotel => "Hotel",
            Self::Shopping => "Shopping",
            Self::Grocery => "Grocery",
            Self::Bank => "Bank/ATM",
            Self::Hospital => "Hospital",
            Self::Pharmacy => "Pharmacy",
            Self::Park => "Park",
            Self::Museum => "Museum",
            Self::Landmark => "Landmark",
            Self::Transit => "Transit",
            Self::Other => "Other",
        }
    }
    
    /// Get icon name
    pub fn icon(&self) -> &str {
        match self {
            Self::Restaurant => "🍽️",
            Self::CoffeeShop => "☕",
            Self::Bar => "🍺",
            Self::GasStation => "⛽",
            Self::Parking => "🅿️",
            Self::Hotel => "🏨",
            Self::Shopping => "🛒",
            Self::Grocery => "🏪",
            Self::Bank => "🏦",
            Self::Hospital => "🏥",
            Self::Pharmacy => "💊",
            Self::Park => "🌳",
            Self::Museum => "🏛️",
            Self::Landmark => "📍",
            Self::Transit => "🚇",
            Self::Other => "📌",
        }
    }
}

/// Point of Interest
#[derive(Debug, Clone)]
pub struct PointOfInterest {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Category
    pub category: POICategory,
    /// Location
    pub location: Location,
    /// Address
    pub address: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Website
    pub website: Option<String>,
    /// Rating (0-5)
    pub rating: Option<f32>,
    /// Is open now
    pub is_open: Option<bool>,
    /// Additional info
    pub info: HashMap<String, String>,
}

impl PointOfInterest {
    /// Create new POI
    pub fn new(id: String, name: String, category: POICategory, location: Location) -> Self {
        Self {
            id,
            name,
            category,
            location,
            address: None,
            phone: None,
            website: None,
            rating: None,
            is_open: None,
            info: HashMap::new(),
        }
    }
    
    /// Builder: add address
    pub fn with_address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }
    
    /// Builder: add rating
    pub fn with_rating(mut self, rating: f32) -> Self {
        self.rating = Some(rating.clamp(0.0, 5.0));
        self
    }
    
    /// Get display string
    pub fn display(&self) -> String {
        format!("{} {}", self.category.icon(), self.name)
    }
}

/// POI search result
#[derive(Debug, Clone)]
pub struct POISearchResult {
    /// POI
    pub poi: PointOfInterest,
    /// Distance from search location (meters)
    pub distance: f32,
}

/// POI Manager
#[derive(Debug)]
pub struct POIManager {
    /// Known POIs
    pois: HashMap<String, PointOfInterest>,
    /// Saved/favorited POIs
    favorites: Vec<String>,
    /// Recently viewed POIs
    recent: Vec<String>,
    /// Max recent items
    max_recent: usize,
}

impl POIManager {
    /// Create new POI manager
    pub fn new() -> Self {
        Self {
            pois: HashMap::new(),
            favorites: Vec::new(),
            recent: Vec::new(),
            max_recent: 20,
        }
    }
    
    /// Add POI
    pub fn add_poi(&mut self, poi: PointOfInterest) {
        self.pois.insert(poi.id.clone(), poi);
    }
    
    /// Get POI by ID
    pub fn get_poi(&self, id: &str) -> Option<&PointOfInterest> {
        self.pois.get(id)
    }
    
    /// Remove POI
    pub fn remove_poi(&mut self, id: &str) -> Option<PointOfInterest> {
        self.pois.remove(id)
    }
    
    /// Find nearby POIs
    pub fn find_nearby(&self, location: &Location, radius: f32) -> Vec<&PointOfInterest> {
        self.pois.values()
            .filter(|poi| location.distance_to(&poi.location) <= radius)
            .collect()
    }
    
    /// Find nearby with distances
    pub fn search_nearby(&self, location: &Location, radius: f32) -> Vec<POISearchResult> {
        let mut results: Vec<_> = self.pois.values()
            .filter_map(|poi| {
                let distance = location.distance_to(&poi.location);
                if distance <= radius {
                    Some(POISearchResult {
                        poi: poi.clone(),
                        distance,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
    
    /// Find by category
    pub fn find_by_category(&self, category: POICategory) -> Vec<&PointOfInterest> {
        self.pois.values()
            .filter(|poi| poi.category == category)
            .collect()
    }
    
    /// Search by name
    pub fn search_by_name(&self, query: &str) -> Vec<&PointOfInterest> {
        let query_lower = query.to_lowercase();
        self.pois.values()
            .filter(|poi| poi.name.to_lowercase().contains(&query_lower))
            .collect()
    }
    
    /// Add to favorites
    pub fn add_favorite(&mut self, id: &str) {
        if !self.favorites.contains(&id.to_string()) {
            self.favorites.push(id.to_string());
        }
    }
    
    /// Remove from favorites
    pub fn remove_favorite(&mut self, id: &str) {
        self.favorites.retain(|f| f != id);
    }
    
    /// Is favorite
    pub fn is_favorite(&self, id: &str) -> bool {
        self.favorites.contains(&id.to_string())
    }
    
    /// Get favorites
    pub fn get_favorites(&self) -> Vec<&PointOfInterest> {
        self.favorites.iter()
            .filter_map(|id| self.pois.get(id))
            .collect()
    }
    
    /// Add to recent
    pub fn add_recent(&mut self, id: &str) {
        self.recent.retain(|r| r != id);
        if self.recent.len() >= self.max_recent {
            self.recent.remove(0);
        }
        self.recent.push(id.to_string());
    }
    
    /// Get recent POIs
    pub fn get_recent(&self) -> Vec<&PointOfInterest> {
        self.recent.iter()
            .rev()
            .filter_map(|id| self.pois.get(id))
            .collect()
    }
    
    /// Get POI count
    pub fn count(&self) -> usize {
        self.pois.len()
    }
    
    /// Clear all POIs
    pub fn clear(&mut self) {
        self.pois.clear();
        self.recent.clear();
    }
}

impl Default for POIManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_poi_category() {
        assert_eq!(POICategory::Restaurant.name(), "Restaurant");
        assert_eq!(POICategory::CoffeeShop.icon(), "☕");
    }
    
    #[test]
    fn test_poi_creation() {
        let location = Location::new(40.7128, -74.0060);
        let poi = PointOfInterest::new(
            "test-1".to_string(),
            "Test Place".to_string(),
            POICategory::Restaurant,
            location,
        );
        
        assert_eq!(poi.name, "Test Place");
        assert_eq!(poi.category, POICategory::Restaurant);
    }
    
    #[test]
    fn test_poi_builder() {
        let location = Location::new(40.7128, -74.0060);
        let poi = PointOfInterest::new(
            "test-1".to_string(),
            "Test Place".to_string(),
            POICategory::Restaurant,
            location,
        )
        .with_address("123 Main St".to_string())
        .with_rating(4.5);
        
        assert_eq!(poi.address, Some("123 Main St".to_string()));
        assert_eq!(poi.rating, Some(4.5));
    }
    
    #[test]
    fn test_poi_manager_creation() {
        let manager = POIManager::new();
        assert_eq!(manager.count(), 0);
    }
    
    #[test]
    fn test_add_poi() {
        let mut manager = POIManager::new();
        let location = Location::new(40.7128, -74.0060);
        let poi = PointOfInterest::new(
            "test-1".to_string(),
            "Test Place".to_string(),
            POICategory::Restaurant,
            location,
        );
        
        manager.add_poi(poi);
        assert_eq!(manager.count(), 1);
        assert!(manager.get_poi("test-1").is_some());
    }
    
    #[test]
    fn test_find_nearby() {
        let mut manager = POIManager::new();
        
        let loc1 = Location::new(40.7128, -74.0060);
        let poi1 = PointOfInterest::new(
            "test-1".to_string(),
            "Near".to_string(),
            POICategory::Restaurant,
            loc1,
        );
        manager.add_poi(poi1);
        
        let loc2 = Location::new(50.0, -80.0);
        let poi2 = PointOfInterest::new(
            "test-2".to_string(),
            "Far".to_string(),
            POICategory::Restaurant,
            loc2,
        );
        manager.add_poi(poi2);
        
        let search_loc = Location::new(40.7130, -74.0062);
        let nearby = manager.find_nearby(&search_loc, 1000.0);
        
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].name, "Near");
    }
    
    #[test]
    fn test_favorites() {
        let mut manager = POIManager::new();
        
        let location = Location::new(40.7128, -74.0060);
        let poi = PointOfInterest::new(
            "test-1".to_string(),
            "Test Place".to_string(),
            POICategory::Restaurant,
            location,
        );
        manager.add_poi(poi);
        
        manager.add_favorite("test-1");
        assert!(manager.is_favorite("test-1"));
        
        manager.remove_favorite("test-1");
        assert!(!manager.is_favorite("test-1"));
    }
    
    #[test]
    fn test_search_by_name() {
        let mut manager = POIManager::new();
        
        let location = Location::new(40.7128, -74.0060);
        let poi = PointOfInterest::new(
            "test-1".to_string(),
            "Joe's Coffee".to_string(),
            POICategory::CoffeeShop,
            location,
        );
        manager.add_poi(poi);
        
        let results = manager.search_by_name("coffee");
        assert_eq!(results.len(), 1);
    }
}
