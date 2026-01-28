# Beta Release Checklist (v0.1.0)

Use this checklist to track progress toward beta release.

**Target**: ClipVault v0.1.0 Beta
**Status**: Phase 9 Complete - Ready for Testing & Release

---

## Phase 9: Distribution Prep ✅ COMPLETE

- [x] Write integration tests (50+ tests)
- [x] Create test documentation
- [x] Write user guide (USER_GUIDE.md)
- [x] Write developer guide (CONTRIBUTING.md)
- [x] Write distribution guide (DISTRIBUTION.md)
- [x] Create DMG build script
- [x] Document code signing process
- [x] Document notarization process
- [x] Create app icon infrastructure
- [x] Create performance benchmarks
- [x] Update README
- [x] Create changelog

**Phase 9**: ✅ 100% Complete

---

## Pre-Release Tasks

### Testing (Week 1)

- [ ] **Execute integration tests**
  ```bash
  cd clipboard-manager
  cargo test --all
  ```
  - [ ] All tests pass
  - [ ] No errors or warnings
  - [ ] Document any failures

- [ ] **Run performance benchmarks**
  ```bash
  cargo bench
  ```
  - [ ] Search <50ms (1000 items)
  - [ ] Database insert <5ms
  - [ ] Encryption <1ms
  - [ ] Image processing <50ms
  - [ ] Document results

- [ ] **Manual testing**
  - [ ] Launch app (menu bar icon appears)
  - [ ] Copy text (saved to database)
  - [ ] Copy image (TIFF→PNG works)
  - [ ] Copy API key (auto-encrypted)
  - [ ] Open popup (Cmd+Shift+C)
  - [ ] Navigate with arrow keys
  - [ ] Navigate with vim keys (j/k)
  - [ ] Search clipboard history
  - [ ] Paste from history (Enter)
  - [ ] Close with Escape
  - [ ] Verify encryption in database
  - [ ] Check memory usage (<50MB)
  - [ ] Check CPU usage (minimal)
  - [ ] Test for 24 hours (stability)

### Assets & Setup (Week 2)

- [ ] **Create app icon**
  - [ ] Design 1024x1024 source image
  - [ ] Export all required sizes (16-1024px)
  - [ ] Create AppIcon.iconset directory
  - [ ] Generate AppIcon.icns file
  - [ ] Test icon in app bundle
  - [ ] Verify icon looks good in:
    - [ ] Menu bar
    - [ ] Finder
    - [ ] About dialog

- [ ] **Apple Developer Account**
  - [ ] Sign up for Apple Developer Program ($99/year)
  - [ ] Verify email and complete setup
  - [ ] Access developer portal
  - [ ] Note Team ID

- [ ] **Code Signing Certificate**
  - [ ] Generate Developer ID Application certificate
  - [ ] Download certificate
  - [ ] Install in Keychain
  - [ ] Verify with: `security find-identity -v -p codesigning`

### Build & Sign (Week 3)

- [ ] **Build release binary**
  ```bash
  cargo clean
  cargo build --release
  ```
  - [ ] Verify binary size (~14MB)
  - [ ] Test binary launches
  - [ ] No debug symbols (stripped)

- [ ] **Create app bundle**
  - [ ] Create ClipVault.app structure
  - [ ] Copy binary to Contents/MacOS/
  - [ ] Copy icon to Contents/Resources/
  - [ ] Create Info.plist
  - [ ] Set bundle identifier
  - [ ] Set version (0.1.0)
  - [ ] Set minimum macOS version
  - [ ] Test app bundle launches

- [ ] **Sign app bundle**
  ```bash
  codesign --force --sign "Developer ID Application: [Name]" \
           --options runtime --timestamp ClipVault.app
  ```
  - [ ] Sign binary first
  - [ ] Sign app bundle
  - [ ] Verify signature: `codesign --verify --deep --strict -v ClipVault.app`
  - [ ] Test Gatekeeper: `spctl --assess --type execute -v ClipVault.app`

- [ ] **Notarize app**
  - [ ] Create app-specific password (appleid.apple.com)
  - [ ] Create ZIP: `ditto -c -k --keepParent ClipVault.app ClipVault.zip`
  - [ ] Submit for notarization:
    ```bash
    xcrun notarytool submit ClipVault.zip \
        --apple-id "your@email.com" \
        --password "app-password" \
        --team-id "TEAM_ID" \
        --wait
    ```
  - [ ] Wait for approval (usually <5 minutes)
  - [ ] Staple ticket: `xcrun stapler staple ClipVault.app`
  - [ ] Verify: `xcrun stapler validate ClipVault.app`

### Distribution (Week 4)

- [ ] **Create DMG installer**
  ```bash
  ./build-dmg.sh 0.1.0
  ```
  - [ ] DMG created successfully
  - [ ] Test DMG opens
  - [ ] Applications symlink works
  - [ ] Window layout correct
  - [ ] DMG size reasonable (<20MB)

- [ ] **Sign DMG** (optional)
  ```bash
  codesign --sign "Developer ID Application: [Name]" ClipVault-0.1.0.dmg
  ```
  - [ ] DMG signed
  - [ ] Verify signature

- [ ] **Test DMG installation**
  - [ ] Open DMG
  - [ ] Drag to Applications
  - [ ] Launch from Applications
  - [ ] All features work
  - [ ] No Gatekeeper warnings
  - [ ] Test on clean macOS install

### Release (Week 4)

- [ ] **Prepare release notes**
  - [ ] Write feature list
  - [ ] Write known issues
  - [ ] Write upgrade instructions
  - [ ] Write credits/acknowledgments

- [ ] **Create GitHub release**
  - [ ] Create tag: v0.1.0
  - [ ] Upload DMG file
  - [ ] Write release description
  - [ ] Mark as pre-release (beta)
  - [ ] Publish release

- [ ] **Update documentation**
  - [ ] Update README with download link
  - [ ] Update CHANGELOG
  - [ ] Update version in Cargo.toml
  - [ ] Update version in Info.plist

- [ ] **Announce release**
  - [ ] Tweet/social media
  - [ ] Email beta testers
  - [ ] Post on relevant forums
  - [ ] Update project website (if exists)

### Post-Release

- [ ] **Monitor feedback**
  - [ ] Watch GitHub issues
  - [ ] Respond to bug reports
  - [ ] Track feature requests
  - [ ] Collect user feedback

- [ ] **Plan v0.2.0**
  - [ ] Review top feature requests
  - [ ] Identify critical bugs
  - [ ] Plan next sprint
  - [ ] Update roadmap

---

## Release Criteria

Beta release is ready when:

- ✅ All tests passing
- ✅ Performance targets met
- ✅ App icon complete
- ✅ Code signed and notarized
- ✅ DMG tested on clean install
- ✅ No critical bugs
- ✅ Documentation complete
- ✅ Release notes written

---

## Testing Checklist

### Functional Tests

- [ ] Clipboard monitoring works
- [ ] Text items saved correctly
- [ ] Image items saved and converted
- [ ] Sensitive data encrypted
- [ ] Menu bar icon functional
- [ ] Popup window opens (Cmd+Shift+C)
- [ ] Keyboard navigation works
- [ ] Search finds items
- [ ] Paste works correctly
- [ ] Auto-cleanup works (7 days)

### Security Tests

- [ ] API keys detected and encrypted
- [ ] Passwords detected and encrypted
- [ ] Encryption key has 0600 permissions
- [ ] Database not readable without key
- [ ] No sensitive data in logs
- [ ] No network requests made

### Performance Tests

- [ ] App launches <2 seconds
- [ ] Popup opens <100ms
- [ ] Search <50ms (1000 items)
- [ ] Memory usage <50MB
- [ ] CPU usage minimal (<5%)
- [ ] No memory leaks (24hr test)
- [ ] Database size reasonable

### Compatibility Tests

- [ ] macOS 12.7.5+
- [ ] Intel Mac
- [ ] Works on multiple Macs
- [ ] Survives system restart
- [ ] Survives logout/login

---

## Documentation Checklist

- [x] README.md complete
- [x] USER_GUIDE.md complete
- [x] CONTRIBUTING.md complete
- [x] DISTRIBUTION.md complete
- [x] CHANGELOG.md complete
- [x] Tests documented
- [x] Icon guide complete
- [ ] Screenshots added (pending icon)
- [ ] Video tutorial (optional)

---

## Known Limitations (v0.1.0)

Document these in release notes:

- Intel only (Apple Silicon coming in v0.2.0)
- Fixed hotkey (customization in future)
- No app exclusion list yet
- No manual item deletion
- No cloud sync
- No snippet templates

---

## Support Plan

- [ ] Set up issue templates
- [ ] Create bug report template
- [ ] Create feature request template
- [ ] Set up support email
- [ ] Create FAQ (already done)
- [ ] Monitor GitHub discussions

---

## Marketing Plan (Optional)

- [ ] Create product website
- [ ] Design marketing materials
- [ ] Write blog post
- [ ] Submit to MacUpdate/Softpedia
- [ ] Post on Hacker News
- [ ] Post on Reddit (r/macapps)
- [ ] Reach out to tech bloggers

---

## Backup Plan

Before release:
- [ ] Backup source code
- [ ] Backup certificates
- [ ] Backup signing keys
- [ ] Document recovery process

---

## Emergency Contacts

In case of issues:
- **Apple Developer Support**: developer.apple.com/support
- **Notarization Issues**: See DISTRIBUTION.md
- **Code Signing Issues**: See DISTRIBUTION.md

---

## Success Metrics

Track after release:
- [ ] Download count
- [ ] GitHub stars
- [ ] Issue count
- [ ] Active users (if telemetry added)
- [ ] User feedback sentiment

---

## Version 0.2.0 Planning

Ideas for next release:
- [ ] Apple Silicon support (universal binary)
- [ ] Customizable hotkeys
- [ ] App exclusion list
- [ ] Adjustable retention period
- [ ] Manual item deletion
- [ ] Pin favorite items
- [ ] Export/import history

---

## Notes

Use this space for notes during release process:

```
[Add your notes here as you work through the checklist]
```

---

**Checklist Version**: 1.0
**Last Updated**: 2026-01-27
**Target Release**: TBD (4-6 weeks)

---

## Quick Commands Reference

```bash
# Testing
cargo test --all
cargo bench

# Building
cargo clean
cargo build --release

# Code signing
codesign --sign "Developer ID" ClipVault.app
xcrun stapler staple ClipVault.app

# DMG creation
./build-dmg.sh 0.1.0

# Verification
codesign --verify -vv ClipVault.app
spctl --assess -vv ClipVault.app
```

---

**Ready to start?** Begin with the Testing section! ✅
