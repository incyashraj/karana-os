import React, { useState, useRef, useEffect } from 'react';
import { X, Minus, Maximize2, Minimize2, Move, RotateCw, Volume2, VolumeX, Play, Pause, SkipBack, SkipForward, Globe, FileText, Terminal, Image, Music, Video, Calendar, Mail, Settings, Grip } from 'lucide-react';
import AppStore from './AppStore';

export interface ARWindowProps {
  id: string;
  title: string;
  type: 'video' | 'browser' | 'terminal' | 'notes' | 'image' | 'music' | 'calendar' | 'mail' | 'appstore' | 'custom';
  initialPosition: { x: number; y: number };
  initialSize: { width: number; height: number };
  initialDepth?: number; // Z-depth for 3D effect
  content?: React.ReactNode;
  url?: string;
  onClose: (id: string) => void;
  onFocus: (id: string) => void;
  isFocused: boolean;
  zIndex: number;
}

const ARWindow: React.FC<ARWindowProps> = ({
  id,
  title,
  type,
  initialPosition,
  initialSize,
  initialDepth = 0,
  content,
  url,
  onClose,
  onFocus,
  isFocused,
  zIndex
}) => {
  const [position, setPosition] = useState(initialPosition);
  const [size, setSize] = useState(initialSize);
  const [depth, setDepth] = useState(initialDepth);
  const [isMinimized, setIsMinimized] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  
  // Video player state
  const [isPlaying, setIsPlaying] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [progress, setProgress] = useState(0);
  const videoRef = useRef<HTMLVideoElement>(null);

  // Browser state
  const [browserUrl, setBrowserUrl] = useState(url || 'https://example.com');

  const windowRef = useRef<HTMLDivElement>(null);

  // Handle dragging
  const handleMouseDown = (e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.window-controls')) return;
    onFocus(id);
    setIsDragging(true);
    setDragOffset({
      x: e.clientX - position.x,
      y: e.clientY - position.y
    });
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (isDragging) {
      setPosition({
        x: e.clientX - dragOffset.x,
        y: e.clientY - dragOffset.y
      });
    }
    if (isResizing) {
      setSize({
        width: Math.max(300, e.clientX - position.x),
        height: Math.max(200, e.clientY - position.y)
      });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    setIsResizing(false);
  };

  useEffect(() => {
    if (isDragging || isResizing) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, isResizing, dragOffset, position]);

  // Video controls
  const togglePlay = () => {
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.pause();
      } else {
        videoRef.current.play();
      }
      setIsPlaying(!isPlaying);
    }
  };

  const handleTimeUpdate = () => {
    if (videoRef.current) {
      setProgress((videoRef.current.currentTime / videoRef.current.duration) * 100);
    }
  };

  // 3D transform based on depth
  const transform3D = `
    perspective(1000px) 
    translateZ(${depth * 50}px) 
    rotateY(${depth * 2}deg)
    ${isMaximized ? 'scale(1)' : `scale(${1 - depth * 0.05})`}
  `;

  const getTypeIcon = () => {
    switch (type) {
      case 'video': return <Video size={14} />;
      case 'browser': return <Globe size={14} />;
      case 'terminal': return <Terminal size={14} />;
      case 'notes': return <FileText size={14} />;
      case 'image': return <Image size={14} />;
      case 'music': return <Music size={14} />;
      case 'calendar': return <Calendar size={14} />;
      case 'mail': return <Mail size={14} />;
      default: return <Settings size={14} />;
    }
  };

  const getTypeColor = () => {
    switch (type) {
      case 'video': return 'from-red-500/20 to-pink-500/20 border-red-500/40';
      case 'browser': return 'from-blue-500/20 to-cyan-500/20 border-blue-500/40';
      case 'terminal': return 'from-green-500/20 to-emerald-500/20 border-green-500/40';
      case 'notes': return 'from-yellow-500/20 to-amber-500/20 border-yellow-500/40';
      case 'image': return 'from-purple-500/20 to-violet-500/20 border-purple-500/40';
      case 'music': return 'from-pink-500/20 to-rose-500/20 border-pink-500/40';
      case 'calendar': return 'from-orange-500/20 to-amber-500/20 border-orange-500/40';
      case 'mail': return 'from-indigo-500/20 to-blue-500/20 border-indigo-500/40';
      default: return 'from-gray-500/20 to-slate-500/20 border-gray-500/40';
    }
  };

  if (isMinimized) {
    return (
      <button
        onClick={() => { setIsMinimized(false); onFocus(id); }}
        className="fixed bottom-20 glass-panel px-4 py-2 rounded-lg flex items-center gap-2 hover:scale-105 transition-transform"
        style={{ left: `${100 + zIndex * 60}px`, zIndex }}
      >
        {getTypeIcon()}
        <span className="text-xs font-medium">{title}</span>
      </button>
    );
  }

  return (
    <div
      ref={windowRef}
      className={`fixed transition-all duration-200 ${isDragging ? 'cursor-grabbing' : ''}`}
      style={{
        left: isMaximized ? 0 : position.x,
        top: isMaximized ? 0 : position.y,
        width: isMaximized ? '100%' : size.width,
        height: isMaximized ? '100%' : size.height,
        zIndex: zIndex + 100,
        transform: isMaximized ? 'none' : transform3D,
        transformStyle: 'preserve-3d'
      }}
      onClick={() => onFocus(id)}
    >
      {/* Window Frame with Glass Effect */}
      <div className={`
        w-full h-full rounded-2xl overflow-hidden
        bg-gradient-to-br ${getTypeColor()}
        backdrop-blur-xl border
        shadow-2xl shadow-black/50
        ${isFocused ? 'ring-2 ring-white/20' : ''}
        transition-all duration-300
      `}>
        
        {/* Title Bar */}
        <div 
          className="h-10 bg-black/40 backdrop-blur-md flex items-center justify-between px-3 cursor-grab"
          onMouseDown={handleMouseDown}
        >
          <div className="flex items-center gap-2">
            <div className="flex gap-1.5 window-controls">
              <button 
                onClick={() => onClose(id)}
                className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition-colors flex items-center justify-center group"
              >
                <X size={8} className="opacity-0 group-hover:opacity-100" />
              </button>
              <button 
                onClick={() => setIsMinimized(true)}
                className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition-colors flex items-center justify-center group"
              >
                <Minus size={8} className="opacity-0 group-hover:opacity-100" />
              </button>
              <button 
                onClick={() => setIsMaximized(!isMaximized)}
                className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition-colors flex items-center justify-center group"
              >
                {isMaximized ? <Minimize2 size={8} className="opacity-0 group-hover:opacity-100" /> : <Maximize2 size={8} className="opacity-0 group-hover:opacity-100" />}
              </button>
            </div>
            <div className="flex items-center gap-2 ml-2">
              {getTypeIcon()}
              <span className="text-xs font-medium text-white/80 truncate max-w-[150px]">{title}</span>
            </div>
          </div>
          
          <div className="flex items-center gap-2 text-white/50">
            {/* Depth Control */}
            <button 
              onClick={() => setDepth(d => Math.max(-2, d - 1))}
              className="p-1 hover:bg-white/10 rounded text-xs window-controls"
              title="Move closer"
            >
              ↗
            </button>
            <span className="text-[10px] font-mono">Z:{depth}</span>
            <button 
              onClick={() => setDepth(d => Math.min(2, d + 1))}
              className="p-1 hover:bg-white/10 rounded text-xs window-controls"
              title="Move farther"
            >
              ↙
            </button>
          </div>
        </div>

        {/* Content Area */}
        <div className="h-[calc(100%-40px)] overflow-hidden">
          {type === 'video' && (
            <div className="relative w-full h-full bg-black flex flex-col">
              <div className="flex-1 flex items-center justify-center overflow-hidden">
                {url ? (
                  <video
                    ref={videoRef}
                    src={url}
                    className="max-w-full max-h-full object-contain"
                    onTimeUpdate={handleTimeUpdate}
                    muted={isMuted}
                  />
                ) : (
                  <div className="text-center">
                    <div className="w-32 h-32 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-red-500/30 to-pink-500/30 flex items-center justify-center">
                      <Video size={48} className="text-red-400" />
                    </div>
                    <p className="text-gray-400 text-sm">Demo Video Player</p>
                    <p className="text-gray-600 text-xs mt-1">Spatial video projection ready</p>
                  </div>
                )}
              </div>
              
              {/* Video Controls */}
              <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4">
                <div className="h-1 bg-white/20 rounded-full mb-3 overflow-hidden">
                  <div className="h-full bg-red-500 rounded-full transition-all" style={{ width: `${progress}%` }} />
                </div>
                <div className="flex items-center justify-center gap-4">
                  <button className="p-2 hover:bg-white/10 rounded-full transition-colors">
                    <SkipBack size={16} />
                  </button>
                  <button 
                    onClick={togglePlay}
                    className="p-3 bg-white/20 hover:bg-white/30 rounded-full transition-colors"
                  >
                    {isPlaying ? <Pause size={20} /> : <Play size={20} />}
                  </button>
                  <button className="p-2 hover:bg-white/10 rounded-full transition-colors">
                    <SkipForward size={16} />
                  </button>
                  <button 
                    onClick={() => setIsMuted(!isMuted)}
                    className="p-2 hover:bg-white/10 rounded-full transition-colors ml-4"
                  >
                    {isMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
                  </button>
                </div>
              </div>
            </div>
          )}

          {type === 'browser' && (
            <div className="w-full h-full flex flex-col bg-black/40">
              {/* URL Bar */}
              <div className="flex items-center gap-2 p-2 bg-white/5 border-b border-white/10">
                <div className="flex gap-1">
                  <button className="p-1 hover:bg-white/10 rounded text-gray-400">←</button>
                  <button className="p-1 hover:bg-white/10 rounded text-gray-400">→</button>
                  <button className="p-1 hover:bg-white/10 rounded text-gray-400">
                    <RotateCw size={12} />
                  </button>
                </div>
                <div className="flex-1 flex items-center bg-black/40 rounded-lg px-3 py-1.5 border border-white/10">
                  <Globe size={12} className="text-gray-500 mr-2" />
                  <input 
                    type="text" 
                    value={browserUrl}
                    onChange={(e) => setBrowserUrl(e.target.value)}
                    className="flex-1 bg-transparent text-xs text-white/80 outline-none"
                    placeholder="Enter URL..."
                  />
                </div>
              </div>
              
              {/* Browser Content */}
              <div className="flex-1 flex items-center justify-center text-center p-8">
                <div>
                  <div className="w-24 h-24 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-blue-500/30 to-cyan-500/30 flex items-center justify-center">
                    <Globe size={40} className="text-blue-400" />
                  </div>
                  <h3 className="text-lg font-bold text-white mb-2">Spatial Browser</h3>
                  <p className="text-gray-400 text-sm max-w-xs">
                    Browse the decentralized web in AR space. 
                    Navigate with eye tracking and gestures.
                  </p>
                  <div className="mt-4 flex gap-2 justify-center flex-wrap">
                    {['DuckDuckGo', 'IPFS', 'ENS Domains'].map(tag => (
                      <span key={tag} className="px-2 py-1 bg-blue-900/40 border border-blue-500/30 rounded text-xs text-blue-300">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {type === 'terminal' && (
            <div className="w-full h-full bg-black/80 font-mono text-sm p-4 overflow-auto backdrop-blur-md">
              <div className="text-green-400">
                <p className="text-green-600">Last login: {new Date().toLocaleString()}</p>
                <p className="mt-2">
                  <span className="text-cyan-400">karana@glasses</span>
                  <span className="text-white">:</span>
                  <span className="text-blue-400">~</span>
                  <span className="text-white">$ </span>
                  <span className="animate-pulse">_</span>
                </p>
                <div className="mt-4 text-gray-500 text-xs">
                  <p># Kāraṇa OS Terminal</p>
                  <p># Run commands directly in your field of view</p>
                  <p># Type 'help' for available commands</p>
                </div>
              </div>
            </div>
          )}

          {type === 'notes' && (
            <div className="w-full h-full bg-gradient-to-br from-yellow-900/20 to-amber-900/20 p-4">
              <textarea 
                className="w-full h-full bg-transparent text-white/90 resize-none outline-none text-sm leading-relaxed"
                placeholder="Start typing your notes...

• Voice-to-text available
• Auto-saves to Celestia DA
• End-to-end encrypted"
                defaultValue={`# Meeting Notes - ${new Date().toLocaleDateString()}

## Attendees
- You (via AR glasses)

## Topics
1. Kāraṇa OS development progress
2. Spatial computing features
3. Next milestones

## Action Items
- [ ] Test AR window system
- [ ] Add gesture controls
- [ ] Implement eye tracking`}
              />
            </div>
          )}

          {type === 'image' && (
            <div className="w-full h-full bg-black flex items-center justify-center">
              <div className="text-center">
                <div className="w-32 h-32 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-purple-500/30 to-violet-500/30 flex items-center justify-center">
                  <Image size={48} className="text-purple-400" />
                </div>
                <p className="text-gray-400 text-sm">Spatial Image Viewer</p>
                <p className="text-gray-600 text-xs mt-1">Pin images in 3D space</p>
              </div>
            </div>
          )}

          {type === 'music' && (
            <div className="w-full h-full bg-gradient-to-br from-pink-900/30 to-rose-900/30 flex flex-col items-center justify-center p-6">
              <div className="w-40 h-40 rounded-2xl bg-gradient-to-br from-pink-500/40 to-purple-500/40 flex items-center justify-center mb-6 shadow-lg shadow-pink-500/20">
                <Music size={64} className="text-pink-300" />
              </div>
              <h3 className="text-lg font-bold text-white mb-1">Spatial Audio</h3>
              <p className="text-gray-400 text-sm mb-6">Immersive 3D sound</p>
              <div className="flex items-center gap-4">
                <button className="p-2 hover:bg-white/10 rounded-full"><SkipBack size={20} /></button>
                <button className="p-4 bg-pink-500/30 hover:bg-pink-500/40 rounded-full">
                  <Play size={24} />
                </button>
                <button className="p-2 hover:bg-white/10 rounded-full"><SkipForward size={20} /></button>
              </div>
            </div>
          )}

          {type === 'appstore' && (
            <AppStore
              onLaunchApp={(packageName, appInfo) => {
                // Show native app launch notification
                console.log(`🚀 Native App Launch Request:`);
                console.log(`   App: ${appInfo.appName}`);
                console.log(`   Package: ${packageName}`);
                console.log(`   APK: ${appInfo.apkPath}`);
                
                // In a real AR OS, this would launch the native Android activity
                // For now, we'll show a visual notification
                const notification = document.createElement('div');
                notification.style.cssText = `
                  position: fixed;
                  top: 20px;
                  right: 20px;
                  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                  color: white;
                  padding: 16px 24px;
                  border-radius: 12px;
                  box-shadow: 0 8px 32px rgba(0,0,0,0.3);
                  z-index: 10000;
                  animation: slideIn 0.3s ease-out;
                  font-family: system-ui;
                `;
                notification.innerHTML = `
                  <div style="display: flex; align-items: center; gap: 12px;">
                    <span style="font-size: 24px;">🚀</span>
                    <div>
                      <div style="font-weight: bold; margin-bottom: 4px;">Launching ${appInfo.appName}</div>
                      <div style="font-size: 11px; opacity: 0.9;">Native Android Activity</div>
                    </div>
                  </div>
                `;
                document.body.appendChild(notification);
                setTimeout(() => notification.remove(), 3000);
              }}
              onClose={onClose}
            />
          )}

          {type === 'calendar' && (
            <div className="w-full h-full bg-gradient-to-br from-orange-900/20 to-amber-900/20 p-4 overflow-auto">
              <div className="text-center mb-4">
                <h3 className="text-lg font-bold text-white">{new Date().toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}</h3>
              </div>
              <div className="grid grid-cols-7 gap-1 text-xs">
                {['S', 'M', 'T', 'W', 'T', 'F', 'S'].map((d, i) => (
                  <div key={i} className="text-center text-gray-500 py-1">{d}</div>
                ))}
                {Array.from({ length: 35 }, (_, i) => {
                  const day = i - new Date(new Date().getFullYear(), new Date().getMonth(), 1).getDay() + 1;
                  const isToday = day === new Date().getDate();
                  const isValidDay = day > 0 && day <= new Date(new Date().getFullYear(), new Date().getMonth() + 1, 0).getDate();
                  return (
                    <div 
                      key={i} 
                      className={`text-center py-2 rounded ${isToday ? 'bg-orange-500 text-white font-bold' : isValidDay ? 'text-gray-300 hover:bg-white/10' : 'text-gray-700'}`}
                    >
                      {isValidDay ? day : ''}
                    </div>
                  );
                })}
              </div>
              <div className="mt-4 space-y-2">
                <div className="p-2 bg-orange-500/20 rounded border-l-2 border-orange-500">
                  <p className="text-xs font-bold text-orange-300">Today</p>
                  <p className="text-xs text-gray-400">Test AR glasses features</p>
                </div>
              </div>
            </div>
          )}

          {type === 'mail' && (
            <div className="w-full h-full bg-gradient-to-br from-indigo-900/20 to-blue-900/20 p-4 overflow-auto">
              <div className="space-y-2">
                {[
                  { from: 'system@karana.os', subject: 'Welcome to Kāraṇa OS', preview: 'Your sovereign identity is ready...', unread: true },
                  { from: 'celestia@da.network', subject: 'Transaction Confirmed', preview: 'Your data has been posted to...', unread: true },
                  { from: 'oracle@ai.karana', subject: 'AI Analysis Complete', preview: 'Vision analysis results are ready...', unread: false },
                ].map((mail, i) => (
                  <div key={i} className={`p-3 rounded-lg border ${mail.unread ? 'bg-indigo-500/10 border-indigo-500/30' : 'bg-black/20 border-white/5'}`}>
                    <div className="flex justify-between items-start mb-1">
                      <span className={`text-xs ${mail.unread ? 'font-bold text-white' : 'text-gray-400'}`}>{mail.from}</span>
                      {mail.unread && <span className="w-2 h-2 rounded-full bg-indigo-500" />}
                    </div>
                    <p className={`text-sm ${mail.unread ? 'font-medium text-white' : 'text-gray-300'}`}>{mail.subject}</p>
                    <p className="text-xs text-gray-500 truncate">{mail.preview}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {type === 'custom' && content}
        </div>

        {/* Resize Handle */}
        {!isMaximized && (
          <div 
            className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize"
            onMouseDown={(e) => { e.stopPropagation(); setIsResizing(true); onFocus(id); }}
          >
            <Grip size={12} className="text-white/30" />
          </div>
        )}
      </div>
    </div>
  );
};

export default ARWindow;
