# Changelog

All notable changes to ClipVault will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 9: Distribution Prep & Testing (In Progress)

#### Added
- Comprehensive integration test suite (50+ tests)
  - Storage integration tests (database + encryption)
  - Sensitive data detection tests
  - Image processing tests (TIFF→PNG, thumbnails)
  - Fuzzy search engine tests
  - Clipboard monitoring tests (headless-safe)
- Complete user documentation (USER_GUIDE.md)
  - Installation instructions
  - Feature overview
  - Keyboard shortcuts reference
  - Security & privacy details
  - Troubleshooting guide
  - FAQ section
- Developer documentation
  - CONTRIBUTING.md with development setup
  - DISTRIBUTION.md with release process
  - Test documentation (tests/README.md)
  - Icon requirements guide
- Distribution infrastructure
  - DMG build script (build-dmg.sh)
  - Code signing documentation
  - Notarization process guide
  - App icon placeholder structure

#### Changed
- Updated README with distribution information
- Enhanced test coverage documentation
- Improved project documentation structure

---

## [0.1.0] - 2026-01-27 (Phase 8 Complete)

### Added

#### Phase 8: Popup Window UI & Keyboard Navigation
- Global hotkey support (Cmd+Shift+C)
- Popup window with clipboard history
- Keyboard navigation (arrow keys and vim keys)
  - ↑/k - Move selection up
  - ↓/j - Move selection down
  - Enter - Paste selected item
  - Escape - Close window
- Click-to-paste functionality
- Automatic decryption for encrypted items
- Window positioning (centered on screen)
- Visual selection indicators

#### Phase 7: Image Processing & Optimization
- TIFF to PNG conversion for screenshots
- Automatic thumbnail generation (200x200px)
- Image compression optimization
- Aspect ratio preservation
- Metadata extraction (dimensions, format)
- Image preview text generation

#### Core Features
- Background clipboard monitoring (500ms polling)
- NSPasteboard integration for text and images
- SQLite database with blob storage
- ChaCha20-Poly1305 encryption for sensitive data
- Automatic sensitive data detection:
  - API keys (OpenAI, GitHub, AWS, Google, Stripe, etc.)
  - JWT tokens
  - Private keys (PEM format)
  - Passwords and secrets
  - Database connection strings
- Fuzzy search engine with relevance scoring
- Menu bar integration
- 7-day automatic cleanup
- Data type detection (text, URL, image, etc.)

### Technical Details

#### Architecture
- Native macOS app using Cacao and objc2
- Rust for performance and safety
- Async runtime with Tokio
- Event-driven clipboard monitoring
- Secure key storage with 0600 permissions

#### Performance
- Binary size: ~14MB (stripped)
- Memory usage: <50MB runtime
- Popup time: <100ms
- Search speed: <50ms for 1000 items
- Image processing: <50ms thumbnail generation
- TIFF→PNG compression: 50-70% size reduction

#### Dependencies
- cacao 0.4.0-beta2 - macOS native UI
- objc2 0.5 - Objective-C bindings
- tokio 1.x - Async runtime
- rusqlite 0.32 - SQLite database
- chacha20poly1305 0.10 - Encryption
- fuzzy-matcher 0.3 - Search
- image 0.25 - Image processing
- global-hotkey 0.6 - Hotkey handling

### Security
- ChaCha20-Poly1305 (AEAD) encryption
- 256-bit encryption keys
- Random nonce per encryption
- Secure key file permissions (0600)
- Pattern-based sensitive data detection
- Local-only storage (no network)

### Known Issues
- Intel only (Apple Silicon support planned)
- Fixed hotkey (customization planned)
- No app exclusion list yet
- Generic app icon (custom icon needed)

---

## Release Notes Format

For future releases:

### [Version] - YYYY-MM-DD

#### Added
- New features

#### Changed
- Changes to existing functionality

#### Deprecated
- Features marked for removal

#### Removed
- Removed features

#### Fixed
- Bug fixes

#### Security
- Security improvements

---

## Versioning Policy

- **0.x.x** - Beta/development versions
- **1.0.0** - First stable release
- **1.x.x** - Stable releases with new features
- **x.x.1** - Patch releases (bug fixes only)

---

Last updated: 2026-01-27
