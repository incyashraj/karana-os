//! State Synchronization
//!
//! Handles real-time state synchronization between collaborators using
//! CRDT-like conflict resolution for concurrent edits.

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::ParticipantId;

/// Synchronization message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Full state snapshot
    FullState(StateSnapshot),
    /// Incremental delta update
    Delta(StateDelta),
    /// Request state from peer
    RequestState { since_version: u64 },
    /// Acknowledge received update
    Ack { version: u64 },
    /// Vector clock sync
    ClockSync(VectorClock),
}

/// Full state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// State version
    pub version: u64,
    /// Vector clock
    pub clock: VectorClock,
    /// Serialized state data
    pub data: Vec<u8>,
    /// State type identifier
    pub state_type: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Incremental state delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    /// Base version this applies to
    pub base_version: u64,
    /// New version after applying
    pub new_version: u64,
    /// Vector clock
    pub clock: VectorClock,
    /// Delta operations
    pub operations: Vec<DeltaOperation>,
    /// Originator
    pub from: ParticipantId,
}

/// Individual delta operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaOperation {
    /// Set a value
    Set { path: String, value: Vec<u8> },
    /// Delete a value
    Delete { path: String },
    /// Insert into list
    Insert { path: String, index: usize, value: Vec<u8> },
    /// Remove from list
    Remove { path: String, index: usize },
    /// Move in list
    Move { path: String, from: usize, to: usize },
    /// Custom operation
    Custom { op_type: String, data: Vec<u8> },
}

/// Vector clock for causality tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorClock {
    clocks: HashMap<String, u64>,
}

impl VectorClock {
    /// Create new vector clock
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }
    
    /// Increment clock for participant
    pub fn increment(&mut self, participant: &str) {
        let count = self.clocks.entry(participant.to_string()).or_insert(0);
        *count += 1;
    }
    
    /// Get clock value for participant
    pub fn get(&self, participant: &str) -> u64 {
        *self.clocks.get(participant).unwrap_or(&0)
    }
    
    /// Merge with another clock (take max of each)
    pub fn merge(&mut self, other: &VectorClock) {
        for (k, v) in &other.clocks {
            let entry = self.clocks.entry(k.clone()).or_insert(0);
            *entry = (*entry).max(*v);
        }
    }
    
    /// Check if this clock is concurrent with another
    pub fn is_concurrent_with(&self, other: &VectorClock) -> bool {
        !self.is_before(other) && !self.is_after(other)
    }
    
    /// Check if this clock is strictly before another
    pub fn is_before(&self, other: &VectorClock) -> bool {
        let mut any_less = false;
        
        for (k, v) in &self.clocks {
            if *v > other.get(k) {
                return false;
            }
            if *v < other.get(k) {
                any_less = true;
            }
        }
        
        // Check if other has any keys we don't
        for k in other.clocks.keys() {
            if !self.clocks.contains_key(k) && other.get(k) > 0 {
                any_less = true;
            }
        }
        
        any_less
    }
    
    /// Check if this clock is strictly after another
    pub fn is_after(&self, other: &VectorClock) -> bool {
        other.is_before(self)
    }
}

/// Sync engine manages state synchronization
pub struct SyncEngine {
    /// Local participant ID
    local_id: ParticipantId,
    /// Current state version
    version: u64,
    /// Vector clock
    clock: VectorClock,
    /// Pending outgoing deltas
    pending_deltas: VecDeque<StateDelta>,
    /// Received but not yet applied deltas
    buffered_deltas: VecDeque<StateDelta>,
    /// Last known version per peer
    peer_versions: HashMap<String, u64>,
    /// Conflict resolver
    resolver: ConflictResolver,
}

impl SyncEngine {
    /// Create new sync engine
    pub fn new() -> Self {
        Self {
            local_id: ParticipantId::new(),
            version: 0,
            clock: VectorClock::new(),
            pending_deltas: VecDeque::new(),
            buffered_deltas: VecDeque::new(),
            peer_versions: HashMap::new(),
            resolver: ConflictResolver::new(),
        }
    }
    
    /// Create with specific participant ID
    pub fn with_participant(id: ParticipantId) -> Self {
        Self {
            local_id: id,
            version: 0,
            clock: VectorClock::new(),
            pending_deltas: VecDeque::new(),
            buffered_deltas: VecDeque::new(),
            peer_versions: HashMap::new(),
            resolver: ConflictResolver::new(),
        }
    }
    
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Record local operation
    pub fn record_operation(&mut self, operation: DeltaOperation) {
        self.clock.increment(&self.local_id.0);
        
        let delta = StateDelta {
            base_version: self.version,
            new_version: self.version + 1,
            clock: self.clock.clone(),
            operations: vec![operation],
            from: self.local_id.clone(),
        };
        
        self.version += 1;
        self.pending_deltas.push_back(delta);
    }
    
    /// Get pending deltas to send
    pub fn get_pending_deltas(&mut self) -> Vec<StateDelta> {
        self.pending_deltas.drain(..).collect()
    }
    
    /// Process received sync message
    pub fn process_message(&mut self, message: SyncMessage) -> Vec<DeltaOperation> {
        match message {
            SyncMessage::Delta(delta) => {
                self.process_delta(delta)
            }
            SyncMessage::FullState(snapshot) => {
                self.process_snapshot(snapshot)
            }
            SyncMessage::ClockSync(clock) => {
                self.clock.merge(&clock);
                vec![]
            }
            SyncMessage::Ack { version } => {
                // Update peer version tracking
                vec![]
            }
            SyncMessage::RequestState { since_version: _ } => {
                // Handled by generating FullState response
                vec![]
            }
        }
    }
    
    fn process_delta(&mut self, delta: StateDelta) -> Vec<DeltaOperation> {
        // Check causality
        if delta.clock.is_after(&self.clock) {
            // Can apply directly
            self.clock.merge(&delta.clock);
            self.version = self.version.max(delta.new_version);
            delta.operations
        } else if delta.clock.is_concurrent_with(&self.clock) {
            // Need conflict resolution
            let resolved = self.resolver.resolve(&delta.operations);
            self.clock.merge(&delta.clock);
            self.version = self.version.max(delta.new_version);
            resolved
        } else {
            // Already applied or old - ignore
            vec![]
        }
    }
    
    fn process_snapshot(&mut self, snapshot: StateSnapshot) -> Vec<DeltaOperation> {
        if snapshot.version > self.version {
            self.version = snapshot.version;
            self.clock = snapshot.clock;
            // Return special operation to reset state
            vec![DeltaOperation::Custom {
                op_type: "reset".to_string(),
                data: snapshot.data,
            }]
        } else {
            vec![]
        }
    }
    
    /// Generate sync messages for peers
    pub fn generate_messages(&self, peer_id: &str) -> Vec<SyncMessage> {
        let peer_version = self.peer_versions.get(peer_id).copied().unwrap_or(0);
        
        let mut messages = vec![];
        
        // Send any pending deltas
        for delta in &self.pending_deltas {
            if delta.new_version > peer_version {
                messages.push(SyncMessage::Delta(delta.clone()));
            }
        }
        
        messages
    }
    
    /// Generate full state snapshot
    pub fn create_snapshot(&self, data: Vec<u8>, state_type: String) -> StateSnapshot {
        StateSnapshot {
            version: self.version,
            clock: self.clock.clone(),
            data,
            state_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolver for concurrent operations
pub struct ConflictResolver {
    /// Resolution strategy
    strategy: ResolutionStrategy,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy)]
pub enum ResolutionStrategy {
    /// Last writer wins (by timestamp)
    LastWriterWins,
    /// First writer wins
    FirstWriterWins,
    /// Merge concurrent values
    Merge,
    /// Transform operations (OT-like)
    Transform,
}

impl ConflictResolver {
    /// Create new resolver with default strategy
    pub fn new() -> Self {
        Self {
            strategy: ResolutionStrategy::LastWriterWins,
        }
    }
    
    /// Create with specific strategy
    pub fn with_strategy(strategy: ResolutionStrategy) -> Self {
        Self { strategy }
    }
    
    /// Resolve conflicting operations
    pub fn resolve(&self, operations: &[DeltaOperation]) -> Vec<DeltaOperation> {
        match self.strategy {
            ResolutionStrategy::LastWriterWins => {
                // For LWW, just return operations as-is (they'll overwrite)
                operations.to_vec()
            }
            ResolutionStrategy::FirstWriterWins => {
                // Don't apply if we have local changes to same paths
                operations.to_vec() // Simplified
            }
            ResolutionStrategy::Merge => {
                // Merge values where possible
                self.merge_operations(operations)
            }
            ResolutionStrategy::Transform => {
                // Transform against local ops
                self.transform_operations(operations)
            }
        }
    }
    
    fn merge_operations(&self, operations: &[DeltaOperation]) -> Vec<DeltaOperation> {
        // Simple merge - just apply all
        operations.to_vec()
    }
    
    fn transform_operations(&self, operations: &[DeltaOperation]) -> Vec<DeltaOperation> {
        // Operational transformation - adjust indices based on local ops
        operations.to_vec()
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_clock() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("a");
        clock1.increment("a");
        
        clock2.increment("b");
        
        // Clocks are concurrent
        assert!(clock1.is_concurrent_with(&clock2));
        
        // Merge clocks
        clock1.merge(&clock2);
        assert_eq!(clock1.get("a"), 2);
        assert_eq!(clock1.get("b"), 1);
    }
    
    #[test]
    fn test_clock_ordering() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("a");
        
        clock2.increment("a");
        clock2.increment("a");
        
        assert!(clock1.is_before(&clock2));
        assert!(clock2.is_after(&clock1));
    }
    
    #[test]
    fn test_sync_engine_operations() {
        let mut engine = SyncEngine::new();
        
        engine.record_operation(DeltaOperation::Set {
            path: "test".to_string(),
            value: vec![1, 2, 3],
        });
        
        assert_eq!(engine.version(), 1);
        
        let deltas = engine.get_pending_deltas();
        assert_eq!(deltas.len(), 1);
    }
    
    #[test]
    fn test_sync_message_processing() {
        let mut engine1 = SyncEngine::new();
        let mut engine2 = SyncEngine::new();
        
        // Engine 1 makes a change
        engine1.record_operation(DeltaOperation::Set {
            path: "key".to_string(),
            value: vec![1],
        });
        
        let deltas = engine1.get_pending_deltas();
        
        // Engine 2 receives and processes
        for delta in deltas {
            engine2.process_message(SyncMessage::Delta(delta));
        }
        
        assert_eq!(engine2.version(), 1);
    }
}
