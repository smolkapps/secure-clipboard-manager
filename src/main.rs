// Clipboard Manager - macOS Native Clipboard History Manager
// Phase 3: Sensitive Data Detection & Encryption

mod clipboard;
mod storage;

use clipboard::ClipboardMonitor;
use storage::{Database, DataProcessor, Encryptor};
use log::{error, info};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Clipboard Manager - Phase 3: Sensitive Data Detection & Encryption");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize data directory
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clipboard-manager");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    // Initialize database
    let db_path = data_dir.join("clipboard.db");
    let db = Database::new(db_path.clone())
        .expect("Failed to initialize database");

    info!("✓ Database initialized at: {}", db_path.display());
    info!("  Items in history: {}", db.count_items().unwrap_or(0));
    info!("  Database size: {} KB", db.get_db_size().unwrap_or(0) / 1024);

    // Initialize encryptor
    let key_path = data_dir.join("encryption.key");
    let encryptor = Encryptor::new(key_path)
        .expect("Failed to initialize encryptor");

    info!("✓ Encryption initialized");

    // Run cleanup on startup (remove items older than 7 days)
    match db.cleanup_old_items(7) {
        Ok(count) if count > 0 => info!("  Cleaned up {} old items", count),
        _ => {}
    }

    info!("");
    info!("✓ Clipboard monitor initialized (polling every 500ms)");
    info!("   Monitoring NSPasteboard for changes...");
    info!("   Auto-detecting and encrypting sensitive data");
    info!("");

    // Create channel for clipboard change notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Create and start clipboard monitor
    let mut monitor = ClipboardMonitor::new();

    // Spawn monitor task
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = monitor.start(tx).await {
            error!("Clipboard monitor error: {}", e);
        }
    });

    // Process clipboard changes and store them
    let mut item_count = 0;
    while let Some(change) = rx.recv().await {
        item_count += 1;

        info!("📋 Clipboard changed (count: {})", change.change_count);
        info!("   Types: {:?}", change.types);

        // Get string content if available
        if let Some(text) = ClipboardMonitor::get_string() {
            // Process the text data
            let processed = DataProcessor::process_text(&text, &change.types);

            // Encrypt if sensitive
            let (blob_data, is_encrypted) = if processed.is_sensitive {
                match encryptor.encrypt(&processed.blob) {
                    Ok(encrypted) => {
                        info!("   🔐 Encrypted sensitive data ({} → {} bytes)",
                              processed.blob.len(), encrypted.len());
                        (encrypted, true)
                    }
                    Err(e) => {
                        error!("   ✗ Encryption failed: {}, storing unencrypted", e);
                        (processed.blob.clone(), false)
                    }
                }
            } else {
                (processed.blob.clone(), false)
            };

            // Store blob
            match db.store_blob(&blob_data) {
                Ok(blob_id) => {
                    // Store metadata
                    let timestamp = chrono::Utc::now().timestamp();
                    match db.store_item(
                        timestamp,
                        processed.data_type.as_str(),
                        processed.is_sensitive,
                        is_encrypted,
                        processed.preview_text.as_deref(),
                        processed.blob.len() as i64, // Original size
                        blob_id,
                        processed.metadata.as_deref(),
                    ) {
                        Ok(item_id) => {
                            let sensitive_marker = if processed.is_sensitive { " 🔒" } else { "" };
                            info!("   ✓ Stored as {} item #{} (blob #{}){}",
                                  processed.data_type.as_str(), item_id, blob_id, sensitive_marker);
                            if let Some(preview) = &processed.preview_text {
                                info!("   Preview: {}", preview);
                            }
                        }
                        Err(e) => error!("   ✗ Failed to store item metadata: {}", e),
                    }
                }
                Err(e) => error!("   ✗ Failed to store blob: {}", e),
            }
        } else {
            info!("   (Non-text content, skipped for Phase 2)");
        }

        info!("");

        // Show stats every 10 items
        if item_count % 10 == 0 {
            if let Ok(count) = db.count_items() {
                if let Ok(size) = db.get_db_size() {
                    info!("📊 Stats: {} items stored, {} KB database size", count, size / 1024);
                    info!("");
                }
            }
        }
    }

    monitor_handle.await.ok();
}
