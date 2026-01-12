/**
 * Visual Intelligence Service
 * 
 * Handles:
 * - Eye tracking and gaze detection
 * - Object recognition from camera feed
 * - Scene understanding and context analysis
 * - Intelligent feedback on what user is viewing
 * - Real-time visual assistance
 */

import { systemState } from './systemState';
import { weatherService } from './realTimeServices';

// ==================== TYPE DEFINITIONS ====================

interface EyeTrackingData {
  gazeX: number; // 0-1 normalized horizontal position
  gazeY: number; // 0-1 normalized vertical position
  fixationDuration: number; // milliseconds
  pupilDilation: number; // relative size
  blinkRate: number; // blinks per minute
  focusIntensity: number; // 0-1, how focused the user is
  isFixated: boolean; // true if gaze is stable
}

interface VisualObject {
  id: string;
  type: 'text' | 'object' | 'person' | 'scene' | 'product' | 'food' | 'animal' | 'vehicle' | 'building' | 'nature' | 'unknown';
  label: string;
  confidence: number; // 0-1
  boundingBox: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  attributes: Record<string, any>;
  recognizedAt: number;
}

interface SceneContext {
  environment: 'indoor' | 'outdoor' | 'vehicle' | 'unknown';
  lighting: 'bright' | 'normal' | 'dim' | 'dark';
  activityType: 'working' | 'shopping' | 'eating' | 'traveling' | 'exercising' | 'relaxing' | 'social' | 'unknown';
  detectedObjects: VisualObject[];
  dominantColors: string[];
  sceneComplexity: number; // 0-1, how busy/cluttered the scene is
  timeContext: string;
}

interface VisualFeedback {
  objectInfo: string;
  intelligentInsight: string;
  actionSuggestions: string[];
  warnings?: string[];
  relevantData?: any;
  confidence: number;
  reasoning: string;
}

interface GazeHistory {
  timestamp: number;
  gazeX: number;
  gazeY: number;
  focusedObject?: VisualObject;
  duration: number;
}

// ==================== EYE TRACKING SYSTEM ====================

class EyeTrackingSystem {
  private gazeHistory: GazeHistory[] = [];
  private currentGaze: EyeTrackingData | null = null;
  private fixationThreshold = 200; // ms to consider as fixation
  private maxHistorySize = 100;
  
  // Simulated eye tracking (in real hardware, this would use actual eye-tracking sensors)
  simulateEyeTracking(cameraFeed: HTMLVideoElement | null): EyeTrackingData {
    // In production, this would use:
    // - IR cameras for pupil detection
    // - Machine learning models for gaze estimation
    // - Calibration data for accuracy
    
    // For simulation, generate realistic eye tracking data
    const baseX = 0.5 + (Math.random() - 0.5) * 0.3;
    const baseY = 0.5 + (Math.random() - 0.5) * 0.3;
    
    // Simulate natural eye movement patterns
    const jitterX = (Math.random() - 0.5) * 0.05; // Small eye movements
    const jitterY = (Math.random() - 0.5) * 0.05;
    
    const gazeX = Math.max(0, Math.min(1, baseX + jitterX));
    const gazeY = Math.max(0, Math.min(1, baseY + jitterY));
    
    // Calculate fixation based on gaze stability
    const isFixated = this.isGazeStable(gazeX, gazeY);
    const fixationDuration = isFixated ? this.getFixationDuration() : 0;
    
    // Pupil dilation correlates with cognitive load and interest
    const pupilDilation = 0.5 + (Math.random() * 0.3);
    
    // Normal blink rate: 15-20 per minute
    const blinkRate = 15 + Math.random() * 5;
    
    // Focus intensity based on fixation and pupil dilation
    const focusIntensity = isFixated ? Math.min(1, pupilDilation + 0.3) : 0.3;
    
    this.currentGaze = {
      gazeX,
      gazeY,
      fixationDuration,
      pupilDilation,
      blinkRate,
      focusIntensity,
      isFixated
    };
    
    return this.currentGaze;
  }
  
  private isGazeStable(x: number, y: number): boolean {
    if (this.gazeHistory.length === 0) return false;
    
    const recent = this.gazeHistory.slice(-5);
    const avgX = recent.reduce((sum, h) => sum + h.gazeX, 0) / recent.length;
    const avgY = recent.reduce((sum, h) => sum + h.gazeY, 0) / recent.length;
    
    const distanceFromAverage = Math.sqrt(
      Math.pow(x - avgX, 2) + Math.pow(y - avgY, 2)
    );
    
    return distanceFromAverage < 0.05; // Within 5% of screen
  }
  
  private getFixationDuration(): number {
    if (this.gazeHistory.length === 0) return 0;
    
    const now = Date.now();
    let duration = 0;
    
    for (let i = this.gazeHistory.length - 1; i >= 0; i--) {
      const entry = this.gazeHistory[i];
      if (now - entry.timestamp > 2000) break; // Only look at last 2 seconds
      duration += entry.duration;
    }
    
    return duration;
  }
  
  recordGaze(gaze: EyeTrackingData, focusedObject?: VisualObject): void {
    this.gazeHistory.push({
      timestamp: Date.now(),
      gazeX: gaze.gazeX,
      gazeY: gaze.gazeY,
      focusedObject,
      duration: 16 // ~60fps frame duration
    });
    
    // Keep history size manageable
    if (this.gazeHistory.length > this.maxHistorySize) {
      this.gazeHistory.shift();
    }
  }
  
  getCurrentGaze(): EyeTrackingData | null {
    return this.currentGaze;
  }
  
  getGazeHistory(): GazeHistory[] {
    return this.gazeHistory;
  }
  
  // Analyze what user has been looking at
  getAttentionPatterns(): {
    mostViewedObjects: { object: VisualObject; totalDuration: number }[];
    averageFocusIntensity: number;
    totalFixationTime: number;
    attentionSpan: number;
  } {
    const objectDurations = new Map<string, { object: VisualObject; duration: number }>();
    let totalFocusIntensity = 0;
    let focusCount = 0;
    let totalFixationTime = 0;
    
    this.gazeHistory.forEach(entry => {
      if (entry.focusedObject) {
        const existing = objectDurations.get(entry.focusedObject.id);
        if (existing) {
          existing.duration += entry.duration;
        } else {
          objectDurations.set(entry.focusedObject.id, {
            object: entry.focusedObject,
            duration: entry.duration
          });
        }
        totalFixationTime += entry.duration;
      }
    });
    
    const mostViewedObjects = Array.from(objectDurations.values())
      .map(item => ({ object: item.object, totalDuration: item.duration }))
      .sort((a, b) => b.totalDuration - a.totalDuration);
    
    const averageFocusIntensity = focusCount > 0 ? totalFocusIntensity / focusCount : 0;
    const attentionSpan = totalFixationTime / Math.max(1, this.gazeHistory.length);
    
    return {
      mostViewedObjects,
      averageFocusIntensity,
      totalFixationTime,
      attentionSpan
    };
  }
}

// ==================== OBJECT RECOGNITION SYSTEM ====================

class ObjectRecognitionSystem {
  private recognizedObjects: Map<string, VisualObject> = new Map();
  private recognitionConfidenceThreshold = 0.6;
  
  // In production, this would use:
  // - TensorFlow.js or ONNX models
  // - Pre-trained object detection models (YOLO, SSD, etc.)
  // - Edge computing for real-time processing
  
  async recognizeObjects(frame: ImageData | HTMLVideoElement | null): Promise<VisualObject[]> {
    if (!frame) {
      return this.generateSimulatedObjects();
    }
    
    // Simulated recognition for now
    return this.generateSimulatedObjects();
  }
  
  private generateSimulatedObjects(): VisualObject[] {
    // Simulate detecting objects based on context
    const context = this.inferContextFromSystemState();
    
    const objects: VisualObject[] = [];
    
    // Generate contextually relevant objects
    if (context.activityType === 'shopping') {
      objects.push(
        this.createObject('product', 'Smartphone', 0.92, { price: '$799', brand: 'TechBrand' }),
        this.createObject('product', 'Laptop', 0.88, { price: '$1299', brand: 'ComputeCorp' }),
        this.createObject('text', 'Sale Sign', 0.95, { text: '30% OFF' })
      );
    } else if (context.activityType === 'eating') {
      objects.push(
        this.createObject('food', 'Pizza', 0.89, { cuisine: 'Italian', calories: 285 }),
        this.createObject('food', 'Salad', 0.85, { type: 'Caesar', healthy: true }),
        this.createObject('text', 'Menu', 0.93, { items: 15 })
      );
    } else if (context.activityType === 'working') {
      objects.push(
        this.createObject('object', 'Computer Monitor', 0.94, { size: '27 inch' }),
        this.createObject('text', 'Code Editor', 0.91, { language: 'TypeScript' }),
        this.createObject('object', 'Coffee Mug', 0.87, { temperature: 'warm' })
      );
    } else {
      // Generic objects
      objects.push(
        this.createObject('text', 'Text Document', 0.90, { wordCount: 523 }),
        this.createObject('person', 'Person', 0.85, { distance: '2m' }),
        this.createObject('scene', 'Indoor Space', 0.88, { lighting: 'natural' })
      );
    }
    
    // Store recognized objects
    objects.forEach(obj => {
      this.recognizedObjects.set(obj.id, obj);
    });
    
    return objects;
  }
  
  private createObject(
    type: VisualObject['type'],
    label: string,
    confidence: number,
    attributes: Record<string, any> = {}
  ): VisualObject {
    return {
      id: `${type}-${label.toLowerCase().replace(/\s/g, '-')}-${Date.now()}`,
      type,
      label,
      confidence,
      boundingBox: {
        x: Math.random() * 0.7 + 0.15,
        y: Math.random() * 0.7 + 0.15,
        width: 0.1 + Math.random() * 0.2,
        height: 0.1 + Math.random() * 0.2
      },
      attributes,
      recognizedAt: Date.now()
    };
  }
  
  private inferContextFromSystemState(): SceneContext {
    const hour = new Date().getHours();
    let activityType: SceneContext['activityType'] = 'unknown';
    
    // Infer activity from time and system state
    if (hour >= 9 && hour < 17) {
      activityType = 'working';
    } else if (hour >= 12 && hour < 14) {
      activityType = 'eating';
    } else if (hour >= 18 && hour < 20) {
      activityType = 'eating';
    } else if (hour >= 7 && hour < 9) {
      activityType = 'exercising';
    }
    
    return {
      environment: 'indoor',
      lighting: 'normal',
      activityType,
      detectedObjects: [],
      dominantColors: ['#FFFFFF', '#000000'],
      sceneComplexity: 0.5,
      timeContext: `${hour}:00`
    };
  }
  
  findObjectAtGazePoint(objects: VisualObject[], gazeX: number, gazeY: number): VisualObject | null {
    // Find object whose bounding box contains the gaze point
    for (const obj of objects) {
      const { x, y, width, height } = obj.boundingBox;
      if (
        gazeX >= x &&
        gazeX <= x + width &&
        gazeY >= y &&
        gazeY <= y + height
      ) {
        return obj;
      }
    }
    return null;
  }
}

// ==================== SCENE UNDERSTANDING SYSTEM ====================

class SceneUnderstandingSystem {
  
  async analyzeScene(objects: VisualObject[], eyeTracking: EyeTrackingData): Promise<SceneContext> {
    const hour = new Date().getHours();
    
    // Infer environment from objects
    const environment = this.inferEnvironment(objects);
    
    // Infer lighting from time and detected brightness
    const lighting = this.inferLighting(hour, objects);
    
    // Infer activity from objects and time
    const activityType = this.inferActivity(objects, hour);
    
    // Analyze scene complexity
    const sceneComplexity = objects.length / 20; // More objects = more complex
    
    return {
      environment,
      lighting,
      activityType,
      detectedObjects: objects,
      dominantColors: this.extractDominantColors(objects),
      sceneComplexity: Math.min(1, sceneComplexity),
      timeContext: new Date().toLocaleTimeString()
    };
  }
  
  private inferEnvironment(objects: VisualObject[]): SceneContext['environment'] {
    const objectTypes = objects.map(o => o.label.toLowerCase());
    
    if (objectTypes.some(t => t.includes('tree') || t.includes('sky') || t.includes('grass'))) {
      return 'outdoor';
    }
    if (objectTypes.some(t => t.includes('steering') || t.includes('dashboard'))) {
      return 'vehicle';
    }
    return 'indoor';
  }
  
  private inferLighting(hour: number, objects: VisualObject[]): SceneContext['lighting'] {
    if (hour >= 6 && hour < 9) return 'bright';
    if (hour >= 9 && hour < 17) return 'normal';
    if (hour >= 17 && hour < 20) return 'dim';
    return 'dark';
  }
  
  private inferActivity(objects: VisualObject[], hour: number): SceneContext['activityType'] {
    const labels = objects.map(o => o.label.toLowerCase());
    
    if (labels.some(l => l.includes('food') || l.includes('menu'))) return 'eating';
    if (labels.some(l => l.includes('product') || l.includes('price'))) return 'shopping';
    if (labels.some(l => l.includes('computer') || l.includes('code'))) return 'working';
    if (labels.some(l => l.includes('exercise') || l.includes('gym'))) return 'exercising';
    if (labels.some(l => l.includes('person') || l.includes('friend'))) return 'social';
    
    // Time-based inference
    if (hour >= 9 && hour < 17) return 'working';
    if (hour >= 12 && hour < 14 || hour >= 18 && hour < 20) return 'eating';
    
    return 'unknown';
  }
  
  private extractDominantColors(objects: VisualObject[]): string[] {
    // In production, this would analyze actual pixel data
    return ['#FFFFFF', '#000000', '#4A90E2'];
  }
}

// ==================== INTELLIGENT FEEDBACK SYSTEM ====================

class IntelligentFeedbackSystem {
  
  async generateFeedback(
    focusedObject: VisualObject | null,
    sceneContext: SceneContext,
    eyeTracking: EyeTrackingData
  ): Promise<VisualFeedback | null> {
    if (!focusedObject) return null;
    
    // Generate contextual feedback based on object type and scene
    switch (focusedObject.type) {
      case 'product':
        return this.analyzeProduct(focusedObject, sceneContext);
      
      case 'food':
        return this.analyzeFood(focusedObject, sceneContext);
      
      case 'text':
        return this.analyzeText(focusedObject, sceneContext, eyeTracking);
      
      case 'person':
        return this.analyzePerson(focusedObject, sceneContext);
      
      case 'object':
        return this.analyzeObject(focusedObject, sceneContext);
      
      default:
        return this.analyzeGeneric(focusedObject, sceneContext);
    }
  }
  
  private async analyzeProduct(object: VisualObject, context: SceneContext): Promise<VisualFeedback> {
    const { label, attributes, confidence } = object;
    const price = attributes.price || 'Unknown';
    const brand = attributes.brand || 'Unknown brand';
    
    // Get shopping intelligence
    const hour = new Date().getHours();
    const isPeakHours = (hour >= 18 && hour <= 20) || (hour >= 12 && hour <= 13);
    
    return {
      objectInfo: `${label} by ${brand} - ${price}`,
      intelligentInsight: isPeakHours 
        ? `Peak shopping hours - stores are busy. Consider shopping earlier for better service.`
        : `Good shopping time - stores are less crowded now.`,
      actionSuggestions: [
        'Compare prices online',
        'Check reviews',
        'Look for similar products',
        'Add to wish list'
      ],
      relevantData: {
        estimatedValue: price,
        brand,
        category: label
      },
      confidence,
      reasoning: `Analyzed product type (${label}), current time (${hour}:00), and shopping context`
    };
  }
  
  private async analyzeFood(object: VisualObject, context: SceneContext): Promise<VisualFeedback> {
    const { label, attributes, confidence } = object;
    const calories = attributes.calories || 'Unknown';
    const isHealthy = attributes.healthy || false;
    
    const hour = new Date().getHours();
    const mealTime = hour >= 12 && hour < 14 ? 'lunch' : hour >= 18 && hour < 20 ? 'dinner' : 'snack';
    
    const healthAdvice = isHealthy 
      ? '✅ Healthy choice!'
      : calories > 400 
        ? '⚠️ High calorie - consider portion size'
        : 'Moderate calories';
    
    return {
      objectInfo: `${label} - ${calories} cal`,
      intelligentInsight: `${healthAdvice} Good for ${mealTime} time.`,
      actionSuggestions: [
        'Check ingredients',
        'See nutritional info',
        'Find similar healthy options',
        'Track in health app'
      ],
      relevantData: {
        calories,
        mealTime,
        isHealthy
      },
      confidence,
      reasoning: `Analyzed food type (${label}), calorie content, and current meal time (${mealTime})`
    };
  }
  
  private async analyzeText(object: VisualObject, context: SceneContext, eyeTracking: EyeTrackingData): Promise<VisualFeedback> {
    const { label, attributes, confidence } = object;
    const text = attributes.text || '';
    const wordCount = attributes.wordCount || 0;
    
    // Analyze reading behavior
    const isIntensiveReading = eyeTracking.focusIntensity > 0.7;
    const readingDuration = eyeTracking.fixationDuration;
    
    let readingFeedback = '';
    if (isIntensiveReading && readingDuration > 5000) {
      readingFeedback = '📖 Long reading session - consider a break soon for eye health.';
    } else if (isIntensiveReading) {
      readingFeedback = '📖 Focused reading detected.';
    } else {
      readingFeedback = '👁️ Scanning text.';
    }
    
    return {
      objectInfo: `${label}${wordCount > 0 ? ` (${wordCount} words)` : ''}`,
      intelligentInsight: readingFeedback,
      actionSuggestions: [
        'Read aloud',
        'Translate text',
        'Save for later',
        'Summarize content'
      ],
      warnings: readingDuration > 10000 ? ['Consider taking an eye break'] : undefined,
      relevantData: {
        text: text.substring(0, 100),
        wordCount,
        readingIntensity: eyeTracking.focusIntensity
      },
      confidence,
      reasoning: `Analyzed text content, reading intensity (${(eyeTracking.focusIntensity * 100).toFixed(0)}%), and duration (${Math.round(readingDuration / 1000)}s)`
    };
  }
  
  private async analyzePerson(object: VisualObject, context: SceneContext): Promise<VisualFeedback> {
    const { label, attributes, confidence } = object;
    const distance = attributes.distance || 'unknown';
    
    return {
      objectInfo: `${label} at ${distance}`,
      intelligentInsight: 'Person detected in view.',
      actionSuggestions: [
        'Identify contact',
        'Start conversation mode',
        'Respect privacy - no recording'
      ],
      warnings: ['Privacy mode active - no facial recognition stored'],
      confidence,
      reasoning: 'Person detection with privacy-first approach'
    };
  }
  
  private async analyzeObject(object: VisualObject, context: SceneContext): Promise<VisualFeedback> {
    const { label, attributes, confidence } = object;
    
    return {
      objectInfo: label,
      intelligentInsight: `${label} detected in your view.`,
      actionSuggestions: [
        'Get more info',
        'Find similar objects',
        'Learn about this'
      ],
      relevantData: attributes,
      confidence,
      reasoning: `Object recognition: ${label} with ${(confidence * 100).toFixed(0)}% confidence`
    };
  }
  
  private async analyzeGeneric(object: VisualObject, context: SceneContext): Promise<VisualFeedback> {
    return {
      objectInfo: object.label,
      intelligentInsight: 'Object detected.',
      actionSuggestions: ['Learn more', 'Search online'],
      confidence: object.confidence,
      reasoning: 'Generic object analysis'
    };
  }
}

// ==================== MAIN VISUAL INTELLIGENCE SERVICE ====================

class VisualIntelligenceService {
  private eyeTracking: EyeTrackingSystem;
  private objectRecognition: ObjectRecognitionSystem;
  private sceneUnderstanding: SceneUnderstandingSystem;
  private feedbackSystem: IntelligentFeedbackSystem;
  
  private isActive = false;
  private updateInterval: any = null;
  private currentScene: SceneContext | null = null;
  private currentFocusedObject: VisualObject | null = null;
  private currentFeedback: VisualFeedback | null = null;
  
  constructor() {
    this.eyeTracking = new EyeTrackingSystem();
    this.objectRecognition = new ObjectRecognitionSystem();
    this.sceneUnderstanding = new SceneUnderstandingSystem();
    this.feedbackSystem = new IntelligentFeedbackSystem();
  }
  
  start(cameraFeed: HTMLVideoElement | null = null): void {
    if (this.isActive) return;
    
    this.isActive = true;
    
    // Update visual intelligence at 15 FPS (comfortable for real-time analysis)
    this.updateInterval = setInterval(() => {
      this.update(cameraFeed);
    }, 1000 / 15);
  }
  
  stop(): void {
    this.isActive = false;
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
  }
  
  private async update(cameraFeed: HTMLVideoElement | null): Promise<void> {
    try {
      // 1. Track eye gaze
      const gazeData = this.eyeTracking.simulateEyeTracking(cameraFeed);
      
      // 2. Recognize objects in view
      const objects = await this.objectRecognition.recognizeObjects(cameraFeed);
      
      // 3. Find what user is looking at
      const focusedObject = this.objectRecognition.findObjectAtGazePoint(
        objects,
        gazeData.gazeX,
        gazeData.gazeY
      );
      
      // 4. Record gaze with focused object
      this.eyeTracking.recordGaze(gazeData, focusedObject || undefined);
      
      // 5. Understand scene context
      this.currentScene = await this.sceneUnderstanding.analyzeScene(objects, gazeData);
      
      // 6. Generate intelligent feedback if user is fixated on something
      if (focusedObject && gazeData.isFixated && gazeData.fixationDuration > 500) {
        this.currentFocusedObject = focusedObject;
        this.currentFeedback = await this.feedbackSystem.generateFeedback(
          focusedObject,
          this.currentScene,
          gazeData
        );
      } else {
        this.currentFeedback = null;
      }
      
    } catch (error) {
      console.error('Visual intelligence update error:', error);
    }
  }
  
  // Public API
  getCurrentGaze(): EyeTrackingData | null {
    return this.eyeTracking.getCurrentGaze();
  }
  
  getCurrentScene(): SceneContext | null {
    return this.currentScene;
  }
  
  getCurrentFocusedObject(): VisualObject | null {
    return this.currentFocusedObject;
  }
  
  getCurrentFeedback(): VisualFeedback | null {
    return this.currentFeedback;
  }
  
  getAttentionPatterns() {
    return this.eyeTracking.getAttentionPatterns();
  }
  
  isRunning(): boolean {
    return this.isActive;
  }
}

// ==================== EXPORT ====================

export const visualIntelligence = new VisualIntelligenceService();

export type {
  EyeTrackingData,
  VisualObject,
  SceneContext,
  VisualFeedback,
  GazeHistory
};
