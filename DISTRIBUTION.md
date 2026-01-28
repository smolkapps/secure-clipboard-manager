# Distribution Guide

This document outlines the process for packaging and distributing ClipVault for macOS.

## Table of Contents

1. [Build Process](#build-process)
2. [App Bundle Creation](#app-bundle-creation)
3. [Code Signing](#code-signing)
4. [Notarization](#notarization)
5. [DMG Creation](#dmg-creation)
6. [Release Checklist](#release-checklist)
7. [Distribution Channels](#distribution-channels)

---

## Build Process

### Prerequisites

- macOS 12.7.5+
- Rust 1.92.0+
- Xcode Command Line Tools
- Apple Developer Account (for signing/notarization)

### Release Build

```bash
# Clean build
cargo clean

# Build with optimizations
cargo build --release

# Binary location
./target/release/clipboard-manager
```

### Build Verification

```bash
# Check binary size (should be ~14MB stripped)
ls -lh target/release/clipboard-manager

# Check dependencies (should be minimal)
otool -L target/release/clipboard-manager

# Test the binary
./target/release/clipboard-manager --version
```

### Optimization Settings

In `Cargo.toml`:

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization (slower compile)
strip = true           # Strip debug symbols
```

**Expected results**:
- Binary size: ~14MB (stripped)
- Startup time: <100ms
- Memory usage: <50MB

---

## App Bundle Creation

### Bundle Structure

```
ClipVault.app/
├── Contents/
│   ├── Info.plist           # App metadata
│   ├── MacOS/
│   │   └── clipboard-manager # Binary
│   ├── Resources/
│   │   └── AppIcon.icns     # App icon
│   └── _CodeSignature/      # Signature (after signing)
```

### Create Bundle Manually

```bash
# Create bundle structure
mkdir -p ClipVault.app/Contents/MacOS
mkdir -p ClipVault.app/Contents/Resources

# Copy binary
cp target/release/clipboard-manager ClipVault.app/Contents/MacOS/

# Copy icon (when ready)
cp resources/AppIcon.icns ClipVault.app/Contents/Resources/

# Create Info.plist (see below)
```

### Info.plist Template

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>clipboard-manager</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.smolkapps.clipboard-manager</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ClipVault</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.7.5</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2026 SmolkApps. All rights reserved.</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
```

**Key fields**:
- `CFBundleIdentifier`: Reverse domain identifier
- `CFBundleShortVersionString`: User-facing version (0.1.0)
- `CFBundleVersion`: Build number (increment for each build)
- `LSUIElement`: true = no dock icon (menu bar only)
- `LSMinimumSystemVersion`: Minimum macOS version

---

## Code Signing

### Why Code Sign?

- **Gatekeeper**: Required to run on modern macOS
- **Trust**: Users see "verified developer"
- **Notarization**: Required for automatic approval
- **Security**: Ensures app hasn't been tampered with

### Prerequisites

1. **Apple Developer Account**: $99/year
2. **Developer ID Certificate**: "Developer ID Application"

### Get Certificate

1. Go to [Apple Developer](https://developer.apple.com)
2. Certificates, Identifiers & Profiles
3. Create "Developer ID Application" certificate
4. Download and install in Keychain

### Find Your Certificate

```bash
# List signing identities
security find-identity -v -p codesigning

# Output example:
# 1) ABC123... "Developer ID Application: Your Name (TEAM_ID)"
```

### Sign the App

```bash
# Sign the binary
codesign --force --sign "Developer ID Application: Your Name (TEAM_ID)" \
         --options runtime \
         --timestamp \
         ClipVault.app/Contents/MacOS/clipboard-manager

# Sign the app bundle
codesign --force --sign "Developer ID Application: Your Name (TEAM_ID)" \
         --options runtime \
         --timestamp \
         --entitlements entitlements.plist \
         ClipVault.app

# Verify signature
codesign --verify --deep --strict --verbose=2 ClipVault.app
spctl --assess --type execute --verbose=4 ClipVault.app
```

### Entitlements File

Create `entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <false/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <false/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <false/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### Common Issues

**"no identity found"**
- Install Developer ID certificate in Keychain
- Restart Keychain Access

**"invalid signature"**
- Ensure binary is not stripped after signing
- Sign with `--force` to overwrite old signature

**"resource fork, Finder information, or similar detritus not allowed"**
```bash
# Clean extended attributes
xattr -cr ClipVault.app
```

---

## Notarization

### Why Notarize?

- **Gatekeeper**: Required for automatic approval on macOS 10.15+
- **User Trust**: No scary warnings
- **Distribution**: Can distribute outside Mac App Store

### Prerequisites

1. **Code signed app**: Must be signed first
2. **App-specific password**: For notarization API

### Create App-Specific Password

1. Go to [appleid.apple.com](https://appleid.apple.com)
2. Sign in
3. Security → App-Specific Passwords
4. Generate password
5. Save it securely

### Notarize the App

```bash
# 1. Create a ZIP (or DMG)
ditto -c -k --keepParent ClipVault.app ClipVault.zip

# 2. Submit for notarization
xcrun notarytool submit ClipVault.zip \
    --apple-id "your@email.com" \
    --password "app-specific-password" \
    --team-id "TEAM_ID" \
    --wait

# 3. Check status (if needed)
xcrun notarytool log "submission-id" \
    --apple-id "your@email.com" \
    --password "app-specific-password" \
    --team-id "TEAM_ID"

# 4. Staple the ticket (allows offline verification)
xcrun stapler staple ClipVault.app

# 5. Verify stapling
xcrun stapler validate ClipVault.app
spctl --assess -vv --type install ClipVault.app
```

### Expected Output

**Success**:
```
ClipVault.app: accepted
source=Notarized Developer ID
origin=Developer ID Application: Your Name (TEAM_ID)
```

**Failure**:
```
ClipVault.app: rejected
(details in notarization log)
```

### Common Notarization Issues

**"The binary uses an SDK older than the 10.9 SDK"**
- Update Xcode/Rust version

**"The signature does not include a secure timestamp"**
- Add `--timestamp` to codesign command

**"The executable does not have the hardened runtime enabled"**
- Add `--options runtime` to codesign command

### Store Credentials

To avoid typing password each time:

```bash
# Store credentials in keychain
xcrun notarytool store-credentials "ClipVault-Notarization" \
    --apple-id "your@email.com" \
    --team-id "TEAM_ID"

# Use stored credentials
xcrun notarytool submit ClipVault.zip \
    --keychain-profile "ClipVault-Notarization" \
    --wait
```

---

## DMG Creation

### Why DMG?

- **Standard format**: Users expect DMG for Mac apps
- **Drag-to-install**: Visual install experience
- **Compact**: Compressed for smaller downloads
- **Professional**: Customizable appearance

### DMG Script

See `build-dmg.sh` for automated DMG creation.

**Manual process**:

```bash
# 1. Create temporary DMG
hdiutil create -size 100m -fs HFS+ -volname "ClipVault" temp.dmg

# 2. Mount it
hdiutil attach temp.dmg -mountpoint /Volumes/ClipVault

# 3. Copy app
cp -R ClipVault.app /Volumes/ClipVault/

# 4. Create Applications symlink
ln -s /Applications /Volumes/ClipVault/Applications

# 5. Customize appearance (optional - requires AppleScript)
# See build-dmg.sh for details

# 6. Unmount
hdiutil detach /Volumes/ClipVault

# 7. Convert to compressed, read-only
hdiutil convert temp.dmg -format UDZO -o ClipVault-0.1.0.dmg

# 8. Clean up
rm temp.dmg
```

### DMG Verification

```bash
# Verify DMG can be mounted
hdiutil verify ClipVault-0.1.0.dmg

# Test installation
open ClipVault-0.1.0.dmg
# Drag app to Applications
# Launch and test
```

---

## Release Checklist

### Pre-Release

- [ ] All tests passing (`cargo test --all`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Version bumped in `Cargo.toml`
- [ ] Version bumped in `Info.plist`
- [ ] CHANGELOG updated
- [ ] Documentation updated
- [ ] README updated

### Build

- [ ] Clean build (`cargo clean && cargo build --release`)
- [ ] Binary tested manually
- [ ] Performance verified (<100ms popup, <50ms search)
- [ ] Memory usage checked (<50MB)

### Bundle

- [ ] App bundle created
- [ ] Info.plist correct
- [ ] Icon included (when ready)
- [ ] Bundle structure verified

### Signing & Notarization

- [ ] Binary signed
- [ ] App bundle signed
- [ ] Signature verified
- [ ] App notarized
- [ ] Notarization ticket stapled
- [ ] Gatekeeper test passed

### DMG

- [ ] DMG created
- [ ] DMG signed and notarized
- [ ] DMG tested (install and launch)
- [ ] File size reasonable (<20MB)

### Release

- [ ] Create GitHub release
- [ ] Upload DMG
- [ ] Write release notes
- [ ] Update website (when available)
- [ ] Announce on social media

### Post-Release

- [ ] Monitor for issues
- [ ] Respond to user feedback
- [ ] Update download counts

---

## Distribution Channels

### Current (v0.1.0)

- **GitHub Releases**: Primary distribution
  - DMG download
  - Release notes
  - Source code

### Future

- **Homebrew**: `brew install clipvault`
- **Mac App Store**: Sandboxed version (requires changes)
- **Website**: Direct download at smolkapps.com
- **Sparkle**: Auto-update framework

---

## Automation

### GitHub Actions (Future)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build
        run: cargo build --release
      - name: Create DMG
        run: ./build-dmg.sh
      - name: Sign & Notarize
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          TEAM_ID: ${{ secrets.TEAM_ID }}
        run: ./sign-and-notarize.sh
      - name: Upload Release
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./ClipVault-${{ github.ref_name }}.dmg
          asset_name: ClipVault-${{ github.ref_name }}.dmg
          asset_content_type: application/octet-stream
```

---

## Version Numbering

### Semantic Versioning

Format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (1.0.0)
- **MINOR**: New features, backwards compatible (0.2.0)
- **PATCH**: Bug fixes (0.1.1)

### Examples

- `0.1.0` - Initial beta release
- `0.1.1` - Bug fix release
- `0.2.0` - New features (custom hotkeys, etc.)
- `1.0.0` - First stable release

### Build Numbers

CFBundleVersion increments for each build:
- `0.1.0` (version) → build 1
- `0.1.1` (version) → build 2
- `0.2.0` (version) → build 3

---

## File Naming

### DMG Files

Format: `ClipVault-{version}.dmg`

Examples:
- `ClipVault-0.1.0.dmg`
- `ClipVault-0.1.1.dmg`
- `ClipVault-1.0.0.dmg`

### App Bundle

Always: `ClipVault.app` (no version in name)

---

## Support & Troubleshooting

### User Reports Issues

1. **Check version**: "About ClipVault" in menu
2. **Check logs**: Console.app → filter "clipboard-manager"
3. **Verify signature**: `codesign --verify -vv /Applications/ClipVault.app`
4. **Check Gatekeeper**: `spctl --assess /Applications/ClipVault.app`

### Re-signing After Modification

If you modify the app after signing:

```bash
# Remove old signature
codesign --remove-signature ClipVault.app

# Re-sign
codesign --sign "Developer ID Application" ClipVault.app

# Re-notarize
# (repeat notarization process)
```

---

## Resources

- [Apple Code Signing Guide](https://developer.apple.com/support/code-signing/)
- [Apple Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [DMG Canvas](https://www.araelium.com/dmgcanvas) - Visual DMG creator
- [create-dmg](https://github.com/sindresorhus/create-dmg) - CLI DMG tool

---

Last updated: 2026-01-27
