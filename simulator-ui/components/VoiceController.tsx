// Voice Controller Component for karana-os
// Provides voice input with waveform visualization and real-time feedback

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Mic, MicOff, Volume2, Loader, Undo2, Zap, Circle } from 'lucide-react';
import { useWebSocket, WsMessage, ToolResult as WsToolResult } from '../services/wsService';

interface VoiceControllerProps {
  onTranscript?: (text: string) => void;
  onToolResult?: (result: WsToolResult) => void;
  className?: string;
}

type ListeningMode = 'off' | 'push-to-talk' | 'continuous';

interface ToolExecution {
  id: string;
  toolName: string;
  result: string;
  timestamp: number;
  confidence: number;
}

export const VoiceController: React.FC<VoiceControllerProps> = ({
  onTranscript,
  onToolResult,
  className = '',
}) => {
  const [listeningMode, setListeningMode] = useState<ListeningMode>('off');
  const [isProcessing, setIsProcessing] = useState(false);
  const [currentTranscript, setCurrentTranscript] = useState('');
  const [partialTranscript, setPartialTranscript] = useState('');
  const [voiceActive, setVoiceActive] = useState(false);
  const [energyLevel, setEnergyLevel] = useState(0);
  const [recentExecutions, setRecentExecutions] = useState<ToolExecution[]>([]);
  const [showFeedback, setShowFeedback] = useState(false);
  const [lastExecution, setLastExecution] = useState<ToolExecution | null>(null);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number | null>(null);

  const { isConnected, lastMessage, subscribe } = useWebSocket();

  // Subscribe to WebSocket messages
  useEffect(() => {
    const unsubscribeTranscription = subscribe('Transcription', (msg) => {
      const transcription = msg as any;
      if (transcription.is_partial) {
        setPartialTranscript(transcription.text);
      } else {
        setCurrentTranscript(transcription.text);
        setPartialTranscript('');
        onTranscript?.(transcription.text);
      }
    });

    const unsubscribeToolResult = subscribe('ToolResult', (msg) => {
      const toolResult = msg as any;
      const execution: ToolExecution = {
        id: toolResult.execution_id,
        toolName: toolResult.tool_name,
        result: toolResult.result,
        timestamp: toolResult.timestamp,
        confidence: toolResult.confidence,
      };
      
      setRecentExecutions(prev => [execution, ...prev].slice(0, 5));
      setLastExecution(execution);
      setShowFeedback(true);
      onToolResult?.(toolResult);

      // Hide feedback after 3 seconds
      setTimeout(() => {
        setShowFeedback(false);
      }, 3000);
    });

    const unsubscribeVoiceActivity = subscribe('VoiceActivity', (msg) => {
      const activity = msg as any;
      setVoiceActive(activity.active);
      setEnergyLevel(activity.energy_level);
    });

    return () => {
      unsubscribeTranscription();
      unsubscribeToolResult();
      unsubscribeVoiceActivity();
    };
  }, [subscribe, onTranscript, onToolResult]);

  // Waveform visualization
  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const draw = () => {
      const width = canvas.width;
      const height = canvas.height;

      // Clear canvas
      ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
      ctx.fillRect(0, 0, width, height);

      // Draw waveform
      if (voiceActive && energyLevel > 0) {
        const barCount = 40;
        const barWidth = width / barCount;
        const time = Date.now() / 1000;

        for (let i = 0; i < barCount; i++) {
          const amplitude = energyLevel * (Math.sin(time * 2 + i / 5) + 1) / 2;
          const barHeight = amplitude * height * 0.8;
          const x = i * barWidth;
          const y = (height - barHeight) / 2;

          // Gradient
          const gradient = ctx.createLinearGradient(0, y, 0, y + barHeight);
          gradient.addColorStop(0, 'rgba(168, 85, 247, 0.8)'); // purple-500
          gradient.addColorStop(1, 'rgba(139, 92, 246, 0.4)'); // purple-600

          ctx.fillStyle = gradient;
          ctx.fillRect(x, y, barWidth - 2, barHeight);
        }
      } else {
        // Idle state - subtle pulse
        const time = Date.now() / 1000;
        const pulse = (Math.sin(time * 2) + 1) / 2 * 0.2;
        ctx.fillStyle = `rgba(168, 85, 247, ${pulse})`;
        ctx.fillRect(0, height / 2 - 2, width, 4);
      }

      animationRef.current = requestAnimationFrame(draw);
    };

    draw();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [voiceActive, energyLevel]);

  const toggleListeningMode = () => {
    if (listeningMode === 'off') {
      setListeningMode('continuous');
    } else if (listeningMode === 'continuous') {
      setListeningMode('push-to-talk');
    } else {
      setListeningMode('off');
    }
  };

  const handleUndo = () => {
    if (recentExecutions.length > 0) {
      // TODO: Send undo command to backend
      console.log('[VOICE] Undo requested for:', recentExecutions[0]);
      setRecentExecutions(prev => prev.slice(1));
    }
  };

  const getStatusText = () => {
    if (!isConnected) return 'Disconnected';
    if (isProcessing) return 'Processing...';
    if (voiceActive) return 'Listening...';
    if (listeningMode === 'continuous') return 'Ready';
    if (listeningMode === 'push-to-talk') return 'Press to talk';
    return 'Off';
  };

  const getStatusColor = () => {
    if (!isConnected) return 'text-red-400';
    if (voiceActive) return 'text-green-400';
    if (listeningMode !== 'off') return 'text-purple-400';
    return 'text-gray-400';
  };

  return (
    <div className={`voice-controller ${className}`}>
      {/* Main Control */}
      <div className="flex flex-col items-center gap-4 p-4 bg-black/80 backdrop-blur-xl rounded-2xl border border-white/10">
        {/* Waveform */}
        <div className="w-full h-24 bg-black/40 rounded-xl overflow-hidden relative">
          <canvas
            ref={canvasRef}
            width={400}
            height={96}
            className="w-full h-full"
          />
          {listeningMode === 'off' && (
            <div className="absolute inset-0 flex items-center justify-center text-white/30 text-sm">
              Click mic to start
            </div>
          )}
        </div>

        {/* Controls */}
        <div className="flex items-center gap-4">
          {/* Mic Toggle */}
          <button
            onClick={toggleListeningMode}
            disabled={!isConnected}
            className={`w-16 h-16 rounded-full flex items-center justify-center transition-all ${
              listeningMode === 'off'
                ? 'bg-gray-700 hover:bg-gray-600'
                : voiceActive
                ? 'bg-green-500 hover:bg-green-600 animate-pulse'
                : 'bg-purple-500 hover:bg-purple-600'
            } disabled:opacity-50 disabled:cursor-not-allowed shadow-lg`}
          >
            {listeningMode === 'off' ? (
              <MicOff size={24} className="text-white" />
            ) : (
              <Mic size={24} className="text-white" />
            )}
          </button>

          {/* Mode Indicator */}
          <div className="flex flex-col items-start">
            <div className={`text-sm font-medium ${getStatusColor()}`}>
              {getStatusText()}
            </div>
            <div className="text-xs text-white/40">
              {listeningMode === 'off' && 'Tap to activate'}
              {listeningMode === 'continuous' && 'Always listening'}
              {listeningMode === 'push-to-talk' && 'Hold to speak'}
            </div>
          </div>

          {/* Undo Button */}
          {recentExecutions.length > 0 && (
            <button
              onClick={handleUndo}
              className="w-10 h-10 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center transition-all"
              title="Undo last action"
            >
              <Undo2 size={18} className="text-white/60" />
            </button>
          )}
        </div>

        {/* Transcription Display */}
        {(currentTranscript || partialTranscript) && (
          <div className="w-full p-3 bg-white/5 rounded-xl border border-white/10">
            <div className="text-white text-sm">
              {currentTranscript || (
                <span className="text-white/50 italic">{partialTranscript}...</span>
              )}
            </div>
          </div>
        )}

        {/* Connection Status */}
        <div className="flex items-center gap-2 text-xs">
          <Circle
            size={8}
            className={isConnected ? 'text-green-400 fill-current' : 'text-red-400 fill-current'}
          />
          <span className="text-white/40">
            {isConnected ? 'Connected' : 'Connecting...'}
          </span>
        </div>
      </div>

      {/* Tool Execution Feedback */}
      {showFeedback && lastExecution && (
        <div className="mt-4 p-4 bg-green-900/30 backdrop-blur-xl rounded-xl border border-green-500/30 animate-in fade-in slide-in-from-bottom-4">
          <div className="flex items-start gap-3">
            <Zap size={20} className="text-green-400 mt-0.5" />
            <div className="flex-1">
              <div className="text-sm font-medium text-green-400">
                {lastExecution.toolName}
              </div>
              <div className="text-sm text-white/80 mt-1">
                {lastExecution.result}
              </div>
              <div className="text-xs text-white/40 mt-1">
                Confidence: {(lastExecution.confidence * 100).toFixed(0)}%
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Recent Executions */}
      {recentExecutions.length > 0 && (
        <div className="mt-4 space-y-2">
          <div className="text-xs text-white/40 uppercase tracking-wider px-2">
            Recent Actions
          </div>
          {recentExecutions.slice(0, 3).map((execution) => (
            <div
              key={execution.id}
              className="p-3 bg-white/5 backdrop-blur-xl rounded-lg border border-white/5 text-xs"
            >
              <div className="flex items-center gap-2">
                <span className="text-purple-400 font-medium">
                  {execution.toolName}
                </span>
                <span className="text-white/60">{execution.result}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default VoiceController;
