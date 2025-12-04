//! Break Reminders for Kāraṇa OS AR Glasses
//!
//! Reminds users to take breaks using 20-20-20 rule and pomodoro technique.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Break type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BreakType {
    /// 20-20-20 rule (every 20 min, look 20 feet away for 20 sec)
    TwentyTwentyTwenty,
    /// Short break (5-10 minutes)
    Short,
    /// Long break (15-30 minutes)
    Long,
    /// Custom break
    Custom(u64),
}

impl BreakType {
    /// Get recommended break duration
    pub fn recommended_duration(&self) -> Duration {
        match self {
            Self::TwentyTwentyTwenty => Duration::from_secs(20),
            Self::Short => Duration::from_secs(300),      // 5 minutes
            Self::Long => Duration::from_secs(900),       // 15 minutes
            Self::Custom(secs) => Duration::from_secs(*secs),
        }
    }
    
    /// Get interval until next break
    pub fn interval(&self) -> Duration {
        match self {
            Self::TwentyTwentyTwenty => Duration::from_secs(1200), // 20 minutes
            Self::Short => Duration::from_secs(1500),             // 25 minutes (pomodoro)
            Self::Long => Duration::from_secs(6000),              // 100 minutes
            Self::Custom(_) => Duration::from_secs(1800),         // 30 minutes
        }
    }
}

/// Break reminder
#[derive(Debug, Clone)]
pub struct BreakReminder {
    /// Break type
    pub break_type: BreakType,
    /// When reminder triggered
    pub triggered_at: Instant,
    /// Was break taken
    pub taken: bool,
    /// Duration of break taken (if any)
    pub break_duration: Option<Duration>,
}

/// Break reminder settings
#[derive(Debug, Clone)]
pub struct BreakSettings {
    /// Enable 20-20-20 rule
    pub twenty_twenty_enabled: bool,
    /// Enable short breaks (pomodoro style)
    pub short_breaks_enabled: bool,
    /// Short break interval
    pub short_break_interval: Duration,
    /// Short break duration
    pub short_break_duration: Duration,
    /// Enable long breaks
    pub long_breaks_enabled: bool,
    /// Long break interval
    pub long_break_interval: Duration,
    /// Long break duration
    pub long_break_duration: Duration,
    /// Snooze duration
    pub snooze_duration: Duration,
    /// Maximum snoozes allowed
    pub max_snoozes: u8,
}

impl Default for BreakSettings {
    fn default() -> Self {
        Self {
            twenty_twenty_enabled: true,
            short_breaks_enabled: true,
            short_break_interval: Duration::from_secs(1500),   // 25 min
            short_break_duration: Duration::from_secs(300),    // 5 min
            long_breaks_enabled: true,
            long_break_interval: Duration::from_secs(6000),    // 100 min
            long_break_duration: Duration::from_secs(900),     // 15 min
            snooze_duration: Duration::from_secs(300),         // 5 min
            max_snoozes: 3,
        }
    }
}

/// Break manager
#[derive(Debug)]
pub struct BreakManager {
    /// Settings
    settings: BreakSettings,
    /// Time since last 20-20-20 break
    last_twenty_twenty: Instant,
    /// Time since last short break
    last_short_break: Instant,
    /// Time since last long break  
    last_long_break: Instant,
    /// Current snooze count
    snooze_count: u8,
    /// Is user on break
    on_break: bool,
    /// Current break type
    current_break: Option<BreakType>,
    /// Break start time
    break_start: Option<Instant>,
    /// Break history
    break_history: VecDeque<BreakReminder>,
    /// Pending reminder
    pending_reminder: Option<BreakReminder>,
    /// Maximum history entries
    max_history: usize,
}

impl BreakManager {
    /// Create new break manager
    pub fn new(settings: BreakSettings) -> Self {
        let now = Instant::now();
        Self {
            settings,
            last_twenty_twenty: now,
            last_short_break: now,
            last_long_break: now,
            snooze_count: 0,
            on_break: false,
            current_break: None,
            break_start: None,
            break_history: VecDeque::with_capacity(100),
            pending_reminder: None,
            max_history: 100,
        }
    }
    
    /// Create with default settings
    pub fn with_defaults() -> Self {
        Self::new(BreakSettings::default())
    }
    
    /// Update settings
    pub fn update_settings(&mut self, settings: BreakSettings) {
        self.settings = settings;
    }
    
    /// Check if break is needed
    pub fn check_break_needed(&mut self) -> Option<BreakType> {
        if self.on_break {
            return None;
        }
        
        let now = Instant::now();
        
        // Check 20-20-20 rule
        if self.settings.twenty_twenty_enabled {
            let elapsed = now.duration_since(self.last_twenty_twenty);
            if elapsed >= BreakType::TwentyTwentyTwenty.interval() {
                return Some(BreakType::TwentyTwentyTwenty);
            }
        }
        
        // Check short break
        if self.settings.short_breaks_enabled {
            let elapsed = now.duration_since(self.last_short_break);
            if elapsed >= self.settings.short_break_interval {
                return Some(BreakType::Short);
            }
        }
        
        // Check long break
        if self.settings.long_breaks_enabled {
            let elapsed = now.duration_since(self.last_long_break);
            if elapsed >= self.settings.long_break_interval {
                return Some(BreakType::Long);
            }
        }
        
        None
    }
    
    /// Time until next break
    pub fn time_until_break(&self) -> Option<(BreakType, Duration)> {
        if self.on_break {
            return None;
        }
        
        let now = Instant::now();
        let mut next_break: Option<(BreakType, Duration)> = None;
        
        // Check 20-20-20
        if self.settings.twenty_twenty_enabled {
            let elapsed = now.duration_since(self.last_twenty_twenty);
            let interval = BreakType::TwentyTwentyTwenty.interval();
            if elapsed < interval {
                let remaining = interval - elapsed;
                if next_break.is_none() || remaining < next_break.as_ref().unwrap().1 {
                    next_break = Some((BreakType::TwentyTwentyTwenty, remaining));
                }
            }
        }
        
        // Check short break
        if self.settings.short_breaks_enabled {
            let elapsed = now.duration_since(self.last_short_break);
            if elapsed < self.settings.short_break_interval {
                let remaining = self.settings.short_break_interval - elapsed;
                if next_break.is_none() || remaining < next_break.as_ref().unwrap().1 {
                    next_break = Some((BreakType::Short, remaining));
                }
            }
        }
        
        // Check long break
        if self.settings.long_breaks_enabled {
            let elapsed = now.duration_since(self.last_long_break);
            if elapsed < self.settings.long_break_interval {
                let remaining = self.settings.long_break_interval - elapsed;
                if next_break.is_none() || remaining < next_break.as_ref().unwrap().1 {
                    next_break = Some((BreakType::Long, remaining));
                }
            }
        }
        
        next_break
    }
    
    /// Start break
    pub fn start_break(&mut self, break_type: BreakType) {
        self.on_break = true;
        self.current_break = Some(break_type);
        self.break_start = Some(Instant::now());
        // Note: snooze_count is NOT reset here - it persists until end_break or skip_break
        
        // Record pending reminder
        self.pending_reminder = Some(BreakReminder {
            break_type,
            triggered_at: Instant::now(),
            taken: true,
            break_duration: None,
        });
    }
    
    /// End break
    pub fn end_break(&mut self) {
        if !self.on_break {
            return;
        }
        
        let now = Instant::now();
        
        // Calculate break duration
        if let Some(start) = self.break_start {
            let duration = now.duration_since(start);
            
            // Update reminder with actual duration
            if let Some(ref mut reminder) = self.pending_reminder {
                reminder.break_duration = Some(duration);
            }
        }
        
        // Store in history
        if let Some(reminder) = self.pending_reminder.take() {
            if self.break_history.len() >= self.max_history {
                self.break_history.pop_front();
            }
            self.break_history.push_back(reminder);
        }
        
        // Update last break times
        if let Some(break_type) = self.current_break {
            match break_type {
                BreakType::TwentyTwentyTwenty => self.last_twenty_twenty = now,
                BreakType::Short => {
                    self.last_short_break = now;
                    self.last_twenty_twenty = now;  // Also reset 20-20-20
                }
                BreakType::Long => {
                    self.last_long_break = now;
                    self.last_short_break = now;
                    self.last_twenty_twenty = now;
                }
                BreakType::Custom(_) => {
                    self.last_twenty_twenty = now;
                }
            }
        }
        
        self.on_break = false;
        self.current_break = None;
        self.break_start = None;
        self.snooze_count = 0;  // Reset snooze count after taking break
    }
    
    /// Skip break
    pub fn skip_break(&mut self) {
        if let Some(break_type) = self.current_break {
            // Record skipped break
            let reminder = BreakReminder {
                break_type,
                triggered_at: Instant::now(),
                taken: false,
                break_duration: None,
            };
            
            if self.break_history.len() >= self.max_history {
                self.break_history.pop_front();
            }
            self.break_history.push_back(reminder);
        }
        
        // Reset timers anyway (don't spam reminders)
        let now = Instant::now();
        if let Some(break_type) = self.current_break {
            match break_type {
                BreakType::TwentyTwentyTwenty => self.last_twenty_twenty = now,
                BreakType::Short => self.last_short_break = now,
                BreakType::Long => self.last_long_break = now,
                BreakType::Custom(_) => {}
            }
        }
        
        self.on_break = false;
        self.current_break = None;
        self.break_start = None;
        self.pending_reminder = None;
        self.snooze_count = 0;  // Reset snooze count after skipping break
    }
    
    /// Snooze break
    pub fn snooze(&mut self) -> bool {
        if self.snooze_count >= self.settings.max_snoozes {
            return false;
        }
        
        self.snooze_count += 1;
        
        // Push back timers by snooze duration
        let snooze = self.settings.snooze_duration;
        let now = Instant::now();
        
        if let Some(break_type) = self.current_break {
            // Temporarily push back the specific timer
            // In reality we'd track snooze expiry separately
            match break_type {
                BreakType::TwentyTwentyTwenty => {
                    self.last_twenty_twenty = now;
                }
                BreakType::Short => {
                    self.last_short_break = now;
                }
                BreakType::Long => {
                    self.last_long_break = now;
                }
                _ => {}
            }
        }
        
        self.on_break = false;
        self.current_break = None;
        self.break_start = None;
        
        true
    }
    
    /// Get snooze count
    pub fn snooze_count(&self) -> u8 {
        self.snooze_count
    }
    
    /// Can snooze
    pub fn can_snooze(&self) -> bool {
        self.snooze_count < self.settings.max_snoozes
    }
    
    /// Is on break
    pub fn is_on_break(&self) -> bool {
        self.on_break
    }
    
    /// Get current break type
    pub fn current_break_type(&self) -> Option<BreakType> {
        self.current_break
    }
    
    /// Get break duration so far
    pub fn current_break_duration(&self) -> Duration {
        self.break_start
            .map(|s| Instant::now().duration_since(s))
            .unwrap_or(Duration::ZERO)
    }
    
    /// Get recommended remaining break time
    pub fn recommended_break_remaining(&self) -> Duration {
        if let Some(break_type) = self.current_break {
            let recommended = break_type.recommended_duration();
            let elapsed = self.current_break_duration();
            
            if elapsed < recommended {
                recommended - elapsed
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        }
    }
    
    /// Get break compliance rate (breaks taken / breaks triggered)
    pub fn compliance_rate(&self) -> f32 {
        if self.break_history.is_empty() {
            return 1.0;
        }
        
        let taken = self.break_history.iter().filter(|b| b.taken).count();
        taken as f32 / self.break_history.len() as f32
    }
    
    /// Get break history
    pub fn history(&self) -> &VecDeque<BreakReminder> {
        &self.break_history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.break_history.clear();
    }
    
    /// Reset all timers
    pub fn reset_timers(&mut self) {
        let now = Instant::now();
        self.last_twenty_twenty = now;
        self.last_short_break = now;
        self.last_long_break = now;
        self.snooze_count = 0;
    }
    
    /// Force trigger break reminder
    pub fn force_break(&mut self, break_type: BreakType) {
        self.start_break(break_type);
    }
}

impl Default for BreakManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_break_manager_creation() {
        let manager = BreakManager::with_defaults();
        assert!(!manager.is_on_break());
    }
    
    #[test]
    fn test_break_types() {
        assert_eq!(
            BreakType::TwentyTwentyTwenty.recommended_duration(),
            Duration::from_secs(20)
        );
        assert_eq!(
            BreakType::Short.recommended_duration(),
            Duration::from_secs(300)
        );
    }
    
    #[test]
    fn test_start_break() {
        let mut manager = BreakManager::with_defaults();
        manager.start_break(BreakType::Short);
        
        assert!(manager.is_on_break());
        assert_eq!(manager.current_break_type(), Some(BreakType::Short));
    }
    
    #[test]
    fn test_end_break() {
        let mut manager = BreakManager::with_defaults();
        manager.start_break(BreakType::Short);
        manager.end_break();
        
        assert!(!manager.is_on_break());
        assert_eq!(manager.history().len(), 1);
        assert!(manager.history().back().unwrap().taken);
    }
    
    #[test]
    fn test_skip_break() {
        let mut manager = BreakManager::with_defaults();
        manager.start_break(BreakType::Short);
        manager.skip_break();
        
        assert!(!manager.is_on_break());
        assert_eq!(manager.history().len(), 1);
        assert!(!manager.history().back().unwrap().taken);
    }
    
    #[test]
    fn test_snooze() {
        let mut manager = BreakManager::with_defaults();
        manager.start_break(BreakType::Short);
        
        assert!(manager.snooze());
        assert!(!manager.is_on_break());
        assert_eq!(manager.snooze_count(), 1);
    }
    
    #[test]
    fn test_max_snoozes() {
        let mut settings = BreakSettings::default();
        settings.max_snoozes = 2;
        
        let mut manager = BreakManager::new(settings);
        
        manager.start_break(BreakType::Short);
        assert!(manager.snooze());
        
        manager.start_break(BreakType::Short);
        assert!(manager.snooze());
        
        manager.start_break(BreakType::Short);
        assert!(!manager.snooze());  // Max reached
    }
    
    #[test]
    fn test_compliance_rate() {
        let mut manager = BreakManager::with_defaults();
        
        // Take one break
        manager.start_break(BreakType::Short);
        manager.end_break();
        
        // Skip one break
        manager.start_break(BreakType::Short);
        manager.skip_break();
        
        assert_eq!(manager.compliance_rate(), 0.5);
    }
    
    #[test]
    fn test_time_until_break() {
        let manager = BreakManager::with_defaults();
        
        // Should have time until next break
        let next = manager.time_until_break();
        assert!(next.is_some());
    }
}
