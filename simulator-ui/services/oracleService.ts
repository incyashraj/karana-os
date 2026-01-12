// Universal Oracle Service - Frontend Integration
// Calls the real Rust backend Karana AI with QueryRouter and Tools

import { karanaApi } from './karanaService';

interface ToolOutput {
  tool_name: string;
  output: string;
  confidence: number;
}

interface AgenticResponse {
  suggestion: string;
  chain: ToolOutput[];
  confidence: number;
  reasoning_steps: string[];
}

interface OracleManifest {
  text: string;
  voice_script: string;
  haptic_pattern: 'Success' | 'Neutral' | 'Warning' | 'Error';
  confidence: number;
  reasoning_trace: string[];
  historical_context: string[];
  suggested_followup: string | null;
}

class UniversalOracleService {
  private memory: Map<string, { response: string; score: number; timestamp: number }> = new Map();
  private sessionHistory: string[] = [];
  private useRealBackend: boolean = true; // Toggle to use real backend

  async mediate(request: string): Promise<OracleManifest> {
    // Try real backend first
    if (this.useRealBackend) {
      try {
        const backendResponse = await karanaApi.processOracleIntent(request);
        
        // Convert backend response to our manifest format
        return {
          text: backendResponse.content,
          voice_script: this.generateVoiceScript({ 
            suggestion: backendResponse.content, 
            confidence: backendResponse.confidence,
            chain: [],
            reasoning_steps: []
          }),
          haptic_pattern: this.getHapticPattern(backendResponse.confidence),
          confidence: backendResponse.confidence,
          reasoning_trace: [`Intent: ${backendResponse.intent_type}`, `Confidence: ${(backendResponse.confidence * 100).toFixed(0)}%`],
          historical_context: this.getHistoricalContext(),
          suggested_followup: backendResponse.requires_confirmation ? 'Confirm this action?' : null
        };
      } catch (error) {
        console.warn('[Oracle] Backend unavailable, using fallback:', error);
        // Fall through to simulated response
      }
    }

    // Fallback: Simulated response (original behavior)
    const plan = this.planRequest(request);
    const chain = await this.executeChain(plan, request);
    const response: AgenticResponse = {
      suggestion: this.synthesizeSuggestion(request, chain, plan),
      chain,
      confidence: plan.confidence,
      reasoning_steps: [
        `Classification: ${plan.classification}`,
        `Tools: ${plan.tools.join(', ')}`,
        `Chain depth: ${chain.length}`
      ]
    };

    this.storeSession(request, response.suggestion, response.confidence);
    return this.generateManifest(response);
  }

  private planRequest(request: string): { classification: string; tools: string[]; confidence: number } {
    const lower = request.toLowerCase();
    
    // OS Operations
    if (lower.includes('battery') || lower.includes('brightness') || lower.includes('volume') || 
        lower.includes('tune') || lower.includes('optimize')) {
      return { classification: 'os', tools: ['os_exec'], confidence: 0.90 };
    }
    
    // Weather/Umbrella queries (multi-step)
    if (lower.includes('weather') || lower.includes('umbrella') || lower.includes('rain')) {
      return { classification: 'general', tools: ['web_api', 'memory_rag'], confidence: 0.85 };
    }
    
    // App operations
    if (lower.includes('install') || lower.includes('open') || lower.includes('launch') || lower.includes('start')) {
      return { classification: 'app', tools: ['app_proxy'], confidence: 0.88 };
    }
    
    // Creative tasks
    if (lower.includes('poem') || lower.includes('write') || lower.includes('create') || 
        lower.includes('story') || lower.includes('haiku')) {
      return { classification: 'creative', tools: ['gen_creative'], confidence: 0.82 };
    }
    
    // Knowledge queries
    if (lower.includes('what') || lower.includes('who') || lower.includes('how') || 
        lower.includes('why') || lower.includes('explain') || lower.includes('quantum') ||
        lower.includes('philosophy')) {
      return { classification: 'knowledge', tools: ['web_api', 'gen_creative'], confidence: 0.78 };
    }
    
    // Health/sensor
    if (lower.includes('health') || lower.includes('heart') || lower.includes('steps') || lower.includes('track')) {
      return { classification: 'health', tools: ['health_sensor'], confidence: 0.85 };
    }
    
    // General fallback
    return { classification: 'general', tools: ['web_api'], confidence: 0.65 };
  }

  private async executeChain(plan: { tools: string[] }, request: string): Promise<ToolOutput[]> {
    const outputs: ToolOutput[] = [];
    let previousOutput: string | null = null;

    for (const tool of plan.tools) {
      const output = await this.executeTool(tool, request, previousOutput);
      outputs.push(output);
      previousOutput = output.output;
    }

    return outputs;
  }

  private async executeTool(tool: string, request: string, context: string | null): Promise<ToolOutput> {
    await new Promise(resolve => setTimeout(resolve, 300)); // Simulate processing

    const lower = request.toLowerCase();

    switch (tool) {
      case 'os_exec':
        if (lower.includes('battery')) {
          return { tool_name: 'os_exec', output: 'Battery optimization enabled - Expected +15% runtime', confidence: 0.95 };
        } else if (lower.includes('brightness')) {
          return { tool_name: 'os_exec', output: 'Brightness adjusted to 70% - Adaptive mode enabled', confidence: 0.95 };
        } else if (lower.includes('volume')) {
          return { tool_name: 'os_exec', output: 'Volume set to 60%', confidence: 0.95 };
        }
        return { tool_name: 'os_exec', output: 'System configuration updated', confidence: 0.85 };

      case 'web_api':
        if (lower.includes('weather') || lower.includes('rain')) {
          return { tool_name: 'web_api', output: 'Paris: 15°C, 80% chance of rain, wind 12 km/h', confidence: 0.90 };
        } else if (lower.includes('quantum')) {
          return { tool_name: 'web_api', output: 'Quantum computing ethics: Balance innovation with safety, governance frameworks needed', confidence: 0.85 };
        }
        return { tool_name: 'web_api', output: `Search results for: ${request}`, confidence: 0.75 };

      case 'memory_rag':
        const historical = this.retrieveContext(request);
        if (historical.length > 0) {
          return { tool_name: 'memory_rag', output: `Context: ${historical[0]}`, confidence: 0.88 };
        }
        return { tool_name: 'memory_rag', output: 'Historical context: Rain pattern - 3/5 days last week. User prefers cover.', confidence: 0.80 };

      case 'app_proxy':
        if (lower.includes('code') || lower.includes('vscode')) {
          return { tool_name: 'app_proxy', output: 'VS Code opened in PWA container - Ready for development', confidence: 0.92 };
        } else if (lower.includes('music')) {
          return { tool_name: 'app_proxy', output: 'Music app launched - Playlist ready', confidence: 0.90 };
        }
        return { tool_name: 'app_proxy', output: `App launched: ${request}`, confidence: 0.85 };

      case 'gen_creative':
        if (lower.includes('love')) {
          return { 
            tool_name: 'gen_creative', 
            output: 'Roses bloom in crimson light,\nHearts entwined through day and night,\nLove\'s embrace, forever bright,\nTwo souls merged in pure delight.', 
            confidence: 0.88 
          };
        } else if (lower.includes('quantum')) {
          return { 
            tool_name: 'gen_creative', 
            output: 'In superposition\'s dance we dwell,\nWhere particles their secrets tell,\nEntangled states that weave and swell,\nReality\'s enigmatic spell.', 
            confidence: 0.85 
          };
        } else if (lower.includes('nature') || lower.includes('haiku')) {
          return { 
            tool_name: 'gen_creative', 
            output: 'Cherry blossoms fall,\nSilent whispers in the breeze,\nSpring\'s eternal song.', 
            confidence: 0.90 
          };
        }
        return { tool_name: 'gen_creative', output: `Creative content on: ${request}`, confidence: 0.80 };

      case 'health_sensor':
        if (lower.includes('heart')) {
          return { tool_name: 'health_sensor', output: 'Heart rate: 72 bpm (normal range)', confidence: 0.95 };
        } else if (lower.includes('steps')) {
          return { tool_name: 'health_sensor', output: 'Steps today: 8,432 - 67% of daily goal', confidence: 0.95 };
        }
        return { tool_name: 'health_sensor', output: 'All sensors nominal', confidence: 0.85 };

      default:
        return { tool_name: tool, output: 'Tool executed', confidence: 0.70 };
    }
  }

  private synthesizeSuggestion(request: string, chain: ToolOutput[], plan: { confidence: number }): string {
    if (chain.length === 0) {
      return '⚠️ Unable to process request - please clarify intent.';
    }

    if (chain.length === 1) {
      return chain[0].output;
    }

    // Multi-step synthesis
    const context = chain.map(o => `${o.tool_name}: ${o.output}`).join(' → ');
    const final = chain[chain.length - 1].output;
    
    return `Based on analysis: ${final} (confidence: ${Math.round(plan.confidence * 100)}%)`;
  }

  private generateManifest(response: AgenticResponse): OracleManifest {
    return {
      text: response.suggestion,
      voice_script: this.generateVoiceScript(response),
      haptic_pattern: this.getHapticPattern(response.confidence),
      confidence: response.confidence,
      reasoning_trace: response.reasoning_steps,
      historical_context: this.getHistoricalContext(),
      suggested_followup: this.suggestFollowup(response)
    };
  }

  private generateVoiceScript(response: AgenticResponse): string {
    if (response.confidence > 0.85) {
      return `✓ ${response.suggestion}`;
    } else if (response.confidence > 0.7) {
      return `Likely: ${response.suggestion}`;
    } else {
      return `Uncertain: ${response.suggestion}`;
    }
  }

  private getHapticPattern(confidence: number): 'Success' | 'Neutral' | 'Warning' | 'Error' {
    if (confidence > 0.85) return 'Success';
    if (confidence > 0.7) return 'Neutral';
    if (confidence > 0.5) return 'Warning';
    return 'Error';
  }

  private suggestFollowup(response: AgenticResponse): string | null {
    if (response.confidence < 0.7) {
      return 'Would you like me to search for more specific information?';
    }
    if (response.chain.length > 1) {
      return 'I used multiple sources - want detailed breakdown?';
    }
    return null;
  }

  private storeSession(intent: string, response: string, confidence: number): void {
    this.memory.set(intent, { response, score: confidence, timestamp: Date.now() });
    this.sessionHistory.push(intent);
    if (this.sessionHistory.length > 50) {
      this.sessionHistory.shift();
    }
  }

  private retrieveContext(intent: string): string[] {
    const context: string[] = [];
    const record = this.memory.get(intent);
    if (record && record.score > 0.7) {
      context.push(`Previous: ${record.response} (score: ${record.score.toFixed(2)})`);
    }
    return context;
  }

  private getHistoricalContext(): string[] {
    return this.sessionHistory.slice(-5).reverse();
  }

  async processFeedback(intent: string, helpful: boolean): Promise<void> {
    const record = this.memory.get(intent);
    if (record) {
      record.score = Math.max(0, Math.min(1, record.score + (helpful ? 0.1 : -0.1)));
      this.memory.set(intent, record);
    }
  }

  getAnalytics(): {
    totalSessions: number;
    avgConfidence: number;
    recentIntents: string[];
  } {
    const sessions = Array.from(this.memory.values());
    return {
      totalSessions: sessions.length,
      avgConfidence: sessions.length > 0 
        ? sessions.reduce((sum, s) => sum + s.score, 0) / sessions.length 
        : 0,
      recentIntents: this.sessionHistory.slice(-10).reverse()
    };
  }
}

export const universalOracle = new UniversalOracleService();
export type { OracleManifest };
