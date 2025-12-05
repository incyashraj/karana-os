// Kāraṇa OS - ML Pipeline
// Multi-stage inference pipelines for complex tasks

use std::collections::HashMap;
use std::time::Instant;

use super::{MLSystem, MLError, MLConfig};
use super::inference::{InferenceRequest, InferenceResult, InputData, OutputData, ImageInput, AudioInput};
use super::models::ModelType;
use super::runtime::Tensor;

/// ML Pipeline for multi-model inference
#[derive(Debug)]
pub struct MLPipeline {
    /// Pipeline name
    name: String,
    /// Pipeline stages
    stages: Vec<PipelineStage>,
    /// ML system reference (would be Arc<Mutex<MLSystem>> in real impl)
    config: MLConfig,
    /// Is initialized
    initialized: bool,
}

/// Pipeline stage
#[derive(Debug, Clone)]
pub struct PipelineStage {
    /// Stage name
    pub name: String,
    /// Model ID to use
    pub model_id: String,
    /// Input mapping (output name -> input name)
    pub input_mapping: HashMap<String, String>,
    /// Optional condition
    pub condition: Option<StageCondition>,
    /// Stage type
    pub stage_type: StageType,
}

/// Stage type
#[derive(Debug, Clone)]
pub enum StageType {
    /// Always run
    Required,
    /// Run if previous output meets condition
    Conditional,
    /// Run in parallel with previous
    Parallel,
    /// Fallback if previous fails
    Fallback,
}

/// Stage condition
#[derive(Debug, Clone)]
pub enum StageCondition {
    /// Run if score above threshold
    ScoreAbove { output: String, threshold: f32 },
    /// Run if score below threshold
    ScoreBelow { output: String, threshold: f32 },
    /// Run if output contains class
    HasClass { output: String, class_id: i32 },
    /// Run if text matches pattern
    TextMatches { output: String, pattern: String },
    /// Always run
    Always,
}

impl MLPipeline {
    /// Create new pipeline
    pub fn new(name: &str, config: MLConfig) -> Self {
        Self {
            name: name.to_string(),
            stages: Vec::new(),
            config,
            initialized: false,
        }
    }

    /// Add stage
    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
    }

    /// Create pipeline with builder pattern
    pub fn builder(name: &str) -> PipelineBuilder {
        PipelineBuilder::new(name)
    }

    /// Run pipeline
    pub fn run(&self, initial_inputs: HashMap<String, InputData>) -> Result<PipelineResult, MLError> {
        let start = Instant::now();
        let mut context = PipelineContext::new(initial_inputs);
        let mut stage_results = Vec::new();

        for (idx, stage) in self.stages.iter().enumerate() {
            // Check condition
            if let Some(ref condition) = stage.condition {
                if !self.check_condition(condition, &context) {
                    stage_results.push(StageResult {
                        stage_name: stage.name.clone(),
                        skipped: true,
                        latency_ms: 0.0,
                        outputs: HashMap::new(),
                    });
                    continue;
                }
            }

            // Build inputs for this stage
            let stage_inputs = self.build_stage_inputs(stage, &context)?;

            // Run inference (simulated)
            let stage_start = Instant::now();
            let outputs = self.run_stage(stage, stage_inputs)?;
            let stage_latency = stage_start.elapsed().as_secs_f64() * 1000.0;

            // Store outputs in context
            for (name, data) in outputs.iter() {
                context.set_output(&format!("{}:{}", stage.name, name), data.clone());
            }

            stage_results.push(StageResult {
                stage_name: stage.name.clone(),
                skipped: false,
                latency_ms: stage_latency,
                outputs,
            });
        }

        Ok(PipelineResult {
            pipeline_name: self.name.clone(),
            total_latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            stage_results,
            final_outputs: context.outputs,
        })
    }

    fn check_condition(&self, condition: &StageCondition, context: &PipelineContext) -> bool {
        match condition {
            StageCondition::Always => true,
            StageCondition::ScoreAbove { output, threshold } => {
                context.get_score(output).map(|s| s > *threshold).unwrap_or(false)
            }
            StageCondition::ScoreBelow { output, threshold } => {
                context.get_score(output).map(|s| s < *threshold).unwrap_or(false)
            }
            StageCondition::HasClass { output, class_id } => {
                context.get_classes(output)
                    .map(|classes| classes.contains(class_id))
                    .unwrap_or(false)
            }
            StageCondition::TextMatches { output, pattern } => {
                context.get_text(output)
                    .map(|text| text.contains(pattern))
                    .unwrap_or(false)
            }
        }
    }

    fn build_stage_inputs(&self, stage: &PipelineStage, context: &PipelineContext) -> Result<HashMap<String, InputData>, MLError> {
        let mut inputs = HashMap::new();

        // Map inputs from context
        for (output_name, input_name) in &stage.input_mapping {
            if let Some(data) = context.outputs.get(output_name) {
                // Convert OutputData back to InputData
                let input = match data {
                    OutputData::Text(s) => InputData::Text(s.clone()),
                    OutputData::Embedding(e) => InputData::Tensor(Tensor::Float32(e.clone())),
                    OutputData::Raw(t) => InputData::Tensor(t.clone()),
                    _ => continue,
                };
                inputs.insert(input_name.clone(), input);
            }
        }

        // Also include original inputs if not overridden
        for (name, data) in &context.initial_inputs {
            if !inputs.contains_key(name) {
                inputs.insert(name.clone(), data.clone());
            }
        }

        Ok(inputs)
    }

    fn run_stage(&self, stage: &PipelineStage, _inputs: HashMap<String, InputData>) -> Result<HashMap<String, OutputData>, MLError> {
        // Simulated stage execution
        // In real implementation, would call MLSystem.infer()
        
        let mut outputs = HashMap::new();

        // Generate mock outputs based on model type
        match stage.model_id.as_str() {
            "whisper-tiny" => {
                outputs.insert("text".to_string(), OutputData::Text("Transcribed speech".to_string()));
            }
            "blip-image-caption" => {
                outputs.insert("caption".to_string(), OutputData::Text("A person in a room".to_string()));
            }
            "intent-classifier" => {
                outputs.insert("intent".to_string(), OutputData::Text("query_weather".to_string()));
                outputs.insert("confidence".to_string(), OutputData::Score(0.95));
            }
            "minilm-l6" => {
                outputs.insert("embedding".to_string(), OutputData::Embedding(vec![0.1; 384]));
            }
            "mediapipe-hands" => {
                outputs.insert("landmarks".to_string(), OutputData::Landmarks(vec![]));
                outputs.insert("confidence".to_string(), OutputData::Score(0.92));
            }
            "gaze-estimation" => {
                outputs.insert("gaze_x".to_string(), OutputData::Score(0.1));
                outputs.insert("gaze_y".to_string(), OutputData::Score(0.2));
            }
            _ => {
                outputs.insert("output".to_string(), OutputData::Score(1.0));
            }
        }

        Ok(outputs)
    }

    /// Get pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get stage count
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

/// Pipeline context during execution
#[derive(Debug)]
struct PipelineContext {
    initial_inputs: HashMap<String, InputData>,
    outputs: HashMap<String, OutputData>,
}

impl PipelineContext {
    fn new(initial_inputs: HashMap<String, InputData>) -> Self {
        // Clone initial inputs into a new map with the right types
        let mut inputs = HashMap::new();
        for (k, v) in initial_inputs {
            inputs.insert(k, v);
        }
        
        Self {
            initial_inputs: inputs,
            outputs: HashMap::new(),
        }
    }

    fn set_output(&mut self, name: &str, data: OutputData) {
        self.outputs.insert(name.to_string(), data);
    }

    fn get_score(&self, name: &str) -> Option<f32> {
        match self.outputs.get(name) {
            Some(OutputData::Score(s)) => Some(*s),
            _ => None,
        }
    }

    fn get_text(&self, name: &str) -> Option<&str> {
        match self.outputs.get(name) {
            Some(OutputData::Text(s)) => Some(s),
            _ => None,
        }
    }

    fn get_classes(&self, name: &str) -> Option<&[i32]> {
        match self.outputs.get(name) {
            Some(OutputData::Classes(c)) => Some(c),
            _ => None,
        }
    }
}

// Need to implement Clone for InputData
impl Clone for InputData {
    fn clone(&self) -> Self {
        match self {
            InputData::Image(i) => InputData::Image(i.clone()),
            InputData::Audio(a) => InputData::Audio(a.clone()),
            InputData::Text(t) => InputData::Text(t.clone()),
            InputData::Tensor(t) => InputData::Tensor(t.clone()),
        }
    }
}

/// Pipeline result
#[derive(Debug)]
pub struct PipelineResult {
    /// Pipeline name
    pub pipeline_name: String,
    /// Total latency
    pub total_latency_ms: f64,
    /// Individual stage results
    pub stage_results: Vec<StageResult>,
    /// Final accumulated outputs
    pub final_outputs: HashMap<String, OutputData>,
}

impl PipelineResult {
    /// Get final text output
    pub fn get_text(&self, name: &str) -> Option<&str> {
        for stage in self.stage_results.iter().rev() {
            if let Some(OutputData::Text(s)) = stage.outputs.get(name) {
                return Some(s);
            }
        }
        None
    }

    /// Get final score output
    pub fn get_score(&self, name: &str) -> Option<f32> {
        for stage in self.stage_results.iter().rev() {
            if let Some(OutputData::Score(s)) = stage.outputs.get(name) {
                return Some(*s);
            }
        }
        None
    }

    /// Check if any stage was skipped
    pub fn has_skipped_stages(&self) -> bool {
        self.stage_results.iter().any(|s| s.skipped)
    }
}

/// Individual stage result
#[derive(Debug)]
pub struct StageResult {
    /// Stage name
    pub stage_name: String,
    /// Was skipped
    pub skipped: bool,
    /// Latency
    pub latency_ms: f64,
    /// Outputs
    pub outputs: HashMap<String, OutputData>,
}

/// Pipeline builder
pub struct PipelineBuilder {
    name: String,
    stages: Vec<PipelineStage>,
}

impl PipelineBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stages: Vec::new(),
        }
    }

    /// Add required stage
    pub fn stage(mut self, name: &str, model_id: &str) -> Self {
        self.stages.push(PipelineStage {
            name: name.to_string(),
            model_id: model_id.to_string(),
            input_mapping: HashMap::new(),
            condition: None,
            stage_type: StageType::Required,
        });
        self
    }

    /// Add conditional stage
    pub fn conditional_stage(mut self, name: &str, model_id: &str, condition: StageCondition) -> Self {
        self.stages.push(PipelineStage {
            name: name.to_string(),
            model_id: model_id.to_string(),
            input_mapping: HashMap::new(),
            condition: Some(condition),
            stage_type: StageType::Conditional,
        });
        self
    }

    /// Map input from previous stage
    pub fn map_input(mut self, from: &str, to: &str) -> Self {
        if let Some(stage) = self.stages.last_mut() {
            stage.input_mapping.insert(from.to_string(), to.to_string());
        }
        self
    }

    /// Build the pipeline
    pub fn build(self, config: MLConfig) -> MLPipeline {
        let mut pipeline = MLPipeline::new(&self.name, config);
        for stage in self.stages {
            pipeline.add_stage(stage);
        }
        pipeline
    }
}

// Pre-built pipelines for common tasks

/// Create speech understanding pipeline
pub fn speech_understanding_pipeline(config: MLConfig) -> MLPipeline {
    MLPipeline::builder("speech-understanding")
        .stage("transcribe", "whisper-tiny")
        .stage("classify-intent", "intent-classifier")
            .map_input("transcribe:text", "input")
        .stage("embed-text", "minilm-l6")
            .map_input("transcribe:text", "input")
        .build(config)
}

/// Create visual understanding pipeline
pub fn visual_understanding_pipeline(config: MLConfig) -> MLPipeline {
    MLPipeline::builder("visual-understanding")
        .stage("caption", "blip-image-caption")
        .stage("detect-objects", "yolov8-nano")
        .stage("detect-hands", "mediapipe-hands")
        .stage("estimate-gaze", "gaze-estimation")
        .build(config)
}

/// Create multimodal understanding pipeline
pub fn multimodal_pipeline(config: MLConfig) -> MLPipeline {
    MLPipeline::builder("multimodal")
        // Audio branch
        .stage("transcribe", "whisper-tiny")
        // Visual branch
        .stage("caption", "blip-image-caption")
        .stage("detect-hands", "mediapipe-hands")
        // Fusion
        .stage("embed-text", "minilm-l6")
            .map_input("transcribe:text", "text_input")
        .stage("classify-intent", "intent-classifier")
            .map_input("transcribe:text", "input")
        .build(config)
}

/// Create gesture recognition pipeline
pub fn gesture_pipeline(config: MLConfig) -> MLPipeline {
    MLPipeline::builder("gesture-recognition")
        .stage("detect-hands", "mediapipe-hands")
        .conditional_stage(
            "track-fingers",
            "mediapipe-hands",
            StageCondition::ScoreAbove {
                output: "detect-hands:confidence".to_string(),
                threshold: 0.7,
            }
        )
        .build(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let config = MLConfig::default();
        let pipeline = MLPipeline::new("test", config);
        assert_eq!(pipeline.name(), "test");
        assert_eq!(pipeline.stage_count(), 0);
    }

    #[test]
    fn test_pipeline_builder() {
        let config = MLConfig::default();
        let pipeline = MLPipeline::builder("test")
            .stage("stage1", "model1")
            .stage("stage2", "model2")
            .build(config);

        assert_eq!(pipeline.stage_count(), 2);
    }

    #[test]
    fn test_pipeline_run() {
        let config = MLConfig::default();
        let pipeline = MLPipeline::builder("test")
            .stage("transcribe", "whisper-tiny")
            .build(config);

        let inputs = HashMap::new();
        let result = pipeline.run(inputs);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.stage_results.len(), 1);
    }

    #[test]
    fn test_conditional_stage() {
        let config = MLConfig::default();
        let pipeline = MLPipeline::builder("test")
            .stage("first", "intent-classifier")
            .conditional_stage(
                "second",
                "minilm-l6",
                StageCondition::ScoreAbove {
                    output: "first:confidence".to_string(),
                    threshold: 0.9,
                }
            )
            .build(config);

        let inputs = HashMap::new();
        let result = pipeline.run(inputs).unwrap();

        // Second stage should run because first stage outputs confidence 0.95
        assert!(!result.stage_results[1].skipped);
    }

    #[test]
    fn test_speech_pipeline() {
        let config = MLConfig::default();
        let pipeline = speech_understanding_pipeline(config);

        assert_eq!(pipeline.name(), "speech-understanding");
        assert!(pipeline.stage_count() >= 3);
    }

    #[test]
    fn test_visual_pipeline() {
        let config = MLConfig::default();
        let pipeline = visual_understanding_pipeline(config);

        assert_eq!(pipeline.name(), "visual-understanding");
        assert!(pipeline.stage_count() >= 4);
    }

    #[test]
    fn test_multimodal_pipeline() {
        let config = MLConfig::default();
        let pipeline = multimodal_pipeline(config);

        assert_eq!(pipeline.name(), "multimodal");
    }

    #[test]
    fn test_pipeline_latency_tracking() {
        let config = MLConfig::default();
        let pipeline = MLPipeline::builder("test")
            .stage("s1", "whisper-tiny")
            .stage("s2", "intent-classifier")
            .build(config);

        let inputs = HashMap::new();
        let result = pipeline.run(inputs).unwrap();

        assert!(result.total_latency_ms > 0.0);
        for stage in &result.stage_results {
            assert!(stage.latency_ms >= 0.0);
        }
    }
}
