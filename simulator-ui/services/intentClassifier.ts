/**
 * Kāraṇa OS - Advanced Intent Classification System
 * 
 * This replaces primitive pattern matching with intelligent multi-model classification.
 * 
 * Architecture:
 * 1. Preprocessing: Spelling correction, normalization, entity extraction
 * 2. Multi-model classification: Gemini (accurate) + Pattern matching (fast fallback)
 * 3. Ensemble voting: Combine results with confidence scoring
 * 4. Ambiguity detection: Ask clarification if needed
 * 
 * Handles:
 * - Multi-intent commands ("take photo and send to mom")
 * - Ambiguous queries ("it's too bright" → display or camera?)
 * - Contextual references ("do that again", "send it to him")
 * - Misspellings ("batry staus", "brightnes 50%")
 * - Natural language ("can you help me see what this is?")
 */

import { GoogleGenerativeAI, Schema, Type } from "@google/genai";
import { systemState, CompleteSystemState } from './systemState';
import { systemContext } from './systemContext';

// =============================================================================
// Types
// =============================================================================

export interface IntentAction {
  layer: string;  // 'HARDWARE', 'BLOCKCHAIN', etc.
  operation: string;  // 'CAMERA_CAPTURE', 'WALLET_CREATE', etc.
  params: Record<string, any>;
  confidence: number;  // 0-1
  reasoning?: string;  // Why this intent was chosen
}

export interface Entity {
  type: 'PERSON' | 'NUMBER' | 'TIME' | 'APP' | 'LOCATION' | 'TOKEN_AMOUNT' | 'DURATION';
  value: string;
  normalizedValue?: any;  // Parsed value (e.g., "5 minutes" → 300000ms)
  position: [number, number];  // Start, end position in input
}

export interface IntentClassificationResult {
  intents: IntentAction[];  // Can have multiple intents from one query
  entities: Entity[];  // Extracted entities (names, numbers, etc.)
  confidence: number;  // Overall confidence (0-1)
  ambiguities: string[];  // Parts that need clarification
  contextUsed: string[];  // Which context was leveraged
  alternativeInterpretations: string[];  // Other possible meanings
  needsClarification: boolean;
  clarificationQuestion?: string;
}

export interface EnrichedContext {
  conversationHistory: Array<{ role: 'user' | 'assistant'; content: string }>;
  systemState: CompleteSystemState;
  userProfile: {
    frequentCommands: string[];
    recentActions: string[];
    contacts: Map<string, string>;  // "mom" → wallet address
    preferences: Record<string, any>;
  };
  temporal: {
    timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
    recentActions: Array<{ action: string; timestamp: number }>;
  };
  spatial?: {
    location?: { lat: number; lng: number };
    lookingAt?: { object: string; confidence: number };
  };
}

// =============================================================================
// Spelling Correction Dictionary
// =============================================================================

const SPELLING_CORRECTIONS: Record<string, string> = {
  // Common typos for Kāraṇa OS commands
  'batry': 'battery',
  'battry': 'battery',
  'batter': 'battery',
  'brightnes': 'brightness',
  'brighness': 'brightness',
  'camer': 'camera',
  'camra': 'camera',
  'walet': 'wallet',
  'wallet': 'wallet',
  'ballance': 'balance',
  'balence': 'balance',
  'transferr': 'transfer',
  'analize': 'analyze',
  'analys': 'analyze',
  'foto': 'photo',
  'fotos': 'photos',
  'pict': 'picture',
  'instal': 'install',
  'opn': 'open',
  'launche': 'launch',
  'tim': 'timer',
  'timmr': 'timer',
  'navgate': 'navigate',
  'navig': 'navigate',
  'secrity': 'security',
  'securty': 'security',
  'dianostic': 'diagnostic',
  'diagnost': 'diagnostic',
  'updat': 'update',
  'updaate': 'update',
  'chck': 'check',
  'ckeck': 'check',
};

// =============================================================================
// Intent Classifier Class
// =============================================================================

export class IntentClassifier {
  private gemini: GoogleGenerativeAI | null = null;
  private modelName = "gemini-2.0-flash-exp";

  constructor() {
    const apiKey = (import.meta as any).env?.VITE_GEMINI_API_KEY;
    if (apiKey) {
      this.gemini = new GoogleGenerativeAI({ apiKey });
    } else {
      console.warn('⚠️ Gemini API key not found. Intent classification will use fallback patterns.');
    }
  }

  /**
   * Main entry point: Classify user intent with full context
   */
  async classify(
    rawInput: string,
    context: EnrichedContext
  ): Promise<IntentClassificationResult> {
    
    // Step 1: Preprocess input
    const preprocessed = this.preprocess(rawInput);
    
    // Step 2: Extract entities
    const entities = this.extractEntities(preprocessed, context);
    
    // Step 3: Multi-model classification
    let result: IntentClassificationResult;
    
    if (this.gemini) {
      // Use Gemini for accurate classification
      try {
        result = await this.classifyWithGemini(preprocessed, entities, context);
      } catch (error) {
        console.error('Gemini classification failed:', error);
        result = this.classifyWithPatterns(preprocessed, entities, context);
      }
    } else {
      // Fallback to pattern matching
      result = this.classifyWithPatterns(preprocessed, entities, context);
    }
    
    // Step 4: Post-processing and validation
    result = this.validateAndEnrich(result, context);
    
    return result;
  }

  /**
   * Preprocess input: spelling correction, normalization, tokenization
   */
  private preprocess(input: string): string {
    // 1. Lowercase
    let processed = input.toLowerCase().trim();
    
    // 2. Spelling correction
    const words = processed.split(/\s+/);
    const correctedWords = words.map(word => {
      // Remove punctuation for matching
      const cleanWord = word.replace(/[^a-z0-9]/g, '');
      return SPELLING_CORRECTIONS[cleanWord] || word;
    });
    processed = correctedWords.join(' ');
    
    // 3. Expand contractions
    processed = processed
      .replace(/won't/g, 'will not')
      .replace(/can't/g, 'cannot')
      .replace(/n't/g, ' not')
      .replace(/'ll/g, ' will')
      .replace(/'re/g, ' are')
      .replace(/'ve/g, ' have')
      .replace(/'m/g, ' am')
      .replace(/'d/g, ' would');
    
    // 4. Normalize whitespace
    processed = processed.replace(/\s+/g, ' ').trim();
    
    return processed;
  }

  /**
   * Extract entities: names, numbers, times, apps, locations
   */
  private extractEntities(input: string, context: EnrichedContext): Entity[] {
    const entities: Entity[] = [];
    
    // Extract numbers (potential amounts, percentages, durations)
    const numberPattern = /\b(\d+(?:\.\d+)?)\s*(%|percent|kara|token|minutes?|mins?|seconds?|secs?|hours?|hrs?)?\b/gi;
    let match;
    while ((match = numberPattern.exec(input)) !== null) {
      const value = match[1];
      const unit = match[2] || '';
      let type: Entity['type'] = 'NUMBER';
      let normalizedValue: any = parseFloat(value);
      
      if (unit.match(/%|percent/i)) {
        normalizedValue = parseFloat(value) / 100;  // Convert to 0-1
      } else if (unit.match(/kara|token/i)) {
        type = 'TOKEN_AMOUNT';
      } else if (unit.match(/min|sec|hour|hr/i)) {
        type = 'DURATION';
        // Convert to milliseconds
        if (unit.match(/hour|hr/i)) {
          normalizedValue = parseFloat(value) * 3600000;
        } else if (unit.match(/min/i)) {
          normalizedValue = parseFloat(value) * 60000;
        } else if (unit.match(/sec/i)) {
          normalizedValue = parseFloat(value) * 1000;
        }
      }
      
      entities.push({
        type,
        value: match[0],
        normalizedValue,
        position: [match.index, match.index + match[0].length],
      });
    }
    
    // Extract app names
    const appNames = systemContext.getAllApps().map(app => app.name.toLowerCase());
    for (const appName of appNames) {
      const index = input.indexOf(appName);
      if (index !== -1) {
        entities.push({
          type: 'APP',
          value: appName,
          normalizedValue: systemContext.findApp(appName),
          position: [index, index + appName.length],
        });
      }
    }
    
    // Extract contact names from user profile
    if (context.userProfile.contacts) {
      for (const [name, address] of context.userProfile.contacts.entries()) {
        const index = input.indexOf(name.toLowerCase());
        if (index !== -1) {
          entities.push({
            type: 'PERSON',
            value: name,
            normalizedValue: address,
            position: [index, index + name.length],
          });
        }
      }
    }
    
    // Extract time references
    const timePatterns = [
      { pattern: /\b(today|now|currently)\b/gi, type: 'TIME' as const },
      { pattern: /\b(yesterday|last\s+\w+)\b/gi, type: 'TIME' as const },
      { pattern: /\b(tomorrow|next\s+\w+)\b/gi, type: 'TIME' as const },
    ];
    
    for (const { pattern, type } of timePatterns) {
      while ((match = pattern.exec(input)) !== null) {
        entities.push({
          type,
          value: match[0],
          position: [match.index, match.index + match[0].length],
        });
      }
    }
    
    return entities;
  }

  /**
   * Classify intent using Gemini with structured output
   */
  private async classifyWithGemini(
    input: string,
    entities: Entity[],
    context: EnrichedContext
  ): Promise<IntentClassificationResult> {
    
    if (!this.gemini) {
      throw new Error('Gemini not initialized');
    }

    // Build comprehensive system prompt
    const systemPrompt = this.buildGeminiSystemPrompt(context);
    
    // Build user prompt with entities
    const userPrompt = this.buildGeminiUserPrompt(input, entities, context);
    
    // Define response schema for structured output
    const responseSchema: Schema = {
      type: Type.OBJECT,
      properties: {
        intents: {
          type: Type.ARRAY,
          items: {
            type: Type.OBJECT,
            properties: {
              layer: { type: Type.STRING, description: "Layer: HARDWARE, NETWORK, BLOCKCHAIN, INTELLIGENCE, INTERFACE, APPLICATIONS, SYSTEM_SERVICES, SPATIAL" },
              operation: { type: Type.STRING, description: "Specific operation to perform" },
              params: { type: Type.OBJECT, description: "Parameters for the operation" },
              confidence: { type: Type.NUMBER, description: "Confidence 0-1" },
              reasoning: { type: Type.STRING, description: "Why this intent was chosen" },
            },
            required: ["layer", "operation", "params", "confidence"],
          },
        },
        overallConfidence: { type: Type.NUMBER, description: "Overall confidence 0-1" },
        needsClarification: { type: Type.BOOLEAN, description: "Whether clarification is needed" },
        clarificationQuestion: { type: Type.STRING, description: "Question to ask user if needsClarification=true" },
        ambiguities: { type: Type.ARRAY, items: { type: Type.STRING } },
        contextUsed: { type: Type.ARRAY, items: { type: Type.STRING } },
        alternativeInterpretations: { type: Type.ARRAY, items: { type: Type.STRING } },
      },
      required: ["intents", "overallConfidence", "needsClarification"],
    };
    
    // Call Gemini
    const model = (this.gemini as any).getGenerativeModel({ model: this.modelName });
    const result = await model.generateContent({
      contents: [
        { role: 'user', parts: [{ text: systemPrompt }] },
        { role: 'model', parts: [{ text: 'Understood. I will classify intents for Kāraṇa OS with complete system awareness.' }] },
        { role: 'user', parts: [{ text: userPrompt }] },
      ],
      generationConfig: {
        responseMimeType: "application/json",
        responseSchema: responseSchema,
      },
    });
    
    const responseText = result.response.text();
    const parsed = JSON.parse(responseText);
    
    return {
      intents: parsed.intents || [],
      entities,
      confidence: parsed.overallConfidence || 0.5,
      ambiguities: parsed.ambiguities || [],
      contextUsed: parsed.contextUsed || [],
      alternativeInterpretations: parsed.alternativeInterpretations || [],
      needsClarification: parsed.needsClarification || false,
      clarificationQuestion: parsed.clarificationQuestion,
    };
  }

  /**
   * Build comprehensive system prompt for Gemini
   */
  private buildGeminiSystemPrompt(context: EnrichedContext): string {
    const state = context.systemState;
    // Safe helpers to avoid undefined property crashes
    const safe = <T>(v: T | undefined | null, fallback: T) => (v === undefined || v === null ? fallback : v);
    const num = (v: any, fallback = 0) => Number.isFinite(v) ? Number(v) : fallback;

    const battery = (num(state.layer1_hardware.power?.batteryLevel, 0) * 100).toFixed(0);
    const brightness = (num(state.layer1_hardware.display?.brightness, 0.5) * 100).toFixed(0);
    const walletExists = !!state.layer3_blockchain.wallet?.exists;
    const balance = num(state.layer3_blockchain.wallet?.balance, 0);
    const installedApps = safe(state.layer8_applications.androidApps, []).filter(a => a.installed).map(a => a.name).join(', ') || 'none';
    const runningApps = safe(state.layer8_applications.androidApps, []).filter(a => a.running).map(a => a.name).join(', ') || 'none';
    
    // Get recent conversation
    const recentConv = context.conversationHistory.slice(-5).map(msg => 
      `${msg.role === 'user' ? 'User' : 'Oracle'}: ${msg.content}`
    ).join('\n');
    
    const frequent = safe(context.userProfile.frequentCommands, []) as any[];
    const contacts = safe(context.userProfile.contacts, new Map<string, string>());

    return `You are the Oracle AI of Kāraṇa OS smart glasses. You control a complete 9-layer operating system.

CURRENT SYSTEM STATE:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Layer 1 - HARDWARE:
  • Battery: ${battery}% (${state.layer1_hardware.power.estimatedRuntime} min remaining)
  • Camera: ${state.layer1_hardware.camera.active ? 'Active' : 'Inactive'} (mode: ${state.layer1_hardware.camera.mode})
  • Display: ${brightness}% brightness, ${state.layer1_hardware.display.mode} mode
  • Audio: Volume ${(state.layer1_hardware.audio.volume * 100).toFixed(0)}%
  • Sensors: GPS ${state.layer1_hardware.sensors.gps?.enabled ? 'ON' : 'OFF'}, IMU ${state.layer1_hardware.sensors.imu?.calibrated ? 'calibrated' : 'uncalibrated'}

Layer 2 - NETWORK:
  • Peers: ${state.layer2_network.peerCount} connected
  • Sync: ${safe(state.layer2_network.syncStatus, 'unknown')} (${num((state as any).layer2_network?.syncProgress, 0)}%)
  • Bandwidth: ${num((state as any).layer2_network?.bandwidth, 0).toFixed(1)} Mbps

Layer 3 - BLOCKCHAIN:
  • Wallet: ${walletExists ? `exists (${state.layer3_blockchain.wallet.did})` : 'not created'}
  • Balance: ${balance} KARA
  • Transactions: ${state.layer3_blockchain.transactions.length} total

Layer 7 - INTERFACE:
  • HUD: ${state.layer7_interface.hud?.visible ?? state.layer7_interface.hud?.enabled ? 'Visible' : 'Hidden'}
  • Gestures: ${(state as any).layer7_interface?.gestures?.enabled ? 'Tracking' : 'Disabled'}
  • Gaze: ${state.layer7_interface.gaze?.enabled ? 'Tracking' : 'Disabled'}
  • AR Mode: ${(state as any).layer7_interface?.arMode?.enabled ?? state.layer7_interface.arMode ? 'Active' : 'Inactive'}

Layer 8 - APPLICATIONS:
  • Installed Apps: ${installedApps}
  • Running Apps: ${runningApps}
  • Timers: ${state.layer8_applications.timers.length} active
  • Navigation: ${state.layer8_applications.navigation.active ? `to ${state.layer8_applications.navigation.destination}` : 'inactive'}

Layer 9 - SYSTEM SERVICES:
  • Security Mode: ${state.layer9_services.security.mode}
  • System Health: ${(state.layer9_services.diagnostics.healthScore * 100).toFixed(0)}%
  • OTA Update: ${state.layer9_services.ota.updateAvailable ? `Available (v${state.layer9_services.ota.version})` : 'Up to date'}

SPATIAL:
  • AR Anchors: ${state.spatial.anchors.length} placed
  • AR Tabs: ${state.spatial.tabs.length} open

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

RECENT CONVERSATION:
${recentConv || 'No previous conversation'}

USER PROFILE:
  • Frequent Commands: ${frequent.slice(0, 5).join(', ') || 'none yet'}
  • Known Contacts: ${Array.from(contacts.keys()).join(', ') || 'none'}
  • Recent Actions: ${context.userProfile.recentActions.slice(0, 3).join(', ') || 'none'}

TIME CONTEXT:
  • Time of Day: ${context.temporal.timeOfDay}
  • Recent Actions: ${context.temporal.recentActions.slice(-3).map(a => a.action).join(', ') || 'none'}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

YOUR TASK:
Classify user intent into 1 or MORE actions. Be intelligent about:
1. Multi-intent commands ("take photo and send to mom" = 2 actions)
2. Contextual references ("do that again" = repeat last action)
3. Pronouns ("send it to him" = use last object + last mentioned person)
4. Ambiguity ("it's too bright" = could be display or camera exposure)
5. Implicit operations (asking about balance → need wallet check first)

AVAILABLE OPERATIONS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

HARDWARE: CAMERA_CAPTURE, CAMERA_RECORD_START, CAMERA_RECORD_STOP, DISPLAY_BRIGHTNESS, POWER_STATUS, POWER_SAVE_MODE, AUDIO_VOLUME
NETWORK: NETWORK_STATUS, BLOCKCHAIN_SYNC
BLOCKCHAIN: WALLET_CREATE, WALLET_BALANCE, WALLET_TRANSFER, WALLET_TRANSACTIONS
INTELLIGENCE: VISION_ANALYZE
INTERFACE: HUD_SHOW, HUD_HIDE, GESTURE_ENABLE, GESTURE_DISABLE, GAZE_ENABLE, GAZE_DISABLE, AR_MODE_ENABLE, AR_MODE_DISABLE
APPLICATIONS: TIMER_CREATE, TIMER_LIST, TIMER_CANCEL, NAVIGATION_START, SETTINGS_OPEN, WELLNESS_STATUS, ANDROID_INSTALL, ANDROID_OPEN, ANDROID_CLOSE
SYSTEM_SERVICES: OTA_CHECK, OTA_INSTALL, SECURITY_MODE, SECURITY_STATUS, DIAGNOSTICS_RUN, DIAGNOSTICS_STATUS
SPATIAL: ANCHOR_CREATE, ANCHOR_LIST, TAB_OPEN, TAB_LIST

RULES:
1. If confidence < 0.7 for any intent, set needsClarification=true
2. Extract all parameters from input (amounts, names, apps, etc.)
3. Use conversation history to resolve ambiguous references
4. Consider system state (can't transfer if no wallet)
5. Be proactive - suggest dependencies (install before open)
6. If multiple interpretations exist, list them in alternativeInterpretations`;
  }

  /**
   * Build user-specific prompt for current query
   */
  private buildGeminiUserPrompt(
    input: string,
    entities: Entity[],
    context: EnrichedContext
  ): string {
    const entitiesStr = entities.length > 0
      ? entities.map(e => `${e.type}: "${e.value}" (normalized: ${JSON.stringify(e.normalizedValue)})`).join(', ')
      : 'none detected';
    
    return `User Input: "${input}"

Extracted Entities: ${entitiesStr}

Classify this into one or more IntentActions. Return JSON matching the schema.

Remember:
- Multiple intents are OK ("take photo and send" = CAMERA_CAPTURE + WALLET_TRANSFER)
- Use context to resolve pronouns and references
- If ambiguous, ask for clarification
- Extract ALL parameters (amounts, recipients, durations, etc.)`;
  }

  /**
   * Fallback pattern matching classification (when Gemini unavailable)
   */
  private classifyWithPatterns(
    input: string,
    entities: Entity[],
    context: EnrichedContext
  ): IntentClassificationResult {
    
    const intents: IntentAction[] = [];
    const contextUsed: string[] = [];
    
    // Helper function
    const matches = (keywords: string[]) => keywords.some(kw => input.includes(kw));
    
    // HARDWARE operations
    if (matches(['camera', 'photo', 'picture', 'capture', 'snap'])) {
      if (matches(['take', 'capture', 'snap'])) {
        intents.push({
          layer: 'HARDWARE',
          operation: 'CAMERA_CAPTURE',
          params: {},
          confidence: 0.9,
          reasoning: 'User wants to take a photo',
        });
      } else if (matches(['record', 'video', 'start recording'])) {
        intents.push({
          layer: 'HARDWARE',
          operation: 'CAMERA_RECORD_START',
          params: {},
          confidence: 0.9,
          reasoning: 'User wants to start recording video',
        });
      }
    }
    
    if (matches(['brightness', 'display', 'screen']) && matches(['set', 'change', 'adjust'])) {
      const numberEntity = entities.find(e => e.type === 'NUMBER' && e.normalizedValue !== undefined);
      const value = numberEntity?.normalizedValue ?? 0.5;
      intents.push({
        layer: 'HARDWARE',
        operation: 'DISPLAY_BRIGHTNESS',
        params: { value },
        confidence: numberEntity ? 0.9 : 0.6,
        reasoning: `Adjust display brightness to ${(value * 100).toFixed(0)}%`,
      });
    }
    
    if (matches(['battery', 'power', 'charge']) && matches(['status', 'check', 'how much', 'level'])) {
      intents.push({
        layer: 'HARDWARE',
        operation: 'POWER_STATUS',
        params: {},
        confidence: 0.95,
        reasoning: 'Check battery status',
      });
    }
    
    // BLOCKCHAIN operations
    if (matches(['wallet', 'create', 'setup']) && !context.systemState.layer3_blockchain.wallet.exists) {
      intents.push({
        layer: 'BLOCKCHAIN',
        operation: 'WALLET_CREATE',
        params: {},
        confidence: 0.9,
        reasoning: 'Create new wallet',
      });
    }
    
    if (matches(['balance', 'how much', 'kara'])) {
      intents.push({
        layer: 'BLOCKCHAIN',
        operation: 'WALLET_BALANCE',
        params: {},
        confidence: 0.95,
        reasoning: 'Check wallet balance',
      });
      contextUsed.push('blockchain state');
    }
    
    if (matches(['send', 'transfer', 'pay'])) {
      const amountEntity = entities.find(e => e.type === 'TOKEN_AMOUNT');
      const personEntity = entities.find(e => e.type === 'PERSON');
      
      intents.push({
        layer: 'BLOCKCHAIN',
        operation: 'WALLET_TRANSFER',
        params: {
          amount: amountEntity?.normalizedValue ?? 0,
          recipient: personEntity?.normalizedValue ?? '',
        },
        confidence: (amountEntity && personEntity) ? 0.9 : 0.5,
        reasoning: `Transfer ${amountEntity?.value ?? '?'} KARA to ${personEntity?.value ?? '?'}`,
      });
      
      if (personEntity) contextUsed.push('user contacts');
    }
    
    // INTELLIGENCE operations
    if (matches(['analyze', 'see', 'look', 'identify', 'what', 'recognize'])) {
      intents.push({
        layer: 'INTELLIGENCE',
        operation: 'VISION_ANALYZE',
        params: {},
        confidence: 0.85,
        reasoning: 'Analyze what user is looking at',
      });
    }
    
    // APPLICATIONS operations
    if (matches(['timer', 'set', 'create', 'countdown'])) {
      const durationEntity = entities.find(e => e.type === 'DURATION');
      intents.push({
        layer: 'APPLICATIONS',
        operation: 'TIMER_CREATE',
        params: { durationMs: durationEntity?.normalizedValue ?? 60000 },
        confidence: durationEntity ? 0.9 : 0.7,
        reasoning: `Set timer for ${durationEntity?.value ?? '1 minute'}`,
      });
    }
    
    // Android app operations
    const appEntity = entities.find(e => e.type === 'APP');
    if (appEntity) {
      if (matches(['install', 'download', 'get'])) {
        intents.push({
          layer: 'APPLICATIONS',
          operation: 'ANDROID_INSTALL',
          params: { appName: appEntity.value },
          confidence: 0.95,
          reasoning: `Install ${appEntity.value}`,
        });
      } else if (matches(['open', 'launch', 'start', 'run'])) {
        intents.push({
          layer: 'APPLICATIONS',
          operation: 'ANDROID_OPEN',
          params: { appName: appEntity.value },
          confidence: 0.95,
          reasoning: `Open ${appEntity.value}`,
        });
      } else if (matches(['close', 'stop', 'exit', 'quit'])) {
        intents.push({
          layer: 'APPLICATIONS',
          operation: 'ANDROID_CLOSE',
          params: { appName: appEntity.value },
          confidence: 0.95,
          reasoning: `Close ${appEntity.value}`,
        });
      }
      contextUsed.push('available apps');
    }
    
    // SYSTEM SERVICES operations
    if (matches(['update', 'ota', 'upgrade'])) {
      if (matches(['check', 'available'])) {
        intents.push({
          layer: 'SYSTEM_SERVICES',
          operation: 'OTA_CHECK',
          params: {},
          confidence: 0.9,
          reasoning: 'Check for system updates',
        });
      }
    }
    
    if (matches(['security', 'privacy']) && matches(['mode', 'set'])) {
      let mode = 'standard';
      if (matches(['paranoid', 'maximum', 'high'])) mode = 'paranoid';
      if (matches(['relaxed', 'low'])) mode = 'relaxed';
      
      intents.push({
        layer: 'SYSTEM_SERVICES',
        operation: 'SECURITY_MODE',
        params: { mode },
        confidence: 0.85,
        reasoning: `Set security mode to ${mode}`,
      });
    }
    
    if (matches(['diagnostic', 'health', 'system check'])) {
      intents.push({
        layer: 'SYSTEM_SERVICES',
        operation: 'DIAGNOSTICS_RUN',
        params: {},
        confidence: 0.85,
        reasoning: 'Run system diagnostics',
      });
    }
    
    // If no intents found, it's a conversation
    if (intents.length === 0) {
      return {
        intents: [],
        entities,
        confidence: 0.3,
        ambiguities: ['Could not understand command'],
        contextUsed: [],
        alternativeInterpretations: ['This might be a general question or conversation'],
        needsClarification: true,
        clarificationQuestion: 'I didn\'t quite understand. Could you rephrase that?',
      };
    }
    
    // Calculate overall confidence
    const avgConfidence = intents.reduce((sum, i) => sum + i.confidence, 0) / intents.length;
    
    return {
      intents,
      entities,
      confidence: avgConfidence,
      ambiguities: [],
      contextUsed,
      alternativeInterpretations: [],
      needsClarification: avgConfidence < 0.6,
      clarificationQuestion: avgConfidence < 0.6 ? 'Did I understand that correctly?' : undefined,
    };
  }

  /**
   * Validate and enrich classification result
   */
  private validateAndEnrich(
    result: IntentClassificationResult,
    context: EnrichedContext
  ): IntentClassificationResult {
    
    // Check for missing dependencies
    for (const intent of result.intents) {
      // If transferring KARA but no wallet, flag it
      if (intent.layer === 'BLOCKCHAIN' && intent.operation === 'WALLET_TRANSFER') {
        if (!context.systemState.layer3_blockchain.wallet.exists) {
          result.ambiguities.push('Wallet needs to be created first');
          result.needsClarification = true;
          result.clarificationQuestion = 'You don\'t have a wallet yet. Should I create one first?';
        }
      }
      
      // If opening app that's not installed
      if (intent.layer === 'APPLICATIONS' && intent.operation === 'ANDROID_OPEN') {
        const appName = intent.params.appName;
        const app = context.systemState.layer8_applications.androidApps.find(a => a.name === appName);
        if (app && !app.installed) {
          result.ambiguities.push(`${appName} is not installed`);
          result.alternativeInterpretations.push(`Install ${appName} first, then open it`);
        }
      }
    }
    
    return result;
  }
}

// Export singleton
export const intentClassifier = new IntentClassifier();
