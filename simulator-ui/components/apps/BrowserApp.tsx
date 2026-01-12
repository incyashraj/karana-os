import React, { useState } from 'react';
import { Search, ArrowLeft, ArrowRight, RotateCw, Globe, Lock } from 'lucide-react';

export const BrowserApp: React.FC = () => {
  const [url, setUrl] = useState('https://en.wikipedia.org/wiki/Augmented_reality');
  const [inputUrl, setInputUrl] = useState('https://en.wikipedia.org/wiki/Augmented_reality');
  const [isLoading, setIsLoading] = useState(false);
  const [history, setHistory] = useState<string[]>(['https://en.wikipedia.org/wiki/Augmented_reality']);
  const [historyIndex, setHistoryIndex] = useState(0);

  const handleNavigate = (e?: React.FormEvent) => {
    e?.preventDefault();
    let target = inputUrl;
    if (!target.startsWith('http')) {
      target = `https://www.google.com/search?q=${encodeURIComponent(target)}`;
    }
    setUrl(target);
    setIsLoading(true);
    // Mock loading
    setTimeout(() => setIsLoading(false), 1000);
    
    const newHistory = history.slice(0, historyIndex + 1);
    newHistory.push(target);
    setHistory(newHistory);
    setHistoryIndex(newHistory.length - 1);
  };

  const goBack = () => {
    if (historyIndex > 0) {
      setHistoryIndex(prev => prev - 1);
      setUrl(history[historyIndex - 1]);
      setInputUrl(history[historyIndex - 1]);
    }
  };

  const goForward = () => {
    if (historyIndex < history.length - 1) {
      setHistoryIndex(prev => prev + 1);
      setUrl(history[historyIndex + 1]);
      setInputUrl(history[historyIndex + 1]);
    }
  };

  return (
    <div className="flex flex-col h-full bg-zinc-900 text-white overflow-hidden rounded-b-3xl">
      {/* Browser Toolbar */}
      <div className="flex items-center gap-2 p-3 bg-zinc-800 border-b border-white/10">
        <div className="flex gap-1">
          <button 
            onClick={goBack}
            disabled={historyIndex === 0}
            className="p-2 rounded-full hover:bg-white/10 disabled:opacity-30 transition-colors"
          >
            <ArrowLeft size={16} />
          </button>
          <button 
            onClick={goForward}
            disabled={historyIndex === history.length - 1}
            className="p-2 rounded-full hover:bg-white/10 disabled:opacity-30 transition-colors"
          >
            <ArrowRight size={16} />
          </button>
          <button 
            onClick={() => { setIsLoading(true); setTimeout(() => setIsLoading(false), 800); }}
            className="p-2 rounded-full hover:bg-white/10 transition-colors"
          >
            <RotateCw size={16} className={isLoading ? 'animate-spin' : ''} />
          </button>
        </div>

        <form onSubmit={handleNavigate} className="flex-1 flex items-center gap-2 px-4 py-2 bg-black/40 rounded-full border border-white/5 focus-within:border-cyan-500/50 transition-colors">
          <Lock size={12} className="text-emerald-400" />
          <input 
            type="text" 
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            className="flex-1 bg-transparent border-none outline-none text-sm font-mono text-white/90 placeholder-white/30"
            placeholder="Search or enter URL"
          />
        </form>
      </div>

      {/* Browser Content */}
      <div className="flex-1 relative bg-white">
        {isLoading && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-zinc-900/50 backdrop-blur-sm">
            <div className="flex flex-col items-center gap-4">
              <Globe size={48} className="text-cyan-400 animate-pulse" />
              <span className="text-xs font-bold tracking-widest text-cyan-400">RESOLVING...</span>
            </div>
          </div>
        )}
        <iframe 
          src={url} 
          className="w-full h-full border-none"
          title="Browser"
          sandbox="allow-scripts allow-same-origin allow-forms"
        />
        
        {/* Overlay for restricted sites */}
        <div className="absolute bottom-0 left-0 right-0 p-2 bg-amber-500/10 border-t border-amber-500/20 text-amber-200 text-[10px] text-center">
          Note: Some websites may refuse to connect in this simulated environment due to X-Frame-Options.
        </div>
      </div>
    </div>
  );
};
