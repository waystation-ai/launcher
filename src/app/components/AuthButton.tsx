'use client';

import { useEffect, useState } from 'react';
import { authService, AuthData } from '@/app/lib/auth-service';

interface AuthButtonProps {
  className?: string;
}

export default function AuthButton({ className = '' }: AuthButtonProps) {
  const [authData, setAuthData] = useState<AuthData | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

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

  const handleLogin = async () => {
    setLoading(true);
    setError(null);
    try {
      await authService.login();
      // The auth state will be updated via the onAuthChange listener
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to login');
      console.error('Login error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleLogout = async () => {
    setLoading(true);
    setError(null);
    try {
      await authService.logout();
      // The auth state will be updated via the onAuthChange listener
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to logout');
      console.error('Logout error:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={`flex ${className}`}>
      {error && <div className="text-red-500 text-sm mr-2">{error}</div>}
      
      {authData ? (
        <button
          onClick={handleLogout}
          disabled={loading}
          className=""
        >
          {loading ? 'Logging out...' : 'Log Out'}
        </button>
      ) : (
        <button
          onClick={handleLogin}
          disabled={loading}
          className=""
        >
          {loading ? 'Logging in...' : 'Sign In'}
        </button>
      )}
    </div>
  );
}
