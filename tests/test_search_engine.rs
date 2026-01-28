// Integration tests for fuzzy search engine
use clipboard_manager::storage::{
    database::ClipboardItem,
    search::SearchEngine,
};

fn create_test_item(id: i64, preview: &str, data_type: &str, timestamp: i64) -> ClipboardItem {
    ClipboardItem {
        id,
        timestamp,
        data_type: data_type.to_string(),
        is_sensitive: false,
        is_encrypted: false,
        preview_text: Some(preview.to_string()),
        data_size: preview.len() as i64,
        data_blob_id: id,
        metadata: None,
    }
}

#[test]
fn test_exact_match() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Hello World", "text", 100),
        create_test_item(2, "Goodbye World", "text", 200),
    ];

    let results = engine.search(&items, "Hello");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, 1);
}

#[test]
fn test_fuzzy_match() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "clipboard manager", "text", 100),
        create_test_item(2, "clip board man", "text", 200),
    ];

    let results = engine.search(&items, "clipman");
    // Both should match with fuzzy search
    assert!(results.len() >= 1);
}

#[test]
fn test_case_insensitive() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Hello World", "text", 100),
    ];

    let results_lower = engine.search(&items, "hello");
    let results_upper = engine.search(&items, "HELLO");
    let results_mixed = engine.search(&items, "HeLLo");

    assert_eq!(results_lower.len(), 1);
    assert_eq!(results_upper.len(), 1);
    assert_eq!(results_mixed.len(), 1);
}

#[test]
fn test_empty_query_returns_all() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Item 1", "text", 100),
        create_test_item(2, "Item 2", "text", 200),
        create_test_item(3, "Item 3", "text", 300),
    ];

    let results = engine.search(&items, "");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_no_matches() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Hello World", "text", 100),
        create_test_item(2, "Goodbye Earth", "text", 200),
    ];

    let results = engine.search(&items, "xyz123notfound");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_relevance_sorting() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "test", "text", 100),              // Exact match
        create_test_item(2, "this is a test case", "text", 200), // Contains match
        create_test_item(3, "t e s t", "text", 150),           // Scattered match
    ];

    let results = engine.search(&items, "test");

    // Should have results
    assert!(!results.is_empty());

    // Scores should be in descending order
    for i in 0..results.len() - 1 {
        assert!(results[i].0 >= results[i + 1].0,
                "Results should be sorted by score: {} >= {}",
                results[i].0, results[i + 1].0);
    }
}

#[test]
fn test_search_by_data_type() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Some text", "text", 100),
        create_test_item(2, "Preview", "image", 200),
        create_test_item(3, "Another", "url", 300),
    ];

    let results = engine.search(&items, "image");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.data_type, "image");
}

#[test]
fn test_multiple_matches_same_score() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "test item", "text", 300), // Newer
        create_test_item(2, "test item", "text", 100), // Older
    ];

    let results = engine.search(&items, "test");
    assert_eq!(results.len(), 2);

    // Same score, so should be sorted by timestamp (newer first)
    assert_eq!(results[0].1.id, 1, "Newer item should come first");
}

#[test]
fn test_partial_word_match() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "JavaScript code snippet", "text", 100),
        create_test_item(2, "Python script", "text", 200),
    ];

    let results = engine.search(&items, "java");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, 1);
}

#[test]
fn test_multiple_word_query() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "clipboard manager for macOS", "text", 100),
        create_test_item(2, "text editor", "text", 200),
    ];

    let results = engine.search(&items, "clipboard macos");
    // Should find the item with both words
    assert!(results.len() >= 1);
}

#[test]
fn test_special_characters() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "user@example.com", "text", 100),
        create_test_item(2, "https://github.com", "url", 200),
        create_test_item(3, "C++ programming", "text", 300),
    ];

    let results_email = engine.search(&items, "user@");
    assert!(results_email.len() >= 1);

    let results_url = engine.search(&items, "github");
    assert!(results_url.len() >= 1);

    let results_cpp = engine.search(&items, "C++");
    assert!(results_cpp.len() >= 1);
}

#[test]
fn test_unicode_search() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Hello 你好 World", "text", 100),
        create_test_item(2, "Привет мир", "text", 200),
        create_test_item(3, "こんにちは世界", "text", 300),
    ];

    let results = engine.search(&items, "你好");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, 1);
}

#[test]
fn test_search_with_no_preview_text() {
    let engine = SearchEngine::new();
    let mut item = create_test_item(1, "test", "text", 100);
    item.preview_text = None;

    let items = vec![item];
    let results = engine.search(&items, "test");

    // Item without preview should still search by data_type
    let results_type = engine.search(&items, "text");
    assert_eq!(results_type.len(), 1);

    // But won't match on non-existent preview
    assert_eq!(results.len(), 0);
}

#[test]
fn test_large_dataset_performance() {
    use std::time::Instant;

    let engine = SearchEngine::new();

    // Create 1000 items
    let items: Vec<ClipboardItem> = (0..1000)
        .map(|i| create_test_item(
            i,
            &format!("Test item number {} with some text", i),
            "text",
            i,
        ))
        .collect();

    let start = Instant::now();
    let results = engine.search(&items, "test");
    let duration = start.elapsed();

    // Should complete in reasonable time (< 50ms for 1000 items)
    assert!(duration.as_millis() < 50, "Search took {}ms, should be < 50ms", duration.as_millis());
    assert_eq!(results.len(), 1000); // All items contain "test"
}

#[test]
fn test_search_numbers() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Order #12345", "text", 100),
        create_test_item(2, "Invoice 67890", "text", 200),
    ];

    let results = engine.search(&items, "12345");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, 1);
}

#[test]
fn test_whitespace_handling() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "test  with   spaces", "text", 100),
    ];

    let results = engine.search(&items, "test spaces");
    assert!(results.len() >= 1);
}

#[test]
fn test_empty_items_list() {
    let engine = SearchEngine::new();
    let items: Vec<ClipboardItem> = vec![];

    let results = engine.search(&items, "test");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_very_long_query() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Short text", "text", 100),
    ];

    let long_query = "a".repeat(1000);
    let results = engine.search(&items, &long_query);
    assert_eq!(results.len(), 0); // Shouldn't match
}

#[test]
fn test_mixed_content_types() {
    let engine = SearchEngine::new();
    let items = vec![
        create_test_item(1, "Code snippet", "text", 100),
        create_test_item(2, "640x480 PNG", "image", 200),
        create_test_item(3, "https://example.com", "url", 300),
        create_test_item(4, "API Key (encrypted)", "text", 400),
    ];

    // Search by content
    let results_code = engine.search(&items, "code");
    assert_eq!(results_code.len(), 1);

    // Search by type
    let results_image = engine.search(&items, "image");
    assert_eq!(results_image.len(), 1);

    // Search by preview
    let results_api = engine.search(&items, "API");
    assert_eq!(results_api.len(), 1);
}
