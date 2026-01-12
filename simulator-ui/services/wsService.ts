// WebSocket Client Service for karana-os
// Handles real-time voice AI updates and tool execution feedback

export interface ToolResult {
  tool_name: string;
  result: string;
  confidence: number;
  execution_id: string;
  timestamp: number;
}

export interface Transcription {
  text: string;
  is_partial: boolean;
  confidence: number;
}

export interface VoiceActivity {
  active: boolean;
  energy_level: number;
}

export interface StateUpdate {
  app_state: string;
  visible_elements: string[];
}

export interface WsError {
  message: string;
  code: string;
}

export type WsMessage =
  | { type: 'ToolResult'; tool_name: string; result: string; confidence: number; execution_id: string; timestamp: number }
  | { type: 'Transcription'; text: string; is_partial: boolean; confidence: number }
  | { type: 'VoiceActivity'; active: boolean; energy_level: number }
  | { type: 'StateUpdate'; app_state: string; visible_elements: string[] }
  | { type: 'Error'; message: string; code: string }
  | { type: 'Connected'; client_id: string; session_id: string }
  | { type: 'Ping' }
  | { type: 'Pong' };

export type MessageHandler = (message: WsMessage) => void;

export class WsService {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectInterval: number = 3000;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private handlers: Map<string, MessageHandler[]> = new Map();
  private clientId: string | null = null;
  private sessionId: string | null = null;
  private isConnecting: boolean = false;
  private isManualClose: boolean = false;
  private pingInterval: NodeJS.Timeout | null = null;

  constructor(url: string = 'ws://localhost:8080') {
    this.url = url;
  }

  /**
   * Connect to WebSocket server
   */
  public async connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('[WS] Already connected');
      return;
    }

    if (this.isConnecting) {
      console.log('[WS] Connection already in progress');
      return;
    }

    this.isConnecting = true;
    this.isManualClose = false;

    return new Promise((resolve, reject) => {
      try {
        console.log('[WS] Connecting to', this.url);
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          console.log('[WS] ✓ Connected');
          this.isConnecting = false;
          this.startPingInterval();
          resolve();
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        this.ws.onerror = (error) => {
          console.error('[WS] Error:', error);
          this.isConnecting = false;
          reject(error);
        };

        this.ws.onclose = (event) => {
          console.log('[WS] Connection closed:', event.code, event.reason);
          this.isConnecting = false;
          this.stopPingInterval();

          if (!this.isManualClose) {
            this.scheduleReconnect();
          }
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  public disconnect(): void {
    this.isManualClose = true;
    this.stopPingInterval();
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.clientId = null;
    this.sessionId = null;
    console.log('[WS] Disconnected');
  }

  /**
   * Send message to server
   */
  public send(message: WsMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('[WS] Cannot send - not connected');
    }
  }

  /**
   * Subscribe to specific message type
   */
  public on(messageType: string, handler: MessageHandler): () => void {
    if (!this.handlers.has(messageType)) {
      this.handlers.set(messageType, []);
    }
    this.handlers.get(messageType)!.push(handler);

    // Return unsubscribe function
    return () => {
      const handlers = this.handlers.get(messageType);
      if (handlers) {
        const index = handlers.indexOf(handler);
        if (index > -1) {
          handlers.splice(index, 1);
        }
      }
    };
  }

  /**
   * Subscribe to all messages
   */
  public onAny(handler: MessageHandler): () => void {
    return this.on('*', handler);
  }

  /**
   * Get connection status
   */
  public isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Get client ID (assigned by server)
   */
  public getClientId(): string | null {
    return this.clientId;
  }

  /**
   * Get session ID
   */
  public getSessionId(): string | null {
    return this.sessionId;
  }

  /**
   * Handle incoming message
   */
  private handleMessage(data: string): void {
    try {
      const message: WsMessage = JSON.parse(data);

      // Handle special message types
      switch (message.type) {
        case 'Connected':
          this.clientId = message.client_id;
          this.sessionId = message.session_id;
          console.log('[WS] Session established:', this.sessionId);
          break;
        case 'Ping':
          this.send({ type: 'Pong' });
          return;
        case 'Pong':
          // Received pong response
          return;
      }

      // Notify specific handlers
      const handlers = this.handlers.get(message.type) || [];
      handlers.forEach(handler => handler(message));

      // Notify wildcard handlers
      const wildcardHandlers = this.handlers.get('*') || [];
      wildcardHandlers.forEach(handler => handler(message));

    } catch (error) {
      console.error('[WS] Failed to parse message:', error);
    }
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimer || this.isManualClose) {
      return;
    }

    console.log(`[WS] Reconnecting in ${this.reconnectInterval}ms...`);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect().catch(error => {
        console.error('[WS] Reconnection failed:', error);
      });
    }, this.reconnectInterval);
  }

  /**
   * Start ping interval to keep connection alive
   */
  private startPingInterval(): void {
    this.stopPingInterval();
    this.pingInterval = setInterval(() => {
      if (this.isConnected()) {
        this.send({ type: 'Ping' });
      }
    }, 30000); // Ping every 30 seconds
  }

  /**
   * Stop ping interval
   */
  private stopPingInterval(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }
}

// Singleton instance
let wsServiceInstance: WsService | null = null;

/**
 * Get or create WsService singleton
 */
export function getWsService(url?: string): WsService {
  if (!wsServiceInstance) {
    wsServiceInstance = new WsService(url);
  }
  return wsServiceInstance;
}

/**
 * React hook for WebSocket connection
 */
export function useWebSocket(url?: string) {
  const [isConnected, setIsConnected] = React.useState(false);
  const [lastMessage, setLastMessage] = React.useState<WsMessage | null>(null);
  const wsService = React.useMemo(() => getWsService(url), [url]);

  React.useEffect(() => {
    // Connect on mount
    wsService.connect().then(() => {
      setIsConnected(true);
    }).catch(error => {
      console.error('[WS] Connection failed:', error);
      setIsConnected(false);
    });

    // Subscribe to all messages
    const unsubscribe = wsService.onAny((message) => {
      setLastMessage(message);
    });

    // Check connection status periodically
    const statusInterval = setInterval(() => {
      setIsConnected(wsService.isConnected());
    }, 1000);

    // Cleanup on unmount
    return () => {
      unsubscribe();
      clearInterval(statusInterval);
      // Note: Don't disconnect here - keep connection alive for other components
    };
  }, [wsService]);

  return {
    isConnected,
    lastMessage,
    send: (message: WsMessage) => wsService.send(message),
    subscribe: (type: string, handler: MessageHandler) => wsService.on(type, handler),
  };
}

// Import React for hook
import React from 'react';
