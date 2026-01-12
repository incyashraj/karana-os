/**
 * Android App Store Component for AR Workspace
 * 
 * Browse, download, install, and launch Android apps
 */

import React, { useState, useEffect } from 'react';
import { 
  androidAppManager, 
  AndroidApp, 
  AppDownloadProgress 
} from '../services/androidAppManager';
import { 
  Search, Download, Trash2, Play, Star, Users, 
  Package, CheckCircle, Loader, X, ArrowLeft,
  Grid3X3, List
} from 'lucide-react';

interface AppStoreProps {
  onLaunchApp?: (packageName: string, appInfo: { appName: string; apkPath?: string }) => void;
  onClose?: () => void;
}

const AppStore: React.FC<AppStoreProps> = ({ onLaunchApp, onClose }) => {
  const [view, setView] = useState<'store' | 'installed' | 'details'>('store');
  const [apps, setApps] = useState<AndroidApp[]>([]);
  const [installedApps, setInstalledApps] = useState<AndroidApp[]>([]);
  const [selectedApp, setSelectedApp] = useState<AndroidApp | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [downloadProgress, setDownloadProgress] = useState<Map<string, AppDownloadProgress>>(new Map());
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  
  // Load apps
  useEffect(() => {
    refreshApps();
  }, []);
  
  const refreshApps = () => {
    const storeApps = androidAppManager.getAppStore();
    const installed = androidAppManager.getInstalledApps();
    setApps(storeApps);
    setInstalledApps(installed);
  };
  
  // Filter apps
  const filteredApps = apps.filter(app => {
    const matchesSearch = app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         app.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = selectedCategory === 'all' || app.category === selectedCategory;
    return matchesSearch && matchesCategory;
  });
  
  // Handle install
  const handleInstall = async (app: AndroidApp) => {
    try {
      // Subscribe to progress
      const unsubscribe = androidAppManager.subscribeToDownload(app.packageName, (progress) => {
        setDownloadProgress(prev => new Map(prev).set(app.packageName, progress));
      });
      
      await androidAppManager.downloadAndInstall(app.packageName);
      
      // Refresh apps
      refreshApps();
      
      // Cleanup
      unsubscribe();
      setDownloadProgress(prev => {
        const next = new Map(prev);
        next.delete(app.packageName);
        return next;
      });
      
    } catch (error: any) {
      alert(`Failed to install: ${error.message}`);
    }
  };
  
  // Handle uninstall
  const handleUninstall = (packageName: string) => {
    if (confirm('Are you sure you want to uninstall this app?')) {
      androidAppManager.uninstallApp(packageName);
      refreshApps();
      if (selectedApp?.packageName === packageName) {
        setSelectedApp(null);
        setView('store');
      }
    }
  };
  
  // Handle launch
  const handleLaunch = (app: AndroidApp) => {
    const result = androidAppManager.launchApp(app.packageName);
    if (result.error) {
      alert(result.error);
    } else if (result.success && onLaunchApp) {
      // Show native app launch notification
      console.log(`🚀 Launching native Android app: ${app.name}`);
      console.log(`   Package: ${app.packageName}`);
      console.log(`   APK Path: ${result.apkPath}`);
      
      // Notify parent component
      onLaunchApp(app.packageName, {
        appName: result.appName,
        apkPath: result.apkPath
      });
      
      // Show success message
      alert(`🚀 Launching ${app.name}\n\nPackage: ${app.packageName}\nAPK: ${result.apkPath}`);
    }
  };
  
  // Show app details
  const showDetails = (app: AndroidApp) => {
    setSelectedApp(app);
    setView('details');
  };
  
  const categories = [
    { id: 'all', label: 'All', icon: '📱' },
    { id: 'productivity', label: 'Productivity', icon: '💼' },
    { id: 'social', label: 'Social', icon: '👥' },
    { id: 'entertainment', label: 'Entertainment', icon: '🎬' },
    { id: 'tools', label: 'Tools', icon: '🔧' },
    { id: 'games', label: 'Games', icon: '🎮' },
    { id: 'news', label: 'News', icon: '📰' },
    { id: 'health', label: 'Health', icon: '💪' },
  ];
  
  return (
    <div className="h-full flex flex-col bg-transparent text-white">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-white/10">
        <div className="flex items-center gap-3">
          {view !== 'store' && (
            <button
              onClick={() => setView('store')}
              className="p-2 hover:bg-white/10 rounded-lg transition"
            >
              <ArrowLeft size={20} />
            </button>
          )}
          <h2 className="text-xl font-bold">
            {view === 'store' ? '📱 App Store' : 
             view === 'installed' ? '📦 Installed Apps' : 
             selectedApp?.name}
          </h2>
        </div>
        
        <div className="flex items-center gap-2">
          <button
            onClick={() => setView('installed')}
            className={`px-3 py-2 rounded-lg transition ${
              view === 'installed' ? 'bg-blue-600' : 'bg-white/10 hover:bg-white/20'
            }`}
          >
            <Package size={18} />
          </button>
          
          {view === 'store' && (
            <button
              onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
              className="p-2 bg-white/10 hover:bg-white/20 rounded-lg transition"
            >
              {viewMode === 'grid' ? <List size={18} /> : <Grid3X3 size={18} />}
            </button>
          )}
          
          {onClose && (
            <button
              onClick={onClose}
              className="p-2 hover:bg-white/10 rounded-lg transition"
            >
              <X size={20} />
            </button>
          )}
        </div>
      </div>
      
      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {view === 'store' && (
          <div className="h-full flex flex-col">
            {/* Search */}
            <div className="p-4 border-b border-white/10">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
                <input
                  type="text"
                  placeholder="Search apps..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:border-blue-500"
                />
              </div>
            </div>
            
            {/* Categories */}
            <div className="px-4 py-3 border-b border-white/10 overflow-x-auto">
              <div className="flex gap-2">
                {categories.map(cat => (
                  <button
                    key={cat.id}
                    onClick={() => setSelectedCategory(cat.id)}
                    className={`px-4 py-2 rounded-full text-sm whitespace-nowrap transition ${
                      selectedCategory === cat.id
                        ? 'bg-blue-600 text-white'
                        : 'bg-white/5 text-gray-300 hover:bg-white/10'
                    }`}
                  >
                    {cat.icon} {cat.label}
                  </button>
                ))}
              </div>
            </div>
            
            {/* Apps Grid/List */}
            <div className="flex-1 overflow-y-auto p-4">
              {viewMode === 'grid' ? (
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                  {filteredApps.map(app => (
                    <AppCard
                      key={app.packageName}
                      app={app}
                      progress={downloadProgress.get(app.packageName)}
                      onInstall={() => handleInstall(app)}
                      onUninstall={() => handleUninstall(app.packageName)}
                      onLaunch={() => handleLaunch(app)}
                      onClick={() => showDetails(app)}
                    />
                  ))}
                </div>
              ) : (
                <div className="space-y-2">
                  {filteredApps.map(app => (
                    <AppListItem
                      key={app.packageName}
                      app={app}
                      progress={downloadProgress.get(app.packageName)}
                      onInstall={() => handleInstall(app)}
                      onUninstall={() => handleUninstall(app.packageName)}
                      onLaunch={() => handleLaunch(app)}
                      onClick={() => showDetails(app)}
                    />
                  ))}
                </div>
              )}
              
              {filteredApps.length === 0 && (
                <div className="flex flex-col items-center justify-center h-full text-gray-400">
                  <Package size={48} className="mb-2" />
                  <p>No apps found</p>
                </div>
              )}
            </div>
          </div>
        )}
        
        {view === 'installed' && (
          <div className="h-full overflow-y-auto p-4">
            {installedApps.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-gray-400">
                <Package size={48} className="mb-2" />
                <p>No apps installed</p>
                <button
                  onClick={() => setView('store')}
                  className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                  Browse App Store
                </button>
              </div>
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
                {installedApps.map(app => (
                  <AppCard
                    key={app.packageName}
                    app={app}
                    onUninstall={() => handleUninstall(app.packageName)}
                    onLaunch={() => handleLaunch(app)}
                    onClick={() => showDetails(app)}
                  />
                ))}
              </div>
            )}
          </div>
        )}
        
        {view === 'details' && selectedApp && (
          <AppDetails
            app={selectedApp}
            progress={downloadProgress.get(selectedApp.packageName)}
            onInstall={() => handleInstall(selectedApp)}
            onUninstall={() => handleUninstall(selectedApp.packageName)}
            onLaunch={() => handleLaunch(selectedApp)}
            onBack={() => setView('store')}
          />
        )}
      </div>
    </div>
  );
};

// App Card Component
const AppCard: React.FC<{
  app: AndroidApp;
  progress?: AppDownloadProgress;
  onInstall?: () => void;
  onUninstall?: () => void;
  onLaunch?: () => void;
  onClick?: () => void;
}> = ({ app, progress, onInstall, onUninstall, onLaunch, onClick }) => {
  return (
    <div
      onClick={onClick}
      className="bg-white/5 rounded-lg p-4 hover:bg-gray-750 transition cursor-pointer"
    >
      <div className="text-4xl mb-2">{app.icon}</div>
      <h3 className="font-semibold text-sm mb-1 truncate">{app.name}</h3>
      <div className="flex items-center gap-1 text-xs text-gray-400 mb-2">
        <Star size={12} className="text-yellow-400" fill="currentColor" />
        <span>{app.rating}</span>
      </div>
      
      {progress ? (
        <div className="space-y-1">
          <div className="text-xs text-blue-400">{progress.status}...</div>
          <div className="h-2 bg-white/10 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-600 transition-all"
              style={{ width: `${progress.progress}%` }}
            />
          </div>
        </div>
      ) : app.isInstalled ? (
        <div className="flex gap-2">
          <button
            onClick={(e) => { e.stopPropagation(); onLaunch?.(); }}
            className="flex-1 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-xs transition"
          >
            <Play size={14} className="inline mr-1" />
            Open
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); onUninstall?.(); }}
            className="px-2 py-1.5 bg-red-600 hover:bg-red-700 rounded text-xs transition"
          >
            <Trash2 size={14} />
          </button>
        </div>
      ) : (
        <button
          onClick={(e) => { e.stopPropagation(); onInstall?.(); }}
          className="w-full px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-xs transition"
        >
          <Download size={14} className="inline mr-1" />
          Install
        </button>
      )}
    </div>
  );
};

// App List Item Component
const AppListItem: React.FC<{
  app: AndroidApp;
  progress?: AppDownloadProgress;
  onInstall?: () => void;
  onUninstall?: () => void;
  onLaunch?: () => void;
  onClick?: () => void;
}> = ({ app, progress, onInstall, onUninstall, onLaunch, onClick }) => {
  return (
    <div
      onClick={onClick}
      className="bg-white/5 rounded-lg p-3 hover:bg-gray-750 transition cursor-pointer flex items-center gap-3"
    >
      <div className="text-3xl">{app.icon}</div>
      <div className="flex-1 min-w-0">
        <h3 className="font-semibold text-sm truncate">{app.name}</h3>
        <p className="text-xs text-gray-400 truncate">{app.description}</p>
        <div className="flex items-center gap-2 text-xs text-gray-500 mt-1">
          <span className="flex items-center gap-0.5">
            <Star size={10} className="text-yellow-400" fill="currentColor" />
            {app.rating}
          </span>
          <span>•</span>
          <span>{app.size.toFixed(1)} MB</span>
        </div>
      </div>
      
      <div>
        {progress ? (
          <div className="text-xs text-blue-400">{progress.progress}%</div>
        ) : app.isInstalled ? (
          <div className="flex gap-2">
            <button
              onClick={(e) => { e.stopPropagation(); onLaunch?.(); }}
              className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-xs transition"
            >
              Open
            </button>
          </div>
        ) : (
          <button
            onClick={(e) => { e.stopPropagation(); onInstall?.(); }}
            className="px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-xs transition"
          >
            Install
          </button>
        )}
      </div>
    </div>
  );
};

// App Details Component
const AppDetails: React.FC<{
  app: AndroidApp;
  progress?: AppDownloadProgress;
  onInstall: () => void;
  onUninstall: () => void;
  onLaunch: () => void;
  onBack: () => void;
}> = ({ app, progress, onInstall, onUninstall, onLaunch, onBack }) => {
  return (
    <div className="h-full overflow-y-auto">
      {/* Hero */}
      <div className="bg-gradient-to-b from-gray-800 to-gray-900 p-6">
        <div className="flex items-start gap-4">
          <div className="text-6xl">{app.icon}</div>
          <div className="flex-1">
            <h2 className="text-2xl font-bold mb-1">{app.name}</h2>
            <p className="text-gray-400 text-sm mb-2">{app.developer}</p>
            <div className="flex items-center gap-4 text-sm">
              <span className="flex items-center gap-1">
                <Star size={14} className="text-yellow-400" fill="currentColor" />
                {app.rating}
              </span>
              <span className="flex items-center gap-1">
                <Users size={14} />
                {app.downloads}
              </span>
              <span>{app.size.toFixed(1)} MB</span>
            </div>
          </div>
        </div>
        
        {/* Action Button */}
        <div className="mt-4">
          {progress ? (
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>{progress.status}...</span>
                <span>{progress.progress}%</span>
              </div>
              <div className="h-3 bg-white/10 rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-600 transition-all"
                  style={{ width: `${progress.progress}%` }}
                />
              </div>
            </div>
          ) : app.isInstalled ? (
            <div className="flex gap-2">
              <button
                onClick={onLaunch}
                className="flex-1 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-semibold transition"
              >
                <Play size={18} className="inline mr-2" />
                Open App
              </button>
              <button
                onClick={onUninstall}
                className="px-4 py-3 bg-red-600 hover:bg-red-700 rounded-lg transition"
              >
                <Trash2 size={18} />
              </button>
            </div>
          ) : (
            <button
              onClick={onInstall}
              className="w-full px-6 py-3 bg-green-600 hover:bg-green-700 rounded-lg font-semibold transition"
            >
              <Download size={18} className="inline mr-2" />
              Install ({app.size.toFixed(1)} MB)
            </button>
          )}
        </div>
      </div>
      
      {/* Details */}
      <div className="p-6 space-y-6">
        <div>
          <h3 className="font-semibold mb-2">About</h3>
          <p className="text-gray-400 text-sm">{app.description}</p>
        </div>
        
        <div>
          <h3 className="font-semibold mb-2">Information</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Version</span>
              <span>{app.version}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Category</span>
              <span className="capitalize">{app.category}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Last Updated</span>
              <span>{app.lastUpdated.toLocaleDateString()}</span>
            </div>
          </div>
        </div>
        
        <div>
          <h3 className="font-semibold mb-2">Permissions</h3>
          <div className="space-y-1">
            {app.permissions.map((perm, idx) => (
              <div key={idx} className="flex items-center gap-2 text-sm text-gray-400">
                <CheckCircle size={14} className="text-green-500" />
                {perm}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default AppStore;
