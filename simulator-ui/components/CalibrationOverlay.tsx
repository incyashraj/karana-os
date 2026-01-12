import React, { useEffect, useState } from 'react';
import { Scan, CheckCircle2, RotateCw } from 'lucide-react';

interface CalibrationOverlayProps {
  onComplete: () => void;
  onCancel: () => void;
  onStart?: () => void;
}

export const CalibrationOverlay: React.FC<CalibrationOverlayProps> = ({ onComplete, onCancel, onStart }) => {
  const [step, setStep] = useState<'init' | 'scanning' | 'complete'>('init');
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    if (step === 'scanning') {
      if (onStart) onStart();
      const interval = setInterval(() => {
        setProgress(prev => {
          if (prev >= 100) {
            clearInterval(interval);
            setStep('complete');
            return 100;
          }
          return prev + 2; // Simulate scanning progress
        });
      }, 50);
      return () => clearInterval(interval);
    }
  }, [step]);

  return (
    <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-black/60 backdrop-blur-sm text-white">
      <div className="max-w-md w-full p-8 text-center space-y-6">
        
        {step === 'init' && (
          <div className="animate-in fade-in zoom-in duration-300 space-y-6">
            <div className="w-20 h-20 mx-auto rounded-full bg-cyan-500/20 flex items-center justify-center border border-cyan-500/50 animate-pulse">
              <RotateCw className="w-10 h-10 text-cyan-400 animate-spin-slow" />
            </div>
            <div>
              <h2 className="text-2xl font-bold font-mono tracking-wider">SENSOR CALIBRATION</h2>
              <p className="text-white/60 mt-2">Hold your device steady at eye level.</p>
            </div>
            <button 
              onClick={() => setStep('scanning')}
              className="px-8 py-3 bg-cyan-500 hover:bg-cyan-400 text-black font-bold rounded-full transition-all transform hover:scale-105"
            >
              START MAPPING
            </button>
          </div>
        )}

        {step === 'scanning' && (
          <div className="space-y-6">
            <div className="relative w-64 h-64 mx-auto border-2 border-cyan-500/30 rounded-lg overflow-hidden">
              <div className="absolute inset-0 grid grid-cols-4 grid-rows-4 gap-1 opacity-30">
                {Array.from({ length: 16 }).map((_, i) => (
                  <div key={i} className="border border-cyan-500/20" />
                ))}
              </div>
              <div className="absolute top-0 left-0 w-full h-1 bg-cyan-400 shadow-[0_0_15px_rgba(34,211,238,0.8)] animate-scan-down" />
              <Scan className="absolute inset-0 m-auto w-12 h-12 text-cyan-400 opacity-50" />
            </div>
            <div>
              <h2 className="text-xl font-mono animate-pulse">MAPPING ENVIRONMENT...</h2>
              <div className="w-full h-2 bg-white/10 rounded-full mt-4 overflow-hidden">
                <div 
                  className="h-full bg-cyan-500 transition-all duration-100 ease-out"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-xs font-mono text-cyan-500/80 mt-2">
                ACQUIRING SPATIAL ANCHORS: {Math.floor(progress * 12.4)}
              </p>
            </div>
          </div>
        )}

        {step === 'complete' && (
          <div className="animate-in fade-in zoom-in duration-300 space-y-6">
            <div className="w-20 h-20 mx-auto rounded-full bg-green-500/20 flex items-center justify-center border border-green-500/50">
              <CheckCircle2 className="w-10 h-10 text-green-400" />
            </div>
            <div>
              <h2 className="text-2xl font-bold font-mono">SYSTEM LOCKED</h2>
              <p className="text-white/60 mt-2">Spatial coordinates established.</p>
            </div>
            <button 
              onClick={onComplete}
              className="px-8 py-3 bg-white text-black font-bold rounded-full hover:bg-gray-200 transition-all"
            >
              ENTER AR MODE
            </button>
          </div>
        )}

      </div>
    </div>
  );
};
