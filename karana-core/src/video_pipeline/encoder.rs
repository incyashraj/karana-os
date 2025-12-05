// Kāraṇa OS - Video Encoder Module
// Hardware-accelerated video encoding for smart glasses

use std::collections::VecDeque;

use super::frame::{VideoFrame, PixelFormat};
use super::VideoError;

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// H.264/AVC
    H264,
    /// H.265/HEVC
    H265,
    /// VP9
    VP9,
    /// AV1
    AV1,
    /// MJPEG
    MJPEG,
}

impl VideoCodec {
    /// Get codec name
    pub fn name(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264/AVC",
            VideoCodec::H265 => "H.265/HEVC",
            VideoCodec::VP9 => "VP9",
            VideoCodec::AV1 => "AV1",
            VideoCodec::MJPEG => "Motion JPEG",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "video/h264",
            VideoCodec::H265 => "video/h265",
            VideoCodec::VP9 => "video/webm",
            VideoCodec::AV1 => "video/av1",
            VideoCodec::MJPEG => "video/mjpeg",
        }
    }

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            VideoCodec::H264 | VideoCodec::H265 => "mp4",
            VideoCodec::VP9 | VideoCodec::AV1 => "webm",
            VideoCodec::MJPEG => "mjpeg",
        }
    }
}

/// Encoder profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderProfile {
    /// Baseline profile (H.264) - low complexity
    Baseline,
    /// Main profile - balanced
    Main,
    /// High profile - best quality
    High,
}

/// Rate control mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateControl {
    /// Constant bitrate
    CBR,
    /// Variable bitrate
    VBR,
    /// Constant quality
    CQ,
    /// Constant rate factor
    CRF,
}

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Video codec
    pub codec: VideoCodec,
    /// Encoder profile
    pub profile: EncoderProfile,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Frame rate (fps)
    pub framerate: f32,
    /// Target bitrate (bits/s)
    pub bitrate: u32,
    /// GOP size (keyframe interval)
    pub gop_size: u32,
    /// B-frames count
    pub b_frames: u32,
    /// Rate control mode
    pub rate_control: RateControl,
    /// Quality (0-51, lower is better)
    pub quality: u8,
    /// Enable hardware acceleration
    pub hw_accel: bool,
    /// Lookahead frames
    pub lookahead: u32,
    /// Max bitrate (for VBR)
    pub max_bitrate: Option<u32>,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            profile: EncoderProfile::Main,
            width: 1280,
            height: 720,
            framerate: 30.0,
            bitrate: 2_000_000, // 2 Mbps
            gop_size: 60,
            b_frames: 2,
            rate_control: RateControl::VBR,
            quality: 23,
            hw_accel: true,
            lookahead: 10,
            max_bitrate: Some(4_000_000),
        }
    }
}

impl EncoderConfig {
    /// Create config for low latency streaming
    pub fn low_latency(width: u32, height: u32) -> Self {
        Self {
            codec: VideoCodec::H264,
            profile: EncoderProfile::Baseline,
            width,
            height,
            framerate: 30.0,
            bitrate: 1_500_000,
            gop_size: 30,
            b_frames: 0, // No B-frames for low latency
            rate_control: RateControl::CBR,
            quality: 28,
            hw_accel: true,
            lookahead: 0,
            max_bitrate: None,
        }
    }

    /// Create config for recording
    pub fn recording(width: u32, height: u32) -> Self {
        Self {
            codec: VideoCodec::H265,
            profile: EncoderProfile::High,
            width,
            height,
            framerate: 30.0,
            bitrate: 5_000_000,
            gop_size: 120,
            b_frames: 3,
            rate_control: RateControl::CRF,
            quality: 18,
            hw_accel: true,
            lookahead: 20,
            max_bitrate: Some(10_000_000),
        }
    }
}

/// Frame type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// I-frame (keyframe)
    Intra,
    /// P-frame (predicted)
    Predicted,
    /// B-frame (bidirectional)
    Bidirectional,
}

/// Encoded packet
#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Encoded data
    pub data: Vec<u8>,
    /// Presentation timestamp
    pub pts: i64,
    /// Decode timestamp
    pub dts: i64,
    /// Frame duration
    pub duration: i64,
    /// Frame type
    pub frame_type: FrameType,
    /// Is keyframe
    pub is_keyframe: bool,
}

/// Encoder statistics
#[derive(Debug, Default)]
pub struct EncoderStats {
    /// Frames encoded
    pub frames_encoded: u64,
    /// Total bytes output
    pub bytes_output: u64,
    /// Keyframes encoded
    pub keyframes: u64,
    /// Average bitrate (bits/s)
    pub avg_bitrate: f64,
    /// Average encode time (ms)
    pub avg_encode_time_ms: f64,
}

/// Video encoder
#[derive(Debug)]
pub struct VideoEncoder {
    /// Configuration
    config: EncoderConfig,
    /// Statistics
    stats: EncoderStats,
    /// Frame counter
    frame_count: u64,
    /// Pending frames for lookahead
    pending_frames: VecDeque<VideoFrame>,
    /// Time base (1/framerate)
    time_base: f64,
    /// Start timestamp
    start_pts: i64,
}

impl VideoEncoder {
    /// Create new encoder
    pub fn new(config: EncoderConfig) -> Result<Self, VideoError> {
        let time_base = 1.0 / config.framerate as f64;

        Ok(Self {
            config,
            stats: EncoderStats::default(),
            frame_count: 0,
            pending_frames: VecDeque::new(),
            time_base,
            start_pts: 0,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &EncoderConfig {
        &self.config
    }

    /// Encode a frame
    pub fn encode(&mut self, frame: &VideoFrame) -> Result<Option<EncodedPacket>, VideoError> {
        // Validate frame dimensions
        if frame.width != self.config.width || frame.height != self.config.height {
            return Err(VideoError::ConfigError(format!(
                "Frame size {}x{} doesn't match encoder {}x{}",
                frame.width, frame.height, self.config.width, self.config.height
            )));
        }

        let start = std::time::Instant::now();

        // Use frame as-is (conversion would happen in real impl)
        let yuv_frame = frame.clone();

        // Simulated encoding
        let packet = self.encode_internal(&yuv_frame)?;

        // Update stats
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.update_stats(&packet, elapsed);

        self.frame_count += 1;

        Ok(Some(packet))
    }

    /// Internal encoding logic
    fn encode_internal(&self, frame: &VideoFrame) -> Result<EncodedPacket, VideoError> {
        let is_keyframe = self.frame_count % self.config.gop_size as u64 == 0;

        let frame_type = if is_keyframe {
            FrameType::Intra
        } else if self.config.b_frames > 0 && self.frame_count % 3 != 0 {
            FrameType::Bidirectional
        } else {
            FrameType::Predicted
        };

        // Simulated compression
        let compressed = self.compress_frame(frame, is_keyframe)?;

        let pts = if frame.pts > 0 {
            frame.pts as i64
        } else {
            (self.frame_count as f64 * self.time_base * 90000.0) as i64
        };

        Ok(EncodedPacket {
            data: compressed,
            pts,
            dts: pts,
            duration: (self.time_base * 90000.0) as i64,
            frame_type,
            is_keyframe,
        })
    }

    /// Compress frame data
    fn compress_frame(&self, frame: &VideoFrame, is_keyframe: bool) -> Result<Vec<u8>, VideoError> {
        // Simulated compression - in real impl would use hardware encoder
        let quality_factor = 1.0 - (self.config.quality as f32 / 51.0);
        let compression_ratio = if is_keyframe { 10.0 } else { 30.0 };
        let size = (frame.data.len() as f32 / compression_ratio * quality_factor) as usize;

        // Create NAL unit structure (simplified)
        let mut output = Vec::with_capacity(size + 16);

        // NAL header
        match self.config.codec {
            VideoCodec::H264 => {
                // H.264 NAL start code
                output.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                // NAL type: 5 = IDR, 1 = non-IDR
                output.push(if is_keyframe { 0x65 } else { 0x41 });
            }
            VideoCodec::H265 => {
                // H.265 NAL start code
                output.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                // NAL type (simplified)
                output.push(if is_keyframe { 0x26 } else { 0x02 });
                output.push(0x01);
            }
            _ => {
                // Generic header
                output.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
            }
        }

        // Simulated compressed data (entropy coding simulation)
        let sample_step = frame.data.len() / size.max(1);
        for i in (0..frame.data.len()).step_by(sample_step.max(1)) {
            let mut byte = frame.data[i];
            // Prevent start code emulation
            if i > 2 && output.len() > 4 {
                let prev = &output[output.len()-3..];
                if prev == [0x00, 0x00, 0x00] {
                    output.push(0x03); // Emulation prevention byte
                }
            }
            // Simple delta coding simulation
            if !is_keyframe && i > 0 && i < frame.data.len() - 1 {
                byte = byte.wrapping_sub(frame.data[i - 1]);
            }
            output.push(byte);
            if output.len() >= size {
                break;
            }
        }

        Ok(output)
    }

    /// Update statistics
    fn update_stats(&mut self, packet: &EncodedPacket, encode_time_ms: f64) {
        self.stats.frames_encoded += 1;
        self.stats.bytes_output += packet.data.len() as u64;
        if packet.is_keyframe {
            self.stats.keyframes += 1;
        }

        // Calculate average bitrate
        let duration = self.stats.frames_encoded as f64 * self.time_base;
        if duration > 0.0 {
            self.stats.avg_bitrate = (self.stats.bytes_output * 8) as f64 / duration;
        }

        // Update average encode time
        let total_time = self.stats.avg_encode_time_ms * (self.stats.frames_encoded - 1) as f64;
        self.stats.avg_encode_time_ms = (total_time + encode_time_ms) / self.stats.frames_encoded as f64;
    }

    /// Get statistics
    pub fn stats(&self) -> &EncoderStats {
        &self.stats
    }

    /// Force keyframe on next encode
    pub fn force_keyframe(&mut self) {
        // Reset frame count to force keyframe
        self.frame_count = (self.frame_count / self.config.gop_size as u64) * self.config.gop_size as u64;
    }

    /// Flush encoder
    pub fn flush(&mut self) -> Vec<EncodedPacket> {
        // In real impl, flush pending frames
        Vec::new()
    }

    /// Reset encoder
    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.stats = EncoderStats::default();
        self.pending_frames.clear();
    }

    /// Stop encoder
    pub fn stop(&mut self) -> Result<(), VideoError> {
        self.flush();
        Ok(())
    }

    /// Start recording to file
    pub fn start_recording(&mut self, _path: &str) -> Result<(), VideoError> {
        // In real impl, open file for writing
        self.reset();
        Ok(())
    }

    /// Stop recording and return path
    pub fn stop_recording(&mut self) -> Result<String, VideoError> {
        // In real impl, finalize and close file
        self.flush();
        Ok(String::new())
    }
}

/// Video decoder
pub struct VideoDecoder {
    /// Codec
    codec: VideoCodec,
    /// Output width
    width: u32,
    /// Output height
    height: u32,
    /// Frames decoded
    frames_decoded: u64,
}

impl VideoDecoder {
    /// Create new decoder
    pub fn new(codec: VideoCodec, width: u32, height: u32) -> Self {
        Self {
            codec,
            width,
            height,
            frames_decoded: 0,
        }
    }

    /// Decode packet
    pub fn decode(&mut self, packet: &EncodedPacket) -> Result<VideoFrame, VideoError> {
        // Simulated decoding
        let mut frame = VideoFrame::new(self.width, self.height, PixelFormat::NV12);
        frame.pts = packet.pts as u64;

        // Fill with decoded data (simulated)
        for (i, byte) in frame.data.iter_mut().enumerate() {
            if i < packet.data.len() {
                *byte = packet.data[i % packet.data.len()];
            }
        }

        self.frames_decoded += 1;

        Ok(frame)
    }

    /// Flush decoder
    pub fn flush(&mut self) -> Vec<VideoFrame> {
        Vec::new()
    }
}

/// Transcoder for format conversion
pub struct Transcoder {
    decoder: VideoDecoder,
    encoder: VideoEncoder,
}

impl Transcoder {
    /// Create transcoder
    pub fn new(
        input_codec: VideoCodec,
        output_config: EncoderConfig,
    ) -> Result<Self, VideoError> {
        let decoder = VideoDecoder::new(input_codec, output_config.width, output_config.height);
        let encoder = VideoEncoder::new(output_config)?;

        Ok(Self { decoder, encoder })
    }

    /// Transcode packet
    pub fn transcode(&mut self, packet: &EncodedPacket) -> Result<Option<EncodedPacket>, VideoError> {
        let frame = self.decoder.decode(packet)?;
        self.encoder.encode(&frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame(width: u32, height: u32) -> VideoFrame {
        let mut frame = VideoFrame::new(width, height, PixelFormat::RGB24);
        for (i, byte) in frame.data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        frame
    }

    #[test]
    fn test_codec_info() {
        assert_eq!(VideoCodec::H264.name(), "H.264/AVC");
        assert_eq!(VideoCodec::H265.mime_type(), "video/h265");
        assert_eq!(VideoCodec::VP9.extension(), "webm");
    }

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert_eq!(config.codec, VideoCodec::H264);
        assert_eq!(config.profile, EncoderProfile::Main);
        assert_eq!(config.framerate, 30.0);
    }

    #[test]
    fn test_low_latency_config() {
        let config = EncoderConfig::low_latency(1280, 720);
        assert_eq!(config.b_frames, 0);
        assert_eq!(config.lookahead, 0);
        assert_eq!(config.rate_control, RateControl::CBR);
    }

    #[test]
    fn test_recording_config() {
        let config = EncoderConfig::recording(1920, 1080);
        assert_eq!(config.codec, VideoCodec::H265);
        assert_eq!(config.profile, EncoderProfile::High);
        assert!(config.b_frames > 0);
    }

    #[test]
    fn test_encoder_creation() {
        let config = EncoderConfig::default();
        let encoder = VideoEncoder::new(config);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_encode_frame() {
        let config = EncoderConfig {
            width: 640,
            height: 480,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(640, 480);

        let result = encoder.encode(&frame);
        assert!(result.is_ok());

        let packet = result.unwrap().unwrap();
        assert!(packet.is_keyframe); // First frame should be keyframe
        assert!(!packet.data.is_empty());
    }

    #[test]
    fn test_gop_structure() {
        let config = EncoderConfig {
            width: 64,
            height: 64,
            gop_size: 4,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(64, 64);

        // Frames 0, 4, 8 should be keyframes
        for i in 0..10 {
            let packet = encoder.encode(&frame).unwrap().unwrap();
            if i % 4 == 0 {
                assert!(packet.is_keyframe, "Frame {} should be keyframe", i);
                assert_eq!(packet.frame_type, FrameType::Intra);
            } else {
                assert!(!packet.is_keyframe, "Frame {} should not be keyframe", i);
            }
        }
    }

    #[test]
    fn test_encoder_stats() {
        let config = EncoderConfig {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(64, 64);

        for _ in 0..5 {
            encoder.encode(&frame).unwrap();
        }

        let stats = encoder.stats();
        assert_eq!(stats.frames_encoded, 5);
        assert!(stats.bytes_output > 0);
        assert!(stats.avg_bitrate > 0.0);
        assert!(stats.avg_encode_time_ms > 0.0);
    }

    #[test]
    fn test_force_keyframe() {
        let config = EncoderConfig {
            width: 64,
            height: 64,
            gop_size: 30,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(64, 64);

        // Encode some frames
        for _ in 0..5 {
            encoder.encode(&frame).unwrap();
        }

        // Force keyframe
        encoder.force_keyframe();
        let packet = encoder.encode(&frame).unwrap().unwrap();
        assert!(packet.is_keyframe);
    }

    #[test]
    fn test_wrong_frame_size() {
        let config = EncoderConfig {
            width: 640,
            height: 480,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(320, 240); // Wrong size

        let result = encoder.encode(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_encoder_reset() {
        let config = EncoderConfig {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(64, 64);

        encoder.encode(&frame).unwrap();
        encoder.encode(&frame).unwrap();

        encoder.reset();

        assert_eq!(encoder.stats().frames_encoded, 0);

        // First frame after reset should be keyframe
        let packet = encoder.encode(&frame).unwrap().unwrap();
        assert!(packet.is_keyframe);
    }

    #[test]
    fn test_decoder_creation() {
        let decoder = VideoDecoder::new(VideoCodec::H264, 640, 480);
        assert_eq!(decoder.width, 640);
        assert_eq!(decoder.height, 480);
    }

    #[test]
    fn test_decode_packet() {
        let mut decoder = VideoDecoder::new(VideoCodec::H264, 64, 64);
        let packet = EncodedPacket {
            data: vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88],
            pts: 0,
            dts: 0,
            duration: 3000,
            frame_type: FrameType::Intra,
            is_keyframe: true,
        };

        let frame = decoder.decode(&packet).unwrap();
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 64);
    }

    #[test]
    fn test_transcoder() {
        let output_config = EncoderConfig {
            codec: VideoCodec::H265,
            width: 64,
            height: 64,
            ..Default::default()
        };
        let mut transcoder = Transcoder::new(VideoCodec::H264, output_config).unwrap();

        let input_packet = EncodedPacket {
            data: vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x99],
            pts: 0,
            dts: 0,
            duration: 3000,
            frame_type: FrameType::Intra,
            is_keyframe: true,
        };

        let output = transcoder.transcode(&input_packet);
        assert!(output.is_ok());
    }

    #[test]
    fn test_packet_timestamps() {
        let config = EncoderConfig {
            width: 64,
            height: 64,
            framerate: 30.0,
            ..Default::default()
        };
        let mut encoder = VideoEncoder::new(config).unwrap();
        let frame = test_frame(64, 64);

        let p1 = encoder.encode(&frame).unwrap().unwrap();
        let p2 = encoder.encode(&frame).unwrap().unwrap();
        let p3 = encoder.encode(&frame).unwrap().unwrap();

        // PTS should increase
        assert!(p2.pts > p1.pts);
        assert!(p3.pts > p2.pts);
    }
}
