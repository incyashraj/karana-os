# Perfect AI Oracle - Quick Implementation Guide

## 🎯 Goal
Make Kāraṇa OS Oracle answer **ANY** user query with satisfactory, intelligent responses.

---

## 🚀 Quick Wins (Start Today)

### 1. Remove Fallback Functions ✅

**File**: `karana-core/src/ai/mod.rs`

**Current Problem** (lines 914-948):
```rust
fn predict_smart_fallback(&self, prompt: &str) -> Result<String> {
    self.predict_smart_fallback_inner(prompt)
}

fn predict_simulated(&self, prompt: &str) -> Result<String> {
    self.predict_smart_fallback_inner(prompt)
}

fn predict_smart_fallback_inner(&self, prompt: &str) -> Result<String> {
    // Pattern matching instead of real AI ❌
}
```

**Solution**: Make `predict()` always use TinyLlama:
```rust
pub fn predict(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
    // Load model if not loaded
    if self.gen_model.is_none() {
        let (model, tokenizer) = Self::load_gen_model(&self.device)?;
        self.gen_model = model;
        self.gen_tokenizer = tokenizer;
    }
    
    let model = self.gen_model.as_mut()
        .ok_or_else(|| anyhow!("Failed to load LLM"))?;
    let tokenizer = self.gen_tokenizer.as_ref()
        .ok_or_else(|| anyhow!("Failed to load tokenizer"))?;
    
    // Format prompt for chat model
    let chat_prompt = format!(
        "<|system|>\nYou are Kāraṇa, an intelligent AI assistant for smart glasses. \
         Be helpful, concise, and accurate.</s>\n\
         <|user|>\n{}</s>\n\
         <|assistant|>\n",
        prompt
    );
    
    // Tokenize
    let tokens = tokenizer.encode(&chat_prompt, true)
        .map_err(|e| anyhow!("Tokenization failed: {}", e))?
        .get_ids()
        .to_vec();
    
    // Generate with sampling
    let mut generated = tokens.clone();
    let mut logits_processor = LogitsProcessor::new(
        42,          // seed
        Some(0.8),   // temperature
        Some(0.95)   // top_p
    );
    
    for _ in 0..max_tokens {
        let input = Tensor::new(generated.as_slice(), &self.device)?
            .unsqueeze(0)?;
        let logits = model.forward(&input)?;
        let logits = logits.squeeze(0)?;
        let next_logits = logits.get(logits.dim(0)? - 1)?;
        let next_token = logits_processor.sample(&next_logits)?;
        
        // Check for EOS
        if next_token == 2 || next_token == tokenizer.token_to_id("</s>").unwrap_or(2) {
            break;
        }
        
        generated.push(next_token);
    }
    
    // Decode only the new tokens
    let response = tokenizer.decode(&generated[tokens.len()..], true)
        .map_err(|e| anyhow!("Decoding failed: {}", e))?;
    
    Ok(response.trim().to_string())
}
```

**Test**:
```bash
cargo test --lib ai::tests::test_real_llm_inference
```

---

### 2. Fix Universal Oracle Synthesis ✅

**File**: `karana-core/src/oracle/universal.rs`

**Current Problem** (line 273):
```rust
fn synthesize_answer(&self, results: &[RagChunk], _query: &str) -> Result<String> {
    // Simple concatenation for now
    // TODO: Use LLM to synthesize coherent answer ❌
    let combined = results.iter()
        .take(2)
        .map(|r| r.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(combined)
}
```

**Solution**: Use real AI to synthesize:
```rust
fn synthesize_answer(&self, results: &[RagChunk], query: &str) -> Result<String> {
    if results.is_empty() {
        return Ok("I don't have information about that.".to_string());
    }
    
    // Build context from top results
    let context = results.iter()
        .take(3)
        .enumerate()
        .map(|(i, chunk)| {
            format!("[Source {}] {}", i + 1, chunk.text.trim())
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    
    // Construct synthesis prompt
    let prompt = format!(
        "Based on these sources, answer the question concisely and accurately.\n\n\
         Question: {}\n\n\
         Sources:\n{}\n\n\
         Answer:",
        query,
        context
    );
    
    // Use AI to synthesize
    let mut ai = self.ai.lock()
        .map_err(|e| anyhow!("AI lock error: {}", e))?;
    
    let answer = ai.predict(&prompt, 150)
        .context("Failed to synthesize answer")?;
    
    // Add source attribution
    let sources = results.iter()
        .take(3)
        .map(|r| r.source_doc.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    
    Ok(format!("{}\n\nSources: {}", answer.trim(), sources))
}
```

---

### 3. Add Real Web Search ✅

**File**: `karana-core/src/oracle/web_search.rs` (NEW)

```rust
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub score: f32,
}

pub struct WebSearchEngine {
    client: reqwest::Client,
    enabled: bool,
}

impl WebSearchEngine {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("KaranaOS/1.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            enabled: true,
        }
    }
    
    /// Search using DuckDuckGo Instant Answer API
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if !self.enabled {
            return Ok(vec![]);
        }
        
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );
        
        let response = self.client.get(&url)
            .send()
            .await?
            .json::<DdgResponse>()
            .await?;
        
        let mut results = Vec::new();
        
        // Add instant answer if available
        if !response.Abstract.is_empty() {
            results.push(SearchResult {
                title: response.Heading.clone(),
                snippet: response.Abstract.clone(),
                url: response.AbstractURL.clone(),
                score: 1.0,
            });
        }
        
        // Add related topics
        for topic in response.RelatedTopics.iter().take(3) {
            if let Some(text) = &topic.Text {
                results.push(SearchResult {
                    title: topic.FirstURL.clone().unwrap_or_default(),
                    snippet: text.clone(),
                    url: topic.FirstURL.clone().unwrap_or_default(),
                    score: 0.8,
                });
            }
        }
        
        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
struct DdgResponse {
    #[serde(default)]
    Abstract: String,
    #[serde(default)]
    AbstractURL: String,
    #[serde(default)]
    Heading: String,
    #[serde(default)]
    RelatedTopics: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    Text: Option<String>,
    FirstURL: Option<String>,
}
```

**Integration in UniversalOracle**:
```rust
// Add to UniversalOracle struct
pub web_search: Arc<WebSearchEngine>,

// In query() method, after local RAG:
if let Some(web_results) = self.query_web(query).await? {
    return Ok(web_results);
}

async fn query_web(&self, query: &str) -> Result<Option<UniversalResponse>> {
    if !self.web_proxy_enabled {
        return Ok(None);
    }
    
    let results = self.web_search.search(query).await?;
    
    if results.is_empty() {
        return Ok(None);
    }
    
    // Use top result
    let top = &results[0];
    
    Ok(Some(UniversalResponse {
        answer: top.snippet.clone(),
        source: ResponseSource::WebProxy,
        confidence: top.score,
        proof: None,
        follow_up: vec![
            "Tell me more".to_string(),
            "Related topics".to_string(),
        ],
    }))
}
```

---

### 4. Enhance Context Understanding ✅

**File**: `karana-core/src/oracle/veil.rs`

**Add to parse_intent**:
```rust
async fn parse_intent(
    &self,
    intent: &str,
    source: InputSource,
    timestamp: u64,
) -> Result<ParsedIntent> {
    // Get enriched context from memory
    let memory_context = if let Some(memory) = &self.memory {
        memory.get_enriched_context(intent)?
    } else {
        serde_json::json!({})
    };
    
    // Try Universal Oracle first for general queries
    if self.is_general_query(intent) {
        let universal_response = self.universal_oracle.query(
            intent,
            &QueryContext {
                user_did: self.user_did.read().await.clone(),
                location: None,
                time_of_day: chrono::Local::now().hour() as u8,
                conversation_history: memory_context.get("recent_intents")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            },
        ).await?;
        
        // Build enhanced response with reasoning
        let action = IntentAction::UniversalQuery {
            answer: universal_response.answer.clone(),
            source: format!("{:?}", universal_response.source),
            confidence: universal_response.confidence,
        };
        
        return Ok(ParsedIntent {
            action,
            confidence: universal_response.confidence,
            source,
            timestamp,
            raw: intent.to_string(),
        });
    }
    
    // Continue with existing AI-based parsing...
}
```

---

### 5. Add Conversation Memory ✅

**File**: `karana-core/src/ai/dialogue.rs` (ENHANCE)

```rust
use std::collections::VecDeque;
use std::time::SystemTime;

pub struct ConversationTurn {
    pub user: String,
    pub assistant: String,
    pub timestamp: SystemTime,
}

pub struct DialogueManager {
    history: VecDeque<ConversationTurn>,
    max_turns: usize,
    ai: Arc<StdMutex<KaranaAI>>,
}

impl DialogueManager {
    pub fn new(ai: Arc<StdMutex<KaranaAI>>) -> Self {
        Self {
            history: VecDeque::new(),
            max_turns: 10,
            ai,
        }
    }
    
    pub fn generate_response(&mut self, user_input: &str) -> Result<String> {
        // Build conversation context
        let context = self.history.iter()
            .map(|turn| format!("User: {}\nKāraṇa: {}", turn.user, turn.assistant))
            .collect::<Vec<_>>()
            .join("\n");
        
        // Construct prompt with personality
        let prompt = if context.is_empty() {
            format!(
                "<|system|>\nYou are Kāraṇa, an intelligent AI assistant for smart glasses. \
                 You are helpful, concise, and proactive.</s>\n\
                 <|user|>\n{}</s>\n\
                 <|assistant|>\n",
                user_input
            )
        } else {
            format!(
                "<|system|>\nYou are Kāraṇa, an intelligent AI assistant for smart glasses. \
                 You are helpful, concise, and proactive. Remember the conversation context.</s>\n\
                 {}\n\
                 <|user|>\n{}</s>\n\
                 <|assistant|>\n",
                context,
                user_input
            )
        };
        
        // Generate response
        let mut ai = self.ai.lock().unwrap();
        let response = ai.predict(&prompt, 200)?;
        
        // Store in history
        self.history.push_back(ConversationTurn {
            user: user_input.to_string(),
            assistant: response.clone(),
            timestamp: SystemTime::now(),
        });
        
        // Prune old history
        while self.history.len() > self.max_turns {
            self.history.pop_front();
        }
        
        Ok(response)
    }
    
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}
```

---

## 🧪 Testing Strategy

### Test 1: Knowledge Questions
```bash
# In simulator or API
curl -X POST http://localhost:8080/api/oracle \
  -H "Content-Type: application/json" \
  -d '{"text": "What is quantum computing?"}'

# Expected: Real explanation, not "I don't know"
```

### Test 2: Multi-Step Queries
```bash
curl -X POST http://localhost:8080/api/oracle \
  -H "Content-Type: application/json" \
  -d '{"text": "What is the weather in Tokyo and should I pack an umbrella?"}'

# Expected: Weather info + recommendation
```

### Test 3: Conversational Memory
```bash
# First query
curl -X POST http://localhost:8080/api/oracle \
  -d '{"text": "My name is Alice"}'

# Second query
curl -X POST http://localhost:8080/api/oracle \
  -d '{"text": "What is my name?"}'

# Expected: "Your name is Alice"
```

---

## 📊 Validation Checklist

- [ ] **No fallback functions** - All responses use real LLM
- [ ] **Universal Oracle** - Synthesizes answers from RAG
- [ ] **Web search** - Can find current information
- [ ] **Conversation memory** - Remembers past exchanges
- [ ] **Context awareness** - Uses user preferences
- [ ] **Response quality** - Natural, helpful answers
- [ ] **Performance** - <2s for most queries
- [ ] **Offline capability** - Works without internet for cached knowledge

---

## 🎯 Success Examples

### Before ❌
**User**: "Explain quantum computing"
**Oracle**: "I can help with system commands. Try 'check balance' or 'send tokens'"

### After ✅
**User**: "Explain quantum computing"
**Oracle**: "Quantum computing leverages quantum mechanics principles like superposition and entanglement to process information. Unlike classical bits (0 or 1), quantum bits (qubits) can exist in multiple states simultaneously, enabling exponential speedup for certain computational problems. Key applications include cryptography, drug discovery, and optimization. Would you like me to recommend learning resources?"

---

### Before ❌
**User**: "What's the weather and should I bring an umbrella?"
**Oracle**: "Opening weather app..."

### After ✅
**User**: "What's the weather and should I bring an umbrella?"
**Oracle**: "Currently 18°C with 70% chance of rain this afternoon. Yes, I recommend bringing an umbrella. There's a weather system moving in around 2 PM. Would you like me to set a reminder?"

---

## 🚀 Next Steps

1. **Week 1**: Implement fallback removal + real synthesis
2. **Week 2**: Add web search + conversation memory
3. **Week 3**: Enhance with Wikipedia knowledge base
4. **Week 4**: Optimize performance + caching
5. **Week 5**: User testing + iteration

**Start with step 1 today!** Remove the fallback functions and use real LLM inference.
