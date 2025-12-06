// Knowledge Sync - Cross-device knowledge synchronization
// Phase 45: Encrypted P2P sync with conflict resolution

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::knowledge_manager::{UserKnowledgeChunk, PrivacyLevel};

/// Sync protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request sync with peer
    SyncRequest {
        device_id: String,
        last_sync_timestamp: u64,
        chunk_ids: Vec<u64>,
    },
    
    /// Response with chunks to sync
    SyncResponse {
        chunks: Vec<UserKnowledgeChunk>,
        deletions: Vec<u64>,
        timestamp: u64,
    },
    
    /// Announce new chunk
    ChunkAnnounce {
        chunk_id: u64,
        device_id: String,
        timestamp: u64,
    },
    
    /// Announce chunk deletion
    ChunkDelete {
        chunk_id: u64,
        device_id: String,
        timestamp: u64,
    },
    
    /// Conflict notification
    ConflictDetected {
        chunk_id: u64,
        local_version: u64,
        remote_version: u64,
    },
}

/// Sync status for a chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    /// Chunk is synced across all devices
    Synced,
    /// Chunk is pending sync
    PendingSync,
    /// Chunk has conflict with remote version
    Conflict,
    /// Chunk is only local (never synced)
    LocalOnly,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Keep local version
    KeepLocal,
    /// Keep remote version
    KeepRemote,
    /// Merge both versions
    Merge,
    /// Keep newest version
    Newest,
    /// Keep both as separate chunks
    KeepBoth,
}

/// Sync metadata for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub chunk_id: u64,
    pub device_id: String,
    pub last_synced: u64,
    pub sync_status: SyncStatus,
    pub version: u64,
    pub hash: String,
}

/// Knowledge sync manager
pub struct KnowledgeSyncManager {
    device_id: String,
    sync_metadata: Arc<RwLock<HashMap<u64, SyncMetadata>>>,
    peer_states: Arc<RwLock<HashMap<String, PeerSyncState>>>,
    conflict_resolution: ConflictResolution,
    auto_sync_enabled: bool,
}

/// Peer sync state
#[derive(Debug, Clone)]
struct PeerSyncState {
    device_id: String,
    last_sync: u64,
    known_chunks: HashSet<u64>,
    sync_in_progress: bool,
}

impl KnowledgeSyncManager {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            sync_metadata: Arc::new(RwLock::new(HashMap::new())),
            peer_states: Arc::new(RwLock::new(HashMap::new())),
            conflict_resolution: ConflictResolution::Newest,
            auto_sync_enabled: true,
        }
    }

    /// Set conflict resolution strategy
    pub fn set_conflict_resolution(&mut self, strategy: ConflictResolution) {
        self.conflict_resolution = strategy;
    }

    /// Enable or disable auto-sync
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync_enabled = enabled;
    }

    /// Mark chunk as needing sync
    pub async fn mark_for_sync(&self, chunk_id: u64) -> Result<()> {
        let mut metadata = self.sync_metadata.write().await;
        
        if let Some(meta) = metadata.get_mut(&chunk_id) {
            meta.sync_status = SyncStatus::PendingSync;
            meta.version += 1;
        } else {
            // New chunk
            metadata.insert(chunk_id, SyncMetadata {
                chunk_id,
                device_id: self.device_id.clone(),
                last_synced: 0,
                sync_status: SyncStatus::PendingSync,
                version: 1,
                hash: String::new(),
            });
        }
        
        Ok(())
    }

    /// Get chunks that need syncing
    pub async fn get_pending_chunks(&self) -> Result<Vec<u64>> {
        let metadata = self.sync_metadata.read().await;
        
        let pending: Vec<u64> = metadata
            .values()
            .filter(|m| m.sync_status == SyncStatus::PendingSync)
            .map(|m| m.chunk_id)
            .collect();
        
        Ok(pending)
    }

    /// Update sync status after successful sync
    pub async fn mark_synced(&self, chunk_id: u64, timestamp: u64) -> Result<()> {
        let mut metadata = self.sync_metadata.write().await;
        
        if let Some(meta) = metadata.get_mut(&chunk_id) {
            meta.sync_status = SyncStatus::Synced;
            meta.last_synced = timestamp;
        }
        
        Ok(())
    }

    /// Register a peer device
    pub async fn register_peer(&self, device_id: String) -> Result<()> {
        let mut peers = self.peer_states.write().await;
        
        peers.insert(device_id.clone(), PeerSyncState {
            device_id,
            last_sync: 0,
            known_chunks: HashSet::new(),
            sync_in_progress: false,
        });
        
        Ok(())
    }

    /// Create sync request for a peer
    pub async fn create_sync_request(&self, peer_device_id: &str) -> Result<SyncMessage> {
        let metadata = self.sync_metadata.read().await;
        let peers = self.peer_states.read().await;
        
        let peer_state = peers.get(peer_device_id)
            .ok_or_else(|| anyhow!("Peer not found"))?;
        
        let chunk_ids: Vec<u64> = metadata.keys().copied().collect();
        
        Ok(SyncMessage::SyncRequest {
            device_id: self.device_id.clone(),
            last_sync_timestamp: peer_state.last_sync,
            chunk_ids,
        })
    }

    /// Process incoming sync request
    pub async fn process_sync_request(
        &self,
        request: SyncMessage,
        local_chunks: &[UserKnowledgeChunk],
    ) -> Result<SyncMessage> {
        if let SyncMessage::SyncRequest { device_id, last_sync_timestamp, chunk_ids } = request {
            // Filter chunks that have been modified since last sync
            let chunks_to_send: Vec<UserKnowledgeChunk> = local_chunks
                .iter()
                .filter(|chunk| {
                    chunk.modified_at > last_sync_timestamp &&
                    chunk.privacy != PrivacyLevel::Private
                })
                .cloned()
                .collect();
            
            // Update peer state
            let mut peers = self.peer_states.write().await;
            if let Some(peer) = peers.get_mut(&device_id) {
                peer.known_chunks.extend(chunk_ids.iter());
            }
            
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            Ok(SyncMessage::SyncResponse {
                chunks: chunks_to_send,
                deletions: vec![],
                timestamp,
            })
        } else {
            Err(anyhow!("Expected SyncRequest"))
        }
    }

    /// Process incoming sync response
    pub async fn process_sync_response(
        &self,
        response: SyncMessage,
        local_chunks: &HashMap<u64, UserKnowledgeChunk>,
    ) -> Result<SyncResult> {
        if let SyncMessage::SyncResponse { chunks, deletions, timestamp } = response {
            let mut to_add = Vec::new();
            let mut to_update = Vec::new();
            let mut conflicts = Vec::new();
            
            for remote_chunk in chunks {
                if let Some(local_chunk) = local_chunks.get(&remote_chunk.id) {
                    // Chunk exists locally - check for conflicts
                    if local_chunk.modified_at > remote_chunk.modified_at {
                        // Local is newer
                        if self.conflict_resolution == ConflictResolution::KeepLocal {
                            continue;
                        } else if self.conflict_resolution == ConflictResolution::KeepRemote {
                            to_update.push(remote_chunk);
                        } else if self.conflict_resolution == ConflictResolution::Newest {
                            // Local is newer, keep it
                            continue;
                        } else {
                            conflicts.push((local_chunk.clone(), remote_chunk));
                        }
                    } else if local_chunk.modified_at < remote_chunk.modified_at {
                        // Remote is newer
                        if self.conflict_resolution == ConflictResolution::Newest {
                            to_update.push(remote_chunk);
                        } else if self.conflict_resolution == ConflictResolution::KeepRemote {
                            to_update.push(remote_chunk);
                        } else if self.conflict_resolution == ConflictResolution::KeepLocal {
                            continue;
                        } else {
                            conflicts.push((local_chunk.clone(), remote_chunk));
                        }
                    }
                    // If timestamps are equal, no conflict
                } else {
                    // New chunk from remote
                    to_add.push(remote_chunk);
                }
            }
            
            Ok(SyncResult {
                chunks_added: to_add,
                chunks_updated: to_update,
                chunks_deleted: deletions,
                conflicts,
                sync_timestamp: timestamp,
            })
        } else {
            Err(anyhow!("Expected SyncResponse"))
        }
    }

    /// Resolve conflict manually
    pub async fn resolve_conflict(
        &self,
        chunk_id: u64,
        resolution: ConflictResolution,
        local: &UserKnowledgeChunk,
        remote: &UserKnowledgeChunk,
    ) -> Result<ConflictResolutionResult> {
        match resolution {
            ConflictResolution::KeepLocal => {
                Ok(ConflictResolutionResult::KeepLocal(local.clone()))
            }
            ConflictResolution::KeepRemote => {
                Ok(ConflictResolutionResult::KeepRemote(remote.clone()))
            }
            ConflictResolution::Newest => {
                if local.modified_at >= remote.modified_at {
                    Ok(ConflictResolutionResult::KeepLocal(local.clone()))
                } else {
                    Ok(ConflictResolutionResult::KeepRemote(remote.clone()))
                }
            }
            ConflictResolution::Merge => {
                // Simple merge: combine texts
                let merged = UserKnowledgeChunk {
                    id: chunk_id,
                    text: format!("{}\n\n{}", local.text, remote.text),
                    embedding: local.embedding.clone(),
                    source: format!("{} + {}", local.source, remote.source),
                    category: local.category.clone(),
                    tags: local.tags.iter().chain(remote.tags.iter())
                        .cloned()
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect(),
                    created_at: local.created_at.min(remote.created_at),
                    modified_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    privacy: local.privacy,
                    pinned: local.pinned || remote.pinned,
                };
                Ok(ConflictResolutionResult::Merged(merged))
            }
            ConflictResolution::KeepBoth => {
                Ok(ConflictResolutionResult::KeepBoth(local.clone(), remote.clone()))
            }
        }
    }

    /// Get sync statistics
    pub async fn get_sync_stats(&self) -> SyncStats {
        let metadata = self.sync_metadata.read().await;
        let peers = self.peer_states.read().await;
        
        let total_chunks = metadata.len();
        let synced = metadata.values().filter(|m| m.sync_status == SyncStatus::Synced).count();
        let pending = metadata.values().filter(|m| m.sync_status == SyncStatus::PendingSync).count();
        let conflicts = metadata.values().filter(|m| m.sync_status == SyncStatus::Conflict).count();
        
        SyncStats {
            total_chunks,
            synced_chunks: synced,
            pending_chunks: pending,
            conflict_chunks: conflicts,
            connected_peers: peers.len(),
            auto_sync_enabled: self.auto_sync_enabled,
        }
    }
}

/// Result of sync operation
#[derive(Debug)]
pub struct SyncResult {
    pub chunks_added: Vec<UserKnowledgeChunk>,
    pub chunks_updated: Vec<UserKnowledgeChunk>,
    pub chunks_deleted: Vec<u64>,
    pub conflicts: Vec<(UserKnowledgeChunk, UserKnowledgeChunk)>,
    pub sync_timestamp: u64,
}

/// Result of conflict resolution
#[derive(Debug)]
pub enum ConflictResolutionResult {
    KeepLocal(UserKnowledgeChunk),
    KeepRemote(UserKnowledgeChunk),
    Merged(UserKnowledgeChunk),
    KeepBoth(UserKnowledgeChunk, UserKnowledgeChunk),
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_chunks: usize,
    pub synced_chunks: usize,
    pub pending_chunks: usize,
    pub conflict_chunks: usize,
    pub connected_peers: usize,
    pub auto_sync_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chunk(id: u64, text: &str, modified_at: u64) -> UserKnowledgeChunk {
        UserKnowledgeChunk {
            id,
            text: text.to_string(),
            embedding: vec![0.1; 384],
            source: "test".to_string(),
            category: "test".to_string(),
            tags: vec![],
            created_at: 1000,
            modified_at,
            privacy: PrivacyLevel::Trusted,
            pinned: false,
        }
    }

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        assert_eq!(manager.device_id, "device1");
        assert!(manager.auto_sync_enabled);
    }

    #[tokio::test]
    async fn test_mark_for_sync() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        manager.mark_for_sync(1).await.unwrap();
        manager.mark_for_sync(2).await.unwrap();
        
        let pending = manager.get_pending_chunks().await.unwrap();
        assert_eq!(pending.len(), 2);
        assert!(pending.contains(&1));
        assert!(pending.contains(&2));
    }

    #[tokio::test]
    async fn test_mark_synced() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        manager.mark_for_sync(1).await.unwrap();
        manager.mark_synced(1, 2000).await.unwrap();
        
        let pending = manager.get_pending_chunks().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_register_peer() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        manager.register_peer("device2".to_string()).await.unwrap();
        
        let stats = manager.get_sync_stats().await;
        assert_eq!(stats.connected_peers, 1);
    }

    #[tokio::test]
    async fn test_create_sync_request() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        manager.register_peer("device2".to_string()).await.unwrap();
        manager.mark_for_sync(1).await.unwrap();
        
        let request = manager.create_sync_request("device2").await.unwrap();
        
        match request {
            SyncMessage::SyncRequest { device_id, chunk_ids, .. } => {
                assert_eq!(device_id, "device1");
                assert_eq!(chunk_ids.len(), 1);
            }
            _ => panic!("Expected SyncRequest"),
        }
    }

    #[tokio::test]
    async fn test_process_sync_response_no_conflict() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        let remote_chunk = create_test_chunk(1, "Remote text", 2000);
        let response = SyncMessage::SyncResponse {
            chunks: vec![remote_chunk.clone()],
            deletions: vec![],
            timestamp: 2000,
        };
        
        let local_chunks = HashMap::new();
        let result = manager.process_sync_response(response, &local_chunks).await.unwrap();
        
        assert_eq!(result.chunks_added.len(), 1);
        assert_eq!(result.chunks_updated.len(), 0);
        assert_eq!(result.conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_conflict_resolution_newest() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        let local = create_test_chunk(1, "Local text", 2000);
        let remote = create_test_chunk(1, "Remote text", 1500);
        
        let result = manager.resolve_conflict(
            1,
            ConflictResolution::Newest,
            &local,
            &remote,
        ).await.unwrap();
        
        match result {
            ConflictResolutionResult::KeepLocal(chunk) => {
                assert_eq!(chunk.text, "Local text");
            }
            _ => panic!("Expected KeepLocal"),
        }
    }

    #[tokio::test]
    async fn test_conflict_resolution_merge() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        let local = create_test_chunk(1, "Local text", 2000);
        let remote = create_test_chunk(1, "Remote text", 2000);
        
        let result = manager.resolve_conflict(
            1,
            ConflictResolution::Merge,
            &local,
            &remote,
        ).await.unwrap();
        
        match result {
            ConflictResolutionResult::Merged(chunk) => {
                assert!(chunk.text.contains("Local text"));
                assert!(chunk.text.contains("Remote text"));
            }
            _ => panic!("Expected Merged"),
        }
    }

    #[tokio::test]
    async fn test_get_sync_stats() {
        let manager = KnowledgeSyncManager::new("device1".to_string());
        
        manager.mark_for_sync(1).await.unwrap();
        manager.mark_for_sync(2).await.unwrap();
        manager.mark_synced(1, 2000).await.unwrap();
        
        let stats = manager.get_sync_stats().await;
        assert_eq!(stats.total_chunks, 2);
        assert_eq!(stats.synced_chunks, 1);
        assert_eq!(stats.pending_chunks, 1);
    }
}
