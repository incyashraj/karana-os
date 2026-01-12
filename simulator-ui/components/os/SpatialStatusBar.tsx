import React from 'react';
import { Wifi, Battery, Activity } from 'lucide-react';

interface SpatialStatusBarProps {
  onOpenControlCenter?: () => void;
}

export const SpatialStatusBar: React.FC<SpatialStatusBarProps> = ({ onOpenControlCenter }) => {
  const [time, setTime] = React.useState(new Date());

  React.useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="fixed top-6 left-6 right-6 z-[100] flex justify-between items-start pointer-events-none mix-blend-screen">
      {/* Left: System Status */}
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2 text-cyan-400/80">
          <Activity size={14} className="animate-pulse" />
          <span className="text-[10px] font-bold tracking-[0.2em] font-mono">KĀRAṆA OS // V.0.9</span>
        </div>
        <div className="h-0.5 w-24 bg-gradient-to-r from-cyan-500/50 to-transparent" />
      </div>

      {/* Right: Environment Status */}
      <div className="flex items-center gap-4 pointer-events-auto">
        <button 
          onClick={onOpenControlCenter}
          className="flex items-center gap-3 px-4 py-2 rounded-full bg-black/20 backdrop-blur-md border border-white/10 shadow-lg hover:bg-white/10 transition-colors active:scale-95"
        >
           <Wifi size={14} className="text-white/70" />
           <div className="w-px h-3 bg-white/10" />
           <Battery size={14} className="text-white/70" />
           <div className="w-px h-3 bg-white/10" />
           <span className="font-mono text-xs font-medium text-white/90">
             {time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
           </span>
        </button>
      </div>
    </div>
  );
};
