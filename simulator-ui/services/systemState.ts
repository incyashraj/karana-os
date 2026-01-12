/**
 * Kāraṇa OS - Complete System State Manager
 * 
 * Single source of truth for the ENTIRE system state across all 9 layers.
 * Enables the Oracle AI to have complete omniscience over the system.
 */

// =============================================================================
// Layer 1: Hardware State
// =============================================================================

export interface CameraState {
  active: boolean;
  mode: 'photo' | 'video' | 'ar' | 'idle';
  resolution: string;
  fps: number;
  exposure: number;
  whiteBalance: 'auto' | 'daylight' | 'cloudy' | 'tungsten';
  lastCapture?: number; // timestamp
}

export interface SensorState {
  imu: {
    accelerometer: { x: number; y: number; z: number };
    gyroscope: { x: number; y: number; z: number };
    magnetometer: { x: number; y: number; z: number };
  };
  gps: {
    latitude: number;
    longitude: number;
    altitude: number;
    accuracy: number;
  };
  proximity: number; // 0-1
  ambientLight: number; // lux
  temperature: number; // celsius
}

export interface DisplayState {
  brightness: number; // 0-1
  colorTemp: number; // kelvin
  refreshRate: number; // hz
  resolution: { width: number; height: number };
  mode: 'standard' | 'night' | 'outdoor' | 'power-saving';
}

export interface AudioState {
  volume: number; // 0-1
  micSensitivity: number; // 0-1
  noiseCancellation: boolean;
  spatialAudio: boolean;
  mode: 'call' | 'music' | 'ambient' | 'whisper';
}

export interface PowerState {
  batteryLevel: number; // 0-1
  charging: boolean;
  thermalState: 'normal' | 'warm' | 'hot' | 'critical';
  powerProfile: 'performance' | 'balanced' | 'power-saver' | 'minimal';
  estimatedRuntime: number; // minutes
}

export interface HardwareState {
  camera: CameraState;
  sensors: SensorState;
  display: DisplayState;
  audio: AudioState;
  power: PowerState;
}

// =============================================================================
// Layer 2: Network State
// =============================================================================

export interface PeerInfo {
  peerId: string;
  multiaddr: string;
  connectedAt: number;
  lastSeen: number;
  latency: number; // ms
  reputation: number; // 0-1
}

export interface NetworkState {
  peers: PeerInfo[];
  peerCount: number;
  syncStatus: 'synced' | 'syncing' | 'behind' | 'offline';
  blocksReceived: number;
  blocksSent: number;
  bytesReceived: number;
  bytesSent: number;
  latestBlock: number;
  connectionQuality: 'excellent' | 'good' | 'fair' | 'poor';
}

// =============================================================================
// Layer 3: Blockchain State
// =============================================================================

export interface WalletState {
  exists: boolean;
  did: string;
  publicKey: string;
  balance: number;
  nonce: number;
  locked: boolean;
}

export interface TransactionRecord {
  id: string;
  type: 'TRANSFER' | 'CONTRACT' | 'STAKE' | 'VOTE';
  from: string;
  to: string;
  amount: number;
  timestamp: number;
  status: 'pending' | 'confirmed' | 'failed';
  signature: string;
  blockHeight?: number;
  daTxHash?: string;
}

export interface BlockchainState {
  wallet: WalletState;
  transactions: TransactionRecord[];
  pendingTxCount: number;
  chainHeight: number;
  chainMode: 'full' | 'light' | 'minimal';
  governanceProposals: number;
  stakingActive: boolean;
}

// =============================================================================
// Layer 4: Oracle Bridge State
// =============================================================================

export interface IntentHistory {
  id: string;
  input: string;
  intent: string;
  confidence: number;
  executed: boolean;
  timestamp: number;
}

export interface OracleState {
  active: boolean;
  mode: 'listening' | 'processing' | 'executing' | 'idle';
  lastIntent?: IntentHistory;
  intentHistory: IntentHistory[];
  zkProofsGenerated: number;
  manifestsRendered: number;
}

// =============================================================================
// Layer 5: Intelligence State
// =============================================================================

export interface SceneInfo {
  objects: Array<{ name: string; confidence: number; bbox: number[] }>;
  sceneType: string;
  lighting: 'bright' | 'normal' | 'dim' | 'dark';
  depth: number[][]; // depth map
  semanticMap: string[][]; // semantic labels
}

export interface IntelligenceState {
  scene: SceneInfo | null;
  lastVisionAnalysis?: {
    object: string;
    confidence: number;
    timestamp: number;
  };
  contextHistory: Array<{
    type: 'vision' | 'voice' | 'gesture' | 'gaze';
    data: any;
    timestamp: number;
  }>;
  predictions: Array<{
    action: string;
    probability: number;
  }>;
}

// =============================================================================
// Layer 6: AI Engine State (Already covered by Oracle)
// =============================================================================

// =============================================================================
// Layer 7: Interface State
// =============================================================================

export interface HUDElement {
  id: string;
  type: 'notification' | 'widget' | 'overlay' | 'menu';
  position: { x: number; y: number; z: number };
  visible: boolean;
  priority: number;
}

export interface GestureState {
  enabled: boolean;
  tracking: boolean;
  lastGesture?: {
    type: string;
    confidence: number;
    timestamp: number;
  };
  vocabulary: string[]; // recognized gestures
}

export interface GazeState {
  enabled: boolean;
  tracking: boolean;
  target?: { x: number; y: number; z: number };
  dwellTime: number; // ms
  calibrated: boolean;
}

export interface VoiceState {
  enabled: boolean;
  mode: 'wake-word' | 'always-on' | 'push-to-talk' | 'off';
  wakeWord: string;
  language: string;
  lastCommand?: {
    text: string;
    confidence: number;
    timestamp: number;
  };
}

export interface InterfaceState {
  hud: {
    elements: HUDElement[];
    visible: boolean;
    opacity: number;
  };
  gesture: GestureState;
  gaze: GazeState;
  voice: VoiceState;
  arMode: boolean;
}

// =============================================================================
// Layer 8: Application State
// =============================================================================

export interface Timer {
  id: string;
  name: string;
  duration: number; // ms
  remaining: number; // ms
  state: 'running' | 'paused' | 'expired';
  createdAt: number;
}

export interface NavigationState {
  active: boolean;
  destination?: {
    name: string;
    coordinates: { lat: number; lng: number };
  };
  route?: any[];
  eta?: number; // seconds
  distance?: number; // meters
}

export interface SettingsConfig {
  [category: string]: {
    [key: string]: any;
  };
}

export interface WellnessStats {
  eyeStrain: number; // 0-1
  posture: number; // 0-1 (1 = good)
  usageTime: number; // minutes today
  breaks: number; // count today
  lastBreak?: number; // timestamp
}

export interface ApplicationState {
  timers: Timer[];
  navigation: NavigationState;
  settings: SettingsConfig;
  wellness: WellnessStats;
  androidApps: Array<{
    id: string;
    name: string;
    installed: boolean;
    running: boolean;
  }>;
}

// =============================================================================
// Layer 9: System Services State
// =============================================================================

export interface OTAState {
  updateAvailable: boolean;
  version?: string;
  size?: number; // bytes
  downloading: boolean;
  progress: number; // 0-1
  lastCheck: number; // timestamp
}

export interface SecurityState {
  mode: 'relaxed' | 'standard' | 'strict' | 'paranoid';
  biometricsEnabled: boolean;
  encryptionActive: boolean;
  permissions: {
    camera: boolean;
    microphone: boolean;
    location: boolean;
    network: boolean;
    storage: boolean;
  };
}

export interface DiagnosticsState {
  healthScore: number; // 0-1
  issues: Array<{
    severity: 'info' | 'warning' | 'error' | 'critical';
    message: string;
    timestamp: number;
  }>;
  lastDiagnosticRun: number;
  metrics: {
    cpu: number; // 0-1
    memory: number; // 0-1
    storage: number; // 0-1
    temperature: number; // celsius
  };
}

export interface SystemServicesState {
  ota: OTAState;
  security: SecurityState;
  diagnostics: DiagnosticsState;
  uptime: number; // seconds
  bootTime: number; // timestamp
  recoveryMode: boolean;
}

// =============================================================================
// AR/Spatial State (Cross-Cutting)
// =============================================================================

export interface SpatialAnchor {
  id: string;
  position: { x: number; y: number; z: number };
  rotation: { x: number; y: number; z: number; w: number };
  roomId: string;
  persistent: boolean;
  visualSignature: string;
  createdAt: number;
}

export interface ARTab {
  id: string;
  type: 'browser' | 'video' | 'editor' | 'terminal' | 'widget';
  anchorId: string;
  url?: string;
  content?: any;
  size: { width: number; height: number };
  visible: boolean;
}

export interface SpatialState {
  anchors: SpatialAnchor[];
  tabs: ARTab[];
  roomId: string;
  slamActive: boolean;
  trackingQuality: 'excellent' | 'good' | 'limited' | 'lost';
  currentPosition: { x: number; y: number; z: number };
  currentOrientation: { x: number; y: number; z: number; w: number };
}

// =============================================================================
// Complete System State
// =============================================================================

export interface CompleteSystemState {
  // Core Layers
  layer1_hardware: HardwareState;
  layer2_network: NetworkState;
  layer3_blockchain: BlockchainState;
  layer4_oracle: OracleState;
  layer5_intelligence: IntelligenceState;
  layer7_interface: InterfaceState;
  layer8_applications: ApplicationState;
  layer9_services: SystemServicesState;
  
  // Cross-Cutting
  spatial: SpatialState;
  
  // Meta
  lastUpdated: number;
  version: string;
}

// =============================================================================
// System State Manager Class
// =============================================================================

class SystemStateManager {
  private state: CompleteSystemState;
  private listeners: Map<string, Array<(state: CompleteSystemState) => void>> = new Map();
  private activityLog: Array<{ layer: string; action: string; timestamp: number }> = [];

  constructor() {
    this.state = this.initializeDefaultState();
  }

  private initializeDefaultState(): CompleteSystemState {
    return {
      layer1_hardware: {
        camera: {
          active: false,
          mode: 'idle',
          resolution: '1920x1080',
          fps: 30,
          exposure: 0.5,
          whiteBalance: 'auto',
        },
        sensors: {
          imu: {
            accelerometer: { x: 0, y: 0, z: 9.8 },
            gyroscope: { x: 0, y: 0, z: 0 },
            magnetometer: { x: 0, y: 0, z: 0 },
          },
          gps: { latitude: 0, longitude: 0, altitude: 0, accuracy: 0 },
          proximity: 0,
          ambientLight: 300,
          temperature: 25,
        },
        display: {
          brightness: 0.8,
          colorTemp: 6500,
          refreshRate: 60,
          resolution: { width: 1920, height: 1080 },
          mode: 'standard',
        },
        audio: {
          volume: 0.7,
          micSensitivity: 0.8,
          noiseCancellation: true,
          spatialAudio: true,
          mode: 'ambient',
        },
        power: {
          batteryLevel: 0.85,
          charging: false,
          thermalState: 'normal',
          powerProfile: 'balanced',
          estimatedRuntime: 180,
        },
      },
      layer2_network: {
        peers: [],
        peerCount: 0,
        syncStatus: 'synced',
        blocksReceived: 0,
        blocksSent: 0,
        bytesReceived: 0,
        bytesSent: 0,
        latestBlock: 0,
        connectionQuality: 'good',
      },
      layer3_blockchain: {
        wallet: {
          exists: false,
          did: '',
          publicKey: '',
          balance: 0,
          nonce: 0,
          locked: false,
        },
        transactions: [],
        pendingTxCount: 0,
        chainHeight: 0,
        chainMode: 'full',
        governanceProposals: 0,
        stakingActive: false,
      },
      layer4_oracle: {
        active: true,
        mode: 'idle',
        intentHistory: [],
        zkProofsGenerated: 0,
        manifestsRendered: 0,
      },
      layer5_intelligence: {
        scene: null,
        contextHistory: [],
        predictions: [],
      },
      layer7_interface: {
        hud: {
          elements: [],
          visible: true,
          opacity: 1,
        },
        gesture: {
          enabled: false,
          tracking: false,
          vocabulary: ['pinch', 'grab', 'push', 'swipe', 'rotate'],
        },
        gaze: {
          enabled: false,
          tracking: false,
          dwellTime: 0,
          calibrated: false,
        },
        voice: {
          enabled: true,
          mode: 'wake-word',
          wakeWord: 'Hey Karana',
          language: 'en-US',
        },
        arMode: false,
      },
      layer8_applications: {
        timers: [],
        navigation: {
          active: false,
        },
        settings: {},
        wellness: {
          eyeStrain: 0,
          posture: 1,
          usageTime: 0,
          breaks: 0,
        },
        androidApps: [],
      },
      layer9_services: {
        ota: {
          updateAvailable: false,
          downloading: false,
          progress: 0,
          lastCheck: Date.now(),
        },
        security: {
          mode: 'standard',
          biometricsEnabled: false,
          encryptionActive: true,
          permissions: {
            camera: true,
            microphone: true,
            location: true,
            network: true,
            storage: true,
          },
        },
        diagnostics: {
          healthScore: 0.95,
          issues: [],
          lastDiagnosticRun: Date.now(),
          metrics: {
            cpu: 0.3,
            memory: 0.4,
            storage: 0.5,
            temperature: 35,
          },
        },
        uptime: 0,
        bootTime: Date.now(),
        recoveryMode: false,
      },
      spatial: {
        anchors: [],
        tabs: [],
        roomId: '',
        slamActive: false,
        trackingQuality: 'good',
        currentPosition: { x: 0, y: 0, z: 0 },
        currentOrientation: { x: 0, y: 0, z: 0, w: 1 },
      },
      lastUpdated: Date.now(),
      version: '1.0.0',
    };
  }

  // Get complete state
  getState(): CompleteSystemState {
    return { ...this.state };
  }

  // Get state by layer
  getLayer(layer: keyof CompleteSystemState): any {
    return this.state[layer];
  }

  // Update state (partial)
  updateState(updates: Partial<CompleteSystemState>): void {
    this.state = {
      ...this.state,
      ...updates,
      lastUpdated: Date.now(),
    };
    this.notifyListeners();
  }

  // Update specific layer
  updateLayer(layer: keyof CompleteSystemState, data: any): void {
    (this.state[layer] as any) = {
      ...(this.state[layer] as any),
      ...data,
    };
    this.state.lastUpdated = Date.now();
    this.notifyListeners();
  }

  // Subscribe to state changes
  subscribe(id: string, callback: (state: CompleteSystemState) => void): void {
    if (!this.listeners.has(id)) {
      this.listeners.set(id, []);
    }
    this.listeners.get(id)!.push(callback);
  }

  // Unsubscribe from state changes
  unsubscribe(id: string): void {
    this.listeners.delete(id);
  }

  // Notify all listeners
  private notifyListeners(): void {
    this.listeners.forEach(callbacks => {
      callbacks.forEach(callback => callback(this.state));
    });
  }

  // Log activity
  logActivity(layer: string, action: string): void {
    this.activityLog.push({
      layer,
      action,
      timestamp: Date.now(),
    });
    // Keep last 1000 entries
    if (this.activityLog.length > 1000) {
      this.activityLog.shift();
    }
  }

  // Get activity log
  getActivityLog(layer?: string, limit: number = 50): Array<{ layer: string; action: string; timestamp: number }> {
    let log = this.activityLog;
    if (layer) {
      log = log.filter(entry => entry.layer === layer);
    }
    return log.slice(-limit);
  }

  // Generate context for AI
  getContextForAI(): string {
    const hw = this.state.layer1_hardware;
    const net = this.state.layer2_network;
    const bc = this.state.layer3_blockchain;
    const apps = this.state.layer8_applications;
    const sys = this.state.layer9_services;

    return `
COMPLETE SYSTEM STATE:

HARDWARE (Layer 1):
- Camera: ${hw.camera.active ? 'active' : 'inactive'} (${hw.camera.mode})
- Battery: ${(hw.power.batteryLevel * 100).toFixed(0)}% (${hw.power.thermalState}, ${hw.power.estimatedRuntime}min remaining)
- Display: ${(hw.display.brightness * 100).toFixed(0)}% brightness (${hw.display.mode})
- Power Profile: ${hw.power.powerProfile}
- Temperature: ${hw.sensors.temperature}°C

NETWORK (Layer 2):
- Peers: ${net.peerCount} connected
- Sync: ${net.syncStatus}
- Connection: ${net.connectionQuality}
- Latest Block: ${net.latestBlock}

BLOCKCHAIN (Layer 3):
- Wallet: ${bc.wallet.exists ? `${bc.wallet.balance} KARA` : 'not created'}
- DID: ${bc.wallet.did || 'none'}
- Transactions: ${bc.transactions.length} total, ${bc.pendingTxCount} pending
- Chain Mode: ${bc.chainMode}

APPLICATIONS (Layer 8):
- Timers: ${apps.timers.length} active
- Navigation: ${apps.navigation.active ? 'active' : 'inactive'}
- Android Apps: ${apps.androidApps.filter(a => a.installed).length} installed, ${apps.androidApps.filter(a => a.running).length} running
- Wellness: ${(apps.wellness.eyeStrain * 100).toFixed(0)}% eye strain, ${apps.wellness.usageTime}min usage today

SYSTEM SERVICES (Layer 9):
- Uptime: ${(sys.uptime / 60).toFixed(0)} minutes
- Health Score: ${(sys.diagnostics.healthScore * 100).toFixed(0)}%
- Security Mode: ${sys.security.mode}
- OTA: ${sys.ota.updateAvailable ? 'update available' : 'up to date'}
- Issues: ${sys.diagnostics.issues.length} active

SPATIAL (AR):
- Anchors: ${this.state.spatial.anchors.length}
- AR Tabs: ${this.state.spatial.tabs.length} open
- SLAM: ${this.state.spatial.slamActive ? 'active' : 'inactive'}
- Tracking: ${this.state.spatial.trackingQuality}

INTERFACE (Layer 7):
- HUD: ${this.state.layer7_interface.hud.visible ? 'visible' : 'hidden'} (${this.state.layer7_interface.hud.elements.length} elements)
- Voice: ${this.state.layer7_interface.voice.enabled ? 'enabled' : 'disabled'} (${this.state.layer7_interface.voice.mode})
- Gesture: ${this.state.layer7_interface.gesture.enabled ? 'enabled' : 'disabled'}
- Gaze: ${this.state.layer7_interface.gaze.enabled ? 'enabled' : 'disabled'}
- AR Mode: ${this.state.layer7_interface.arMode ? 'active' : 'inactive'}
`;
  }
}

// Export singleton instance
export const systemState = new SystemStateManager();
