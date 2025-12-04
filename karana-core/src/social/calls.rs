//! Call Management for Kāraṇa OS AR Glasses
//!
//! Voice and video call handling with AR interface.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Call type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallType {
    /// Voice call
    Voice,
    /// Video call
    Video,
}

/// Call state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallState {
    /// Idle
    Idle,
    /// Outgoing call ringing
    Ringing,
    /// Incoming call
    Incoming,
    /// Call connected
    Connected,
    /// Call on hold
    OnHold,
    /// Call ended
    Ended,
    /// Call failed
    Failed,
}

/// Call direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallDirection {
    /// Incoming call
    Incoming,
    /// Outgoing call
    Outgoing,
}

/// Call end reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallEndReason {
    /// Normal end
    Normal,
    /// Declined by recipient
    Declined,
    /// No answer
    NoAnswer,
    /// Busy
    Busy,
    /// Network error
    NetworkError,
    /// Cancelled
    Cancelled,
}

/// Active call
#[derive(Debug, Clone)]
pub struct Call {
    /// Call ID
    pub id: String,
    /// Contact ID
    pub contact_id: String,
    /// Contact name
    pub contact_name: String,
    /// Call type
    pub call_type: CallType,
    /// Call state
    pub state: CallState,
    /// Direction
    pub direction: CallDirection,
    /// Start time
    pub start_time: Instant,
    /// Connect time (when answered)
    pub connect_time: Option<Instant>,
    /// End time
    pub end_time: Option<Instant>,
    /// Is muted
    pub muted: bool,
    /// Is speaker on
    pub speaker: bool,
    /// Is video enabled (for video calls)
    pub video_enabled: bool,
    /// End reason
    pub end_reason: Option<CallEndReason>,
}

impl Call {
    /// Create new outgoing call
    pub fn outgoing(id: String, contact_id: String, contact_name: String, call_type: CallType) -> Self {
        Self {
            id,
            contact_id,
            contact_name,
            call_type,
            state: CallState::Ringing,
            direction: CallDirection::Outgoing,
            start_time: Instant::now(),
            connect_time: None,
            end_time: None,
            muted: false,
            speaker: false,
            video_enabled: call_type == CallType::Video,
            end_reason: None,
        }
    }
    
    /// Create new incoming call
    pub fn incoming(id: String, contact_id: String, contact_name: String, call_type: CallType) -> Self {
        Self {
            id,
            contact_id,
            contact_name,
            call_type,
            state: CallState::Incoming,
            direction: CallDirection::Incoming,
            start_time: Instant::now(),
            connect_time: None,
            end_time: None,
            muted: false,
            speaker: false,
            video_enabled: call_type == CallType::Video,
            end_reason: None,
        }
    }
    
    /// Answer call
    pub fn answer(&mut self) {
        if self.state == CallState::Incoming {
            self.state = CallState::Connected;
            self.connect_time = Some(Instant::now());
        }
    }
    
    /// Connect call (for outgoing when answered)
    pub fn connect(&mut self) {
        if self.state == CallState::Ringing {
            self.state = CallState::Connected;
            self.connect_time = Some(Instant::now());
        }
    }
    
    /// End call
    pub fn end(&mut self, reason: CallEndReason) {
        self.state = CallState::Ended;
        self.end_time = Some(Instant::now());
        self.end_reason = Some(reason);
    }
    
    /// Put on hold
    pub fn hold(&mut self) {
        if self.state == CallState::Connected {
            self.state = CallState::OnHold;
        }
    }
    
    /// Resume from hold
    pub fn resume(&mut self) {
        if self.state == CallState::OnHold {
            self.state = CallState::Connected;
        }
    }
    
    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }
    
    /// Toggle speaker
    pub fn toggle_speaker(&mut self) {
        self.speaker = !self.speaker;
    }
    
    /// Toggle video
    pub fn toggle_video(&mut self) {
        if self.call_type == CallType::Video {
            self.video_enabled = !self.video_enabled;
        }
    }
    
    /// Get call duration
    pub fn duration(&self) -> Duration {
        match (self.connect_time, self.end_time) {
            (Some(connect), Some(end)) => end.duration_since(connect),
            (Some(connect), None) => Instant::now().duration_since(connect),
            _ => Duration::ZERO,
        }
    }
    
    /// Is active (connected or on hold)
    pub fn is_active(&self) -> bool {
        matches!(self.state, CallState::Connected | CallState::OnHold)
    }
    
    /// Format duration
    pub fn format_duration(&self) -> String {
        let dur = self.duration();
        let secs = dur.as_secs();
        let mins = secs / 60;
        let hours = mins / 60;
        
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, mins % 60, secs % 60)
        } else {
            format!("{}:{:02}", mins, secs % 60)
        }
    }
}

/// Call history entry
#[derive(Debug, Clone)]
pub struct CallHistoryEntry {
    /// Call ID
    pub id: String,
    /// Contact ID
    pub contact_id: String,
    /// Contact name
    pub contact_name: String,
    /// Call type
    pub call_type: CallType,
    /// Direction
    pub direction: CallDirection,
    /// Duration
    pub duration: Duration,
    /// Timestamp (as duration since some epoch)
    pub timestamp: Instant,
    /// End reason
    pub end_reason: CallEndReason,
    /// Was answered
    pub answered: bool,
}

impl From<&Call> for CallHistoryEntry {
    fn from(call: &Call) -> Self {
        Self {
            id: call.id.clone(),
            contact_id: call.contact_id.clone(),
            contact_name: call.contact_name.clone(),
            call_type: call.call_type,
            direction: call.direction,
            duration: call.duration(),
            timestamp: call.start_time,
            end_reason: call.end_reason.unwrap_or(CallEndReason::Normal),
            answered: call.connect_time.is_some(),
        }
    }
}

/// Call manager
#[derive(Debug)]
pub struct CallManager {
    /// Current active call
    active_call: Option<Call>,
    /// Call history
    history: VecDeque<CallHistoryEntry>,
    /// Max history entries
    max_history: usize,
    /// Calls made today
    calls_made_today: u32,
    /// Calls received today
    calls_received_today: u32,
    /// Total call time today
    call_time_today: Duration,
    /// Last call ID
    last_call_id: u64,
}

impl CallManager {
    /// Create new call manager
    pub fn new() -> Self {
        Self {
            active_call: None,
            history: VecDeque::with_capacity(100),
            max_history: 100,
            calls_made_today: 0,
            calls_received_today: 0,
            call_time_today: Duration::ZERO,
            last_call_id: 0,
        }
    }
    
    /// Start outgoing call
    pub fn start_call(&mut self, contact_id: String, contact_name: String, call_type: CallType) -> Option<&Call> {
        if self.active_call.is_some() {
            return None;
        }
        
        self.last_call_id += 1;
        let call_id = format!("call-{}", self.last_call_id);
        
        let call = Call::outgoing(call_id, contact_id, contact_name, call_type);
        self.active_call = Some(call);
        self.calls_made_today += 1;
        
        self.active_call.as_ref()
    }
    
    /// Handle incoming call
    pub fn incoming_call(&mut self, call_id: String, contact_id: String, contact_name: String, call_type: CallType) -> bool {
        if self.active_call.is_some() {
            return false;
        }
        
        let call = Call::incoming(call_id, contact_id, contact_name, call_type);
        self.active_call = Some(call);
        self.calls_received_today += 1;
        
        true
    }
    
    /// Answer incoming call
    pub fn answer(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.answer();
        }
    }
    
    /// Decline incoming call
    pub fn decline(&mut self) {
        if let Some(ref mut call) = self.active_call {
            if call.state == CallState::Incoming {
                call.end(CallEndReason::Declined);
                self.end_and_store_call();
            }
        }
    }
    
    /// End current call
    pub fn end_call(&mut self) {
        if let Some(ref mut call) = self.active_call {
            if call.is_active() {
                call.end(CallEndReason::Normal);
            } else if call.state == CallState::Ringing {
                call.end(CallEndReason::Cancelled);
            }
            self.end_and_store_call();
        }
    }
    
    /// Internal: end and store call in history
    fn end_and_store_call(&mut self) {
        if let Some(call) = self.active_call.take() {
            self.call_time_today += call.duration();
            
            let entry = CallHistoryEntry::from(&call);
            if self.history.len() >= self.max_history {
                self.history.pop_front();
            }
            self.history.push_back(entry);
        }
    }
    
    /// Hold call
    pub fn hold(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.hold();
        }
    }
    
    /// Resume call
    pub fn resume(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.resume();
        }
    }
    
    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.toggle_mute();
        }
    }
    
    /// Toggle speaker
    pub fn toggle_speaker(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.toggle_speaker();
        }
    }
    
    /// Toggle video
    pub fn toggle_video(&mut self) {
        if let Some(ref mut call) = self.active_call {
            call.toggle_video();
        }
    }
    
    /// Get active call
    pub fn active_call(&self) -> Option<&Call> {
        self.active_call.as_ref()
    }
    
    /// Get mutable active call
    pub fn active_call_mut(&mut self) -> Option<&mut Call> {
        self.active_call.as_mut()
    }
    
    /// Is in call
    pub fn is_in_call(&self) -> bool {
        self.active_call.as_ref().map(|c| c.is_active()).unwrap_or(false)
    }
    
    /// Has incoming call
    pub fn has_incoming_call(&self) -> bool {
        self.active_call.as_ref().map(|c| c.state == CallState::Incoming).unwrap_or(false)
    }
    
    /// Get call history
    pub fn history(&self) -> &VecDeque<CallHistoryEntry> {
        &self.history
    }
    
    /// Get missed calls
    pub fn missed_calls(&self) -> Vec<&CallHistoryEntry> {
        self.history.iter()
            .filter(|e| e.direction == CallDirection::Incoming && !e.answered)
            .collect()
    }
    
    /// Get call history with contact
    pub fn history_with(&self, contact_id: &str) -> Vec<&CallHistoryEntry> {
        self.history.iter()
            .filter(|e| e.contact_id == contact_id)
            .collect()
    }
    
    /// Get calls made today
    pub fn calls_made_today(&self) -> u32 {
        self.calls_made_today
    }
    
    /// Get calls received today
    pub fn calls_received_today(&self) -> u32 {
        self.calls_received_today
    }
    
    /// Get call time today
    pub fn call_time_today(&self) -> Duration {
        // Include current call
        let current = self.active_call
            .as_ref()
            .filter(|c| c.is_active())
            .map(|c| c.duration())
            .unwrap_or(Duration::ZERO);
        
        self.call_time_today + current
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Reset daily stats
    pub fn reset_daily_stats(&mut self) {
        self.calls_made_today = 0;
        self.calls_received_today = 0;
        self.call_time_today = Duration::ZERO;
    }
}

impl Default for CallManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_call_creation() {
        let call = Call::outgoing(
            "1".to_string(),
            "alice".to_string(),
            "Alice".to_string(),
            CallType::Voice,
        );
        
        assert_eq!(call.state, CallState::Ringing);
        assert_eq!(call.direction, CallDirection::Outgoing);
    }
    
    #[test]
    fn test_call_answer() {
        let mut call = Call::incoming(
            "1".to_string(),
            "alice".to_string(),
            "Alice".to_string(),
            CallType::Voice,
        );
        
        assert_eq!(call.state, CallState::Incoming);
        
        call.answer();
        assert_eq!(call.state, CallState::Connected);
        assert!(call.connect_time.is_some());
    }
    
    #[test]
    fn test_call_duration() {
        let mut call = Call::incoming(
            "1".to_string(),
            "alice".to_string(),
            "Alice".to_string(),
            CallType::Voice,
        );
        
        call.answer();
        std::thread::sleep(Duration::from_millis(10));
        
        let dur = call.duration();
        assert!(dur.as_millis() >= 10);
    }
    
    #[test]
    fn test_call_manager() {
        let mut manager = CallManager::new();
        
        manager.start_call("alice".to_string(), "Alice".to_string(), CallType::Voice);
        
        assert!(manager.active_call().is_some());
        assert_eq!(manager.calls_made_today(), 1);
    }
    
    #[test]
    fn test_incoming_call() {
        let mut manager = CallManager::new();
        
        manager.incoming_call(
            "1".to_string(),
            "alice".to_string(),
            "Alice".to_string(),
            CallType::Voice,
        );
        
        assert!(manager.has_incoming_call());
        
        manager.answer();
        assert!(manager.is_in_call());
    }
    
    #[test]
    fn test_end_call() {
        let mut manager = CallManager::new();
        
        manager.start_call("alice".to_string(), "Alice".to_string(), CallType::Voice);
        
        // Simulate connect
        if let Some(call) = manager.active_call_mut() {
            call.connect();
        }
        
        manager.end_call();
        
        assert!(manager.active_call().is_none());
        assert_eq!(manager.history().len(), 1);
    }
    
    #[test]
    fn test_toggle_mute() {
        let mut manager = CallManager::new();
        
        manager.start_call("alice".to_string(), "Alice".to_string(), CallType::Voice);
        
        assert!(!manager.active_call().unwrap().muted);
        
        manager.toggle_mute();
        assert!(manager.active_call().unwrap().muted);
    }
}
