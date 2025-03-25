'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/app/hooks/useAuth';
import { resetOnboardingStatus } from '@/app/lib/utils/onboarding';
import AuthButton from "@/app/components/AuthButton";
import Footer from '@/app/components/Footer';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';

export default function Settings() {
  const { authData } = useAuth();
  const [updateStatus, setUpdateStatus] = useState<'idle' | 'checking' | 'available' | 'downloading' | 'ready'>('idle');
  const [updateInfo, setUpdateInfo] = useState<{ version: string, notes: string } | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<{ downloaded: number, total: number } | null>(null);
  const [appVersion, setAppVersion] = useState<string>('Loading...');

  useEffect(() => {
    // Get app version
    getVersion().then(version => {
      setAppVersion(version);
    }).catch(err => {
      console.error('Failed to get app version:', err);
      setAppVersion('Unknown');
    });
  }, []);

  const checkForUpdates = async () => {
    if (updateStatus === 'checking' || updateStatus === 'downloading') return;
    
    setUpdateStatus('checking');
    try {
      const update = await check();
      if (update) {
        console.log(`Found update ${update.version} from ${update.date} with notes ${update.body || ''}`);
        setUpdateStatus('available');
        setUpdateInfo({ 
          version: update.version || 'Unknown', 
          notes: update.body || 'No release notes available' 
        });
      } else {
        setUpdateStatus('idle');
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
      setUpdateStatus('idle');
    }
  };

  const downloadAndInstallUpdate = async () => {
    if (updateStatus !== 'available') return;
    
    setUpdateStatus('downloading');
    setDownloadProgress({ downloaded: 0, total: 0 });
    
    try {
      const update = await check();
      if (!update) {
        setUpdateStatus('idle');
        return;
      }
      
      let downloaded = 0;
      let contentLength = 0;
      
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 0;
            setDownloadProgress({ downloaded: 0, total: contentLength });
            console.log(`Started downloading ${event.data.contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            setDownloadProgress({ downloaded, total: contentLength });
            console.log(`Downloaded ${downloaded} from ${contentLength}`);
            break;
          case 'Finished':
            console.log('Download finished');
            setUpdateStatus('ready');
            break;
        }
      });

      console.log('Update installed');
      await relaunch();
    } catch (error) {
      console.error('Failed to download and install update:', error);
      setUpdateStatus('available');
    }
  };

  return (
    <div className="relative pb-16">
      <div className="container mx-auto px-4 py-6">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>
      <div className="mb-8">
  
        <div className="grid grid-cols-1 gap-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Authentication</h3>
              <p className="text-sm text-gray-500">Status: <span className={authData ? 'text-green-600' : 'text-red-600'}>{authData ? 'Authenticated' : 'Not authenticated'}</span></p>
            </div>
            <AuthButton className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center"/>
          </div>


          {/* Setting Item 3 - Version */}
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Version</h3>
              <p className="text-sm text-gray-500">Current app version</p>
            </div>
            <div className="px-4 py-2 text-sm font-medium">
              {appVersion}
            </div>
          </div>

          {/* Setting Item 4 - Updates */}
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Updates</h3>
              <p className="text-sm text-gray-500">
                {(() => {
                  switch (updateStatus) {
                    case 'idle':
                      return 'Check for updates';
                    case 'checking':
                      return 'Checking for updates...';
                    case 'available':
                      return `Update available: ${updateInfo?.version}`;
                    case 'downloading':
                      return downloadProgress 
                        ? `Downloading: ${Math.round((downloadProgress.downloaded / downloadProgress.total) * 100)}%` 
                        : 'Downloading update...';
                    case 'ready':
                      return 'Update ready to install';
                    default:
                      return 'Check for updates';
                  }
                })()}
              </p>
            </div>
            {updateStatus === 'available' ? (
              <button 
                onClick={downloadAndInstallUpdate}
                className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center"
              >
                Update
              </button>
            ) : (
              <button 
                onClick={checkForUpdates}
                disabled={updateStatus !== 'idle'}
                className={`aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center ${updateStatus !== 'idle' ? 'opacity-50 cursor-not-allowed' : ''}`}
              >
                Check
              </button>
            )}
          </div>
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Onboarding</h3>
              <p className="text-sm text-gray-500">Reset the onboarding to start over</p>
            </div>
            <button 
              onClick={() => {
                resetOnboardingStatus();
                alert('Onboarding has been reset. Restart the app to see the onboarding screen.');
              }} 
              className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center">
              Reset
            </button>
          </div>

          
        </div>
      </div>
      <h1 className="text-2xl font-bold mb-3">Credits</h1>
          {/* Credits Section */}
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500">
                This app was inspired by Fleur and partially based on its{' '}
                <a 
                  href="https://github.com/fleuristes/fleur" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="text-blue-500 hover:underline"
                >
                  source code
                </a>
                . Thanks to  <a 
                  href="https://github.com/ferrucc-io" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="text-blue-500 hover:underline"
                >Ferruccio Balestreri</a> and others who came up with the idea!
              </p>
            </div>
          </div>
      
      </div>
      <div className="fixed bottom-0 left-0 right-0 z-10">
        <Footer />
      </div>
    </div>
  );
}
