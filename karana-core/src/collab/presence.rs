//! Presence System
//!
//! Tracks and shares user presence in collaborative sessions.
//! Shows other users' positions, head orientation, gaze, and activity status.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::spatial::WorldPosition;
use super::ParticipantId;

/// User presence state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenceState {
    /// World position
    pub position: WorldPosition,
    /// Head orientation (quaternion: x, y, z, w)
    pub head_orientation: [f32; 4],
    /// Gaze direction (normalized vector)
    pub gaze_direction: [f32; 3],
    /// Current activity
    pub activity: PresenceActivity,
    /// Status message
    pub status: Option<String>,
    /// Last update timestamp (ms)
    pub updated_at: u64,
}

impl PresenceState {
    /// Create new presence at a position
    pub fn at_position(position: WorldPosition) -> Self {
        Self {
            position,
            head_orientation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            gaze_direction: [0.0, 0.0, -1.0], // Looking forward
            activity: PresenceActivity::Active,
            status: None,
            updated_at: Self::now(),
        }
    }
    
    /// Update position
    pub fn set_position(&mut self, position: WorldPosition) {
        self.position = position;
        self.updated_at = Self::now();
    }
    
    /// Update head orientation
    pub fn set_head_orientation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.head_orientation = [x, y, z, w];
        self.updated_at = Self::now();
    }
    
    /// Update gaze direction
    pub fn set_gaze(&mut self, x: f32, y: f32, z: f32) {
        // Normalize
        let len = (x * x + y * y + z * z).sqrt();
        if len > 0.001 {
            self.gaze_direction = [x / len, y / len, z / len];
        }
        self.updated_at = Self::now();
    }
    
    /// Set activity status
    pub fn set_activity(&mut self, activity: PresenceActivity) {
        self.activity = activity;
        self.updated_at = Self::now();
    }
    
    /// Set status message
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
        self.updated_at = Self::now();
    }
    
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// Check if presence is stale
    pub fn is_stale(&self, timeout: Duration) -> bool {
        let now = Self::now();
        now.saturating_sub(self.updated_at) > timeout.as_millis() as u64
    }
}

/// User activity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PresenceActivity {
    /// User is active and engaged
    #[default]
    Active,
    /// User is idle (no recent interaction)
    Idle,
    /// User is away (glasses removed or paused)
    Away,
    /// User is in do-not-disturb mode
    DoNotDisturb,
    /// User is typing/composing
    Typing,
    /// User is speaking
    Speaking,
    /// User is viewing shared content
    Viewing,
    /// User is editing shared content
    Editing,
}

impl PresenceActivity {
    pub fn icon(&self) -> &'static str {
        match self {
            PresenceActivity::Active => "🟢",
            PresenceActivity::Idle => "🟡",
            PresenceActivity::Away => "⚫",
            PresenceActivity::DoNotDisturb => "🔴",
            PresenceActivity::Typing => "✍️",
            PresenceActivity::Speaking => "🎤",
            PresenceActivity::Viewing => "👁️",
            PresenceActivity::Editing => "✏️",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            PresenceActivity::Active => "Active",
            PresenceActivity::Idle => "Idle",
            PresenceActivity::Away => "Away",
            PresenceActivity::DoNotDisturb => "Do Not Disturb",
            PresenceActivity::Typing => "Typing",
            PresenceActivity::Speaking => "Speaking",
            PresenceActivity::Viewing => "Viewing",
            PresenceActivity::Editing => "Editing",
        }
    }
}

/// Avatar representation for AR presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceAvatar {
    /// Avatar style
    pub style: AvatarStyle,
    /// Primary color (hex)
    pub color: String,
    /// Display name label
    pub show_name: bool,
    /// Gaze ray visualization
    pub show_gaze: bool,
    /// Activity indicator
    pub show_activity: bool,
}

impl Default for PresenceAvatar {
    fn default() -> Self {
        Self {
            style: AvatarStyle::Orb,
            color: "#4A9EFF".to_string(),
            show_name: true,
            show_gaze: false,
            show_activity: true,
        }
    }
}

/// Avatar visualization styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarStyle {
    /// Simple floating orb
    Orb,
    /// Floating head model
    Head,
    /// Full body silhouette
    Silhouette,
    /// Minimal dot indicator
    Dot,
    /// Custom 3D model
    Custom,
}

/// Presence manager - tracks all participants' presence
pub struct PresenceManager {
    /// Local user's presence
    local_presence: PresenceState,
    /// Remote participants' presence
    remote_presence: HashMap<ParticipantId, RemotePresence>,
    /// Avatar configuration
    avatar_config: PresenceAvatar,
    /// Update interval
    update_interval: Duration,
    /// Last broadcast time
    last_broadcast: Option<std::time::Instant>,
}

/// Remote participant presence with rendering info
#[derive(Debug, Clone)]
pub struct RemotePresence {
    /// Participant ID
    pub participant_id: ParticipantId,
    /// Display name
    pub display_name: String,
    /// Current presence state
    pub state: PresenceState,
    /// Avatar config
    pub avatar: PresenceAvatar,
    /// Interpolated position for smooth rendering
    pub render_position: WorldPosition,
    /// Previous position for interpolation
    prev_position: WorldPosition,
    /// Interpolation progress
    interp_progress: f32,
}

impl RemotePresence {
    pub fn new(participant_id: ParticipantId, display_name: String) -> Self {
        Self {
            participant_id,
            display_name,
            state: PresenceState::default(),
            avatar: PresenceAvatar::default(),
            render_position: WorldPosition::default(),
            prev_position: WorldPosition::default(),
            interp_progress: 1.0,
        }
    }
    
    /// Update presence state (triggers interpolation)
    pub fn update_state(&mut self, state: PresenceState) {
        self.prev_position = self.render_position.clone();
        self.state = state;
        self.interp_progress = 0.0;
    }
    
    /// Tick interpolation for smooth movement
    pub fn tick(&mut self, dt: f32) {
        if self.interp_progress < 1.0 {
            self.interp_progress = (self.interp_progress + dt * 10.0).min(1.0);
            
            // Lerp position
            let t = self.interp_progress;
            self.render_position = WorldPosition {
                room_id: self.state.position.room_id.clone(),
                local: crate::spatial::world_coords::LocalCoord {
                    x: self.prev_position.local.x + (self.state.position.local.x - self.prev_position.local.x) * t,
                    y: self.prev_position.local.y + (self.state.position.local.y - self.prev_position.local.y) * t,
                    z: self.prev_position.local.z + (self.state.position.local.z - self.prev_position.local.z) * t,
                },
                gps: self.state.position.gps.clone(),
                floor: self.state.position.floor,
                version: self.state.position.version,
            };
        }
    }
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new() -> Self {
        Self {
            local_presence: PresenceState::default(),
            remote_presence: HashMap::new(),
            avatar_config: PresenceAvatar::default(),
            update_interval: Duration::from_millis(100), // 10 Hz updates
            last_broadcast: None,
        }
    }
    
    /// Get local presence state
    pub fn local_presence(&self) -> &PresenceState {
        &self.local_presence
    }
    
    /// Update local presence
    pub fn update_local_presence(&mut self, state: PresenceState) {
        self.local_presence = state;
    }
    
    /// Set local position
    pub fn set_position(&mut self, position: WorldPosition) {
        self.local_presence.set_position(position);
    }
    
    /// Set local head orientation
    pub fn set_head_orientation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.local_presence.set_head_orientation(x, y, z, w);
    }
    
    /// Set local gaze direction
    pub fn set_gaze(&mut self, x: f32, y: f32, z: f32) {
        self.local_presence.set_gaze(x, y, z);
    }
    
    /// Set local activity
    pub fn set_activity(&mut self, activity: PresenceActivity) {
        self.local_presence.set_activity(activity);
    }
    
    /// Get avatar config
    pub fn avatar_config(&self) -> &PresenceAvatar {
        &self.avatar_config
    }
    
    /// Set avatar config
    pub fn set_avatar_config(&mut self, config: PresenceAvatar) {
        self.avatar_config = config;
    }
    
    /// Add or update remote presence
    pub fn update_remote(&mut self, participant_id: ParticipantId, display_name: String, state: PresenceState) {
        if let Some(remote) = self.remote_presence.get_mut(&participant_id) {
            remote.update_state(state);
            remote.display_name = display_name;
        } else {
            let mut remote = RemotePresence::new(participant_id.clone(), display_name);
            remote.state = state;
            self.remote_presence.insert(participant_id, remote);
        }
    }
    
    /// Remove remote presence
    pub fn remove_remote(&mut self, participant_id: &ParticipantId) {
        self.remote_presence.remove(participant_id);
    }
    
    /// Get all remote presences
    pub fn remote_presences(&self) -> impl Iterator<Item = &RemotePresence> {
        self.remote_presence.values()
    }
    
    /// Get a specific remote presence
    pub fn get_remote(&self, participant_id: &ParticipantId) -> Option<&RemotePresence> {
        self.remote_presence.get(participant_id)
    }
    
    /// Tick for interpolation
    pub fn tick(&mut self, dt: f32) {
        for remote in self.remote_presence.values_mut() {
            remote.tick(dt);
        }
    }
    
    /// Check if we should broadcast presence update
    pub fn should_broadcast(&mut self) -> bool {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_broadcast {
            if now.duration_since(last) >= self.update_interval {
                self.last_broadcast = Some(now);
                true
            } else {
                false
            }
        } else {
            self.last_broadcast = Some(now);
            true
        }
    }
    
    /// Get nearby participants (within distance)
    pub fn nearby_participants(&self, max_distance: f32) -> Vec<&RemotePresence> {
        let local_pos = &self.local_presence.position;
        
        self.remote_presence.values()
            .filter(|remote| {
                // Same room check
                if remote.state.position.room_id != local_pos.room_id {
                    return false;
                }
                
                let dx = remote.state.position.local.x - local_pos.local.x;
                let dy = remote.state.position.local.y - local_pos.local.y;
                let dz = remote.state.position.local.z - local_pos.local.z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                
                dist <= max_distance
            })
            .collect()
    }
    
    /// Remove stale presences
    pub fn cleanup_stale(&mut self, timeout: Duration) {
        self.remote_presence.retain(|_, remote| {
            !remote.state.is_stale(timeout)
        });
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::world_coords::LocalCoord;
    
    #[test]
    fn test_presence_state() {
        let mut state = PresenceState::default();
        assert_eq!(state.activity, PresenceActivity::Active);
        
        state.set_activity(PresenceActivity::Idle);
        assert_eq!(state.activity, PresenceActivity::Idle);
    }
    
    #[test]
    fn test_gaze_normalization() {
        let mut state = PresenceState::default();
        state.set_gaze(2.0, 0.0, 0.0);
        
        // Should be normalized to unit length
        let len = state.gaze_direction.iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_presence_manager() {
        let mut manager = PresenceManager::new();
        
        manager.set_activity(PresenceActivity::Speaking);
        assert_eq!(manager.local_presence().activity, PresenceActivity::Speaking);
    }
    
    #[test]
    fn test_remote_presence() {
        let mut manager = PresenceManager::new();
        
        let pid = ParticipantId::new();
        manager.update_remote(pid.clone(), "Test".to_string(), PresenceState::default());
        
        assert!(manager.get_remote(&pid).is_some());
    }
    
    #[test]
    fn test_interpolation() {
        let mut remote = RemotePresence::new(ParticipantId::new(), "Test".to_string());
        
        // Update to new position
        let new_state = PresenceState::at_position(WorldPosition {
            room_id: Some(crate::spatial::world_coords::RoomId("room".to_string())),
            local: LocalCoord { x: 10.0, y: 0.0, z: 0.0 },
            gps: None,
            floor: 0,
            version: 1,
        });
        remote.update_state(new_state);
        
        // Should start interpolating
        assert_eq!(remote.interp_progress, 0.0);
        
        // Tick forward
        remote.tick(0.1);
        assert!(remote.interp_progress > 0.0);
    }
    
    #[test]
    fn test_activity_icons() {
        assert_eq!(PresenceActivity::Active.icon(), "🟢");
        assert_eq!(PresenceActivity::DoNotDisturb.icon(), "🔴");
    }
    
    #[test]
    fn test_stale_detection() {
        // Create a fresh state with current time
        let mut state = PresenceState::at_position(WorldPosition::default());
        assert!(!state.is_stale(Duration::from_secs(60)));
        
        // Simulate old timestamp
        state.updated_at = 0;
        assert!(state.is_stale(Duration::from_secs(60)));
    }
}
