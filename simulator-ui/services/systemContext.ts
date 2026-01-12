/**
 * System Context Service
 * 
 * Makes the Oracle AI aware of EVERYTHING in the system:
 * - Available apps (installed, not installed, running)
 * - System capabilities (vision, wallet, AR workspace, etc.)
 * - User preferences and history
 * - Current state (mode, active apps, wallet status)
 * 
 * This is the "eyes and ears" of the AI
 */

export interface AndroidApp {
  id: string;
  name: string;
  package_name: string;
  category: string;
  installed: boolean;
  running: boolean;
  description: string;
  capabilities: string[];
}

export interface SystemCapability {
  id: string;
  name: string;
  description: string;
  available: boolean;
  keywords: string[];
}

export interface SystemState {
  mode: string;
  walletConnected: boolean;
  walletBalance: number;
  walletDid: string;
  backendConnected: boolean;
  installedApps: AndroidApp[];
  runningApps: string[];
  recentActivity: string[];
  availableCapabilities: SystemCapability[];
}

export class SystemContext {
  private static instance: SystemContext;
  
  // All available Android apps
  private availableApps: AndroidApp[] = [
    {
      id: 'youtube',
      name: 'YouTube',
      package_name: 'com.google.android.youtube',
      category: 'Video',
      installed: false,
      running: false,
      description: 'Watch videos with P2P streaming & local AI recommendations',
      capabilities: ['video', 'streaming', 'entertainment', 'watch']
    },
    {
      id: 'whatsapp',
      name: 'WhatsApp',
      package_name: 'com.whatsapp',
      category: 'Messaging',
      installed: false,
      running: false,
      description: 'Secure messaging with P2P network & enhanced privacy',
      capabilities: ['messaging', 'chat', 'call', 'communication']
    },
    {
      id: 'instagram',
      name: 'Instagram',
      package_name: 'com.instagram.android',
      category: 'Social',
      installed: false,
      running: false,
      description: 'Social media with local AI filters & content moderation',
      capabilities: ['social', 'photos', 'stories', 'feed']
    },
    {
      id: 'tiktok',
      name: 'TikTok',
      package_name: 'com.zhiliaoapp.musically',
      category: 'Video',
      installed: false,
      running: false,
      description: 'Short videos with heavy AI optimization',
      capabilities: ['video', 'entertainment', 'social', 'watch']
    },
    {
      id: 'twitter',
      name: 'Twitter',
      package_name: 'com.twitter.android',
      category: 'Social',
      installed: false,
      running: false,
      description: 'Social network with privacy protection',
      capabilities: ['social', 'news', 'feed', 'posts']
    },
    {
      id: 'spotify',
      name: 'Spotify',
      package_name: 'com.spotify.music',
      category: 'Music',
      installed: false,
      running: false,
      description: 'Music streaming with spatial audio',
      capabilities: ['music', 'audio', 'streaming', 'entertainment']
    },
    {
      id: 'telegram',
      name: 'Telegram',
      package_name: 'org.telegram.messenger',
      category: 'Messaging',
      installed: false,
      running: false,
      description: 'Messaging with blockchain payments integration',
      capabilities: ['messaging', 'chat', 'communication', 'payments']
    },
    {
      id: 'facebook',
      name: 'Facebook',
      package_name: 'com.facebook.katana',
      category: 'Social',
      installed: false,
      running: false,
      description: 'Social network with ad blocking & privacy layer',
      capabilities: ['social', 'feed', 'photos', 'news']
    },
    {
      id: 'netflix',
      name: 'Netflix',
      package_name: 'com.netflix.mediaclient',
      category: 'Video',
      installed: false,
      running: false,
      description: 'Streaming service with optimized playback',
      capabilities: ['video', 'streaming', 'entertainment', 'movies', 'tv']
    },
    {
      id: 'gmail',
      name: 'Gmail',
      package_name: 'com.google.android.gm',
      category: 'Productivity',
      installed: false,
      running: false,
      description: 'Email with privacy-enhanced features',
      capabilities: ['email', 'communication', 'productivity']
    },
    {
      id: 'chrome',
      name: 'Chrome',
      package_name: 'com.android.chrome',
      category: 'Productivity',
      installed: false,
      running: false,
      description: 'Web browser with decentralized features',
      capabilities: ['browser', 'web', 'search', 'internet']
    },
    {
      id: 'maps',
      name: 'Google Maps',
      package_name: 'com.google.android.apps.maps',
      category: 'Navigation',
      installed: false,
      running: false,
      description: 'Navigation with offline-first mode',
      capabilities: ['navigation', 'maps', 'location', 'directions']
    }
  ];

  // System capabilities
  private capabilities: SystemCapability[] = [
    {
      id: 'vision',
      name: 'Computer Vision',
      description: 'Analyze what you see in real-time',
      available: true,
      keywords: ['analyze', 'see', 'look', 'identify', 'recognize', 'scan', 'vision']
    },
    {
      id: 'wallet',
      name: 'Digital Wallet',
      description: 'Manage cryptocurrency and digital identity',
      available: true,
      keywords: ['wallet', 'balance', 'send', 'pay', 'transfer', 'money', 'crypto', 'did']
    },
    {
      id: 'oracle',
      name: 'AI Assistant',
      description: 'Natural language interface to control everything',
      available: true,
      keywords: ['help', 'what', 'how', 'why', 'explain', 'tell', 'show']
    },
    {
      id: 'ar_workspace',
      name: 'AR Workspace',
      description: 'Create and manipulate 3D objects in space',
      available: true,
      keywords: ['workspace', 'ar', '3d', 'create', 'place', 'object']
    },
    {
      id: 'android_apps',
      name: 'Android Apps',
      description: 'Install and run Android applications',
      available: true,
      keywords: ['app', 'install', 'open', 'launch', 'run', 'start', 'close']
    },
    {
      id: 'timers',
      name: 'Timers & Reminders',
      description: 'Set timers and get reminders',
      available: true,
      keywords: ['timer', 'reminder', 'alarm', 'notify', 'alert', 'schedule']
    },
    {
      id: 'notifications',
      name: 'Notifications',
      description: 'System-wide notification management',
      available: true,
      keywords: ['notification', 'alert', 'message', 'notify']
    },
    {
      id: 'settings',
      name: 'System Settings',
      description: 'Configure OS preferences',
      available: true,
      keywords: ['settings', 'preferences', 'configure', 'setup', 'options']
    }
  ];

  private currentState: SystemState = {
    mode: 'IDLE',
    walletConnected: false,
    walletBalance: 0,
    walletDid: 'Not Connected',
    backendConnected: false,
    installedApps: [],
    runningApps: [],
    recentActivity: [],
    availableCapabilities: this.capabilities
  };

  private constructor() {}

  static getInstance(): SystemContext {
    if (!SystemContext.instance) {
      SystemContext.instance = new SystemContext();
    }
    return SystemContext.instance;
  }

  // Update system state
  updateState(updates: Partial<SystemState>) {
    this.currentState = { ...this.currentState, ...updates };
  }

  // Update app installation status
  updateAppStatus(appId: string, installed: boolean, running: boolean = false) {
    const app = this.availableApps.find(a => a.id === appId);
    if (app) {
      app.installed = installed;
      app.running = running;
      
      // Update installed/running lists
      if (installed) {
        if (!this.currentState.installedApps.find(a => a.id === appId)) {
          this.currentState.installedApps.push(app);
        }
      } else {
        this.currentState.installedApps = this.currentState.installedApps.filter(a => a.id !== appId);
      }
      
      if (running) {
        if (!this.currentState.runningApps.includes(appId)) {
          this.currentState.runningApps.push(appId);
        }
      } else {
        this.currentState.runningApps = this.currentState.runningApps.filter(id => id !== appId);
      }
    }
  }

  // Find app by name or keyword
  findApp(query: string): AndroidApp | null {
    const lowerQuery = query.toLowerCase();
    
    // Direct name match
    let app = this.availableApps.find(a => 
      a.name.toLowerCase() === lowerQuery || 
      a.id.toLowerCase() === lowerQuery
    );
    
    if (app) return app;
    
    // Partial name match
    app = this.availableApps.find(a => 
      a.name.toLowerCase().includes(lowerQuery) ||
      a.id.toLowerCase().includes(lowerQuery)
    );
    
    if (app) return app;
    
    // Capability match
    app = this.availableApps.find(a => 
      a.capabilities.some(cap => cap.includes(lowerQuery) || lowerQuery.includes(cap))
    );
    
    return app;
  }

  // Find capability by keyword
  findCapability(query: string): SystemCapability | null {
    const lowerQuery = query.toLowerCase();
    return this.capabilities.find(cap => 
      cap.keywords.some(kw => lowerQuery.includes(kw) || kw.includes(lowerQuery))
    );
  }

  // Get current system context for AI
  getContextForAI(): string {
    const state = this.currentState;
    
    const context = `
SYSTEM STATE:
- Mode: ${state.mode}
- Backend: ${state.backendConnected ? 'Connected' : 'Disconnected'}
- Wallet: ${state.walletConnected ? `Connected (${state.walletDid}, Balance: ${state.walletBalance})` : 'Not Connected'}

INSTALLED APPS (${state.installedApps.length}):
${state.installedApps.map(app => `- ${app.name} (${app.running ? 'Running' : 'Ready'})`).join('\n') || '- None installed'}

AVAILABLE APPS TO INSTALL (${this.availableApps.filter(a => !a.installed).length}):
${this.availableApps.filter(a => !a.installed).map(app => 
  `- ${app.name}: ${app.description} (Keywords: ${app.capabilities.join(', ')})`
).join('\n')}

SYSTEM CAPABILITIES:
${this.capabilities.map(cap => 
  `- ${cap.name}: ${cap.description} (Keywords: ${cap.keywords.join(', ')})`
).join('\n')}

RECENT ACTIVITY:
${state.recentActivity.slice(-5).map((activity, i) => `${i + 1}. ${activity}`).join('\n') || '- No recent activity'}
`.trim();
    
    return context;
  }

  // Add activity to history
  addActivity(activity: string) {
    this.currentState.recentActivity.push(`[${new Date().toLocaleTimeString()}] ${activity}`);
    if (this.currentState.recentActivity.length > 50) {
      this.currentState.recentActivity = this.currentState.recentActivity.slice(-50);
    }
  }

  // Get all available apps
  getAllApps(): AndroidApp[] {
    return [...this.availableApps];
  }

  // Get installed apps
  getInstalledApps(): AndroidApp[] {
    return this.availableApps.filter(a => a.installed);
  }

  // Get running apps
  getRunningApps(): AndroidApp[] {
    return this.availableApps.filter(a => a.running);
  }

  // Get current state
  getState(): SystemState {
    return { ...this.currentState };
  }
}

export const systemContext = SystemContext.getInstance();
