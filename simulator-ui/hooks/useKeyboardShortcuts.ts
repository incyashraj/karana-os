import { useEffect, useCallback } from 'react';
import { AppMode } from '../types';

interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: () => void;
  description: string;
}

export const useKeyboardShortcuts = (
  setMode: (mode: AppMode) => void,
  setShowSettings: (show: boolean) => void,
  setShowTimers: (show: boolean) => void,
  setShowNotifications: (show: boolean) => void
) => {
  const shortcuts: KeyboardShortcut[] = [
    {
      key: 'Escape',
      action: () => setMode(AppMode.IDLE),
      description: 'Return to Home'
    },
    {
      key: 'v',
      ctrl: true,
      action: () => setMode(AppMode.ANALYZING),
      description: 'Open Vision Analyzer'
    },
    {
      key: 'o',
      ctrl: true,
      action: () => setMode(AppMode.ORACLE),
      description: 'Open Oracle Chat'
    },
    {
      key: 'w',
      ctrl: true,
      action: () => setMode(AppMode.WALLET),
      description: 'Open Wallet'
    },
    {
      key: 'a',
      ctrl: true,
      action: () => setMode(AppMode.AR_WORKSPACE),
      description: 'Open AR Workspace'
    },
    {
      key: ',',
      ctrl: true,
      action: () => setShowSettings(true),
      description: 'Open Settings'
    },
    {
      key: 't',
      ctrl: true,
      action: () => setShowTimers(true),
      description: 'Toggle Timers'
    },
    {
      key: 'n',
      ctrl: true,
      action: () => setShowNotifications(true),
      description: 'Toggle Notifications'
    }
  ];

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const shortcut = shortcuts.find(s => {
      const keyMatch = s.key.toLowerCase() === event.key.toLowerCase();
      const ctrlMatch = s.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
      const shiftMatch = s.shift ? event.shiftKey : !event.shiftKey;
      const altMatch = s.alt ? event.altKey : !event.altKey;
      
      return keyMatch && ctrlMatch && shiftMatch && altMatch;
    });

    if (shortcut) {
      event.preventDefault();
      shortcut.action();
    }
  }, [shortcuts]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return shortcuts;
};
