// Kāraṇa OS - Video Streaming Module
// RTSP/WebRTC streaming for smart glasses

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use super::encoder::{EncodedPacket, FrameType, VideoCodec};
use super::VideoError;

/// Streaming protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamProtocol {
    /// RTSP (Real-Time Streaming Protocol)
    RTSP,
    /// WebRTC
    WebRTC,
    /// RTMP (Real-Time Messaging Protocol)
    RTMP,
    /// HLS (HTTP Live Streaming)
    HLS,
    /// SRT (Secure Reliable Transport)
    SRT,
}

/// Stream state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected and streaming
    Streaming,
    /// Paused
    Paused,
    /// Error state
    Error,
}

/// Stream quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityPreset {
    /// Low quality (360p)
    Low,
    /// Medium quality (720p)
    Medium,
    /// High quality (1080p)
    High,
    /// Ultra quality (4K)
    Ultra,
    /// Adaptive (automatic)
    Adaptive,
}

impl QualityPreset {
    /// Get resolution
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            QualityPreset::Low => (640, 360),
            QualityPreset::Medium => (1280, 720),
            QualityPreset::High => (1920, 1080),
            QualityPreset::Ultra => (3840, 2160),
            QualityPreset::Adaptive => (1280, 720), // Default
        }
    }

    /// Get target bitrate
    pub fn bitrate(&self) -> u32 {
        match self {
            QualityPreset::Low => 500_000,
            QualityPreset::Medium => 2_000_000,
            QualityPreset::High => 5_000_000,
            QualityPreset::Ultra => 15_000_000,
            QualityPreset::Adaptive => 2_000_000,
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Protocol
    pub protocol: StreamProtocol,
    /// Server URL
    pub url: String,
    /// Stream key (if required)
    pub stream_key: Option<String>,
    /// Quality preset
    pub quality: QualityPreset,
    /// Video codec
    pub codec: VideoCodec,
    /// Enable audio
    pub audio_enabled: bool,
    /// Buffer size (ms)
    pub buffer_ms: u32,
    /// Reconnect on failure
    pub auto_reconnect: bool,
    /// Max reconnect attempts
    pub max_reconnects: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            protocol: StreamProtocol::WebRTC,
            url: String::new(),
            stream_key: None,
            quality: QualityPreset::Medium,
            codec: VideoCodec::H264,
            audio_enabled: true,
            buffer_ms: 500,
            auto_reconnect: true,
            max_reconnects: 3,
        }
    }
}

/// RTSP session info
#[derive(Debug, Clone)]
pub struct RtspSession {
    /// Session ID
    pub session_id: String,
    /// Control URL
    pub control_url: String,
    /// Transport info
    pub transport: RtspTransport,
    /// Sequence number
    pub cseq: u32,
}

/// RTSP transport configuration
#[derive(Debug, Clone)]
pub struct RtspTransport {
    /// Client RTP port
    pub client_port: u16,
    /// Server RTP port
    pub server_port: Option<u16>,
    /// RTCP port
    pub rtcp_port: u16,
    /// Use TCP interleaved
    pub interleaved: bool,
}

/// RTP packet
#[derive(Debug, Clone)]
pub struct RtpPacket {
    /// Version (always 2)
    pub version: u8,
    /// Padding flag
    pub padding: bool,
    /// Extension flag
    pub extension: bool,
    /// CSRC count
    pub cc: u8,
    /// Marker bit
    pub marker: bool,
    /// Payload type
    pub payload_type: u8,
    /// Sequence number
    pub sequence: u16,
    /// Timestamp
    pub timestamp: u32,
    /// SSRC
    pub ssrc: u32,
    /// Payload data
    pub payload: Vec<u8>,
}

impl RtpPacket {
    /// Create new RTP packet
    pub fn new(sequence: u16, timestamp: u32, ssrc: u32, payload: Vec<u8>) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            cc: 0,
            marker: true,
            payload_type: 96, // Dynamic payload type for H.264
            sequence,
            timestamp,
            ssrc,
            payload,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12 + self.payload.len());

        // First byte: V=2, P, X, CC
        let byte0 = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | self.cc;
        bytes.push(byte0);

        // Second byte: M, PT
        let byte1 = ((self.marker as u8) << 7) | self.payload_type;
        bytes.push(byte1);

        // Sequence number (2 bytes, big endian)
        bytes.extend_from_slice(&self.sequence.to_be_bytes());

        // Timestamp (4 bytes, big endian)
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());

        // SSRC (4 bytes, big endian)
        bytes.extend_from_slice(&self.ssrc.to_be_bytes());

        // Payload
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }

        let version = (data[0] >> 6) & 0x03;
        let padding = ((data[0] >> 5) & 0x01) != 0;
        let extension = ((data[0] >> 4) & 0x01) != 0;
        let cc = data[0] & 0x0F;
        let marker = ((data[1] >> 7) & 0x01) != 0;
        let payload_type = data[1] & 0x7F;
        let sequence = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let header_len = 12 + (cc as usize * 4);
        if data.len() < header_len {
            return None;
        }

        let payload = data[header_len..].to_vec();

        Some(Self {
            version,
            padding,
            extension,
            cc,
            marker,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            payload,
        })
    }
}

/// Video streamer
#[derive(Debug)]
pub struct VideoStreamer {
    /// Configuration
    config: StreamConfig,
    /// Current state
    state: StreamState,
    /// RTSP session
    rtsp_session: Option<RtspSession>,
    /// RTP sequence number
    rtp_sequence: u16,
    /// RTP SSRC
    ssrc: u32,
    /// Statistics
    stats: StreamStats,
    /// Reconnect attempts
    reconnect_count: u32,
}

/// Streaming statistics
#[derive(Debug, Default)]
pub struct StreamStats {
    /// Packets sent
    pub packets_sent: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Packets lost
    pub packets_lost: u64,
    /// Jitter (ms)
    pub jitter_ms: f64,
    /// Round-trip time (ms)
    pub rtt_ms: f64,
    /// Current bitrate (bits/s)
    pub bitrate: u64,
    /// Stream uptime (seconds)
    pub uptime_secs: f64,
}

impl VideoStreamer {
    /// Create new streamer
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            state: StreamState::Disconnected,
            rtsp_session: None,
            rtp_sequence: rand::random(),
            ssrc: rand::random(),
            stats: StreamStats::default(),
            reconnect_count: 0,
        }
    }

    /// Get current state
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Get statistics
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Connect to server
    pub fn connect(&mut self) -> Result<(), VideoError> {
        self.state = StreamState::Connecting;

        match self.config.protocol {
            StreamProtocol::RTSP => self.connect_rtsp(),
            StreamProtocol::WebRTC => self.connect_webrtc(),
            StreamProtocol::RTMP => self.connect_rtmp(),
            _ => Err(VideoError::ConfigError("Unsupported protocol".into())),
        }
    }

    /// RTSP connection
    fn connect_rtsp(&mut self) -> Result<(), VideoError> {
        // Simulated RTSP handshake
        // OPTIONS -> DESCRIBE -> SETUP -> PLAY

        let session = RtspSession {
            session_id: format!("{:08X}", rand::random::<u32>()),
            control_url: format!("{}/trackID=0", self.config.url),
            transport: RtspTransport {
                client_port: 5000,
                server_port: Some(5001),
                rtcp_port: 5002,
                interleaved: false,
            },
            cseq: 1,
        };

        self.rtsp_session = Some(session);
        self.state = StreamState::Streaming;
        Ok(())
    }

    /// WebRTC connection
    fn connect_webrtc(&mut self) -> Result<(), VideoError> {
        // Simulated WebRTC setup
        // ICE gathering -> Offer/Answer -> DTLS -> SRTP
        self.state = StreamState::Streaming;
        Ok(())
    }

    /// RTMP connection
    fn connect_rtmp(&mut self) -> Result<(), VideoError> {
        // Simulated RTMP handshake
        self.state = StreamState::Streaming;
        Ok(())
    }

    /// Disconnect
    pub fn disconnect(&mut self) {
        if self.config.protocol == StreamProtocol::RTSP {
            // Send TEARDOWN
            self.rtsp_session = None;
        }
        self.state = StreamState::Disconnected;
    }

    /// Send video packet
    pub fn send(&mut self, packet: &EncodedPacket) -> Result<(), VideoError> {
        if self.state != StreamState::Streaming {
            return Err(VideoError::NotInitialized);
        }

        // Fragment into RTP packets (max MTU = 1400)
        let rtp_packets = self.fragment_to_rtp(packet)?;

        for rtp in &rtp_packets {
            self.send_rtp_packet(rtp)?;
        }

        // Update stats
        self.stats.packets_sent += rtp_packets.len() as u64;
        self.stats.bytes_sent += packet.data.len() as u64;

        Ok(())
    }

    /// Fragment encoded packet into RTP packets
    fn fragment_to_rtp(&mut self, packet: &EncodedPacket) -> Result<Vec<RtpPacket>, VideoError> {
        const MAX_RTP_PAYLOAD: usize = 1400;

        let mut rtp_packets = Vec::new();
        let timestamp = packet.pts as u32;

        if packet.data.len() <= MAX_RTP_PAYLOAD {
            // Single NAL unit
            let rtp = RtpPacket::new(
                self.next_sequence(),
                timestamp,
                self.ssrc,
                packet.data.clone(),
            );
            rtp_packets.push(rtp);
        } else {
            // Fragmentation (FU-A for H.264)
            let nal_header = packet.data.get(4).copied().unwrap_or(0);
            let nal_type = nal_header & 0x1F;
            let fu_indicator = (nal_header & 0xE0) | 28; // FU-A type

            let mut offset = 5; // Skip NAL start code + header
            let mut first = true;

            while offset < packet.data.len() {
                let end = (offset + MAX_RTP_PAYLOAD - 2).min(packet.data.len());
                let is_last = end >= packet.data.len();

                let mut fu_header = nal_type;
                if first {
                    fu_header |= 0x80; // Start bit
                    first = false;
                }
                if is_last {
                    fu_header |= 0x40; // End bit
                }

                let mut payload = vec![fu_indicator, fu_header];
                payload.extend_from_slice(&packet.data[offset..end]);

                let mut rtp = RtpPacket::new(
                    self.next_sequence(),
                    timestamp,
                    self.ssrc,
                    payload,
                );
                rtp.marker = is_last;
                rtp_packets.push(rtp);

                offset = end;
            }
        }

        Ok(rtp_packets)
    }

    /// Get next RTP sequence number
    fn next_sequence(&mut self) -> u16 {
        let seq = self.rtp_sequence;
        self.rtp_sequence = self.rtp_sequence.wrapping_add(1);
        seq
    }

    /// Send RTP packet (simulated)
    fn send_rtp_packet(&mut self, _packet: &RtpPacket) -> Result<(), VideoError> {
        // In real implementation, send via UDP socket
        Ok(())
    }

    /// Send a video frame (convenience method)
    pub fn send_frame(&mut self, frame: &super::frame::VideoFrame) -> Result<(), VideoError> {
        if self.state != StreamState::Streaming {
            return Err(VideoError::ConfigError("Not streaming".into()));
        }

        // Create a simple encoded representation (in real impl, use encoder)
        let timestamp = frame.pts as u32;
        let rtp = RtpPacket::new(
            self.next_sequence(),
            timestamp,
            self.ssrc,
            frame.data.clone(),
        );
        
        self.send_rtp_packet(&rtp)?;
        self.stats.packets_sent += 1;
        self.stats.bytes_sent += frame.data.len() as u64;
        
        Ok(())
    }

    /// Pause streaming
    pub fn pause(&mut self) -> Result<(), VideoError> {
        if self.state == StreamState::Streaming {
            self.state = StreamState::Paused;
        }
        Ok(())
    }

    /// Resume streaming
    pub fn resume(&mut self) -> Result<(), VideoError> {
        if self.state == StreamState::Paused {
            self.state = StreamState::Streaming;
        }
        Ok(())
    }
}

/// WebRTC peer connection
#[derive(Debug)]
pub struct WebRtcPeer {
    /// Peer ID
    pub id: String,
    /// Local description (SDP)
    pub local_sdp: Option<String>,
    /// Remote description (SDP)
    pub remote_sdp: Option<String>,
    /// ICE candidates
    pub ice_candidates: Vec<IceCandidate>,
    /// Connection state
    pub state: PeerState,
}

/// ICE candidate
#[derive(Debug, Clone)]
pub struct IceCandidate {
    /// Candidate type
    pub candidate_type: IceCandidateType,
    /// Transport protocol
    pub protocol: String,
    /// IP address
    pub address: IpAddr,
    /// Port
    pub port: u16,
    /// Priority
    pub priority: u32,
}

/// ICE candidate type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceCandidateType {
    Host,
    ServerReflexive,
    PeerReflexive,
    Relay,
}

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

impl WebRtcPeer {
    /// Create new peer
    pub fn new(id: String) -> Self {
        Self {
            id,
            local_sdp: None,
            remote_sdp: None,
            ice_candidates: Vec::new(),
            state: PeerState::New,
        }
    }

    /// Create offer
    pub fn create_offer(&mut self) -> Result<String, VideoError> {
        let sdp = self.generate_sdp(true)?;
        self.local_sdp = Some(sdp.clone());
        Ok(sdp)
    }

    /// Create answer
    pub fn create_answer(&mut self) -> Result<String, VideoError> {
        if self.remote_sdp.is_none() {
            return Err(VideoError::ConfigError("No remote offer".into()));
        }
        let sdp = self.generate_sdp(false)?;
        self.local_sdp = Some(sdp.clone());
        Ok(sdp)
    }

    /// Set remote description
    pub fn set_remote_description(&mut self, sdp: String) -> Result<(), VideoError> {
        self.remote_sdp = Some(sdp);
        Ok(())
    }

    /// Add ICE candidate
    pub fn add_ice_candidate(&mut self, candidate: IceCandidate) {
        self.ice_candidates.push(candidate);
    }

    /// Generate SDP
    fn generate_sdp(&self, is_offer: bool) -> Result<String, VideoError> {
        let session_id = rand::random::<u64>();
        let sdp = format!(
            r#"v=0
o=- {} 2 IN IP4 127.0.0.1
s=-
t=0 0
a=group:BUNDLE 0
a=msid-semantic: WMS
m=video 9 UDP/TLS/RTP/SAVPF 96
c=IN IP4 0.0.0.0
a=rtcp:9 IN IP4 0.0.0.0
a=ice-ufrag:{}
a=ice-pwd:{}
a=fingerprint:sha-256 {}
a=setup:{}
a=mid:0
a=sendonly
a=rtcp-mux
a=rtpmap:96 H264/90000
a=fmtp:96 level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f
"#,
            session_id,
            Self::random_string(4),
            Self::random_string(22),
            Self::random_fingerprint(),
            if is_offer { "actpass" } else { "active" }
        );
        Ok(sdp)
    }

    fn random_string(len: usize) -> String {
        (0..len)
            .map(|_| {
                let idx = rand::random::<u8>() % 36;
                if idx < 26 {
                    (b'a' + idx) as char
                } else {
                    (b'0' + idx - 26) as char
                }
            })
            .collect()
    }

    fn random_fingerprint() -> String {
        (0..32)
            .map(|i| {
                if i > 0 && i % 2 == 0 {
                    ':'
                } else {
                    let hex = rand::random::<u8>() % 16;
                    if hex < 10 {
                        (b'0' + hex) as char
                    } else {
                        (b'A' + hex - 10) as char
                    }
                }
            })
            .collect()
    }
}

/// Stream multiplexer for adaptive bitrate streaming
pub struct StreamMultiplexer {
    /// Quality variants
    variants: HashMap<QualityPreset, StreamVariant>,
    /// Current quality
    current_quality: QualityPreset,
    /// Bandwidth estimate (bits/s)
    bandwidth_estimate: u64,
}

/// Stream variant info
#[derive(Debug, Clone)]
pub struct StreamVariant {
    /// Quality preset
    pub quality: QualityPreset,
    /// Resolution
    pub resolution: (u32, u32),
    /// Bitrate
    pub bitrate: u32,
    /// Segment duration (ms)
    pub segment_duration_ms: u32,
}

impl StreamMultiplexer {
    /// Create new multiplexer
    pub fn new() -> Self {
        let mut variants = HashMap::new();
        
        for quality in [QualityPreset::Low, QualityPreset::Medium, QualityPreset::High] {
            variants.insert(quality, StreamVariant {
                quality,
                resolution: quality.resolution(),
                bitrate: quality.bitrate(),
                segment_duration_ms: 2000,
            });
        }

        Self {
            variants,
            current_quality: QualityPreset::Medium,
            bandwidth_estimate: 2_000_000,
        }
    }

    /// Update bandwidth estimate
    pub fn update_bandwidth(&mut self, bandwidth: u64) {
        // Exponential moving average
        self.bandwidth_estimate = (self.bandwidth_estimate * 7 + bandwidth) / 8;

        // Select appropriate quality
        self.current_quality = if self.bandwidth_estimate > 6_000_000 {
            QualityPreset::High
        } else if self.bandwidth_estimate > 1_000_000 {
            QualityPreset::Medium
        } else {
            QualityPreset::Low
        };
    }

    /// Get current quality
    pub fn current_quality(&self) -> QualityPreset {
        self.current_quality
    }

    /// Get variant for quality
    pub fn get_variant(&self, quality: QualityPreset) -> Option<&StreamVariant> {
        self.variants.get(&quality)
    }
}

impl Default for StreamMultiplexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple random number generator
mod rand {
    use std::cell::Cell;

    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x853c49e6748fea9b);
    }

    pub fn random<T: RandomValue>() -> T {
        T::random_value()
    }

    pub trait RandomValue {
        fn random_value() -> Self;
    }

    fn next_u64() -> u64 {
        SEED.with(|seed| {
            let mut x = seed.get();
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            seed.set(x);
            x.wrapping_mul(0x2545f4914f6cdd1d)
        })
    }

    impl RandomValue for u8 {
        fn random_value() -> Self {
            next_u64() as u8
        }
    }

    impl RandomValue for u16 {
        fn random_value() -> Self {
            next_u64() as u16
        }
    }

    impl RandomValue for u32 {
        fn random_value() -> Self {
            next_u64() as u32
        }
    }

    impl RandomValue for u64 {
        fn random_value() -> Self {
            next_u64()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_preset() {
        assert_eq!(QualityPreset::Low.resolution(), (640, 360));
        assert_eq!(QualityPreset::Medium.resolution(), (1280, 720));
        assert_eq!(QualityPreset::High.bitrate(), 5_000_000);
    }

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.protocol, StreamProtocol::WebRTC);
        assert_eq!(config.quality, QualityPreset::Medium);
        assert!(config.auto_reconnect);
    }

    #[test]
    fn test_rtp_packet_creation() {
        let packet = RtpPacket::new(100, 90000, 0x12345678, vec![1, 2, 3, 4]);
        
        assert_eq!(packet.version, 2);
        assert_eq!(packet.sequence, 100);
        assert_eq!(packet.timestamp, 90000);
        assert_eq!(packet.ssrc, 0x12345678);
        assert_eq!(packet.payload.len(), 4);
    }

    #[test]
    fn test_rtp_packet_serialization() {
        let packet = RtpPacket::new(100, 90000, 0x12345678, vec![1, 2, 3, 4]);
        let bytes = packet.to_bytes();

        assert!(bytes.len() >= 12);
        
        // Parse back
        let parsed = RtpPacket::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.sequence, packet.sequence);
        assert_eq!(parsed.timestamp, packet.timestamp);
        assert_eq!(parsed.ssrc, packet.ssrc);
        assert_eq!(parsed.payload, packet.payload);
    }

    #[test]
    fn test_streamer_creation() {
        let config = StreamConfig::default();
        let streamer = VideoStreamer::new(config);
        
        assert_eq!(streamer.state(), StreamState::Disconnected);
    }

    #[test]
    fn test_streamer_connect_rtsp() {
        let config = StreamConfig {
            protocol: StreamProtocol::RTSP,
            url: "rtsp://localhost:8554/stream".into(),
            ..Default::default()
        };
        let mut streamer = VideoStreamer::new(config);
        
        let result = streamer.connect();
        assert!(result.is_ok());
        assert_eq!(streamer.state(), StreamState::Streaming);
    }

    #[test]
    fn test_streamer_connect_webrtc() {
        let config = StreamConfig {
            protocol: StreamProtocol::WebRTC,
            ..Default::default()
        };
        let mut streamer = VideoStreamer::new(config);
        
        let result = streamer.connect();
        assert!(result.is_ok());
        assert_eq!(streamer.state(), StreamState::Streaming);
    }

    #[test]
    fn test_streamer_pause_resume() {
        let mut streamer = VideoStreamer::new(StreamConfig::default());
        streamer.connect().unwrap();

        streamer.pause().unwrap();
        assert_eq!(streamer.state(), StreamState::Paused);

        streamer.resume().unwrap();
        assert_eq!(streamer.state(), StreamState::Streaming);
    }

    #[test]
    fn test_streamer_disconnect() {
        let mut streamer = VideoStreamer::new(StreamConfig::default());
        streamer.connect().unwrap();
        streamer.disconnect();

        assert_eq!(streamer.state(), StreamState::Disconnected);
    }

    #[test]
    fn test_send_packet() {
        let mut streamer = VideoStreamer::new(StreamConfig::default());
        streamer.connect().unwrap();

        let packet = EncodedPacket {
            data: vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x99],
            pts: 0,
            dts: 0,
            duration: 3000,
            frame_type: FrameType::Intra,
            is_keyframe: true,
        };

        let result = streamer.send(&packet);
        assert!(result.is_ok());
        assert!(streamer.stats().packets_sent > 0);
    }

    #[test]
    fn test_send_without_connect() {
        let mut streamer = VideoStreamer::new(StreamConfig::default());

        let packet = EncodedPacket {
            data: vec![0x01],
            pts: 0,
            dts: 0,
            duration: 3000,
            frame_type: FrameType::Intra,
            is_keyframe: true,
        };

        let result = streamer.send(&packet);
        assert!(result.is_err());
    }

    #[test]
    fn test_webrtc_peer() {
        let mut peer = WebRtcPeer::new("peer1".into());
        assert_eq!(peer.state, PeerState::New);

        let offer = peer.create_offer().unwrap();
        assert!(offer.contains("v=0"));
        assert!(offer.contains("m=video"));
    }

    #[test]
    fn test_webrtc_answer() {
        let mut peer = WebRtcPeer::new("peer1".into());
        
        // Need remote SDP first
        let result = peer.create_answer();
        assert!(result.is_err());

        // Set remote SDP
        peer.set_remote_description("v=0\r\no=...".into()).unwrap();
        
        let answer = peer.create_answer().unwrap();
        assert!(answer.contains("v=0"));
    }

    #[test]
    fn test_ice_candidate() {
        let mut peer = WebRtcPeer::new("peer1".into());
        
        let candidate = IceCandidate {
            candidate_type: IceCandidateType::Host,
            protocol: "udp".into(),
            address: "192.168.1.100".parse().unwrap(),
            port: 5000,
            priority: 100,
        };

        peer.add_ice_candidate(candidate);
        assert_eq!(peer.ice_candidates.len(), 1);
    }

    #[test]
    fn test_multiplexer() {
        let mux = StreamMultiplexer::new();
        assert_eq!(mux.current_quality(), QualityPreset::Medium);
    }

    #[test]
    fn test_adaptive_quality() {
        let mut mux = StreamMultiplexer::new();

        // Multiple high bandwidth updates to move EMA
        for _ in 0..10 {
            mux.update_bandwidth(10_000_000);
        }
        assert_eq!(mux.current_quality(), QualityPreset::High);

        // Many low bandwidth updates to decay EMA below 1M threshold
        for _ in 0..50 {
            mux.update_bandwidth(500_000);
        }
        assert_eq!(mux.current_quality(), QualityPreset::Low);
    }

    #[test]
    fn test_get_variant() {
        let mux = StreamMultiplexer::new();
        
        let variant = mux.get_variant(QualityPreset::Medium).unwrap();
        assert_eq!(variant.resolution, (1280, 720));
    }
}
