import React from 'react';
import { Command } from 'lucide-react';

interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  description: string;
}

interface KeyboardHelpProps {
  isOpen: boolean;
  onClose: () => void;
  shortcuts: KeyboardShortcut[];
}

export const KeyboardHelp: React.FC<KeyboardHelpProps> = ({ isOpen, onClose, shortcuts }) => {
  if (!isOpen) return null;

  return (
    <div className="absolute inset-0 z-[70] flex items-center justify-center bg-black/80 backdrop-blur-sm" onClick={onClose}>
      <div className="glass-panel p-6 rounded-xl max-w-md w-full border-cyan-500/30" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-3 mb-4 border-b border-cyan-500/20 pb-3">
          <Command className="text-cyan-400" size={24} />
          <h3 className="text-xl font-bold text-white">Keyboard Shortcuts</h3>
        </div>
        
        <div className="space-y-2 max-h-96 overflow-y-auto scrollbar-hide">
          {shortcuts.map((shortcut, index) => (
            <div key={index} className="flex justify-between items-center py-2 px-3 bg-slate-800/30 rounded hover:bg-slate-800/50 transition-colors">
              <span className="text-sm text-slate-300">{shortcut.description}</span>
              <div className="flex gap-1">
                {shortcut.ctrl && (
                  <kbd className="px-2 py-1 bg-slate-700 text-slate-200 rounded text-xs font-mono border border-slate-600">
                    Ctrl
                  </kbd>
                )}
                <kbd className="px-2 py-1 bg-slate-700 text-slate-200 rounded text-xs font-mono border border-slate-600 uppercase">
                  {shortcut.key}
                </kbd>
              </div>
            </div>
          ))}
        </div>

        <button
          onClick={onClose}
          className="mt-4 w-full py-2 bg-cyan-600 hover:bg-cyan-500 text-white font-bold rounded-lg transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  );
};
