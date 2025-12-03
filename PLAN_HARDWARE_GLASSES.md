# PLAN_HARDWARE_GLASSES.md
# Rokid/XReal Smart Glasses Hardware Integration

## Overview

This plan covers the integration with real smart glasses hardware:
- **Rokid Max / Air** - Android-based AR glasses with SDK
- **XReal Air / Light** - Consumer AR glasses with spatial tracking
- **Target**: Run Kāraṇa OS natively or as companion app

---

## Hardware Specifications

### Rokid Max Pro (Primary Target)
| Component | Specification | Kāraṇa Usage |
|-----------|---------------|--------------|
| Display | 1920x1080 per eye, 120Hz | AR whisper overlay |
| CPU | Qualcomm QCS6490 | Edge inference |
| NPU | 12 TOPS | Phi-3 q4 inference |
| RAM | 6GB | Model + runtime |
| Cameras | 2x RGB + depth | Gaze tracking |
| Audio | Dual speakers + 4 mics | Voice input |
| IMU | 6-axis | Gesture detection |
| Haptic | Linear motor | Pulse feedback |
| OS | Android 12 | Native app |

### XReal Air 2 Ultra (Secondary Target)
| Component | Specification | Kāraṇa Usage |
|-----------|---------------|--------------|
| Display | 1080p OLED, 120Hz | AR overlay |
| Tracking | 6DoF SLAM | Spatial anchor |
| Cameras | 2x RGB | Limited gaze |
| Audio | Open-ear speakers | Voice input |
| Connection | USB-C DP Alt | Companion mode |

---

## Architecture

### Native Mode (Rokid with SOC)
```
┌──────────────────────────────────────────────────────────┐
│                    ROKID GLASSES                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              KARANA OS (Native)                    │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │  │
│  │  │ OracleVeil│→│ NPU Phi-3│→│ MinimalManifest  │ │  │
│  │  │ (Voice+  │  │ (12 TOPS)│  │ (Display+Haptic)│ │  │
│  │  │  Gaze)   │  │          │  │                 │ │  │
│  │  └──────────┘  └──────────┘  └──────────────────┘ │  │
│  │       ↑                              ↓            │  │
│  │  ┌────┴────┐                  ┌──────┴──────┐     │  │
│  │  │ Sensors │                  │   Display   │     │  │
│  │  │ (Cam+   │                  │  (Overlay)  │     │  │
│  │  │  Mic)   │                  │             │     │  │
│  │  └─────────┘                  └─────────────┘     │  │
│  └────────────────────────────────────────────────────┘  │
│                         │                                 │
│                    ┌────┴────┐                           │
│                    │ P2P Sync│ ← WiFi/BT to backend      │
│                    └─────────┘                           │
└──────────────────────────────────────────────────────────┘
```

### Companion Mode (XReal tethered to phone/PC)
```
┌────────────────────┐         ┌──────────────────────────┐
│   XREAL GLASSES    │◄───────►│     COMPANION DEVICE     │
│  (Display Only)    │  USB-C  │  (Phone/PC running       │
│                    │         │   Kāraṇa backend)        │
│  ┌──────────────┐  │         │  ┌────────────────────┐  │
│  │ AR Overlay   │  │         │  │    KaranaMonad     │  │
│  │ (from host)  │  │         │  │    + OracleVeil    │  │
│  └──────────────┘  │         │  └────────────────────┘  │
│                    │         │           │              │
│  ┌──────────────┐  │ Audio   │  ┌────────┴─────────┐   │
│  │ Mic Passthru │──┼────────►│  │ Voice Processing │   │
│  └──────────────┘  │         │  └──────────────────┘   │
└────────────────────┘         └──────────────────────────┘
```

---

## Phase 1: Rokid SDK Integration

### 1.1 Project Setup

**File: `karana-android/app/build.gradle.kts`**
```kotlin
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "ai.karana.glasses"
    compileSdk = 34
    
    defaultConfig {
        applicationId = "ai.karana.glasses"
        minSdk = 29  // Android 10 for Rokid
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
        
        ndk {
            abiFilters += listOf("arm64-v8a")  // ARM64 only
        }
    }
    
    buildFeatures {
        compose = true
    }
    
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }
}

dependencies {
    // Rokid SDK
    implementation(files("libs/rokid-glass-sdk-2.0.0.aar"))
    
    // XR rendering
    implementation("androidx.xr:xr-core:1.0.0")
    implementation("androidx.xr:xr-compose:1.0.0")
    
    // Candle for AI (JNI)
    implementation(project(":candle-android"))
    
    // Audio
    implementation("com.google.oboe:oboe:1.8.0")
}
```

### 1.2 Rokid Glass Service

**File: `karana-android/app/src/main/java/ai/karana/glasses/RokidGlassService.kt`**
```kotlin
package ai.karana.glasses

import android.app.Service
import android.content.Intent
import android.os.IBinder
import com.rokid.glass.sdk.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

class RokidGlassService : Service() {
    
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    
    // Rokid SDK components
    private lateinit var glassDevice: GlassDevice
    private lateinit var displayManager: GlassDisplayManager
    private lateinit var sensorManager: GlassSensorManager
    private lateinit var hapticController: GlassHapticController
    
    // Karana components (JNI)
    private lateinit var oracle: OracleVeilJni
    
    override fun onCreate() {
        super.onCreate()
        initializeGlass()
        initializeKarana()
        startSensorLoop()
    }
    
    private fun initializeGlass() {
        glassDevice = GlassDevice.getInstance(this)
        displayManager = glassDevice.displayManager
        sensorManager = glassDevice.sensorManager
        hapticController = glassDevice.hapticController
        
        // Enable always-on display for AR overlay
        displayManager.setAlwaysOn(true)
        displayManager.setBrightness(0.7f)
    }
    
    private fun initializeKarana() {
        // Load Rust library
        System.loadLibrary("karana_core")
        
        // Initialize Oracle via JNI
        oracle = OracleVeilJni()
        oracle.initialize(
            modelPath = "${filesDir}/models/phi-3-mini-q4.gguf",
            npuEnabled = true
        )
    }
    
    private fun startSensorLoop() {
        // Voice input stream
        scope.launch {
            collectVoiceInput()
        }
        
        // Gaze tracking stream
        scope.launch {
            collectGazeData()
        }
        
        // IMU gesture stream
        scope.launch {
            collectGestureData()
        }
    }
    
    private suspend fun collectVoiceInput() {
        val voiceProcessor = VoiceProcessor(this)
        
        voiceProcessor.transcriptionFlow.collect { transcript ->
            // Send to Oracle for processing
            val response = oracle.mediate(
                intent = transcript,
                contextType = "voice",
                timestamp = System.currentTimeMillis()
            )
            
            // Display whisper response
            displayWhisper(response.whisper)
            
            // Play haptic feedback
            playHaptic(response.hapticPattern)
        }
    }
    
    private suspend fun collectGazeData() {
        sensorManager.gazeFlow.collect { gaze ->
            // Update Oracle context with gaze point
            oracle.updateContext(
                key = "gaze_point",
                value = "${gaze.x},${gaze.y}"
            )
            
            // Check for gaze-triggered actions
            if (gaze.dwellTimeMs > 500) {
                val target = identifyGazeTarget(gaze)
                if (target != null) {
                    oracle.mediate(
                        intent = "focus on $target",
                        contextType = "gaze",
                        timestamp = System.currentTimeMillis()
                    )
                }
            }
        }
    }
    
    private suspend fun collectGestureData() {
        sensorManager.imuFlow.collect { imu ->
            val gesture = detectGesture(imu)
            if (gesture != GestureType.NONE) {
                oracle.mediate(
                    intent = "gesture:${gesture.name}",
                    contextType = "gesture",
                    timestamp = System.currentTimeMillis()
                )
            }
        }
    }
    
    private fun displayWhisper(whisper: String) {
        displayManager.showOverlay(
            text = whisper,
            position = OverlayPosition.BOTTOM_CENTER,
            duration = 3000,
            style = WhisperStyle.SUBTLE
        )
    }
    
    private fun playHaptic(pattern: String) {
        when (pattern) {
            "success" -> hapticController.vibrate(
                VibrationPattern.builder()
                    .addPulse(50, 100)  // 50ms at 100% intensity
                    .build()
            )
            "confirm" -> hapticController.vibrate(
                VibrationPattern.builder()
                    .addPulse(30, 80)
                    .addPause(50)
                    .addPulse(30, 80)
                    .build()
            )
            "error" -> hapticController.vibrate(
                VibrationPattern.builder()
                    .addPulse(100, 100)
                    .addPause(50)
                    .addPulse(100, 100)
                    .addPause(50)
                    .addPulse(100, 100)
                    .build()
            )
        }
    }
    
    override fun onBind(intent: Intent?): IBinder? = null
    
    override fun onDestroy() {
        scope.cancel()
        oracle.shutdown()
        super.onDestroy()
    }
}
```

---

## Phase 2: JNI Bridge for Rust Core

### 2.1 JNI Definitions

**File: `karana-core/src/jni/mod.rs`**
```rust
#![cfg(target_os = "android")]

use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jlong, jstring, jboolean};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::oracle::veil::OracleVeil;
use crate::oracle::command::OracleChannels;

static mut RUNTIME: Option<Runtime> = None;
static mut ORACLE: Option<Arc<OracleVeil>> = None;

/// Initialize the Karana Oracle
#[no_mangle]
pub extern "C" fn Java_ai_karana_glasses_OracleVeilJni_initialize(
    env: JNIEnv,
    _class: JClass,
    model_path: JString,
    npu_enabled: jboolean,
) -> jboolean {
    let model_path: String = env.get_string(model_path)
        .expect("Invalid model path")
        .into();
    
    // Create Tokio runtime
    let rt = Runtime::new().expect("Failed to create runtime");
    
    // Initialize Oracle
    let result = rt.block_on(async {
        OracleVeil::new_with_npu(&model_path, npu_enabled != 0).await
    });
    
    match result {
        Ok(oracle) => {
            unsafe {
                RUNTIME = Some(rt);
                ORACLE = Some(Arc::new(oracle));
            }
            android_logger::init_once(
                android_logger::Config::default()
                    .with_min_level(log::Level::Info)
            );
            log::info!("[JNI] Oracle initialized with NPU={}", npu_enabled != 0);
            1
        }
        Err(e) => {
            log::error!("[JNI] Failed to initialize Oracle: {}", e);
            0
        }
    }
}

/// Process an intent through the Oracle
#[no_mangle]
pub extern "C" fn Java_ai_karana_glasses_OracleVeilJni_mediate(
    env: JNIEnv,
    _class: JClass,
    intent: JString,
    context_type: JString,
    timestamp: jlong,
) -> jstring {
    let intent: String = env.get_string(intent)
        .expect("Invalid intent")
        .into();
    let context_type: String = env.get_string(context_type)
        .expect("Invalid context type")
        .into();
    
    let response = unsafe {
        if let (Some(rt), Some(oracle)) = (&RUNTIME, &ORACLE) {
            rt.block_on(async {
                let ctx = crate::oracle::OracleContext {
                    source: match context_type.as_str() {
                        "voice" => crate::oracle::InputSource::Voice,
                        "gaze" => crate::oracle::InputSource::Gaze,
                        "gesture" => crate::oracle::InputSource::Gesture,
                        _ => crate::oracle::InputSource::Unknown,
                    },
                    timestamp: timestamp as u64,
                    ..Default::default()
                };
                
                match oracle.mediate(&intent, ctx).await {
                    Ok(resp) => serde_json::to_string(&resp).unwrap_or_default(),
                    Err(e) => format!(r#"{{"error":"{}"}}"#, e),
                }
            })
        } else {
            r#"{"error":"Oracle not initialized"}"#.to_string()
        }
    };
    
    env.new_string(response)
        .expect("Failed to create response string")
        .into_inner()
}

/// Update Oracle context
#[no_mangle]
pub extern "C" fn Java_ai_karana_glasses_OracleVeilJni_updateContext(
    env: JNIEnv,
    _class: JClass,
    key: JString,
    value: JString,
) {
    let key: String = env.get_string(key)
        .expect("Invalid key")
        .into();
    let value: String = env.get_string(value)
        .expect("Invalid value")
        .into();
    
    unsafe {
        if let Some(oracle) = &ORACLE {
            oracle.update_context(&key, &value);
        }
    }
}

/// Shutdown the Oracle
#[no_mangle]
pub extern "C" fn Java_ai_karana_glasses_OracleVeilJni_shutdown(
    _env: JNIEnv,
    _class: JClass,
) {
    log::info!("[JNI] Shutting down Oracle");
    unsafe {
        ORACLE = None;
        RUNTIME = None;
    }
}
```

### 2.2 Android CMake Build

**File: `karana-android/app/src/main/cpp/CMakeLists.txt`**
```cmake
cmake_minimum_required(VERSION 3.22)
project(karana_core)

# Rust library (pre-built)
add_library(karana_core SHARED IMPORTED)
set_target_properties(karana_core PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../../../libs/${ANDROID_ABI}/libkarana_core.so
)

# JNI glue
add_library(karana_jni SHARED
    karana_jni_glue.cpp
)

target_link_libraries(karana_jni
    karana_core
    android
    log
)
```

### 2.3 Kotlin JNI Wrapper

**File: `karana-android/app/src/main/java/ai/karana/glasses/OracleVeilJni.kt`**
```kotlin
package ai.karana.glasses

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

class OracleVeilJni {
    
    companion object {
        init {
            System.loadLibrary("karana_jni")
        }
    }
    
    // Native methods
    private external fun initialize(modelPath: String, npuEnabled: Boolean): Boolean
    private external fun mediate(intent: String, contextType: String, timestamp: Long): String
    private external fun updateContext(key: String, value: String)
    private external fun shutdown()
    
    // Kotlin API
    fun initialize(modelPath: String, npuEnabled: Boolean = true): Boolean {
        return initialize(modelPath, npuEnabled)
    }
    
    suspend fun mediate(
        intent: String,
        contextType: String = "voice",
        timestamp: Long = System.currentTimeMillis()
    ): OracleResponse {
        val jsonResponse = mediate(intent, contextType, timestamp)
        return Json.decodeFromString(jsonResponse)
    }
    
    fun updateContext(key: String, value: String) {
        updateContext(key, value)
    }
    
    fun shutdown() {
        shutdown()
    }
}

@Serializable
data class OracleResponse(
    val whisper: String,
    val hapticPattern: String = "none",
    val arOverlay: ArOverlay? = null,
    val error: String? = null
)

@Serializable
data class ArOverlay(
    val type: String,
    val content: String,
    val position: String,
    val duration: Long
)
```

---

## Phase 3: NPU Acceleration

### 3.1 Qualcomm NPU Integration

**File: `karana-core/src/ai/npu.rs`**
```rust
//! Qualcomm NPU (QNN/SNPE) acceleration for Phi-3 inference

use anyhow::{Result, anyhow};
use std::path::Path;

#[cfg(target_os = "android")]
use qnn_sys::*;

pub struct NpuAccelerator {
    #[cfg(target_os = "android")]
    context: QnnContext,
    #[cfg(target_os = "android")]
    graph: QnnGraph,
    model_loaded: bool,
}

impl NpuAccelerator {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "android")]
        {
            // Initialize QNN runtime
            let backend = qnn_get_backend(QNN_BACKEND_HTP)?;  // Hexagon Tensor Processor
            let context = QnnContext::create(backend)?;
            
            Ok(Self {
                context,
                graph: QnnGraph::empty(),
                model_loaded: false,
            })
        }
        
        #[cfg(not(target_os = "android"))]
        {
            Ok(Self { model_loaded: false })
        }
    }
    
    /// Load quantized model for NPU
    pub fn load_model(&mut self, model_path: &Path) -> Result<()> {
        #[cfg(target_os = "android")]
        {
            // Load QNN-compiled model (.so or .bin)
            let qnn_model_path = model_path.with_extension("qnn");
            
            if qnn_model_path.exists() {
                // Pre-compiled QNN model
                self.graph = self.context.load_binary(&qnn_model_path)?;
                log::info!("[NPU] Loaded pre-compiled QNN model");
            } else {
                // Convert GGUF to QNN on first run
                log::info!("[NPU] Converting GGUF to QNN format...");
                let converter = GgufToQnnConverter::new(&self.context)?;
                self.graph = converter.convert(model_path)?;
                
                // Cache the compiled model
                self.graph.save_binary(&qnn_model_path)?;
                log::info!("[NPU] Cached QNN model for future use");
            }
            
            self.model_loaded = true;
            Ok(())
        }
        
        #[cfg(not(target_os = "android"))]
        {
            log::warn!("[NPU] NPU not available on this platform, using CPU");
            self.model_loaded = true;
            Ok(())
        }
    }
    
    /// Run inference on NPU
    pub fn infer(&self, input_tokens: &[u32], max_tokens: usize) -> Result<Vec<u32>> {
        #[cfg(target_os = "android")]
        {
            if !self.model_loaded {
                return Err(anyhow!("Model not loaded"));
            }
            
            // Prepare input tensor
            let input_tensor = QnnTensor::from_slice(
                input_tokens,
                &[1, input_tokens.len() as u32],  // [batch, seq_len]
                QnnDataType::Uint32,
            )?;
            
            // Run inference
            let mut outputs = Vec::with_capacity(max_tokens);
            let mut current_input = input_tensor;
            
            for _ in 0..max_tokens {
                let output = self.graph.execute(&[&current_input])?;
                let next_token = output[0].argmax()?;
                
                if next_token == 2 {  // EOS token
                    break;
                }
                
                outputs.push(next_token);
                
                // Append token to input for next iteration
                current_input = QnnTensor::append(&current_input, next_token)?;
            }
            
            Ok(outputs)
        }
        
        #[cfg(not(target_os = "android"))]
        {
            // Fallback to CPU inference via Candle
            Err(anyhow!("NPU not available, use Candle CPU inference"))
        }
    }
    
    /// Get NPU performance stats
    pub fn get_stats(&self) -> NpuStats {
        #[cfg(target_os = "android")]
        {
            NpuStats {
                available: true,
                model_loaded: self.model_loaded,
                backend: "Qualcomm HTP".to_string(),
                tops: 12.0,
                memory_mb: self.graph.memory_usage_mb(),
            }
        }
        
        #[cfg(not(target_os = "android"))]
        {
            NpuStats {
                available: false,
                model_loaded: self.model_loaded,
                backend: "None".to_string(),
                tops: 0.0,
                memory_mb: 0,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NpuStats {
    pub available: bool,
    pub model_loaded: bool,
    pub backend: String,
    pub tops: f32,
    pub memory_mb: usize,
}
```

### 3.2 Hybrid CPU/NPU Inference

**File: `karana-core/src/ai/hybrid.rs`**
```rust
//! Hybrid inference: NPU for main model, CPU for embeddings

use anyhow::Result;
use crate::ai::npu::NpuAccelerator;
use candle_core::{Device, Tensor};

pub struct HybridInference {
    npu: Option<NpuAccelerator>,
    cpu_device: Device,
    use_npu: bool,
}

impl HybridInference {
    pub fn new(npu_enabled: bool) -> Result<Self> {
        let npu = if npu_enabled {
            match NpuAccelerator::new() {
                Ok(n) => {
                    log::info!("[AI] NPU accelerator initialized");
                    Some(n)
                }
                Err(e) => {
                    log::warn!("[AI] NPU init failed, using CPU: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            npu,
            cpu_device: Device::Cpu,
            use_npu: npu.is_some(),
        })
    }
    
    pub fn load_model(&mut self, model_path: &str) -> Result<()> {
        if let Some(ref mut npu) = self.npu {
            npu.load_model(std::path::Path::new(model_path))?;
        }
        Ok(())
    }
    
    /// Generate text with hybrid inference
    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Tokenize on CPU
        let tokens = self.tokenize(prompt)?;
        
        // Run inference on NPU if available
        let output_tokens = if self.use_npu {
            if let Some(ref npu) = self.npu {
                npu.infer(&tokens, max_tokens)?
            } else {
                self.cpu_infer(&tokens, max_tokens)?
            }
        } else {
            self.cpu_infer(&tokens, max_tokens)?
        };
        
        // Detokenize on CPU
        self.detokenize(&output_tokens)
    }
    
    /// Compute embeddings (always CPU for flexibility)
    pub fn embed(&self, text: &str) -> Result<Tensor> {
        // Embeddings use smaller model, CPU is fine
        let tokens = self.tokenize(text)?;
        let embeddings = self.cpu_embed(&tokens)?;
        Ok(embeddings)
    }
    
    fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        // Use tokenizers crate
        todo!("Implement tokenization")
    }
    
    fn detokenize(&self, tokens: &[u32]) -> Result<String> {
        // Use tokenizers crate
        todo!("Implement detokenization")
    }
    
    fn cpu_infer(&self, tokens: &[u32], max_tokens: usize) -> Result<Vec<u32>> {
        // Candle CPU inference fallback
        todo!("Implement CPU inference")
    }
    
    fn cpu_embed(&self, tokens: &[u32]) -> Result<Tensor> {
        // Candle CPU embeddings
        todo!("Implement CPU embeddings")
    }
}
```

---

## Phase 4: Gaze Tracking

### 4.1 OpenCV Gaze Detection

**File: `karana-core/src/hardware/gaze.rs`**
```rust
//! Eye gaze tracking using OpenCV on Rokid cameras

use anyhow::Result;
use opencv::{
    core::{Mat, Point, Rect, Scalar, Size},
    imgproc,
    objdetect::CascadeClassifier,
    prelude::*,
};

pub struct GazeTracker {
    eye_cascade: CascadeClassifier,
    face_cascade: CascadeClassifier,
    last_gaze: Option<GazePoint>,
    calibration: GazeCalibration,
}

#[derive(Debug, Clone, Copy)]
pub struct GazePoint {
    pub x: f32,      // 0.0 to 1.0, left to right
    pub y: f32,      // 0.0 to 1.0, top to bottom
    pub confidence: f32,
    pub dwell_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GazeCalibration {
    pub screen_width: u32,
    pub screen_height: u32,
    pub eye_offset_x: f32,
    pub eye_offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Default for GazeCalibration {
    fn default() -> Self {
        Self {
            screen_width: 1920,
            screen_height: 1080,
            eye_offset_x: 0.5,
            eye_offset_y: 0.5,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

impl GazeTracker {
    pub fn new() -> Result<Self> {
        // Load Haar cascades for face/eye detection
        let face_cascade = CascadeClassifier::new(
            "/system/etc/opencv/haarcascade_frontalface_default.xml"
        )?;
        let eye_cascade = CascadeClassifier::new(
            "/system/etc/opencv/haarcascade_eye.xml"
        )?;
        
        Ok(Self {
            eye_cascade,
            face_cascade,
            last_gaze: None,
            calibration: GazeCalibration::default(),
        })
    }
    
    /// Process a camera frame and detect gaze point
    pub fn process_frame(&mut self, frame: &Mat) -> Result<Option<GazePoint>> {
        // Convert to grayscale
        let mut gray = Mat::default();
        imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
        
        // Detect faces
        let mut faces = opencv::core::Vector::<Rect>::new();
        self.face_cascade.detect_multi_scale(
            &gray,
            &mut faces,
            1.1,
            3,
            0,
            Size::new(30, 30),
            Size::new(0, 0),
        )?;
        
        if faces.is_empty() {
            return Ok(None);
        }
        
        // Get the largest face (assumed to be user)
        let face = faces.iter()
            .max_by_key(|f| f.width * f.height)
            .unwrap();
        
        // Detect eyes within face region
        let face_roi = Mat::roi(&gray, face)?;
        let mut eyes = opencv::core::Vector::<Rect>::new();
        self.eye_cascade.detect_multi_scale(
            &face_roi,
            &mut eyes,
            1.1,
            3,
            0,
            Size::new(20, 20),
            Size::new(0, 0),
        )?;
        
        if eyes.len() < 2 {
            return Ok(None);
        }
        
        // Calculate gaze from eye positions
        let gaze = self.calculate_gaze(&eyes, &face)?;
        
        // Update dwell time
        let gaze = if let Some(last) = &self.last_gaze {
            let distance = ((gaze.x - last.x).powi(2) + (gaze.y - last.y).powi(2)).sqrt();
            if distance < 0.05 {  // Within 5% = same target
                GazePoint {
                    dwell_ms: last.dwell_ms + 33,  // ~30fps
                    ..gaze
                }
            } else {
                gaze
            }
        } else {
            gaze
        };
        
        self.last_gaze = Some(gaze);
        Ok(Some(gaze))
    }
    
    fn calculate_gaze(
        &self,
        eyes: &opencv::core::Vector<Rect>,
        face: &Rect,
    ) -> Result<GazePoint> {
        // Sort eyes by x position (left eye, right eye)
        let mut eye_centers: Vec<Point> = eyes.iter()
            .map(|e| Point::new(e.x + e.width / 2, e.y + e.height / 2))
            .collect();
        eye_centers.sort_by_key(|p| p.x);
        
        // Calculate midpoint between eyes
        let midpoint = Point::new(
            (eye_centers[0].x + eye_centers[1].x) / 2,
            (eye_centers[0].y + eye_centers[1].y) / 2,
        );
        
        // Normalize to face coordinates
        let rel_x = (midpoint.x as f32) / (face.width as f32);
        let rel_y = (midpoint.y as f32) / (face.height as f32);
        
        // Apply calibration
        let screen_x = (rel_x - self.calibration.eye_offset_x) * self.calibration.scale_x + 0.5;
        let screen_y = (rel_y - self.calibration.eye_offset_y) * self.calibration.scale_y + 0.5;
        
        Ok(GazePoint {
            x: screen_x.clamp(0.0, 1.0),
            y: screen_y.clamp(0.0, 1.0),
            confidence: 0.8,  // TODO: Calculate based on detection quality
            dwell_ms: 0,
        })
    }
    
    /// Calibrate gaze tracking with user looking at known points
    pub fn calibrate(&mut self, point: CalibrationPoint, detected: GazePoint) {
        // Adjust offsets based on calibration data
        // This is simplified; production would use more sophisticated calibration
        match point {
            CalibrationPoint::Center => {
                self.calibration.eye_offset_x = detected.x;
                self.calibration.eye_offset_y = detected.y;
            }
            CalibrationPoint::TopLeft => {
                // Adjust scale based on corner detection
                let expected_x = 0.1;
                let expected_y = 0.1;
                self.calibration.scale_x *= expected_x / (detected.x - self.calibration.eye_offset_x + 0.5);
                self.calibration.scale_y *= expected_y / (detected.y - self.calibration.eye_offset_y + 0.5);
            }
            // ... other calibration points
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CalibrationPoint {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
```

---

## Phase 5: Haptic Patterns

### 5.1 Linear Motor Driver

**File: `karana-core/src/hardware/haptic_driver.rs`**
```rust
//! Direct haptic motor control for glasses

use anyhow::Result;
use std::time::Duration;

#[cfg(target_os = "android")]
use android_hal::haptic::LinearMotor;

pub struct HapticDriver {
    #[cfg(target_os = "android")]
    motor: LinearMotor,
    patterns: PatternLibrary,
}

/// Pre-defined haptic patterns for Oracle responses
pub struct PatternLibrary {
    pub success: HapticSequence,
    pub confirm: HapticSequence,
    pub error: HapticSequence,
    pub attention: HapticSequence,
    pub thinking: HapticSequence,
    pub navigation: HapticSequence,
}

#[derive(Clone)]
pub struct HapticSequence {
    pub pulses: Vec<HapticPulse>,
}

#[derive(Clone, Copy)]
pub struct HapticPulse {
    pub duration_ms: u32,
    pub intensity: f32,  // 0.0 to 1.0
    pub pause_after_ms: u32,
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self {
            // Single short pulse - action completed
            success: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 50, intensity: 0.8, pause_after_ms: 0 },
                ],
            },
            // Double tap - confirmation required
            confirm: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 30, intensity: 0.6, pause_after_ms: 50 },
                    HapticPulse { duration_ms: 30, intensity: 0.6, pause_after_ms: 0 },
                ],
            },
            // Triple harsh - error
            error: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 40 },
                    HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 40 },
                    HapticPulse { duration_ms: 80, intensity: 1.0, pause_after_ms: 0 },
                ],
            },
            // Escalating pulse - get attention
            attention: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 100 },
                    HapticPulse { duration_ms: 30, intensity: 0.5, pause_after_ms: 100 },
                    HapticPulse { duration_ms: 40, intensity: 0.8, pause_after_ms: 0 },
                ],
            },
            // Gentle repeating - processing
            thinking: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 200 },
                    HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 200 },
                    HapticPulse { duration_ms: 20, intensity: 0.3, pause_after_ms: 200 },
                ],
            },
            // Directional tick - navigation
            navigation: HapticSequence {
                pulses: vec![
                    HapticPulse { duration_ms: 15, intensity: 0.5, pause_after_ms: 0 },
                ],
            },
        }
    }
}

impl HapticDriver {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "android")]
        {
            let motor = LinearMotor::open()?;
            Ok(Self {
                motor,
                patterns: PatternLibrary::default(),
            })
        }
        
        #[cfg(not(target_os = "android"))]
        {
            Ok(Self {
                patterns: PatternLibrary::default(),
            })
        }
    }
    
    /// Play a named pattern
    pub async fn play(&self, pattern_name: &str) -> Result<()> {
        let sequence = match pattern_name {
            "success" => &self.patterns.success,
            "confirm" => &self.patterns.confirm,
            "error" => &self.patterns.error,
            "attention" => &self.patterns.attention,
            "thinking" => &self.patterns.thinking,
            "navigation" | "nav_left" | "nav_right" => &self.patterns.navigation,
            _ => return Ok(()),  // Unknown pattern, ignore
        };
        
        self.play_sequence(sequence).await
    }
    
    /// Play a custom sequence
    pub async fn play_sequence(&self, sequence: &HapticSequence) -> Result<()> {
        for pulse in &sequence.pulses {
            self.vibrate(pulse.duration_ms, pulse.intensity)?;
            if pulse.pause_after_ms > 0 {
                tokio::time::sleep(Duration::from_millis(pulse.pause_after_ms as u64)).await;
            }
        }
        Ok(())
    }
    
    fn vibrate(&self, duration_ms: u32, intensity: f32) -> Result<()> {
        #[cfg(target_os = "android")]
        {
            self.motor.vibrate(duration_ms, (intensity * 255.0) as u8)?;
        }
        
        #[cfg(not(target_os = "android"))]
        {
            log::debug!("[HAPTIC] Vibrate {}ms @ {:.0}%", duration_ms, intensity * 100.0);
        }
        
        Ok(())
    }
}
```

---

## Phase 6: XReal Companion Mode

### 6.1 USB Display Streaming

**File: `karana-companion/src/xreal_bridge.rs`**
```rust
//! XReal glasses companion app (runs on phone/PC)

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct XRealBridge {
    display_tx: mpsc::Sender<OverlayFrame>,
    audio_rx: mpsc::Receiver<AudioChunk>,
    usb_connection: UsbDisplayConnection,
}

#[derive(Clone)]
pub struct OverlayFrame {
    pub text: String,
    pub position: OverlayPosition,
    pub style: WhisperStyle,
    pub timestamp: u64,
}

#[derive(Clone, Copy)]
pub enum OverlayPosition {
    TopCenter,
    BottomCenter,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy)]
pub enum WhisperStyle {
    Subtle,      // Low opacity, small font
    Normal,      // Standard visibility
    Emphasized,  // High contrast, larger
    Alert,       // Red tint, pulsing
}

pub struct AudioChunk {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

impl XRealBridge {
    pub async fn connect() -> Result<Self> {
        // Find XReal glasses via USB
        let usb = UsbDisplayConnection::find_xreal()?;
        
        // Create channels
        let (display_tx, mut display_rx) = mpsc::channel(32);
        let (audio_tx, audio_rx) = mpsc::channel(64);
        
        // Start USB streaming tasks
        let usb_clone = usb.clone();
        tokio::spawn(async move {
            while let Some(frame) = display_rx.recv().await {
                if let Err(e) = usb_clone.send_overlay(&frame).await {
                    log::error!("[XREAL] Display send failed: {}", e);
                }
            }
        });
        
        let usb_clone = usb.clone();
        tokio::spawn(async move {
            loop {
                match usb_clone.receive_audio().await {
                    Ok(chunk) => {
                        let _ = audio_tx.send(chunk).await;
                    }
                    Err(e) => {
                        log::error!("[XREAL] Audio receive failed: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(Self {
            display_tx,
            audio_rx,
            usb_connection: usb,
        })
    }
    
    /// Send a whisper overlay to the glasses
    pub async fn show_whisper(&self, whisper: &str, style: WhisperStyle) -> Result<()> {
        let frame = OverlayFrame {
            text: whisper.to_string(),
            position: OverlayPosition::BottomCenter,
            style,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        };
        
        self.display_tx.send(frame).await?;
        Ok(())
    }
    
    /// Get next audio chunk from glasses mic
    pub async fn receive_audio(&mut self) -> Option<AudioChunk> {
        self.audio_rx.recv().await
    }
}

struct UsbDisplayConnection {
    // USB DP Alt mode display connection
    // This is platform-specific (libusb on Linux, WinUSB on Windows)
}

impl UsbDisplayConnection {
    fn find_xreal() -> Result<Self> {
        // Enumerate USB devices, find XReal by VID/PID
        todo!("Implement USB device discovery")
    }
    
    async fn send_overlay(&self, frame: &OverlayFrame) -> Result<()> {
        // Render text to framebuffer and send via DisplayPort
        todo!("Implement DP streaming")
    }
    
    async fn receive_audio(&self) -> Result<AudioChunk> {
        // Receive audio from USB audio class device
        todo!("Implement USB audio receive")
    }
    
    fn clone(&self) -> Self {
        todo!()
    }
}
```

---

## Build & Deployment

### Android Build Script

**File: `scripts/build_android.sh`**
```bash
#!/bin/bash
set -e

# Build Rust core for Android
echo "Building Rust core for Android ARM64..."
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/26.1.10909125
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

cd karana-core
cargo ndk -t arm64-v8a build --release

# Copy library to Android project
cp target/aarch64-linux-android/release/libkarana_core.so \
   ../karana-android/app/libs/arm64-v8a/

# Build Android APK
echo "Building Android APK..."
cd ../karana-android
./gradlew assembleRelease

echo "APK built: app/build/outputs/apk/release/app-release.apk"
```

### Model Preparation

**File: `scripts/prepare_models.sh`**
```bash
#!/bin/bash
set -e

MODELS_DIR="karana-android/app/src/main/assets/models"
mkdir -p $MODELS_DIR

# Download Phi-3 Mini Q4 GGUF
echo "Downloading Phi-3 Mini Q4..."
wget -O $MODELS_DIR/phi-3-mini-q4.gguf \
    "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf"

# Prepare for QNN conversion (Rokid NPU)
echo "Preparing QNN model..."
python scripts/convert_to_qnn.py \
    --input $MODELS_DIR/phi-3-mini-q4.gguf \
    --output $MODELS_DIR/phi-3-mini-q4.qnn \
    --target qcs6490

echo "Models prepared!"
```

---

## Implementation Timeline

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Android project setup | Build system, JNI skeleton |
| 2 | Rokid SDK integration | Camera, display, sensors |
| 3 | JNI bridge | OracleVeil accessible from Kotlin |
| 4 | NPU acceleration | Phi-3 running on Hexagon |
| 5 | Gaze tracking | OpenCV eye detection working |
| 6 | Haptic patterns | All feedback patterns implemented |
| 7 | XReal companion | USB streaming working |
| 8 | Integration testing | End-to-end on real hardware |

---

## Hardware Requirements

### Development
- Android Studio Arctic Fox+
- NDK r26+
- Rokid SDK 2.0+
- QEMU with ARM64 emulation (for testing without hardware)

### Production
- Rokid Max Pro (primary target)
- OR XReal Air 2 Ultra + Android phone
- Models downloaded to device storage

---

*PLAN_HARDWARE_GLASSES.md - December 3, 2025*
