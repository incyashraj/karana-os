/**
 * Ambient Intelligence Layer
 * 
 * Philosophy:
 * - Operates in background, doesn't demand attention
 * - Learns user patterns without explicit input
 * - Provides proactive assistance at the right moment
 * - Respects user's focus and attention
 */

import { systemState } from './systemState';
import { visualIntelligence, SceneContext, VisualObject } from './visualIntelligence';
import { weatherService } from './realTimeServices';

// ==================== TYPE DEFINITIONS ====================

interface UserPattern {
  timeOfDay: number;
  activity: string;
  location: string;
  duration: number;
  frequency: number;
}

interface ProactiveNotification {
  id: string;
  type: 'health' | 'productivity' | 'contextual' | 'safety' | 'suggestion';
  priority: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  reasoning: string;
  actionable: boolean;
  actions?: string[];
  timestamp: number;
  expiresAt?: number;
}

interface AttentionState {
  currentFocus: 'high' | 'medium' | 'low' | 'distracted';
  cognitiveLoad: number; // 0-1
  eyeStrain: number; // 0-1
  shouldInterrupt: boolean;
  bestTimeToNotify: number; // timestamp
}

interface ContextualInsight {
  type: 'pattern' | 'anomaly' | 'opportunity' | 'warning';
  message: string;
  confidence: number;
  relevant: boolean;
  timestamp: number;
}

// ==================== PATTERN LEARNING SYSTEM ====================

class PatternLearningSystem {
  private patterns: UserPattern[] = [];
  private maxPatterns = 100;
  
  recordActivity(activity: string, location: string, duration: number): void {
    const hour = new Date().getHours();
    
    // Find existing pattern
    const existing = this.patterns.find(p =>
      p.timeOfDay === hour &&
      p.activity === activity &&
      p.location === location
    );
    
    if (existing) {
      // Update frequency and average duration
      existing.frequency++;
      existing.duration = (existing.duration + duration) / 2;
    } else {
      // Create new pattern
      this.patterns.push({
        timeOfDay: hour,
        activity,
        location,
        duration,
        frequency: 1
      });
    }
    
    // Keep only most frequent patterns
    if (this.patterns.length > this.maxPatterns) {
      this.patterns.sort((a, b) => b.frequency - a.frequency);
      this.patterns = this.patterns.slice(0, this.maxPatterns);
    }
  }
  
  predictNextActivity(): UserPattern | null {
    const hour = new Date().getHours();
    
    // Find most frequent pattern for this time
    const relevantPatterns = this.patterns
      .filter(p => Math.abs(p.timeOfDay - hour) <= 1)
      .sort((a, b) => b.frequency - a.frequency);
    
    return relevantPatterns[0] || null;
  }
  
  detectAnomaly(currentActivity: string): boolean {
    const hour = new Date().getHours();
    const expected = this.predictNextActivity();
    
    if (!expected) return false;
    
    // Anomaly if current activity is very different from expected
    return expected.activity !== currentActivity && expected.frequency > 5;
  }
  
  getPatterns(): UserPattern[] {
    return this.patterns;
  }
}

// ==================== ATTENTION MANAGEMENT SYSTEM ====================

class AttentionManagementSystem {
  private notificationQueue: ProactiveNotification[] = [];
  private lastNotificationTime = 0;
  private minNotificationInterval = 300000; // 5 minutes minimum between notifications
  
  assessAttentionState(): AttentionState {
    const gaze = visualIntelligence.getCurrentGaze();
    const scene = visualIntelligence.getCurrentScene();
    
    if (!gaze || !scene) {
      return {
        currentFocus: 'medium',
        cognitiveLoad: 0.5,
        eyeStrain: 0.3,
        shouldInterrupt: false,
        bestTimeToNotify: Date.now() + 300000
      };
    }
    
    // Determine focus level from gaze behavior
    let currentFocus: AttentionState['currentFocus'] = 'medium';
    if (gaze.focusIntensity > 0.8 && gaze.isFixated) {
      currentFocus = 'high';
    } else if (gaze.focusIntensity < 0.3 || gaze.blinkRate > 25) {
      currentFocus = 'low';
    } else if (!gaze.isFixated) {
      currentFocus = 'distracted';
    }
    
    // Estimate cognitive load
    const cognitiveLoad = Math.min(1, gaze.focusIntensity * 0.6 + scene.sceneComplexity * 0.4);
    
    // Calculate eye strain
    // In production, this would use actual usage metrics
    const usageTime = 60; // Simulated: minutes of usage
    const eyeStrain = Math.min(1, (usageTime / 120) * 0.7 + (1 - gaze.blinkRate / 20) * 0.3);
    
    // Should we interrupt?
    const shouldInterrupt = 
      currentFocus === 'low' || 
      currentFocus === 'distracted' ||
      eyeStrain > 0.8;
    
    // Best time to notify
    const bestTimeToNotify = shouldInterrupt 
      ? Date.now() 
      : Date.now() + 180000; // Wait 3 minutes if focused
    
    return {
      currentFocus,
      cognitiveLoad,
      eyeStrain,
      shouldInterrupt,
      bestTimeToNotify
    };
  }
  
  queueNotification(notification: ProactiveNotification): void {
    // Add to queue if not duplicate
    const isDuplicate = this.notificationQueue.some(n => 
      n.message === notification.message && Date.now() - n.timestamp < 600000
    );
    
    if (!isDuplicate) {
      this.notificationQueue.push(notification);
      this.notificationQueue.sort((a, b) => {
        // Sort by priority
        const priorityMap = { critical: 4, high: 3, medium: 2, low: 1 };
        return priorityMap[b.priority] - priorityMap[a.priority];
      });
    }
  }
  
  getNextNotification(): ProactiveNotification | null {
    const attentionState = this.assessAttentionState();
    
    // Don't interrupt if user is highly focused
    if (attentionState.currentFocus === 'high' && !attentionState.shouldInterrupt) {
      return null;
    }
    
    // Respect minimum interval between notifications
    const timeSinceLastNotification = Date.now() - this.lastNotificationTime;
    if (timeSinceLastNotification < this.minNotificationInterval) {
      return null;
    }
    
    // Clean up expired notifications
    this.notificationQueue = this.notificationQueue.filter(n => 
      !n.expiresAt || n.expiresAt > Date.now()
    );
    
    // Get highest priority notification
    const notification = this.notificationQueue.shift();
    
    if (notification) {
      this.lastNotificationTime = Date.now();
    }
    
    return notification || null;
  }
  
  clearQueue(): void {
    this.notificationQueue = [];
  }
  
  getQueueSize(): number {
    return this.notificationQueue.length;
  }
}

// ==================== CONTEXTUAL INSIGHTS SYSTEM ====================

class ContextualInsightsSystem {
  private insights: ContextualInsight[] = [];
  private maxInsights = 50;
  
  async generateInsights(scene: SceneContext | null, attentionState: AttentionState): Promise<ContextualInsight[]> {
    const newInsights: ContextualInsight[] = [];
    
    // Health insights
    if (attentionState.eyeStrain > 0.7) {
      newInsights.push({
        type: 'warning',
        message: 'High eye strain detected. Consider taking a break.',
        confidence: 0.85,
        relevant: true,
        timestamp: Date.now()
      });
    }
    
    // Cognitive load insights
    if (attentionState.cognitiveLoad > 0.8) {
      newInsights.push({
        type: 'warning',
        message: 'High cognitive load. Break down tasks or take a short rest.',
        confidence: 0.78,
        relevant: true,
        timestamp: Date.now()
      });
    }
    
    // Activity pattern insights
    if (scene) {
      const hour = new Date().getHours();
      
      // Late night working
      if (scene.activityType === 'working' && (hour >= 22 || hour <= 5)) {
        newInsights.push({
          type: 'warning',
          message: 'Working late. Consider sleep for better cognitive performance tomorrow.',
          confidence: 0.82,
          relevant: true,
          timestamp: Date.now()
        });
      }
      
      // Meal time suggestions
      if ((hour >= 12 && hour <= 14) || (hour >= 18 && hour <= 20)) {
        if (scene.activityType !== 'eating') {
          newInsights.push({
            type: 'opportunity',
            message: `Meal time detected. Consider ${hour < 15 ? 'lunch' : 'dinner'} break.`,
            confidence: 0.70,
            relevant: true,
            timestamp: Date.now()
          });
        }
      }
      
      // Exercise opportunities
      if (hour >= 7 && hour <= 9 || hour >= 17 && hour <= 19) {
        if (scene.activityType !== 'exercising') {
          newInsights.push({
            type: 'opportunity',
            message: 'Great time for physical activity. Consider a workout or walk.',
            confidence: 0.65,
            relevant: true,
            timestamp: Date.now()
          });
        }
      }
    }
    
    // Weather-based insights
    try {
      const weather = await weatherService.getWeather();
      if (weather) {
        // Beautiful weather opportunity
        if (weather.condition.toLowerCase().includes('clear') && 
            weather.temperature >= 20 && weather.temperature <= 28) {
          newInsights.push({
            type: 'opportunity',
            message: 'Perfect weather outside. Consider outdoor activities.',
            confidence: 0.75,
            relevant: true,
            timestamp: Date.now()
          });
        }
        
        // Bad weather warning
        if (weather.condition.toLowerCase().includes('rain') ||
            weather.condition.toLowerCase().includes('storm')) {
          newInsights.push({
            type: 'warning',
            message: 'Unfavorable weather. Plan for indoor activities.',
            confidence: 0.88,
            relevant: true,
            timestamp: Date.now()
          });
        }
      }
    } catch (error) {
      // Silent fail for weather insights
    }
    
    // Store insights
    this.insights.push(...newInsights);
    
    // Keep only recent insights
    if (this.insights.length > this.maxInsights) {
      this.insights = this.insights.slice(-this.maxInsights);
    }
    
    return newInsights;
  }
  
  getRelevantInsights(limit = 5): ContextualInsight[] {
    const now = Date.now();
    const recentThreshold = 3600000; // 1 hour
    
    return this.insights
      .filter(i => i.relevant && now - i.timestamp < recentThreshold)
      .sort((a, b) => b.confidence - a.confidence)
      .slice(0, limit);
  }
}

// ==================== PROACTIVE ASSISTANCE SYSTEM ====================

class ProactiveAssistanceSystem {
  async generateProactiveAssistance(
    scene: SceneContext | null,
    attentionState: AttentionState,
    patterns: UserPattern[]
  ): Promise<ProactiveNotification[]> {
    const notifications: ProactiveNotification[] = [];
    
    // Health-based notifications
    if (attentionState.eyeStrain > 0.75) {
      notifications.push({
        id: `health-eye-${Date.now()}`,
        type: 'health',
        priority: 'high',
        message: '👁️ Eye break recommended - you\'ve been focused for a while',
        reasoning: `Eye strain at ${Math.round(attentionState.eyeStrain * 100)}%`,
        actionable: true,
        actions: ['Take 20-second break', 'Look at distant object', 'Close eyes for a moment'],
        timestamp: Date.now(),
        expiresAt: Date.now() + 600000 // 10 minutes
      });
    }
    
    // Productivity-based notifications
    if (attentionState.currentFocus === 'distracted' && scene?.activityType === 'working') {
      notifications.push({
        id: `productivity-focus-${Date.now()}`,
        type: 'productivity',
        priority: 'medium',
        message: '🎯 Focus seems scattered - consider a focused work technique',
        reasoning: 'Detected low focus intensity during work activity',
        actionable: true,
        actions: ['Start Pomodoro timer', 'Clear distractions', 'Break down current task'],
        timestamp: Date.now(),
        expiresAt: Date.now() + 900000 // 15 minutes
      });
    }
    
    // Pattern-based notifications
    const expectedActivity = patterns.length > 0 ? patterns[0] : null;
    if (expectedActivity && scene) {
      const hour = new Date().getHours();
      if (Math.abs(expectedActivity.timeOfDay - hour) <= 1) {
        notifications.push({
          id: `pattern-activity-${Date.now()}`,
          type: 'suggestion',
          priority: 'low',
          message: `📊 You usually ${expectedActivity.activity} around this time`,
          reasoning: `Pattern observed ${expectedActivity.frequency} times`,
          actionable: false,
          timestamp: Date.now(),
          expiresAt: Date.now() + 1800000 // 30 minutes
        });
      }
    }
    
    // Contextual safety notifications
    if (scene && attentionState.cognitiveLoad > 0.85) {
      if (scene.environment === 'outdoor' || scene.environment === 'vehicle') {
        notifications.push({
          id: `safety-attention-${Date.now()}`,
          type: 'safety',
          priority: 'critical',
          message: '⚠️ High cognitive load detected - stay aware of surroundings',
          reasoning: 'High cognitive load in dynamic environment',
          actionable: true,
          actions: ['Reduce distractions', 'Focus on environment', 'Take break if possible'],
          timestamp: Date.now(),
          expiresAt: Date.now() + 300000 // 5 minutes
        });
      }
    }
    
    return notifications;
  }
}

// ==================== MAIN AMBIENT INTELLIGENCE SERVICE ====================

class AmbientIntelligenceService {
  private patternLearning: PatternLearningSystem;
  private attentionManagement: AttentionManagementSystem;
  private contextualInsights: ContextualInsightsSystem;
  private proactiveAssistance: ProactiveAssistanceSystem;
  
  private isActive = false;
  private updateInterval: any = null;
  
  constructor() {
    this.patternLearning = new PatternLearningSystem();
    this.attentionManagement = new AttentionManagementSystem();
    this.contextualInsights = new ContextualInsightsSystem();
    this.proactiveAssistance = new ProactiveAssistanceSystem();
  }
  
  start(): void {
    if (this.isActive) return;
    
    this.isActive = true;
    
    // Update ambient intelligence every 10 seconds (non-intrusive)
    this.updateInterval = setInterval(() => {
      this.update();
    }, 10000);
  }
  
  stop(): void {
    this.isActive = false;
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
  }
  
  private async update(): Promise<void> {
    try {
      // Get current context
      const scene = visualIntelligence.getCurrentScene();
      const attentionState = this.attentionManagement.assessAttentionState();
      
      // Learn patterns (passive learning)
      if (scene) {
        this.patternLearning.recordActivity(
          scene.activityType,
          scene.environment,
          10 // 10 second update interval
        );
      }
      
      // Generate contextual insights
      const insights = await this.contextualInsights.generateInsights(scene, attentionState);
      
      // Generate proactive assistance
      const patterns = this.patternLearning.getPatterns();
      const notifications = await this.proactiveAssistance.generateProactiveAssistance(
        scene,
        attentionState,
        patterns
      );
      
      // Queue notifications
      notifications.forEach(notif => {
        this.attentionManagement.queueNotification(notif);
      });
      
    } catch (error) {
      console.error('Ambient intelligence update error:', error);
    }
  }
  
  // Public API
  getNextNotification(): ProactiveNotification | null {
    return this.attentionManagement.getNextNotification();
  }
  
  getRecentInsights(limit = 5): ContextualInsight[] {
    return this.contextualInsights.getRelevantInsights(limit);
  }
  
  getLearnedPatterns(): UserPattern[] {
    return this.patternLearning.getPatterns();
  }
  
  getAttentionState(): AttentionState {
    return this.attentionManagement.assessAttentionState();
  }
  
  clearNotifications(): void {
    this.attentionManagement.clearQueue();
  }
  
  getNotificationQueueSize(): number {
    return this.attentionManagement.getQueueSize();
  }
  
  isRunning(): boolean {
    return this.isActive;
  }
}

// ==================== EXPORT ====================

export const ambientIntelligence = new AmbientIntelligenceService();

export type {
  UserPattern,
  ProactiveNotification,
  AttentionState,
  ContextualInsight
};
