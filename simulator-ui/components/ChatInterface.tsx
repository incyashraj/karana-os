import React, { useState, useRef, useEffect } from 'react';
import { Send, Mic, User, Cpu, Sparkles } from 'lucide-react';
import { ChatMessage } from '../types';

interface ChatInterfaceProps {
  messages: ChatMessage[];
  onSendMessage: (text: string) => void;
  isProcessing: boolean;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ messages, onSendMessage, isProcessing }) => {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      onSendMessage(input);
      setInput("");
    }
  };

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="w-full max-w-lg h-[50vh] flex flex-col pointer-events-auto">
      <div className="glass-panel rounded-t-xl p-3 border-b-0 flex items-center gap-2 text-purple-300 bg-purple-900/20">
         <Sparkles size={16} />
         <span className="text-xs font-bold tracking-widest uppercase">Oracle Bridge Active</span>
      </div>
      
      <div className="flex-1 overflow-y-auto p-4 space-y-4 mb-0 scrollbar-hide glass-panel border-y-0 bg-black/40">
        {messages.length === 0 && (
          <div className="text-center text-purple-200/40 mt-10 font-mono text-sm">
            <p className="mb-2">LISTENING FOR INTENTS...</p>
            <p>"Send 50 tokens to Alice"</p>
            <p>"What am I looking at?"</p>
            <p>"Navigate home"</p>
          </div>
        )}
        {messages.map((msg) => (
          <div key={msg.id} className={`flex gap-3 ${msg.role === 'user' ? 'flex-row-reverse' : 'flex-row'}`}>
            <div className={`w-8 h-8 rounded-full flex items-center justify-center shrink-0 border ${
              msg.role === 'user' 
                ? 'bg-cyan-900/50 border-cyan-500/50 text-cyan-400' 
                : msg.role === 'system'
                  ? 'bg-emerald-900/50 border-emerald-500/50 text-emerald-400'
                  : 'bg-purple-900/50 border-purple-500/50 text-purple-400'
            }`}>
              {msg.role === 'user' ? <User size={14} /> : msg.role === 'system' ? <Cpu size={14} /> : <Sparkles size={14} />}
            </div>
            
            <div className={`p-3 rounded-lg text-sm max-w-[85%] backdrop-blur-md border shadow-lg ${
              msg.role === 'user' 
                ? 'bg-cyan-950/60 border-cyan-500/30 text-cyan-100 rounded-tr-none' 
                : msg.role === 'system'
                  ? 'bg-emerald-950/60 border-emerald-500/30 text-emerald-100'
                  : 'bg-purple-950/60 border-purple-500/30 text-purple-100 rounded-tl-none'
            }`}>
              {msg.text}
              {msg.metadata && msg.metadata.type === 'TRANSFER' && (
                 <div className="mt-2 pt-2 border-t border-white/10 text-xs font-mono opacity-80">
                    <div className="flex justify-between"><span>ACTION:</span> <span>TRANSFER</span></div>
                    <div className="flex justify-between"><span>TO:</span> <span>{msg.metadata.recipient}</span></div>
                    <div className="flex justify-between"><span>AMT:</span> <span>{msg.metadata.amount} KARA</span></div>
                 </div>
              )}
            </div>
          </div>
        ))}
        {isProcessing && (
           <div className="flex gap-3">
             <div className="w-8 h-8 rounded-full bg-purple-900/50 border border-purple-500/30 text-purple-400 flex items-center justify-center shrink-0 animate-pulse">
               <Sparkles size={14} />
             </div>
             <div className="p-3 rounded-lg text-xs font-mono text-purple-300 bg-purple-900/20 border border-purple-500/20 flex items-center gap-2">
               <span>PROCESSING INTENT</span>
               <span className="flex gap-1">
                 <span className="w-1 h-1 bg-purple-400 rounded-full animate-bounce"></span>
                 <span className="w-1 h-1 bg-purple-400 rounded-full animate-bounce delay-100"></span>
                 <span className="w-1 h-1 bg-purple-400 rounded-full animate-bounce delay-200"></span>
               </span>
             </div>
           </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} className="relative glass-panel rounded-b-xl border-t-0 p-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Speak or Type Command..."
          className="w-full bg-black/40 border border-purple-500/20 rounded-lg py-3 px-10 text-purple-100 focus:outline-none focus:border-purple-400/50 focus:bg-purple-900/10 placeholder-purple-400/30 transition-all font-mono text-sm"
        />
        <button type="button" className="absolute left-4 top-1/2 -translate-y-1/2 text-purple-400 hover:text-purple-200 transition-colors">
          <Mic size={16} />
        </button>
        <button 
          type="submit" 
          disabled={!input.trim() || isProcessing}
          className="absolute right-4 top-1/2 -translate-y-1/2 text-purple-400 hover:text-purple-200 disabled:opacity-30 transition-colors"
        >
          <Send size={16} />
        </button>
      </form>
    </div>
  );
};
