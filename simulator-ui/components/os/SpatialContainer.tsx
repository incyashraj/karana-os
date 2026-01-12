import React from 'react';
import { X, Minus, Move } from 'lucide-react';

interface SpatialContainerProps {
  children: React.ReactNode;
  isActive: boolean;
  onClose: () => void;
  title?: string;
  position?: { x: number; y: number; scale: number };
}

export const SpatialContainer: React.FC<SpatialContainerProps> = ({ children, isActive, onClose, title, position }) => {
  if (!isActive) return null;

  const style = position ? {
    transform: `translate(${position.x}px, ${position.y}px) scale(${position.scale})`,
    transition: 'transform 0.1s cubic-bezier(0.2, 0, 0, 1)' // Smooth follow
  } : {};

  return (
    <div className="absolute inset-0 flex items-center justify-center z-30 pointer-events-none p-12">
      <div 
        style={style}
        className="
        relative w-full max-w-5xl h-[80vh]
        bg-[#0f1115]/80 backdrop-blur-2xl
        rounded-[40px] border border-white/10 
        shadow-[0_20px_50px_rgba(0,0,0,0.5)]
        animate-in fade-in zoom-in-95 duration-300
        pointer-events-auto
        flex flex-col overflow-hidden
      ">
        {/* Subtle Noise Texture for realism */}
        <div className="absolute inset-0 opacity-[0.03] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] pointer-events-none" />
        
        {/* Header / Window Controls */}
        <div className="spatial-handle flex items-center justify-between px-8 py-6 border-b border-white/5 bg-white/5 cursor-move group">
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full bg-red-500/80 hover:bg-red-400 cursor-pointer transition-colors" onClick={onClose} />
            <div className="w-3 h-3 rounded-full bg-yellow-500/80 hover:bg-yellow-400 cursor-pointer transition-colors" />
            <div className="w-3 h-3 rounded-full bg-green-500/80 hover:bg-green-400 cursor-pointer transition-colors" />
          </div>
          
          <h2 className="text-sm font-medium tracking-widest text-white/40 uppercase absolute left-1/2 -translate-x-1/2 flex items-center gap-2">
            {title}
            <Move size={12} className="opacity-0 group-hover:opacity-50 transition-opacity" />
          </h2>
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-hidden relative">
          {children}
        </div>
      </div>
    </div>
  );
};
