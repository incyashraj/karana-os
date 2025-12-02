//! # Kāraṇa Camera Module
//!
//! Real camera capture and image processing for smart glasses.
//!
//! ## Features
//! - Photo capture from camera or simulated
//! - Basic image processing (resize, save)
//! - Object detection integration
//! - QR code scanning

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// Device path (e.g., /dev/video0)
    pub device: String,
    /// Capture width
    pub width: u32,
    /// Capture height
    pub height: u32,
    /// Output directory for photos
    pub output_dir: PathBuf,
    /// Enable simulated mode (for testing without camera)
    pub simulated: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            device: "/dev/video0".to_string(),
            width: 1280,
            height: 720,
            output_dir: PathBuf::from("/tmp/karana_photos"),
            simulated: true, // Default to simulated since not all systems have cameras
        }
    }
}

/// Camera capture result
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// Path to saved image
    pub path: PathBuf,
    /// Timestamp
    pub timestamp: u64,
    /// Image dimensions
    pub width: u32,
    pub height: u32,
    /// Was this a simulated capture
    pub simulated: bool,
    /// Optional detected objects
    pub detected_objects: Vec<String>,
}

/// Camera capture and processing
pub struct Camera {
    config: CameraConfig,
    capture_count: u32,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Result<Self> {
        // Create output directory
        fs::create_dir_all(&config.output_dir)?;
        
        log::info!("[CAMERA] Initialized: device={}, {}x{}, output={}",
            config.device, config.width, config.height, config.output_dir.display());
        
        Ok(Self {
            config,
            capture_count: 0,
        })
    }

    /// Check if real camera is available
    pub fn is_camera_available(&self) -> bool {
        Path::new(&self.config.device).exists()
    }

    /// Capture a photo
    pub fn capture(&mut self) -> Result<CaptureResult> {
        self.capture_count += 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let filename = format!("photo_{:05}_{}.png", self.capture_count, timestamp);
        let path = self.config.output_dir.join(&filename);

        let (width, height, simulated) = if !self.config.simulated && self.is_camera_available() {
            // Try real camera capture (requires v4l2 support)
            match self.capture_real() {
                Ok(img) => {
                    img.save(&path)?;
                    (img.width(), img.height(), false)
                }
                Err(e) => {
                    log::warn!("[CAMERA] Real capture failed ({}), using simulated", e);
                    let img = self.capture_simulated();
                    img.save(&path)?;
                    (img.width(), img.height(), true)
                }
            }
        } else {
            // Simulated capture
            let img = self.capture_simulated();
            img.save(&path)?;
            (img.width(), img.height(), true)
        };

        log::info!("[CAMERA] 📸 Captured: {} ({}x{}, simulated={})", 
            path.display(), width, height, simulated);

        Ok(CaptureResult {
            path,
            timestamp,
            width,
            height,
            simulated,
            detected_objects: Vec::new(),
        })
    }

    /// Generate a simulated test image
    fn capture_simulated(&self) -> RgbImage {
        let width = self.config.width;
        let height = self.config.height;
        
        let mut img: RgbImage = ImageBuffer::new(width, height);
        
        // Create a gradient background
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x as f32 / width as f32) * 100.0 + 50.0) as u8;
            let g = ((y as f32 / height as f32) * 100.0 + 100.0) as u8;
            let b = 180u8;
            *pixel = Rgb([r, g, b]);
        }

        // Add a simple crosshair in the center
        let cx = width / 2;
        let cy = height / 2;
        let crosshair_size = 50;
        
        for x in (cx - crosshair_size)..(cx + crosshair_size) {
            if x < width {
                img.put_pixel(x, cy, Rgb([255, 255, 255]));
            }
        }
        for y in (cy - crosshair_size)..(cy + crosshair_size) {
            if y < height {
                img.put_pixel(cx, y, Rgb([255, 255, 255]));
            }
        }

        // Add timestamp overlay (as a simple block)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        // Draw timestamp indicator (colored block based on time)
        let block_x = 20;
        let block_y = height - 40;
        let block_color = Rgb([
            ((timestamp % 256) as u8),
            ((timestamp / 2 % 256) as u8),
            200u8
        ]);
        
        for dx in 0..80 {
            for dy in 0..20 {
                if block_x + dx < width && block_y + dy < height {
                    img.put_pixel(block_x + dx, block_y + dy, block_color);
                }
            }
        }

        img
    }

    /// Try to capture from real camera (Linux v4l2)
    #[cfg(feature = "v4l2")]
    fn capture_real(&self) -> Result<DynamicImage> {
        use rscam::{Camera as V4l2Camera, Config};
        
        log::info!("[CAMERA] Attempting v4l2 capture from {}", self.config.device);
        
        // Open camera device
        let mut camera = V4l2Camera::new(&self.config.device)
            .map_err(|e| anyhow!("Failed to open camera: {}", e))?;
        
        // Configure capture (try common formats)
        // MJPEG is widely supported and efficient
        let config = Config {
            interval: (1, 30), // 30 fps
            resolution: (self.config.width, self.config.height),
            format: b"MJPG",
            ..Default::default()
        };
        
        camera.start(&config).map_err(|e| {
            log::warn!("[CAMERA] MJPG not supported, trying YUYV");
            e
        }).or_else(|_| {
            // Fall back to YUYV if MJPG fails
            let config_yuyv = Config {
                interval: (1, 30),
                resolution: (self.config.width, self.config.height),
                format: b"YUYV",
                ..Default::default()
            };
            camera.start(&config_yuyv)
        }).map_err(|e| anyhow!("Failed to start camera: {}", e))?;
        
        // Capture a frame
        let frame = camera.capture()
            .map_err(|e| anyhow!("Failed to capture frame: {}", e))?;
        
        // Decode frame based on format
        // If MJPEG, decode directly
        let img = if frame.len() > 2 && frame[0] == 0xFF && frame[1] == 0xD8 {
            // JPEG magic bytes - decode as JPEG
            image::load_from_memory(&frame[..])
                .map_err(|e| anyhow!("Failed to decode JPEG: {}", e))?
        } else {
            // Assume YUYV - convert to RGB
            let width = self.config.width as usize;
            let height = self.config.height as usize;
            let mut rgb_data = vec![0u8; width * height * 3];
            
            // YUYV to RGB conversion
            for i in 0..(width * height / 2) {
                let y0 = frame[i * 4] as f32;
                let u = frame[i * 4 + 1] as f32 - 128.0;
                let y1 = frame[i * 4 + 2] as f32;
                let v = frame[i * 4 + 3] as f32 - 128.0;
                
                // Convert YUV to RGB
                let r0 = (y0 + 1.402 * v).clamp(0.0, 255.0) as u8;
                let g0 = (y0 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                let b0 = (y0 + 1.772 * u).clamp(0.0, 255.0) as u8;
                
                let r1 = (y1 + 1.402 * v).clamp(0.0, 255.0) as u8;
                let g1 = (y1 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                let b1 = (y1 + 1.772 * u).clamp(0.0, 255.0) as u8;
                
                rgb_data[i * 6] = r0;
                rgb_data[i * 6 + 1] = g0;
                rgb_data[i * 6 + 2] = b0;
                rgb_data[i * 6 + 3] = r1;
                rgb_data[i * 6 + 4] = g1;
                rgb_data[i * 6 + 5] = b1;
            }
            
            image::DynamicImage::ImageRgb8(
                image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
                    .ok_or_else(|| anyhow!("Failed to create RGB image"))?
            )
        };
        
        log::info!("[CAMERA] ✓ Real capture: {}x{}", img.width(), img.height());
        Ok(img)
    }
    
    /// Fallback when v4l2 feature is not enabled
    #[cfg(not(feature = "v4l2"))]
    fn capture_real(&self) -> Result<DynamicImage> {
        // Check if camera device exists
        if !self.is_camera_available() {
            return Err(anyhow!("Camera device {} not found", self.config.device));
        }

        // v4l2 feature not enabled
        Err(anyhow!("Real camera capture requires 'v4l2' feature. Build with: cargo build --features v4l2"))
    }

    /// Take a photo and analyze it for objects (simple version)
    pub fn capture_and_analyze(&mut self) -> Result<CaptureResult> {
        let mut result = self.capture()?;
        
        // Simple "analysis" - in reality this would use an ML model
        // For now, return simulated objects
        result.detected_objects = vec![
            "person".to_string(),
            "table".to_string(),
            "laptop".to_string(),
        ];

        log::info!("[CAMERA] Objects detected: {:?}", result.detected_objects);
        
        Ok(result)
    }
    
    /// Take a photo and analyze it using the AI BLIP model
    /// This provides real image captioning and object detection
    pub fn capture_and_analyze_with_ai(
        &mut self, 
        ai: &mut crate::ai::KaranaAI
    ) -> Result<CaptureResult> {
        let mut result = self.capture()?;
        
        // Use BLIP model for image captioning
        match ai.describe_image(&result.path.to_string_lossy()) {
            Ok(caption) => {
                log::info!("[CAMERA] AI Caption: {}", caption);
                
                // Extract objects from caption (simple parsing)
                // BLIP typically generates captions like "a person sitting at a table with a laptop"
                let objects = Self::extract_objects_from_caption(&caption);
                result.detected_objects = objects;
                
                log::info!("[CAMERA] ✓ AI Analysis complete: {} objects detected", 
                    result.detected_objects.len());
            },
            Err(e) => {
                log::warn!("[CAMERA] AI analysis failed ({}), using fallback", e);
                // Fallback to simulated objects
                result.detected_objects = vec![
                    "unknown".to_string(),
                ];
            }
        }
        
        Ok(result)
    }
    
    /// Extract object nouns from an image caption
    fn extract_objects_from_caption(caption: &str) -> Vec<String> {
        // Common objects that BLIP often mentions
        const OBJECT_WORDS: &[&str] = &[
            "person", "people", "man", "woman", "child", "dog", "cat", "bird",
            "table", "chair", "desk", "laptop", "computer", "phone", "screen",
            "car", "bus", "truck", "bike", "bicycle", "motorcycle",
            "tree", "plant", "flower", "grass", "road", "building",
            "book", "bottle", "cup", "glass", "plate", "food",
            "bed", "couch", "sofa", "window", "door", "wall",
            "bag", "backpack", "suitcase", "umbrella", "hat", "shirt",
            "clock", "tv", "television", "monitor", "keyboard", "mouse"
        ];
        
        let caption_lower = caption.to_lowercase();
        let mut found = Vec::new();
        
        for &obj in OBJECT_WORDS {
            if caption_lower.contains(obj) {
                found.push(obj.to_string());
            }
        }
        
        if found.is_empty() {
            // If no specific objects found, extract nouns heuristically
            // (simple approach: words after "a/an/the")
            let words: Vec<&str> = caption_lower.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if (*word == "a" || *word == "an" || *word == "the") && i + 1 < words.len() {
                    let next = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric());
                    if !next.is_empty() && next.len() > 2 {
                        found.push(next.to_string());
                    }
                }
            }
        }
        
        found
    }

    /// Get the path to the most recent photo
    pub fn latest_photo(&self) -> Option<PathBuf> {
        let mut entries: Vec<_> = fs::read_dir(&self.config.output_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "png").unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.path());
        entries.last().map(|e| e.path())
    }

    /// Clear all captured photos
    pub fn clear_photos(&self) -> Result<u32> {
        let mut count = 0;
        for entry in fs::read_dir(&self.config.output_dir)? {
            let entry = entry?;
            if entry.path().extension().map(|ext| ext == "png").unwrap_or(false) {
                fs::remove_file(entry.path())?;
                count += 1;
            }
        }
        log::info!("[CAMERA] Cleared {} photos", count);
        Ok(count)
    }

    /// Get capture statistics
    pub fn stats(&self) -> CameraStats {
        let photo_count = fs::read_dir(&self.config.output_dir)
            .map(|entries| entries.count())
            .unwrap_or(0);

        CameraStats {
            capture_count: self.capture_count,
            photos_stored: photo_count,
            output_dir: self.config.output_dir.clone(),
            camera_available: self.is_camera_available(),
            simulated_mode: self.config.simulated,
        }
    }
}

/// Camera statistics
#[derive(Debug)]
pub struct CameraStats {
    pub capture_count: u32,
    pub photos_stored: usize,
    pub output_dir: PathBuf,
    pub camera_available: bool,
    pub simulated_mode: bool,
}

/// QR Code scanner (simple implementation)
pub struct QRScanner {
    camera: Camera,
}

impl QRScanner {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }

    /// Scan for QR code
    pub fn scan(&mut self) -> Result<Option<String>> {
        let capture = self.camera.capture()?;
        
        // Load the image
        let img = image::open(&capture.path)?;
        
        // For now, return a simulated QR code result
        // Real implementation would use the bardecoder or rqrr crate
        if capture.simulated {
            log::info!("[QR] Simulated scan - no QR code found");
            return Ok(None);
        }

        // Placeholder for real QR detection
        log::info!("[QR] Scanning image at {}...", capture.path.display());
        
        // TODO: Integrate actual QR code scanning library
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_config_default() {
        let config = CameraConfig::default();
        assert!(config.simulated);
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    #[test]
    fn test_simulated_capture() {
        let config = CameraConfig {
            output_dir: PathBuf::from("/tmp/karana_test_photos"),
            simulated: true,
            ..Default::default()
        };
        
        let mut camera = Camera::new(config).unwrap();
        let result = camera.capture().unwrap();
        
        assert!(result.simulated);
        assert!(result.path.exists());
        assert_eq!(result.width, 1280);
        assert_eq!(result.height, 720);
        
        // Cleanup
        fs::remove_file(&result.path).ok();
    }

    #[test]
    fn test_capture_and_analyze() {
        let config = CameraConfig {
            output_dir: PathBuf::from("/tmp/karana_test_photos2"),
            simulated: true,
            ..Default::default()
        };
        
        let mut camera = Camera::new(config).unwrap();
        let result = camera.capture_and_analyze().unwrap();
        
        assert!(!result.detected_objects.is_empty());
        assert!(result.detected_objects.contains(&"person".to_string()));
        
        // Cleanup
        fs::remove_file(&result.path).ok();
    }
}
