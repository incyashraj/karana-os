//! Spatial Persistence Module
//!
//! Handles persistence of spatial anchors with optional ZK attestation
//! and chain/swarm sync for distributed AR content.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::anchor::{AnchorId, SpatialAnchor};
use super::world_coords::RoomId;

// ============================================================================
// PERSISTENCE MODES
// ============================================================================

/// How anchor data is persisted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistenceMode {
    /// Local device only (fastest, no sync)
    Local,
    /// Synchronized with local network peers
    LocalNetwork,
    /// Backed up to cloud service
    Cloud,
    /// Attested on blockchain (tamper-proof)
    Chain,
    /// Distributed across swarm (censorship-resistant)
    Swarm,
}

impl Default for PersistenceMode {
    fn default() -> Self {
        Self::Local
    }
}

// ============================================================================
// ZK ATTESTATION
// ============================================================================

/// Zero-knowledge proof that anchor exists at claimed position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAttestation {
    /// Anchor being attested
    pub anchor_id: AnchorId,
    /// Position commitment (hides actual position)
    pub position_commitment: [u8; 32],
    /// Proof data
    pub proof: ZkProof,
    /// Attester DID
    pub attester_did: String,
    /// When attested
    pub attested_at: u64,
    /// Expiry timestamp (0 = never)
    pub expires_at: u64,
}

/// Simplified ZK proof structure
/// In production, this would be a proper ZK-SNARK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    /// Proof type (e.g., "groth16", "plonk")
    pub proof_type: String,
    /// Proof bytes
    pub data: Vec<u8>,
    /// Public inputs
    pub public_inputs: Vec<[u8; 32]>,
}

impl ZkProof {
    /// Create a mock proof (for testing)
    pub fn mock() -> Self {
        Self {
            proof_type: "mock".to_string(),
            data: vec![0u8; 64],
            public_inputs: vec![],
        }
    }
    
    /// Verify the proof (mock implementation)
    pub fn verify(&self) -> bool {
        // Real implementation would verify the ZK proof
        self.proof_type == "mock" || !self.data.is_empty()
    }
}

/// Create position commitment (Pedersen commitment style)
pub fn create_position_commitment(
    x: f32,
    y: f32,
    z: f32,
    room_id: &RoomId,
    blinding: &[u8; 32],
) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(x.to_le_bytes());
    hasher.update(y.to_le_bytes());
    hasher.update(z.to_le_bytes());
    hasher.update(room_id.0.as_bytes());
    hasher.update(blinding);
    
    let result = hasher.finalize();
    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&result);
    commitment
}

// ============================================================================
// CHAIN RECORD
// ============================================================================

/// Record of an anchor stored on chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAnchorRecord {
    /// Anchor ID
    pub anchor_id: AnchorId,
    /// Content hash
    pub content_hash: [u8; 32],
    /// Position commitment (ZK)
    pub position_commitment: [u8; 32],
    /// Owner DID
    pub owner_did: String,
    /// Transaction hash where recorded
    pub tx_hash: [u8; 32],
    /// Block number
    pub block_number: u64,
    /// Timestamp
    pub created_at: u64,
}

/// Proof of anchor existence and position
pub type AnchorProof = SpatialAttestation;

/// Integrity proof for anchor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorIntegrityProof {
    /// Anchor ID
    pub anchor_id: AnchorId,
    /// Content hash
    pub content_hash: [u8; 32],
    /// Merkle root of anchor data
    pub merkle_root: [u8; 32],
    /// Signature over merkle root
    pub signature: Vec<u8>,
    /// Signer DID
    pub signer_did: String,
    /// Timestamp
    pub created_at: u64,
}

impl AnchorIntegrityProof {
    /// Create a new integrity proof
    pub fn new(anchor: &SpatialAnchor, signer_did: String) -> Self {
        use sha2::{Sha256, Digest};
        
        // Compute content hash
        let content_hash = anchor.content.hash();
        
        // Compute merkle root (simplified - just hash of id + content_hash)
        let mut hasher = Sha256::new();
        hasher.update(anchor.id.to_le_bytes());
        hasher.update(&content_hash);
        let merkle_root: [u8; 32] = hasher.finalize().into();
        
        Self {
            anchor_id: anchor.id,
            content_hash,
            merkle_root,
            signature: vec![], // Would be signed in real implementation
            signer_did,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Verify the integrity proof
    pub fn verify(&self) -> bool {
        // In real implementation, would verify signature
        !self.merkle_root.iter().all(|&b| b == 0)
    }
}

// ============================================================================
// PERSISTENCE STORE
// ============================================================================

/// Local persistence store for anchors
pub struct AnchorStore {
    /// Base path for storage
    base_path: PathBuf,
    /// Cached anchors
    cache: HashMap<AnchorId, StoredAnchor>,
    /// Attestations
    attestations: HashMap<AnchorId, SpatialAttestation>,
    /// Chain records
    chain_records: HashMap<AnchorId, ChainAnchorRecord>,
    /// Dirty anchors needing sync
    dirty: Vec<AnchorId>,
}

/// Anchor with persistence metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAnchor {
    /// The anchor itself
    pub anchor: SpatialAnchor,
    /// Persistence mode
    pub mode: PersistenceMode,
    /// Last synced timestamp
    pub last_synced: u64,
    /// Sync status
    pub sync_status: SyncStatus,
}

/// Sync status for an anchor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Up to date locally only
    LocalOnly,
    /// Pending upload
    PendingUpload,
    /// Synced with remote
    Synced,
    /// Conflict needs resolution
    Conflict,
    /// Sync failed
    Failed,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::LocalOnly
    }
}

impl AnchorStore {
    /// Create a new anchor store
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            cache: HashMap::new(),
            attestations: HashMap::new(),
            chain_records: HashMap::new(),
            dirty: Vec::new(),
        }
    }
    
    /// Save an anchor (async version for API compatibility)
    pub async fn save_async(&mut self, anchor: &SpatialAnchor) -> Result<()> {
        self.save(anchor.clone(), PersistenceMode::Local)
    }
    
    /// Delete anchor (async version for API compatibility)
    pub async fn delete_async(&mut self, id: AnchorId) -> Result<()> {
        self.delete(id)
    }
    
    /// Get all anchors (async version for API compatibility)
    pub async fn get_all(&self) -> Result<Vec<SpatialAnchor>> {
        Ok(self.cache.values().map(|s| s.anchor.clone()).collect())
    }
    
    /// Save an anchor
    pub fn save(&mut self, anchor: SpatialAnchor, mode: PersistenceMode) -> Result<()> {
        let stored = StoredAnchor {
            anchor: anchor.clone(),
            mode,
            last_synced: 0,
            sync_status: SyncStatus::LocalOnly,
        };
        
        self.cache.insert(anchor.id, stored);
        
        if mode != PersistenceMode::Local {
            self.dirty.push(anchor.id);
        }
        
        // Persist to disk
        self.write_to_disk(anchor.id)?;
        
        Ok(())
    }
    
    /// Load an anchor
    pub fn load(&mut self, id: AnchorId) -> Result<Option<StoredAnchor>> {
        // Check cache first
        if let Some(stored) = self.cache.get(&id) {
            return Ok(Some(stored.clone()));
        }
        
        // Try loading from disk
        self.read_from_disk(id)
    }
    
    /// Delete an anchor
    pub fn delete(&mut self, id: AnchorId) -> Result<()> {
        self.cache.remove(&id);
        self.attestations.remove(&id);
        
        let path = self.anchor_path(id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        
        Ok(())
    }
    
    /// Get all anchors in a room
    pub fn get_by_room(&self, room_id: &RoomId) -> Vec<&StoredAnchor> {
        self.cache
            .values()
            .filter(|sa| {
                sa.anchor.position.room_id.as_ref() == Some(room_id)
            })
            .collect()
    }
    
    /// Get dirty anchors needing sync
    pub fn get_dirty(&self) -> Vec<AnchorId> {
        self.dirty.clone()
    }
    
    /// Mark anchor as synced
    pub fn mark_synced(&mut self, id: AnchorId) {
        if let Some(stored) = self.cache.get_mut(&id) {
            stored.sync_status = SyncStatus::Synced;
            stored.last_synced = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        self.dirty.retain(|&aid| aid != id);
    }
    
    /// Add attestation for anchor
    pub fn add_attestation(&mut self, attestation: SpatialAttestation) {
        self.attestations.insert(attestation.anchor_id, attestation);
    }
    
    /// Get attestation for anchor
    pub fn get_attestation(&self, id: AnchorId) -> Option<&SpatialAttestation> {
        self.attestations.get(&id)
    }
    
    /// Add chain record for anchor
    pub fn add_chain_record(&mut self, record: ChainAnchorRecord) {
        self.chain_records.insert(record.anchor_id, record);
    }
    
    /// Get chain record for anchor
    pub fn get_chain_record(&self, id: AnchorId) -> Option<&ChainAnchorRecord> {
        self.chain_records.get(&id)
    }
    
    /// Get anchor file path
    fn anchor_path(&self, id: AnchorId) -> PathBuf {
        self.base_path.join(format!("anchor_{}.json", id))
    }
    
    /// Write anchor to disk
    fn write_to_disk(&self, id: AnchorId) -> Result<()> {
        if let Some(stored) = self.cache.get(&id) {
            let path = self.anchor_path(id);
            
            // Ensure directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            let json = serde_json::to_string_pretty(stored)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }
    
    /// Read anchor from disk
    fn read_from_disk(&mut self, id: AnchorId) -> Result<Option<StoredAnchor>> {
        let path = self.anchor_path(id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let json = std::fs::read_to_string(path)?;
        let stored: StoredAnchor = serde_json::from_str(&json)?;
        
        self.cache.insert(id, stored.clone());
        Ok(Some(stored))
    }
    
    /// Load all anchors from disk
    pub fn load_all(&mut self) -> Result<usize> {
        if !self.base_path.exists() {
            return Ok(0);
        }
        
        let mut count = 0;
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(filename) = path.file_stem() {
                    if let Some(name) = filename.to_str() {
                        if name.starts_with("anchor_") {
                            if let Ok(id) = name.trim_start_matches("anchor_").parse::<u64>() {
                                if self.read_from_disk(id)?.is_some() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    /// Get cache count
    pub fn cache_count(&self) -> usize {
        self.cache.len()
    }
}

// ============================================================================
// SYNC SERVICE
// ============================================================================

/// Service for syncing anchors with remote storage
pub struct SyncService {
    /// Backend type
    backend: SyncBackend,
    /// Pending sync queue
    queue: Vec<AnchorId>,
    /// Last sync timestamp
    last_sync: u64,
}

/// Sync backend types
#[derive(Debug, Clone)]
pub enum SyncBackend {
    /// No remote sync
    None,
    /// Local network discovery (mDNS)
    LocalNetwork { port: u16 },
    /// Cloud service
    Cloud { endpoint: String, api_key: String },
    /// Blockchain
    Chain { rpc_url: String, contract: String },
    /// IPFS/libp2p swarm
    Swarm { bootstrap_peers: Vec<String> },
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new(SyncBackend::None)
    }
}

impl SyncService {
    /// Create new sync service
    pub fn new(backend: SyncBackend) -> Self {
        Self {
            backend,
            queue: Vec::new(),
            last_sync: 0,
        }
    }
    
    /// Queue anchor for sync
    pub fn queue_sync(&mut self, id: AnchorId) {
        if !self.queue.contains(&id) {
            self.queue.push(id);
        }
    }
    
    /// Sync pending anchors
    pub async fn sync(&mut self, store: &mut AnchorStore) -> Result<usize> {
        let mut synced = 0;
        
        let to_sync: Vec<AnchorId> = self.queue.drain(..).collect();
        
        for id in to_sync {
            if let Ok(Some(stored)) = store.load(id) {
                match self.sync_anchor(&stored).await {
                    Ok(()) => {
                        store.mark_synced(id);
                        synced += 1;
                    }
                    Err(_) => {
                        // Re-queue for retry
                        self.queue.push(id);
                    }
                }
            }
        }
        
        self.last_sync = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(synced)
    }
    
    /// Sync a single anchor
    async fn sync_anchor(&self, _stored: &StoredAnchor) -> Result<()> {
        match &self.backend {
            SyncBackend::None => Ok(()),
            SyncBackend::LocalNetwork { .. } => {
                // Would broadcast to local network
                Ok(())
            }
            SyncBackend::Cloud { .. } => {
                // Would POST to cloud API
                Ok(())
            }
            SyncBackend::Chain { .. } => {
                // Would submit transaction
                Ok(())
            }
            SyncBackend::Swarm { .. } => {
                // Would publish to IPFS
                Ok(())
            }
        }
    }
    
    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::anchor::{AnchorContent, AnchorState, Quaternion};
    use crate::spatial::world_coords::WorldPosition;
    
    fn make_test_anchor(id: AnchorId) -> SpatialAnchor {
        SpatialAnchor {
            id,
            position: WorldPosition::from_local(1.0, 2.0, 3.0),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: format!("Anchor {}", id) },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        }
    }
    
    #[test]
    fn test_position_commitment() {
        let blinding = [1u8; 32];
        let room = RoomId::new("test_room");
        
        let commitment1 = create_position_commitment(1.0, 2.0, 3.0, &room, &blinding);
        let commitment2 = create_position_commitment(1.0, 2.0, 3.0, &room, &blinding);
        let commitment3 = create_position_commitment(1.0, 2.0, 4.0, &room, &blinding);
        
        assert_eq!(commitment1, commitment2);
        assert_ne!(commitment1, commitment3);
    }
    
    #[test]
    fn test_anchor_store_save_load() {
        let temp_path = std::env::temp_dir().join(format!("karana_test_{}", std::process::id()));
        let mut store = AnchorStore::new(temp_path.clone());
        
        let anchor = make_test_anchor(1);
        store.save(anchor.clone(), PersistenceMode::Local).unwrap();
        
        // Clear cache to force disk read
        store.cache.clear();
        
        let loaded = store.load(1).unwrap().unwrap();
        assert_eq!(loaded.anchor.id, anchor.id);
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_path);
    }
    
    #[test]
    fn test_anchor_store_delete() {
        let temp_path = std::env::temp_dir().join(format!("karana_test_del_{}", std::process::id()));
        let mut store = AnchorStore::new(temp_path.clone());
        
        let anchor = make_test_anchor(1);
        store.save(anchor, PersistenceMode::Local).unwrap();
        assert_eq!(store.cache_count(), 1);
        
        store.delete(1).unwrap();
        assert_eq!(store.cache_count(), 0);
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_path);
    }
    
    #[test]
    fn test_dirty_tracking() {
        let temp_path = std::env::temp_dir().join(format!("karana_test_dirty_{}", std::process::id()));
        let mut store = AnchorStore::new(temp_path.clone());
        
        // Local mode doesn't mark dirty
        store.save(make_test_anchor(1), PersistenceMode::Local).unwrap();
        assert!(store.get_dirty().is_empty());
        
        // Chain mode marks dirty
        store.save(make_test_anchor(2), PersistenceMode::Chain).unwrap();
        assert_eq!(store.get_dirty().len(), 1);
        
        // Mark synced
        store.mark_synced(2);
        assert!(store.get_dirty().is_empty());
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_path);
    }
    
    #[test]
    fn test_zk_proof_mock() {
        let proof = ZkProof::mock();
        assert!(proof.verify());
    }
    
    #[test]
    fn test_sync_service() {
        let mut service = SyncService::new(SyncBackend::None);
        
        service.queue_sync(1);
        service.queue_sync(2);
        service.queue_sync(1); // Duplicate
        
        assert_eq!(service.queue_length(), 2);
    }
}
