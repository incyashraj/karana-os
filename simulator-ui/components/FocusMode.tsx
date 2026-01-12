/**
 * Focus Mode UI Component
 * 
 * Philosophy:
 * - Minimal by default, contextual when needed
 * - Information appears only when relevant
 * - User's attention is protected
 * - Reduces cognitive load and distraction
 */

import React, { useState, useEffect } from 'react';
import { visualIntelligence, VisualFeedback, EyeTrackingData } from '../services/visualIntelligence';

interface FocusModeProps {
  isActive: boolean;
}

const FocusMode: React.FC<FocusModeProps> = ({ isActive }) => {
  const [feedback, setFeedback] = useState<VisualFeedback | null>(null);
  const [gazeData, setGazeData] = useState<EyeTrackingData | null>(null);
  const [showFeedback, setShowFeedback] = useState(false);
  const [fadingOut, setFadingOut] = useState(false);
  
  useEffect(() => {
    if (!isActive) return;
    
    const interval = setInterval(() => {
      const currentFeedback = visualIntelligence.getCurrentFeedback();
      const currentGaze = visualIntelligence.getCurrentGaze();
      
      setGazeData(currentGaze);
      
      // Only show feedback if user is fixated on something interesting
      if (currentFeedback && currentGaze?.isFixated) {
        setFeedback(currentFeedback);
        setShowFeedback(true);
        setFadingOut(false);
      } else {
        // Fade out after user looks away
        if (showFeedback) {
          setFadingOut(true);
          setTimeout(() => {
            setShowFeedback(false);
            setFeedback(null);
          }, 800);
        }
      }
    }, 200);
    
    return () => clearInterval(interval);
  }, [isActive, showFeedback]);
  
  if (!isActive || !showFeedback || !feedback) {
    return null;
  }
  
  return (
    <div className={`focus-feedback ${fadingOut ? 'fading-out' : 'fading-in'}`}>
      {/* Minimal, Non-Intrusive Feedback */}
      <div className="feedback-card">
        {/* Object Info - Clean and Brief */}
        <div className="feedback-header">
          <span className="object-label">{feedback.objectInfo}</span>
          <span className="confidence-badge">
            {Math.round(feedback.confidence * 100)}%
          </span>
        </div>
        
        {/* Intelligent Insight - The Key Information */}
        <div className="feedback-insight">
          {feedback.intelligentInsight}
        </div>
        
        {/* Warnings - Only if Critical */}
        {feedback.warnings && feedback.warnings.length > 0 && (
          <div className="feedback-warnings">
            {feedback.warnings.map((warning, idx) => (
              <div key={idx} className="warning-item">
                ⚠️ {warning}
              </div>
            ))}
          </div>
        )}
        
        {/* Quick Actions - Contextual, Not Overwhelming */}
        {feedback.actionSuggestions && feedback.actionSuggestions.length > 0 && (
          <div className="feedback-actions">
            {feedback.actionSuggestions.slice(0, 3).map((action, idx) => (
              <button key={idx} className="action-btn">
                {action}
              </button>
            ))}
          </div>
        )}
        
        {/* Reasoning - Transparent but Subtle */}
        <div className="feedback-reasoning">
          💡 {feedback.reasoning}
        </div>
      </div>
      
      {/* Gaze Indicator - Shows where you're looking (optional, for debugging) */}
      {gazeData && (
        <div 
          className="gaze-indicator"
          style={{
            left: `${gazeData.gazeX * 100}%`,
            top: `${gazeData.gazeY * 100}%`,
            opacity: gazeData.isFixated ? 0.8 : 0.3
          }}
        />
      )}
      
      <style jsx>{`
        .focus-feedback {
          position: fixed;
          bottom: 20px;
          right: 20px;
          z-index: 9999;
          pointer-events: none;
        }
        
        .fading-in {
          animation: fadeIn 0.3s ease-out;
        }
        
        .fading-out {
          animation: fadeOut 0.8s ease-out;
        }
        
        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        
        @keyframes fadeOut {
          from {
            opacity: 1;
            transform: translateY(0);
          }
          to {
            opacity: 0;
            transform: translateY(20px);
          }
        }
        
        .feedback-card {
          background: rgba(20, 20, 30, 0.6);
          backdrop-filter: blur(24px);
          -webkit-backdrop-filter: blur(24px);
          border: 1px solid rgba(255, 255, 255, 0.1);
          border-radius: 24px;
          padding: 24px;
          max-width: 400px;
          box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
          pointer-events: auto;
        }
        
        .feedback-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
          padding-bottom: 12px;
          border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .object-label {
          font-size: 16px;
          font-weight: 600;
          color: #FFFFFF;
        }
        
        .confidence-badge {
          font-size: 12px;
          padding: 4px 10px;
          background: rgba(74, 144, 226, 0.2);
          border: 1px solid rgba(74, 144, 226, 0.4);
          border-radius: 12px;
          color: #4A90E2;
        }
        
        .feedback-insight {
          font-size: 14px;
          line-height: 1.6;
          color: #E0E0E0;
          margin-bottom: 12px;
        }
        
        .feedback-warnings {
          margin-bottom: 12px;
        }
        
        .warning-item {
          font-size: 13px;
          color: #FFB74D;
          background: rgba(255, 183, 77, 0.1);
          border-left: 3px solid #FFB74D;
          padding: 8px 12px;
          margin-bottom: 6px;
          border-radius: 4px;
        }
        
        .feedback-actions {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
          margin-bottom: 12px;
        }
        
        .action-btn {
          font-size: 12px;
          padding: 8px 16px;
          background: rgba(255, 255, 255, 0.05);
          border: 1px solid rgba(255, 255, 255, 0.1);
          border-radius: 16px;
          color: #60a5fa;
          cursor: pointer;
          transition: all 0.2s;
          font-weight: 500;
        }
        
        .action-btn:hover {
          background: rgba(255, 255, 255, 0.15);
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }
        
        .feedback-reasoning {
          font-size: 11px;
          color: rgba(255, 255, 255, 0.5);
          font-style: italic;
          padding-top: 8px;
          border-top: 1px solid rgba(255, 255, 255, 0.05);
        }
        
        .gaze-indicator {
          position: fixed;
          width: 12px;
          height: 12px;
          background: rgba(74, 144, 226, 0.6);
          border: 2px solid rgba(255, 255, 255, 0.8);
          border-radius: 50%;
          transform: translate(-50%, -50%);
          pointer-events: none;
          transition: opacity 0.2s;
          box-shadow: 0 0 20px rgba(74, 144, 226, 0.8);
        }
      `}</style>
    </div>
  );
};

export default FocusMode;
