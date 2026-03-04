# TODO: Mac App Store Publishing Setup

## Status
Build check incomplete — `cargo check` timed out (likely cold compile of dependencies). Need to verify the build is clean before setting up CI/CD.

## Steps Required

### 1. Verify Build
```bash
cd /Users/Shared/projects/2026/secure-clipboard-manager
cargo check  # or cargo build --release
```

### 2. If Build is Clean — Set Up GitHub Actions
Create `.github/workflows/mac-appstore.yml` with:
- Build macOS binary with `cargo build --release`
- Code sign with Developer ID or Mac App Store certificate
- Create `.app` bundle and `.pkg` installer
- Upload to Mac App Store via `xcrun altool` or Transporter

### 3. GitHub Secrets Already Set
The following shared secrets are already on `smolkapps/secure-clipboard-manager`:
- ✅ `APPSTORE_API_KEY_ID`
- ✅ `APPSTORE_ISSUER_ID`
- ✅ `APPSTORE_API_PRIVATE_KEY`
- ✅ `KEYCHAIN_PASSWORD`
- ✅ `BUILD_CERTIFICATE_BASE64`
- ✅ `P12_PASSWORD`

Wait — these were set for iOS. Check below.

### 4. Mac App Store Differences from iOS
- Needs a **Mac App Distribution** certificate (may differ from iOS distribution cert)
- Needs a **Mac Installer Distribution** certificate for `.pkg`
- Needs a Mac-specific **provisioning profile**
- Needs an `.entitlements` file for sandboxing (Mac App Store requirement)
- The existing `release.yml` workflow may already handle some of this — review it first

### 5. Existing Workflow
There's already a `.github/workflows/release.yml` — review what it does before creating a new one. May just need modifications.

### 6. Mac App Store Checklist
- [ ] Verify build compiles cleanly
- [ ] Review existing `release.yml` workflow
- [ ] Check if Mac-specific signing certificates exist
- [ ] Create Mac provisioning profile in Apple Developer Portal
- [ ] Set up `.entitlements` for App Sandbox
- [ ] Create or update GitHub Actions workflow
- [ ] Create App Store Connect record for the Mac app

---
*Created 2026-03-04 by Will*
