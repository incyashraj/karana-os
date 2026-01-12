import React, { useState, useEffect } from 'react';
import { Bell, MessageSquare, Mail, Calendar, AlertTriangle } from 'lucide-react';

export interface Notification {
  id: string;
  app: string;
  title: string;
  message: string;
  icon: React.ReactNode;
  timestamp: number;
  priority: 'low' | 'normal' | 'high';
}

export const NotificationStream: React.FC = () => {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  // Simulate incoming notifications
  useEffect(() => {
    const mockNotifications: Omit<Notification, 'id' | 'timestamp'>[] = [
      { app: 'Messages', title: 'Sarah Connor', message: 'Are you seeing this?', icon: <MessageSquare size={16} />, priority: 'normal' },
      { app: 'Calendar', title: 'Meeting in 10m', message: 'Project Review with Team', icon: <Calendar size={16} />, priority: 'normal' },
      { app: 'System', title: 'Battery Low', message: '15% remaining. Switch to eco mode?', icon: <AlertTriangle size={16} />, priority: 'high' },
      { app: 'Mail', title: 'Newsletter', message: 'Weekly Tech Digest', icon: <Mail size={16} />, priority: 'low' },
    ];

    const interval = setInterval(() => {
      if (Math.random() > 0.7) {
        const randomNotif = mockNotifications[Math.floor(Math.random() * mockNotifications.length)];
        const newNotif: Notification = {
          ...randomNotif,
          id: Date.now().toString(),
          timestamp: Date.now()
        };
        
        setNotifications(prev => [newNotif, ...prev].slice(0, 5));

        // Auto dismiss after 5 seconds
        setTimeout(() => {
          setNotifications(prev => prev.filter(n => n.id !== newNotif.id));
        }, 5000);
      }
    }, 8000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="fixed top-24 right-6 z-[90] flex flex-col gap-3 w-80 pointer-events-none">
      {notifications.map((notif) => (
        <div 
          key={notif.id}
          className="
            pointer-events-auto
            bg-black/60 backdrop-blur-xl border border-white/10 
            rounded-2xl p-4 shadow-lg animate-in slide-in-from-right-8 fade-in duration-300
            flex gap-4 items-start
          "
        >
          <div className={`
            w-10 h-10 rounded-full flex items-center justify-center shrink-0
            ${notif.priority === 'high' ? 'bg-red-500/20 text-red-400' : 'bg-white/10 text-white'}
          `}>
            {notif.icon}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex justify-between items-baseline mb-1">
              <span className="text-[10px] font-bold uppercase tracking-wider text-white/50">{notif.app}</span>
              <span className="text-[10px] text-white/30">Now</span>
            </div>
            <h4 className="text-sm font-bold text-white leading-tight mb-1">{notif.title}</h4>
            <p className="text-xs text-white/70 leading-relaxed truncate">{notif.message}</p>
          </div>
        </div>
      ))}
    </div>
  );
};
