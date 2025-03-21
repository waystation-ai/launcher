# Deep Link Debugging Guide

This document provides instructions for debugging and fixing deep link handling in the WayStation Tauri app.

## Overview

The app uses the `waystation://` custom URL protocol to handle OAuth callbacks. When the browser redirects to a `waystation://` URL, the operating system should launch our app and pass the URL to it. However, this requires proper registration of the protocol handler with the operating system.

## Manual Protocol Registration on macOS

For development environments, you may need to manually register the protocol handler:

1. Run the provided script:

```bash
./scripts/register_protocol_macos.sh
```

This script:
- Registers the `waystation://` protocol with macOS
- Associates it with the app bundle ID `ai.waystation.launcher`
- Restarts the Launch Services database
- Verifies the registration

2. Restart your browser after running the script

## Building with Protocol Registration

When building the app for distribution:

1. The `Info.plist` file in `src-tauri/` contains the necessary URL scheme registration
2. The `build.rs` script copies this file to the appropriate location during the build process
3. When the app is installed, the protocol should be automatically registered

## Debugging Deep Links

If you're having issues with deep links:

1. Check the console logs for messages from the deep link handler
2. Verify the protocol registration:
   ```bash
   defaults read com.apple.LaunchServices LSHandlers | grep waystation
   ```
3. Try manually triggering a deep link:
   ```bash
   open "waystation://oauth/callback?code=test&state=test"
   ```
4. Check if the app receives the deep link event

## Common Issues

1. **Browser Cancelling Navigation**: Some browsers may block navigation to custom protocols for security reasons. Try using a different browser or check browser settings.

2. **Protocol Not Registered**: The OS doesn't know which app should handle the `waystation://` protocol. Run the registration script.

3. **App Not Receiving Events**: The app might not be properly set up to receive deep link events. Check the console logs for errors.

4. **Development vs Production**: Protocol handlers sometimes work differently in development mode. Try building and installing the app properly.

## Testing Deep Links

You can test deep links with this command:

```bash
open "waystation://oauth/callback?code=TEST_CODE&state=TEST_STATE"
```

The app should:
1. Launch if not already running
2. Receive the deep link event
3. Log the URL to the console
4. Attempt to process it as an OAuth callback

If this works but the real OAuth flow doesn't, the issue might be with the OAuth configuration rather than the deep link handling.
