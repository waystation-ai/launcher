'use client';

import Image from 'next/image';
import Link from 'next/link';
import clsx from 'clsx';
import { useTrackEvent } from '@/app/lib/utils/track-event';

interface ProviderCardProps {
  name: string;
  description: string;
  isConnected: boolean;
  provider: string;
}

export default function ProviderCard({ name, description, isConnected, provider }: ProviderCardProps) {
  const trackEvent = useTrackEvent();
  
  function trackConnect() {
    trackEvent(isConnected ? 'disconnectProvider' : 'connectProvider', { provider });
  }  

  return (
    <>
      <div className="bg-gradient-to-br from-white/60 to-white/30 backdrop-blur-lg rounded-xl p-6 flex flex-col space-y-4 hover:from-white/100 hover:to-white/70 transition-all shadow-xl hover:scale-105 duration-500 relative">
        <div className="flex items-center space-x-4">
          <div className="relative w-12 h-12 shrink-0">
            <Image src={`/images/tools/${provider}.svg`} alt={name} fill className="object-contain"/>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-1">{name}</h3>
          </div>
        </div>
        <p className="flex-grow leading-relaxed">{description}</p>
        
        {/* Connect/Disconnect button for all providers */}
        <div className="flex items-center space-x-2">
          <Link
            href={`/api/auth/${provider}/${isConnected ? 'disconnect' : 'connect'}`}
            onClick={trackConnect}
            prefetch={false}
            className={clsx('connect-btn px-4 py-2 text-sm font-medium rounded-lg hover:scale-105 transition-transform duration-300 flex-grow text-center',
              {
                'bg-red-500 hover:bg-red-600 text-white' : isConnected,
                'bg-blue-600 hover:bg-blue-700 text-white' : !isConnected
              }
            )}>
            {isConnected ? 'Disconnect' : 'Connect'}
          </Link>
          
        </div>
      </div>
      
    </>
  );
}
