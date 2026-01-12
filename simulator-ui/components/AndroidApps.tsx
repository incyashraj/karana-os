import React, { useState, useEffect } from 'react';

interface AndroidApp {
  id: string;
  name: string;
  package_name: string;
  category: string;
  icon: string;
  installed: boolean;
  running: boolean;
  version?: string;
  description: string;
}

interface AndroidAppsProps {
  onClose: () => void;
}

const POPULAR_APPS: AndroidApp[] = [
  {
    id: 'youtube',
    name: 'YouTube',
    package_name: 'com.google.android.youtube',
    category: 'Video',
    icon: '📺',
    installed: false,
    running: false,
    description: 'Watch videos with P2P streaming & local AI recommendations'
  },
  {
    id: 'whatsapp',
    name: 'WhatsApp',
    package_name: 'com.whatsapp',
    category: 'Messaging',
    icon: '💬',
    installed: false,
    running: false,
    description: 'Secure messaging with P2P network & enhanced privacy'
  },
  {
    id: 'instagram',
    name: 'Instagram',
    package_name: 'com.instagram.android',
    category: 'Social',
    icon: '📷',
    installed: false,
    running: false,
    description: 'Social media with local AI filters & content moderation'
  },
  {
    id: 'tiktok',
    name: 'TikTok',
    package_name: 'com.zhiliaoapp.musically',
    category: 'Video',
    icon: '🎵',
    installed: false,
    running: false,
    description: 'Short videos with heavy AI optimization'
  },
  {
    id: 'twitter',
    name: 'Twitter/X',
    package_name: 'com.twitter.android',
    category: 'Social',
    icon: '🐦',
    installed: false,
    running: false,
    description: 'Social network with privacy protection'
  },
  {
    id: 'spotify',
    name: 'Spotify',
    package_name: 'com.spotify.music',
    category: 'Music',
    icon: '🎧',
    installed: false,
    running: false,
    description: 'Music streaming with spatial audio'
  },
  {
    id: 'telegram',
    name: 'Telegram',
    package_name: 'org.telegram.messenger',
    category: 'Messaging',
    icon: '✈️',
    installed: false,
    running: false,
    description: 'Messaging with blockchain payments integration'
  },
  {
    id: 'facebook',
    name: 'Facebook',
    package_name: 'com.facebook.katana',
    category: 'Social',
    icon: '👥',
    installed: false,
    running: false,
    description: 'Social network with ad blocking & privacy layer'
  },
];

export const AndroidApps: React.FC<AndroidAppsProps> = ({ onClose }) => {
  const [apps, setApps] = useState<AndroidApp[]>(POPULAR_APPS);
  const [selectedCategory, setSelectedCategory] = useState<string>('All');
  const [searchQuery, setSearchQuery] = useState('');
  const [installing, setInstalling] = useState<string | null>(null);
  const [launching, setLaunching] = useState<string | null>(null);

  const categories = ['All', 'Video', 'Social', 'Messaging', 'Music'];

  const filteredApps = apps.filter(app => {
    const matchesCategory = selectedCategory === 'All' || app.category === selectedCategory;
    const matchesSearch = app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         app.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  const handleInstall = async (app: AndroidApp) => {
    setInstalling(app.id);
    
    try {
      // Simulate installation
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      setApps(prev => prev.map(a => 
        a.id === app.id ? { ...a, installed: true } : a
      ));
      
      // Show success notification
      console.log(`✅ ${app.name} installed successfully!`);
    } catch (error) {
      console.error('Installation failed:', error);
    } finally {
      setInstalling(null);
    }
  };

  const handleLaunch = async (app: AndroidApp) => {
    setLaunching(app.id);
    
    try {
      // Simulate launch
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setApps(prev => prev.map(a => 
        a.id === app.id ? { ...a, running: true } : a
      ));
      
      console.log(`🚀 ${app.name} launched!`);
      
      // Auto-close after launching
      setTimeout(() => onClose(), 500);
    } catch (error) {
      console.error('Launch failed:', error);
    } finally {
      setLaunching(null);
    }
  };

  const handleUninstall = async (app: AndroidApp) => {
    if (!confirm(`Uninstall ${app.name}?`)) return;
    
    setApps(prev => prev.map(a => 
      a.id === app.id ? { ...a, installed: false, running: false } : a
    ));
  };

  return (
    <div className="fixed inset-0 bg-black/90 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div className="bg-gray-900/95 border-2 border-cyan-500/50 rounded-2xl w-full max-w-6xl max-h-[90vh] overflow-hidden shadow-2xl shadow-cyan-500/20">
        {/* Header */}
        <div className="bg-gradient-to-r from-cyan-600/20 to-blue-600/20 border-b border-cyan-500/30 p-6">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h2 className="text-3xl font-bold text-cyan-400 hud-font">📱 Android Apps</h2>
              <p className="text-gray-400 mt-1">Run Android apps with AR optimizations</p>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white text-2xl w-10 h-10 flex items-center justify-center rounded-lg hover:bg-white/10 transition-colors"
            >
              ×
            </button>
          </div>

          {/* Search & Filter */}
          <div className="flex gap-4 flex-wrap">
            <input
              type="text"
              placeholder="🔍 Search apps..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="flex-1 min-w-[200px] bg-black/30 border border-cyan-500/30 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-cyan-500"
            />
            <div className="flex gap-2">
              {categories.map(cat => (
                <button
                  key={cat}
                  onClick={() => setSelectedCategory(cat)}
                  className={`px-4 py-2 rounded-lg font-medium transition-all ${
                    selectedCategory === cat
                      ? 'bg-cyan-500 text-black'
                      : 'bg-black/30 text-gray-400 hover:bg-white/10 hover:text-white'
                  }`}
                >
                  {cat}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Apps Grid */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-200px)]">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredApps.map(app => (
              <div
                key={app.id}
                className={`bg-gradient-to-br from-gray-800/50 to-gray-900/50 border rounded-xl p-5 transition-all ${
                  app.running
                    ? 'border-green-500/50 shadow-lg shadow-green-500/20'
                    : app.installed
                    ? 'border-cyan-500/30 hover:border-cyan-500/50'
                    : 'border-gray-700/30 hover:border-gray-600/50'
                }`}
              >
                {/* App Header */}
                <div className="flex items-start gap-3 mb-3">
                  <div className="text-4xl">{app.icon}</div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-xl font-bold text-white truncate">{app.name}</h3>
                    <p className="text-xs text-gray-500 truncate">{app.package_name}</p>
                    <span className="inline-block mt-1 px-2 py-0.5 bg-cyan-500/20 text-cyan-400 text-xs rounded">
                      {app.category}
                    </span>
                  </div>
                  {app.running && (
                    <div className="flex items-center gap-1 text-green-400 text-xs">
                      <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                      Running
                    </div>
                  )}
                </div>

                {/* Description */}
                <p className="text-sm text-gray-400 mb-4 line-clamp-2">{app.description}</p>

                {/* Optimizations Badge */}
                <div className="flex gap-2 mb-4 flex-wrap">
                  <span className="text-xs px-2 py-1 bg-blue-500/20 text-blue-400 rounded">🚀 HW Accel</span>
                  <span className="text-xs px-2 py-1 bg-purple-500/20 text-purple-400 rounded">🧠 Edge AI</span>
                  <span className="text-xs px-2 py-1 bg-green-500/20 text-green-400 rounded">🔒 Privacy</span>
                </div>

                {/* Actions */}
                <div className="flex gap-2">
                  {!app.installed ? (
                    <button
                      onClick={() => handleInstall(app)}
                      disabled={installing === app.id}
                      className="flex-1 bg-cyan-500 hover:bg-cyan-600 disabled:bg-gray-600 text-black font-bold py-2 px-4 rounded-lg transition-colors disabled:cursor-not-allowed"
                    >
                      {installing === app.id ? '⏳ Installing...' : '📥 Install'}
                    </button>
                  ) : (
                    <>
                      <button
                        onClick={() => handleLaunch(app)}
                        disabled={launching === app.id || app.running}
                        className="flex-1 bg-green-500 hover:bg-green-600 disabled:bg-gray-600 text-black font-bold py-2 px-4 rounded-lg transition-colors disabled:cursor-not-allowed"
                      >
                        {launching === app.id ? '⏳ Launching...' : app.running ? '✓ Running' : '▶️ Launch'}
                      </button>
                      <button
                        onClick={() => handleUninstall(app)}
                        className="bg-red-500/20 hover:bg-red-500/30 text-red-400 font-bold py-2 px-3 rounded-lg transition-colors"
                        title="Uninstall"
                      >
                        🗑️
                      </button>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>

          {filteredApps.length === 0 && (
            <div className="text-center py-12">
              <p className="text-gray-500 text-lg">No apps found</p>
              <p className="text-gray-600 text-sm mt-2">Try a different search or category</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-cyan-500/30 bg-gray-900/50 p-4">
          <div className="flex items-center justify-between text-sm">
            <div className="text-gray-400">
              <span className="text-cyan-400 font-bold">{apps.filter(a => a.installed).length}</span> installed
              <span className="mx-2">•</span>
              <span className="text-green-400 font-bold">{apps.filter(a => a.running).length}</span> running
            </div>
            <div className="text-gray-500 text-xs">
              💡 Tip: Say "Open YouTube" to launch via voice
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
