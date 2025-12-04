//! Usage Tracking for Kāraṇa OS AR Glasses
//!
//! Tracks daily and weekly usage patterns.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session start time
    pub start: Instant,
    /// Session end time (None if ongoing)
    pub end: Option<Instant>,
    /// Session duration
    pub duration: Duration,
    /// Is session active
    pub active: bool,
}

impl SessionInfo {
    /// Create new session
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            end: None,
            duration: Duration::ZERO,
            active: true,
        }
    }
    
    /// End session
    pub fn end(&mut self) {
        if self.active {
            let now = Instant::now();
            self.end = Some(now);
            self.duration = now.duration_since(self.start);
            self.active = false;
        }
    }
    
    /// Get current duration
    pub fn current_duration(&self) -> Duration {
        if self.active {
            Instant::now().duration_since(self.start)
        } else {
            self.duration
        }
    }
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage statistics
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Total usage today
    pub daily_total: Duration,
    /// Total usage this week
    pub weekly_total: Duration,
    /// Number of sessions today
    pub daily_sessions: usize,
    /// Average session length today
    pub avg_session_length: Duration,
    /// Longest session today
    pub longest_session: Duration,
    /// Most active hour (0-23)
    pub most_active_hour: u8,
}

/// Usage tracker
#[derive(Debug)]
pub struct UsageTracker {
    /// Current session
    current_session: Option<SessionInfo>,
    /// Today's sessions
    today_sessions: Vec<SessionInfo>,
    /// Historical daily totals (last 7 days)
    daily_history: VecDeque<Duration>,
    /// Total usage today
    daily_usage: Duration,
    /// Total usage this week
    weekly_usage: Duration,
    /// Usage by hour (for pattern analysis)
    hourly_usage: [Duration; 24],
    /// Last update time
    last_update: Instant,
}

impl UsageTracker {
    /// Create new usage tracker
    pub fn new() -> Self {
        Self {
            current_session: None,
            today_sessions: Vec::new(),
            daily_history: VecDeque::with_capacity(7),
            daily_usage: Duration::ZERO,
            weekly_usage: Duration::ZERO,
            hourly_usage: [Duration::ZERO; 24],
            last_update: Instant::now(),
        }
    }
    
    /// Start new session
    pub fn start_session(&mut self) {
        if self.current_session.is_none() {
            self.current_session = Some(SessionInfo::new());
        }
    }
    
    /// End current session
    pub fn end_session(&mut self) {
        if let Some(mut session) = self.current_session.take() {
            session.end();
            self.daily_usage += session.duration;
            self.today_sessions.push(session);
        }
    }
    
    /// Is session active
    pub fn is_session_active(&self) -> bool {
        self.current_session.is_some()
    }
    
    /// Get current session duration
    pub fn current_session_duration(&self) -> Duration {
        self.current_session
            .as_ref()
            .map(|s| s.current_duration())
            .unwrap_or(Duration::ZERO)
    }
    
    /// Get daily usage
    pub fn daily_usage(&self) -> Duration {
        let current = self.current_session_duration();
        self.daily_usage + current
    }
    
    /// Get weekly usage
    pub fn weekly_usage(&self) -> Duration {
        let daily = self.daily_usage();
        self.weekly_usage + daily
    }
    
    /// Update tracker
    pub fn update(&mut self, delta: Duration) {
        if let Some(ref session) = self.current_session {
            // Would track hourly usage in real implementation
            // based on current time
        }
        
        self.last_update = Instant::now();
    }
    
    /// Get today's session count
    pub fn session_count(&self) -> usize {
        let current = if self.current_session.is_some() { 1 } else { 0 };
        self.today_sessions.len() + current
    }
    
    /// Get average session length
    pub fn average_session_length(&self) -> Duration {
        let total_sessions = self.session_count();
        if total_sessions == 0 {
            return Duration::ZERO;
        }
        
        let total_time = self.daily_usage();
        total_time / total_sessions as u32
    }
    
    /// Get longest session today
    pub fn longest_session(&self) -> Duration {
        let past_longest = self.today_sessions
            .iter()
            .map(|s| s.duration)
            .max()
            .unwrap_or(Duration::ZERO);
        
        let current = self.current_session_duration();
        past_longest.max(current)
    }
    
    /// Get most active hour
    pub fn most_active_hour(&self) -> u8 {
        self.hourly_usage
            .iter()
            .enumerate()
            .max_by_key(|(_, d)| *d)
            .map(|(h, _)| h as u8)
            .unwrap_or(0)
    }
    
    /// Get usage statistics
    pub fn stats(&self) -> UsageStats {
        UsageStats {
            daily_total: self.daily_usage(),
            weekly_total: self.weekly_usage(),
            daily_sessions: self.session_count(),
            avg_session_length: self.average_session_length(),
            longest_session: self.longest_session(),
            most_active_hour: self.most_active_hour(),
        }
    }
    
    /// Record day end (for daily rollover)
    pub fn end_day(&mut self) {
        // Move current daily to history
        if self.daily_history.len() >= 7 {
            self.daily_history.pop_front();
        }
        self.daily_history.push_back(self.daily_usage);
        
        // Update weekly total
        self.weekly_usage = self.daily_history.iter().sum();
        
        // Reset daily
        self.daily_usage = Duration::ZERO;
        self.today_sessions.clear();
        self.hourly_usage = [Duration::ZERO; 24];
    }
    
    /// Get daily average (last 7 days)
    pub fn daily_average(&self) -> Duration {
        if self.daily_history.is_empty() {
            return self.daily_usage();
        }
        
        let total: Duration = self.daily_history.iter().sum();
        total / self.daily_history.len() as u32
    }
    
    /// Reset all tracking
    pub fn reset(&mut self) {
        self.current_session = None;
        self.today_sessions.clear();
        self.daily_history.clear();
        self.daily_usage = Duration::ZERO;
        self.weekly_usage = Duration::ZERO;
        self.hourly_usage = [Duration::ZERO; 24];
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usage_tracker_creation() {
        let tracker = UsageTracker::new();
        assert!(!tracker.is_session_active());
    }
    
    #[test]
    fn test_start_session() {
        let mut tracker = UsageTracker::new();
        tracker.start_session();
        
        assert!(tracker.is_session_active());
    }
    
    #[test]
    fn test_end_session() {
        let mut tracker = UsageTracker::new();
        tracker.start_session();
        
        std::thread::sleep(Duration::from_millis(10));
        
        tracker.end_session();
        
        assert!(!tracker.is_session_active());
        assert!(tracker.daily_usage().as_millis() >= 10);
    }
    
    #[test]
    fn test_session_count() {
        let mut tracker = UsageTracker::new();
        
        tracker.start_session();
        tracker.end_session();
        tracker.start_session();
        tracker.end_session();
        
        assert_eq!(tracker.session_count(), 2);
    }
    
    #[test]
    fn test_session_info() {
        let session = SessionInfo::new();
        assert!(session.active);
        
        let mut session = SessionInfo::new();
        std::thread::sleep(Duration::from_millis(10));
        session.end();
        
        assert!(!session.active);
        assert!(session.duration.as_millis() >= 10);
    }
    
    #[test]
    fn test_stats() {
        let tracker = UsageTracker::new();
        let stats = tracker.stats();
        
        assert_eq!(stats.daily_sessions, 0);
    }
    
    #[test]
    fn test_end_day() {
        let mut tracker = UsageTracker::new();
        
        // Simulate some usage
        tracker.start_session();
        std::thread::sleep(Duration::from_millis(10));
        tracker.end_session();
        
        let daily = tracker.daily_usage();
        tracker.end_day();
        
        // Daily should be reset
        assert_eq!(tracker.daily_usage(), Duration::ZERO);
        // History should have the day's usage
        assert_eq!(tracker.daily_history.len(), 1);
    }
}
