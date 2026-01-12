import React, { useState, useEffect } from 'react';
import { Battery, Wifi, Activity, Clock, Navigation, ShieldCheck, Wallet, BarChart3 } from 'lucide-react';
import { AppMode, WalletState } from '../types';

interface HUDProps {
  mode: AppMode;
  wallet: WalletState;
  ephemeralMode?: boolean;
  children: React.ReactNode;
  onShowDashboard?: () => void;
}

export const HUD: React.FC<HUDProps> = ({ mode, wallet, ephemeralMode, children, onShowDashboard }) => {
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const getModeColor = () => {
    switch (mode) {
      case AppMode.ANALYZING: return 'text-amber-400 border-amber-400';
      case AppMode.ORACLE: return 'text-purple-400 border-purple-400';
      case AppMode.NAVIGATION: return 'text-emerald-400 border-emerald-400';
      case AppMode.WALLET: return 'text-blue-400 border-blue-400';
      default: return 'text-cyan-400 border-cyan-400';
    }
  };

  const themeColor = getModeColor();
  const themeHex = mode === AppMode.ANALYZING ? '#fbbf24' : mode === AppMode.ORACLE ? '#c084fc' : mode === AppMode.NAVIGATION ? '#34d399' : mode === AppMode.WALLET ? '#60a5fa' : '#22d3ee';

  return (
    <div className="relative w-full h-full flex flex-col justify-between p-6 pointer-events-none select-none font-sans">
      {/* Top Bar */}
      <div className="flex justify-between items-start">
        {/* Left: OS Info */}
        <div className="flex gap-3">
          <div className="glass-panel px-4 py-2 rounded-full flex items-center gap-3">
             <ShieldCheck size={16} className={mode === AppMode.ANALYZING ? 'text-amber-400' : 'text-cyan-400'} />
             <span className="text-xs font-bold tracking-wider opacity-80">KĀRAṆA OS</span>
             <div className="w-px h-3 bg-white/20"></div>
             <span className="text-xs font-medium text-white/60">{mode}</span>
          </div>
          
          {/* Identity Pill */}
          <div className="glass-panel px-3 py-2 rounded-full flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${ephemeralMode ? 'bg-purple-500' : 'bg-green-500'} animate-pulse`}></div>
            <span className="font-mono text-[10px] text-white/60">
              {ephemeralMode ? 'EPHEMERAL' : wallet.did === 'Not Connected' ? 'GUEST' : wallet.did.substring(0, 8) + '...'}
            </span>
          </div>
        </div>

        {/* Right: Status */}
        <div className="flex gap-3 items-start">
          {/* Wallet Balance (if connected) */}
          {wallet.did !== 'Not Connected' && (
            <div className="glass-panel px-4 py-2 rounded-full flex items-center gap-2">
              <Wallet size={14} className="text-blue-400" />
              <span className="font-mono text-xs">{wallet.balance.toFixed(4)} KAR</span>
            </div>
          )}

          {/* Time & System */}
          <div className="glass-panel px-4 py-2 rounded-full flex items-center gap-4">
            <span className="font-mono text-sm font-medium">{time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
            <div className="w-px h-3 bg-white/20"></div>
            <div className="flex gap-2 text-white/60">
              <Wifi size={14} />
              <Battery size={14} />
            </div>
          </div>
          
          {/* Intelligence Dashboard Button */}
          {onShowDashboard && (
            <button
              onClick={onShowDashboard}
              className="glass-panel px-3 py-2 rounded-full flex items-center gap-2 text-purple-400 hover:bg-purple-500/20 transition-all pointer-events-auto cursor-pointer"
              title="Intelligence Dashboard"
            >
              <BarChart3 size={14} />
            </button>
          )}
        </div>
      </div>

      {/* Center Content Area (Children) */}
      <div className="flex-1 relative flex items-center justify-center pointer-events-auto z-10">
        {children}
      </div>

      {/* Decorative Corners & HUD Lines */}
      <svg className="absolute top-0 left-0 w-full h-full -z-10 opacity-60 pointer-events-none" viewBox="0 0 1920 1080" preserveAspectRatio="none">
         <path d="M 30 150 V 30 H 150" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M 1890 150 V 30 H 1770" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M 30 930 V 1050 H 150" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M 1890 930 V 1050 H 1770" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         
         {/* Center Reticle */}
         {(mode === AppMode.IDLE || mode === AppMode.ANALYZING) && (
            <g opacity="0.4">
              <circle cx="960" cy="540" r="150" stroke={themeHex} strokeWidth="1" fill="none" strokeDasharray="4 4" />
              <circle cx="960" cy="540" r="5" fill={themeHex} />
              <path d="M 940 540 H 860" stroke={themeHex} strokeWidth="1" />
              <path d="M 980 540 H 1060" stroke={themeHex} strokeWidth="1" />
            </g>
         )}
      </svg>

      {/* Bottom Bar */}
      <div className="flex justify-between items-end">
        <div className={`glass-panel px-4 py-2 rounded-tr-xl flex items-center gap-2 ${themeColor} border-b-0 border-l-0`}>
          <Activity size={18} className="animate-pulse" />
          <span className="text-xs tracking-wider font-bold">SYSTEM OPTIMAL // ED25519 READY</span>
        </div>
        
        {/* Navigation Hint */}
         {mode === AppMode.NAVIGATION && (
           <div className="absolute bottom-40 left-1/2 transform -translate-x-1/2 flex flex-col items-center animate-bounce z-0">
             <Navigation size={64} className="text-emerald-400 mb-2 drop-shadow-[0_0_10px_rgba(52,211,153,0.8)]" fill="currentColor" fillOpacity={0.2} />
             <div className="glass-panel px-6 py-2 text-emerald-400 text-lg font-bold border-emerald-500/50">
                DESTINATION: 150m AHEAD
             </div>
           </div>
         )}
      </div>
    </div>
  );
};
