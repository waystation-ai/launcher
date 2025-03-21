#!/bin/bash

# Script to test deep link handling by opening a test URL with the waystation:// protocol

# Default values
CODE="TEST_CODE"
STATE="TEST_STATE"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --code)
      CODE="$2"
      shift 2
      ;;
    --state)
      STATE="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Construct the test URL
TEST_URL="waystation://oauth/callback?code=${CODE}&state=${STATE}"

echo "Opening test URL: ${TEST_URL}"

# Open the URL (this will trigger the deep link handler)
open "${TEST_URL}"

echo "URL opened. Check the app logs for deep link handling events."
echo "If the app didn't launch or didn't receive the deep link, make sure the protocol is registered:"
echo "  ./scripts/register_protocol_macos.sh"
