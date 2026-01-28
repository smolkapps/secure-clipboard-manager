#!/bin/bash

# build-dmg.sh - Create a DMG installer for ClipVault
#
# Usage: ./build-dmg.sh [version]
# Example: ./build-dmg.sh 0.1.0
#
# Requirements:
# - ClipVault.app must exist in current directory
# - App must be signed and notarized

set -e  # Exit on error

# Configuration
APP_NAME="ClipVault"
VERSION="${1:-0.1.0}"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
VOLUME_NAME="${APP_NAME}"
APP_BUNDLE="${APP_NAME}.app"
TEMP_DMG="temp-${APP_NAME}.dmg"
BACKGROUND_IMAGE="resources/dmg-background.png"  # Optional

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if [ ! -d "${APP_BUNDLE}" ]; then
        log_error "${APP_BUNDLE} not found!"
        log_info "Please build the app bundle first."
        exit 1
    fi

    if ! command -v hdiutil &> /dev/null; then
        log_error "hdiutil not found!"
        log_info "This script requires macOS."
        exit 1
    fi

    log_info "Prerequisites OK"
}

# Create temporary DMG
create_temp_dmg() {
    log_info "Creating temporary DMG..."

    # Remove old temp DMG if exists
    if [ -f "${TEMP_DMG}" ]; then
        rm "${TEMP_DMG}"
    fi

    # Create temporary DMG (100MB should be plenty)
    hdiutil create -size 100m -fs HFS+ -volname "${VOLUME_NAME}" "${TEMP_DMG}"

    log_info "Temporary DMG created"
}

# Mount DMG and copy files
setup_dmg_contents() {
    log_info "Setting up DMG contents..."

    # Mount temporary DMG
    hdiutil attach "${TEMP_DMG}" -mountpoint "/Volumes/${VOLUME_NAME}"

    # Copy app bundle
    log_info "Copying ${APP_BUNDLE}..."
    cp -R "${APP_BUNDLE}" "/Volumes/${VOLUME_NAME}/"

    # Create Applications symlink (for drag-to-install)
    log_info "Creating Applications symlink..."
    ln -s /Applications "/Volumes/${VOLUME_NAME}/Applications"

    log_info "DMG contents ready"
}

# Customize DMG appearance (optional)
customize_dmg_appearance() {
    log_info "Customizing DMG appearance..."

    # Set icon positions and window size using AppleScript
    # This requires the DMG to be mounted

    osascript <<EOF
tell application "Finder"
    tell disk "${VOLUME_NAME}"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {100, 100, 600, 400}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 128

        -- Position icons
        set position of item "${APP_NAME}.app" of container window to {150, 150}
        set position of item "Applications" of container window to {350, 150}

        -- Update and close
        update without registering applications
        delay 2
        close
    end tell
end tell
EOF

    log_info "DMG appearance customized"
}

# Unmount and finalize DMG
finalize_dmg() {
    log_info "Finalizing DMG..."

    # Unmount
    hdiutil detach "/Volumes/${VOLUME_NAME}"

    # Remove old DMG if exists
    if [ -f "${DMG_NAME}" ]; then
        log_warn "Removing existing ${DMG_NAME}"
        rm "${DMG_NAME}"
    fi

    # Convert to compressed, read-only DMG
    hdiutil convert "${TEMP_DMG}" -format UDZO -o "${DMG_NAME}"

    # Clean up temporary DMG
    rm "${TEMP_DMG}"

    log_info "DMG created: ${DMG_NAME}"
}

# Verify DMG
verify_dmg() {
    log_info "Verifying DMG..."

    hdiutil verify "${DMG_NAME}"

    # Get file size
    SIZE=$(du -h "${DMG_NAME}" | cut -f1)
    log_info "DMG size: ${SIZE}"

    log_info "DMG verification passed"
}

# Sign DMG (optional, requires Developer ID)
sign_dmg() {
    if [ -z "${DEVELOPER_ID}" ]; then
        log_warn "DEVELOPER_ID not set, skipping DMG signing"
        log_info "Set DEVELOPER_ID environment variable to sign DMG"
        return
    fi

    log_info "Signing DMG..."

    codesign --force \
             --sign "${DEVELOPER_ID}" \
             --timestamp \
             "${DMG_NAME}"

    # Verify signature
    codesign --verify --verbose=2 "${DMG_NAME}"

    log_info "DMG signed successfully"
}

# Main execution
main() {
    log_info "Starting DMG build for ${APP_NAME} v${VERSION}"
    log_info "========================================"

    check_prerequisites
    create_temp_dmg
    setup_dmg_contents

    # Customize appearance (can fail gracefully)
    if customize_dmg_appearance; then
        log_info "DMG appearance customized"
    else
        log_warn "Could not customize DMG appearance (continuing anyway)"
    fi

    finalize_dmg
    verify_dmg

    # Sign if Developer ID provided
    sign_dmg

    log_info "========================================"
    log_info "DMG build complete: ${DMG_NAME}"
    log_info ""
    log_info "Next steps:"
    log_info "1. Test the DMG: open ${DMG_NAME}"
    log_info "2. Notarize if signed: xcrun notarytool submit ${DMG_NAME} ..."
    log_info "3. Upload to GitHub releases"
}

# Run main function
main "$@"
