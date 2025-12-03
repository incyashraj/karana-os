/**
 * Kāraṇa OS Oracle - AI Bridge Layer
 * 
 * Architecture: User → AI Oracle → Blockchain
 * 
 * This service is the INTERFACE between natural language and the OS.
 * ALL operations go through the Rust backend (which is on blockchain).
 * 
 * Flow:
 * 1. User speaks/types → Frontend captures
 * 2. Frontend sends to Rust Oracle (/api/ai/oracle)
 * 3. Rust Oracle parses intent, returns structured response
 * 4. Frontend executes the action (open app, sign tx, etc.)
 * 5. For knowledge queries, use Gemini AI for real answers
 */

import { karanaApi, OracleIntentResponse } from './karanaService';
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
   * Routes to Rust backend for OS operations, Gemini for knowledge
   */
  async process(userInput: string, context?: OracleContext): Promise<OracleResponse> {
    const input = userInput.trim();
    
    // Add to conversation history
    this.conversationHistory.push({ role: 'user', content: input });
    if (this.conversationHistory.length > 20) {
      this.conversationHistory = this.conversationHistory.slice(-20);
    }
    
    // First, try to detect if this is a knowledge query vs OS command
    const isKnowledgeQuery = this.isKnowledgeQuery(input);
    
    try {
      // Route through Rust backend for OS operations
      const backendResponse = await karanaApi.processOracle(input, {
        vision_object: context?.visionObject,
        wallet_balance: context?.walletBalance,
        active_app: context?.currentApp
      });
      
      // Map backend response to our format
      const response = this.mapBackendResponse(backendResponse);
      
      // If it's a knowledge query or low confidence, enhance with Gemini
      if (isKnowledgeQuery || (response.confidence < 0.7 && response.intent.type === 'SPEAK')) {
        return await this.enhanceWithGemini(input, response, context);
      }
      
      this.conversationHistory.push({ role: 'assistant', content: response.message });
      return response;
      
    } catch (error) {
      console.warn('Backend Oracle unavailable, using local processing:', error);
      
      // Fallback to local processing if backend is down
      return this.processLocally(input, context);
    }
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
      /how (do i|to) (send|transfer|pay)/,
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
    context?: OracleContext
  ): Promise<OracleResponse> {
    if (!this.gemini) {
      return backendResponse;
    }
    
    try {
      const systemPrompt = `You are the Kāraṇa OS Oracle - an AI assistant for a blockchain-powered smart glasses operating system.

You're helpful, concise, and knowledgeable. Keep responses under 150 words unless detailed explanations are needed.

Current context:
- User's KARA balance: ${context?.walletBalance || 'unknown'}
- Looking at: ${context?.visionObject || 'nothing specific'}
- Current app: ${context?.currentApp || 'none'}

Capabilities you can mention:
- Send/receive KARA tokens via voice
- Open AR apps (browser, video, notes, music, terminal)
- Analyze what user is looking at
- Set reminders and timers
- Navigate to places
- Vote on DAO proposals

Answer naturally and helpfully.`;

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
