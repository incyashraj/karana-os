import React, { useState, useRef, useEffect } from 'react';
import { Camera, Video, RefreshCw, Image as ImageIcon, X } from 'lucide-react';

export const CameraApp: React.FC = () => {
  const [mode, setMode] = useState<'photo' | 'video'>('photo');
  const [captures, setCaptures] = useState<string[]>([]);
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [isFlashing, setIsFlashing] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    // In a real app, we would request a new stream or hook into the existing one.
    // For this simulator, we'll just try to get the camera again.
    navigator.mediaDevices.getUserMedia({ video: true })
      .then(stream => {
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
        }
      })
      .catch(err => console.error("Camera access denied:", err));
  }, []);

  const handleCapture = () => {
    setIsFlashing(true);
    setTimeout(() => setIsFlashing(false), 150);

    if (videoRef.current) {
      const canvas = document.createElement('canvas');
      canvas.width = videoRef.current.videoWidth;
      canvas.height = videoRef.current.videoHeight;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.drawImage(videoRef.current, 0, 0);
        const dataUrl = canvas.toDataURL('image/jpeg');
        setCaptures(prev => [dataUrl, ...prev]);
      }
    }
  };

  return (
    <div className="flex flex-col h-full bg-black text-white overflow-hidden rounded-b-3xl relative">
      {/* Viewfinder */}
      <div className="flex-1 relative bg-black overflow-hidden">
        <video 
          ref={videoRef}
          autoPlay 
          playsInline 
          muted 
          className="w-full h-full object-cover"
        />
        
        {/* Flash Effect */}
        <div className={`absolute inset-0 bg-white pointer-events-none transition-opacity duration-150 ${isFlashing ? 'opacity-100' : 'opacity-0'}`} />

        {/* Grid Overlay */}
        <div className="absolute inset-0 pointer-events-none opacity-20">
          <div className="w-full h-full grid grid-cols-3 grid-rows-3">
            {[...Array(9)].map((_, i) => (
              <div key={i} className="border border-white/50" />
            ))}
          </div>
        </div>
      </div>

      {/* Controls */}
      <div className="h-32 bg-black/80 backdrop-blur-md flex items-center justify-between px-8 pb-4">
        {/* Gallery Preview */}
        <button 
          onClick={() => captures.length > 0 && setSelectedImage(captures[0])}
          className="w-12 h-12 rounded-lg bg-white/10 overflow-hidden border border-white/20 hover:border-white transition-colors"
        >
          {captures.length > 0 ? (
            <img src={captures[0]} alt="Last capture" className="w-full h-full object-cover" />
          ) : (
            <div className="w-full h-full flex items-center justify-center">
              <ImageIcon size={20} className="text-white/30" />
            </div>
          )}
        </button>

        {/* Shutter Button */}
        <button 
          onClick={handleCapture}
          className="w-20 h-20 rounded-full border-4 border-white flex items-center justify-center hover:scale-105 active:scale-95 transition-all"
        >
          <div className="w-16 h-16 rounded-full bg-white" />
        </button>

        {/* Mode Switch */}
        <button 
          onClick={() => setMode(mode === 'photo' ? 'video' : 'photo')}
          className="w-12 h-12 rounded-full bg-white/10 flex items-center justify-center hover:bg-white/20 transition-colors"
        >
          <RefreshCw size={20} />
        </button>
      </div>

      {/* Image Viewer Modal */}
      {selectedImage && (
        <div className="absolute inset-0 z-50 bg-black flex flex-col">
          <div className="flex justify-end p-4">
            <button onClick={() => setSelectedImage(null)} className="p-2 bg-white/10 rounded-full">
              <X size={24} />
            </button>
          </div>
          <div className="flex-1 flex items-center justify-center p-4">
            <img src={selectedImage} alt="Capture" className="max-w-full max-h-full rounded-lg shadow-2xl" />
          </div>
          <div className="h-24 flex items-center gap-2 overflow-x-auto p-4 bg-white/5">
            {captures.map((cap, i) => (
              <button 
                key={i} 
                onClick={() => setSelectedImage(cap)}
                className={`h-full aspect-square rounded-md overflow-hidden border-2 ${selectedImage === cap ? 'border-cyan-500' : 'border-transparent'}`}
              >
                <img src={cap} alt={`Capture ${i}`} className="w-full h-full object-cover" />
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
