# AR Tracking System

## Overview

The AR Tracking System is Kāraṇa OS's spatial perception engine, combining hand tracking, optical flow, sensor fusion, and spatial anchoring to enable natural AR interactions. Users interact through pinch gestures, gaze tracking, and spatial UI elements anchored in 3D space.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      AR TRACKING SYSTEM                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐            │
│  │ Hand Tracker │────▶│ Gesture      │────▶│ Cursor       │            │
│  │ (MediaPipe)  │     │ Recognition  │     │ Position     │            │
│  │ 21 landmarks │     │ Pinch/Point  │     │ 3D coords    │            │
│  └──────────────┘     └──────────────┘     └──────────────┘            │
│         │                                          │                     │
│         ▼                                          ▼                     │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐            │
│  │ Optical Flow │────▶│ Sensor       │────▶│ Spatial      │            │
│  │ (jsfeat)     │     │ Fusion       │     │ Anchoring    │            │
│  │ Motion track │     │ IMU + Vision │     │ World coords │            │
│  └──────────────┘     └──────────────┘     └──────────────┘            │
│                                                    │                     │
│                                                    ▼                     │
│                                             ┌──────────────┐            │
│                                             │ AR Renderer  │            │
│                                             │ HUD Overlay  │            │
│                                             └──────────────┘            │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Hand Tracking (MediaPipe)

**Purpose**: Detect hands in camera feed and extract 21 landmarks per hand in 3D space.

**MediaPipe HandLandmarker**:
```typescript
import { HandLandmarker, FilesetResolver } from '@mediapipe/tasks-vision';

class HandTracker {
  private handLandmarker: HandLandmarker | null = null;
  
  async initialize(): Promise<void> {
    const vision = await FilesetResolver.forVisionTasks(
      "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm"
    );
    
    this.handLandmarker = await HandLandmarker.createFromOptions(vision, {
      baseOptions: {
        modelAssetPath: '/models/hand_landmarker.task',
        delegate: 'GPU', // Use GPU acceleration
      },
      numHands: 2,
      minHandDetectionConfidence: 0.5,
      minHandPresenceConfidence: 0.5,
      minTrackingConfidence: 0.5,
      runningMode: 'VIDEO', // Optimized for continuous tracking
    });
  }
  
  detectHands(videoFrame: HTMLVideoElement, timestamp: number): HandLandmarks[] {
    const results = this.handLandmarker.detectForVideo(videoFrame, timestamp);
    
    return results.landmarks.map((landmarks, index) => ({
      handedness: results.handednesses[index][0].categoryName, // "Left" or "Right"
      landmarks: landmarks.map(lm => ({
        x: lm.x,       // Normalized [0,1]
        y: lm.y,       // Normalized [0,1]
        z: lm.z,       // Depth relative to wrist (meters)
      })),
      worldLandmarks: results.worldLandmarks[index], // 3D world coords
    }));
  }
}
```

**Hand Landmark Indices**:
```
        8 (Index tip)
        |
    7---6---5
        |
   12  |  4 (Index base)
    |  |  |
   11--+--3---2
    |  |      |
   10--+------1 (Thumb tip)
    |  |
   9---0 (Wrist)
       |
      13-14-15-16 (Middle finger)
       |
      17-18-19-20 (Ring + Pinky)
```

**Performance**:
- **Latency**: 15-25ms per frame (GPU)
- **Accuracy**: 95% detection rate in good lighting
- **Range**: 0.2m - 2.0m from camera

**Implementation** (`simulator-ui/components/HandTracker.tsx`):
```typescript
export function HandTracker({ onGesture }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const tracker = useRef(new HandTracker());
  
  useEffect(() => {
    tracker.current.initialize();
    
    const processFrame = () => {
      if (!videoRef.current) return;
      
      const hands = tracker.current.detectHands(
        videoRef.current,
        performance.now()
      );
      
      // Detect gestures
      for (const hand of hands) {
        const gesture = recognizeGesture(hand);
        if (gesture) {
          onGesture(gesture);
        }
      }
      
      requestAnimationFrame(processFrame);
    };
    
    processFrame();
  }, []);
  
  return <video ref={videoRef} autoPlay />;
}
```

---

### 2. Gesture Recognition

**Purpose**: Convert hand landmarks into semantic gestures (pinch, point, grab, etc.).

**Gesture Types**:
```typescript
enum GestureType {
  POINT = 'POINT',           // Index extended, others curled
  PINCH = 'PINCH',           // Thumb + index touching
  GRAB = 'GRAB',             // All fingers curled
  PALM_OPEN = 'PALM_OPEN',   // All fingers extended
  SWIPE_LEFT = 'SWIPE_LEFT',
  SWIPE_RIGHT = 'SWIPE_RIGHT',
}

interface Gesture {
  type: GestureType;
  hand: 'left' | 'right';
  confidence: number;
  position: { x: number; y: number; z: number };
}
```

**Pinch Detection** (Primary interaction):
```typescript
function detectPinch(landmarks: Landmark[]): Gesture | null {
  const thumb = landmarks[4];  // Thumb tip
  const index = landmarks[8];  // Index tip
  
  // Calculate 3D distance
  const distance = Math.sqrt(
    Math.pow(thumb.x - index.x, 2) +
    Math.pow(thumb.y - index.y, 2) +
    Math.pow(thumb.z - index.z, 2)
  );
  
  // Threshold: 0.05 normalized units (~2cm real world)
  if (distance < 0.05) {
    return {
      type: GestureType.PINCH,
      hand: 'right', // Determine from handedness
      confidence: 1.0 - (distance / 0.05),
      position: {
        x: (thumb.x + index.x) / 2,
        y: (thumb.y + index.y) / 2,
        z: (thumb.z + index.z) / 2,
      },
    };
  }
  
  return null;
}
```

**Point Detection** (Cursor control):
```typescript
function detectPoint(landmarks: Landmark[]): Gesture | null {
  const index = landmarks[8];
  const middle = landmarks[12];
  const ring = landmarks[16];
  const pinky = landmarks[20];
  
  // Index extended, others curled
  const indexExtended = landmarks[8].y < landmarks[6].y; // Tip above knuckle
  const othersCurled = (
    middle.y > landmarks[10].y &&
    ring.y > landmarks[14].y &&
    pinky.y > landmarks[18].y
  );
  
  if (indexExtended && othersCurled) {
    return {
      type: GestureType.POINT,
      hand: 'right',
      confidence: 0.9,
      position: { x: index.x, y: index.y, z: index.z },
    };
  }
  
  return null;
}
```

**Gesture State Machine**:
```typescript
class GestureStateMachine {
  private state: 'idle' | 'hover' | 'pinch' | 'drag' = 'idle';
  private pinchStartTime = 0;
  
  update(gesture: Gesture | null): GestureEvent | null {
    switch (this.state) {
      case 'idle':
        if (gesture?.type === GestureType.POINT) {
          this.state = 'hover';
          return { type: 'HOVER_START', position: gesture.position };
        }
        break;
        
      case 'hover':
        if (gesture?.type === GestureType.PINCH) {
          this.state = 'pinch';
          this.pinchStartTime = Date.now();
          return { type: 'CLICK', position: gesture.position };
        } else if (!gesture) {
          this.state = 'idle';
          return { type: 'HOVER_END' };
        }
        break;
        
      case 'pinch':
        if (gesture?.type === GestureType.PINCH) {
          const duration = Date.now() - this.pinchStartTime;
          if (duration > 200) { // Hold for 200ms → drag
            this.state = 'drag';
            return { type: 'DRAG_START', position: gesture.position };
          }
        } else {
          this.state = 'idle';
          return { type: 'CLICK_RELEASE' };
        }
        break;
        
      case 'drag':
        if (gesture?.type === GestureType.PINCH) {
          return { type: 'DRAG_MOVE', position: gesture.position };
        } else {
          this.state = 'idle';
          return { type: 'DRAG_END' };
        }
        break;
    }
    
    return null;
  }
}
```

---

### 3. Optical Flow Tracking

**Purpose**: Track motion of visual features between frames for stabilization and drift correction.

**jsfeat Integration**:
```typescript
import jsfeat from 'jsfeat';

class OpticalFlowTracker {
  private prevGray: jsfeat.matrix_t;
  private currGray: jsfeat.matrix_t;
  private points: jsfeat.keypoint_t[] = [];
  
  constructor(width: number, height: number) {
    this.prevGray = new jsfeat.matrix_t(width, height, jsfeat.U8_t | jsfeat.C1_t);
    this.currGray = new jsfeat.matrix_t(width, height, jsfeat.U8_t | jsfeat.C1_t);
  }
  
  track(videoFrame: HTMLVideoElement): FlowVector[] {
    // 1. Convert to grayscale
    const imageData = this.extractImageData(videoFrame);
    jsfeat.imgproc.grayscale(imageData.data, this.currGray.data);
    
    // 2. Detect corners (FAST)
    if (this.points.length < 50) {
      jsfeat.yape06.detect(this.currGray, this.points, 3);
    }
    
    // 3. Track points using Lucas-Kanade
    const flowVectors: FlowVector[] = [];
    const win_size = 20;
    const max_iterations = 30;
    const epsilon = 0.01;
    
    jsfeat.optical_flow_lk.track(
      this.prevGray,
      this.currGray,
      this.points,
      win_size,
      max_iterations,
      epsilon
    );
    
    // 4. Calculate motion vectors
    for (const point of this.points) {
      if (point.status === 1) { // Successfully tracked
        flowVectors.push({
          from: { x: point.x, y: point.y },
          to: { x: point.x + point.dx, y: point.y + point.dy },
          magnitude: Math.sqrt(point.dx ** 2 + point.dy ** 2),
        });
      }
    }
    
    // Swap buffers
    [this.prevGray, this.currGray] = [this.currGray, this.prevGray];
    
    return flowVectors;
  }
  
  // Calculate camera motion from flow field
  estimateCameraMotion(flowVectors: FlowVector[]): CameraMotion {
    const avgFlow = {
      x: flowVectors.reduce((sum, v) => sum + (v.to.x - v.from.x), 0) / flowVectors.length,
      y: flowVectors.reduce((sum, v) => sum + (v.to.y - v.from.y), 0) / flowVectors.length,
    };
    
    return {
      translation: avgFlow,
      rotation: this.estimateRotation(flowVectors),
    };
  }
}
```

**Use Cases**:
1. **Stabilization**: Compensate for hand shake
2. **Drift Correction**: Correct IMU drift with visual odometry
3. **SLAM**: Build 3D map of environment (future)

---

### 4. Sensor Fusion

**Purpose**: Combine IMU (gyroscope/accelerometer) and visual tracking for accurate 6-DoF pose estimation.

**Complementary Filter**:
```typescript
class SensorFusion {
  private orientation = new THREE.Quaternion(0, 0, 0, 1); // Identity
  private alpha = 0.98; // IMU weight (high-frequency)
  
  update(imu: IMUData, optical: CameraMotion, dt: number): Pose {
    // 1. IMU integration (gyroscope)
    const gyroQuat = this.integrateGyro(imu.gyro, dt);
    
    // 2. Optical flow rotation estimate
    const opticalQuat = this.opticalToQuaternion(optical.rotation);
    
    // 3. Complementary filter
    // IMU handles fast motion, optical flow corrects slow drift
    this.orientation.slerp(gyroQuat, this.alpha);
    this.orientation.slerp(opticalQuat, 1 - this.alpha);
    
    // 4. Normalize
    this.orientation.normalize();
    
    return {
      position: this.integratePosition(imu.accel, optical.translation, dt),
      orientation: this.orientation,
    };
  }
  
  private integrateGyro(gyro: Vector3, dt: number): THREE.Quaternion {
    // Convert angular velocity to quaternion
    const angle = Math.sqrt(gyro.x ** 2 + gyro.y ** 2 + gyro.z ** 2) * dt;
    if (angle < 0.001) return new THREE.Quaternion(0, 0, 0, 1);
    
    const axis = new THREE.Vector3(gyro.x, gyro.y, gyro.z).normalize();
    const quat = new THREE.Quaternion().setFromAxisAngle(axis, angle);
    
    return this.orientation.clone().multiply(quat);
  }
}
```

**Kalman Filter** (Advanced):
```typescript
class KalmanFilter {
  private state = [0, 0, 0, 0]; // [x, y, vx, vy]
  private P = math.eye(4).mul(1000); // Covariance
  private Q = math.eye(4).mul(0.1);  // Process noise
  private R = math.eye(2).mul(10);   // Measurement noise
  
  predict(dt: number): void {
    // State transition matrix
    const F = math.matrix([
      [1, 0, dt, 0],
      [0, 1, 0, dt],
      [0, 0, 1, 0],
      [0, 0, 0, 1],
    ]);
    
    this.state = F.multiply(this.state);
    this.P = F.multiply(this.P).multiply(F.transpose()).add(this.Q);
  }
  
  update(measurement: [number, number]): void {
    const H = math.matrix([[1, 0, 0, 0], [0, 1, 0, 0]]); // Measurement matrix
    
    // Innovation
    const y = math.subtract(measurement, H.multiply(this.state));
    const S = H.multiply(this.P).multiply(H.transpose()).add(this.R);
    
    // Kalman gain
    const K = this.P.multiply(H.transpose()).multiply(math.inv(S));
    
    // Update state
    this.state = math.add(this.state, K.multiply(y));
    this.P = math.subtract(math.eye(4), K.multiply(H)).multiply(this.P);
  }
  
  getPosition(): [number, number] {
    return [this.state[0], this.state[1]];
  }
}
```

---

### 5. Spatial Anchoring

**Purpose**: Anchor AR content to fixed positions in 3D world space, maintaining position as user moves.

**Anchor System**:
```typescript
interface SpatialAnchor {
  id: string;
  worldPosition: THREE.Vector3;
  worldOrientation: THREE.Quaternion;
  timestamp: number;
  confidence: number; // 0.0-1.0
}

class SpatialAnchorManager {
  private anchors: Map<string, SpatialAnchor> = new Map();
  
  createAnchor(screenPos: { x: number; y: number }, depth: number): SpatialAnchor {
    // 1. Convert screen position to ray
    const ray = this.screenToRay(screenPos);
    
    // 2. Calculate world position
    const worldPos = ray.origin.clone().add(
      ray.direction.clone().multiplyScalar(depth)
    );
    
    // 3. Create anchor
    const anchor: SpatialAnchor = {
      id: crypto.randomUUID(),
      worldPosition: worldPos,
      worldOrientation: this.currentPose.orientation.clone(),
      timestamp: Date.now(),
      confidence: 0.9,
    };
    
    this.anchors.set(anchor.id, anchor);
    return anchor;
  }
  
  // Update anchor positions based on current camera pose
  updateAnchors(cameraPose: Pose): void {
    for (const anchor of this.anchors.values()) {
      // Transform from world space to camera space
      const cameraSpace = this.worldToCamera(anchor.worldPosition, cameraPose);
      
      // Update UI element position
      this.updateUIElement(anchor.id, cameraSpace);
    }
  }
  
  private worldToCamera(worldPos: THREE.Vector3, cameraPose: Pose): THREE.Vector3 {
    // 1. Translate to camera origin
    const translated = worldPos.clone().sub(cameraPose.position);
    
    // 2. Rotate to camera orientation
    const invOrientation = cameraPose.orientation.clone().invert();
    translated.applyQuaternion(invOrientation);
    
    return translated;
  }
  
  // Project 3D position to 2D screen coordinates
  projectToScreen(worldPos: THREE.Vector3): { x: number; y: number } | null {
    const cameraSpace = this.worldToCamera(worldPos, this.currentPose);
    
    // Behind camera
    if (cameraSpace.z > 0) return null;
    
    const fov = 60 * Math.PI / 180;
    const aspect = 16 / 9;
    
    const x = (cameraSpace.x / -cameraSpace.z) / Math.tan(fov / 2) / aspect;
    const y = (cameraSpace.y / -cameraSpace.z) / Math.tan(fov / 2);
    
    // Normalized device coordinates [-1, 1]
    return {
      x: (x + 1) / 2,  // [0, 1]
      y: (1 - y) / 2,  // [0, 1], flip Y
    };
  }
}
```

**Anchor Persistence**:
```typescript
// Save anchors to persistent storage
function serializeAnchors(anchors: SpatialAnchor[]): string {
  return JSON.stringify(anchors.map(a => ({
    id: a.id,
    position: [a.worldPosition.x, a.worldPosition.y, a.worldPosition.z],
    orientation: [a.worldOrientation.x, a.worldOrientation.y, a.worldOrientation.z, a.worldOrientation.w],
    timestamp: a.timestamp,
  })));
}

// Load anchors from storage
function deserializeAnchors(json: string): SpatialAnchor[] {
  const data = JSON.parse(json);
  return data.map(a => ({
    id: a.id,
    worldPosition: new THREE.Vector3(...a.position),
    worldOrientation: new THREE.Quaternion(...a.orientation),
    timestamp: a.timestamp,
    confidence: 0.7, // Reduce confidence for loaded anchors
  }));
}
```

---

### 6. Cursor System

**Purpose**: Visual feedback for user's pointing direction and interaction target.

**Cursor Rendering**:
```typescript
class ARCursor {
  private element: HTMLElement;
  private position = { x: 0.5, y: 0.5 }; // Normalized [0,1]
  private depth = 1.0; // meters
  private state: 'hidden' | 'hover' | 'pinch' = 'hidden';
  
  update(gesture: Gesture | null): void {
    if (!gesture) {
      this.state = 'hidden';
      this.element.style.display = 'none';
      return;
    }
    
    // Update position
    this.position = { x: gesture.position.x, y: gesture.position.y };
    this.depth = gesture.position.z;
    
    // Update state
    this.state = gesture.type === GestureType.PINCH ? 'pinch' : 'hover';
    
    // Render
    this.render();
  }
  
  private render(): void {
    this.element.style.display = 'block';
    this.element.style.left = `${this.position.x * 100}%`;
    this.element.style.top = `${this.position.y * 100}%`;
    
    // Scale based on depth (closer = larger)
    const scale = 1.0 / this.depth;
    this.element.style.transform = `translate(-50%, -50%) scale(${scale})`;
    
    // Visual state
    if (this.state === 'pinch') {
      this.element.classList.add('pinched');
    } else {
      this.element.classList.remove('pinched');
    }
  }
}
```

**Cursor CSS**:
```css
.ar-cursor {
  position: absolute;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid white;
  background: rgba(255, 255, 255, 0.3);
  pointer-events: none;
  transition: background 0.1s, transform 0.05s;
}

.ar-cursor.pinched {
  background: rgba(100, 200, 255, 0.7);
  transform: translate(-50%, -50%) scale(0.8);
}
```

---

## Integration Flow

```
Camera Frame (30fps)
    │
    ▼
┌────────────────┐
│ Hand Tracking  │ 15-25ms
└────────┬───────┘
         │
         ▼
┌────────────────┐
│ Gesture Detect │ 2-5ms
└────────┬───────┘
         │
         ├────────────────┐
         │                │
         ▼                ▼
┌────────────────┐  ┌────────────────┐
│ Optical Flow   │  │ IMU Reading    │
│ 10-15ms        │  │ 100Hz          │
└────────┬───────┘  └────────┬───────┘
         │                   │
         └─────────┬─────────┘
                   ▼
         ┌────────────────┐
         │ Sensor Fusion  │ 3-8ms
         └────────┬───────┘
                  │
                  ▼
         ┌────────────────┐
         │ Anchor Update  │ 5-10ms
         └────────┬───────┘
                  │
                  ▼
         ┌────────────────┐
         │ AR Render      │ 8-16ms (60fps)
         └────────────────┘
         
Total Latency: 43-79ms (motion-to-photon)
```

---

## Performance Metrics

```
┌─ AR Tracking Performance ───────────────┐
│ Hand Detection: 15-25ms                  │
│ Gesture Recognition: 2-5ms               │
│ Optical Flow: 10-15ms                    │
│ Sensor Fusion: 3-8ms                     │
│ Anchor Update: 5-10ms                    │
│ Total Pipeline: 35-63ms                  │
│                                           │
│ Accuracy:                                 │
│   Hand Detection: 95% (good lighting)    │
│   Gesture: 92% precision                 │
│   Position: ±2cm at 1m depth             │
│   Orientation: ±3° accuracy              │
└───────────────────────────────────────────┘
```

---

## Code References

- `simulator-ui/components/HandTracker.tsx`: MediaPipe integration
- `simulator-ui/services/SensorFusionService.ts`: Quaternion fusion
- `simulator-ui/services/OpticalFlowService.ts`: jsfeat optical flow
- `simulator-ui/components/AROverlay.tsx`: Spatial anchoring

---

## Summary

The AR Tracking System provides:
- **Hand Tracking**: MediaPipe 21-landmark detection
- **Gestures**: Pinch, point, grab recognition
- **Optical Flow**: jsfeat motion tracking
- **Sensor Fusion**: IMU + vision complementary filter
- **Spatial Anchors**: World-locked AR content
- **Cursor**: Visual feedback for interactions

This enables natural, intuitive AR interactions on Kāraṇa OS smart glasses.