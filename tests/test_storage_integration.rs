// Integration tests for storage layer (database + encryption)
use clipboard_manager::storage::{
    database::Database,
    encryption::Encryptor,
};
use tempfile::TempDir;

#[test]
fn test_database_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let _db = Database::new(db_path.clone()).unwrap();
    // Database file should exist
    assert!(db_path.exists());
}

#[test]
fn test_insert_and_retrieve_text() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    let text = b"Hello, World!";
    let blob_id = db.store_blob(text).unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    let item_id = db.store_item(
        timestamp,
        "text",
        false,
        false,
        Some("Hello, World!"),
        text.len() as i64,
        blob_id,
        None,
        1,
    ).unwrap();
    assert!(item_id > 0);

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

    let db = Database::new(db_path).unwrap();
    let encryptor = Encryptor::new(key_path).unwrap();

    let sensitive_text = b"sk-1234567890abcdefghijklmnopqrstuvwxyz";
    let encrypted = encryptor.encrypt(sensitive_text).unwrap();

    // Insert encrypted item
    let blob_id = db.store_blob(&encrypted).unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    let item_id = db.store_item(
        timestamp,
        "text",
        true,   // is_sensitive
        true,   // is_encrypted
        Some("API Key (encrypted)"),
        encrypted.len() as i64,
        blob_id,
        None,
        1,
    ).unwrap();
    assert!(item_id > 0);

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
    let db = Database::new(db_path).unwrap();

    // Use explicit timestamps so ordering is deterministic
    let base_ts = chrono::Utc::now().timestamp();
    for i in 1..=5 {
        let text = format!("Item {}", i);
        let blob_id = db.store_blob(text.as_bytes()).unwrap();
        db.store_item(
            base_ts + i,
            "text",
            false,
            false,
            Some(&text),
            text.len() as i64,
            blob_id,
            None,
            1,
        ).unwrap();
    }

    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(items[0].preview_text, Some("Item 5".to_string()));
    assert_eq!(items[4].preview_text, Some("Item 1".to_string()));
}

#[test]
fn test_cleanup_old_items() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert an item with a timestamp 2 days in the past
    let blob_id = db.store_blob(b"Test item").unwrap();
    let old_ts = chrono::Utc::now().timestamp() - (2 * 86400);
    db.store_item(
        old_ts,
        "text",
        false,
        false,
        Some("Test item"),
        9,
        blob_id,
        None,
        1,
    ).unwrap();

    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);

    // Cleanup items older than 1 day — should catch our 2-day-old item
    let deleted = db.cleanup_old_items(1).unwrap();
    assert_eq!(deleted, 1);

    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_image_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    let image_data = vec![0x89, 0x50, 0x4E, 0x47];
    let blob_id = db.store_blob(&image_data).unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    let item_id = db.store_item(
        timestamp,
        "image",
        false,
        false,
        Some("640x480 PNG"),
        image_data.len() as i64,
        blob_id,
        Some(r#"{"width":640,"height":480,"format":"PNG"}"#),
        1,
    ).unwrap();

    assert!(item_id > 0);
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data_type, "image");

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
        "Unicode: \u{4f60}\u{597d}\u{4e16}\u{754c} \u{1f512}".as_bytes().to_vec(),
    ];

    for plaintext in test_cases {
        let encrypted = encryptor.encrypt(&plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

#[test]
fn test_item_count() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert 4 items
    let base_ts = chrono::Utc::now().timestamp();
    for i in 0..4 {
        let text = format!("Item {}", i);
        let blob_id = db.store_blob(text.as_bytes()).unwrap();
        db.store_item(
            base_ts + i,
            "text",
            false,
            false,
            Some(&text),
            text.len() as i64,
            blob_id,
            None,
            1,
        ).unwrap();
    }

    // Verify all items are there
    let items = db.get_recent_items(20).unwrap();
    assert_eq!(items.len(), 4);
}

#[test]
fn test_get_item_count() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Initially should be 0
    let count = db.count_items().unwrap();
    assert_eq!(count, 0);

    // Add 3 items
    for i in 1..=3 {
        let text = format!("Item {}", i);
        let blob_id = db.store_blob(text.as_bytes()).unwrap();
        let timestamp = chrono::Utc::now().timestamp() + i as i64;
        db.store_item(
            timestamp,
            "text",
            false,
            false,
            Some(&text),
            text.len() as i64,
            blob_id,
            None,
            1,
        ).unwrap();
    }

    let count = db.count_items().unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_pin_and_delete() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    let blob_id = db.store_blob(b"pinnable").unwrap();
    let ts = chrono::Utc::now().timestamp();
    let item_id = db.store_item(
        ts,
        "text",
        false,
        false,
        Some("pinnable"),
        8,
        blob_id,
        None,
        1,
    ).unwrap();

    // Toggle pin
    let pinned = db.toggle_pin(item_id).unwrap();
    assert!(pinned);

    let items = db.get_recent_items(10).unwrap();
    assert!(items[0].is_pinned);

    // Toggle again
    let pinned = db.toggle_pin(item_id).unwrap();
    assert!(!pinned);

    // Delete
    db.delete_item(item_id).unwrap();
    assert_eq!(db.count_items().unwrap(), 0);
}

#[test]
fn test_remove_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert two items with the same preview
    for i in 0..2 {
        let blob_id = db.store_blob(b"same text").unwrap();
        let timestamp = chrono::Utc::now().timestamp() + i;
        db.store_item(timestamp, "text", false, false, Some("same text"), 9, blob_id, None, 1).unwrap();
    }

    assert_eq!(db.count_items().unwrap(), 2);

    // Remove duplicates should find and remove both
    let (removed, max_count) = db.remove_duplicates(Some("same text"), "text").unwrap();
    assert_eq!(removed, 2);
    assert_eq!(max_count, 1);
    assert_eq!(db.count_items().unwrap(), 0);
}

#[test]
fn test_enforce_history_limit() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert 10 items
    for i in 0..10 {
        let text = format!("Item {}", i);
        let blob_id = db.store_blob(text.as_bytes()).unwrap();
        let timestamp = chrono::Utc::now().timestamp() + i;
        db.store_item(timestamp, "text", false, false, Some(&text), text.len() as i64, blob_id, None, 1).unwrap();
    }

    assert_eq!(db.count_items().unwrap(), 10);

    // Enforce limit of 5
    let trimmed = db.enforce_history_limit(5).unwrap();
    assert_eq!(trimmed, 5);
    assert_eq!(db.count_items().unwrap(), 5);

    // Remaining items should be the 5 newest
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(items[0].preview_text, Some("Item 9".to_string()));
}

#[test]
fn test_soft_delete_and_purge() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert 3 items
    for i in 0..3 {
        let text = format!("Item {}", i);
        let blob_id = db.store_blob(text.as_bytes()).unwrap();
        let timestamp = chrono::Utc::now().timestamp() + i;
        db.store_item(timestamp, "text", false, false, Some(&text), text.len() as i64, blob_id, None, 1).unwrap();
    }

    assert_eq!(db.count_items().unwrap(), 3);

    // Soft delete all
    let deleted = db.soft_delete_all_items().unwrap();
    assert_eq!(deleted, 3);
    assert_eq!(db.count_items().unwrap(), 0);

    // Purge should not remove anything yet (items just deleted)
    let purged = db.purge_deleted_items().unwrap();
    assert_eq!(purged, 0);
}

#[test]
fn test_copy_count_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    // Insert item with copy_count = 3
    let blob_id = db.store_blob(b"repeated").unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    db.store_item(timestamp, "text", false, false, Some("repeated"), 8, blob_id, None, 3).unwrap();

    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items[0].copy_count, 3);
}

#[test]
fn test_database_size() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();

    let size = db.get_db_size().unwrap();
    assert!(size > 0, "Database should have non-zero size after initialization");
}
