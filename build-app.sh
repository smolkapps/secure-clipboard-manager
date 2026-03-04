#!/bin/bash

# build-app.sh - Build ClipVault.app bundle
#
# Usage: ./build-app.sh
#
# Optional environment variables:
#   DEVELOPER_ID  - Code signing identity (e.g. "Developer ID Application: Name (TEAM)")
#   SKIP_BUILD    - Set to 1 to skip cargo build (use existing binary)

set -e

# Configuration
APP_NAME="ClipVault"
BINARY_NAME="clipboard-manager"
APP_BUNDLE="${APP_NAME}.app"
HELPER_NAME="ClipVaultLauncher"
HELPER_BINARY="clipvault-launcher"
HELPER_BUNDLE="${HELPER_NAME}.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Step 1: Build release binaries (Main App + Helper App)
if [ "${SKIP_BUILD}" = "1" ]; then
    log_warn "Skipping cargo build (SKIP_BUILD=1)"
else
    log_info "Building release binaries..."
    cargo build --release --bin "${BINARY_NAME}"
    cargo build --release --bin "${HELPER_BINARY}"
    log_info "Build complete"
fi

# Verify binaries exist
BINARY="${SCRIPT_DIR}/target/release/${BINARY_NAME}"
if [ ! -f "${BINARY}" ]; then
    log_error "Binary not found at ${BINARY}"
    log_info "Run 'cargo build --release' first"
    exit 1
fi
HELPER_BIN="${SCRIPT_DIR}/target/release/${HELPER_BINARY}"
if [ ! -f "${HELPER_BIN}" ]; then
    log_error "Helper Binary not found at ${HELPER_BIN}"
    exit 1
fi

BINARY_SIZE=$(du -h "${BINARY}" | cut -f1)
log_info "Binary: ${BINARY} (${BINARY_SIZE})"

# Step 2: Create app bundle structure
log_info "Creating app bundles: ${APP_BUNDLE} and ${HELPER_BUNDLE}"

# Clean old bundle if exists
if [ -d "${SCRIPT_DIR}/${APP_BUNDLE}" ]; then
    log_warn "Removing existing ${APP_BUNDLE}"
    rm -rf "${SCRIPT_DIR}/${APP_BUNDLE}"
fi

# Main App Structure
mkdir -p "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Resources"
mkdir -p "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems"

# Helper App Structure inside Main App
mkdir -p "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems/${HELPER_BUNDLE}/Contents/MacOS"

# Step 3: Copy files
log_info "Copying binaries..."
cp "${BINARY}" "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/MacOS/${BINARY_NAME}"
cp "${HELPER_BIN}" "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems/${HELPER_BUNDLE}/Contents/MacOS/${HELPER_BINARY}"

log_info "Copying Info.plist..."
cp "${SCRIPT_DIR}/resources/Info.plist" "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Info.plist"

# Create basic Info.plist for Helper App showing it's a background element
cat <<'PLIST' > "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems/${HELPER_BUNDLE}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.smolkapps.clipboard-manager.Launcher</string>
    <key>CFBundleName</key>
    <string>ClipVaultLauncher</string>
    <key>CFBundleExecutable</key>
    <string>clipvault-launcher</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
PLIST

log_info "Copying app icon..."
if [ -f "${SCRIPT_DIR}/resources/AppIcon.icns" ]; then
    cp "${SCRIPT_DIR}/resources/AppIcon.icns" "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Resources/AppIcon.icns"
else
    log_warn "AppIcon.icns not found, app will use default icon"
fi

# Step 4: Clean extended attributes
xattr -cr "${SCRIPT_DIR}/${APP_BUNDLE}" 2>/dev/null || true

# Step 5: Code signing (optional but required for App Store)
if [ -n "${DEVELOPER_ID}" ]; then
    log_info "Signing nested Helper App with: ${DEVELOPER_ID}"
    
    # Sign Helper Binary
    codesign --force \
             --sign "${DEVELOPER_ID}" \
             --options runtime \
             --timestamp \
             "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems/${HELPER_BUNDLE}/Contents/MacOS/${HELPER_BINARY}"

    # Sign Helper Bundle with Sandboxing Entitlements
    codesign --force \
             --sign "${DEVELOPER_ID}" \
             --options runtime \
             --timestamp \
             --entitlements "${SCRIPT_DIR}/resources/entitlements.plist" \
             "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/Library/LoginItems/${HELPER_BUNDLE}"

    log_info "Signing main app with: ${DEVELOPER_ID}"

    # Sign Main Binary
    codesign --force \
             --sign "${DEVELOPER_ID}" \
             --options runtime \
             --timestamp \
             "${SCRIPT_DIR}/${APP_BUNDLE}/Contents/MacOS/${BINARY_NAME}"

    # Sign Main App Bundle
    codesign --force \
             --sign "${DEVELOPER_ID}" \
             --options runtime \
             --timestamp \
             --entitlements "${SCRIPT_DIR}/resources/entitlements.plist" \
             "${SCRIPT_DIR}/${APP_BUNDLE}"

    log_info "Verifying signature..."
    codesign --verify --deep --strict --verbose=2 "${SCRIPT_DIR}/${APP_BUNDLE}"
    log_info "Signature verified"
else
    log_warn "DEVELOPER_ID not set, skipping code signing"
    log_info "For ad-hoc signing (local testing only):"
    log_info "  codesign --force --sign - ${APP_BUNDLE}"
fi

# Step 6: Summary
BUNDLE_SIZE=$(du -sh "${SCRIPT_DIR}/${APP_BUNDLE}" | cut -f1)

echo ""
log_info "========================================"
log_info "App bundle created successfully!"
log_info "========================================"
log_info ""
log_info "  Bundle:  ${SCRIPT_DIR}/${APP_BUNDLE}"
log_info "  Size:    ${BUNDLE_SIZE}"
log_info ""
log_info "Test it:"
log_info "  open ${SCRIPT_DIR}/${APP_BUNDLE}"
log_info ""
log_info "Next steps:"
if [ -z "${DEVELOPER_ID}" ]; then
    log_info "  1. Sign:     DEVELOPER_ID='Developer ID Application: ...' ./build-app.sh"
    log_info "  2. Create DMG: ./build-dmg.sh 0.1.0"
else
    log_info "  1. Create DMG: ./build-dmg.sh 0.1.0"
    log_info "  2. Notarize:  xcrun notarytool submit ClipVault-0.1.0.dmg ..."
fi
