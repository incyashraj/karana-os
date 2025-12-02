//! User Behavior Learning - Adaptive system that learns from user interactions
//!
//! Builds a profile of user preferences, command styles, and correction patterns
//! to improve future responses and reduce friction.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How the user rephrased or corrected a misunderstood command
#[derive(Debug, Clone)]
pub struct CorrectionRecord {
    pub original_input: String,
    pub misunderstood_as: String,
    pub corrected_input: String,
    pub intended_action: String,
    pub timestamp: u64,
}

/// Preference strength based on frequency
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreferenceStrength {
    Weak,      // 1-2 occurrences
    Moderate,  // 3-5 occurrences
    Strong,    // 6-10 occurrences
    VeryStrong, // 11+ occurrences
}

impl PreferenceStrength {
    pub fn from_count(count: u32) -> Self {
        match count {
            1..=2 => PreferenceStrength::Weak,
            3..=5 => PreferenceStrength::Moderate,
            6..=10 => PreferenceStrength::Strong,
            _ => PreferenceStrength::VeryStrong,
        }
    }
    
    pub fn weight(&self) -> f32 {
        match self {
            PreferenceStrength::Weak => 0.25,
            PreferenceStrength::Moderate => 0.5,
            PreferenceStrength::Strong => 0.75,
            PreferenceStrength::VeryStrong => 1.0,
        }
    }
}

/// Learned alias or phrase that maps to an action
#[derive(Debug, Clone)]
pub struct LearnedPhrase {
    pub phrase: String,
    pub action: String,
    pub confidence: f32,
    pub use_count: u32,
    pub last_used: u64,
}

/// User's preferred values for common operations
#[derive(Debug, Clone)]
pub struct PreferredValues {
    // Common transaction amounts the user uses
    pub common_amounts: HashMap<u64, u32>,  // amount -> frequency
    
    // Frequently used addresses/recipients
    pub frequent_recipients: HashMap<String, u32>,  // address -> frequency
    
    // Preferred staking durations
    pub staking_durations: HashMap<u64, u32>,  // days -> frequency
    
    // Photo settings
    pub photo_resolution: Option<(u32, u32)>,
    
    // Timer defaults
    pub default_timer_minutes: Option<u64>,
}

impl Default for PreferredValues {
    fn default() -> Self {
        Self {
            common_amounts: HashMap::new(),
            frequent_recipients: HashMap::new(),
            staking_durations: HashMap::new(),
            photo_resolution: None,
            default_timer_minutes: None,
        }
    }
}

/// Complete user behavior profile
#[derive(Debug)]
pub struct UserProfile {
    // Core identity
    pub user_id: String,
    pub created_at: u64,
    pub last_active: u64,
    
    // Interaction style
    pub verbosity_preference: VerbosityPreference,
    pub confirmation_preference: ConfirmationPreference,
    
    // Learned phrases/aliases
    pub learned_phrases: Vec<LearnedPhrase>,
    
    // Corrections for learning
    pub corrections: VecDeque<CorrectionRecord>,
    max_corrections: usize,
    
    // Command preferences
    pub preferred_values: PreferredValues,
    
    // Expertise level (affects explanations)
    pub expertise_level: ExpertiseLevel,
    
    // Statistics
    pub total_interactions: u64,
    pub successful_interactions: u64,
    pub correction_count: u64,
}

/// How verbose the user prefers responses
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerbosityPreference {
    Minimal,     // Just essential info
    Normal,      // Standard responses
    Detailed,    // Extra context and explanations
}

/// When the user wants confirmation prompts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmationPreference {
    Always,          // Confirm everything
    HighValueOnly,   // Only confirm transactions > threshold
    Never,           // Trust the user
}

/// User's expertise level for tailored explanations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpertiseLevel {
    Beginner,    // Explain everything, use simple terms
    Intermediate, // Standard explanations
    Expert,       // Technical details, skip basics
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            user_id: format!("user_{}", rand_id()),
            created_at: current_timestamp(),
            last_active: current_timestamp(),
            verbosity_preference: VerbosityPreference::Normal,
            confirmation_preference: ConfirmationPreference::HighValueOnly,
            learned_phrases: Vec::new(),
            corrections: VecDeque::with_capacity(100),
            max_corrections: 100,
            preferred_values: PreferredValues::default(),
            expertise_level: ExpertiseLevel::Intermediate,
            total_interactions: 0,
            successful_interactions: 0,
            correction_count: 0,
        }
    }
}

impl UserProfile {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record an interaction
    pub fn record_interaction(&mut self, successful: bool) {
        self.total_interactions += 1;
        if successful {
            self.successful_interactions += 1;
        }
        self.last_active = current_timestamp();
    }
    
    /// Record a correction (user rephrased after misunderstanding)
    pub fn record_correction(&mut self, original: &str, misunderstood: &str, corrected: &str, intended: &str) {
        let record = CorrectionRecord {
            original_input: original.to_string(),
            misunderstood_as: misunderstood.to_string(),
            corrected_input: corrected.to_string(),
            intended_action: intended.to_string(),
            timestamp: current_timestamp(),
        };
        
        self.corrections.push_back(record);
        self.correction_count += 1;
        
        if self.corrections.len() > self.max_corrections {
            self.corrections.pop_front();
        }
        
        // Learn from the correction - add as new phrase
        self.learn_phrase(original, intended, 0.8);
    }
    
    /// Learn a new phrase mapping
    pub fn learn_phrase(&mut self, phrase: &str, action: &str, confidence: f32) {
        let normalized = normalize_phrase(phrase);
        
        // Check if we already know this phrase
        if let Some(existing) = self.learned_phrases.iter_mut()
            .find(|p| normalize_phrase(&p.phrase) == normalized)
        {
            existing.use_count += 1;
            existing.last_used = current_timestamp();
            // Increase confidence with use
            existing.confidence = (existing.confidence + confidence) / 2.0;
        } else {
            self.learned_phrases.push(LearnedPhrase {
                phrase: phrase.to_string(),
                action: action.to_string(),
                confidence,
                use_count: 1,
                last_used: current_timestamp(),
            });
        }
    }
    
    /// Look up a phrase to see if we've learned its meaning
    pub fn lookup_phrase(&self, phrase: &str) -> Option<&LearnedPhrase> {
        let normalized = normalize_phrase(phrase);
        
        self.learned_phrases.iter()
            .filter(|p| {
                let p_norm = normalize_phrase(&p.phrase);
                p_norm == normalized || 
                normalized.contains(&p_norm) || 
                p_norm.contains(&normalized)
            })
            .max_by(|a, b| {
                // Prefer higher confidence and more recent
                let a_score = a.confidence * a.use_count as f32;
                let b_score = b.confidence * b.use_count as f32;
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
    
    /// Record a transaction amount
    pub fn record_amount(&mut self, amount: u64) {
        *self.preferred_values.common_amounts.entry(amount).or_insert(0) += 1;
    }
    
    /// Get most common transaction amounts
    pub fn common_amounts(&self, limit: usize) -> Vec<(u64, u32)> {
        let mut amounts: Vec<_> = self.preferred_values.common_amounts.iter()
            .map(|(&a, &c)| (a, c))
            .collect();
        amounts.sort_by(|a, b| b.1.cmp(&a.1));
        amounts.truncate(limit);
        amounts
    }
    
    /// Record a recipient address
    pub fn record_recipient(&mut self, address: &str) {
        *self.preferred_values.frequent_recipients.entry(address.to_string()).or_insert(0) += 1;
    }
    
    /// Get most frequent recipients
    pub fn frequent_recipients(&self, limit: usize) -> Vec<(&str, u32)> {
        let mut recipients: Vec<_> = self.preferred_values.frequent_recipients.iter()
            .map(|(a, &c)| (a.as_str(), c))
            .collect();
        recipients.sort_by(|a, b| b.1.cmp(&a.1));
        recipients.truncate(limit);
        recipients
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_interactions == 0 {
            return 1.0;
        }
        self.successful_interactions as f32 / self.total_interactions as f32
    }
    
    /// Adjust expertise based on interactions
    pub fn update_expertise(&mut self) {
        // More interactions + higher success rate = higher expertise
        if self.total_interactions > 100 && self.success_rate() > 0.9 {
            self.expertise_level = ExpertiseLevel::Expert;
        } else if self.total_interactions > 20 && self.success_rate() > 0.7 {
            self.expertise_level = ExpertiseLevel::Intermediate;
        } else {
            self.expertise_level = ExpertiseLevel::Beginner;
        }
    }
    
    /// Get explanation appropriate for user's level
    pub fn tailor_explanation(&self, beginner: &str, intermediate: &str, expert: &str) -> String {
        match self.expertise_level {
            ExpertiseLevel::Beginner => beginner.to_string(),
            ExpertiseLevel::Intermediate => intermediate.to_string(),
            ExpertiseLevel::Expert => expert.to_string(),
        }
    }
    
    /// Describe the user's profile
    pub fn describe(&self) -> String {
        let mut desc = format!(
            "📊 User Profile\n\
             • Interactions: {} ({:.1}% success)\n\
             • Expertise: {:?}\n\
             • Learned phrases: {}\n",
            self.total_interactions,
            self.success_rate() * 100.0,
            self.expertise_level,
            self.learned_phrases.len()
        );
        
        if !self.learned_phrases.is_empty() {
            desc.push_str("\n📝 Custom phrases I've learned:\n");
            for phrase in self.learned_phrases.iter().take(5) {
                desc.push_str(&format!(
                    "  \"{}\" → {} (used {} times)\n",
                    phrase.phrase, phrase.action, phrase.use_count
                ));
            }
        }
        
        if self.correction_count > 0 {
            desc.push_str(&format!(
                "\n🔧 Corrections applied: {} (I'm learning from these!)\n",
                self.correction_count
            ));
        }
        
        let common = self.common_amounts(3);
        if !common.is_empty() {
            desc.push_str("\n💰 Your common amounts: ");
            let amounts: Vec<_> = common.iter().map(|(a, _)| format!("{} KARA", a)).collect();
            desc.push_str(&amounts.join(", "));
            desc.push('\n');
        }
        
        desc
    }
}

/// The learning system that manages user behavior learning
pub struct LearningSystem {
    profile: UserProfile,
    
    // Short-term memory for current session
    session_commands: VecDeque<(String, String, bool)>, // (input, action, success)
    
    // Feedback tracking
    positive_feedback: HashMap<String, u32>,  // action -> thumbs up count
    negative_feedback: HashMap<String, u32>,  // action -> thumbs down count
}

impl LearningSystem {
    pub fn new() -> Self {
        Self {
            profile: UserProfile::new(),
            session_commands: VecDeque::with_capacity(50),
            positive_feedback: HashMap::new(),
            negative_feedback: HashMap::new(),
        }
    }
    
    pub fn with_profile(profile: UserProfile) -> Self {
        Self {
            profile,
            session_commands: VecDeque::with_capacity(50),
            positive_feedback: HashMap::new(),
            negative_feedback: HashMap::new(),
        }
    }
    
    /// Process user input and potentially use learned phrases
    pub fn enhance_input(&self, input: &str) -> Option<String> {
        if let Some(learned) = self.profile.lookup_phrase(input) {
            if learned.confidence > 0.7 {
                return Some(learned.action.clone());
            }
        }
        None
    }
    
    /// Record command execution
    pub fn record_command(&mut self, input: &str, action: &str, success: bool) {
        self.profile.record_interaction(success);
        
        self.session_commands.push_back((
            input.to_string(),
            action.to_string(),
            success,
        ));
        
        if self.session_commands.len() > 50 {
            self.session_commands.pop_front();
        }
        
        // If successful, reinforce the phrase-action mapping
        if success {
            self.profile.learn_phrase(input, action, 0.6);
        }
    }
    
    /// Handle user feedback ("that was wrong", "no that's not what I meant")
    pub fn handle_negative_feedback(&mut self, correct_action: Option<&str>) {
        // Get the last command
        if let Some((input, wrong_action, _)) = self.session_commands.back() {
            *self.negative_feedback.entry(wrong_action.clone()).or_insert(0) += 1;
            
            if let Some(correct) = correct_action {
                self.profile.record_correction(input, wrong_action, input, correct);
            }
        }
    }
    
    /// Handle positive feedback ("thanks", "perfect")
    pub fn handle_positive_feedback(&mut self) {
        if let Some((_, action, _)) = self.session_commands.back() {
            *self.positive_feedback.entry(action.clone()).or_insert(0) += 1;
        }
    }
    
    /// Get the user profile
    pub fn profile(&self) -> &UserProfile {
        &self.profile
    }
    
    /// Get mutable profile
    pub fn profile_mut(&mut self) -> &mut UserProfile {
        &mut self.profile
    }
    
    /// Check if an action has low feedback score (user often dislikes it)
    pub fn action_score(&self, action: &str) -> f32 {
        let positive = *self.positive_feedback.get(action).unwrap_or(&0) as f32;
        let negative = *self.negative_feedback.get(action).unwrap_or(&0) as f32;
        
        if positive + negative == 0.0 {
            return 0.5; // Neutral
        }
        
        positive / (positive + negative)
    }
    
    /// Suggest based on partial input (autocomplete)
    pub fn suggest_completions(&self, partial: &str) -> Vec<String> {
        let normalized = normalize_phrase(partial);
        
        self.profile.learned_phrases.iter()
            .filter(|p| normalize_phrase(&p.phrase).starts_with(&normalized))
            .map(|p| p.phrase.clone())
            .take(5)
            .collect()
    }
    
    /// Export learning data for persistence
    pub fn export(&self) -> LearningData {
        LearningData {
            profile: self.profile.clone(),
            positive_feedback: self.positive_feedback.clone(),
            negative_feedback: self.negative_feedback.clone(),
        }
    }
}

impl Default for LearningSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Exportable learning data
#[derive(Debug, Clone)]
pub struct LearningData {
    pub profile: UserProfile,
    pub positive_feedback: HashMap<String, u32>,
    pub negative_feedback: HashMap<String, u32>,
}

impl Clone for UserProfile {
    fn clone(&self) -> Self {
        Self {
            user_id: self.user_id.clone(),
            created_at: self.created_at,
            last_active: self.last_active,
            verbosity_preference: self.verbosity_preference,
            confirmation_preference: self.confirmation_preference,
            learned_phrases: self.learned_phrases.clone(),
            corrections: self.corrections.clone(),
            max_corrections: self.max_corrections,
            preferred_values: self.preferred_values.clone(),
            expertise_level: self.expertise_level,
            total_interactions: self.total_interactions,
            successful_interactions: self.successful_interactions,
            correction_count: self.correction_count,
        }
    }
}

// Helper functions

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn rand_id() -> u64 {
    // Simple pseudo-random based on timestamp
    let ts = current_timestamp();
    ts.wrapping_mul(1103515245).wrapping_add(12345) % 1000000
}

fn normalize_phrase(phrase: &str) -> String {
    phrase.to_lowercase()
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile::new();
        assert!(profile.user_id.starts_with("user_"));
        assert_eq!(profile.expertise_level, ExpertiseLevel::Intermediate);
    }
    
    #[test]
    fn test_learning_phrase() {
        let mut profile = UserProfile::new();
        
        profile.learn_phrase("yo check my money", "check_balance", 0.8);
        profile.learn_phrase("yo check my money", "check_balance", 0.8);
        
        let lookup = profile.lookup_phrase("yo check my money");
        assert!(lookup.is_some());
        assert_eq!(lookup.unwrap().use_count, 2);
    }
    
    #[test]
    fn test_correction_learning() {
        let mut profile = UserProfile::new();
        
        profile.record_correction(
            "send money to bob",
            "check_balance",
            "no, send tokens",
            "send_tokens"
        );
        
        assert_eq!(profile.correction_count, 1);
        
        // Should have learned the phrase
        let lookup = profile.lookup_phrase("send money to bob");
        assert!(lookup.is_some());
        assert_eq!(lookup.unwrap().action, "send_tokens");
    }
    
    #[test]
    fn test_learning_system_enhance() {
        let mut system = LearningSystem::new();
        
        // Teach it a phrase
        system.profile_mut().learn_phrase("yolo balance", "check_balance", 0.9);
        
        // Should now recognize it
        let enhanced = system.enhance_input("yolo balance");
        assert_eq!(enhanced, Some("check_balance".to_string()));
    }
    
    #[test]
    fn test_common_amounts() {
        let mut profile = UserProfile::new();
        
        profile.record_amount(100);
        profile.record_amount(100);
        profile.record_amount(50);
        profile.record_amount(100);
        profile.record_amount(25);
        
        let common = profile.common_amounts(2);
        assert_eq!(common[0].0, 100);  // Most common
        assert_eq!(common[0].1, 3);    // Count of 3
    }
    
    #[test]
    fn test_expertise_progression() {
        let mut profile = UserProfile::new();
        
        // Simulate many successful interactions
        for _ in 0..150 {
            profile.record_interaction(true);
        }
        
        profile.update_expertise();
        assert_eq!(profile.expertise_level, ExpertiseLevel::Expert);
    }
    
    #[test]
    fn test_normalize_phrase() {
        assert_eq!(
            normalize_phrase("  Hello,  World!!!  "),
            "hello world"
        );
    }
}
