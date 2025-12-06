// User Knowledge Manager - CRUD operations for personal RAG knowledge base
// Phase 43: Allow users to add, edit, delete, and organize their own knowledge

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::embeddings::EmbeddingGenerator;
use super::universal::RagChunk;

/// User knowledge manager
pub struct KnowledgeManager {
    /// User's personal knowledge chunks
    chunks: Arc<RwLock<Vec<UserKnowledgeChunk>>>,
    
    /// Embedding generator for new chunks
    embedding_gen: Arc<EmbeddingGenerator>,
    
    /// User's DID
    user_did: String,
    
    /// Storage path for persistence
    storage_path: PathBuf,
    
    /// Categories/tags for organization
    categories: Arc<RwLock<HashMap<String, Vec<u64>>>>, // category -> chunk_ids
    
    /// Next chunk ID
    next_id: Arc<RwLock<u64>>,
}

/// User knowledge chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKnowledgeChunk {
    /// Unique ID
    pub id: u64,
    
    /// Text content
    pub text: String,
    
    /// Embedding vector
    #[serde(skip)]
    pub embedding: Vec<f32>,
    
    /// Source/reference (URL, book, etc.)
    pub source: String,
    
    /// Category/tag
    pub category: String,
    
    /// User-defined tags
    pub tags: Vec<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Is this chunk pinned (high priority)?
    pub pinned: bool,
    
    /// Privacy level
    pub privacy: PrivacyLevel,
}

/// Privacy level for knowledge chunks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PrivacyLevel {
    /// Only on this device
    Private,
    /// Share with trusted peers
    Trusted,
    /// Share with all swarm peers
    Public,
}

/// Knowledge operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeOpResult {
    pub success: bool,
    pub message: String,
    pub chunk_id: Option<u64>,
}

impl KnowledgeManager {
    /// Create a new knowledge manager
    pub fn new(user_did: String, storage_path: PathBuf) -> Result<Self> {
        // Create storage directory if it doesn't exist
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let manager = Self {
            chunks: Arc::new(RwLock::new(Vec::new())),
            embedding_gen: Arc::new(EmbeddingGenerator::default()),
            user_did,
            storage_path,
            categories: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        };

        // Load existing knowledge if available
        if manager.storage_path.exists() {
            manager.load_from_disk()?;
        }

        Ok(manager)
    }

    /// Add a new knowledge chunk
    pub async fn add_chunk(
        &self,
        text: String,
        source: String,
        category: String,
        tags: Vec<String>,
        privacy: PrivacyLevel,
    ) -> Result<KnowledgeOpResult> {
        // Generate embedding
        let embedding = self.embedding_gen.embed(&text)?;

        // Get next ID
        let id = {
            let mut next_id = self.next_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Create chunk
        let chunk = UserKnowledgeChunk {
            id,
            text: text.clone(),
            embedding,
            source,
            category: category.clone(),
            tags,
            created_at: now,
            modified_at: now,
            pinned: false,
            privacy,
        };

        // Add to chunks
        {
            let mut chunks = self.chunks.write().await;
            chunks.push(chunk);
        }

        // Update category index
        {
            let mut categories = self.categories.write().await;
            categories.entry(category).or_insert_with(Vec::new).push(id);
        }

        // Persist
        self.save_to_disk().await?;

        Ok(KnowledgeOpResult {
            success: true,
            message: format!("Added knowledge chunk #{}", id),
            chunk_id: Some(id),
        })
    }

    /// Update an existing chunk
    pub async fn update_chunk(
        &self,
        id: u64,
        text: Option<String>,
        source: Option<String>,
        category: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<KnowledgeOpResult> {
        let mut chunks = self.chunks.write().await;
        
        let chunk = chunks.iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| anyhow!("Chunk #{} not found", id))?;

        let mut changed = false;
        let mut category_change = None;

        // Update text and regenerate embedding if changed
        if let Some(new_text) = text {
            if new_text != chunk.text {
                let new_embedding = self.embedding_gen.embed(&new_text)?;
                chunk.text = new_text;
                chunk.embedding = new_embedding;
                changed = true;
            }
        }

        if let Some(new_source) = source {
            chunk.source = new_source;
            changed = true;
        }

        if let Some(new_category) = category {
            if new_category != chunk.category {
                let old_category = chunk.category.clone();
                chunk.category = new_category.clone();
                category_change = Some((old_category, new_category));
                changed = true;
            }
        }

        if let Some(new_tags) = tags {
            if new_tags != chunk.tags {
                chunk.tags = new_tags;
                changed = true;
            }
        }

        if changed {
            chunk.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
        }

        // Release chunks lock
        drop(chunks);

        // Update category index if needed
        if let Some((old_cat, new_cat)) = category_change {
            let mut categories = self.categories.write().await;
            
            // Remove from old category
            if let Some(old_ids) = categories.get_mut(&old_cat) {
                old_ids.retain(|&cid| cid != id);
            }
            
            // Add to new category
            categories.entry(new_cat).or_insert_with(Vec::new).push(id);
        }

        if changed {
            self.save_to_disk().await?;
        }

        Ok(KnowledgeOpResult {
            success: true,
            message: format!("Updated chunk #{}", id),
            chunk_id: Some(id),
        })
    }

    /// Delete a chunk
    pub async fn delete_chunk(&self, id: u64) -> Result<KnowledgeOpResult> {
        let mut chunks = self.chunks.write().await;
        
        let idx = chunks.iter()
            .position(|c| c.id == id)
            .ok_or_else(|| anyhow!("Chunk #{} not found", id))?;

        let chunk = chunks.remove(idx);

        // Update category index
        drop(chunks);
        let mut categories = self.categories.write().await;
        if let Some(cat_ids) = categories.get_mut(&chunk.category) {
            cat_ids.retain(|&cid| cid != id);
        }

        drop(categories);
        self.save_to_disk().await?;

        Ok(KnowledgeOpResult {
            success: true,
            message: format!("Deleted chunk #{}", id),
            chunk_id: Some(id),
        })
    }

    /// Pin/unpin a chunk (high priority)
    pub async fn toggle_pin(&self, id: u64) -> Result<KnowledgeOpResult> {
        let mut chunks = self.chunks.write().await;
        
        let chunk = chunks.iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| anyhow!("Chunk #{} not found", id))?;

        chunk.pinned = !chunk.pinned;
        
        drop(chunks);
        self.save_to_disk().await?;

        Ok(KnowledgeOpResult {
            success: true,
            message: format!("Chunk #{} pin toggled", id),
            chunk_id: Some(id),
        })
    }

    /// Search user's knowledge
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<UserKnowledgeChunk>> {
        let query_embedding = self.embedding_gen.embed(query)?;
        let chunks = self.chunks.read().await;

        let mut scored: Vec<(f32, UserKnowledgeChunk)> = chunks.iter()
            .map(|chunk| {
                let similarity = super::embeddings::cosine_similarity(&query_embedding, &chunk.embedding);
                // Boost pinned chunks
                let score = if chunk.pinned { similarity * 1.2 } else { similarity };
                (score, chunk.clone())
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        Ok(scored.iter().take(limit).map(|(_, c)| c.clone()).collect())
    }

    /// Get chunks by category
    pub async fn get_by_category(&self, category: &str) -> Result<Vec<UserKnowledgeChunk>> {
        let categories = self.categories.read().await;
        let chunk_ids = categories.get(category)
            .ok_or_else(|| anyhow!("Category '{}' not found", category))?;

        let chunks = self.chunks.read().await;
        Ok(chunks.iter()
            .filter(|c| chunk_ids.contains(&c.id))
            .cloned()
            .collect())
    }

    /// Get chunks by tag
    pub async fn get_by_tag(&self, tag: &str) -> Result<Vec<UserKnowledgeChunk>> {
        let chunks = self.chunks.read().await;
        Ok(chunks.iter()
            .filter(|c| c.tags.contains(&tag.to_string()))
            .cloned()
            .collect())
    }

    /// List all categories
    pub async fn list_categories(&self) -> Vec<String> {
        let categories = self.categories.read().await;
        categories.keys().cloned().collect()
    }

    /// Get all chunks (for export/backup)
    pub async fn get_all_chunks(&self) -> Vec<UserKnowledgeChunk> {
        self.chunks.read().await.clone()
    }

    /// Get pinned chunks
    pub async fn get_pinned(&self) -> Vec<UserKnowledgeChunk> {
        let chunks = self.chunks.read().await;
        chunks.iter().filter(|c| c.pinned).cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> KnowledgeStats {
        let chunks = self.chunks.read().await;
        let categories = self.categories.read().await;

        let total_chunks = chunks.len();
        let pinned_chunks = chunks.iter().filter(|c| c.pinned).count();
        let total_categories = categories.len();
        
        let privacy_breakdown = {
            let mut map = HashMap::new();
            for chunk in chunks.iter() {
                *map.entry(chunk.privacy).or_insert(0) += 1;
            }
            map
        };

        KnowledgeStats {
            total_chunks,
            pinned_chunks,
            total_categories,
            privacy_breakdown,
        }
    }

    /// Convert to RAG chunks for querying
    pub async fn to_rag_chunks(&self, privacy_filter: Option<PrivacyLevel>) -> Vec<RagChunk> {
        let chunks = self.chunks.read().await;
        
        chunks.iter()
            .filter(|c| {
                if let Some(filter) = privacy_filter {
                    c.privacy == filter || c.privacy == PrivacyLevel::Public
                } else {
                    true
                }
            })
            .map(|c| RagChunk {
                text: c.text.clone(),
                embedding: c.embedding.clone(),
                score: if c.pinned { 1.0 } else { 0.9 },
                source_doc: c.source.clone(),
                timestamp: c.modified_at,
            })
            .collect()
    }

    /// Save to disk
    async fn save_to_disk(&self) -> Result<()> {
        let chunks = self.chunks.read().await;
        let json = serde_json::to_string_pretty(&*chunks)?;
        fs::write(&self.storage_path, json)?;
        Ok(())
    }

    /// Load from disk
    fn load_from_disk(&self) -> Result<()> {
        let json = fs::read_to_string(&self.storage_path)?;
        let mut chunks: Vec<UserKnowledgeChunk> = serde_json::from_str(&json)?;
        
        // Regenerate embeddings (not serialized)
        for chunk in &mut chunks {
            chunk.embedding = self.embedding_gen.embed(&chunk.text)?;
        }

        // Rebuild category index
        let mut categories = HashMap::new();
        let mut max_id = 0;
        
        for chunk in &chunks {
            categories.entry(chunk.category.clone())
                .or_insert_with(Vec::new)
                .push(chunk.id);
            if chunk.id > max_id {
                max_id = chunk.id;
            }
        }

        // Update state
        tokio::runtime::Runtime::new()?.block_on(async {
            *self.chunks.write().await = chunks;
            *self.categories.write().await = categories;
            *self.next_id.write().await = max_id + 1;
        });

        Ok(())
    }
}

/// Knowledge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_chunks: usize,
    pub pinned_chunks: usize,
    pub total_categories: usize,
    pub privacy_breakdown: HashMap<PrivacyLevel, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_knowledge_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        
        let manager = KnowledgeManager::new(
            "did:karana:test".to_string(),
            storage_path,
        ).unwrap();
        
        assert_eq!(manager.user_did, "did:karana:test");
    }

    #[tokio::test]
    async fn test_add_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        let result = manager.add_chunk(
            "Rust is a systems programming language".to_string(),
            "rust-lang.org".to_string(),
            "programming".to_string(),
            vec!["rust".to_string(), "systems".to_string()],
            PrivacyLevel::Private,
        ).await.unwrap();

        assert!(result.success);
        assert_eq!(result.chunk_id, Some(1));
    }

    #[tokio::test]
    async fn test_search() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        manager.add_chunk(
            "Rust is great for systems programming".to_string(),
            "test".to_string(),
            "programming".to_string(),
            vec![],
            PrivacyLevel::Private,
        ).await.unwrap();

        let results = manager.search("Rust programming", 5).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_update_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        let add_result = manager.add_chunk(
            "Original text".to_string(),
            "source".to_string(),
            "test".to_string(),
            vec![],
            PrivacyLevel::Private,
        ).await.unwrap();

        let chunk_id = add_result.chunk_id.unwrap();

        let update_result = manager.update_chunk(
            chunk_id,
            Some("Updated text".to_string()),
            None,
            None,
            None,
        ).await.unwrap();

        assert!(update_result.success);
    }

    #[tokio::test]
    async fn test_delete_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        let add_result = manager.add_chunk(
            "Test".to_string(),
            "source".to_string(),
            "test".to_string(),
            vec![],
            PrivacyLevel::Private,
        ).await.unwrap();

        let chunk_id = add_result.chunk_id.unwrap();
        let delete_result = manager.delete_chunk(chunk_id).await.unwrap();
        
        assert!(delete_result.success);
    }

    #[tokio::test]
    async fn test_pin_toggle() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        let add_result = manager.add_chunk(
            "Test".to_string(),
            "source".to_string(),
            "test".to_string(),
            vec![],
            PrivacyLevel::Private,
        ).await.unwrap();

        let chunk_id = add_result.chunk_id.unwrap();
        manager.toggle_pin(chunk_id).await.unwrap();

        let pinned = manager.get_pinned().await;
        assert_eq!(pinned.len(), 1);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("knowledge.json");
        let manager = KnowledgeManager::new("did:karana:test".to_string(), storage_path).unwrap();

        manager.add_chunk(
            "Test 1".to_string(),
            "source".to_string(),
            "cat1".to_string(),
            vec![],
            PrivacyLevel::Private,
        ).await.unwrap();

        manager.add_chunk(
            "Test 2".to_string(),
            "source".to_string(),
            "cat2".to_string(),
            vec![],
            PrivacyLevel::Public,
        ).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_chunks, 2);
        assert_eq!(stats.total_categories, 2);
    }
}
