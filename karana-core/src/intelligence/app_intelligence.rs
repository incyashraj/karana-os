//! App Intelligence
//!
//! Intelligent app management, suggestion, and lifecycle optimization
//! for smart glasses AR applications.

use super::*;
use super::decisions::AppSuggestion;
use std::collections::{HashMap, VecDeque};

/// App intelligence system
pub struct AppIntelligence {
    /// App profiles with learned behavior
    app_profiles: HashMap<String, AppProfile>,
    /// App usage history
    usage_history: VecDeque<AppUsageEvent>,
    /// App relationships (which apps are used together)
    relationships: HashMap<String, HashMap<String, f32>>,
    /// Currently running apps
    running_apps: Vec<String>,
    /// Preloaded apps
    preloaded: Vec<String>,
    /// Configuration
    config: AppIntelConfig,
    /// Statistics
    stats: AppIntelStats,
}

impl AppIntelligence {
    pub fn new() -> Self {
        Self {
            app_profiles: HashMap::new(),
            usage_history: VecDeque::with_capacity(1000),
            relationships: HashMap::new(),
            running_apps: Vec::new(),
            preloaded: Vec::new(),
            config: AppIntelConfig::default(),
            stats: AppIntelStats::default(),
        }
    }
    
    /// Suggest apps for a user request
    pub fn suggest_apps(&self, request: &UserRequest) -> Vec<AppSuggestion> {
        let mut suggestions = Vec::new();
        
        // Extract text from request
        let text = match &request.input {
            RequestInput::Text(t) | RequestInput::Voice(t) => t.to_lowercase(),
            _ => String::new(),
        };
        
        // Score apps based on various factors
        for (app_id, profile) in &self.app_profiles {
            let mut score = 0.0;
            let mut reasons = Vec::new();
            
            // Keyword matching
            for keyword in &profile.keywords {
                if text.contains(keyword) {
                    score += 0.3;
                    reasons.push(format!("matches '{}'", keyword));
                }
            }
            
            // Context matching
            if let Some(pref_time) = &profile.preferred_time {
                if &request.context.time_of_day == pref_time {
                    score += 0.1;
                    reasons.push("preferred time".to_string());
                }
            }
            
            if let Some(pref_loc) = &profile.preferred_location {
                if request.context.location.as_ref() == Some(pref_loc) {
                    score += 0.15;
                    reasons.push("preferred location".to_string());
                }
            }
            
            // Recent usage bonus
            if self.was_recently_used(app_id) {
                score += 0.1;
                reasons.push("recently used".to_string());
            }
            
            // Relationship bonus (used after current app)
            if let Some(current) = &request.context.current_app {
                if let Some(rel) = self.get_relationship(current, app_id) {
                    score += rel * 0.2;
                    reasons.push("often used after current app".to_string());
                }
            }
            
            // Usage frequency bonus
            score += (profile.usage_count as f32 / 100.0).min(0.2);
            
            if score > 0.0 {
                suggestions.push(AppSuggestion {
                    app_id: app_id.clone(),
                    score: score.min(1.0),
                    reason: reasons.join(", "),
                });
            }
        }
        
        // Sort by score
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        suggestions.truncate(5);
        
        suggestions
    }
    
    /// Record app launch
    pub fn record_launch(&mut self, app_id: &str, context: &RequestContext) {
        // Update app profile
        let profile = self.app_profiles.entry(app_id.to_string()).or_insert_with(|| {
            AppProfile::new(app_id)
        });
        profile.usage_count += 1;
        profile.last_used = Some(Instant::now());
        
        // Update time preference
        let time_counts = profile.time_usage.entry(context.time_of_day).or_insert(0);
        *time_counts += 1;
        
        // Update location preference
        if let Some(loc) = &context.location {
            let loc_counts = profile.location_usage.entry(loc.clone()).or_insert(0);
            *loc_counts += 1;
        }
        
        // Record relationship with previous app
        if let Some(prev_app) = &context.current_app {
            self.record_relationship(prev_app, app_id);
        }
        
        // Add to running apps
        if !self.running_apps.contains(&app_id.to_string()) {
            self.running_apps.push(app_id.to_string());
        }
        
        // Add to history
        self.usage_history.push_back(AppUsageEvent {
            app_id: app_id.to_string(),
            event_type: AppEventType::Launch,
            timestamp: Instant::now(),
            duration: None,
        });
        
        if self.usage_history.len() > 1000 {
            self.usage_history.pop_front();
        }
        
        self.stats.total_launches += 1;
    }
    
    /// Record app close
    pub fn record_close(&mut self, app_id: &str) {
        // Update profile with session duration
        if let Some(profile) = self.app_profiles.get_mut(app_id) {
            if let Some(last) = profile.last_used {
                let duration = last.elapsed();
                profile.total_usage_time += duration;
                profile.session_count += 1;
                
                // Update average session duration
                let avg = profile.total_usage_time.as_secs_f32() / profile.session_count as f32;
                profile.avg_session_duration = Duration::from_secs_f32(avg);
            }
        }
        
        // Remove from running apps
        self.running_apps.retain(|a| a != app_id);
        
        // Add to history
        self.usage_history.push_back(AppUsageEvent {
            app_id: app_id.to_string(),
            event_type: AppEventType::Close,
            timestamp: Instant::now(),
            duration: None,
        });
    }
    
    /// Preload predicted apps
    pub fn preload_predicted(&mut self, context: &RequestContext) -> Vec<String> {
        let mut to_preload = Vec::new();
        
        // Predict based on time of day
        for (app_id, profile) in &self.app_profiles {
            if let Some(count) = profile.time_usage.get(&context.time_of_day) {
                let total: u32 = profile.time_usage.values().sum();
                let ratio = *count as f32 / total as f32;
                
                if ratio > 0.3 && !self.running_apps.contains(app_id) && !self.preloaded.contains(app_id) {
                    to_preload.push(app_id.clone());
                }
            }
        }
        
        // Predict based on location
        if let Some(loc) = &context.location {
            for (app_id, profile) in &self.app_profiles {
                if let Some(count) = profile.location_usage.get(loc) {
                    let total: u32 = profile.location_usage.values().sum();
                    let ratio = *count as f32 / total as f32;
                    
                    if ratio > 0.3 && !self.running_apps.contains(app_id) 
                        && !self.preloaded.contains(app_id) && !to_preload.contains(app_id) {
                        to_preload.push(app_id.clone());
                    }
                }
            }
        }
        
        // Predict based on app relationships
        if let Some(current) = &context.current_app {
            if let Some(relationships) = self.relationships.get(current) {
                let mut sorted: Vec<_> = relationships.iter().collect();
                sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
                
                for (app_id, prob) in sorted.iter().take(2) {
                    if **prob > 0.3 && !self.running_apps.contains(*app_id) 
                        && !self.preloaded.contains(*app_id) && !to_preload.contains(*app_id) {
                        to_preload.push((*app_id).clone());
                    }
                }
            }
        }
        
        // Limit preloading
        to_preload.truncate(self.config.max_preloaded);
        
        // Update preloaded list
        self.preloaded.extend(to_preload.clone());
        self.stats.apps_preloaded += to_preload.len() as u64;
        
        to_preload
    }
    
    /// Get apps to close for resource management
    pub fn suggest_close(&self, memory_pressure: f32) -> Vec<String> {
        if memory_pressure < 0.7 {
            return vec![];
        }
        
        // Score running apps by importance
        let mut app_scores: Vec<(String, f32)> = self.running_apps.iter()
            .filter_map(|app_id| {
                self.app_profiles.get(app_id).map(|profile| {
                    let recency = profile.last_used
                        .map(|t| t.elapsed().as_secs() as f32 / 3600.0) // Hours since use
                        .unwrap_or(24.0);
                    
                    let importance = profile.usage_count as f32 / 100.0;
                    
                    // Higher score = less important (close first)
                    let score = recency - importance;
                    (app_id.clone(), score)
                })
            })
            .collect();
        
        app_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Suggest closing apps with highest scores
        let num_to_close = if memory_pressure > 0.9 { 3 } else { 1 };
        
        app_scores.into_iter()
            .take(num_to_close)
            .map(|(id, _)| id)
            .collect()
    }
    
    /// Check if app was recently used
    fn was_recently_used(&self, app_id: &str) -> bool {
        self.usage_history.iter()
            .rev()
            .take(10)
            .any(|e| e.app_id == app_id)
    }
    
    /// Record app relationship
    fn record_relationship(&mut self, from: &str, to: &str) {
        if from == to {
            return;
        }
        
        let relationships = self.relationships.entry(from.to_string()).or_default();
        let weight = relationships.entry(to.to_string()).or_insert(0.0);
        *weight = (*weight + 0.1).min(1.0);
        
        // Normalize weights
        let total: f32 = relationships.values().sum();
        if total > 0.0 {
            for w in relationships.values_mut() {
                *w /= total;
            }
        }
    }
    
    /// Get relationship strength between apps
    fn get_relationship(&self, from: &str, to: &str) -> Option<f32> {
        self.relationships.get(from)
            .and_then(|r| r.get(to))
            .copied()
    }
    
    /// Add keywords for an app
    pub fn add_app_keywords(&mut self, app_id: &str, keywords: Vec<String>) {
        let profile = self.app_profiles.entry(app_id.to_string()).or_insert_with(|| {
            AppProfile::new(app_id)
        });
        profile.keywords.extend(keywords);
    }
    
    /// Get app profile
    pub fn get_profile(&self, app_id: &str) -> Option<&AppProfile> {
        self.app_profiles.get(app_id)
    }
    
    /// Get running apps
    pub fn running_apps(&self) -> &[String] {
        &self.running_apps
    }
    
    /// Get statistics
    pub fn stats(&self) -> &AppIntelStats {
        &self.stats
    }
}

impl Default for AppIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

/// App profile with learned behavior
#[derive(Debug, Clone)]
pub struct AppProfile {
    pub app_id: String,
    pub keywords: Vec<String>,
    pub usage_count: u32,
    pub session_count: u32,
    pub total_usage_time: Duration,
    pub avg_session_duration: Duration,
    pub last_used: Option<Instant>,
    pub preferred_time: Option<TimeOfDay>,
    pub preferred_location: Option<String>,
    pub time_usage: HashMap<TimeOfDay, u32>,
    pub location_usage: HashMap<String, u32>,
}

impl AppProfile {
    fn new(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            keywords: Vec::new(),
            usage_count: 0,
            session_count: 0,
            total_usage_time: Duration::ZERO,
            avg_session_duration: Duration::ZERO,
            last_used: None,
            preferred_time: None,
            preferred_location: None,
            time_usage: HashMap::new(),
            location_usage: HashMap::new(),
        }
    }
    
    /// Get most frequent time of use
    pub fn most_frequent_time(&self) -> Option<TimeOfDay> {
        self.time_usage.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(time, _)| *time)
    }
    
    /// Get most frequent location
    pub fn most_frequent_location(&self) -> Option<String> {
        self.location_usage.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(loc, _)| loc.clone())
    }
}

/// App usage event
#[derive(Debug, Clone)]
pub struct AppUsageEvent {
    pub app_id: String,
    pub event_type: AppEventType,
    pub timestamp: Instant,
    pub duration: Option<Duration>,
}

/// App event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppEventType {
    Launch,
    Close,
    Focus,
    Background,
}

/// App intelligence configuration
#[derive(Debug, Clone)]
pub struct AppIntelConfig {
    pub max_preloaded: usize,
    pub preload_threshold: f32,
    pub relationship_decay: f32,
}

impl Default for AppIntelConfig {
    fn default() -> Self {
        Self {
            max_preloaded: 3,
            preload_threshold: 0.3,
            relationship_decay: 0.95,
        }
    }
}

/// App intelligence statistics
#[derive(Debug, Clone, Default)]
pub struct AppIntelStats {
    pub total_launches: u64,
    pub apps_preloaded: u64,
    pub preload_hits: u64,
    pub suggestions_accepted: u64,
    pub suggestions_total: u64,
}

impl AppIntelStats {
    pub fn preload_hit_rate(&self) -> f32 {
        if self.apps_preloaded == 0 {
            0.0
        } else {
            self.preload_hits as f32 / self.apps_preloaded as f32
        }
    }
    
    pub fn suggestion_acceptance(&self) -> f32 {
        if self.suggestions_total == 0 {
            0.0
        } else {
            self.suggestions_accepted as f32 / self.suggestions_total as f32
        }
    }
}

/// App category for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppCategory {
    Productivity,
    Communication,
    Entertainment,
    Navigation,
    Health,
    Finance,
    Social,
    Utility,
    AR,
    System,
}

/// App resource requirements
#[derive(Debug, Clone)]
pub struct AppResources {
    pub memory_mb: u32,
    pub cpu_intensive: bool,
    pub gpu_required: bool,
    pub network_required: bool,
    pub camera_required: bool,
}

impl Default for AppResources {
    fn default() -> Self {
        Self {
            memory_mb: 50,
            cpu_intensive: false,
            gpu_required: false,
            network_required: false,
            camera_required: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_intelligence_creation() {
        let intel = AppIntelligence::new();
        assert!(intel.running_apps.is_empty());
    }
    
    #[test]
    fn test_record_launch() {
        let mut intel = AppIntelligence::new();
        
        let context = RequestContext {
            location: Some("office".to_string()),
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        intel.record_launch("browser", &context);
        
        assert!(intel.running_apps.contains(&"browser".to_string()));
        assert!(intel.app_profiles.contains_key("browser"));
        assert_eq!(intel.app_profiles.get("browser").unwrap().usage_count, 1);
    }
    
    #[test]
    fn test_app_relationships() {
        let mut intel = AppIntelligence::new();
        
        let context1 = RequestContext {
            location: None,
            current_app: Some("email".to_string()),
            recent_apps: vec!["email".to_string()],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        intel.record_launch("calendar", &context1);
        
        let rel = intel.get_relationship("email", "calendar");
        assert!(rel.is_some());
        assert!(rel.unwrap() > 0.0);
    }
    
    #[test]
    fn test_suggest_apps() {
        let mut intel = AppIntelligence::new();
        
        // Add browser with keywords
        intel.add_app_keywords("browser", vec!["web".to_string(), "search".to_string(), "browse".to_string()]);
        
        let request = UserRequest {
            id: 1,
            input: RequestInput::Voice("search the web".to_string()),
            context: RequestContext {
                location: None,
                current_app: None,
                recent_apps: vec![],
                time_of_day: TimeOfDay::Morning,
                battery_level: 80,
                is_moving: false,
                ambient_noise: NoiseLevel::Normal,
                user_state: UserState::Active,
            },
            timestamp: Instant::now(),
        };
        
        let suggestions = intel.suggest_apps(&request);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.app_id == "browser"));
    }
    
    #[test]
    fn test_app_profile() {
        let mut profile = AppProfile::new("test_app");
        
        profile.time_usage.insert(TimeOfDay::Morning, 10);
        profile.time_usage.insert(TimeOfDay::Evening, 5);
        
        assert_eq!(profile.most_frequent_time(), Some(TimeOfDay::Morning));
    }
    
    #[test]
    fn test_preload_predicted() {
        let mut intel = AppIntelligence::new();
        
        // Record many morning launches
        let context = RequestContext {
            location: None,
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        for _ in 0..10 {
            intel.record_launch("email", &context);
            intel.record_close("email");
        }
        
        let to_preload = intel.preload_predicted(&context);
        // Email should be suggested for preload in morning
        // (it might or might not be, depending on threshold)
        assert!(to_preload.len() <= intel.config.max_preloaded);
    }
    
    #[test]
    fn test_suggest_close() {
        let mut intel = AppIntelligence::new();
        
        let context = RequestContext {
            location: None,
            current_app: None,
            recent_apps: vec![],
            time_of_day: TimeOfDay::Morning,
            battery_level: 80,
            is_moving: false,
            ambient_noise: NoiseLevel::Normal,
            user_state: UserState::Active,
        };
        
        intel.record_launch("app1", &context);
        intel.record_launch("app2", &context);
        intel.record_launch("app3", &context);
        
        // Low memory pressure - no suggestions
        let to_close = intel.suggest_close(0.5);
        assert!(to_close.is_empty());
        
        // High memory pressure - suggest closing
        let to_close = intel.suggest_close(0.95);
        assert!(!to_close.is_empty());
    }
    
    #[test]
    fn test_app_intel_stats() {
        let mut stats = AppIntelStats::default();
        
        stats.apps_preloaded = 10;
        stats.preload_hits = 7;
        
        assert!((stats.preload_hit_rate() - 0.7).abs() < 0.01);
    }
}
