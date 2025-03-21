'use client';

import { useState } from 'react';
// For Tauri v2
import { invoke } from '@tauri-apps/api/core';

interface ClaudeButtonProps {
  className?: string;
}

export default function ClaudeButton({ className }: ClaudeButtonProps) {
  const [isRestarting, setIsRestarting] = useState(false);
  const [restartStatus, setRestartStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  const handleOpenClaude = async () => {
    try {
      setIsRestarting(true);
      setRestartStatus('idle');
      setErrorMessage('');
      
      const result = await invoke('restart_claude_app');
      
      setRestartStatus('success');
      console.log('Claude app restarted successfully:', result);
    } catch (error) {
      setRestartStatus('error');
      setErrorMessage(error instanceof Error ? error.message : String(error));
      console.error('Failed to restart Claude app:', error);
    } finally {
      setIsRestarting(false);
    }
  };

  return (
    <button
      onClick={handleOpenClaude}
      disabled={isRestarting}
      className={className}
    >
      {isRestarting ? 'Opening...' : 'Open Claude'}
      {restartStatus === 'error' && (
        <div className="text-red-500 text-xs mt-1">
          {errorMessage || 'Failed to open Claude'}
        </div>
      )}
    </button>
  );
}
