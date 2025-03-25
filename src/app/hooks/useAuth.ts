import { useState, useEffect } from 'react';
import { authService, AuthData } from '@/app/lib/auth-service';

export function useAuth() {
  const [authData, setAuthData] = useState<AuthData | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    // Subscribe to auth changes
    const unsubscribe = authService.onAuthChange((data) => {
      setAuthData(data);
      setIsAuthenticated(!!data);
      setIsInitialized(true);
    });

    // Cleanup subscription on unmount
    return () => {
      unsubscribe();
    };
  }, []);

  return {
    authData,
    isAuthenticated,
    isInitialized,
    // Add any other auth-related helpers here
    getAccessToken: () => authData?.access_token || null,
    getUserInfo: () => authData?.user_info || null,
  };
}
