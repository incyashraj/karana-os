/**
 * Kāraṇa OS API Service
 * 
 * Connects the React frontend to the Rust backend API.
 * All operations go through real Ed25519 signing and Celestia DA.
 */

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws';

// =============================================================================
// Types (matching Rust API types)
// =============================================================================

export interface WalletInfo {
  did: string;
  public_key: string;
  balance: number;
  device_id: string;
}

export interface WalletCreationResponse {
  did: string;
  public_key: string;
  recovery_phrase: string[];
}

export interface SignedTransactionResponse {
  tx_hash: string;
  signature: string;
  sender: string;
  recipient: string;
  amount: number;
  timestamp: number;
  nonce: number;
}

export interface VisionAnalysisResponse {
  detected_object: string;
  category: string;
  description: string;
  confidence: number;
  related_tags: string[];
  processing_time_ms: number;
}

export interface OracleIntentResponse {
  intent_type: 
    | 'SPEAK' 
    | 'TRANSFER' 
    | 'ANALYZE' 
    | 'NAVIGATE' 
    | 'TIMER' 
    | 'WALLET'
    | 'OPEN_APP'
    | 'CLOSE_APP'
    | 'PLAY_VIDEO'
    | 'OPEN_BROWSER'
    | 'TAKE_NOTE'
    | 'SET_REMINDER'
    | 'PLAY_MUSIC'
    | 'HELP';
  content: string;
  data?: {
    amount?: number;
    recipient?: string;
    location?: string;
    duration?: string;
    app_type?: string;
    url?: string;
    query?: string;
    memo?: string;
  };
  requires_confirmation: boolean;
  suggested_actions: string[];
  confidence: number;
}

export interface Transaction {
  id: string;
  tx_type: string;
  amount: number;
  recipient: string;
  sender: string;
  timestamp: number;
  status: string;
  signature?: string;
  da_tx_hash?: string;
}

export interface OsStateInfo {
  mode: 'IDLE' | 'ANALYZING' | 'ORACLE' | 'NAVIGATION' | 'WALLET';
  version: string;
  uptime_seconds: number;
  wallet_connected: boolean;
  camera_active: boolean;
}

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// =============================================================================
// API Client
// =============================================================================

class KaranaApiClient {
  private baseUrl: string;
  private ws: WebSocket | null = null;
  private wsListeners: Map<string, ((data: any) => void)[]> = new Map();

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    });

    const json: ApiResponse<T> = await response.json();

    if (!json.success) {
      throw new Error(json.error || 'API request failed');
    }

    return json.data as T;
  }

  // ===========================================================================
  // Wallet Operations
  // ===========================================================================

  /**
   * Create a new wallet with Ed25519 keypair
   * Returns the recovery phrase - MUST be backed up!
   */
  async createWallet(): Promise<WalletCreationResponse> {
    return this.request<WalletCreationResponse>('/api/wallet/create', {
      method: 'POST',
    });
  }

  /**
   * Restore wallet from 24-word mnemonic phrase
   */
  async restoreWallet(mnemonic: string): Promise<WalletInfo> {
    return this.request<WalletInfo>('/api/wallet/restore', {
      method: 'POST',
      body: JSON.stringify({ mnemonic }),
    });
  }

  /**
   * Get current wallet info (DID, balance, etc.)
   */
  async getWalletInfo(): Promise<WalletInfo> {
    return this.request<WalletInfo>('/api/wallet/info');
  }

  /**
   * Sign and submit a transaction
   */
  async signTransaction(
    action: string,
    recipient: string,
    amount: number,
    memo?: string
  ): Promise<SignedTransactionResponse> {
    return this.request<SignedTransactionResponse>('/api/wallet/sign', {
      method: 'POST',
      body: JSON.stringify({ action, recipient, amount, memo }),
    });
  }

  /**
   * Get transaction history
   */
  async getTransactions(): Promise<Transaction[]> {
    return this.request<Transaction[]>('/api/wallet/transactions');
  }

  // ===========================================================================
  // AI Vision Operations
  // ===========================================================================

  /**
   * Analyze an image using on-device AI
   */
  async analyzeVision(imageBase64: string): Promise<VisionAnalysisResponse> {
    return this.request<VisionAnalysisResponse>('/api/ai/vision', {
      method: 'POST',
      body: JSON.stringify({ image_base64: imageBase64 }),
    });
  }

  // ===========================================================================
  // Oracle (NLP) Operations
  // ===========================================================================

  /**
   * Process natural language intent with context awareness
   */
  async processOracle(
    text: string, 
    context?: {
      vision_object?: string;
      wallet_balance?: number;
      active_app?: string;
    }
  ): Promise<OracleIntentResponse> {
    return this.request<OracleIntentResponse>('/api/ai/oracle', {
      method: 'POST',
      body: JSON.stringify({ text, context }),
    });
  }

  // ===========================================================================
  // Celestia DA Operations
  // ===========================================================================

  /**
   * Submit data to Celestia Data Availability layer
   */
  async submitToDA(data: string, namespace?: string): Promise<{ tx_hash: string; height: number }> {
    return this.request('/api/da/submit', {
      method: 'POST',
      body: JSON.stringify({ data, namespace }),
    });
  }

  /**
   * Check DA submission status
   */
  async getDAStatus(txHash: string): Promise<{ status: string; confirmations: number }> {
    return this.request(`/api/da/status/${txHash}`);
  }

  // ===========================================================================
  // OS State
  // ===========================================================================

  /**
   * Get current OS state
   */
  async getOsState(): Promise<OsStateInfo> {
    return this.request<OsStateInfo>('/api/os/state');
  }

  /**
   * Health check
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/health`);
      return response.ok;
    } catch {
      return false;
    }
  }

  // ===========================================================================
  // WebSocket Connection
  // ===========================================================================

  /**
   * Connect to WebSocket for real-time updates
   */
  connectWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(WS_URL);

      this.ws.onopen = () => {
        console.log('[WS] Connected to Kāraṇa OS');
        resolve();
      };

      this.ws.onerror = (error) => {
        console.error('[WS] Error:', error);
        reject(error);
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          const listeners = this.wsListeners.get(message.type) || [];
          listeners.forEach((listener) => listener(message));
        } catch (e) {
          console.error('[WS] Failed to parse message:', e);
        }
      };

      this.ws.onclose = () => {
        console.log('[WS] Disconnected from Kāraṇa OS');
        // Attempt reconnection after 3 seconds
        setTimeout(() => this.connectWebSocket(), 3000);
      };
    });
  }

  /**
   * Subscribe to a WebSocket channel
   */
  subscribe(channel: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'Subscribe', channel }));
    }
  }

  /**
   * Add listener for WebSocket events
   */
  onEvent(eventType: string, callback: (data: any) => void): () => void {
    const listeners = this.wsListeners.get(eventType) || [];
    listeners.push(callback);
    this.wsListeners.set(eventType, listeners);

    // Return unsubscribe function
    return () => {
      const current = this.wsListeners.get(eventType) || [];
      this.wsListeners.set(
        eventType,
        current.filter((l) => l !== callback)
      );
    };
  }

  /**
   * Disconnect WebSocket
   */
  disconnectWebSocket(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// Export singleton instance
export const karanaApi = new KaranaApiClient();

// Also export class for testing
export { KaranaApiClient };
