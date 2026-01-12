import React, { useState, useEffect, useRef } from 'react';
import { Pin, Globe, Hand, Move, X } from 'lucide-react';
import { HandLandmarker, FilesetResolver } from "@mediapipe/tasks-vision";
import { quat } from 'gl-matrix';
import { CalibrationOverlay } from './CalibrationOverlay';
import { SensorFusionService } from '../services/SensorFusionService';
import { VisionService } from '../services/VisionService';
import { SpatialAnchorService, WorldAnchor } from '../services/SpatialAnchorService';

interface ARWindow {
  anchorId: string;
  title: string;
  content: React.ReactNode;
  width: number;
  height: number;
}

interface AROverlayProps {
  cameraFeedRef?: React.RefObject<any>;
  windows?: ARWindow[];
  onWindowClose?: (anchorId: string) => void;
}

export const AROverlay: React.FC<AROverlayProps> = ({ 
  cameraFeedRef, 
  windows = [],
  onWindowClose 
}) => {
  // Core State
  const [mousePos, setMousePos] = useState({ x: window.innerWidth / 2, y: window.innerHeight / 2 });
  const [isDragging, setIsDragging] = useState(false);
  const [handDetected, setHandDetected] = useState(false);
  const [gesture, setGesture] = useState<string>("None");
  
  // Calibration State
  const [isCalibrated, setIsCalibrated] = useState(false);
  const [showCalibration, setShowCalibration] = useState(true);
  const [gyroEnabled, setGyroEnabled] = useState(false);

  // Services
  const sensorFusionRef = useRef<SensorFusionService>(new SensorFusionService());
  const visionServiceRef = useRef<VisionService>(new VisionService());
  const spatialAnchorRef = useRef<SpatialAnchorService>(new SpatialAnchorService());
  
  // Tracking State
  const lastMousePos = useRef({ x: 0, y: 0 });
  const handLandmarkerRef = useRef<HandLandmarker | null>(null);
  const requestRef = useRef<number>(0);
  const lastPinchState = useRef(false);
  const draggedAnchorRef = useRef<string | null>(null);
  const dragStartPosRef = useRef<{ x: number; y: number } | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const cursorRef = useRef<HTMLDivElement>(null);
  
  // Window Positions (updated every frame)
  const [windowPositions, setWindowPositions] = useState<Map<string, { x: number; y: number; visible: boolean; depth: number }>>(new Map());

  // Gyroscope Handler
  useEffect(() => {
    if (!gyroEnabled || !isCalibrated) return;

    const handleOrientation = (event: DeviceOrientationEvent) => {
      if (event.alpha === null || event.beta === null || event.gamma === null) return;

      // Update rotation
      const currentRotation = sensorFusionRef.current.update(event.alpha, event.beta, event.gamma);
      
      // Update spatial anchor service (will be combined with translation)
      // Translation is updated in the video processing loop via optical flow
    };

    window.addEventListener('deviceorientation', handleOrientation);
    return () => window.removeEventListener('deviceorientation', handleOrientation);
  }, [gyroEnabled, isCalibrated]);

  const enableSensors = async () => {
    if (typeof (DeviceOrientationEvent as any).requestPermission === 'function') {
      try {
        const permission = await (DeviceOrientationEvent as any).requestPermission();
        if (permission === 'granted') {
          setGyroEnabled(true);
        }
      } catch (e) {
        console.error('Permission denied:', e);
      }
    } else {
      setGyroEnabled(true);
    }
  };

  const handleCalibrationComplete = () => {
    setIsCalibrated(true);
    setShowCalibration(false);
    
    // Set world origin
    const initialRotation = sensorFusionRef.current.getQuaternion();
    spatialAnchorRef.current.setOrigin(initialRotation);

    console.log('AR System Calibrated. World origin set.');
  };

  // Create anchors for new windows automatically
  useEffect(() => {
    // Always try to create anchors for new windows, even if not calibrated yet.
    // They will be placed relative to the current (or default) camera view.
    windows.forEach((window, index) => {
      // Check if anchor already exists
      const existing = spatialAnchorRef.current.getAnchorScreenPosition(window.anchorId);
      if (!existing) {
        // Create new anchor at center of screen
        // If not calibrated, this will be at (0,0,-2) relative to default camera
        const x = window.innerWidth / 2 + (index - windows.length / 2) * 50; // Slight offset for stacking
        const y = window.innerHeight / 2;
        spatialAnchorRef.current.createAnchor(window.anchorId, x, y, 2.0, window.title);
        console.log('Created anchor for:', window.title, 'at', x, y);
      }
    });
  }, [windows]); // Removed isCalibrated dependency

  // Initialize MediaPipe Hand Tracking
  useEffect(() => {
    const initHandLandmarker = async () => {
      try {
        const vision = await FilesetResolver.forVisionTasks(
          "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.0/wasm"
        );
        handLandmarkerRef.current = await HandLandmarker.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath: `https://storage.googleapis.com/mediapipe-models/hand_landmarker/hand_landmarker/float16/1/hand_landmarker.task`,
            delegate: "GPU"
          },
          runningMode: "VIDEO",
          numHands: 1
        });
        console.log("HandLandmarker initialized");
      } catch (error) {
        console.error("Error initializing HandLandmarker:", error);
      }
    };
    initHandLandmarker();
  }, []);

  // Main Processing Loop
  const processVideo = () => {
    const videoElement = cameraFeedRef?.current?.video;
    
    if (videoElement && videoElement.readyState >= 2) {
      // 1. Update Camera Pose (Only if calibrated and sensors active)
      if (isCalibrated && gyroEnabled) {
        const result = visionServiceRef.current.processFrame(videoElement);
        const currentRotation = sensorFusionRef.current.getQuaternion();
        spatialAnchorRef.current.updateCameraPose(currentRotation, result);

        // Draw tracked points
        if (canvasRef.current && result.points) {
          const ctx = canvasRef.current.getContext('2d');
          if (ctx) {
            ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
            ctx.fillStyle = '#06b6d4'; // Cyan
            
            // Scale points to screen
            const scaleX = window.innerWidth / 320; // VisionService width
            const scaleY = window.innerHeight / 240; // VisionService height
            
            result.points.forEach(p => {
              ctx.beginPath();
              ctx.arc(p.x * scaleX, p.y * scaleY, 2, 0, Math.PI * 2);
              ctx.fill();
            });
          }
        }
      }

      // 2. Update Window Positions (Always run to ensure rendering)
      const newPositions = new Map<string, { x: number; y: number; visible: boolean; depth: number }>();
      windows.forEach(window => {
        const screenPos = spatialAnchorRef.current.getAnchorScreenPosition(window.anchorId);
        if (screenPos) {
          newPositions.set(window.anchorId, {
            x: screenPos.x,
            y: screenPos.y,
            visible: screenPos.visible,
            depth: screenPos.depth
          });
        }
      });
      setWindowPositions(newPositions);

      // 3. Hand Tracking (always run for both desktop and mobile)
      if (handLandmarkerRef.current) {
        const results = handLandmarkerRef.current.detectForVideo(videoElement, performance.now());
        
        if (results.landmarks && results.landmarks.length > 0) {
          setHandDetected(true);
          const landmarks = results.landmarks[0];
          
          const indexTip = landmarks[8];
          const thumbTip = landmarks[4];

          // Correct coordinate mapping for object-cover
          let x = 0, y = 0;
          
          if (videoElement.videoWidth && videoElement.videoHeight) {
            const videoAspect = videoElement.videoWidth / videoElement.videoHeight;
            const screenAspect = window.innerWidth / window.innerHeight;
            
            let scale, offsetX, offsetY;
            
            if (screenAspect < videoAspect) {
              // Screen is narrower than video (Mobile Portrait)
              // Video is scaled to match screen height
              scale = window.innerHeight / videoElement.videoHeight;
              const scaledWidth = videoElement.videoWidth * scale;
              offsetX = (scaledWidth - window.innerWidth) / 2;
              offsetY = 0;
              
              x = (indexTip.x * scaledWidth) - offsetX;
              y = (indexTip.y * window.innerHeight) - offsetY;
            } else {
              // Screen is wider than video (Desktop)
              // Video is scaled to match screen width
              scale = window.innerWidth / videoElement.videoWidth;
              const scaledHeight = videoElement.videoHeight * scale;
              offsetX = 0;
              offsetY = (scaledHeight - window.innerHeight) / 2;
              
              x = (indexTip.x * window.innerWidth) - offsetX;
              y = (indexTip.y * scaledHeight) - offsetY;
            }
          } else {
            // Fallback if video dimensions not ready
            x = indexTip.x * window.innerWidth;
            y = indexTip.y * window.innerHeight;
          }
          
          // No smoothing for instant response
          // Direct DOM update for zero latency
          if (cursorRef.current) {
            cursorRef.current.style.transform = `translate(${x}px, ${y}px) translate(-50%, -50%)`;
          }
          // Keep state for logic, but maybe we can debounce this if it causes lag?
          // For now, we still need it for click detection logic below
          // setMousePos({ x, y }); // Removing state update to prevent re-renders

          const distance = Math.hypot(indexTip.x - thumbTip.x, indexTip.y - thumbTip.y);
          const isPinching = distance < 0.06; // Slightly relaxed threshold

          // Handle Pinch Gestures
          if (isPinching && !lastPinchState.current) {
            // Pinch Start
            setGesture("Pinch");
            setIsDragging(true);
            
            // Check if we're grabbing a window header
            const element = document.elementFromPoint(x, y);
            const windowElement = element?.closest('[data-anchor-id]');
            
            if (windowElement) {
              const anchorId = windowElement.getAttribute('data-anchor-id');
              if (anchorId) {
                draggedAnchorRef.current = anchorId;
                dragStartPosRef.current = { x, y };
                setGesture("Dragging Window");
              }
            } else {
              // Click on buttons/interactive elements
              const clickableElement = element?.closest('button, [role="button"], a, input, select, textarea');
              if (clickableElement) {
                console.log('Pinch click on:', clickableElement);
                const clickEvent = new MouseEvent('click', {
                  view: window,
                  bubbles: true,
                  cancelable: true,
                  clientX: x,
                  clientY: y
                });
                clickableElement.dispatchEvent(clickEvent);
              }
            }
          } else if (!isPinching && lastPinchState.current) {
            // Pinch End
            setGesture("Open Hand");
            setIsDragging(false);
            
            if (draggedAnchorRef.current && dragStartPosRef.current && isCalibrated) {
              // Update anchor position in world space
              spatialAnchorRef.current.createAnchor(
                draggedAnchorRef.current,
                x,
                y,
                2.0
              );
            }
            
            draggedAnchorRef.current = null;
            dragStartPosRef.current = null;
          } else if (isPinching && draggedAnchorRef.current && dragStartPosRef.current) {
            // Dragging - update anchor in real-time
            if (isCalibrated) {
              spatialAnchorRef.current.createAnchor(
                draggedAnchorRef.current,
                x,
                y,
                2.0
              );
            }
          }

          lastPinchState.current = isPinching;
          lastMousePos.current = { x, y };
        } else {
          setHandDetected(false);
          setGesture("None");
          // handPosBuffer.current = []; // Removed
        }
      }
    }

    requestRef.current = requestAnimationFrame(processVideo);
  };

  useEffect(() => {
    if (cameraFeedRef?.current?.video) {
      console.log("AROverlay: Starting AR processing loop");
      requestRef.current = requestAnimationFrame(processVideo);
    }
    return () => cancelAnimationFrame(requestRef.current);
  }, [cameraFeedRef, windows]); // Removed isCalibrated dependency for immediate start

  // Mouse Fallback
  useEffect(() => {
    if (handDetected) return;

    const handleMouseMove = (e: MouseEvent) => {
      // setMousePos({ x: e.clientX, y: e.clientY }); // Removed state update
      if (cursorRef.current) {
        cursorRef.current.style.transform = `translate(${e.clientX}px, ${e.clientY}px) translate(-50%, -50%)`;
      }
      
      if (isDragging && draggedAnchorRef.current) {
        spatialAnchorRef.current.createAnchor(
          draggedAnchorRef.current,
          e.clientX,
          e.clientY,
          2.0
        );
      }
    };

    const handleMouseDown = (e: MouseEvent) => {
      const element = document.elementFromPoint(e.clientX, e.clientY);
      const windowElement = element?.closest('[data-anchor-id]');
      
      if (windowElement) {
        const anchorId = windowElement.getAttribute('data-anchor-id');
        if (anchorId) {
          draggedAnchorRef.current = anchorId;
          setIsDragging(true);
        }
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      draggedAnchorRef.current = null;
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('mouseup', handleMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mousedown', handleMouseDown);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [handDetected, isDragging]);

  // Debug Info
  const cameraPos = spatialAnchorRef.current.getCameraPosition();

  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden">
      {/* Tracking Status Indicator */}
      <div className="absolute top-24 right-4 flex flex-col gap-2 items-end z-[20]">
        <div className={`px-3 py-1 rounded-full text-[10px] font-bold tracking-wider backdrop-blur-md border ${
          handDetected ? 'bg-emerald-500/20 border-emerald-500/50 text-emerald-400' : 'bg-red-500/20 border-red-500/50 text-red-400'
        }`}>
          HAND: {handDetected ? 'LOCKED' : 'SEARCHING'}
        </div>
        <div className={`px-3 py-1 rounded-full text-[10px] font-bold tracking-wider backdrop-blur-md border ${
          isCalibrated ? 'bg-cyan-500/20 border-cyan-500/50 text-cyan-400' : 'bg-amber-500/20 border-amber-500/50 text-amber-400'
        }`}>
          WORLD: {isCalibrated ? 'TRACKING' : 'WAITING'}
        </div>
      </div>

      {/* Calibration Overlay */}
      {showCalibration && (
        <div className="pointer-events-auto">
          <CalibrationOverlay 
            onComplete={handleCalibrationComplete} 
            onCancel={() => setShowCalibration(false)}
            onStart={enableSensors}
          />
        </div>
      )}

      {/* Point Cloud Visualization */}
      {isCalibrated && (
        <canvas 
          ref={canvasRef}
          width={window.innerWidth}
          height={window.innerHeight}
          className="absolute inset-0 pointer-events-none opacity-50"
        />
      )}

      {/* Gaze Cursor */}
      <div 
        ref={cursorRef}
        className={`fixed w-8 h-8 border-2 rounded-full pointer-events-none flex items-center justify-center z-[100] ${
          isDragging ? 'border-amber-400 scale-125' : 'border-cyan-400/50'
        }`}
        style={{ left: 0, top: 0, transform: `translate(${window.innerWidth/2}px, ${window.innerHeight/2}px) translate(-50%, -50%)` }}
      >
        <div className={`w-1 h-1 rounded-full ${isDragging ? 'bg-amber-400' : 'bg-cyan-400'}`} />
      </div>

      {/* AR Windows */}
      {windows.map(window => {
        const pos = windowPositions.get(window.anchorId);
        if (!pos || !pos.visible) return null;

        const scale = Math.max(0.5, Math.min(1.5, 2 / pos.depth)); // Scale based on depth

        return (
          <div
            key={window.anchorId}
            data-anchor-id={window.anchorId}
            className="absolute pointer-events-auto bg-black/80 backdrop-blur-xl border border-white/20 rounded-2xl shadow-2xl"
            style={{
              left: pos.x,
              top: pos.y,
              width: window.width,
              height: window.height,
              transform: `translate(-50%, -50%) scale(${scale})`,
              opacity: pos.depth < 0.5 ? 0.5 : 1
            }}
          >
            {/* Window Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-white/10 bg-white/5 cursor-move">
              <div className="flex items-center gap-2">
                <Globe size={14} className="text-cyan-400" />
                <span className="text-xs font-bold text-white/80">{window.title}</span>
              </div>
              <button 
                onClick={() => onWindowClose?.(window.anchorId)}
                className="text-white/40 hover:text-white/80 transition-colors"
              >
                <X size={14} />
              </button>
            </div>

            {/* Window Content */}
            <div className="p-4 overflow-auto" style={{ height: window.height - 50 }}>
              {window.content}
            </div>
          </div>
        );
      })}

      {/* Debug Info */}
      {isCalibrated && (
        <div className="absolute top-4 left-4 text-[10px] font-mono text-white/30 bg-black/50 p-3 rounded pointer-events-none">
          <div>CAMERA: {cameraPos.x.toFixed(2)}, {cameraPos.y.toFixed(2)}, {cameraPos.z.toFixed(2)}</div>
          <div>GYRO: {gyroEnabled ? 'ON' : 'OFF'}</div>
          <div>HAND: {handDetected ? gesture : 'None'}</div>
          <div>ANCHORS: {windows.length}</div>
        </div>
      )}

      {/* Instructions */}
      <div className="absolute top-4 right-4 text-right pointer-events-auto">
        <div className="text-xs font-bold text-white/50 bg-black/50 px-3 py-2 rounded">SPATIAL AR MODE</div>
        <div className="text-[10px] text-white/30 mt-1">PINCH/DRAG HEADER to Reposition</div>
        
        {!gyroEnabled && isCalibrated && (
          <button 
            onClick={enableSensors}
            className="mt-2 px-3 py-1 bg-cyan-500/20 border border-cyan-500/50 rounded text-[10px] text-cyan-400 font-bold hover:bg-cyan-500/40 transition-colors"
          >
            ENABLE GYRO
          </button>
        )}

        {handDetected && (
          <div className="text-xs text-green-400 font-bold mt-2 flex items-center justify-end gap-1 bg-black/50 px-2 py-1 rounded">
            <Hand size={12} /> TRACKING
          </div>
        )}
      </div>
    </div>
  );
};

export default AROverlay;
