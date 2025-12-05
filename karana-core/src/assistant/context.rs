// Context Management for Kāraṇa OS
// Maintains conversational and environmental context

use std::collections::HashMap;

/// Context type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextType {
    Temporal,
    Spatial,
    Conversational,
    Environmental,
    User,
    App,
}

/// Context entry
#[derive(Debug, Clone)]
pub struct ContextEntry {
    pub key: String,
    pub value: String,
    pub context_type: ContextType,
    pub timestamp: u64,
    pub ttl_ms: Option<u64>,
    pub source: String,
}

/// Current context snapshot
#[derive(Debug, Clone, Default)]
pub struct CurrentContext {
    pub time_of_day: String,
    pub location: Option<LocationContext>,
    pub weather: Option<WeatherContext>,
    pub calendar: Option<CalendarContext>,
    pub active_app: Option<String>,
    pub recent_topics: Vec<String>,
    pub user_state: UserState,
    pub custom: HashMap<String, String>,
}

/// Location context
#[derive(Debug, Clone)]
pub struct LocationContext {
    pub name: String,
    pub location_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub is_home: bool,
    pub is_work: bool,
}

/// Weather context
#[derive(Debug, Clone)]
pub struct WeatherContext {
    pub condition: String,
    pub temperature_celsius: f32,
    pub humidity_percent: f32,
    pub is_raining: bool,
}

/// Calendar context
#[derive(Debug, Clone)]
pub struct CalendarContext {
    pub next_event: Option<String>,
    pub next_event_time: Option<u64>,
    pub events_today: u32,
    pub is_busy: bool,
}

/// User state
#[derive(Debug, Clone, Default)]
pub struct UserState {
    pub is_moving: bool,
    pub is_driving: bool,
    pub is_in_meeting: bool,
    pub activity_level: String,
    pub mood_estimate: Option<String>,
}

/// Context manager
pub struct ContextManager {
    entries: HashMap<String, ContextEntry>,
    current: CurrentContext,
    history: Vec<ContextEntry>,
    max_history: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        let mut manager = Self {
            entries: HashMap::new(),
            current: CurrentContext::default(),
            history: Vec::new(),
            max_history: 1000,
        };
        manager.initialize_defaults();
        manager
    }

    fn initialize_defaults(&mut self) {
        self.current.time_of_day = "afternoon".to_string();
        self.current.user_state = UserState::default();
    }

    pub fn set(&mut self, key: &str, value: &str, context_type: ContextType) {
        let entry = ContextEntry {
            key: key.to_string(),
            value: value.to_string(),
            context_type,
            timestamp: 0,
            ttl_ms: None,
            source: "system".to_string(),
        };
        
        // Add to history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(entry.clone());
        
        self.entries.insert(key.to_string(), entry);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|e| e.value.as_str())
    }

    pub fn set_with_ttl(&mut self, key: &str, value: &str, context_type: ContextType, ttl_ms: u64) {
        let mut entry = ContextEntry {
            key: key.to_string(),
            value: value.to_string(),
            context_type,
            timestamp: 0,
            ttl_ms: Some(ttl_ms),
            source: "system".to_string(),
        };
        entry.ttl_ms = Some(ttl_ms);
        self.entries.insert(key.to_string(), entry);
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn get_current_context(&self) -> &CurrentContext {
        &self.current
    }

    pub fn update_location(&mut self, location: LocationContext) {
        self.current.location = Some(location);
    }

    pub fn update_weather(&mut self, weather: WeatherContext) {
        self.current.weather = Some(weather);
    }

    pub fn update_calendar(&mut self, calendar: CalendarContext) {
        self.current.calendar = Some(calendar);
    }

    pub fn set_active_app(&mut self, app_id: Option<String>) {
        self.current.active_app = app_id;
    }

    pub fn add_recent_topic(&mut self, topic: &str) {
        // Keep last 5 topics
        if self.current.recent_topics.len() >= 5 {
            self.current.recent_topics.remove(0);
        }
        self.current.recent_topics.push(topic.to_string());
    }

    pub fn update_user_state(&mut self, state: UserState) {
        self.current.user_state = state;
    }

    pub fn set_custom(&mut self, key: &str, value: &str) {
        self.current.custom.insert(key.to_string(), value.to_string());
    }

    pub fn get_custom(&self, key: &str) -> Option<&str> {
        self.current.custom.get(key).map(|s| s.as_str())
    }

    pub fn get_entries_by_type(&self, context_type: ContextType) -> Vec<&ContextEntry> {
        self.entries.values()
            .filter(|e| e.context_type == context_type)
            .collect()
    }

    pub fn clean_expired(&mut self, current_time: u64) {
        self.entries.retain(|_, entry| {
            if let Some(ttl) = entry.ttl_ms {
                current_time - entry.timestamp < ttl
            } else {
                true
            }
        });
    }

    pub fn get_history(&self, limit: usize) -> &[ContextEntry] {
        let start = if self.history.len() > limit {
            self.history.len() - limit
        } else {
            0
        };
        &self.history[start..]
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current = CurrentContext::default();
        self.initialize_defaults();
    }

    pub fn to_prompt_context(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Time: {}", self.current.time_of_day));

        if let Some(loc) = &self.current.location {
            parts.push(format!("Location: {} ({})", loc.name, loc.location_type));
        }

        if let Some(weather) = &self.current.weather {
            parts.push(format!("Weather: {} {}°C", weather.condition, weather.temperature_celsius));
        }

        if let Some(cal) = &self.current.calendar {
            if let Some(event) = &cal.next_event {
                parts.push(format!("Next event: {}", event));
            }
        }

        if !self.current.recent_topics.is_empty() {
            parts.push(format!("Recent topics: {}", self.current.recent_topics.join(", ")));
        }

        parts.join(". ")
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert!(manager.get_current_context().time_of_day.len() > 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut manager = ContextManager::new();
        manager.set("test_key", "test_value", ContextType::User);
        
        assert_eq!(manager.get("test_key"), Some("test_value"));
    }

    #[test]
    fn test_remove() {
        let mut manager = ContextManager::new();
        manager.set("test_key", "test_value", ContextType::User);
        
        assert!(manager.remove("test_key"));
        assert!(manager.get("test_key").is_none());
    }

    #[test]
    fn test_location_update() {
        let mut manager = ContextManager::new();
        let location = LocationContext {
            name: "Home".to_string(),
            location_type: "residential".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
            is_home: true,
            is_work: false,
        };
        
        manager.update_location(location);
        assert!(manager.get_current_context().location.is_some());
    }

    #[test]
    fn test_weather_update() {
        let mut manager = ContextManager::new();
        let weather = WeatherContext {
            condition: "sunny".to_string(),
            temperature_celsius: 22.0,
            humidity_percent: 45.0,
            is_raining: false,
        };
        
        manager.update_weather(weather);
        assert!(manager.get_current_context().weather.is_some());
    }

    #[test]
    fn test_recent_topics() {
        let mut manager = ContextManager::new();
        
        for i in 0..7 {
            manager.add_recent_topic(&format!("topic_{}", i));
        }
        
        // Should only keep last 5
        assert_eq!(manager.get_current_context().recent_topics.len(), 5);
    }

    #[test]
    fn test_custom_context() {
        let mut manager = ContextManager::new();
        manager.set_custom("custom_key", "custom_value");
        
        assert_eq!(manager.get_custom("custom_key"), Some("custom_value"));
    }

    #[test]
    fn test_entries_by_type() {
        let mut manager = ContextManager::new();
        manager.set("user1", "value1", ContextType::User);
        manager.set("user2", "value2", ContextType::User);
        manager.set("app1", "value3", ContextType::App);
        
        let user_entries = manager.get_entries_by_type(ContextType::User);
        assert_eq!(user_entries.len(), 2);
    }

    #[test]
    fn test_history() {
        let mut manager = ContextManager::new();
        manager.set("key1", "value1", ContextType::User);
        manager.set("key2", "value2", ContextType::User);
        manager.set("key3", "value3", ContextType::User);
        
        let history = manager.get_history(2);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut manager = ContextManager::new();
        manager.set("key", "value", ContextType::User);
        manager.clear();
        
        assert!(manager.get("key").is_none());
    }

    #[test]
    fn test_prompt_context() {
        let mut manager = ContextManager::new();
        manager.add_recent_topic("weather");
        
        let prompt = manager.to_prompt_context();
        assert!(prompt.contains("Time:"));
    }

    #[test]
    fn test_active_app() {
        let mut manager = ContextManager::new();
        manager.set_active_app(Some("maps".to_string()));
        
        assert_eq!(manager.get_current_context().active_app, Some("maps".to_string()));
    }
}
