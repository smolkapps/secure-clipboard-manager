#!/bin/bash
# Simplified DMG builder - skip appearance customization

set -e

APP_NAME="ClipVault"
VERSION="${1:-0.1.0}"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
VOLUME_NAME="${APP_NAME}"
APP_BUNDLE="${APP_NAME}.app"

echo "Building ${DMG_NAME}..."

# Check app exists
if [ ! -d "${APP_BUNDLE}" ]; then
    echo "ERROR: ${APP_BUNDLE} not found"
    exit 1
fi

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf ${TEMP_DIR}" EXIT

# Copy app and create Applications symlink
cp -R "${APP_BUNDLE}" "${TEMP_DIR}/"
ln -s /Applications "${TEMP_DIR}/Applications"

# Create DMG
rm -f "${DMG_NAME}"
hdiutil create -volname "${VOLUME_NAME}" \
               -srcfolder "${TEMP_DIR}" \
               -ov -format UDZO \
               "${DMG_NAME}"

# Verify
hdiutil verify "${DMG_NAME}"

SIZE=$(du -h "${DMG_NAME}" | cut -f1)
echo "✓ DMG created: ${DMG_NAME} (${SIZE})"
