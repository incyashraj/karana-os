// Kāraṇa OS - Video Processing Module
// Real-time video filters and effects

use std::collections::VecDeque;

use super::frame::{VideoFrame, PixelFormat};
use super::VideoError;

/// Video filter types
#[derive(Debug, Clone)]
pub enum VideoFilter {
    /// Image stabilization
    Stabilization {
        strength: f32,
    },
    /// Auto exposure adjustment
    AutoExposure {
        target_brightness: f32,
    },
    /// HDR tone mapping
    ToneMapping {
        exposure: f32,
    },
    /// Brightness/Contrast adjustment
    BrightnessContrast {
        brightness: f32,
        contrast: f32,
    },
    /// Color saturation
    Saturation {
        value: f32,
    },
    /// Gamma correction
    Gamma {
        value: f32,
    },
    /// Sharpening
    Sharpen {
        strength: f32,
    },
    /// Gaussian blur
    Blur {
        radius: u32,
    },
    /// Grayscale conversion
    Grayscale,
    /// Color inversion
    Invert,
    /// Sepia tone
    Sepia {
        strength: f32,
    },
    /// Vignette effect
    Vignette {
        strength: f32,
        radius: f32,
    },
    /// Edge detection
    EdgeDetect,
    /// Noise reduction
    Denoise {
        strength: f32,
    },
    /// White balance correction
    WhiteBalance {
        temperature: f32,
        tint: f32,
    },
    /// Lens distortion correction
    LensCorrection {
        k1: f32,
        k2: f32,
    },
    /// Crop and scale
    CropScale {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Rotation
    Rotate {
        angle: f32,
    },
    /// Flip
    Flip {
        horizontal: bool,
        vertical: bool,
    },
    /// Custom LUT (Look-Up Table)
    Lut {
        table: Vec<u8>,
    },
    /// Chroma key (green screen)
    ChromaKey {
        hue: f32,
        tolerance: f32,
    },
    /// Face detection overlay (for AR)
    FaceDetect {
        draw_boxes: bool,
    },
}

/// Video processor with filter chain
#[derive(Debug)]
pub struct VideoProcessor {
    /// Frame width
    width: u32,
    /// Frame height
    height: u32,
    /// Filter chain
    filters: Vec<VideoFilter>,
    /// Processing metrics
    metrics: ProcessingMetrics,
    /// Previous frames for temporal processing
    frame_history: VecDeque<FrameStats>,
    /// Stabilization state
    stabilizer: Option<Stabilizer>,
}

/// Processing metrics
#[derive(Debug, Default)]
pub struct ProcessingMetrics {
    /// Frames processed
    pub frames_processed: u64,
    /// Total processing time (us)
    pub total_time_us: u64,
    /// Average processing time (us)
    pub avg_time_us: f64,
}

/// Frame statistics for temporal processing
#[derive(Debug, Clone, Default)]
struct FrameStats {
    /// Average brightness
    brightness: f32,
    /// Motion vectors
    motion: (f32, f32),
}

/// Image stabilization state
#[derive(Debug)]
struct Stabilizer {
    /// Smoothing factor
    smoothing: f32,
    /// Current offset
    offset_x: f32,
    offset_y: f32,
    /// Motion history
    motion_history: VecDeque<(f32, f32)>,
}

impl Stabilizer {
    fn new(smoothing: f32) -> Self {
        Self {
            smoothing,
            offset_x: 0.0,
            offset_y: 0.0,
            motion_history: VecDeque::with_capacity(30),
        }
    }

    fn update(&mut self, motion: (f32, f32)) -> (f32, f32) {
        self.motion_history.push_back(motion);
        if self.motion_history.len() > 30 {
            self.motion_history.pop_front();
        }

        // Calculate average motion
        let (sum_x, sum_y): (f32, f32) = self.motion_history.iter()
            .fold((0.0, 0.0), |(ax, ay), (mx, my)| (ax + mx, ay + my));
        let count = self.motion_history.len() as f32;
        let avg_motion = (sum_x / count, sum_y / count);

        // Smooth offset
        self.offset_x = self.offset_x * self.smoothing + avg_motion.0 * (1.0 - self.smoothing);
        self.offset_y = self.offset_y * self.smoothing + avg_motion.1 * (1.0 - self.smoothing);

        (-self.offset_x, -self.offset_y)
    }
}

impl VideoProcessor {
    /// Create new video processor
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            filters: Vec::new(),
            metrics: ProcessingMetrics::default(),
            frame_history: VecDeque::with_capacity(30),
            stabilizer: None,
        }
    }

    /// Add filter to chain
    pub fn add_filter(&mut self, filter: VideoFilter) {
        // Initialize stabilizer if needed
        if matches!(filter, VideoFilter::Stabilization { .. }) {
            if let VideoFilter::Stabilization { strength } = filter {
                self.stabilizer = Some(Stabilizer::new(strength));
            }
        }
        self.filters.push(filter);
    }

    /// Remove filter at index
    pub fn remove_filter(&mut self, index: usize) -> Option<VideoFilter> {
        if index < self.filters.len() {
            Some(self.filters.remove(index))
        } else {
            None
        }
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.filters.clear();
        self.stabilizer = None;
    }

    /// Get filter count
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Process frame through filter chain
    pub fn process(&mut self, input: &VideoFrame) -> Result<VideoFrame, VideoError> {
        let start = std::time::Instant::now();

        // Convert to RGB24 for processing
        let mut frame = input.to_rgb24();

        // Apply each filter
        for filter in &self.filters.clone() {
            frame = self.apply_filter(&frame, filter)?;
        }

        // Update metrics
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.frames_processed += 1;
        self.metrics.total_time_us += elapsed;
        self.metrics.avg_time_us = self.metrics.total_time_us as f64 / self.metrics.frames_processed as f64;

        // Update frame stats
        let stats = FrameStats {
            brightness: self.calculate_brightness(&frame),
            motion: (0.0, 0.0), // Would calculate motion vectors in real impl
        };
        self.frame_history.push_back(stats);
        if self.frame_history.len() > 30 {
            self.frame_history.pop_front();
        }

        // Preserve metadata
        frame.timestamp = input.timestamp;
        frame.sequence = input.sequence;
        frame.pts = input.pts;

        Ok(frame)
    }

    /// Apply single filter
    fn apply_filter(&mut self, input: &VideoFrame, filter: &VideoFilter) -> Result<VideoFrame, VideoError> {
        match filter {
            VideoFilter::Stabilization { .. } => {
                self.apply_stabilization(input)
            }
            VideoFilter::AutoExposure { target_brightness } => {
                self.apply_auto_exposure(input, *target_brightness)
            }
            VideoFilter::ToneMapping { exposure } => {
                self.apply_tone_mapping(input, *exposure)
            }
            VideoFilter::BrightnessContrast { brightness, contrast } => {
                Ok(self.apply_brightness_contrast(input, *brightness, *contrast))
            }
            VideoFilter::Saturation { value } => {
                Ok(self.apply_saturation(input, *value))
            }
            VideoFilter::Gamma { value } => {
                Ok(self.apply_gamma(input, *value))
            }
            VideoFilter::Sharpen { strength } => {
                Ok(self.apply_sharpen(input, *strength))
            }
            VideoFilter::Blur { radius } => {
                Ok(self.apply_blur(input, *radius))
            }
            VideoFilter::Grayscale => {
                Ok(input.to_grayscale().to_rgb24())
            }
            VideoFilter::Invert => {
                Ok(self.apply_invert(input))
            }
            VideoFilter::Sepia { strength } => {
                Ok(self.apply_sepia(input, *strength))
            }
            VideoFilter::Vignette { strength, radius } => {
                Ok(self.apply_vignette(input, *strength, *radius))
            }
            VideoFilter::EdgeDetect => {
                Ok(self.apply_edge_detect(input))
            }
            VideoFilter::Denoise { strength } => {
                Ok(self.apply_denoise(input, *strength))
            }
            VideoFilter::WhiteBalance { temperature, tint } => {
                Ok(self.apply_white_balance(input, *temperature, *tint))
            }
            VideoFilter::LensCorrection { k1, k2 } => {
                Ok(self.apply_lens_correction(input, *k1, *k2))
            }
            VideoFilter::CropScale { x, y, width, height } => {
                input.crop(*x, *y, *width, *height)
                    .ok_or(VideoError::ProcessingError("Invalid crop region".into()))
            }
            VideoFilter::Rotate { angle } => {
                Ok(self.apply_rotate(input, *angle))
            }
            VideoFilter::Flip { horizontal, vertical } => {
                let mut result = input.clone();
                if *horizontal {
                    result = result.flip_horizontal();
                }
                if *vertical {
                    result = result.flip_vertical();
                }
                Ok(result)
            }
            VideoFilter::Lut { table } => {
                Ok(self.apply_lut(input, table))
            }
            VideoFilter::ChromaKey { hue, tolerance } => {
                Ok(self.apply_chroma_key(input, *hue, *tolerance))
            }
            VideoFilter::FaceDetect { .. } => {
                // Face detection would use ML - just pass through
                Ok(input.clone())
            }
        }
    }

    /// Calculate average brightness
    fn calculate_brightness(&self, frame: &VideoFrame) -> f32 {
        let sum: u64 = frame.data.iter().map(|&x| x as u64).sum();
        sum as f32 / frame.data.len() as f32 / 255.0
    }

    /// Apply image stabilization
    fn apply_stabilization(&mut self, input: &VideoFrame) -> Result<VideoFrame, VideoError> {
        // Simple stabilization - would use feature tracking in real impl
        if let Some(ref mut stabilizer) = self.stabilizer {
            let motion = (0.0, 0.0); // Would detect actual motion
            let (offset_x, offset_y) = stabilizer.update(motion);

            // Apply offset by cropping and scaling
            let crop_margin = 20;
            let new_x = (crop_margin as f32 + offset_x).max(0.0) as u32;
            let new_y = (crop_margin as f32 + offset_y).max(0.0) as u32;

            if let Some(cropped) = input.crop(
                new_x,
                new_y,
                input.width - crop_margin * 2,
                input.height - crop_margin * 2,
            ) {
                return Ok(cropped.resize(input.width, input.height));
            }
        }
        Ok(input.clone())
    }

    /// Apply auto exposure adjustment
    fn apply_auto_exposure(&mut self, input: &VideoFrame, target: f32) -> Result<VideoFrame, VideoError> {
        let current = self.calculate_brightness(input);
        
        if current > 0.01 {
            let adjustment = (target / current).clamp(0.5, 2.0);
            Ok(self.apply_brightness_contrast(input, adjustment - 1.0, 1.0))
        } else {
            Ok(input.clone())
        }
    }

    /// Apply HDR tone mapping
    fn apply_tone_mapping(&self, input: &VideoFrame, exposure: f32) -> Result<VideoFrame, VideoError> {
        let mut output = input.clone();

        for chunk in output.data.chunks_mut(3) {
            for c in chunk.iter_mut() {
                let v = *c as f32 / 255.0;
                // Reinhard tone mapping
                let mapped = v * exposure / (1.0 + v * exposure);
                *c = (mapped * 255.0).clamp(0.0, 255.0) as u8;
            }
        }

        Ok(output)
    }

    /// Apply brightness and contrast
    fn apply_brightness_contrast(&self, input: &VideoFrame, brightness: f32, contrast: f32) -> VideoFrame {
        let mut output = input.clone();

        for c in &mut output.data {
            let v = *c as f32 / 255.0;
            let adjusted = (v - 0.5) * contrast + 0.5 + brightness;
            *c = (adjusted * 255.0).clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply saturation
    fn apply_saturation(&self, input: &VideoFrame, saturation: f32) -> VideoFrame {
        let mut output = input.clone();

        for chunk in output.data.chunks_mut(3) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;

            // Calculate grayscale (luminance)
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;

            // Interpolate between gray and original color
            chunk[0] = (gray + (r - gray) * saturation).clamp(0.0, 255.0) as u8;
            chunk[1] = (gray + (g - gray) * saturation).clamp(0.0, 255.0) as u8;
            chunk[2] = (gray + (b - gray) * saturation).clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply gamma correction
    fn apply_gamma(&self, input: &VideoFrame, gamma: f32) -> VideoFrame {
        let mut output = input.clone();
        let inv_gamma = 1.0 / gamma;

        for c in &mut output.data {
            let v = *c as f32 / 255.0;
            *c = (v.powf(inv_gamma) * 255.0).clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply sharpening
    fn apply_sharpen(&self, input: &VideoFrame, strength: f32) -> VideoFrame {
        // Unsharp mask: original + strength * (original - blurred)
        let blurred = self.apply_blur(input, 1);
        let mut output = input.clone();

        for i in 0..output.data.len() {
            let orig = input.data[i] as f32;
            let blur = blurred.data[i] as f32;
            let sharp = orig + strength * (orig - blur);
            output.data[i] = sharp.clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply Gaussian blur
    fn apply_blur(&self, input: &VideoFrame, radius: u32) -> VideoFrame {
        // Simple box blur for performance
        let mut output = input.clone();
        let r = radius as i32;

        for y in 0..input.height as i32 {
            for x in 0..input.width as i32 {
                let mut sum = [0u32; 3];
                let mut count = 0u32;

                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = (x + dx).clamp(0, input.width as i32 - 1) as u32;
                        let ny = (y + dy).clamp(0, input.height as i32 - 1) as u32;
                        let idx = (ny * input.stride + nx * 3) as usize;

                        sum[0] += input.data[idx] as u32;
                        sum[1] += input.data[idx + 1] as u32;
                        sum[2] += input.data[idx + 2] as u32;
                        count += 1;
                    }
                }

                let idx = (y as u32 * output.stride + x as u32 * 3) as usize;
                output.data[idx] = (sum[0] / count) as u8;
                output.data[idx + 1] = (sum[1] / count) as u8;
                output.data[idx + 2] = (sum[2] / count) as u8;
            }
        }

        output
    }

    /// Apply color inversion
    fn apply_invert(&self, input: &VideoFrame) -> VideoFrame {
        let mut output = input.clone();

        for c in &mut output.data {
            *c = 255 - *c;
        }

        output
    }

    /// Apply sepia effect
    fn apply_sepia(&self, input: &VideoFrame, strength: f32) -> VideoFrame {
        let mut output = input.clone();

        for chunk in output.data.chunks_mut(3) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;

            // Sepia transform
            let sr = 0.393 * r + 0.769 * g + 0.189 * b;
            let sg = 0.349 * r + 0.686 * g + 0.168 * b;
            let sb = 0.272 * r + 0.534 * g + 0.131 * b;

            // Blend with original
            chunk[0] = (r + (sr - r) * strength).clamp(0.0, 255.0) as u8;
            chunk[1] = (g + (sg - g) * strength).clamp(0.0, 255.0) as u8;
            chunk[2] = (b + (sb - b) * strength).clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply vignette effect
    fn apply_vignette(&self, input: &VideoFrame, strength: f32, radius: f32) -> VideoFrame {
        let mut output = input.clone();
        let cx = input.width as f32 / 2.0;
        let cy = input.height as f32 / 2.0;
        let max_dist = ((cx * cx + cy * cy) as f32).sqrt();

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt() / max_dist;

                // Vignette falloff
                let vignette = 1.0 - strength * ((dist / radius).clamp(0.0, 1.0)).powf(2.0);

                let idx = (y * output.stride + x * 3) as usize;
                output.data[idx] = (output.data[idx] as f32 * vignette) as u8;
                output.data[idx + 1] = (output.data[idx + 1] as f32 * vignette) as u8;
                output.data[idx + 2] = (output.data[idx + 2] as f32 * vignette) as u8;
            }
        }

        output
    }

    /// Apply edge detection (Sobel)
    fn apply_edge_detect(&self, input: &VideoFrame) -> VideoFrame {
        let gray = input.to_grayscale();
        let mut output = VideoFrame::new(input.width, input.height, PixelFormat::RGB24);

        let sobel_x: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let sobel_y: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        for y in 1..input.height - 1 {
            for x in 1..input.width - 1 {
                let mut gx = 0i32;
                let mut gy = 0i32;

                for dy in 0..3 {
                    for dx in 0..3 {
                        let px = (x + dx - 1) as usize;
                        let py = (y + dy - 1) as usize;
                        let val = gray.data[py * gray.width as usize + px] as i32;

                        gx += val * sobel_x[dy as usize][dx as usize];
                        gy += val * sobel_y[dy as usize][dx as usize];
                    }
                }

                let magnitude = ((gx * gx + gy * gy) as f32).sqrt().min(255.0) as u8;
                let idx = (y * output.stride + x * 3) as usize;
                output.data[idx] = magnitude;
                output.data[idx + 1] = magnitude;
                output.data[idx + 2] = magnitude;
            }
        }

        output
    }

    /// Apply noise reduction (simple)
    fn apply_denoise(&self, input: &VideoFrame, strength: f32) -> VideoFrame {
        // Bilateral-like filter (simplified)
        let mut output = input.clone();
        let radius = 2;
        let sigma_space = 2.0 * strength;
        let sigma_color = 30.0 * strength;

        for y in radius..input.height - radius {
            for x in radius..input.width - radius {
                let center_idx = (y * input.stride + x * 3) as usize;
                let center = [
                    input.data[center_idx] as f32,
                    input.data[center_idx + 1] as f32,
                    input.data[center_idx + 2] as f32,
                ];

                let mut sum = [0.0f32; 3];
                let mut weight_sum = 0.0f32;

                for dy in -(radius as i32)..=(radius as i32) {
                    for dx in -(radius as i32)..=(radius as i32) {
                        let nx = (x as i32 + dx) as u32;
                        let ny = (y as i32 + dy) as u32;
                        let idx = (ny * input.stride + nx * 3) as usize;

                        let pixel = [
                            input.data[idx] as f32,
                            input.data[idx + 1] as f32,
                            input.data[idx + 2] as f32,
                        ];

                        // Spatial weight
                        let space_dist = (dx * dx + dy * dy) as f32;
                        let space_weight = (-space_dist / (2.0 * sigma_space * sigma_space)).exp();

                        // Color weight
                        let color_dist = (center[0] - pixel[0]).powi(2)
                            + (center[1] - pixel[1]).powi(2)
                            + (center[2] - pixel[2]).powi(2);
                        let color_weight = (-color_dist / (2.0 * sigma_color * sigma_color)).exp();

                        let weight = space_weight * color_weight;
                        sum[0] += pixel[0] * weight;
                        sum[1] += pixel[1] * weight;
                        sum[2] += pixel[2] * weight;
                        weight_sum += weight;
                    }
                }

                let out_idx = (y * output.stride + x * 3) as usize;
                output.data[out_idx] = (sum[0] / weight_sum).clamp(0.0, 255.0) as u8;
                output.data[out_idx + 1] = (sum[1] / weight_sum).clamp(0.0, 255.0) as u8;
                output.data[out_idx + 2] = (sum[2] / weight_sum).clamp(0.0, 255.0) as u8;
            }
        }

        output
    }

    /// Apply white balance correction
    fn apply_white_balance(&self, input: &VideoFrame, temperature: f32, tint: f32) -> VideoFrame {
        let mut output = input.clone();

        // Temperature: negative = cooler (more blue), positive = warmer (more red)
        // Tint: negative = more green, positive = more magenta
        let r_mult = 1.0 + temperature * 0.1;
        let g_mult = 1.0 - tint * 0.1;
        let b_mult = 1.0 - temperature * 0.1;

        for chunk in output.data.chunks_mut(3) {
            chunk[0] = (chunk[0] as f32 * r_mult).clamp(0.0, 255.0) as u8;
            chunk[1] = (chunk[1] as f32 * g_mult).clamp(0.0, 255.0) as u8;
            chunk[2] = (chunk[2] as f32 * b_mult).clamp(0.0, 255.0) as u8;
        }

        output
    }

    /// Apply lens distortion correction
    fn apply_lens_correction(&self, input: &VideoFrame, k1: f32, k2: f32) -> VideoFrame {
        let mut output = VideoFrame::new(input.width, input.height, PixelFormat::RGB24);
        let cx = input.width as f32 / 2.0;
        let cy = input.height as f32 / 2.0;
        let max_r = ((cx * cx + cy * cy) as f32).sqrt();

        for y in 0..input.height {
            for x in 0..input.width {
                // Normalized coordinates
                let dx = (x as f32 - cx) / max_r;
                let dy = (y as f32 - cy) / max_r;
                let r2 = dx * dx + dy * dy;
                let r4 = r2 * r2;

                // Radial distortion
                let factor = 1.0 + k1 * r2 + k2 * r4;
                let src_x = cx + dx * factor * max_r;
                let src_y = cy + dy * factor * max_r;

                // Sample source (with bounds check)
                if src_x >= 0.0 && src_x < input.width as f32 - 1.0
                    && src_y >= 0.0 && src_y < input.height as f32 - 1.0
                {
                    let sx = src_x as u32;
                    let sy = src_y as u32;
                    let src_idx = (sy * input.stride + sx * 3) as usize;
                    let dst_idx = (y * output.stride + x * 3) as usize;
                    output.data[dst_idx..dst_idx + 3].copy_from_slice(&input.data[src_idx..src_idx + 3]);
                }
            }
        }

        output
    }

    /// Apply rotation
    fn apply_rotate(&self, input: &VideoFrame, angle: f32) -> VideoFrame {
        // For 90 degree rotations, use optimized version
        if (angle - 90.0).abs() < 0.1 {
            return input.rotate_90_cw();
        }

        // General rotation (simplified - proper implementation would use bilinear interpolation)
        let mut output = VideoFrame::new(input.width, input.height, PixelFormat::RGB24);
        let cx = input.width as f32 / 2.0;
        let cy = input.height as f32 / 2.0;
        let cos_a = angle.to_radians().cos();
        let sin_a = angle.to_radians().sin();

        for y in 0..input.height {
            for x in 0..input.width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;

                let src_x = (dx * cos_a + dy * sin_a + cx) as i32;
                let src_y = (-dx * sin_a + dy * cos_a + cy) as i32;

                if src_x >= 0 && src_x < input.width as i32
                    && src_y >= 0 && src_y < input.height as i32
                {
                    let src_idx = (src_y as u32 * input.stride + src_x as u32 * 3) as usize;
                    let dst_idx = (y * output.stride + x * 3) as usize;
                    output.data[dst_idx..dst_idx + 3].copy_from_slice(&input.data[src_idx..src_idx + 3]);
                }
            }
        }

        output
    }

    /// Apply LUT (Look-Up Table)
    fn apply_lut(&self, input: &VideoFrame, lut: &[u8]) -> VideoFrame {
        if lut.len() < 256 {
            return input.clone();
        }

        let mut output = input.clone();

        for c in &mut output.data {
            *c = lut[*c as usize];
        }

        output
    }

    /// Apply chroma key (green screen)
    fn apply_chroma_key(&self, input: &VideoFrame, key_hue: f32, tolerance: f32) -> VideoFrame {
        let mut output = input.clone();

        for y in 0..input.height {
            for x in 0..input.width {
                let idx = (y * input.stride + x * 3) as usize;
                let r = input.data[idx] as f32;
                let g = input.data[idx + 1] as f32;
                let b = input.data[idx + 2] as f32;

                // Convert to HSV
                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                let delta = max - min;

                let hue = if delta < 0.001 {
                    0.0
                } else if max == r {
                    60.0 * (((g - b) / delta) % 6.0)
                } else if max == g {
                    60.0 * ((b - r) / delta + 2.0)
                } else {
                    60.0 * ((r - g) / delta + 4.0)
                };

                let saturation = if max < 0.001 { 0.0 } else { delta / max };

                // Check if matches key color
                let hue_diff = (hue - key_hue).abs().min(360.0 - (hue - key_hue).abs());
                if hue_diff < tolerance * 180.0 && saturation > 0.2 {
                    // Make transparent (set to black for now - real impl would use alpha)
                    output.data[idx] = 0;
                    output.data[idx + 1] = 0;
                    output.data[idx + 2] = 0;
                }
            }
        }

        output
    }

    /// Get processing metrics
    pub fn metrics(&self) -> &ProcessingMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ProcessingMetrics::default();
    }
}

/// Processing pipeline builder
pub struct ProcessingPipeline {
    filters: Vec<VideoFilter>,
}

impl ProcessingPipeline {
    /// Create new pipeline builder
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    /// Add filter
    pub fn add(mut self, filter: VideoFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Build and apply to processor
    pub fn build(self, processor: &mut VideoProcessor) {
        processor.clear_filters();
        for filter in self.filters {
            processor.add_filter(filter);
        }
    }
}

impl Default for ProcessingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame() -> VideoFrame {
        let mut frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        // Fill with gradient
        for y in 0..100 {
            for x in 0..100 {
                frame.set_pixel(x, y, x as u8 * 2, y as u8 * 2, 128, 255);
            }
        }
        frame
    }

    #[test]
    fn test_processor_creation() {
        let processor = VideoProcessor::new(640, 480);
        assert_eq!(processor.filter_count(), 0);
    }

    #[test]
    fn test_add_remove_filter() {
        let mut processor = VideoProcessor::new(640, 480);
        
        processor.add_filter(VideoFilter::Grayscale);
        assert_eq!(processor.filter_count(), 1);

        processor.add_filter(VideoFilter::Invert);
        assert_eq!(processor.filter_count(), 2);

        processor.remove_filter(0);
        assert_eq!(processor.filter_count(), 1);
    }

    #[test]
    fn test_brightness_contrast() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::BrightnessContrast {
            brightness: 0.2,
            contrast: 1.0,
        });

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Output should be brighter
        let input_avg = processor.calculate_brightness(&input);
        let output_avg = processor.calculate_brightness(&output);
        assert!(output_avg > input_avg);
    }

    #[test]
    fn test_grayscale() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Grayscale);

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Check R=G=B for each pixel
        for chunk in output.data.chunks(3) {
            assert!((chunk[0] as i32 - chunk[1] as i32).abs() < 2);
            assert!((chunk[1] as i32 - chunk[2] as i32).abs() < 2);
        }
    }

    #[test]
    fn test_invert() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Invert);

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Check inversion
        let (r, g, b, _) = input.get_pixel(50, 50).unwrap();
        let (ir, ig, ib, _) = output.get_pixel(50, 50).unwrap();
        assert_eq!(ir, 255 - r);
        assert_eq!(ig, 255 - g);
        assert_eq!(ib, 255 - b);
    }

    #[test]
    fn test_blur() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Blur { radius: 2 });

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        assert_eq!(output.width, input.width);
        assert_eq!(output.height, input.height);
    }

    #[test]
    fn test_vignette() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Vignette {
            strength: 0.5,
            radius: 0.8,
        });

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Corners should be darker than center
        let (cr, cg, cb, _) = output.get_pixel(50, 50).unwrap();
        let (tr, tg, tb, _) = output.get_pixel(0, 0).unwrap();

        let center_brightness = (cr as u32 + cg as u32 + cb as u32) / 3;
        let corner_brightness = (tr as u32 + tg as u32 + tb as u32) / 3;

        assert!(center_brightness > corner_brightness);
    }

    #[test]
    fn test_pipeline_builder() {
        let mut processor = VideoProcessor::new(100, 100);

        ProcessingPipeline::new()
            .add(VideoFilter::Grayscale)
            .add(VideoFilter::BrightnessContrast { brightness: 0.1, contrast: 1.2 })
            .add(VideoFilter::Sharpen { strength: 0.5 })
            .build(&mut processor);

        assert_eq!(processor.filter_count(), 3);
    }

    #[test]
    fn test_sepia() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Sepia { strength: 1.0 });

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Sepia should have warm tones (more red than blue)
        let mut r_sum = 0u64;
        let mut b_sum = 0u64;
        for chunk in output.data.chunks(3) {
            r_sum += chunk[0] as u64;
            b_sum += chunk[2] as u64;
        }
        assert!(r_sum > b_sum);
    }

    #[test]
    fn test_saturation() {
        let mut processor = VideoProcessor::new(100, 100);

        let input = test_frame();

        // Desaturate
        processor.add_filter(VideoFilter::Saturation { value: 0.0 });
        let desaturated = processor.process(&input).unwrap();

        // Should be grayscale
        for chunk in desaturated.data.chunks(3) {
            assert!((chunk[0] as i32 - chunk[1] as i32).abs() < 5);
        }
    }

    #[test]
    fn test_gamma() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Gamma { value: 2.2 });

        let input = test_frame();
        let output = processor.process(&input).unwrap();

        // Gamma correction with gamma > 1 brightens midtones (inverse gamma)
        // At x=50, y=50: input = 100, with gamma 2.2 -> brighter
        let mid = input.data[50 * 3 * 100 + 50 * 3] as f32;
        let mid_out = output.data[50 * 3 * 100 + 50 * 3] as f32;
        assert!(mid_out > mid || mid > 245.0); // Midtones become brighter unless already bright
    }

    #[test]
    fn test_metrics() {
        let mut processor = VideoProcessor::new(100, 100);
        processor.add_filter(VideoFilter::Grayscale);

        let input = test_frame();
        processor.process(&input).unwrap();
        processor.process(&input).unwrap();

        assert_eq!(processor.metrics().frames_processed, 2);
        assert!(processor.metrics().avg_time_us > 0.0);
    }
}
