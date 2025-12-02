//! Context Engine - Makes the OS aware of user's situation
//! 
//! Tracks time, location, activity, and environment to provide
//! contextually-aware responses and proactive suggestions.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Time of day classification for context-aware responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    EarlyMorning,  // 5-8
    Morning,       // 8-12
    Afternoon,     // 12-17
    Evening,       // 17-21
    Night,         // 21-1
    LateNight,     // 1-5
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=7 => TimeOfDay::EarlyMorning,
            8..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            21..=23 | 0 => TimeOfDay::Night,
            _ => TimeOfDay::LateNight,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            TimeOfDay::EarlyMorning => "early morning",
            TimeOfDay::Morning => "morning",
            TimeOfDay::Afternoon => "afternoon",
            TimeOfDay::Evening => "evening",
            TimeOfDay::Night => "night",
            TimeOfDay::LateNight => "late night",
        }
    }
}

/// Day of week for pattern recognition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayType {
    Weekday,
    Weekend,
}

/// Simulated location context (for AR glasses)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    Home,
    Office,
    Outdoors,
    Transit,
    Unknown,
}

impl Location {
    pub fn description(&self) -> &'static str {
        match self {
            Location::Home => "at home",
            Location::Office => "at work",
            Location::Outdoors => "outdoors",
            Location::Transit => "in transit",
            Location::Unknown => "somewhere",
        }
    }
}

/// Current user activity state
#[derive(Debug, Clone, PartialEq)]
pub enum Activity {
    Idle,
    Working,
    Browsing,
    Communicating,
    Entertainment,
    Exercise,
}

/// Environmental context (battery, connectivity, etc.)
#[derive(Debug, Clone)]
pub struct Environment {
    pub battery_level: u8,
    pub is_charging: bool,
    pub wifi_connected: bool,
    pub bluetooth_connected: bool,
    pub ambient_light: AmbientLight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AmbientLight {
    Dark,
    Dim,
    Normal,
    Bright,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            battery_level: 80,
            is_charging: false,
            wifi_connected: true,
            bluetooth_connected: false,
            ambient_light: AmbientLight::Normal,
        }
    }
}

/// A snapshot of the current context
#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub timestamp: u64,
    pub time_of_day: TimeOfDay,
    pub day_type: DayType,
    pub location: Location,
    pub activity: Activity,
    pub environment: Environment,
    pub last_actions: Vec<String>,
    pub session_duration: Duration,
}

/// Record of a user action for pattern learning
#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub action: String,
    pub timestamp: u64,
    pub time_of_day: TimeOfDay,
    pub day_type: DayType,
    pub location: Location,
    pub was_proactive: bool,  // Did we suggest this action?
    pub was_accepted: bool,   // Did user accept proactive suggestion?
}

/// The main context engine that tracks and predicts user behavior
pub struct ContextEngine {
    // Current state
    session_start: Instant,
    current_location: Location,
    current_activity: Activity,
    environment: Environment,
    
    // History for pattern recognition
    action_history: VecDeque<ActionRecord>,
    max_history: usize,
    
    // Learned patterns: (time_of_day, action) -> frequency
    time_patterns: HashMap<(TimeOfDay, String), u32>,
    
    // Learned patterns: (location, action) -> frequency
    location_patterns: HashMap<(Location, String), u32>,
    
    // Sequential patterns: (action_a, action_b) -> frequency
    sequence_patterns: HashMap<(String, String), u32>,
    
    // Recent actions for sequence detection (last 5)
    recent_actions: VecDeque<String>,
    
    // Proactive suggestion tracking
    last_suggestion_time: Option<Instant>,
    suggestion_cooldown: Duration,
    declined_suggestions: HashMap<String, u32>,
}

impl ContextEngine {
    pub fn new() -> Self {
        Self {
            session_start: Instant::now(),
            current_location: Location::Unknown,
            current_activity: Activity::Idle,
            environment: Environment::default(),
            action_history: VecDeque::with_capacity(1000),
            max_history: 1000,
            time_patterns: HashMap::new(),
            location_patterns: HashMap::new(),
            sequence_patterns: HashMap::new(),
            recent_actions: VecDeque::with_capacity(5),
            last_suggestion_time: None,
            suggestion_cooldown: Duration::from_secs(60),
            declined_suggestions: HashMap::new(),
        }
    }
    
    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Get current hour (0-23)
    fn current_hour() -> u32 {
        let secs = Self::current_timestamp();
        // Simple hour calculation (UTC)
        ((secs % 86400) / 3600) as u32
    }
    
    /// Get current day type
    fn current_day_type() -> DayType {
        let secs = Self::current_timestamp();
        let days_since_epoch = secs / 86400;
        // Jan 1, 1970 was Thursday (day 4)
        let day_of_week = (days_since_epoch + 4) % 7;
        if day_of_week >= 5 {
            DayType::Weekend
        } else {
            DayType::Weekday
        }
    }
    
    /// Record that user performed an action
    pub fn record_action(&mut self, action: &str) {
        let time_of_day = TimeOfDay::from_hour(Self::current_hour());
        let day_type = Self::current_day_type();
        
        // Create record
        let record = ActionRecord {
            action: action.to_string(),
            timestamp: Self::current_timestamp(),
            time_of_day,
            day_type,
            location: self.current_location.clone(),
            was_proactive: false,
            was_accepted: false,
        };
        
        // Update time patterns
        let time_key = (time_of_day, action.to_string());
        *self.time_patterns.entry(time_key).or_insert(0) += 1;
        
        // Update location patterns
        let loc_key = (self.current_location.clone(), action.to_string());
        *self.location_patterns.entry(loc_key).or_insert(0) += 1;
        
        // Update sequence patterns (what comes after what)
        if let Some(prev_action) = self.recent_actions.back() {
            let seq_key = (prev_action.clone(), action.to_string());
            *self.sequence_patterns.entry(seq_key).or_insert(0) += 1;
        }
        
        // Add to recent actions
        self.recent_actions.push_back(action.to_string());
        if self.recent_actions.len() > 5 {
            self.recent_actions.pop_front();
        }
        
        // Add to history
        self.action_history.push_back(record);
        if self.action_history.len() > self.max_history {
            self.action_history.pop_front();
        }
    }
    
    /// Record a proactive suggestion and whether it was accepted
    pub fn record_proactive_response(&mut self, action: &str, accepted: bool) {
        if !accepted {
            *self.declined_suggestions.entry(action.to_string()).or_insert(0) += 1;
        }
        
        let time_of_day = TimeOfDay::from_hour(Self::current_hour());
        let day_type = Self::current_day_type();
        
        let record = ActionRecord {
            action: action.to_string(),
            timestamp: Self::current_timestamp(),
            time_of_day,
            day_type,
            location: self.current_location.clone(),
            was_proactive: true,
            was_accepted: accepted,
        };
        
        self.action_history.push_back(record);
    }
    
    /// Update current location
    pub fn set_location(&mut self, location: Location) {
        self.current_location = location;
    }
    
    /// Update current activity
    pub fn set_activity(&mut self, activity: Activity) {
        self.current_activity = activity;
    }
    
    /// Update environment
    pub fn update_environment(&mut self, env: Environment) {
        self.environment = env;
    }
    
    /// Get a snapshot of current context
    pub fn snapshot(&self) -> ContextSnapshot {
        ContextSnapshot {
            timestamp: Self::current_timestamp(),
            time_of_day: TimeOfDay::from_hour(Self::current_hour()),
            day_type: Self::current_day_type(),
            location: self.current_location.clone(),
            activity: self.current_activity.clone(),
            environment: self.environment.clone(),
            last_actions: self.recent_actions.iter().cloned().collect(),
            session_duration: self.session_start.elapsed(),
        }
    }
    
    /// Get predicted next actions based on patterns
    pub fn predict_next_actions(&self, limit: usize) -> Vec<(String, f32)> {
        let mut predictions: HashMap<String, f32> = HashMap::new();
        let time_of_day = TimeOfDay::from_hour(Self::current_hour());
        
        // Score based on time patterns
        for ((time, action), count) in &self.time_patterns {
            if *time == time_of_day {
                *predictions.entry(action.clone()).or_insert(0.0) += *count as f32 * 2.0;
            }
        }
        
        // Score based on location patterns
        for ((loc, action), count) in &self.location_patterns {
            if *loc == self.current_location {
                *predictions.entry(action.clone()).or_insert(0.0) += *count as f32 * 1.5;
            }
        }
        
        // Score based on sequence patterns (what typically follows recent actions)
        if let Some(last_action) = self.recent_actions.back() {
            for ((prev, next), count) in &self.sequence_patterns {
                if prev == last_action {
                    *predictions.entry(next.clone()).or_insert(0.0) += *count as f32 * 3.0;
                }
            }
        }
        
        // Penalize frequently declined suggestions
        for (action, decline_count) in &self.declined_suggestions {
            if let Some(score) = predictions.get_mut(action) {
                *score *= 0.5_f32.powi(*decline_count as i32);
            }
        }
        
        // Sort by score
        let mut sorted: Vec<_> = predictions.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(limit);
        
        sorted
    }
    
    /// Generate a proactive suggestion based on context
    pub fn get_proactive_suggestion(&mut self) -> Option<String> {
        // Check cooldown
        if let Some(last_time) = self.last_suggestion_time {
            if last_time.elapsed() < self.suggestion_cooldown {
                return None;
            }
        }
        
        // Get predictions
        let predictions = self.predict_next_actions(3);
        
        // Need sufficient confidence
        if let Some((action, score)) = predictions.first() {
            if *score > 5.0 {
                self.last_suggestion_time = Some(Instant::now());
                let context = self.snapshot();
                return Some(format!(
                    "Good {}! Based on your patterns, would you like to {}?",
                    context.time_of_day.description(),
                    action
                ));
            }
        }
        
        // Context-based suggestions
        let context = self.snapshot();
        
        // Battery warnings
        if context.environment.battery_level < 20 && !context.environment.is_charging {
            self.last_suggestion_time = Some(Instant::now());
            return Some(format!(
                "Battery at {}%. Would you like to enter power-saving mode?",
                context.environment.battery_level
            ));
        }
        
        // Morning routine
        if context.time_of_day == TimeOfDay::Morning && context.session_duration < Duration::from_secs(60) {
            self.last_suggestion_time = Some(Instant::now());
            return Some("Good morning! Would you like to check your balance or review pending transactions?".to_string());
        }
        
        None
    }
    
    /// Get contextual greeting
    pub fn get_contextual_greeting(&self) -> String {
        let context = self.snapshot();
        let time_greeting = match context.time_of_day {
            TimeOfDay::EarlyMorning | TimeOfDay::Morning => "Good morning",
            TimeOfDay::Afternoon => "Good afternoon",
            TimeOfDay::Evening => "Good evening",
            TimeOfDay::Night | TimeOfDay::LateNight => "Hello",
        };
        
        let location_note = match context.location {
            Location::Home => "Welcome home",
            Location::Office => "Ready for work",
            Location::Outdoors => "Enjoying the day",
            Location::Transit => "Safe travels",
            Location::Unknown => time_greeting,
        };
        
        if context.location != Location::Unknown {
            format!("{}! {}", time_greeting, location_note)
        } else {
            time_greeting.to_string()
        }
    }
    
    /// Get total actions recorded
    pub fn total_actions(&self) -> usize {
        self.action_history.len()
    }
    
    /// Get pattern statistics
    pub fn pattern_stats(&self) -> (usize, usize, usize) {
        (
            self.time_patterns.len(),
            self.location_patterns.len(),
            self.sequence_patterns.len(),
        )
    }
    
    /// Describe what the engine has learned
    pub fn describe_learning(&self) -> String {
        let (time_pats, loc_pats, seq_pats) = self.pattern_stats();
        let total = self.total_actions();
        
        if total == 0 {
            return "I'm still learning your patterns. Keep using the system!".to_string();
        }
        
        let mut description = format!(
            "I've learned from {} actions:\n",
            total
        );
        
        // Top time-based patterns
        let time_of_day = TimeOfDay::from_hour(Self::current_hour());
        let mut current_time_patterns: Vec<_> = self.time_patterns
            .iter()
            .filter(|((t, _), _)| *t == time_of_day)
            .map(|((_, action), count)| (action.clone(), *count))
            .collect();
        current_time_patterns.sort_by(|a, b| b.1.cmp(&a.1));
        
        if !current_time_patterns.is_empty() {
            description.push_str(&format!("\n• In the {}, you often: ", time_of_day.description()));
            let top: Vec<_> = current_time_patterns.iter().take(3).map(|(a, _)| a.as_str()).collect();
            description.push_str(&top.join(", "));
        }
        
        // Top sequence patterns
        if let Some(last) = self.recent_actions.back() {
            let mut next_patterns: Vec<_> = self.sequence_patterns
                .iter()
                .filter(|((prev, _), _)| prev == last)
                .map(|((_, next), count)| (next.clone(), *count))
                .collect();
            next_patterns.sort_by(|a, b| b.1.cmp(&a.1));
            
            if !next_patterns.is_empty() {
                description.push_str(&format!("\n• After '{}', you typically: ", last));
                let top: Vec<_> = next_patterns.iter().take(2).map(|(a, _)| a.as_str()).collect();
                description.push_str(&top.join(" or "));
            }
        }
        
        description.push_str(&format!(
            "\n\n📊 Patterns tracked: {} time, {} location, {} sequence",
            time_pats, loc_pats, seq_pats
        ));
        
        description
    }
    
    /// Export patterns for persistence (could save to file/db)
    pub fn export_patterns(&self) -> LearnedPatterns {
        LearnedPatterns {
            time_patterns: self.time_patterns.iter()
                .map(|((t, a), c)| (format!("{:?}", t), a.clone(), *c))
                .collect(),
            sequence_patterns: self.sequence_patterns.iter()
                .map(|((a, b), c)| (a.clone(), b.clone(), *c))
                .collect(),
            declined_suggestions: self.declined_suggestions.clone(),
        }
    }
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable pattern data for persistence
#[derive(Debug, Clone)]
pub struct LearnedPatterns {
    pub time_patterns: Vec<(String, String, u32)>,
    pub sequence_patterns: Vec<(String, String, u32)>,
    pub declined_suggestions: HashMap<String, u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_engine_basic() {
        let mut engine = ContextEngine::new();
        
        // Record some actions
        engine.record_action("check_balance");
        engine.record_action("send_tokens");
        engine.record_action("check_balance");
        
        assert_eq!(engine.total_actions(), 3);
        
        // Check patterns learned
        let (time_pats, _, seq_pats) = engine.pattern_stats();
        assert!(time_pats > 0);
        assert!(seq_pats > 0);
    }
    
    #[test]
    fn test_time_of_day() {
        assert_eq!(TimeOfDay::from_hour(6), TimeOfDay::EarlyMorning);
        assert_eq!(TimeOfDay::from_hour(10), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(19), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(22), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(3), TimeOfDay::LateNight);
    }
    
    #[test]
    fn test_predictions() {
        let mut engine = ContextEngine::new();
        
        // Simulate repeated pattern
        for _ in 0..10 {
            engine.record_action("check_balance");
            engine.record_action("stake_tokens");
        }
        
        // Should predict stake_tokens after check_balance
        let predictions = engine.predict_next_actions(5);
        assert!(!predictions.is_empty());
        
        // The most likely next action should be in predictions
        let actions: Vec<_> = predictions.iter().map(|(a, _)| a.as_str()).collect();
        assert!(actions.contains(&"stake_tokens") || actions.contains(&"check_balance"));
    }
    
    #[test]
    fn test_snapshot() {
        let engine = ContextEngine::new();
        let snapshot = engine.snapshot();
        
        // Should have valid timestamp
        assert!(snapshot.timestamp > 0);
        
        // Session duration should be minimal
        assert!(snapshot.session_duration < Duration::from_secs(5));
    }
    
    #[test]
    fn test_location_update() {
        let mut engine = ContextEngine::new();
        
        engine.set_location(Location::Office);
        engine.record_action("check_email");
        
        engine.set_location(Location::Home);
        engine.record_action("watch_video");
        
        let (_, loc_pats, _) = engine.pattern_stats();
        assert!(loc_pats >= 2);
    }
}
