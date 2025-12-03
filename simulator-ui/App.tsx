import React, { useState, useRef, useEffect, useCallback } from 'react';
import CameraFeed, { CameraFeedHandle } from './components/CameraFeed';
import { HUD } from './components/HUD';
import { ChatInterface } from './components/ChatInterface';
import ARWorkspace, { ARWorkspaceHandle } from './components/ARWorkspace';
import { AppMode, AnalysisResult, ChatMessage, WalletState, Transaction, OSTimer, OSNotification } from './types';
import { karanaApi, WalletInfo, VisionAnalysisResponse, OracleIntentResponse } from './services/karanaService';
import { oracleAI, OracleResponse } from './services/oracleAI';
import { Eye, MessageSquare, Navigation2, X, AlertTriangle, Scan, Wallet, ShieldCheck, RefreshCw, Plus, Monitor, Bell, Clock } from 'lucide-react';

// Default wallet state before connecting to backend
const INITIAL_WALLET: WalletState = {
  balance: 0,
  did: 'Not Connected',
  transactions: []
};

const App: React.FC = () => {
  const [mode, setMode] = useState<AppMode>(AppMode.IDLE);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [wallet, setWallet] = useState<WalletState>(INITIAL_WALLET);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingTx, setPendingTx] = useState<Transaction | null>(null);
  const [backendConnected, setBackendConnected] = useState(false);
  const [showRecoveryPhrase, setShowRecoveryPhrase] = useState<string[] | null>(null);
  
  // Timer and Notification State
  const [timers, setTimers] = useState<OSTimer[]>([]);
  const [notifications, setNotifications] = useState<OSNotification[]>([]);
  const [showTimers, setShowTimers] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);

  const cameraRef = useRef<CameraFeedHandle>(null);
  const arWorkspaceRef = useRef<ARWorkspaceHandle>(null);

  // Connect to backend on mount
  useEffect(() => {
    const checkConnection = async () => {
      const isHealthy = await karanaApi.healthCheck();
      setBackendConnected(isHealthy);
      
      if (isHealthy) {
        // Try to get existing wallet info
        try {
          const info = await karanaApi.getWalletInfo();
          setWallet({
            balance: info.balance,
            did: info.did,
            transactions: []
          });
          // Load transaction history
          try {
            const txs = await karanaApi.getTransactions();
            setWallet(prev => ({ ...prev, transactions: txs.map(tx => ({
              id: tx.id,
              type: tx.tx_type,
              amount: tx.amount,
              recipient: tx.recipient,
              timestamp: tx.timestamp * 1000,
              status: tx.status,
              signature: tx.signature
            })) }));
          } catch {}
        } catch {
          // No wallet exists yet
        }

        // Connect WebSocket for real-time updates
        try {
          await karanaApi.connectWebSocket();
          karanaApi.subscribe('transactions');
          karanaApi.subscribe('wallet');
          
          karanaApi.onEvent('TransactionConfirmed', (data) => {
            console.log('[WS] Transaction confirmed:', data);
          });
          
          karanaApi.onEvent('WalletUpdate', (data) => {
            setWallet(prev => ({ ...prev, balance: data.balance }));
          });
        } catch (e) {
          console.warn('WebSocket connection failed, will use polling');
        }
      }
    };

    checkConnection();
    
    // Poll connection every 5 seconds if not connected
    const interval = setInterval(async () => {
      if (!backendConnected) {
        await checkConnection();
      }
    }, 5000);

    return () => {
      clearInterval(interval);
      karanaApi.disconnectWebSocket();
    };
  }, [backendConnected]);

  // Timer tick effect - update every second
  useEffect(() => {
    const timerInterval = setInterval(() => {
      setTimers(prev => prev.map(timer => {
        if (timer.state !== 'RUNNING') return timer;
        
        const newRemaining = timer.remainingMs - 1000;
        
        if (newRemaining <= 0) {
          // Timer completed - add notification
          const notification: OSNotification = {
            id: `notif_${Date.now()}`,
            title: '⏱️ Timer Complete',
            body: timer.label || 'Your timer has finished!',
            priority: 'HIGH',
            category: 'TIMER',
            timestamp: Date.now(),
            read: false
          };
          setNotifications(prev => [notification, ...prev]);
          
          // For recurring timers, reset
          if (timer.type === 'RECURRING' && timer.recurring) {
            return {
              ...timer,
              remainingMs: timer.durationMs,
              completesAt: Date.now() + timer.durationMs
            };
          }
          
          return { ...timer, remainingMs: 0, state: 'COMPLETED' as const };
        }
        
        return { ...timer, remainingMs: newRemaining };
      }));
    }, 1000);

    return () => clearInterval(timerInterval);
  }, []);

  // Timer management functions
  const createTimer = useCallback((durationMs: number, label: string, type: 'COUNTDOWN' | 'STOPWATCH' | 'RECURRING' = 'COUNTDOWN') => {
    const timer: OSTimer = {
      id: `timer_${Date.now()}`,
      type,
      label: label || `Timer (${Math.round(durationMs / 60000)} min)`,
      durationMs,
      remainingMs: durationMs,
      state: 'RUNNING',
      createdAt: Date.now(),
      completesAt: Date.now() + durationMs
    };
    setTimers(prev => [...prev, timer]);
    return timer;
  }, []);

  const cancelTimer = useCallback((timerId: string) => {
    setTimers(prev => prev.map(t => 
      t.id === timerId ? { ...t, state: 'CANCELLED' as const } : t
    ));
  }, []);

  const pauseTimer = useCallback((timerId: string) => {
    setTimers(prev => prev.map(t => 
      t.id === timerId && t.state === 'RUNNING' ? { ...t, state: 'PAUSED' as const } : t
    ));
  }, []);

  const resumeTimer = useCallback((timerId: string) => {
    setTimers(prev => prev.map(t => 
      t.id === timerId && t.state === 'PAUSED' 
        ? { ...t, state: 'RUNNING' as const, completesAt: Date.now() + t.remainingMs } 
        : t
    ));
  }, []);

  // Notification management
  const addNotification = useCallback((title: string, body: string, priority: 'LOW' | 'NORMAL' | 'HIGH' | 'URGENT' = 'NORMAL', category: 'SYSTEM' | 'TIMER' | 'TRANSACTION' | 'GOVERNANCE' | 'SOCIAL' | 'GENERAL' = 'GENERAL') => {
    const notification: OSNotification = {
      id: `notif_${Date.now()}`,
      title,
      body,
      priority,
      category,
      timestamp: Date.now(),
      read: false
    };
    setNotifications(prev => [notification, ...prev]);
    return notification;
  }, []);

  const dismissNotification = useCallback((notifId: string) => {
    setNotifications(prev => prev.filter(n => n.id !== notifId));
  }, []);

  const markNotificationRead = useCallback((notifId: string) => {
    setNotifications(prev => prev.map(n => 
      n.id === notifId ? { ...n, read: true } : n
    ));
  }, []);

  const clearAllNotifications = useCallback(() => {
    setNotifications([]);
  }, []);

  // Create new wallet
  const handleCreateWallet = async () => {
    try {
      setIsProcessing(true);
      const result = await karanaApi.createWallet();
      setWallet({
        balance: 1000, // Initial balance from backend
        did: result.did,
        transactions: []
      });
      setShowRecoveryPhrase(result.recovery_phrase);
    } catch (err: any) {
      setError(err.message || 'Failed to create wallet');
    } finally {
      setIsProcessing(false);
    }
  };

  // Intent Handlers
  const handleAnalyze = async () => {
    if (isProcessing) return;
    setIsProcessing(true);
    setMode(AppMode.ANALYZING);
    setAnalysis(null);
    setError(null);

    try {
      await new Promise(resolve => setTimeout(resolve, 800)); // Scan animation delay
      const frame = cameraRef.current?.captureFrame();
      if (!frame) throw new Error("Could not capture camera frame");

      // Use real backend AI vision
      const result = await karanaApi.analyzeVision(frame);
      
      // Map to frontend type
      setAnalysis({
        detectedObject: result.detected_object,
        category: result.category,
        description: result.description,
        confidence: result.confidence,
        relatedTags: result.related_tags,
      });
    } catch (err: any) {
      setError(err.message || "Vision Layer: Analysis Failed");
      setMode(AppMode.IDLE);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleOracleInput = async (text: string) => {
    // 1. User Message
    const userMsg: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      text,
      timestamp: Date.now()
    };
    setChatMessages(prev => [...prev, userMsg]);
    setIsProcessing(true);

    try {
      // Build context for Oracle AI
      const context: any = {
        walletBalance: wallet.balance,
        walletDid: wallet.did,
      };
      
      if (mode === AppMode.ANALYZING && analysis) {
        context.visionObject = analysis.detectedObject;
        context.visionDescription = analysis.description;
      }

      // 2. Process with OracleAI (intelligent AI responses)
      const response = await oracleAI.process(text, context);
      
      // 3. Oracle Response (System Message)
      const oracleMsg: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        text: response.message,
        timestamp: Date.now(),
        metadata: response.data ? {
          type: response.intent.type,
          ...response.data
        } : undefined
      };
      setChatMessages(prev => [...prev, oracleMsg]);

      // 4. Execute actions based on intent
      await executeOracleAction(response);

    } catch (err: any) {
      console.error('Oracle error:', err);
      setError(err.message || "Oracle Layer Unreachable");
    } finally {
      setIsProcessing(false);
    }
  };

  // Execute Oracle actions - bridge between AI and system
  const executeOracleAction = useCallback(async (response: OracleResponse) => {
    const { intent, data } = response;
    const intentType = intent.type;
    
    // Helper to open AR app
    const openARApp = (appType: string, options?: { url?: string }) => {
      setMode(AppMode.AR_WORKSPACE);
      setTimeout(() => {
        arWorkspaceRef.current?.spawnApp(appType as any, options);
      }, 100);
    };
    
    // Handle all intent types
    switch (intentType) {
      // ========================
      // BLOCKCHAIN / WALLET
      // ========================
      case 'TRANSFER':
        if (data?.amount && data?.recipient) {
          setPendingTx({
            id: `tx_${Date.now()}`,
            type: 'TRANSFER',
            amount: data.amount,
            recipient: data.recipient,
            timestamp: Date.now(),
            status: 'PENDING'
          });
        }
        break;
        
      case 'CHECK_BALANCE':
      case 'TRANSACTION_HISTORY':
      case 'STAKE_TOKENS':
      case 'UNSTAKE':
      case 'SHOW_DID':
      case 'BACKUP_WALLET':
        setMode(AppMode.WALLET);
        break;
        
      case 'CREATE_WALLET':
        if (!wallet.did || wallet.did === 'Not Connected') {
          await handleCreateWallet();
        }
        break;
        
      // ========================
      // GOVERNANCE
      // ========================
      case 'CREATE_PROPOSAL':
      case 'VOTE_PROPOSAL':
      case 'LIST_PROPOSALS':
      case 'PROPOSAL_STATUS':
        // Open governance view (for now, show in chat)
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: '🏛️ Governance features coming soon. Use the Wallet panel to manage tokens.',
          timestamp: Date.now()
        }]);
        break;
        
      // ========================
      // VISION / CAMERA
      // ========================
      case 'ANALYZE_VISION':
      case 'IDENTIFY_OBJECT':
      case 'EXPLAIN_SCENE':
      case 'CAPTURE_PHOTO':
        handleAnalyze();
        break;
        
      case 'SCAN_QR':
        setMode(AppMode.ANALYZING);
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: '📷 QR Scanner activated. Point camera at QR code.',
          timestamp: Date.now()
        }]);
        break;
        
      // ========================
      // TIMERS / REMINDERS
      // ========================
      case 'SET_TIMER':
      case 'SET_ALARM':
      case 'SET_REMINDER':
        if (data?.durationMs) {
          const timer = createTimer(data.durationMs, data.label || data.duration || 'Timer');
          setShowTimers(true);
          setChatMessages(prev => [...prev, {
            id: Date.now().toString(),
            role: 'system',
            text: `⏱️ Timer "${timer.label}" started! Will complete at ${new Date(timer.completesAt!).toLocaleTimeString()}`,
            timestamp: Date.now()
          }]);
        }
        break;
        
      case 'START_STOPWATCH':
        const stopwatch = createTimer(0, 'Stopwatch', 'STOPWATCH');
        setShowTimers(true);
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: `⏱️ Stopwatch started!`,
          timestamp: Date.now()
        }]);
        break;
        
      case 'STOP_STOPWATCH':
      case 'LAP_STOPWATCH':
        // Find active stopwatch and pause
        const activeStopwatch = timers.find(t => t.type === 'STOPWATCH' && t.state === 'RUNNING');
        if (activeStopwatch) {
          pauseTimer(activeStopwatch.id);
        }
        break;
        
      case 'LIST_TIMERS':
        setShowTimers(true);
        const activeTimers = timers.filter(t => t.state === 'RUNNING' || t.state === 'PAUSED');
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: activeTimers.length > 0 
            ? `📋 Active Timers:\n${activeTimers.map(t => `• ${t.label}: ${Math.ceil(t.remainingMs / 1000)}s remaining`).join('\n')}`
            : '📋 No active timers',
          timestamp: Date.now()
        }]);
        break;
        
      case 'CANCEL_TIMER':
        if (data?.timerId) {
          cancelTimer(data.timerId);
        } else {
          // Cancel all timers
          timers.forEach(t => {
            if (t.state === 'RUNNING') cancelTimer(t.id);
          });
        }
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: '⏹️ Timer(s) cancelled',
          timestamp: Date.now()
        }]);
        break;
        
      // ========================
      // NOTIFICATIONS
      // ========================
      case 'SHOW_NOTIFICATIONS':
        setShowNotifications(true);
        const unread = notifications.filter(n => !n.read);
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: unread.length > 0 
            ? `🔔 You have ${unread.length} unread notification(s):\n${unread.slice(0, 5).map(n => `• ${n.title}: ${n.body}`).join('\n')}`
            : '🔔 No unread notifications',
          timestamp: Date.now()
        }]);
        break;
        
      case 'CLEAR_NOTIFICATIONS':
        clearAllNotifications();
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: '🧹 All notifications cleared',
          timestamp: Date.now()
        }]);
        break;
        
      case 'DISMISS_NOTIFICATION':
        if (data?.notificationId) {
          dismissNotification(data.notificationId);
        }
        break;
        
      // ========================
      // AR APPLICATIONS
      // ========================
      case 'PLAY_VIDEO':
      case 'SEARCH_VIDEO':
        openARApp('video', { url: data?.url });
        break;

      case 'OPEN_BROWSER':
      case 'SEARCH_WEB':
      case 'OPEN_URL':
        openARApp('browser', { url: data?.url });
        break;
        
      case 'OPEN_TERMINAL':
      case 'RUN_COMMAND':
        openARApp('terminal');
        break;
        
      case 'CREATE_NOTE':
      case 'OPEN_NOTES':
      case 'SAVE_NOTE':
      case 'SEARCH_NOTES':
        openARApp('notes');
        break;
        
      case 'PLAY_MUSIC':
      case 'OPEN_MUSIC':
      case 'PAUSE_MUSIC':
      case 'NEXT_TRACK':
        openARApp('music');
        break;
        
      case 'OPEN_CALENDAR':
      case 'CREATE_EVENT':
      case 'LIST_EVENTS':
        openARApp('calendar');
        break;
        
      case 'OPEN_MAIL':
      case 'COMPOSE_MAIL':
      case 'CHECK_MAIL':
        openARApp('mail');
        break;
        
      case 'OPEN_GALLERY':
      case 'SHOW_PHOTO':
        openARApp('image');
        break;
        
      case 'CLOSE_APP':
        if (data?.appType) {
          // Close specific app type
          const windows = arWorkspaceRef.current?.getOpenWindows() || [];
          const targetWindow = windows.find(w => w.type === data.appType);
          if (targetWindow) {
            arWorkspaceRef.current?.closeApp(targetWindow.id);
          }
        }
        break;
        
      case 'CLOSE_ALL_APPS':
        arWorkspaceRef.current?.closeAllApps();
        setMode(AppMode.IDLE);
        break;
        
      case 'ARRANGE_WINDOWS':
        // Trigger layout in AR workspace
        break;
        
      // ========================
      // SYSTEM CONTROL
      // ========================
      case 'SYSTEM_STATUS':
      case 'BATTERY_STATUS':
      case 'NETWORK_STATUS':
        setChatMessages(prev => [...prev, {
          id: Date.now().toString(),
          role: 'system',
          text: `📊 System Status:\n• Backend: ${backendConnected ? '✅ Connected' : '❌ Disconnected'}\n• Wallet: ${wallet.did !== 'Not Connected' ? '✅ Active' : '⚠️ Not Created'}\n• Balance: ${wallet.balance} KARA`,
          timestamp: Date.now()
        }]);
        break;
        
      case 'ADJUST_BRIGHTNESS':
      case 'TOGGLE_NIGHT_MODE':
      case 'PRIVACY_MODE':
      case 'POWER_PROFILE':
        // System settings - would interact with real hardware
        break;
        
      // ========================
      // NAVIGATION
      // ========================
      case 'NAVIGATE_TO':
      case 'SHOW_DIRECTIONS':
      case 'NEARBY_PLACES':
      case 'SHARE_LOCATION':
        setMode(AppMode.NAVIGATION);
        break;
        
      // ========================
      // HELP & INFO
      // ========================
      case 'HELP':
      case 'ABOUT_KARANA':
      case 'SETTINGS':
        // Already handled in message response
        break;
        
      // ========================
      // GENERAL / CONVERSATION
      // ========================
      case 'ANSWER_QUESTION':
      case 'EXPLAIN':
      case 'TRANSLATE':
      case 'WEB_SEARCH':
      case 'CONVERSATION':
      case 'CLARIFY':
        // Response already shown in chat
        break;
        
      case 'CONFIRM':
        // Confirmation was handled
        break;
        
      case 'CANCEL':
        // Cancelled, clear any pending states
        setPendingTx(null);
        break;
    }
  }, [handleAnalyze, handleCreateWallet, backendConnected, wallet, createTimer, cancelTimer, pauseTimer, timers, notifications, clearAllNotifications, dismissNotification]);

  const confirmTransaction = async () => {
    if (!pendingTx) return;
    
    setIsProcessing(true);
    try {
      // Real Ed25519 signing via backend
      const signedTx = await karanaApi.signTransaction(
        'TRANSFER',
        pendingTx.recipient,
        pendingTx.amount
      );
      
      // Update wallet state
      setWallet(prev => ({
        ...prev,
        balance: prev.balance - pendingTx.amount,
        transactions: [{
          id: signedTx.tx_hash,
          type: 'TRANSFER',
          amount: pendingTx.amount,
          recipient: pendingTx.recipient,
          timestamp: signedTx.timestamp * 1000,
          status: 'CONFIRMED',
          signature: signedTx.signature
        }, ...prev.transactions]
      }));
      
      setChatMessages(prev => [...prev, {
        id: Date.now().toString(),
        role: 'system',
        text: `??? Transaction Signed with Ed25519 (${signedTx.signature.slice(0, 16)}...). Hash: ${signedTx.tx_hash.slice(0, 12)}...`,
        timestamp: Date.now()
      }]);

      setPendingTx(null);
    } catch (err: any) {
      setError(err.message || 'Transaction signing failed');
    } finally {
      setIsProcessing(false);
    }
  };

  const toggleMode = (targetMode: AppMode) => {
    if (mode === targetMode) {
      setMode(AppMode.IDLE);
    } else {
      setMode(targetMode);
      if (targetMode === AppMode.ANALYZING) {
        handleAnalyze();
      }
    }
  };

  return (
    <div className="relative w-screen h-screen bg-black text-white overflow-hidden font-sans">
      <CameraFeed active={true} ref={cameraRef} />

      {/* Connection Status Banner */}
      {!backendConnected && (
        <div className="absolute top-0 left-0 right-0 z-50 bg-red-900/80 text-red-100 text-center py-2 text-sm flex items-center justify-center gap-2">
          <RefreshCw className="animate-spin" size={14} />
          <span>Connecting to K??ra???a OS Backend...</span>
          <span className="text-xs opacity-60">Make sure cargo run --bin karana_api_server is running</span>
        </div>
      )}

      <HUD mode={mode} wallet={wallet}>
        
        {/* === ANALYSIS MODE === */}
        {mode === AppMode.ANALYZING && (
          <div className="relative z-20 w-full max-w-lg mx-auto flex flex-col items-center">
            {isProcessing && !analysis && (
              <div className="flex flex-col items-center gap-4">
                <div className="relative w-24 h-24">
                  <div className="absolute inset-0 border-4 border-amber-400/30 rounded-full animate-ping"></div>
                  <div className="absolute inset-0 border-4 border-t-amber-400 rounded-full animate-spin"></div>
                  <Scan className="absolute inset-0 m-auto text-amber-400 animate-pulse" size={32} />
                </div>
                <div className="hud-font text-amber-400 tracking-widest text-lg animate-pulse">VISION LAYER ANALYZING...</div>
              </div>
            )}

            {analysis && (
              <div className="glass-panel p-6 rounded-xl border-amber-500/50 text-left animate-in fade-in slide-in-from-bottom-8 duration-500 w-full">
                <div className="flex justify-between items-start mb-4 border-b border-amber-500/30 pb-2">
                   <div>
                     <h2 className="text-2xl font-bold text-amber-400 uppercase tracking-widest">{analysis.detectedObject}</h2>
                     <span className="text-xs text-amber-200/70 font-mono uppercase">{analysis.category}</span>
                   </div>
                   <div className="text-right">
                     <div className="text-3xl font-mono text-amber-400">{analysis.confidence.toFixed(0)}%</div>
                     <div className="text-[10px] text-amber-300">CONFIDENCE</div>
                   </div>
                </div>
                <p className="text-amber-100 mb-4 leading-relaxed font-light">{analysis.description}</p>
                <div className="flex flex-wrap gap-2">
                  {analysis.relatedTags.map(tag => (
                    <span key={tag} className="px-2 py-1 bg-amber-900/40 border border-amber-500/30 rounded text-xs text-amber-300 font-mono">#{tag}</span>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* === ORACLE MODE === */}
        {mode === AppMode.ORACLE && (
          <div className="relative z-20 w-full flex justify-center items-center">
            <ChatInterface 
              messages={chatMessages} 
              onSendMessage={handleOracleInput}
              isProcessing={isProcessing}
            />
          </div>
        )}

        {/* === WALLET MODE === */}
        {mode === AppMode.WALLET && (
          <div className="glass-panel p-6 w-full max-w-md rounded-xl border-blue-400/30 animate-in fade-in zoom-in-95">
             <div className="flex items-center gap-3 mb-6 border-b border-blue-400/20 pb-4">
                <ShieldCheck className="text-blue-400" size={32} />
                <div>
                  <h2 className="text-xl font-bold text-blue-100">SOVEREIGN IDENTITY</h2>
                  <p className="text-xs font-mono text-blue-300 truncate max-w-[250px]">{wallet.did}</p>
                </div>
             </div>
             
             {wallet.did === 'Not Connected' ? (
               <div className="text-center py-8">
                 <p className="text-blue-300 mb-4">No wallet connected. Create one to get started.</p>
                 <button 
                   onClick={handleCreateWallet}
                   disabled={isProcessing || !backendConnected}
                   className="px-6 py-3 bg-blue-600 hover:bg-blue-500 rounded-lg font-bold flex items-center gap-2 mx-auto disabled:opacity-50"
                 >
                   <Plus size={20} />
                   Create Wallet
                 </button>
               </div>
             ) : (
               <div className="space-y-4">
                  <div className="bg-blue-900/20 p-4 rounded-lg border border-blue-500/20 text-center">
                     <div className="text-xs text-blue-300 mb-1">TOTAL BALANCE</div>
                     <div className="text-4xl font-mono font-bold text-blue-400">{wallet.balance.toLocaleString()} KARA</div>
                  </div>

                  <div className="space-y-2">
                     <h3 className="text-sm font-bold text-blue-200 uppercase tracking-wider">Recent Activity</h3>
                     {wallet.transactions.length === 0 ? (
                       <p className="text-sm text-gray-500 text-center py-4">No transactions yet</p>
                     ) : (
                       wallet.transactions.slice(0, 5).map(tx => (
                         <div key={tx.id} className="flex justify-between items-center p-3 bg-black/40 rounded border border-white/5">
                            <div className="flex items-center gap-3">
                               <div className={`p-2 rounded-full ${tx.type === 'TRANSFER' ? 'bg-red-500/20 text-red-400' : 'bg-green-500/20 text-green-400'}`}>
                                 {tx.type === 'TRANSFER' ? <Navigation2 size={12} className="rotate-45" /> : <Wallet size={12} />}
                               </div>
                               <div className="flex flex-col">
                                  <span className="text-sm font-bold text-gray-200">{tx.type}</span>
                                  <span className="text-xs text-gray-500 font-mono">To: {tx.recipient}</span>
                               </div>
                            </div>
                            <div className="text-right">
                               <div className="text-sm font-bold text-white">-{tx.amount}</div>
                               <div className={`text-[10px] ${tx.status === 'CONFIRMED' ? 'text-green-500' : 'text-yellow-500'}`}>{tx.status}</div>
                            </div>
                         </div>
                       ))
                     )}
                  </div>
               </div>
             )}
          </div>
        )}

        {/* === RECOVERY PHRASE MODAL === */}
        {showRecoveryPhrase && (
          <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm">
            <div className="glass-panel p-6 max-w-md w-full rounded-2xl border-yellow-500/50">
               <div className="flex flex-col items-center text-center gap-4">
                  <div className="w-16 h-16 rounded-full bg-yellow-900/30 border border-yellow-500 text-yellow-400 flex items-center justify-center">
                     <AlertTriangle size={32} />
                  </div>
                  <h3 className="text-xl font-bold text-white">BACKUP YOUR RECOVERY PHRASE</h3>
                  <p className="text-sm text-gray-400">Write these 24 words down and store them safely. This is the ONLY way to recover your wallet.</p>
                  <div className="w-full bg-black/60 p-4 rounded border border-yellow-500/30 grid grid-cols-3 gap-2 text-left font-mono text-sm">
                    {showRecoveryPhrase.map((word, i) => (
                      <div key={i} className="text-yellow-300">
                        <span className="text-yellow-600/70 mr-1">{i+1}.</span>{word}
                      </div>
                    ))}
                  </div>
                  <button 
                    onClick={() => setShowRecoveryPhrase(null)} 
                    className="w-full py-3 rounded-lg bg-yellow-600 hover:bg-yellow-500 text-black font-bold"
                  >
                    I've Backed Up My Phrase
                  </button>
               </div>
            </div>
          </div>
        )}

        {/* === TRANSACTION SIGNING MODAL === */}
        {pendingTx && (
          <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm">
            <div className="glass-panel p-6 max-w-sm w-full rounded-2xl border-emerald-500/50 shadow-[0_0_50px_rgba(16,185,129,0.2)] animate-in zoom-in-95">
               <div className="flex flex-col items-center text-center gap-4">
                  <div className="w-16 h-16 rounded-full bg-emerald-900/30 border border-emerald-500 text-emerald-400 flex items-center justify-center animate-pulse">
                     <ShieldCheck size={32} />
                  </div>
                  <h3 className="text-xl font-bold text-white">SIGN TRANSACTION</h3>
                  <div className="w-full bg-emerald-900/10 p-4 rounded border border-emerald-500/20 font-mono text-sm space-y-2">
                     <div className="flex justify-between"><span>ACTION:</span> <span className="text-emerald-300">TRANSFER</span></div>
                     <div className="flex justify-between"><span>TO:</span> <span className="text-emerald-300">{pendingTx.recipient}</span></div>
                     <div className="flex justify-between border-t border-emerald-500/20 pt-2 mt-2"><span>AMOUNT:</span> <span className="text-xl font-bold text-emerald-400">{pendingTx.amount} KARA</span></div>
                  </div>
                  <div className="flex gap-3 w-full mt-2">
                     <button onClick={() => setPendingTx(null)} className="flex-1 py-3 rounded-lg border border-red-500/50 text-red-400 hover:bg-red-900/20">CANCEL</button>
                     <button 
                       onClick={confirmTransaction} 
                       disabled={isProcessing}
                       className="flex-1 py-3 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold shadow-lg shadow-emerald-900/50 disabled:opacity-50"
                     >
                       {isProcessing ? 'SIGNING...' : 'SIGN (Ed25519)'}
                     </button>
                  </div>
               </div>
            </div>
          </div>
        )}

        {/* Error Toast */}
        {error && (
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 glass-panel border-red-500 text-red-400 p-4 rounded-lg flex items-center gap-3">
            <AlertTriangle />
            <span>{error}</span>
            <button onClick={() => setError(null)}><X size={16}/></button>
          </div>
        )}

      </HUD>

      {/* AR Workspace Mode */}
      <ARWorkspace 
        ref={arWorkspaceRef}
        isActive={mode === AppMode.AR_WORKSPACE}
        onClose={() => setMode(AppMode.IDLE)}
      />

      {/* === TIMERS OVERLAY === */}
      {showTimers && (
        <div className="absolute top-4 right-20 z-40 w-72 animate-in slide-in-from-right-4 duration-300">
          <div className="glass-panel p-4 rounded-xl border-amber-500/30">
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-bold text-amber-300 flex items-center gap-2">
                <Clock size={16} /> Active Timers
              </h3>
              <button onClick={() => setShowTimers(false)} className="text-gray-400 hover:text-white">
                <X size={16} />
              </button>
            </div>
            {timers.filter(t => t.state === 'RUNNING' || t.state === 'PAUSED').length === 0 ? (
              <p className="text-sm text-gray-500 text-center py-4">No active timers</p>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {timers.filter(t => t.state === 'RUNNING' || t.state === 'PAUSED').map(timer => {
                  const mins = Math.floor(timer.remainingMs / 60000);
                  const secs = Math.floor((timer.remainingMs % 60000) / 1000);
                  const progress = timer.type !== 'STOPWATCH' 
                    ? ((timer.durationMs - timer.remainingMs) / timer.durationMs) * 100 
                    : 0;
                  
                  return (
                    <div key={timer.id} className="bg-black/40 p-3 rounded-lg border border-amber-500/20">
                      <div className="flex justify-between items-center mb-2">
                        <span className="text-sm font-medium text-amber-200">{timer.label}</span>
                        <div className="flex gap-1">
                          {timer.state === 'RUNNING' ? (
                            <button onClick={() => pauseTimer(timer.id)} className="text-yellow-400 hover:text-yellow-300 p-1">
                              ⏸
                            </button>
                          ) : (
                            <button onClick={() => resumeTimer(timer.id)} className="text-green-400 hover:text-green-300 p-1">
                              ▶
                            </button>
                          )}
                          <button onClick={() => cancelTimer(timer.id)} className="text-red-400 hover:text-red-300 p-1">
                            ✕
                          </button>
                        </div>
                      </div>
                      <div className="text-2xl font-mono text-amber-400 text-center">
                        {String(mins).padStart(2, '0')}:{String(secs).padStart(2, '0')}
                      </div>
                      {timer.type !== 'STOPWATCH' && (
                        <div className="h-1 bg-gray-700 rounded-full mt-2 overflow-hidden">
                          <div 
                            className="h-full bg-amber-500 transition-all duration-1000"
                            style={{ width: `${progress}%` }}
                          />
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}

      {/* === NOTIFICATIONS OVERLAY === */}
      {showNotifications && (
        <div className="absolute top-4 right-20 z-40 w-80 animate-in slide-in-from-right-4 duration-300">
          <div className="glass-panel p-4 rounded-xl border-purple-500/30">
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-bold text-purple-300 flex items-center gap-2">
                <Bell size={16} /> Notifications
              </h3>
              <div className="flex gap-2">
                {notifications.length > 0 && (
                  <button onClick={clearAllNotifications} className="text-xs text-gray-400 hover:text-white">
                    Clear All
                  </button>
                )}
                <button onClick={() => setShowNotifications(false)} className="text-gray-400 hover:text-white">
                  <X size={16} />
                </button>
              </div>
            </div>
            {notifications.length === 0 ? (
              <p className="text-sm text-gray-500 text-center py-4">No notifications</p>
            ) : (
              <div className="space-y-2 max-h-80 overflow-y-auto">
                {notifications.slice(0, 10).map(notif => (
                  <div 
                    key={notif.id} 
                    className={`p-3 rounded-lg border ${
                      notif.read 
                        ? 'bg-black/20 border-gray-700' 
                        : notif.priority === 'URGENT' 
                          ? 'bg-red-900/30 border-red-500/50' 
                          : notif.priority === 'HIGH'
                            ? 'bg-amber-900/30 border-amber-500/50'
                            : 'bg-purple-900/20 border-purple-500/30'
                    }`}
                    onClick={() => markNotificationRead(notif.id)}
                  >
                    <div className="flex justify-between items-start">
                      <div className="flex-1">
                        <div className="text-sm font-medium text-white flex items-center gap-1">
                          {!notif.read && <span className="w-2 h-2 bg-purple-500 rounded-full" />}
                          {notif.title}
                        </div>
                        <p className="text-xs text-gray-400 mt-1">{notif.body}</p>
                        <span className="text-[10px] text-gray-600 mt-1 block">
                          {new Date(notif.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                      <button 
                        onClick={(e) => {
                          e.stopPropagation();
                          dismissNotification(notif.id);
                        }} 
                        className="text-gray-500 hover:text-white p-1"
                      >
                        <X size={12} />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Left Edge Dock - Vertically Centered */}
      <div className="absolute left-4 top-1/2 -translate-y-1/2 flex flex-col gap-4 z-50 pointer-events-auto">
        <DockButton 
          icon={<Eye size={24} />} 
          label="VISION" 
          active={mode === AppMode.ANALYZING} 
          color="amber"
          onClick={() => toggleMode(AppMode.ANALYZING)} 
          labelPosition="right"
        />
        <DockButton 
          icon={<MessageSquare size={24} />} 
          label="ORACLE" 
          active={mode === AppMode.ORACLE} 
          color="purple"
          onClick={() => toggleMode(AppMode.ORACLE)} 
          labelPosition="right"
        />
        <DockButton 
          icon={<Monitor size={24} />} 
          label="AR" 
          active={mode === AppMode.AR_WORKSPACE} 
          color="cyan"
          onClick={() => toggleMode(AppMode.AR_WORKSPACE)} 
          labelPosition="right"
        />
      </div>

      {/* Right Edge Dock - Vertically Centered */}
      <div className="absolute right-4 top-1/2 -translate-y-1/2 flex flex-col gap-4 z-50 pointer-events-auto">
        <DockButton 
          icon={<Navigation2 size={24} />} 
          label="NAV" 
          active={mode === AppMode.NAVIGATION} 
          color="emerald"
          onClick={() => toggleMode(AppMode.NAVIGATION)} 
          labelPosition="left"
        />
        <DockButton 
          icon={<Wallet size={24} />} 
          label="WALLET" 
          active={mode === AppMode.WALLET} 
          color="blue"
          onClick={() => toggleMode(AppMode.WALLET)} 
          labelPosition="left"
        />
        <DockButton 
          icon={
            <div className="relative">
              <Clock size={24} />
              {timers.filter(t => t.state === 'RUNNING').length > 0 && (
                <span className="absolute -top-1 -right-1 w-3 h-3 bg-amber-500 rounded-full animate-pulse" />
              )}
            </div>
          } 
          label="TIMERS" 
          active={showTimers} 
          color="amber"
          onClick={() => setShowTimers(!showTimers)} 
          labelPosition="left"
        />
        <DockButton 
          icon={
            <div className="relative">
              <Bell size={24} />
              {notifications.filter(n => !n.read).length > 0 && (
                <span className="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full text-[8px] flex items-center justify-center">
                  {notifications.filter(n => !n.read).length}
                </span>
              )}
            </div>
          } 
          label="ALERTS" 
          active={showNotifications} 
          color="purple"
          onClick={() => setShowNotifications(!showNotifications)} 
          labelPosition="left"
        />
      </div>

    </div>
  );
};

// Helper Component for Dock Buttons
const DockButton = ({ icon, label, active, color, onClick, labelPosition = 'top' }: any) => {
  const colors: any = {
    amber: 'bg-amber-500/20 border-amber-400 text-amber-400 shadow-[0_0_20px_rgba(251,191,36,0.3)]',
    purple: 'bg-purple-500/20 border-purple-400 text-purple-400 shadow-[0_0_20px_rgba(192,132,252,0.3)]',
    emerald: 'bg-emerald-500/20 border-emerald-400 text-emerald-400 shadow-[0_0_20px_rgba(52,211,153,0.3)]',
    blue: 'bg-blue-500/20 border-blue-400 text-blue-400 shadow-[0_0_20px_rgba(96,165,250,0.3)]',
    cyan: 'bg-cyan-500/20 border-cyan-400 text-cyan-400 shadow-[0_0_20px_rgba(34,211,238,0.3)]',
  };

  const labelStyles: any = {
    top: 'absolute -top-10 left-1/2 -translate-x-1/2',
    right: 'absolute left-full ml-3 top-1/2 -translate-y-1/2',
    left: 'absolute right-full mr-3 top-1/2 -translate-y-1/2',
  };

  const activeTransform: any = {
    top: 'scale-110 -translate-y-2',
    right: 'scale-110 translate-x-2',
    left: 'scale-110 -translate-x-2',
  };

  return (
    <button
      onClick={onClick}
      className={`p-4 rounded-xl backdrop-blur-md border transition-all duration-300 group relative ${
        active 
          ? `${colors[color]} ${activeTransform[labelPosition]}` 
          : 'bg-gray-900/60 border-gray-600 text-gray-400 hover:bg-gray-800 hover:text-white'
      }`}
    >
      {icon}
      <span className={`${labelStyles[labelPosition]} text-[10px] font-bold tracking-widest bg-black/90 px-2 py-1 rounded border border-gray-800 transition-all whitespace-nowrap ${active ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}>
        {label}
      </span>
    </button>
  );
}

export default App;
