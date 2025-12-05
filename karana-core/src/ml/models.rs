// Kāraṇa OS - ML Model Registry
// Model metadata and management

use std::collections::HashMap;

/// Model type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    /// Speech to text (e.g., Whisper)
    SpeechToText,
    /// Text to speech
    TextToSpeech,
    /// Image captioning (e.g., BLIP)
    ImageCaptioning,
    /// Object detection (e.g., YOLO)
    ObjectDetection,
    /// Face detection
    FaceDetection,
    /// Face recognition
    FaceRecognition,
    /// Hand pose estimation
    HandPose,
    /// Body pose estimation
    BodyPose,
    /// Gaze estimation
    GazeEstimation,
    /// Text embedding (e.g., MiniLM)
    TextEmbedding,
    /// Intent classification
    IntentClassification,
    /// Named entity recognition
    NER,
    /// Sentiment analysis
    SentimentAnalysis,
    /// Image classification
    ImageClassification,
    /// Image segmentation
    ImageSegmentation,
    /// Depth estimation
    DepthEstimation,
    /// Optical flow
    OpticalFlow,
    /// Text generation
    TextGeneration,
    /// Custom model
    Custom,
}

impl ModelType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SpeechToText => "Speech to Text",
            Self::TextToSpeech => "Text to Speech",
            Self::ImageCaptioning => "Image Captioning",
            Self::ObjectDetection => "Object Detection",
            Self::FaceDetection => "Face Detection",
            Self::FaceRecognition => "Face Recognition",
            Self::HandPose => "Hand Pose",
            Self::BodyPose => "Body Pose",
            Self::GazeEstimation => "Gaze Estimation",
            Self::TextEmbedding => "Text Embedding",
            Self::IntentClassification => "Intent Classification",
            Self::NER => "Named Entity Recognition",
            Self::SentimentAnalysis => "Sentiment Analysis",
            Self::ImageClassification => "Image Classification",
            Self::ImageSegmentation => "Image Segmentation",
            Self::DepthEstimation => "Depth Estimation",
            Self::OpticalFlow => "Optical Flow",
            Self::TextGeneration => "Text Generation",
            Self::Custom => "Custom",
        }
    }

    /// Get typical input modality
    pub fn input_modality(&self) -> Modality {
        match self {
            Self::SpeechToText => Modality::Audio,
            Self::TextToSpeech => Modality::Text,
            Self::ImageCaptioning => Modality::Image,
            Self::ObjectDetection => Modality::Image,
            Self::FaceDetection => Modality::Image,
            Self::FaceRecognition => Modality::Image,
            Self::HandPose => Modality::Image,
            Self::BodyPose => Modality::Image,
            Self::GazeEstimation => Modality::Image,
            Self::TextEmbedding => Modality::Text,
            Self::IntentClassification => Modality::Text,
            Self::NER => Modality::Text,
            Self::SentimentAnalysis => Modality::Text,
            Self::ImageClassification => Modality::Image,
            Self::ImageSegmentation => Modality::Image,
            Self::DepthEstimation => Modality::Image,
            Self::OpticalFlow => Modality::Video,
            Self::TextGeneration => Modality::Text,
            Self::Custom => Modality::Unknown,
        }
    }

    /// Get typical output modality
    pub fn output_modality(&self) -> Modality {
        match self {
            Self::SpeechToText => Modality::Text,
            Self::TextToSpeech => Modality::Audio,
            Self::ImageCaptioning => Modality::Text,
            Self::ObjectDetection => Modality::Structured,
            Self::FaceDetection => Modality::Structured,
            Self::FaceRecognition => Modality::Embedding,
            Self::HandPose => Modality::Structured,
            Self::BodyPose => Modality::Structured,
            Self::GazeEstimation => Modality::Structured,
            Self::TextEmbedding => Modality::Embedding,
            Self::IntentClassification => Modality::Structured,
            Self::NER => Modality::Structured,
            Self::SentimentAnalysis => Modality::Structured,
            Self::ImageClassification => Modality::Structured,
            Self::ImageSegmentation => Modality::Image,
            Self::DepthEstimation => Modality::Image,
            Self::OpticalFlow => Modality::Structured,
            Self::TextGeneration => Modality::Text,
            Self::Custom => Modality::Unknown,
        }
    }
}

/// Data modality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    Text,
    Audio,
    Image,
    Video,
    Embedding,
    Structured,
    Unknown,
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Unique model identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Model type
    pub model_type: ModelType,
    /// Path to model file
    pub path: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Input tensor shapes
    pub input_shapes: Vec<(String, Vec<i64>)>,
    /// Output tensor shapes
    pub output_shapes: Vec<(String, Vec<i64>)>,
    /// Is quantized
    pub quantized: bool,
    /// Version string
    pub version: String,
}

impl ModelInfo {
    /// Get human-readable size
    pub fn size_human(&self) -> String {
        let bytes = self.size_bytes;
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Check if model is suitable for edge deployment
    pub fn is_edge_suitable(&self) -> bool {
        // Edge suitable if < 100MB and quantized
        self.size_bytes < 100 * 1024 * 1024 && self.quantized
    }

    /// Get input names
    pub fn input_names(&self) -> Vec<&str> {
        self.input_shapes.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Get output names
    pub fn output_names(&self) -> Vec<&str> {
        self.output_shapes.iter().map(|(n, _)| n.as_str()).collect()
    }
}

/// Model registry
#[derive(Debug)]
pub struct ModelRegistry {
    /// Registered models
    models: HashMap<String, ModelInfo>,
    /// Models by type
    by_type: HashMap<ModelType, Vec<String>>,
}

impl ModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            by_type: HashMap::new(),
        }
    }

    /// Register a model
    pub fn register(&mut self, info: ModelInfo) {
        let id = info.id.clone();
        let model_type = info.model_type;

        self.models.insert(id.clone(), info);

        self.by_type
            .entry(model_type)
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// Unregister a model
    pub fn unregister(&mut self, id: &str) -> Option<ModelInfo> {
        if let Some(info) = self.models.remove(id) {
            if let Some(ids) = self.by_type.get_mut(&info.model_type) {
                ids.retain(|i| i != id);
            }
            Some(info)
        } else {
            None
        }
    }

    /// Get model by ID
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.get(id)
    }

    /// Get models by type
    pub fn get_by_type(&self, model_type: ModelType) -> Vec<&ModelInfo> {
        self.by_type
            .get(&model_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.models.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all models
    pub fn all(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// Get model count
    pub fn count(&self) -> usize {
        self.models.len()
    }

    /// Check if model exists
    pub fn contains(&self, id: &str) -> bool {
        self.models.contains_key(id)
    }

    /// Get total size of all models
    pub fn total_size(&self) -> u64 {
        self.models.values().map(|m| m.size_bytes).sum()
    }

    /// Get edge-suitable models
    pub fn edge_suitable(&self) -> Vec<&ModelInfo> {
        self.models.values().filter(|m| m.is_edge_suitable()).collect()
    }

    /// Find best model for a task
    pub fn find_best(&self, model_type: ModelType, max_size: Option<u64>, quantized_only: bool) -> Option<&ModelInfo> {
        self.get_by_type(model_type)
            .into_iter()
            .filter(|m| {
                if quantized_only && !m.quantized {
                    return false;
                }
                if let Some(max) = max_size {
                    if m.size_bytes > max {
                        return false;
                    }
                }
                true
            })
            .min_by_key(|m| m.size_bytes) // Prefer smaller models
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Model download status
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    NotDownloaded,
    Downloading { progress: f32 },
    Downloaded,
    Failed(String),
}

/// Remote model repository
#[derive(Debug)]
pub struct ModelRepository {
    /// Base URL
    base_url: String,
    /// Available models
    available: Vec<RemoteModelInfo>,
    /// Download cache directory
    cache_dir: String,
}

/// Remote model info
#[derive(Debug, Clone)]
pub struct RemoteModelInfo {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Model type
    pub model_type: ModelType,
    /// Download URL
    pub url: String,
    /// File size
    pub size_bytes: u64,
    /// SHA256 checksum
    pub checksum: String,
    /// Description
    pub description: String,
}

impl ModelRepository {
    /// Create new repository
    pub fn new(base_url: &str, cache_dir: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            available: Vec::new(),
            cache_dir: cache_dir.to_string(),
        }
    }

    /// Refresh available models
    pub fn refresh(&mut self) -> Result<(), String> {
        // Would fetch from remote server
        // For now, populate with known models
        self.available = vec![
            RemoteModelInfo {
                id: "whisper-tiny".to_string(),
                name: "Whisper Tiny (English)".to_string(),
                model_type: ModelType::SpeechToText,
                url: format!("{}/whisper-tiny-en.onnx", self.base_url),
                size_bytes: 75 * 1024 * 1024,
                checksum: "abc123...".to_string(),
                description: "Fast speech recognition, English only".to_string(),
            },
            RemoteModelInfo {
                id: "whisper-base".to_string(),
                name: "Whisper Base (Multilingual)".to_string(),
                model_type: ModelType::SpeechToText,
                url: format!("{}/whisper-base.onnx", self.base_url),
                size_bytes: 150 * 1024 * 1024,
                checksum: "def456...".to_string(),
                description: "Medium speech recognition, multilingual".to_string(),
            },
        ];

        Ok(())
    }

    /// Get available models
    pub fn available(&self) -> &[RemoteModelInfo] {
        &self.available
    }

    /// Download a model
    pub fn download(&self, _id: &str, _progress: impl Fn(f32)) -> Result<String, String> {
        // Would actually download the model
        // Returns path to downloaded file
        Ok(format!("{}/model.onnx", self.cache_dir))
    }

    /// Check if model is downloaded
    pub fn is_downloaded(&self, id: &str) -> bool {
        let path = format!("{}/{}.onnx", self.cache_dir, id);
        std::path::Path::new(&path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_model() {
        let mut registry = ModelRegistry::new();

        registry.register(ModelInfo {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            model_type: ModelType::TextEmbedding,
            path: "test.onnx".to_string(),
            size_bytes: 1024,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test-model"));
    }

    #[test]
    fn test_get_by_type() {
        let mut registry = ModelRegistry::new();

        registry.register(ModelInfo {
            id: "model1".to_string(),
            name: "Model 1".to_string(),
            model_type: ModelType::TextEmbedding,
            path: "model1.onnx".to_string(),
            size_bytes: 1024,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        registry.register(ModelInfo {
            id: "model2".to_string(),
            name: "Model 2".to_string(),
            model_type: ModelType::TextEmbedding,
            path: "model2.onnx".to_string(),
            size_bytes: 2048,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: false,
            version: "1.0.0".to_string(),
        });

        let embedders = registry.get_by_type(ModelType::TextEmbedding);
        assert_eq!(embedders.len(), 2);
    }

    #[test]
    fn test_model_size_human() {
        let info = ModelInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            model_type: ModelType::Custom,
            path: "test.onnx".to_string(),
            size_bytes: 75 * 1024 * 1024,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        };

        assert_eq!(info.size_human(), "75.0 MB");
    }

    #[test]
    fn test_edge_suitable() {
        let mut registry = ModelRegistry::new();

        registry.register(ModelInfo {
            id: "small".to_string(),
            name: "Small Model".to_string(),
            model_type: ModelType::Custom,
            path: "small.onnx".to_string(),
            size_bytes: 10 * 1024 * 1024, // 10MB
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        registry.register(ModelInfo {
            id: "large".to_string(),
            name: "Large Model".to_string(),
            model_type: ModelType::Custom,
            path: "large.onnx".to_string(),
            size_bytes: 500 * 1024 * 1024, // 500MB
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: false,
            version: "1.0.0".to_string(),
        });

        let edge = registry.edge_suitable();
        assert_eq!(edge.len(), 1);
        assert_eq!(edge[0].id, "small");
    }

    #[test]
    fn test_find_best() {
        let mut registry = ModelRegistry::new();

        registry.register(ModelInfo {
            id: "big-stt".to_string(),
            name: "Big STT".to_string(),
            model_type: ModelType::SpeechToText,
            path: "big.onnx".to_string(),
            size_bytes: 500 * 1024 * 1024,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        registry.register(ModelInfo {
            id: "small-stt".to_string(),
            name: "Small STT".to_string(),
            model_type: ModelType::SpeechToText,
            path: "small.onnx".to_string(),
            size_bytes: 75 * 1024 * 1024,
            input_shapes: vec![],
            output_shapes: vec![],
            quantized: true,
            version: "1.0.0".to_string(),
        });

        let best = registry.find_best(
            ModelType::SpeechToText,
            Some(100 * 1024 * 1024),
            true
        );

        assert!(best.is_some());
        assert_eq!(best.unwrap().id, "small-stt");
    }

    #[test]
    fn test_model_type_modality() {
        assert_eq!(ModelType::SpeechToText.input_modality(), Modality::Audio);
        assert_eq!(ModelType::SpeechToText.output_modality(), Modality::Text);

        assert_eq!(ModelType::ImageCaptioning.input_modality(), Modality::Image);
        assert_eq!(ModelType::ImageCaptioning.output_modality(), Modality::Text);
    }
}
