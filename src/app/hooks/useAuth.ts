import { useState, useEffect } from 'react';
import { authService, AuthData } from '@/app/lib/auth-service';

export function useAuth() {
  const [authData, setAuthData] = useState<AuthData | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    // Subscribe to auth changes
    const unsubscribe = authService.onAuthChange((data) => {
      setAuthData(data);
      setIsAuthenticated(!!data);
    });

    // Cleanup subscription on unmount
    return () => {
      unsubscribe();
    };
  }, []);

  return {
    authData,
    isAuthenticated,
    // Add any other auth-related helpers here
    getAccessToken: () => authData?.access_token || null,
    getUserInfo: () => authData?.user_info || null,
  };
}
