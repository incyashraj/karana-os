import React from 'react';
import { Grip } from 'lucide-react';

interface SystemMenuProps {
  onToggle: () => void;
  isOpen: boolean;
}

export const SystemMenu: React.FC<SystemMenuProps> = ({ onToggle, isOpen }) => {
  return (
    <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-[100]">
      <button
        onClick={onToggle}
        className={`
          group flex items-center justify-center w-16 h-16 rounded-full
          backdrop-blur-xl border border-white/10 shadow-2xl transition-all duration-300
          ${isOpen 
            ? 'bg-white text-black rotate-90 scale-90' 
            : 'bg-black/40 text-white hover:bg-white/10 hover:scale-110'
          }
        `}
      >
        <Grip size={24} />
      </button>
    </div>
  );
};
