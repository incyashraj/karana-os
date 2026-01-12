// Kāraṇa OS - ReAct Agent (Reasoning + Acting)
// Implements interleaved reasoning and tool execution for complex queries

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::KaranaAI;
use super::reasoning::ReasoningContext;
use super::agentic::{ToolRegistry, ToolOutput};

/// Maximum iterations to prevent infinite loops
const MAX_ITERATIONS: usize = 5;

/// Confidence threshold for early stopping
const CONFIDENCE_THRESHOLD: f32 = 0.85;

/// ReAct agent step combining thought, action, and observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub iteration: usize,
    pub thought: String,
    pub action: Option<ToolCall>,
    pub observation: Option<String>,
    pub confidence: f32,
}

/// Tool call specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: serde_json::Value,
}

/// Final agent response with reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub answer: String,
    pub chain: Vec<AgentStep>,
    pub total_confidence: f32,
    pub iterations_used: usize,
    pub sources: Vec<String>,
}

/// ReAct agent - combines reasoning with tool execution
pub struct ReActAgent {
    llm: Arc<Mutex<KaranaAI>>,
    tools: Arc<ToolRegistry>,
    max_iterations: usize,
    confidence_threshold: f32,
}

impl ReActAgent {
    pub fn new(llm: Arc<Mutex<KaranaAI>>, tools: Arc<ToolRegistry>) -> Self {
        Self {
            llm,
            tools,
            max_iterations: MAX_ITERATIONS,
            confidence_threshold: CONFIDENCE_THRESHOLD,
        }
    }
    
    /// Run the ReAct loop to answer a query
    pub async fn run(&self, query: &str, context: &ReasoningContext) -> Result<AgentResponse> {
        log::info!("[ReActAgent] Starting reasoning chain for: {}", query);
        
        let mut chain: Vec<AgentStep> = Vec::new();
        let mut accumulated_observations = String::new();
        
        for iteration in 0..self.max_iterations {
            log::debug!("[ReActAgent] Iteration {}/{}", iteration + 1, self.max_iterations);
            
            // 1. Generate thought and decide action
            let prompt = self.build_prompt(query, context, &chain, &accumulated_observations);
            let llm_response = {
                let mut llm = self.llm.lock().await;
                llm.predict(&prompt, 200)?
            };
            
            // 2. Parse LLM response
            let (thought, action) = self.parse_response(&llm_response)?;
            
            log::debug!("[ReActAgent] Thought: {}", thought);
            
            // 3. Execute tool if action specified
            let observation = if let Some(ref tool_call) = action {
                log::info!("[ReActAgent] Executing tool: {}", tool_call.tool_name);
                
                match self.tools.call_tool(&tool_call.tool_name, &tool_call.args).await {
                    Ok(result) => {
                        log::debug!("[ReActAgent] Tool result: {}", result);
                        accumulated_observations.push_str(&format!("\n{}: {}", tool_call.tool_name, result));
                        Some(result)
                    }
                    Err(e) => {
                        log::warn!("[ReActAgent] Tool execution failed: {}", e);
                        Some(format!("Tool execution failed: {}", e))
                    }
                }
            } else {
                None
            };
            
            // 4. Calculate confidence
            let confidence = self.calculate_confidence(&thought, &observation);
            
            // 5. Record step
            let step = AgentStep {
                iteration: iteration + 1,
                thought: thought.clone(),
                action: action.clone(),
                observation: observation.clone(),
                confidence,
            };
            chain.push(step);
            
            // 6. Check if we can answer (no action = final answer)
            if action.is_none() && confidence >= self.confidence_threshold {
                log::info!("[ReActAgent] Reached answer with confidence: {:.2}", confidence);
                
                return Ok(AgentResponse {
                    answer: thought,
                    chain: chain.clone(),
                    total_confidence: confidence,
                    iterations_used: iteration + 1,
                    sources: self.extract_sources(&chain),
                });
            }
        }
        
        // Max iterations reached - synthesize best answer
        log::warn!("[ReActAgent] Max iterations reached, synthesizing answer");
        
        let final_answer = self.synthesize_final_answer(&chain, &accumulated_observations).await?;
        let avg_confidence = chain.iter().map(|s| s.confidence).sum::<f32>() / chain.len() as f32;
        let sources = self.extract_sources(&chain);
        
        Ok(AgentResponse {
            answer: final_answer,
            chain,
            total_confidence: avg_confidence,
            iterations_used: self.max_iterations,
            sources,
        })
    }
    
    /// Build ReAct prompt with context and previous steps
    fn build_prompt(&self, query: &str, context: &ReasoningContext, chain: &[AgentStep], observations: &str) -> String {
        let mut prompt = String::new();
        
        // System instruction
        prompt.push_str("You are Kāraṇa, an AI assistant with access to tools. ");
        prompt.push_str("Answer questions by thinking step-by-step and using tools when needed.\n\n");
        
        // Available tools
        prompt.push_str("Available tools:\n");
        for tool_name in self.tools.list_tools() {
            prompt.push_str(&format!("- {} - Use for relevant queries\n", tool_name));
        }
        prompt.push_str("\n");
        
        // Format instructions
        prompt.push_str("Response format:\n");
        prompt.push_str("Thought: [your reasoning about what to do next]\n");
        prompt.push_str("Action: [tool_name(args)] OR None if you can answer directly\n");
        prompt.push_str("If Action is None, your Thought should be the final answer.\n\n");
        
        // Context
        if !context.conversation_history.is_empty() {
            prompt.push_str("Recent conversation:\n");
            for msg in context.conversation_history.iter().rev().take(3).rev() {
                prompt.push_str(&format!("- {}\n", msg));
            }
            prompt.push_str("\n");
        }
        
        // Question
        prompt.push_str(&format!("Question: {}\n\n", query));
        
        // Previous steps
        if !chain.is_empty() {
            prompt.push_str("Previous steps:\n");
            for step in chain {
                prompt.push_str(&format!("Thought: {}\n", step.thought));
                if let Some(ref action) = step.action {
                    prompt.push_str(&format!("Action: {}\n", action.tool_name));
                } else {
                    prompt.push_str("Action: None\n");
                }
                if let Some(ref obs) = step.observation {
                    prompt.push_str(&format!("Observation: {}\n", obs));
                }
                prompt.push_str("\n");
            }
        }
        
        // Accumulated knowledge
        if !observations.is_empty() {
            prompt.push_str("What we know so far:\n");
            prompt.push_str(observations);
            prompt.push_str("\n\n");
        }
        
        // Next step
        prompt.push_str("Thought:");
        
        prompt
    }
    
    /// Parse LLM response into thought and action
    fn parse_response(&self, response: &str) -> Result<(String, Option<ToolCall>)> {
        let lines: Vec<&str> = response.lines().collect();
        
        let mut thought = String::new();
        let mut action_line = None;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.starts_with("Thought:") {
                thought = trimmed.strip_prefix("Thought:").unwrap().trim().to_string();
            } else if trimmed.starts_with("Action:") {
                action_line = Some(trimmed.strip_prefix("Action:").unwrap().trim().to_string());
            } else if !thought.is_empty() && !trimmed.is_empty() && action_line.is_none() {
                // Continue thought if no action yet
                thought.push(' ');
                thought.push_str(trimmed);
            }
        }
        
        // If no explicit thought found, use full response
        if thought.is_empty() {
            thought = response.trim().to_string();
        }
        
        // Parse action
        let action = if let Some(action_str) = action_line {
            if action_str.to_lowercase() == "none" || action_str.is_empty() {
                None
            } else {
                // Parse tool call: "tool_name(args)"
                self.parse_tool_call(&action_str)?
            }
        } else {
            None
        };
        
        Ok((thought, action))
    }
    
    /// Parse tool call string
    fn parse_tool_call(&self, action_str: &str) -> Result<Option<ToolCall>> {
        // Simple parsing: "tool_name(arg1, arg2)" or just "tool_name"
        
        if let Some(paren_pos) = action_str.find('(') {
            let tool_name = action_str[..paren_pos].trim().to_string();
            
            // Extract arguments (simple implementation)
            let args_str = action_str[paren_pos + 1..].trim_end_matches(')').trim();
            
            // For now, treat all args as a single string parameter
            let args = if args_str.is_empty() {
                serde_json::json!({})
            } else {
                // Try to parse as JSON, fallback to simple string
                if let Ok(json_args) = serde_json::from_str::<serde_json::Value>(args_str) {
                    json_args
                } else {
                    // Treat as expression for calculator or location for weather
                    if tool_name == "calculator" {
                        serde_json::json!({ "expression": args_str })
                    } else if tool_name == "weather" {
                        serde_json::json!({ "location": args_str })
                    } else {
                        serde_json::json!({ "query": args_str })
                    }
                }
            };
            
            Ok(Some(ToolCall { tool_name, args }))
        } else {
            // Just tool name, no args
            let tool_name = action_str.trim().to_string();
            Ok(Some(ToolCall {
                tool_name,
                args: serde_json::json!({}),
            }))
        }
    }
    
    /// Calculate confidence based on thought and observation
    fn calculate_confidence(&self, thought: &str, observation: &Option<String>) -> f32 {
        let mut confidence: f32 = 0.5;
        
        // Higher confidence if we have an observation (tool executed successfully)
        if observation.is_some() {
            confidence += 0.2;
        }
        
        // Lower confidence for uncertain language
        if thought.to_lowercase().contains("might") 
            || thought.to_lowercase().contains("maybe") 
            || thought.to_lowercase().contains("possibly") {
            confidence -= 0.1;
        }
        
        // Higher confidence for definitive language
        if thought.to_lowercase().contains("confirmed") 
            || thought.to_lowercase().contains("verified") 
            || thought.to_lowercase().contains("determined") {
            confidence += 0.15;
        }
        
        confidence.clamp(0.0, 1.0)
    }
    
    /// Extract sources from agent steps
    fn extract_sources(&self, chain: &[AgentStep]) -> Vec<String> {
        chain.iter()
            .filter_map(|step| {
                step.action.as_ref().map(|action| action.tool_name.clone())
            })
            .collect()
    }
    
    /// Synthesize final answer from accumulated evidence
    async fn synthesize_final_answer(&self, chain: &[AgentStep], observations: &str) -> Result<String> {
        let mut llm = self.llm.lock().await;
        
        let prompt = format!(
            "Based on the following reasoning chain, provide a concise final answer:\n\n{}\n\nFinal answer:",
            observations
        );
        
        llm.predict(&prompt, 100)
    }
}

/* Tests disabled - need mock implementations
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tool_call() {
        let agent = ReActAgent {
            llm: Arc::new(Mutex::new(KaranaAI::new_mock())),
            tools: Arc::new(ToolRegistry::new()),
            max_iterations: 3,
            confidence_threshold: 0.8,
        };
        
        // Test with arguments
        let call = agent.parse_tool_call("calculator(15 + 25)").unwrap().unwrap();
        assert_eq!(call.tool_name, "calculator");
        
        // Test without arguments
        let call = agent.parse_tool_call("weather").unwrap();
        assert!(call.is_some());
    }
    
    #[test]
    fn test_parse_response() {
        let agent = ReActAgent {
            llm: Arc::new(Mutex::new(KaranaAI::new_mock())),
            tools: Arc::new(ToolRegistry::new()),
            max_iterations: 3,
            confidence_threshold: 0.8,
        };
        
        let response = "Thought: I need to calculate the sum\nAction: calculator(15 + 25)";
        let (thought, action) = agent.parse_response(response).unwrap();
        
        assert!(thought.contains("calculate"));
        assert!(action.is_some());
    }
    
    #[test]
    fn test_confidence_calculation() {
        let agent = ReActAgent {
            llm: Arc::new(Mutex::new(KaranaAI::new_mock())),
            tools: Arc::new(ToolRegistry::new()),
            max_iterations: 3,
            confidence_threshold: 0.8,
        };
        
        // With observation
        let conf = agent.calculate_confidence("The answer is clear", &Some("Result: 42".to_string()));
        assert!(conf > 0.6);
        
        // Uncertain language
        let conf = agent.calculate_confidence("Maybe the answer is 42", &None);
        assert!(conf < 0.5);
    }
}
*/
