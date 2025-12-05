// QR Code and Barcode Scanning for Kāraṇa OS
// Handles detection and decoding of QR codes, barcodes, and data matrices

use super::*;
use std::collections::HashMap;
use std::time::Instant;

/// Barcode format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarcodeFormat {
    /// QR Code
    QRCode,
    /// Data Matrix
    DataMatrix,
    /// Aztec Code
    Aztec,
    /// PDF417
    PDF417,
    /// UPC-A
    UPCA,
    /// UPC-E
    UPCE,
    /// EAN-8
    EAN8,
    /// EAN-13
    EAN13,
    /// Code 39
    Code39,
    /// Code 93
    Code93,
    /// Code 128
    Code128,
    /// ITF (Interleaved 2 of 5)
    ITF,
    /// Codabar
    Codabar,
    /// Unknown format
    Unknown,
}

impl BarcodeFormat {
    /// Check if format is 2D (QR, DataMatrix, etc.)
    pub fn is_2d(&self) -> bool {
        matches!(self, 
            BarcodeFormat::QRCode |
            BarcodeFormat::DataMatrix |
            BarcodeFormat::Aztec |
            BarcodeFormat::PDF417
        )
    }
    
    /// Check if format is 1D (linear barcode)
    pub fn is_1d(&self) -> bool {
        !self.is_2d() && *self != BarcodeFormat::Unknown
    }
    
    /// Get typical use case
    pub fn typical_use(&self) -> &'static str {
        match self {
            BarcodeFormat::QRCode => "URLs, contact info, general data",
            BarcodeFormat::DataMatrix => "Industrial marking, small items",
            BarcodeFormat::Aztec => "Tickets, boarding passes",
            BarcodeFormat::PDF417 => "ID cards, drivers licenses",
            BarcodeFormat::UPCA | BarcodeFormat::UPCE => "Retail products (US/Canada)",
            BarcodeFormat::EAN8 | BarcodeFormat::EAN13 => "Retail products (International)",
            BarcodeFormat::Code39 => "Logistics, inventory",
            BarcodeFormat::Code93 => "Logistics, Canadian post",
            BarcodeFormat::Code128 => "Shipping, logistics",
            BarcodeFormat::ITF => "Shipping cartons",
            BarcodeFormat::Codabar => "Libraries, blood banks",
            BarcodeFormat::Unknown => "Unknown",
        }
    }
}

/// A scanned barcode/QR code result
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub id: u64,
    pub format: BarcodeFormat,
    pub raw_data: Vec<u8>,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub corner_points: Vec<(f32, f32)>,
    pub confidence: f32,
    pub timestamp: Instant,
    pub content_type: ContentType,
}

impl ScanResult {
    pub fn new(format: BarcodeFormat, text: String, bounding_box: BoundingBox) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        
        let content_type = ContentType::detect(&text);
        
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            format,
            raw_data: text.as_bytes().to_vec(),
            text,
            bounding_box,
            corner_points: Vec::new(),
            confidence: 1.0,
            timestamp: Instant::now(),
            content_type,
        }
    }
    
    /// Set corner points (for 2D codes)
    pub fn with_corners(mut self, corners: Vec<(f32, f32)>) -> Self {
        self.corner_points = corners;
        self
    }
    
    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
    
    /// Check if scan is recent
    pub fn is_recent(&self, max_age_ms: u64) -> bool {
        self.timestamp.elapsed().as_millis() as u64 <= max_age_ms
    }
}

/// Content type detected from scanned data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    /// Web URL
    URL(String),
    /// Email address
    Email(String),
    /// Phone number
    Phone(String),
    /// SMS message
    SMS { number: String, message: String },
    /// WiFi network credentials
    WiFi { ssid: String, password: Option<String>, security: String },
    /// Geographic coordinates
    GeoLocation { lat: String, lon: String },
    /// Contact card (vCard)
    Contact(String),
    /// Calendar event
    CalendarEvent(String),
    /// Plain text
    Text(String),
    /// Product code (UPC/EAN)
    Product(String),
    /// Binary data
    Binary,
}

impl ContentType {
    /// Detect content type from text
    pub fn detect(text: &str) -> Self {
        let text_lower = text.to_lowercase();
        
        // Check for URL
        if text_lower.starts_with("http://") || text_lower.starts_with("https://") {
            return ContentType::URL(text.to_string());
        }
        
        // Check for mailto
        if text_lower.starts_with("mailto:") {
            let email = text[7..].to_string();
            return ContentType::Email(email);
        }
        
        // Check for tel
        if text_lower.starts_with("tel:") {
            let phone = text[4..].to_string();
            return ContentType::Phone(phone);
        }
        
        // Check for SMS
        if text_lower.starts_with("smsto:") || text_lower.starts_with("sms:") {
            let parts: Vec<&str> = text.splitn(2, ':').collect();
            if parts.len() > 1 {
                let content = parts[1];
                let sms_parts: Vec<&str> = content.splitn(2, ':').collect();
                return ContentType::SMS {
                    number: sms_parts.get(0).unwrap_or(&"").to_string(),
                    message: sms_parts.get(1).unwrap_or(&"").to_string(),
                };
            }
        }
        
        // Check for WiFi
        if text_lower.starts_with("wifi:") {
            return Self::parse_wifi(text);
        }
        
        // Check for geo coordinates
        if text_lower.starts_with("geo:") {
            let coords = &text[4..];
            let parts: Vec<&str> = coords.split(',').collect();
            if parts.len() >= 2 {
                return ContentType::GeoLocation {
                    lat: parts[0].to_string(),
                    lon: parts[1].split('?').next().unwrap_or(parts[1]).to_string(),
                };
            }
        }
        
        // Check for vCard
        if text_lower.contains("begin:vcard") {
            return ContentType::Contact(text.to_string());
        }
        
        // Check for calendar event
        if text_lower.contains("begin:vevent") || text_lower.contains("begin:vcalendar") {
            return ContentType::CalendarEvent(text.to_string());
        }
        
        // Check for product code (numeric only)
        if text.chars().all(|c| c.is_ascii_digit()) && (text.len() == 8 || text.len() == 12 || text.len() == 13) {
            return ContentType::Product(text.to_string());
        }
        
        // Default to plain text
        ContentType::Text(text.to_string())
    }
    
    fn parse_wifi(text: &str) -> ContentType {
        let mut ssid = String::new();
        let mut password = None;
        let mut security = "WPA".to_string();
        
        // Parse WIFI:S:ssid;T:security;P:password;;
        for part in text[5..].split(';') {
            if part.starts_with("S:") {
                ssid = part[2..].to_string();
            } else if part.starts_with("P:") {
                password = Some(part[2..].to_string());
            } else if part.starts_with("T:") {
                security = part[2..].to_string();
            }
        }
        
        ContentType::WiFi { ssid, password, security }
    }
    
    /// Check if content type has an actionable intent
    pub fn is_actionable(&self) -> bool {
        matches!(self,
            ContentType::URL(_) |
            ContentType::Email(_) |
            ContentType::Phone(_) |
            ContentType::WiFi { .. } |
            ContentType::GeoLocation { .. }
        )
    }
    
    /// Get action description
    pub fn action_description(&self) -> &'static str {
        match self {
            ContentType::URL(_) => "Open in browser",
            ContentType::Email(_) => "Send email",
            ContentType::Phone(_) => "Make call",
            ContentType::SMS { .. } => "Send SMS",
            ContentType::WiFi { .. } => "Connect to network",
            ContentType::GeoLocation { .. } => "Open in maps",
            ContentType::Contact(_) => "Add contact",
            ContentType::CalendarEvent(_) => "Add to calendar",
            ContentType::Product(_) => "Look up product",
            ContentType::Text(_) => "Copy text",
            ContentType::Binary => "View data",
        }
    }
}

/// QR/Barcode scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub enabled_formats: Vec<BarcodeFormat>,
    pub scan_interval_ms: u32,
    pub auto_focus_on_code: bool,
    pub vibrate_on_scan: bool,
    pub sound_on_scan: bool,
    pub duplicate_filter_ms: u64,
    pub scan_area: Option<ROI>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enabled_formats: vec![
                BarcodeFormat::QRCode,
                BarcodeFormat::DataMatrix,
                BarcodeFormat::EAN13,
                BarcodeFormat::EAN8,
                BarcodeFormat::UPCA,
                BarcodeFormat::Code128,
            ],
            scan_interval_ms: 100,
            auto_focus_on_code: true,
            vibrate_on_scan: true,
            sound_on_scan: false,
            duplicate_filter_ms: 3000,
            scan_area: None,
        }
    }
}

/// Barcode/QR scanner manager
pub struct BarcodeScanner {
    config: ScannerConfig,
    recent_scans: Vec<ScanResult>,
    scan_history: Vec<ScanResult>,
    max_history: usize,
    total_scans: u64,
    callbacks: Vec<Box<dyn Fn(&ScanResult) + Send + Sync>>,
}

impl BarcodeScanner {
    pub fn new(config: ScannerConfig) -> Self {
        Self {
            config,
            recent_scans: Vec::new(),
            scan_history: Vec::new(),
            max_history: 100,
            total_scans: 0,
            callbacks: Vec::new(),
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: ScannerConfig) {
        self.config = config;
    }
    
    /// Check if a format is enabled
    pub fn is_format_enabled(&self, format: BarcodeFormat) -> bool {
        self.config.enabled_formats.contains(&format)
    }
    
    /// Process a scan result
    pub fn process_scan(&mut self, result: ScanResult) -> bool {
        // Check if it's a duplicate
        if self.is_duplicate(&result) {
            return false;
        }
        
        // Check if format is enabled
        if !self.is_format_enabled(result.format) {
            return false;
        }
        
        // Add to recent scans
        self.recent_scans.push(result.clone());
        
        // Add to history
        if self.scan_history.len() >= self.max_history {
            self.scan_history.remove(0);
        }
        self.scan_history.push(result.clone());
        
        self.total_scans += 1;
        
        // Trigger callbacks
        for callback in &self.callbacks {
            callback(&result);
        }
        
        true
    }
    
    /// Check if scan is a duplicate (same content recently scanned)
    fn is_duplicate(&self, result: &ScanResult) -> bool {
        let filter_duration = self.config.duplicate_filter_ms;
        
        self.recent_scans.iter().any(|scan| {
            scan.text == result.text && 
            scan.is_recent(filter_duration)
        })
    }
    
    /// Clean up old recent scans
    pub fn cleanup_recent(&mut self) {
        let filter_duration = self.config.duplicate_filter_ms;
        self.recent_scans.retain(|scan| scan.is_recent(filter_duration));
    }
    
    /// Get scan history
    pub fn history(&self) -> &[ScanResult] {
        &self.scan_history
    }
    
    /// Get recent scans
    pub fn recent_scans(&self) -> &[ScanResult] {
        &self.recent_scans
    }
    
    /// Get total scan count
    pub fn total_scans(&self) -> u64 {
        self.total_scans
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.scan_history.clear();
        self.recent_scans.clear();
    }
    
    /// Get last scan result
    pub fn last_scan(&self) -> Option<&ScanResult> {
        self.scan_history.last()
    }
}

/// Product database for barcode lookups
pub struct ProductDatabase {
    products: HashMap<String, ProductInfo>,
    api_endpoint: Option<String>,
}

/// Product information
#[derive(Debug, Clone)]
pub struct ProductInfo {
    pub barcode: String,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub ingredients: Option<Vec<String>>,
    pub nutrition: Option<HashMap<String, String>>,
}

impl ProductDatabase {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
            api_endpoint: None,
        }
    }
    
    /// Set API endpoint for online lookups
    pub fn set_api_endpoint(&mut self, endpoint: &str) {
        self.api_endpoint = Some(endpoint.to_string());
    }
    
    /// Add product to local cache
    pub fn add_product(&mut self, info: ProductInfo) {
        self.products.insert(info.barcode.clone(), info);
    }
    
    /// Lookup product by barcode
    pub fn lookup(&self, barcode: &str) -> Option<&ProductInfo> {
        self.products.get(barcode)
    }
    
    /// Check if product exists in cache
    pub fn has_product(&self, barcode: &str) -> bool {
        self.products.contains_key(barcode)
    }
    
    /// Get cached product count
    pub fn cached_count(&self) -> usize {
        self.products.len()
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.products.clear();
    }
}

impl Default for ProductDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// QR code generator (for sharing)
pub struct QRGenerator {
    default_size: u32,
    error_correction: ErrorCorrection,
}

/// Error correction levels for QR codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCorrection {
    /// ~7% recovery
    Low,
    /// ~15% recovery
    Medium,
    /// ~25% recovery
    Quartile,
    /// ~30% recovery
    High,
}

impl QRGenerator {
    pub fn new() -> Self {
        Self {
            default_size: 256,
            error_correction: ErrorCorrection::Medium,
        }
    }
    
    /// Set default output size
    pub fn set_default_size(&mut self, size: u32) {
        self.default_size = size.clamp(64, 2048);
    }
    
    /// Set error correction level
    pub fn set_error_correction(&mut self, level: ErrorCorrection) {
        self.error_correction = level;
    }
    
    /// Generate QR code data (simulated - returns placeholder)
    pub fn generate(&self, content: &str) -> QRCodeData {
        let capacity = self.calculate_capacity();
        
        QRCodeData {
            content: content.to_string(),
            size: self.default_size,
            error_correction: self.error_correction,
            version: self.calculate_version(content.len()),
            data: Vec::new(), // Real impl would generate actual QR data
            fits: content.len() <= capacity,
        }
    }
    
    /// Calculate QR version needed for data length
    fn calculate_version(&self, data_len: usize) -> u8 {
        // Simplified version calculation
        if data_len <= 25 { 1 }
        else if data_len <= 47 { 2 }
        else if data_len <= 77 { 3 }
        else if data_len <= 114 { 4 }
        else if data_len <= 154 { 5 }
        else if data_len <= 500 { 10 }
        else if data_len <= 1000 { 20 }
        else { 40 }
    }
    
    /// Calculate capacity based on error correction
    fn calculate_capacity(&self) -> usize {
        match self.error_correction {
            ErrorCorrection::Low => 2953,
            ErrorCorrection::Medium => 2331,
            ErrorCorrection::Quartile => 1663,
            ErrorCorrection::High => 1273,
        }
    }
    
    /// Generate WiFi QR code
    pub fn generate_wifi(&self, ssid: &str, password: &str, security: &str) -> QRCodeData {
        let content = format!("WIFI:S:{};T:{};P:{};;", ssid, security, password);
        self.generate(&content)
    }
    
    /// Generate contact QR code
    pub fn generate_contact(&self, name: &str, phone: &str, email: &str) -> QRCodeData {
        let content = format!(
            "BEGIN:VCARD\nVERSION:3.0\nN:{}\nTEL:{}\nEMAIL:{}\nEND:VCARD",
            name, phone, email
        );
        self.generate(&content)
    }
}

impl Default for QRGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generated QR code data
#[derive(Debug, Clone)]
pub struct QRCodeData {
    pub content: String,
    pub size: u32,
    pub error_correction: ErrorCorrection,
    pub version: u8,
    pub data: Vec<u8>,
    pub fits: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_barcode_format_types() {
        assert!(BarcodeFormat::QRCode.is_2d());
        assert!(BarcodeFormat::DataMatrix.is_2d());
        assert!(!BarcodeFormat::EAN13.is_2d());
        
        assert!(BarcodeFormat::Code128.is_1d());
        assert!(!BarcodeFormat::QRCode.is_1d());
    }
    
    #[test]
    fn test_scan_result_creation() {
        let bb = BoundingBox::new(100.0, 100.0, 200.0, 200.0);
        let result = ScanResult::new(
            BarcodeFormat::QRCode,
            "https://example.com".to_string(),
            bb,
        );
        
        assert_eq!(result.format, BarcodeFormat::QRCode);
        assert_eq!(result.text, "https://example.com");
        assert!(matches!(result.content_type, ContentType::URL(_)));
    }
    
    #[test]
    fn test_content_type_url() {
        let content = ContentType::detect("https://example.com");
        assert!(matches!(content, ContentType::URL(_)));
        assert!(content.is_actionable());
    }
    
    #[test]
    fn test_content_type_email() {
        let content = ContentType::detect("mailto:test@example.com");
        assert!(matches!(content, ContentType::Email(_)));
    }
    
    #[test]
    fn test_content_type_phone() {
        let content = ContentType::detect("tel:+1234567890");
        assert!(matches!(content, ContentType::Phone(_)));
    }
    
    #[test]
    fn test_content_type_wifi() {
        let content = ContentType::detect("WIFI:S:MyNetwork;T:WPA;P:password123;;");
        
        if let ContentType::WiFi { ssid, password, security } = content {
            assert_eq!(ssid, "MyNetwork");
            assert_eq!(password, Some("password123".to_string()));
            assert_eq!(security, "WPA");
        } else {
            panic!("Expected WiFi content type");
        }
    }
    
    #[test]
    fn test_content_type_geo() {
        let content = ContentType::detect("geo:37.7749,-122.4194");
        
        if let ContentType::GeoLocation { lat, lon } = content {
            assert_eq!(lat, "37.7749");
            assert_eq!(lon, "-122.4194");
        } else {
            panic!("Expected GeoLocation content type");
        }
    }
    
    #[test]
    fn test_content_type_product() {
        let content = ContentType::detect("5901234123457");
        assert!(matches!(content, ContentType::Product(_)));
    }
    
    #[test]
    fn test_content_type_text() {
        let content = ContentType::detect("Just some plain text");
        assert!(matches!(content, ContentType::Text(_)));
        assert!(!content.is_actionable());
    }
    
    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        
        assert!(config.enabled_formats.contains(&BarcodeFormat::QRCode));
        assert!(config.vibrate_on_scan);
        assert_eq!(config.scan_interval_ms, 100);
    }
    
    #[test]
    fn test_barcode_scanner() {
        let config = ScannerConfig::default();
        let mut scanner = BarcodeScanner::new(config);
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let result = ScanResult::new(
            BarcodeFormat::QRCode,
            "test data".to_string(),
            bb,
        );
        
        assert!(scanner.process_scan(result.clone()));
        assert_eq!(scanner.total_scans(), 1);
        
        // Duplicate should be rejected
        assert!(!scanner.process_scan(result));
        assert_eq!(scanner.total_scans(), 1);
    }
    
    #[test]
    fn test_scanner_format_filter() {
        let config = ScannerConfig {
            enabled_formats: vec![BarcodeFormat::QRCode],
            ..Default::default()
        };
        let mut scanner = BarcodeScanner::new(config);
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        
        // QR should be accepted
        let qr = ScanResult::new(BarcodeFormat::QRCode, "qr data".to_string(), bb);
        assert!(scanner.process_scan(qr));
        
        // EAN13 should be rejected (not in enabled formats)
        let ean = ScanResult::new(BarcodeFormat::EAN13, "1234567890123".to_string(), bb);
        assert!(!scanner.process_scan(ean));
    }
    
    #[test]
    fn test_product_database() {
        let mut db = ProductDatabase::new();
        
        let product = ProductInfo {
            barcode: "1234567890123".to_string(),
            name: "Test Product".to_string(),
            brand: Some("Test Brand".to_string()),
            category: None,
            price: Some(9.99),
            currency: Some("USD".to_string()),
            description: None,
            image_url: None,
            ingredients: None,
            nutrition: None,
        };
        
        db.add_product(product);
        
        assert!(db.has_product("1234567890123"));
        assert!(!db.has_product("0000000000000"));
        
        let found = db.lookup("1234567890123").unwrap();
        assert_eq!(found.name, "Test Product");
    }
    
    #[test]
    fn test_qr_generator() {
        let generator = QRGenerator::new();
        
        let qr = generator.generate("Hello World");
        
        assert_eq!(qr.content, "Hello World");
        assert!(qr.fits);
        assert!(qr.version <= 40);
    }
    
    #[test]
    fn test_qr_generator_wifi() {
        let generator = QRGenerator::new();
        
        let qr = generator.generate_wifi("MySSID", "mypassword", "WPA");
        
        assert!(qr.content.contains("WIFI:"));
        assert!(qr.content.contains("S:MySSID"));
        assert!(qr.content.contains("P:mypassword"));
    }
    
    #[test]
    fn test_qr_generator_contact() {
        let generator = QRGenerator::new();
        
        let qr = generator.generate_contact("John Doe", "+1234567890", "john@example.com");
        
        assert!(qr.content.contains("BEGIN:VCARD"));
        assert!(qr.content.contains("John Doe"));
    }
    
    #[test]
    fn test_scan_history() {
        let config = ScannerConfig {
            duplicate_filter_ms: 0, // Disable duplicate filter for test
            ..Default::default()
        };
        let mut scanner = BarcodeScanner::new(config);
        
        let bb = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        
        for i in 0..5 {
            let result = ScanResult::new(
                BarcodeFormat::QRCode,
                format!("scan_{}", i),
                bb,
            );
            scanner.process_scan(result);
        }
        
        assert_eq!(scanner.history().len(), 5);
        assert_eq!(scanner.last_scan().unwrap().text, "scan_4");
    }
}
