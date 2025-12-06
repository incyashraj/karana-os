// Swarm Knowledge Gossip - P2P knowledge sharing via libp2p
// Phase 42: Decentralized RAG across peer network

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::universal::{RagChunk, UniversalResponse, ResponseSource};

/// Swarm knowledge message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmKnowledgeMessage {
    /// Query for knowledge
    Query {
        query_id: String,
        query_text: String,
        query_embedding: Vec<f32>,
        requester_did: String,
    },
    
    /// Response with knowledge chunks
    Response {
        query_id: String,
        chunks: Vec<RagChunk>,
        responder_did: String,
    },
    
    /// Broadcast new knowledge chunk to swarm
    Broadcast {
        chunk: RagChunk,
        source_did: String,
    },
    
    /// Request for knowledge sync
    SyncRequest {
        since_timestamp: u64,
        requester_did: String,
    },
    
    /// Knowledge sync response
    SyncResponse {
        chunks: Vec<RagChunk>,
        responder_did: String,
    },
}

/// Swarm knowledge manager
pub struct SwarmKnowledge {
    /// Shared knowledge cache from peers
    peer_knowledge: Arc<RwLock<HashMap<String, Vec<RagChunk>>>>,
    
    /// Pending queries waiting for responses
    pending_queries: Arc<RwLock<HashMap<String, PendingQuery>>>,
    
    /// Local peer ID
    local_did: String,
    
    /// Trust scores for peers (0.0-1.0)
    peer_trust: Arc<RwLock<HashMap<String, f32>>>,
    
    /// Maximum knowledge chunks to cache per peer
    max_chunks_per_peer: usize,
}

/// Pending query awaiting swarm responses
struct PendingQuery {
    query_text: String,
    responses: Vec<(String, Vec<RagChunk>)>, // (peer_did, chunks)
    started_at: std::time::Instant,
}

impl SwarmKnowledge {
    /// Create a new swarm knowledge manager
    pub fn new(local_did: String) -> Self {
        Self {
            peer_knowledge: Arc::new(RwLock::new(HashMap::new())),
            pending_queries: Arc::new(RwLock::new(HashMap::new())),
            local_did,
            peer_trust: Arc::new(RwLock::new(HashMap::new())),
            max_chunks_per_peer: 100,
        }
    }

    /// Query the swarm for knowledge
    pub async fn query_swarm(
        &self,
        query_text: &str,
        query_embedding: Vec<f32>,
    ) -> Result<Option<UniversalResponse>> {
        let query_id = uuid::Uuid::new_v4().to_string();
        
        // Register pending query
        {
            let mut pending = self.pending_queries.write().await;
            pending.insert(query_id.clone(), PendingQuery {
                query_text: query_text.to_string(),
                responses: Vec::new(),
                started_at: std::time::Instant::now(),
            });
        }
        
        // Broadcast query to swarm
        let message = SwarmKnowledgeMessage::Query {
            query_id: query_id.clone(),
            query_text: query_text.to_string(),
            query_embedding: query_embedding.clone(),
            requester_did: self.local_did.clone(),
        };
        
        self.broadcast_message(&message).await?;
        
        // Wait for responses (timeout after 500ms)
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Collect and synthesize responses
        let response = self.synthesize_swarm_response(&query_id).await?;
        
        // Clean up pending query
        {
            let mut pending = self.pending_queries.write().await;
            pending.remove(&query_id);
        }
        
        Ok(response)
    }

    /// Handle incoming swarm message
    pub async fn handle_message(&self, message: SwarmKnowledgeMessage) -> Result<()> {
        match message {
            SwarmKnowledgeMessage::Query { query_id, query_text, query_embedding, requester_did } => {
                self.handle_query(query_id, query_text, query_embedding, requester_did).await
            }
            
            SwarmKnowledgeMessage::Response { query_id, chunks, responder_did } => {
                self.handle_response(query_id, chunks, responder_did).await
            }
            
            SwarmKnowledgeMessage::Broadcast { chunk, source_did } => {
                self.handle_broadcast(chunk, source_did).await
            }
            
            SwarmKnowledgeMessage::SyncRequest { since_timestamp, requester_did } => {
                self.handle_sync_request(since_timestamp, requester_did).await
            }
            
            SwarmKnowledgeMessage::SyncResponse { chunks, responder_did } => {
                self.handle_sync_response(chunks, responder_did).await
            }
        }
    }

    /// Handle query from peer
    async fn handle_query(
        &self,
        query_id: String,
        query_text: String,
        query_embedding: Vec<f32>,
        requester_did: String,
    ) -> Result<()> {
        // Search local knowledge
        let peer_knowledge = self.peer_knowledge.read().await;
        let mut all_chunks = Vec::new();
        
        for chunks in peer_knowledge.values() {
            all_chunks.extend(chunks.clone());
        }
        
        // Find top matches using cosine similarity
        let mut scored_chunks: Vec<(f32, RagChunk)> = all_chunks.iter()
            .map(|chunk| {
                let similarity = super::embeddings::cosine_similarity(&query_embedding, &chunk.embedding);
                (similarity, chunk.clone())
            })
            .collect();
        
        scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Take top 3 matches above threshold
        let response_chunks: Vec<RagChunk> = scored_chunks.iter()
            .take(3)
            .filter(|(score, _)| *score > 0.5)
            .map(|(_, chunk)| chunk.clone())
            .collect();
        
        if !response_chunks.is_empty() {
            // Send response
            let response = SwarmKnowledgeMessage::Response {
                query_id,
                chunks: response_chunks,
                responder_did: self.local_did.clone(),
            };
            
            self.send_message_to_peer(&requester_did, &response).await?;
        }
        
        Ok(())
    }

    /// Handle response from peer
    async fn handle_response(
        &self,
        query_id: String,
        chunks: Vec<RagChunk>,
        responder_did: String,
    ) -> Result<()> {
        let mut pending = self.pending_queries.write().await;
        
        if let Some(query) = pending.get_mut(&query_id) {
            query.responses.push((responder_did, chunks));
        }
        
        Ok(())
    }

    /// Handle knowledge broadcast from peer
    async fn handle_broadcast(
        &self,
        chunk: RagChunk,
        source_did: String,
    ) -> Result<()> {
        // Check peer trust
        let trust = {
            let trust_map = self.peer_trust.read().await;
            trust_map.get(&source_did).copied().unwrap_or(0.5)
        };
        
        if trust < 0.3 {
            // Don't accept from untrusted peers
            return Ok(());
        }
        
        // Add to peer knowledge cache
        let mut knowledge = self.peer_knowledge.write().await;
        let peer_chunks = knowledge.entry(source_did).or_insert_with(Vec::new);
        
        // Avoid duplicates
        if !peer_chunks.iter().any(|c| c.text == chunk.text) {
            peer_chunks.push(chunk);
            
            // Enforce max chunks limit
            if peer_chunks.len() > self.max_chunks_per_peer {
                peer_chunks.remove(0);
            }
        }
        
        Ok(())
    }

    /// Handle sync request from peer
    async fn handle_sync_request(
        &self,
        since_timestamp: u64,
        requester_did: String,
    ) -> Result<()> {
        // Get knowledge chunks updated after timestamp
        let knowledge = self.peer_knowledge.read().await;
        let mut sync_chunks = Vec::new();
        
        for chunks in knowledge.values() {
            for chunk in chunks {
                if chunk.timestamp > since_timestamp {
                    sync_chunks.push(chunk.clone());
                }
            }
        }
        
        if !sync_chunks.is_empty() {
            let response = SwarmKnowledgeMessage::SyncResponse {
                chunks: sync_chunks,
                responder_did: self.local_did.clone(),
            };
            
            self.send_message_to_peer(&requester_did, &response).await?;
        }
        
        Ok(())
    }

    /// Handle sync response from peer
    async fn handle_sync_response(
        &self,
        chunks: Vec<RagChunk>,
        responder_did: String,
    ) -> Result<()> {
        let mut knowledge = self.peer_knowledge.write().await;
        let peer_chunks = knowledge.entry(responder_did).or_insert_with(Vec::new);
        
        for chunk in chunks {
            if !peer_chunks.iter().any(|c| c.text == chunk.text) {
                peer_chunks.push(chunk);
            }
        }
        
        Ok(())
    }

    /// Synthesize response from swarm responses
    async fn synthesize_swarm_response(
        &self,
        query_id: &str,
    ) -> Result<Option<UniversalResponse>> {
        let pending = self.pending_queries.read().await;
        
        let query = pending.get(query_id)
            .ok_or_else(|| anyhow!("Query not found"))?;
        
        if query.responses.is_empty() {
            return Ok(None);
        }
        
        // Combine chunks from all responses
        let mut all_chunks = Vec::new();
        for (peer_did, chunks) in &query.responses {
            let trust = {
                let trust_map = self.peer_trust.read().await;
                trust_map.get(peer_did).copied().unwrap_or(0.5)
            };
            
            // Weight chunks by peer trust
            for chunk in chunks {
                let mut weighted_chunk = chunk.clone();
                weighted_chunk.score *= trust;
                all_chunks.push(weighted_chunk);
            }
        }
        
        if all_chunks.is_empty() {
            return Ok(None);
        }
        
        // Sort by weighted score
        all_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Take top chunk
        let best_chunk = &all_chunks[0];
        
        Ok(Some(UniversalResponse {
            answer: best_chunk.text.clone(),
            source: ResponseSource::SwarmPeers,
            confidence: best_chunk.score,
            proof: None,
            follow_up: vec![
                "Ask swarm for more".to_string(),
                "Related topics?".to_string(),
            ],
        }))
    }

    /// Broadcast message to all peers
    async fn broadcast_message(&self, _message: &SwarmKnowledgeMessage) -> Result<()> {
        // TODO: Integrate with libp2p gossipsub
        // Publish to /karana/knowledge topic
        Ok(())
    }

    /// Send message to specific peer
    async fn send_message_to_peer(
        &self,
        _peer_did: &str,
        _message: &SwarmKnowledgeMessage,
    ) -> Result<()> {
        // TODO: Integrate with libp2p direct messaging
        Ok(())
    }

    /// Update peer trust score
    pub async fn update_peer_trust(&self, peer_did: &str, trust_delta: f32) {
        let mut trust_map = self.peer_trust.write().await;
        let current = trust_map.get(peer_did).copied().unwrap_or(0.5);
        let new_trust = (current + trust_delta).clamp(0.0, 1.0);
        trust_map.insert(peer_did.to_string(), new_trust);
    }

    /// Get peer knowledge statistics
    pub async fn get_stats(&self) -> SwarmKnowledgeStats {
        let knowledge = self.peer_knowledge.read().await;
        let trust_map = self.peer_trust.read().await;
        
        let total_peers = knowledge.len();
        let total_chunks: usize = knowledge.values().map(|v| v.len()).sum();
        let trusted_peers = trust_map.values().filter(|&&t| t > 0.7).count();
        
        SwarmKnowledgeStats {
            total_peers,
            total_chunks,
            trusted_peers,
            avg_trust: if trust_map.is_empty() { 0.0 } else {
                trust_map.values().sum::<f32>() / trust_map.len() as f32
            },
        }
    }
}

/// Swarm knowledge statistics
#[derive(Debug, Clone)]
pub struct SwarmKnowledgeStats {
    pub total_peers: usize,
    pub total_chunks: usize,
    pub trusted_peers: usize,
    pub avg_trust: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_swarm_knowledge_creation() {
        let swarm = SwarmKnowledge::new("did:karana:test".to_string());
        assert_eq!(swarm.local_did, "did:karana:test");
    }

    #[tokio::test]
    async fn test_handle_broadcast() {
        let swarm = SwarmKnowledge::new("did:karana:local".to_string());
        
        let chunk = RagChunk {
            text: "Test knowledge".to_string(),
            embedding: vec![0.1; 384],
            score: 0.9,
            source_doc: "test".to_string(),
            timestamp: 100,
        };
        
        swarm.handle_broadcast(chunk.clone(), "did:karana:peer1".to_string()).await.unwrap();
        
        let stats = swarm.get_stats().await;
        assert_eq!(stats.total_peers, 1);
        assert_eq!(stats.total_chunks, 1);
    }

    #[tokio::test]
    async fn test_peer_trust() {
        let swarm = SwarmKnowledge::new("did:karana:local".to_string());
        
        swarm.update_peer_trust("peer1", 0.3).await;
        
        let trust_map = swarm.peer_trust.read().await;
        let trust = trust_map.get("peer1").unwrap();
        assert!((trust - 0.8).abs() < 0.01); // 0.5 + 0.3
    }

    #[tokio::test]
    async fn test_get_stats() {
        let swarm = SwarmKnowledge::new("did:karana:local".to_string());
        
        let chunk = RagChunk {
            text: "Knowledge 1".to_string(),
            embedding: vec![0.1; 384],
            score: 0.9,
            source_doc: "test".to_string(),
            timestamp: 100,
        };
        
        swarm.handle_broadcast(chunk, "peer1".to_string()).await.unwrap();
        swarm.update_peer_trust("peer1", 0.3).await;
        
        let stats = swarm.get_stats().await;
        assert_eq!(stats.total_peers, 1);
        assert_eq!(stats.total_chunks, 1);
        assert_eq!(stats.trusted_peers, 1); // 0.5 + 0.3 = 0.8 > 0.7
    }
}
