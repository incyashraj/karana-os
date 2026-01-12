import React, { useState, useEffect, useRef } from 'react';
import { RefreshCw } from 'lucide-react';
import CameraFeed from './components/CameraFeed';
import { ChatInterface } from './components/ChatInterface';
import { VoiceController } from './components/VoiceController';
import { SpatialStatusBar } from './components/os/SpatialStatusBar';
import { SystemMenu } from './components/os/SystemMenu';
import { AppLauncher } from './components/os/AppLauncher';
import { ControlCenter } from './components/os/ControlCenter';
import { NotificationStream } from './components/os/NotificationStream';
import { SpatialContainer } from './components/os/SpatialContainer';
import { WalletApp } from './components/apps/WalletApp';
import { VisionApp } from './components/apps/VisionApp';
import { BrowserApp } from './components/apps/BrowserApp';
import { MusicApp } from './components/apps/MusicApp';
import { CameraApp } from './components/apps/CameraApp';
import { MapsApp } from './components/apps/MapsApp';
import { GalleryApp } from './components/apps/GalleryApp';
import { WeatherApp } from './components/apps/WeatherApp';
import { AROverlay } from './components/AROverlay';
import * as geminiService from './services/geminiService';
import { universalOracle } from './services/oracleService';
import { getWsService } from './services/wsService';
import { 
  AppMode, 
  AnalysisResult, 
  ChatMessage, 
  WalletState
} from './types';
import { SpatialState } from './types/spatial';

const App: React.FC = () => {
  // --- SPATIAL STATE ---
  const [activeApp, setActiveApp] = useState<string | null>(null);
  const [isLauncherOpen, setIsLauncherOpen] = useState(false);
  const [isControlCenterOpen, setIsControlCenterOpen] = useState(false);
  const [isARSimMode, setIsARSimMode] = useState(false);
  
  // Window Spatial State (x, y, scale)
  const [spatialState, setSpatialState] = useState<SpatialState>({
    x: 0,
    y: 0,
    scale: 1,
    isDragging: false
  });
  
  // World View Offset (Panning)
  const [viewOffset, setViewOffset] = useState({ x: 0, y: 0 });

  // --- APP DATA STATE ---
  const [wallet, setWallet] = useState<WalletState>({
    balance: 0,
    did: 'Not Connected',
    transactions: []
  });
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [backendConnected, setBackendConnected] = useState(false);
  const [showVoiceController, setShowVoiceController] = useState(true);
  
  // Settings State
  const [uxLevel, setUxLevel] = useState<'minimal' | 'standard' | 'advanced'>('standard');
  const [securityPreset, setSecurityPreset] = useState<'standard' | 'high' | 'paranoid'>('standard');

  const cameraRef = useRef<any>(null);

  // --- LOGIC HANDLERS ---
  const handleCreateWallet = async () => {
    setIsProcessing(true);
    setTimeout(() => {
      const newWallet = {
        balance: 100.0000,
        did: 'did:kara:z8s9...3k2j',
        transactions: []
      };
      setWallet(newWallet);
      setIsProcessing(false);
    }, 1500);
  };

  const handleOracleInput = async (text: string) => {
    const userMsg: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      text,
      timestamp: Date.now()
    };
    setChatMessages(prev => [...prev, userMsg]);
    setIsProcessing(true);

    try {
      // Use Universal Oracle for intelligent response
      const manifest = await universalOracle.mediate(text);
      
      const aiMsg: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        text: manifest.text,
        timestamp: Date.now(),
        manifest: manifest // Attach full manifest for UI display
      };
      
      setChatMessages(prev => [...prev, aiMsg]);
      
      // Execute OS operations if applicable
      if (text.toLowerCase().includes('battery') || text.toLowerCase().includes('optimize')) {
        // Simulated battery optimization
        console.log('🔋 Battery optimization triggered');
      } else if (text.toLowerCase().includes('open') || text.toLowerCase().includes('launch')) {
        // Auto-launch apps mentioned
        if (text.toLowerCase().includes('wallet')) setActiveApp('wallet');
        else if (text.toLowerCase().includes('map')) setActiveApp('maps');
        else if (text.toLowerCase().includes('music')) setActiveApp('music');
        else if (text.toLowerCase().includes('code') || text.toLowerCase().includes('vscode')) setActiveApp('browser');
      }
      
    } catch (error) {
      console.error('Oracle error:', error);
      const errorMsg: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        text: '⚠️ Oracle processing failed. Please try again.',
        timestamp: Date.now()
      };
      setChatMessages(prev => [...prev, errorMsg]);
    } finally {
      setIsProcessing(false);
    }
  };

  // --- VOICE AI HANDLERS ---
  const handleVoiceTranscript = (text: string) => {
    console.log('[Voice]', text);
    // Send to Oracle for processing
    handleOracleInput(text);
  };

  const handleVoiceToolResult = (result: any) => {
    console.log('[Tool]', result);
    // Handle tool execution results
    if (result.tool_name === 'launch_app') {
      const appName = result.result.toLowerCase();
      if (appName.includes('camera')) setActiveApp('camera');
      else if (appName.includes('wallet')) setActiveApp('wallet');
      else if (appName.includes('browser')) setActiveApp('browser');
      else if (appName.includes('music')) setActiveApp('music');
      else if (appName.includes('maps')) setActiveApp('maps');
      else if (appName.includes('weather')) setActiveApp('weather');
    } else if (result.tool_name === 'navigate') {
      if (result.result.includes('home')) {
        setActiveApp(null);
      }
    }
  };

  // --- EFFECTS ---
  useEffect(() => {
    // Connect to WebSocket on mount
    const ws = getWsService('ws://localhost:8080');
    ws.connect().catch(error => {
      console.error('[WS] Connection failed:', error);
    });

    return () => {
      // Keep connection alive for other components
      // ws.disconnect();
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'a' && e.ctrlKey) {
        setIsARSimMode(prev => !prev);
      }
      if (e.key === 'v' && e.ctrlKey && e.shiftKey) {
        setShowVoiceController(prev => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => setBackendConnected(true), 1000);
    
    // System Boot Message
    setChatMessages([
      {
        id: 'system_boot',
        role: 'system',
        text: 'KĀRAṆA OS v0.9.5 INITIALIZED. LOCAL NEURAL ENGINE: ONLINE (KARANA-NATIVE-V1).',
        timestamp: Date.now()
      }
    ]);

    return () => clearTimeout(timer);
  }, []);

  // --- RENDER ---
  return (
    <div className={`relative w-screen h-screen bg-black text-white overflow-hidden font-sans selection:bg-cyan-500/30 ${isARSimMode ? 'cursor-none' : ''}`}>
      {/* 1. Reality Layer (Camera) */}
      <CameraFeed active={true} ref={cameraRef} />
      
      {/* 2. Ambient Layer (Subtle Overlays) */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_rgba(0,0,0,0)_0%,_rgba(0,0,0,0.2)_100%)] pointer-events-none" />
      
      {/* AR Simulation Layer */}
      {isARSimMode && (
        <AROverlay 
          cameraFeedRef={cameraRef}
          windows={[
            activeApp === 'vision' ? {
              anchorId: 'vision-window',
              title: 'VISION INTELLIGENCE',
              content: <VisionApp isProcessing={isProcessing} analysis={analysis} />,
              width: 500,
              height: 600
            } : null,
            activeApp === 'oracle' ? {
              anchorId: 'oracle-window',
              title: 'ORACLE BRIDGE',
              content: <ChatInterface messages={chatMessages} onSendMessage={handleOracleInput} isProcessing={isProcessing} />,
              width: 500,
              height: 600
            } : null,
            activeApp === 'wallet' ? {
              anchorId: 'wallet-window',
              title: 'SOVEREIGN ASSETS',
              content: <WalletApp wallet={wallet} onCreateWallet={handleCreateWallet} isProcessing={isProcessing} backendConnected={backendConnected} />,
              width: 500,
              height: 600
            } : null,
            activeApp === 'browser' ? {
              anchorId: 'browser-window',
              title: 'NEURAL NET',
              content: <BrowserApp />,
              width: 800,
              height: 600
            } : null,
            activeApp === 'music' ? {
              anchorId: 'music-window',
              title: 'SONIC FIELD',
              content: <MusicApp />,
              width: 400,
              height: 600
            } : null,
            activeApp === 'camera' ? {
              anchorId: 'camera-window',
              title: 'OPTICAL SENSOR',
              content: <CameraApp />,
              width: 500,
              height: 700
            } : null,
            activeApp === 'maps' ? {
              anchorId: 'maps-window',
              title: 'SPATIAL NAVIGATION',
              content: <MapsApp />,
              width: 800,
              height: 600
            } : null,
            activeApp === 'gallery' ? {
              anchorId: 'gallery-window',
              title: 'VISUAL MEMORY',
              content: <GalleryApp />,
              width: 800,
              height: 600
            } : null,
            activeApp === 'weather' ? {
              anchorId: 'weather-window',
              title: 'ATMOSPHERIC DATA',
              content: <WeatherApp />,
              width: 400,
              height: 600
            } : null,
            activeApp === 'system' ? {
              anchorId: 'system-window',
              title: 'SYSTEM CONFIGURATION',
              content: (
                <div className="p-8 space-y-8 overflow-auto h-full">
                  <div className="glass-panel p-8 rounded-3xl border border-white/10 bg-white/5">
                    <label className="block text-xs font-bold tracking-widest text-white/40 mb-6 uppercase">Interface Density</label>
                    <div className="flex gap-4">
                      {['minimal', 'standard', 'advanced'].map(level => (
                        <button
                          key={level}
                          onClick={() => setUxLevel(level as any)}
                          className={`
                            flex-1 py-4 rounded-2xl text-xs font-bold uppercase tracking-wider transition-all
                            ${uxLevel === level 
                              ? 'bg-cyan-500 text-black shadow-[0_0_30px_rgba(6,182,212,0.4)]' 
                              : 'bg-black/20 text-white/40 hover:bg-white/10 hover:text-white'
                            }
                          `}
                        >
                          {level}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="glass-panel p-8 rounded-3xl border border-white/10 bg-white/5">
                    <label className="block text-xs font-bold tracking-widest text-white/40 mb-6 uppercase">Security Protocol</label>
                    <div className="flex gap-4">
                      {['standard', 'high', 'paranoid'].map(level => (
                        <button
                          key={level}
                          onClick={() => setSecurityPreset(level as any)}
                          className={`
                            flex-1 py-4 rounded-2xl text-xs font-bold uppercase tracking-wider transition-all
                            ${securityPreset === level 
                              ? 'bg-emerald-500 text-black shadow-[0_0_30px_rgba(16,185,129,0.4)]' 
                              : 'bg-black/20 text-white/40 hover:bg-white/10 hover:text-white'
                            }
                          `}
                        >
                          {level}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              ),
              width: 600,
              height: 500
            } : null,
          ].filter(Boolean) as any}
          onWindowClose={(anchorId) => setActiveApp(null)}
        />
      )}

      {/* 3. Heads-Up Layer (Status) */}
      <SpatialStatusBar onOpenControlCenter={() => setIsControlCenterOpen(true)} />
      
      {/* Control Center */}
      <ControlCenter isOpen={isControlCenterOpen} onClose={() => setIsControlCenterOpen(false)} />
      
      {/* Notification Stream */}
      <NotificationStream />

      {/* 4. Focus Layer (Active App) - Only show in non-AR mode */}
      {!isARSimMode && <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        
        {/* Vision App */}
        <SpatialContainer 
          isActive={activeApp === 'vision'} 
          onClose={() => setActiveApp(null)}
          title="VISION INTELLIGENCE"
          position={activeApp === 'vision' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <VisionApp isProcessing={isProcessing} analysis={analysis} />
        </SpatialContainer>

        {/* Oracle App */}
        <SpatialContainer 
          isActive={activeApp === 'oracle'} 
          onClose={() => setActiveApp(null)}
          title="ORACLE BRIDGE"
          position={activeApp === 'oracle' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <ChatInterface 
            messages={chatMessages}
            onSendMessage={handleOracleInput}
            isProcessing={isProcessing}
          />
        </SpatialContainer>

        {/* Wallet App */}
        <SpatialContainer 
          isActive={activeApp === 'wallet'} 
          onClose={() => setActiveApp(null)}
          title="SOVEREIGN ASSETS"
          position={activeApp === 'wallet' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <WalletApp 
            wallet={wallet} 
            onCreateWallet={handleCreateWallet}
            isProcessing={isProcessing}
            backendConnected={backendConnected}
          />
        </SpatialContainer>

        {/* Browser App */}
        <SpatialContainer 
          isActive={activeApp === 'browser'} 
          onClose={() => setActiveApp(null)}
          title="NEURAL NET"
          position={activeApp === 'browser' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <BrowserApp />
        </SpatialContainer>

        {/* Music App */}
        <SpatialContainer 
          isActive={activeApp === 'music'} 
          onClose={() => setActiveApp(null)}
          title="SONIC FIELD"
          position={activeApp === 'music' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <MusicApp />
        </SpatialContainer>

        {/* Camera App */}
        <SpatialContainer 
          isActive={activeApp === 'camera'} 
          onClose={() => setActiveApp(null)}
          title="OPTICAL SENSOR"
          position={activeApp === 'camera' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <CameraApp />
        </SpatialContainer>

        {/* Maps App */}
        <SpatialContainer 
          isActive={activeApp === 'maps'} 
          onClose={() => setActiveApp(null)}
          title="SPATIAL NAVIGATION"
          position={activeApp === 'maps' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <MapsApp />
        </SpatialContainer>

        {/* Gallery App */}
        <SpatialContainer 
          isActive={activeApp === 'gallery'} 
          onClose={() => setActiveApp(null)}
          title="VISUAL MEMORY"
          position={activeApp === 'gallery' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <GalleryApp />
        </SpatialContainer>

        {/* Weather App */}
        <SpatialContainer 
          isActive={activeApp === 'weather'} 
          onClose={() => setActiveApp(null)}
          title="ATMOSPHERIC DATA"
          position={activeApp === 'weather' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <WeatherApp />
        </SpatialContainer>

        {/* Settings App */}
        <SpatialContainer 
          isActive={activeApp === 'system'} 
          onClose={() => setActiveApp(null)}
          title="SYSTEM CONFIGURATION"
          position={activeApp === 'system' ? { ...spatialState, x: spatialState.x + viewOffset.x, y: spatialState.y + viewOffset.y } : undefined}
        >
          <div className="p-8 space-y-8">
            <div className="glass-panel p-8 rounded-3xl border border-white/10 bg-white/5">
              <label className="block text-xs font-bold tracking-widest text-white/40 mb-6 uppercase">Interface Density</label>
              <div className="flex gap-4">
                {['minimal', 'standard', 'advanced'].map(level => (
                  <button
                    key={level}
                    onClick={() => setUxLevel(level as any)}
                    className={`
                      flex-1 py-4 rounded-2xl text-xs font-bold uppercase tracking-wider transition-all
                      ${uxLevel === level 
                        ? 'bg-cyan-500 text-black shadow-[0_0_30px_rgba(6,182,212,0.4)]' 
                        : 'bg-black/20 text-white/40 hover:bg-white/10 hover:text-white'
                      }
                    `}
                  >
                    {level}
                  </button>
                ))}
              </div>
            </div>

            <div className="glass-panel p-8 rounded-3xl border border-white/10 bg-white/5">
              <label className="block text-xs font-bold tracking-widest text-white/40 mb-6 uppercase">Security Protocol</label>
              <div className="flex gap-4">
                {['standard', 'high', 'paranoid'].map(level => (
                  <button
                    key={level}
                    onClick={() => setSecurityPreset(level as any)}
                    className={`
                      flex-1 py-4 rounded-2xl text-xs font-bold uppercase tracking-wider transition-all
                      ${securityPreset === level 
                        ? 'bg-emerald-500 text-black shadow-[0_0_30px_rgba(16,185,129,0.4)]' 
                        : 'bg-black/20 text-white/40 hover:bg-white/10 hover:text-white'
                      }
                    `}
                  >
                    {level}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </SpatialContainer>

      </div>}

      {/* 5. App Launcher Overlay */}
      <AppLauncher 
        isOpen={isLauncherOpen} 
        onLaunch={(appId) => setActiveApp(appId)} 
        onClose={() => setIsLauncherOpen(false)} 
      />

      {/* 6. System Menu Trigger */}
      <SystemMenu 
        isOpen={isLauncherOpen} 
        onToggle={() => setIsLauncherOpen(!isLauncherOpen)} 
      />

      {/* 7. Notifications */}
      {!backendConnected && (
        <div className="absolute top-24 right-6 z-[200] glass-panel px-4 py-2 rounded-full flex items-center gap-3 text-red-300 border border-red-500/30 bg-red-900/20 backdrop-blur-md">
          <RefreshCw size={14} className="animate-spin" />
          <span className="text-[10px] font-bold tracking-wider">CONNECTING TO KERNEL...</span>
        </div>
      )}

      {/* Mobile AR Toggle */}
      <button 
        onClick={() => setIsARSimMode(!isARSimMode)}
        className="fixed bottom-6 right-6 z-[200] w-12 h-12 rounded-full bg-cyan-500/20 border border-cyan-500/50 flex items-center justify-center text-cyan-400 backdrop-blur-md active:scale-95 transition-all"
      >
        <span className="font-bold text-[10px]">AR</span>
      </button>

      {/* Voice Controller - Floating */}
      {showVoiceController && (
        <div className="fixed bottom-24 right-6 z-[201] max-w-md">
          <VoiceController
            onTranscript={handleVoiceTranscript}
            onToolResult={handleVoiceToolResult}
            className="shadow-2xl"
          />
          <div className="mt-2 text-center">
            <span className="text-[10px] text-white/30">
              Press Ctrl+Shift+V to toggle
            </span>
          </div>
        </div>
      )}

    </div>
  );
};

export default App;
