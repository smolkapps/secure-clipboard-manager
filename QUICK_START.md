# ClipVault Quick Start

**For Users**: See [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
**For Developers**: See [CONTRIBUTING.md](CONTRIBUTING.md)
**For Release**: See [DISTRIBUTION.md](DISTRIBUTION.md)

---

## As a User

### Install
1. Download `ClipVault.dmg` from releases (coming soon)
2. Drag to Applications
3. Launch ClipVault

### Use
- **Global hotkey**: `Cmd+Shift+C` - Open clipboard history
- **Navigate**: Arrow keys or `j`/`k` (vim)
- **Select**: Press `Enter` to paste
- **Search**: Start typing to search

### Features
- ✅ Automatic clipboard monitoring
- ✅ Encrypted sensitive data (API keys, passwords)
- ✅ Image support (TIFF→PNG)
- ✅ Fuzzy search
- ✅ 7-day auto-cleanup

---

## As a Developer

### Setup
```bash
# Clone
git clone https://github.com/smolkapps/secure-clipboard-manager.git
cd secure-clipboard-manager/clipboard-manager

# Build
cargo build --release

# Test
cargo test --all

# Run
./target/release/clipboard-manager
```

### Project Structure
- `src/` - Source code
  - `clipboard/` - NSPasteboard monitoring
  - `storage/` - Database + encryption
  - `ui/` - Menu bar + popup window
- `tests/` - Integration tests
- `docs/` - User documentation

### Contributing
1. Fork repository
2. Create feature branch
3. Write tests
4. Submit PR

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## As a Maintainer

### Release Process

1. **Test**
```bash
cargo test --all
cargo bench
```

2. **Build**
```bash
cargo build --release
```

3. **Create App Bundle**
```bash
# See DISTRIBUTION.md for details
# Create ClipVault.app with Info.plist
```

4. **Sign & Notarize**
```bash
# Requires Apple Developer account
codesign --sign "Developer ID" ClipVault.app
xcrun notarytool submit ClipVault.zip --wait
```

5. **Create DMG**
```bash
./build-dmg.sh 0.1.0
```

6. **Release**
- Create GitHub release
- Upload DMG
- Write release notes

See [DISTRIBUTION.md](DISTRIBUTION.md) for complete guide.

---

## Testing

### Run All Tests
```bash
cargo test --all
```

### Run Specific Tests
```bash
cargo test --test test_storage_integration
cargo test --test test_sensitive_detection
cargo test --test test_image_processing
cargo test --test test_search_engine
```

### Run Benchmarks
```bash
cargo bench
```

See [tests/README.md](tests/README.md) for test documentation.

---

## Documentation Map

- **[README.md](README.md)** - Project overview
- **[docs/USER_GUIDE.md](docs/USER_GUIDE.md)** - Complete user guide
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development guide
- **[DISTRIBUTION.md](DISTRIBUTION.md)** - Release process
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
- **[tests/README.md](tests/README.md)** - Testing guide
- **[resources/ICON_README.md](resources/ICON_README.md)** - Icon requirements

---

## Key Features

### Privacy & Security
- ChaCha20-Poly1305 encryption
- Auto-detect sensitive data (API keys, passwords)
- Local-only storage (no cloud)
- 0600 key file permissions

### Performance
- <50MB RAM usage
- <100ms popup time
- <50ms search (1000 items)
- Rust native performance

### macOS Integration
- Native menu bar app
- NSPasteboard monitoring
- Global hotkey support
- macOS UI guidelines

---

## Common Tasks

### Add New Feature
1. Create feature branch
2. Write code + tests
3. Update documentation
4. Submit PR

### Fix Bug
1. Write failing test
2. Fix bug
3. Verify test passes
4. Submit PR

### Update Dependencies
```bash
cargo update
cargo test --all
```

### Check Code Quality
```bash
cargo fmt      # Format code
cargo clippy   # Lint code
cargo test     # Run tests
```

---

## Support

- **Issues**: [GitHub Issues](https://github.com/smolkapps/secure-clipboard-manager/issues)
- **Email**: support@smolkapps.com
- **Docs**: Full documentation in repo

---

## Quick Links

- [Project README](README.md)
- [User Guide](docs/USER_GUIDE.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Distribution Guide](DISTRIBUTION.md)
- [Test Documentation](tests/README.md)
- [Changelog](CHANGELOG.md)

---

**Version**: 0.1.0
**Last Updated**: 2026-01-27
