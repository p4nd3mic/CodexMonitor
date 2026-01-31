#!/bin/bash
# Sign CodexMonitor development build with entitlements
# This helps reduce permission prompts during development

set -e

APP_PATH="${1:-src-tauri/target/debug/bundle/macos/CodexMonitor.app}"
ENTITLEMENTS="src-tauri/Entitlements.plist"

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "Error: App not found at $APP_PATH"
    echo "Usage: $0 [path-to-app]"
    echo ""
    echo "Run 'npm run tauri build -- --debug' first to create the app bundle."
    exit 1
fi

# Check if entitlements file exists
if [ ! -f "$ENTITLEMENTS" ]; then
    echo "Error: Entitlements file not found at $ENTITLEMENTS"
    exit 1
fi

echo "Signing CodexMonitor at: $APP_PATH"
echo "Using entitlements: $ENTITLEMENTS"
echo ""

# Sign with ad-hoc identity (works for local development)
codesign --force --deep --sign - --entitlements "$ENTITLEMENTS" "$APP_PATH"

echo ""
echo "Done! App signed with ad-hoc identity."
echo ""
echo "Note: For persistent file access permissions, you may still need to:"
echo "  1. Open System Settings > Privacy & Security > Full Disk Access"
echo "  2. Click the '+' button and add CodexMonitor.app"
echo "  3. Alternatively, add CodexMonitor to 'Files and Folders' permissions"
echo ""
echo "The app is at: $APP_PATH"
