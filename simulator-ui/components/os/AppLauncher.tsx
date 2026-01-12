import React from 'react';
import { 
  Scan, 
  MessageSquare, 
  Wallet, 
  Settings, 
  Globe, 
  Terminal,
  Music,
  Camera,
  Map,
  Calendar,
  Image,
  CloudRain
} from 'lucide-react';

interface AppLauncherProps {
  isOpen: boolean;
  onLaunch: (appId: string) => void;
  onClose: () => void;
}

export const AppLauncher: React.FC<AppLauncherProps> = ({ isOpen, onLaunch, onClose }) => {
  if (!isOpen) return null;

  const apps = [
    { id: 'vision', label: 'Vision', icon: <Scan size={32} />, color: 'text-amber-400', bg: 'bg-amber-400/10' },
    { id: 'oracle', label: 'Oracle', icon: <MessageSquare size={32} />, color: 'text-purple-400', bg: 'bg-purple-400/10' },
    { id: 'wallet', label: 'Wallet', icon: <Wallet size={32} />, color: 'text-blue-400', bg: 'bg-blue-400/10' },
    { id: 'browser', label: 'Browser', icon: <Globe size={32} />, color: 'text-emerald-400', bg: 'bg-emerald-400/10' },
    { id: 'camera', label: 'Camera', icon: <Camera size={32} />, color: 'text-pink-400', bg: 'bg-pink-400/10' },
    { id: 'music', label: 'Music', icon: <Music size={32} />, color: 'text-cyan-400', bg: 'bg-cyan-400/10' },
    { id: 'maps', label: 'Maps', icon: <Map size={32} />, color: 'text-orange-400', bg: 'bg-orange-400/10' },
    { id: 'gallery', label: 'Gallery', icon: <Image size={32} />, color: 'text-rose-400', bg: 'bg-rose-400/10' },
    { id: 'weather', label: 'Weather', icon: <CloudRain size={32} />, color: 'text-sky-400', bg: 'bg-sky-400/10' },
    { id: 'calendar', label: 'Calendar', icon: <Calendar size={32} />, color: 'text-red-400', bg: 'bg-red-400/10' },
    { id: 'system', label: 'Settings', icon: <Settings size={32} />, color: 'text-slate-400', bg: 'bg-slate-400/10' },
  ];

  return (
    <div 
      className="absolute inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-xl animate-in fade-in duration-200"
      onClick={onClose}
    >
      <div 
        className="w-full max-w-4xl p-12 grid grid-cols-4 gap-8"
        onClick={e => e.stopPropagation()}
      >
        {apps.map((app, index) => (
          <button
            key={app.id}
            onClick={() => {
              onLaunch(app.id);
              onClose();
            }}
            className="
              group flex flex-col items-center gap-4 p-6 rounded-3xl 
              bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/20
              transition-all duration-300 hover:scale-105 hover:shadow-[0_0_30px_rgba(255,255,255,0.1)]
            "
            style={{ animationDelay: `${index * 50}ms` }}
          >
            <div className={`
              w-20 h-20 rounded-2xl flex items-center justify-center 
              ${app.bg} ${app.color} shadow-inner
              group-hover:shadow-[0_0_20px_currentColor] transition-shadow duration-500
            `}>
              {app.icon}
            </div>
            <span className="text-sm font-medium text-white/80 tracking-wide group-hover:text-white">
              {app.label}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};
