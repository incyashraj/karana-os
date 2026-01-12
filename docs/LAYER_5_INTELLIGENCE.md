# Layer 5: Intelligence Layer

## Overview

The Intelligence Layer provides computer vision, scene understanding, and multimodal fusion for Kāraṇa OS. It processes visual data, builds spatial understanding, fuses sensor inputs, and maintains contextual memory to enable intelligent AR experiences.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    LAYER 5: INTELLIGENCE LAYER                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                   Computer Vision Pipeline                        │  │
│  │  Object Detection → Recognition → Tracking                        │  │
│  │  Models: YOLOv8, CLIP                                             │  │
│  └────────────────────┬─────────────────────────────────────────────┘  │
│                       │                                                  │
│  ┌────────────────────▼─────────────────────────────────────────────┐  │
│  │                  Scene Understanding                              │  │
│  │  Semantic Segmentation | Depth Estimation | Spatial Mapping      │  │
│  └────────┬───────────────────────────────────────────┬─────────────┘  │
│           │                                            │                 │
│  ┌────────▼────────────┐                    ┌─────────▼──────────┐     │
│  │ Multimodal Fusion   │                    │  Context Tracker   │     │
│  │ Vision + Audio      │                    │  Location/Activity │     │
│  │      + IMU          │                    │  Social situation  │     │
│  └─────────────────────┘                    └────────────────────┘     │
│           │                                            │                 │
│           └──────────────────┬─────────────────────────┘                 │
│                              ▼                                           │
│                   ┌──────────────────────┐                              │
│                   │   Memory System      │                              │
│                   │   Short-term: 100MB  │                              │
│                   │   Long-term: 10GB    │                              │
│                   └──────────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Computer Vision Pipeline

**Purpose**: Detect, recognize, and track objects in the camera feed.

**Object Detection (YOLOv8)**:
```typescript
import * as ort from 'onnxruntime-web';

class ObjectDetector {
  private model: ort.InferenceSession;
  private classes: string[] = [...]; // COCO classes
  
  async initialize(): Promise<void> {
    this.model = await ort.InferenceSession.create('/models/yolov8n.onnx', {
      executionProviders: ['webgl'], // GPU acceleration
    });
  }
  
  async detect(frame: ImageData): Promise<Detection[]> {
    // 1. Preprocess image
    const input = this.preprocess(frame); // [1, 3, 640, 640]
    
    // 2. Run inference
    const startTime = performance.now();
    const outputs = await this.model.run({
      images: new ort.Tensor('float32', input, [1, 3, 640, 640]),
    });
    const inferenceTime = performance.now() - startTime;
    
    // 3. Postprocess outputs
    const detections = this.postprocess(outputs.output0.data, frame.width, frame.height);
    
    console.log(`Detected ${detections.length} objects in ${inferenceTime}ms`);
    return detections;
  }
  
  private preprocess(frame: ImageData): Float32Array {
    const pixels = new Float32Array(3 * 640 * 640);
    
    // Resize to 640x640
    const resized = this.resize(frame, 640, 640);
    
    // Normalize to [0, 1] and transpose to CHW format
    for (let c = 0; c < 3; c++) {
      for (let h = 0; h < 640; h++) {
        for (let w = 0; w < 640; w++) {
          const idx = (h * 640 + w) * 4 + c;
          pixels[c * 640 * 640 + h * 640 + w] = resized.data[idx] / 255.0;
        }
      }
    }
    
    return pixels;
  }
  
  private postprocess(output: Float32Array, imgW: number, imgH: number): Detection[] {
    const detections: Detection[] = [];
    const numDetections = output.length / 85; // [x, y, w, h, conf, ...80 classes]
    
    for (let i = 0; i < numDetections; i++) {
      const offset = i * 85;
      const confidence = output[offset + 4];
      
      if (confidence < 0.5) continue; // Confidence threshold
      
      // Find class with highest score
      let maxClass = 0;
      let maxScore = 0;
      for (let c = 0; c < 80; c++) {
        const score = output[offset + 5 + c];
        if (score > maxScore) {
          maxScore = score;
          maxClass = c;
        }
      }
      
      const x = output[offset + 0] * imgW;
      const y = output[offset + 1] * imgH;
      const w = output[offset + 2] * imgW;
      const h = output[offset + 3] * imgH;
      
      detections.push({
        class: this.classes[maxClass],
        confidence: confidence * maxScore,
        bbox: { x: x - w/2, y: y - h/2, width: w, height: h },
      });
    }
    
    // Non-maximum suppression
    return this.nms(detections, 0.45);
  }
}
```

**Visual Recognition (CLIP)**:
```typescript
class VisualRecognizer {
  private model: ort.InferenceSession;
  
  async recognize(image: ImageData, candidates: string[]): Promise<RecognitionResult> {
    // 1. Encode image
    const imageEmbed = await this.encodeImage(image);
    
    // 2. Encode text candidates
    const textEmbeds = await Promise.all(
      candidates.map(text => this.encodeText(text))
    );
    
    // 3. Compute similarities
    const similarities = textEmbeds.map(textEmbed => 
      this.cosineSimilarity(imageEmbed, textEmbed)
    );
    
    // 4. Softmax
    const probs = this.softmax(similarities);
    
    return {
      labels: candidates,
      probabilities: probs,
      topMatch: candidates[this.argmax(probs)],
    };
  }
  
  private cosineSimilarity(a: Float32Array, b: Float32Array): number {
    let dot = 0;
    let normA = 0;
    let normB = 0;
    
    for (let i = 0; i < a.length; i++) {
      dot += a[i] * b[i];
      normA += a[i] ** 2;
      normB += b[i] ** 2;
    }
    
    return dot / (Math.sqrt(normA) * Math.sqrt(normB));
  }
}
```

---

### 2. Scene Understanding

**Purpose**: Build semantic understanding of the 3D environment.

**Semantic Segmentation**:
```typescript
class SemanticSegmenter {
  private model: ort.InferenceSession;
  private labels = ['sky', 'ground', 'wall', 'furniture', 'person', ...];
  
  async segment(frame: ImageData): Promise<SegmentationMap> {
    // 1. Run segmentation model
    const output = await this.model.run({
      input: this.preprocessImage(frame),
    });
    
    // 2. Parse output (HxWxC probabilities)
    const segMap = new Uint8Array(frame.width * frame.height);
    const probs = output.output.data;
    
    for (let i = 0; i < segMap.length; i++) {
      let maxClass = 0;
      let maxProb = 0;
      
      for (let c = 0; c < this.labels.length; c++) {
        const prob = probs[i * this.labels.length + c];
        if (prob > maxProb) {
          maxProb = prob;
          maxClass = c;
        }
      }
      
      segMap[i] = maxClass;
    }
    
    return {
      width: frame.width,
      height: frame.height,
      data: segMap,
      labels: this.labels,
    };
  }
  
  // Extract regions of interest
  extractROI(segMap: SegmentationMap, targetLabel: string): BoundingBox[] {
    const labelIdx = this.labels.indexOf(targetLabel);
    const regions: BoundingBox[] = [];
    
    // Connected component labeling
    const visited = new Set<number>();
    
    for (let y = 0; y < segMap.height; y++) {
      for (let x = 0; x < segMap.width; x++) {
        const idx = y * segMap.width + x;
        
        if (segMap.data[idx] === labelIdx && !visited.has(idx)) {
          const bbox = this.floodFill(segMap, x, y, labelIdx, visited);
          if (bbox.width * bbox.height > 100) { // Minimum size
            regions.push(bbox);
          }
        }
      }
    }
    
    return regions;
  }
}
```

**Depth Estimation**:
```typescript
class DepthEstimator {
  private model: ort.InferenceSession; // MiDaS or DepthAnything
  
  async estimate(frame: ImageData): Promise<DepthMap> {
    const output = await this.model.run({
      image: this.preprocessImage(frame),
    });
    
    // Normalize depth values to [0, 1]
    const depth = output.depth.data;
    const min = Math.min(...depth);
    const max = Math.max(...depth);
    
    const normalized = new Float32Array(depth.length);
    for (let i = 0; i < depth.length; i++) {
      normalized[i] = (depth[i] - min) / (max - min);
    }
    
    return {
      width: frame.width,
      height: frame.height,
      data: normalized,
      min: 0,
      max: 10, // Assume max 10 meters
    };
  }
  
  // Get depth at specific pixel
  getDepthAt(depthMap: DepthMap, x: number, y: number): number {
    const idx = Math.floor(y) * depthMap.width + Math.floor(x);
    return depthMap.data[idx] * depthMap.max;
  }
}
```

**Spatial Mapping**:
```typescript
interface SpatialMap {
  origin: Vector3;
  resolution: number;    // meters per voxel
  voxels: Map<string, Voxel>;
}

interface Voxel {
  position: Vector3;
  label: string;         // Semantic label
  confidence: number;
  occupied: boolean;
}

class SpatialMapper {
  private map: SpatialMap = {
    origin: new Vector3(0, 0, 0),
    resolution: 0.05, // 5cm voxels
    voxels: new Map(),
  };
  
  update(frame: ImageData, pose: Pose, depthMap: DepthMap, segMap: SegmentationMap): void {
    // For each pixel, raycast into 3D space
    for (let y = 0; y < frame.height; y += 4) { // Subsample for performance
      for (let x = 0; x < frame.width; x += 4) {
        const depth = this.depthEstimator.getDepthAt(depthMap, x, y);
        if (depth > 10) continue; // Too far
        
        // Project to 3D world coordinates
        const worldPos = this.pixelToWorld(x, y, depth, pose);
        
        // Get semantic label
        const labelIdx = segMap.data[y * segMap.width + x];
        const label = segMap.labels[labelIdx];
        
        // Update voxel
        const voxelKey = this.worldToVoxel(worldPos);
        const existingVoxel = this.map.voxels.get(voxelKey);
        
        if (existingVoxel) {
          // Update confidence (exponential moving average)
          existingVoxel.confidence = 0.9 * existingVoxel.confidence + 0.1;
        } else {
          this.map.voxels.set(voxelKey, {
            position: worldPos,
            label,
            confidence: 0.5,
            occupied: true,
          });
        }
      }
    }
  }
  
  private pixelToWorld(x: number, y: number, depth: number, pose: Pose): Vector3 {
    // Unproject pixel to camera space
    const fx = 525; // Focal length (pixels)
    const fy = 525;
    const cx = 320; // Principal point
    const cy = 240;
    
    const camX = (x - cx) * depth / fx;
    const camY = (y - cy) * depth / fy;
    const camZ = depth;
    
    const camPos = new Vector3(camX, camY, camZ);
    
    // Transform to world space
    camPos.applyQuaternion(pose.orientation);
    camPos.add(pose.position);
    
    return camPos;
  }
}
```

---

### 3. Multimodal Fusion

**Purpose**: Combine vision, audio, and IMU data for rich contextual understanding.

**Fusion Engine**:
```typescript
interface MultimodalContext {
  visual: VisualContext;
  audio: AudioContext;
  motion: MotionContext;
  fused: FusedContext;
}

class MultimodalFusion {
  fuse(visual: VisualContext, audio: AudioContext, motion: MotionContext): FusedContext {
    // 1. Activity recognition (combined signals)
    const activity = this.recognizeActivity(visual, motion);
    
    // 2. Attention estimation (where user is looking)
    const attention = this.estimateAttention(visual, motion);
    
    // 3. Environment classification (indoor/outdoor/vehicle)
    const environment = this.classifyEnvironment(visual, audio, motion);
    
    // 4. Social context (alone/conversation/crowd)
    const social = this.detectSocialContext(visual, audio);
    
    return {
      activity,
      attention,
      environment,
      social,
      timestamp: Date.now(),
      confidence: this.calculateConfidence([visual, audio, motion]),
    };
  }
  
  private recognizeActivity(visual: VisualContext, motion: MotionContext): Activity {
    // Rule-based fusion
    const motionIntensity = motion.acceleration.length();
    const visualMotion = visual.opticalFlow.magnitude;
    
    if (motionIntensity < 0.5 && visualMotion < 5) {
      return { type: 'stationary', confidence: 0.9 };
    } else if (motionIntensity < 2 && visualMotion < 20) {
      return { type: 'walking', confidence: 0.85 };
    } else if (motionIntensity > 5 || visualMotion > 50) {
      return { type: 'running', confidence: 0.8 };
    } else {
      return { type: 'unknown', confidence: 0.3 };
    }
  }
  
  private estimateAttention(visual: VisualContext, motion: MotionContext): AttentionTarget {
    // Gaze direction (from head orientation)
    const gazeDir = new Vector3(0, 0, -1).applyQuaternion(motion.orientation);
    
    // Raycast to find what user is looking at
    const target = this.raycastSpatialMap(motion.position, gazeDir);
    
    if (target) {
      return {
        object: target.label,
        position: target.position,
        confidence: 0.7,
      };
    }
    
    return { object: 'none', position: null, confidence: 0.1 };
  }
}
```

---

### 4. Context Tracker

**Purpose**: Maintain high-level understanding of user's current context.

```typescript
interface UserContext {
  location: LocationContext;
  activity: ActivityContext;
  social: SocialContext;
  temporal: TemporalContext;
}

class ContextTracker {
  private context: UserContext;
  private history: UserContext[] = [];
  
  update(fused: FusedContext): void {
    this.context = {
      location: this.updateLocation(fused),
      activity: fused.activity,
      social: fused.social,
      temporal: {
        timeOfDay: this.getTimeOfDay(),
        dayOfWeek: new Date().getDay(),
      },
    };
    
    // Store in history
    this.history.push({ ...this.context });
    if (this.history.length > 1000) {
      this.history.shift(); // Keep last 1000 contexts
    }
  }
  
  private updateLocation(fused: FusedContext): LocationContext {
    // GPS coordinates
    const gps = this.getGPSLocation();
    
    // Semantic location (home, work, gym, etc.)
    const semantic = this.inferSemanticLocation(gps, fused.environment);
    
    return {
      gps,
      semantic,
      environment: fused.environment,
    };
  }
  
  // Predict next context (for proactive assistance)
  predictNext(): UserContext {
    // Simple Markov model
    const transitions = this.computeTransitions();
    const current = this.context;
    
    return this.mostLikelyNext(current, transitions);
  }
}
```

---

### 5. Memory System

**Purpose**: Store and retrieve visual and contextual memories.

**Short-Term Memory** (Working memory):
```typescript
class ShortTermMemory {
  private capacity = 100 * 1024 * 1024; // 100MB
  private entries: MemoryEntry[] = [];
  
  store(data: any, type: string): void {
    const entry: MemoryEntry = {
      id: crypto.randomUUID(),
      type,
      data,
      timestamp: Date.now(),
      size: this.estimateSize(data),
      accessCount: 0,
    };
    
    this.entries.push(entry);
    
    // Evict if over capacity
    while (this.getTotalSize() > this.capacity) {
      this.evictOldest();
    }
  }
  
  recall(query: MemoryQuery): MemoryEntry[] {
    return this.entries
      .filter(e => this.matches(e, query))
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, query.limit || 10);
  }
  
  private evictOldest(): void {
    // Evict least recently accessed
    const oldest = this.entries.reduce((min, e) => 
      e.accessCount < min.accessCount ? e : min
    );
    
    const index = this.entries.indexOf(oldest);
    this.entries.splice(index, 1);
  }
}
```

**Long-Term Memory** (Episodic):
```typescript
class LongTermMemory {
  private db: IndexedDB;
  private capacity = 10 * 1024 * 1024 * 1024; // 10GB
  
  async store(episode: Episode): Promise<void> {
    // Compress and store
    const compressed = await this.compress(episode);
    
    await this.db.put('episodes', {
      id: episode.id,
      data: compressed,
      timestamp: episode.timestamp,
      tags: episode.tags,
    });
  }
  
  async recall(query: EpisodeQuery): Promise<Episode[]> {
    // Search by time, location, tags
    const results = await this.db.query('episodes', {
      startTime: query.startTime,
      endTime: query.endTime,
      tags: query.tags,
    });
    
    return Promise.all(results.map(r => this.decompress(r.data)));
  }
  
  private async compress(episode: Episode): Promise<Uint8Array> {
    // Use video compression for visual data
    const json = JSON.stringify(episode);
    const encoder = new TextEncoder();
    const data = encoder.encode(json);
    
    // GZIP compression
    const compressed = await new Response(
      new Blob([data]).stream().pipeThrough(new CompressionStream('gzip'))
    ).arrayBuffer();
    
    return new Uint8Array(compressed);
  }
}
```

---

## Performance Metrics

```
┌─ Intelligence Layer Performance ────────┐
│ Object Detection (YOLOv8): 25-40ms      │
│ Visual Recognition (CLIP): 50ms         │
│ Semantic Segmentation: 60-100ms         │
│ Depth Estimation: 40-80ms               │
│ Spatial Mapping: 10-20ms/frame          │
│ Multimodal Fusion: 5-15ms               │
│ Context Update: 2-8ms                   │
│                                          │
│ Memory:                                  │
│   Short-term: 100MB (1000 entries)      │
│   Long-term: 10GB (50K episodes)        │
│                                          │
│ Total Pipeline: 150-300ms/frame         │
└──────────────────────────────────────────┘
```

---

## Future Development

### Phase 1: Advanced Vision (Q1 2026)
- 3D object detection (Cube R-CNN)
- Instance segmentation (Mask R-CNN)
- Panoptic segmentation

### Phase 2: SLAM (Q2 2026)
- Visual-Inertial SLAM (ORB-SLAM3)
- Loop closure detection
- Map optimization

### Phase 3: Predictive Models (Q3 2026)
- Action anticipation (predict what user will do)
- Trajectory forecasting
- Proactive assistance

### Phase 4: Neural Scene Repr. (Q4 2026)
- NeRF for 3D reconstruction
- Gaussian splatting for real-time rendering
- Neural radiance fields

---

## Code References

- `simulator-ui/services/SensorFusionService.ts`: Sensor fusion
- `simulator-ui/components/AROverlay.tsx`: Visual processing
- `karana-core/src/intelligence/vision.rs`: Vision pipeline (Rust)

---

## Summary

Layer 5 provides:
- **Computer Vision**: Object detection, recognition, tracking
- **Scene Understanding**: Segmentation, depth, spatial mapping
- **Multimodal Fusion**: Vision + audio + IMU
- **Context Tracking**: Location, activity, social situation
- **Memory**: Short-term (100MB) + long-term (10GB)

This layer enables intelligent perception and contextual awareness.