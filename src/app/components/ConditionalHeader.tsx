"use client";

import { useEffect, useState } from "react";
import { isOnboardingCompleted } from '@/app/lib/utils/onboarding';
import { authService } from '@/app/lib/auth-service';
import Navigation from '@/app/components/Navigation';

export default function ConditionalHeader() {
  const hasOnboardingCompleted = isOnboardingCompleted();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  
  useEffect(() => {
    const unsubscribe = authService.onAuthChange((data) => {
      setIsAuthenticated(!!data);
    });
    return () => unsubscribe();
  }, []);
  
  const showOnboarding = !hasOnboardingCompleted && !isAuthenticated;

  // Don't render the header if onboarding is being shown
  if (showOnboarding) {
    return null;
  }

  return (
    <header className="bg-white/80 rounded-bl-2xl rounded-br-2xl shadow-[0px_2px_16px_0px_rgba(0,0,0,0.08)] border-b border-white backdrop-blur-xl px-4 sm:px-6 py-4 flex flex-row justify-between items-center sticky top-0 z-50 gap-4 sm:gap-0">
      <Navigation />
    </header>
  );
}
