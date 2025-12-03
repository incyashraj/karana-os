//! Multi-user Collaboration System
//!
//! Enables shared spatial experiences between multiple Kāraṇa OS users.
//! Features:
//! - Shared anchors: Multiple users can see the same virtual content
//! - Presence: See other users' positions and gaze in AR
//! - Collaborative tabs: Share AR tabs with others
//! - Synchronized state: Real-time updates across all participants
//!
//! Privacy: All sharing is explicit and consent-based via the Oracle permission system.

mod shared_anchors;
mod presence;
mod collaborative_tabs;
mod sync;

pub use shared_anchors::*;
pub use presence::*;
pub use collaborative_tabs::*;
pub use sync::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::spatial::WorldPosition;

/// Unique identifier for a collaboration session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(u64);

impl SessionId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
    
    pub fn from_code(code: &str) -> Option<Self> {
        // Parse session code like "ABCD-1234"
        let clean: String = code.chars().filter(|c| c.is_alphanumeric()).collect();
        u64::from_str_radix(&clean, 36).ok().map(Self)
    }
    
    pub fn to_code(&self) -> String {
        // Generate shareable code like "ABCD-1234"
        let base36 = format!("{:0>8}", radix_fmt(self.0, 36));
        if base36.len() >= 8 {
            format!("{}-{}", &base36[0..4].to_uppercase(), &base36[4..8].to_uppercase())
        } else {
            base36.to_uppercase()
        }
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_code())
    }
}

/// Format number in given radix
fn radix_fmt(mut n: u64, radix: u64) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % radix) as usize] as char);
        n /= radix;
    }
    result.into_iter().rev().collect()
}

/// Unique identifier for a participant
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticipantId(pub String);

impl ParticipantId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_did(did: &str) -> Self {
        Self(did.to_string())
    }
}

impl Default for ParticipantId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Collaboration session role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRole {
    /// Created the session, full control
    Host,
    /// Can edit shared content
    Collaborator,
    /// Can only view shared content
    Viewer,
}

impl SessionRole {
    pub fn can_edit(&self) -> bool {
        matches!(self, SessionRole::Host | SessionRole::Collaborator)
    }
    
    pub fn can_share(&self) -> bool {
        matches!(self, SessionRole::Host)
    }
    
    pub fn can_kick(&self) -> bool {
        matches!(self, SessionRole::Host)
    }
}

/// Session privacy mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPrivacy {
    /// Only invited users can join
    Private,
    /// Anyone with the code can join
    Public,
    /// Requires approval to join
    RequestToJoin,
}

/// Collaboration session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session name
    pub name: String,
    /// Privacy mode
    pub privacy: SessionPrivacy,
    /// Maximum participants (0 = unlimited)
    pub max_participants: usize,
    /// Allow presence sharing
    pub share_presence: bool,
    /// Allow anchor sharing
    pub share_anchors: bool,
    /// Allow tab sharing
    pub share_tabs: bool,
    /// Session expiry duration (None = no expiry)
    pub expires_after: Option<Duration>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "Collaboration".to_string(),
            privacy: SessionPrivacy::Private,
            max_participants: 10,
            share_presence: true,
            share_anchors: true,
            share_tabs: true,
            expires_after: Some(Duration::from_secs(3600 * 24)), // 24 hours
        }
    }
}

/// A participant in a collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Unique ID
    pub id: ParticipantId,
    /// Display name
    pub display_name: String,
    /// Avatar URL or identifier
    pub avatar: Option<String>,
    /// Role in session
    pub role: SessionRole,
    /// Current presence state
    pub presence: PresenceState,
    /// When they joined
    pub joined_at: u64,
    /// Last activity timestamp
    pub last_active: u64,
}

impl Participant {
    pub fn new(id: ParticipantId, display_name: String, role: SessionRole) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id,
            display_name,
            avatar: None,
            role,
            presence: PresenceState::default(),
            joined_at: now,
            last_active: now,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    pub fn is_active(&self, timeout: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        now.saturating_sub(self.last_active) < timeout.as_millis() as u64
    }
}

/// Collaboration session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSession {
    /// Session ID
    pub id: SessionId,
    /// Configuration
    pub config: SessionConfig,
    /// Participants
    pub participants: HashMap<ParticipantId, Participant>,
    /// Shared anchors
    pub shared_anchors: Vec<SharedAnchorInfo>,
    /// Shared tabs
    pub shared_tabs: Vec<SharedTabInfo>,
    /// Session created timestamp
    pub created_at: u64,
    /// Host participant ID
    pub host_id: ParticipantId,
}

impl CollabSession {
    pub fn new(host_id: ParticipantId, host_name: String, config: SessionConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let mut participants = HashMap::new();
        let host = Participant::new(host_id.clone(), host_name, SessionRole::Host);
        participants.insert(host_id.clone(), host);
        
        Self {
            id: SessionId::new(),
            config,
            participants,
            shared_anchors: vec![],
            shared_tabs: vec![],
            created_at: now,
            host_id,
        }
    }
    
    pub fn join_code(&self) -> String {
        self.id.to_code()
    }
    
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }
    
    pub fn is_full(&self) -> bool {
        self.config.max_participants > 0 && 
        self.participants.len() >= self.config.max_participants
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_after) = self.config.expires_after {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            // Use >= to handle zero-duration expiry correctly
            now.saturating_sub(self.created_at) >= expires_after.as_millis() as u64
        } else {
            false
        }
    }
    
    pub fn add_participant(&mut self, id: ParticipantId, name: String, role: SessionRole) -> Result<()> {
        if self.is_full() {
            return Err(anyhow!("Session is full"));
        }
        
        if self.participants.contains_key(&id) {
            return Err(anyhow!("Already in session"));
        }
        
        let participant = Participant::new(id.clone(), name, role);
        self.participants.insert(id, participant);
        Ok(())
    }
    
    pub fn remove_participant(&mut self, id: &ParticipantId) -> Option<Participant> {
        self.participants.remove(id)
    }
    
    pub fn get_participant(&self, id: &ParticipantId) -> Option<&Participant> {
        self.participants.get(id)
    }
    
    pub fn get_participant_mut(&mut self, id: &ParticipantId) -> Option<&mut Participant> {
        self.participants.get_mut(id)
    }
    
    pub fn update_presence(&mut self, id: &ParticipantId, presence: PresenceState) {
        if let Some(participant) = self.participants.get_mut(id) {
            participant.presence = presence;
            participant.update_activity();
        }
    }
}

/// Basic shared anchor info (full struct in shared_anchors.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedAnchorInfo {
    pub anchor_id: String,
    pub owner_id: ParticipantId,
    pub position: WorldPosition,
    pub name: Option<String>,
}

/// Basic shared tab info (full struct in collaborative_tabs.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedTabInfo {
    pub tab_id: String,
    pub owner_id: ParticipantId,
    pub title: String,
    pub tab_type: String,
}

/// Collaboration manager - handles all multi-user features
pub struct CollabManager {
    /// Active sessions we're participating in
    sessions: HashMap<SessionId, CollabSession>,
    /// Our participant ID
    local_participant: ParticipantId,
    /// Our display name
    local_name: String,
    /// Presence manager
    presence_manager: PresenceManager,
    /// Shared anchor manager
    shared_anchor_manager: SharedAnchorManager,
    /// Collaborative tab manager
    collab_tab_manager: CollaborativeTabManager,
    /// Sync engine
    sync_engine: SyncEngine,
}

impl CollabManager {
    /// Create a new collaboration manager
    pub fn new(participant_id: ParticipantId, display_name: String) -> Self {
        Self {
            sessions: HashMap::new(),
            local_participant: participant_id,
            local_name: display_name,
            presence_manager: PresenceManager::new(),
            shared_anchor_manager: SharedAnchorManager::new(),
            collab_tab_manager: CollaborativeTabManager::new(),
            sync_engine: SyncEngine::new(),
        }
    }
    
    /// Create a new collaboration session
    pub fn create_session(&mut self, config: SessionConfig) -> SessionId {
        let session = CollabSession::new(
            self.local_participant.clone(),
            self.local_name.clone(),
            config,
        );
        let id = session.id;
        self.sessions.insert(id, session);
        
        log::info!("[COLLAB] Created session {}", id);
        id
    }
    
    /// Join an existing session by code
    pub fn join_session(&mut self, code: &str) -> Result<SessionId> {
        let session_id = SessionId::from_code(code)
            .ok_or_else(|| anyhow!("Invalid session code"))?;
        
        // In production, this would connect to the session host via P2P
        // For now, we'll create a placeholder
        log::info!("[COLLAB] Joining session {}", code);
        
        Ok(session_id)
    }
    
    /// Leave a session
    pub fn leave_session(&mut self, id: SessionId) -> Result<()> {
        self.sessions.remove(&id)
            .ok_or_else(|| anyhow!("Not in session"))?;
        
        log::info!("[COLLAB] Left session {}", id);
        Ok(())
    }
    
    /// Get a session by ID
    pub fn get_session(&self, id: SessionId) -> Option<&CollabSession> {
        self.sessions.get(&id)
    }
    
    /// Get mutable session
    pub fn get_session_mut(&mut self, id: SessionId) -> Option<&mut CollabSession> {
        self.sessions.get_mut(&id)
    }
    
    /// List all active sessions
    pub fn list_sessions(&self) -> Vec<&CollabSession> {
        self.sessions.values().collect()
    }
    
    /// Update our presence in a session
    pub fn update_presence(&mut self, session_id: SessionId, presence: PresenceState) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.update_presence(&self.local_participant, presence.clone());
            
            // Also update in presence manager for local tracking
            self.presence_manager.update_local_presence(presence);
        }
    }
    
    /// Share an anchor with a session
    pub fn share_anchor(
        &mut self,
        session_id: SessionId,
        anchor_id: &str,
        position: WorldPosition,
        name: Option<String>,
    ) -> Result<()> {
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Not in session"))?;
        
        if !session.config.share_anchors {
            return Err(anyhow!("Anchor sharing not enabled"));
        }
        
        let info = SharedAnchorInfo {
            anchor_id: anchor_id.to_string(),
            owner_id: self.local_participant.clone(),
            position,
            name,
        };
        
        session.shared_anchors.push(info);
        log::info!("[COLLAB] Shared anchor {} in session {}", anchor_id, session_id);
        
        Ok(())
    }
    
    /// Share a tab with a session
    pub fn share_tab(
        &mut self,
        session_id: SessionId,
        tab_id: &str,
        title: String,
        tab_type: String,
    ) -> Result<()> {
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Not in session"))?;
        
        if !session.config.share_tabs {
            return Err(anyhow!("Tab sharing not enabled"));
        }
        
        let info = SharedTabInfo {
            tab_id: tab_id.to_string(),
            owner_id: self.local_participant.clone(),
            title,
            tab_type,
        };
        
        session.shared_tabs.push(info);
        log::info!("[COLLAB] Shared tab {} in session {}", tab_id, session_id);
        
        Ok(())
    }
    
    /// Get presence manager
    pub fn presence(&self) -> &PresenceManager {
        &self.presence_manager
    }
    
    /// Get presence manager mutably
    pub fn presence_mut(&mut self) -> &mut PresenceManager {
        &mut self.presence_manager
    }
    
    /// Get shared anchor manager
    pub fn shared_anchors(&self) -> &SharedAnchorManager {
        &self.shared_anchor_manager
    }
    
    /// Get collaborative tab manager
    pub fn collab_tabs(&self) -> &CollaborativeTabManager {
        &self.collab_tab_manager
    }
    
    /// Get sync engine
    pub fn sync(&self) -> &SyncEngine {
        &self.sync_engine
    }
    
    /// Process incoming sync message
    pub fn process_sync_message(&mut self, _session_id: SessionId, message: SyncMessage) {
        // Process message through sync engine
        let _ops = self.sync_engine.process_message(message);
        // Operations would be applied to session state
    }
    
    /// Generate sync messages for outgoing updates
    pub fn generate_sync_messages(&self, session_id: SessionId) -> Vec<SyncMessage> {
        if let Some(session) = self.sessions.get(&session_id) {
            // Generate messages for each participant
            let mut messages = vec![];
            for (_, participant) in &session.participants {
                messages.extend(self.sync_engine.generate_messages(&participant.id.0));
            }
            messages
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_id_code() {
        let id = SessionId(12345678);
        let code = id.to_code();
        
        // Should be able to parse back
        let parsed = SessionId::from_code(&code);
        assert!(parsed.is_some());
    }
    
    #[test]
    fn test_create_session() {
        let mut manager = CollabManager::new(
            ParticipantId::new(),
            "Test User".to_string(),
        );
        
        let session_id = manager.create_session(SessionConfig::default());
        
        let session = manager.get_session(session_id);
        assert!(session.is_some());
        assert_eq!(session.unwrap().participant_count(), 1);
    }
    
    #[test]
    fn test_session_roles() {
        assert!(SessionRole::Host.can_edit());
        assert!(SessionRole::Host.can_share());
        assert!(SessionRole::Host.can_kick());
        
        assert!(SessionRole::Collaborator.can_edit());
        assert!(!SessionRole::Collaborator.can_share());
        
        assert!(!SessionRole::Viewer.can_edit());
    }
    
    #[test]
    fn test_participant_activity() {
        let mut participant = Participant::new(
            ParticipantId::new(),
            "Test".to_string(),
            SessionRole::Viewer,
        );
        
        assert!(participant.is_active(Duration::from_secs(60)));
        
        // Simulate old timestamp
        participant.last_active = 0;
        assert!(!participant.is_active(Duration::from_secs(60)));
    }
    
    #[test]
    fn test_session_expiry() {
        let mut session = CollabSession::new(
            ParticipantId::new(),
            "Host".to_string(),
            SessionConfig {
                expires_after: Some(Duration::from_secs(0)),
                ..Default::default()
            },
        );
        
        // Should be expired immediately
        assert!(session.is_expired());
    }
    
    #[test]
    fn test_share_anchor() {
        let mut manager = CollabManager::new(
            ParticipantId::new(),
            "Test User".to_string(),
        );
        
        let session_id = manager.create_session(SessionConfig::default());
        
        let result = manager.share_anchor(
            session_id,
            "anchor-123",
            WorldPosition::default(),
            Some("My Anchor".to_string()),
        );
        
        assert!(result.is_ok());
        
        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.shared_anchors.len(), 1);
    }
}
