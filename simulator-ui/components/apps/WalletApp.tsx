import React from 'react';
import { ShieldCheck, Plus, Wallet, Navigation2 } from 'lucide-react';
import { WalletState } from '../../types';

interface WalletAppProps {
  wallet: WalletState;
  onCreateWallet: () => void;
  isProcessing: boolean;
  backendConnected: boolean;
}

export const WalletApp: React.FC<WalletAppProps> = ({ wallet, onCreateWallet, isProcessing, backendConnected }) => {
  return (
    <div className="h-full flex flex-col">
       <div className="flex items-center gap-3 mb-6 border-b border-blue-400/20 pb-4">
          <ShieldCheck className="text-blue-400" size={32} />
          <div>
            <h2 className="text-xl font-bold text-blue-100">SOVEREIGN IDENTITY</h2>
            <p className="text-xs font-mono text-blue-300 truncate max-w-[250px]">{wallet.did}</p>
          </div>
       </div>
       
       {wallet.did === 'Not Connected' ? (
         <div className="text-center py-8 flex-1 flex flex-col justify-center">
           <p className="text-blue-300 mb-4">No wallet connected. Create one to get started.</p>
           <button 
             onClick={onCreateWallet}
             disabled={isProcessing || !backendConnected}
             className="px-6 py-3 bg-blue-600 hover:bg-blue-500 rounded-lg font-bold flex items-center gap-2 mx-auto disabled:opacity-50 transition-all"
           >
             <Plus size={20} />
             Create Wallet
           </button>
         </div>
       ) : (
         <div className="space-y-4 flex-1 overflow-auto">
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
  );
};
