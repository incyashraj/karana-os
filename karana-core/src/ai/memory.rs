// Phase 54.3: Long-Term Memory & Feedback System
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub intent: String,
    pub response: String,
    pub confidence: f32,
    pub feedback_score: f32,
    pub timestamp: i64,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub recent_intents: Vec<String>,
    pub user_preferences: HashMap<String, Value>,
    pub location_history: Vec<String>,
    pub common_patterns: HashMap<String, f32>,  // intent -> frequency
}

pub struct OracleMemory {
    sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
    context: Arc<RwLock<ContextWindow>>,
    feedback_threshold: f32,
}

impl OracleMemory {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            context: Arc::new(RwLock::new(ContextWindow {
                recent_intents: Vec::new(),
                user_preferences: HashMap::new(),
                location_history: vec!["Paris".to_string()],  // Default
                common_patterns: HashMap::new(),
            })),
            feedback_threshold: 0.7,
        }
    }

    /// Store session with initial confidence
    pub fn store_session(&self, intent: &str, response: &str, confidence: f32, tools: Vec<String>) -> Result<()> {
        let mut sessions = self.sessions.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        let record = SessionRecord {
            intent: intent.to_string(),
            response: response.to_string(),
            confidence,
            feedback_score: 0.5, // Neutral initial
            timestamp: chrono::Utc::now().timestamp(),
            tools_used: tools,
        };

        sessions.insert(intent.to_string(), record);

        // Update context window
        let mut context = self.context.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        context.recent_intents.push(intent.to_string());
        if context.recent_intents.len() > 50 {
            context.recent_intents.remove(0);
        }

        // Update pattern frequency
        *context.common_patterns.entry(intent.to_string()).or_insert(0.0) += 1.0;

        Ok(())
    }

    /// Retrieve relevant context for current request
    pub fn retrieve_context(&self, intent: &str) -> Result<Vec<String>> {
        let sessions = self.sessions.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        let mut relevant = Vec::new();

        // Exact match
        if let Some(record) = sessions.get(intent) {
            if record.feedback_score > self.feedback_threshold {
                relevant.push(format!(
                    "Previous: {} → {} (score: {:.2})",
                    record.intent, record.response, record.feedback_score
                ));
            }
        }

        // Semantic similarity (simplified - in production use vector embeddings)
        for (key, record) in sessions.iter() {
            if key != intent && self.is_similar(intent, key) {
                if record.feedback_score > self.feedback_threshold {
                    relevant.push(format!(
                        "Related: {} → {} (score: {:.2})",
                        record.intent, record.response, record.feedback_score
                    ));
                }
            }
        }

        // Add common patterns
        let context = self.context.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(freq) = context.common_patterns.get(intent) {
            if *freq > 3.0 {
                relevant.push(format!("Frequent request ({}x)", freq));
            }
        }

        Ok(relevant)
    }

    /// Update feedback score (voice/haptic confirm)
    pub fn feedback_update(&self, intent: &str, helpful: bool) -> Result<()> {
        let mut sessions = self.sessions.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(record) = sessions.get_mut(intent) {
            // Update score with exponential moving average
            let delta = if helpful { 0.1 } else { -0.1 };
            record.feedback_score = (record.feedback_score + delta).clamp(0.0, 1.0);
        }

        Ok(())
    }

    /// Get enriched context for reasoning
    pub fn get_enriched_context(&self, intent: &str) -> Result<Value> {
        let historical = self.retrieve_context(intent)?;
        let context = self.context.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        Ok(json!({
            "historical_responses": historical,
            "recent_intents": context.recent_intents.iter().rev().take(10).collect::<Vec<_>>(),
            "location": context.location_history.last().unwrap_or(&"Unknown".to_string()),
            "user_preferences": context.user_preferences,
            "patterns": context.common_patterns.iter()
                .filter(|(_, freq)| **freq > 2.0)
                .map(|(k, v)| (k.clone(), *v))
                .collect::<HashMap<_, _>>(),
        }))
    }

    /// Simple similarity check (in production: use embeddings)
    fn is_similar(&self, a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        
        // Check for common keywords
        let keywords_a: Vec<&str> = a_lower.split_whitespace().collect();
        let keywords_b: Vec<&str> = b_lower.split_whitespace().collect();
        
        let common: usize = keywords_a.iter()
            .filter(|k| keywords_b.contains(k))
            .count();
        
        common >= 2 || (common >= 1 && keywords_a.len() <= 3)
    }

    /// Get analytics for self-improvement
    pub fn get_analytics(&self) -> Result<MemoryAnalytics> {
        let sessions = self.sessions.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        let total = sessions.len();
        let positive_feedback = sessions.values()
            .filter(|r| r.feedback_score > 0.7)
            .count();
        
        let avg_confidence: f32 = if total > 0 {
            sessions.values().map(|r| r.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };

        Ok(MemoryAnalytics {
            total_sessions: total,
            positive_feedback_count: positive_feedback,
            positive_feedback_rate: if total > 0 { positive_feedback as f32 / total as f32 } else { 0.0 },
            avg_confidence,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryAnalytics {
    pub total_sessions: usize,
    pub positive_feedback_count: usize,
    pub positive_feedback_rate: f32,
    pub avg_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_and_retrieval() {
        let memory = OracleMemory::new();
        
        memory.store_session(
            "weather Paris",
            "15°C, rain",
            0.85,
            vec!["web_api".to_string()]
        ).unwrap();

        memory.feedback_update("weather Paris", true).unwrap();
        
        let context = memory.retrieve_context("weather Paris").unwrap();
        assert!(!context.is_empty());
    }

    #[test]
    fn test_pattern_detection() {
        let memory = OracleMemory::new();
        
        for _ in 0..5 {
            memory.store_session(
                "umbrella needed?",
                "Yes, rain forecast",
                0.90,
                vec![]
            ).unwrap();
        }

        let enriched = memory.get_enriched_context("umbrella needed?").unwrap();
        let patterns = enriched["patterns"].as_object().unwrap();
        
        assert!(patterns.get("umbrella needed?").is_some());
    }
}
