import React from 'react';
import { CloudRain, Wind, Droplets, Sun } from 'lucide-react';

export const WeatherApp: React.FC = () => {
  return (
    <div className="flex flex-col h-full bg-gradient-to-b from-sky-900 to-slate-900 text-white rounded-b-3xl overflow-hidden relative">
      {/* Background Animation (Simple CSS) */}
      <div className="absolute top-10 right-10 w-32 h-32 bg-yellow-400 rounded-full blur-[60px] opacity-20 animate-pulse" />
      
      <div className="flex-1 flex flex-col items-center justify-center p-8 text-center z-10">
        <div className="mb-4 p-4 bg-white/5 rounded-full backdrop-blur-sm border border-white/10 shadow-[0_0_30px_rgba(6,182,212,0.2)]">
            <CloudRain size={64} className="text-cyan-300" />
        </div>
        <h1 className="text-6xl font-bold tracking-tighter mb-2">72°</h1>
        <p className="text-xl font-medium text-cyan-200">Heavy Rain</p>
        <p className="text-sm text-white/40 mt-1">Neo-Tokyo, Sector 7</p>
      </div>

      <div className="bg-black/20 backdrop-blur-md p-6 grid grid-cols-3 gap-4 border-t border-white/5">
        <div className="flex flex-col items-center gap-2">
            <Wind size={20} className="text-white/50" />
            <span className="text-sm font-bold">12 km/h</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <Droplets size={20} className="text-white/50" />
            <span className="text-sm font-bold">84%</span>
        </div>
        <div className="flex flex-col items-center gap-2">
            <Sun size={20} className="text-white/50" />
            <span className="text-sm font-bold">6:42 PM</span>
        </div>
      </div>
    </div>
  );
};
