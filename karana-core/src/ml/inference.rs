// Kāraṇa OS - ML Inference Engine
// High-level inference API

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::runtime::{MLRuntime, RuntimeConfig, Tensor, RuntimeError};
use super::models::ModelInfo;
use super::{MLConfig, MLError};

/// Inference engine for a specific model
#[derive(Debug)]
pub struct InferenceEngine {
    /// Model information
    model_info: ModelInfo,
    /// Runtime (would hold actual ONNX session)
    runtime: MLRuntime,
    /// Preprocessors
    preprocessors: Vec<Box<dyn Preprocessor>>,
    /// Postprocessors
    postprocessors: Vec<Box<dyn Postprocessor>>,
    /// Statistics
    stats: InferenceStats,
    /// Warm-up done
    warmed_up: bool,
}

/// Inference statistics
#[derive(Debug, Default, Clone)]
pub struct InferenceStats {
    /// Total inferences
    pub total: u64,
    /// Successful inferences
    pub successful: u64,
    /// Total preprocess time
    pub preprocess_time_ms: f64,
    /// Total inference time
    pub inference_time_ms: f64,
    /// Total postprocess time
    pub postprocess_time_ms: f64,
    /// Min latency
    pub min_latency_ms: f64,
    /// Max latency
    pub max_latency_ms: f64,
    /// Avg latency
    pub avg_latency_ms: f64,
}

impl InferenceEngine {
    /// Create new inference engine
    pub fn new(model_info: ModelInfo, config: &MLConfig) -> Result<Self, MLError> {
        let runtime_config = RuntimeConfig {
            thread_count: config.thread_count,
            ..Default::default()
        };

        let mut runtime = MLRuntime::new(runtime_config);

        // Create session
        runtime.create_session(&model_info.id, &model_info.path)
            .map_err(|e| MLError::LoadError(e.to_string()))?;

        Ok(Self {
            model_info,
            runtime,
            preprocessors: Vec::new(),
            postprocessors: Vec::new(),
            stats: InferenceStats::default(),
            warmed_up: false,
        })
    }

    /// Run inference
    pub fn run(&mut self, request: InferenceRequest) -> Result<InferenceResult, MLError> {
        let start = Instant::now();
        self.stats.total += 1;

        // Preprocess
        let preprocess_start = Instant::now();
        let inputs = self.preprocess(request.inputs)?;
        let preprocess_time = preprocess_start.elapsed().as_secs_f64() * 1000.0;

        // Run inference
        let inference_start = Instant::now();
        let raw_outputs = self.runtime.run(&self.model_info.id, inputs)
            .map_err(|e| MLError::InferenceError(e.to_string()))?;
        let inference_time = inference_start.elapsed().as_secs_f64() * 1000.0;

        // Postprocess
        let postprocess_start = Instant::now();
        let outputs = self.postprocess(raw_outputs)?;
        let postprocess_time = postprocess_start.elapsed().as_secs_f64() * 1000.0;

        let total_time = start.elapsed().as_secs_f64() * 1000.0;

        // Update stats
        self.stats.successful += 1;
        self.stats.preprocess_time_ms += preprocess_time;
        self.stats.inference_time_ms += inference_time;
        self.stats.postprocess_time_ms += postprocess_time;

        if self.stats.min_latency_ms == 0.0 || total_time < self.stats.min_latency_ms {
            self.stats.min_latency_ms = total_time;
        }
        if total_time > self.stats.max_latency_ms {
            self.stats.max_latency_ms = total_time;
        }
        self.stats.avg_latency_ms = 
            (self.stats.preprocess_time_ms + self.stats.inference_time_ms + self.stats.postprocess_time_ms) 
            / self.stats.successful as f64;

        Ok(InferenceResult {
            outputs,
            latency_ms: total_time,
            model_id: self.model_info.id.clone(),
        })
    }

    fn preprocess(&self, inputs: HashMap<String, InputData>) -> Result<HashMap<String, Tensor>, MLError> {
        let mut tensors = HashMap::new();

        for (name, data) in inputs {
            let tensor = match data {
                InputData::Image(img) => self.preprocess_image(&img)?,
                InputData::Audio(audio) => self.preprocess_audio(&audio)?,
                InputData::Text(text) => self.preprocess_text(&text)?,
                InputData::Tensor(t) => t,
            };
            tensors.insert(name, tensor);
        }

        Ok(tensors)
    }

    fn preprocess_image(&self, image: &ImageInput) -> Result<Tensor, MLError> {
        // Normalize to [-1, 1] or [0, 1] based on model requirements
        let normalized: Vec<f32> = image.data.iter()
            .map(|&v| v as f32 / 255.0)
            .collect();

        Ok(Tensor::Float32(normalized))
    }

    fn preprocess_audio(&self, audio: &AudioInput) -> Result<Tensor, MLError> {
        // Audio is already normalized f32 samples
        Ok(Tensor::Float32(audio.data.clone()))
    }

    fn preprocess_text(&self, text: &str) -> Result<Tensor, MLError> {
        // Would use tokenizer to convert to token IDs
        // For now, simple character encoding
        let tokens: Vec<i32> = text.chars()
            .take(128) // Max length
            .map(|c| c as i32)
            .collect();

        Ok(Tensor::Int32(tokens))
    }

    fn postprocess(&self, outputs: HashMap<String, Tensor>) -> Result<HashMap<String, OutputData>, MLError> {
        let mut results = HashMap::new();

        for (name, tensor) in outputs {
            let data = match &tensor {
                Tensor::Float32(v) => {
                    if v.len() == 1 {
                        OutputData::Score(v[0])
                    } else {
                        OutputData::Embedding(v.clone())
                    }
                }
                Tensor::String(s) => OutputData::Text(s.clone()),
                Tensor::Int32(v) => OutputData::Classes(v.clone()),
                _ => OutputData::Raw(tensor),
            };
            results.insert(name, data);
        }

        Ok(results)
    }

    /// Warm up the model
    pub fn warmup(&mut self) -> Result<(), MLError> {
        if self.warmed_up {
            return Ok(());
        }

        // Run a few dummy inferences to warm up
        for _ in 0..3 {
            let dummy_inputs = self.create_dummy_inputs();
            let _ = self.runtime.run(&self.model_info.id, dummy_inputs);
        }

        self.warmed_up = true;
        Ok(())
    }

    fn create_dummy_inputs(&self) -> HashMap<String, Tensor> {
        let mut inputs = HashMap::new();

        for (name, shape) in &self.model_info.input_shapes {
            let size: i64 = shape.iter()
                .map(|&d| if d < 0 { 1 } else { d })
                .product();
            inputs.insert(name.clone(), Tensor::Float32(vec![0.0; size as usize]));
        }

        inputs
    }

    /// Get statistics
    pub fn stats(&self) -> &InferenceStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = InferenceStats::default();
    }

    /// Get model info
    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

/// Inference request
#[derive(Debug)]
pub struct InferenceRequest {
    /// Input data
    pub inputs: HashMap<String, InputData>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

impl InferenceRequest {
    /// Create new request
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add image input
    pub fn with_image(mut self, name: &str, image: ImageInput) -> Self {
        self.inputs.insert(name.to_string(), InputData::Image(image));
        self
    }

    /// Add audio input
    pub fn with_audio(mut self, name: &str, audio: AudioInput) -> Self {
        self.inputs.insert(name.to_string(), InputData::Audio(audio));
        self
    }

    /// Add text input
    pub fn with_text(mut self, name: &str, text: &str) -> Self {
        self.inputs.insert(name.to_string(), InputData::Text(text.to_string()));
        self
    }

    /// Add raw tensor input
    pub fn with_tensor(mut self, name: &str, tensor: Tensor) -> Self {
        self.inputs.insert(name.to_string(), InputData::Tensor(tensor));
        self
    }
}

impl Default for InferenceRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Input data types
#[derive(Debug)]
pub enum InputData {
    Image(ImageInput),
    Audio(AudioInput),
    Text(String),
    Tensor(Tensor),
}

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGB,
    RGBA,
    BGR,
    Grayscale,
}

/// Image input
#[derive(Debug, Clone)]
pub struct ImageInput {
    /// Raw pixel data (RGB or RGBA)
    pub data: Vec<u8>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Image format
    pub format: ImageFormat,
}

impl ImageInput {
    /// Get channels count
    pub fn channels(&self) -> u8 {
        match self.format {
            ImageFormat::RGB | ImageFormat::BGR => 3,
            ImageFormat::RGBA => 4,
            ImageFormat::Grayscale => 1,
        }
    }

    /// Create from RGB data
    pub fn from_rgb(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            format: ImageFormat::RGB,
        }
    }

    /// Create from RGBA data
    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            format: ImageFormat::RGBA,
        }
    }

    /// Resize image (simple nearest neighbor)
    pub fn resize(&self, new_width: u32, new_height: u32) -> Self {
        let channels = self.channels() as u32;
        let mut new_data = vec![0u8; (new_width * new_height * channels) as usize];
        
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x * self.width / new_width) as usize;
                let src_y = (y * self.height / new_height) as usize;
                
                for c in 0..channels as usize {
                    let src_idx = (src_y * self.width as usize + src_x) * channels as usize + c;
                    let dst_idx = (y as usize * new_width as usize + x as usize) * channels as usize + c;
                    
                    if src_idx < self.data.len() && dst_idx < new_data.len() {
                        new_data[dst_idx] = self.data[src_idx];
                    }
                }
            }
        }

        Self {
            data: new_data,
            width: new_width,
            height: new_height,
            format: self.format,
        }
    }
}

/// Audio input
#[derive(Debug, Clone)]
pub struct AudioInput {
    /// Audio samples as f32 (normalized -1.0 to 1.0)
    pub data: Vec<f32>,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
}

impl AudioInput {
    /// Create from f32 samples
    pub fn new(data: Vec<f32>, sample_rate: u32, channels: u8) -> Self {
        Self { data, sample_rate, channels }
    }

    /// Create from i16 samples (auto-converts to f32)
    pub fn from_i16(samples: Vec<i16>, sample_rate: u32) -> Self {
        let data: Vec<f32> = samples.iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();
        Self { data, sample_rate, channels: 1 }
    }

    /// Duration in seconds
    pub fn duration(&self) -> f32 {
        self.data.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }

    /// Resample to target rate (simple linear interpolation)
    pub fn resample(&self, target_rate: u32) -> Self {
        if target_rate == self.sample_rate {
            return self.clone();
        }

        let ratio = self.sample_rate as f64 / target_rate as f64;
        let new_len = (self.data.len() as f64 / ratio) as usize;
        let mut new_data = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_pos = i as f64 * ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f64;

            let sample = if src_idx + 1 < self.data.len() {
                let s1 = self.data[src_idx] as f64;
                let s2 = self.data[src_idx + 1] as f64;
                (s1 * (1.0 - frac) + s2 * frac) as f32
            } else if src_idx < self.data.len() {
                self.data[src_idx]
            } else {
                0.0
            };

            new_data.push(sample);
        }

        Self {
            data: new_data,
            sample_rate: target_rate,
            channels: self.channels,
        }
    }
}

/// Inference result
#[derive(Debug)]
pub struct InferenceResult {
    /// Output data
    pub outputs: HashMap<String, OutputData>,
    /// Latency in ms
    pub latency_ms: f64,
    /// Model ID
    pub model_id: String,
}

impl InferenceResult {
    /// Get text output
    pub fn get_text(&self, name: &str) -> Option<&str> {
        match self.outputs.get(name) {
            Some(OutputData::Text(s)) => Some(s),
            _ => None,
        }
    }

    /// Get score output
    pub fn get_score(&self, name: &str) -> Option<f32> {
        match self.outputs.get(name) {
            Some(OutputData::Score(s)) => Some(*s),
            _ => None,
        }
    }

    /// Get embedding output
    pub fn get_embedding(&self, name: &str) -> Option<&[f32]> {
        match self.outputs.get(name) {
            Some(OutputData::Embedding(e)) => Some(e),
            _ => None,
        }
    }

    /// Get boxes output (for detection)
    pub fn get_boxes(&self, name: &str) -> Option<&[BoundingBox]> {
        match self.outputs.get(name) {
            Some(OutputData::Boxes(b)) => Some(b),
            _ => None,
        }
    }
}

/// Output data types
#[derive(Debug, Clone)]
pub enum OutputData {
    Text(String),
    Score(f32),
    Scores(Vec<f32>),
    Embedding(Vec<f32>),
    Classes(Vec<i32>),
    Boxes(Vec<BoundingBox>),
    Landmarks(Vec<Landmark>),
    Raw(Tensor),
}

/// Bounding box
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub class_id: i32,
    pub label: Option<String>,
}

/// Landmark point
#[derive(Debug, Clone)]
pub struct Landmark {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visibility: f32,
}

/// Preprocessor trait
pub trait Preprocessor: std::fmt::Debug + Send + Sync {
    fn process(&self, input: &InputData) -> Result<Tensor, MLError>;
}

/// Postprocessor trait
pub trait Postprocessor: std::fmt::Debug + Send + Sync {
    fn process(&self, tensor: &Tensor) -> Result<OutputData, MLError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::models::ModelType;

    fn test_model_info() -> ModelInfo {
        ModelInfo {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            model_type: ModelType::TextEmbedding,
            path: "test.onnx".to_string(),
            size_bytes: 1024,
            input_shapes: vec![("input".to_string(), vec![1, 128])],
            output_shapes: vec![("output".to_string(), vec![1, 384])],
            quantized: true,
            version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn test_engine_creation() {
        let config = MLConfig::default();
        let engine = InferenceEngine::new(test_model_info(), &config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_inference_request_builder() {
        let request = InferenceRequest::new()
            .with_text("input", "hello world");

        assert!(request.inputs.contains_key("input"));
    }


    #[test]
    fn test_image_input() {
        let data = vec![0u8; 100 * 100 * 3];
        let image = ImageInput::from_rgb(data, 100, 100);

        assert_eq!(image.width, 100);
        assert_eq!(image.height, 100);
        assert_eq!(image.channels(), 3);
    }

    #[test]
    fn test_image_resize() {
        let data = vec![128u8; 100 * 100 * 3];
        let image = ImageInput::from_rgb(data, 100, 100);
        let resized = image.resize(50, 50);

        assert_eq!(resized.width, 50);
        assert_eq!(resized.height, 50);
        assert_eq!(resized.data.len(), 50 * 50 * 3);
    }

    #[test]
    fn test_audio_input() {
        let samples: Vec<f32> = vec![0.0; 16000];
        let audio = AudioInput::new(samples, 16000, 1);

        assert_eq!(audio.duration(), 1.0);
    }

    #[test]
    fn test_audio_resample() {
        let samples: Vec<f32> = vec![0.0; 16000];
        let audio = AudioInput::new(samples, 16000, 1);
        let resampled = audio.resample(8000);

        assert_eq!(resampled.sample_rate, 8000);
        assert!(resampled.data.len() < audio.data.len());
    }

    #[test]
    fn test_inference_stats() {
        let stats = InferenceStats::default();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.successful, 0);
    }
}
