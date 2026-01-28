# Integration Tests

This directory contains comprehensive integration tests for the ClipVault clipboard manager.

## Test Organization

### Core Integration Tests

1. **test_storage_integration.rs** - Database and encryption integration
   - Database creation and initialization
   - Text and image item storage
   - Encrypted item storage with encryption/decryption
   - Multi-item ordering and retrieval
   - Cleanup operations
   - Concurrent database access
   - Blob storage and retrieval

2. **test_sensitive_detection.rs** - Sensitive data detection accuracy
   - API key detection (OpenAI, GitHub, AWS, Google, etc.)
   - JWT token detection
   - Private key detection (PEM format)
   - Password-like string detection
   - Environment variable secrets
   - Connection string detection
   - Normal text classification (avoiding false positives)

3. **test_image_processing.rs** - Image processing and conversion
   - PNG, TIFF, JPEG support
   - TIFF → PNG conversion
   - Thumbnail generation (200x200px max)
   - Aspect ratio preservation
   - Compression ratio calculation
   - Metadata generation
   - Invalid image handling

4. **test_search_engine.rs** - Fuzzy search functionality
   - Exact and fuzzy matching
   - Case-insensitive search
   - Relevance scoring and sorting
   - Search by data type
   - Unicode support
   - Performance with large datasets (1000+ items)
   - Special character handling

5. **test_clipboard_monitoring.rs** - NSPasteboard polling (headless-safe)
   - Monitor creation and configuration
   - Change count tracking
   - String extraction
   - Image extraction
   - Performance benchmarks
   - Safe for CI environments (no GUI required)

## Running Tests

### Run all tests
```bash
cargo test --all
```

### Run specific test file
```bash
cargo test --test test_storage_integration
cargo test --test test_sensitive_detection
cargo test --test test_image_processing
cargo test --test test_search_engine
cargo test --test test_clipboard_monitoring
```

### Run specific test case
```bash
cargo test test_insert_and_retrieve_text
cargo test test_detect_openai_api_key
cargo test test_tiff_to_png_conversion
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests with logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Test Coverage

The integration tests cover:

- ✅ **Storage Layer**: Database operations, blob storage, encryption
- ✅ **Data Processing**: Text/image processing, sensitive detection
- ✅ **Search Engine**: Fuzzy matching, relevance scoring, performance
- ✅ **Clipboard Monitoring**: NSPasteboard polling, change detection
- ✅ **Image Processing**: Format conversion, thumbnails, compression
- ✅ **Security**: Encryption/decryption, key management, sensitive data

### Coverage Highlights

- **28+ integration test cases** covering critical paths
- **Performance benchmarks** for search (<50ms) and monitoring
- **Headless-safe tests** that work in CI environments
- **Edge case testing** (empty data, invalid inputs, large datasets)

## Performance Targets

Tests validate these performance targets:

| Operation | Target | Test |
|-----------|--------|------|
| Fuzzy search (1000 items) | <50ms | `test_large_dataset_performance` |
| Change count (1000 calls) | <100ms | `test_change_count_performance` |
| String read (100 calls) | <500ms | `test_get_string_performance` |
| Monitor creation (100x) | <100ms | `test_monitor_creation_performance` |

## CI/CD Integration

All tests are designed to run in CI environments:

- **No GUI required** - Tests use headless-safe operations
- **No external dependencies** - All tests use temporary files/databases
- **Deterministic** - Tests don't rely on system clipboard state
- **Fast** - Full test suite completes in seconds

## Writing New Tests

When adding new tests:

1. **Use descriptive names**: `test_<what>_<condition>_<expected>`
2. **Use temporary files**: Use `tempfile::TempDir` for isolation
3. **Test edge cases**: Empty inputs, large data, invalid data
4. **Add performance tests**: For critical operations
5. **Document what you test**: Add comments for complex scenarios

Example:
```rust
#[test]
fn test_database_handles_empty_preview() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Test that None preview is handled correctly
    let item_id = db.insert_item(
        "text",
        false,
        false,
        None,  // No preview
        b"data",
        None,
    ).unwrap();

    assert!(item_id > 0);
}
```

## Troubleshooting

### Tests fail in CI but pass locally
- Check if tests depend on clipboard state
- Ensure tests use temporary directories
- Verify tests don't require GUI/windowing system

### Performance tests fail
- CI environments are slower than local machines
- Increase timeout thresholds for CI (use `#[cfg(not(ci))]` for strict local tests)

### Image processing tests fail
- Ensure `image` crate features are enabled in Cargo.toml
- Check that test images are valid format

## Test Data

Tests create temporary data in:
- Temporary directories (auto-cleaned by `TempDir`)
- In-memory structures
- No persistent state between test runs

## Future Test Additions

Planned test coverage:

- [ ] UI integration tests (when popup window is testable)
- [ ] Global hotkey tests (requires mocking)
- [ ] End-to-end workflow tests
- [ ] Stress tests (10,000+ items)
- [ ] Multi-threaded access tests
