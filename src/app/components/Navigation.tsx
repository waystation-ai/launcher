'use client';

import Link from "next/link";
import Image from "next/image";
import { useEffect, useState } from 'react';
import { authService, AuthData, UserInfo } from '@/app/lib/auth-service';
import AuthButton from "@/app/components/AuthButton";

export default function Navigation() {
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

  // Render user info if authenticated
  const renderUserInfo = (userInfo: UserInfo) => {
    return (
      <Link href="/settings">
      <div className="flex items-center space-x-2">
        {userInfo.picture && (
          <img
            src={userInfo.picture}
            alt={userInfo.name || 'User'}
            className="w-8 h-8 rounded-full"
          />
        )}
        <div className="text-sm">
          <div className="font-medium">{userInfo.name || 'User'}</div>
          {userInfo.email && <div className="text-xs text-gray-500">{userInfo.email}</div>}
        </div>
      </div>
      </Link>
    );
  };

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between flex-grow">
      <div className="flex w-full md:w-auto">
        <Link className="flex items-center gap-2" href="/">
        <Image src="/images/logo.svg" width={32} height={32} alt="WayStation" className="h-8 w-8" />
        <p className="text-2xl font-bold aurora-text">WayStation</p>
        </Link>
      </div>
      <div className="flex items-center gap-4 mt-2 md:mt-0">
        {authData?.user_info && renderUserInfo(authData.user_info)}
        {!authData && (
          <AuthButton className="aurora-btn px-4 py-2 text-sm font-bold rounded hover:scale-105 transition-transform duration-300 w-auto text-center" />
        )}
      </div>

    </div>
  )
}
