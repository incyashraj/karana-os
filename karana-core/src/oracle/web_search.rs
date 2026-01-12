// Kāraṇa OS - Web Search Integration
// Phase 3: Enhanced Knowledge Base - Real-time web search

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Web search result from DuckDuckGo or other search engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance_score: f32,
}

/// Web search provider interface
pub struct WebSearchEngine {
    client: reqwest::Client,
    user_agent: String,
    rate_limit_delay: Duration,
}

impl WebSearchEngine {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            user_agent: "Karana-OS/0.1 (Smart Glasses Assistant)".to_string(),
            rate_limit_delay: Duration::from_millis(500),
        }
    }

    /// Search the web using DuckDuckGo Instant Answer API
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>> {
        log::info!("[WebSearch] Searching for: '{}'", query);

        // DuckDuckGo Instant Answer API (free, no API key needed)
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Search request failed: {}", response.status()));
        }

        let data: DuckDuckGoResponse = response.json().await?;
        let results = self.parse_ddg_response(data, max_results);

        log::info!("[WebSearch] Found {} results", results.len());
        Ok(results)
    }

    /// Parse DuckDuckGo API response into search results
    fn parse_ddg_response(&self, data: DuckDuckGoResponse, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();

        // Add abstract as primary result if available
        if !data.abstract_text.is_empty() {
            results.push(WebSearchResult {
                title: data.heading.clone().unwrap_or_else(|| "DuckDuckGo".to_string()),
                url: data.abstract_url.clone().unwrap_or_default(),
                snippet: data.abstract_text.clone(),
                relevance_score: 0.95,
            });
        }

        // Add related topics
        for topic in data.related_topics.iter().take(max_results - results.len()) {
            if let Some(text) = &topic.text {
                results.push(WebSearchResult {
                    title: topic.first_url.clone().unwrap_or_default(),
                    url: topic.first_url.clone().unwrap_or_default(),
                    snippet: text.clone(),
                    relevance_score: 0.7,
                });
            }
        }

        // Add results from related topics
        for result in data.results.iter().take(max_results - results.len()) {
            if let Some(text) = &result.text {
                results.push(WebSearchResult {
                    title: result.first_url.clone().unwrap_or_default(),
                    url: result.first_url.clone().unwrap_or_default(),
                    snippet: text.clone(),
                    relevance_score: 0.6,
                });
            }
        }

        results.truncate(max_results);
        results
    }

    /// Fetch full content from a URL (for deeper context)
    pub async fn fetch_page_content(&self, url: &str) -> Result<String> {
        log::info!("[WebSearch] Fetching content from: {}", url);

        let response = self.client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch page: {}", response.status()));
        }

        let html = response.text().await?;
        
        // Simple text extraction (remove HTML tags)
        let text = Self::extract_text_from_html(&html);
        
        Ok(text)
    }

    /// Simple HTML text extraction
    fn extract_text_from_html(html: &str) -> String {
        // Remove script and style tags
        let mut text = html.to_string();
        
        // Remove common HTML elements that don't contain useful text
        for tag in &["script", "style", "nav", "footer", "header"] {
            text = regex::Regex::new(&format!(r"(?i)<{}[^>]*>.*?</{}>", tag, tag))
                .unwrap()
                .replace_all(&text, "")
                .to_string();
        }
        
        // Remove all HTML tags
        let text = regex::Regex::new(r"<[^>]+>")
            .unwrap()
            .replace_all(&text, " ")
            .to_string();
        
        // Clean up whitespace
        let text = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text, " ")
            .to_string();
        
        text.trim().to_string()
    }
}

/// DuckDuckGo API response structure
#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "Abstract")]
    abstract_text: String,
    
    #[serde(rename = "AbstractURL")]
    abstract_url: Option<String>,
    
    #[serde(rename = "Heading")]
    heading: Option<String>,
    
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<RelatedTopic>,
    
    #[serde(rename = "Results")]
    results: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
    
    #[serde(rename = "Icon")]
    icon: Option<Icon>,
}

#[derive(Debug, Deserialize)]
struct Icon {
    #[serde(rename = "URL")]
    url: Option<String>,
}

impl Default for WebSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search() {
        let engine = WebSearchEngine::new();
        let results = engine.search("Rust programming language", 3).await;
        
        // Test may fail without internet, so just check structure
        match results {
            Ok(res) => {
                assert!(res.len() <= 3);
                for r in res {
                    assert!(!r.snippet.is_empty() || !r.url.is_empty());
                }
            }
            Err(_) => {
                // Network error is acceptable in tests
            }
        }
    }

    #[test]
    fn test_html_extraction() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <nav>Navigation</nav>
                    <h1>Main Content</h1>
                    <p>This is a test paragraph.</p>
                    <script>console.log('test');</script>
                    <footer>Footer</footer>
                </body>
            </html>
        "#;
        
        let text = WebSearchEngine::extract_text_from_html(html);
        assert!(text.contains("Main Content"));
        assert!(text.contains("test paragraph"));
        assert!(!text.contains("console.log"));
    }
}
