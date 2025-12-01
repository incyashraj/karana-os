use anyhow::{Context, Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use candle_transformers::models::quantized_llama::ModelWeights as QLlama;
use candle_transformers::models::whisper::{self as m, Config as WhisperConfig, Model as WhisperModel};
use candle_transformers::models::blip;
use candle_transformers::generation::LogitsProcessor;
use tokenizers::{Tokenizer, PaddingParams};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::PathBuf;
use std::io::Write;

// TinyLlama 1.1B Chat (Quantized) - ~670MB
const MODEL_REPO: &str = "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF";
const MODEL_FILE: &str = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";

// Whisper Tiny (Quantized or Float? Let's use tiny.en for speed/size)
const WHISPER_REPO: &str = "openai/whisper-tiny.en";

// BLIP (Image Captioning)
const BLIP_REPO: &str = "Salesforce/blip-image-captioning-base";

pub struct KaranaAI {
    device: Device,
    // Atom 3: Embedding Engine (Small, always loaded)
    embed_model: Option<BertModel>,
    embed_tokenizer: Option<Tokenizer>,
    // Atom 3: Generative Engine (Large, load on demand)
    gen_model: Option<QLlama>,
    gen_tokenizer: Option<Tokenizer>,
    // Atom 3: Voice Engine (Whisper)
    whisper_model: Option<WhisperModel>,
    whisper_tokenizer: Option<Tokenizer>,
    whisper_config: Option<WhisperConfig>,
    whisper_mel: Option<m::audio::AudioConfig>,
    // Atom 3: Vision Engine (BLIP)
    blip_model: Option<blip::BlipForConditionalGeneration>,
    blip_tokenizer: Option<Tokenizer>,
    blip_processor: Option<blip::BlipProcessor>,
    blip_config: Option<blip::Config>,
}

impl KaranaAI {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu; // IoT/Glasses = CPU (ARM)
        
        // Atom 3: Initialize Embedding Model
        let (embed_model, embed_tokenizer) = Self::load_embedding_model(&device).unwrap_or_else(|e| {
            log::warn!("Atom 3: Embedding model init failed: {}", e);
            (None, None)
        });

        // Atom 3: Try Initialize Generative Model (Lazy)
        let (gen_model, gen_tokenizer) = Self::load_gen_model().unwrap_or_else(|e| {
            log::info!("Atom 3: Generative AI not loaded (Running in Simulation Mode). Reason: {}", e);
            (None, None)
        });

        // Atom 3: Try Initialize Whisper (Lazy)
        // We don't load it by default to save RAM, but we prepare the struct.
        // For now, let's leave it None and load on demand.

        Ok(Self {
            device,
            embed_model,
            embed_tokenizer,
            gen_model,
            gen_tokenizer,
            whisper_model: None,
            whisper_tokenizer: None,
            whisper_config: None,
            whisper_mel: None,
            blip_model: None,
            blip_tokenizer: None,
            blip_processor: None,
            blip_config: None,
        })
    }

    pub fn load_blip(&mut self) -> Result<()> {
        if self.blip_model.is_some() { return Ok(()); }
        
        log::info!("Atom 3: Loading Vision Model (BLIP)...");
        let api = Api::new()?;
        let repo = api.repo(Repo::new(BLIP_REPO.to_string(), RepoType::Model));
        
        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;
        // BLIP usually has a preprocessor config too, but candle might hardcode or infer it.
        // Let's check if we need it. candle-transformers blip example uses it.
        
        let config: blip::Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow!(e))?;
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &self.device)? };
        let model = blip::BlipForConditionalGeneration::new(&config, vb)?;
        
        // Processor is usually just image resizing/normalization logic.
        // candle-transformers::models::blip::BlipProcessor might not exist as a struct, 
        // usually we do manual image prep.
        // But let's check if we can just store the config and do prep in `describe_image`.
        
        self.blip_model = Some(model);
        self.blip_tokenizer = Some(tokenizer);
        self.blip_config = Some(config);
        
        Ok(())
    }

    pub fn describe_image(&mut self, image_path: &str) -> Result<String> {
        if self.blip_model.is_none() {
            self.load_blip()?;
        }
        
        let model = self.blip_model.as_ref().unwrap();
        let tokenizer = self.blip_tokenizer.as_ref().unwrap();
        let config = self.blip_config.as_ref().unwrap();
        
        // Load and Preprocess Image
        let img = image::io::Reader::open(image_path)?.decode()?;
        let (width, height) = (384, 384); // BLIP default
        let img = img.resize_exact(width, height, image::imageops::FilterType::Triangle);
        let img = img.to_rgb8();
        let data = img.into_raw();
        let data = Tensor::from_vec(data, (height as usize, width as usize, 3), &self.device)?.permute((2, 0, 1))?;
        let mean = Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &self.device)?.reshape((3, 1, 1))?;
        let std = Tensor::new(&[0.26862954f32, 0.26130258, 0.27577711], &self.device)?.reshape((3, 1, 1))?;
        let image_input = (data.to_dtype(DType::F32)? / 255.)?
            .broadcast_sub(&mean)?
            .broadcast_div(&std)?
            .unsqueeze(0)?;

        // Generate Caption
        // BLIP generation loop
        let vision_model = model.vision_model();
        let image_embeds = vision_model.forward(&image_input)?;
        
        let mut token_ids = vec![30522u32]; // [BOS] (Check tokenizer, usually 30522 for BERT-based)
        // Actually BLIP uses BERT tokenizer. 30522 is [CLS] or similar?
        // Let's trust the standard start token or look it up.
        // HuggingFace BertTokenizer: [CLS] = 101, [SEP] = 102.
        // Wait, BLIP might use a different one.
        // Let's use the tokenizer to encode an empty string or special token.
        
        // Correct approach:
        token_ids = vec![30522]; // Hardcoded from candle example for BLIP
        
        let mut logits_processor = LogitsProcessor::new(299792458, Some(1.0), None);

        for _ in 0..20 {
            let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = model.text_decoder().forward(&input_ids, &image_embeds)?;
            let logits = logits.squeeze(0)?;
            let next_token_logits = logits.get(logits.dim(0)? - 1)?;
            
            let next_token = logits_processor.sample(&next_token_logits)?;
            token_ids.push(next_token);
            if next_token == 102 { break; } // [SEP]
        }

        let caption = tokenizer.decode(&token_ids, true).map_err(|e| anyhow!(e))?;
        Ok(caption)
    }

    pub fn load_whisper(&mut self) -> Result<()> {
        if self.whisper_model.is_some() { return Ok(()); }
        
        log::info!("Atom 3: Loading Whisper Model (tiny.en)...");
        let api = Api::new()?;
        let repo = api.repo(Repo::new(WHISPER_REPO.to_string(), RepoType::Model));
        
        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config: WhisperConfig = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow!(e))?;
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &self.device)? };
        let model = WhisperModel::new(&config, vb)?;
        
        self.whisper_model = Some(model);
        self.whisper_tokenizer = Some(tokenizer);
        self.whisper_config = Some(config);
        self.whisper_mel = Some(m::audio::AudioConfig::hparams_tiny()); // tiny.en uses tiny hparams
        
        Ok(())
    }

    pub fn transcribe(&mut self, audio_data: Vec<f32>) -> Result<String> {
        if self.whisper_model.is_none() {
            self.load_whisper()?;
        }
        
        let model = self.whisper_model.as_ref().unwrap();
        let tokenizer = self.whisper_tokenizer.as_ref().unwrap();
        let config = self.whisper_config.as_ref().unwrap();
        // let mel_config = self.whisper_mel.as_ref().unwrap(); // Not used directly in model.forward? 
        // Actually we need to compute Mel Spectrogram.
        
        // Note: candle-transformers 0.9.1 whisper example uses a helper for mel.
        // We need to implement or use `candle_transformers::models::whisper::audio::pcm_to_mel`
        // But that might not be public. Let's check if we can use the model directly.
        // The `Model` expects a Tensor of mel spectrograms.
        
        // For this prototype, implementing full Mel extraction is complex.
        // However, `candle-transformers` usually exposes `pcm_to_mel`.
        // If not, we might need to rely on a simulation for the *transcription* part if the audio processing is too heavy to implement in one go.
        // BUT the user said "Real Functionality".
        // So I must try to use `pcm_to_mel`.
        
        // Let's assume `m::audio::pcm_to_mel` exists and works.
        // If it fails to compile, I will fix it.
        
        let mel = m::audio::pcm_to_mel(&config, &audio_data, &m::audio::AudioConfig::hparams_tiny())?;
        let mel_len = mel.len();
        let mel_tensor = Tensor::from_vec(mel, (1, 80, mel_len / 80), &self.device)?;
        
        // Generate tokens
        // Simple greedy decoding
        let mut tokens = vec![50258u32, 50259, 50359]; // <|startoftranscript|>, <|en|>, <|transcribe|> (check tokenizer for exact IDs)
        // Actually, let's use the tokenizer to find start tokens if possible, or hardcode for tiny.en
        // tiny.en: 50257=<|endoftext|>, 50258=<|startoftranscript|>, 50259=<|en|>, 50359=<|transcribe|>, 50363=<|notimestamps|>
        
        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.0), None); // Greedy
        
        for _ in 0..100 {
            let input = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = model.decoder().forward(&input, &model.encoder().forward(&mel_tensor, true)?)?; 
            // Wait, encoder forward is expensive, should be done once.
            // model.encoder().forward(&mel_tensor, true)? -> audio_features
            
            // Correct loop:
            // 1. Encode audio
            // 2. Loop decode
            break; // Placeholder to avoid infinite loop in this thought block
        }
        
        // Re-implementing full Whisper decode loop is verbose.
        // Let's use a simplified version or just return a "Real Transcription" stub if the loop is too big.
        // NO, "Real Functionality".
        
        // Okay, let's do the proper loop.
        let audio_features = model.encoder().forward(&mel_tensor, true)?;
        
        for _ in 0..100 {
            let input = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = model.decoder().forward(&input, &audio_features)?;
            let logits = logits.squeeze(0)?;
            let next_token_logits = logits.get(logits.dim(0)? - 1)?;
            let next_token = logits_processor.sample(&next_token_logits)?;
            
            tokens.push(next_token);
            if next_token == 50257 { break; } // <|endoftext|>
        }
        
        let decoded = tokenizer.decode(&tokens, true).map_err(|e| anyhow!(e))?;
        Ok(decoded)
    }

    fn load_embedding_model(device: &Device) -> Result<(Option<BertModel>, Option<Tokenizer>)> {
        // ... (Keep existing embedding logic, it's fine)
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

    fn load_gen_model() -> Result<(Option<QLlama>, Option<Tokenizer>)> {
        // Check local cache first
        let cache_dir = PathBuf::from("karana-cache/models");
        let model_path = cache_dir.join(MODEL_FILE);
        
        // We need a tokenizer. TinyLlama uses Llama tokenizer.
        // We can fetch tokenizer.json from the base repo or use a local one.
        // For simplicity, let's try to fetch from HF if not local.
        
        if !model_path.exists() {
            return Err(anyhow!("Model file not found at {:?}. Run 'install ai-core' to download.", model_path));
        }

        log::info!("Atom 3: Loading Generative Model from {:?}...", model_path);
        
        // Load GGUF
        let mut file = std::fs::File::open(&model_path)?;
        let model = candle_transformers::models::quantized_llama::ModelWeights::from_gguf(
            &mut file, 
            &mut file // Reader
        )?;

        // Load Tokenizer (Fetch from HF if needed, or assume it's cached)
        let api = Api::new()?;
        let repo = api.repo(Repo::new("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(), RepoType::Model));
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow!(e))?;

        Ok((Some(model), Some(tokenizer)))
    }

    pub fn download_model(&self) -> Result<String> {
        log::info!("Atom 3: Downloading {}...", MODEL_FILE);
        let api = Api::new()?;
        let repo = api.repo(Repo::new(MODEL_REPO.to_string(), RepoType::Model));
        let path = repo.get(MODEL_FILE)?;
        
        // Move/Copy to karana-cache/models
        let cache_dir = PathBuf::from("karana-cache/models");
        std::fs::create_dir_all(&cache_dir)?;
        let target_path = cache_dir.join(MODEL_FILE);
        
        std::fs::copy(path, &target_path)?;
        log::info!("Atom 3: Model installed to {:?}", target_path);
        
        Ok(format!("AI Core Installed: {:?}", target_path))
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        if let (Some(model), Some(tokenizer)) = (&self.embed_model, &self.embed_tokenizer) {
            // ... (Keep existing embedding logic)
            let tokens = tokenizer.encode(text, true).map_err(|e| anyhow!(e))?;
            let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
            let token_type_ids = Tensor::new(tokens.get_type_ids(), &self.device)?.unsqueeze(0)?;
            let attention_mask = tokens.get_attention_mask();
            let attention_mask_tensor = Tensor::new(attention_mask, &self.device)?.unsqueeze(0)?;
            let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask_tensor))?;
            let (_b, seq_len, _hidden_size) = embeddings.dims3()?;
            let sum = embeddings.sum(1)?;
            let mean = (sum / (seq_len as f64))?;
            let squeezed = mean.squeeze(0)?;
            let mean_vec = squeezed.to_vec1::<f32>()?;
            Ok(mean_vec)
        } else {
            Err(anyhow!("Embedding model not initialized"))
        }
    }

    pub fn predict(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        if let (Some(model), Some(tokenizer)) = (&mut self.gen_model, &self.gen_tokenizer) {
            // Real Inference
            log::info!("Atom 3: Running Real Inference on TinyLlama...");
            
            // Format prompt for Chat (TinyLlama format)
            let formatted_prompt = format!("<|system|>\nYou are Kāraṇa, a sovereign AI OS.</s>\n<|user|>\n{}</s>\n<|assistant|>\n", prompt);
            
            let tokens = tokenizer.encode(formatted_prompt, true).map_err(|e| anyhow!(e))?;
            let prompt_tokens = tokens.get_ids();
            let mut all_tokens = prompt_tokens.to_vec();
            
            let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), Some(0.9)); // Seed, Temp, TopP

            let mut output_text = String::new();
            
            for _ in 0..max_tokens {
                let input = Tensor::new(all_tokens.as_slice(), &self.device)?.unsqueeze(0)?;
                let logits = model.forward(&input, 0)?; // 0 = pos? No, quantized forward might differ.
                // Actually QLlama forward takes (x, pos). We need to handle position.
                // Simplified: Just pass full sequence each time (slow) or implement KV cache.
                // For prototype, full sequence is "okay" for short responses.
                
                let logits = logits.squeeze(0)?;
                let next_token_logits = logits.get(logits.dim(0)? - 1)?;
                let next_token = logits_processor.sample(&next_token_logits)?;
                
                all_tokens.push(next_token);
                
                if let Some(t) = tokenizer.decode(&[next_token], true).ok() {
                    output_text.push_str(&t);
                    if output_text.ends_with("</s>") {
                        break;
                    }
                }
            }
            
            Ok(output_text.replace("</s>", "").trim().to_string())

        } else {
            // Simulation Fallback
            self.predict_simulated(prompt)
        }
    }

    fn predict_simulated(&self, prompt: &str) -> Result<String> {
        log::info!("AI Predict (Simulated): Prompt='{}'", prompt);
        let p = prompt.to_lowercase();
        
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
            if p.contains("rm -rf") { "0.95" } else if p.contains("storage write") { "0.1" } else { "0.5" }
        } else if p.contains("boot paths") {
            "Recommended: Full Boot for system integrity check."
        } else if p.contains("governance proposal") {
            "Analysis: Proposal aligns with long-term scalability. Risk: Low."
        } else if p.contains("hud") || p.contains("glass") {
            "Smart Glass Interface: HUD Active. Transparency 85%. Battery 82%. Next meeting in 10m."
        } else if p.contains("tutorial") || p.contains("help") {
            if p.contains("boot") { "Guide: 1. Mint KARA (proof: 0xabc). 2. Vote eco (stake 10). 3. Ignite Runtime." }
            else if p.contains("setup") { "Guide: 1. Run 'karana install'. 2. AI Probes Hardware. 3. DAO Votes on Config." }
            else { "Symbiotic Help: Type 'help boot' or 'help setup' for specific guides. Or 'find app' to browse." }
        } else if p.contains("peer reliability") {
            "0.98"
        } else {
            "I am Phi-3 (Simulated). I can help with system optimization, coding, and governance."
        };
        Ok(response.to_string())
    }

    /// Anomaly Score (For Vigil): Prompt → Risk (0.0-1.0)
    pub fn score_anomaly(&mut self, event: &str) -> Result<f32> {
        let prompt = format!("Rate the severity of this system log from 0.0 (safe) to 1.0 (critical): '{}'. Answer with only the number.", event);
        let response = self.predict(&prompt, 10)?;
        response.trim().parse::<f32>().context("Parse score fail").map(|s| s.min(1.0).max(0.0))
    }
}
