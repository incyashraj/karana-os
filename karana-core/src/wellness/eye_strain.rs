//! Eye Strain Monitoring for Kāraṇa OS AR Glasses
//!
//! Tracks blink rate, focus distance, and eye fatigue indicators.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use super::{HealthAlert, AlertType};

/// Eye strain severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EyeStrainLevel {
    /// No strain detected
    None,
    /// Low strain
    Low,
    /// Moderate strain
    Moderate,
    /// High strain
    High,
    /// Severe strain
    Severe,
}

impl EyeStrainLevel {
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            EyeStrainLevel::None => "Eyes are rested",
            EyeStrainLevel::Low => "Minor eye fatigue",
            EyeStrainLevel::Moderate => "Moderate eye strain",
            EyeStrainLevel::High => "High eye strain",
            EyeStrainLevel::Severe => "Severe eye strain - take a break",
        }
    }
}

/// Blink rate data
#[derive(Debug, Clone)]
pub struct BlinkRate {
    /// Blinks per minute
    pub bpm: f32,
    /// Is rate healthy (normal is 15-20 bpm)
    pub is_healthy: bool,
    /// Measurement period
    pub period: Duration,
}

impl BlinkRate {
    /// Normal blink rate range
    const NORMAL_MIN: f32 = 15.0;
    const NORMAL_MAX: f32 = 20.0;
    
    /// Check if rate is healthy
    pub fn check_healthy(bpm: f32) -> bool {
        bpm >= Self::NORMAL_MIN && bpm <= Self::NORMAL_MAX * 1.5
    }
}

/// Eye strain monitor
#[derive(Debug)]
pub struct EyeStrainMonitor {
    /// Blink timestamps (for rate calculation)
    blinks: VecDeque<Instant>,
    /// Blink window duration
    blink_window: Duration,
    /// Focus distance history (meters)
    focus_distances: VecDeque<(Instant, f32)>,
    /// Focus window duration
    focus_window: Duration,
    /// Current strain level
    current_level: EyeStrainLevel,
    /// Time at current strain level
    level_duration: Duration,
    /// Low blink rate duration threshold
    low_blink_threshold: Duration,
    /// Last blink reminder
    last_blink_reminder: Option<Instant>,
    /// Blink reminder interval
    blink_reminder_interval: Duration,
    /// Total session blinks
    total_blinks: u64,
    /// Session start
    session_start: Instant,
}

impl EyeStrainMonitor {
    /// Create new eye strain monitor
    pub fn new() -> Self {
        Self {
            blinks: VecDeque::new(),
            blink_window: Duration::from_secs(60),
            focus_distances: VecDeque::new(),
            focus_window: Duration::from_secs(300), // 5 minutes
            current_level: EyeStrainLevel::None,
            level_duration: Duration::ZERO,
            low_blink_threshold: Duration::from_secs(30),
            last_blink_reminder: None,
            blink_reminder_interval: Duration::from_secs(120),
            total_blinks: 0,
            session_start: Instant::now(),
        }
    }
    
    /// Record a blink
    pub fn record_blink(&mut self) {
        self.blinks.push_back(Instant::now());
        self.total_blinks += 1;
        
        // Clean old blinks
        let cutoff = Instant::now() - self.blink_window;
        while self.blinks.front().map(|t| *t < cutoff).unwrap_or(false) {
            self.blinks.pop_front();
        }
    }
    
    /// Record focus distance
    pub fn record_focus_distance(&mut self, distance: f32) {
        self.focus_distances.push_back((Instant::now(), distance));
        
        // Clean old data
        let cutoff = Instant::now() - self.focus_window;
        while self.focus_distances.front().map(|(t, _)| *t < cutoff).unwrap_or(false) {
            self.focus_distances.pop_front();
        }
    }
    
    /// Get current blink rate
    pub fn current_blink_rate(&self) -> BlinkRate {
        let window_secs = self.blink_window.as_secs_f32();
        let bpm = (self.blinks.len() as f32 / window_secs) * 60.0;
        
        BlinkRate {
            bpm,
            is_healthy: BlinkRate::check_healthy(bpm),
            period: self.blink_window,
        }
    }
    
    /// Get average blink rate for session
    pub fn average_blink_rate(&self) -> f32 {
        let session_duration = Instant::now().duration_since(self.session_start);
        let minutes = session_duration.as_secs_f32() / 60.0;
        
        if minutes > 0.0 {
            self.total_blinks as f32 / minutes
        } else {
            0.0
        }
    }
    
    /// Get average focus distance
    pub fn average_focus_distance(&self) -> Option<f32> {
        if self.focus_distances.is_empty() {
            return None;
        }
        
        let sum: f32 = self.focus_distances.iter().map(|(_, d)| *d).sum();
        Some(sum / self.focus_distances.len() as f32)
    }
    
    /// Check if focus is too close (causes strain)
    pub fn is_focus_too_close(&self) -> bool {
        self.average_focus_distance()
            .map(|d| d < 0.5) // Less than 50cm
            .unwrap_or(false)
    }
    
    /// Get current eye strain level
    pub fn current_level(&self) -> EyeStrainLevel {
        self.current_level
    }
    
    /// Calculate eye strain level
    fn calculate_strain_level(&self) -> EyeStrainLevel {
        let mut strain_score = 0;
        
        // Low blink rate increases strain
        let blink_rate = self.current_blink_rate();
        if blink_rate.bpm < 10.0 {
            strain_score += 3;
        } else if blink_rate.bpm < 15.0 {
            strain_score += 1;
        }
        
        // Close focus increases strain
        if self.is_focus_too_close() {
            strain_score += 2;
        }
        
        // Prolonged use increases strain
        let session_mins = Instant::now().duration_since(self.session_start).as_secs() / 60;
        if session_mins > 120 {
            strain_score += 3;
        } else if session_mins > 60 {
            strain_score += 2;
        } else if session_mins > 30 {
            strain_score += 1;
        }
        
        match strain_score {
            0 => EyeStrainLevel::None,
            1..=2 => EyeStrainLevel::Low,
            3..=4 => EyeStrainLevel::Moderate,
            5..=6 => EyeStrainLevel::High,
            _ => EyeStrainLevel::Severe,
        }
    }
    
    /// Update monitor (call periodically)
    pub fn update(&mut self, delta: Duration) -> Option<HealthAlert> {
        let new_level = self.calculate_strain_level();
        
        if new_level != self.current_level {
            self.current_level = new_level;
            self.level_duration = Duration::ZERO;
        } else {
            self.level_duration += delta;
        }
        
        // Generate alert if strain is moderate or above for sustained period
        if self.current_level >= EyeStrainLevel::Moderate && 
           self.level_duration >= Duration::from_secs(60) {
            return Some(HealthAlert::new(
                AlertType::EyeStrain,
                format!("Eye strain level: {:?}", self.current_level),
            ));
        }
        
        // Check for low blink rate
        let blink_rate = self.current_blink_rate();
        if !blink_rate.is_healthy && blink_rate.bpm < 10.0 {
            if self.should_remind_blink() {
                self.last_blink_reminder = Some(Instant::now());
                return Some(HealthAlert::new(
                    AlertType::LowBlinkRate,
                    format!("Blink rate is low: {:.1} bpm", blink_rate.bpm),
                ));
            }
        }
        
        None
    }
    
    /// Check if should send blink reminder
    fn should_remind_blink(&self) -> bool {
        match self.last_blink_reminder {
            Some(last) => Instant::now().duration_since(last) >= self.blink_reminder_interval,
            None => true,
        }
    }
    
    /// Reset session
    pub fn reset_session(&mut self) {
        self.blinks.clear();
        self.focus_distances.clear();
        self.total_blinks = 0;
        self.session_start = Instant::now();
        self.current_level = EyeStrainLevel::None;
        self.level_duration = Duration::ZERO;
    }
}

impl Default for EyeStrainMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eye_strain_monitor_creation() {
        let monitor = EyeStrainMonitor::new();
        assert_eq!(monitor.current_level(), EyeStrainLevel::None);
    }
    
    #[test]
    fn test_record_blink() {
        let mut monitor = EyeStrainMonitor::new();
        
        for _ in 0..15 {
            monitor.record_blink();
        }
        
        assert_eq!(monitor.total_blinks, 15);
    }
    
    #[test]
    fn test_blink_rate_healthy() {
        assert!(BlinkRate::check_healthy(17.0));
        assert!(!BlinkRate::check_healthy(5.0));
    }
    
    #[test]
    fn test_focus_distance() {
        let mut monitor = EyeStrainMonitor::new();
        
        monitor.record_focus_distance(0.3);
        monitor.record_focus_distance(0.4);
        
        assert!(monitor.is_focus_too_close());
    }
    
    #[test]
    fn test_strain_levels() {
        assert!(EyeStrainLevel::Severe > EyeStrainLevel::None);
        assert!(EyeStrainLevel::High > EyeStrainLevel::Low);
    }
    
    #[test]
    fn test_average_focus() {
        let mut monitor = EyeStrainMonitor::new();
        
        monitor.record_focus_distance(1.0);
        monitor.record_focus_distance(2.0);
        monitor.record_focus_distance(3.0);
        
        let avg = monitor.average_focus_distance().unwrap();
        assert!((avg - 2.0).abs() < 0.01);
    }
}
