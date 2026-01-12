import React from 'react';
import { Scan, MessageSquare, ArrowRight } from 'lucide-react';
import { AnalysisResult } from '../../types';

interface VisionAppProps {
  isProcessing: boolean;
  analysis: AnalysisResult | null;
  onDiscuss?: (context: string) => void;
}

export const VisionApp: React.FC<VisionAppProps> = ({ isProcessing, analysis, onDiscuss }) => {
  return (
    <div className="h-full flex flex-col items-center justify-center p-8 relative">
      {/* Scanning State */}
      {isProcessing && !analysis && (
        <div className="flex flex-col items-center gap-6">
          <div className="relative w-32 h-32">
            <div className="absolute inset-0 border-4 border-amber-400/30 rounded-full animate-ping duration-[2000ms]"></div>
            <div className="absolute inset-0 border-4 border-t-amber-400 rounded-full animate-spin duration-[1500ms]"></div>
            <div className="absolute inset-4 border-2 border-amber-400/20 rounded-full animate-pulse"></div>
            <Scan className="absolute inset-0 m-auto text-amber-400 animate-pulse" size={40} />
          </div>
          <div className="flex flex-col items-center gap-2">
            <div className="text-amber-400 tracking-[0.2em] text-lg font-bold animate-pulse font-mono">ANALYZING SCENE</div>
            <div className="text-amber-400/50 text-xs font-mono">PROCESSING VISUAL DATA...</div>
          </div>
        </div>
      )}

      {/* Result State */}
      {analysis && (
        <div className="w-full h-full flex flex-col animate-in fade-in slide-in-from-bottom-4 duration-500">
          {/* Header Card */}
          <div className="flex justify-between items-start mb-6 p-6 bg-amber-950/20 border border-amber-500/20 rounded-2xl backdrop-blur-sm">
             <div>
               <div className="flex items-center gap-3 mb-2">
                 <h2 className="text-3xl font-bold text-amber-400 uppercase tracking-wide">{analysis.detectedObject}</h2>
                 <span className="px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 text-[10px] font-bold uppercase tracking-wider border border-amber-500/20">
                   {analysis.category}
                 </span>
               </div>
               <div className="flex gap-2">
                  {analysis.relatedTags.map(tag => (
                    <span key={tag} className="text-xs text-amber-400/60 font-mono">#{tag}</span>
                  ))}
               </div>
             </div>
             <div className="text-right">
               <div className="text-4xl font-mono font-bold text-amber-400">{analysis.confidence.toFixed(0)}<span className="text-lg opacity-50">%</span></div>
               <div className="text-[10px] text-amber-300 tracking-widest opacity-60">CONFIDENCE</div>
             </div>
          </div>

          {/* Description Body */}
          <div className="flex-1 overflow-auto p-6 bg-black/20 rounded-2xl border border-white/5 mb-6">
            <p className="text-amber-100 text-lg leading-relaxed font-light">
              {analysis.description}
            </p>
          </div>

          {/* Actions */}
          <div className="mt-auto flex gap-4">
            <button 
              onClick={() => onDiscuss?.(`I am looking at ${analysis.detectedObject}. ${analysis.description}`)}
              className="flex-1 py-4 bg-amber-600/20 hover:bg-amber-600/40 border border-amber-500/50 rounded-xl text-amber-200 font-bold flex items-center justify-center gap-3 transition-all group"
            >
              <MessageSquare size={20} className="group-hover:scale-110 transition-transform" />
              <span>Discuss with Oracle</span>
              <ArrowRight size={16} className="opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" />
            </button>
          </div>
        </div>
      )}
      
      {/* Idle State */}
      {!isProcessing && !analysis && (
        <div className="text-center text-amber-200/30 flex flex-col items-center gap-4">
            <div className="w-24 h-24 rounded-full border-2 border-dashed border-amber-500/20 flex items-center justify-center">
              <Scan size={48} className="opacity-50" />
            </div>
            <p className="font-mono text-sm tracking-widest uppercase">Visual Layer Standby</p>
        </div>
      )}
    </div>
  );
};
