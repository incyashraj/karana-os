/**
 * Kāraṇa OS - Context Manager
 * 
 * Enriches every user input with comprehensive context from all sources:
 * - Conversation history (last 10 exchanges)
 * - Complete system state (all 9 layers)
 * - User profile (preferences, habits, frequent contacts)
 * - Temporal context (time of day, recent actions, patterns)
 * - Spatial context (location, what user is looking at, nearby)
 * 
 * This context enables the AI to:
 * - Resolve pronouns ("send it to him" → "send photo to John")
 * - Understand references ("do that again" → repeat last action)
 * - Make intelligent suggestions based on patterns
 * - Provide personalized responses
 */

import { systemState, CompleteSystemState } from './systemState';
import { userProfileManager, UserProfile } from './userProfile';

// =============================================================================
// Types
// =============================================================================

export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  intent?: string;  // What the AI understood
  actions?: string[];  // What actions were taken
}

export interface RecentAction {
  action: string;  // e.g., "CAMERA_CAPTURE", "WALLET_TRANSFER"
  layer: string;
  params: Record<string, any>;
  timestamp: number;
  success: boolean;
  result?: any;
}

export interface TemporalContext {
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  dayOfWeek: string;
  date: string;
  recentActions: RecentAction[];  // Last 10 actions
  patterns: {
    frequentTime: string;  // When user most uses the OS
    commonSequences: string[][];  // Common action sequences
  };
}

export interface SpatialContext {
  location?: {
    lat: number;
    lng: number;
    accuracy: number;
  };
  lookingAt?: {
    object: string;
    description: string;
    confidence: number;
    timestamp: number;
  };
  nearbyDevices?: {
    name: string;
    type: string;
    distance: number;
  }[];
  environment?: 'indoor' | 'outdoor' | 'vehicle' | 'unknown';
}

export interface ReferenceContext {
  lastMentionedPerson?: string;  // For "him", "her", "them"
  lastMentionedObject?: string;  // For "it", "that", "this"
  lastMentionedApp?: string;  // For "that app"
  lastMentionedLocation?: string;  // For "there"
  lastMentionedAmount?: number;  // For "that much"
}

export interface EnrichedContext {
  // Raw input
  rawInput: string;
  preprocessedInput: string;
  
  // Conversation context
  conversationHistory: ConversationMessage[];
  currentTurn: number;
  
  // System state
  systemState: CompleteSystemState;
  
  // User profile
  userProfile: UserProfile;
  
  // Temporal context
  temporal: TemporalContext;
  
  // Spatial context
  spatial: SpatialContext;
  
  // Reference resolution
  references: ReferenceContext;
  
  // Metadata
  enrichedAt: number;
  contextQuality: number;  // 0-1, how complete the context is
}

// =============================================================================
// Context Manager Class
// =============================================================================

export class ContextManager {
  private conversationHistory: ConversationMessage[] = [];
  private recentActions: RecentAction[] = [];
  private maxHistoryLength = 50;  // Keep last 50 messages
  private maxActionsLength = 100;  // Keep last 100 actions
  
  // Reference tracking for pronoun resolution
  private references: ReferenceContext = {};

  /**
   * Main entry point: Enrich user input with all available context
   */
  enrich(
    rawInput: string,
    preprocessedInput: string
  ): EnrichedContext {
    
    const systemStateSnapshot = systemState.getState();
    const userProfile = userProfileManager.getProfile();
    const temporal = this.buildTemporalContext();
    const spatial = this.buildSpatialContext(systemStateSnapshot);
    
    // Calculate context quality (how much info we have)
    const contextQuality = this.calculateContextQuality(
      this.conversationHistory.length,
      this.recentActions.length,
      userProfile,
      spatial
    );
    
    return {
      rawInput,
      preprocessedInput,
      conversationHistory: this.conversationHistory.slice(-10),  // Last 10
      currentTurn: this.conversationHistory.length / 2,  // Rough turn count
      systemState: systemStateSnapshot,
      userProfile,
      temporal,
      spatial,
      references: { ...this.references },
      enrichedAt: Date.now(),
      contextQuality,
    };
  }

  /**
   * Add user message to conversation history
   */
  addUserMessage(content: string, intent?: string): void {
    this.conversationHistory.push({
      role: 'user',
      content,
      timestamp: Date.now(),
      intent,
    });
    
    // Update references from user input
    this.updateReferences(content);
    
    // Trim history if too long
    if (this.conversationHistory.length > this.maxHistoryLength) {
      this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
    }
  }

  /**
   * Add assistant message to conversation history
   */
  addAssistantMessage(content: string, actions?: string[]): void {
    this.conversationHistory.push({
      role: 'assistant',
      content,
      timestamp: Date.now(),
      actions,
    });
    
    // Trim history if too long
    if (this.conversationHistory.length > this.maxHistoryLength) {
      this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
    }
  }

  /**
   * Record an action that was executed
   */
  recordAction(
    action: string,
    layer: string,
    params: Record<string, any>,
    success: boolean,
    result?: any
  ): void {
    
    this.recentActions.push({
      action,
      layer,
      params,
      timestamp: Date.now(),
      success,
      result,
    });
    
    // Update references from action results
    this.updateReferencesFromAction(action, params, result);
    
    // Trim actions if too long
    if (this.recentActions.length > this.maxActionsLength) {
      this.recentActions = this.recentActions.slice(-this.maxActionsLength);
    }
    
    // Record in user profile for pattern learning
    userProfileManager.recordAction(action, params);
  }

  /**
   * Resolve pronouns and references in input
   */
  resolveReferences(input: string, context: EnrichedContext): string {
    let resolved = input;
    
    // Resolve "it", "that", "this"
    if (this.references.lastMentionedObject) {
      resolved = resolved.replace(/\b(it|that|this)\b/gi, this.references.lastMentionedObject);
    }
    
    // Resolve "him", "her", "them"
    if (this.references.lastMentionedPerson) {
      resolved = resolved.replace(/\b(him|her|them)\b/gi, this.references.lastMentionedPerson);
    }
    
    // Resolve "there"
    if (this.references.lastMentionedLocation) {
      resolved = resolved.replace(/\bthere\b/gi, this.references.lastMentionedLocation);
    }
    
    // Resolve "that app"
    if (this.references.lastMentionedApp) {
      resolved = resolved.replace(/\bthat app\b/gi, this.references.lastMentionedApp);
    }
    
    // Resolve temporal references
    resolved = this.resolveTemporal(resolved, context);
    
    // Resolve "do that again", "repeat that"
    if (/\b(again|repeat|do that)\b/i.test(input) && this.recentActions.length > 0) {
      const lastAction = this.recentActions[this.recentActions.length - 1];
      resolved = `Repeat action: ${lastAction.action}`;
    }
    
    return resolved;
  }

  /**
   * Resolve temporal references (yesterday, last week, etc.)
   */
  private resolveTemporal(input: string, context: EnrichedContext): string {
    let resolved = input;
    const now = new Date();
    
    // "yesterday"
    if (/\byesterday\b/i.test(input)) {
      const yesterday = new Date(now);
      yesterday.setDate(yesterday.getDate() - 1);
      resolved = resolved.replace(/\byesterday\b/gi, yesterday.toLocaleDateString());
    }
    
    // "last week"
    if (/\blast week\b/i.test(input)) {
      const lastWeek = new Date(now);
      lastWeek.setDate(lastWeek.getDate() - 7);
      resolved = resolved.replace(/\blast week\b/gi, `around ${lastWeek.toLocaleDateString()}`);
    }
    
    // "last time"
    if (/\blast time\b/i.test(input) && this.recentActions.length > 0) {
      const lastTime = new Date(this.recentActions[this.recentActions.length - 1].timestamp);
      resolved = resolved.replace(/\blast time\b/gi, lastTime.toLocaleString());
    }
    
    return resolved;
  }

  /**
   * Update reference tracking from user input
   */
  private updateReferences(input: string): void {
    const lower = input.toLowerCase();
    
    // Track mentioned apps
    const appNames = ['youtube', 'whatsapp', 'instagram', 'tiktok', 'twitter', 'spotify', 'telegram', 'facebook', 'netflix', 'gmail', 'chrome', 'maps'];
    for (const app of appNames) {
      if (lower.includes(app)) {
        this.references.lastMentionedApp = app;
        this.references.lastMentionedObject = app;
        break;
      }
    }
    
    // Track mentioned people (from user profile contacts)
    const profile = userProfileManager.getProfile();
    for (const [name, _] of profile.contacts.entries()) {
      if (lower.includes(name.toLowerCase())) {
        this.references.lastMentionedPerson = name;
        break;
      }
    }
    
    // Track amounts
    const amountMatch = input.match(/(\d+(?:\.\d+)?)\s*(?:kara|token)/i);
    if (amountMatch) {
      this.references.lastMentionedAmount = parseFloat(amountMatch[1]);
    }
    
    // Track locations (simple patterns)
    const locationMatch = input.match(/\b(at|to|near)\s+([A-Z][a-zA-Z\s]+)/);
    if (locationMatch) {
      this.references.lastMentionedLocation = locationMatch[2].trim();
    }
  }

  /**
   * Update references from action results
   */
  private updateReferencesFromAction(action: string, params: any, result: any): void {
    // If photo was captured, track it
    if (action === 'CAMERA_CAPTURE' && result?.imageData) {
      this.references.lastMentionedObject = 'photo';
    }
    
    // If vision analysis done, track the object
    if (action === 'VISION_ANALYZE' && result?.object) {
      this.references.lastMentionedObject = result.object;
    }
    
    // If app was opened, track it
    if (action === 'ANDROID_OPEN' && params.appName) {
      this.references.lastMentionedApp = params.appName;
      this.references.lastMentionedObject = params.appName;
    }
    
    // If transfer was made, track recipient
    if (action === 'WALLET_TRANSFER' && params.recipient) {
      this.references.lastMentionedPerson = params.recipient;
      this.references.lastMentionedAmount = params.amount;
    }
    
    // If navigation started, track destination
    if (action === 'NAVIGATION_START' && params.destination) {
      this.references.lastMentionedLocation = params.destination;
    }
  }

  /**
   * Build temporal context
   */
  private buildTemporalContext(): TemporalContext {
    const now = new Date();
    const hour = now.getHours();
    
    let timeOfDay: TemporalContext['timeOfDay'];
    if (hour >= 5 && hour < 12) timeOfDay = 'morning';
    else if (hour >= 12 && hour < 17) timeOfDay = 'afternoon';
    else if (hour >= 17 && hour < 21) timeOfDay = 'evening';
    else timeOfDay = 'night';
    
    const dayOfWeek = now.toLocaleDateString('en-US', { weekday: 'long' });
    const date = now.toLocaleDateString();
    
    // Get recent actions (last 10)
    const recentActions = this.recentActions.slice(-10);
    
    // Detect patterns
    const patterns = this.detectPatterns();
    
    return {
      timeOfDay,
      dayOfWeek,
      date,
      recentActions,
      patterns,
    };
  }

  /**
   * Build spatial context from system state
   */
  private buildSpatialContext(state: CompleteSystemState): SpatialContext {
    const spatial: SpatialContext = {};
    
    // Get GPS location if available
    if (state.layer1_hardware.sensors.gps.enabled) {
      spatial.location = {
        lat: state.layer1_hardware.sensors.gps.lastLocation.lat,
        lng: state.layer1_hardware.sensors.gps.lastLocation.lng,
        accuracy: state.layer1_hardware.sensors.gps.lastLocation.accuracy,
      };
    }
    
    // Get last vision analysis (what user was looking at)
    if (state.layer5_intelligence.lastVisionAnalysis) {
      spatial.lookingAt = {
        object: state.layer5_intelligence.lastVisionAnalysis.object,
        description: state.layer5_intelligence.lastVisionAnalysis.description,
        confidence: state.layer5_intelligence.lastVisionAnalysis.confidence,
        timestamp: state.layer5_intelligence.lastVisionAnalysis.timestamp,
      };
    }
    
    // Detect environment (simple heuristic based on sensors)
    if (state.layer1_hardware.sensors.imu.accelerationMagnitude > 5) {
      spatial.environment = 'vehicle';
    } else if (state.layer1_hardware.sensors.gps.enabled && state.layer1_hardware.sensors.gps.lastLocation.accuracy < 10) {
      spatial.environment = 'outdoor';
    } else {
      spatial.environment = 'indoor';
    }
    
    return spatial;
  }

  /**
   * Detect usage patterns from action history
   */
  private detectPatterns(): TemporalContext['patterns'] {
    const patterns: TemporalContext['patterns'] = {
      frequentTime: 'unknown',
      commonSequences: [],
    };
    
    if (this.recentActions.length < 5) {
      return patterns;
    }
    
    // Detect most frequent time of day
    const timeDistribution: Record<string, number> = {
      morning: 0,
      afternoon: 0,
      evening: 0,
      night: 0,
    };
    
    for (const action of this.recentActions) {
      const date = new Date(action.timestamp);
      const hour = date.getHours();
      
      if (hour >= 5 && hour < 12) timeDistribution.morning++;
      else if (hour >= 12 && hour < 17) timeDistribution.afternoon++;
      else if (hour >= 17 && hour < 21) timeDistribution.evening++;
      else timeDistribution.night++;
    }
    
    patterns.frequentTime = Object.keys(timeDistribution).reduce((a, b) => 
      timeDistribution[a] > timeDistribution[b] ? a : b
    );
    
    // Detect common action sequences (2-3 actions in a row)
    const sequences: Map<string, number> = new Map();
    
    for (let i = 0; i < this.recentActions.length - 1; i++) {
      const seq = [this.recentActions[i].action, this.recentActions[i + 1].action].join(' → ');
      sequences.set(seq, (sequences.get(seq) || 0) + 1);
    }
    
    // Get top 3 sequences
    patterns.commonSequences = Array.from(sequences.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3)
      .filter(([_, count]) => count >= 2)  // At least happened twice
      .map(([seq, _]) => seq.split(' → '));
    
    return patterns;
  }

  /**
   * Calculate context quality score (0-1)
   */
  private calculateContextQuality(
    historyLength: number,
    actionsLength: number,
    profile: UserProfile,
    spatial: SpatialContext
  ): number {
    let score = 0;
    
    // Conversation history (0-0.25)
    score += Math.min(historyLength / 20, 0.25);
    
    // Action history (0-0.25)
    score += Math.min(actionsLength / 50, 0.25);
    
    // User profile completeness (0-0.25)
    let profileScore = 0;
    if (profile.contacts.size > 0) profileScore += 0.1;
    if (profile.preferences && Object.keys(profile.preferences).length > 0) profileScore += 0.1;
    if (profile.statistics.totalCommands > 10) profileScore += 0.05;
    score += profileScore;
    
    // Spatial context (0-0.25)
    let spatialScore = 0;
    if (spatial.location) spatialScore += 0.1;
    if (spatial.lookingAt) spatialScore += 0.1;
    if (spatial.environment) spatialScore += 0.05;
    score += spatialScore;
    
    return Math.min(score, 1.0);
  }

  /**
   * Get conversation history
   */
  getHistory(): ConversationMessage[] {
    return [...this.conversationHistory];
  }

  /**
   * Get recent actions
   */
  getRecentActions(): RecentAction[] {
    return [...this.recentActions];
  }

  /**
   * Clear conversation history (useful for testing or reset)
   */
  clearHistory(): void {
    this.conversationHistory = [];
  }

  /**
   * Clear action history
   */
  clearActions(): void {
    this.recentActions = [];
  }

  /**
   * Reset all context
   */
  reset(): void {
    this.conversationHistory = [];
    this.recentActions = [];
    this.references = {};
  }

  /**
   * Get a human-readable summary of current context for debugging
   */
  getSummary(): string {
    const history = this.conversationHistory.slice(-3);
    const actions = this.recentActions.slice(-3);
    const refs = this.references;
    
    let summary = '📊 CONTEXT SUMMARY\n\n';
    
    summary += '💬 Recent Conversation:\n';
    if (history.length === 0) {
      summary += '  (no conversation yet)\n';
    } else {
      for (const msg of history) {
        summary += `  ${msg.role}: ${msg.content.substring(0, 50)}${msg.content.length > 50 ? '...' : ''}\n`;
      }
    }
    
    summary += '\n🎯 Recent Actions:\n';
    if (actions.length === 0) {
      summary += '  (no actions yet)\n';
    } else {
      for (const action of actions) {
        summary += `  ${action.action} (${action.success ? '✓' : '✗'})\n`;
      }
    }
    
    summary += '\n🔗 References:\n';
    if (refs.lastMentionedPerson) summary += `  Person: ${refs.lastMentionedPerson}\n`;
    if (refs.lastMentionedObject) summary += `  Object: ${refs.lastMentionedObject}\n`;
    if (refs.lastMentionedApp) summary += `  App: ${refs.lastMentionedApp}\n`;
    if (refs.lastMentionedLocation) summary += `  Location: ${refs.lastMentionedLocation}\n`;
    if (refs.lastMentionedAmount) summary += `  Amount: ${refs.lastMentionedAmount} KARA\n`;
    
    return summary;
  }
}

// Export singleton instance
export const contextManager = new ContextManager();
