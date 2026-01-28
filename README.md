# 🔐 ClipVault - Secure Clipboard Manager for macOS

Privacy-focused clipboard manager for macOS with automatic encryption. Built in Rust for maximum performance and security.

## ✨ Features

- **🔒 Auto-Encryption**: Automatically detects and encrypts sensitive data (API keys, passwords, tokens)
- **🔍 Fuzzy Search**: Fast search across your clipboard history
- **📋 Menu Bar Integration**: Quick access from your menu bar
- **⚡ High Performance**: <50MB RAM usage, <100ms popup time
- **🎯 7-Day Retention**: Privacy-focused automatic cleanup
- **🖼️ Image Support**: TIFF→PNG conversion (unique feature!)

## 🚀 Why ClipVault?

| Feature | ClipVault | Alfred | Raycast | Paste |
|---------|-----------|--------|---------|-------|
| Privacy-First Encryption | ✅ | ❌ | ❌ | ⚠️ |
| Native Performance (Rust) | ✅ | ⚠️ | ⚠️ | ❌ |
| One-Time Purchase | ✅ | ✅ | Free | Subscription |
| Auto-Detect Sensitive Data | ✅ | ❌ | ❌ | ❌ |
| Image Optimization | ✅ | ⚠️ | ⚠️ | ✅ |

**Like Ditto for Windows?** This is your Mac alternative!

## 📦 Installation

### Option 1: Download DMG (Coming Soon)

1. Download `ClipVault.dmg` from [releases](https://github.com/smolkapps/secure-clipboard-manager/releases)
2. Open the DMG and drag ClipVault to Applications
3. Launch ClipVault from Applications

**Note**: Beta releases available soon. Currently in development.

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/smolkapps/secure-clipboard-manager.git
cd secure-clipboard-manager/clipboard-manager

# Build and run
cargo build --release
./target/release/clipboard-manager
```

### Requirements

- **macOS**: 12.7.5 or later
- **Architecture**: Intel Mac (Apple Silicon support coming soon)
- **For building**: Rust 1.92.0+, Xcode Command Line Tools

## 🎯 Usage

1. **Launch**: Run the app and look for the 📋 icon in your menu bar
2. **Click the icon**: See your recent clipboard history (last 10 items)
3. **Monitor**: App runs in background, automatically saving clipboard items
4. **Secure**: Sensitive data (API keys, passwords) automatically encrypted

### Current Features (v0.1.0)

- ✅ Background clipboard monitoring (text + images)
- ✅ SQLite storage with encryption
- ✅ Menu bar icon with history
- ✅ Fuzzy search engine
- ✅ Sensitive data detection
- ✅ Image support with TIFF→PNG conversion
- ✅ Automatic thumbnail generation (200x200px)
- ✅ Global hotkey (Cmd+Shift+C)
- ✅ Popup window UI with keyboard navigation
- ✅ Click-to-paste functionality

**Phase 8 Complete**: All core features implemented!

## 🔒 Security Features

### Automatic Sensitive Data Detection

ClipVault automatically detects and encrypts:
- API Keys (OpenAI, GitHub, AWS, Google, etc.)
- JWT Tokens
- Private Keys (PEM format)
- Password-like strings
- Environment variables with secrets

### Encryption

- **Algorithm**: ChaCha20-Poly1305 (AEAD)
- **Key Storage**: Secure local storage with 0600 permissions
- **Nonce**: Random per-encryption (unique ciphertext)

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│         Menu Bar App                │
│         (Cacao + objc2)             │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│      Clipboard Monitor              │
│      (NSPasteboard polling)         │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│         Storage Engine              │
│    (SQLite + ChaCha20-Poly1305)     │
└─────────────────────────────────────┘
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run specific test suite
cargo test --test test_storage_integration
cargo test --test test_sensitive_detection
cargo test --test test_image_processing
cargo test --test test_search_engine
cargo test --test test_clipboard_monitoring

# Run with output
cargo test -- --nocapture
```

**Test Coverage**:
- ✅ 50+ integration tests covering critical paths
- ✅ Storage layer (database + encryption)
- ✅ Sensitive data detection
- ✅ Image processing (TIFF→PNG, thumbnails)
- ✅ Fuzzy search engine
- ✅ Clipboard monitoring (headless-safe)

See [tests/README.md](tests/README.md) for detailed testing documentation.

## 📊 Performance

- **Memory Usage**: ~14MB binary, <50MB runtime
- **Database Size**: ~36KB for typical usage
- **Search Speed**: <50ms for fuzzy search
- **Clipboard Detection**: 500ms polling interval
- **Image Processing**: <50ms thumbnail generation
- **TIFF→PNG Compression**: Typical 50-70% size reduction

## 📚 Documentation

- **[User Guide](docs/USER_GUIDE.md)** - Complete user documentation
- **[Contributing Guide](CONTRIBUTING.md)** - Development setup and guidelines
- **[Distribution Guide](DISTRIBUTION.md)** - Build, sign, and package for release
- **[Test Documentation](tests/README.md)** - Testing guidelines and coverage
- **[Icon Guide](resources/ICON_README.md)** - App icon requirements

## 🛠️ Development

Built with:
- **Rust 1.92.0**
- **Cacao 0.4.0-beta2** (native macOS UI)
- **objc2** (Objective-C bindings)
- **rusqlite** (SQLite database)
- **chacha20poly1305** (encryption)
- **fuzzy-matcher** (search)

## 📝 Roadmap

### ✅ Phase 8: Polish & Performance (COMPLETED)
- [x] Global hotkey (Cmd+Shift+C)
- [x] Popup window UI with keyboard navigation
- [x] Click-to-paste functionality
- [x] Performance benchmarks

### Phase 9: Distribution Prep (IN PROGRESS)
- [x] Comprehensive integration tests (50+ tests)
- [x] User documentation (USER_GUIDE.md)
- [x] Distribution documentation (DISTRIBUTION.md)
- [x] DMG build script
- [ ] App icon design
- [ ] Performance benchmarks
- [ ] Beta release

### Future Releases
- [ ] Apple Silicon (M1/M2/M3) universal binary
- [ ] Customizable hotkeys
- [ ] Optional cloud sync (encrypted)
- [ ] App exclusion list
- [ ] Code syntax highlighting
- [ ] Mac App Store submission

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup instructions
- Code style guide
- Testing guidelines
- Pull request process

**Quick Start for Contributors**:
```bash
# Clone and build
git clone https://github.com/smolkapps/secure-clipboard-manager.git
cd secure-clipboard-manager/clipboard-manager
cargo build

# Run tests
cargo test --all

# Format code
cargo fmt

# Run linter
cargo clippy
```

## 📄 License

TBD - Commercial one-time purchase planned

## 🙏 Acknowledgments

Inspired by:
- **Ditto** (Windows) - The gold standard for clipboard managers
- **Alfred** (macOS) - Powerful workflow automation
- **Raycast** (macOS) - Modern launcher with clipboard features

---

**Built with ❤️ in Rust for macOS**

⚠️ **Development Status**: Currently in active development. Basic functionality working, UI features in progress.
