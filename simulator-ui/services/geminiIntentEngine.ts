/**
 * Kāraṇa OS - Gemini Intent Engine
 * 
 * The MASTER BRAIN that understands user intent with deep OS awareness.
 * 
 * Unlike the basic intentClassifier (which just extracts intents), this engine:
 * - Provides conversational AI with full system context
 * - Handles ANY user request, not just pre-defined commands
 * - Can reason about complex multi-step operations
 * - Generates natural, context-aware responses
 * - Learns from user corrections and improves over time
 * 
 * This is the TRUE intelligence layer that makes Oracle usable.
 */

import { GoogleGenerativeAI } from "@google/genai";
import { intentClassifier, IntentClassificationResult, IntentAction } from './intentClassifier';
import { contextManager, EnrichedContext } from './contextManager';
import { userProfileManager } from './userProfile';
import { systemState } from './systemState';

// =============================================================================
// Types
// =============================================================================

export interface GeminiResponse {
  // What the AI understood
  understanding: string;
  
  // What actions should be taken
  actions: IntentAction[];
  
  // Natural language response to user
  message: string;
  
  // Whether actions need confirmation
  needsConfirmation: boolean;
  confirmationMessage?: string;
  
  // Suggested follow-up questions/actions
  suggestions: string[];
  
  // Confidence in understanding (0-1)
  confidence: number;
  
  // If AI needs more information
  needsClarification: boolean;
  clarificationQuestion?: string;
  
  // Reasoning (for debugging)
  reasoning?: string;
}

// =============================================================================
// Gemini Intent Engine
// =============================================================================

export class GeminiIntentEngine {
  private gemini: GoogleGenerativeAI | null = null;
  // Use stable, widely available model
  private modelName = "gemini-1.5-flash";
  private conversationContext: string[] = [];  // Rolling context window

  constructor() {
    const apiKey = (import.meta as any).env?.VITE_GEMINI_API_KEY;
    if (apiKey) {
      this.gemini = new GoogleGenerativeAI({ apiKey });
    } else {
      console.warn('⚠️ Gemini API key not found. Advanced AI features will be limited.');
    }
  }

  /**
   * Main entry point: Process user request with COMPLETE intelligence
   */
  async process(userInput: string): Promise<GeminiResponse> {
    console.log('[GeminiIntentEngine] Processing:', userInput);
    console.log('[GeminiIntentEngine] Gemini available:', this.gemini !== null);
    
    // Step 1: Build enriched context
    const preprocessed = this.preprocess(userInput);
    const context = contextManager.enrich(userInput, preprocessed);
    
    // Step 2: Add to conversation history
    contextManager.addUserMessage(userInput);
    
    // Step 3: Classify intent (get structured actions)
    const classification = await intentClassifier.classify(preprocessed, context);
    console.log('[GeminiIntentEngine] Classification result:', {
      needsClarification: classification.needsClarification,
      intents: classification.intents.length,
      confidence: classification.confidence
    });
    
    // Step 4: If Gemini available, ALWAYS use it (even for unclear queries - it can handle general conversation)
    if (this.gemini) {
      console.log('[GeminiIntentEngine] Using Gemini for processing');
      try {
        const geminiResponse = await this.processWithGemini(
          userInput,
          preprocessed,
          context,
          classification
        );
        
        // Record success
        for (const action of geminiResponse.actions) {
          userProfileManager.recordSuccess(action.operation, geminiResponse.confidence);
        }
        
        // Add AI response to history
        contextManager.addAssistantMessage(
          geminiResponse.message,
          geminiResponse.actions.map(a => a.operation)
        );
        
        return geminiResponse;
      } catch (error) {
        console.error('Gemini processing failed:', error);
        // Fall through to classification-based response
      }
    } else {
      console.log('[GeminiIntentEngine] Gemini not available, using fallback');
    }
    
    // Step 5: Fallback to classification-based response
    console.log('[GeminiIntentEngine] Using fallback classification-based response');
    return this.buildResponseFromClassification(classification, context);
  }

  /**
   * Process with Gemini for truly intelligent responses
   */
  private async processWithGemini(
    rawInput: string,
    preprocessed: string,
    context: EnrichedContext,
    classification: IntentClassificationResult
  ): Promise<GeminiResponse> {
    
    if (!this.gemini) {
      throw new Error('Gemini not initialized');
    }

    // Build comprehensive system prompt
    const systemPrompt = this.buildSystemPrompt(context);
    
    // Build user prompt with classification hints
    const userPrompt = this.buildUserPrompt(rawInput, classification, context);
    
    // Call Gemini for natural conversation + action planning
    const model = (this.gemini as any).getGenerativeModel({ model: this.modelName });
    
    const result = await model.generateContent({
      contents: [
        // System context
        { role: 'user', parts: [{ text: systemPrompt }] },
        { role: 'model', parts: [{ text: 'I understand. I am the Oracle AI with complete omniscience over Kāraṇa OS. I will provide intelligent, context-aware assistance.' }] },
        
        // Recent conversation (last 5 exchanges)
        ...this.buildConversationHistory(context),
        
        // Current request
        { role: 'user', parts: [{ text: userPrompt }] },
      ],
      generationConfig: {
        // Force JSON so we can parse reliably
        responseMimeType: 'application/json'
      }
    });
    
    const responseText = result.response.text();
    
    // Parse Gemini's response
    return this.parseGeminiResponse(responseText, classification, context);
  }

  /**
   * Build comprehensive system prompt with COMPLETE OS awareness
   */
  private buildSystemPrompt(context: EnrichedContext): string {
    const state = context.systemState;
    const profile = context.userProfile;
    const temporal = context.temporal;

    const safe = <T>(v: T | undefined | null, fallback: T) => (v === undefined || v === null ? fallback : v);
    const num = (v: any, fallback = 0) => Number.isFinite(v) ? Number(v) : fallback;
    const arr = <T>(v: T[] | undefined | null): T[] => (Array.isArray(v) ? v : []);
    const mapEntries = (m: Map<any, any> | undefined | null) => (m instanceof Map ? Array.from(m.entries()) : []);
    
    // System state summary
    const battery = (num(state.layer1_hardware.power?.batteryLevel, 0) * 100).toFixed(0);
    const brightness = (num(state.layer1_hardware.display?.brightness, 0.5) * 100).toFixed(0);
    const wallet = state.layer3_blockchain.wallet?.exists 
      ? `exists (${num(state.layer3_blockchain.wallet?.balance, 0)} KARA)`
      : 'not created';
    const installedApps = arr(state.layer8_applications.androidApps)
      .filter(a => a.installed)
      .map(a => a.name)
      .join(', ') || 'none';
    const runningApps = arr(state.layer8_applications.androidApps)
      .filter(a => a.running)
      .map(a => a.name)
      .join(', ') || 'none';
    
    // User preferences
    const securityMode = safe(profile.preferences?.defaultSecurityMode, 'balanced');
    const favoriteApps = arr(profile.favoriteApps).join(', ') || 'none set';
    
    // Contacts
    const contacts = mapEntries(profile.contacts)
      .map(([name, addr]) => `"${name}" → ${String(addr).substring(0, 20)}...`)
      .join(', ') || 'none';
    
    // Recent actions
    const recentActions = arr(temporal.recentActions)
      .slice(-5)
      .map(a => `${a.action} (${a.success ? '✓' : '✗'})`)
      .join(', ') || 'none';
    
    // Common patterns
    const topCommands = userProfileManager.getTopCommands(5)
      .map(p => `${p.command} (${p.frequency}x)`)
      .join(', ') || 'none yet';

    const tempC = num(state.layer1_hardware.power?.temperatureCelsius, 0).toFixed(1);
    const isCharging = (state.layer1_hardware.power as any)?.isCharging ?? (state.layer1_hardware.power as any)?.charging ?? false;
    const audioVol = (num(state.layer1_hardware.audio?.volume, 0) * 100).toFixed(0);
    const spatialAudio = (state.layer1_hardware.audio as any)?.spatialAudioEnabled ?? false;
    const gpsEnabled = (state.layer1_hardware.sensors.gps as any)?.enabled ?? false;
    const gpsAcc = num((state.layer1_hardware.sensors.gps as any)?.lastLocation?.accuracy ?? state.layer1_hardware.sensors.gps?.accuracy, 0);
    const imuCal = (state.layer1_hardware.sensors.imu as any)?.calibrated ?? false;
    const syncProgress = num((state as any).layer2_network?.syncProgress, 0);
    const bandwidth = num((state as any).layer2_network?.bandwidth, 0).toFixed(1);

    return `You are the Oracle AI - the primary intelligence layer of Kāraṇa OS smart glasses.

You are NOT a chatbot with limited commands. You are a TRUE AI-FIRST OPERATING SYSTEM INTERFACE.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
COMPLETE SYSTEM STATE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

HARDWARE (Layer 1):
  • Battery: ${battery}% (${num(state.layer1_hardware.power?.estimatedRuntime, 0)} min, ${isCharging ? 'charging' : 'not charging'})
  • Temperature: ${tempC}°C
  • Camera: ${state.layer1_hardware.camera?.active ? 'Active' : 'Inactive'}, Mode: ${state.layer1_hardware.camera?.mode || 'auto'}
  • Display: ${brightness}% brightness, ${state.layer1_hardware.display?.mode || 'standard'} mode
  • Audio: ${audioVol}% volume, Spatial: ${spatialAudio ? 'ON' : 'OFF'}
  • GPS: ${gpsEnabled ? `ON (${gpsAcc}m accuracy)` : 'OFF'}
  • IMU: ${imuCal ? 'Calibrated' : 'Uncalibrated'}

NETWORK (Layer 2):
  • Peers: ${num(state.layer2_network.peerCount, 0)} connected
  • Sync: ${safe(state.layer2_network.syncStatus, 'unknown')} (${syncProgress}%)
  • Bandwidth: ${bandwidth} Mbps

BLOCKCHAIN (Layer 3):
  • Wallet: ${wallet}
  • Transactions: ${state.layer3_blockchain.transactions.length} total
  • Pending TX: ${state.layer3_blockchain.transactions.filter(tx => tx.status === 'pending').length}

INTELLIGENCE (Layer 5):
  • Last Vision: ${state.layer5_intelligence.lastVisionAnalysis?.object || 'none'}
  • Scene Understanding: ${state.layer5_intelligence.contextAwareness.currentScene}
  • Attention: ${state.layer5_intelligence.contextAwareness.userAttentionFocus}

INTERFACE (Layer 7):
  • HUD: ${state.layer7_interface.hud.enabled ? 'Visible' : 'Hidden'} (opacity: ${state.layer7_interface.hud.opacity})
  • Gestures: ${state.layer7_interface.gestures.enabled ? 'Tracking' : 'Disabled'}
  • Gaze: ${state.layer7_interface.gaze.enabled ? 'Tracking' : 'Disabled'} (calibrated: ${state.layer7_interface.gaze.calibrated})
  • Voice: ${state.layer7_interface.voice.enabled ? 'Active' : 'Disabled'} (language: ${state.layer7_interface.voice.language})
  • AR Mode: ${state.layer7_interface.arMode.enabled ? 'Active' : 'Inactive'}

APPLICATIONS (Layer 8):
  • Installed Apps: ${installedApps}
  • Running Apps: ${runningApps}
  • Timers: ${state.layer8_applications.timers.length} active
  • Navigation: ${state.layer8_applications.navigation.active ? `Active (to: ${state.layer8_applications.navigation.destination})` : 'Inactive'}
  • Settings: ${state.layer8_applications.settings.shown ? 'Open' : 'Closed'}

SYSTEM SERVICES (Layer 9):
  • Security: ${securityMode} mode
  • Health Score: ${(state.layer9_services.diagnostics.healthScore * 100).toFixed(0)}%
  • OTA Updates: ${state.layer9_services.ota.updateAvailable ? `Available (v${state.layer9_services.ota.version})` : 'Up to date'}

SPATIAL:
  • AR Anchors: ${state.spatial.anchors.length} placed
  • AR Tabs: ${state.spatial.tabs.length} open
  • SLAM Status: ${state.spatial.slamStatus}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
USER PROFILE & CONTEXT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

User: ${profile.displayName}
Time: ${temporal.timeOfDay} (${temporal.dayOfWeek}, ${temporal.date})
Location: ${context.spatial.location ? `${context.spatial.location.lat.toFixed(4)}, ${context.spatial.location.lng.toFixed(4)}` : 'unknown'}
Looking At: ${context.spatial.lookingAt?.object || 'nothing specific'}
Environment: ${context.spatial.environment || 'unknown'}

Preferences:
  • Security: ${securityMode}
  • Favorite Apps: ${favoriteApps}
  • Brightness: ${(profile.preferences.defaultBrightness * 100).toFixed(0)}%
  • Voice Input: ${profile.preferences.voiceInputEnabled ? 'Enabled' : 'Disabled'}

Known Contacts:
  ${contacts}

Recent Actions (last 5):
  ${recentActions}

Most Used Commands:
  ${topCommands}

Usage Stats:
  • Total Commands: ${profile.statistics.totalCommands}
  • Success Rate: ${profile.statistics.successRate.toFixed(1)}%
  • Average Confidence: ${(profile.statistics.averageConfidence * 100).toFixed(1)}%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
YOUR CAPABILITIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

You can control EVERYTHING in this OS:

HARDWARE: Camera (photo/video), Display (brightness/mode), Audio (volume), Power (status/save mode), Sensors (GPS/IMU)
NETWORK: Check peers, sync blockchain, manage connections
BLOCKCHAIN: Create wallet, check balance, send KARA tokens, view transactions, governance voting
INTELLIGENCE: Analyze vision, identify objects, understand scenes, answer questions
INTERFACE: Show/hide HUD, enable gestures/gaze/voice, enter AR mode, adjust opacity
APPLICATIONS: Install/open/close ANY Android app, set timers, navigate, check wellness, open settings
SYSTEM: Run diagnostics, check updates, change security mode, view logs
SPATIAL: Create AR anchors, open AR tabs, manage spatial workspace

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HOW TO RESPOND
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

FORMAT YOUR RESPONSE AS JSON:
{
  "understanding": "Brief summary of what you understood",
  "message": "Natural, conversational response to the user (2-3 sentences max)",
  "actions": [
    {
      "layer": "HARDWARE|NETWORK|BLOCKCHAIN|INTELLIGENCE|INTERFACE|APPLICATIONS|SYSTEM_SERVICES|SPATIAL",
      "operation": "SPECIFIC_OPERATION",
      "params": { "key": "value" },
      "confidence": 0.95,
      "reasoning": "Why this action"
    }
  ],
  "needsConfirmation": false,
  "confirmationMessage": "Optional: What needs confirmation",
  "suggestions": ["Follow-up 1", "Follow-up 2", "Follow-up 3"],
  "confidence": 0.95,
  "needsClarification": false,
  "clarificationQuestion": "Optional: What you need to know"
}

RULES:
1. **Understand ANYTHING**: Users can ask ANYTHING - technical questions, casual chat, complex tasks, ambiguous requests
2. **Be Conversational**: Talk like a helpful friend, not a robot
3. **Be Proactive**: Suggest related actions, warn about issues (low battery, etc.)
4. **Use Context**: Reference conversation history, user preferences, recent actions
5. **Multi-Step Planning**: Break complex requests into logical action sequences
6. **Ask When Unsure**: If ambiguous (confidence < 0.7), ask for clarification
7. **Learn**: Remember corrections, adapt to user patterns
8. **Be Smart About Dependencies**: Auto-handle prerequisites (create wallet before transfer, install before open)
9. **Personalize**: Use user's name, remember their preferences, suggest based on habits
10. **Safety First**: Confirm high-stakes actions (transfers, deletions, security changes)

EXAMPLES:

User: "battery low, what should I do?"
Response: {
  "understanding": "User concerned about low battery",
  "message": "Your battery is at ${battery}%. I can enable power save mode to extend it by ~30 minutes, or you could close ${runningApps} to save power. Want me to optimize it?",
  "actions": [],
  "needsConfirmation": false,
  "suggestions": ["Enable power save mode", "Close running apps", "Check what's draining battery"],
  "confidence": 0.95
}

User: "take a picture and send 5 KARA to mom"
Response: {
  "understanding": "Take photo, then transfer 5 KARA to mom contact",
  "message": "I'll take a photo and send 5 KARA to mom. Just so you know, mom's address is ${profile.contacts.get('mom')}. Ready?",
  "actions": [
    {"layer": "HARDWARE", "operation": "CAMERA_CAPTURE", "params": {}, "confidence": 0.98},
    {"layer": "BLOCKCHAIN", "operation": "WALLET_TRANSFER", "params": {"amount": 5, "recipient": "${profile.contacts.get('mom')}"}, "confidence": 0.95}
  ],
  "needsConfirmation": true,
  "confirmationMessage": "This will transfer 5 KARA to mom. Confirm?",
  "confidence": 0.96
}

User: "I'm bored"
Response: {
  "understanding": "User is bored, wants entertainment suggestions",
  "message": "Let's fix that! Based on your usage, you enjoy YouTube and Spotify. I can open either, or we could explore something new. What sounds good?",
  "actions": [],
  "suggestions": ["Open YouTube", "Play music on Spotify", "Explore new apps", "Check what's trending"],
  "confidence": 0.85
}

NOW, RESPOND TO THE USER'S REQUEST WITH THIS FULL CONTEXT.`;
  }

  /**
   * Build user prompt with classification hints
   */
  private buildUserPrompt(
    rawInput: string,
    classification: IntentClassificationResult,
    context: EnrichedContext
  ): string {
    const hints = classification.intents.length > 0
      ? `\n\nIntent Classification Hints (you can refine these):\n${JSON.stringify(classification.intents, null, 2)}\n\nEntities Found: ${JSON.stringify(classification.entities, null, 2)}`
      : '';
    
    return `User Request: "${rawInput}"${hints}

Provide intelligent, context-aware response in JSON format.`;
  }

  /**
   * Build conversation history for context
   */
  private buildConversationHistory(context: EnrichedContext): Array<{ role: string; parts: Array<{ text: string }> }> {
    return context.conversationHistory.slice(-5).map(msg => ({
      role: msg.role === 'user' ? 'user' : 'model',
      parts: [{ text: msg.content }],
    }));
  }

  /**
   * Parse Gemini's JSON response
   */
  private parseGeminiResponse(
    responseText: string,
    classification: IntentClassificationResult,
    context: EnrichedContext
  ): GeminiResponse {
    
    try {
      // Try to extract JSON from response
      const jsonMatch = responseText.match(/\{[\s\S]*\}/);
      if (!jsonMatch) {
        throw new Error('No JSON found in response');
      }
      
      const parsed = JSON.parse(jsonMatch[0]);
      
      // Validate and return
      return {
        understanding: parsed.understanding || 'Processing your request',
        actions: parsed.actions || classification.intents,
        message: parsed.message || 'Let me help you with that.',
        needsConfirmation: parsed.needsConfirmation || false,
        confirmationMessage: parsed.confirmationMessage,
        suggestions: parsed.suggestions || [],
        confidence: parsed.confidence || classification.confidence,
        needsClarification: parsed.needsClarification || false,
        clarificationQuestion: parsed.clarificationQuestion,
        reasoning: parsed.reasoning,
      };
      
    } catch (error) {
      console.error('Failed to parse Gemini response:', error);
      console.log('Raw response:', responseText);
      
      // Fallback: show Gemini's raw response as message so user still gets an answer
      return {
        understanding: 'General response',
        actions: classification.intents,
        message: responseText || 'Let me rephrase that.',
        needsConfirmation: false,
        suggestions: ['Ask another question', 'Try a different phrasing'],
        confidence: classification.confidence || 0.5,
        needsClarification: false,
      };
    }
  }

  /**
   * Build response from classification (fallback when Gemini unavailable)
   */
  private buildResponseFromClassification(
    classification: IntentClassificationResult,
    context: EnrichedContext
  ): GeminiResponse {
    
    if (classification.needsClarification) {
      return {
        understanding: 'Request unclear',
        actions: [],
        message: classification.clarificationQuestion || 'Could you rephrase that?',
        needsConfirmation: false,
        suggestions: ['Try saying it differently', 'Be more specific', 'Ask for help'],
        confidence: classification.confidence,
        needsClarification: true,
        clarificationQuestion: classification.clarificationQuestion,
      };
    }
    
    // Generate simple message based on first intent
    const firstIntent = classification.intents[0];
    let message = 'Processing your request...';
    
    if (firstIntent) {
      const opName = firstIntent.operation.toLowerCase().replace(/_/g, ' ');
      message = `I'll ${opName} for you.`;
      
      // Add context if available
      if (classification.intents.length > 1) {
        message += ` This involves ${classification.intents.length} steps.`;
      }
    }
    
    return {
      understanding: `Executing: ${classification.intents.map(i => i.operation).join(', ')}`,
      actions: classification.intents,
      message,
      needsConfirmation: classification.intents.some(i => 
        i.layer === 'BLOCKCHAIN' || 
        i.operation.includes('DELETE') ||
        i.operation.includes('SECURITY')
      ),
      suggestions: [],
      confidence: classification.confidence,
      needsClarification: false,
    };
  }

  /**
   * Simple preprocessing
   */
  private preprocess(input: string): string {
    return input.toLowerCase().trim();
  }

  /**
   * Check if Gemini is available
   */
  isAvailable(): boolean {
    return this.gemini !== null;
  }
}

// Export singleton
export const geminiIntentEngine = new GeminiIntentEngine();
