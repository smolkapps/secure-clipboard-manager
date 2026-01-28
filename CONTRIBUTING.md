# Contributing to ClipVault

Thank you for your interest in contributing to ClipVault! This document provides guidelines and instructions for contributing.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Project Structure](#project-structure)
3. [Code Style Guide](#code-style-guide)
4. [Testing](#testing)
5. [Pull Request Process](#pull-request-process)
6. [Issue Reporting](#issue-reporting)
7. [Feature Requests](#feature-requests)

---

## Development Setup

### Prerequisites

- **macOS**: 12.7.5 or later (Intel or Apple Silicon)
- **Rust**: 1.92.0 or later
- **Xcode**: Command Line Tools (for macOS frameworks)

### Initial Setup

1. **Install Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Install Xcode Command Line Tools**:
```bash
xcode-select --install
```

3. **Clone the repository**:
```bash
git clone https://github.com/smolkapps/secure-clipboard-manager.git
cd secure-clipboard-manager/clipboard-manager
```

4. **Build the project**:
```bash
cargo build
```

5. **Run tests**:
```bash
cargo test --all
```

6. **Run the application**:
```bash
cargo run
```

### Development Tools

**Recommended**:
- **IDE**: VS Code with rust-analyzer extension
- **Debugger**: CodeLLDB extension for VS Code
- **Linter**: clippy (`cargo clippy`)
- **Formatter**: rustfmt (`cargo fmt`)

**VS Code Extensions**:
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates",
    "tamasfe.even-better-toml"
  ]
}
```

---

## Project Structure

```
clipboard-manager/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library entry point
│   ├── clipboard/           # Clipboard monitoring
│   │   ├── mod.rs
│   │   ├── monitor.rs       # NSPasteboard polling
│   │   ├── processor.rs     # Clipboard data processing
│   │   └── history.rs       # History management
│   ├── storage/             # Data storage
│   │   ├── mod.rs
│   │   ├── database.rs      # SQLite database
│   │   ├── encryption.rs    # ChaCha20-Poly1305 encryption
│   │   ├── processor.rs     # Data type detection
│   │   └── search.rs        # Fuzzy search engine
│   ├── ui/                  # User interface
│   │   ├── mod.rs
│   │   ├── menubar.rs       # Menu bar interface
│   │   ├── popup.rs         # Popup window
│   │   ├── hotkey.rs        # Global hotkey handling
│   │   ├── statusbar.rs     # Status bar item
│   │   └── actions.rs       # UI actions
│   └── search/              # Search module (unused, deprecated)
├── tests/                   # Integration tests
│   ├── test_storage_integration.rs
│   ├── test_sensitive_detection.rs
│   ├── test_image_processing.rs
│   ├── test_search_engine.rs
│   ├── test_clipboard_monitoring.rs
│   └── README.md
├── resources/               # App resources
│   └── AppIcon.iconset/     # App icon assets
├── docs/                    # Documentation
│   └── USER_GUIDE.md
├── Cargo.toml               # Rust dependencies
└── README.md                # Project README
```

### Module Responsibilities

**clipboard**: Monitors system clipboard and detects changes
- Uses `objc2-app-kit` for NSPasteboard access
- Polls every 500ms for changes
- Extracts text and image data

**storage**: Manages data persistence and encryption
- SQLite database with blob storage
- ChaCha20-Poly1305 encryption for sensitive data
- Automatic sensitive data detection
- Image processing (TIFF→PNG conversion, thumbnails)

**ui**: macOS native UI components
- Menu bar integration using `cacao`
- Popup window with keyboard navigation
- Global hotkey (Cmd+Shift+C)

**search**: Fuzzy search across clipboard history
- Uses `fuzzy-matcher` (Skim algorithm)
- Scores and ranks results by relevance

---

## Code Style Guide

### General Principles

1. **Follow Rust conventions**: Use `rustfmt` and `clippy`
2. **Write self-documenting code**: Clear variable names
3. **Add comments for "why"**: Not "what"
4. **Handle errors explicitly**: No unwrap() in production code
5. **Test your code**: Unit tests for logic, integration tests for features

### Naming Conventions

```rust
// Modules: snake_case
mod clipboard_monitor;

// Types: PascalCase
struct ClipboardMonitor;
enum DataType;

// Functions: snake_case
fn process_clipboard_data();

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_POLL_INTERVAL: u64 = 500;

// Private fields: snake_case with underscore prefix
struct Monitor {
    _last_change_count: i64,
}
```

### Error Handling

**Do**:
```rust
// Return Result for operations that can fail
fn load_data() -> Result<Data, String> {
    let file = fs::read("data.txt")
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(parse_data(&file)?)
}

// Use descriptive error messages
Err(format!("Failed to encrypt data: {}", e))
```

**Don't**:
```rust
// Don't use unwrap() in production code
let data = load_data().unwrap();

// Don't use generic errors
Err("error".to_string())
```

### Documentation

**Public APIs**:
```rust
/// Monitors the system clipboard for changes
///
/// Polls NSPasteboard at a configurable interval and sends
/// change events to the provided channel.
///
/// # Examples
///
/// ```
/// let monitor = ClipboardMonitor::new();
/// monitor.start(tx).await?;
/// ```
pub struct ClipboardMonitor { /* ... */ }
```

**Internal code**:
```rust
// Extract TIFF data from pasteboard (macOS default screenshot format)
let tiff_type = NSString::from_str("public.tiff");
```

### Testing

**Unit tests**: In same file as code
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ClipboardMonitor::new();
        assert_eq!(monitor.poll_interval_ms, 500);
    }
}
```

**Integration tests**: In `tests/` directory
```rust
// tests/test_storage_integration.rs
#[test]
fn test_database_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db = ClipboardDatabase::new(temp_dir.path().join("test.db")).unwrap();
    assert!(db_path.exists());
}
```

### Unsafe Code

**Guidelines**:
1. **Minimize unsafe blocks**: Keep as small as possible
2. **Document safety invariants**: Explain why it's safe
3. **Prefer safe abstractions**: Wrap unsafe in safe API

```rust
/// Get string content from clipboard
///
/// # Safety
/// This is safe because NSPasteboard is thread-safe and
/// we're only reading, not modifying.
pub fn get_string() -> Option<String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        // ...
    }
}
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test --all

# Specific test file
cargo test --test test_storage_integration

# Specific test case
cargo test test_encrypt_decrypt

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Coverage

We aim for:
- **Unit tests**: Core logic in each module
- **Integration tests**: Feature workflows
- **Performance tests**: Critical operations (<50ms search, <100ms popup)

### Writing Tests

**Good test**:
```rust
#[test]
fn test_sensitive_detection_api_key() {
    let text = "sk-1234567890abcdefghijklmnopqrstuvwxyz";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive,
            "OpenAI API key should be detected as sensitive");
}
```

**Bad test**:
```rust
#[test]
fn test_stuff() {
    let x = process("test");
    assert!(x.is_ok());  // What are we testing?
}
```

### CI/CD

Tests run automatically on:
- Every pull request
- Every commit to main
- Must pass before merge

---

## Pull Request Process

### Before Submitting

1. **Create an issue first**: Discuss the change
2. **Fork the repository**: Work on your fork
3. **Create a feature branch**: `git checkout -b feature/my-feature`
4. **Write tests**: For new functionality
5. **Run tests**: `cargo test --all`
6. **Run clippy**: `cargo clippy`
7. **Run rustfmt**: `cargo fmt`
8. **Update documentation**: If adding features

### PR Guidelines

**Good PR**:
- Clear, descriptive title
- References related issue (#123)
- Explains what and why
- Includes tests
- Updates docs if needed
- Passes CI

**PR Template**:
```markdown
## Description
Brief description of changes

## Related Issue
Fixes #123

## Changes
- Added X feature
- Fixed Y bug
- Updated Z docs

## Testing
- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guide
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No new clippy warnings
```

### Review Process

1. **Automated checks**: CI runs tests and linting
2. **Code review**: Maintainer reviews code
3. **Address feedback**: Make requested changes
4. **Approval**: Maintainer approves
5. **Merge**: Squash and merge to main

---

## Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
## Description
Clear description of the bug

## Steps to Reproduce
1. Launch ClipVault
2. Copy text with API key
3. Check database

## Expected Behavior
API key should be encrypted

## Actual Behavior
API key stored in plaintext

## Environment
- macOS version: 13.5
- ClipVault version: 0.1.0
- Architecture: Intel/Apple Silicon

## Logs
Paste relevant logs here
```

### Finding Logs

```bash
# Run with logging
RUST_LOG=debug cargo run

# macOS Console.app
# Filter: process:clipboard-manager
```

---

## Feature Requests

### Before Requesting

1. **Check existing issues**: Avoid duplicates
2. **Consider scope**: Fits project goals?
3. **Provide use case**: Why is this useful?

### Feature Request Template

```markdown
## Feature Description
Clear description of the feature

## Use Case
Why do you need this feature?

## Proposed Solution
How should it work?

## Alternatives Considered
What alternatives did you consider?

## Additional Context
Screenshots, mockups, etc.
```

---

## Development Workflow

### Typical Workflow

1. **Find/create issue**
2. **Fork repository**
3. **Create feature branch**: `feature/issue-123-add-feature`
4. **Write code**
5. **Write tests**
6. **Run tests**: `cargo test --all`
7. **Run linting**: `cargo clippy`
8. **Format code**: `cargo fmt`
9. **Commit**: Clear commit messages
10. **Push to fork**
11. **Create PR**
12. **Address review feedback**
13. **Merge**

### Commit Messages

**Format**:
```
<type>: <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

**Examples**:
```
feat: Add fuzzy search to clipboard history

Implements Skim-based fuzzy matching with relevance scoring.
Search results sorted by score and recency.

Fixes #45
```

```
fix: Prevent database corruption on crash

Add WAL mode to SQLite for better crash resilience.

Fixes #67
```

---

## Architecture Decisions

### Why Rust?

- **Performance**: Native speed, <50MB RAM
- **Safety**: No crashes, memory safety
- **macOS native**: Excellent Objective-C interop

### Why SQLite?

- **Simple**: Single file, no server
- **Fast**: Local queries <1ms
- **Reliable**: ACID transactions

### Why ChaCha20-Poly1305?

- **Modern**: Industry standard (TLS 1.3)
- **Fast**: Hardware acceleration on modern CPUs
- **Secure**: AEAD (authenticated encryption)

### Why Cacao?

- **Native**: Uses AppKit under the hood
- **Rust-friendly**: Safe abstractions
- **Maintained**: Active development

---

## Getting Help

### Resources

- **Docs**: `/docs` directory
- **Tests**: `/tests` for examples
- **Issues**: GitHub issues for questions

### Contact

- **GitHub Issues**: Technical questions
- **Email**: dev@smolkapps.com
- **Discussions**: GitHub Discussions for general chat

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (TBD - likely MIT or commercial).

---

Thank you for contributing to ClipVault! 🎉
