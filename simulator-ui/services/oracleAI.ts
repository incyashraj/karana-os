/**
 * Kāraṇa OS Oracle - AI Bridge Layer
 * 
 * Architecture: User → AI Oracle → System Actions
 * 
 * This is the BRAIN of Kāraṇa OS. It understands EVERYTHING:
 * - Available apps, installed apps, running apps
 * - System capabilities (vision, wallet, AR, etc.)
 * - User intent from natural language
 * - Can install, launch, and manage apps automatically
 * 
 * Flow:
 * 1. User speaks/types → Oracle analyzes with full system context
 * 2. Oracle decides what action to take (install app, launch, analyze vision, etc.)
 * 3. Oracle executes or asks for permission
 * 4. Oracle responds to user with results
 */

import { karanaApi, OracleIntentResponse } from './karanaService';
import { systemContext, AndroidApp } from './systemContext';
import { GoogleGenAI } from "@google/genai";

// ============================================================================
// Types - Matching the Rust backend
// ============================================================================

export interface OracleContext {
  walletBalance?: number;
  walletDid?: string;
  visionObject?: string;
  visionDescription?: string;
  currentApp?: string;
  previousIntent?: string;
  conversationHistory?: string[];
}

export interface OracleResponse {
  intent: {
    type: string;
    category: IntentCategory;
  };
  message: string;
  data?: Record<string, any>;
  requiresConfirmation: boolean;
  suggestedActions: string[];
  confidence: number;
}

export type IntentCategory = 
  | 'BLOCKCHAIN'
  | 'AR_APP' 
  | 'PRODUCTIVITY'
  | 'VISION'
  | 'NAVIGATION'
  | 'SYSTEM'
  | 'KNOWLEDGE';

// ============================================================================
// Gemini AI for Knowledge Queries
// ============================================================================

const getGeminiAI = () => {
  const apiKey = (import.meta as any).env?.VITE_GEMINI_API_KEY;
  if (!apiKey) {
    console.warn('Gemini API key not found');
    return null;
  }
  return new GoogleGenAI({ apiKey });
};

const GEMINI_MODEL = "gemini-2.0-flash";

// ============================================================================
// Oracle AI Class - The Bridge
// ============================================================================

class OracleAI {
  private gemini: GoogleGenAI | null = null;
  private conversationHistory: Array<{ role: string; content: string }> = [];
  
  constructor() {
    this.gemini = getGeminiAI();
  }
  
  /**
   * Process user input through the Oracle
   * With FULL system awareness - knows about all apps, capabilities, state
   */
  async process(userInput: string, context?: OracleContext): Promise<OracleResponse> {
    const input = userInput.trim();
    
    // Add to conversation history
    this.conversationHistory.push({ role: 'user', content: input });
    if (this.conversationHistory.length > 20) {
      this.conversationHistory = this.conversationHistory.slice(-20);
    }
    
    // Get full system context
    const systemInfo = systemContext.getContextForAI();
    
    // Check if this is an app-related request
    const appIntent = this.detectAppIntent(input);
    if (appIntent) {
      return await this.handleAppIntent(appIntent, input);
    }
    
    // Check for other system capabilities
    const capabilityIntent = this.detectCapabilityIntent(input);
    if (capabilityIntent) {
      return this.handleCapabilityIntent(capabilityIntent, input);
    }
    
    // Check if this is a knowledge query vs OS command
    const isKnowledgeQuery = this.isKnowledgeQuery(input);
    
    try {
      // Route through Rust backend for OS operations with full context
      const backendResponse = await karanaApi.processOracle(input, {
        vision_object: context?.visionObject,
        wallet_balance: context?.walletBalance,
        active_app: context?.currentApp,
        system_context: systemInfo // Pass full system context to backend
      });
      
      // Map backend response to our format
      const response = this.mapBackendResponse(backendResponse);
      
      // If it's a knowledge query or low confidence, enhance with Gemini
      if (isKnowledgeQuery || (response.confidence < 0.7 && response.intent.type === 'SPEAK')) {
        return await this.enhanceWithGemini(input, response, context, systemInfo);
      }
      
      this.conversationHistory.push({ role: 'assistant', content: response.message });
      return response;
      
    } catch (error) {
      console.warn('Backend Oracle unavailable, using local processing:', error);
      
      // Fallback to local processing with system context
      return this.processLocallyWithContext(input, context, systemInfo);
    }
  }

  /**
   * Detect if user wants to do something with an app
   */
  private detectAppIntent(input: string): { action: string; app: AndroidApp | null; query: string } | null {
    const text = input.toLowerCase();
    
    // Detect action
    let action: string | null = null;
    if (/\b(open|launch|start|run|show|use)\b/.test(text)) {
      action = 'launch';
    } else if (/\b(install|download|get|add)\b/.test(text)) {
      action = 'install';
    } else if (/\b(close|exit|quit|stop)\b/.test(text)) {
      action = 'close';
    } else if (/\b(uninstall|remove|delete)\b/.test(text)) {
      action = 'uninstall';
    }
    
    if (!action) return null;
    
    // Extract app name - try to find known apps
    let foundApp: AndroidApp | null = null;
    const allApps = systemContext.getAllApps();
    
    for (const app of allApps) {
      const appName = app.name.toLowerCase();
      const appId = app.id.toLowerCase();
      
      if (text.includes(appName) || text.includes(appId)) {
        foundApp = app;
        break;
      }
      
      // Check capabilities/keywords
      for (const cap of app.capabilities) {
        if (text.includes(cap)) {
          foundApp = app;
          break;
        }
      }
      
      if (foundApp) break;
    }
    
    return foundApp ? { action, app: foundApp, query: input } : null;
  }

  /**
   * Handle app-related intents with automatic installation
   */
  private async handleAppIntent(intent: { action: string; app: AndroidApp; query: string }): Promise<OracleResponse> {
    const { action, app, query } = intent;
    
    systemContext.addActivity(`App intent: ${action} ${app.name}`);
    
    // Handle based on action
    switch (action) {
      case 'launch':
        if (!app.installed) {
          // App not installed - offer to install
          return {
            intent: { type: 'INSTALL_APP', category: 'AR_APP' },
            message: `${app.name} is not installed. Would you like me to install it first? It's ${app.description.toLowerCase()}`,
            data: { 
              appId: app.id, 
              appName: app.name,
              needsInstall: true,
              autoInstall: true // Flag for auto-installation
            },
            requiresConfirmation: true,
            suggestedActions: [
              `Yes, install ${app.name}`,
              'No thanks',
              'Tell me more about this app'
            ],
            confidence: 0.95
          };
        } else if (app.running) {
          // Already running
          return {
            intent: { type: 'APP_INFO', category: 'AR_APP' },
            message: `${app.name} is already running! Bringing it to focus.`,
            data: { appId: app.id, appName: app.name, alreadyRunning: true },
            requiresConfirmation: false,
            suggestedActions: [],
            confidence: 1.0
          };
        } else {
          // Installed, ready to launch
          return {
            intent: { type: 'OPEN_APP', category: 'AR_APP' },
            message: `Launching ${app.name}...`,
            data: { appId: app.id, appName: app.name, launch: true },
            requiresConfirmation: false,
            suggestedActions: [],
            confidence: 1.0
          };
        }
        
      case 'install':
        if (app.installed) {
          return {
            intent: { type: 'APP_INFO', category: 'AR_APP' },
            message: `${app.name} is already installed! Would you like me to launch it?`,
            data: { appId: app.id, appName: app.name, alreadyInstalled: true },
            requiresConfirmation: false,
            suggestedActions: [`Launch ${app.name}`, 'No thanks'],
            confidence: 1.0
          };
        } else {
          return {
            intent: { type: 'INSTALL_APP', category: 'AR_APP' },
            message: `Installing ${app.name}... This will enable: ${app.description.toLowerCase()}`,
            data: { appId: app.id, appName: app.name, install: true },
            requiresConfirmation: false,
            suggestedActions: [],
            confidence: 1.0
          };
        }
        
      case 'close':
        if (!app.running) {
          return {
            intent: { type: 'APP_INFO', category: 'AR_APP' },
            message: `${app.name} is not currently running.`,
            data: { appId: app.id, appName: app.name },
            requiresConfirmation: false,
            suggestedActions: [`Launch ${app.name}`],
            confidence: 1.0
          };
        } else {
          return {
            intent: { type: 'CLOSE_APP', category: 'AR_APP' },
            message: `Closing ${app.name}...`,
            data: { appId: app.id, appName: app.name, close: true },
            requiresConfirmation: false,
            suggestedActions: [],
            confidence: 1.0
          };
        }
        
      case 'uninstall':
        if (!app.installed) {
          return {
            intent: { type: 'APP_INFO', category: 'AR_APP' },
            message: `${app.name} is not installed.`,
            data: { appId: app.id, appName: app.name },
            requiresConfirmation: false,
            suggestedActions: [`Install ${app.name}`],
            confidence: 1.0
          };
        } else {
          return {
            intent: { type: 'UNINSTALL_APP', category: 'AR_APP' },
            message: `Are you sure you want to uninstall ${app.name}?`,
            data: { appId: app.id, appName: app.name, uninstall: true },
            requiresConfirmation: true,
            suggestedActions: ['Yes, uninstall', 'No, keep it'],
            confidence: 1.0
          };
        }
    }
    
    return {
      intent: { type: 'SPEAK', category: 'SYSTEM' },
      message: `I understand you want to ${action} ${app.name}, but something went wrong.`,
      requiresConfirmation: false,
      suggestedActions: [],
      confidence: 0.5
    };
  }

  /**
   * Detect system capability intents
   */
  private detectCapabilityIntent(input: string): { capability: string; action: string } | null {
    const text = input.toLowerCase();
    
    // Vision/Analysis
    if (/\b(analyze|scan|look|see|identify|recognize|what is this|what's this)\b/.test(text)) {
      return { capability: 'vision', action: 'analyze' };
    }
    
    // Wallet
    if (/\b(wallet|balance|send|transfer|pay|money|funds|kara)\b/.test(text)) {
      return { capability: 'wallet', action: 'show' };
    }
    
    // AR Workspace
    if (/\b(workspace|create object|place|ar|3d)\b/.test(text)) {
      return { capability: 'ar_workspace', action: 'open' };
    }
    
    // Settings
    if (/\b(settings|preferences|configure|setup)\b/.test(text)) {
      return { capability: 'settings', action: 'open' };
    }
    
    return null;
  }

  /**
   * Handle system capability intents
   */
  private handleCapabilityIntent(intent: { capability: string; action: string }, input: string): OracleResponse {
    systemContext.addActivity(`Capability intent: ${intent.capability} - ${intent.action}`);
    
    switch (intent.capability) {
      case 'vision':
        return {
          intent: { type: 'ANALYZE', category: 'VISION' },
          message: 'Activating vision analysis...',
          data: { capability: 'vision' },
          requiresConfirmation: false,
          suggestedActions: [],
          confidence: 0.9
        };
        
      case 'wallet':
        return {
          intent: { type: 'WALLET', category: 'BLOCKCHAIN' },
          message: 'Opening your sovereign wallet...',
          data: { capability: 'wallet' },
          requiresConfirmation: false,
          suggestedActions: ['Check balance', 'Send KARA', 'View transactions'],
          confidence: 0.9
        };
        
      case 'ar_workspace':
        return {
          intent: { type: 'OPEN_APP', category: 'AR_APP' },
          message: 'Opening AR Workspace...',
          data: { capability: 'ar_workspace', mode: 'AR_WORKSPACE' },
          requiresConfirmation: false,
          suggestedActions: [],
          confidence: 0.9
        };
        
      case 'settings':
        return {
          intent: { type: 'SYSTEM', category: 'SYSTEM' },
          message: 'Opening system settings...',
          data: { capability: 'settings', showSettings: true },
          requiresConfirmation: false,
          suggestedActions: [],
          confidence: 0.9
        };
    }
    
    return {
      intent: { type: 'SPEAK', category: 'SYSTEM' },
      message: 'I can help with that!',
      requiresConfirmation: false,
      suggestedActions: [],
      confidence: 0.5
    };
  }

  /**
   * Fallback local processing with full system context
   */
  private async processLocallyWithContext(input: string, context?: OracleContext, systemInfo?: string): Promise<OracleResponse> {
    // Try Gemini with system context if available
    if (this.gemini && systemInfo) {
      return await this.enhanceWithGemini(input, {
        intent: { type: 'SPEAK', category: 'KNOWLEDGE' },
        message: 'Let me help you with that...',
        requiresConfirmation: false,
        suggestedActions: [],
        confidence: 0.5
      }, context, systemInfo);
    }
    
    // Ultimate fallback
    return {
      intent: { type: 'SPEAK', category: 'SYSTEM' },
      message: 'Backend is currently offline. Some features may be unavailable.',
      requiresConfirmation: false,
      suggestedActions: ['Check connection', 'Try again'],
      confidence: 0.3
    };
  }

  /**
   * Check if this is a knowledge/question query vs an OS command
   */
  private isKnowledgeQuery(input: string): boolean {
    const text = input.toLowerCase();
    
    // Question patterns
    const questionPatterns = [
      /^(what|who|where|when|why|how|is|are|can|could|would|will|do|does|did)\b/,
      /\?$/,
      /^(explain|describe|tell me about|define)/,
      /^(calculate|compute|solve)/,
    ];
    
    // Exclude OS-specific question patterns
    const osPatterns = [
      /what('s| is) my balance/,
      /how much (kara|money|funds)/,
      /what('s| is) (this|that)/, // vision
      /how (do i|to) (send|transfer|pay|open|install)/,
    ];
    
    // Check if it matches OS patterns first
    for (const pattern of osPatterns) {
      if (pattern.test(text)) return false;
    }
    
    // Check if it's a general question
    for (const pattern of questionPatterns) {
      if (pattern.test(text)) return true;
    }
    
    return false;
  }
  
  /**
   * Map the Rust backend response to our frontend format
   */
  private mapBackendResponse(backendResponse: OracleIntentResponse): OracleResponse {
    const category = this.categorizeIntent(backendResponse.intent_type);
    
    return {
      intent: {
        type: backendResponse.intent_type,
        category
      },
      message: backendResponse.content,
      data: backendResponse.data ? {
        amount: backendResponse.data.amount,
        recipient: backendResponse.data.recipient,
        url: backendResponse.data.url,
        query: backendResponse.data.query,
        duration: backendResponse.data.duration,
        appType: backendResponse.data.app_type,
        memo: backendResponse.data.memo,
        location: backendResponse.data.location,
        ...this.parseTimerData(backendResponse.data)
      } : undefined,
      requiresConfirmation: backendResponse.requires_confirmation,
      suggestedActions: backendResponse.suggested_actions,
      confidence: backendResponse.confidence
    };
  }
  
  /**
   * Parse timer-specific data from duration strings
   */
  private parseTimerData(data: any): Record<string, any> {
    if (!data.duration) return {};
    
    const duration = data.duration.toLowerCase();
    let durationMs = 0;
    
    // Parse "X minutes", "X hours", "X seconds"
    const match = duration.match(/(\d+)\s*(second|minute|hour|min|sec|hr)/i);
    if (match) {
      const num = parseInt(match[1]);
      const unit = match[2].toLowerCase();
      
      if (unit.startsWith('sec')) durationMs = num * 1000;
      else if (unit.startsWith('min')) durationMs = num * 60 * 1000;
      else if (unit.startsWith('hour') || unit === 'hr') durationMs = num * 60 * 60 * 1000;
    }
    
    return {
      durationMs,
      label: data.query || data.memo || duration
    };
  }
  
  /**
   * Categorize intent type
   */
  private categorizeIntent(intentType: string): IntentCategory {
    const blockchainIntents = ['TRANSFER', 'WALLET', 'STAKE', 'VOTE'];
    const arAppIntents = ['OPEN_APP', 'CLOSE_APP', 'PLAY_VIDEO', 'OPEN_BROWSER', 'TAKE_NOTE', 'PLAY_MUSIC'];
    const productivityIntents = ['SET_REMINDER', 'TIMER', 'NAVIGATE'];
    const visionIntents = ['ANALYZE', 'SCAN'];
    const systemIntents = ['HELP', 'STATUS'];
    
    if (blockchainIntents.some(i => intentType.includes(i))) return 'BLOCKCHAIN';
    if (arAppIntents.some(i => intentType.includes(i))) return 'AR_APP';
    if (productivityIntents.some(i => intentType.includes(i))) return 'PRODUCTIVITY';
    if (visionIntents.some(i => intentType.includes(i))) return 'VISION';
    if (systemIntents.some(i => intentType.includes(i))) return 'SYSTEM';
    
    return 'KNOWLEDGE';
  }
  
  /**
   * Enhance response with Gemini for knowledge queries
   */
  private async enhanceWithGemini(
    input: string, 
    backendResponse: OracleResponse,
    context?: OracleContext,
    systemInfo?: string
  ): Promise<OracleResponse> {
    if (!this.gemini) {
      return backendResponse;
    }
    
    try {
      const systemPrompt = `You are the Kāraṇa OS Oracle - the BRAIN of a revolutionary AR glasses operating system.

You have COMPLETE awareness of the system:

${systemInfo || 'System information unavailable'}

Current Context:
- User's KARA balance: ${context?.walletBalance || 'unknown'}
- Looking at: ${context?.visionObject || 'nothing specific'}
- Current app: ${context?.currentApp || 'none'}

Your Capabilities:
- Install & launch ANY Android app (YouTube, WhatsApp, Instagram, TikTok, Spotify, etc.)
- Send/receive KARA tokens via voice
- Analyze what user is looking at in real-time
- Set reminders and timers
- Navigate to places
- Open AR workspace for 3D creation
- Control system settings
- Answer questions about anything

Be proactive! If user asks to open an app that's not installed, offer to install it automatically.
Keep responses natural, concise (under 150 words), and actionable.`;

      const response = await this.gemini.models.generateContent({
        model: GEMINI_MODEL,
        contents: [
          { role: 'user', parts: [{ text: systemPrompt }] },
          ...this.conversationHistory.slice(-6).map(msg => ({
            role: msg.role === 'user' ? 'user' : 'model',
            parts: [{ text: msg.content }]
          })),
          { role: 'user', parts: [{ text: input }] }
        ],
      });
      
      const aiMessage = response.text || backendResponse.message;
      
      this.conversationHistory.push({ role: 'assistant', content: aiMessage });
      
      return {
        ...backendResponse,
        intent: {
          type: 'ANSWER_QUESTION',
          category: 'KNOWLEDGE'
        },
        message: aiMessage,
        confidence: 0.85
      };
      
    } catch (error) {
      console.error('Gemini API error:', error);
      return backendResponse;
    }
  }
  
  /**
   * Local processing fallback when backend is unavailable
   */
  private async processLocally(input: string, context?: OracleContext): Promise<OracleResponse> {
    const text = input.toLowerCase();
    
    // Basic intent detection
    
    // Transfer
    if (/send|transfer|pay/.test(text) && /\d+/.test(text) && /to\s+\w+/.test(text)) {
      const amountMatch = text.match(/(\d+)/);
      const recipientMatch = text.match(/to\s+(\w+)/);
      
      return {
        intent: { type: 'TRANSFER', category: 'BLOCKCHAIN' },
        message: `Ready to send ${amountMatch?.[1]} KARA to ${recipientMatch?.[1]}`,
        data: {
          amount: amountMatch ? parseInt(amountMatch[1]) : 0,
          recipient: recipientMatch?.[1] || 'unknown'
        },
        requiresConfirmation: true,
        suggestedActions: ['Confirm', 'Cancel'],
        confidence: 0.9
      };
    }
    
    // Balance check
    if (/balance|how much|funds|kara/.test(text) && /check|show|my|have/.test(text)) {
      return {
        intent: { type: 'CHECK_BALANCE', category: 'BLOCKCHAIN' },
        message: `Your balance is ${context?.walletBalance || 0} KARA`,
        requiresConfirmation: false,
        suggestedActions: ['Send funds', 'View history'],
        confidence: 0.9
      };
    }
    
    // Timer
    if (/timer|remind|alarm/.test(text)) {
      const match = text.match(/(\d+)\s*(second|minute|hour|min|sec)/i);
      const durationMs = match ? this.parseDuration(match[0]) : 5 * 60 * 1000;
      
      return {
        intent: { type: 'SET_TIMER', category: 'PRODUCTIVITY' },
        message: `Setting timer for ${match?.[0] || '5 minutes'}`,
        data: { durationMs, duration: match?.[0] || '5 minutes' },
        requiresConfirmation: false,
        suggestedActions: ['Cancel timer'],
        confidence: 0.85
      };
    }
    
    // Open browser
    if (/open.*(browser|web)|browse/.test(text)) {
      return {
        intent: { type: 'OPEN_BROWSER', category: 'AR_APP' },
        message: 'Opening browser',
        requiresConfirmation: false,
        suggestedActions: [],
        confidence: 0.9
      };
    }
    
    // Play video
    if (/play.*(video|youtube)|watch/.test(text)) {
      return {
        intent: { type: 'PLAY_VIDEO', category: 'AR_APP' },
        message: 'Opening video player',
        requiresConfirmation: false,
        suggestedActions: [],
        confidence: 0.9
      };
    }
    
    // Help
    if (/help|what can you|commands/.test(text)) {
      return {
        intent: { type: 'HELP', category: 'SYSTEM' },
        message: this.getHelpText(),
        requiresConfirmation: false,
        suggestedActions: ['Send payment', 'Open browser', 'Set timer'],
        confidence: 0.95
      };
    }
    
    // Try Gemini for knowledge queries
    if (this.gemini) {
      return this.enhanceWithGemini(input, {
        intent: { type: 'CONVERSATION', category: 'KNOWLEDGE' },
        message: input,
        requiresConfirmation: false,
        suggestedActions: [],
        confidence: 0.5
      }, context);
    }
    
    // Default fallback
    return {
      intent: { type: 'CONVERSATION', category: 'KNOWLEDGE' },
      message: `I understood: "${input}". Try "help" for available commands, or connect to the Kāraṇa backend for full functionality.`,
      requiresConfirmation: false,
      suggestedActions: ['Help', 'Check balance', 'Open browser'],
      confidence: 0.4
    };
  }
  
  /**
   * Parse duration string to milliseconds
   */
  private parseDuration(duration: string): number {
    const match = duration.match(/(\d+)\s*(second|minute|hour|min|sec|hr)/i);
    if (!match) return 5 * 60 * 1000; // Default 5 minutes
    
    const num = parseInt(match[1]);
    const unit = match[2].toLowerCase();
    
    if (unit.startsWith('sec')) return num * 1000;
    if (unit.startsWith('min')) return num * 60 * 1000;
    if (unit.startsWith('hour') || unit === 'hr') return num * 60 * 60 * 1000;
    
    return num * 60 * 1000;
  }
  
  /**
   * Get help text
   */
  private getHelpText(): string {
    return `🔮 **Kāraṇa OS Oracle**

💰 **Blockchain**
• "Send 50 KARA to alice"
• "Check my balance"
• "Show transactions"

🖥️ **AR Apps**
• "Open browser"
• "Play a video"
• "Take a note"
• "Open terminal"

⏰ **Timers**
• "Set timer for 5 minutes"
• "Remind me in 30 minutes"

👁️ **Vision**
• "What is this?"
• "Analyze what I see"

🧭 **Navigation**
• "Navigate to Central Park"

Just speak naturally!`;
  }
  
  /**
   * Get conversation history
   */
  getHistory() {
    return this.conversationHistory;
  }
  
  /**
   * Clear conversation history
   */
  clearHistory() {
    this.conversationHistory = [];
  }
}

// Export singleton
export const oracleAI = new OracleAI();
