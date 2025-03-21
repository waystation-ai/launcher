'use client';

import { useState } from 'react';
// For Tauri v2
import { invoke } from '@tauri-apps/api/core';

interface InstallButtonProps {
  className?: string;
}

export default function InstallButton({ className }: InstallButtonProps) {
  const [isInstalling, setIsInstalling] = useState(false);
  const [installStatus, setInstallStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  const handleInstall = async () => {
    try {
      setIsInstalling(true);
      setInstallStatus('idle');
      setErrorMessage('');
      
      const result = await invoke('install_waystation_mcp');
      
      setInstallStatus('success');
      console.log('Installation successful:', result);
    } catch (error) {
      setInstallStatus('error');
      setErrorMessage(error instanceof Error ? error.message : String(error));
      console.error('Installation failed:', error);
    } finally {
      setIsInstalling(false);
    }
  };

  return (
    <button
      onClick={handleInstall}
      disabled={isInstalling}
      className={className}
    >
      {isInstalling ? 'Installing...' : 'Install'}
      {installStatus === 'success' && <span className="ml-2">✓</span>}
      {installStatus === 'error' && (
        <div className="text-red-500 text-xs mt-1">
          {errorMessage || 'Installation failed'}
        </div>
      )}
    </button>
  );
}
