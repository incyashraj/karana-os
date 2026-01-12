import React, { useState } from 'react';
import { 
  Wifi, 
  Bluetooth, 
  Moon, 
  Sun, 
  Volume2, 
  Mic, 
  MicOff, 
  Eye, 
  EyeOff,
  Cast,
  Battery
} from 'lucide-react';

interface ControlCenterProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ControlCenter: React.FC<ControlCenterProps> = ({ isOpen, onClose }) => {
  const [brightness, setBrightness] = useState(80);
  const [volume, setVolume] = useState(60);
  const [toggles, setToggles] = useState({
    wifi: true,
    bluetooth: true,
    dnd: false,
    mic: true,
    passthrough: true,
    airplay: false
  });

  if (!isOpen) return null;

  const toggle = (key: keyof typeof toggles) => {
    setToggles(prev => ({ ...prev, [key]: !prev[key] }));
  };

  return (
    <>
      <div className="fixed inset-0 z-[140]" onClick={onClose} />
      <div 
        className="absolute top-20 right-6 z-[150] w-80 animate-in slide-in-from-top-4 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="bg-black/60 backdrop-blur-2xl border border-white/10 rounded-3xl p-6 shadow-[0_0_50px_rgba(0,0,0,0.5)] text-white">
        
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-xs font-bold tracking-widest text-white/60">CONTROL CENTER</span>
          </div>
          <div className="flex items-center gap-2 text-xs font-mono text-white/40">
            <Battery size={12} />
            <span>84%</span>
          </div>
        </div>

        {/* Toggles Grid */}
        <div className="grid grid-cols-2 gap-3 mb-6">
          {/* Connectivity Group */}
          <div className="col-span-2 bg-white/5 rounded-2xl p-4 grid grid-cols-4 gap-2">
            <ToggleButton 
              active={toggles.wifi} 
              icon={<Wifi size={20} />} 
              label="Wi-Fi" 
              onClick={() => toggle('wifi')} 
            />
            <ToggleButton 
              active={toggles.bluetooth} 
              icon={<Bluetooth size={20} />} 
              label="Bluetooth" 
              onClick={() => toggle('bluetooth')} 
            />
            <ToggleButton 
              active={toggles.airplay} 
              icon={<Cast size={20} />} 
              label="Cast" 
              onClick={() => toggle('airplay')} 
            />
            <ToggleButton 
              active={toggles.dnd} 
              icon={<Moon size={20} />} 
              label="DND" 
              onClick={() => toggle('dnd')} 
              activeColor="bg-indigo-500"
            />
          </div>

          {/* Hardware Toggles */}
          <BigToggle 
            active={toggles.mic} 
            icon={toggles.mic ? <Mic size={24} /> : <MicOff size={24} />} 
            label="Microphone" 
            status={toggles.mic ? "On" : "Muted"}
            onClick={() => toggle('mic')}
            color="bg-amber-500"
          />
          <BigToggle 
            active={toggles.passthrough} 
            icon={toggles.passthrough ? <Eye size={24} /> : <EyeOff size={24} />} 
            label="Passthrough" 
            status={toggles.passthrough ? "Active" : "Paused"}
            onClick={() => toggle('passthrough')}
            color="bg-cyan-500"
          />
        </div>

        {/* Sliders */}
        <div className="space-y-4 bg-white/5 rounded-2xl p-4">
          <div className="flex items-center gap-3">
            <Sun size={16} className="text-white/50" />
            <input 
              type="range" 
              value={brightness} 
              onChange={(e) => setBrightness(parseInt(e.target.value))}
              className="flex-1 h-12 bg-black/20 rounded-xl appearance-none cursor-pointer overflow-hidden [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-0 [&::-webkit-slider-thumb]:h-0 [&::-webkit-slider-thumb]:shadow-[-100vw_0_0_100vw_white]"
            />
          </div>
          <div className="flex items-center gap-3">
            <Volume2 size={16} className="text-white/50" />
            <input 
              type="range" 
              value={volume} 
              onChange={(e) => setVolume(parseInt(e.target.value))}
              className="flex-1 h-12 bg-black/20 rounded-xl appearance-none cursor-pointer overflow-hidden [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-0 [&::-webkit-slider-thumb]:h-0 [&::-webkit-slider-thumb]:shadow-[-100vw_0_0_100vw_white]"
            />
          </div>
        </div>

      </div>
    </div>
    </>
  );
};

const ToggleButton: React.FC<{ active: boolean; icon: React.ReactNode; label: string; onClick: () => void; activeColor?: string }> = ({ active, icon, label, onClick, activeColor = 'bg-blue-500' }) => (
  <button 
    onClick={onClick}
    className={`flex flex-col items-center justify-center gap-1 aspect-square rounded-xl transition-all ${active ? `${activeColor} text-white` : 'bg-white/10 text-white/50 hover:bg-white/20'}`}
  >
    {icon}
  </button>
);

const BigToggle: React.FC<{ active: boolean; icon: React.ReactNode; label: string; status: string; onClick: () => void; color: string }> = ({ active, icon, label, status, onClick, color }) => (
  <button 
    onClick={onClick}
    className={`flex flex-col justify-between p-4 rounded-2xl transition-all h-32 text-left ${active ? `${color} text-black` : 'bg-white/5 text-white hover:bg-white/10'}`}
  >
    <div className={`w-8 h-8 rounded-full flex items-center justify-center ${active ? 'bg-black/10' : 'bg-white/10'}`}>
      {icon}
    </div>
    <div>
      <div className={`text-[10px] font-bold uppercase tracking-wider ${active ? 'text-black/60' : 'text-white/40'}`}>{status}</div>
      <div className="font-medium text-sm">{label}</div>
    </div>
  </button>
);
