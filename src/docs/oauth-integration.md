# OAuth Integration in Tauri App

This document explains how OAuth authentication is implemented in the WayStation Tauri app.

## Overview

The OAuth flow in this app follows these steps:

1. User clicks the "Sign In / Log In" button
2. The app opens the browser to the authentication provider's login page
3. User authenticates in the browser
4. The browser redirects back to the app via a deep link
5. The app exchanges the authorization code for tokens
6. The app securely stores the tokens and uses them for API calls

## Implementation Details

### Rust Side (Backend)

The Rust implementation handles:

- Opening the browser for authentication
- Processing the deep link callback
- Exchanging the authorization code for tokens
- Securely storing tokens
- Refreshing tokens when they expire

Key files:
- `src-tauri/src/lib.rs` - Contains the OAuth implementation
- `src-tauri/Cargo.toml` - Dependencies for OAuth
- `src-tauri/tauri.conf.json` - Configuration for deep linking
- `src-tauri/capabilities/default.json` - Permissions for OAuth

### TypeScript Side (Frontend)

The TypeScript implementation handles:

- Providing a login button UI
- Managing authentication state
- Making authenticated API requests
- Refreshing tokens when needed

Key files:
- `src/lib/auth-service.ts` - Authentication service
- `src/app/ui/components/AuthButton.tsx` - Login button component
- `src/app/auth-example/page.tsx` - Example page demonstrating the flow

## How to Use

1. Import the `AuthButton` component:
   ```tsx
   import AuthButton from '../ui/components/AuthButton';
   ```

2. Add it to your UI:
   ```tsx
   <AuthButton />
   ```

3. Make authenticated API requests using the auth service:
   ```tsx
   import { authService } from '../../lib/auth-service';

   // ...

   const response = await authService.fetchWithAuth('https://api.example.com/data');
   const data = await response.json();
   ```

## Configuration

The OAuth configuration is defined in `src-tauri/src/lib.rs`:

- `AUTH_URL` - The authorization URL
- `TOKEN_URL` - The token URL
- `USER_INFO_URL` - The user info URL
- `CLIENT_ID` - The client ID
- `REDIRECT_URI` - The redirect URI (must match the deep link protocol)

## Deep Linking

The app uses deep linking to handle the redirect from the browser back to the app. The deep link protocol is defined in `src-tauri/tauri.conf.json` and is set to `waystation://`.

When the browser redirects to `waystation://oauth/callback?code=...`, the app captures this URL and processes it to extract the authorization code.
