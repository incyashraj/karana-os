/**
 * Kāraṇa OS - User Profile Manager
 * 
 * Stores and manages user-specific data for personalization:
 * - Preferences (security mode, brightness, favorite apps)
 * - Contacts (name → wallet address mappings)
 * - Command patterns (frequency of each command)
 * - Learning data (corrections, dismissed suggestions)
 * - Statistics (total commands, success rate, confidence)
 * 
 * Enables:
 * - Personalized responses ("your usual brightness is 80%")
 * - Quick contact references ("mom" → did:example:alice)
 * - Pattern-based suggestions ("you usually check battery at 9am")
 * - Continuous improvement (learns from corrections)
 */

// =============================================================================
// Types
// =============================================================================

export interface UserPreferences {
  // System preferences
  defaultSecurityMode: 'paranoid' | 'standard' | 'relaxed';
  defaultBrightness: number;  // 0-1
  defaultVolume: number;  // 0-1
  powerSaveThreshold: number;  // Battery % to trigger power save
  
  // Interface preferences
  hudAlwaysVisible: boolean;
  voiceInputEnabled: boolean;
  gesturesEnabled: boolean;
  gazeTrackingEnabled: boolean;
  
  // App preferences
  favoriteApps: string[];
  defaultNavigationApp: string;
  defaultMusicApp: string;
  defaultBrowser: string;
  
  // Notification preferences
  notificationsEnabled: boolean;
  quietHoursStart?: number;  // Hour (0-23)
  quietHoursEnd?: number;  // Hour (0-23)
  
  // Privacy preferences
  dataCollectionConsent: boolean;
  locationTrackingConsent: boolean;
  visionAnalysisConsent: boolean;
}

export interface CommandPattern {
  command: string;
  frequency: number;  // How many times used
  lastUsed: number;  // Timestamp
  averageConfidence: number;  // Avg confidence of AI classification
  successRate: number;  // % of successful executions
}

export interface LearningData {
  // User corrections ("I said 5 KARA, not 10")
  corrections: Map<string, string>;  // wrong → correct
  
  // Dismissed suggestions (so we don't repeat them)
  dismissedSuggestions: Set<string>;
  
  // Failed commands (for improvement)
  failedCommands: Array<{
    command: string;
    reason: string;
    timestamp: number;
  }>;
  
  // Custom vocabulary (user teaches new words/names)
  customVocabulary: Map<string, string>;  // abbreviation → full term
}

export interface UserStatistics {
  // Overall stats
  totalCommands: number;
  successfulCommands: number;
  failedCommands: number;
  successRate: number;  // %
  averageConfidence: number;  // 0-1
  
  // Usage stats
  firstUsed: number;  // Timestamp of first command
  lastUsed: number;  // Timestamp of last command
  totalUsageTime: number;  // Milliseconds
  averageSessionLength: number;  // Milliseconds
  
  // Feature usage
  mostUsedLayer: string;  // Which layer gets used most
  mostUsedOperation: string;  // Which operation gets used most
  leastUsedFeatures: string[];  // Features user hasn't tried
}

export interface UserProfile {
  // Identity
  userId: string;  // DID or unique ID
  displayName: string;
  createdAt: number;
  
  // Preferences
  preferences: UserPreferences;
  
  // Contacts (for quick reference in commands)
  contacts: Map<string, string>;  // nickname → wallet address/DID
  
  // Command patterns
  commandPatterns: Map<string, CommandPattern>;
  
  // Learning data
  learningData: LearningData;
  
  // Statistics
  statistics: UserStatistics;
  
  // Recent actions (for pattern detection)
  recentActions: string[];  // Last 50 action names
}

// =============================================================================
// User Profile Manager
// =============================================================================

export class UserProfileManager {
  private profile: UserProfile;
  private storageKey = 'karana_user_profile';

  constructor() {
    this.profile = this.loadProfile();
  }

  /**
   * Get current user profile
   */
  getProfile(): UserProfile {
    return { ...this.profile };
  }

  /**
   * Update a preference
   */
  updatePreference<K extends keyof UserPreferences>(
    key: K,
    value: UserPreferences[K]
  ): void {
    this.profile.preferences[key] = value;
    this.saveProfile();
  }

  /**
   * Add or update a contact
   */
  addContact(nickname: string, address: string): void {
    this.profile.contacts.set(nickname.toLowerCase(), address);
    this.saveProfile();
  }

  /**
   * Remove a contact
   */
  removeContact(nickname: string): void {
    this.profile.contacts.delete(nickname.toLowerCase());
    this.saveProfile();
  }

  /**
   * Get contact address by nickname
   */
  getContact(nickname: string): string | undefined {
    return this.profile.contacts.get(nickname.toLowerCase());
  }

  /**
   * Record that a command was executed
   */
  recordAction(action: string, params: Record<string, any>): void {
    // Add to recent actions
    this.profile.recentActions.push(action);
    if (this.profile.recentActions.length > 50) {
      this.profile.recentActions = this.profile.recentActions.slice(-50);
    }
    
    // Update command pattern
    const pattern = this.profile.commandPatterns.get(action);
    if (pattern) {
      pattern.frequency++;
      pattern.lastUsed = Date.now();
    } else {
      this.profile.commandPatterns.set(action, {
        command: action,
        frequency: 1,
        lastUsed: Date.now(),
        averageConfidence: 1.0,
        successRate: 1.0,
      });
    }
    
    // Update statistics
    this.profile.statistics.totalCommands++;
    this.profile.statistics.lastUsed = Date.now();
    
    this.saveProfile();
  }

  /**
   * Record a successful command execution
   */
  recordSuccess(action: string, confidence: number): void {
    this.profile.statistics.successfulCommands++;
    this.profile.statistics.successRate = 
      (this.profile.statistics.successfulCommands / this.profile.statistics.totalCommands) * 100;
    
    // Update command pattern
    const pattern = this.profile.commandPatterns.get(action);
    if (pattern) {
      // Exponential moving average for confidence
      pattern.averageConfidence = 0.8 * pattern.averageConfidence + 0.2 * confidence;
      pattern.successRate = ((pattern.frequency - 1) * pattern.successRate + 100) / pattern.frequency;
    }
    
    // Update average confidence
    const totalConfidence = Array.from(this.profile.commandPatterns.values())
      .reduce((sum, p) => sum + p.averageConfidence, 0);
    this.profile.statistics.averageConfidence = 
      totalConfidence / this.profile.commandPatterns.size;
    
    this.saveProfile();
  }

  /**
   * Record a failed command execution
   */
  recordFailure(action: string, reason: string): void {
    this.profile.statistics.failedCommands++;
    this.profile.statistics.successRate = 
      (this.profile.statistics.successfulCommands / this.profile.statistics.totalCommands) * 100;
    
    // Add to failed commands log
    this.profile.learningData.failedCommands.push({
      command: action,
      reason,
      timestamp: Date.now(),
    });
    
    // Keep only last 100 failed commands
    if (this.profile.learningData.failedCommands.length > 100) {
      this.profile.learningData.failedCommands = 
        this.profile.learningData.failedCommands.slice(-100);
    }
    
    // Update command pattern
    const pattern = this.profile.commandPatterns.get(action);
    if (pattern) {
      pattern.successRate = ((pattern.frequency - 1) * pattern.successRate + 0) / pattern.frequency;
    }
    
    this.saveProfile();
  }

  /**
   * Record a user correction (learning)
   */
  recordCorrection(wrong: string, correct: string): void {
    this.profile.learningData.corrections.set(wrong.toLowerCase(), correct.toLowerCase());
    this.saveProfile();
  }

  /**
   * Check if a suggestion was previously dismissed
   */
  wasSuggestionDismissed(suggestion: string): boolean {
    return this.profile.learningData.dismissedSuggestions.has(suggestion);
  }

  /**
   * Dismiss a suggestion (user doesn't want it)
   */
  dismissSuggestion(suggestion: string): void {
    this.profile.learningData.dismissedSuggestions.add(suggestion);
    this.saveProfile();
  }

  /**
   * Add custom vocabulary (user teaches new words)
   */
  addCustomVocabulary(abbreviation: string, fullTerm: string): void {
    this.profile.learningData.customVocabulary.set(
      abbreviation.toLowerCase(),
      fullTerm.toLowerCase()
    );
    this.saveProfile();
  }

  /**
   * Get most frequently used commands
   */
  getTopCommands(limit: number = 10): CommandPattern[] {
    return Array.from(this.profile.commandPatterns.values())
      .sort((a, b) => b.frequency - a.frequency)
      .slice(0, limit);
  }

  /**
   * Get commands with low success rate (need improvement)
   */
  getProblematicCommands(threshold: number = 0.7): CommandPattern[] {
    return Array.from(this.profile.commandPatterns.values())
      .filter(p => p.successRate < threshold * 100 && p.frequency >= 3)
      .sort((a, b) => a.successRate - b.successRate);
  }

  /**
   * Detect usage patterns for proactive suggestions
   */
  detectPatterns(): {
    timeOfDayPatterns: Map<string, string[]>;  // time → common actions
    sequencePatterns: string[][];  // common action sequences
    locationPatterns: Map<string, string[]>;  // location → common actions
  } {
    const patterns = {
      timeOfDayPatterns: new Map<string, string[]>(),
      sequencePatterns: [] as string[][],
      locationPatterns: new Map<string, string[]>(),
    };
    
    // TODO: Implement pattern detection from action history
    // This would analyze timestamps, locations, and sequences
    // For now, return empty patterns
    
    return patterns;
  }

  /**
   * Get personalized greeting based on time and usage
   */
  getPersonalizedGreeting(): string {
    const hour = new Date().getHours();
    const name = this.profile.displayName;
    
    let timeGreeting = '';
    if (hour >= 5 && hour < 12) timeGreeting = 'Good morning';
    else if (hour >= 12 && hour < 17) timeGreeting = 'Good afternoon';
    else if (hour >= 17 && hour < 21) timeGreeting = 'Good evening';
    else timeGreeting = 'Good night';
    
    const greetings = [
      `${timeGreeting}, ${name}!`,
      `${timeGreeting}! How can I help you today?`,
      `Hey ${name}, what can I do for you?`,
      `${timeGreeting}! Ready to assist.`,
    ];
    
    return greetings[Math.floor(Math.random() * greetings.length)];
  }

  /**
   * Export profile for backup
   */
  exportProfile(): string {
    const exportData = {
      ...this.profile,
      contacts: Array.from(this.profile.contacts.entries()),
      commandPatterns: Array.from(this.profile.commandPatterns.entries()),
      learningData: {
        corrections: Array.from(this.profile.learningData.corrections.entries()),
        dismissedSuggestions: Array.from(this.profile.learningData.dismissedSuggestions),
        failedCommands: this.profile.learningData.failedCommands,
        customVocabulary: Array.from(this.profile.learningData.customVocabulary.entries()),
      },
    };
    
    return JSON.stringify(exportData, null, 2);
  }

  /**
   * Import profile from backup
   */
  importProfile(data: string): void {
    try {
      const imported = JSON.parse(data);
      
      // Reconstruct Maps and Sets
      imported.contacts = new Map(imported.contacts);
      imported.commandPatterns = new Map(imported.commandPatterns);
      imported.learningData.corrections = new Map(imported.learningData.corrections);
      imported.learningData.dismissedSuggestions = new Set(imported.learningData.dismissedSuggestions);
      imported.learningData.customVocabulary = new Map(imported.learningData.customVocabulary);
      
      this.profile = imported;
      this.saveProfile();
    } catch (error) {
      console.error('Failed to import profile:', error);
      throw new Error('Invalid profile data');
    }
  }

  /**
   * Reset profile to defaults
   */
  resetProfile(): void {
    this.profile = this.createDefaultProfile();
    this.saveProfile();
  }

  /**
   * Load profile from localStorage
   */
  private loadProfile(): UserProfile {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (stored) {
        const parsed = JSON.parse(stored);
        
        // Reconstruct Maps and Sets
        parsed.contacts = new Map(parsed.contacts);
        parsed.commandPatterns = new Map(parsed.commandPatterns);
        parsed.learningData.corrections = new Map(parsed.learningData.corrections);
        parsed.learningData.dismissedSuggestions = new Set(parsed.learningData.dismissedSuggestions);
        parsed.learningData.customVocabulary = new Map(parsed.learningData.customVocabulary);
        
        return parsed;
      }
    } catch (error) {
      console.error('Failed to load profile:', error);
    }
    
    return this.createDefaultProfile();
  }

  /**
   * Save profile to localStorage
   */
  private saveProfile(): void {
    try {
      const toStore = {
        ...this.profile,
        contacts: Array.from(this.profile.contacts.entries()),
        commandPatterns: Array.from(this.profile.commandPatterns.entries()),
        learningData: {
          corrections: Array.from(this.profile.learningData.corrections.entries()),
          dismissedSuggestions: Array.from(this.profile.learningData.dismissedSuggestions),
          failedCommands: this.profile.learningData.failedCommands,
          customVocabulary: Array.from(this.profile.learningData.customVocabulary.entries()),
        },
      };
      
      localStorage.setItem(this.storageKey, JSON.stringify(toStore));
    } catch (error) {
      console.error('Failed to save profile:', error);
    }
  }

  /**
   * Create default profile for new user
   */
  private createDefaultProfile(): UserProfile {
    return {
      userId: `user_${Date.now()}`,
      displayName: 'User',
      createdAt: Date.now(),
      
      preferences: {
        defaultSecurityMode: 'standard',
        defaultBrightness: 0.7,
        defaultVolume: 0.7,
        powerSaveThreshold: 20,
        
        hudAlwaysVisible: true,
        voiceInputEnabled: true,
        gesturesEnabled: true,
        gazeTrackingEnabled: false,
        
        favoriteApps: [],
        defaultNavigationApp: 'maps',
        defaultMusicApp: 'spotify',
        defaultBrowser: 'chrome',
        
        notificationsEnabled: true,
        quietHoursStart: undefined,
        quietHoursEnd: undefined,
        
        dataCollectionConsent: false,
        locationTrackingConsent: false,
        visionAnalysisConsent: false,
      },
      
      contacts: new Map(),
      commandPatterns: new Map(),
      
      learningData: {
        corrections: new Map(),
        dismissedSuggestions: new Set(),
        failedCommands: [],
        customVocabulary: new Map(),
      },
      
      statistics: {
        totalCommands: 0,
        successfulCommands: 0,
        failedCommands: 0,
        successRate: 100,
        averageConfidence: 1.0,
        
        firstUsed: Date.now(),
        lastUsed: Date.now(),
        totalUsageTime: 0,
        averageSessionLength: 0,
        
        mostUsedLayer: 'UNKNOWN',
        mostUsedOperation: 'UNKNOWN',
        leastUsedFeatures: [],
      },
      
      recentActions: [],
    };
  }
}

// Export singleton instance
export const userProfileManager = new UserProfileManager();
