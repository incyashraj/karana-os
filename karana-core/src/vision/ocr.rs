// OCR (Optical Character Recognition) for Kāraṇa OS
// Handles text detection and recognition in camera frames

use super::*;
use std::collections::HashMap;
use std::time::Instant;

/// Text block orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOrientation {
    Horizontal,
    Vertical,
    Rotated(i16), // degrees
    Unknown,
}

/// Detected text block
#[derive(Debug, Clone)]
pub struct TextBlock {
    pub id: u64,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub orientation: TextOrientation,
    pub language: Option<String>,
    pub lines: Vec<TextLine>,
    pub timestamp: Instant,
}

impl TextBlock {
    pub fn new(text: String, bounding_box: BoundingBox, confidence: f32) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            text,
            bounding_box,
            confidence,
            orientation: TextOrientation::Horizontal,
            language: None,
            lines: Vec::new(),
            timestamp: Instant::now(),
        }
    }
    
    /// Get word count
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
    
    /// Get character count
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }
    
    /// Check if text contains a pattern (case insensitive)
    pub fn contains(&self, pattern: &str) -> bool {
        self.text.to_lowercase().contains(&pattern.to_lowercase())
    }
}

/// A line of text within a block
#[derive(Debug, Clone)]
pub struct TextLine {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub words: Vec<Word>,
}

/// A single word
#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
}

/// OCR configuration
#[derive(Debug, Clone)]
pub struct OCRConfig {
    pub min_confidence: f32,
    pub languages: Vec<String>,
    pub detect_orientation: bool,
    pub detect_paragraphs: bool,
    pub page_segmentation_mode: PageSegmentationMode,
    pub whitelist: Option<String>,
    pub blacklist: Option<String>,
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            languages: vec!["en".to_string()],
            detect_orientation: true,
            detect_paragraphs: true,
            page_segmentation_mode: PageSegmentationMode::Auto,
            whitelist: None,
            blacklist: None,
        }
    }
}

/// Page segmentation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSegmentationMode {
    /// Auto-detect
    Auto,
    /// Single column of text
    SingleColumn,
    /// Single block of text
    SingleBlock,
    /// Single line
    SingleLine,
    /// Single word
    SingleWord,
    /// Sparse text (scattered)
    Sparse,
}

/// OCR engine manager
pub struct OCREngine {
    config: OCRConfig,
    processed_count: u64,
    cache: HashMap<u64, Vec<TextBlock>>,
    max_cache_size: usize,
}

impl OCREngine {
    pub fn new(config: OCRConfig) -> Self {
        Self {
            config,
            processed_count: 0,
            cache: HashMap::new(),
            max_cache_size: 100,
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &OCRConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: OCRConfig) {
        self.config = config;
    }
    
    /// Process frame for text (simulated - real impl would use ML model)
    pub fn process_frame(&mut self, _frame: &CameraFrame) -> Vec<TextBlock> {
        self.processed_count += 1;
        
        // In real implementation, this would:
        // 1. Preprocess image (binarization, deskew)
        // 2. Run text detection model
        // 3. Run text recognition model
        // 4. Post-process results
        
        Vec::new()
    }
    
    /// Process a region of interest
    pub fn process_roi(&mut self, _frame: &CameraFrame, _roi: &ROI) -> Vec<TextBlock> {
        self.processed_count += 1;
        Vec::new()
    }
    
    /// Filter results by confidence
    pub fn filter_results(&self, blocks: Vec<TextBlock>) -> Vec<TextBlock> {
        blocks.into_iter()
            .filter(|b| b.confidence >= self.config.min_confidence)
            .collect()
    }
    
    /// Get total frames processed
    pub fn processed_count(&self) -> u64 {
        self.processed_count
    }
    
    /// Cache results for a frame
    pub fn cache_results(&mut self, frame_id: u64, results: Vec<TextBlock>) {
        if self.cache.len() >= self.max_cache_size {
            // Remove oldest entry
            if let Some(&oldest_key) = self.cache.keys().next() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(frame_id, results);
    }
    
    /// Get cached results
    pub fn get_cached(&self, frame_id: u64) -> Option<&Vec<TextBlock>> {
        self.cache.get(&frame_id)
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Text recognizer for specific patterns
pub struct PatternRecognizer {
    patterns: HashMap<String, TextPattern>,
}

/// A pattern to recognize
#[derive(Debug, Clone)]
pub struct TextPattern {
    pub name: String,
    pub regex: String,
    pub description: String,
    pub priority: u8,
}

impl PatternRecognizer {
    pub fn new() -> Self {
        let mut recognizer = Self {
            patterns: HashMap::new(),
        };
        
        // Add common patterns
        recognizer.add_pattern(TextPattern {
            name: "email".to_string(),
            regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
            description: "Email address".to_string(),
            priority: 5,
        });
        
        recognizer.add_pattern(TextPattern {
            name: "phone".to_string(),
            regex: r"\+?[1-9]\d{1,14}".to_string(),
            description: "Phone number".to_string(),
            priority: 5,
        });
        
        recognizer.add_pattern(TextPattern {
            name: "url".to_string(),
            regex: r"https?://[^\s]+".to_string(),
            description: "Web URL".to_string(),
            priority: 4,
        });
        
        recognizer.add_pattern(TextPattern {
            name: "price".to_string(),
            regex: r"\$\d+(?:\.\d{2})?".to_string(),
            description: "Price".to_string(),
            priority: 3,
        });
        
        recognizer.add_pattern(TextPattern {
            name: "date".to_string(),
            regex: r"\d{1,2}/\d{1,2}/\d{2,4}".to_string(),
            description: "Date".to_string(),
            priority: 3,
        });
        
        recognizer
    }
    
    /// Add a pattern
    pub fn add_pattern(&mut self, pattern: TextPattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }
    
    /// Remove a pattern
    pub fn remove_pattern(&mut self, name: &str) {
        self.patterns.remove(name);
    }
    
    /// Get all patterns
    pub fn patterns(&self) -> &HashMap<String, TextPattern> {
        &self.patterns
    }
    
    /// Check if text matches any pattern
    pub fn match_patterns(&self, text: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        
        for (name, pattern) in &self.patterns {
            // Simplified matching - real impl would use regex crate
            // For now, just check if pattern name keywords exist
            let matched = match name.as_str() {
                "email" => text.contains('@') && text.contains('.'),
                "phone" => text.chars().filter(|c| c.is_ascii_digit()).count() >= 7,
                "url" => text.contains("http://") || text.contains("https://"),
                "price" => text.contains('$') && text.chars().any(|c| c.is_ascii_digit()),
                "date" => text.matches('/').count() == 2,
                _ => false,
            };
            
            if matched {
                matches.push(PatternMatch {
                    pattern_name: name.clone(),
                    matched_text: text.to_string(),
                    confidence: 0.8,
                });
            }
        }
        
        matches
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// A pattern match result
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub matched_text: String,
    pub confidence: f32,
}

/// Real-time text translation support
pub struct TextTranslator {
    source_language: String,
    target_language: String,
    translation_cache: HashMap<String, String>,
}

impl TextTranslator {
    pub fn new(source: &str, target: &str) -> Self {
        Self {
            source_language: source.to_string(),
            target_language: target.to_string(),
            translation_cache: HashMap::new(),
        }
    }
    
    /// Set languages
    pub fn set_languages(&mut self, source: &str, target: &str) {
        self.source_language = source.to_string();
        self.target_language = target.to_string();
        self.translation_cache.clear();
    }
    
    /// Get source language
    pub fn source_language(&self) -> &str {
        &self.source_language
    }
    
    /// Get target language
    pub fn target_language(&self) -> &str {
        &self.target_language
    }
    
    /// Translate text (simulated - real impl would use translation API)
    pub fn translate(&mut self, text: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.translation_cache.get(text) {
            return Some(cached.clone());
        }
        
        // Real implementation would call translation service
        // For now, return None to indicate translation not available
        None
    }
    
    /// Cache a translation
    pub fn cache_translation(&mut self, source: &str, translated: &str) {
        self.translation_cache.insert(source.to_string(), translated.to_string());
    }
    
    /// Clear translation cache
    pub fn clear_cache(&mut self) {
        self.translation_cache.clear();
    }
}

/// Live OCR overlay for AR display
#[derive(Debug, Clone)]
pub struct OCROverlay {
    pub text_blocks: Vec<OverlayTextBlock>,
    pub show_bounding_boxes: bool,
    pub highlight_patterns: bool,
    pub font_size: f32,
    pub opacity: f32,
}

/// Overlay text block for display
#[derive(Debug, Clone)]
pub struct OverlayTextBlock {
    pub original_text: String,
    pub display_text: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub pattern_type: Option<String>,
    pub action_available: bool,
}

impl OCROverlay {
    pub fn new() -> Self {
        Self {
            text_blocks: Vec::new(),
            show_bounding_boxes: true,
            highlight_patterns: true,
            font_size: 14.0,
            opacity: 0.9,
        }
    }
    
    /// Update overlay from OCR results
    pub fn update(&mut self, blocks: &[TextBlock], pattern_recognizer: &PatternRecognizer) {
        self.text_blocks.clear();
        
        for block in blocks {
            let patterns = pattern_recognizer.match_patterns(&block.text);
            let pattern_type = patterns.first().map(|p| p.pattern_name.clone());
            
            let (cx, cy) = block.bounding_box.center();
            
            self.text_blocks.push(OverlayTextBlock {
                original_text: block.text.clone(),
                display_text: block.text.clone(),
                position: (cx, cy),
                size: (block.bounding_box.width, block.bounding_box.height),
                pattern_type: pattern_type.clone(),
                action_available: pattern_type.is_some(),
            });
        }
    }
    
    /// Clear overlay
    pub fn clear(&mut self) {
        self.text_blocks.clear();
    }
}

impl Default for OCROverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_block_creation() {
        let bb = BoundingBox::new(10.0, 10.0, 200.0, 50.0);
        let block = TextBlock::new("Hello World".to_string(), bb, 0.95);
        
        assert_eq!(block.text, "Hello World");
        assert_eq!(block.word_count(), 2);
        assert_eq!(block.char_count(), 11);
        assert_eq!(block.confidence, 0.95);
    }
    
    #[test]
    fn test_text_block_contains() {
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let block = TextBlock::new("Contact: support@example.com".to_string(), bb, 0.9);
        
        assert!(block.contains("support"));
        assert!(block.contains("EXAMPLE")); // Case insensitive
        assert!(!block.contains("hello"));
    }
    
    #[test]
    fn test_ocr_config_default() {
        let config = OCRConfig::default();
        
        assert_eq!(config.min_confidence, 0.6);
        assert!(config.languages.contains(&"en".to_string()));
        assert!(config.detect_orientation);
    }
    
    #[test]
    fn test_ocr_engine() {
        let config = OCRConfig::default();
        let mut engine = OCREngine::new(config);
        
        assert_eq!(engine.processed_count(), 0);
        
        // Process would return empty in simulated mode
        let frame = CameraFrame::new(
            CameraId::Main,
            640, 480,
            FrameFormat::RGB,
            vec![0; 640 * 480 * 3],
        );
        
        let _ = engine.process_frame(&frame);
        assert_eq!(engine.processed_count(), 1);
    }
    
    #[test]
    fn test_ocr_cache() {
        let config = OCRConfig::default();
        let mut engine = OCREngine::new(config);
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let block = TextBlock::new("Test".to_string(), bb, 0.9);
        
        engine.cache_results(1, vec![block]);
        
        assert!(engine.get_cached(1).is_some());
        assert!(engine.get_cached(2).is_none());
        
        engine.clear_cache();
        assert!(engine.get_cached(1).is_none());
    }
    
    #[test]
    fn test_pattern_recognizer_email() {
        let recognizer = PatternRecognizer::new();
        
        let matches = recognizer.match_patterns("Contact: test@example.com");
        
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_name == "email"));
    }
    
    #[test]
    fn test_pattern_recognizer_phone() {
        let recognizer = PatternRecognizer::new();
        
        let matches = recognizer.match_patterns("Call us: 1234567890");
        
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_name == "phone"));
    }
    
    #[test]
    fn test_pattern_recognizer_url() {
        let recognizer = PatternRecognizer::new();
        
        let matches = recognizer.match_patterns("Visit https://example.com");
        
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_name == "url"));
    }
    
    #[test]
    fn test_pattern_recognizer_price() {
        let recognizer = PatternRecognizer::new();
        
        let matches = recognizer.match_patterns("Price: $19.99");
        
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_name == "price"));
    }
    
    #[test]
    fn test_pattern_recognizer_date() {
        let recognizer = PatternRecognizer::new();
        
        let matches = recognizer.match_patterns("Date: 12/25/2024");
        
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.pattern_name == "date"));
    }
    
    #[test]
    fn test_text_translator() {
        let mut translator = TextTranslator::new("en", "es");
        
        assert_eq!(translator.source_language(), "en");
        assert_eq!(translator.target_language(), "es");
        
        // No translation available in simulated mode
        assert!(translator.translate("Hello").is_none());
        
        // Test caching
        translator.cache_translation("Hello", "Hola");
        assert_eq!(translator.translate("Hello"), Some("Hola".to_string()));
    }
    
    #[test]
    fn test_ocr_overlay() {
        let mut overlay = OCROverlay::new();
        let recognizer = PatternRecognizer::new();
        
        let bb = BoundingBox::new(100.0, 100.0, 200.0, 50.0);
        let block = TextBlock::new("test@example.com".to_string(), bb, 0.9);
        
        overlay.update(&[block], &recognizer);
        
        assert_eq!(overlay.text_blocks.len(), 1);
        assert!(overlay.text_blocks[0].action_available);
        assert_eq!(overlay.text_blocks[0].pattern_type, Some("email".to_string()));
    }
    
    #[test]
    fn test_ocr_overlay_clear() {
        let mut overlay = OCROverlay::new();
        let recognizer = PatternRecognizer::new();
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let block = TextBlock::new("Test".to_string(), bb, 0.9);
        
        overlay.update(&[block], &recognizer);
        assert_eq!(overlay.text_blocks.len(), 1);
        
        overlay.clear();
        assert!(overlay.text_blocks.is_empty());
    }
    
    #[test]
    fn test_filter_results() {
        let config = OCRConfig {
            min_confidence: 0.8,
            ..Default::default()
        };
        let engine = OCREngine::new(config);
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let high_conf = TextBlock::new("High".to_string(), bb, 0.9);
        let low_conf = TextBlock::new("Low".to_string(), bb, 0.5);
        
        let filtered = engine.filter_results(vec![high_conf, low_conf]);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].text, "High");
    }
}
