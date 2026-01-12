/**
 * Kāraṇa OS - Intelligent Command Router
 * 
 * HYBRID INTELLIGENCE ARCHITECTURE:
 * 
 * Tier 1: Pattern-Based OS Commands (100% reliable, offline)
 * - Camera, battery, display, wallet, apps, timers, etc.
 * - Fast, deterministic, always works
 * - No AI dependency for critical functions
 * 
 * Tier 2: Entity Extraction (Smart parsing)
 * - Numbers, contacts, times, durations, amounts
 * - Makes commands flexible and natural
 * 
 * Tier 3: Real-Time Internet Services
 * - News (location-aware)
 * - Weather forecasts
 * - Web search
 * - Time/date/timezone
 * - General knowledge
 * 
 * Tier 4: Optional Cloud AI (User choice)
 * - Complex reasoning
 * - Advanced natural language
 * - Only if user enables it
 */

import { systemState } from './systemState';
import { systemContext } from './systemContext';
import { edgeIntelligence, type EdgeResult } from './edgeIntelligence';
import { comprehensiveAI } from './comprehensiveAI';
import { 
  locationService, 
  newsService, 
  weatherService, 
  webSearchService, 
  timeService,
  knowledgeService,
  type NewsArticle,
  type WeatherData,
  type SearchResult
} from './realTimeServices';

// =============================================================================
// TYPES
// =============================================================================

export interface CommandIntent {
  // What layer of OS
  layer: 'HARDWARE' | 'NETWORK' | 'BLOCKCHAIN' | 'INTELLIGENCE' | 
         'INTERFACE' | 'APPLICATIONS' | 'SYSTEM_SERVICES' | 'SPATIAL' | 'KNOWLEDGE';
  
  // Specific operation
  operation: string;
  
  // Extracted parameters
  params: Record<string, any>;
  
  // Confidence (0-1)
  confidence: number;
  
  // Needs confirmation?
  requiresConfirmation: boolean;
}

export interface CommandResponse {
  // Natural language response
  message: string;
  
  // Actions to execute
  intents: CommandIntent[];
  
  // Follow-up suggestions
  suggestions: string[];
  
  // Need more info? (optional, defaults to false)
  needsClarification?: boolean;
  clarificationQuestion?: string;
  
  // Raw data (for UI)
  data?: any;
}

// =============================================================================
// ENTITY EXTRACTION
// =============================================================================

class EntityExtractor {
  
  extractNumber(text: string): number | null {
    const match = text.match(/\b(\d+\.?\d*)\b/);
    return match ? parseFloat(match[1]) : null;
  }
  
  extractAmount(text: string): number | null {
    // "5 KARA", "10 tokens", "2.5 coins"
    const match = text.match(/\b(\d+\.?\d*)\s*(kara|token|coin|dollar|usd)?/i);
    return match ? parseFloat(match[1]) : null;
  }
  
  extractDuration(text: string): number | null {
    // "5 minutes", "2 hours", "30 seconds", "1.5 hrs"
    const match = text.match(/\b(\d+\.?\d*)\s*(second|sec|minute|min|hour|hr|h)s?\b/i);
    if (!match) return null;
    
    const value = parseFloat(match[1]);
    const unit = match[2].toLowerCase();
    
    // Convert to seconds
    if (unit.startsWith('h')) return value * 3600;
    if (unit.startsWith('m')) return value * 60;
    return value;
  }
  
  extractPercentage(text: string): number | null {
    // "50%", "75 percent"
    const match = text.match(/\b(\d+\.?\d*)\s*%|percent\b/i);
    if (!match) return null;
    
    const value = parseFloat(match[1]);
    return Math.min(100, Math.max(0, value)) / 100; // Normalize to 0-1
  }
  
  extractContact(text: string): string | null {
    // "to mom", "to john", "to alice"
    const match = text.match(/\b(?:to|for)\s+(\w+)\b/i);
    return match ? match[1].toLowerCase() : null;
  }
  
  extractAppName(text: string): string | null {
    // Match known apps
    const apps = systemContext.getAllApps();
    const textLower = text.toLowerCase();
    
    for (const app of apps) {
      if (textLower.includes(app.name.toLowerCase())) {
        return app.name;
      }
      
      // Check aliases
      for (const capability of app.capabilities) {
        if (textLower.includes(capability.toLowerCase())) {
          return app.name;
        }
      }
    }
    
    return null;
  }
  
  extractTimerName(text: string): string | null {
    // "cooking timer", "workout timer", "meeting timer"
    const match = text.match(/(\w+)\s+timer/i);
    return match ? match[1].toLowerCase() : null;
  }
}

const entityExtractor = new EntityExtractor();

// =============================================================================
// PATTERN-BASED COMMAND ROUTER
// =============================================================================

export class IntelligentRouter {
  
  private conversationHistory: Array<{ input: string; timestamp: number }> = [];
  private userPreferences: Record<string, any> = {};
  
  /**
   * Main entry point: Route any user command
   */
  async route(userInput: string): Promise<CommandResponse> {
    const text = userInput.toLowerCase().trim();
    
    console.log('[IntelligentRouter] Processing:', userInput);
    
    // Store in conversation history
    this.conversationHistory.push({ input: userInput, timestamp: Date.now() });
    if (this.conversationHistory.length > 10) this.conversationHistory.shift();
    
    // TIER -1: Comprehensive AI (for complex reasoning queries)
    // This handles questions that require actual intelligent reasoning
    if (this.requiresComprehensiveReasoning(text)) {
      console.log('[IntelligentRouter] → Comprehensive AI (INTELLIGENT REASONING)');
      try {
        const aiResponse = await comprehensiveAI.processQuery(userInput);
        
        return {
          message: `${aiResponse.answer}\n\n💡 *${aiResponse.reasoning}*`,
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'COMPREHENSIVE_REASONING',
            params: { confidence: aiResponse.confidence },
            confidence: aiResponse.confidence,
            requiresConfirmation: false
          }],
          suggestions: aiResponse.followUpSuggestions,
          data: { 
            reasoning: aiResponse.reasoning,
            sources: aiResponse.sources,
            relatedTopics: aiResponse.relatedTopics
          }
        };
      } catch (e) {
        console.log('[IntelligentRouter] Comprehensive AI failed, continuing:', e);
      }
    }
    
    // TIER 0: Edge Intelligence (FREE, FAST, SMART)
    // Let edge AI handle complex queries first - it's free and uses real APIs!
    if (this.shouldUseEdgeIntelligence(text)) {
      console.log('[IntelligentRouter] → Edge Intelligence (FREE)');
      try {
        const edgeResult = await edgeIntelligence.process(userInput);
        
        // Convert edge result to our format
        if (edgeResult.type === 'weather' || edgeResult.type === 'news' || 
            edgeResult.type === 'search' || edgeResult.type === 'location' || 
            edgeResult.type === 'time') {
          return {
            message: edgeResult.message,
            intents: [{
              layer: 'INTELLIGENCE',
              operation: edgeResult.type.toUpperCase(),
              params: { data: edgeResult.data },
              confidence: 0.95,
              requiresConfirmation: false
            }],
            suggestions: this.getSuggestionsForType(edgeResult.type),
            data: edgeResult.data
          };
        }
      } catch (e) {
        console.log('[IntelligentRouter] Edge intelligence failed, falling back:', e);
      }
    }
    
    // Try multi-intent parsing first (handle complex commands)
    const multiIntent = this.parseMultiIntent(text, userInput);
    if (multiIntent) {
      console.log('[IntelligentRouter] Multi-intent command:', multiIntent.intents.length, 'actions');
      return multiIntent;
    }
    
    // Try pattern-based OS commands (fast, reliable)
    const osCommand = this.matchOSCommand(text, userInput);
    if (osCommand) {
      console.log('[IntelligentRouter] Matched OS command:', osCommand.intents[0]?.operation);
      return osCommand;
    }
    
    // Try real-time internet services (news, weather, search, time)
    const internetService = await this.matchInternetService(text, userInput);
    if (internetService) {
      console.log('[IntelligentRouter] Internet service matched');
      return internetService;
    }
    
    // Check if it's a general knowledge question
    if (this.isKnowledgeQuery(text)) {
      console.log('[IntelligentRouter] Knowledge query detected');
      return await this.handleKnowledgeQuery(userInput);
    }
    
    // Fallback: conversational response
    return this.handleConversational(text);
  }
  
  /**
   * Check if query requires comprehensive AI reasoning
   */
  private requiresComprehensiveReasoning(text: string): boolean {
    // Questions that need actual intelligent reasoning (not just pattern matching)
    const reasoningPatterns = [
      // Decision-making
      /\b(should i|do i need|is it good|is it bad|recommended|advise|suggest)\b/i,
      // Practical advice
      /\b(what (should|can|could) i|how (should|can|could) i)\b/i,
      // Timing & optimization
      /\b(best time|good time|when (should|to)|right time)\b/i,
      // Comparisons & choices
      /\b(better to|which is|should i.*or|versus|vs|compare)\b/i,
      // Health & wellness
      /\b(healthy|exercise|rest|break|tired|wellness|fitness)\b/i,
      // Food & dining
      /\b(eat|food|meal|restaurant|hungry|cuisine|cooking)\b/i,
      // Travel & transportation
      /\b(go to|travel|transport|drive|walk|commute|distance)\b/i,
      // Shopping & buying
      /\b(buy|purchase|shop|store|expensive|worth|price)\b/i,
      // Activity planning
      /\b(do today|activity|activities|plan|schedule|free time)\b/i,
      // Contextual "what" questions
      /\b(what.*wear|what.*bring|what.*take|what.*pack)\b/i
    ];
    
    return reasoningPatterns.some(pattern => pattern.test(text));
  }
  
  /**
   * Decide if edge intelligence should handle this
   */
  private shouldUseEdgeIntelligence(text: string): boolean {
    // Use edge intelligence for data queries (free real-time APIs!)
    const edgePatterns = [
      /\b(weather|temperature|forecast|climate)\b/i,
      /\b(news|headlines|latest|breaking)\b/i,
      /\b(search|find|look up|what is|who is)\b/i,
      /\b(where am i|location|place)\b/i,
      /\b(time|date|day|clock)\b/i
    ];
    
    return edgePatterns.some(pattern => pattern.test(text));
  }
  
  /**
   * Get suggestions based on result type
   */
  private getSuggestionsForType(type: string): string[] {
    const suggestions: Record<string, string[]> = {
      weather: ["Tomorrow's forecast", "Weekly weather", "Set weather alerts"],
      news: ["Technology news", "Business news", "Local news"],
      search: ["Search more", "Related topics", "Save this"],
      location: ["Navigate home", "Nearby places", "Share location"],
      time: ["Set alarm", "Set timer", "World clock"]
    };
    
    return suggestions[type] || ["Ask another question", "Go back"];
  }
  
  /**
   * Match OS commands using patterns
   */
  private matchOSCommand(text: string, original: string): CommandResponse | null {
    
    // ============= LAYER 1: HARDWARE =============
    
    // CAMERA - Enhanced patterns
    if (/\b(take|capture|snap|shoot|grab|get)\b.*\b(photo|picture|pic|image|shot|selfie)\b/i.test(text) ||
        /\b(camera|photo|picture|pic)\b/i.test(text) && !/\b(open|launch|start)\b/i.test(text) ||
        /^(selfie|smile|cheese)$/i.test(text)) {
      
      const isSelfie = /\b(selfie|self portrait)\b/i.test(text);
      
      return {
        message: isSelfie ? "📸 Switching to selfie mode..." : "📸 Opening camera to take a photo...",
        intents: [{
          layer: 'HARDWARE',
          operation: 'CAMERA_CAPTURE',
          params: { mode: isSelfie ? 'selfie' : 'normal' },
          confidence: 0.98,
          requiresConfirmation: false
        }],
        suggestions: ["Analyze this photo", "Take another", "Record video", "Apply filter"]
      };
    }
    
    // VIDEO RECORDING - Enhanced patterns
    if (/\b(record|start recording|video|film|shoot video)\b/i.test(text) ||
        /\b(make|create)\b.*\b(video|recording)\b/i.test(text)) {
      return {
        message: "🎥 Starting video recording...",
        intents: [{
          layer: 'HARDWARE',
          operation: 'CAMERA_RECORD_START',
          params: {},
          confidence: 0.95,
          requiresConfirmation: false
        }],
        suggestions: ["Stop recording", "Take photo instead", "Pause recording"]
      };
    }
    
    // VISUAL INTELLIGENCE - Eye tracking and object recognition
    if (/\b(what|identify|recognize|tell me|what's|whats)\b.*\b(am i looking at|is this|am i seeing|do i see|that)\b/i.test(text) ||
        /\b(enable|activate|start|turn on)\b.*\b(visual|eye tracking|gaze|focus mode|visual intelligence)\b/i.test(text) ||
        /\b(visual intelligence|eye track|gaze track|focus mode|smart view)\b/i.test(text)) {
      
      if (/\b(enable|activate|start|turn on)\b/i.test(text)) {
        return {
          message: "👁️ Activating Visual Intelligence...\n\nI'll now track what you're looking at and provide intelligent feedback on objects, text, products, and more in your view.",
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'VISUAL_INTELLIGENCE_ENABLE',
            params: {},
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Enable ambient intelligence", "What am I looking at?", "Disable visual intelligence"]
        };
      }
      
      if (/\b(disable|deactivate|stop|turn off)\b/i.test(text)) {
        return {
          message: "👁️ Deactivating Visual Intelligence...",
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'VISUAL_INTELLIGENCE_DISABLE',
            params: {},
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Enable visual intelligence"]
        };
      }
      
      return {
        message: "👁️ Analyzing what you're looking at...\n\nI'll identify objects, text, products, or people in your current view and provide intelligent context.",
        intents: [{
          layer: 'INTELLIGENCE',
          operation: 'VISUAL_ANALYZE_CURRENT',
          params: {},
          confidence: 0.92,
          requiresConfirmation: false
        }],
        suggestions: ["Enable continuous tracking", "More details", "Search for this online"]
      };
    }
    
    // AMBIENT INTELLIGENCE - Proactive assistance
    if (/\b(enable|activate|start|turn on)\b.*\b(ambient|proactive|smart assist|background intelligence)\b/i.test(text) ||
        /\b(ambient intelligence|smart notifications|proactive help|background AI)\b/i.test(text)) {
      
      if (/\b(enable|activate|start|turn on)\b/i.test(text)) {
        return {
          message: "🧠 Activating Ambient Intelligence...\n\nI'll now:\n• Learn your patterns\n• Monitor your attention & eye strain\n• Provide proactive suggestions\n• Alert you at the right moments\n\nAll while respecting your focus and minimizing distractions.",
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'AMBIENT_INTELLIGENCE_ENABLE',
            params: {},
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Show attention patterns", "View smart suggestions", "Disable ambient intelligence"]
        };
      }
      
      if (/\b(disable|deactivate|stop|turn off)\b/i.test(text)) {
        return {
          message: "🧠 Deactivating Ambient Intelligence...",
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'AMBIENT_INTELLIGENCE_DISABLE',
            params: {},
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Enable ambient intelligence"]
        };
      }
    }
    
    // FOCUS MODE - Minimal UI
    if (/\b(enable|activate|start|turn on)\b.*\b(focus mode|focus|distraction free|minimal mode)\b/i.test(text) ||
        /\b(focus mode|distraction free|minimize distractions)\b/i.test(text)) {
      
      if (/\b(disable|deactivate|stop|turn off)\b/i.test(text)) {
        return {
          message: "🎯 Disabling Focus Mode - Full interface restored",
          intents: [{
            layer: 'INTELLIGENCE',
            operation: 'FOCUS_MODE_DISABLE',
            params: {},
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Enable focus mode"]
        };
      }
      
      return {
        message: "🎯 Enabling Focus Mode...\n\nReducing distractions:\n• Minimal UI elements\n• Context-aware information only\n• Smart notification filtering\n• Your attention is protected",
        intents: [{
          layer: 'INTELLIGENCE',
          operation: 'FOCUS_MODE_ENABLE',
          params: {},
          confidence: 0.95,
          requiresConfirmation: false
        }],
        suggestions: ["Enable visual intelligence", "Enable ambient intelligence", "Disable focus mode"]
      };
    }
    
    // ATTENTION & EYE HEALTH
    if (/\b(eye|eyes|strain|tired eyes|eye health)\b/i.test(text) ||
        /\b(attention|focus|concentration)\b.*\b(level|status|state)\b/i.test(text) ||
        /\b(how focused am i|am i focused|focus check)\b/i.test(text)) {
      
      return {
        message: "👁️ Analyzing your attention and eye health...\n\nI'll check:\n• Eye strain level\n• Focus intensity\n• Blink rate\n• Cognitive load\n• Recommended break time",
        intents: [{
          layer: 'INTELLIGENCE',
          operation: 'ATTENTION_ANALYZE',
          params: {},
          confidence: 0.90,
          requiresConfirmation: false
        }],
        suggestions: ["Take eye break", "Show attention patterns", "Enable eye health alerts"]
      };
    }
    
    // BATTERY - Enhanced patterns with context-aware responses
    if (/\b(battery|power|charge)\b.*\b(status|level|how much|remaining|left|life)\b/i.test(text) ||
        /\b(how much|what'?s|check|show)\b.*\b(battery|power|charge)\b/i.test(text) ||
        /^(battery|power)$/i.test(text)) {
      const battery = systemState.getState().layer1_hardware.power;
      const level = Math.round(battery.batteryLevel * 100);
      const time = battery.estimatedRuntime;
      const isLow = level < 20;
      const isCritical = level < 10;
      
      let emoji = level > 80 ? '🔋' : level > 50 ? '🔋' : level > 20 ? '🪫' : '🪫';
      let statusMsg = battery.charging ? ' ⚡ Charging' : '';
      
      if (isCritical) {
        statusMsg += ' ⚠️ Critical - Please charge soon!';
      } else if (isLow) {
        statusMsg += ' ⚠️ Low battery';
      }
      
      return {
        message: `${emoji} Battery at ${level}% (${time} minutes remaining)${statusMsg}`,
        intents: [{
          layer: 'HARDWARE',
          operation: 'POWER_STATUS',
          params: {},
          confidence: 1.0,
          requiresConfirmation: false
        }],
        suggestions: isLow 
          ? ["Enable power saving", "Close apps", "Lower brightness", "Check battery drain"] 
          : ["View battery stats", "Battery optimization"],
        data: { level, time, charging: battery.charging, isLow, isCritical }
      };
    }
    
    // BRIGHTNESS - Enhanced patterns with shortcuts
    if (/\b(brightness|display|screen)\b/i.test(text) || /\b(dim|brighten|darker|lighter)\b/i.test(text)) {
      const current = Math.round(systemState.getState().layer1_hardware.display.brightness * 100);
      
      // Handle relative adjustments
      if (/\b(increase|up|more|brighten|lighter|raise)\b/i.test(text)) {
        const newLevel = Math.min(1.0, current / 100 + 0.2);
        return {
          message: `💡 Increasing brightness to ${Math.round(newLevel * 100)}%`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'DISPLAY_BRIGHTNESS',
            params: { value: newLevel },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Increase more", "Set to max", "Auto brightness"]
        };
      }
      
      if (/\b(decrease|down|less|dim|darker|lower)\b/i.test(text)) {
        const newLevel = Math.max(0.1, current / 100 - 0.2);
        return {
          message: `💡 Decreasing brightness to ${Math.round(newLevel * 100)}%`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'DISPLAY_BRIGHTNESS',
            params: { value: newLevel },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Decrease more", "Set to minimum", "Auto brightness"]
        };
      }
      
      // Handle shortcuts
      if (/\b(max|maximum|full|100)\b/i.test(text)) {
        return {
          message: `💡 Setting brightness to maximum (100%)`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'DISPLAY_BRIGHTNESS',
            params: { value: 1.0 },
            confidence: 0.98,
            requiresConfirmation: false
          }],
          suggestions: ["Decrease brightness", "Auto brightness"]
        };
      }
      
      if (/\b(min|minimum|lowest)\b/i.test(text)) {
        return {
          message: `💡 Setting brightness to minimum (10%)`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'DISPLAY_BRIGHTNESS',
            params: { value: 0.1 },
            confidence: 0.98,
            requiresConfirmation: false
          }],
          suggestions: ["Increase brightness", "Auto brightness"]
        };
      }
      
      // Extract percentage
      const percentage = entityExtractor.extractPercentage(text);
      
      if (percentage !== null) {
        return {
          message: `💡 Setting brightness to ${Math.round(percentage * 100)}%`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'DISPLAY_BRIGHTNESS',
            params: { value: percentage },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Increase brightness", "Decrease brightness", "Auto brightness"]
        };
      } else {
        return {
          message: `Current brightness: ${current}%`,
          intents: [],
          suggestions: ["Set to 50%", "Set to 100%", "Dim screen", "Brighten screen", "Enable auto brightness"],
          needsClarification: true,
          clarificationQuestion: "What brightness level? (e.g., '50%', 'maximum', 'brighten')"
        };
      }
    }
    
    // VOLUME
    if (/\b(volume|sound|audio)\b/i.test(text)) {
      const percentage = entityExtractor.extractPercentage(text);
      
      if (percentage !== null) {
        return {
          message: `🔊 Setting volume to ${Math.round(percentage * 100)}%`,
          intents: [{
            layer: 'HARDWARE',
            operation: 'AUDIO_VOLUME',
            params: { value: percentage },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Mute", "Max volume", "Enable spatial audio"]
        };
      }
    }
    
    // ============= LAYER 3: BLOCKCHAIN =============
    
    // WALLET CREATE
    if (/\b(create|make|new|setup|generate)\b.*\bwallet\b/i.test(text)) {
      return {
        message: "🔐 Creating a new wallet for you...",
        intents: [{
          layer: 'BLOCKCHAIN',
          operation: 'WALLET_CREATE',
          params: {},
          confidence: 0.95,
          requiresConfirmation: true
        }],
        suggestions: ["View wallet", "Check balance", "Secure wallet"]
      };
    }
    
    // WALLET BALANCE
    if (/\b(balance|how much|funds|money)\b/i.test(text) ||
        /\b(check|show|view)\b.*\b(balance|wallet)\b/i.test(text)) {
      const wallet = systemState.getState().layer3_blockchain.wallet;
      
      if (!wallet.exists) {
        return {
          message: "You don't have a wallet yet.",
          intents: [],
          suggestions: ["Create wallet"],
          needsClarification: true,
          clarificationQuestion: "Would you like me to create a wallet for you?"
        };
      }
      
      return {
        message: `💰 Your balance: ${wallet.balance} KARA`,
        intents: [{
          layer: 'BLOCKCHAIN',
          operation: 'WALLET_BALANCE',
          params: {},
          confidence: 1.0,
          requiresConfirmation: false
        }],
        suggestions: ["Send KARA", "Receive KARA", "Transaction history"],
        data: { balance: wallet.balance, did: wallet.did }
      };
    }
    
    // WALLET TRANSFER
    if (/\b(send|transfer|pay|give)\b/i.test(text)) {
      const amount = entityExtractor.extractAmount(text);
      const contact = entityExtractor.extractContact(text);
      
      if (amount && contact) {
        return {
          message: `Sending ${amount} KARA to ${contact}. Please confirm.`,
          intents: [{
            layer: 'BLOCKCHAIN',
            operation: 'WALLET_TRANSFER',
            params: { amount, recipient: contact },
            confidence: 0.92,
            requiresConfirmation: true
          }],
          suggestions: ["Cancel", "Change amount", "View balance"]
        };
      } else {
        return {
          message: "I need more details for the transfer.",
          intents: [],
          suggestions: [],
          needsClarification: true,
          clarificationQuestion: !amount ? "How much KARA do you want to send?" : "Who should I send it to?"
        };
      }
    }
    
    // ============= LAYER 5: INTELLIGENCE =============
    
    // VISION ANALYSIS
    if (/\b(what|analyze|identify|recognize|scan|look|see)\b.*\b(this|that|it)\b/i.test(text) ||
        /\b(what'?s|identify|tell me)\b.*\b(this|that|looking at)\b/i.test(text)) {
      return {
        message: "👁️ Analyzing what I see...",
        intents: [{
          layer: 'INTELLIGENCE',
          operation: 'VISION_ANALYZE',
          params: {},
          confidence: 0.88,
          requiresConfirmation: false
        }],
        suggestions: ["Take photo", "More details", "Search online"]
      };
    }
    
    // ============= LAYER 8: APPLICATIONS =============
    
    // TIMERS - Enhanced natural language support
    if (/\b(set|create|start|make)\b.*\btimer\b/i.test(text) ||
        /\btimer\b.*\b(for|of)\b/i.test(text) ||
        /\b(remind me|alarm)\b.*\b(in|after)\b/i.test(text)) {
      const duration = entityExtractor.extractDuration(text);
      const name = entityExtractor.extractTimerName(text);
      
      if (duration) {
        const minutes = Math.round(duration / 60);
        const seconds = duration % 60;
        const timeStr = minutes > 0 
          ? `${minutes} minute${minutes !== 1 ? 's' : ''}${seconds > 0 ? ` ${seconds}s` : ''}`
          : `${seconds} second${seconds !== 1 ? 's' : ''}`;
        
        return {
          message: `⏱️ Setting ${name || 'timer'} for ${timeStr}`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'TIMER_CREATE',
            params: { durationMs: duration * 1000, name: name || 'Timer' },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["View all timers", "Pause timer", "Cancel timer", "Add another timer"]
        };
      } else {
        return {
          message: "How long should the timer be?",
          intents: [],
          suggestions: ["5 minutes", "10 minutes", "1 hour", "30 seconds"],
          needsClarification: true,
          clarificationQuestion: "How long? (e.g., '5 minutes', '30 seconds', '1 hour')"
        };
      }
    }
    
    if (/\b(cancel|stop|delete|remove)\b.*\btimer\b/i.test(text)) {
      const name = entityExtractor.extractTimerName(text);
      
      return {
        message: name ? `Canceling ${name} timer...` : "Canceling timer...",
        intents: [{
          layer: 'APPLICATIONS',
          operation: 'TIMER_CANCEL',
          params: { name },
          confidence: 0.92,
          requiresConfirmation: false
        }],
        suggestions: ["View remaining timers"]
      };
    }
    
    if (/\b(show|list|view)\b.*\btimer/i.test(text)) {
      const timers = systemState.getState().layer8_applications.timers;
      
      if (timers.length === 0) {
        return {
          message: "You don't have any active timers.",
          intents: [],
          suggestions: ["Set timer for 5 minutes", "Set cooking timer"]
        };
      }
      
      const timerList = timers.map(t => 
        `${t.name}: ${Math.round(t.remaining / 60)}m remaining`
      ).join(', ');
      
      return {
        message: `Active timers: ${timerList}`,
        intents: [{
          layer: 'APPLICATIONS',
          operation: 'TIMER_LIST',
          params: {},
          confidence: 1.0,
          requiresConfirmation: false
        }],
        suggestions: ["Cancel all timers", "Add another timer"],
        data: { timers }
      };
    }
    
    // APPS
    if (/\b(open|launch|start|run)\b.*\b(app|application)\b/i.test(text) ||
        /\b(open|launch|start)\b\s+(\w+)/i.test(text)) {
      
      // Check for App Store specifically
      if (/\b(app store|appstore|play store|store)\b/i.test(text)) {
        return {
          message: `📱 Opening App Store...`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'APP_OPEN',
            params: { name: 'App Store' },
            confidence: 0.98,
            requiresConfirmation: false
          }],
          suggestions: ["Browse apps", "View installed apps", "Search for app"]
        };
      }
      
      const appName = entityExtractor.extractAppName(text);
      
      if (appName) {
        return {
          message: `🚀 Opening ${appName}...`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'ANDROID_OPEN',
            params: { appName },
            confidence: 0.90,
            requiresConfirmation: false
          }],
          suggestions: [`Close ${appName}`, "View running apps"]
        };
      }
    }
    
    if (/\b(close|quit|exit)\b.*\b(app|application)\b/i.test(text)) {
      const appName = entityExtractor.extractAppName(text);
      
      if (appName) {
        return {
          message: `Closing ${appName}...`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'ANDROID_CLOSE',
            params: { appName },
            confidence: 0.90,
            requiresConfirmation: false
          }],
          suggestions: ["Open another app", "View running apps"]
        };
      }
    }
    
    if (/\b(show|list|what)\b.*\b(running|open|active)\b.*\bapps?\b/i.test(text)) {
      const apps = systemContext.getAllApps().filter(a => a.running);
      
      if (apps.length === 0) {
        return {
          message: "No apps are currently running.",
          intents: [],
          suggestions: ["View installed apps", "Open an app"]
        };
      }
      
      const appList = apps.map(a => a.name).join(', ');
      return {
        message: `Running apps: ${appList}`,
        intents: [{
          layer: 'APPLICATIONS',
          operation: 'ANDROID_LIST_RUNNING',
          params: {},
          confidence: 1.0,
          requiresConfirmation: false
        }],
        suggestions: ["Close all apps", "View installed apps"],
        data: { apps }
      };
    }
    
    // INSTALL/DOWNLOAD APPS
    if (/\b(install|download|get)\b.*\b(app|application)\b/i.test(text)) {
      const appName = text.match(/(?:install|download|get)\s+(?:app\s+)?([a-z0-9\s]+?)(?:\s+app)?$/i)?.[1]?.trim();
      
      if (appName) {
        return {
          message: `📦 Searching for "${appName}" in App Store...`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'APP_SEARCH_AND_INSTALL',
            params: { query: appName },
            confidence: 0.85,
            requiresConfirmation: false
          }],
          suggestions: ["Open App Store", "View installed apps"]
        };
      } else {
        return {
          message: "📱 Opening App Store to browse apps...",
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'APP_OPEN',
            params: { name: 'App Store' },
            confidence: 0.90,
            requiresConfirmation: false
          }],
          suggestions: ["Search for app", "View categories"]
        };
      }
    }
    
    // UNINSTALL APPS
    if (/\b(uninstall|remove|delete)\b.*\b(app|application)\b/i.test(text)) {
      const appName = text.match(/(?:uninstall|remove|delete)\s+(?:app\s+)?([a-z0-9\s]+?)(?:\s+app)?$/i)?.[1]?.trim();
      
      if (appName) {
        return {
          message: `🗑️ Uninstalling "${appName}"...`,
          intents: [{
            layer: 'APPLICATIONS',
            operation: 'APP_UNINSTALL',
            params: { name: appName },
            confidence: 0.88,
            requiresConfirmation: true
          }],
          suggestions: ["View installed apps", "Open App Store"]
        };
      }
    }
    
    // VIEW INSTALLED APPS
    if (/\b(show|list|view)\b.*\b(installed|my)\b.*\bapps?\b/i.test(text)) {
      return {
        message: "📦 Opening installed apps list...",
        intents: [{
          layer: 'APPLICATIONS',
          operation: 'APP_LIST_INSTALLED',
          params: {},
          confidence: 0.95,
          requiresConfirmation: false
        }],
        suggestions: ["Install new app", "Open App Store", "Uninstall app"]
      };
    }
    
    // SETTINGS
    if (/\b(open|show|view)\b.*\bsettings?\b/i.test(text)) {
      return {
        message: "⚙️ Opening settings...",
        intents: [{
          layer: 'APPLICATIONS',
          operation: 'SETTINGS_OPEN',
          params: {},
          confidence: 0.95,
          requiresConfirmation: false
        }],
        suggestions: ["Close settings", "Change security mode"]
      };
    }
    
    // ============= GREETINGS & CASUAL =============
    
    if (/^(hi|hello|hey|sup|yo|howdy|greetings|good\s+(morning|afternoon|evening|day))\b/i.test(text)) {
      const hour = new Date().getHours();
      const greeting = hour < 12 ? 'Good morning' : hour < 18 ? 'Good afternoon' : 'Good evening';
      const state = systemState.getState();
      const battery = Math.round(state.layer1_hardware.power.batteryLevel * 100);
      
      // Proactive context-aware information
      const proactive: string[] = [];
      
      if (battery < 20) {
        proactive.push(`⚠️ Battery low: ${battery}%`);
      }
      
      if (state.layer8_applications.timers.length > 0) {
        proactive.push(`⏱️ ${state.layer8_applications.timers.length} active timer(s)`);
      }
      
      // Add smart suggestions based on time
      const timeSuggestions: string[] = [];
      if (hour >= 6 && hour < 12) {
        timeSuggestions.push("Morning news", "Today's weather");
      } else if (hour >= 12 && hour < 18) {
        timeSuggestions.push("Latest news", "Check weather");
      } else {
        timeSuggestions.push("Evening news", "What time is it");
      }
      
      const contextMsg = proactive.length > 0 ? '\n' + proactive.join(' • ') + '\n' : '';
      
      return {
        message: `${greeting}!${contextMsg}\n\nHow can I help you today?`,
        intents: [],
        suggestions: [...timeSuggestions, "Take photo", "Check battery"]
      };
    }
    
    if (/^(thanks|thank you|thx|ty|appreciate it)\b/i.test(text)) {
      const responses = [
        "You're welcome! Anything else?",
        "Happy to help! What's next?",
        "My pleasure! Need anything else?",
        "Anytime! What else can I do?"
      ];
      const response = responses[Math.floor(Math.random() * responses.length)];
      
      return {
        message: response,
        intents: [],
        suggestions: ["Latest news", "Check weather", "Take photo", "What time is it"]
      };
    }
    
    if (/^(bye|goodbye|see you|later|cya)\b/i.test(text)) {
      const hour = new Date().getHours();
      const farewell = hour < 18 ? 'Have a great day!' : 'Have a great evening!';
      
      return {
        message: `${farewell} Feel free to ask anytime. 👋`,
        intents: [],
        suggestions: []
      };
    }
    
    if (/^(help|what can you do|capabilities|commands|features)\b/i.test(text)) {
      return {
        message: "I'm your intelligent AI assistant! Here's what I can do:\n\n🌐 **Real-Time Information**\n• Latest news (location-aware)\n• Weather forecasts\n• Web search\n• Current time & date\n• Your location\n\n📸 **Camera & Vision**\n• Take photos & videos\n• Analyze what you're looking at\n\n🔋 **System Control**\n• Battery status & power saving\n• Brightness & volume adjustments\n\n💰 **Blockchain & Wallet**\n• Create & manage wallet\n• Check balance\n• Send KARA tokens\n\n⏱️ **Apps & Productivity**\n• Set timers & reminders\n• Manage Android apps\n• Calendar & events\n\n⚙️ **System Settings**\n• Security modes\n• Diagnostics\n• Customization\n\nJust ask me naturally in your own words!",
        intents: [],
        suggestions: ["Show news", "Check weather", "Take photo", "What time is it", "Check battery"]
      };
    }
    
    // No pattern matched
    return null;
  }
  
  /**
   * Parse multi-intent commands (e.g., "take a photo and set timer for 5 minutes")
   */
  private parseMultiIntent(text: string, original: string): CommandResponse | null {
    // Split by common conjunctions
    const segments = text.split(/\b(and|then|after that|also)\b/i)
      .filter(s => s.trim() && !/^(and|then|after that|also)$/i.test(s.trim()));
    
    if (segments.length < 2) return null;
    
    console.log('[IntelligentRouter] Multi-intent segments:', segments);
    
    const intents: CommandIntent[] = [];
    let message = "I'll do that for you:\n";
    
    for (const segment of segments) {
      const cmd = this.matchOSCommand(segment.trim(), segment.trim());
      if (cmd && cmd.intents.length > 0) {
        intents.push(...cmd.intents);
        message += `• ${cmd.message}\n`;
      }
    }
    
    if (intents.length < 2) return null;
    
    return {
      message: message.trim(),
      intents,
      suggestions: ["Do more", "Undo", "Repeat"],
      needsClarification: false
    };
  }
  
  /**
   * Match internet-based services (news, weather, search, time)
   */
  private async matchInternetService(text: string, original: string): Promise<CommandResponse | null> {
    
    // ============= NEWS =============
    
    if (/\b(news|headlines|latest|breaking)\b/i.test(text)) {
      try {
        const location = await locationService.getLocation();
        const category = this.extractNewsCategory(text);
        const articles = await newsService.getNews(category);
        
        const newsList = articles.slice(0, 3).map((article, idx) => 
          `${idx + 1}. **${article.title}**\n   ${article.description.substring(0, 100)}...\n   _${article.source}_ • ${this.getRelativeTime(article.publishedAt)}`
        ).join('\n\n');
        
        return {
          message: `📰 **Latest ${category} News** (${location.city}, ${location.country}):\n\n${newsList}\n\nWant more details? I can search for specific topics.`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'NEWS_FETCH',
            params: { category, location: location.city },
            confidence: 0.95,
            requiresConfirmation: false
          }],
          suggestions: ["Tech news", "Business news", "Sports news", "Refresh news"],
          data: { articles }
        };
      } catch (error) {
        console.error('News fetch failed:', error);
      }
    }
    
    // ============= WEATHER =============
    
    // Umbrella-specific queries
    if (/\b(umbrella|need.*umbrella|bring.*umbrella|take.*umbrella)\b/i.test(text)) {
      try {
        const weather = await weatherService.getWeather();
        const umbrellaAdvice = this.getUmbrellaAdvice(weather);
        
        const forecastStr = weather.forecast.slice(0, 3).map(f => 
          `• ${f.day}: ${f.high}°C / ${f.low}°C - ${f.condition}`
        ).join('\n');
        
        return {
          message: `${umbrellaAdvice.emoji} **${umbrellaAdvice.answer}**\n\n**Current Weather:**\n${weather.temperature}°C, ${weather.condition}\n💧 Humidity: ${weather.humidity}%\n\n**Forecast:**\n${forecastStr}\n\n${umbrellaAdvice.explanation}`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'WEATHER_UMBRELLA_CHECK',
            params: { needed: umbrellaAdvice.needed },
            confidence: 0.98,
            requiresConfirmation: false
          }],
          suggestions: umbrellaAdvice.needed 
            ? ["Full weather forecast", "Rain alerts", "Weather radar"]
            : ["Tomorrow's weather", "Weekly forecast", "Weather updates"],
          data: { weather, umbrella: umbrellaAdvice }
        };
      } catch (error) {
        console.error('Weather fetch failed:', error);
      }
    }
    
    // General weather queries
    if (/\b(weather|temperature|forecast|rain|sunny|cloudy|climate)\b/i.test(text)) {
      try {
        const city = this.extractCity(text);
        const weather = await weatherService.getWeather(city);
        
        const emoji = this.getWeatherEmoji(weather.condition);
        const forecastStr = weather.forecast.slice(0, 3).map(f => 
          `• ${f.day}: ${f.high}°C / ${f.low}°C - ${f.condition}`
        ).join('\n');
        
        return {
          message: `${emoji} **Weather in ${weather.location}**\n\n**Now:** ${weather.temperature}°C, ${weather.condition}\n💧 Humidity: ${weather.humidity}%\n💨 Wind: ${weather.windSpeed} km/h\n\n**Forecast:**\n${forecastStr}`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'WEATHER_FETCH',
            params: { city: weather.location },
            confidence: 0.98,
            requiresConfirmation: false
          }],
          suggestions: ["Tomorrow's weather", "Weekly forecast", "Weather alerts"],
          data: { weather }
        };
      } catch (error) {
        console.error('Weather fetch failed:', error);
      }
    }
    
    // ============= WEB SEARCH =============
    
    if (/\b(search|look up|find|google)\b/i.test(text) || 
        /\b(what is|who is|where is|when is|how to)\b/i.test(text)) {
      try {
        const query = this.extractSearchQuery(text);
        if (query) {
          const results = await webSearchService.search(query, 3);
          
          const resultsList = results.map((result, idx) =>
            `${idx + 1}. **${result.title}**\n   ${result.snippet}\n   🔗 ${result.displayUrl}`
          ).join('\n\n');
          
          return {
            message: `🔍 **Search results for "${query}":**\n\n${resultsList}`,
            intents: [{
              layer: 'KNOWLEDGE',
              operation: 'WEB_SEARCH',
              params: { query },
              confidence: 0.90,
              requiresConfirmation: false
            }],
            suggestions: ["Refine search", "More results", "Related topics"],
            data: { results, query }
          };
        }
      } catch (error) {
        console.error('Search failed:', error);
      }
    }
    
    // ============= TIME & DATE =============
    
    if (/\b(time|clock|hour|minute)\b/i.test(text) && !/\btimer\b/i.test(text)) {
      try {
        const location = await locationService.getLocation();
        const time = timeService.getCurrentTime(location.timezone);
        const date = timeService.getCurrentDate(location.timezone);
        
        return {
          message: `🕐 **Current Time**\n\n⏰ ${time}\n📅 ${date}\n🌍 ${location.city}, ${location.country}`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'TIME_QUERY',
            params: { timezone: location.timezone },
            confidence: 1.0,
            requiresConfirmation: false
          }],
          suggestions: ["Set alarm", "World clock", "Set timer"]
        };
      } catch (error) {
        console.error('Time query failed:', error);
      }
    }
    
    if (/\b(date|day|today|tomorrow|calendar)\b/i.test(text) && !/\btimer\b/i.test(text)) {
      try {
        const location = await locationService.getLocation();
        const date = timeService.getCurrentDate(location.timezone);
        const time = timeService.getCurrentTime(location.timezone);
        
        return {
          message: `📅 **Date & Time**\n\n${date}\n⏰ ${time}\n🌍 ${location.city}, ${location.country}`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'DATE_QUERY',
            params: { timezone: location.timezone },
            confidence: 1.0,
            requiresConfirmation: false
          }],
          suggestions: ["View calendar", "Set reminder", "Check events"]
        };
      } catch (error) {
        console.error('Date query failed:', error);
      }
    }
    
    // ============= LOCATION =============
    
    if (/\b(where am i|my location|current location|gps)\b/i.test(text)) {
      try {
        const location = await locationService.getLocation();
        
        return {
          message: `📍 **Your Location**\n\n🏙️ ${location.city}, ${location.region}\n🌍 ${location.country}\n🕐 Timezone: ${location.timezone}\n📐 Coordinates: ${location.coordinates.lat.toFixed(4)}, ${location.coordinates.lng.toFixed(4)}`,
          intents: [{
            layer: 'KNOWLEDGE',
            operation: 'LOCATION_QUERY',
            params: { location },
            confidence: 1.0,
            requiresConfirmation: false
          }],
          suggestions: ["Nearby places", "Navigation", "Share location"]
        };
      } catch (error) {
        console.error('Location query failed:', error);
      }
    }
    
    return null;
  }
  
  // Helper methods for internet services
  
  private extractNewsCategory(text: string): string {
    if (/\b(tech|technology|science)\b/i.test(text)) return 'technology';
    if (/\b(business|finance|market|stock)\b/i.test(text)) return 'business';
    if (/\b(sport|sports|football|basketball)\b/i.test(text)) return 'sports';
    if (/\b(health|medical|wellness)\b/i.test(text)) return 'health';
    if (/\b(entertainment|movie|music|celebrity)\b/i.test(text)) return 'entertainment';
    return 'general';
  }
  
  private extractCity(text: string): string | undefined {
    // Extract city name after "in" or "for"
    const match = text.match(/\b(?:in|for)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)\b/);
    return match ? match[1] : undefined;
  }
  
  private extractSearchQuery(text: string): string | null {
    // Remove command words and extract the query
    const cleaned = text
      .replace(/\b(search|look up|find|google|for|about)\b/gi, '')
      .replace(/\b(what is|who is|where is|when is|how to)\b/gi, (match) => match.toLowerCase())
      .trim();
    
    return cleaned.length > 2 ? cleaned : null;
  }
  
  private getWeatherEmoji(condition: string): string {
    const lower = condition.toLowerCase();
    if (lower.includes('sunny') || lower.includes('clear')) return '☀️';
    if (lower.includes('cloud')) return '☁️';
    if (lower.includes('rain')) return '🌧️';
    if (lower.includes('storm')) return '⛈️';
    if (lower.includes('snow')) return '❄️';
    return '🌤️';
  }
  
  private getUmbrellaAdvice(weather: WeatherData): { 
    needed: boolean; 
    answer: string; 
    explanation: string; 
    emoji: string;
  } {
    const condition = weather.condition.toLowerCase();
    const todayCondition = condition;
    const forecast = weather.forecast;
    
    // Check for rain-related conditions
    const rainyConditions = ['rain', 'rainy', 'shower', 'storm', 'drizzle', 'precipitation', 'thunder'];
    const todayRain = rainyConditions.some(r => todayCondition.includes(r));
    
    // Check tomorrow's forecast
    const tomorrowCondition = forecast[1]?.condition.toLowerCase() || '';
    const tomorrowRain = rainyConditions.some(r => tomorrowCondition.includes(r));
    
    // Check humidity (high humidity might indicate incoming rain)
    const highHumidity = weather.humidity > 80;
    
    if (todayRain) {
      return {
        needed: true,
        answer: "YES! Definitely bring an umbrella! ☔",
        explanation: "It's currently raining or likely to rain today. You'll need it!",
        emoji: "☔"
      };
    }
    
    if (tomorrowRain) {
      return {
        needed: true,
        answer: "Better bring one just in case! ☂️",
        explanation: `Today looks clear, but rain is forecasted for ${forecast[1].day}. Better safe than wet!`,
        emoji: "☂️"
      };
    }
    
    // Check next 3 days for rain
    const upcomingRain = forecast.slice(0, 3).some(day => 
      rainyConditions.some(r => day.condition.toLowerCase().includes(r))
    );
    
    if (upcomingRain) {
      return {
        needed: false,
        answer: "No umbrella needed today",
        explanation: "Current conditions are clear, but rain is expected later this week. You might want to keep one handy.",
        emoji: "🌤️"
      };
    }
    
    if (highHumidity && todayCondition.includes('cloud')) {
      return {
        needed: false,
        answer: "No umbrella needed, but...",
        explanation: `High humidity (${weather.humidity}%) and cloudy conditions. Rain is possible but not forecasted.`,
        emoji: "☁️"
      };
    }
    
    return {
      needed: false,
      answer: "No umbrella needed! Enjoy the clear weather! ☀️",
      explanation: `Clear skies with ${weather.temperature}°C. Perfect weather ahead!`,
      emoji: "☀️"
    };
  }
  
  private getRelativeTime(timestamp: string): string {
    const now = Date.now();
    const then = new Date(timestamp).getTime();
    const diff = now - then;
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }
  
  /**
   * Check if it's a knowledge query (not OS command or internet service)
   */
  private isKnowledgeQuery(text: string): boolean {
    const knowledgePatterns = [
      /\b(who|what|when|where|why|how)\b.*\b(is|are|was|were|did|does)\b/i,
      /\b(tell me|explain|describe|define)\b/i,
      /\b(president|capital|population|meaning|history)\b/i,
      /\b(calculate|convert|translate)\b/i,
    ];
    
    return knowledgePatterns.some(pattern => pattern.test(text));
  }
  
  /**
   * Handle knowledge queries with built-in knowledge + optional cloud AI
   */
  private async handleKnowledgeQuery(query: string): Promise<CommandResponse> {
    // Try built-in knowledge first
    const fact = await knowledgeService.getFactAbout(query);
    
    if (fact && !fact.includes('Interesting topic')) {
      return {
        message: `💡 ${fact}\n\nWant to know more? I can search the web for you!`,
        intents: [{
          layer: 'KNOWLEDGE',
          operation: 'FACT_QUERY',
          params: { query, answer: fact },
          confidence: 0.85,
          requiresConfirmation: false
        }],
        suggestions: [`Search "${query}"`, "Ask another question", "Related facts"]
      };
    }
    
    // Suggest searching instead
    return {
      message: `I can search the web for "${query}" if you'd like! Or try asking about:\n• Latest news\n• Weather\n• Time & date\n• Your location\n• OS controls (battery, camera, wallet)`,
      intents: [],
      suggestions: [`Search "${query}"`, "Show news", "Check weather", "What time is it"]
    };
  }
  
  /**
   * Handle conversational/unclear input with context-aware suggestions
   */
  private handleConversational(text: string): CommandResponse {
    // Check recent context for smart suggestions
    const state = systemState.getState();
    const battery = state.layer1_hardware.power.batteryLevel;
    const hasWallet = state.layer3_blockchain.wallet.exists;
    const activeTimers = state.layer8_applications.timers.length;
    
    // Smart suggestions based on state and time of day
    const suggestions: string[] = [];
    const hour = new Date().getHours();
    
    // Time-based suggestions
    if (hour >= 6 && hour < 12) {
      suggestions.push("Morning news", "Today's weather");
    } else if (hour >= 12 && hour < 18) {
      suggestions.push("Latest news", "Check weather");
    } else {
      suggestions.push("Evening news", "Tomorrow's weather");
    }
    
    // State-based suggestions
    if (battery < 0.3) {
      suggestions.push("Check battery", "Power saving");
    }
    
    suggestions.push("Take photo");
    
    if (hasWallet) {
      suggestions.push("View balance");
    } else {
      suggestions.push("Create wallet");
    }
    
    if (activeTimers > 0) {
      suggestions.push("View timers");
    }
    
    // Check for common typos or partial matches
    const fuzzyMatch = this.findFuzzyMatch(text);
    if (fuzzyMatch) {
      return {
        message: `Did you mean "${fuzzyMatch}"?\n\nOr I can help with:\n• 📰 News & Information\n• 🌤️ Weather & Time\n• 📸 Camera & Photos\n• 🔋 System Controls\n• 💰 Wallet & Blockchain\n• 🔍 Web Search`,
        intents: [],
        suggestions: [fuzzyMatch, ...suggestions.slice(0, 5)],
        needsClarification: false
      };
    }
    
    return {
      message: "I'm your AI assistant! I can help with:\n\n🌐 **Internet Services**\n• Latest news (location-aware)\n• Weather forecasts\n• Web search\n• Time & date\n\n📱 **Device Controls**\n• Camera & photos 📸\n• Battery & display 🔋\n• Wallet & transactions 💰\n• Timers & apps ⏱️\n• Settings ⚙️\n\nWhat would you like to do?",
      intents: [],
      suggestions,
      needsClarification: false
    };
  }
  
  /**
   * Find fuzzy matches for typos/partial commands
   */
  private findFuzzyMatch(text: string): string | null {
    const commonCommands = [
      "latest news", "show news", "weather", "check weather",
      "search", "what time", "my location",
      "battery status", "take photo", "check balance", "set timer",
      "brightness up", "volume up", "record video", "open camera",
      "create wallet", "send KARA", "view timers", "open settings"
    ];
    
    for (const cmd of commonCommands) {
      // Check if text is similar (contains most words or is prefix)
      const textWords = text.toLowerCase().split(/\s+/);
      const cmdWords = cmd.toLowerCase().split(/\s+/);
      
      const overlap = textWords.filter(w => cmdWords.some(cw => cw.includes(w) || w.includes(cw)));
      
      if (overlap.length >= Math.min(textWords.length, cmdWords.length) * 0.6) {
        return cmd;
      }
    }
    
    return null;
  }
}

// Export singleton
export const intelligentRouter = new IntelligentRouter();
