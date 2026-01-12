# Layer 7: Interface Layer

## Overview

The Interface Layer is the user-facing layer of Kāraṇa OS, providing voice UI, HUD rendering, gesture interactions, gaze tracking, and AR spatial rendering. It translates user inputs into system commands and renders outputs in immersive AR.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      LAYER 7: INTERFACE LAYER                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Voice UI    │  │  Gesture UI  │  │   Gaze UI    │  │   HUD      │ │
│  │  Speech I/O  │  │  Hand track  │  │  Eye track   │  │  Overlay   │ │
│  │  Commands    │  │  Pinch/swipe │  │  Dwell click │  │  2D/3D     │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬─────┘ │
│         │                 │                  │                 │        │
│         └─────────────────┴──────────────────┴─────────────────┘        │
│                               │                                          │
│                               ▼                                          │
│                   ┌────────────────────────┐                            │
│                   │  Interaction Manager   │                            │
│                   │  Fuse multimodal input │                            │
│                   └───────────┬────────────┘                            │
│                               │                                          │
│                               ▼                                          │
│                   ┌────────────────────────┐                            │
│                   │     AR Renderer        │                            │
│                   │  WebGL/Three.js        │                            │
│                   │  60-90 FPS             │                            │
│                   └────────────────────────┘                            │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Voice UI

**Purpose**: Enable hands-free voice control and natural conversation.

**Voice Input** (Speech Recognition):
```typescript
class VoiceInput {
  private recognition: SpeechRecognition;
  private isListening = false;
  
  initialize(): void {
    this.recognition = new (window.SpeechRecognition || window.webkitSpeechRecognition)();
    this.recognition.continuous = true;
    this.recognition.interimResults = true;
    this.recognition.lang = 'en-US';
    
    this.recognition.onresult = (event) => {
      const result = event.results[event.results.length - 1];
      const transcript = result[0].transcript;
      
      if (result.isFinal) {
        this.onFinalTranscript(transcript);
      } else {
        this.onInterimTranscript(transcript);
      }
    };
  }
  
  startListening(): void {
    if (!this.isListening) {
      this.recognition.start();
      this.isListening = true;
    }
  }
  
  stopListening(): void {
    if (this.isListening) {
      this.recognition.stop();
      this.isListening = false;
    }
  }
  
  private onFinalTranscript(text: string): void {
    // Send to AI Engine (Layer 6) for intent classification
    this.aiEngine.processInput(text);
  }
}
```

**Voice Output** (Text-to-Speech):
```typescript
class VoiceOutput {
  private synth: SpeechSynthesis;
  private voice: SpeechSynthesisVoice;
  
  initialize(): void {
    this.synth = window.speechSynthesis;
    
    // Select voice
    const voices = this.synth.getVoices();
    this.voice = voices.find(v => v.lang === 'en-US' && v.name.includes('Female')) || voices[0];
  }
  
  speak(text: string, options?: SpeakOptions): void {
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.voice = this.voice;
    utterance.rate = options?.rate || 1.0;
    utterance.pitch = options?.pitch || 1.0;
    utterance.volume = options?.volume || 0.8;
    
    // Add to queue
    this.synth.speak(utterance);
  }
  
  // Interrupt current speech
  stop(): void {
    this.synth.cancel();
  }
}
```

**Wake Word Detection**:
```typescript
class WakeWordDetector {
  private model: PorcupineWorker; // Picovoice Porcupine
  private wakeWords = ['Hey Karana', 'Karana'];
  
  async initialize(): Promise<void> {
    this.model = await Porcupine.PorcupineWorker.create(
      ACCESS_KEY,
      [Porcupine.BuiltInKeyword.JARVIS], // Placeholder, custom model in production
      (keywordIndex) => this.onWakeWord(keywordIndex)
    );
  }
  
  private onWakeWord(index: number): void {
    console.log(`Wake word detected: ${this.wakeWords[index]}`);
    
    // Activate voice input
    this.voiceInput.startListening();
    
    // Show listening indicator
    this.hud.showListeningIndicator();
  }
}
```

---

### 2. Gesture UI

**Purpose**: Enable natural hand gesture interactions (pinch, swipe, grab).

**Gesture Handler**:
```typescript
class GestureHandler {
  private currentGesture: Gesture | null = null;
  private gestureStart: { x: number; y: number } | null = null;
  
  handleGesture(gesture: Gesture, event: GestureEvent): void {
    switch (event.type) {
      case 'HOVER_START':
        this.onHoverStart(gesture, event.position);
        break;
        
      case 'CLICK':
        this.onClick(gesture, event.position);
        break;
        
      case 'DRAG_START':
        this.onDragStart(gesture, event.position);
        break;
        
      case 'DRAG_MOVE':
        this.onDragMove(gesture, event.position);
        break;
        
      case 'DRAG_END':
        this.onDragEnd(gesture);
        break;
        
      case 'SWIPE':
        this.onSwipe(gesture, event.direction);
        break;
    }
  }
  
  private onClick(gesture: Gesture, position: Vector2): void {
    // Raycast to find UI element
    const element = this.hud.raycast(position);
    
    if (element) {
      element.onClick();
      this.playHapticFeedback('click');
    }
  }
  
  private onDragMove(gesture: Gesture, position: Vector2): void {
    if (this.gestureStart) {
      const delta = {
        x: position.x - this.gestureStart.x,
        y: position.y - this.gestureStart.y,
      };
      
      // Update dragged element
      this.draggedElement?.onDrag(delta);
    }
  }
  
  private playHapticFeedback(type: string): void {
    if ('vibrate' in navigator) {
      const patterns = {
        'click': [10],
        'success': [10, 20, 10],
        'error': [50, 30, 50],
      };
      
      navigator.vibrate(patterns[type] || [10]);
    }
  }
}
```

**Gesture Commands**:
```typescript
const gestureCommands = {
  'pinch': {
    description: 'Select/click',
    action: 'click',
  },
  'pinch_hold': {
    description: 'Drag',
    action: 'drag',
  },
  'swipe_left': {
    description: 'Go back',
    action: 'navigate_back',
  },
  'swipe_right': {
    description: 'Go forward',
    action: 'navigate_forward',
  },
  'palm_open': {
    description: 'Home screen',
    action: 'show_home',
  },
  'thumbs_up': {
    description: 'Approve/like',
    action: 'approve',
  },
};
```

---

### 3. Gaze UI

**Purpose**: Enable eye-gaze interaction for cursor control and selection.

**Gaze Tracker**:
```typescript
class GazeTracker {
  private calibrated = false;
  private gazeCursor = { x: 0.5, y: 0.5 }; // Normalized [0,1]
  
  async calibrate(): Promise<void> {
    // Show calibration dots
    const points = [
      { x: 0.1, y: 0.1 },
      { x: 0.9, y: 0.1 },
      { x: 0.5, y: 0.5 },
      { x: 0.1, y: 0.9 },
      { x: 0.9, y: 0.9 },
    ];
    
    for (const point of points) {
      await this.calibratePoint(point);
    }
    
    this.calibrated = true;
  }
  
  update(frame: VideoFrame): void {
    if (!this.calibrated) return;
    
    // Detect eyes
    const eyes = this.detectEyes(frame);
    
    if (eyes) {
      // Estimate gaze direction
      const gazeDir = this.estimateGaze(eyes);
      
      // Project to screen coordinates
      this.gazeCursor = this.projectGaze(gazeDir);
      
      // Update cursor
      this.hud.updateGazeCursor(this.gazeCursor);
    }
  }
  
  private estimateGaze(eyes: EyeFeatures): Vector3 {
    // Simplified model: gaze direction from pupil position
    const leftGaze = this.pupilToGaze(eyes.left);
    const rightGaze = this.pupilToGaze(eyes.right);
    
    // Average both eyes
    return leftGaze.add(rightGaze).divideScalar(2);
  }
}
```

**Dwell Selection** (Look at element for 800ms to select):
```typescript
class DwellSelector {
  private dwellTarget: UIElement | null = null;
  private dwellStartTime = 0;
  private dwellDuration = 800; // ms
  
  update(gazeCursor: Vector2): void {
    const element = this.hud.raycast(gazeCursor);
    
    if (element === this.dwellTarget) {
      // Continue dwelling
      const elapsed = Date.now() - this.dwellStartTime;
      
      if (elapsed >= this.dwellDuration) {
        // Trigger selection
        element.onClick();
        this.dwellTarget = null;
        
        // Visual feedback
        this.showSelectionAnimation(element);
      } else {
        // Show progress
        this.showDwellProgress(element, elapsed / this.dwellDuration);
      }
    } else {
      // New target
      this.dwellTarget = element;
      this.dwellStartTime = Date.now();
    }
  }
  
  private showDwellProgress(element: UIElement, progress: number): void {
    // Draw circular progress around element
    this.hud.drawCircle(element.position, element.radius, progress);
  }
}
```

---

### 4. HUD (Heads-Up Display)

**Purpose**: Render 2D overlay UI and 3D AR content.

**HUD Renderer**:
```typescript
class HUDRenderer {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private elements: Map<string, UIElement> = new Map();
  
  initialize(canvas: HTMLCanvasElement): void {
    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(60, 16/9, 0.1, 1000);
    this.renderer = new THREE.WebGLRenderer({ canvas, alpha: true });
    this.renderer.setSize(1280, 720);
  }
  
  render(cameraPose: Pose): void {
    // Update camera transform
    this.camera.position.copy(cameraPose.position);
    this.camera.quaternion.copy(cameraPose.orientation);
    
    // Render 3D elements
    this.renderer.render(this.scene, this.camera);
    
    // Render 2D overlay
    this.render2DOverlay();
  }
  
  private render2DOverlay(): void {
    const ctx = this.overlay2DContext;
    ctx.clearRect(0, 0, 1280, 720);
    
    // Draw status bar
    this.drawStatusBar(ctx);
    
    // Draw notifications
    this.drawNotifications(ctx);
    
    // Draw cursor
    this.drawCursor(ctx);
  }
  
  addElement(id: string, element: UIElement): void {
    this.elements.set(id, element);
    
    if (element.is3D) {
      this.scene.add(element.object3D);
    }
  }
  
  removeElement(id: string): void {
    const element = this.elements.get(id);
    if (element && element.is3D) {
      this.scene.remove(element.object3D);
    }
    this.elements.delete(id);
  }
}
```

**UI Elements**:
```typescript
interface UIElement {
  id: string;
  type: 'button' | 'label' | 'card' | 'list' | 'menu';
  position: Vector3;      // 3D position or 2D screen coords
  size: { width: number; height: number };
  is3D: boolean;
  object3D?: THREE.Object3D;
  onClick?: () => void;
  onHover?: () => void;
}

class Button implements UIElement {
  id: string;
  type = 'button' as const;
  position: Vector3;
  size = { width: 100, height: 40 };
  is3D = false;
  
  constructor(
    public text: string,
    position: Vector3,
    public onClick: () => void
  ) {
    this.id = crypto.randomUUID();
    this.position = position;
  }
  
  render(ctx: CanvasRenderingContext2D): void {
    ctx.fillStyle = '#007bff';
    ctx.fillRect(
      this.position.x - this.size.width / 2,
      this.position.y - this.size.height / 2,
      this.size.width,
      this.size.height
    );
    
    ctx.fillStyle = 'white';
    ctx.font = '16px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(this.text, this.position.x, this.position.y + 5);
  }
}
```

**3D UI Elements**:
```typescript
class ARCard {
  private mesh: THREE.Mesh;
  
  constructor(content: CardContent, worldPos: Vector3) {
    // Create plane geometry
    const geometry = new THREE.PlaneGeometry(0.3, 0.4); // 30cm x 40cm
    
    // Create canvas texture
    const canvas = this.createCanvasTexture(content);
    const texture = new THREE.CanvasTexture(canvas);
    
    const material = new THREE.MeshBasicMaterial({
      map: texture,
      transparent: true,
      side: THREE.DoubleSide,
    });
    
    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.position.copy(worldPos);
    
    // Always face camera (billboard)
    this.mesh.onBeforeRender = (renderer, scene, camera) => {
      this.mesh.quaternion.copy(camera.quaternion);
    };
  }
  
  private createCanvasTexture(content: CardContent): HTMLCanvasElement {
    const canvas = document.createElement('canvas');
    canvas.width = 512;
    canvas.height = 682;
    
    const ctx = canvas.getContext('2d')!;
    
    // Background
    ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
    ctx.fillRect(0, 0, 512, 682);
    
    // Title
    ctx.fillStyle = 'white';
    ctx.font = 'bold 32px Arial';
    ctx.fillText(content.title, 20, 50);
    
    // Content
    ctx.font = '24px Arial';
    ctx.fillText(content.text, 20, 100, 472);
    
    return canvas;
  }
}
```

---

### 5. AR Renderer

**Purpose**: Render spatial AR content anchored in 3D world.

**AR Scene Manager**:
```typescript
class ARSceneManager {
  private anchors: Map<string, SpatialAnchor> = new Map();
  private arObjects: Map<string, THREE.Object3D> = new Map();
  
  addARObject(anchorId: string, object: THREE.Object3D): void {
    const anchor = this.anchors.get(anchorId);
    if (!anchor) {
      throw new Error(`Anchor not found: ${anchorId}`);
    }
    
    // Position object at anchor
    object.position.copy(anchor.worldPosition);
    object.quaternion.copy(anchor.worldOrientation);
    
    this.arObjects.set(anchorId, object);
    this.scene.add(object);
  }
  
  update(cameraPose: Pose): void {
    // Update all AR objects based on camera movement
    for (const [anchorId, object] of this.arObjects) {
      const anchor = this.anchors.get(anchorId);
      if (!anchor) continue;
      
      // Transform to camera space
      const cameraSpacePos = anchor.worldPosition.clone().sub(cameraPose.position);
      cameraSpacePos.applyQuaternion(cameraPose.orientation.clone().invert());
      
      // Check if visible
      if (this.isVisible(cameraSpacePos)) {
        object.visible = true;
        
        // Update occlusion (if depth map available)
        this.updateOcclusion(object, cameraSpacePos);
      } else {
        object.visible = false;
      }
    }
  }
  
  private isVisible(cameraSpacePos: Vector3): boolean {
    // Check if in front of camera
    if (cameraSpacePos.z > 0) return false;
    
    // Check if in FOV
    const fov = 60 * Math.PI / 180;
    const angle = Math.atan2(cameraSpacePos.x, -cameraSpacePos.z);
    
    return Math.abs(angle) < fov / 2;
  }
}
```

**Occlusion Handling**:
```typescript
class OcclusionRenderer {
  private depthTexture: THREE.DataTexture;
  
  updateOcclusion(object: THREE.Object3D, depthMap: DepthMap): void {
    // Create depth texture
    this.depthTexture = new THREE.DataTexture(
      depthMap.data,
      depthMap.width,
      depthMap.height,
      THREE.RedFormat,
      THREE.FloatType
    );
    
    // Custom shader for depth testing
    object.material = new THREE.ShaderMaterial({
      uniforms: {
        depthMap: { value: this.depthTexture },
        objectDepth: { value: object.position.length() },
      },
      vertexShader: `
        varying vec2 vUv;
        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform sampler2D depthMap;
        uniform float objectDepth;
        varying vec2 vUv;
        
        void main() {
          float sceneDepth = texture2D(depthMap, vUv).r;
          
          if (objectDepth > sceneDepth) {
            discard; // Behind real-world object
          }
          
          gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
      `,
    });
  }
}
```

---

## Interaction Flow

```
User Input
   │
   ├─ Voice: "Hey Karana, navigate home"
   │     │
   │     ▼
   │  Voice Input → Speech Recognition → AI Engine (Layer 6)
   │
   ├─ Gesture: Pinch + drag
   │     │
   │     ▼
   │  Hand Tracker → Gesture Recognition → Interaction Manager
   │
   └─ Gaze: Look at button for 800ms
         │
         ▼
      Gaze Tracker → Dwell Selector → Button onClick
         
                    ▼
            Interaction Manager
                    │
                    ├─ Update HUD
                    ├─ Render AR objects
                    └─ Play haptic feedback
```

---

## Performance Metrics

```
┌─ Interface Layer Performance ───────────┐
│ Voice Recognition: 50-200ms              │
│ Voice Synthesis: 100-300ms               │
│ Gesture Processing: 2-5ms                │
│ Gaze Tracking: 10-20ms                   │
│ HUD Rendering: 11-16ms (60-90 FPS)      │
│ AR Rendering: 11-16ms (60-90 FPS)       │
│                                           │
│ Input Latency:                            │
│   Voice: 50-200ms                         │
│   Gesture: 35-79ms (motion-to-photon)    │
│   Gaze: 10-20ms                          │
└───────────────────────────────────────────┘
```

---

## Future Development

### Phase 1: Advanced Gaze (Q1 2026)
- Foveated rendering (render center sharper)
- Predictive gaze (anticipate where user will look)
- Smooth pursuit tracking

### Phase 2: Haptics (Q2 2026)
- Spatial audio haptics
- Ultrasonic haptic feedback
- Mid-air tactile sensation

### Phase 3: Neural Interface (Q3 2026)
- EMG for silent speech
- EEG for thought control
- BCIs for direct neural input

### Phase 4: Holographic Display (Q4 2026)
- Volumetric display
- Light field rendering
- True 3D holography

---

## Code References

- `simulator-ui/components/HUD.tsx`: HUD rendering
- `simulator-ui/components/AROverlay.tsx`: AR scene management
- `simulator-ui/services/gestureHandler.ts`: Gesture processing

---

## Summary

Layer 7 provides:
- **Voice UI**: Speech recognition + synthesis
- **Gesture UI**: Hand tracking for pinch/swipe/grab
- **Gaze UI**: Eye tracking + dwell selection
- **HUD**: 2D overlay at 60-90 FPS
- **AR Renderer**: 3D spatial content with occlusion

This layer enables natural multimodal interaction with Kāraṇa OS.