// Embedding generation for RAG queries
// Phase 42: Real vector embeddings using sentence transformers

use anyhow::{Result, anyhow};
use std::path::Path;

/// Embedding generator using sentence transformer models
pub struct EmbeddingGenerator {
    /// Model type
    pub model: EmbeddingModel,
    /// Embedding dimension
    pub dim: usize,
    /// Whether model is loaded
    loaded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModel {
    /// MiniLM-L6-v2 (384 dimensions, fast, good quality)
    MiniLM,
    /// All-MiniLM-L12-v2 (384 dimensions, slower, better quality)
    MiniLMv12,
    /// BGE-small (384 dimensions, optimized for retrieval)
    BGESmall,
    /// Stub model for testing (returns zero vectors)
    Stub,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new(model: EmbeddingModel) -> Result<Self> {
        let dim = match model {
            EmbeddingModel::MiniLM => 384,
            EmbeddingModel::MiniLMv12 => 384,
            EmbeddingModel::BGESmall => 384,
            EmbeddingModel::Stub => 384,
        };

        Ok(Self {
            model,
            dim,
            loaded: false,
        })
    }

    /// Load the model from disk
    pub fn load(&mut self, _model_path: &Path) -> Result<()> {
        // TODO: Implement model loading via ONNX or Candle
        // For now, mark as loaded for stub model
        if self.model == EmbeddingModel::Stub {
            self.loaded = true;
            return Ok(());
        }

        // Real implementation would:
        // 1. Load tokenizer from model_path/tokenizer.json
        // 2. Load ONNX model from model_path/model.onnx
        // 3. Initialize inference session
        
        Err(anyhow!("Real model loading not yet implemented. Use EmbeddingModel::Stub for now."))
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Generate embedding for a text string
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if !self.loaded {
            return Err(anyhow!("Model not loaded. Call load() first."));
        }

        match self.model {
            EmbeddingModel::Stub => {
                // Stub: Generate deterministic pseudo-embedding based on text hash
                Ok(self.stub_embed(text))
            }
            _ => {
                // TODO: Real embedding via ONNX/Candle
                // 1. Tokenize text
                // 2. Run through transformer
                // 3. Mean pooling
                // 4. Normalize
                Err(anyhow!("Real embedding not yet implemented"))
            }
        }
    }

    /// Generate embeddings for a batch of texts (more efficient)
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if !self.loaded {
            return Err(anyhow!("Model not loaded. Call load() first."));
        }

        // For now, just call embed() for each text
        // TODO: Implement true batching for efficiency
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }

    /// Stub embedding: deterministic hash-based pseudo-embedding
    fn stub_embed(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut embedding = vec![0.0; self.dim];
        
        // Generate pseudo-random values based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Use hash to seed deterministic "random" values
        let mut state = hash;
        for i in 0..self.dim {
            // Simple LCG for pseudo-random generation
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let value = ((state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            embedding[i] = value;
        }

        // Normalize to unit length (as real embeddings would be)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        let mut generator = Self::new(EmbeddingModel::Stub).unwrap();
        generator.loaded = true;
        generator
    }
}

/// Compute cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// Compute L2 (Euclidean) distance between two embeddings
pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_generator_creation() {
        let generator = EmbeddingGenerator::new(EmbeddingModel::Stub).unwrap();
        assert_eq!(generator.dim, 384);
        assert!(!generator.is_loaded());
    }

    #[test]
    fn test_stub_embedding() {
        let mut generator = EmbeddingGenerator::new(EmbeddingModel::Stub).unwrap();
        generator.loaded = true;

        let emb = generator.embed("hello world").unwrap();
        assert_eq!(emb.len(), 384);

        // Check normalization (should be unit vector)
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_stub_deterministic() {
        let mut generator = EmbeddingGenerator::new(EmbeddingModel::Stub).unwrap();
        generator.loaded = true;

        let emb1 = generator.embed("test").unwrap();
        let emb2 = generator.embed("test").unwrap();
        assert_eq!(emb1, emb2);
    }

    #[test]
    fn test_stub_different_texts() {
        let mut generator = EmbeddingGenerator::new(EmbeddingModel::Stub).unwrap();
        generator.loaded = true;

        let emb1 = generator.embed("hello").unwrap();
        let emb2 = generator.embed("world").unwrap();
        assert_ne!(emb1, emb2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);

        let d = vec![0.707, 0.707, 0.0];
        let sim = cosine_similarity(&a, &d);
        assert!((sim - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_l2_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((l2_distance(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![3.0, 4.0, 0.0];
        assert!((l2_distance(&a, &c) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_batch_embedding() {
        let mut generator = EmbeddingGenerator::new(EmbeddingModel::Stub).unwrap();
        generator.loaded = true;

        let texts = vec!["hello".to_string(), "world".to_string()];
        let embeddings = generator.embed_batch(&texts).unwrap();
        
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }
}
