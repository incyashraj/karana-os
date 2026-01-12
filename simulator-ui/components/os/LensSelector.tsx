import React from 'react';
import { 
  Scan, 
  MessageSquare, 
  Wallet, 
  Settings, 
  LayoutGrid, 
  Globe,
  Terminal
} from 'lucide-react';

interface LensSelectorProps {
  activeLens: string | null;
  onSelectLens: (lensId: string) => void;
}

export const LensSelector: React.FC<LensSelectorProps> = ({ activeLens, onSelectLens }) => {
  const lenses = [
    { id: 'vision', icon: <Scan size={24} />, label: 'VISION', color: 'text-amber-400', border: 'border-amber-400/30' },
    { id: 'oracle', icon: <MessageSquare size={24} />, label: 'ORACLE', color: 'text-purple-400', border: 'border-purple-400/30' },
    { id: 'wallet', icon: <Wallet size={24} />, label: 'ASSETS', color: 'text-blue-400', border: 'border-blue-400/30' },
    { id: 'net', icon: <Globe size={24} />, label: 'NETWORK', color: 'text-emerald-400', border: 'border-emerald-400/30' },
    { id: 'system', icon: <Settings size={24} />, label: 'SYSTEM', color: 'text-slate-400', border: 'border-slate-400/30' },
  ];

  return (
    <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-[100] flex flex-col items-center gap-4">
      
      {/* Active Lens Label */}
      <div className={`
        text-xs font-bold tracking-[0.3em] text-white/60 uppercase transition-all duration-300
        ${activeLens ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'}
      `}>
        {lenses.find(l => l.id === activeLens)?.label || 'SELECT LENS'}
      </div>

      {/* Lens Dock */}
      <div className="flex items-end gap-4 px-6 py-4 rounded-3xl bg-black/20 backdrop-blur-2xl border border-white/5 shadow-[0_0_40px_rgba(0,0,0,0.5)]">
        {lenses.map((lens) => {
          const isActive = activeLens === lens.id;
          
          return (
            <button
              key={lens.id}
              onClick={() => onSelectLens(isActive ? '' : lens.id)} // Toggle off if active
              className={`
                group relative flex items-center justify-center
                transition-all duration-500 ease-out
                ${isActive 
                  ? 'w-16 h-16 -translate-y-4 bg-white/10 shadow-[0_0_30px_rgba(255,255,255,0.1)]' 
                  : 'w-12 h-12 hover:w-14 hover:h-14 hover:bg-white/5 hover:-translate-y-2'
                }
                rounded-2xl border ${isActive ? lens.border : 'border-white/5'}
              `}
            >
              <div className={`
                transition-all duration-500
                ${isActive ? `scale-110 ${lens.color}` : 'text-white/40 group-hover:text-white/80'}
              `}>
                {lens.icon}
              </div>

              {/* Active Indicator Glow */}
              {isActive && (
                <div className={`absolute inset-0 rounded-2xl opacity-20 blur-md ${lens.color.replace('text', 'bg')}`} />
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
};
