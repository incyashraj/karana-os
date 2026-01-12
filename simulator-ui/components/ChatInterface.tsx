import React, { useState, useRef, useEffect } from 'react';
import { Send, Mic, User, Cpu, Sparkles, ArrowRight, Command, ThumbsUp, ThumbsDown, Info, Zap } from 'lucide-react';
import { ChatMessage } from '../types';
import { universalOracle, OracleManifest } from '../services/oracleService';

interface ChatInterfaceProps {
  messages: ChatMessage[];
  onSendMessage: (text: string) => void;
  isProcessing: boolean;
}

interface ExtendedMessage extends ChatMessage {
  manifest?: OracleManifest;
  feedbackGiven?: boolean;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ messages, onSendMessage, isProcessing }) => {
  const [input, setInput] = useState("");
  const [extendedMessages, setExtendedMessages] = useState<ExtendedMessage[]>([]);
  const [showAnalytics, setShowAnalytics] = useState(false);
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
  }, [extendedMessages]);

  useEffect(() => {
    setExtendedMessages(messages.map(m => ({ ...m, feedbackGiven: false })));
  }, [messages]);

  const handleFeedback = async (messageId: string, helpful: boolean) => {
    const message = extendedMessages.find(m => m.id === messageId);
    if (message && message.role === 'user') {
      await universalOracle.processFeedback(message.text, helpful);
      setExtendedMessages(prev => prev.map(m => 
        m.id === messageId ? { ...m, feedbackGiven: true } : m
      ));
    }
  };

  const suggestions = [
    "Should I bring umbrella for commute?",
    "Tune battery for optimal runtime",
    "Write a haiku about nature",
    "Open VS Code editor"
  ];

  return (
    <div className="w-full h-full flex flex-col pointer-events-auto relative">
      {/* Header */}
      <div className="absolute top-0 left-0 right-0 p-4 bg-gradient-to-b from-black/60 to-transparent z-10 flex items-center justify-between">
         <div className="flex items-center gap-2 text-purple-300">
            <Zap size={16} className="animate-pulse" />
            <span className="text-xs font-bold tracking-widest uppercase">Universal Oracle</span>
         </div>
         <button 
           onClick={() => setShowAnalytics(!showAnalytics)}
           className="text-[10px] font-mono text-white/40 hover:text-white/80 transition-colors flex items-center gap-1"
         >
           <Info size={12} />
           {showAnalytics ? 'HIDE' : 'STATS'}
         </button>
      </div>

      {/* Analytics Panel */}
      {showAnalytics && (
        <div className="absolute top-16 right-4 p-4 bg-black/90 backdrop-blur-xl border border-purple-500/30 rounded-2xl z-20 w-64 shadow-2xl">
          <h4 className="text-xs font-bold text-purple-300 mb-3 uppercase tracking-wide">Oracle Analytics</h4>
          {(() => {
            const analytics = universalOracle.getAnalytics();
            return (
              <>
                <div className="space-y-2 text-xs">
                  <div className="flex justify-between">
                    <span className="text-white/40">Total Sessions:</span>
                    <span className="text-white font-mono">{analytics.totalSessions}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-white/40">Avg Confidence:</span>
                    <span className={`font-mono font-bold ${
                      analytics.avgConfidence > 0.85 ? 'text-green-400' :
                      analytics.avgConfidence > 0.7 ? 'text-yellow-400' : 'text-red-400'
                    }`}>
                      {(analytics.avgConfidence * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
                {analytics.recentIntents.length > 0 && (
                  <div className="mt-4 pt-3 border-t border-white/10">
                    <div className="text-xs text-white/40 mb-2">Recent Queries:</div>
                    <div className="space-y-1 max-h-32 overflow-y-auto">
                      {analytics.recentIntents.slice(0, 5).map((intent, i) => (
                        <div key={i} className="text-[10px] text-white/60 truncate font-mono">
                          • {intent}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            );
          })()}
        </div>
      )}
      
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-6 pt-16 space-y-6 scrollbar-hide">
        {extendedMessages.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center text-center opacity-60">
            <div className="w-20 h-20 rounded-full bg-purple-500/10 border border-purple-500/30 flex items-center justify-center mb-6 animate-pulse">
              <Cpu size={40} className="text-purple-400" />
            </div>
            <h3 className="text-xl font-light text-white mb-2">How can I assist you?</h3>
            <div className="grid grid-cols-2 gap-3 mt-8 w-full max-w-md">
              {suggestions.map((s, i) => (
                <button 
                  key={i}
                  onClick={() => onSendMessage(s)}
                  className="p-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 text-sm text-left text-purple-200 transition-all hover:scale-105"
                >
                  {s}
                </button>
              ))}
            </div>
          </div>
        )}

        {extendedMessages.map((msg) => (
          <div key={msg.id} className={`flex gap-4 ${msg.role === 'user' ? 'flex-row-reverse' : 'flex-row'}`}>
            {/* Avatar */}
            <div className={`w-10 h-10 rounded-full flex items-center justify-center shrink-0 border shadow-lg ${
              msg.role === 'user' 
                ? 'bg-cyan-900/80 border-cyan-500/50 text-cyan-400' 
                : 'bg-purple-900/80 border-purple-500/50 text-purple-400'
            }`}>
              {msg.role === 'user' ? <User size={18} /> : <Zap size={18} />}
            </div>
            
            {/* Bubble */}
            <div className="flex flex-col gap-2 max-w-[80%]">
              <div className={`p-4 rounded-2xl text-sm backdrop-blur-md border shadow-xl leading-relaxed ${
                msg.role === 'user' 
                  ? 'bg-cyan-950/40 border-cyan-500/20 text-cyan-50 rounded-tr-sm' 
                  : 'bg-purple-950/40 border-purple-500/20 text-purple-50 rounded-tl-sm'
              }`}>
                {msg.text}
                
                {/* Oracle Manifest Card */}
                {msg.manifest && msg.role === 'assistant' && (
                  <div className="mt-4 space-y-2">
                    {/* Confidence Bar */}
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-white/40">Confidence:</span>
                      <div className="flex-1 h-1.5 bg-black/40 rounded-full overflow-hidden">
                        <div 
                          className={`h-full transition-all duration-500 ${
                            msg.manifest.confidence > 0.85 ? 'bg-green-500' :
                            msg.manifest.confidence > 0.7 ? 'bg-yellow-500' : 'bg-red-500'
                          }`}
                          style={{ width: `${msg.manifest.confidence * 100}%` }}
                        />
                      </div>
                      <span className={`font-mono font-bold ${
                        msg.manifest.confidence > 0.85 ? 'text-green-400' :
                        msg.manifest.confidence > 0.7 ? 'text-yellow-400' : 'text-red-400'
                      }`}>
                        {(msg.manifest.confidence * 100).toFixed(0)}%
                      </span>
                    </div>

                    {/* Reasoning Trace */}
                    {msg.manifest.reasoning_trace.length > 0 && (
                      <details className="group">
                        <summary className="text-xs text-purple-300 cursor-pointer hover:text-purple-200 transition-colors flex items-center gap-1">
                          <span>🧠 Reasoning Chain ({msg.manifest.reasoning_trace.length} steps)</span>
                        </summary>
                        <div className="mt-2 p-2 bg-black/40 rounded-lg space-y-1">
                          {msg.manifest.reasoning_trace.map((step, i) => (
                            <div key={i} className="text-[10px] font-mono text-white/60">
                              {i + 1}. {step}
                            </div>
                          ))}
                        </div>
                      </details>
                    )}

                    {/* Haptic Pattern */}
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-white/40">Haptic:</span>
                      <div className="flex gap-1">
                        {msg.manifest.haptic_pattern === 'Success' && <span className="text-green-400">● (single pulse)</span>}
                        {msg.manifest.haptic_pattern === 'Neutral' && <span className="text-yellow-400">●● (double pulse)</span>}
                        {msg.manifest.haptic_pattern === 'Warning' && <span className="text-orange-400">●●● (triple pulse)</span>}
                        {msg.manifest.haptic_pattern === 'Error' && <span className="text-red-400">●●●● (quad pulse)</span>}
                      </div>
                    </div>

                    {/* Follow-up Suggestion */}
                    {msg.manifest.suggested_followup && (
                      <div className="mt-2 p-2 bg-purple-500/10 border border-purple-500/20 rounded-lg text-xs text-purple-200">
                        💡 {msg.manifest.suggested_followup}
                      </div>
                    )}
                  </div>
                )}
                
                {/* Action Metadata Card */}
                {msg.metadata && msg.metadata.type === 'TRANSFER' && (
                   <div className="mt-3 p-3 rounded-lg bg-black/40 border border-white/10 text-xs font-mono">
                      <div className="flex justify-between mb-1"><span className="text-white/40">ACTION</span> <span className="text-emerald-400 font-bold">TRANSFER</span></div>
                      <div className="flex justify-between mb-1"><span className="text-white/40">TO</span> <span className="text-white">{msg.metadata.recipient}</span></div>
                      <div className="flex justify-between pt-2 border-t border-white/10"><span className="text-white/40">AMOUNT</span> <span className="text-xl font-bold text-white">{msg.metadata.amount} KARA</span></div>
                      <button className="w-full mt-3 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-white font-bold transition-colors">
                        CONFIRM TRANSACTION
                      </button>
                   </div>
                )}
              </div>

              {/* Feedback Buttons */}
              {msg.role === 'assistant' && !msg.feedbackGiven && (
                <div className="flex gap-2 items-center">
                  <button
                    onClick={() => handleFeedback(msg.id, true)}
                    className="p-1.5 rounded-full bg-green-500/10 hover:bg-green-500/20 border border-green-500/30 text-green-400 transition-all hover:scale-110"
                    title="Helpful"
                  >
                    <ThumbsUp size={12} />
                  </button>
                  <button
                    onClick={() => handleFeedback(msg.id, false)}
                    className="p-1.5 rounded-full bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 text-red-400 transition-all hover:scale-110"
                    title="Not helpful"
                  >
                    <ThumbsDown size={12} />
                  </button>
                  <span className="text-[9px] text-white/30 ml-1">Feedback improves Oracle</span>
                </div>
              )}
            </div>
          </div>
        ))}
        
        {isProcessing && (
           <div className="flex gap-4">
             <div className="w-10 h-10 rounded-full bg-purple-900/50 border border-purple-500/30 text-purple-400 flex items-center justify-center shrink-0 animate-pulse">
               <Sparkles size={18} />
             </div>
             <div className="flex items-center gap-1 h-10 px-4 rounded-2xl bg-purple-900/20 border border-purple-500/20">
               <span className="w-1.5 h-1.5 bg-purple-400 rounded-full animate-bounce"></span>
               <span className="w-1.5 h-1.5 bg-purple-400 rounded-full animate-bounce delay-100"></span>
               <span className="w-1.5 h-1.5 bg-purple-400 rounded-full animate-bounce delay-200"></span>
             </div>
           </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="p-4 bg-gradient-to-t from-black/80 to-transparent">
        <form onSubmit={handleSubmit} className="relative group">
          <div className="absolute inset-0 bg-purple-500/20 blur-xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask Oracle..."
            className="w-full bg-black/60 backdrop-blur-xl border border-white/10 rounded-full py-4 pl-12 pr-14 text-white focus:outline-none focus:border-purple-500/50 focus:bg-black/80 transition-all font-medium shadow-2xl"
          />
          <button type="button" className="absolute left-4 top-1/2 -translate-y-1/2 text-white/40 hover:text-white transition-colors">
            <Command size={18} />
          </button>
          
          <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
            {input.trim() ? (
              <button 
                type="submit" 
                disabled={isProcessing}
                className="p-2 bg-purple-600 hover:bg-purple-500 rounded-full text-white shadow-lg shadow-purple-900/50 transition-all hover:scale-110"
              >
                <ArrowRight size={18} />
              </button>
            ) : (
              <button type="button" className="p-2 text-white/40 hover:text-purple-400 transition-colors">
                <Mic size={20} />
              </button>
            )}
          </div>
        </form>
      </div>
    </div>
  );
};
