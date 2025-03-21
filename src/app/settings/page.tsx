'use client';

import { useState, useEffect } from 'react';
import { authService, AuthData } from '@/app/lib/auth-service';
import AuthButton from "@/app/components/AuthButton";

export default function Settings() {
  const [authData, setAuthData] = useState<AuthData | null>(null);

  useEffect(() => {
    // Subscribe to auth changes
    const unsubscribe = authService.onAuthChange((data) => {
      setAuthData(data);
    });

    // Cleanup subscription on unmount
    return () => {
      unsubscribe();
    };
  }, []);

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>
      <div className="mb-8">
  
        <div className="grid grid-cols-1 gap-4">
          {/* Setting Item 1 */}
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Authentication</h3>
              <p className="text-sm text-gray-500">Status: <span className={authData ? 'text-green-600' : 'text-red-600'}>{authData ? 'Authenticated' : 'Not authenticated'}
          </span></p>
            </div>
            <div>
              <AuthButton className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center"/>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Onboarding</h3>
              <p className="text-sm text-gray-500">Reset the onboarding to start over</p>
            </div>
            <div>
              <div className="h-6 w-11 bg-gray-200 rounded-full relative cursor-pointer">
                <button className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center">Reset</button>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Version</h3>
              <p className="text-sm text-gray-500">Current app version</p>
            </div>
            <div>
              <div className="h-6 w-11  relative cursor-pointer">
                0.2.2
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Updates</h3>
              <p className="text-sm text-gray-500">Check for updates</p>
            </div>
            <div>
              {/* Switch component */}
              <div className="h-6 w-11 bg-gray-200 rounded-full relative cursor-pointer">
              <button className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center">Check</button>
              </div>
            </div>
          </div>
                    
          
        </div>
      </div>
      
      
    </div>
  );
}
