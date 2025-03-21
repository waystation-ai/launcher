#!/bin/bash

# Script to register the waystation:// protocol on macOS
# This is a workaround for development environments where the protocol
# might not be properly registered by Tauri

# Define the app bundle ID and protocol
BUNDLE_ID="ai.waystation.launcher"
PROTOCOL="waystation"

echo "Registering $PROTOCOL:// protocol for $BUNDLE_ID..."

# Register the protocol handler
defaults write com.apple.LaunchServices LSHandlers -array-add "{LSHandlerURLScheme=\"$PROTOCOL\";LSHandlerRoleAll=\"$BUNDLE_ID\";}"

# Restart the Launch Services database
echo "Restarting Launch Services database..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -kill -r -domain local -domain system -domain user

echo "Protocol registration complete!"
echo "You may need to restart your browser for the changes to take effect."

# Verify registration
echo "Verifying registration..."
defaults read com.apple.LaunchServices LSHandlers | grep $PROTOCOL

echo "Done!"
