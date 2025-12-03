# Plan: Multimodal Input System

## Overview
The Oracle needs to understand user intent from multiple input modalities:
- **Voice** - Primary interaction mode
- **Gaze** - Eye tracking for context and selection
- **Gesture** - IMU-based head movements for confirmation

---

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MultimodalSense                          │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │   Voice   │  │   Gaze    │  │  Gesture  │               │
│  │ (Whisper) │  │ (OpenCV)  │  │   (IMU)   │               │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘               │
│        │              │              │                      │
│        └──────────────┼──────────────┘                      │
│                       ↓                                     │
│              ┌────────────────┐                             │
│              │  Tensor Fusion │                             │
│              │  (Multi-Head)  │                             │
│              └────────┬───────┘                             │
│                       ↓                                     │
│              MultimodalInput                                │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### File: `karana-core/src/oracle/sense.rs`

### Step 1: Core Structures

```rust
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Multimodal input capture and fusion
pub struct MultimodalSense {
    /// Voice input handler
    voice: VoiceSense,
    
    /// Gaze tracking handler
    gaze: GazeSense,
    
    /// IMU gesture handler
    gesture: GestureSense,
    
    /// Fusion model for combining modalities
    fusion: FusionModel,
    
    /// Configuration
    config: SenseConfig,
}

/// Raw voice input
pub struct VoiceInput {
    /// Transcribed text (from Whisper)
    pub transcription: String,
    
    /// MFCC features for semantic embedding
    pub mfcc_features: Vec<f32>,
    
    /// Confidence score
    pub confidence: f32,
    
    /// Duration in seconds
    pub duration_secs: f32,
}

/// Raw gaze input
pub struct GazeInput {
    /// Eye position vector [x, y] in normalized screen coords
    pub position: [f32; 2],
    
    /// Gaze direction vector [x, y, z]
    pub direction: [f32; 3],
    
    /// Pupil dilation (attention indicator)
    pub pupil_dilation: f32,
    
    /// What the user is looking at (object ID or None)
    pub target_object: Option<String>,
    
    /// Dwell time on current target (ms)
    pub dwell_time_ms: u64,
}

/// Raw gesture input
pub struct GestureInput {
    /// Detected gesture type
    pub gesture: GestureType,
    
    /// Confidence score
    pub confidence: f32,
    
    /// Raw IMU data [accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z]
    pub raw_imu: [f32; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    None,
    Nod,          // Yes / Confirm
    Shake,        // No / Cancel  
    TiltLeft,     // Previous / Back
    TiltRight,    // Next / Forward
    LookUp,       // Open menu
    LookDown,     // Close / Dismiss
    Blink,        // Select
    DoubleBlink,  // Quick action
}

/// Fused multimodal input for Oracle
#[derive(Debug, Clone)]
pub struct MultimodalInput {
    /// Voice transcription
    pub voice_text: Option<String>,
    
    /// Voice embedding (MFCC features)
    pub voice_embedding: Option<Vec<f32>>,
    
    /// Direct text input (simulator mode)
    pub text: Option<String>,
    
    /// Gaze target
    pub gaze_target: Option<String>,
    
    /// Gaze direction
    pub gaze_direction: Option<[f32; 3]>,
    
    /// Detected gesture
    pub gesture: GestureType,
    
    /// Fused semantic embedding
    pub fused_embedding: Option<Vec<f32>>,
    
    /// Overall confidence
    pub confidence: f32,
    
    /// Timestamp
    pub timestamp: u64,
}

pub struct SenseConfig {
    /// Wake word for voice activation
    pub wake_word: String,
    
    /// Gaze dwell threshold for selection (ms)
    pub gaze_dwell_threshold_ms: u64,
    
    /// Gesture confidence threshold
    pub gesture_threshold: f32,
    
    /// Whether to use simulated sensors
    pub simulate: bool,
}

impl Default for SenseConfig {
    fn default() -> Self {
        Self {
            wake_word: "hey karana".into(),
            gaze_dwell_threshold_ms: 500,
            gesture_threshold: 0.7,
            simulate: true,
        }
    }
}
```

### Step 2: Voice Sensing

```rust
/// Voice input processing
pub struct VoiceSense {
    /// Whisper model for transcription
    whisper: Option<Arc<Mutex<crate::ai::KaranaAI>>>,
    
    /// Audio buffer
    buffer: Vec<f32>,
    
    /// Wake word detector state
    wake_word_detected: bool,
    
    /// Config
    config: SenseConfig,
}

impl VoiceSense {
    pub fn new(ai: Arc<Mutex<crate::ai::KaranaAI>>, config: SenseConfig) -> Self {
        Self {
            whisper: Some(ai),
            buffer: Vec::new(),
            wake_word_detected: false,
            config,
        }
    }
    
    /// Process audio samples and return voice input
    pub fn process(&mut self, samples: &[f32]) -> Result<Option<VoiceInput>> {
        self.buffer.extend_from_slice(samples);
        
        // Check for wake word if not already detected
        if !self.wake_word_detected {
            if self.detect_wake_word() {
                self.wake_word_detected = true;
                self.buffer.clear();
                log::info!("[VOICE] Wake word detected!");
                return Ok(None);  // Wait for actual command
            }
            return Ok(None);
        }
        
        // Check for silence (end of utterance)
        if self.detect_silence() {
            let audio = std::mem::take(&mut self.buffer);
            self.wake_word_detected = false;
            
            // Transcribe
            let transcription = self.transcribe(&audio)?;
            
            // Extract MFCC features
            let mfcc = self.extract_mfcc(&audio);
            
            return Ok(Some(VoiceInput {
                transcription,
                mfcc_features: mfcc,
                confidence: 0.9,  // TODO: Get from Whisper
                duration_secs: audio.len() as f32 / 16000.0,
            }));
        }
        
        Ok(None)
    }
    
    fn detect_wake_word(&self) -> bool {
        // Simplified: Check if buffer contains enough audio to analyze
        // Real impl would use phonetic matching
        if self.buffer.len() < 16000 {  // 1 second @ 16kHz
            return false;
        }
        
        // Quick transcribe and check
        if let Some(ai) = &self.whisper {
            if let Ok(text) = ai.lock().unwrap().transcribe(self.buffer.clone()) {
                let lower = text.to_lowercase();
                return lower.contains("hey karana") || 
                       lower.contains("okay karana") ||
                       lower.contains("hi karana");
            }
        }
        false
    }
    
    fn detect_silence(&self) -> bool {
        // Check last 0.5 seconds for silence
        let samples_to_check = 8000;  // 0.5s @ 16kHz
        if self.buffer.len() < samples_to_check {
            return false;
        }
        
        let tail = &self.buffer[self.buffer.len() - samples_to_check..];
        let rms: f32 = (tail.iter().map(|x| x * x).sum::<f32>() / samples_to_check as f32).sqrt();
        
        rms < 0.01  // Silence threshold
    }
    
    fn transcribe(&self, audio: &[f32]) -> Result<String> {
        if let Some(ai) = &self.whisper {
            ai.lock().unwrap().transcribe(audio.to_vec())
        } else {
            Err(anyhow::anyhow!("Whisper not initialized"))
        }
    }
    
    fn extract_mfcc(&self, audio: &[f32]) -> Vec<f32> {
        // Simplified MFCC extraction
        // Real impl would use a proper DSP library
        
        let frame_size = 512;
        let num_frames = audio.len() / frame_size;
        let num_coeffs = 13;
        
        let mut mfcc = vec![0.0f32; num_coeffs];
        
        for i in 0..num_frames {
            let frame = &audio[i * frame_size..(i + 1) * frame_size];
            
            // Compute energy in frame
            let energy: f32 = frame.iter().map(|x| x * x).sum();
            mfcc[0] += energy.log10().max(-10.0);
            
            // Simple spectral features (placeholder for real MFCC)
            for j in 1..num_coeffs {
                let freq_bin: f32 = frame.iter()
                    .enumerate()
                    .map(|(k, &x)| x * (2.0 * std::f32::consts::PI * j as f32 * k as f32 / frame_size as f32).cos())
                    .sum();
                mfcc[j] += freq_bin.abs();
            }
        }
        
        // Normalize
        for c in &mut mfcc {
            *c /= num_frames as f32;
        }
        
        mfcc
    }
}
```

### Step 3: Gaze Sensing

```rust
/// Gaze tracking using OpenCV
pub struct GazeSense {
    /// OpenCV VideoCapture (if available)
    #[cfg(feature = "opencv")]
    capture: Option<opencv::videoio::VideoCapture>,
    
    /// Eye cascade classifier
    #[cfg(feature = "opencv")]
    eye_cascade: Option<opencv::objdetect::CascadeClassifier>,
    
    /// Last known gaze position
    last_position: [f32; 2],
    
    /// Gaze history for smoothing
    history: Vec<[f32; 2]>,
    
    /// Current target being gazed at
    current_target: Option<(String, u64)>,  // (target_id, start_time)
    
    /// Simulated mode
    simulate: bool,
}

impl GazeSense {
    pub fn new(simulate: bool) -> Self {
        Self {
            #[cfg(feature = "opencv")]
            capture: if !simulate {
                opencv::videoio::VideoCapture::new(0, opencv::videoio::CAP_ANY).ok()
            } else {
                None
            },
            #[cfg(feature = "opencv")]
            eye_cascade: if !simulate {
                let cascade_path = "/usr/share/opencv4/haarcascades/haarcascade_eye.xml";
                opencv::objdetect::CascadeClassifier::new(cascade_path).ok()
            } else {
                None
            },
            last_position: [0.5, 0.5],
            history: Vec::new(),
            current_target: None,
            simulate,
        }
    }
    
    /// Capture and process gaze
    pub fn capture(&mut self) -> Result<GazeInput> {
        if self.simulate {
            return Ok(self.simulate_gaze());
        }
        
        #[cfg(feature = "opencv")]
        {
            self.capture_opencv()
        }
        
        #[cfg(not(feature = "opencv"))]
        {
            Ok(self.simulate_gaze())
        }
    }
    
    #[cfg(feature = "opencv")]
    fn capture_opencv(&mut self) -> Result<GazeInput> {
        use opencv::prelude::*;
        
        let capture = self.capture.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Camera not initialized"))?;
        
        let mut frame = opencv::core::Mat::default();
        capture.read(&mut frame)?;
        
        if frame.empty() {
            return Err(anyhow::anyhow!("Empty frame"));
        }
        
        // Convert to grayscale
        let mut gray = opencv::core::Mat::default();
        opencv::imgproc::cvt_color(&frame, &mut gray, opencv::imgproc::COLOR_BGR2GRAY, 0)?;
        
        // Detect eyes
        let eye_cascade = self.eye_cascade.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Eye cascade not loaded"))?;
        
        let mut eyes = opencv::core::Vector::<opencv::core::Rect>::new();
        eye_cascade.detect_multi_scale(
            &gray,
            &mut eyes,
            1.1,
            3,
            0,
            opencv::core::Size::new(30, 30),
            opencv::core::Size::new(0, 0),
        )?;
        
        // Calculate gaze from eye positions
        if eyes.len() >= 2 {
            let eye1 = eyes.get(0)?;
            let eye2 = eyes.get(1)?;
            
            // Average eye center
            let cx = (eye1.x + eye1.width / 2 + eye2.x + eye2.width / 2) as f32 / 2.0;
            let cy = (eye1.y + eye1.height / 2 + eye2.y + eye2.height / 2) as f32 / 2.0;
            
            // Normalize to [0, 1]
            let width = frame.cols() as f32;
            let height = frame.rows() as f32;
            let position = [cx / width, cy / height];
            
            // Smooth with history
            self.history.push(position);
            if self.history.len() > 5 {
                self.history.remove(0);
            }
            
            let smoothed: [f32; 2] = [
                self.history.iter().map(|p| p[0]).sum::<f32>() / self.history.len() as f32,
                self.history.iter().map(|p| p[1]).sum::<f32>() / self.history.len() as f32,
            ];
            
            self.last_position = smoothed;
            
            // Calculate 3D direction (simplified)
            let direction = [
                (smoothed[0] - 0.5) * 2.0,  // X: -1 to 1
                (smoothed[1] - 0.5) * 2.0,  // Y: -1 to 1
                1.0,                         // Z: forward
            ];
            
            // Estimate pupil dilation (placeholder)
            let pupil_dilation = (eye1.width + eye2.width) as f32 / 100.0;
            
            return Ok(GazeInput {
                position: smoothed,
                direction,
                pupil_dilation,
                target_object: self.detect_target(&smoothed),
                dwell_time_ms: self.calculate_dwell_time(&smoothed),
            });
        }
        
        // No eyes detected, return last known
        Ok(GazeInput {
            position: self.last_position,
            direction: [0.0, 0.0, 1.0],
            pupil_dilation: 0.5,
            target_object: None,
            dwell_time_ms: 0,
        })
    }
    
    fn simulate_gaze(&mut self) -> GazeInput {
        // Simulate gaze wandering slightly around center
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        self.last_position[0] += rng.gen_range(-0.02..0.02);
        self.last_position[1] += rng.gen_range(-0.02..0.02);
        self.last_position[0] = self.last_position[0].clamp(0.0, 1.0);
        self.last_position[1] = self.last_position[1].clamp(0.0, 1.0);
        
        GazeInput {
            position: self.last_position,
            direction: [
                (self.last_position[0] - 0.5) * 2.0,
                (self.last_position[1] - 0.5) * 2.0,
                1.0,
            ],
            pupil_dilation: 0.5,
            target_object: None,
            dwell_time_ms: 0,
        }
    }
    
    fn detect_target(&mut self, position: &[f32; 2]) -> Option<String> {
        // Would query AR scene graph for object at position
        // Placeholder: detect if looking at specific regions
        
        if position[0] < 0.2 && position[1] < 0.2 {
            return Some("menu".into());
        }
        if position[0] > 0.8 && position[1] < 0.2 {
            return Some("notifications".into());
        }
        
        None
    }
    
    fn calculate_dwell_time(&mut self, position: &[f32; 2]) -> u64 {
        let target = self.detect_target(position);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        match (&self.current_target, target) {
            (Some((old_target, start)), Some(ref new_target)) if old_target == new_target => {
                // Same target, return dwell time
                now - *start
            }
            (_, Some(new_target)) => {
                // New target, reset
                self.current_target = Some((new_target, now));
                0
            }
            (_, None) => {
                self.current_target = None;
                0
            }
        }
    }
}
```

### Step 4: Gesture Sensing

```rust
/// IMU-based gesture detection
pub struct GestureSense {
    /// IMU data buffer
    buffer: Vec<[f32; 6]>,
    
    /// Gesture detection model (simple thresholds for now)
    thresholds: GestureThresholds,
    
    /// Simulated mode
    simulate: bool,
}

struct GestureThresholds {
    nod_accel_y: f32,
    shake_accel_x: f32,
    tilt_gyro_z: f32,
}

impl Default for GestureThresholds {
    fn default() -> Self {
        Self {
            nod_accel_y: 2.0,   // m/s²
            shake_accel_x: 2.0,
            tilt_gyro_z: 1.0,   // rad/s
        }
    }
}

impl GestureSense {
    pub fn new(simulate: bool) -> Self {
        Self {
            buffer: Vec::new(),
            thresholds: GestureThresholds::default(),
            simulate,
        }
    }
    
    /// Process IMU sample and detect gesture
    pub fn process(&mut self, imu_sample: [f32; 6]) -> GestureInput {
        self.buffer.push(imu_sample);
        
        // Keep last 50 samples (~0.5s at 100Hz)
        if self.buffer.len() > 50 {
            self.buffer.remove(0);
        }
        
        let gesture = self.detect_gesture();
        
        GestureInput {
            gesture,
            confidence: 0.8,
            raw_imu: imu_sample,
        }
    }
    
    fn detect_gesture(&self) -> GestureType {
        if self.buffer.len() < 20 {
            return GestureType::None;
        }
        
        // Calculate motion statistics
        let accel_x: Vec<f32> = self.buffer.iter().map(|s| s[0]).collect();
        let accel_y: Vec<f32> = self.buffer.iter().map(|s| s[1]).collect();
        let gyro_z: Vec<f32> = self.buffer.iter().map(|s| s[5]).collect();
        
        let accel_x_range = accel_x.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                          - accel_x.iter().cloned().fold(f32::INFINITY, f32::min);
        let accel_y_range = accel_y.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                          - accel_y.iter().cloned().fold(f32::INFINITY, f32::min);
        let gyro_z_max = gyro_z.iter().cloned().fold(f32::NEG_INFINITY, f32::max).abs();
        
        // Detect gestures based on motion patterns
        
        // Nod: Up-down head motion (Y acceleration)
        if accel_y_range > self.thresholds.nod_accel_y {
            // Check for alternating pattern
            let zero_crossings = self.count_zero_crossings(&accel_y);
            if zero_crossings >= 2 && zero_crossings <= 4 {
                return GestureType::Nod;
            }
        }
        
        // Shake: Left-right head motion (X acceleration)
        if accel_x_range > self.thresholds.shake_accel_x {
            let zero_crossings = self.count_zero_crossings(&accel_x);
            if zero_crossings >= 2 && zero_crossings <= 6 {
                return GestureType::Shake;
            }
        }
        
        // Tilt: Rotational motion (Z gyroscope)
        if gyro_z_max > self.thresholds.tilt_gyro_z {
            if gyro_z.last().unwrap_or(&0.0) > &0.0 {
                return GestureType::TiltRight;
            } else {
                return GestureType::TiltLeft;
            }
        }
        
        GestureType::None
    }
    
    fn count_zero_crossings(&self, signal: &[f32]) -> usize {
        let mean: f32 = signal.iter().sum::<f32>() / signal.len() as f32;
        signal.windows(2)
            .filter(|w| (w[0] - mean).signum() != (w[1] - mean).signum())
            .count()
    }
}
```

### Step 5: Fusion Model

```rust
/// Fuses multiple input modalities into unified embedding
pub struct FusionModel {
    /// Embedding dimension
    embed_dim: usize,
}

impl FusionModel {
    pub fn new() -> Self {
        Self { embed_dim: 128 }
    }
    
    /// Fuse all inputs into single MultimodalInput
    pub fn fuse(
        &self,
        voice: Option<VoiceInput>,
        gaze: GazeInput,
        gesture: GestureInput,
    ) -> MultimodalInput {
        let mut fused_embedding = vec![0.0f32; self.embed_dim];
        let mut confidence = 0.0f32;
        let mut weight_sum = 0.0f32;
        
        // Voice contribution (if available)
        if let Some(ref v) = voice {
            // Project MFCC to embedding space (simple linear projection)
            for (i, &mfcc) in v.mfcc_features.iter().enumerate() {
                let idx = i % self.embed_dim;
                fused_embedding[idx] += mfcc * 0.5;
            }
            confidence += v.confidence * 0.5;
            weight_sum += 0.5;
        }
        
        // Gaze contribution
        // Encode position and direction
        fused_embedding[0] += gaze.position[0];
        fused_embedding[1] += gaze.position[1];
        fused_embedding[2] += gaze.direction[0];
        fused_embedding[3] += gaze.direction[1];
        fused_embedding[4] += gaze.pupil_dilation;
        
        // Dwell time encodes attention
        let dwell_factor = (gaze.dwell_time_ms as f32 / 1000.0).min(1.0);
        fused_embedding[5] += dwell_factor;
        
        confidence += dwell_factor * 0.2;
        weight_sum += 0.2;
        
        // Gesture contribution
        if gesture.gesture != GestureType::None {
            let gesture_idx = match gesture.gesture {
                GestureType::Nod => 10,
                GestureType::Shake => 11,
                GestureType::TiltLeft => 12,
                GestureType::TiltRight => 13,
                GestureType::LookUp => 14,
                GestureType::LookDown => 15,
                GestureType::Blink => 16,
                GestureType::DoubleBlink => 17,
                GestureType::None => 0,
            };
            fused_embedding[gesture_idx] = 1.0;
            confidence += gesture.confidence * 0.3;
            weight_sum += 0.3;
        }
        
        // Normalize confidence
        confidence = if weight_sum > 0.0 { confidence / weight_sum } else { 0.0 };
        
        // Normalize embedding
        let norm: f32 = fused_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut fused_embedding {
                *v /= norm;
            }
        }
        
        MultimodalInput {
            voice_text: voice.as_ref().map(|v| v.transcription.clone()),
            voice_embedding: voice.as_ref().map(|v| v.mfcc_features.clone()),
            text: None,
            gaze_target: gaze.target_object,
            gaze_direction: Some(gaze.direction),
            gesture: gesture.gesture,
            fused_embedding: Some(fused_embedding),
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}
```

### Step 6: Main MultimodalSense Implementation

```rust
impl MultimodalSense {
    pub fn new(ai: Arc<Mutex<crate::ai::KaranaAI>>, config: SenseConfig) -> Self {
        Self {
            voice: VoiceSense::new(ai, config.clone()),
            gaze: GazeSense::new(config.simulate),
            gesture: GestureSense::new(config.simulate),
            fusion: FusionModel::new(),
            config,
        }
    }
    
    /// Capture and process all modalities
    pub async fn capture(&mut self) -> Result<Option<MultimodalInput>> {
        // Capture gaze (always available)
        let gaze = self.gaze.capture()?;
        
        // Process audio buffer (returns VoiceInput when utterance complete)
        let voice = self.voice.process(&[])?;  // TODO: Real audio input
        
        // Get current gesture state
        let gesture = self.gesture.process([0.0; 6]);  // TODO: Real IMU input
        
        // Only return fused input if we have meaningful data
        if voice.is_some() || gaze.dwell_time_ms > self.config.gaze_dwell_threshold_ms 
            || gesture.gesture != GestureType::None {
            
            let fused = self.fusion.fuse(voice, gaze, gesture);
            return Ok(Some(fused));
        }
        
        Ok(None)
    }
    
    /// Direct text input (for simulator/testing)
    pub fn text_input(&self, text: &str) -> MultimodalInput {
        MultimodalInput {
            text: Some(text.to_string()),
            voice_text: None,
            voice_embedding: None,
            gaze_target: None,
            gaze_direction: None,
            gesture: GestureType::None,
            fused_embedding: None,
            confidence: 1.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}
```

---

## Integration with Oracle

```rust
// In oracle/veil.rs

impl OracleVeil {
    pub async fn listen_and_mediate(&mut self) -> Result<()> {
        loop {
            // Capture multimodal input
            if let Some(input) = self.sense.capture().await? {
                log::info!("[ORACLE] Input captured: voice={:?}, gaze={:?}, gesture={:?}",
                    input.voice_text.as_ref().map(|s| &s[..20.min(s.len())]),
                    input.gaze_target,
                    input.gesture
                );
                
                // Process through Oracle
                let manifest = self.mediate(input).await?;
                
                // Output manifest
                self.output_manifest(&manifest).await?;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_mfcc_extraction() {
        let voice = VoiceSense::new(/* mock AI */);
        let audio = vec![0.1f32; 16000];  // 1 second
        let mfcc = voice.extract_mfcc(&audio);
        assert_eq!(mfcc.len(), 13);
    }
    
    #[test]
    fn test_gesture_nod_detection() {
        let mut gesture = GestureSense::new(false);
        
        // Simulate nod motion (Y oscillation)
        for i in 0..30 {
            let y = 3.0 * (i as f32 * 0.3).sin();
            gesture.process([0.0, y, 0.0, 0.0, 0.0, 0.0]);
        }
        
        let result = gesture.process([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(result.gesture, GestureType::Nod);
    }
    
    #[test]
    fn test_fusion() {
        let fusion = FusionModel::new();
        
        let voice = VoiceInput {
            transcription: "hello".into(),
            mfcc_features: vec![0.1; 13],
            confidence: 0.9,
            duration_secs: 0.5,
        };
        
        let gaze = GazeInput {
            position: [0.5, 0.5],
            direction: [0.0, 0.0, 1.0],
            pupil_dilation: 0.5,
            target_object: Some("button".into()),
            dwell_time_ms: 600,
        };
        
        let gesture = GestureInput {
            gesture: GestureType::Nod,
            confidence: 0.8,
            raw_imu: [0.0; 6],
        };
        
        let fused = fusion.fuse(Some(voice), gaze, gesture);
        
        assert!(fused.voice_text.is_some());
        assert!(fused.gaze_target.is_some());
        assert_eq!(fused.gesture, GestureType::Nod);
        assert!(fused.confidence > 0.5);
    }
}
```

---

## Timeline

| Task | Duration |
|------|----------|
| Voice sensing | 4 hours |
| Gaze tracking | 4 hours |
| Gesture detection | 3 hours |
| Fusion model | 2 hours |
| Integration | 2 hours |
| Testing | 2 hours |
| **Total** | **17 hours** |

---

## Success Criteria

- [ ] Voice transcription works with wake word
- [ ] Gaze position tracked (simulated or real)
- [ ] Gestures detected (nod, shake, tilt)
- [ ] Fusion produces meaningful embeddings
- [ ] < 100ms sensing latency
- [ ] Works in simulated mode for testing

---

*Multimodal Input Plan v1.0 - December 3, 2025*
