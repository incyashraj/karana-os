import React, { useState } from 'react';
import { Shield, Eye, Lock, Smartphone, X, Check, AlertTriangle, Activity } from 'lucide-react';

export type UXLevel = 'BEGINNER' | 'INTERMEDIATE' | 'ADVANCED' | 'EXPERT';
export type SecurityPreset = 'PARANOID' | 'HIGH' | 'BALANCED' | 'RELAXED';
export type PrivacyZone = 'HOME' | 'WORK' | 'PUBLIC' | 'TRAVEL';

interface SettingsOverlayProps {
  isOpen: boolean;
  onClose: () => void;
  uxLevel: UXLevel;
  setUxLevel: (level: UXLevel) => void;
  securityPreset: SecurityPreset;
  setSecurityPreset: (preset: SecurityPreset) => void;
  ephemeralMode: boolean;
  setEphemeralMode: (enabled: boolean) => void;
}

export const SettingsOverlay: React.FC<SettingsOverlayProps> = ({
  isOpen, onClose, uxLevel, setUxLevel, securityPreset, setSecurityPreset, ephemeralMode, setEphemeralMode
}) => {
  const [activeTab, setActiveTab] = useState<'UX' | 'SECURITY' | 'PRIVACY' | 'SYSTEM'>('UX');

  if (!isOpen) return null;

  return (
      <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4" onClick={onClose}>
      <div className="glass-panel w-full max-w-2xl overflow-hidden flex flex-col max-h-[80vh]" onClick={(e) => e.stopPropagation()}>        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-white/10">
          <div className="flex items-center gap-2 text-white">
            <Activity size={20} className="text-cyan-400" />
            <span className="font-sans font-bold text-xl tracking-wider">System Configuration</span>
          </div>
          <button onClick={onClose} className="text-white/60 hover:text-white transition-colors">
            <X size={24} />
          </button>
        </div>

        {/* Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <div className="w-48 border-r border-white/10 p-4 flex flex-col gap-2">
            <TabButton active={activeTab === 'UX'} onClick={() => setActiveTab('UX')} icon={<Eye size={16} />} label="Experience" />
            <TabButton active={activeTab === 'SECURITY'} onClick={() => setActiveTab('SECURITY')} icon={<Shield size={16} />} label="Security" />
            <TabButton active={activeTab === 'PRIVACY'} onClick={() => setActiveTab('PRIVACY')} icon={<Lock size={16} />} label="Privacy" />
            <TabButton active={activeTab === 'SYSTEM'} onClick={() => setActiveTab('SYSTEM')} icon={<Smartphone size={16} />} label="System" />
          </div>

          {/* Main Panel */}
          <div className="flex-1 p-6 overflow-y-auto">
            
            {activeTab === 'UX' && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-bold text-white mb-1">UX Level (Phase 58)</h3>
                  <p className="text-sm text-slate-400 mb-4">Adjust interface complexity and feature visibility.</p>
                  <div className="grid grid-cols-1 gap-3">
                    {['BEGINNER', 'INTERMEDIATE', 'ADVANCED', 'EXPERT'].map((level) => (
                      <button
                        key={level}
                        onClick={() => setUxLevel(level as UXLevel)}
                        className={`flex items-center justify-between p-3 rounded-lg border transition-all ${
                          uxLevel === level 
                            ? 'bg-cyan-900/40 border-cyan-500 text-cyan-100' 
                            : 'bg-slate-800/40 border-white/5 text-slate-400 hover:bg-slate-800'
                        }`}
                      >
                        <span className="font-bold">{level}</span>
                        {uxLevel === level && <Check size={18} className="text-cyan-400" />}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'SECURITY' && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-bold text-white mb-1">Security Preset (Phase 59)</h3>
                  <p className="text-sm text-slate-400 mb-4">Manage permissions and spending limits.</p>
                  <div className="grid grid-cols-2 gap-3">
                    {['PARANOID', 'HIGH', 'BALANCED', 'RELAXED'].map((preset) => (
                      <button
                        key={preset}
                        onClick={() => setSecurityPreset(preset as SecurityPreset)}
                        className={`flex flex-col items-start p-3 rounded-lg border transition-all ${
                          securityPreset === preset 
                            ? 'bg-emerald-900/40 border-emerald-500 text-emerald-100' 
                            : 'bg-slate-800/40 border-white/5 text-slate-400 hover:bg-slate-800'
                        }`}
                      >
                        <span className="font-bold text-sm">{preset}</span>
                        <span className="text-xs opacity-60 mt-1">
                          {preset === 'PARANOID' ? 'Max security, manual approvals' : 
                           preset === 'RELAXED' ? 'Min friction, auto-approvals' : 'Standard protection'}
                        </span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'PRIVACY' && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-bold text-white mb-1">Privacy Dashboard (Phase 60)</h3>
                  <p className="text-sm text-slate-400 mb-4">Control data retention and visibility.</p>
                  
                  <div className="bg-slate-800/40 rounded-lg p-4 border border-white/5 mb-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-white font-medium">Ephemeral Mode</span>
                      <button 
                        onClick={() => setEphemeralMode(!ephemeralMode)}
                        className={`w-12 h-6 rounded-full relative transition-colors ${ephemeralMode ? 'bg-purple-500' : 'bg-slate-600'}`}
                      >
                        <div className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${ephemeralMode ? 'left-7' : 'left-1'}`} />
                      </button>
                    </div>
                    <p className="text-xs text-slate-400">
                      {ephemeralMode 
                        ? 'Active: No data is being saved. Session will be wiped on exit.' 
                        : 'Inactive: Data retention policies apply.'}
                    </p>
                  </div>

                  <div className="bg-slate-800/40 rounded-lg p-4 border border-white/5">
                    <h4 className="text-sm font-bold text-white mb-3">Data Retention</h4>
                    <div className="space-y-2">
                      <RetentionRow label="Camera Feed" value="7 Days" />
                      <RetentionRow label="Voice Logs" value="24 Hours" />
                      <RetentionRow label="Location History" value="30 Days" />
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'SYSTEM' && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-lg font-bold text-white mb-1">Interoperability (Phase 62)</h3>
                  <p className="text-sm text-slate-400 mb-4">Manage connected devices.</p>
                  
                  <div className="bg-slate-800/40 rounded-lg p-4 border border-white/5 flex items-center justify-between">
                    <div>
                      <div className="text-white font-medium">Pair New Device</div>
                      <div className="text-xs text-slate-400">Enter 6-digit code from companion app</div>
                    </div>
                    <button className="px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-sm rounded transition-colors">
                      Start Pairing
                    </button>
                  </div>
                </div>

                <div>
                  <h3 className="text-lg font-bold text-white mb-1 mt-6">System Health</h3>
                  <div className="grid grid-cols-2 gap-3 mt-2">
                    <StatusCard label="Thermal" value="34°C" status="good" />
                    <StatusCard label="Battery" value="82%" status="good" />
                    <StatusCard label="Memory" value="1.2GB / 2GB" status="warning" />
                    <StatusCard label="Compute" value="Distributed" status="good" />
                  </div>
                </div>
              </div>
            )}

          </div>
        </div>
      </div>
    </div>
  );
};

const TabButton: React.FC<{ active: boolean; onClick: () => void; icon: React.ReactNode; label: string }> = ({ active, onClick, icon, label }) => (
  <button
    onClick={onClick}
    className={`flex items-center gap-3 p-3 rounded-lg transition-all w-full text-left ${
      active ? 'bg-cyan-900/30 text-cyan-400 border border-cyan-500/30' : 'text-slate-400 hover:bg-slate-800 hover:text-white'
    }`}
  >
    {icon}
    <span className="font-medium text-sm">{label}</span>
  </button>
);

const RetentionRow: React.FC<{ label: string; value: string }> = ({ label, value }) => (
  <div className="flex justify-between items-center text-sm">
    <span className="text-slate-300">{label}</span>
    <span className="text-slate-500 font-mono">{value}</span>
  </div>
);

const StatusCard: React.FC<{ label: string; value: string; status: 'good' | 'warning' | 'critical' }> = ({ label, value, status }) => (
  <div className="bg-slate-900/50 p-3 rounded border border-white/5">
    <div className="text-xs text-slate-500 uppercase tracking-wider mb-1">{label}</div>
    <div className={`font-mono font-bold ${
      status === 'good' ? 'text-emerald-400' : status === 'warning' ? 'text-amber-400' : 'text-red-400'
    }`}>
      {value}
    </div>
  </div>
);
