// Kāraṇa OS - Advanced Reasoning Engine
// Phase 2: Chain-of-Thought and Multi-Step Planning

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::SystemTime;

use super::KaranaAI;

/// Reasoning context for multi-step queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningContext {
    pub user_query: String,
    pub conversation_history: Vec<String>,
    pub current_location: Option<String>,
    pub current_time: u64,
    pub available_tools: Vec<String>,
    pub user_preferences: Vec<(String, String)>,
}

impl ReasoningContext {
    pub fn new(query: &str) -> Self {
        Self {
            user_query: query.to_string(),
            conversation_history: Vec::new(),
            current_location: None,
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            available_tools: vec![
                "web_search".to_string(),
                "calculator".to_string(),
                "memory_recall".to_string(),
                "file_search".to_string(),
                "blockchain_query".to_string(),
                "vision_analysis".to_string(),
            ],
            user_preferences: Vec::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut context = format!("Current query: {}", self.user_query);
        
        if !self.conversation_history.is_empty() {
            context.push_str(&format!(
                "\nRecent conversation: {}",
                self.conversation_history.last().unwrap()
            ));
        }
        
        if let Some(loc) = &self.current_location {
            context.push_str(&format!("\nLocation: {}", loc));
        }
        
        context
    }
}

/// Single reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_number: usize,
    pub description: String,
    pub thought_process: String,
    pub tool_used: Option<String>,
    pub result: String,
    pub confidence: f32,
}

/// Complete reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub query: String,
    pub steps: Vec<ReasoningStep>,
    pub final_answer: Option<String>,
    pub total_confidence: f32,
    pub sources: Vec<String>,
}

impl ReasoningChain {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            steps: Vec::new(),
            final_answer: None,
            total_confidence: 0.0,
            sources: Vec::new(),
        }
    }

    pub fn add_step(&mut self, description: &str, thought: &str) {
        self.steps.push(ReasoningStep {
            step_number: self.steps.len() + 1,
            description: description.to_string(),
            thought_process: thought.to_string(),
            tool_used: None,
            result: String::new(),
            confidence: 0.8,
        });
    }

    pub fn add_step_with_tool(&mut self, description: &str, thought: &str, tool: &str, result: &str) {
        self.steps.push(ReasoningStep {
            step_number: self.steps.len() + 1,
            description: description.to_string(),
            thought_process: thought.to_string(),
            tool_used: Some(tool.to_string()),
            result: result.to_string(),
            confidence: 0.85,
        });
        self.sources.push(tool.to_string());
    }

    pub fn set_final_answer(&mut self, answer: String) {
        self.final_answer = Some(answer);
        // Calculate average confidence
        if !self.steps.is_empty() {
            self.total_confidence = self.steps.iter()
                .map(|s| s.confidence)
                .sum::<f32>() / self.steps.len() as f32;
        }
    }
}

/// Query decomposition result
#[derive(Debug, Clone)]
pub struct QueryDecomposition {
    pub original_query: String,
    pub sub_queries: Vec<String>,
    pub query_type: QueryType,
    pub requires_tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    Simple,           // Single-step answer
    MultiStep,        // Needs multiple reasoning steps
    Computational,    // Needs calculator
    KnowledgeBased,   // Needs information retrieval
    ActionBased,      // Needs to execute actions
    Conversational,   // Casual chat
}

/// Chain-of-Thought Reasoner
pub struct ChainOfThoughtReasoner {
    ai: Arc<StdMutex<KaranaAI>>,
    max_steps: usize,
}

impl ChainOfThoughtReasoner {
    pub fn new(ai: Arc<StdMutex<KaranaAI>>) -> Self {
        Self {
            ai,
            max_steps: 10,
        }
    }

    /// Main reasoning entry point
    pub fn reason(&self, query: &str, context: &ReasoningContext) -> Result<ReasoningChain> {
        log::info!("[Reasoning] Starting chain-of-thought for: {}", query);
        
        let mut chain = ReasoningChain::new(query);
        
        // Step 1: Decompose the query
        let decomposition = self.decompose_query(query)?;
        chain.add_step(
            "Query Analysis",
            &format!("Identified as {:?} query with {} sub-questions", 
                decomposition.query_type, decomposition.sub_queries.len())
        );
        
        // Step 2: Handle based on query type
        match decomposition.query_type {
            QueryType::Simple => {
                self.handle_simple_query(&mut chain, query, context)?;
            }
            QueryType::MultiStep => {
                self.handle_multistep_query(&mut chain, &decomposition, context)?;
            }
            QueryType::Computational => {
                self.handle_computational_query(&mut chain, query)?;
            }
            QueryType::KnowledgeBased => {
                self.handle_knowledge_query(&mut chain, query, context)?;
            }
            QueryType::ActionBased => {
                self.handle_action_query(&mut chain, query, context)?;
            }
            QueryType::Conversational => {
                self.handle_conversational_query(&mut chain, query, context)?;
            }
        }
        
        // Step 3: Synthesize final answer
        let final_answer = self.synthesize_chain(&chain)?;
        chain.set_final_answer(final_answer);
        
        log::info!("[Reasoning] ✓ Completed with {} steps, confidence: {:.0}%", 
            chain.steps.len(), chain.total_confidence * 100.0);
        
        Ok(chain)
    }

    /// Decompose complex query into sub-queries
    fn decompose_query(&self, query: &str) -> Result<QueryDecomposition> {
        let lower = query.to_lowercase();
        
        // Detect query type
        let query_type = if lower.contains("and then") || lower.contains("after that") {
            QueryType::MultiStep
        } else if lower.contains("calculate") || lower.contains("how many") || 
                  lower.contains("what's") && (lower.contains("+") || lower.contains("*")) {
            QueryType::Computational
        } else if lower.contains("what is") || lower.contains("explain") || 
                  lower.contains("tell me about") {
            QueryType::KnowledgeBased
        } else if lower.contains("open") || lower.contains("send") || 
                  lower.contains("create") || lower.contains("delete") {
            QueryType::ActionBased
        } else if lower.contains("hello") || lower.contains("how are you") ||
                  lower.contains("thanks") || lower.contains("bye") {
            QueryType::Conversational
        } else {
            QueryType::Simple
        };
        
        // Split multi-step queries
        let sub_queries = if query_type == QueryType::MultiStep {
            query.split(" and then ")
                .chain(query.split(" then "))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![query.to_string()]
        };
        
        // Determine required tools
        let requires_tools = self.identify_required_tools(query);
        
        Ok(QueryDecomposition {
            original_query: query.to_string(),
            sub_queries,
            query_type,
            requires_tools,
        })
    }

    fn identify_required_tools(&self, query: &str) -> Vec<String> {
        let mut tools = Vec::new();
        let lower = query.to_lowercase();
        
        if lower.contains("search") || lower.contains("find") || lower.contains("look up") {
            tools.push("web_search".to_string());
        }
        if lower.contains("calculate") || lower.contains("compute") {
            tools.push("calculator".to_string());
        }
        if lower.contains("remember") || lower.contains("recall") || lower.contains("last time") {
            tools.push("memory_recall".to_string());
        }
        if lower.contains("file") || lower.contains("document") {
            tools.push("file_search".to_string());
        }
        if lower.contains("balance") || lower.contains("transaction") || lower.contains("blockchain") {
            tools.push("blockchain_query".to_string());
        }
        if lower.contains("what am i looking at") || lower.contains("describe this") {
            tools.push("vision_analysis".to_string());
        }
        
        tools
    }

    fn handle_simple_query(&self, chain: &mut ReasoningChain, query: &str, context: &ReasoningContext) -> Result<()> {
        let prompt = format!(
            "Context: {}\nQuestion: {}\n\nProvide a brief, direct answer:",
            context.to_string(),
            query
        );
        
        let mut ai = self.ai.lock().unwrap();
        let answer = ai.predict(&prompt, 100)?;
        
        chain.add_step("Direct Answer", &answer);
        Ok(())
    }

    fn handle_multistep_query(&self, chain: &mut ReasoningChain, decomposition: &QueryDecomposition, context: &ReasoningContext) -> Result<()> {
        for (i, sub_query) in decomposition.sub_queries.iter().enumerate() {
            let step_result = self.reason_step(sub_query, context)?;
            chain.add_step(
                &format!("Step {}: {}", i + 1, sub_query),
                &step_result
            );
        }
        Ok(())
    }

    fn handle_computational_query(&self, chain: &mut ReasoningChain, query: &str) -> Result<()> {
        // Simple calculator (would use real calculator tool in production)
        let result = self.extract_and_calculate(query)?;
        chain.add_step_with_tool(
            "Calculation",
            &format!("Computing: {}", query),
            "calculator",
            &result
        );
        Ok(())
    }

    fn handle_knowledge_query(&self, chain: &mut ReasoningChain, query: &str, context: &ReasoningContext) -> Result<()> {
        // Would use RAG/web search in production
        let prompt = format!(
            "Explain concisely: {}\nContext: {}",
            query,
            context.to_string()
        );
        
        let mut ai = self.ai.lock().unwrap();
        let answer = ai.predict(&prompt, 200)?;
        
        chain.add_step_with_tool(
            "Knowledge Retrieval",
            "Searching knowledge base...",
            "local_rag",
            &answer
        );
        Ok(())
    }

    fn handle_action_query(&self, chain: &mut ReasoningChain, query: &str, _context: &ReasoningContext) -> Result<()> {
        chain.add_step(
            "Action Planning",
            &format!("Preparing to execute: {}", query)
        );
        Ok(())
    }

    fn handle_conversational_query(&self, chain: &mut ReasoningChain, query: &str, _context: &ReasoningContext) -> Result<()> {
        let mut ai = self.ai.lock().unwrap();
        let response = ai.predict(&format!("Respond naturally to: {}", query), 80)?;
        chain.add_step("Conversation", &response);
        Ok(())
    }

    fn reason_step(&self, query: &str, context: &ReasoningContext) -> Result<String> {
        let prompt = format!(
            "Reasoning step:\nContext: {}\nQuery: {}\n\nThought process:",
            context.to_string(),
            query
        );
        
        let mut ai = self.ai.lock().unwrap();
        ai.predict(&prompt, 100)
    }

    fn extract_and_calculate(&self, query: &str) -> Result<String> {
        // Simple math extraction (would use proper calculator in production)
        let lower = query.to_lowercase();
        
        if lower.contains("what") && lower.contains("plus") {
            return Ok("42 (calculation result)".to_string());
        }
        
        Ok("Calculation complete".to_string())
    }

    fn synthesize_chain(&self, chain: &ReasoningChain) -> Result<String> {
        if chain.steps.is_empty() {
            return Ok("I couldn't process that query.".to_string());
        }
        
        // Build summary from all steps
        let thoughts = chain.steps.iter()
            .map(|s| s.thought_process.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        let prompt = format!(
            "Synthesize a final answer from these reasoning steps:\n\n{}\n\nFinal answer:",
            thoughts
        );
        
        let mut ai = self.ai.lock().unwrap();
        ai.predict(&prompt, 150)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_decomposition() {
        let ai = Arc::new(StdMutex::new(KaranaAI::new().unwrap()));
        let reasoner = ChainOfThoughtReasoner::new(ai);
        
        let decomp = reasoner.decompose_query("What is Bitcoin and then how do I buy it").unwrap();
        assert_eq!(decomp.query_type, QueryType::MultiStep);
        assert!(decomp.sub_queries.len() > 1);
    }

    #[test]
    fn test_tool_identification() {
        let ai = Arc::new(StdMutex::new(KaranaAI::new().unwrap()));
        let reasoner = ChainOfThoughtReasoner::new(ai);
        
        let tools = reasoner.identify_required_tools("search for Bitcoin price");
        assert!(tools.contains(&"web_search".to_string()));
    }
}
