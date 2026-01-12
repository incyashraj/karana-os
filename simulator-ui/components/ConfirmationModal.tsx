import React from 'react';
import { X, AlertTriangle, Clock, Zap, HardDrive, Shield } from 'lucide-react';

export interface ActionStep {
  operation: string;
  layer: string;
  params: any;
  duration?: number;
  resources?: {
    battery?: number;
    storage?: number;
    network?: boolean;
    camera?: boolean;
  };
  dependencies?: string[];
}

export interface Risk {
  type: 'FINANCIAL' | 'BATTERY' | 'STORAGE' | 'TIME' | 'SECURITY' | 'DATA';
  severity: 'LOW' | 'MEDIUM' | 'HIGH';
  message: string;
}

export interface ConfirmationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  onCancel: () => void;
  title?: string;
  message: string;
  steps?: ActionStep[];
  totalDuration?: number;
  resources?: {
    battery?: number;
    storage?: number;
    network?: boolean;
    camera?: boolean;
  };
  risks?: Risk[];
}

const getRiskIcon = (type: Risk['type']) => {
  switch (type) {
    case 'FINANCIAL': return '💰';
    case 'BATTERY': return '🔋';
    case 'STORAGE': return '💾';
    case 'TIME': return '⏱️';
    case 'SECURITY': return '🔓';
    case 'DATA': return '📊';
    default: return '⚠️';
  }
};

const getRiskColor = (severity: Risk['severity']) => {
  switch (severity) {
    case 'LOW': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    case 'MEDIUM': return 'text-orange-600 bg-orange-50 border-orange-200';
    case 'HIGH': return 'text-red-600 bg-red-50 border-red-200';
  }
};

const formatDuration = (ms: number) => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
};

export const ConfirmationModal: React.FC<ConfirmationModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
  onCancel,
  title = 'Confirm Action',
  message,
  steps = [],
  totalDuration,
  resources,
  risks = [],
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-white rounded-xl shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-indigo-50 to-purple-50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-indigo-100 flex items-center justify-center">
              <AlertTriangle className="w-5 h-5 text-indigo-600" />
            </div>
            <h2 className="text-xl font-bold text-gray-900">{title}</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/50 rounded-lg transition-colors"
            aria-label="Close"
          >
            <X className="w-5 h-5 text-gray-500" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {/* Message */}
          <p className="text-gray-700 leading-relaxed mb-6">{message}</p>

          {/* Action Plan */}
          {steps.length > 0 && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-900 mb-3 flex items-center gap-2">
                <span className="w-6 h-6 rounded-full bg-indigo-100 text-indigo-600 text-xs flex items-center justify-center font-bold">
                  {steps.length}
                </span>
                Action Plan
              </h3>
              <div className="space-y-2">
                {steps.map((step, index) => (
                  <div
                    key={index}
                    className="bg-gray-50 rounded-lg p-3 border border-gray-200"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-xs font-mono text-gray-500">
                            Step {index + 1}
                          </span>
                          <span className="text-sm font-semibold text-gray-900">
                            {step.operation}
                          </span>
                        </div>
                        <div className="text-xs text-gray-600">
                          Layer: {step.layer}
                        </div>
                        {step.dependencies && step.dependencies.length > 0 && (
                          <div className="text-xs text-gray-500 mt-1">
                            ↳ Depends on: Step {step.dependencies.join(', Step ')}
                          </div>
                        )}
                      </div>
                      {step.duration && (
                        <div className="flex items-center gap-1 text-xs text-gray-600">
                          <Clock className="w-3 h-3" />
                          {formatDuration(step.duration)}
                        </div>
                      )}
                    </div>
                    {step.resources && (
                      <div className="flex items-center gap-3 mt-2 text-xs text-gray-600">
                        {step.resources.battery && (
                          <span className="flex items-center gap-1">
                            <Zap className="w-3 h-3" />
                            {step.resources.battery}mAh
                          </span>
                        )}
                        {step.resources.storage && (
                          <span className="flex items-center gap-1">
                            <HardDrive className="w-3 h-3" />
                            {step.resources.storage}MB
                          </span>
                        )}
                        {step.resources.network && (
                          <span className="text-blue-600">🌐 Network</span>
                        )}
                        {step.resources.camera && (
                          <span className="text-purple-600">📷 Camera</span>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Resource Summary */}
          {(totalDuration || resources) && (
            <div className="mb-6 bg-blue-50 rounded-lg p-4 border border-blue-200">
              <h3 className="text-sm font-semibold text-blue-900 mb-2">
                Resource Requirements
              </h3>
              <div className="grid grid-cols-2 gap-3">
                {totalDuration && (
                  <div className="flex items-center gap-2 text-sm text-blue-800">
                    <Clock className="w-4 h-4" />
                    <span>Total Time: {formatDuration(totalDuration)}</span>
                  </div>
                )}
                {resources?.battery && (
                  <div className="flex items-center gap-2 text-sm text-blue-800">
                    <Zap className="w-4 h-4" />
                    <span>Battery: {resources.battery}mAh</span>
                  </div>
                )}
                {resources?.storage && (
                  <div className="flex items-center gap-2 text-sm text-blue-800">
                    <HardDrive className="w-4 h-4" />
                    <span>Storage: {resources.storage}MB</span>
                  </div>
                )}
                {resources?.network && (
                  <div className="flex items-center gap-2 text-sm text-blue-800">
                    <span>🌐 Network Required</span>
                  </div>
                )}
                {resources?.camera && (
                  <div className="flex items-center gap-2 text-sm text-blue-800">
                    <span>📷 Camera Required</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Risks */}
          {risks.length > 0 && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-900 mb-3 flex items-center gap-2">
                <AlertTriangle className="w-4 h-4 text-orange-600" />
                Warnings & Risks
              </h3>
              <div className="space-y-2">
                {risks.map((risk, index) => (
                  <div
                    key={index}
                    className={`rounded-lg p-3 border ${getRiskColor(risk.severity)}`}
                  >
                    <div className="flex items-start gap-2">
                      <span className="text-lg">{getRiskIcon(risk.type)}</span>
                      <div className="flex-1">
                        <div className="font-semibold text-sm mb-1">
                          {risk.type} - {risk.severity}
                        </div>
                        <div className="text-sm">{risk.message}</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 bg-gray-50">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="px-6 py-2 text-sm font-medium text-white bg-gradient-to-r from-indigo-600 to-purple-600 rounded-lg hover:from-indigo-700 hover:to-purple-700 transition-all shadow-md hover:shadow-lg"
          >
            Confirm & Execute
          </button>
        </div>
      </div>
    </div>
  );
};
