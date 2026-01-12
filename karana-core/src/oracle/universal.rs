// Universal Oracle - Handle any random query with RAG + tools
// Phase 41: Transform oracle from intent executor to universal knowledge companion

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use serde::{Deserialize, Serialize};
use anyhow::{Result, bail};

use super::embeddings::{EmbeddingGenerator, cosine_similarity};
use super::swarm_knowledge::SwarmKnowledge;
use super::knowledge_manager::KnowledgeManager;
use super::knowledge_graph::{KnowledgeGraphBuilder, KnowledgeGraph};
use super::web_search::{WebSearchEngine, WebSearchResult};
use super::knowledge_base::{OfflineKnowledgeBase, WikiArticle};
use super::cache::{SearchCache, EmbeddingCache, CachedResult};

/// Universal query response with provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    pub answer: String,
    pub source: ResponseSource,
    pub confidence: f32,
    pub proof: Option<Vec<u8>>,  // ZK attestation
    pub follow_up: Vec<String>,
}

/// Knowledge graph insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInsights {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub central_nodes: Vec<(String, usize)>,
    pub num_clusters: usize,
    pub density: f32,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseSource {
    LocalKnowledge,      // RAG from local vector DB
    SwarmPeers,          // libp2p gossip from peers
    ChainOracle,         // L3 oracle query
    WebProxy,            // External web search
    ComputedAnswer,      // Math/logic computation
}

/// RAG chunk from local knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub text: String,
    pub embedding: Vec<f32>,
    pub score: f32,
    pub source_doc: String,
    pub timestamp: u64,
}

/// Universal Oracle - handles any query
pub struct UniversalOracle {
    pub local_knowledge: Arc<LocalKnowledgeBase>,
    pub swarm_query_enabled: bool,
    pub web_proxy_enabled: bool,
    pub user_knowledge_enabled: bool,
    pub embedding_dim: usize,
    embedding_gen: Arc<EmbeddingGenerator>,
    swarm_knowledge: Arc<SwarmKnowledge>,
    knowledge_manager: Option<Arc<KnowledgeManager>>,
    graph_builder: Arc<KnowledgeGraphBuilder>,
    ai: Option<Arc<StdMutex<crate::ai::KaranaAI>>>,  // For LLM synthesis
    web_search: Option<Arc<WebSearchEngine>>,  // Phase 3: Web search
    offline_kb: Option<Arc<StdMutex<OfflineKnowledgeBase>>>,  // Phase 3: Wikipedia
    search_cache: Arc<SearchCache>,  // Phase 5: Result caching
    embedding_cache: Arc<EmbeddingCache>,  // Phase 5: Embedding caching
}

impl UniversalOracle {
    pub fn new() -> Result<Self> {
        Ok(Self {
            local_knowledge: Arc::new(LocalKnowledgeBase::new()?),
            swarm_query_enabled: true,
            web_proxy_enabled: false,  // Disabled by default (privacy)
            user_knowledge_enabled: true,
            embedding_dim: 384,  // MiniLM-L6-v2 dimension
            embedding_gen: Arc::new(EmbeddingGenerator::default()),
            swarm_knowledge: Arc::new(SwarmKnowledge::new("did:karana:anonymous".to_string())),
            knowledge_manager: None,  // Set via set_knowledge_manager()
            graph_builder: Arc::new(KnowledgeGraphBuilder::new()),
            ai: None,  // Set via set_ai()
            web_search: Some(Arc::new(WebSearchEngine::new())),  // Phase 3: Enabled by default
            offline_kb: None,  // Set via set_offline_kb()
            search_cache: Arc::new(SearchCache::new(1000)),  // Phase 5: 1000 cached queries
            embedding_cache: Arc::new(EmbeddingCache::new(500)),  // Phase 5: 500 cached embeddings
        })
    }
    
    /// Set the knowledge manager for user's personal knowledge
    pub fn set_knowledge_manager(&mut self, manager: Arc<KnowledgeManager>) {
        self.knowledge_manager = Some(manager);
    }
    
    /// Set AI instance for LLM-based synthesis
    pub fn set_ai(&mut self, ai: Arc<StdMutex<crate::ai::KaranaAI>>) {
        self.ai = Some(ai);
    }
    
    /// Set offline knowledge base (Wikipedia)
    pub fn set_offline_kb(&mut self, kb: Arc<StdMutex<OfflineKnowledgeBase>>) {
        self.offline_kb = Some(kb);
    }
    
    /// Enable/disable web search
    pub fn set_web_search_enabled(&mut self, enabled: bool) {
        if enabled && self.web_search.is_none() {
            self.web_search = Some(Arc::new(WebSearchEngine::new()));
        } else if !enabled {
            self.web_search = None;
        }
    }
    
    /// Get reference to knowledge manager
    pub fn knowledge_manager(&self) -> Option<Arc<KnowledgeManager>> {
        self.knowledge_manager.clone()
    }

    /// Main entry point - handle any query
    pub async fn query(&self, query: &str, context: &QueryContext) -> Result<UniversalResponse> {
        // 0. Check cache first (Phase 5: Performance optimization)
        if let Some(cached) = self.search_cache.get(query) {
            log::info!("[UniversalOracle] ⚡ Cache hit for: {}", query);
            return Ok(UniversalResponse {
                answer: cached.answer,
                source: ResponseSource::LocalKnowledge,
                confidence: cached.confidence,
                proof: None,
                follow_up: vec!["Tell me more".to_string()],
            });
        }

        // 1. Try computation first (math, logic) - deterministic and fast
        if let Some(response) = self.compute_answer(query)? {
            // Cache computational results (they don't change)
            self.search_cache.put(
                query.to_string(),
                CachedResult::new(response.answer.clone(), response.confidence, 86400) // 24h TTL
            );
            return Ok(response);
        }

        // 2. Embed query (with caching)
        let embedding = self.embed_query(query)?;

        // 3. Try user's personal knowledge first (highest priority)
        if self.user_knowledge_enabled {
            if let Some(response) = self.query_user_knowledge(&embedding, query).await? {
                // Cache user knowledge results (1 hour TTL)
                self.search_cache.put(
                    query.to_string(),
                    CachedResult::new(response.answer.clone(), response.confidence, 3600)
                );
                return Ok(response);
            }
        }

        // 4. Try offline Wikipedia knowledge base (fast, comprehensive)
        if let Some(response) = self.query_offline_wikipedia(query).await? {
            // Cache Wikipedia results (6 hours TTL - semi-static)
            self.search_cache.put(
                query.to_string(),
                CachedResult::new(response.answer.clone(), response.confidence, 21600)
            );
            return Ok(response);
        }

        // 5. Try local RAG (offline, fast)
        if let Some(response) = self.query_local(&embedding, query).await? {
            // Cache RAG results (2 hours TTL)
            self.search_cache.put(
                query.to_string(),
                CachedResult::new(response.answer.clone(), response.confidence, 7200)
            );
            return Ok(response);
        }

        // 6. Try swarm peers (semi-online, decentralized)
        if self.swarm_query_enabled {
            if let Some(response) = self.query_swarm(query, context).await? {
                // Cache swarm results (30 min TTL - dynamic)
                self.search_cache.put(
                    query.to_string(),
                    CachedResult::new(response.answer.clone(), response.confidence, 1800)
                );
                return Ok(response);
            }
        }

        // 7. Try web search (online, broad knowledge)
        if let Some(response) = self.query_web_search(query).await? {
            // Cache web results (15 min TTL - frequently changing)
            self.search_cache.put(
                query.to_string(),
                CachedResult::new(response.answer.clone(), response.confidence, 900)
            );
            return Ok(response);
        }

        // 8. Fallback to web proxy (fully online, privacy concern)
        if self.web_proxy_enabled {
            if let Some(response) = self.query_web_proxy(query, context).await? {
                return Ok(response);
            }
        }

        // 9. Generate best-effort response
        self.generate_fallback(query)
    }

    /// Embed query text to vector (with caching)
    fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached_embedding) = self.embedding_cache.get(text) {
            log::debug!("[UniversalOracle] Embedding cache hit");
            return Ok(cached_embedding);
        }
        
        // Generate embedding
        let embedding = self.embedding_gen.embed(text)?;
        
        // Cache it
        self.embedding_cache.put(text.to_string(), embedding.clone());
        
        Ok(embedding)
    }

    /// Query user's personal knowledge base
    async fn query_user_knowledge(&self, embedding: &[f32], query: &str) -> Result<Option<UniversalResponse>> {
        let manager = match &self.knowledge_manager {
            Some(m) => m,
            None => return Ok(None),
        };

        // Search user's knowledge with higher limit (they want their own knowledge prioritized)
        let results = manager.search(query, 3).await?;
        
        if results.is_empty() {
            return Ok(None);
        }

        // User knowledge gets priority - use first result if confidence is decent
        let best = &results[0];
        let similarity = cosine_similarity(embedding, &best.embedding);
        
        if similarity < 0.4 {
            return Ok(None);
        }

        Ok(Some(UniversalResponse {
            answer: best.text.clone(),
            source: ResponseSource::LocalKnowledge,
            confidence: similarity,
            proof: None,
            follow_up: vec![
                "Tell me more from my notes".to_string(),
                format!("Other notes in {}", best.category),
            ],
        }))
    }

    /// Query local RAG knowledge base
    async fn query_local(&self, embedding: &[f32], query: &str) -> Result<Option<UniversalResponse>> {
        let results = self.local_knowledge.search(embedding, 3)?;
        
        if results.is_empty() || results[0].score < 0.3 {
            // No results or too low confidence
            return Ok(None);
        }

        // Combine top-k results
        let answer = self.synthesize_answer(&results, query)?;
        
        Ok(Some(UniversalResponse {
            answer,
            source: ResponseSource::LocalKnowledge,
            confidence: results[0].score,
            proof: None,
            follow_up: vec![
                "Tell me more".to_string(),
                "Related topics?".to_string(),
            ],
        }))
    }

    /// Query swarm peers via libp2p gossip
    async fn query_swarm(&self, query: &str, _context: &QueryContext) -> Result<Option<UniversalResponse>> {
        let embedding = self.embed_query(query)?;
        self.swarm_knowledge.query_swarm(query, embedding).await
    }

    /// Compute answer for math/logic queries
    fn compute_answer(&self, query: &str) -> Result<Option<UniversalResponse>> {
        // Check if this is a computable query
        if let Some(answer) = self.try_compute_math(query)? {
            return Ok(Some(UniversalResponse {
                answer,
                source: ResponseSource::ComputedAnswer,
                confidence: 1.0,
                proof: None,
                follow_up: vec!["Show steps?".to_string()],
            }));
        }

        Ok(None)
    }

    /// Query offline Wikipedia knowledge base (Phase 3)
    async fn query_offline_wikipedia(&self, query: &str) -> Result<Option<UniversalResponse>> {
        let kb = match &self.offline_kb {
            Some(kb) => kb,
            None => return Ok(None),
        };

        let kb_lock = kb.lock().unwrap();
        
        // Try exact title match first
        if let Some(article) = kb_lock.get_article(query).ok().flatten() {
            log::info!("[UniversalOracle] Found exact Wikipedia article: {}", article.title);
            return Ok(Some(UniversalResponse {
                answer: article.summary.clone(),
                source: ResponseSource::LocalKnowledge,
                confidence: 0.95,
                proof: None,
                follow_up: vec![
                    "Tell me more".to_string(),
                    "Related topics".to_string(),
                ],
            }));
        }
        
        // Try full-text search
        let results = kb_lock.full_text_search(query, 3).ok().unwrap_or_default();
        
        if !results.is_empty() {
            let best = &results[0];
            log::info!("[UniversalOracle] Found Wikipedia match: {}", best.title);
            
            return Ok(Some(UniversalResponse {
                answer: format!("{}\n\nSource: Wikipedia - {}", best.summary, best.title),
                source: ResponseSource::LocalKnowledge,
                confidence: 0.8,
                proof: None,
                follow_up: vec![
                    "Read full article".to_string(),
                    format!("More about {}", best.categories.first().unwrap_or(&"this topic".to_string())),
                ],
            }));
        }
        
        Ok(None)
    }

    /// Query web search (Phase 3)
    async fn query_web_search(&self, query: &str) -> Result<Option<UniversalResponse>> {
        let search = match &self.web_search {
            Some(s) => s,
            None => return Ok(None),
        };

        log::info!("[UniversalOracle] Searching web for: {}", query);
        
        let results = match search.search(query, 5).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[UniversalOracle] Web search failed: {}", e);
                return Ok(None);
            }
        };
        
        if results.is_empty() {
            return Ok(None);
        }
        
        // Synthesize answer from web results
        let synthesized = self.synthesize_web_results(&results, query)?;
        
        log::info!("[UniversalOracle] Web search successful, {} results", results.len());
        
        Ok(Some(UniversalResponse {
            answer: synthesized,
            source: ResponseSource::WebProxy,
            confidence: 0.75,
            proof: None,
            follow_up: vec![
                "More details".to_string(),
                "Related searches".to_string(),
            ],
        }))
    }

    /// Synthesize answer from web search results
    fn synthesize_web_results(&self, results: &[WebSearchResult], query: &str) -> Result<String> {
        if results.is_empty() {
            return Ok(String::new());
        }
        
        // Build context from search results
        let context = results.iter()
            .take(3)
            .enumerate()
            .map(|(i, r)| format!("[{}] {}: {}", i + 1, r.title, r.snippet))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        // Try LLM synthesis if available
        if let Some(ai) = &self.ai {
            let prompt = format!(
                "Based on these web search results, provide a clear answer.\n\n{}\n\nQuestion: {}\n\nAnswer:",
                context, query
            );
            
            let mut ai_lock = ai.lock().unwrap();
            if let Ok(answer) = ai_lock.predict(&prompt, 150) {
                return Ok(format!("{}\n\nSources: {}", 
                    answer.trim(),
                    results.iter().take(2).map(|r| &r.title).cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
        
        // Fallback: use best snippet
        Ok(format!("{}\n\nSource: {}", results[0].snippet, results[0].title))
    }

    /// Query external web via proxy
    async fn query_web_proxy(&self, _query: &str, _context: &QueryContext) -> Result<Option<UniversalResponse>> {
        // TODO: Implement web proxy (privacy-preserving)
        Ok(None)
    }

    /// Generate fallback response
    fn generate_fallback(&self, query: &str) -> Result<UniversalResponse> {
        Ok(UniversalResponse {
            answer: format!("I don't have information about '{}' in my knowledge base. Would you like me to search the swarm?", query),
            source: ResponseSource::LocalKnowledge,
            confidence: 0.1,
            proof: None,
            follow_up: vec![
                "Search swarm".to_string(),
                "Add to knowledge".to_string(),
            ],
        })
    }

    /// Build knowledge graph visualization
    pub async fn build_knowledge_graph(&self) -> Result<Option<KnowledgeGraph>> {
        if let Some(ref km) = self.knowledge_manager {
            let chunks = km.get_all_chunks().await;
            
            if chunks.is_empty() {
                return Ok(None);
            }

            let graph = self.graph_builder.build_from_chunks(&chunks)?;
            Ok(Some(graph))
        } else {
            Ok(None)
        }
    }

    /// Get graph analysis metrics
    pub async fn get_graph_insights(&self) -> Result<Option<GraphInsights>> {
        if let Some(graph) = self.build_knowledge_graph().await? {
            let central_nodes = super::knowledge_graph::GraphAnalyzer::find_central_nodes(&graph, 5);
            let clusters = super::knowledge_graph::GraphAnalyzer::find_clusters(&graph);
            let density = super::knowledge_graph::GraphAnalyzer::calculate_density(&graph);

            Ok(Some(GraphInsights {
                total_nodes: graph.nodes.len(),
                total_edges: graph.edges.len(),
                central_nodes,
                num_clusters: clusters.len(),
                density,
                last_updated: graph.metadata.last_updated,
            }))
        } else {
            Ok(None)
        }
    }

    /// Synthesize answer from RAG results using real LLM
    fn synthesize_answer(&self, results: &[RagChunk], query: &str) -> Result<String> {
        // PHASE 1: Perfect AI Oracle - Real LLM Synthesis
        
        if results.is_empty() {
            return Ok("I couldn't find relevant information to answer that question.".to_string());
        }
        
        // Build context from RAG results (top 5 most relevant)
        let context = results.iter()
            .take(5)
            .enumerate()
            .map(|(i, r)| {
                format!("Source {}: {}\n{}", 
                    i + 1, 
                    r.source_doc,
                    r.text.chars().take(500).collect::<String>() // Limit context length
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        // Construct prompt for LLM synthesis
        let prompt = format!(
            "Based on the following information sources, provide a clear and concise answer to the question.\n\n\
             Information:\n{}\n\n\
             Question: {}\n\n\
             Answer (be natural and helpful):",
            context, query
        );
        
        // Use real AI to synthesize natural language answer if available
        if let Some(ai) = &self.ai {
            let mut ai_lock = ai.lock().unwrap();
            match ai_lock.predict(&prompt, 200) {
                Ok(answer) => {
                    log::info!("[UniversalOracle] ✓ LLM synthesized answer ({} chars)", answer.len());
                    return Ok(answer.trim().to_string());
                }
                Err(e) => {
                    log::warn!("[UniversalOracle] LLM synthesis failed: {}. Using fallback.", e);
                }
            }
        }
        
        // Fallback: improved concatenation with source attribution
        let fallback = results.iter()
            .take(2)
            .map(|r| r.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(format!("Based on available information: {}", fallback))
    }

    /// Try to compute math answer
    fn try_compute_math(&self, query: &str) -> Result<Option<String>> {
        let q = query.to_lowercase();

        // Simple patterns
        if q.contains("what is") && q.contains("+") {
            if let Some(result) = parse_addition(&q) {
                return Ok(Some(format!("The answer is {}", result)));
            }
        }

        if q.contains("square root") {
            if let Some(result) = parse_sqrt(&q) {
                return Ok(Some(format!("The square root is {:.2}", result)));
            }
        }

        Ok(None)
    }
}

impl Default for UniversalOracle {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Query context for personalization
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub location: Option<String>,
    pub time_of_day: String,
    pub recent_topics: Vec<String>,
    pub user_preferences: HashMap<String, String>,
}

/// Local knowledge base using vector search
pub struct LocalKnowledgeBase {
    chunks: Vec<RagChunk>,
}

impl LocalKnowledgeBase {
    pub fn new() -> Result<Self> {
        Ok(Self {
            chunks: Self::load_default_knowledge()?,
        })
    }

    /// Load default knowledge chunks
    fn load_default_knowledge() -> Result<Vec<RagChunk>> {
        let generator = EmbeddingGenerator::default();
        
        let texts = vec![
            ("Kāraṇa OS is a self-sovereign operating system for smart glasses with blockchain integration.", "system"),
            ("The capital of France is Paris, located on the Seine River in northern France.", "geography"),
            ("Quantum entanglement is a phenomenon where particles become correlated such that the state of one particle instantly affects the state of another, regardless of distance.", "physics"),
            ("The Pythagorean theorem states that in a right triangle, a² + b² = c², where c is the hypotenuse.", "mathematics"),
        ];
        
        let mut chunks = Vec::new();
        for (text, source) in texts {
            let embedding = generator.embed(text)?;
            chunks.push(RagChunk {
                text: text.to_string(),
                embedding,
                score: 0.0,
                source_doc: source.to_string(),
                timestamp: 0,
            });
        }
        
        Ok(chunks)
    }

    /// Search for similar chunks using cosine similarity
    pub fn search(&self, embedding: &[f32], k: usize) -> Result<Vec<RagChunk>> {
        // Compute cosine similarity for each chunk
        let mut results: Vec<RagChunk> = self.chunks.iter().map(|chunk| {
            let similarity = cosine_similarity(embedding, &chunk.embedding);
            RagChunk {
                text: chunk.text.clone(),
                embedding: chunk.embedding.clone(),
                score: similarity,
                source_doc: chunk.source_doc.clone(),
                timestamp: chunk.timestamp,
            }
        }).collect();
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Return top-k
        results.truncate(k);
        Ok(results)
    }

    /// Add new knowledge chunk
    pub fn add_chunk(&mut self, chunk: RagChunk) -> Result<()> {
        self.chunks.push(chunk);
        Ok(())
    }

    /// Get total chunks
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// Parse simple addition from text
fn parse_addition(text: &str) -> Option<f64> {
    // Very simple parser for "what is X + Y"
    let parts: Vec<&str> = text.split('+').collect();
    if parts.len() == 2 {
        let a = parts[0].split_whitespace().last()?.trim_matches(|c: char| !c.is_numeric() && c != '.')
            .parse::<f64>().ok()?;
        let b = parts[1].split_whitespace().next()?.trim_matches(|c: char| !c.is_numeric() && c != '.')
            .parse::<f64>().ok()?;
        return Some(a + b);
    }
    None
}

/// Parse square root from text
fn parse_sqrt(text: &str) -> Option<f64> {
    // Simple parser for "square root of X"
    if let Some(idx) = text.find("of ") {
        let num_str = &text[idx + 3..].split_whitespace().next()?;
        let num = num_str.parse::<f64>().ok()?;
        return Some(num.sqrt());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_oracle_creation() {
        let oracle = UniversalOracle::new().unwrap();
        assert_eq!(oracle.embedding_dim, 384);
        assert!(oracle.swarm_query_enabled);
    }

    #[tokio::test]
    async fn test_query_local_knowledge() {
        let oracle = UniversalOracle::new().unwrap();
        let context = QueryContext {
            location: None,
            time_of_day: "morning".to_string(),
            recent_topics: vec![],
            user_preferences: HashMap::new(),
        };

        let response = oracle.query("What is Karana OS?", &context).await.unwrap();
        assert!(response.answer.contains("Kāraṇa") || response.confidence < 0.5);
    }

    #[tokio::test]
    async fn test_math_computation() {
        let oracle = UniversalOracle::new().unwrap();
        let context = QueryContext {
            location: None,
            time_of_day: "morning".to_string(),
            recent_topics: vec![],
            user_preferences: HashMap::new(),
        };

        let response = oracle.query("What is 5 + 3?", &context).await.unwrap();
        assert_eq!(response.source, ResponseSource::ComputedAnswer);
        assert!(response.answer.contains("8"));
    }

    #[test]
    fn test_local_knowledge_base() {
        let kb = LocalKnowledgeBase::new().unwrap();
        assert!(!kb.is_empty());
        assert!(kb.len() >= 4);
    }

    #[test]
    fn test_parse_addition() {
        assert_eq!(parse_addition("what is 10 + 5"), Some(15.0));
        assert_eq!(parse_addition("what is 3.5 + 2.5"), Some(6.0));
    }

    #[test]
    fn test_parse_sqrt() {
        assert_eq!(parse_sqrt("square root of 16"), Some(4.0));
        assert_eq!(parse_sqrt("square root of 25"), Some(5.0));
    }
}
