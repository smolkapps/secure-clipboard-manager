// Integration tests for storage layer (database + encryption)
use clipboard_manager::storage::{
    database::{ClipboardDatabase, ClipboardItem},
    encryption::Encryptor,
};
use tempfile::TempDir;

#[test]
fn test_database_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = ClipboardDatabase::new(db_path.clone()).unwrap();

    // Database file should exist
    assert!(db_path.exists());
}

#[test]
fn test_insert_and_retrieve_text() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Insert text item
    let text = b"Hello, World!";
    let item_id = db.insert_item(
        "text",
        false,
        false,
        Some("Hello, World!"),
        text,
        None,
    ).unwrap();

    assert!(item_id > 0);

    // Retrieve item
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data_type, "text");
    assert_eq!(items[0].preview_text, Some("Hello, World!".to_string()));
}

#[test]
fn test_insert_encrypted_item() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let key_path = temp_dir.path().join("test.key");

    let db = ClipboardDatabase::new(db_path).unwrap();
    let encryptor = Encryptor::new(key_path).unwrap();

    // Encrypt sensitive data
    let sensitive_text = b"sk-1234567890abcdefghijklmnopqrstuvwxyz";
    let encrypted = encryptor.encrypt(sensitive_text).unwrap();

    // Insert encrypted item
    let item_id = db.insert_item(
        "text",
        true,  // is_sensitive
        true,  // is_encrypted
        Some("API Key (encrypted)"),
        &encrypted,
        None,
    ).unwrap();

    assert!(item_id > 0);

    // Retrieve and decrypt
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_sensitive);
    assert!(items[0].is_encrypted);

    let blob = db.get_blob(items[0].data_blob_id).unwrap();
    let decrypted = encryptor.decrypt(&blob).unwrap();
    assert_eq!(decrypted, sensitive_text);
}

#[test]
fn test_multiple_items_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Insert multiple items
    for i in 1..=5 {
        let text = format!("Item {}", i);
        db.insert_item(
            "text",
            false,
            false,
            Some(&text),
            text.as_bytes(),
            None,
        ).unwrap();

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Retrieve items (should be in reverse chronological order)
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 5);

    // Most recent should be "Item 5"
    assert_eq!(items[0].preview_text, Some("Item 5".to_string()));
    assert_eq!(items[4].preview_text, Some("Item 1".to_string()));
}

#[test]
fn test_cleanup_old_items() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Insert item
    let text = b"Test item";
    db.insert_item(
        "text",
        false,
        false,
        Some("Test item"),
        text,
        None,
    ).unwrap();

    // Verify it exists
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);

    // Cleanup items older than 0 seconds (should delete everything)
    let deleted = db.cleanup_old_items(0).unwrap();
    assert_eq!(deleted, 1);

    // Verify it's gone
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_image_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Create fake image data (just some bytes)
    let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header

    let item_id = db.insert_item(
        "image",
        false,
        false,
        Some("640x480 PNG"),
        &image_data,
        Some(r#"{"width":640,"height":480,"format":"PNG"}"#),
    ).unwrap();

    assert!(item_id > 0);

    // Retrieve and verify
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data_type, "image");
    assert!(items[0].metadata.is_some());

    let blob = db.get_blob(items[0].data_blob_id).unwrap();
    assert_eq!(blob, image_data);
}

#[test]
fn test_encryption_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let key_path = temp_dir.path().join("test.key");
    let encryptor = Encryptor::new(key_path).unwrap();

    let test_cases = vec![
        b"Short text".to_vec(),
        b"sk-1234567890abcdefghijklmnopqrstuvwxyz".to_vec(),
        vec![0u8; 1000], // Binary data
        "Unicode: 你好世界 🔒".as_bytes().to_vec(),
    ];

    for plaintext in test_cases {
        let encrypted = encryptor.encrypt(&plaintext).unwrap();

        // Encrypted should be different
        assert_ne!(encrypted, plaintext);

        // Should decrypt correctly
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

#[test]
fn test_concurrent_database_access() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create database
    {
        let db = ClipboardDatabase::new(db_path.clone()).unwrap();
        db.insert_item("text", false, false, Some("Initial"), b"Initial", None).unwrap();
    }

    // Access from multiple "threads" (sequential but simulates concurrent access)
    for i in 0..10 {
        let db = ClipboardDatabase::new(db_path.clone()).unwrap();
        let text = format!("Item {}", i);
        db.insert_item("text", false, false, Some(&text), text.as_bytes(), None).unwrap();
    }

    // Verify all items are there
    let db = ClipboardDatabase::new(db_path).unwrap();
    let items = db.get_recent_items(20).unwrap();
    assert_eq!(items.len(), 11); // Initial + 10 items
}

#[test]
fn test_get_item_count() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Initially should be 0
    let count = db.get_item_count().unwrap();
    assert_eq!(count, 0);

    // Add 3 items
    for i in 1..=3 {
        db.insert_item("text", false, false, Some(&format!("Item {}", i)), b"data", None).unwrap();
    }

    let count = db.get_item_count().unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_empty_preview_text() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ClipboardDatabase::new(db_path).unwrap();

    // Insert item without preview
    let item_id = db.insert_item(
        "text",
        false,
        false,
        None,  // No preview
        b"Some data",
        None,
    ).unwrap();

    assert!(item_id > 0);

    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].preview_text.is_none());
}
