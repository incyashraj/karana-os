# Plan: Oracle Veil Core Implementation

## Overview
The `OracleVeil` is the central AI gateway - the sole user interface. It mediates all interactions between the user and the backend weave.

---

## Target Architecture (From Vision)

```rust
pub struct OracleVeil {
    ai: Phi3,                    // Candle Phi-3 q4.gguf
    device: Device,              // CPU/NPU
    monad: KaranaMonad,          // Backend command interface
    sense: MultimodalSense,      // Input capture
    manifest: ManifestEngine,    // Output rendering
}

impl OracleVeil {
    pub async fn mediate(&mut self, input: MultimodalInput) -> Result<Manifest> {
        // Step 1: Parse Intent (Gaze/Voice → Embed)
        let tensor = self.input_to_tensor(&input)?;
        let intent_prompt = self.ai.embed_to_prompt(&tensor)?;
        let intent = self.ai.predict(&intent_prompt, 20)?;

        // Step 2: ZK-Sign (Prove "This intent is mine")
        let commitment = Sha256::digest(&intent.as_bytes()).into();
        let proof = prove_intent(&intent.as_bytes(), commitment)?;

        // Step 3: Command Backend (Oracle to Monad)
        let backend_resp = self.monad.execute_intent(&intent, proof.clone()).await?;

        // Step 4: Process & Manifest (AI Re-Present)
        let output_prompt = format!("Present backend: {} for glasses (minimal AR/haptic)", backend_resp);
        let manifest = self.ai.predict(&output_prompt, 10)?;
        let haptic = if manifest.contains("success") { HapticPulse::Short } else { None };

        Ok(Manifest { text: manifest, haptic, ar_whisper: "Overlay bar graph" })
    }
}
```

---

## Implementation Plan

### File: `karana-core/src/oracle/veil.rs`

### Step 1: Define Core Structures

```rust
use anyhow::Result;
use candle_core::Device;
use tokio::sync::mpsc;
use crate::ai::KaranaAI;
use crate::zk;
use crate::hardware::haptic::HapticPattern;

/// The Oracle Veil - Sole user interface
pub struct OracleVeil {
    /// AI model for intent parsing and response generation
    ai: Arc<Mutex<KaranaAI>>,
    
    /// Device for AI inference
    device: Device,
    
    /// Channel to command the Monad (backend)
    monad_tx: mpsc::Sender<OracleCommand>,
    
    /// Channel to receive Monad responses
    monad_rx: mpsc::Receiver<MonadResponse>,
    
    /// User's DID for signing
    user_did: String,
    
    /// Conversation context
    dialogue_state: DialogueState,
}

/// Commands Oracle sends to Monad
pub enum OracleCommand {
    /// Execute an intent (e.g., "tune battery")
    ExecuteIntent { intent: String, proof: Vec<u8> },
    
    /// Query state (e.g., "get balance")
    QueryState { query: String },
    
    /// Store data (e.g., "save note")
    Store { data: Vec<u8>, context: String },
    
    /// Governance action (e.g., "vote yes on proposal 1")
    Governance { action: String, params: serde_json::Value },
}

/// Responses from Monad to Oracle
pub struct MonadResponse {
    pub success: bool,
    pub data: String,
    pub proof_hash: Option<Vec<u8>>,
    pub chain_tx: Option<String>,
}

/// What the Oracle manifests to the user
pub struct Manifest {
    /// Short text for AR whisper (max 50 chars)
    pub text: String,
    
    /// Haptic pattern to play
    pub haptic: Option<HapticPattern>,
    
    /// Optional AR overlay type
    pub ar_overlay: Option<AROverlay>,
}

pub enum AROverlay {
    Text(String),          // Faint text whisper
    Progress(f32),         // Bar graph (0.0-1.0)
    Confirmation,          // Checkmark
    Warning,               // Alert icon
}

/// Conversation state for context awareness
pub struct DialogueState {
    pub history: Vec<ConversationTurn>,
    pub pending_confirmation: Option<PendingAction>,
    pub context: OracleContext,
}
```

### Step 2: Implement Core Mediate Function

```rust
impl OracleVeil {
    pub async fn new(
        ai: Arc<Mutex<KaranaAI>>,
        monad_tx: mpsc::Sender<OracleCommand>,
        monad_rx: mpsc::Receiver<MonadResponse>,
        user_did: String,
    ) -> Self {
        Self {
            ai,
            device: Device::Cpu,
            monad_tx,
            monad_rx,
            user_did,
            dialogue_state: DialogueState::default(),
        }
    }
    
    /// The main entry point - mediates user intent to backend and back
    pub async fn mediate(&mut self, input: MultimodalInput) -> Result<Manifest> {
        // ═══════════════════════════════════════════════════════════════
        // STEP 1: Parse Intent from multimodal input
        // ═══════════════════════════════════════════════════════════════
        let intent = self.parse_intent(&input).await?;
        log::info!("[ORACLE] Intent parsed: {}", intent.action);
        
        // ═══════════════════════════════════════════════════════════════
        // STEP 2: ZK-Sign the intent (prove ownership)
        // ═══════════════════════════════════════════════════════════════
        let intent_bytes = serde_json::to_vec(&intent)?;
        let commitment = zk::compute_hash(&intent_bytes);
        let proof = zk::prove_intent(&intent_bytes, commitment)?;
        log::info!("[ORACLE] ZK proof generated: {} bytes", proof.len());
        
        // ═══════════════════════════════════════════════════════════════
        // STEP 3: Command Backend (send to Monad)
        // ═══════════════════════════════════════════════════════════════
        let command = self.intent_to_command(&intent, proof.clone());
        self.monad_tx.send(command).await?;
        
        // Wait for response
        let response = self.monad_rx.recv().await
            .ok_or_else(|| anyhow::anyhow!("Monad channel closed"))?;
        
        log::info!("[ORACLE] Backend response: {}", response.data);
        
        // ═══════════════════════════════════════════════════════════════
        // STEP 4: Manifest response (minimal AR/haptic)
        // ═══════════════════════════════════════════════════════════════
        let manifest = self.create_manifest(&intent, &response).await?;
        
        // Update dialogue state
        self.dialogue_state.history.push(ConversationTurn {
            role: "user".into(),
            content: input.text.clone().unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
        self.dialogue_state.history.push(ConversationTurn {
            role: "oracle".into(),
            content: manifest.text.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
        
        Ok(manifest)
    }
    
    /// Parse multimodal input into structured intent
    async fn parse_intent(&self, input: &MultimodalInput) -> Result<ParsedIntent> {
        let ai = self.ai.lock().unwrap();
        
        // Combine all input modalities
        let prompt = if let Some(text) = &input.text {
            text.clone()
        } else if let Some(voice) = &input.voice_transcription {
            voice.clone()
        } else {
            return Err(anyhow::anyhow!("No input to parse"));
        };
        
        // Use AI to parse intent
        let response = ai.predict(&prompt, 50)?;
        
        // Try to parse as structured action
        if let Ok(action) = serde_json::from_str::<ParsedIntent>(&response) {
            return Ok(action);
        }
        
        // Fallback to semantic matching
        // ... use existing semantic matching logic
        
        Ok(ParsedIntent {
            action: "unknown".into(),
            params: serde_json::Value::Null,
            confidence: 0.5,
        })
    }
    
    /// Convert parsed intent to Monad command
    fn intent_to_command(&self, intent: &ParsedIntent, proof: Vec<u8>) -> OracleCommand {
        match intent.action.as_str() {
            "transfer" | "stake" | "vote" => {
                OracleCommand::Governance {
                    action: intent.action.clone(),
                    params: intent.params.clone(),
                }
            },
            "store" | "save" | "note" => {
                let data = intent.params.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .as_bytes()
                    .to_vec();
                OracleCommand::Store {
                    data,
                    context: intent.action.clone(),
                }
            },
            "balance" | "status" | "files" | "proposals" => {
                OracleCommand::QueryState {
                    query: intent.action.clone(),
                }
            },
            _ => {
                OracleCommand::ExecuteIntent {
                    intent: serde_json::to_string(intent).unwrap(),
                    proof,
                }
            }
        }
    }
    
    /// Create minimal manifest from backend response
    async fn create_manifest(&self, intent: &ParsedIntent, response: &MonadResponse) -> Result<Manifest> {
        // Format for minimal display
        let text = if response.success {
            // Truncate to 50 chars for AR whisper
            let short = if response.data.len() > 47 {
                format!("{}...", &response.data[..47])
            } else {
                response.data.clone()
            };
            format!("✓ {}", short)
        } else {
            format!("✗ Failed: {}", &response.data[..40.min(response.data.len())])
        };
        
        // Determine haptic
        let haptic = if response.success {
            Some(HapticPattern::Success)
        } else {
            Some(HapticPattern::Error)
        };
        
        // Determine AR overlay based on intent type
        let ar_overlay = match intent.action.as_str() {
            "balance" => {
                // Show as progress bar (balance/max)
                if let Ok(balance) = response.data.parse::<f32>() {
                    Some(AROverlay::Progress(balance / 10000.0))
                } else {
                    Some(AROverlay::Text(response.data.clone()))
                }
            },
            "transfer" | "stake" | "vote" => {
                Some(AROverlay::Confirmation)
            },
            _ => {
                Some(AROverlay::Text(text.clone()))
            }
        };
        
        Ok(Manifest {
            text,
            haptic,
            ar_overlay,
        })
    }
}
```

### Step 3: Supporting Types

```rust
/// Multimodal input from sensors
pub struct MultimodalInput {
    /// Transcribed voice (if any)
    pub voice_transcription: Option<String>,
    
    /// Voice MFCC features for embedding
    pub voice_features: Option<Vec<f32>>,
    
    /// Direct text input (for simulator)
    pub text: Option<String>,
    
    /// Gaze direction vector [x, y, z]
    pub gaze_vector: Option<[f32; 3]>,
    
    /// Gaze target (what user is looking at)
    pub gaze_target: Option<String>,
    
    /// IMU gesture detected
    pub gesture: Option<Gesture>,
    
    /// Timestamp
    pub timestamp: u64,
}

pub enum Gesture {
    Nod,        // Yes/confirm
    Shake,      // No/cancel
    Tilt,       // Menu/options
    None,
}

/// Parsed intent from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    pub action: String,
    pub params: serde_json::Value,
    pub confidence: f32,
}

/// Conversation turn for history
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

impl Default for DialogueState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            pending_confirmation: None,
            context: OracleContext::default(),
        }
    }
}
```

---

## Integration Points

### 1. Update `monad.rs`
Add channel endpoints for Oracle communication:

```rust
impl KaranaMonad {
    pub fn create_oracle_channels() -> (
        mpsc::Sender<OracleCommand>,
        mpsc::Receiver<MonadResponse>,
        mpsc::Receiver<OracleCommand>,
        mpsc::Sender<MonadResponse>,
    ) {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (resp_tx, resp_rx) = mpsc::channel(100);
        (cmd_tx, resp_rx, cmd_rx, resp_tx)
    }
    
    pub async fn handle_oracle_command(&mut self, cmd: OracleCommand) -> MonadResponse {
        match cmd {
            OracleCommand::ExecuteIntent { intent, proof } => {
                // Verify proof first
                // Execute intent
                // Return response
            },
            // ... other commands
        }
    }
}
```

### 2. Update `api/handlers.rs`
Route all API calls through OracleVeil:

```rust
pub async fn process_oracle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OracleRequest>,
) -> impl IntoResponse {
    let input = MultimodalInput {
        text: Some(req.text),
        ..Default::default()
    };
    
    let manifest = state.oracle_veil.mediate(input).await?;
    
    // Return manifest as API response
}
```

### 3. Add to `lib.rs`
Export the new Oracle veil module:

```rust
pub mod oracle;
pub use oracle::veil::OracleVeil;
```

---

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_intent_parsing() {
        let oracle = setup_test_oracle();
        let input = MultimodalInput {
            text: Some("send 50 to alice".into()),
            ..Default::default()
        };
        let intent = oracle.parse_intent(&input).await.unwrap();
        assert_eq!(intent.action, "transfer");
    }
    
    #[tokio::test]
    async fn test_zk_signing() {
        let intent = ParsedIntent { action: "transfer".into(), .. };
        let proof = sign_intent(&intent).unwrap();
        assert!(verify_intent_proof(&intent, &proof));
    }
}
```

### Integration Tests
```bash
# Test full flow
echo "send 50 to alice" | nc localhost 9000
# Should see:
# [ORACLE] Intent parsed: transfer
# [ORACLE] ZK proof generated: 128 bytes
# [ORACLE] Backend response: Transfer complete
# Manifest: ✓ Sent 50 KARA to alice
```

---

## Dependencies

### Cargo.toml additions:
```toml
# Already have most, may need:
sha2 = "0.10"  # For commitment hashing
```

---

## Timeline

| Task | Duration | Dependencies |
|------|----------|--------------|
| Create `veil.rs` structure | 2 hours | None |
| Implement `mediate()` | 4 hours | Structure |
| Add ZK intent signing | 2 hours | `zk/mod.rs` |
| Wire to Monad | 3 hours | `monad.rs` |
| API integration | 2 hours | `handlers.rs` |
| Testing | 3 hours | All above |
| **Total** | **16 hours** | |

---

## Success Criteria

- [ ] `OracleVeil::mediate()` processes text input end-to-end
- [ ] Every intent is ZK-signed before execution
- [ ] Manifest returns in < 500ms
- [ ] Haptic patterns fire for success/error
- [ ] Conversation history maintained across turns

---

*Oracle Veil Plan v1.0 - December 3, 2025*
