import { markOnboardingCompleted, resetOnboardingStatus } from '@/app/lib/utils/onboarding';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onOpenUrl } from '@tauri-apps/plugin-deep-link';

// OAuth configuration
const REDIRECT_URI = 'waystation://oauth/callback';

export interface UserInfo {
  sub: string;
  name?: string;
  email?: string;
  picture?: string;
}

export interface AuthData {
  access_token: string;
  refresh_token?: string;
  id_token?: string;
  expires_at?: number;
  user_info?: UserInfo;
}

class AuthService {
  private authListeners: ((authData: AuthData | null) => void)[] = [];
  private authData: AuthData | null = null;
  private refreshTimer: number | null = null;

  constructor() {
    this.initialize();
  }

  private async initialize() {
    // Listen for auth events from Tauri
    await listen<AuthData>('auth-success', (event) => {
      this.setAuthData(event.payload);
    });

    await listen<string>('auth-error', (event) => {
      console.error('Authentication error:', event.payload);
      this.setAuthData(null);
    });

    // Check if we're already authenticated
    try {
      const authData = await invoke<AuthData | null>('get_auth_data');
      if (authData) {
        this.setAuthData(authData);
      }
    } catch (error) {
      console.error('Failed to get auth data:', error);
    }

    try {
      console.log('Setting up deep link handler...');
      
      // Set up deep link handler using the plugin
      await onOpenUrl((urls: string[]) => {
        console.log('Deep link handler called with URLs:', urls);
        
        // The plugin provides an array of URLs, but we only need the first one
        const url = urls[0];
        if (!url)
          return;

        console.log('Processing deep link:', url);
        
        if (url == "waystation://home") {
          console.log('Navigate to home page');
          if (window.location.pathname !== '/') {
            window.location.href = '/';
          } else {
            window.location.reload();
          }
        }
        else if (url == "waystation://onboarding") {
          console.log('Reset onboarding');
          
          resetOnboardingStatus();
        }
          // Process the OAuth callback URL
        else if (url.startsWith(REDIRECT_URI)) {
          console.log('URL matches REDIRECT_URI, processing OAuth callback');
          this.processOAuthCallback(url);
        } else {
          console.warn('URL does not match REDIRECT_URI, ignoring');
        }
      });
      
      console.log('Deep link handler setup complete');
    } catch (error) {
      console.error('Failed to set up deep link handler:', error);
    }
  }

  public async login(): Promise<void> {
    try {
      await invoke<string>('login');
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  }

  public async logout(): Promise<void> {
    try {
      await invoke<void>('logout');
      this.setAuthData(null);
    } catch (error) {
      console.error('Logout failed:', error);
      throw error;
    }
  }

  public async refreshToken(): Promise<AuthData> {
    try {
      const authData = await invoke<AuthData>('refresh_token');
      this.setAuthData(authData);
      return authData;
    } catch (error) {
      console.error('Token refresh failed:', error);
      throw error;
    }
  }

  public getAuthData(): AuthData | null {
    return this.authData;
  }

  public isAuthenticated(): boolean {
    return !!this.authData;
  }

  public getAccessToken(): string | null {
    return this.authData?.access_token || null;
  }

  public getUserInfo(): UserInfo | null {
    return this.authData?.user_info || null;
  }

  public onAuthChange(listener: (authData: AuthData | null) => void): () => void {
    this.authListeners.push(listener);
    
    // Call the listener immediately with the current auth state
    listener(this.authData);
    
    // Return a function to remove the listener
    return () => {
      this.authListeners = this.authListeners.filter(l => l !== listener);
    };
  }

  private setAuthData(authData: AuthData | null): void {
    this.authData = authData;
    
    // Clear any existing refresh timer
    if (this.refreshTimer !== null) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
    
    // Set up token refresh if we have an expiration time
    if (authData?.expires_at && authData.refresh_token) {
      const expiresAt = authData.expires_at * 1000; // Convert to milliseconds
      const now = Date.now();
      const timeUntilExpiry = expiresAt - now;
      
      // Refresh 5 minutes before expiry
      const refreshTime = Math.max(0, timeUntilExpiry - 5 * 60 * 1000);
      
      if (refreshTime > 0) {
        this.refreshTimer = window.setTimeout(() => {
          this.refreshToken().catch(console.error);
        }, refreshTime);
      } else {
        // Token is already expired or about to expire, refresh now
        this.refreshToken().catch(console.error);
      }
    }
    
    // Notify all listeners
    this.authListeners.forEach(listener => listener(authData));
  }

  /**
   * Process the OAuth callback URL
   * This is called when the app receives a deep link after authentication
   */
  private async processOAuthCallback(url: string): Promise<void> {
    console.log('Processing OAuth callback URL:', url);
    try {      
      console.log('Invoking handle_redirect_uri with URL:', url);
      // Call the Rust function to handle the redirect URI
      const authData = await invoke<AuthData>('handle_redirect_uri', { url });
      console.log('Received auth data:', authData);
      this.setAuthData(authData);
      markOnboardingCompleted();
    } catch (error) {
      console.error('Failed to process OAuth callback:', error);
      this.setAuthData(null);
    }
  }

}

// Export a singleton instance
export const authService = new AuthService();
