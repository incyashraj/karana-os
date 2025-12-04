//! Health & Wellness Monitoring for Kāraṇa OS AR Glasses
//!
//! Tracks eye strain, usage patterns, posture, and wellness to protect
//! users from the potential negative effects of extended AR use.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use nalgebra::Vector3;

pub mod eye_strain;
pub mod posture;
pub mod usage;
pub mod breaks;
pub mod ambient;

pub use eye_strain::{EyeStrainMonitor, EyeStrainLevel, BlinkRate};
pub use posture::{PostureMonitor, PostureState, NeckPosition};
pub use usage::{UsageTracker, UsageStats, SessionInfo};
pub use breaks::{BreakManager, BreakType, BreakReminder};
pub use ambient::{AmbientMonitor, LightingConditions, AmbientAlert};

/// Health alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Minor concern
    Minor,
    /// Moderate concern
    Moderate,
    /// Serious concern
    Serious,
    /// Critical - immediate action recommended
    Critical,
}

/// Health alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertType {
    /// Eye strain detected
    EyeStrain,
    /// Blink rate low
    LowBlinkRate,
    /// Extended use without break
    ExtendedUse,
    /// Poor posture detected
    PoorPosture,
    /// Neck strain risk
    NeckStrain,
    /// Low light conditions
    LowLight,
    /// High brightness
    HighBrightness,
    /// Blue light exposure
    BlueLightExposure,
    /// Motion sickness risk
    MotionSickness,
    /// Focus fatigue
    FocusFatigue,
    /// Session limit reached
    SessionLimit,
    /// Daily limit reached
    DailyLimit,
}

impl AlertType {
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            AlertType::EyeStrain => "Eye strain detected - consider taking a break",
            AlertType::LowBlinkRate => "Blink rate is low - remember to blink",
            AlertType::ExtendedUse => "Extended use detected - time for a break",
            AlertType::PoorPosture => "Poor posture detected - adjust your position",
            AlertType::NeckStrain => "Neck strain risk - check head position",
            AlertType::LowLight => "Low ambient light - may cause eye strain",
            AlertType::HighBrightness => "High brightness detected - consider reducing",
            AlertType::BlueLightExposure => "Blue light exposure - enable filter",
            AlertType::MotionSickness => "Motion sickness risk - take a break",
            AlertType::FocusFatigue => "Focus fatigue - rest your eyes",
            AlertType::SessionLimit => "Session limit reached",
            AlertType::DailyLimit => "Daily use limit reached",
        }
    }
    
    /// Default severity
    pub fn default_severity(&self) -> AlertSeverity {
        match self {
            AlertType::EyeStrain => AlertSeverity::Moderate,
            AlertType::LowBlinkRate => AlertSeverity::Minor,
            AlertType::ExtendedUse => AlertSeverity::Moderate,
            AlertType::PoorPosture => AlertSeverity::Minor,
            AlertType::NeckStrain => AlertSeverity::Moderate,
            AlertType::LowLight => AlertSeverity::Minor,
            AlertType::HighBrightness => AlertSeverity::Minor,
            AlertType::BlueLightExposure => AlertSeverity::Minor,
            AlertType::MotionSickness => AlertSeverity::Serious,
            AlertType::FocusFatigue => AlertSeverity::Moderate,
            AlertType::SessionLimit => AlertSeverity::Moderate,
            AlertType::DailyLimit => AlertSeverity::Serious,
        }
    }
}

/// Health alert
#[derive(Debug, Clone)]
pub struct HealthAlert {
    /// Alert type
    pub alert_type: AlertType,
    /// Severity
    pub severity: AlertSeverity,
    /// Message
    pub message: String,
    /// Suggested action
    pub suggestion: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Is acknowledged
    pub acknowledged: bool,
    /// Auto-dismiss after
    pub auto_dismiss: Option<Duration>,
}

impl HealthAlert {
    /// Create new alert
    pub fn new(alert_type: AlertType, message: String) -> Self {
        Self {
            alert_type,
            severity: alert_type.default_severity(),
            message,
            suggestion: Self::default_suggestion(alert_type),
            timestamp: Instant::now(),
            acknowledged: false,
            auto_dismiss: Some(Duration::from_secs(30)),
        }
    }
    
    /// Default suggestion for alert type
    fn default_suggestion(alert_type: AlertType) -> String {
        match alert_type {
            AlertType::EyeStrain => "Look at something 20 feet away for 20 seconds",
            AlertType::LowBlinkRate => "Consciously blink a few times",
            AlertType::ExtendedUse => "Take a 5-10 minute break",
            AlertType::PoorPosture => "Sit up straight and relax shoulders",
            AlertType::NeckStrain => "Gently stretch your neck",
            AlertType::LowLight => "Increase ambient lighting",
            AlertType::HighBrightness => "Reduce display brightness",
            AlertType::BlueLightExposure => "Enable night mode or blue light filter",
            AlertType::MotionSickness => "Remove glasses and focus on a fixed point",
            AlertType::FocusFatigue => "Close your eyes for 30 seconds",
            AlertType::SessionLimit => "Consider ending this session",
            AlertType::DailyLimit => "Consider taking a longer break",
        }.to_string()
    }
    
    /// Set severity
    pub fn with_severity(mut self, severity: AlertSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    /// Age of alert
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

/// Wellness settings
#[derive(Debug, Clone)]
pub struct WellnessSettings {
    /// Enable eye strain monitoring
    pub eye_strain_monitoring: bool,
    /// Enable posture monitoring
    pub posture_monitoring: bool,
    /// Enable break reminders
    pub break_reminders: bool,
    /// Enable blink reminders
    pub blink_reminders: bool,
    /// Session time limit (0 = unlimited)
    pub session_limit: Duration,
    /// Daily use limit (0 = unlimited)
    pub daily_limit: Duration,
    /// Break interval
    pub break_interval: Duration,
    /// Break duration
    pub break_duration: Duration,
    /// Blue light filter auto-enable
    pub auto_blue_light_filter: bool,
    /// Blue light filter start hour
    pub blue_light_start_hour: u8,
    /// Brightness auto-adjust
    pub auto_brightness: bool,
    /// Motion sensitivity (for motion sickness prevention)
    pub motion_sensitivity: f32,
}

impl Default for WellnessSettings {
    fn default() -> Self {
        Self {
            eye_strain_monitoring: true,
            posture_monitoring: true,
            break_reminders: true,
            blink_reminders: true,
            session_limit: Duration::from_secs(2 * 3600), // 2 hours
            daily_limit: Duration::from_secs(8 * 3600), // 8 hours
            break_interval: Duration::from_secs(20 * 60), // 20 minutes
            break_duration: Duration::from_secs(20), // 20 seconds (20-20-20 rule)
            auto_blue_light_filter: true,
            blue_light_start_hour: 20, // 8 PM
            auto_brightness: true,
            motion_sensitivity: 0.5,
        }
    }
}

/// Health monitoring manager
pub struct WellnessManager {
    /// Settings
    settings: WellnessSettings,
    /// Eye strain monitor
    eye_strain: EyeStrainMonitor,
    /// Posture monitor
    posture: PostureMonitor,
    /// Usage tracker
    usage: UsageTracker,
    /// Break manager
    breaks: BreakManager,
    /// Ambient monitor
    ambient: AmbientMonitor,
    /// Active alerts
    active_alerts: Vec<HealthAlert>,
    /// Alert history
    alert_history: VecDeque<HealthAlert>,
    /// Max history size
    max_history: usize,
    /// Alert cooldowns (to prevent spam)
    alert_cooldowns: HashMap<AlertType, Instant>,
    /// Alert cooldown duration
    cooldown_duration: Duration,
    /// Is monitoring active
    monitoring_active: bool,
    /// Current session start
    session_start: Option<Instant>,
    /// Total alerts generated
    total_alerts: u64,
}

impl WellnessManager {
    /// Create new wellness manager
    pub fn new() -> Self {
        Self {
            settings: WellnessSettings::default(),
            eye_strain: EyeStrainMonitor::new(),
            posture: PostureMonitor::new(),
            usage: UsageTracker::new(),
            breaks: BreakManager::with_defaults(),
            ambient: AmbientMonitor::new(),
            active_alerts: Vec::new(),
            alert_history: VecDeque::new(),
            max_history: 100,
            alert_cooldowns: HashMap::new(),
            cooldown_duration: Duration::from_secs(300), // 5 minutes
            monitoring_active: false,
            session_start: None,
            total_alerts: 0,
        }
    }
    
    /// Start monitoring session
    pub fn start_session(&mut self) {
        self.monitoring_active = true;
        self.session_start = Some(Instant::now());
        self.usage.start_session();
    }
    
    /// End monitoring session
    pub fn end_session(&mut self) {
        self.monitoring_active = false;
        self.usage.end_session();
        self.session_start = None;
    }
    
    /// Is session active
    pub fn is_session_active(&self) -> bool {
        self.monitoring_active
    }
    
    /// Get session duration
    pub fn session_duration(&self) -> Duration {
        self.session_start
            .map(|start| Instant::now().duration_since(start))
            .unwrap_or(Duration::ZERO)
    }
    
    /// Update health monitoring
    pub fn update(&mut self, delta: Duration) -> Vec<HealthAlert> {
        if !self.monitoring_active {
            return Vec::new();
        }
        
        let mut new_alerts = Vec::new();
        
        // Update eye strain
        if self.settings.eye_strain_monitoring {
            if let Some(alert) = self.eye_strain.update(delta) {
                if self.should_alert(alert.alert_type) {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Update posture
        if self.settings.posture_monitoring {
            if let Some(alert) = self.posture.update(delta) {
                if self.should_alert(alert.alert_type) {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Check if break is needed
        if self.settings.break_reminders {
            if let Some(break_type) = self.breaks.check_break_needed() {
                let alert = HealthAlert::new(
                    AlertType::ExtendedUse,
                    format!("Time for a {} break", match break_type {
                        BreakType::TwentyTwentyTwenty => "20-20-20",
                        BreakType::Short => "short",
                        BreakType::Long => "long",
                        BreakType::Custom(_) => "custom",
                    }),
                );
                if self.should_alert(alert.alert_type) {
                    new_alerts.push(alert);
                }
            }
        }
        
        // Update usage
        self.usage.update(delta);
        
        // Check session limit
        if self.settings.session_limit.as_secs() > 0 {
            if self.session_duration() >= self.settings.session_limit {
                if self.should_alert(AlertType::SessionLimit) {
                    new_alerts.push(HealthAlert::new(
                        AlertType::SessionLimit,
                        format!("Session limit of {} minutes reached", 
                            self.settings.session_limit.as_secs() / 60),
                    ));
                }
            }
        }
        
        // Check daily limit
        let daily = self.usage.daily_usage();
        if self.settings.daily_limit.as_secs() > 0 && daily >= self.settings.daily_limit {
            if self.should_alert(AlertType::DailyLimit) {
                new_alerts.push(HealthAlert::new(
                    AlertType::DailyLimit,
                    format!("Daily limit of {} hours reached",
                        self.settings.daily_limit.as_secs() / 3600),
                ));
            }
        }
        
        // Record alerts
        for alert in &new_alerts {
            self.total_alerts += 1;
            self.active_alerts.push(alert.clone());
            self.alert_cooldowns.insert(alert.alert_type, Instant::now());
        }
        
        new_alerts
    }
    
    /// Check if should alert (cooldown check)
    fn should_alert(&self, alert_type: AlertType) -> bool {
        if let Some(last) = self.alert_cooldowns.get(&alert_type) {
            Instant::now().duration_since(*last) >= self.cooldown_duration
        } else {
            true
        }
    }
    
    /// Record blink
    pub fn record_blink(&mut self) {
        self.eye_strain.record_blink();
    }
    
    /// Record gaze data
    pub fn record_gaze(&mut self, focus_distance: f32) {
        self.eye_strain.record_focus_distance(focus_distance);
    }
    
    /// Record head position
    pub fn record_head_position(&mut self, position: Vector3<f32>, rotation: Vector3<f32>) {
        self.posture.record_position(position, rotation);
    }
    
    /// Record ambient light
    pub fn record_ambient_light(&mut self, lux: f32, delta: Duration) {
        self.ambient.update(lux, delta);
    }
    
    /// Acknowledge alert
    pub fn acknowledge_alert(&mut self, index: usize) {
        if index < self.active_alerts.len() {
            let alert = self.active_alerts.remove(index);
            self.add_to_history(alert);
        }
    }
    
    /// Dismiss all alerts
    pub fn dismiss_all_alerts(&mut self) {
        while let Some(alert) = self.active_alerts.pop() {
            self.add_to_history(alert);
        }
    }
    
    /// Add alert to history
    fn add_to_history(&mut self, mut alert: HealthAlert) {
        alert.acknowledged = true;
        
        if self.alert_history.len() >= self.max_history {
            self.alert_history.pop_front();
        }
        self.alert_history.push_back(alert);
    }
    
    /// Get active alerts
    pub fn active_alerts(&self) -> &[HealthAlert] {
        &self.active_alerts
    }
    
    /// Get active alert count
    pub fn active_alert_count(&self) -> usize {
        self.active_alerts.len()
    }
    
    /// Get settings
    pub fn settings(&self) -> &WellnessSettings {
        &self.settings
    }
    
    /// Get mutable settings
    pub fn settings_mut(&mut self) -> &mut WellnessSettings {
        &mut self.settings
    }
    
    /// Get usage tracker
    pub fn usage(&self) -> &UsageTracker {
        &self.usage
    }
    
    /// Get eye strain monitor
    pub fn eye_strain(&self) -> &EyeStrainMonitor {
        &self.eye_strain
    }
    
    /// Get posture monitor
    pub fn posture(&self) -> &PostureMonitor {
        &self.posture
    }
    
    /// Get ambient monitor
    pub fn ambient(&self) -> &AmbientMonitor {
        &self.ambient
    }
    
    /// Get break manager
    pub fn breaks(&self) -> &BreakManager {
        &self.breaks
    }
    
    /// Start a break
    pub fn start_break(&mut self, break_type: BreakType) {
        self.breaks.start_break(break_type);
    }
    
    /// End current break
    pub fn end_break(&mut self) {
        self.breaks.end_break();
    }
    
    /// Get wellness score (0-100)
    pub fn wellness_score(&self) -> u8 {
        let mut score = 100i32;
        
        // Deduct for eye strain
        let eye_strain_level = self.eye_strain.current_level();
        score -= match eye_strain_level {
            EyeStrainLevel::None => 0,
            EyeStrainLevel::Low => 5,
            EyeStrainLevel::Moderate => 15,
            EyeStrainLevel::High => 30,
            EyeStrainLevel::Severe => 50,
        };
        
        // Deduct for poor posture
        if !self.posture.is_good_posture() {
            score -= 10;
        }
        
        // Deduct for low break compliance
        let compliance = self.breaks.compliance_rate();
        if compliance < 0.5 {
            score -= 15;
        } else if compliance < 0.8 {
            score -= 5;
        }
        
        // Deduct for active alerts
        score -= (self.active_alerts.len() * 5) as i32;
        
        score.max(0).min(100) as u8
    }
    
    /// Generate health report
    pub fn generate_report(&self) -> WellnessReport {
        let history = self.breaks.history();
        let breaks_taken = history.iter().filter(|b| b.taken).count();
        let breaks_missed = history.iter().filter(|b| !b.taken).count();
        
        WellnessReport {
            wellness_score: self.wellness_score(),
            session_duration: self.session_duration(),
            daily_usage: self.usage.daily_usage(),
            eye_strain_level: self.eye_strain.current_level(),
            average_blink_rate: self.eye_strain.average_blink_rate(),
            posture_good_percentage: self.posture.good_posture_percentage(),
            breaks_taken,
            breaks_missed,
            total_alerts: self.total_alerts,
            active_alerts: self.active_alerts.len(),
        }
    }
}

impl Default for WellnessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Wellness report
#[derive(Debug, Clone)]
pub struct WellnessReport {
    /// Overall wellness score (0-100)
    pub wellness_score: u8,
    /// Current session duration
    pub session_duration: Duration,
    /// Today's total usage
    pub daily_usage: Duration,
    /// Current eye strain level
    pub eye_strain_level: EyeStrainLevel,
    /// Average blink rate (per minute)
    pub average_blink_rate: f32,
    /// Percentage of time with good posture
    pub posture_good_percentage: f32,
    /// Breaks taken today
    pub breaks_taken: usize,
    /// Breaks missed today
    pub breaks_missed: usize,
    /// Total alerts generated this session
    pub total_alerts: u64,
    /// Current active alerts
    pub active_alerts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wellness_manager_creation() {
        let wm = WellnessManager::new();
        assert!(!wm.is_session_active());
    }
    
    #[test]
    fn test_start_session() {
        let mut wm = WellnessManager::new();
        wm.start_session();
        
        assert!(wm.is_session_active());
    }
    
    #[test]
    fn test_session_duration() {
        let mut wm = WellnessManager::new();
        wm.start_session();
        
        // Allow some time to pass
        std::thread::sleep(Duration::from_millis(10));
        
        let duration = wm.session_duration();
        assert!(duration.as_millis() >= 10);
    }
    
    #[test]
    fn test_wellness_score() {
        let wm = WellnessManager::new();
        let score = wm.wellness_score();
        assert!(score <= 100);
    }
    
    #[test]
    fn test_alert_creation() {
        let alert = HealthAlert::new(AlertType::EyeStrain, "Test".to_string());
        assert!(!alert.acknowledged);
        assert_eq!(alert.alert_type, AlertType::EyeStrain);
    }
    
    #[test]
    fn test_alert_severity() {
        assert!(AlertSeverity::Critical > AlertSeverity::Minor);
        assert!(AlertType::MotionSickness.default_severity() > AlertType::LowLight.default_severity());
    }
    
    #[test]
    fn test_default_settings() {
        let settings = WellnessSettings::default();
        assert!(settings.eye_strain_monitoring);
        assert!(settings.break_reminders);
    }
    
    #[test]
    fn test_generate_report() {
        let wm = WellnessManager::new();
        let report = wm.generate_report();
        
        assert!(report.wellness_score <= 100);
    }
}
