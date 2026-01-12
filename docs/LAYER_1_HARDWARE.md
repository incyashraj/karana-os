# Layer 1: Hardware Abstraction Layer (HAL)

## Overview

The Hardware Abstraction Layer is the foundation of Kāraṇa OS, providing unified interfaces to all physical components of smart glasses. It isolates hardware-specific implementations from higher layers, enabling portability across different hardware platforms while optimizing for real-time performance.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    LAYER 1: HARDWARE ABSTRACTION                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │                 HardwareManager (Core Orchestrator)             │    │
│  │  - 60 FPS tick() loop (16.67ms per frame)                       │    │
│  │  - Device initialization & power management                     │    │
│  │  - Event publication to upper layers                            │    │
│  └────┬───────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       ├──────────────┬──────────────┬──────────────┬──────────────┐   │
│       │              │              │              │              │   │
│  ┌────▼────┐   ┌────▼────┐   ┌────▼────┐   ┌────▼────┐   ┌────▼────┐
│  │ Camera  │   │ Sensors │   │  Audio  │   │ Display │   │  Power  │
│  │ Manager │   │ Fusion  │   │ Manager │   │ Manager │   │ Manager │
│  └─────────┘   └─────────┘   └─────────┘   └─────────┘   └─────────┘
│       │              │              │              │              │   │
│  ┌────▼────────────────────────────────────────────────────────▼────┐
│  │                    Hardware Event Bus                            │
│  │  Events: CameraFrame | PoseUpdate | AudioReady | DisplaySync    │
│  └──────────────────────────────────────────────────────────────────┘
└───────────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Camera Manager

**Purpose**: Manages RGB camera capture for computer vision and AR tracking.

**Key Responsibilities**:
- Frame capture at 30-60 FPS
- Auto-exposure and white balance
- Image preprocessing (denoising, sharpening)
- Frame buffer management

**Implementation** (`simulator-ui/components/CameraFeed.tsx`):
```typescript
interface CameraManager {
  captureFrame(): ImageData;
  setExposure(value: number): void;
  setResolution(width: number, height: number): void;
  getFrameRate(): number;
}
```

**Technical Stack**:
- **Simulation**: HTML5 MediaDevices API (`getUserMedia`)
- **Production**: V4L2 (Linux) / AVFoundation (iOS) / Camera2 (Android)
- **Resolution**: 1280x720 @ 30fps (configurable)
- **Format**: RGBA8888 (32-bit per pixel)

**Integration Points**:
- **→ Layer 5 (Intelligence)**: Raw frames for scene understanding
- **→ AR Tracking System**: Frames for optical flow calculation
- **→ Layer 6 (AI)**: Frames for visual question answering

**Performance Metrics**:
- Latency: <5ms (capture to buffer)
- Memory: 2.76MB per frame (1280x720x4)
- CPU: ~8% (H.264 decoding on hardware)

---

### 2. Sensor Fusion Service

**Purpose**: Combines IMU, GPS, and magnetometer data to calculate precise 6DOF pose.

**Key Responsibilities**:
- Quaternion-based rotation tracking
- Position estimation via dead reckoning
- Sensor calibration and drift correction
- Coordinate system transformations

**Implementation** (`simulator-ui/services/SensorFusionService.ts`):
```typescript
class SensorFusionService {
  private rotation: quat = quat.create();
  private rotationOffset: quat = quat.create();
  
  // Update from DeviceOrientation API
  update(alpha: number, beta: number, gamma: number): quat {
    // Convert Euler angles to quaternion
    const q = this.eulerToQuaternion(alpha, beta, gamma);
    
    // Apply calibration offset
    quat.multiply(this.rotation, q, this.rotationOffset);
    
    return this.rotation;
  }
  
  getQuaternion(): quat { return this.rotation; }
  getEuler(): { pitch: number; yaw: number; roll: number; }
}
```

**Sensor Data Flow**:
```
IMU (Gyro + Accel) ──┐
                     ├──► Complementary Filter ──► Quaternion
Magnetometer ────────┤
                     └──► Drift Correction
GPS ────────────────────► Position (WGS84)
```

**Algorithms**:
- **Complementary Filter**: Fuses gyro (high-freq) + accel (low-freq)
- **Madgwick AHRS**: Gradient descent orientation estimation
- **Dead Reckoning**: Integrates acceleration for position

**Integration Points**:
- **→ AR Tracking**: Rotation quaternion for world anchors
- **→ Layer 7 (Interface)**: Head orientation for gaze tracking
- **→ Layer 5 (Intelligence)**: Spatial context for scene understanding

**Performance Metrics**:
- Update Rate: 60-100 Hz
- Orientation Accuracy: ±2° (after calibration)
- Position Drift: <1m per minute (GPS fusion)

---

### 3. Audio Manager

**Purpose**: Captures microphone audio and plays back voice/system sounds.

**Key Responsibilities**:
- Continuous audio capture (16kHz mono)
- Voice Activity Detection (VAD)
- Noise suppression (AEC, ANS)
- Audio playback with spatial positioning

**Implementation**:
```typescript
interface AudioManager {
  startCapture(): AudioStream;
  detectVoiceActivity(): boolean;
  playSpatialSound(buffer: AudioBuffer, position: vec3): void;
  enableNoiseReduction(): void;
}
```

**Audio Pipeline**:
```
Microphone ──► AEC ──► ANS ──► VAD ──► Buffer ──► Layer 6 (AI)
                │                        │
Speaker ◄───────┘ (echo cancellation)    └──► Layer 7 (Voice UI)
```

**Technical Stack**:
- **Capture**: Web Audio API / ALSA / CoreAudio
- **Processing**: WebRTC audio processing modules
- **Format**: 16-bit PCM, 16kHz, mono
- **Buffer**: 10ms chunks (160 samples)

**Integration Points**:
- **→ Layer 6 (AI)**: Audio for speech recognition
- **→ Layer 7 (Interface)**: Voice commands
- **→ Universal Oracle**: Real-time conversation

**Performance Metrics**:
- Latency: <20ms (mic to buffer)
- SNR Improvement: +15dB (with ANS)
- VAD Accuracy: 96% (in quiet environments)

---

### 4. Display Manager

**Purpose**: Renders AR overlays and UI elements to waveguide display.

**Key Responsibilities**:
- Frame composition (layers, transparency, depth)
- Color correction for waveguide optics
- Refresh rate optimization (60/90/120 Hz)
- Brightness adjustment based on ambient light

**Implementation**:
```typescript
interface DisplayManager {
  renderFrame(layers: Layer[]): void;
  setRefreshRate(hz: 60 | 90 | 120): void;
  setBrightness(percent: number): void;
  enableLowLatencyMode(): void;
}
```

**Rendering Pipeline**:
```
UI Layers ──┐
AR Content ─┼──► Compositor ──► Color Correction ──► Waveguide
Text/HUD ───┘        │                                   │
                     ├──► Depth Sorting                  │
                     └──► Alpha Blending                 │
                                                         ▼
                                                    User's Eye
```

**Technical Details**:
- **Resolution**: 1920x1080 per eye (future: 2560x1440)
- **FOV**: 50° diagonal (future: 70°)
- **Latency**: 11ms (photon-to-photon)
- **Technology**: MicroLED + holographic waveguide

**Integration Points**:
- **← Layer 7 (Interface)**: HUD elements, text, icons
- **← AR Tracking**: 3D anchors, spatial windows
- **← Layer 8 (Apps)**: Application UI

**Performance Metrics**:
- Frame Rate: 90 FPS target (11ms per frame)
- Render Time: <5ms (GPU-accelerated)
- Power: 150mW @ 50% brightness

---

### 5. Power Manager

**Purpose**: Monitors battery, thermal state, and manages power budgets across components.

**Key Responsibilities**:
- Real-time battery monitoring
- Thermal throttling
- Component power gating
- Remaining runtime prediction

**Implementation**:
```typescript
interface PowerManager {
  getBatteryLevel(): number; // 0.0 - 1.0
  getTemperature(): number; // Celsius
  getRemainingTime(): number; // minutes
  requestPowerMode(mode: 'performance' | 'balanced' | 'saver'): void;
  getThermalState(): 'nominal' | 'warm' | 'hot' | 'critical';
}
```

**Power Budget Allocation**:
```
Total: 5W (3.7V @ 1350mAh battery, 2 hours runtime)
├── Display: 1.8W (36%)
├── SoC/GPU: 1.5W (30%)
├── Camera: 0.8W (16%)
├── Sensors: 0.4W (8%)
├── Audio: 0.3W (6%)
└── Wireless: 0.2W (4%)
```

**Thermal Management**:
- **Nominal (<35°C)**: Full performance
- **Warm (35-45°C)**: Reduce display brightness 10%
- **Hot (45-55°C)**: Throttle GPU 50%, camera 30fps
- **Critical (>55°C)**: Emergency shutdown non-essential

**Integration Points**:
- **→ Layer 9 (System Services)**: Thermal governor decisions
- **→ Resource Manager**: Power mode selection
- **→ Layer 6 (AI)**: Model quantization triggers

**Performance Metrics**:
- Battery Life: 2-3 hours (normal use)
- Charge Time: 45 minutes (USB-C PD, 15W)
- Standby: 72 hours

---

## Cross-Layer Communication

### Event Bus Integration

Hardware layer publishes events consumed by upper layers:

```typescript
enum HardwareEvent {
  CameraFrameReady,    // 30-60 Hz
  PoseUpdated,         // 60-100 Hz
  AudioCaptured,       // 100 Hz (10ms chunks)
  BatteryChanged,      // 1 Hz
  ThermalWarning,      // On threshold cross
  DisplayVSync,        // 90 Hz
}
```

**Event Flow Example**:
```
Camera captures frame ──► Event::CameraFrameReady
                          │
                          ├──► Layer 5: Scene understanding
                          ├──► AR Tracker: Optical flow
                          └──► Layer 6 AI: Visual QA
```

---

## Hardware Abstraction Traits

To support multiple hardware platforms, each component implements platform-agnostic traits:

```rust
// Rust trait definition (karana-core)
pub trait CameraDevice {
    fn init(&mut self, config: CameraConfig) -> Result<()>;
    fn capture_frame(&mut self) -> Result<Frame>;
    fn set_exposure(&mut self, ev: f32) -> Result<()>;
    fn get_capabilities(&self) -> CameraCapabilities;
}

// Implementations
impl CameraDevice for V4L2Camera { /* Linux */ }
impl CameraDevice for AVFoundationCamera { /* iOS */ }
impl CameraDevice for Camera2Device { /* Android */ }
impl CameraDevice for SimulatedCamera { /* Web simulation */ }
```

---

## Platform-Specific Implementations

### Simulation (Web Browser)
- **Camera**: `navigator.mediaDevices.getUserMedia()`
- **Sensors**: `DeviceOrientationEvent` (gyro simulation)
- **Audio**: Web Audio API
- **Display**: HTML5 Canvas + WebGL
- **Location**: Browser Geolocation API

### Production Hardware (Qualcomm XR2+ Gen 2)
- **Camera**: V4L2 kernel driver
- **Sensors**: Industrial I/O (IIO) subsystem
- **Audio**: ALSA + Qualcomm Hexagon DSP
- **Display**: DRM/KMS + Vulkan
- **Power**: Linux power supply framework

---

## Calibration & Initialization

### Startup Sequence (First Boot)

```
1. Hardware Detection
   ├── Enumerate I2C devices (sensors)
   ├── Initialize V4L2 camera
   ├── Test display connection
   └── Verify battery present

2. Sensor Calibration
   ├── IMU: Collect 5s of stationary data
   ├── Magnetometer: Hard/soft iron compensation
   ├── Camera: Intrinsic parameters (focal length, distortion)
   └── Store calibration to NVRAM

3. Display Calibration
   ├── Color temperature adjustment
   ├── Brightness curve
   └── IPD (interpupillary distance) measurement

4. Functional Test
   ├── Capture test frame
   ├── Record audio sample
   ├── Display test pattern
   └── Publish Event::SystemReady
```

### Runtime Calibration (User-Triggered)

User-initiated recalibration via voice: "Calibrate sensors"

```typescript
// simulator-ui/components/CalibrationOverlay.tsx
<CalibrationOverlay 
  onComplete={() => {
    sensorFusion.setOrigin(currentRotation);
    spatialAnchors.resetWorldOrigin();
  }}
/>
```

---

## Performance Optimization

### 1. Zero-Copy Frame Transfer
- Camera frames use DMA to GPU memory
- No CPU copies for rendering pipeline
- Saves 5ms per frame

### 2. Sensor Fusion on DSP
- Offload quaternion math to Hexagon DSP
- Reduces CPU load from 15% → 2%

### 3. Power Gating
- Camera auto-suspend when no app needs it
- Display partial refresh for static content
- GPS duty cycling (1Hz → 0.1Hz when stationary)

### 4. Thermal Throttling
- Predictive algorithm (30s horizon)
- Gradual reduction to avoid jarring UX
- Priority: Display > AI > Camera > Sensors

---

## Future Development Roadmap

### Phase 1: Multi-Camera Support (Q1 2026)
- Add depth camera for SLAM
- Wide-angle fisheye for peripheral vision
- Dual RGB for stereoscopic AR

### Phase 2: Advanced Sensor Fusion (Q2 2026)
- Visual-Inertial Odometry (VIO)
- Integrate UWB for precise indoor positioning
- Barometer for altitude estimation

### Phase 3: Neural Display Rendering (Q3 2026)
- Foveated rendering (eye tracking)
- AI upscaling (render 720p, display 1080p)
- Adaptive refresh rate (48-120Hz)

### Phase 4: Wireless Power (Q4 2026)
- Qi wireless charging in case
- RF energy harvesting (experimental)
- Solar cells in frame (sunny outdoor)

---

## Integration with Other Layers

### → Layer 2 (P2P Network)
- Hardware serial numbers for device identity
- WiFi/Bluetooth MAC addresses for peer discovery

### → Layer 5 (Intelligence)
- Camera frames for scene understanding
- IMU data for activity recognition
- Microphone for ambient sound classification

### → Layer 6 (AI Engine)
- Audio for speech-to-text
- Camera for visual question answering
- Sensors for context-aware responses

### → Layer 7 (Interface)
- Display for HUD rendering
- Speakers for TTS output
- Haptic feedback for notifications

### → Layer 9 (System Services)
- Power state for OTA update decisions
- Thermal state for throttling strategies
- Battery for low-power warnings

---

## Debugging & Diagnostics

### Hardware Metrics Dashboard

Accessible via voice: "Show hardware stats"

```
┌─ Hardware Status ─────────────────┐
│ Camera:   1280x720 @ 30fps ✓      │
│ IMU:      60Hz, drift: 0.8°/min ✓ │
│ Audio:    16kHz, SNR: 25dB ✓      │
│ Display:  90fps, 45% brightness ✓ │
│ Battery:  67%, 1h 23m remaining ✓ │
│ Temp:     38°C (nominal) ✓        │
└────────────────────────────────────┘
```

### Event Tracing

```bash
# Enable hardware event logging
export KARANA_HAL_TRACE=1

# Logs:
[16.67ms] Camera: Frame captured (2.3ms)
[16.67ms] Sensor: Pose updated (0.8ms)
[26.67ms] Audio: Buffer ready (0.5ms)
[33.34ms] Display: VSync (0.1ms)
```

---

## Code References

### Simulation UI
- `simulator-ui/components/CameraFeed.tsx`: Camera capture
- `simulator-ui/services/SensorFusionService.ts`: IMU fusion
- `simulator-ui/services/VisionService.ts`: Optical flow
- `simulator-ui/services/SpatialAnchorService.ts`: Coordinate transforms

### Production Core (Rust)
- `karana-core/src/hal/camera.rs`: Camera trait & V4L2 impl
- `karana-core/src/hal/sensors.rs`: IMU/GPS/Mag interfaces
- `karana-core/src/hal/audio.rs`: Audio capture/playback
- `karana-core/src/hal/display.rs`: DRM/KMS rendering
- `karana-core/src/hal/power.rs`: Battery & thermal management

---

## Testing

### Unit Tests
```rust
#[test]
fn test_sensor_fusion_quaternion() {
    let mut fusion = SensorFusion::new();
    fusion.update(0.0, 0.0, 0.0); // Identity
    assert_quaternion_near(fusion.get_rotation(), quat::identity());
}
```

### Integration Tests
```bash
# Hardware-in-loop test
cargo test --features hardware_test hal_camera_capture
```

### Performance Tests
```bash
# Benchmark camera latency
cargo bench --bench hal_latency
# Target: <5ms capture latency
```

---

## Summary

Layer 1 provides:
- **Unified Hardware Interface**: Abstract away platform differences
- **Real-Time Performance**: 60-90 FPS with <11ms latency
- **Power Efficiency**: 2+ hour battery, intelligent thermal management
- **Robust Calibration**: Auto-calibrate sensors, handle drift
- **Event-Driven**: Asynchronous event bus for upper layers

This layer is critical for AR glasses where sensor precision, display latency, and power consumption directly impact user experience.
