# ClipVault Documentation Index

Complete guide to all ClipVault documentation.

---

## For Users 👤

### Getting Started
- **[README.md](README.md)** - Project overview and quick start
- **[QUICK_START.md](QUICK_START.md)** - Quick reference guide
- **[docs/USER_GUIDE.md](docs/USER_GUIDE.md)** - Complete user manual

### Features & Usage
- Installation instructions
- Keyboard shortcuts
- Security & privacy
- Troubleshooting
- FAQ

**Start here**: [User Guide](docs/USER_GUIDE.md)

---

## For Developers 💻

### Development Setup
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Developer guide
  - Environment setup
  - Project structure
  - Code style guide
  - PR process

### Testing
- **[tests/README.md](tests/README.md)** - Testing guide
  - How to run tests
  - Test coverage
  - Writing new tests
  - CI/CD integration

### Code Organization
```
src/
├── clipboard/    # NSPasteboard monitoring
├── storage/      # Database + encryption
├── ui/           # Menu bar + popup
└── search/       # Fuzzy search
```

**Start here**: [Contributing Guide](CONTRIBUTING.md)

---

## For Maintainers 🔧

### Release Process
- **[DISTRIBUTION.md](DISTRIBUTION.md)** - Complete release guide
  - Build process
  - Code signing
  - Notarization
  - DMG creation
  - Release checklist

### Build Tools
- **[build-dmg.sh](build-dmg.sh)** - DMG creation script
- **[Cargo.toml](Cargo.toml)** - Rust dependencies

### App Resources
- **[resources/ICON_README.md](resources/ICON_README.md)** - Icon requirements

**Start here**: [Distribution Guide](DISTRIBUTION.md)

---

## Project Information 📋

### Version History
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and release notes

### Phase Reports
- **[PHASE9_MASTER_SUMMARY.md](PHASE9_MASTER_SUMMARY.md)** - Phase 9 overview
- **[PHASE9_COMPLETION_REPORT.md](PHASE9_COMPLETION_REPORT.md)** - Phase 9 details
- **[PHASE8_MASTER_REPORT.md](PHASE8_MASTER_REPORT.md)** - Phase 8 report

---

## Technical Documentation 🔬

### Testing
| Document | Description |
|----------|-------------|
| [tests/README.md](tests/README.md) | Test documentation and guide |
| [tests/test_storage_integration.rs](tests/test_storage_integration.rs) | Database + encryption tests |
| [tests/test_sensitive_detection.rs](tests/test_sensitive_detection.rs) | Security detection tests |
| [tests/test_image_processing.rs](tests/test_image_processing.rs) | Image processing tests |
| [tests/test_search_engine.rs](tests/test_search_engine.rs) | Search engine tests |
| [tests/test_clipboard_monitoring.rs](tests/test_clipboard_monitoring.rs) | Clipboard tests |

### Benchmarks
| Document | Description |
|----------|-------------|
| [benches/benchmarks.rs](benches/benchmarks.rs) | Performance benchmarks |

---

## Documentation by Purpose

### 📖 Learning ClipVault
1. [README.md](README.md) - Start here
2. [QUICK_START.md](QUICK_START.md) - Quick reference
3. [docs/USER_GUIDE.md](docs/USER_GUIDE.md) - Complete guide

### 🛠️ Developing ClipVault
1. [CONTRIBUTING.md](CONTRIBUTING.md) - Setup and guidelines
2. [tests/README.md](tests/README.md) - Testing guide
3. Source code comments

### 🚀 Releasing ClipVault
1. [DISTRIBUTION.md](DISTRIBUTION.md) - Release process
2. [build-dmg.sh](build-dmg.sh) - Build script
3. [CHANGELOG.md](CHANGELOG.md) - Version history

### 🎨 Branding ClipVault
1. [resources/ICON_README.md](resources/ICON_README.md) - Icon guide

---

## Documentation by Role

### I'm a User
- [User Guide](docs/USER_GUIDE.md)
- [Quick Start](QUICK_START.md)
- [FAQ](docs/USER_GUIDE.md#faq)

### I'm a Contributor
- [Contributing Guide](CONTRIBUTING.md)
- [Test Documentation](tests/README.md)
- [Code Style](CONTRIBUTING.md#code-style-guide)

### I'm a Maintainer
- [Distribution Guide](DISTRIBUTION.md)
- [Release Checklist](DISTRIBUTION.md#release-checklist)
- [Build Scripts](build-dmg.sh)

### I'm a Designer
- [Icon Requirements](resources/ICON_README.md)
- [Brand Guidelines](resources/ICON_README.md#design-guidelines)

---

## Quick Access

### Installation
- [User Guide - Installation](docs/USER_GUIDE.md#installation)
- [Contributing - Dev Setup](CONTRIBUTING.md#development-setup)

### Usage
- [User Guide - Quick Start](docs/USER_GUIDE.md#quick-start)
- [User Guide - Keyboard Shortcuts](docs/USER_GUIDE.md#keyboard-shortcuts)

### Security
- [User Guide - Security](docs/USER_GUIDE.md#security--privacy)
- [README - Security Features](README.md#security-features)

### Troubleshooting
- [User Guide - Troubleshooting](docs/USER_GUIDE.md#troubleshooting)
- [User Guide - FAQ](docs/USER_GUIDE.md#faq)

### Development
- [Contributing - Code Style](CONTRIBUTING.md#code-style-guide)
- [Contributing - Testing](CONTRIBUTING.md#testing)
- [Contributing - PR Process](CONTRIBUTING.md#pull-request-process)

### Release
- [Distribution - Build Process](DISTRIBUTION.md#build-process)
- [Distribution - Code Signing](DISTRIBUTION.md#code-signing)
- [Distribution - Notarization](DISTRIBUTION.md#notarization)

---

## Documentation Stats

### Total Documentation
- **Lines**: ~5,000+ lines
- **Files**: 17 documentation files
- **Coverage**: User, Developer, Maintainer

### By Category
- **User Documentation**: ~1,000 lines
- **Developer Documentation**: ~1,500 lines
- **Technical Documentation**: ~2,500 lines

### Languages
- Markdown (documentation)
- Rust (code)
- Shell (scripts)

---

## Contributing to Documentation

### How to Improve Docs
1. Find documentation gap
2. Create issue
3. Write improvement
4. Submit PR

### Documentation Standards
- Clear, concise language
- Code examples
- Screenshots (where helpful)
- Updated version numbers

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## External Resources

### Rust
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### macOS Development
- [Apple Developer](https://developer.apple.com)
- [macOS Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/macos)

### Tools
- [Criterion.rs](https://github.com/bheisler/criterion.rs) - Benchmarking
- [tempfile](https://docs.rs/tempfile/) - Test isolation

---

## Version

**Documentation Version**: 0.1.0
**Last Updated**: 2026-01-27
**Status**: Complete

---

## Quick Links

| Document | Purpose | Audience |
|----------|---------|----------|
| [README](README.md) | Project overview | Everyone |
| [User Guide](docs/USER_GUIDE.md) | How to use | Users |
| [Contributing](CONTRIBUTING.md) | How to develop | Developers |
| [Distribution](DISTRIBUTION.md) | How to release | Maintainers |
| [Quick Start](QUICK_START.md) | Quick reference | Everyone |
| [Changelog](CHANGELOG.md) | Version history | Everyone |
| [Tests](tests/README.md) | Testing guide | Developers |
| [Icon Guide](resources/ICON_README.md) | Icon requirements | Designers |

---

**Need help?** Check the [User Guide FAQ](docs/USER_GUIDE.md#faq) or [create an issue](https://github.com/smolkapps/secure-clipboard-manager/issues).
