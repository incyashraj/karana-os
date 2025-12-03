//! Cognitive accessibility features

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Reading assist mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingMode {
    /// Normal display
    Normal,
    /// Bionic reading (bold beginnings)
    Bionic,
    /// Line focus (highlight current line)
    LineFocus,
    /// Word by word highlight
    WordByWord,
    /// Sentence focus
    SentenceFocus,
}

/// Focus assist level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusLevel {
    /// Disabled
    Off,
    /// Subtle (dim distractions slightly)
    Subtle,
    /// Medium (dim significantly)
    Medium,
    /// Strong (hide non-essential elements)
    Strong,
}

/// Animation reduction level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationLevel {
    /// All animations
    Full,
    /// Reduced (essential only)
    Reduced,
    /// Minimal
    Minimal,
    /// No animations
    None,
}

/// Reminder entry
#[derive(Debug, Clone)]
pub struct Reminder {
    /// Unique identifier
    pub id: u64,
    /// Reminder message
    pub message: String,
    /// When to trigger
    pub trigger_time: Instant,
    /// Whether reminder has been shown
    pub shown: bool,
    /// Recurrence interval (None = one-time)
    pub recurrence: Option<Duration>,
}

/// Routine step
#[derive(Debug, Clone)]
pub struct RoutineStep {
    /// Step title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    /// Expected duration
    pub duration: Duration,
    /// Whether step is completed
    pub completed: bool,
}

/// Cognitive assist system
#[derive(Debug)]
pub struct CognitiveAssist {
    /// Reading assist mode
    reading_mode: ReadingMode,
    /// Reading speed (words per minute)
    reading_speed: u32,
    /// Focus assist level
    focus_level: FocusLevel,
    /// Animation reduction level
    animation_level: AnimationLevel,
    /// Simplify UI
    simplified_ui: bool,
    /// Reduce motion
    reduce_motion: bool,
    /// Show time estimates
    time_estimates: bool,
    /// Break reminders enabled
    break_reminders: bool,
    /// Break reminder interval
    break_interval: Duration,
    /// Last break reminder
    last_break: Instant,
    /// Active reminders
    reminders: VecDeque<Reminder>,
    /// Next reminder ID
    next_reminder_id: u64,
    /// Current routine steps
    routine_steps: Vec<RoutineStep>,
    /// Current routine step index
    current_step: usize,
    /// Limit notifications
    notification_limit: Option<u32>,
    /// Notifications shown this period
    notifications_shown: u32,
    /// Last notification period start
    notification_period_start: Instant,
    /// Notification period duration
    notification_period: Duration,
}

impl CognitiveAssist {
    /// Create new cognitive assist system
    pub fn new() -> Self {
        Self {
            reading_mode: ReadingMode::Normal,
            reading_speed: 200,
            focus_level: FocusLevel::Off,
            animation_level: AnimationLevel::Full,
            simplified_ui: false,
            reduce_motion: false,
            time_estimates: true,
            break_reminders: false,
            break_interval: Duration::from_secs(30 * 60), // 30 minutes
            last_break: Instant::now(),
            reminders: VecDeque::new(),
            next_reminder_id: 1,
            routine_steps: Vec::new(),
            current_step: 0,
            notification_limit: None,
            notifications_shown: 0,
            notification_period_start: Instant::now(),
            notification_period: Duration::from_secs(60),
        }
    }
    
    /// Set reading mode
    pub fn set_reading_mode(&mut self, mode: ReadingMode) {
        self.reading_mode = mode;
    }
    
    /// Get reading mode
    pub fn reading_mode(&self) -> ReadingMode {
        self.reading_mode
    }
    
    /// Set reading speed (wpm)
    pub fn set_reading_speed(&mut self, wpm: u32) {
        self.reading_speed = wpm.clamp(50, 500);
    }
    
    /// Get reading speed
    pub fn reading_speed(&self) -> u32 {
        self.reading_speed
    }
    
    /// Calculate reading time for text
    pub fn estimate_reading_time(&self, word_count: u32) -> Duration {
        let minutes = word_count as f32 / self.reading_speed as f32;
        Duration::from_secs_f32(minutes * 60.0)
    }
    
    /// Set focus level
    pub fn set_focus_level(&mut self, level: FocusLevel) {
        self.focus_level = level;
    }
    
    /// Get focus level
    pub fn focus_level(&self) -> FocusLevel {
        self.focus_level
    }
    
    /// Set animation level
    pub fn set_animation_level(&mut self, level: AnimationLevel) {
        self.animation_level = level;
    }
    
    /// Get animation level
    pub fn animation_level(&self) -> AnimationLevel {
        self.animation_level
    }
    
    /// Enable/disable simplified UI
    pub fn set_simplified_ui(&mut self, enabled: bool) {
        self.simplified_ui = enabled;
    }
    
    /// Check if simplified UI is enabled
    pub fn simplified_ui(&self) -> bool {
        self.simplified_ui
    }
    
    /// Enable/disable reduce motion
    pub fn set_reduce_motion(&mut self, enabled: bool) {
        self.reduce_motion = enabled;
        if enabled && self.animation_level == AnimationLevel::Full {
            self.animation_level = AnimationLevel::Reduced;
        }
    }
    
    /// Check if reduce motion is enabled
    pub fn reduce_motion(&self) -> bool {
        self.reduce_motion
    }
    
    /// Enable/disable time estimates
    pub fn set_time_estimates(&mut self, enabled: bool) {
        self.time_estimates = enabled;
    }
    
    /// Check if time estimates are shown
    pub fn time_estimates_enabled(&self) -> bool {
        self.time_estimates
    }
    
    /// Enable/disable break reminders
    pub fn set_break_reminders(&mut self, enabled: bool, interval: Option<Duration>) {
        self.break_reminders = enabled;
        if let Some(interval) = interval {
            self.break_interval = interval;
        }
        self.last_break = Instant::now();
    }
    
    /// Check if break reminders are enabled
    pub fn break_reminders_enabled(&self) -> bool {
        self.break_reminders
    }
    
    /// Check if break is due
    pub fn is_break_due(&self) -> bool {
        self.break_reminders && self.last_break.elapsed() >= self.break_interval
    }
    
    /// Acknowledge break taken
    pub fn acknowledge_break(&mut self) {
        self.last_break = Instant::now();
    }
    
    /// Time until next break
    pub fn time_until_break(&self) -> Option<Duration> {
        if !self.break_reminders {
            return None;
        }
        
        let elapsed = self.last_break.elapsed();
        if elapsed >= self.break_interval {
            Some(Duration::ZERO)
        } else {
            Some(self.break_interval - elapsed)
        }
    }
    
    /// Add reminder
    pub fn add_reminder(&mut self, message: String, delay: Duration, recurrence: Option<Duration>) -> u64 {
        let id = self.next_reminder_id;
        self.next_reminder_id += 1;
        
        let reminder = Reminder {
            id,
            message,
            trigger_time: Instant::now() + delay,
            shown: false,
            recurrence,
        };
        
        self.reminders.push_back(reminder);
        id
    }
    
    /// Remove reminder by ID
    pub fn remove_reminder(&mut self, id: u64) -> bool {
        let len_before = self.reminders.len();
        self.reminders.retain(|r| r.id != id);
        self.reminders.len() < len_before
    }
    
    /// Get pending reminders that are due
    pub fn due_reminders(&self) -> Vec<&Reminder> {
        let now = Instant::now();
        self.reminders
            .iter()
            .filter(|r| !r.shown && now >= r.trigger_time)
            .collect()
    }
    
    /// Mark reminder as shown
    pub fn mark_reminder_shown(&mut self, id: u64) {
        if let Some(reminder) = self.reminders.iter_mut().find(|r| r.id == id) {
            reminder.shown = true;
            
            // If recurring, schedule next occurrence
            if let Some(interval) = reminder.recurrence {
                reminder.trigger_time = Instant::now() + interval;
                reminder.shown = false;
            }
        }
    }
    
    /// Set routine steps
    pub fn set_routine(&mut self, steps: Vec<RoutineStep>) {
        self.routine_steps = steps;
        self.current_step = 0;
    }
    
    /// Get current routine step
    pub fn current_routine_step(&self) -> Option<&RoutineStep> {
        self.routine_steps.get(self.current_step)
    }
    
    /// Get all routine steps
    pub fn routine_steps(&self) -> &[RoutineStep] {
        &self.routine_steps
    }
    
    /// Complete current routine step
    pub fn complete_current_step(&mut self) -> bool {
        if self.current_step < self.routine_steps.len() {
            self.routine_steps[self.current_step].completed = true;
            self.current_step += 1;
            true
        } else {
            false
        }
    }
    
    /// Skip current routine step
    pub fn skip_current_step(&mut self) -> bool {
        if self.current_step < self.routine_steps.len() {
            self.current_step += 1;
            true
        } else {
            false
        }
    }
    
    /// Reset routine
    pub fn reset_routine(&mut self) {
        for step in &mut self.routine_steps {
            step.completed = false;
        }
        self.current_step = 0;
    }
    
    /// Get routine progress (0.0 - 1.0)
    pub fn routine_progress(&self) -> f32 {
        if self.routine_steps.is_empty() {
            return 0.0;
        }
        
        let completed = self.routine_steps.iter().filter(|s| s.completed).count();
        completed as f32 / self.routine_steps.len() as f32
    }
    
    /// Set notification limit
    pub fn set_notification_limit(&mut self, limit: Option<u32>, period: Duration) {
        self.notification_limit = limit;
        self.notification_period = period;
        self.notifications_shown = 0;
        self.notification_period_start = Instant::now();
    }
    
    /// Check if notification can be shown
    pub fn can_show_notification(&mut self) -> bool {
        // Reset period if expired
        if self.notification_period_start.elapsed() >= self.notification_period {
            self.notifications_shown = 0;
            self.notification_period_start = Instant::now();
        }
        
        match self.notification_limit {
            Some(limit) => self.notifications_shown < limit,
            None => true,
        }
    }
    
    /// Record notification shown
    pub fn record_notification(&mut self) {
        self.notifications_shown += 1;
    }
    
    /// Apply bionic reading to text (bold first part of words)
    pub fn bionic_text(&self, text: &str) -> Vec<(String, bool)> {
        if self.reading_mode != ReadingMode::Bionic {
            return vec![(text.to_string(), false)];
        }
        
        let mut result = Vec::new();
        
        for word in text.split_whitespace() {
            let chars: Vec<char> = word.chars().collect();
            let bold_len = match chars.len() {
                0..=2 => chars.len(),
                3..=4 => 2,
                5..=7 => 3,
                _ => chars.len() / 2,
            };
            
            let bold_part: String = chars[..bold_len].iter().collect();
            let normal_part: String = chars[bold_len..].iter().collect();
            
            result.push((bold_part, true));
            result.push((normal_part, false));
            result.push((" ".to_string(), false));
        }
        
        result
    }
    
    /// Get dim level for focus assist (0.0 = no dim, 1.0 = fully dimmed)
    pub fn focus_dim_level(&self) -> f32 {
        match self.focus_level {
            FocusLevel::Off => 0.0,
            FocusLevel::Subtle => 0.2,
            FocusLevel::Medium => 0.5,
            FocusLevel::Strong => 0.8,
        }
    }
    
    /// Update cognitive assist state
    pub fn update(&mut self) {
        // Clean up shown one-time reminders
        self.reminders.retain(|r| !r.shown || r.recurrence.is_some());
    }
}

impl Default for CognitiveAssist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cognitive_assist_creation() {
        let ca = CognitiveAssist::new();
        assert_eq!(ca.reading_mode(), ReadingMode::Normal);
        assert_eq!(ca.focus_level(), FocusLevel::Off);
    }
    
    #[test]
    fn test_reading_time_estimate() {
        let ca = CognitiveAssist::new();
        
        // 200 wpm default
        let time = ca.estimate_reading_time(200);
        assert!((time.as_secs_f32() - 60.0).abs() < 1.0);
    }
    
    #[test]
    fn test_bionic_reading() {
        let mut ca = CognitiveAssist::new();
        ca.set_reading_mode(ReadingMode::Bionic);
        
        let result = ca.bionic_text("Hello");
        
        // Should have bold and non-bold parts
        let bold_parts: Vec<_> = result.iter().filter(|(_, b)| *b).collect();
        assert!(!bold_parts.is_empty());
    }
    
    #[test]
    fn test_break_reminders() {
        let mut ca = CognitiveAssist::new();
        
        assert!(!ca.is_break_due());
        
        ca.set_break_reminders(true, Some(Duration::from_millis(50)));
        
        std::thread::sleep(Duration::from_millis(60));
        assert!(ca.is_break_due());
        
        ca.acknowledge_break();
        assert!(!ca.is_break_due());
    }
    
    #[test]
    fn test_reminders() {
        let mut ca = CognitiveAssist::new();
        
        let id = ca.add_reminder(
            "Test reminder".to_string(),
            Duration::from_millis(10),
            None,
        );
        
        std::thread::sleep(Duration::from_millis(20));
        
        let due = ca.due_reminders();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].message, "Test reminder");
        
        ca.mark_reminder_shown(id);
        ca.update();
        
        // Should be removed after shown (non-recurring)
        let due = ca.due_reminders();
        assert!(due.is_empty());
    }
    
    #[test]
    fn test_routine() {
        let mut ca = CognitiveAssist::new();
        
        let steps = vec![
            RoutineStep {
                title: "Step 1".to_string(),
                description: None,
                duration: Duration::from_secs(60),
                completed: false,
            },
            RoutineStep {
                title: "Step 2".to_string(),
                description: Some("Description".to_string()),
                duration: Duration::from_secs(120),
                completed: false,
            },
        ];
        
        ca.set_routine(steps);
        
        assert_eq!(ca.routine_progress(), 0.0);
        
        ca.complete_current_step();
        assert!((ca.routine_progress() - 0.5).abs() < 0.01);
        
        ca.complete_current_step();
        assert!((ca.routine_progress() - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_notification_limit() {
        let mut ca = CognitiveAssist::new();
        
        ca.set_notification_limit(Some(2), Duration::from_secs(60));
        
        assert!(ca.can_show_notification());
        ca.record_notification();
        assert!(ca.can_show_notification());
        ca.record_notification();
        assert!(!ca.can_show_notification());
    }
    
    #[test]
    fn test_focus_dim_level() {
        let mut ca = CognitiveAssist::new();
        
        assert_eq!(ca.focus_dim_level(), 0.0);
        
        ca.set_focus_level(FocusLevel::Medium);
        assert!((ca.focus_dim_level() - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_animation_level() {
        let mut ca = CognitiveAssist::new();
        
        ca.set_reduce_motion(true);
        assert!(ca.reduce_motion());
        assert_eq!(ca.animation_level(), AnimationLevel::Reduced);
    }
}
