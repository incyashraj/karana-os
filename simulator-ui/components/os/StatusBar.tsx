import React, { useState, useEffect } from 'react';
import { Wifi, Battery, Volume2, Search } from 'lucide-react';

export const StatusBar: React.FC = () => {
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="fixed top-0 left-0 right-0 h-8 z-[100] flex items-center justify-between px-4 bg-black/20 backdrop-blur-md border-b border-white/5 text-xs font-medium text-white/80 select-none">
      <div className="flex items-center gap-4">
        <span className="font-bold tracking-wider">KĀRAṆA OS</span>
        <div className="flex items-center gap-2 px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 transition-colors cursor-pointer">
          <span className="opacity-60">File</span>
        </div>
        <div className="flex items-center gap-2 px-2 py-0.5 rounded hover:bg-white/10 transition-colors cursor-pointer">
          <span className="opacity-60">Edit</span>
        </div>
        <div className="flex items-center gap-2 px-2 py-0.5 rounded hover:bg-white/10 transition-colors cursor-pointer">
          <span className="opacity-60">View</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-3 px-3 py-1 rounded-full bg-black/20 border border-white/5">
           <Wifi size={12} />
           <Volume2 size={12} />
           <Battery size={12} />
        </div>
        <span className="font-mono">
          {time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
        </span>
      </div>
    </div>
  );
};
