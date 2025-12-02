//! # Kāraṇa Timer System
//!
//! Real-time timers and alarms for the glasses HUD.
//!
//! ## Features
//! - Countdown timers with callbacks
//! - Recurring alarms
//! - Stopwatch functionality
//! - Background timer execution

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;

/// Timer state
#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Running,
    Paused,
    Completed,
    Cancelled,
}

/// Timer type
#[derive(Debug, Clone, PartialEq)]
pub enum TimerType {
    /// One-shot countdown
    Countdown,
    /// Recurring alarm
    Recurring { interval: Duration },
    /// Stopwatch (counts up)
    Stopwatch,
}

/// A single timer instance
#[derive(Debug, Clone)]
pub struct Timer {
    /// Unique timer ID
    pub id: u64,
    /// Human-readable name
    pub name: String,
    /// Timer type
    pub timer_type: TimerType,
    /// Total duration (for countdown)
    pub duration: Duration,
    /// Time remaining (for countdown) or elapsed (for stopwatch)
    pub remaining: Duration,
    /// Timer state
    pub state: TimerState,
    /// When the timer was started
    pub started_at: Option<Instant>,
    /// When the timer was last paused
    pub paused_at: Option<Instant>,
    /// Number of times alarm has triggered (for recurring)
    pub trigger_count: u32,
    /// Optional message to display on completion
    pub message: Option<String>,
}

impl Timer {
    /// Create a new countdown timer
    pub fn countdown(id: u64, name: &str, duration: Duration) -> Self {
        Self {
            id,
            name: name.to_string(),
            timer_type: TimerType::Countdown,
            duration,
            remaining: duration,
            state: TimerState::Paused,
            started_at: None,
            paused_at: None,
            trigger_count: 0,
            message: None,
        }
    }

    /// Create a new stopwatch
    pub fn stopwatch(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            timer_type: TimerType::Stopwatch,
            duration: Duration::ZERO,
            remaining: Duration::ZERO,
            state: TimerState::Paused,
            started_at: None,
            paused_at: None,
            trigger_count: 0,
            message: None,
        }
    }

    /// Create a recurring alarm
    pub fn recurring(id: u64, name: &str, interval: Duration) -> Self {
        Self {
            id,
            name: name.to_string(),
            timer_type: TimerType::Recurring { interval },
            duration: interval,
            remaining: interval,
            state: TimerState::Paused,
            started_at: None,
            paused_at: None,
            trigger_count: 0,
            message: None,
        }
    }

    /// Start or resume the timer
    pub fn start(&mut self) {
        match self.state {
            TimerState::Paused => {
                self.started_at = Some(Instant::now());
                self.state = TimerState::Running;
            }
            TimerState::Completed if matches!(self.timer_type, TimerType::Recurring { .. }) => {
                // Restart recurring timer
                self.remaining = self.duration;
                self.started_at = Some(Instant::now());
                self.state = TimerState::Running;
            }
            _ => {}
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            if let Some(started) = self.started_at {
                let elapsed = started.elapsed();
                match self.timer_type {
                    TimerType::Countdown | TimerType::Recurring { .. } => {
                        self.remaining = self.remaining.saturating_sub(elapsed);
                    }
                    TimerType::Stopwatch => {
                        self.remaining += elapsed;
                    }
                }
            }
            self.state = TimerState::Paused;
            self.paused_at = Some(Instant::now());
            self.started_at = None;
        }
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        match self.timer_type {
            TimerType::Countdown | TimerType::Recurring { .. } => {
                self.remaining = self.duration;
            }
            TimerType::Stopwatch => {
                self.remaining = Duration::ZERO;
            }
        }
        self.state = TimerState::Paused;
        self.started_at = None;
        self.paused_at = None;
    }

    /// Update timer state (call periodically)
    pub fn update(&mut self) -> bool {
        if self.state != TimerState::Running {
            return false;
        }

        if let Some(started) = self.started_at {
            let elapsed = started.elapsed();
            
            match self.timer_type {
                TimerType::Countdown => {
                    if elapsed >= self.remaining {
                        self.remaining = Duration::ZERO;
                        self.state = TimerState::Completed;
                        self.trigger_count += 1;
                        return true; // Timer completed
                    }
                }
                TimerType::Recurring { interval } => {
                    if elapsed >= self.remaining {
                        // Timer triggered, reset for next interval
                        self.trigger_count += 1;
                        self.remaining = interval;
                        self.started_at = Some(Instant::now());
                        return true; // Timer triggered
                    }
                }
                TimerType::Stopwatch => {
                    // Stopwatch just keeps counting
                }
            }
        }

        false
    }

    /// Get current remaining time (for countdown) or elapsed time (for stopwatch)
    pub fn current_time(&self) -> Duration {
        match self.state {
            TimerState::Running => {
                if let Some(started) = self.started_at {
                    let elapsed = started.elapsed();
                    match self.timer_type {
                        TimerType::Countdown | TimerType::Recurring { .. } => {
                            self.remaining.saturating_sub(elapsed)
                        }
                        TimerType::Stopwatch => {
                            self.remaining + elapsed
                        }
                    }
                } else {
                    self.remaining
                }
            }
            _ => self.remaining,
        }
    }

    /// Format time for display (MM:SS or HH:MM:SS)
    pub fn format_time(&self) -> String {
        let secs = self.current_time().as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
}

/// Timer manager for multiple timers
pub struct TimerManager {
    timers: Arc<Mutex<HashMap<u64, Timer>>>,
    next_id: AtomicU64,
    running: Arc<AtomicBool>,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(Timer) + Send + 'static>>>>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(1),
            running: Arc::new(AtomicBool::new(false)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create and start a countdown timer
    pub fn set_timer(&self, name: &str, duration: Duration, message: Option<&str>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut timer = Timer::countdown(id, name, duration);
        timer.message = message.map(|s| s.to_string());
        timer.start();

        let mut timers = self.timers.lock().unwrap();
        timers.insert(id, timer);

        log::info!("[TIMER] ⏱️ Created timer #{}: {} ({} seconds)", 
            id, name, duration.as_secs());

        id
    }

    /// Create a stopwatch
    pub fn create_stopwatch(&self, name: &str) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let timer = Timer::stopwatch(id, name);

        let mut timers = self.timers.lock().unwrap();
        timers.insert(id, timer);

        log::info!("[TIMER] ⏱️ Created stopwatch #{}: {}", id, name);

        id
    }

    /// Set a recurring alarm
    pub fn set_alarm(&self, name: &str, interval: Duration, message: Option<&str>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut timer = Timer::recurring(id, name, interval);
        timer.message = message.map(|s| s.to_string());
        timer.start();

        let mut timers = self.timers.lock().unwrap();
        timers.insert(id, timer);

        log::info!("[TIMER] 🔔 Created alarm #{}: {} (every {} seconds)", 
            id, name, interval.as_secs());

        id
    }

    /// Get a timer by ID
    pub fn get(&self, id: u64) -> Option<Timer> {
        let timers = self.timers.lock().unwrap();
        timers.get(&id).cloned()
    }

    /// Start a timer
    pub fn start(&self, id: u64) -> bool {
        let mut timers = self.timers.lock().unwrap();
        if let Some(timer) = timers.get_mut(&id) {
            timer.start();
            true
        } else {
            false
        }
    }

    /// Pause a timer
    pub fn pause(&self, id: u64) -> bool {
        let mut timers = self.timers.lock().unwrap();
        if let Some(timer) = timers.get_mut(&id) {
            timer.pause();
            true
        } else {
            false
        }
    }

    /// Reset a timer
    pub fn reset(&self, id: u64) -> bool {
        let mut timers = self.timers.lock().unwrap();
        if let Some(timer) = timers.get_mut(&id) {
            timer.reset();
            true
        } else {
            false
        }
    }

    /// Cancel/delete a timer
    pub fn cancel(&self, id: u64) -> bool {
        let mut timers = self.timers.lock().unwrap();
        timers.remove(&id).is_some()
    }

    /// List all active timers
    pub fn list_active(&self) -> Vec<Timer> {
        let timers = self.timers.lock().unwrap();
        timers.values()
            .filter(|t| t.state == TimerState::Running || t.state == TimerState::Paused)
            .cloned()
            .collect()
    }

    /// Register a callback for timer completion
    pub fn on_complete<F>(&self, callback: F)
    where
        F: Fn(Timer) + Send + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }

    /// Update all timers and trigger callbacks
    pub fn update(&self) -> Vec<Timer> {
        let mut completed = Vec::new();
        let mut timers = self.timers.lock().unwrap();
        
        for timer in timers.values_mut() {
            if timer.update() {
                completed.push(timer.clone());
            }
        }

        drop(timers);

        // Trigger callbacks for completed timers
        if !completed.is_empty() {
            let callbacks = self.callbacks.lock().unwrap();
            for timer in &completed {
                log::info!("[TIMER] ⏰ Timer completed: {} ({})", timer.name, timer.id);
                for callback in callbacks.iter() {
                    callback(timer.clone());
                }
            }
        }

        // Clean up completed non-recurring timers
        let mut timers = self.timers.lock().unwrap();
        timers.retain(|_, t| {
            !(t.state == TimerState::Completed && !matches!(t.timer_type, TimerType::Recurring { .. }))
        });

        completed
    }

    /// Start background update thread
    pub fn start_background(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        let timers = self.timers.clone();
        let callbacks = self.callbacks.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            log::info!("[TIMER] Background timer thread started");
            
            while running.load(Ordering::Relaxed) {
                // Update timers
                let mut completed = Vec::new();
                {
                    let mut timers = timers.lock().unwrap();
                    for timer in timers.values_mut() {
                        if timer.update() {
                            completed.push(timer.clone());
                        }
                    }
                }

                // Trigger callbacks
                if !completed.is_empty() {
                    let callbacks = callbacks.lock().unwrap();
                    for timer in &completed {
                        for callback in callbacks.iter() {
                            callback(timer.clone());
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
            
            log::info!("[TIMER] Background timer thread stopped");
        });

        Ok(())
    }

    /// Stop background thread
    pub fn stop_background(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Get formatted status for HUD display
    pub fn hud_status(&self) -> String {
        let timers = self.timers.lock().unwrap();
        let active: Vec<_> = timers.values()
            .filter(|t| t.state == TimerState::Running)
            .collect();

        if active.is_empty() {
            return String::new();
        }

        let timer = active.first().unwrap();
        format!("⏱️ {} {}", timer.name, timer.format_time())
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick timer parsing from natural language
pub fn parse_timer_duration(input: &str) -> Option<Duration> {
    let input = input.to_lowercase();
    
    // Parse patterns like "5 minutes", "30 seconds", "1 hour"
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    for i in 0..parts.len() {
        if let Ok(num) = parts[i].parse::<u64>() {
            if let Some(unit) = parts.get(i + 1) {
                let unit = unit.trim_end_matches('s'); // Remove plural
                match unit {
                    "second" | "sec" => return Some(Duration::from_secs(num)),
                    "minute" | "min" => return Some(Duration::from_secs(num * 60)),
                    "hour" | "hr" => return Some(Duration::from_secs(num * 3600)),
                    _ => {}
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countdown_timer() {
        let mut timer = Timer::countdown(1, "test", Duration::from_secs(5));
        assert_eq!(timer.state, TimerState::Paused);
        
        timer.start();
        assert_eq!(timer.state, TimerState::Running);
        
        timer.pause();
        assert_eq!(timer.state, TimerState::Paused);
    }

    #[test]
    fn test_stopwatch() {
        let timer = Timer::stopwatch(1, "test");
        assert_eq!(timer.timer_type, TimerType::Stopwatch);
        assert_eq!(timer.remaining, Duration::ZERO);
    }

    #[test]
    fn test_format_time() {
        let timer = Timer::countdown(1, "test", Duration::from_secs(3661)); // 1h 1m 1s
        assert_eq!(timer.format_time(), "01:01:01");
        
        let timer2 = Timer::countdown(2, "test", Duration::from_secs(65)); // 1m 5s
        assert_eq!(timer2.format_time(), "01:05");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_timer_duration("5 minutes"), Some(Duration::from_secs(300)));
        assert_eq!(parse_timer_duration("30 seconds"), Some(Duration::from_secs(30)));
        assert_eq!(parse_timer_duration("1 hour"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_timer_duration("invalid"), None);
    }

    #[test]
    fn test_timer_manager() {
        let manager = TimerManager::new();
        let id = manager.set_timer("test", Duration::from_secs(60), None);
        
        let timer = manager.get(id).unwrap();
        assert_eq!(timer.name, "test");
        assert_eq!(timer.state, TimerState::Running);
        
        manager.pause(id);
        let timer = manager.get(id).unwrap();
        assert_eq!(timer.state, TimerState::Paused);
    }
}
