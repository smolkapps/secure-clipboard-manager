# ClipVault User Guide

Welcome to ClipVault - the privacy-focused clipboard manager for macOS!

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Features](#features)
4. [Keyboard Shortcuts](#keyboard-shortcuts)
5. [Security & Privacy](#security--privacy)
6. [Troubleshooting](#troubleshooting)
7. [FAQ](#faq)

---

## Installation

### Option 1: Download DMG (Recommended)

1. Download `ClipVault.dmg` from the releases page
2. Open the DMG file
3. Drag ClipVault to your Applications folder
4. Launch ClipVault from Applications

### Option 2: Build from Source

```bash
# Requirements: macOS 12.7.5+, Rust 1.92.0+
git clone https://github.com/smolkapps/secure-clipboard-manager.git
cd secure-clipboard-manager/clipboard-manager
cargo build --release
./target/release/clipboard-manager
```

### First Launch

On first launch, macOS may show a security warning:

1. Go to **System Preferences → Security & Privacy**
2. Click **"Open Anyway"** next to ClipVault
3. Confirm you want to open the app

ClipVault will appear in your menu bar with a 📋 icon.

---

## Quick Start

### 1. Launch ClipVault

Double-click ClipVault in Applications. The app runs in the background with a menu bar icon.

### 2. Copy Something

Copy any text or image using `Cmd+C` as normal. ClipVault automatically saves it.

### 3. Access Your History

**Method 1: Menu Bar**
- Click the 📋 icon in your menu bar
- See your last 10 clipboard items
- Click any item to view details

**Method 2: Global Hotkey** (Recommended)
- Press `Cmd+Shift+C` anywhere
- Popup window shows your clipboard history
- Use arrow keys or `j/k` to navigate
- Press `Enter` to paste selected item

### 4. Search Your History

In the popup window:
- Type to search across all clipboard items
- Fuzzy matching finds items even with typos
- Results sorted by relevance

---

## Features

### Automatic Clipboard Monitoring

ClipVault monitors your clipboard every 500ms and automatically saves:

- ✅ **Plain text** - Code, notes, URLs, etc.
- ✅ **Images** - Screenshots, copied images
- ✅ **Rich text** - Formatted content
- ✅ **URLs** - Auto-detected and tagged

### Privacy-First Encryption

ClipVault automatically detects and encrypts sensitive data:

**Auto-encrypted content:**
- API keys (OpenAI, GitHub, AWS, Google, Stripe, etc.)
- JWT tokens
- Private keys (PEM format)
- Passwords and password-like strings
- Database connection strings
- Environment variables with secrets

**Security features:**
- ChaCha20-Poly1305 encryption (industry standard)
- Unique encryption key per installation
- Key stored with 0600 permissions (owner-only access)
- Random nonce per encryption (unique ciphertext)

### Smart Image Handling

**TIFF → PNG Conversion**
- macOS screenshots are TIFF (large files)
- ClipVault converts to optimized PNG
- Typical 50-70% size reduction
- Original quality preserved

**Automatic Thumbnails**
- 200x200px thumbnails for quick preview
- Aspect ratio preserved
- Fast loading in UI

### Fuzzy Search

Find anything in your clipboard history:

- **Fast**: <50ms search across 1000+ items
- **Smart**: Handles typos and partial matches
- **Ranked**: Most relevant results first
- **Unicode**: Works with any language

### 7-Day Auto-Cleanup

Privacy-focused retention:
- Items automatically deleted after 7 days
- Keeps your history lean and private
- Configurable (future version)

---

## Keyboard Shortcuts

### Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+C` | Open clipboard history popup |
| `Cmd+C` | Copy (system default, monitored) |
| `Cmd+V` | Paste (system default) |

### In Popup Window

| Shortcut | Action |
|----------|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Paste selected item and close |
| `Escape` | Close popup |
| Type text | Search clipboard history |

**Vim users:** Use `j`/`k` for navigation!

---

## Security & Privacy

### What Data Does ClipVault Store?

**Stored locally:**
- Clipboard text content
- Images (converted to PNG)
- Timestamps
- Data type metadata

**Storage location:**
```
~/Library/Application Support/com.smolkapps.clipboard-manager/
├── clipboard.db          # SQLite database
└── encryption.key        # Encryption key (0600 permissions)
```

**NOT stored:**
- No cloud sync (future opt-in feature)
- No analytics or telemetry
- No external network requests

### Encryption Details

**Algorithm**: ChaCha20-Poly1305 (AEAD)
- Authenticated encryption with associated data
- Same algorithm used by TLS 1.3
- IETF RFC 8439 standard

**Key Management**:
- 256-bit encryption key
- Generated on first launch
- Stored with 0600 permissions (owner read/write only)
- Never transmitted or shared

**Sensitive Data Detection**:
- Pattern-based detection (regex)
- Conservative approach (better to encrypt unnecessarily)
- Encrypted items marked with 🔒 icon

### Privacy Settings

**Current (v0.1.0):**
- 7-day retention (automatic cleanup)
- Local-only storage
- No network access

**Future (planned):**
- Configurable retention period
- Optional cloud sync (encrypted)
- Exclude specific apps from monitoring

---

## Troubleshooting

### ClipVault Won't Launch

**Error: "Cannot be opened because the developer cannot be verified"**

Solution:
1. Open **System Preferences → Security & Privacy**
2. Click **"Open Anyway"** next to ClipVault message
3. Relaunch ClipVault

**Error: "Damaged and can't be opened"**

This is a macOS Gatekeeper warning. Solution:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /Applications/ClipVault.app
```

### Clipboard Items Not Saving

1. **Check menu bar icon**: Is ClipVault running?
2. **Check permissions**: Does ClipVault have Accessibility permissions?
   - System Preferences → Security & Privacy → Accessibility
   - Enable ClipVault
3. **Check disk space**: Ensure you have free disk space
4. **Restart ClipVault**: Quit and relaunch

### Popup Window Not Showing

1. **Check hotkey conflict**: Another app might use `Cmd+Shift+C`
2. **Check Accessibility permissions**: Required for global hotkeys
3. **Try menu bar**: Click 📋 icon to verify app is working

### Search Not Finding Items

1. **Check spelling**: Use fuzzy search (slight typos okay)
2. **Check retention**: Items auto-delete after 7 days
3. **Check preview**: Search works on preview text only

### High Memory Usage

ClipVault should use <50MB RAM. If higher:

1. **Check item count**: Database might have many items
2. **Check image sizes**: Large images use more memory
3. **Manual cleanup**: Run cleanup in preferences (future feature)

### Performance Issues

**Popup slow to open:**
- Expected: <100ms
- If slower: Check item count and image sizes

**Search slow:**
- Expected: <50ms for 1000 items
- If slower: Report issue with item count

---

## FAQ

### General

**Q: Is ClipVault free?**
A: Currently yes (beta). Future: One-time purchase ($15-20), no subscription.

**Q: Does it work on Apple Silicon?**
A: Current version is Intel only. Apple Silicon support coming soon (universal binary).

**Q: Does ClipVault work offline?**
A: Yes, completely offline. No network access required.

**Q: Can I sync across devices?**
A: Not yet. Optional encrypted cloud sync planned for future version.

### Privacy & Security

**Q: Is my clipboard data secure?**
A: Yes. Sensitive data is encrypted with ChaCha20-Poly1305. All data stored locally only.

**Q: Can ClipVault read my passwords?**
A: ClipVault can only access what you copy to clipboard. It automatically encrypts detected passwords.

**Q: Where is my data stored?**
A: `~/Library/Application Support/com.smolkapps.clipboard-manager/`

**Q: Can I delete my data?**
A: Yes. Quit ClipVault and delete the folder above, or wait 7 days for auto-cleanup.

**Q: Does ClipVault send data anywhere?**
A: No. Zero network requests. All data stays on your Mac.

### Features

**Q: How many items can ClipVault store?**
A: Thousands. Items auto-delete after 7 days to keep database lean.

**Q: Can I customize the hotkey?**
A: Not yet. Customizable hotkeys planned for future version.

**Q: Can I exclude certain apps?**
A: Not yet. App exclusion list planned for future version.

**Q: Does it support file paths?**
A: Yes, copied file paths are saved as text. Full file support (not just paths) planned.

**Q: What image formats are supported?**
A: TIFF, PNG, JPEG. All converted to optimized PNG for storage.

### Troubleshooting

**Q: Why isn't my clipboard item encrypted?**
A: Only sensitive data is auto-encrypted (API keys, passwords, etc.). Normal text is not encrypted.

**Q: Can I manually encrypt an item?**
A: Not yet. Manual encryption toggle planned for future version.

**Q: How do I uninstall ClipVault?**
A:
1. Quit ClipVault
2. Delete `/Applications/ClipVault.app`
3. Delete `~/Library/Application Support/com.smolkapps.clipboard-manager/`

**Q: Does ClipVault affect system clipboard?**
A: No. ClipVault only reads from clipboard, never modifies it (except when you paste from history).

### Performance

**Q: How much disk space does ClipVault use?**
A: Typical: ~5-50MB depending on usage. 7-day retention keeps it lean.

**Q: How much RAM does it use?**
A: <50MB typically. Binary is ~14MB.

**Q: Does it slow down my Mac?**
A: No. Clipboard polling is very lightweight (500ms interval).

---

## Getting Help

### Support Channels

- **GitHub Issues**: [Report bugs and request features](https://github.com/smolkapps/secure-clipboard-manager/issues)
- **Email**: support@smolkapps.com
- **Documentation**: [Full docs on GitHub](https://github.com/smolkapps/secure-clipboard-manager)

### Before Reporting Issues

Please include:
1. macOS version (`About This Mac`)
2. ClipVault version (`About ClipVault` in menu)
3. Steps to reproduce
4. Expected vs actual behavior
5. Console logs if available

### Feature Requests

We welcome feature requests! Please:
1. Check existing issues first
2. Describe use case clearly
3. Explain expected behavior

---

## What's Next?

### Coming Soon (v0.2)

- [ ] Customizable hotkeys
- [ ] Adjustable retention period
- [ ] App exclusion list
- [ ] Manual item deletion
- [ ] Search history
- [ ] Pin favorite items

### Future Versions

- [ ] Apple Silicon (M1/M2/M3) support
- [ ] Optional encrypted cloud sync
- [ ] Snippet templates
- [ ] Full file support (not just paths)
- [ ] Markdown preview
- [ ] Code syntax highlighting

---

**Thank you for using ClipVault!**

Built with ❤️ in Rust for macOS

Version 0.1.0 | Last updated: 2026-01-27
