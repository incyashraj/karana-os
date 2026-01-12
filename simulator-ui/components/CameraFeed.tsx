import React, { useEffect, useRef, forwardRef, useImperativeHandle } from 'react';

interface CameraFeedProps {
  active: boolean;
  onFrameCapture?: (dataUrl: string) => void;
  onVideoReady?: (video: HTMLVideoElement) => void;
}

export interface CameraFeedHandle {
  captureFrame: () => string | null;
  video: HTMLVideoElement | null;
}

const CameraFeed = forwardRef<CameraFeedHandle, CameraFeedProps>(({ active, onVideoReady }, ref) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useImperativeHandle(ref, () => ({
    captureFrame: () => {
      if (!videoRef.current || !canvasRef.current) return null;
      
      const video = videoRef.current;
      const canvas = canvasRef.current;
      const context = canvas.getContext('2d');
      
      if (!context) return null;

      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      
      return canvas.toDataURL('image/jpeg', 0.8);
    },
    video: videoRef.current
  }));

  useEffect(() => {
    let stream: MediaStream | null = null;

    const startCamera = async () => {
      try {
        stream = await navigator.mediaDevices.getUserMedia({ 
          video: { facingMode: 'environment', width: { ideal: 1920 }, height: { ideal: 1080 } },
          audio: false 
        });
        
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          videoRef.current.onloadedmetadata = () => {
            console.log("CameraFeed: Video metadata loaded, attempting to play");
            videoRef.current?.play().then(() => console.log("CameraFeed: Video playing")).catch(e => console.error("CameraFeed: Play failed", e));
            if (onVideoReady && videoRef.current) {
              onVideoReady(videoRef.current);
            }
          };
        }
      } catch (err) {
        console.warn("Camera access denied or unavailable, using fallback.");
      }
    };

    if (active) {
      startCamera();
    }

    return () => {
      if (stream) {
        stream.getTracks().forEach(track => track.stop());
      }
    };
  }, [active]);

  return (
    <div className="absolute inset-0 w-full h-full bg-black z-0 overflow-hidden">
      {/* Real Camera Feed */}
      <video 
        ref={videoRef}
        autoPlay 
        playsInline 
        muted 
        className="w-full h-full object-cover opacity-80"
      />
      
      {/* Fallback Background (if camera fails or loads slow) */}
      <div className="absolute inset-0 -z-20 bg-[url('https://picsum.photos/1920/1080?grayscale&blur=2')] bg-cover bg-center" />
      
      {/* Hidden Canvas for Capture */}
      <canvas ref={canvasRef} className="hidden" />
      
      {/* Scanlines Effect */}
      <div className="absolute inset-0 bg-[linear-gradient(rgba(18,16,16,0)_50%,rgba(0,0,0,0.25)_50%),linear-gradient(90deg,rgba(255,0,0,0.06),rgba(0,255,0,0.02),rgba(0,0,255,0.06))] z-0 bg-[length:100%_2px,3px_100%] pointer-events-none" />
    </div>
  );
});

export default CameraFeed;
