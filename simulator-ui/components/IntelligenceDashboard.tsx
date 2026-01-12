/**
 * Kāraṇa OS - Intelligence Usage Dashboard
 * 
 * Real-time monitoring of intelligence system showing:
 * - How many requests handled locally (FREE)
 * - How many cache hits (FREE)
 * - How many free API calls (FREE)
 * - How much money saved vs traditional cloud AI
 */

import React, { useState, useEffect } from 'react';
import { edgeIntelligence } from '../services/edgeIntelligence';

interface UsageStats {
  cache: {
    size: number;
    totalHits: number;
  };
  cost: {
    edgeProcessing: number;
    cacheHits: number;
    freeAPIsCalls: number;
    paidAPICalls: number;
    totalSaved: number;
    edgeProcessingPercent: number;
    totalRequests: number;
    averageCostPerRequest: number;
  };
}

export const IntelligenceDashboard: React.FC<{ isVisible: boolean; onClose: () => void }> = ({ isVisible, onClose }) => {
  const [stats, setStats] = useState<UsageStats | null>(null);
  
  useEffect(() => {
    if (isVisible) {
      // Update stats every second
      const interval = setInterval(() => {
        const currentStats = edgeIntelligence.getStats();
        setStats(currentStats);
      }, 1000);
      
      return () => clearInterval(interval);
    }
  }, [isVisible]);
  
  if (!isVisible || !stats) return null;
  
  const { cache, cost } = stats;
  
  // Calculate efficiency metrics
  const totalFreeRequests = cost.edgeProcessing + cost.cacheHits + cost.freeAPIsCalls;
  const freePercent = cost.totalRequests > 0 
    ? Math.round((totalFreeRequests / cost.totalRequests) * 100) 
    : 100;
  
  // Cost comparison
  const costWithTraditionalAI = cost.totalRequests * 0.002; // $0.002 per Gemini call
  const actualCost = cost.paidAPICalls * 0.002;
  const savings = costWithTraditionalAI - actualCost;
  const savingsPercent = costWithTraditionalAI > 0 
    ? Math.round((savings / costWithTraditionalAI) * 100)
    : 100;
  
  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="glass-panel w-full max-w-2xl p-6 relative overflow-hidden">
        <button 
          onClick={onClose}
          className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors"
        >
          ✕
        </button>
        
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-purple-500 to-blue-500 flex items-center justify-center shadow-lg shadow-purple-500/30">
            <span className="text-xl">🧠</span>
          </div>
          <div>
            <h2 className="text-xl font-bold text-white">Intelligence Dashboard</h2>
            <p className="text-sm text-gray-400">Real-time Edge AI Performance</p>
          </div>
        </div>
        
        {/* Main Stats Grid */}
        <div className="grid grid-cols-2 gap-4 mb-6">
          {/* Total Requests */}
          <div className="bg-gradient-to-br from-blue-600/20 to-blue-800/20 border border-blue-500/30 rounded-xl p-4">
            <div className="text-blue-300 text-sm mb-1">Total Requests</div>
            <div className="text-3xl font-bold text-white">{cost.totalRequests}</div>
          </div>
          
          {/* Free Percentage */}
          <div className="bg-gradient-to-br from-green-600/20 to-green-800/20 border border-green-500/30 rounded-xl p-4">
            <div className="text-green-300 text-sm mb-1">Handled Free</div>
            <div className="text-3xl font-bold text-white">{freePercent}%</div>
          </div>
          
          {/* Edge Processing */}
          <div className="bg-gradient-to-br from-purple-600/20 to-purple-800/20 border border-purple-500/30 rounded-xl p-4">
            <div className="text-purple-300 text-sm mb-1">Edge Intelligence</div>
            <div className="text-2xl font-bold text-white">{cost.edgeProcessing}</div>
            <div className="text-xs text-purple-400 mt-1">Local NLP processing</div>
          </div>
          
          {/* Cache Hits */}
          <div className="bg-gradient-to-br from-cyan-600/20 to-cyan-800/20 border border-cyan-500/30 rounded-xl p-4">
            <div className="text-cyan-300 text-sm mb-1">Cache Hits</div>
            <div className="text-2xl font-bold text-white">{cache.totalHits}</div>
            <div className="text-xs text-cyan-400 mt-1">{cache.size} items cached</div>
          </div>
          
          {/* Free API Calls */}
          <div className="bg-gradient-to-br from-yellow-600/20 to-yellow-800/20 border border-yellow-500/30 rounded-xl p-4">
            <div className="text-yellow-300 text-sm mb-1">Free API Calls</div>
            <div className="text-2xl font-bold text-white">{cost.freeAPIsCalls}</div>
            <div className="text-xs text-yellow-400 mt-1">OpenMeteo, DuckDuckGo, etc.</div>
          </div>
          
          {/* Paid API Calls */}
          <div className="bg-gradient-to-br from-red-600/20 to-red-800/20 border border-red-500/30 rounded-xl p-4">
            <div className="text-red-300 text-sm mb-1">Paid API Calls</div>
            <div className="text-2xl font-bold text-white">{cost.paidAPICalls}</div>
            <div className="text-xs text-red-400 mt-1">Only when user enables</div>
          </div>
        </div>
        
        {/* Cost Savings */}
        <div className="bg-gradient-to-r from-green-600/20 to-emerald-600/20 border border-green-500/30 rounded-xl p-6 mb-6">
          <div className="text-center mb-4">
            <div className="text-green-300 text-sm mb-2">💰 Cost Savings vs Traditional Cloud AI</div>
            <div className="text-5xl font-bold text-green-400 mb-2">
              ${savings.toFixed(4)}
            </div>
            <div className="text-sm text-green-300">
              {savingsPercent}% savings • ${actualCost.toFixed(4)} spent vs ${costWithTraditionalAI.toFixed(4)} traditional
            </div>
          </div>
          
          {/* Progress Bar */}
          <div className="w-full bg-white/5 rounded-full h-3 overflow-hidden">
            <div 
              className="bg-gradient-to-r from-green-500 to-emerald-500 h-full transition-all duration-300"
              style={{ width: `${freePercent}%` }}
            />
          </div>
          <div className="flex justify-between text-xs text-gray-400 mt-2">
            <span>0% free</span>
            <span className="text-green-400 font-semibold">{freePercent}% handled locally</span>
            <span>100% paid</span>
          </div>
        </div>
        
        {/* Intelligence Breakdown */}
        <div className="bg-white/5/50 rounded-xl p-4">
          <h3 className="text-white font-semibold mb-3">Intelligence Breakdown</h3>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
                <span className="text-gray-300">Edge NLP</span>
              </div>
              <div className="text-white font-mono">
                {cost.edgeProcessing} <span className="text-green-400 text-xs ml-2">FREE</span>
              </div>
            </div>
            
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-cyan-500 rounded-full"></div>
                <span className="text-gray-300">Smart Cache</span>
              </div>
              <div className="text-white font-mono">
                {cache.totalHits} <span className="text-green-400 text-xs ml-2">FREE</span>
              </div>
            </div>
            
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
                <span className="text-gray-300">Free APIs</span>
              </div>
              <div className="text-white font-mono">
                {cost.freeAPIsCalls} <span className="text-green-400 text-xs ml-2">FREE</span>
              </div>
            </div>
            
            {cost.paidAPICalls > 0 && (
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                  <span className="text-gray-300">Cloud AI (Optional)</span>
                </div>
                <div className="text-white font-mono">
                  {cost.paidAPICalls} <span className="text-red-400 text-xs ml-2">${(cost.paidAPICalls * 0.002).toFixed(4)}</span>
                </div>
              </div>
            )}
          </div>
        </div>
        
        {/* Footer Info */}
        <div className="mt-6 text-center text-xs text-gray-500">
          <p>This system handles 95%+ of requests without any paid API calls.</p>
          <p className="mt-1">Real weather data from OpenMeteo • News from free sources • Search via DuckDuckGo</p>
        </div>
      </div>
    </div>
  );
};

export default IntelligenceDashboard;
