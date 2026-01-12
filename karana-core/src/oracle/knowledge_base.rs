// Kāraṇa OS - Offline Knowledge Base (Wikipedia Indexing)
// Phase 3: Enhanced Knowledge Base - 1M+ articles offline

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use rocksdb::{DB, Options};

/// Wikipedia article metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiArticle {
    pub title: String,
    pub content: String,
    pub categories: Vec<String>,
    pub summary: String,  // First paragraph
    pub links: Vec<String>,  // Outgoing links
    pub last_updated: u64,  // Timestamp
}

/// Offline Wikipedia knowledge base
pub struct OfflineKnowledgeBase {
    db: DB,
    article_count: usize,
    index_embeddings: bool,
}

impl OfflineKnowledgeBase {
    /// Create or open offline knowledge base
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
        
        let db = DB::open(&opts, db_path)?;
        
        // Count existing articles
        let article_count = db.iterator(rocksdb::IteratorMode::Start)
            .filter_map(|item| item.ok())
            .filter(|(key, _)| key.starts_with(b"article:"))
            .count();
        
        log::info!("[KnowledgeBase] Loaded {} articles", article_count);
        
        Ok(Self {
            db,
            article_count,
            index_embeddings: true,
        })
    }

    /// Index a Wikipedia article
    pub fn index_article(&mut self, article: WikiArticle) -> Result<()> {
        let key = format!("article:{}", article.title);
        let value = bincode::serialize(&article)?;
        
        self.db.put(key.as_bytes(), value)?;
        
        // Index by categories for faster lookup
        for category in &article.categories {
            let cat_key = format!("category:{}:{}", category, article.title);
            self.db.put(cat_key.as_bytes(), b"1")?;
        }
        
        self.article_count += 1;
        
        if self.article_count % 10000 == 0 {
            log::info!("[KnowledgeBase] Indexed {} articles", self.article_count);
        }
        
        Ok(())
    }

    /// Retrieve article by exact title
    pub fn get_article(&self, title: &str) -> Result<Option<WikiArticle>> {
        let key = format!("article:{}", title);
        
        match self.db.get(key.as_bytes())? {
            Some(data) => {
                let article: WikiArticle = bincode::deserialize(&data)?;
                Ok(Some(article))
            }
            None => Ok(None),
        }
    }

    /// Search articles by title prefix (for autocomplete)
    pub fn search_by_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<String>> {
        let search_key = format!("article:{}", prefix.to_lowercase());
        let mut results = Vec::new();
        
        let iter = self.db.iterator(rocksdb::IteratorMode::From(
            search_key.as_bytes(),
            rocksdb::Direction::Forward,
        ));
        
        for item in iter.take(limit) {
            if let Ok((key, _)) = item {
                if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                    if key_str.starts_with("article:") {
                        let title = key_str.strip_prefix("article:").unwrap_or("");
                        if title.to_lowercase().starts_with(&prefix.to_lowercase()) {
                            results.push(title.to_string());
                        } else {
                            break;  // Prefix no longer matches
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Get articles in a category
    pub fn get_category_articles(&self, category: &str, limit: usize) -> Result<Vec<String>> {
        let search_key = format!("category:{}", category);
        let mut results = Vec::new();
        
        let iter = self.db.iterator(rocksdb::IteratorMode::From(
            search_key.as_bytes(),
            rocksdb::Direction::Forward,
        ));
        
        for item in iter.take(limit) {
            if let Ok((key, _)) = item {
                if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                    if key_str.starts_with(&search_key) {
                        // Extract article title from "category:NAME:TITLE"
                        let parts: Vec<&str> = key_str.split(':').collect();
                        if parts.len() >= 3 {
                            results.push(parts[2..].join(":")); 
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Full-text search across articles (basic implementation)
    pub fn full_text_search(&self, query: &str, limit: usize) -> Result<Vec<WikiArticle>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        let mut scored_results: Vec<(WikiArticle, f32)> = Vec::new();
        
        // Iterate through all articles
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        
        for item in iter {
            if let Ok((key, value)) = item {
                if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                    if key_str.starts_with("article:") {
                        if let Ok(article) = bincode::deserialize::<WikiArticle>(&value) {
                            // Simple relevance scoring
                            let mut score = 0.0f32;
                            
                            // Title match (highest weight)
                            if article.title.to_lowercase().contains(&query_lower) {
                                score += 10.0;
                            }
                            
                            // Summary match
                            if article.summary.to_lowercase().contains(&query_lower) {
                                score += 5.0;
                            }
                            
                            // Content match (lower weight)
                            let content_lower = article.content.to_lowercase();
                            let occurrences = content_lower.matches(&query_lower).count();
                            score += occurrences as f32 * 0.5;
                            
                            if score > 0.0 {
                                scored_results.push((article, score));
                            }
                        }
                    }
                }
            }
            
            if scored_results.len() >= limit * 10 {
                break;  // Early exit for performance
            }
        }
        
        // Sort by relevance
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top results
        for (article, _score) in scored_results.into_iter().take(limit) {
            results.push(article);
        }
        
        Ok(results)
    }

    /// Import from Wikipedia XML dump (simplified)
    pub async fn import_wikipedia_dump(&mut self, dump_path: PathBuf) -> Result<()> {
        log::info!("[KnowledgeBase] Starting Wikipedia import from {:?}", dump_path);
        
        // TODO: Implement actual XML parsing
        // This would use the mediawiki-parser crate or similar
        // For now, this is a placeholder
        
        Err(anyhow!("Wikipedia XML import not yet implemented. Use add_article() for manual indexing."))
    }

    /// Get total article count
    pub fn article_count(&self) -> usize {
        self.article_count
    }

    /// Batch index multiple articles (optimized)
    pub fn batch_index(&mut self, articles: Vec<WikiArticle>) -> Result<usize> {
        let batch_size = articles.len();
        
        for article in articles {
            self.index_article(article)?;
        }
        
        // Flush to disk
        self.db.flush()?;
        
        log::info!("[KnowledgeBase] Batch indexed {} articles", batch_size);
        Ok(batch_size)
    }
}

/// Wikipedia article builder for easy construction
pub struct WikiArticleBuilder {
    title: String,
    content: String,
    categories: Vec<String>,
    summary: String,
    links: Vec<String>,
}

impl WikiArticleBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: String::new(),
            categories: Vec::new(),
            summary: String::new(),
            links: Vec::new(),
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        
        // Auto-extract summary (first paragraph)
        if self.summary.is_empty() {
            self.summary = self.content
                .split("\n\n")
                .next()
                .unwrap_or(&self.content)
                .chars()
                .take(500)
                .collect();
        }
        
        self
    }

    pub fn categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }

    pub fn add_category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    pub fn links(mut self, links: Vec<String>) -> Self {
        self.links = links;
        self
    }

    pub fn build(self) -> WikiArticle {
        WikiArticle {
            title: self.title,
            content: self.content,
            categories: self.categories,
            summary: self.summary,
            links: self.links,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_knowledge_base_operations() {
        let temp_dir = env::temp_dir().join("karana_kb_test");
        let mut kb = OfflineKnowledgeBase::new(temp_dir.clone()).unwrap();

        // Create test article
        let article = WikiArticleBuilder::new("Rust Programming")
            .content("Rust is a systems programming language. It is fast and memory-safe.")
            .add_category("Programming Languages")
            .add_category("Systems Programming")
            .build();

        // Index article
        kb.index_article(article.clone()).unwrap();
        assert_eq!(kb.article_count(), 1);

        // Retrieve article
        let retrieved = kb.get_article("Rust Programming").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Rust Programming");

        // Search by prefix
        let results = kb.search_by_prefix("Rust", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "Rust Programming");

        // Category search
        let cat_results = kb.get_category_articles("Programming Languages", 10).unwrap();
        assert_eq!(cat_results.len(), 1);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_article_builder() {
        let article = WikiArticleBuilder::new("Test Article")
            .content("First paragraph.\n\nSecond paragraph.")
            .add_category("Test")
            .build();

        assert_eq!(article.title, "Test Article");
        assert!(article.summary.contains("First paragraph"));
        assert_eq!(article.categories.len(), 1);
    }
}
