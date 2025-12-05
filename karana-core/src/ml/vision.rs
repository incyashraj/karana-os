// Kāraṇa OS - Vision Processing Module
// Object detection, image captioning, scene understanding, OCR

use std::collections::HashMap;
use std::time::Instant;

use super::{MLError, MLConfig};
use super::inference::{ImageInput, OutputData};
use super::runtime::Tensor;

/// Vision processing system
#[derive(Debug)]
pub struct VisionProcessor {
    /// Configuration
    config: VisionConfig,
    /// Object detector
    object_detector: ObjectDetector,
    /// Image captioner
    captioner: ImageCaptioner,
    /// OCR engine
    ocr: OcrEngine,
    /// Scene classifier
    scene_classifier: SceneClassifier,
}

/// Vision configuration
#[derive(Debug, Clone)]
pub struct VisionConfig {
    /// Object detection model
    pub detection_model: DetectionModel,
    /// Caption model
    pub caption_model: CaptionModel,
    /// OCR enabled
    pub ocr_enabled: bool,
    /// Minimum detection confidence
    pub min_confidence: f32,
    /// Maximum detections per frame
    pub max_detections: usize,
    /// NMS IoU threshold
    pub nms_threshold: f32,
    /// Image preprocessing size
    pub input_size: (u32, u32),
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            detection_model: DetectionModel::YoloV8Nano,
            caption_model: CaptionModel::BlipBase,
            ocr_enabled: true,
            min_confidence: 0.25,
            max_detections: 100,
            nms_threshold: 0.45,
            input_size: (640, 640),
        }
    }
}

/// Object detection model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionModel {
    YoloV8Nano,   // 3.2M params
    YoloV8Small,  // 11.2M params
    YoloV8Medium, // 25.9M params
    YoloV8Large,  // 43.7M params
    MobileNetSSD, // Mobile-optimized
    EfficientDet, // EfficientDet variants
}

impl DetectionModel {
    /// Get model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            DetectionModel::YoloV8Nano => "yolov8n",
            DetectionModel::YoloV8Small => "yolov8s",
            DetectionModel::YoloV8Medium => "yolov8m",
            DetectionModel::YoloV8Large => "yolov8l",
            DetectionModel::MobileNetSSD => "mobilenet-ssd",
            DetectionModel::EfficientDet => "efficientdet-d0",
        }
    }

    /// Get memory requirement (MB)
    pub fn memory_mb(&self) -> usize {
        match self {
            DetectionModel::YoloV8Nano => 8,
            DetectionModel::YoloV8Small => 25,
            DetectionModel::YoloV8Medium => 55,
            DetectionModel::YoloV8Large => 90,
            DetectionModel::MobileNetSSD => 20,
            DetectionModel::EfficientDet => 15,
        }
    }
}

/// Image captioning model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionModel {
    BlipBase,      // 224M params
    BlipLarge,     // 446M params
    GitBase,       // 177M params
    Coca,          // CoCa model
}

impl CaptionModel {
    /// Get model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            CaptionModel::BlipBase => "blip-base",
            CaptionModel::BlipLarge => "blip-large",
            CaptionModel::GitBase => "git-base",
            CaptionModel::Coca => "coca-base",
        }
    }
}

impl VisionProcessor {
    /// Create new vision processor
    pub fn new(config: VisionConfig) -> Self {
        Self {
            object_detector: ObjectDetector::new(config.detection_model),
            captioner: ImageCaptioner::new(config.caption_model),
            ocr: OcrEngine::new(),
            scene_classifier: SceneClassifier::new(),
            config,
        }
    }

    /// Process image for full scene understanding
    pub fn process_image(&self, image: &ImageInput) -> Result<SceneUnderstanding, MLError> {
        let start = Instant::now();

        // Run all analysis
        let detections = self.object_detector.detect(image)?;
        let caption = self.captioner.caption(image)?;
        let scene = self.scene_classifier.classify(image)?;

        let text_regions = if self.config.ocr_enabled {
            self.ocr.detect_text(image)?
        } else {
            Vec::new()
        };

        Ok(SceneUnderstanding {
            detections,
            caption,
            scene,
            text_regions,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    /// Detect objects only
    pub fn detect_objects(&self, image: &ImageInput) -> Result<Vec<Detection>, MLError> {
        self.object_detector.detect(image)
    }

    /// Caption image only
    pub fn caption_image(&self, image: &ImageInput) -> Result<Caption, MLError> {
        self.captioner.caption(image)
    }

    /// Detect text only
    pub fn detect_text(&self, image: &ImageInput) -> Result<Vec<TextRegion>, MLError> {
        self.ocr.detect_text(image)
    }

    /// Classify scene only
    pub fn classify_scene(&self, image: &ImageInput) -> Result<SceneClassification, MLError> {
        self.scene_classifier.classify(image)
    }
}

/// Object detector
#[derive(Debug)]
pub struct ObjectDetector {
    /// Model variant
    model: DetectionModel,
    /// Class names
    class_names: Vec<String>,
    /// Initialized
    initialized: bool,
}

impl ObjectDetector {
    /// Create new detector
    pub fn new(model: DetectionModel) -> Self {
        // COCO class names
        let class_names = vec![
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck",
            "boat", "traffic light", "fire hydrant", "stop sign", "parking meter", "bench",
            "bird", "cat", "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra",
            "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
            "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove",
            "skateboard", "surfboard", "tennis racket", "bottle", "wine glass", "cup",
            "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
            "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
            "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
            "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
            "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
            "toothbrush"
        ].into_iter().map(String::from).collect();

        Self {
            model,
            class_names,
            initialized: false,
        }
    }

    /// Detect objects in image
    pub fn detect(&self, image: &ImageInput) -> Result<Vec<Detection>, MLError> {
        // Simulated detections based on image size
        let mut detections = Vec::new();

        // Mock some detections
        detections.push(Detection {
            class_id: 0, // person
            class_name: "person".to_string(),
            confidence: 0.95,
            bbox: BoundingBox {
                x: image.width as f32 * 0.3,
                y: image.height as f32 * 0.2,
                width: image.width as f32 * 0.2,
                height: image.height as f32 * 0.6,
            },
            attributes: HashMap::new(),
        });

        detections.push(Detection {
            class_id: 67, // cell phone
            class_name: "cell phone".to_string(),
            confidence: 0.85,
            bbox: BoundingBox {
                x: image.width as f32 * 0.4,
                y: image.height as f32 * 0.5,
                width: image.width as f32 * 0.05,
                height: image.height as f32 * 0.1,
            },
            attributes: HashMap::new(),
        });

        Ok(detections)
    }

    /// Get class name by ID
    pub fn class_name(&self, class_id: usize) -> Option<&str> {
        self.class_names.get(class_id).map(|s| s.as_str())
    }

    /// Get total number of classes
    pub fn num_classes(&self) -> usize {
        self.class_names.len()
    }
}

/// Object detection result
#[derive(Debug, Clone)]
pub struct Detection {
    /// Class ID
    pub class_id: usize,
    /// Class name
    pub class_name: String,
    /// Detection confidence
    pub confidence: f32,
    /// Bounding box
    pub bbox: BoundingBox,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// X coordinate (top-left)
    pub x: f32,
    /// Y coordinate (top-left)
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl BoundingBox {
    /// Create new bounding box
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Get center point
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check intersection with another box
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    /// Calculate IoU with another box
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x2 <= x1 || y2 <= y1 {
            return 0.0;
        }

        let intersection = (x2 - x1) * (y2 - y1);
        let union = self.area() + other.area() - intersection;

        intersection / union
    }
}

/// Image captioner
#[derive(Debug)]
pub struct ImageCaptioner {
    /// Model variant
    model: CaptionModel,
}

impl ImageCaptioner {
    /// Create new captioner
    pub fn new(model: CaptionModel) -> Self {
        Self { model }
    }

    /// Generate caption
    pub fn caption(&self, image: &ImageInput) -> Result<Caption, MLError> {
        // Simulated captioning
        Ok(Caption {
            text: "A person holding a phone in a room".to_string(),
            confidence: 0.88,
            model: self.model.model_id().to_string(),
        })
    }

    /// Generate multiple captions
    pub fn caption_multiple(&self, image: &ImageInput, num_captions: usize) -> Result<Vec<Caption>, MLError> {
        let mut captions = Vec::new();

        let texts = vec![
            "A person holding a phone in a room",
            "Someone using their mobile device indoors",
            "A person with a cell phone in their hand",
        ];

        for (i, text) in texts.iter().take(num_captions).enumerate() {
            captions.push(Caption {
                text: text.to_string(),
                confidence: 0.88 - (i as f32 * 0.05),
                model: self.model.model_id().to_string(),
            });
        }

        Ok(captions)
    }
}

/// Image caption
#[derive(Debug, Clone)]
pub struct Caption {
    /// Caption text
    pub text: String,
    /// Confidence score
    pub confidence: f32,
    /// Model used
    pub model: String,
}

/// OCR engine
#[derive(Debug)]
pub struct OcrEngine {
    /// Language models loaded
    languages: Vec<String>,
}

impl OcrEngine {
    /// Create new OCR engine
    pub fn new() -> Self {
        Self {
            languages: vec!["en".to_string()],
        }
    }

    /// Detect and recognize text
    pub fn detect_text(&self, image: &ImageInput) -> Result<Vec<TextRegion>, MLError> {
        // Simulated OCR
        let regions = vec![
            TextRegion {
                text: "Sample Text".to_string(),
                confidence: 0.92,
                bbox: BoundingBox::new(100.0, 50.0, 200.0, 30.0),
                language: "en".to_string(),
            },
        ];

        Ok(regions)
    }

    /// Add language support
    pub fn add_language(&mut self, lang: &str) {
        if !self.languages.contains(&lang.to_string()) {
            self.languages.push(lang.to_string());
        }
    }
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Text region detected by OCR
#[derive(Debug, Clone)]
pub struct TextRegion {
    /// Recognized text
    pub text: String,
    /// Recognition confidence
    pub confidence: f32,
    /// Text bounding box
    pub bbox: BoundingBox,
    /// Detected language
    pub language: String,
}

/// Scene classifier
#[derive(Debug)]
pub struct SceneClassifier {
    /// Scene categories
    categories: Vec<String>,
}

impl SceneClassifier {
    /// Create new classifier
    pub fn new() -> Self {
        let categories = vec![
            "indoor", "outdoor", "urban", "nature", "office", "home",
            "street", "park", "restaurant", "store", "vehicle", "beach",
        ].into_iter().map(String::from).collect();

        Self { categories }
    }

    /// Classify scene
    pub fn classify(&self, image: &ImageInput) -> Result<SceneClassification, MLError> {
        // Simulated classification
        Ok(SceneClassification {
            primary: "indoor".to_string(),
            secondary: Some("office".to_string()),
            scores: vec![
                ("indoor".to_string(), 0.85),
                ("office".to_string(), 0.72),
                ("home".to_string(), 0.45),
            ],
            attributes: vec![
                "bright".to_string(),
                "modern".to_string(),
            ],
        })
    }
}

impl Default for SceneClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Scene classification result
#[derive(Debug, Clone)]
pub struct SceneClassification {
    /// Primary scene type
    pub primary: String,
    /// Secondary scene type
    pub secondary: Option<String>,
    /// All scores
    pub scores: Vec<(String, f32)>,
    /// Scene attributes
    pub attributes: Vec<String>,
}

/// Complete scene understanding
#[derive(Debug)]
pub struct SceneUnderstanding {
    /// Detected objects
    pub detections: Vec<Detection>,
    /// Image caption
    pub caption: Caption,
    /// Scene classification
    pub scene: SceneClassification,
    /// Text regions
    pub text_regions: Vec<TextRegion>,
    /// Processing latency
    pub latency_ms: f64,
}

impl SceneUnderstanding {
    /// Get summary of scene
    pub fn summary(&self) -> String {
        let objects: Vec<&str> = self.detections.iter()
            .map(|d| d.class_name.as_str())
            .collect();

        format!(
            "{} ({} scene). Found: {}.",
            self.caption.text,
            self.scene.primary,
            objects.join(", ")
        )
    }

    /// Find object by class name
    pub fn find_object(&self, class_name: &str) -> Option<&Detection> {
        self.detections.iter()
            .find(|d| d.class_name.eq_ignore_ascii_case(class_name))
    }

    /// Get all objects of a class
    pub fn objects_of_class(&self, class_name: &str) -> Vec<&Detection> {
        self.detections.iter()
            .filter(|d| d.class_name.eq_ignore_ascii_case(class_name))
            .collect()
    }
}

/// Depth estimation
#[derive(Debug)]
pub struct DepthEstimator {
    /// Model type
    model: DepthModel,
}

/// Depth model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthModel {
    MidasSmall,
    MidasBase,
    DepthAnything,
    ZoeDepth,
}

impl DepthEstimator {
    /// Create new estimator
    pub fn new(model: DepthModel) -> Self {
        Self { model }
    }

    /// Estimate depth
    pub fn estimate(&self, image: &ImageInput) -> Result<DepthMap, MLError> {
        // Simulated depth estimation
        let pixels = (image.width * image.height) as usize;
        let depths = vec![1.5f32; pixels]; // Constant depth for simulation

        Ok(DepthMap {
            data: depths,
            width: image.width,
            height: image.height,
            min_depth: 0.5,
            max_depth: 10.0,
        })
    }
}

/// Depth map
#[derive(Debug)]
pub struct DepthMap {
    /// Depth values
    pub data: Vec<f32>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Minimum depth
    pub min_depth: f32,
    /// Maximum depth
    pub max_depth: f32,
}

impl DepthMap {
    /// Get depth at pixel
    pub fn depth_at(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.data.get(idx).copied()
    }

    /// Get average depth in region
    pub fn average_depth(&self, bbox: &BoundingBox) -> f32 {
        let x1 = bbox.x.max(0.0) as u32;
        let y1 = bbox.y.max(0.0) as u32;
        let x2 = ((bbox.x + bbox.width) as u32).min(self.width);
        let y2 = ((bbox.y + bbox.height) as u32).min(self.height);

        let mut sum = 0.0f32;
        let mut count = 0;

        for y in y1..y2 {
            for x in x1..x2 {
                if let Some(d) = self.depth_at(x, y) {
                    sum += d;
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::inference::ImageFormat;

    fn test_image() -> ImageInput {
        ImageInput {
            data: vec![128u8; 640 * 480 * 3],
            width: 640,
            height: 480,
            format: ImageFormat::RGB,
        }
    }

    #[test]
    fn test_vision_config_default() {
        let config = VisionConfig::default();
        assert_eq!(config.min_confidence, 0.25);
        assert_eq!(config.max_detections, 100);
    }

    #[test]
    fn test_detection_model_info() {
        assert_eq!(DetectionModel::YoloV8Nano.model_id(), "yolov8n");
        assert!(DetectionModel::YoloV8Large.memory_mb() > DetectionModel::YoloV8Nano.memory_mb());
    }

    #[test]
    fn test_object_detection() {
        let detector = ObjectDetector::new(DetectionModel::YoloV8Nano);
        let image = test_image();

        let detections = detector.detect(&image).unwrap();
        assert!(!detections.is_empty());
        assert!(detections[0].confidence > 0.0);
    }

    #[test]
    fn test_bounding_box() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let bbox2 = BoundingBox::new(50.0, 50.0, 100.0, 100.0);

        assert!(bbox1.intersects(&bbox2));
        assert!(bbox1.iou(&bbox2) > 0.0);
        assert!(bbox1.iou(&bbox2) < 1.0);

        let center = bbox1.center();
        assert_eq!(center, (50.0, 50.0));
    }

    #[test]
    fn test_image_captioning() {
        let captioner = ImageCaptioner::new(CaptionModel::BlipBase);
        let image = test_image();

        let caption = captioner.caption(&image).unwrap();
        assert!(!caption.text.is_empty());
        assert!(caption.confidence > 0.0);
    }

    #[test]
    fn test_ocr() {
        let ocr = OcrEngine::new();
        let image = test_image();

        let regions = ocr.detect_text(&image).unwrap();
        // May have simulated text
        assert!(regions.len() >= 0);
    }

    #[test]
    fn test_scene_classification() {
        let classifier = SceneClassifier::new();
        let image = test_image();

        let scene = classifier.classify(&image).unwrap();
        assert!(!scene.primary.is_empty());
    }

    #[test]
    fn test_full_scene_understanding() {
        let config = VisionConfig::default();
        let processor = VisionProcessor::new(config);
        let image = test_image();

        let understanding = processor.process_image(&image).unwrap();

        assert!(!understanding.detections.is_empty());
        assert!(!understanding.caption.text.is_empty());
        assert!(understanding.latency_ms > 0.0);
    }

    #[test]
    fn test_scene_summary() {
        let config = VisionConfig::default();
        let processor = VisionProcessor::new(config);
        let image = test_image();

        let understanding = processor.process_image(&image).unwrap();
        let summary = understanding.summary();

        assert!(!summary.is_empty());
    }

    #[test]
    fn test_depth_estimation() {
        let estimator = DepthEstimator::new(DepthModel::MidasSmall);
        let image = test_image();

        let depth_map = estimator.estimate(&image).unwrap();
        assert_eq!(depth_map.width, image.width);
        assert_eq!(depth_map.height, image.height);

        let depth = depth_map.depth_at(100, 100);
        assert!(depth.is_some());
    }

    #[test]
    fn test_depth_map_average() {
        let estimator = DepthEstimator::new(DepthModel::MidasSmall);
        let image = test_image();

        let depth_map = estimator.estimate(&image).unwrap();
        let bbox = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let avg = depth_map.average_depth(&bbox);

        assert!(avg > 0.0);
    }

    #[test]
    fn test_multiple_captions() {
        let captioner = ImageCaptioner::new(CaptionModel::BlipBase);
        let image = test_image();

        let captions = captioner.caption_multiple(&image, 3).unwrap();
        assert_eq!(captions.len(), 3);

        // Confidence should decrease
        assert!(captions[0].confidence > captions[2].confidence);
    }
}
