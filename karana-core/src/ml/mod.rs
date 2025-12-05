// Kāraṇa OS - Machine Learning Module
// Real AI model loading and inference using ONNX Runtime

pub mod runtime;
pub mod models;
pub mod inference;
pub mod pipeline;
pub mod speech;
pub mod vision;
pub mod hands;
pub mod gaze;

pub use runtime::{MLRuntime, RuntimeConfig, ExecutionProvider};
pub use models::{ModelRegistry, ModelInfo, ModelType};
pub use inference::{InferenceEngine, InferenceRequest, InferenceResult, ImageInput, ImageFormat, AudioInput};
pub use pipeline::{MLPipeline, PipelineStage};
pub use speech::{SpeechRecognizer, SpeechConfig, WhisperModel, TranscriptionResult, CommandRecognizer};
pub use vision::{VisionProcessor, VisionConfig, ObjectDetector, Detection, SceneUnderstanding};
pub use hands::{HandTracker, HandConfig, TrackedHand, Gesture, HandLandmark, GestureRecognizer};
pub use gaze::{GazeTracker, GazeConfig, GazePoint, FaceTracker, FaceData, BlinkEvent};

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Main ML system manager
#[derive(Debug)]
pub struct MLSystem {
    /// Runtime configuration
    config: MLConfig,
    /// Model registry
    registry: ModelRegistry,
    /// Active inference engines
    engines: HashMap<String, InferenceEngine>,
    /// Performance metrics
    metrics: MLMetrics,
    /// Is system initialized
    initialized: bool,
}

/// ML system configuration
#[derive(Debug, Clone)]
pub struct MLConfig {
    /// Models directory
    pub models_dir: String,
    /// Max concurrent inferences
    pub max_concurrent: usize,
    /// Default execution provider
    pub execution_provider: ExecutionProvider,
    /// Enable model caching
    pub enable_caching: bool,
    /// Max memory usage (bytes)
    pub max_memory: u64,
    /// Inference timeout
    pub inference_timeout: Duration,
    /// Enable quantization
    pub enable_quantization: bool,
    /// Thread count for CPU inference
    pub thread_count: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            models_dir: "~/.karana/models".to_string(),
            max_concurrent: 4,
            execution_provider: ExecutionProvider::CPU,
            enable_caching: true,
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB
            inference_timeout: Duration::from_secs(30),
            enable_quantization: true,
            thread_count: 4,
        }
    }
}

/// ML system metrics
#[derive(Debug, Default)]
pub struct MLMetrics {
    /// Total inferences
    pub total_inferences: u64,
    /// Successful inferences
    pub successful: u64,
    /// Failed inferences
    pub failed: u64,
    /// Average latency
    pub avg_latency_ms: f32,
    /// Peak memory usage
    pub peak_memory: u64,
    /// Models loaded
    pub models_loaded: usize,
}

impl MLSystem {
    /// Create new ML system
    pub fn new(config: MLConfig) -> Self {
        Self {
            config,
            registry: ModelRegistry::new(),
            engines: HashMap::new(),
            metrics: MLMetrics::default(),
            initialized: false,
        }
    }

    /// Initialize ML system
    pub fn initialize(&mut self) -> Result<(), MLError> {
        if self.initialized {
            return Ok(());
        }

        // Register default models
        self.register_default_models();

        self.initialized = true;
        Ok(())
    }

    fn register_default_models(&mut self) {
        // Speech-to-text model (Whisper)
        self.registry.register(ModelInfo {
            id: "whisper-tiny".to_string(),
            name: "Whisper Tiny".to_string(),
            model_type: ModelType::SpeechToText,
            path: "whisper-tiny-en.onnx".to_string(),
            size_bytes: 75 * 1024 * 1024, // ~75MB
            input_shapes: vec![("audio".to_string(), vec![-1, 80, 3000])],
            output_shapes: vec![("text".to_string(), vec![-1])],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Vision model (BLIP/CLIP)
        self.registry.register(ModelInfo {
            id: "blip-image-caption".to_string(),
            name: "BLIP Image Captioning".to_string(),
            model_type: ModelType::ImageCaptioning,
            path: "blip-image-captioning.onnx".to_string(),
            size_bytes: 500 * 1024 * 1024, // ~500MB
            input_shapes: vec![("image".to_string(), vec![1, 3, 384, 384])],
            output_shapes: vec![("caption".to_string(), vec![-1])],
            quantized: false,
            version: "1.0.0".to_string(),
        });

        // Embedding model (MiniLM)
        self.registry.register(ModelInfo {
            id: "minilm-l6".to_string(),
            name: "MiniLM L6 v2".to_string(),
            model_type: ModelType::TextEmbedding,
            path: "minilm-l6-v2.onnx".to_string(),
            size_bytes: 22 * 1024 * 1024, // ~22MB
            input_shapes: vec![
                ("input_ids".to_string(), vec![-1, -1]),
                ("attention_mask".to_string(), vec![-1, -1]),
            ],
            output_shapes: vec![("embeddings".to_string(), vec![-1, 384])],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Hand pose model
        self.registry.register(ModelInfo {
            id: "mediapipe-hands".to_string(),
            name: "MediaPipe Hands".to_string(),
            model_type: ModelType::HandPose,
            path: "hand_landmark.onnx".to_string(),
            size_bytes: 12 * 1024 * 1024, // ~12MB
            input_shapes: vec![("image".to_string(), vec![1, 3, 224, 224])],
            output_shapes: vec![
                ("landmarks".to_string(), vec![1, 21, 3]),
                ("confidence".to_string(), vec![1]),
            ],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Face detection model
        self.registry.register(ModelInfo {
            id: "blazeface".to_string(),
            name: "BlazeFace".to_string(),
            model_type: ModelType::FaceDetection,
            path: "blazeface.onnx".to_string(),
            size_bytes: 500 * 1024, // ~500KB
            input_shapes: vec![("image".to_string(), vec![1, 3, 128, 128])],
            output_shapes: vec![
                ("boxes".to_string(), vec![-1, 4]),
                ("scores".to_string(), vec![-1]),
            ],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Object detection
        self.registry.register(ModelInfo {
            id: "yolov8-nano".to_string(),
            name: "YOLOv8 Nano".to_string(),
            model_type: ModelType::ObjectDetection,
            path: "yolov8n.onnx".to_string(),
            size_bytes: 6 * 1024 * 1024, // ~6MB
            input_shapes: vec![("images".to_string(), vec![1, 3, 640, 640])],
            output_shapes: vec![("output0".to_string(), vec![1, 84, 8400])],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Intent classification
        self.registry.register(ModelInfo {
            id: "intent-classifier".to_string(),
            name: "Intent Classifier".to_string(),
            model_type: ModelType::IntentClassification,
            path: "intent-classifier.onnx".to_string(),
            size_bytes: 5 * 1024 * 1024, // ~5MB
            input_shapes: vec![
                ("input_ids".to_string(), vec![-1, 128]),
                ("attention_mask".to_string(), vec![-1, 128]),
            ],
            output_shapes: vec![("logits".to_string(), vec![-1, 50])],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        // Eye tracking / gaze estimation
        self.registry.register(ModelInfo {
            id: "gaze-estimation".to_string(),
            name: "Gaze Estimation".to_string(),
            model_type: ModelType::GazeEstimation,
            path: "gaze-estimation.onnx".to_string(),
            size_bytes: 8 * 1024 * 1024, // ~8MB
            input_shapes: vec![("eye_image".to_string(), vec![1, 3, 64, 64])],
            output_shapes: vec![("gaze_direction".to_string(), vec![1, 2])],
            quantized: true,
            version: "1.0.0".to_string(),
        });
    }

    /// Load a model
    pub fn load_model(&mut self, model_id: &str) -> Result<(), MLError> {
        let model_info = self.registry.get(model_id)
            .ok_or_else(|| MLError::ModelNotFound(model_id.to_string()))?
            .clone();

        if self.engines.contains_key(model_id) {
            return Ok(()); // Already loaded
        }

        let engine = InferenceEngine::new(model_info, &self.config)?;
        self.engines.insert(model_id.to_string(), engine);
        self.metrics.models_loaded += 1;

        Ok(())
    }

    /// Unload a model
    pub fn unload_model(&mut self, model_id: &str) -> Result<(), MLError> {
        if self.engines.remove(model_id).is_some() {
            self.metrics.models_loaded -= 1;
            Ok(())
        } else {
            Err(MLError::ModelNotLoaded(model_id.to_string()))
        }
    }

    /// Run inference
    pub fn infer(&mut self, model_id: &str, request: InferenceRequest) -> Result<InferenceResult, MLError> {
        let start = Instant::now();

        let engine = self.engines.get_mut(model_id)
            .ok_or_else(|| MLError::ModelNotLoaded(model_id.to_string()))?;

        let result = engine.run(request);

        let latency = start.elapsed().as_millis() as f32;
        self.metrics.total_inferences += 1;

        match &result {
            Ok(_) => {
                self.metrics.successful += 1;
                // Update rolling average
                self.metrics.avg_latency_ms = 
                    (self.metrics.avg_latency_ms * (self.metrics.successful - 1) as f32 + latency) 
                    / self.metrics.successful as f32;
            }
            Err(_) => {
                self.metrics.failed += 1;
            }
        }

        result
    }

    /// Get available models
    pub fn available_models(&self) -> Vec<&ModelInfo> {
        self.registry.all()
    }

    /// Get loaded models
    pub fn loaded_models(&self) -> Vec<&str> {
        self.engines.keys().map(|s| s.as_str()).collect()
    }

    /// Get metrics
    pub fn metrics(&self) -> &MLMetrics {
        &self.metrics
    }

    /// Check if model is loaded
    pub fn is_loaded(&self, model_id: &str) -> bool {
        self.engines.contains_key(model_id)
    }
}

/// ML system errors
#[derive(Debug, Clone)]
pub enum MLError {
    ModelNotFound(String),
    ModelNotLoaded(String),
    LoadError(String),
    InferenceError(String),
    InvalidInput(String),
    Timeout,
    OutOfMemory,
    RuntimeError(String),
}

impl std::fmt::Display for MLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelNotFound(id) => write!(f, "Model not found: {}", id),
            Self::ModelNotLoaded(id) => write!(f, "Model not loaded: {}", id),
            Self::LoadError(msg) => write!(f, "Load error: {}", msg),
            Self::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::Timeout => write!(f, "Inference timeout"),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_system_creation() {
        let config = MLConfig::default();
        let system = MLSystem::new(config);
        assert!(!system.initialized);
    }

    #[test]
    fn test_ml_system_initialization() {
        let config = MLConfig::default();
        let mut system = MLSystem::new(config);
        system.initialize().unwrap();
        assert!(system.initialized);
    }

    #[test]
    fn test_default_models_registered() {
        let config = MLConfig::default();
        let mut system = MLSystem::new(config);
        system.initialize().unwrap();
        
        let models = system.available_models();
        assert!(models.len() >= 8);
    }

    #[test]
    fn test_model_registry() {
        let config = MLConfig::default();
        let mut system = MLSystem::new(config);
        system.initialize().unwrap();
        
        // Check specific models are registered
        let whisper = system.registry.get("whisper-tiny");
        assert!(whisper.is_some());
        
        let hands = system.registry.get("mediapipe-hands");
        assert!(hands.is_some());
    }
}
