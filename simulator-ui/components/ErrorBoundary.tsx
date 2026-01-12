import React, { Component, ReactNode } from 'react';
import { AlertTriangle } from 'lucide-react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Kāraṇa OS Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="w-screen h-screen bg-black flex items-center justify-center p-4">
          <div className="glass-panel p-8 rounded-xl max-w-md w-full border-red-500/50">
            <div className="flex flex-col items-center text-center gap-4">
              <div className="w-16 h-16 rounded-full bg-red-900/30 border border-red-500 flex items-center justify-center">
                <AlertTriangle className="text-red-400" size={32} />
              </div>
              
              <div>
                <h2 className="text-2xl font-bold text-white mb-2">System Error</h2>
                <p className="text-sm text-slate-400 mb-4">
                  Kāraṇa OS encountered an unexpected error. The system will attempt to recover.
                </p>
                
                {this.state.error && (
                  <details className="text-left">
                    <summary className="text-xs text-red-400 cursor-pointer hover:text-red-300 mb-2">
                      Technical Details
                    </summary>
                    <pre className="text-xs text-slate-500 bg-black/40 p-3 rounded overflow-auto max-h-32">
                      {this.state.error.toString()}
                    </pre>
                  </details>
                )}
              </div>

              <button
                onClick={() => window.location.reload()}
                className="w-full py-3 bg-red-600 hover:bg-red-500 text-white font-bold rounded-lg transition-colors"
              >
                Restart System
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
