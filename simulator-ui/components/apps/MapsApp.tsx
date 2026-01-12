import React from 'react';
import { Navigation, MapPin, Search, Locate } from 'lucide-react';

export const MapsApp: React.FC = () => {
  return (
    <div className="flex flex-col h-full bg-zinc-900 text-white rounded-b-3xl overflow-hidden relative">
      {/* Map View (Mock) */}
      <div className="absolute inset-0 bg-zinc-800 overflow-hidden">
         {/* Grid Pattern to simulate map */}
         <div className="absolute inset-0 opacity-20" 
              style={{ 
                  backgroundImage: 'linear-gradient(#444 1px, transparent 1px), linear-gradient(90deg, #444 1px, transparent 1px)', 
                  backgroundSize: '40px 40px' 
              }} 
         />
         
         {/* Roads */}
         <div className="absolute top-1/2 left-0 right-0 h-4 bg-zinc-600 transform -rotate-12" />
         <div className="absolute top-0 bottom-0 left-1/3 w-4 bg-zinc-600 transform rotate-12" />
         
         {/* Location Marker */}
         <div className="absolute top-1/2 left-1/3 transform -translate-x-1/2 -translate-y-1/2">
            <div className="w-4 h-4 bg-cyan-500 rounded-full shadow-[0_0_20px_rgba(6,182,212,0.8)] animate-pulse" />
            <div className="absolute inset-0 w-4 h-4 bg-cyan-500 rounded-full animate-ping opacity-50" />
         </div>
      </div>
      
      {/* UI Overlay */}
      <div className="relative z-10 flex flex-col h-full pointer-events-none">
        {/* Search Bar */}
        <div className="p-4 pointer-events-auto">
            <div className="bg-black/60 backdrop-blur-md border border-white/10 rounded-full flex items-center px-4 py-3 gap-3 shadow-lg">
                <Search size={18} className="text-white/50" />
                <input type="text" placeholder="Search destination..." className="bg-transparent border-none outline-none text-sm text-white w-full placeholder-white/30" />
            </div>
        </div>

        <div className="flex-1" />

        {/* Navigation Card */}
        <div className="p-4 pointer-events-auto">
            <div className="bg-black/80 backdrop-blur-xl border border-white/10 rounded-2xl p-4 shadow-2xl">
                <div className="flex items-start gap-4">
                    <div className="w-12 h-12 rounded-xl bg-cyan-500 flex items-center justify-center shrink-0">
                        <Navigation size={24} className="text-black fill-current" />
                    </div>
                    <div>
                        <h3 className="font-bold text-lg">12 min to Home</h3>
                        <p className="text-white/60 text-sm">Fastest route via Neural Highway</p>
                    </div>
                </div>
                <div className="mt-4 flex gap-2">
                    <button className="flex-1 bg-white text-black font-bold py-3 rounded-xl hover:bg-gray-200 transition-colors">Start</button>
                    <button className="px-4 bg-white/10 rounded-xl hover:bg-white/20 transition-colors text-white">
                        <Locate size={20} />
                    </button>
                </div>
            </div>
        </div>
      </div>
    </div>
  );
};
