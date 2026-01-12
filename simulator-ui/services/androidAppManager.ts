/**
 * Android App Management Service
 * 
 * Handles downloading, installing, and managing Android apps on Kāraṇa OS
 * Simulates Android app ecosystem with real app store browsing
 */

// ==================== TYPE DEFINITIONS ====================

export interface AndroidApp {
  packageName: string;
  name: string;
  version: string;
  description: string;
  category: 'productivity' | 'social' | 'entertainment' | 'tools' | 'games' | 'education' | 'lifestyle' | 'news' | 'health' | 'finance';
  icon: string;
  size: number; // in MB
  rating: number; // 0-5
  downloads: string;
  developer: string;
  permissions: string[];
  screenshots: string[];
  isInstalled: boolean;
  installedVersion?: string;
  lastUpdated: Date;
  apkUrl: string; // Real APK download URL
  downloadedApkPath?: string; // Local path after download
}

export interface AppDownloadProgress {
  packageName: string;
  appName: string;
  progress: number; // 0-100
  status: 'downloading' | 'installing' | 'complete' | 'error';
  error?: string;
}

export interface InstalledApp extends AndroidApp {
  installDate: Date;
  lastOpened?: Date;
  usageTime: number; // in minutes
}

// ==================== APP STORE DATA ====================

const APP_STORE: AndroidApp[] = [
  // Productivity
  {
    packageName: 'com.google.docs',
    name: 'Google Docs',
    version: '1.24.52',
    description: 'Create and edit documents on the go',
    category: 'productivity',
    icon: '📝',
    size: 45.2,
    rating: 4.3,
    downloads: '1B+',
    developer: 'Google LLC',
    permissions: ['Storage', 'Network', 'Camera'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-15'),
    apkUrl: 'https://www.apkmirror.com/apk/google-inc/docs/docs-1-24-52-release/'
  },
  {
    packageName: 'com.notion.app',
    name: 'Notion',
    version: '0.6.2350',
    description: 'Notes, tasks, wikis, and databases',
    category: 'productivity',
    icon: '📋',
    size: 67.8,
    rating: 4.6,
    downloads: '50M+',
    developer: 'Notion Labs',
    permissions: ['Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-01'),
    apkUrl: 'https://www.apkpure.com/notion/com.notion.app'
  },
  {
    packageName: 'com.todoist',
    name: 'Todoist',
    version: '9.8.2',
    description: 'To-do list and task manager',
    category: 'productivity',
    icon: '✅',
    size: 34.5,
    rating: 4.5,
    downloads: '10M+',
    developer: 'Doist Inc.',
    permissions: ['Network', 'Notifications'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-28'),
    apkUrl: 'https://www.apkpure.com/todoist/com.todoist'
  },
  
  // Social
  {
    packageName: 'com.twitter.android',
    name: 'X (Twitter)',
    version: '10.23.0',
    description: 'Social networking platform',
    category: 'social',
    icon: '𝕏',
    size: 89.3,
    rating: 3.9,
    downloads: '500M+',
    developer: 'X Corp.',
    permissions: ['Camera', 'Microphone', 'Location', 'Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-05'),
    apkUrl: 'https://www.apkmirror.com/apk/twitter-inc/twitter/'
  },
  {
    packageName: 'com.discord',
    name: 'Discord',
    version: '223.14',
    description: 'Talk, chat, hang out',
    category: 'social',
    icon: '💬',
    size: 102.7,
    rating: 4.4,
    downloads: '500M+',
    developer: 'Discord Inc.',
    permissions: ['Camera', 'Microphone', 'Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-03'),
    apkUrl: 'https://www.apkpure.com/discord/com.discord'
  },
  
  // Entertainment
  {
    packageName: 'com.spotify.music',
    name: 'Spotify',
    version: '8.9.16.534',
    description: 'Music and podcasts',
    category: 'entertainment',
    icon: '🎵',
    size: 78.4,
    rating: 4.5,
    downloads: '1B+',
    developer: 'Spotify Ltd.',
    permissions: ['Storage', 'Network', 'Microphone'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-02'),
    apkUrl: 'https://www.apkpure.com/spotify/com.spotify.music'
  },
  {
    packageName: 'com.netflix.mediaclient',
    name: 'Netflix',
    version: '8.107.1',
    description: 'Watch TV shows and movies',
    category: 'entertainment',
    icon: '🎬',
    size: 134.2,
    rating: 4.3,
    downloads: '1B+',
    developer: 'Netflix Inc.',
    permissions: ['Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-30'),
    apkUrl: 'https://www.apkmirror.com/apk/netflix-inc/netflix/'
  },
  {
    packageName: 'com.youtube.android',
    name: 'YouTube',
    version: '19.47.53',
    description: 'Watch videos, music, and more',
    category: 'entertainment',
    icon: '▶️',
    size: 156.8,
    rating: 4.2,
    downloads: '10B+',
    developer: 'Google LLC',
    permissions: ['Camera', 'Microphone', 'Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-04'),
    apkUrl: 'https://www.apkmirror.com/apk/google-inc/youtube/'
  },
  
  // Tools
  {
    packageName: 'com.google.android.apps.maps',
    name: 'Google Maps',
    version: '11.114.0101',
    description: 'Real-time GPS navigation',
    category: 'tools',
    icon: '🗺️',
    size: 112.5,
    rating: 4.4,
    downloads: '10B+',
    developer: 'Google LLC',
    permissions: ['Location', 'Network', 'Storage'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-01'),
    apkUrl: 'https://www.apkmirror.com/apk/google-inc/maps/'
  },
  {
    packageName: 'com.google.android.apps.translate',
    name: 'Google Translate',
    version: '8.14.62',
    description: 'Translate between languages',
    category: 'tools',
    icon: '🌐',
    size: 56.3,
    rating: 4.5,
    downloads: '1B+',
    developer: 'Google LLC',
    permissions: ['Camera', 'Microphone', 'Network', 'Storage'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-25'),
    apkUrl: 'https://www.apkmirror.com/apk/google-inc/translate/'
  },
  
  // Games
  {
    packageName: 'com.chess.app',
    name: 'Chess',
    version: '4.7.2',
    description: 'Play chess online or offline',
    category: 'games',
    icon: '♟️',
    size: 45.8,
    rating: 4.7,
    downloads: '100M+',
    developer: 'Chess.com',
    permissions: ['Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-20'),
    apkUrl: 'https://www.apkpure.com/chess/com.chess.app'
  },
  
  // News
  {
    packageName: 'com.nytimes.android',
    name: 'NYTimes',
    version: '10.36.0',
    description: 'Breaking news and stories',
    category: 'news',
    icon: '📰',
    size: 67.2,
    rating: 4.3,
    downloads: '10M+',
    developer: 'The New York Times',
    permissions: ['Network', 'Notifications'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-12-05'),
    apkUrl: 'https://www.apkpure.com/nytimes/com.nytimes.android'
  },
  
  // Health & Fitness
  {
    packageName: 'com.fitbit.FitbitMobile',
    name: 'Fitbit',
    version: '4.23',
    description: 'Health and fitness tracker',
    category: 'health',
    icon: '💪',
    size: 143.6,
    rating: 4.2,
    downloads: '100M+',
    developer: 'Fitbit LLC',
    permissions: ['Location', 'Bluetooth', 'Storage', 'Network'],
    screenshots: [],
    isInstalled: false,
    lastUpdated: new Date('2024-11-28'),
    apkUrl: 'https://www.apkpure.com/fitbit/com.fitbit.FitbitMobile'
  },
];

// ==================== ANDROID APP MANAGER ====================

class AndroidAppManager {
  private installedApps: Map<string, InstalledApp> = new Map();
  private downloadQueue: AppDownloadProgress[] = [];
  private listeners: Map<string, Set<(progress: AppDownloadProgress) => void>> = new Map();
  
  constructor() {
    // Load installed apps from localStorage
    this.loadInstalledApps();
  }
  
  // Get all available apps from store
  getAppStore(): AndroidApp[] {
    return APP_STORE.map(app => ({
      ...app,
      isInstalled: this.installedApps.has(app.packageName)
    }));
  }
  
  // Search apps
  searchApps(query: string): AndroidApp[] {
    const lowerQuery = query.toLowerCase();
    return this.getAppStore().filter(app =>
      app.name.toLowerCase().includes(lowerQuery) ||
      app.description.toLowerCase().includes(lowerQuery) ||
      app.category.includes(lowerQuery)
    );
  }
  
  // Get apps by category
  getAppsByCategory(category: AndroidApp['category']): AndroidApp[] {
    return this.getAppStore().filter(app => app.category === category);
  }
  
  // Get app details
  getAppDetails(packageName: string): AndroidApp | null {
    const app = APP_STORE.find(a => a.packageName === packageName);
    if (!app) return null;
    
    return {
      ...app,
      isInstalled: this.installedApps.has(packageName),
      installedVersion: this.installedApps.get(packageName)?.installedVersion
    };
  }
  
  // Download and install app
  async downloadAndInstall(packageName: string): Promise<void> {
    const app = APP_STORE.find(a => a.packageName === packageName);
    if (!app) {
      throw new Error('App not found in store');
    }
    
    if (this.installedApps.has(packageName)) {
      throw new Error('App already installed');
    }
    
    // Create download progress
    const progress: AppDownloadProgress = {
      packageName,
      appName: app.name,
      progress: 0,
      status: 'downloading'
    };
    
    this.downloadQueue.push(progress);
    
    // Simulate APK download from real URL
    return new Promise((resolve, reject) => {
      console.log(`📥 Downloading APK from: ${app.apkUrl}`);
      
      const downloadInterval = setInterval(() => {
        progress.progress += 8; // Slower for realistic download
        
        // Phase transitions
        if (progress.progress >= 60 && progress.status === 'downloading') {
          progress.status = 'installing';
          console.log(`📦 Installing ${app.name} APK...`);
        }
        
        this.notifyListeners(packageName, progress);
        
        if (progress.progress >= 100) {
          clearInterval(downloadInterval);
          progress.status = 'complete';
          this.notifyListeners(packageName, progress);
          
          // Simulate APK installation
          const apkPath = `/data/app/${packageName}-${Date.now()}/base.apk`;
          console.log(`✅ APK installed at: ${apkPath}`);
          
          // Install app
          const installedApp: InstalledApp = {
            ...app,
            isInstalled: true,
            installedVersion: app.version,
            installDate: new Date(),
            usageTime: 0,
            downloadedApkPath: apkPath
          };
          
          this.installedApps.set(packageName, installedApp);
          this.saveInstalledApps();
          
          // Remove from download queue
          this.downloadQueue = this.downloadQueue.filter(d => d.packageName !== packageName);
          
          console.log(`🎉 ${app.name} successfully installed!`);
          resolve();
        }
      }, 400); // ~5 seconds for full download + install
    });
  }
  
  // Uninstall app
  uninstallApp(packageName: string): boolean {
    if (!this.installedApps.has(packageName)) {
      return false;
    }
    
    this.installedApps.delete(packageName);
    this.saveInstalledApps();
    return true;
  }
  
  // Get installed apps
  getInstalledApps(): InstalledApp[] {
    return Array.from(this.installedApps.values()).sort((a, b) => 
      b.installDate.getTime() - a.installDate.getTime()
    );
  }
  
  // Launch app (native Android activity)
  launchApp(packageName: string): { success: boolean; error?: string; appName?: string; apkPath?: string } {
    const app = this.installedApps.get(packageName);
    if (!app) {
      return { success: false, error: 'App not installed' };
    }
    
    if (!app.downloadedApkPath) {
      return { success: false, error: 'APK file not found' };
    }
    
    // Update last opened
    app.lastOpened = new Date();
    this.saveInstalledApps();
    
    // Launch native Android activity
    console.log(`🚀 Launching native app: ${app.name}`);
    console.log(`   Package: ${packageName}`);
    console.log(`   APK: ${app.downloadedApkPath}`);
    console.log(`   Activity: ${packageName}.MainActivity`);
    
    return { 
      success: true, 
      appName: app.name,
      apkPath: app.downloadedApkPath
    };
  }
  
  // Update app usage
  updateUsage(packageName: string, minutes: number): void {
    const app = this.installedApps.get(packageName);
    if (app) {
      app.usageTime += minutes;
      this.saveInstalledApps();
    }
  }
  
  // Subscribe to download progress
  subscribeToDownload(packageName: string, callback: (progress: AppDownloadProgress) => void): () => void {
    if (!this.listeners.has(packageName)) {
      this.listeners.set(packageName, new Set());
    }
    
    this.listeners.get(packageName)!.add(callback);
    
    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(packageName);
      if (listeners) {
        listeners.delete(callback);
        if (listeners.size === 0) {
          this.listeners.delete(packageName);
        }
      }
    };
  }
  
  private notifyListeners(packageName: string, progress: AppDownloadProgress): void {
    const listeners = this.listeners.get(packageName);
    if (listeners) {
      listeners.forEach(callback => callback(progress));
    }
  }
  
  // Persistence
  private saveInstalledApps(): void {
    const data = Array.from(this.installedApps.entries()).map(([pkg, app]) => ({
      packageName: pkg,
      version: app.installedVersion || app.version,
      installDate: app.installDate.toISOString(),
      lastOpened: app.lastOpened?.toISOString(),
      usageTime: app.usageTime
    }));
    
    localStorage.setItem('karana_installed_apps', JSON.stringify(data));
  }
  
  private loadInstalledApps(): void {
    try {
      const data = localStorage.getItem('karana_installed_apps');
      if (!data) return;
      
      const installed = JSON.parse(data);
      installed.forEach((item: any) => {
        const app = APP_STORE.find(a => a.packageName === item.packageName);
        if (app) {
          const installedApp: InstalledApp = {
            ...app,
            isInstalled: true,
            installedVersion: item.version,
            installDate: new Date(item.installDate),
            lastOpened: item.lastOpened ? new Date(item.lastOpened) : undefined,
            usageTime: item.usageTime || 0
          };
          this.installedApps.set(item.packageName, installedApp);
        }
      });
    } catch (error) {
      console.error('Failed to load installed apps:', error);
    }
  }
  
  // Get download queue
  getDownloadQueue(): AppDownloadProgress[] {
    return this.downloadQueue;
  }
}

// ==================== EXPORT ====================

export const androidAppManager = new AndroidAppManager();
