export enum AppMode {
  IDLE = 'IDLE',
  ANALYZING = 'ANALYZING',
  ORACLE = 'ORACLE',
  NAVIGATION = 'NAVIGATION',
  WALLET = 'WALLET',
  AR_WORKSPACE = 'AR_WORKSPACE'
}

export interface AnalysisResult {
  detectedObject: string;
  category: string;
  description: string;
  confidence: number;
  relatedTags: string[];
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  text: string;
  timestamp: number;
  metadata?: any;
  manifest?: OracleManifest;
}

export interface OracleManifest {
  text: string;
  voice_script: string;
  haptic_pattern: 'Success' | 'Neutral' | 'Warning' | 'Error';
  confidence: number;
  reasoning_trace: string[];
  historical_context: string[];
  suggested_followup: string | null;
}

export type IntentType = 'SPEAK' | 'TRANSFER' | 'ANALYZE' | 'NAVIGATE' | 'TIMER' | 'WALLET';

export interface OracleIntent {
  type: IntentType;
  content: string; // The natural language response
  data?: {
    amount?: number;
    recipient?: string;
    location?: string;
    duration?: string;
    objectOfInterest?: string;
  };
}

export interface WalletState {
  balance: number;
  did: string;
  transactions: Transaction[];
}

export interface Transaction {
  id: string;
  type: 'TRANSFER' | 'STAKE' | 'VOTE';
  amount: number;
  recipient: string;
  timestamp: number;
  status: 'PENDING' | 'CONFIRMED' | 'FAILED';
  signature?: string; // Ed25519 signature simulation
}

// ========================
// TIMER TYPES
// ========================
export type TimerType = 'COUNTDOWN' | 'STOPWATCH' | 'RECURRING';
export type TimerState = 'RUNNING' | 'PAUSED' | 'COMPLETED' | 'CANCELLED';

export interface OSTimer {
  id: string;
  type: TimerType;
  label: string;
  durationMs: number;
  remainingMs: number;
  state: TimerState;
  createdAt: number;
  completesAt?: number;
  recurring?: {
    interval: 'DAILY' | 'WEEKLY' | 'CUSTOM';
    customMs?: number;
  };
}

// ========================
// NOTIFICATION TYPES  
// ========================
export type NotificationPriority = 'LOW' | 'NORMAL' | 'HIGH' | 'URGENT';
export type NotificationCategory = 'SYSTEM' | 'TIMER' | 'TRANSACTION' | 'GOVERNANCE' | 'SOCIAL' | 'GENERAL';

export interface OSNotification {
  id: string;
  title: string;
  body: string;
  priority: NotificationPriority;
  category: NotificationCategory;
  timestamp: number;
  read: boolean;
  actionUrl?: string;
  metadata?: Record<string, any>;
}

// ========================
// GOVERNANCE TYPES
// ========================
export type ProposalStatus = 'ACTIVE' | 'PASSED' | 'REJECTED' | 'EXECUTED' | 'CANCELLED';

export interface Proposal {
  id: string;
  title: string;
  description: string;
  proposer: string;
  status: ProposalStatus;
  votesFor: number;
  votesAgainst: number;
  createdAt: number;
  endsAt: number;
  executionData?: string;
}

// ========================
// UI TYPES
// ========================
export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface ActionStep {
  id: string;
  label: string;
  status: 'pending' | 'active' | 'completed' | 'failed';
}

export interface Risk {
  level: 'low' | 'medium' | 'high';
  description: string;
}

export interface Suggestion {
  id: string;
  text: string;
  icon?: string;
}
