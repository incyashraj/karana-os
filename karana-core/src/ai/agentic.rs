// Phase 54.1: Agentic Reasoning - Multi-Step Analysis & Tool Chaining
// Phase 6: Real Tool Ecosystem with production implementations
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

// Tool trait for extensible tool system
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    async fn execute(&self, args: &ToolArgs) -> Result<ToolOutput>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ToolArgs {
    args: HashMap<String, Value>,
}

impl ToolArgs {
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: String, value: Value) {
        self.args.insert(key, value);
    }
    
    pub fn get_string(&self, key: &str) -> Result<String> {
        self.args.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Missing or invalid argument: {}", key))
    }
    
    pub fn get_f64(&self, key: &str) -> Result<f64> {
        self.args.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid number argument: {}", key))
    }
    
    pub fn get_optional_string(&self, key: &str) -> Option<String> {
        self.args.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// Production tool registry with real implementations
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register core tools
        registry.register_tool(Arc::new(CalculatorTool));
        registry.register_tool(Arc::new(WeatherTool::new()));
        registry.register_tool(Arc::new(WikipediaTool::new()));
        registry.register_tool(Arc::new(WebSearchTool::new()));
        
        registry
    }
    
    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
    
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    pub async fn call_tool(&self, tool_name: &str, input: &Value) -> Result<String> {
        let tool = self.get_tool(tool_name)
            .ok_or_else(|| anyhow!("Tool not found: {}", tool_name))?;
        
        // Parse input into ToolArgs
        let mut args = ToolArgs::new();
        if let Some(obj) = input.as_object() {
            for (key, value) in obj {
                args.insert(key.clone(), value.clone());
            }
        }
        
        let output = tool.execute(&args).await?;
        Ok(output.output)
    }
}

// ============================================================================
// TOOL OUTPUT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool_name: String,
    pub output: String,
    pub confidence: f32,
}

// ============================================================================
// TOOL IMPLEMENTATIONS
// ============================================================================

/// Calculator tool - deterministic math evaluation
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }
    
    fn description(&self) -> &str {
        "Evaluate mathematical expressions. Supports +, -, *, /, %, ^, sqrt, sin, cos, tan, log, ln. Examples: '15 + 25', '42 * 7', 'sqrt(144)', '15% of 200'"
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                required: true,
            }
        ]
    }
    
    async fn execute(&self, args: &ToolArgs) -> Result<ToolOutput> {
        let mut expr = args.get_string("expression")?;
        
        // Handle percentage calculations
        if expr.contains('%') {
            expr = self.parse_percentage(&expr)?;
        }
        
        // Handle "X of Y" pattern (e.g., "15% of 200")
        if expr.to_lowercase().contains(" of ") {
            expr = self.parse_of_pattern(&expr)?;
        }
        
        // Use meval for safe math evaluation
        let result = meval::eval_str(&expr)
            .map_err(|e| anyhow!("Math evaluation error: {}", e))?;
        
        Ok(ToolOutput {
            tool_name: "calculator".to_string(),
            output: result.to_string(),
            confidence: 1.0,  // Deterministic
        })
    }
}

impl CalculatorTool {
    fn parse_percentage(&self, expr: &str) -> Result<String> {
        // Convert "15%" to "0.15"
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)%").unwrap();
        let result = re.replace_all(expr, |caps: &regex::Captures| {
            let num: f64 = caps[1].parse().unwrap();
            format!("{}", num / 100.0)
        });
        Ok(result.to_string())
    }
    
    fn parse_of_pattern(&self, expr: &str) -> Result<String> {
        // Convert "15% of 200" to "0.15 * 200"
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)%\s+of\s+(\d+(?:\.\d+)?)").unwrap();
        if let Some(caps) = re.captures(expr) {
            let percent: f64 = caps[1].parse()?;
            let value: f64 = caps[2].parse()?;
            return Ok(format!("{}", (percent / 100.0) * value));
        }
        
        // Convert "X of Y" to "X * Y"
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s+of\s+(\d+(?:\.\d+)?)").unwrap();
        if let Some(caps) = re.captures(expr) {
            let x: f64 = caps[1].parse()?;
            let y: f64 = caps[2].parse()?;
            return Ok(format!("{}", x * y));
        }
        
        Ok(expr.to_string())
    }
}

/// Weather tool - fetches current weather data
pub struct WeatherTool {
    cache: std::sync::Mutex<HashMap<String, (String, std::time::Instant)>>,
}

impl WeatherTool {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }
    
    fn description(&self) -> &str {
        "Get current weather for a location. Returns temperature, conditions, and precipitation. Use 'current' for user's current location."
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "location".to_string(),
                param_type: "string".to_string(),
                description: "City name or 'current' for current location".to_string(),
                required: true,
            }
        ]
    }
    
    async fn execute(&self, args: &ToolArgs) -> Result<ToolOutput> {
        let location = args.get_string("location")?;
        
        // Check cache (5 minute TTL)
        {
            let cache = self.cache.lock().unwrap();
            if let Some((cached, timestamp)) = cache.get(&location) {
                if timestamp.elapsed().as_secs() < 300 {
                    log::debug!("[WeatherTool] Cache hit for: {}", location);
                    return Ok(ToolOutput {
                        tool_name: "weather".to_string(),
                        output: cached.clone(),
                        confidence: 0.9,
                    });
                }
            }
        }
        
        // Fetch from wttr.in (free, no API key required)
        let actual_location = if location.to_lowercase() == "current" {
            ""  // wttr.in uses IP geolocation for empty location
        } else {
            &location
        };
        
        let url = format!("https://wttr.in/{}?format=j1", urlencoding::encode(actual_location));
        
        let response = reqwest::get(&url)
            .await
            .context("Failed to fetch weather data")?
            .text()
            .await?;
        
        let data: Value = serde_json::from_str(&response)
            .context("Failed to parse weather JSON")?;
        
        // Extract relevant information
        let current = &data["current_condition"][0];
        let temp_c = current["temp_C"].as_str().unwrap_or("?");
        let temp_f = current["temp_F"].as_str().unwrap_or("?");
        let desc = current["weatherDesc"][0]["value"].as_str().unwrap_or("Unknown");
        let precip_mm = current["precipMM"].as_str().unwrap_or("0");
        let humidity = current["humidity"].as_str().unwrap_or("?");
        let wind_kmph = current["windspeedKmph"].as_str().unwrap_or("?");
        
        // Get location name from response
        let loc_name = data["nearest_area"][0]["areaName"][0]["value"]
            .as_str()
            .unwrap_or(&location);
        
        let summary = format!(
            "{}: {}°C ({}°F), {}. Precipitation: {}mm, Humidity: {}%, Wind: {} km/h",
            loc_name, temp_c, temp_f, desc, precip_mm, humidity, wind_kmph
        );
        
        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(location.clone(), (summary.clone(), std::time::Instant::now()));
        }
        
        Ok(ToolOutput {
            tool_name: "weather".to_string(),
            output: summary,
            confidence: 0.9,
        })
    }
}

/// Wikipedia tool - searches offline knowledge base
pub struct WikipediaTool {
    cache: std::sync::Mutex<HashMap<String, (String, std::time::Instant)>>,
}

impl WikipediaTool {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Tool for WikipediaTool {
    fn name(&self) -> &str {
        "wikipedia"
    }
    
    fn description(&self) -> &str {
        "Search Wikipedia for factual information. Returns article summaries for people, places, concepts, history, science, etc."
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "topic".to_string(),
                param_type: "string".to_string(),
                description: "Topic or article name to search for".to_string(),
                required: true,
            }
        ]
    }
    
    async fn execute(&self, args: &ToolArgs) -> Result<ToolOutput> {
        let topic = args.get_string("topic")?;
        
        // Check cache (1 hour TTL - Wikipedia is fairly static)
        {
            let cache = self.cache.lock().unwrap();
            if let Some((cached, timestamp)) = cache.get(&topic) {
                if timestamp.elapsed().as_secs() < 3600 {
                    log::debug!("[WikipediaTool] Cache hit for: {}", topic);
                    return Ok(ToolOutput {
                        tool_name: "wikipedia".to_string(),
                        output: cached.clone(),
                        confidence: 0.95,
                    });
                }
            }
        }
        
        // Use Wikipedia API for summary
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(&topic)
        );
        
        let response = reqwest::get(&url)
            .await
            .context("Failed to fetch Wikipedia data")?;
        
        if !response.status().is_success() {
            return Ok(ToolOutput {
                tool_name: "wikipedia".to_string(),
                output: format!("No Wikipedia article found for '{}'", topic),
                confidence: 0.3,
            });
        }
        
        let data: serde_json::Value = response.json().await?;
        
        // Extract summary
        let title = data["title"].as_str().unwrap_or(&topic);
        let extract = data["extract"].as_str().unwrap_or("No summary available");
        
        // Get first 500 characters for concise response
        let summary = if extract.len() > 500 {
            format!("{}...", &extract[..500])
        } else {
            extract.to_string()
        };
        
        let result = format!("{}: {}", title, summary);
        
        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(topic.clone(), (result.clone(), std::time::Instant::now()));
        }
        
        Ok(ToolOutput {
            tool_name: "wikipedia".to_string(),
            output: result,
            confidence: 0.95,
        })
    }
}

/// Web search tool - searches the web for current information
pub struct WebSearchTool {
    cache: std::sync::Mutex<HashMap<String, (String, std::time::Instant)>>,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    
    fn description(&self) -> &str {
        "Search the web for current information, news, or topics not in Wikipedia. Use for recent events, prices, reviews, etc."
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Search query".to_string(),
                required: true,
            }
        ]
    }
    
    async fn execute(&self, args: &ToolArgs) -> Result<ToolOutput> {
        let query = args.get_string("query")?;
        
        // Check cache (15 minute TTL - web content changes frequently)
        {
            let cache = self.cache.lock().unwrap();
            if let Some((cached, timestamp)) = cache.get(&query) {
                if timestamp.elapsed().as_secs() < 900 {
                    log::debug!("[WebSearchTool] Cache hit for: {}", query);
                    return Ok(ToolOutput {
                        tool_name: "web_search".to_string(),
                        output: cached.clone(),
                        confidence: 0.85,
                    });
                }
            }
        }
        
        // Use DuckDuckGo Instant Answer API
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(&query)
        );
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        
        let response = client.get(&url)
            .header("User-Agent", "Karana-OS/0.1")
            .send()
            .await
            .context("Failed to fetch search results")?;
        
        let data: serde_json::Value = response.json().await?;
        
        // Extract relevant information
        let mut results = Vec::new();
        
        // Try Abstract first (general answer)
        if let Some(abstract_text) = data["Abstract"].as_str() {
            if !abstract_text.is_empty() {
                results.push(abstract_text.to_string());
            }
        }
        
        // Try Answer (instant answer)
        if let Some(answer) = data["Answer"].as_str() {
            if !answer.is_empty() {
                results.push(answer.to_string());
            }
        }
        
        // Try Definition
        if let Some(definition) = data["Definition"].as_str() {
            if !definition.is_empty() {
                results.push(definition.to_string());
            }
        }
        
        // Try Related Topics
        if let Some(topics) = data["RelatedTopics"].as_array() {
            for topic in topics.iter().take(3) {
                if let Some(text) = topic["Text"].as_str() {
                    if !text.is_empty() {
                        results.push(text.to_string());
                    }
                }
            }
        }
        
        let summary = if results.is_empty() {
            format!("No instant results found for '{}'. Try being more specific.", query)
        } else {
            results.join(" | ")
        };
        
        // Truncate if too long
        let summary = if summary.len() > 800 {
            format!("{}...", &summary[..800])
        } else {
            summary
        };
        
        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(query.clone(), (summary.clone(), std::time::Instant::now()));
        }
        
        Ok(ToolOutput {
            tool_name: "web_search".to_string(),
            output: summary,
            confidence: 0.80,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgenticResponse {
    pub suggestion: String,
    pub chain: Vec<ToolOutput>,
    pub confidence: f32,
    pub reasoning_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPlan {
    pub classification: String,  // os/general/app/creative/random
    pub tools_needed: Vec<String>,
    pub chain_plan: Vec<ChainStep>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub step_id: usize,
    pub tool: String,
    pub input_source: InputSource,  // direct | previous_output
    pub input_data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Direct,
    PreviousOutput(usize),  // Reference step_id
}

pub struct AgenticReasoner {
    tool_registry: ToolRegistry,
    max_chain_depth: usize,
}

impl AgenticReasoner {
    pub fn new(tool_registry: ToolRegistry) -> Self {
        Self {
            tool_registry,
            max_chain_depth: 3,
        }
    }

    /// Universal reasoning: Analyze request → Plan → Execute chain → Synthesize
    pub async fn reason_universal(&self, request: &str, context: &Value) -> Result<AgenticResponse> {
        // Step 1: Plan with Chain-of-Thought
        let plan = self.plan_request(request, context)?;
        
        // Step 2: Execute tool chain
        let chain_outputs = self.execute_chain(&plan, request).await?;
        
        // Step 3: Synthesize final suggestion
        let suggestion = self.synthesize_suggestion(request, &chain_outputs, &plan)?;
        
        Ok(AgenticResponse {
            suggestion,
            chain: chain_outputs,
            confidence: plan.confidence,
            reasoning_steps: vec![
                format!("Classification: {}", plan.classification),
                format!("Tools: {:?}", plan.tools_needed),
                format!("Chain depth: {}", plan.chain_plan.len()),
            ],
        })
    }

    /// Step 1: Analyze request and create execution plan
    fn plan_request(&self, request: &str, context: &Value) -> Result<ReasoningPlan> {
        // Simplified planning logic (in production, this would use Phi-3 with CoT prompting)
        let request_lower = request.to_lowercase();
        
        let (classification, tools, confidence) = if request_lower.contains("weather") || request_lower.contains("umbrella") {
            ("general", vec!["web_api".to_string(), "memory_rag".to_string()], 0.85)
        } else if request_lower.contains("tune") || request_lower.contains("battery") || request_lower.contains("brightness") {
            ("os", vec!["os_exec".to_string()], 0.90)
        } else if request_lower.contains("install") || request_lower.contains("open") {
            ("app", vec!["app_proxy".to_string()], 0.88)
        } else if request_lower.contains("poem") || request_lower.contains("write") || request_lower.contains("create") {
            ("creative", vec!["gen_creative".to_string()], 0.82)
        } else if request_lower.contains("quantum") || request_lower.contains("philosophy") || request_lower.contains("ethics") {
            ("random", vec!["web_api".to_string(), "gen_creative".to_string()], 0.75)
        } else {
            ("general", vec!["web_api".to_string()], 0.65)
        };

        // Build chain plan
        let mut chain_plan = Vec::new();
        for (idx, tool) in tools.iter().enumerate() {
            chain_plan.push(ChainStep {
                step_id: idx,
                tool: tool.clone(),
                input_source: if idx == 0 {
                    InputSource::Direct
                } else {
                    InputSource::PreviousOutput(idx - 1)
                },
                input_data: if idx == 0 {
                    json!({ "query": request, "context": context })
                } else {
                    json!({ "previous_step": idx - 1 })
                },
            });
        }

        Ok(ReasoningPlan {
            classification: classification.to_string(),
            tools_needed: tools,
            chain_plan,
            confidence: confidence as f32,
        })
    }

    /// Step 2: Execute tool chain with dependency resolution
    async fn execute_chain(&self, plan: &ReasoningPlan, request: &str) -> Result<Vec<ToolOutput>> {
        let mut outputs = Vec::new();
        let mut previous_output: Option<String> = None;

        for step in &plan.chain_plan {
            if step.step_id >= self.max_chain_depth {
                break;
            }

            let input_data = match &step.input_source {
                InputSource::Direct => step.input_data.clone(),
                InputSource::PreviousOutput(step_id) => {
                    if let Some(prev) = &previous_output {
                        json!({
                            "query": request,
                            "context": prev,
                            "previous_step": step_id
                        })
                    } else {
                        step.input_data.clone()
                    }
                }
            };

            let output = self.tool_registry.call_tool(&step.tool, &input_data).await?;
            
            outputs.push(ToolOutput {
                tool_name: step.tool.clone(),
                output: output.clone(),
                confidence: plan.confidence * 0.95_f32.powi(step.step_id as i32), // Decay with depth
            });

            previous_output = Some(output);
        }

        Ok(outputs)
    }

    /// Step 3: Synthesize final suggestion from chain outputs
    fn synthesize_suggestion(&self, request: &str, chain: &[ToolOutput], plan: &ReasoningPlan) -> Result<String> {
        if chain.is_empty() {
            return Ok("Unable to process request - please clarify intent.".to_string());
        }

        // For single-step chains, return the output directly
        if chain.len() == 1 {
            return Ok(format!("{} (confidence: {:.0}%)", 
                chain[0].output, 
                chain[0].confidence * 100.0
            ));
        }

        // For multi-step chains, synthesize
        let context_summary: Vec<String> = chain.iter()
            .map(|o| format!("{}: {}", o.tool_name, o.output))
            .collect();

        Ok(format!(
            "Based on analysis: {} → Suggestion: {} (confidence: {:.0}%)",
            context_summary.join(" → "),
            chain.last().unwrap().output,
            plan.confidence * 100.0
        ))
    }

    /// Confidence-gated execution
    pub async fn execute_with_confidence_gate(&self, request: &str, context: &Value) -> Result<AgenticResponse> {
        let response = self.reason_universal(request, context).await?;

        // Confidence gates
        if response.confidence < 0.6 {
            return Ok(AgenticResponse {
                suggestion: format!("⚠️ Low confidence ({:.0}%). Could you clarify: \"{}\"?", 
                    response.confidence * 100.0, request),
                chain: vec![],
                confidence: response.confidence,
                reasoning_steps: vec!["Clarification needed".to_string()],
            });
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_classification() {
        let registry = ToolRegistry::new_mock();
        let reasoner = AgenticReasoner::new(registry);
        
        let response = reasoner.reason_universal(
            "What's the weather like?",
            &json!({})
        ).await.unwrap();

        assert!(response.confidence > 0.7);
        assert!(!response.chain.is_empty());
    }

    #[tokio::test]
    async fn test_confidence_gate() {
        let registry = ToolRegistry::new_mock();
        let reasoner = AgenticReasoner::new(registry);
        
        let response = reasoner.execute_with_confidence_gate(
            "asdfghjkl qwerty",  // Nonsense
            &json!({})
        ).await.unwrap();

        assert!(response.confidence < 0.6);
        assert!(response.suggestion.contains("clarify"));
    }
}
