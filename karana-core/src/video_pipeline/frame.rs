// Kāraṇa OS - Video Frame Module
// Video frame representation and buffer management

use std::sync::Arc;
use std::time::Instant;

/// Pixel format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// RGB24 - 3 bytes per pixel (R, G, B)
    RGB24,
    /// RGBA32 - 4 bytes per pixel (R, G, B, A)
    RGBA32,
    /// BGR24 - 3 bytes per pixel (B, G, R)
    BGR24,
    /// BGRA32 - 4 bytes per pixel (B, G, R, A)
    BGRA32,
    /// YUV420 planar (I420)
    YUV420,
    /// NV12 - Y plane + interleaved UV
    NV12,
    /// NV21 - Y plane + interleaved VU
    NV21,
    /// YUYV (YUY2) - packed YUV
    YUYV,
    /// UYVY - packed YUV
    UYVY,
    /// Grayscale 8-bit
    Gray8,
    /// Grayscale 16-bit
    Gray16,
}

impl PixelFormat {
    /// Bytes per pixel (for packed formats)
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::RGB24 | PixelFormat::BGR24 => 3,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => 4,
            PixelFormat::YUV420 | PixelFormat::NV12 | PixelFormat::NV21 => 0, // Planar
            PixelFormat::YUYV | PixelFormat::UYVY => 2,
            PixelFormat::Gray8 => 1,
            PixelFormat::Gray16 => 2,
        }
    }

    /// Calculate buffer size for given dimensions
    pub fn buffer_size(&self, width: u32, height: u32) -> usize {
        let w = width as usize;
        let h = height as usize;
        
        match self {
            PixelFormat::RGB24 | PixelFormat::BGR24 => w * h * 3,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => w * h * 4,
            PixelFormat::YUV420 => w * h + (w / 2) * (h / 2) * 2, // Y + U + V
            PixelFormat::NV12 | PixelFormat::NV21 => w * h + (w * h / 2), // Y + UV
            PixelFormat::YUYV | PixelFormat::UYVY => w * h * 2,
            PixelFormat::Gray8 => w * h,
            PixelFormat::Gray16 => w * h * 2,
        }
    }

    /// Check if format is planar
    pub fn is_planar(&self) -> bool {
        matches!(self, PixelFormat::YUV420 | PixelFormat::NV12 | PixelFormat::NV21)
    }

    /// Check if format has alpha channel
    pub fn has_alpha(&self) -> bool {
        matches!(self, PixelFormat::RGBA32 | PixelFormat::BGRA32)
    }

    /// Get the color space
    pub fn color_space(&self) -> ColorSpace {
        match self {
            PixelFormat::RGB24 | PixelFormat::RGBA32 | 
            PixelFormat::BGR24 | PixelFormat::BGRA32 => ColorSpace::RGB,
            PixelFormat::YUV420 | PixelFormat::NV12 | PixelFormat::NV21 |
            PixelFormat::YUYV | PixelFormat::UYVY => ColorSpace::YUV,
            PixelFormat::Gray8 | PixelFormat::Gray16 => ColorSpace::Grayscale,
        }
    }
}

/// Color space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    RGB,
    YUV,
    Grayscale,
}

/// Video frame representation
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Raw pixel data
    pub data: Vec<u8>,
    /// Timestamp (monotonic)
    pub timestamp: u64,
    /// Frame sequence number
    pub sequence: u64,
    /// Presentation timestamp (PTS) in microseconds
    pub pts: u64,
    /// Frame duration in microseconds
    pub duration: u64,
    /// Is keyframe (for encoded)
    pub keyframe: bool,
    /// Stride (bytes per row)
    pub stride: u32,
    /// Additional planes for planar formats
    pub planes: Vec<FramePlane>,
}

/// Frame plane for planar formats
#[derive(Debug, Clone)]
pub struct FramePlane {
    /// Plane data offset in main buffer
    pub offset: usize,
    /// Plane stride
    pub stride: u32,
    /// Plane height
    pub height: u32,
}

impl VideoFrame {
    /// Create new video frame
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        let size = format.buffer_size(width, height);
        let stride = match format {
            PixelFormat::RGB24 | PixelFormat::BGR24 => width * 3,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => width * 4,
            PixelFormat::YUV420 | PixelFormat::NV12 | PixelFormat::NV21 => width,
            PixelFormat::YUYV | PixelFormat::UYVY => width * 2,
            PixelFormat::Gray8 => width,
            PixelFormat::Gray16 => width * 2,
        };

        let planes = Self::create_planes(width, height, format);

        Self {
            width,
            height,
            format,
            data: vec![0u8; size],
            timestamp: 0,
            sequence: 0,
            pts: 0,
            duration: 0,
            keyframe: false,
            stride,
            planes,
        }
    }

    /// Create from existing data
    pub fn from_data(width: u32, height: u32, format: PixelFormat, data: Vec<u8>) -> Self {
        let stride = match format {
            PixelFormat::RGB24 | PixelFormat::BGR24 => width * 3,
            PixelFormat::RGBA32 | PixelFormat::BGRA32 => width * 4,
            PixelFormat::YUV420 | PixelFormat::NV12 | PixelFormat::NV21 => width,
            PixelFormat::YUYV | PixelFormat::UYVY => width * 2,
            PixelFormat::Gray8 => width,
            PixelFormat::Gray16 => width * 2,
        };

        let planes = Self::create_planes(width, height, format);

        Self {
            width,
            height,
            format,
            data,
            timestamp: 0,
            sequence: 0,
            pts: 0,
            duration: 0,
            keyframe: false,
            stride,
            planes,
        }
    }

    /// Create plane descriptors for planar formats
    fn create_planes(width: u32, height: u32, format: PixelFormat) -> Vec<FramePlane> {
        match format {
            PixelFormat::YUV420 => {
                let y_size = (width * height) as usize;
                let uv_stride = width / 2;
                let uv_height = height / 2;
                vec![
                    FramePlane { offset: 0, stride: width, height },
                    FramePlane { offset: y_size, stride: uv_stride, height: uv_height },
                    FramePlane { offset: y_size + (uv_stride * uv_height) as usize, stride: uv_stride, height: uv_height },
                ]
            }
            PixelFormat::NV12 | PixelFormat::NV21 => {
                let y_size = (width * height) as usize;
                vec![
                    FramePlane { offset: 0, stride: width, height },
                    FramePlane { offset: y_size, stride: width, height: height / 2 },
                ]
            }
            _ => Vec::new(),
        }
    }

    /// Get pixel at coordinates (for packed RGB formats)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }

        match self.format {
            PixelFormat::RGB24 => {
                let offset = (y * self.stride + x * 3) as usize;
                Some((self.data[offset], self.data[offset + 1], self.data[offset + 2], 255))
            }
            PixelFormat::RGBA32 => {
                let offset = (y * self.stride + x * 4) as usize;
                Some((self.data[offset], self.data[offset + 1], self.data[offset + 2], self.data[offset + 3]))
            }
            PixelFormat::BGR24 => {
                let offset = (y * self.stride + x * 3) as usize;
                Some((self.data[offset + 2], self.data[offset + 1], self.data[offset], 255))
            }
            PixelFormat::BGRA32 => {
                let offset = (y * self.stride + x * 4) as usize;
                Some((self.data[offset + 2], self.data[offset + 1], self.data[offset], self.data[offset + 3]))
            }
            PixelFormat::Gray8 => {
                let offset = (y * self.stride + x) as usize;
                let v = self.data[offset];
                Some((v, v, v, 255))
            }
            _ => None, // YUV formats need conversion
        }
    }

    /// Set pixel at coordinates (for packed RGB formats)
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        match self.format {
            PixelFormat::RGB24 => {
                let offset = (y * self.stride + x * 3) as usize;
                self.data[offset] = r;
                self.data[offset + 1] = g;
                self.data[offset + 2] = b;
                true
            }
            PixelFormat::RGBA32 => {
                let offset = (y * self.stride + x * 4) as usize;
                self.data[offset] = r;
                self.data[offset + 1] = g;
                self.data[offset + 2] = b;
                self.data[offset + 3] = a;
                true
            }
            PixelFormat::BGR24 => {
                let offset = (y * self.stride + x * 3) as usize;
                self.data[offset] = b;
                self.data[offset + 1] = g;
                self.data[offset + 2] = r;
                true
            }
            PixelFormat::BGRA32 => {
                let offset = (y * self.stride + x * 4) as usize;
                self.data[offset] = b;
                self.data[offset + 1] = g;
                self.data[offset + 2] = r;
                self.data[offset + 3] = a;
                true
            }
            PixelFormat::Gray8 => {
                let offset = (y * self.stride + x) as usize;
                // Convert to grayscale using luminance formula
                self.data[offset] = ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8;
                true
            }
            _ => false,
        }
    }

    /// Get Y plane data (for YUV formats)
    pub fn y_plane(&self) -> Option<&[u8]> {
        if self.planes.is_empty() {
            return None;
        }
        let plane = &self.planes[0];
        let size = (plane.stride * plane.height) as usize;
        Some(&self.data[plane.offset..plane.offset + size])
    }

    /// Get UV/U plane data
    pub fn uv_plane(&self) -> Option<&[u8]> {
        if self.planes.len() < 2 {
            return None;
        }
        let plane = &self.planes[1];
        let size = (plane.stride * plane.height) as usize;
        Some(&self.data[plane.offset..plane.offset + size])
    }

    /// Convert to RGB24
    pub fn to_rgb24(&self) -> VideoFrame {
        if self.format == PixelFormat::RGB24 {
            return self.clone();
        }

        let mut output = VideoFrame::new(self.width, self.height, PixelFormat::RGB24);

        match self.format {
            PixelFormat::RGBA32 => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x * 4) as usize;
                        let dst = (y * output.stride + x * 3) as usize;
                        output.data[dst] = self.data[src];
                        output.data[dst + 1] = self.data[src + 1];
                        output.data[dst + 2] = self.data[src + 2];
                    }
                }
            }
            PixelFormat::BGR24 => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x * 3) as usize;
                        let dst = (y * output.stride + x * 3) as usize;
                        output.data[dst] = self.data[src + 2];
                        output.data[dst + 1] = self.data[src + 1];
                        output.data[dst + 2] = self.data[src];
                    }
                }
            }
            PixelFormat::BGRA32 => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x * 4) as usize;
                        let dst = (y * output.stride + x * 3) as usize;
                        output.data[dst] = self.data[src + 2];
                        output.data[dst + 1] = self.data[src + 1];
                        output.data[dst + 2] = self.data[src];
                    }
                }
            }
            PixelFormat::NV12 => {
                self.nv12_to_rgb(&mut output);
            }
            PixelFormat::YUV420 => {
                self.yuv420_to_rgb(&mut output);
            }
            PixelFormat::Gray8 => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x) as usize;
                        let dst = (y * output.stride + x * 3) as usize;
                        let v = self.data[src];
                        output.data[dst] = v;
                        output.data[dst + 1] = v;
                        output.data[dst + 2] = v;
                    }
                }
            }
            _ => {}
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output.pts = self.pts;
        output
    }

    /// Convert NV12 to RGB
    fn nv12_to_rgb(&self, output: &mut VideoFrame) {
        let y_plane = &self.data[..self.width as usize * self.height as usize];
        let uv_plane = &self.data[self.width as usize * self.height as usize..];

        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = (y * self.width + x) as usize;
                let uv_idx = ((y / 2) * self.width + (x / 2) * 2) as usize;

                let y_val = y_plane[y_idx] as f32;
                let u_val = uv_plane[uv_idx] as f32 - 128.0;
                let v_val = uv_plane[uv_idx + 1] as f32 - 128.0;

                // YUV to RGB conversion
                let r = (y_val + 1.402 * v_val).clamp(0.0, 255.0) as u8;
                let g = (y_val - 0.344 * u_val - 0.714 * v_val).clamp(0.0, 255.0) as u8;
                let b = (y_val + 1.772 * u_val).clamp(0.0, 255.0) as u8;

                let dst = (y * output.stride + x * 3) as usize;
                output.data[dst] = r;
                output.data[dst + 1] = g;
                output.data[dst + 2] = b;
            }
        }
    }

    /// Convert YUV420 to RGB
    fn yuv420_to_rgb(&self, output: &mut VideoFrame) {
        let y_size = (self.width * self.height) as usize;
        let uv_size = y_size / 4;

        let y_plane = &self.data[..y_size];
        let u_plane = &self.data[y_size..y_size + uv_size];
        let v_plane = &self.data[y_size + uv_size..];

        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = (y * self.width + x) as usize;
                let uv_idx = ((y / 2) * (self.width / 2) + x / 2) as usize;

                let y_val = y_plane[y_idx] as f32;
                let u_val = u_plane[uv_idx] as f32 - 128.0;
                let v_val = v_plane[uv_idx] as f32 - 128.0;

                let r = (y_val + 1.402 * v_val).clamp(0.0, 255.0) as u8;
                let g = (y_val - 0.344 * u_val - 0.714 * v_val).clamp(0.0, 255.0) as u8;
                let b = (y_val + 1.772 * u_val).clamp(0.0, 255.0) as u8;

                let dst = (y * output.stride + x * 3) as usize;
                output.data[dst] = r;
                output.data[dst + 1] = g;
                output.data[dst + 2] = b;
            }
        }
    }

    /// Convert to grayscale
    pub fn to_grayscale(&self) -> VideoFrame {
        if self.format == PixelFormat::Gray8 {
            return self.clone();
        }

        let mut output = VideoFrame::new(self.width, self.height, PixelFormat::Gray8);

        match self.format {
            PixelFormat::RGB24 | PixelFormat::RGBA32 => {
                let bpp = if self.format == PixelFormat::RGB24 { 3 } else { 4 };
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x * bpp) as usize;
                        let r = self.data[src] as u32;
                        let g = self.data[src + 1] as u32;
                        let b = self.data[src + 2] as u32;
                        let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                        output.data[(y * self.width + x) as usize] = gray;
                    }
                }
            }
            PixelFormat::BGR24 | PixelFormat::BGRA32 => {
                let bpp = if self.format == PixelFormat::BGR24 { 3 } else { 4 };
                for y in 0..self.height {
                    for x in 0..self.width {
                        let src = (y * self.stride + x * bpp) as usize;
                        let b = self.data[src] as u32;
                        let g = self.data[src + 1] as u32;
                        let r = self.data[src + 2] as u32;
                        let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                        output.data[(y * self.width + x) as usize] = gray;
                    }
                }
            }
            PixelFormat::NV12 | PixelFormat::YUV420 => {
                // Y plane is already grayscale
                let y_size = (self.width * self.height) as usize;
                output.data[..y_size].copy_from_slice(&self.data[..y_size]);
            }
            _ => {}
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output
    }

    /// Resize frame using bilinear interpolation
    pub fn resize(&self, new_width: u32, new_height: u32) -> VideoFrame {
        // Convert to RGB24 for resizing
        let rgb = if self.format == PixelFormat::RGB24 {
            self.clone()
        } else {
            self.to_rgb24()
        };

        let mut output = VideoFrame::new(new_width, new_height, PixelFormat::RGB24);

        let x_ratio = self.width as f32 / new_width as f32;
        let y_ratio = self.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = x as f32 * x_ratio;
                let src_y = y as f32 * y_ratio;

                let x0 = src_x as u32;
                let y0 = src_y as u32;
                let x1 = (x0 + 1).min(self.width - 1);
                let y1 = (y0 + 1).min(self.height - 1);

                let x_lerp = src_x - x0 as f32;
                let y_lerp = src_y - y0 as f32;

                // Bilinear interpolation
                for c in 0..3 {
                    let v00 = rgb.data[(y0 * rgb.stride + x0 * 3) as usize + c] as f32;
                    let v10 = rgb.data[(y0 * rgb.stride + x1 * 3) as usize + c] as f32;
                    let v01 = rgb.data[(y1 * rgb.stride + x0 * 3) as usize + c] as f32;
                    let v11 = rgb.data[(y1 * rgb.stride + x1 * 3) as usize + c] as f32;

                    let top = v00 * (1.0 - x_lerp) + v10 * x_lerp;
                    let bottom = v01 * (1.0 - x_lerp) + v11 * x_lerp;
                    let value = top * (1.0 - y_lerp) + bottom * y_lerp;

                    output.data[(y * output.stride + x * 3) as usize + c] = value as u8;
                }
            }
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output
    }

    /// Crop a region from the frame
    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Option<VideoFrame> {
        if x + width > self.width || y + height > self.height {
            return None;
        }

        // Convert to RGB24 for cropping
        let rgb = if self.format == PixelFormat::RGB24 {
            self.clone()
        } else {
            self.to_rgb24()
        };

        let mut output = VideoFrame::new(width, height, PixelFormat::RGB24);

        for dy in 0..height {
            let src_row = ((y + dy) * rgb.stride + x * 3) as usize;
            let dst_row = (dy * output.stride) as usize;
            let row_bytes = (width * 3) as usize;
            output.data[dst_row..dst_row + row_bytes].copy_from_slice(&rgb.data[src_row..src_row + row_bytes]);
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        Some(output)
    }

    /// Flip frame horizontally
    pub fn flip_horizontal(&self) -> VideoFrame {
        let rgb = if self.format == PixelFormat::RGB24 {
            self.clone()
        } else {
            self.to_rgb24()
        };

        let mut output = VideoFrame::new(self.width, self.height, PixelFormat::RGB24);

        for y in 0..self.height {
            for x in 0..self.width {
                let src = (y * rgb.stride + x * 3) as usize;
                let dst = (y * output.stride + (self.width - 1 - x) * 3) as usize;
                output.data[dst..dst + 3].copy_from_slice(&rgb.data[src..src + 3]);
            }
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output
    }

    /// Flip frame vertically
    pub fn flip_vertical(&self) -> VideoFrame {
        let rgb = if self.format == PixelFormat::RGB24 {
            self.clone()
        } else {
            self.to_rgb24()
        };

        let mut output = VideoFrame::new(self.width, self.height, PixelFormat::RGB24);
        let row_bytes = (self.width * 3) as usize;

        for y in 0..self.height {
            let src_row = (y * rgb.stride) as usize;
            let dst_row = ((self.height - 1 - y) * output.stride) as usize;
            output.data[dst_row..dst_row + row_bytes].copy_from_slice(&rgb.data[src_row..src_row + row_bytes]);
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output
    }

    /// Rotate frame 90 degrees clockwise
    pub fn rotate_90_cw(&self) -> VideoFrame {
        let rgb = if self.format == PixelFormat::RGB24 {
            self.clone()
        } else {
            self.to_rgb24()
        };

        let mut output = VideoFrame::new(self.height, self.width, PixelFormat::RGB24);

        for y in 0..self.height {
            for x in 0..self.width {
                let src = (y * rgb.stride + x * 3) as usize;
                let new_x = self.height - 1 - y;
                let new_y = x;
                let dst = (new_y * output.stride + new_x * 3) as usize;
                output.data[dst..dst + 3].copy_from_slice(&rgb.data[src..src + 3]);
            }
        }

        output.timestamp = self.timestamp;
        output.sequence = self.sequence;
        output
    }

    /// Get frame size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Frame buffer for pooling
#[derive(Debug)]
pub struct FrameBuffer {
    /// Pool of available frames
    pool: Vec<VideoFrame>,
    /// Maximum pool size
    max_size: usize,
    /// Frame dimensions
    width: u32,
    height: u32,
    /// Pixel format
    format: PixelFormat,
}

impl FrameBuffer {
    /// Create new frame buffer
    pub fn new(width: u32, height: u32, format: PixelFormat, pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(VideoFrame::new(width, height, format));
        }

        Self {
            pool,
            max_size: pool_size,
            width,
            height,
            format,
        }
    }

    /// Get a frame from the pool
    pub fn acquire(&mut self) -> VideoFrame {
        self.pool.pop().unwrap_or_else(|| {
            VideoFrame::new(self.width, self.height, self.format)
        })
    }

    /// Return a frame to the pool
    pub fn release(&mut self, frame: VideoFrame) {
        if self.pool.len() < self.max_size {
            self.pool.push(frame);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.pool.len(), self.max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_buffer_size() {
        assert_eq!(PixelFormat::RGB24.buffer_size(100, 100), 30000);
        assert_eq!(PixelFormat::RGBA32.buffer_size(100, 100), 40000);
        assert_eq!(PixelFormat::Gray8.buffer_size(100, 100), 10000);
    }

    #[test]
    fn test_video_frame_creation() {
        let frame = VideoFrame::new(640, 480, PixelFormat::RGB24);
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert_eq!(frame.format, PixelFormat::RGB24);
        assert_eq!(frame.data.len(), 640 * 480 * 3);
    }

    #[test]
    fn test_pixel_access() {
        let mut frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        
        frame.set_pixel(50, 50, 255, 128, 64, 255);
        let pixel = frame.get_pixel(50, 50).unwrap();
        
        assert_eq!(pixel.0, 255);
        assert_eq!(pixel.1, 128);
        assert_eq!(pixel.2, 64);
    }

    #[test]
    fn test_grayscale_conversion() {
        let mut frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        frame.set_pixel(50, 50, 255, 255, 255, 255);

        let gray = frame.to_grayscale();
        assert_eq!(gray.format, PixelFormat::Gray8);
        
        let pixel = gray.data[(50 * 100 + 50) as usize];
        assert_eq!(pixel, 255);
    }

    #[test]
    fn test_resize() {
        let frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        let resized = frame.resize(50, 50);
        
        assert_eq!(resized.width, 50);
        assert_eq!(resized.height, 50);
    }

    #[test]
    fn test_crop() {
        let frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        let cropped = frame.crop(10, 10, 50, 50).unwrap();
        
        assert_eq!(cropped.width, 50);
        assert_eq!(cropped.height, 50);
    }

    #[test]
    fn test_crop_out_of_bounds() {
        let frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        let result = frame.crop(80, 80, 50, 50);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_flip_horizontal() {
        let mut frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        frame.set_pixel(0, 50, 255, 0, 0, 255);
        
        let flipped = frame.flip_horizontal();
        let pixel = flipped.get_pixel(99, 50).unwrap();
        
        assert_eq!(pixel.0, 255);
    }

    #[test]
    fn test_flip_vertical() {
        let mut frame = VideoFrame::new(100, 100, PixelFormat::RGB24);
        frame.set_pixel(50, 0, 255, 0, 0, 255);
        
        let flipped = frame.flip_vertical();
        let pixel = flipped.get_pixel(50, 99).unwrap();
        
        assert_eq!(pixel.0, 255);
    }

    #[test]
    fn test_rotate_90() {
        let frame = VideoFrame::new(100, 50, PixelFormat::RGB24);
        let rotated = frame.rotate_90_cw();
        
        assert_eq!(rotated.width, 50);
        assert_eq!(rotated.height, 100);
    }

    #[test]
    fn test_frame_buffer() {
        let mut buffer = FrameBuffer::new(640, 480, PixelFormat::RGB24, 5);
        
        let (available, max) = buffer.stats();
        assert_eq!(available, 5);
        assert_eq!(max, 5);

        let frame1 = buffer.acquire();
        let (available, _) = buffer.stats();
        assert_eq!(available, 4);

        buffer.release(frame1);
        let (available, _) = buffer.stats();
        assert_eq!(available, 5);
    }

    #[test]
    fn test_nv12_format() {
        let frame = VideoFrame::new(640, 480, PixelFormat::NV12);
        assert_eq!(frame.planes.len(), 2);
        
        // Y plane: 640 * 480 = 307200
        // UV plane: 640 * 240 = 153600
        assert_eq!(frame.data.len(), 307200 + 153600);
    }

    #[test]
    fn test_yuv420_format() {
        let frame = VideoFrame::new(640, 480, PixelFormat::YUV420);
        assert_eq!(frame.planes.len(), 3);
        
        // Y: 640 * 480 = 307200
        // U: 320 * 240 = 76800
        // V: 320 * 240 = 76800
        assert_eq!(frame.data.len(), 307200 + 76800 + 76800);
    }
}
