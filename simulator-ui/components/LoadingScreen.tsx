import React from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingScreenProps {
  message?: string;
}

export const LoadingScreen: React.FC<LoadingScreenProps> = ({ 
  message = "Initializing Kāraṇa OS..." 
}) => {
  return (
    <div className="absolute inset-0 z-[100] bg-black flex items-center justify-center">
      <div className="flex flex-col items-center gap-6">
        {/* Animated Logo/Icon */}
        <div className="relative w-32 h-32">
          <div className="absolute inset-0 border-4 border-cyan-400/20 rounded-full"></div>
          <div className="absolute inset-0 border-4 border-t-cyan-400 border-r-transparent border-b-transparent border-l-transparent rounded-full animate-spin"></div>
          <div className="absolute inset-4 border-4 border-purple-400/20 rounded-full"></div>
          <div className="absolute inset-4 border-4 border-t-transparent border-r-purple-400 border-b-transparent border-l-transparent rounded-full animate-spin" style={{ animationDuration: '2s', animationDirection: 'reverse' }}></div>
          <Loader2 className="absolute inset-0 m-auto text-cyan-400 animate-pulse" size={48} />
        </div>

        {/* Message */}
        <div className="text-center">
          <div className="text-xl font-bold text-white hud-font tracking-widest animate-pulse">
            {message}
          </div>
          <div className="mt-2 flex items-center justify-center gap-1">
            <span className="w-2 h-2 bg-cyan-400 rounded-full animate-bounce"></span>
            <span className="w-2 h-2 bg-cyan-400 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }}></span>
            <span className="w-2 h-2 bg-cyan-400 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }}></span>
          </div>
        </div>

        {/* System Info */}
        <div className="glass-panel px-6 py-3 rounded-lg border-cyan-500/20">
          <div className="text-xs text-slate-400 font-mono space-y-1">
            <div>✓ Neural Engine: <span className="text-emerald-400">Online</span></div>
            <div>✓ Vision Layer: <span className="text-emerald-400">Ready</span></div>
            <div>✓ Blockchain Node: <span className="text-amber-400">Syncing...</span></div>
          </div>
        </div>
      </div>
    </div>
  );
};
