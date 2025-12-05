// Image Processing for Kāraṇa OS
// Handles filters, transformations, and image enhancement

use super::*;

/// Image filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// No filter
    None,
    /// Gaussian blur
    GaussianBlur,
    /// Sharpen
    Sharpen,
    /// Edge detection (Sobel)
    EdgeDetect,
    /// Grayscale conversion
    Grayscale,
    /// Sepia tone
    Sepia,
    /// High contrast
    HighContrast,
    /// Night vision (green tint)
    NightVision,
    /// Thermal-like visualization
    ThermalMap,
}

/// Image processor for frame manipulation
pub struct ImageProcessor {
    brightness: f32,
    contrast: f32,
    saturation: f32,
    gamma: f32,
    active_filters: Vec<FilterType>,
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            active_filters: Vec::new(),
        }
    }
    
    /// Set brightness adjustment (-1.0 to 1.0)
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(-1.0, 1.0);
    }
    
    /// Get current brightness
    pub fn brightness(&self) -> f32 {
        self.brightness
    }
    
    /// Set contrast adjustment (0.0 to 3.0)
    pub fn set_contrast(&mut self, contrast: f32) {
        self.contrast = contrast.clamp(0.0, 3.0);
    }
    
    /// Get current contrast
    pub fn contrast(&self) -> f32 {
        self.contrast
    }
    
    /// Set saturation adjustment (0.0 to 2.0)
    pub fn set_saturation(&mut self, saturation: f32) {
        self.saturation = saturation.clamp(0.0, 2.0);
    }
    
    /// Get current saturation
    pub fn saturation(&self) -> f32 {
        self.saturation
    }
    
    /// Set gamma correction (0.1 to 3.0)
    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma.clamp(0.1, 3.0);
    }
    
    /// Get current gamma
    pub fn gamma(&self) -> f32 {
        self.gamma
    }
    
    /// Add a filter to the processing chain
    pub fn add_filter(&mut self, filter: FilterType) {
        if !self.active_filters.contains(&filter) {
            self.active_filters.push(filter);
        }
    }
    
    /// Remove a filter from the processing chain
    pub fn remove_filter(&mut self, filter: FilterType) {
        self.active_filters.retain(|&f| f != filter);
    }
    
    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.active_filters.clear();
    }
    
    /// Get active filters
    pub fn active_filters(&self) -> &[FilterType] {
        &self.active_filters
    }
    
    /// Apply brightness/contrast/saturation to a pixel (RGB)
    pub fn adjust_pixel(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        // Convert to float
        let mut rf = r as f32 / 255.0;
        let mut gf = g as f32 / 255.0;
        let mut bf = b as f32 / 255.0;
        
        // Apply brightness
        rf += self.brightness;
        gf += self.brightness;
        bf += self.brightness;
        
        // Apply contrast
        rf = (rf - 0.5) * self.contrast + 0.5;
        gf = (gf - 0.5) * self.contrast + 0.5;
        bf = (bf - 0.5) * self.contrast + 0.5;
        
        // Apply saturation
        let gray = 0.299 * rf + 0.587 * gf + 0.114 * bf;
        rf = gray + self.saturation * (rf - gray);
        gf = gray + self.saturation * (gf - gray);
        bf = gray + self.saturation * (bf - gray);
        
        // Apply gamma
        rf = rf.max(0.0).powf(1.0 / self.gamma);
        gf = gf.max(0.0).powf(1.0 / self.gamma);
        bf = bf.max(0.0).powf(1.0 / self.gamma);
        
        // Clamp and convert back
        (
            (rf.clamp(0.0, 1.0) * 255.0) as u8,
            (gf.clamp(0.0, 1.0) * 255.0) as u8,
            (bf.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
    
    /// Apply sepia tone to a pixel
    pub fn apply_sepia(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let rf = r as f32;
        let gf = g as f32;
        let bf = b as f32;
        
        let nr = (0.393 * rf + 0.769 * gf + 0.189 * bf).min(255.0) as u8;
        let ng = (0.349 * rf + 0.686 * gf + 0.168 * bf).min(255.0) as u8;
        let nb = (0.272 * rf + 0.534 * gf + 0.131 * bf).min(255.0) as u8;
        
        (nr, ng, nb)
    }
    
    /// Apply night vision effect
    pub fn apply_night_vision(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        // Green tint
        (gray / 4, gray, gray / 4)
    }
    
    /// Apply thermal map effect
    pub fn apply_thermal(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let intensity = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
        
        // Map intensity to thermal colors (cold blue -> hot red)
        if intensity < 0.25 {
            let t = intensity * 4.0;
            ((t * 255.0) as u8, 0, 255)
        } else if intensity < 0.5 {
            let t = (intensity - 0.25) * 4.0;
            (255, (t * 255.0) as u8, ((1.0 - t) * 255.0) as u8)
        } else if intensity < 0.75 {
            let t = (intensity - 0.5) * 4.0;
            (255, 255, 0)
        } else {
            (255, ((1.0 - (intensity - 0.75) * 4.0) * 255.0) as u8, 0)
        }
    }
    
    /// Reset all adjustments to defaults
    pub fn reset(&mut self) {
        self.brightness = 0.0;
        self.contrast = 1.0;
        self.saturation = 1.0;
        self.gamma = 1.0;
        self.active_filters.clear();
    }
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram for image analysis
#[derive(Debug, Clone)]
pub struct Histogram {
    pub red: [u32; 256],
    pub green: [u32; 256],
    pub blue: [u32; 256],
    pub luminance: [u32; 256],
    pub total_pixels: u32,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            luminance: [0; 256],
            total_pixels: 0,
        }
    }
    
    /// Calculate histogram from RGB frame data
    pub fn from_rgb_data(data: &[u8]) -> Self {
        let mut hist = Self::new();
        
        for chunk in data.chunks(3) {
            if chunk.len() == 3 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                
                hist.red[r as usize] += 1;
                hist.green[g as usize] += 1;
                hist.blue[b as usize] += 1;
                
                let lum = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
                hist.luminance[lum as usize] += 1;
                
                hist.total_pixels += 1;
            }
        }
        
        hist
    }
    
    /// Get average luminance
    pub fn average_luminance(&self) -> f32 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        
        let sum: u64 = self.luminance.iter()
            .enumerate()
            .map(|(i, &count)| i as u64 * count as u64)
            .sum();
        
        sum as f32 / self.total_pixels as f32
    }
    
    /// Check if image is underexposed
    pub fn is_underexposed(&self, threshold: f32) -> bool {
        self.average_luminance() < threshold
    }
    
    /// Check if image is overexposed
    pub fn is_overexposed(&self, threshold: f32) -> bool {
        self.average_luminance() > threshold
    }
    
    /// Get dynamic range (difference between darkest and brightest)
    pub fn dynamic_range(&self) -> u8 {
        let min = self.luminance.iter()
            .position(|&x| x > 0)
            .unwrap_or(0);
        let max = self.luminance.iter()
            .rposition(|&x| x > 0)
            .unwrap_or(255);
        
        (max - min) as u8
    }
    
    /// Get contrast estimate (standard deviation of luminance)
    pub fn contrast_estimate(&self) -> f32 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        
        let mean = self.average_luminance();
        let variance: f32 = self.luminance.iter()
            .enumerate()
            .map(|(i, &count)| {
                let diff = i as f32 - mean;
                diff * diff * count as f32
            })
            .sum::<f32>() / self.total_pixels as f32;
        
        variance.sqrt()
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Region of interest for processing
#[derive(Debug, Clone, Copy)]
pub struct ROI {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ROI {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    /// Create ROI covering entire frame
    pub fn full_frame(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, width, height }
    }
    
    /// Center ROI of given size
    pub fn center(frame_width: u32, frame_height: u32, roi_width: u32, roi_height: u32) -> Self {
        Self {
            x: (frame_width.saturating_sub(roi_width)) / 2,
            y: (frame_height.saturating_sub(roi_height)) / 2,
            width: roi_width.min(frame_width),
            height: roi_height.min(frame_height),
        }
    }
    
    /// Check if point is within ROI
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
    
    /// Get area in pixels
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
    
    /// Get center point
    pub fn center_point(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
    
    /// Clip ROI to frame bounds
    pub fn clip_to_frame(&self, frame_width: u32, frame_height: u32) -> ROI {
        let x = self.x.min(frame_width);
        let y = self.y.min(frame_height);
        let width = self.width.min(frame_width.saturating_sub(x));
        let height = self.height.min(frame_height.saturating_sub(y));
        
        ROI { x, y, width, height }
    }
}

/// Auto-exposure calculator
pub struct AutoExposure {
    target_luminance: f32,
    current_exposure_ms: f32,
    min_exposure_ms: f32,
    max_exposure_ms: f32,
    adjustment_speed: f32,
}

impl AutoExposure {
    pub fn new() -> Self {
        Self {
            target_luminance: 128.0,
            current_exposure_ms: 16.0,
            min_exposure_ms: 0.1,
            max_exposure_ms: 100.0,
            adjustment_speed: 0.1,
        }
    }
    
    /// Set target luminance (0-255)
    pub fn set_target(&mut self, target: f32) {
        self.target_luminance = target.clamp(0.0, 255.0);
    }
    
    /// Set exposure limits
    pub fn set_limits(&mut self, min_ms: f32, max_ms: f32) {
        self.min_exposure_ms = min_ms.max(0.01);
        self.max_exposure_ms = max_ms.max(self.min_exposure_ms);
    }
    
    /// Calculate new exposure based on current histogram
    pub fn calculate(&mut self, histogram: &Histogram) -> f32 {
        let current_lum = histogram.average_luminance();
        
        if current_lum < 1.0 {
            // Very dark, increase exposure significantly
            self.current_exposure_ms = (self.current_exposure_ms * 1.5)
                .min(self.max_exposure_ms);
        } else {
            // Calculate ratio
            let ratio = self.target_luminance / current_lum;
            
            // Apply adjustment with smoothing
            let new_exposure = self.current_exposure_ms * ratio;
            self.current_exposure_ms += (new_exposure - self.current_exposure_ms) 
                * self.adjustment_speed;
        }
        
        self.current_exposure_ms = self.current_exposure_ms
            .clamp(self.min_exposure_ms, self.max_exposure_ms);
        
        self.current_exposure_ms
    }
    
    /// Get current exposure
    pub fn current_exposure(&self) -> f32 {
        self.current_exposure_ms
    }
}

impl Default for AutoExposure {
    fn default() -> Self {
        Self::new()
    }
}

/// White balance adjustment
pub struct WhiteBalance {
    pub red_gain: f32,
    pub green_gain: f32,
    pub blue_gain: f32,
    auto_mode: bool,
}

impl WhiteBalance {
    pub fn new() -> Self {
        Self {
            red_gain: 1.0,
            green_gain: 1.0,
            blue_gain: 1.0,
            auto_mode: true,
        }
    }
    
    /// Set manual white balance gains
    pub fn set_gains(&mut self, red: f32, green: f32, blue: f32) {
        self.red_gain = red.clamp(0.5, 2.0);
        self.green_gain = green.clamp(0.5, 2.0);
        self.blue_gain = blue.clamp(0.5, 2.0);
        self.auto_mode = false;
    }
    
    /// Enable auto white balance
    pub fn set_auto(&mut self, auto: bool) {
        self.auto_mode = auto;
    }
    
    /// Check if auto mode is enabled
    pub fn is_auto(&self) -> bool {
        self.auto_mode
    }
    
    /// Calculate white balance from gray world assumption
    pub fn calculate_from_histogram(&mut self, histogram: &Histogram) {
        if !self.auto_mode || histogram.total_pixels == 0 {
            return;
        }
        
        // Calculate average for each channel
        let avg_r: f32 = histogram.red.iter()
            .enumerate()
            .map(|(i, &c)| i as f32 * c as f32)
            .sum::<f32>() / histogram.total_pixels as f32;
        
        let avg_g: f32 = histogram.green.iter()
            .enumerate()
            .map(|(i, &c)| i as f32 * c as f32)
            .sum::<f32>() / histogram.total_pixels as f32;
        
        let avg_b: f32 = histogram.blue.iter()
            .enumerate()
            .map(|(i, &c)| i as f32 * c as f32)
            .sum::<f32>() / histogram.total_pixels as f32;
        
        // Gray world: assume average should be gray
        let avg = (avg_r + avg_g + avg_b) / 3.0;
        
        if avg_r > 0.0 { self.red_gain = (avg / avg_r).clamp(0.5, 2.0); }
        if avg_g > 0.0 { self.green_gain = (avg / avg_g).clamp(0.5, 2.0); }
        if avg_b > 0.0 { self.blue_gain = (avg / avg_b).clamp(0.5, 2.0); }
    }
    
    /// Apply white balance to a pixel
    pub fn apply(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        (
            ((r as f32 * self.red_gain).min(255.0)) as u8,
            ((g as f32 * self.green_gain).min(255.0)) as u8,
            ((b as f32 * self.blue_gain).min(255.0)) as u8,
        )
    }
    
    /// Preset: Daylight
    pub fn preset_daylight(&mut self) {
        self.set_gains(1.0, 1.0, 1.1);
    }
    
    /// Preset: Cloudy
    pub fn preset_cloudy(&mut self) {
        self.set_gains(1.1, 1.0, 0.95);
    }
    
    /// Preset: Incandescent
    pub fn preset_incandescent(&mut self) {
        self.set_gains(0.9, 1.0, 1.3);
    }
    
    /// Preset: Fluorescent
    pub fn preset_fluorescent(&mut self) {
        self.set_gains(0.95, 1.0, 1.15);
    }
}

impl Default for WhiteBalance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_processor_defaults() {
        let proc = ImageProcessor::new();
        
        assert_eq!(proc.brightness(), 0.0);
        assert_eq!(proc.contrast(), 1.0);
        assert_eq!(proc.saturation(), 1.0);
        assert_eq!(proc.gamma(), 1.0);
    }
    
    #[test]
    fn test_brightness_adjustment() {
        let mut proc = ImageProcessor::new();
        
        // No change
        let (r, g, b) = proc.adjust_pixel(128, 128, 128);
        assert!((r as i32 - 128).abs() <= 1);
        
        // Increase brightness
        proc.set_brightness(0.5);
        let (r, g, b) = proc.adjust_pixel(100, 100, 100);
        assert!(r > 100);
        assert!(g > 100);
        assert!(b > 100);
    }
    
    #[test]
    fn test_contrast_adjustment() {
        let mut proc = ImageProcessor::new();
        
        proc.set_contrast(2.0);
        
        // Low values should get lower
        let (r, _, _) = proc.adjust_pixel(64, 64, 64);
        assert!(r < 64);
        
        // High values should get higher
        let (r, _, _) = proc.adjust_pixel(192, 192, 192);
        assert!(r > 192);
    }
    
    #[test]
    fn test_saturation_adjustment() {
        let mut proc = ImageProcessor::new();
        
        // Desaturate
        proc.set_saturation(0.0);
        let (r, g, b) = proc.adjust_pixel(255, 0, 0);
        
        // Should be gray
        assert_eq!(r, g);
        assert_eq!(g, b);
    }
    
    #[test]
    fn test_sepia_filter() {
        let proc = ImageProcessor::new();
        
        let (r, g, b) = proc.apply_sepia(100, 100, 100);
        
        // Sepia should have warm tones (r > g > b)
        assert!(r >= g);
        assert!(g >= b);
    }
    
    #[test]
    fn test_night_vision() {
        let proc = ImageProcessor::new();
        
        let (r, g, b) = proc.apply_night_vision(100, 100, 100);
        
        // Green should be dominant
        assert!(g > r);
        assert!(g > b);
    }
    
    #[test]
    fn test_filter_management() {
        let mut proc = ImageProcessor::new();
        
        proc.add_filter(FilterType::Grayscale);
        proc.add_filter(FilterType::Sharpen);
        
        assert_eq!(proc.active_filters().len(), 2);
        
        proc.remove_filter(FilterType::Grayscale);
        assert_eq!(proc.active_filters().len(), 1);
        
        proc.clear_filters();
        assert!(proc.active_filters().is_empty());
    }
    
    #[test]
    fn test_histogram_from_rgb() {
        // Create simple test data: 4 pixels of varying brightness
        let data = vec![
            0, 0, 0,       // Black
            255, 255, 255, // White
            128, 128, 128, // Gray
            128, 128, 128, // Gray
        ];
        
        let hist = Histogram::from_rgb_data(&data);
        
        assert_eq!(hist.total_pixels, 4);
        assert_eq!(hist.red[0], 1);
        assert_eq!(hist.red[255], 1);
        assert_eq!(hist.red[128], 2);
    }
    
    #[test]
    fn test_histogram_average_luminance() {
        // All gray pixels at 100
        let data = vec![100, 100, 100, 100, 100, 100];
        let hist = Histogram::from_rgb_data(&data);
        
        let avg = hist.average_luminance();
        assert!((avg - 100.0).abs() < 1.0);
    }
    
    #[test]
    fn test_histogram_exposure_detection() {
        // Dark image
        let dark_data = vec![20, 20, 20, 30, 30, 30];
        let dark_hist = Histogram::from_rgb_data(&dark_data);
        assert!(dark_hist.is_underexposed(100.0));
        
        // Bright image
        let bright_data = vec![230, 230, 230, 240, 240, 240];
        let bright_hist = Histogram::from_rgb_data(&bright_data);
        assert!(bright_hist.is_overexposed(200.0));
    }
    
    #[test]
    fn test_roi_creation() {
        let roi = ROI::new(10, 20, 100, 50);
        
        assert_eq!(roi.area(), 5000);
        assert_eq!(roi.center_point(), (60, 45));
    }
    
    #[test]
    fn test_roi_contains() {
        let roi = ROI::new(10, 10, 100, 100);
        
        assert!(roi.contains(50, 50));
        assert!(roi.contains(10, 10));
        assert!(!roi.contains(5, 5));
        assert!(!roi.contains(110, 50));
    }
    
    #[test]
    fn test_roi_center() {
        let roi = ROI::center(640, 480, 100, 100);
        
        assert_eq!(roi.x, 270);
        assert_eq!(roi.y, 190);
    }
    
    #[test]
    fn test_roi_clip() {
        let roi = ROI::new(600, 400, 100, 100);
        let clipped = roi.clip_to_frame(640, 480);
        
        assert_eq!(clipped.x, 600);
        assert_eq!(clipped.y, 400);
        assert_eq!(clipped.width, 40);
        assert_eq!(clipped.height, 80);
    }
    
    #[test]
    fn test_auto_exposure() {
        let mut ae = AutoExposure::new();
        ae.set_target(128.0);
        
        // Underexposed image - 100 pixels of [50, 50, 50]
        let dark_data: Vec<u8> = (0..100).flat_map(|_| vec![50u8, 50, 50]).collect();
        let dark_hist = Histogram::from_rgb_data(&dark_data);
        
        let initial = ae.current_exposure();
        ae.calculate(&dark_hist);
        
        // Should increase exposure for dark image
        assert!(ae.current_exposure() > initial);
    }
    
    #[test]
    fn test_white_balance_presets() {
        let mut wb = WhiteBalance::new();
        
        wb.preset_incandescent();
        assert!(!wb.is_auto());
        assert!(wb.blue_gain > wb.red_gain);
        
        wb.set_auto(true);
        assert!(wb.is_auto());
    }
    
    #[test]
    fn test_white_balance_apply() {
        let mut wb = WhiteBalance::new();
        wb.set_gains(1.5, 1.0, 0.8);
        
        let (r, g, b) = wb.apply(100, 100, 100);
        
        assert_eq!(r, 150);
        assert_eq!(g, 100);
        assert_eq!(b, 80);
    }
    
    #[test]
    fn test_processor_reset() {
        let mut proc = ImageProcessor::new();
        
        proc.set_brightness(0.5);
        proc.set_contrast(2.0);
        proc.add_filter(FilterType::Sepia);
        
        proc.reset();
        
        assert_eq!(proc.brightness(), 0.0);
        assert_eq!(proc.contrast(), 1.0);
        assert!(proc.active_filters().is_empty());
    }
}
