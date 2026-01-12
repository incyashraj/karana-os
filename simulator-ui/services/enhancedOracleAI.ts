/**
 * Kāraṇa OS - Enhanced Oracle AI (Complete System Control)
 * 
 * This is the MASTER BRAIN that controls ALL 9 layers + cross-cutting systems.
 * The Oracle has COMPLETE omniscience over the entire system and can execute
 * any operation across all layers through natural language.
 * 
 * Architecture:
 * User → Oracle AI → System State Manager → All Layers → Hardware/Backend
 */

import { karanaApi } from './karanaService';
import { systemState, CompleteSystemState } from './systemState';
import { systemContext } from './systemContext';
import { GoogleGenAI } from "@google/genai";

// =============================================================================
// Intent Categories (Maps to System Layers)
// =============================================================================

export type LayerIntent =
  | 'HARDWARE'        // Layer 1: Camera, sensors, display, audio, power
  | 'NETWORK'         // Layer 2: Peers, sync, connections
  | 'BLOCKCHAIN'      // Layer 3: Wallet, transactions, governance
  | 'ORACLE'          // Layer 4: Self-introspection, ZK proofs
  | 'INTELLIGENCE'    // Layer 5: Vision, scene understanding
  | 'INTERFACE'       // Layer 7: HUD, voice, gestures, gaze
  | 'APPLICATIONS'    // Layer 8: Timers, navigation, settings, android apps
  | 'SYSTEM_SERVICES' // Layer 9: OTA, security, diagnostics
  | 'SPATIAL'         // Cross-cutting: AR anchors, tabs
  | 'CONVERSATION'    // General conversation
  | 'UNKNOWN';

export interface IntentAction {
  layer: LayerIntent;
  operation: string;
  params: Record<string, any>;
  requiresConfirmation: boolean;
  estimatedDuration?: number; // ms
}

export interface OracleResponse {
  message: string;
  actions: IntentAction[];
  confidence: number;
  suggestedFollowups: string[];
  systemStateAffected: string[]; // which layers will be modified
}

// =============================================================================
// Enhanced Oracle AI Class
// =============================================================================

class EnhancedOracleAI {
  private gemini: GoogleGenAI | null = null;
  private conversationHistory: Array<{ role: 'user' | 'assistant'; content: string }> = [];
  private maxHistoryLength = 20;

  constructor() {
    // Initialize Gemini if API key available
    const apiKey = (import.meta as any).env?.VITE_GEMINI_API_KEY;
    if (apiKey) {
      this.gemini = new GoogleGenAI({ apiKey });
    }
  }

  /**
   * Main entry point: Process user input with complete system awareness
   */
  async process(input: string): Promise<OracleResponse> {
    // Add to conversation history
    this.conversationHistory.push({ role: 'user', content: input });
    
    // Get complete system state
    const state = systemState.getState();
    
    // Classify intent with full context
    const intent = this.classifyIntent(input.toLowerCase(), state);
    
    // Generate execution plan
    const actions = this.planActions(intent, input, state);
    
    // Generate human-friendly response
    const message = await this.generateResponse(input, intent, actions, state);
    
    // Add AI response to history
    this.conversationHistory.push({ role: 'assistant', content: message });
    
    // Trim history if too long
    if (this.conversationHistory.length > this.maxHistoryLength) {
      this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
    }
    
    // Determine affected system layers
    const systemStateAffected = actions.map(a => a.layer).filter((v, i, a) => a.indexOf(v) === i);
    
    return {
      message,
      actions,
      confidence: intent.confidence,
      suggestedFollowups: this.generateFollowups(intent, state),
      systemStateAffected: systemStateAffected,
    };
  }

  /**
   * Classify user intent by analyzing input against current system state
   */
  private classifyIntent(
    input: string,
    state: CompleteSystemState
  ): { layer: LayerIntent; operation: string; confidence: number; params: any } {
    
    // Layer 1: Hardware Control
    if (this.matchesAny(input, ['camera', 'photo', 'picture', 'video', 'record', 'capture'])) {
      if (this.matchesAny(input, ['take', 'capture', 'snap'])) {
        return { layer: 'HARDWARE', operation: 'CAMERA_CAPTURE', confidence: 0.95, params: {} };
      }
      if (this.matchesAny(input, ['start', 'begin', 'record'])) {
        return { layer: 'HARDWARE', operation: 'CAMERA_RECORD_START', confidence: 0.95, params: {} };
      }
      if (this.matchesAny(input, ['stop'])) {
        return { layer: 'HARDWARE', operation: 'CAMERA_RECORD_STOP', confidence: 0.95, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['brightness', 'display', 'screen'])) {
      const match = input.match(/(\d+)%?/);
      const value = match ? parseInt(match[1]) / 100 : 0.5;
      return { layer: 'HARDWARE', operation: 'DISPLAY_BRIGHTNESS', confidence: 0.9, params: { value } };
    }
    
    if (this.matchesAny(input, ['battery', 'power', 'charge'])) {
      if (this.matchesAny(input, ['check', 'status', 'level', 'how much'])) {
        return { layer: 'HARDWARE', operation: 'POWER_STATUS', confidence: 0.95, params: {} };
      }
      if (this.matchesAny(input, ['save', 'saver', 'saving', 'optimize', 'low'])) {
        return { layer: 'HARDWARE', operation: 'POWER_SAVE_MODE', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['volume', 'sound', 'audio'])) {
      const match = input.match(/(\d+)%?/);
      const value = match ? parseInt(match[1]) / 100 : 0.7;
      if (this.matchesAny(input, ['up', 'increase', 'louder'])) {
        return { layer: 'HARDWARE', operation: 'AUDIO_VOLUME', confidence: 0.9, params: { value: Math.min(1, state.layer1_hardware.audio.volume + 0.1) } };
      }
      if (this.matchesAny(input, ['down', 'decrease', 'quieter', 'lower'])) {
        return { layer: 'HARDWARE', operation: 'AUDIO_VOLUME', confidence: 0.9, params: { value: Math.max(0, state.layer1_hardware.audio.volume - 0.1) } };
      }
      return { layer: 'HARDWARE', operation: 'AUDIO_VOLUME', confidence: 0.85, params: { value } };
    }
    
    // Layer 2: Network Control
    if (this.matchesAny(input, ['network', 'peer', 'connection', 'connect', 'sync'])) {
      if (this.matchesAny(input, ['status', 'check', 'how many', 'list'])) {
        return { layer: 'NETWORK', operation: 'NETWORK_STATUS', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['sync', 'synchronize'])) {
        return { layer: 'NETWORK', operation: 'BLOCKCHAIN_SYNC', confidence: 0.9, params: {} };
      }
    }
    
    // Layer 3: Blockchain Control
    if (this.matchesAny(input, ['wallet', 'balance', 'kara', 'token', 'coin'])) {
      if (this.matchesAny(input, ['create', 'new', 'setup'])) {
        return { layer: 'BLOCKCHAIN', operation: 'WALLET_CREATE', confidence: 0.95, params: {} };
      }
      if (this.matchesAny(input, ['balance', 'how much', 'check'])) {
        return { layer: 'BLOCKCHAIN', operation: 'WALLET_BALANCE', confidence: 0.95, params: {} };
      }
      if (this.matchesAny(input, ['send', 'transfer', 'pay'])) {
        // Extract amount and recipient
        const amountMatch = input.match(/(\d+(?:\.\d+)?)\s*(?:kara|token)?/i);
        const amount = amountMatch ? parseFloat(amountMatch[1]) : 0;
        
        // Try to find recipient name
        const recipientWords = ['to', 'for'];
        let recipient = '';
        for (const word of recipientWords) {
          const idx = input.indexOf(word);
          if (idx !== -1) {
            recipient = input.substring(idx + word.length).trim().split(' ')[0];
            break;
          }
        }
        
        return { 
          layer: 'BLOCKCHAIN', 
          operation: 'WALLET_TRANSFER', 
          confidence: amount > 0 ? 0.9 : 0.6, 
          params: { amount, recipient } 
        };
      }
      if (this.matchesAny(input, ['transaction', 'history', 'recent', 'past'])) {
        return { layer: 'BLOCKCHAIN', operation: 'WALLET_TRANSACTIONS', confidence: 0.9, params: {} };
      }
    }
    
    // Layer 5: Intelligence/Vision
    if (this.matchesAny(input, ['see', 'look', 'vision', 'analyze', 'identify', 'what', 'object', 'scene'])) {
      if (this.matchesAny(input, ['what', 'identify', 'recognize', 'see', 'looking at'])) {
        return { layer: 'INTELLIGENCE', operation: 'VISION_ANALYZE', confidence: 0.9, params: {} };
      }
    }
    
    // Layer 7: Interface Control
    if (this.matchesAny(input, ['hud', 'overlay', 'interface', 'display'])) {
      if (this.matchesAny(input, ['hide', 'off', 'disable'])) {
        return { layer: 'INTERFACE', operation: 'HUD_HIDE', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['show', 'on', 'enable', 'display'])) {
        return { layer: 'INTERFACE', operation: 'HUD_SHOW', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['gesture', 'hand'])) {
      if (this.matchesAny(input, ['enable', 'on', 'start', 'track'])) {
        return { layer: 'INTERFACE', operation: 'GESTURE_ENABLE', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['disable', 'off', 'stop'])) {
        return { layer: 'INTERFACE', operation: 'GESTURE_DISABLE', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['gaze', 'eye'])) {
      if (this.matchesAny(input, ['enable', 'on', 'start', 'track', 'calibrate'])) {
        return { layer: 'INTERFACE', operation: 'GAZE_ENABLE', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['disable', 'off', 'stop'])) {
        return { layer: 'INTERFACE', operation: 'GAZE_DISABLE', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['ar mode', 'augmented reality'])) {
      if (this.matchesAny(input, ['enable', 'on', 'start', 'activate'])) {
        return { layer: 'INTERFACE', operation: 'AR_MODE_ENABLE', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['disable', 'off', 'stop', 'exit'])) {
        return { layer: 'INTERFACE', operation: 'AR_MODE_DISABLE', confidence: 0.9, params: {} };
      }
    }
    
    // Layer 8: Applications
    if (this.matchesAny(input, ['timer', 'alarm', 'countdown', 'stopwatch'])) {
      if (this.matchesAny(input, ['set', 'create', 'start', 'new'])) {
        const durationMatch = input.match(/(\d+)\s*(second|minute|hour|min|sec|hr)/i);
        let durationMs = 0;
        if (durationMatch) {
          const value = parseInt(durationMatch[1]);
          const unit = durationMatch[2].toLowerCase();
          if (unit.startsWith('h')) durationMs = value * 3600000;
          else if (unit.startsWith('m')) durationMs = value * 60000;
          else if (unit.startsWith('s')) durationMs = value * 1000;
        }
        return { layer: 'APPLICATIONS', operation: 'TIMER_CREATE', confidence: 0.9, params: { durationMs } };
      }
      if (this.matchesAny(input, ['list', 'show', 'check', 'what'])) {
        return { layer: 'APPLICATIONS', operation: 'TIMER_LIST', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['stop', 'cancel', 'delete', 'remove'])) {
        return { layer: 'APPLICATIONS', operation: 'TIMER_CANCEL', confidence: 0.8, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['navigate', 'directions', 'route', 'go to', 'take me'])) {
      // Extract destination
      const toIndex = input.indexOf(' to ');
      const destination = toIndex !== -1 ? input.substring(toIndex + 4) : '';
      return { layer: 'APPLICATIONS', operation: 'NAVIGATION_START', confidence: 0.85, params: { destination } };
    }
    
    if (this.matchesAny(input, ['setting', 'config', 'preference'])) {
      return { layer: 'APPLICATIONS', operation: 'SETTINGS_OPEN', confidence: 0.8, params: {} };
    }
    
    if (this.matchesAny(input, ['wellness', 'health', 'eye strain', 'posture', 'break'])) {
      return { layer: 'APPLICATIONS', operation: 'WELLNESS_STATUS', confidence: 0.85, params: {} };
    }
    
    // Android Apps (Layer 8 sub-category)
    const appNames = ['youtube', 'whatsapp', 'instagram', 'tiktok', 'twitter', 'spotify', 'telegram', 'facebook', 'netflix', 'gmail', 'chrome', 'maps'];
    for (const appName of appNames) {
      if (input.includes(appName)) {
        const app = systemContext.findApp(appName);
        if (app) {
          if (this.matchesAny(input, ['install', 'download', 'get'])) {
            return { layer: 'APPLICATIONS', operation: 'ANDROID_INSTALL', confidence: 0.9, params: { appName: app.name } };
          } else if (this.matchesAny(input, ['open', 'launch', 'start', 'run'])) {
            return { layer: 'APPLICATIONS', operation: 'ANDROID_OPEN', confidence: 0.9, params: { appName: app.name } };
          } else if (this.matchesAny(input, ['close', 'stop', 'exit', 'quit'])) {
            return { layer: 'APPLICATIONS', operation: 'ANDROID_CLOSE', confidence: 0.9, params: { appName: app.name } };
          }
        }
        break;
      }
    }
    
    // Layer 9: System Services
    if (this.matchesAny(input, ['update', 'upgrade', 'ota'])) {
      if (this.matchesAny(input, ['check', 'available', 'new'])) {
        return { layer: 'SYSTEM_SERVICES', operation: 'OTA_CHECK', confidence: 0.9, params: {} };
      }
      if (this.matchesAny(input, ['install', 'download', 'apply'])) {
        return { layer: 'SYSTEM_SERVICES', operation: 'OTA_INSTALL', confidence: 0.85, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['security', 'privacy', 'permission'])) {
      if (this.matchesAny(input, ['mode', 'level', 'set'])) {
        if (this.matchesAny(input, ['paranoid', 'maximum', 'high', 'strict'])) {
          return { layer: 'SYSTEM_SERVICES', operation: 'SECURITY_MODE', confidence: 0.9, params: { mode: 'paranoid' } };
        } else if (this.matchesAny(input, ['relaxed', 'low', 'minimal'])) {
          return { layer: 'SYSTEM_SERVICES', operation: 'SECURITY_MODE', confidence: 0.9, params: { mode: 'relaxed' } };
        } else if (this.matchesAny(input, ['standard', 'normal', 'default'])) {
          return { layer: 'SYSTEM_SERVICES', operation: 'SECURITY_MODE', confidence: 0.9, params: { mode: 'standard' } };
        }
      }
      if (this.matchesAny(input, ['check', 'status', 'list'])) {
        return { layer: 'SYSTEM_SERVICES', operation: 'SECURITY_STATUS', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['diagnostic', 'health', 'check', 'status', 'system'])) {
      if (this.matchesAny(input, ['run', 'perform', 'execute', 'do'])) {
        return { layer: 'SYSTEM_SERVICES', operation: 'DIAGNOSTICS_RUN', confidence: 0.85, params: {} };
      }
      if (this.matchesAny(input, ['status', 'report', 'health', 'score'])) {
        return { layer: 'SYSTEM_SERVICES', operation: 'DIAGNOSTICS_STATUS', confidence: 0.9, params: {} };
      }
    }
    
    // Spatial/AR Operations
    if (this.matchesAny(input, ['anchor', 'pin', 'place', 'ar'])) {
      if (this.matchesAny(input, ['create', 'place', 'new', 'add'])) {
        return { layer: 'SPATIAL', operation: 'ANCHOR_CREATE', confidence: 0.85, params: {} };
      }
      if (this.matchesAny(input, ['list', 'show', 'all'])) {
        return { layer: 'SPATIAL', operation: 'ANCHOR_LIST', confidence: 0.9, params: {} };
      }
    }
    
    if (this.matchesAny(input, ['tab', 'window', 'browser'])) {
      if (this.matchesAny(input, ['open', 'new', 'create'])) {
        return { layer: 'SPATIAL', operation: 'TAB_OPEN', confidence: 0.85, params: {} };
      }
      if (this.matchesAny(input, ['list', 'show', 'all'])) {
        return { layer: 'SPATIAL', operation: 'TAB_LIST', confidence: 0.9, params: {} };
      }
    }
    
    // Conversational fallback
    return { layer: 'CONVERSATION', operation: 'CHAT', confidence: 0.7, params: {} };
  }

  /**
   * Plan actions based on classified intent
   */
  private planActions(
    intent: { layer: LayerIntent; operation: string; confidence: number; params: any },
    input: string,
    state: CompleteSystemState
  ): IntentAction[] {
    const actions: IntentAction[] = [];
    
    // Add primary action
    actions.push({
      layer: intent.layer,
      operation: intent.operation,
      params: intent.params,
      requiresConfirmation: this.requiresConfirmation(intent),
      estimatedDuration: this.estimateDuration(intent),
    });
    
    // Add dependent actions
    const dependencies = this.getDependencies(intent, state);
    actions.push(...dependencies);
    
    return actions;
  }

  /**
   * Check if an operation requires user confirmation
   */
  private requiresConfirmation(intent: { layer: LayerIntent; operation: string; params: any }): boolean {
    // Operations that modify blockchain or spend money
    if (intent.layer === 'BLOCKCHAIN') {
      if (intent.operation === 'WALLET_TRANSFER') return true;
      if (intent.operation === 'WALLET_CREATE') return true;
    }
    
    // Operations that modify security settings
    if (intent.layer === 'SYSTEM_SERVICES') {
      if (intent.operation === 'SECURITY_MODE') return true;
      if (intent.operation === 'OTA_INSTALL') return true;
    }
    
    // App installations
    if (intent.layer === 'APPLICATIONS' && intent.operation === 'ANDROID_INSTALL') {
      return true;
    }
    
    return false;
  }

  /**
   * Estimate how long an operation will take
   */
  private estimateDuration(intent: { layer: LayerIntent; operation: string }): number {
    if (intent.layer === 'HARDWARE' && intent.operation.startsWith('CAMERA')) return 500;
    if (intent.layer === 'INTELLIGENCE') return 1000;
    if (intent.layer === 'BLOCKCHAIN' && intent.operation === 'WALLET_CREATE') return 2000;
    if (intent.layer === 'SYSTEM_SERVICES' && intent.operation === 'DIAGNOSTICS_RUN') return 5000;
    return 200;
  }

  /**
   * Get dependent actions that should be executed first/together
   */
  private getDependencies(
    intent: { layer: LayerIntent; operation: string; params: any },
    state: CompleteSystemState
  ): IntentAction[] {
    const deps: IntentAction[] = [];
    
    // If sending KARA but no wallet, create wallet first
    if (intent.layer === 'BLOCKCHAIN' && intent.operation === 'WALLET_TRANSFER') {
      if (!state.layer3_blockchain.wallet.exists) {
        deps.push({
          layer: 'BLOCKCHAIN',
          operation: 'WALLET_CREATE',
          params: {},
          requiresConfirmation: true,
        });
      }
    }
    
    // If using vision but camera inactive, activate camera first
    if (intent.layer === 'INTELLIGENCE' && intent.operation === 'VISION_ANALYZE') {
      if (!state.layer1_hardware.camera.active) {
        deps.push({
          layer: 'HARDWARE',
          operation: 'CAMERA_ACTIVATE',
          params: {},
          requiresConfirmation: false,
        });
      }
    }
    
    // If opening Android app but not installed, install first
    if (intent.layer === 'APPLICATIONS' && intent.operation === 'ANDROID_OPEN') {
      const app = systemContext.findApp(intent.params.appName);
      if (app && !app.installed) {
        deps.push({
          layer: 'APPLICATIONS',
          operation: 'ANDROID_INSTALL',
          params: { appName: app.name },
          requiresConfirmation: true,
        });
      }
    }
    
    return deps;
  }

  /**
   * Generate human-friendly response using Gemini or fallback
   */
  private async generateResponse(
    input: string,
    intent: { layer: LayerIntent; operation: string; confidence: number; params: any },
    actions: IntentAction[],
    state: CompleteSystemState
  ): Promise<string> {
    // If Gemini available, use it for natural responses
    if (this.gemini) {
      try {
        const systemPrompt = this.buildSystemPrompt(state);
        const userPrompt = `User said: "${input}"\n\nDetected intent: ${intent.operation}\nActions planned: ${actions.map(a => a.operation).join(', ')}\n\nProvide a brief, helpful response explaining what you're about to do. Be conversational and friendly. Keep it under 50 words.`;
        
        const model = (this.gemini as any).getGenerativeModel({ model: "gemini-2.0-flash-exp" });
        const result = await model.generateContent({
          contents: [
            { role: 'user', parts: [{ text: systemPrompt }] },
            { role: 'model', parts: [{ text: 'Understood. I have complete awareness of all system layers and capabilities.' }] },
            { role: 'user', parts: [{ text: userPrompt }] }
          ],
        });
        
        return result.response.text();
      } catch (err) {
        console.warn('Gemini unavailable, using fallback responses:', err);
      }
    }
    
    // Fallback responses
    return this.generateFallbackResponse(intent, actions, state);
  }

  /**
   * Build system prompt for Gemini with complete system state
   */
  private buildSystemPrompt(state: CompleteSystemState): string {
    return `You are the Oracle AI controlling Kāraṇa OS smart glasses. You have COMPLETE omniscience over the entire system.

${systemState.getContextForAI()}

You can control EVERYTHING through natural language. Be helpful, concise, and proactive. Always let the user know what you're doing and why.`;
  }

  /**
   * Generate fallback response without Gemini
   */
  private generateFallbackResponse(
    intent: { layer: LayerIntent; operation: string; params: any },
    actions: IntentAction[],
    state: CompleteSystemState
  ): string {
    const op = intent.operation;
    
    // Hardware responses
    if (op === 'CAMERA_CAPTURE') return '📸 Capturing photo...';
    if (op === 'CAMERA_RECORD_START') return '🎥 Starting video recording...';
    if (op === 'DISPLAY_BRIGHTNESS') return `🔆 Adjusting brightness to ${(intent.params.value * 100).toFixed(0)}%`;
    if (op === 'POWER_STATUS') return `🔋 Battery at ${(state.layer1_hardware.power.batteryLevel * 100).toFixed(0)}%, ${state.layer1_hardware.power.estimatedRuntime} minutes remaining`;
    if (op === 'POWER_SAVE_MODE') return '⚡ Activating power save mode...';
    if (op === 'AUDIO_VOLUME') return `🔊 Volume set to ${(intent.params.value * 100).toFixed(0)}%`;
    
    // Network responses
    if (op === 'NETWORK_STATUS') return `🌐 Connected to ${state.layer2_network.peerCount} peers. Sync status: ${state.layer2_network.syncStatus}`;
    
    // Blockchain responses
    if (op === 'WALLET_CREATE') return '🏦 Creating your sovereign wallet with Ed25519 encryption...';
    if (op === 'WALLET_BALANCE') return `💰 Your balance: ${state.layer3_blockchain.wallet.balance} KARA`;
    if (op === 'WALLET_TRANSFER') return `💸 Preparing to send ${intent.params.amount} KARA to ${intent.params.recipient}`;
    if (op === 'WALLET_TRANSACTIONS') return `📜 You have ${state.layer3_blockchain.transactions.length} transactions`;
    
    // Intelligence responses
    if (op === 'VISION_ANALYZE') return 'Analyzing what you are looking at...';
    
    // Interface responses
    if (op === 'HUD_HIDE') return '🙈 Hiding HUD elements';
    if (op === 'HUD_SHOW') return '👀 Showing HUD elements';
    if (op === 'GESTURE_ENABLE') return '👋 Hand gesture tracking activated';
    if (op === 'GAZE_ENABLE') return '👁️ Gaze tracking activated';
    if (op === 'AR_MODE_ENABLE') return '🥽 Entering AR mode...';
    
    // Application responses
    if (op === 'TIMER_CREATE') return `⏱️ Timer set for ${intent.params.durationMs / 1000} seconds`;
    if (op === 'TIMER_LIST') return `⏱️ You have ${state.layer8_applications.timers.length} active timers`;
    if (op === 'NAVIGATION_START') return `🗺️ Navigating to ${intent.params.destination}...`;
    if (op === 'WELLNESS_STATUS') return `💚 Usage: ${state.layer8_applications.wellness.usageTime}min today, Eye strain: ${(state.layer8_applications.wellness.eyeStrain * 100).toFixed(0)}%`;
    if (op === 'ANDROID_INSTALL') return `📲 Installing ${intent.params.appName}...`;
    if (op === 'ANDROID_OPEN') return `🚀 Launching ${intent.params.appName}...`;
    
    // System services responses
    if (op === 'OTA_CHECK') return state.layer9_services.ota.updateAvailable ? `📦 Update available: v${state.layer9_services.ota.version}` : '✅ System is up to date';
    if (op === 'SECURITY_MODE') return `🔒 Security mode set to ${intent.params.mode}`;
    if (op === 'DIAGNOSTICS_STATUS') return `🏥 System health: ${(state.layer9_services.diagnostics.healthScore * 100).toFixed(0)}%`;
    if (op === 'DIAGNOSTICS_RUN') return '🔍 Running comprehensive system diagnostics...';
    
    // Spatial responses
    if (op === 'ANCHOR_CREATE') return '📍 Creating AR anchor at current location...';
    if (op === 'TAB_OPEN') return '🪟 Opening new AR tab...';
    
    return 'Processing your request...';
  }

  /**
   * Generate suggested follow-up actions
   */
  private generateFollowups(
    intent: { layer: LayerIntent; operation: string; params: any },
    state: CompleteSystemState
  ): string[] {
    const suggestions: string[] = [];
    
    if (intent.layer === 'HARDWARE' && intent.operation === 'CAMERA_CAPTURE') {
      suggestions.push('Send this photo to someone', 'Take another photo', 'Record a video');
    } else if (intent.layer === 'BLOCKCHAIN' && intent.operation === 'WALLET_CREATE') {
      suggestions.push('Check my balance', 'Send KARA to someone', 'View transactions');
    } else if (intent.layer === 'INTELLIGENCE' && intent.operation === 'VISION_ANALYZE') {
      suggestions.push('Tell me more about this', 'Take a photo', 'Search online');
    } else if (intent.layer === 'APPLICATIONS' && intent.operation === 'TIMER_CREATE') {
      suggestions.push('List all timers', 'Cancel timer', 'Set another timer');
    }
    
    return suggestions;
  }

  /**
   * Helper: Check if input matches any of the given keywords
   */
  private matchesAny(input: string, keywords: string[]): boolean {
    return keywords.some(keyword => input.includes(keyword));
  }
}

// Export singleton instance
export const enhancedOracle = new EnhancedOracleAI();
