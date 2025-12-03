import React, { useState, useEffect } from 'react';
import { Battery, Wifi, Activity, Clock, Navigation, ShieldCheck, Wallet } from 'lucide-react';
import { AppMode, WalletState } from '../types';

interface HUDProps {
  mode: AppMode;
  wallet: WalletState;
  children: React.ReactNode;
}

export const HUD: React.FC<HUDProps> = ({ mode, wallet, children }) => {
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
    <div className="relative w-full h-full flex flex-col justify-between p-4 sm:p-6 pointer-events-none select-none font-rajdhani">
      {/* Top Bar */}
      <div className="flex justify-between items-start">
        {/* Left: OS Info & Wallet */}
        <div className="flex flex-col gap-2 items-start">
          <div className={`glass-panel px-4 py-2 rounded-br-xl flex items-center gap-3 ${themeColor} border-t-0 border-l-0`}>
             <ShieldCheck size={16} />
             <div className="flex flex-col leading-none">
                <span className="text-[10px] opacity-70 tracking-widest">KĀRAṆA OS</span>
                <span className="text-lg font-bold tracking-wider">{mode}</span>
             </div>
          </div>
          
          {/* Simulated Identity Badge */}
          <div className="glass-panel px-3 py-1 rounded-r-lg border-l-0 flex items-center gap-2 opacity-80">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
            <span className="hud-font text-xs text-gray-300">{wallet.did.substring(0, 16)}...</span>
          </div>
        </div>

        {/* Right: Balance & Status */}
        <div className="flex flex-col gap-2 items-end">
          <div className={`glass-panel px-6 py-2 rounded-bl-xl flex items-center gap-6 ${themeColor} border-t-0 border-r-0`}>
            <div className="flex items-center gap-2">
              <Clock size={16} />
              <span className="hud-font text-lg">{time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
            </div>
            <div className="flex items-center gap-2 border-l border-white/10 pl-4">
              <Wifi size={16} />
              <Battery size={16} />
            </div>
          </div>

          {/* Wallet Balance Widget */}
          <div className="glass-panel px-4 py-1 rounded-l-lg border-r-0 flex items-center gap-3 text-cyan-300">
             <Wallet size={14} />
             <span className="hud-font text-base font-bold">{wallet.balance.toLocaleString()} KARA</span>
          </div>
        </div>
      </div>

      {/* Center Content Area (Children) */}
      <div className="flex-1 relative flex items-center justify-center pointer-events-auto z-10">
        {children}
      </div>

      {/* Decorative Corners & HUD Lines */}
      <svg className="absolute top-0 left-0 w-full h-full -z-10 opacity-60 pointer-events-none">
         <path d="M 30 150 V 30 H 150" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M calc(100% - 30px) 150 V 30 H calc(100% - 150px)" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M 30 calc(100% - 150px) V calc(100% - 30px) H 150" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         <path d="M calc(100% - 30px) calc(100% - 150px) V calc(100% - 30px) H calc(100% - 150px)" fill="none" stroke={themeHex} strokeWidth="2" strokeDasharray="10 5" opacity="0.5" />
         
         {/* Center Reticle */}
         {(mode === AppMode.IDLE || mode === AppMode.ANALYZING) && (
            <g opacity="0.4">
              <circle cx="50%" cy="50%" r="150" stroke={themeHex} strokeWidth="1" fill="none" strokeDasharray="4 4" />
              <circle cx="50%" cy="50%" r="5" fill={themeHex} />
              <path d="M calc(50% - 20px) 50% H calc(50% - 100px)" stroke={themeHex} strokeWidth="1" />
              <path d="M calc(50% + 20px) 50% H calc(50% + 100px)" stroke={themeHex} strokeWidth="1" />
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
