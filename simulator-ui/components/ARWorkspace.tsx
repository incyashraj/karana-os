import React, { useState, useCallback, useImperativeHandle, forwardRef } from 'react';
import ARWindow from './ARWindow';
import { 
  Monitor, Video, Globe, Terminal, FileText, Image, Music, Calendar, Mail, 
  Plus, Layers, Grid3X3, X, Sparkles, Box, Layout
} from 'lucide-react';

export interface WindowConfig {
  id: string;
  title: string;
  type: 'video' | 'browser' | 'terminal' | 'notes' | 'image' | 'music' | 'calendar' | 'mail' | 'custom';
  position: { x: number; y: number };
  size: { width: number; height: number };
  depth: number;
  url?: string;
  content?: React.ReactNode;
}

// Expose methods for external control (e.g., from Oracle)
export interface ARWorkspaceHandle {
  spawnApp: (type: WindowConfig['type'], options?: { url?: string; title?: string }) => string;
  closeApp: (id: string) => void;
  getOpenWindows: () => WindowConfig[];
  focusApp: (id: string) => void;
  closeAllApps: () => void;
}

interface ARWorkspaceProps {
  isActive: boolean;
  onClose: () => void;
  onWindowsChange?: (windows: WindowConfig[]) => void;
}

const SAMPLE_VIDEOS = [
  { name: 'Demo Reel', url: '' },
  { name: 'Tutorial', url: '' },
  { name: 'Presentation', url: '' },
];

const APP_TEMPLATES: Record<WindowConfig['type'], Omit<WindowConfig, 'id' | 'position'>> = {
  video: { title: 'Video Player', type: 'video', size: { width: 640, height: 400 }, depth: 0 },
  browser: { title: 'Browser', type: 'browser', size: { width: 800, height: 500 }, depth: 0 },
  terminal: { title: 'Terminal', type: 'terminal', size: { width: 500, height: 350 }, depth: 0 },
  notes: { title: 'Notes', type: 'notes', size: { width: 400, height: 500 }, depth: 0 },
  image: { title: 'Gallery', type: 'image', size: { width: 500, height: 400 }, depth: 0 },
  music: { title: 'Music', type: 'music', size: { width: 350, height: 450 }, depth: 0 },
  calendar: { title: 'Calendar', type: 'calendar', size: { width: 350, height: 400 }, depth: 0 },
  mail: { title: 'Mail', type: 'mail', size: { width: 450, height: 500 }, depth: 0 },
  custom: { title: 'Custom', type: 'custom', size: { width: 500, height: 400 }, depth: 0 },
};

const APP_TEMPLATE_LIST = Object.values(APP_TEMPLATES).filter(t => t.type !== 'custom');

const ARWorkspace = forwardRef<ARWorkspaceHandle, ARWorkspaceProps>(({ isActive, onClose, onWindowsChange }, ref) => {
  const [windows, setWindows] = useState<WindowConfig[]>([]);
  const [focusedWindowId, setFocusedWindowId] = useState<string | null>(null);
  const [showLauncher, setShowLauncher] = useState(true);
  const [layoutMode, setLayoutMode] = useState<'free' | 'grid' | 'stack'>('free');

  // Generate unique ID
  const generateId = () => `window_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;

  // Calculate next window position
  const getNextPosition = useCallback(() => {
    const offset = windows.length * 40;
    return {
      x: 100 + (offset % 200),
      y: 80 + (offset % 150)
    };
  }, [windows.length]);

  // Spawn new window
  const spawnWindow = useCallback((template: Omit<WindowConfig, 'id' | 'position'>, options?: { url?: string }) => {
    const newWindow: WindowConfig = {
      ...template,
      id: generateId(),
      position: getNextPosition(),
      url: options?.url || template.url
    };
    setWindows(prev => {
      const updated = [...prev, newWindow];
      onWindowsChange?.(updated);
      return updated;
    });
    setFocusedWindowId(newWindow.id);
    setShowLauncher(false);
    return newWindow.id;
  }, [getNextPosition, onWindowsChange]);

  // Spawn app by type (for Oracle control)
  const spawnApp = useCallback((type: WindowConfig['type'], options?: { url?: string; title?: string }) => {
    const template = APP_TEMPLATES[type];
    if (!template) return '';
    return spawnWindow({
      ...template,
      title: options?.title || template.title
    }, { url: options?.url });
  }, [spawnWindow]);

  // Close window
  const closeWindow = useCallback((id: string) => {
    setWindows(prev => {
      const updated = prev.filter(w => w.id !== id);
      onWindowsChange?.(updated);
      return updated;
    });
    if (focusedWindowId === id) {
      setFocusedWindowId(windows.length > 1 ? windows[windows.length - 2]?.id : null);
    }
  }, [focusedWindowId, windows.length, onWindowsChange]);

  // Close all windows
  const closeAllApps = useCallback(() => {
    setWindows([]);
    setFocusedWindowId(null);
    onWindowsChange?.([]);
  }, [onWindowsChange]);

  // Focus window
  const focusWindow = useCallback((id: string) => {
    setFocusedWindowId(id);
    // Move to end of array for z-index ordering
    setWindows(prev => {
      const window = prev.find(w => w.id === id);
      if (!window) return prev;
      return [...prev.filter(w => w.id !== id), window];
    });
  }, []);

  // Get open windows (for Oracle queries)
  const getOpenWindows = useCallback(() => windows, [windows]);

  // Expose imperative handle for Oracle control
  useImperativeHandle(ref, () => ({
    spawnApp,
    closeApp: closeWindow,
    getOpenWindows,
    focusApp: focusWindow,
    closeAllApps
  }), [spawnApp, closeWindow, getOpenWindows, focusWindow, closeAllApps]);

  // Layout windows in grid
  const applyGridLayout = () => {
    const cols = Math.ceil(Math.sqrt(windows.length));
    const cellWidth = Math.floor(window.innerWidth / cols) - 40;
    const cellHeight = Math.floor((window.innerHeight - 100) / Math.ceil(windows.length / cols)) - 40;
    
    setWindows(prev => prev.map((w, i) => ({
      ...w,
      position: {
        x: (i % cols) * (cellWidth + 40) + 20,
        y: Math.floor(i / cols) * (cellHeight + 40) + 60
      },
      size: {
        width: Math.max(300, cellWidth),
        height: Math.max(200, cellHeight)
      },
      depth: 0
    })));
    setLayoutMode('grid');
  };

  // Stack windows in cascade
  const applyStackLayout = () => {
    setWindows(prev => prev.map((w, i) => ({
      ...w,
      position: { x: 100 + i * 50, y: 80 + i * 40 },
      size: { width: 600, height: 400 },
      depth: -i * 0.5
    })));
    setLayoutMode('stack');
  };

  // Arrange windows in 3D arc
  const apply3DArcLayout = () => {
    const count = windows.length;
    const centerX = window.innerWidth / 2;
    const radius = 400;
    
    setWindows(prev => prev.map((w, i) => {
      const angle = ((i - (count - 1) / 2) / Math.max(1, count - 1)) * 0.8;
      return {
        ...w,
        position: {
          x: centerX + Math.sin(angle) * radius - 250,
          y: 120
        },
        size: { width: 500, height: 350 },
        depth: Math.cos(angle) * 2 - 1
      };
    }));
    setLayoutMode('free');
  };

  if (!isActive) return null;

  return (
    <div className="fixed inset-0 z-40">
      {/* Spatial Grid Background */}
      <div className="absolute inset-0 pointer-events-none">
        <div 
          className="w-full h-full opacity-10"
          style={{
            backgroundImage: `
              linear-gradient(rgba(59, 130, 246, 0.3) 1px, transparent 1px),
              linear-gradient(90deg, rgba(59, 130, 246, 0.3) 1px, transparent 1px)
            `,
            backgroundSize: '50px 50px',
            perspective: '1000px',
            transform: 'rotateX(60deg)',
            transformOrigin: 'center bottom'
          }}
        />
      </div>

      {/* Windows */}
      {windows.map((windowConfig, index) => (
        <ARWindow
          key={windowConfig.id}
          id={windowConfig.id}
          title={windowConfig.title}
          type={windowConfig.type}
          initialPosition={windowConfig.position}
          initialSize={windowConfig.size}
          initialDepth={windowConfig.depth}
          url={windowConfig.url}
          content={windowConfig.content}
          onClose={closeWindow}
          onFocus={focusWindow}
          isFocused={focusedWindowId === windowConfig.id}
          zIndex={index}
        />
      ))}

      {/* App Launcher Overlay */}
      {showLauncher && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/40 backdrop-blur-sm z-50">
          <div className="glass-panel p-8 rounded-3xl border border-white/20 max-w-2xl w-full mx-4 animate-in zoom-in-95 duration-300">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-cyan-500/30 to-blue-500/30 flex items-center justify-center">
                  <Sparkles className="text-cyan-400" size={24} />
                </div>
                <div>
                  <h2 className="text-xl font-bold text-white">AR Workspace</h2>
                  <p className="text-sm text-gray-400">Spatial Computing • Vision Pro Style</p>
                </div>
              </div>
              <button 
                onClick={onClose}
                className="p-2 hover:bg-white/10 rounded-full transition-colors"
              >
                <X size={20} className="text-gray-400" />
              </button>
            </div>

            {/* Quick Launch Apps */}
            <div className="grid grid-cols-4 gap-4 mb-6">
              {APP_TEMPLATE_LIST.map((app, i) => (
                <button
                  key={i}
                  onClick={() => spawnWindow(app)}
                  className="flex flex-col items-center gap-2 p-4 rounded-2xl bg-white/5 hover:bg-white/10 border border-white/10 hover:border-white/20 transition-all group"
                >
                  <div className={`w-14 h-14 rounded-2xl flex items-center justify-center transition-transform group-hover:scale-110 ${
                    app.type === 'video' ? 'bg-red-500/20 text-red-400' :
                    app.type === 'browser' ? 'bg-blue-500/20 text-blue-400' :
                    app.type === 'terminal' ? 'bg-green-500/20 text-green-400' :
                    app.type === 'notes' ? 'bg-yellow-500/20 text-yellow-400' :
                    app.type === 'image' ? 'bg-purple-500/20 text-purple-400' :
                    app.type === 'music' ? 'bg-pink-500/20 text-pink-400' :
                    app.type === 'calendar' ? 'bg-orange-500/20 text-orange-400' :
                    'bg-indigo-500/20 text-indigo-400'
                  }`}>
                    {app.type === 'video' && <Video size={24} />}
                    {app.type === 'browser' && <Globe size={24} />}
                    {app.type === 'terminal' && <Terminal size={24} />}
                    {app.type === 'notes' && <FileText size={24} />}
                    {app.type === 'image' && <Image size={24} />}
                    {app.type === 'music' && <Music size={24} />}
                    {app.type === 'calendar' && <Calendar size={24} />}
                    {app.type === 'mail' && <Mail size={24} />}
                  </div>
                  <span className="text-xs font-medium text-gray-300">{app.title}</span>
                </button>
              ))}
            </div>

            {/* Layout Options */}
            {windows.length > 1 && (
              <div className="flex items-center justify-center gap-3 pt-4 border-t border-white/10">
                <span className="text-xs text-gray-500 mr-2">Arrange:</span>
                <button 
                  onClick={apply3DArcLayout}
                  className="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs flex items-center gap-1.5"
                >
                  <Box size={14} /> 3D Arc
                </button>
                <button 
                  onClick={applyGridLayout}
                  className="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs flex items-center gap-1.5"
                >
                  <Grid3X3 size={14} /> Grid
                </button>
                <button 
                  onClick={applyStackLayout}
                  className="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs flex items-center gap-1.5"
                >
                  <Layers size={14} /> Stack
                </button>
              </div>
            )}

            {/* Tips */}
            <div className="mt-6 pt-4 border-t border-white/10 text-center">
              <p className="text-xs text-gray-500">
                💡 Drag windows to reposition • Use Z controls for depth • Pinch to resize
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Floating Controls */}
      {!showLauncher && (
        <div className="fixed bottom-24 right-6 flex flex-col gap-2 z-[200]">
          <button
            onClick={() => setShowLauncher(true)}
            className="p-3 rounded-full glass-panel border border-cyan-500/30 text-cyan-400 hover:bg-cyan-500/20 transition-all shadow-lg shadow-cyan-500/20"
            title="Open App Launcher"
          >
            <Plus size={20} />
          </button>
          
          {windows.length > 1 && (
            <>
              <button
                onClick={apply3DArcLayout}
                className="p-3 rounded-full glass-panel border border-white/20 text-white/70 hover:bg-white/10 transition-all"
                title="3D Arc Layout"
              >
                <Box size={18} />
              </button>
              <button
                onClick={applyGridLayout}
                className="p-3 rounded-full glass-panel border border-white/20 text-white/70 hover:bg-white/10 transition-all"
                title="Grid Layout"
              >
                <Grid3X3 size={18} />
              </button>
            </>
          )}
          
          <button
            onClick={onClose}
            className="p-3 rounded-full glass-panel border border-red-500/30 text-red-400 hover:bg-red-500/20 transition-all"
            title="Exit AR Workspace"
          >
            <X size={18} />
          </button>
        </div>
      )}

      {/* Window Count Indicator */}
      {windows.length > 0 && (
        <div className="fixed top-20 right-6 glass-panel px-3 py-1.5 rounded-full flex items-center gap-2 z-[200]">
          <Monitor size={14} className="text-cyan-400" />
          <span className="text-xs font-mono text-white/80">{windows.length} windows</span>
        </div>
      )}
    </div>
  );
});

ARWorkspace.displayName = 'ARWorkspace';

export default ARWorkspace;
