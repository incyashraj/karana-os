use anyhow::{Context, Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use tokenizers::{Tokenizer, PaddingParams};
use hf_hub::{api::sync::Api, Repo, RepoType};

// Mocking Phi-3 for the prototype since we can't download 2.3GB in this env
// In a real deployment, this would use candle_transformers::models::quantized_phi3
pub struct KaranaAI {
    // model: Phi3, // Real model
    device: Device,
    // Atom 3: Embedding Engine (Keep this real as it's small)
    embed_model: Option<BertModel>,
    embed_tokenizer: Option<Tokenizer>,
}

impl KaranaAI {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu; // Fallback to CPU
        
        // Atom 3: Initialize Embedding Model (Lazy load or download if missing)
        let (embed_model, embed_tokenizer) = Self::load_embedding_model(&device).unwrap_or_else(|e| {
            log::info!("Warning: Failed to load embedding model: {}. Semantic search will be disabled.", e);
            (None, None)
        });

        Ok(Self {
            device,
            embed_model,
            embed_tokenizer,
        })
    }

    fn load_embedding_model(device: &Device) -> Result<(Option<BertModel>, Option<Tokenizer>)> {
        log::info!("Atom 3: Loading Embedding Model (all-MiniLM-L6-v2)...");
        let api = Api::new()?;
        let repo = api.repo(Repo::new("sentence-transformers/all-MiniLM-L6-v2".to_string(), RepoType::Model));
        
        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config_content = std::fs::read_to_string(config_filename)?;
        let config: BertConfig = serde_json::from_str(&config_content)?;
        
        let mut tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow!(e))?;
        if let Some(pp) = tokenizer.get_padding_mut() {
            pp.strategy = tokenizers::PaddingStrategy::BatchLongest
        } else {
            let pp = PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            tokenizer.with_padding(Some(pp));
        }

        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, device)? };
        let model = BertModel::load(vb, &config)?;

        Ok((Some(model), Some(tokenizer)))
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        if let (Some(model), Some(tokenizer)) = (&self.embed_model, &self.embed_tokenizer) {
            log::info!("Atom 3: Embedding text: '{}'", text);
            let tokens = tokenizer.encode(text, true).map_err(|e| anyhow!(e))?;
            let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
            let token_type_ids = Tensor::new(tokens.get_type_ids(), &self.device)?.unsqueeze(0)?;
            
            log::info!("Atom 3: Token IDs shape: {:?}", token_ids.shape());
            
            // Generate attention mask (1 for real tokens, 0 for padding)
            let attention_mask = tokens.get_attention_mask();
            let attention_mask_tensor = Tensor::new(attention_mask, &self.device)?.unsqueeze(0)?;
            log::info!("Atom 3: Attention Mask shape: {:?}", attention_mask_tensor.shape());

            log::info!("Atom 3: Running Bert Forward...");
            let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask_tensor))?;
            log::info!("Atom 3: Bert Forward Complete.");
            
            // Mean pooling
            let dims = embeddings.dims3()?;
            log::info!("Atom 3: Embeddings dims: {:?}", dims);
            let (_b, seq_len, _hidden_size) = dims;
            
            let sum = embeddings.sum(1)?;
            log::info!("Atom 3: Sum complete.");
            
            let mean = (sum / (seq_len as f64))?;
            log::info!("Atom 3: Mean complete.");
            
            let squeezed = mean.squeeze(0)?;
            log::info!("Atom 3: Squeeze complete.");
            
            let mean_vec = squeezed.to_vec1::<f32>()?;
            log::info!("Atom 3: Vector extraction complete. Size: {}", mean_vec.len());
            
            Ok(mean_vec)
        } else {
            Err(anyhow!("Embedding model not initialized"))
        }
    }

    /// Predict: Natural language → Response (e.g., "Tune storage?")
    /// In a real implementation, this would run the Phi-3 model.
    /// For this prototype environment (where we can't download 2GB), we simulate the AI's "thinking".
    pub fn predict(&mut self, prompt: &str, _max_tokens: usize) -> Result<String> {
        log::info!("AI Predict: Prompt='{}'", prompt);
        
        let p = prompt.to_lowercase();
        
        // Simulate Phi-3 Responses based on context keywords
        let response = if p.contains("tune") || p.contains("compress") {
            "Shard 60% local for battery efficiency. Compress using Zstd."
        } else if p.contains("optimize storage") {
            "Storage Optimization: Moving cold shards to IPFS swarm. Local space saved: 450MB."
        } else if p.contains("setup user") || p.contains("sync") {
            "User Identity Verified (DID:did:karana:user). Syncing /home/me/Documents... ZK-Proof generated."
        } else if (p.contains("code") || p.contains("rust")) && !p.contains("download") && !p.contains("install") {
            "Rendering Dev Environment: Rust Toolchain loaded. Dependency graph visualized. No anomalies detected."
        } else if p.contains("battery") || p.contains("predict") {
            "Battery Analysis: 3h remaining. Suggestion: Dim screen & throttle background swarm? (Vote required)"
        } else if p.contains("propose") || p.contains("ar ui") {
            "Drafting DAO Proposal: 'Enable AR Gesture Interface'. Staking 50 KARA for submission."
        } else if p.contains("restore") || p.contains("snapshot") {
            "System Rollback: Reverting to State Root 0x82a... Integrity verified via ZK-SNARK."
        } else if p.contains("join") || p.contains("testnet") {
            "Swarm Connection: Dialing bootnodes... Connected to 5 peers. Syncing headers."
        } else if p.contains("find app") || p.contains("search app") {
            "Searching Kāraṇa Bazaar (ZK-Indexed)... Found matches. Top result: 'Rust Native IDE' (Stake: 500 KARA)."
        } else if p.contains("install") {
            "Verifying App Manifest... ZK-Proof Valid. Sandbox initialized. Installing..."
        } else if p.contains("test bazaar flow") {
            "Initiating Bazaar Test Sequence. Try typing 'find app rust' to search, then 'install rust-native-ide' to test the fetch/verify loop."
        } else if p.contains("threat") || p.contains("severity") {
            if p.contains("rm -rf") {
                "0.95"
            } else if p.contains("storage write") {
                "0.1"
            } else {
                "0.5"
            }
        } else if p.contains("boot paths") {
            "Recommended: Full Boot for system integrity check."
        } else if p.contains("governance proposal") {
            "Analysis: Proposal aligns with long-term scalability. Risk: Low."
        } else if p.contains("hud") || p.contains("glass") {
            "Smart Glass Interface: HUD Active. Transparency 85%. Battery 82%. Next meeting in 10m."
        } else if p.contains("tutorial") || p.contains("help") {
            if p.contains("boot") {
                "Guide: 1. Mint KARA (proof: 0xabc). 2. Vote eco (stake 10). 3. Ignite Runtime."
            } else if p.contains("setup") {
                "Guide: 1. Run 'karana install'. 2. AI Probes Hardware. 3. DAO Votes on Config."
            } else {
                "Symbiotic Help: Type 'help boot' or 'help setup' for specific guides. Or 'find app' to browse."
            }
        } else if p.contains("peer reliability") {
            "0.98"
        } else {
            "I am Phi-3 (Simulated). I can help with system optimization, coding, and governance."
        };

        log::info!("AI Response: {}", response);
        Ok(response.to_string())
    }

    /// Anomaly Score (For Vigil): Prompt → Risk (0.0-1.0)
    pub fn score_anomaly(&mut self, event: &str) -> Result<f32> {
        let prompt = format!("Rate the severity of this system log from 0.0 (safe) to 1.0 (critical): '{}'. Answer with only the number.", event);
        let response = self.predict(&prompt, 10)?;
        response.trim().parse::<f32>().context("Parse score fail").map(|s| s.min(1.0).max(0.0))
    }
}
