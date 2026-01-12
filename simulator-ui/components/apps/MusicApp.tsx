import React, { useState, useEffect, useRef } from 'react';
import { Play, Pause, SkipBack, SkipForward, Volume2, Music, Disc, List } from 'lucide-react';

interface Track {
  id: string;
  title: string;
  artist: string;
  duration: string;
  cover: string;
}

const MOCK_TRACKS: Track[] = [
  { id: '1', title: 'Neon Horizon', artist: 'Cyber Dreams', duration: '3:45', cover: 'bg-cyan-500' },
  { id: '2', title: 'Digital Rain', artist: 'System Core', duration: '4:20', cover: 'bg-emerald-500' },
  { id: '3', title: 'Void Walker', artist: 'Null Pointer', duration: '2:55', cover: 'bg-purple-500' },
  { id: '4', title: 'Quantum Flux', artist: 'The Entanglement', duration: '5:10', cover: 'bg-pink-500' },
  { id: '5', title: 'Binary Sunset', artist: 'Legacy Code', duration: '3:30', cover: 'bg-amber-500' },
];

export const MusicApp: React.FC = () => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTrackIndex, setCurrentTrackIndex] = useState(0);
  const [progress, setProgress] = useState(0);
  const [volume, setVolume] = useState(80);
  
  const currentTrack = MOCK_TRACKS[currentTrackIndex];

  useEffect(() => {
    let interval: any;
    if (isPlaying) {
      interval = setInterval(() => {
        setProgress(prev => {
          if (prev >= 100) {
            handleNext();
            return 0;
          }
          return prev + 0.5;
        });
      }, 100);
    }
    return () => clearInterval(interval);
  }, [isPlaying]);

  const handlePlayPause = () => setIsPlaying(!isPlaying);
  
  const handleNext = () => {
    setCurrentTrackIndex(prev => (prev + 1) % MOCK_TRACKS.length);
    setProgress(0);
  };

  const handlePrev = () => {
    setCurrentTrackIndex(prev => (prev - 1 + MOCK_TRACKS.length) % MOCK_TRACKS.length);
    setProgress(0);
  };

  return (
    <div className="flex flex-col h-full bg-gradient-to-b from-zinc-900 to-black text-white overflow-hidden rounded-b-3xl">
      {/* Now Playing Header */}
      <div className="p-6 flex flex-col items-center gap-6 flex-1 justify-center">
        <div className={`
          w-48 h-48 rounded-full shadow-[0_0_50px_rgba(0,0,0,0.5)] 
          flex items-center justify-center relative overflow-hidden
          ${isPlaying ? 'animate-[spin_10s_linear_infinite]' : ''}
        `}>
          <div className={`absolute inset-0 opacity-50 ${currentTrack.cover}`} />
          <div className="absolute inset-2 bg-black rounded-full flex items-center justify-center">
            <div className={`w-16 h-16 rounded-full ${currentTrack.cover} opacity-20`} />
          </div>
          <Disc size={48} className="text-white/50 relative z-10" />
        </div>

        <div className="text-center space-y-2">
          <h2 className="text-2xl font-bold tracking-tight">{currentTrack.title}</h2>
          <p className="text-white/50 font-medium tracking-wide uppercase text-xs">{currentTrack.artist}</p>
        </div>
      </div>

      {/* Controls */}
      <div className="p-6 bg-white/5 backdrop-blur-md border-t border-white/10 space-y-6">
        {/* Progress */}
        <div className="space-y-2">
          <div className="h-1 bg-white/10 rounded-full overflow-hidden">
            <div 
              className="h-full bg-cyan-500 transition-all duration-100 ease-linear"
              style={{ width: `${progress}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] font-mono text-white/30">
            <span>0:00</span>
            <span>{currentTrack.duration}</span>
          </div>
        </div>

        {/* Buttons */}
        <div className="flex items-center justify-center gap-8">
          <button onClick={handlePrev} className="text-white/70 hover:text-white transition-colors">
            <SkipBack size={24} />
          </button>
          <button 
            onClick={handlePlayPause}
            className="w-16 h-16 rounded-full bg-cyan-500 text-black flex items-center justify-center hover:scale-105 active:scale-95 transition-all shadow-[0_0_20px_rgba(6,182,212,0.4)]"
          >
            {isPlaying ? <Pause size={24} fill="currentColor" /> : <Play size={24} fill="currentColor" className="ml-1" />}
          </button>
          <button onClick={handleNext} className="text-white/70 hover:text-white transition-colors">
            <SkipForward size={24} />
          </button>
        </div>

        {/* Volume */}
        <div className="flex items-center gap-4 px-4">
          <Volume2 size={16} className="text-white/50" />
          <input 
            type="range" 
            min="0" 
            max="100" 
            value={volume}
            onChange={(e) => setVolume(parseInt(e.target.value))}
            className="flex-1 h-1 bg-white/10 rounded-lg appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
          />
        </div>
      </div>
    </div>
  );
};
