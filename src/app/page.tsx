"use client";
import { useEffect, useState } from "react";
import { fetch } from '@tauri-apps/plugin-http';
import Link from "next/link";
import Image from "next/image";

import { useAuth } from "@/app/hooks/useAuth";

import ClaudeButton from "@/app/components/ClaudeButton";
import { OnboardingScreen } from "@/app/components/OnboardingScreen";
import { isOnboardingCompleted, markOnboardingCompleted } from "@/app/lib/utils/onboarding";
import { ProviderIcon } from "@/app/components/ProviderIcon";

// Define types for the provider data
interface ProviderTool {
  name: string;
  summary: string;
  description: string;
}

interface Provider {
  id: string;
  name: string;
  description: string;
  icon: string;
  isConnected: boolean;
  tools: ProviderTool[];
}

interface ProviderConfig {
  name: string;
  description: string;
  icon: string;
  isConnected: boolean;
}


export default function Home() {
  const { authData, isAuthenticated, isInitialized } = useAuth();
  const [providers, setProviders] = useState<[string, ProviderConfig][]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    // Only fetch providers when auth is fully initialized
    if (!isInitialized) {
      return;
    }
    
    // Set loading state to true when we start fetching
    setIsLoading(true);
    
    // Fetch providers from the API
    const fetchProviders = async () => {
      try {
        // Prepare headers - include auth token if available
        const headers: Record<string, string> = {
          'Content-Type': 'application/json'
        };
        
        if (authData?.access_token) {
          console.log('Token:', authData.access_token);
          headers['Authorization'] = `Bearer ${authData.access_token}`;
        }
        
        const response = await fetch('https://waystation.ai/api/marketplace', {
          method: 'GET',
          headers
        });
        if (!response.ok) {
          throw new Error('Failed to fetch providers');
        }
        const data = await response.json();

        console.log('Providers:', data);
        
        // Transform the data into the format expected by the markup
        // From: [{id: 'monday', name: 'Monday', ...}, ...]
        // To: [['monday', {name: 'Monday', ...}], ...]
        const transformedData = data.map((provider: Provider) => [
          provider.id, 
          {
            name: provider.name,
            description: provider.description,
            icon: provider.icon,
            isConnected: provider.isConnected
          }
        ]);
        
        setProviders(transformedData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'An error occurred');
        console.error('Error fetching providers:', err);
      } finally {
        setIsLoading(false);
      }
    };

    fetchProviders();
  }, [authData, isInitialized]);
  
  const hasOnboardingCompleted = isOnboardingCompleted();

  const handleOnboardingComplete = () => {
    markOnboardingCompleted();
  };


  // Determine if onboarding should be shown
  const showOnboarding = !hasOnboardingCompleted && !isAuthenticated;

  return (
    <div className="flex flex-col">
      {/* Only show header and main content when onboarding is not shown */}
      {!showOnboarding && (
        <>
          {/* Hero Section */}
          <main className="flex-1 flex flex-col items-center justify-center mx-6 my-6">
            <div className="flex flex-row justify-between items-center w-full">
              <div>
                <h2 className="text-2xl">Apps</h2>
                <p className="text-sm text-gray-600">Connect your favorite apps to Claude and let Claude interact with them for you</p>
              </div>
              <ClaudeButton className="aurora-btn px-4 py-2 ml-3 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center" />
            </div>
            {/* Provider Grid */}
            {!isInitialized && <p>Initializing...</p>}
            {isInitialized && isLoading && <p>Loading apps...</p>}
            {isInitialized && error && <p className="text-red-500">Error: {error}</p>}
            {isInitialized && !isLoading && !error && (
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 lg:grid-cols-8 gap-6 w-full my-6">
                {providers.map(([provider, config]) => (
                  <Link key={provider} href={`https://waystation.ai/connect/claude/${provider}?redirect_uri=waystation://home`} target="_blank" title={config.description} className="provider-card flex flex-col items-center justify-center p-4 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200 relative">
                    {config.isConnected && (
                      <div className="absolute top-2 right-2">
                        <Image src="/images/ico-connected.svg" width={16} height={16} alt="Connected" />
                      </div>
                    )}
                    <ProviderIcon provider={provider} url={config.icon} />
                    <p className="mt-2 text-sm text-gray-600 text-center">{config.name}</p>
                    <p className="mt-1 text-xs text-[#9ca2b0] text-center">{config.isConnected ? "Connected" : '\u00A0'}</p>
                  </Link>
                ))}
              </div>
            )}
          </main>
        </>
      )}
      <OnboardingScreen isOpen={showOnboarding} onComplete={handleOnboardingComplete} />
    </div>
  );
}
