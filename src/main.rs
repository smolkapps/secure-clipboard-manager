// Clipboard Manager - macOS Native Clipboard History Manager
// Phase 4: Menu Bar UI

mod clipboard;
mod storage;
mod ui;

use cacao::appkit::App;
use clipboard::ClipboardMonitor;
use storage::{Database, DataProcessor, Encryptor};
use ui::MenuBarApp;
use log::{error, info};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Acquire an exclusive file lock. Returns the File handle which must be kept
/// alive for the duration of the process — the lock is released automatically
/// when the handle is dropped (including on crash/kill).
fn acquire_instance_lock(data_dir: &Path) -> Option<File> {
    let lock_path = data_dir.join("instance.lock");
    let file = File::create(&lock_path).ok()?;
    let fd = file.as_raw_fd();
    let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
        Some(file)
    } else {
        None
    }
}

fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Clipboard Manager - Phase 4: Menu Bar UI");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize data directory
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clipboard-manager");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    // Single-instance check: acquire exclusive flock on data_dir/instance.lock.
    // The lock is per-user (each user has their own ~/Library/Application Support/)
    // so multiple users can each run their own instance without conflict.
    let _instance_lock = match acquire_instance_lock(&data_dir) {
        Some(lock) => {
            info!("✓ Single-instance lock acquired (PID: {})", std::process::id());
            lock
        }
        None => {
            error!("Another ClipVault instance is already running for this user. Exiting.");
            std::process::exit(0);
        }
    };

    // Initialize database
    let db_path = data_dir.join("clipboard.db");
    let db = Database::new(db_path.clone())
        .expect("Failed to initialize database");

    info!("✓ Database initialized at: {}", db_path.display());
    info!("  Items in history: {}", db.count_items().unwrap_or(0));
    info!("  Database size: {} KB", db.get_db_size().unwrap_or(0) / 1024);

    // Purge soft-deleted items older than 7 days
    match db.purge_deleted_items() {
        Ok(0) => {}
        Ok(n) => info!("  Purged {} expired deleted items", n),
        Err(e) => log::error!("  Failed to purge deleted items: {}", e),
    }

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
    info!("✓ Starting clipboard monitor in background...");

    // Wrap database and encryptor in Arc<Mutex> for thread sharing
    let db_shared = Arc::new(Mutex::new(db));
    let encryptor_shared = Arc::new(Mutex::new(encryptor));

    // Clone for background thread
    let db_clone = Arc::clone(&db_shared);
    let encryptor_clone = Arc::clone(&encryptor_shared);

    // Spawn background thread for clipboard monitoring
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut monitor = ClipboardMonitor::new();

            info!("✓ Clipboard monitor initialized (polling every 500ms)");
            info!("   Auto-detecting and encrypting sensitive data");
            info!("");

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

                // Try to get image data first
                let processed_opt = if let Some((image_data, uti_type)) = ClipboardMonitor::get_image() {
                    info!("   🖼️  Image detected: {} ({} bytes)", uti_type, image_data.len());
                    match DataProcessor::process_image(&image_data, &uti_type) {
                        Ok(processed) => Some(processed),
                        Err(e) => {
                            error!("   ✗ Failed to process image: {}", e);
                            None
                        }
                    }
                } else if let Some(text) = ClipboardMonitor::get_string() {
                    // Process text data
                    Some(DataProcessor::process_text(&text, &change.types))
                } else {
                    info!("   (Unsupported content type)");
                    None
                };

                // Store processed data
                if let Some(processed) = processed_opt {
                    // Encrypt if sensitive
                    let (blob_data, is_encrypted) = if processed.is_sensitive {
                        if let Ok(enc) = encryptor_clone.lock() {
                            match enc.encrypt(&processed.blob) {
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
                        }
                    } else {
                        (processed.blob.clone(), false)
                    };

                    // Store to database
                    if let Ok(db) = db_clone.lock() {
                        // Remove existing duplicates before inserting the new entry
                        let prev_copy_count = match db.remove_duplicates(
                            processed.preview_text.as_deref(),
                            processed.data_type.as_str(),
                        ) {
                            Ok((_removed, prev_count)) => prev_count,
                            Err(e) => {
                                error!("   ✗ Failed to remove duplicates: {}", e);
                                0
                            }
                        };

                        match db.store_blob(&blob_data) {
                            Ok(blob_id) => {
                                let timestamp = chrono::Utc::now().timestamp();
                                match db.store_item(
                                    timestamp,
                                    processed.data_type.as_str(),
                                    processed.is_sensitive,
                                    is_encrypted,
                                    processed.preview_text.as_deref(),
                                    processed.blob.len() as i64,
                                    blob_id,
                                    processed.metadata.as_deref(),
                                    prev_copy_count + 1,
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
                    }
                }

                info!("");

                // Show stats every 10 items
                if item_count % 10 == 0 {
                    if let Ok(db) = db_clone.lock() {
                        if let Ok(count) = db.count_items() {
                            if let Ok(size) = db.get_db_size() {
                                info!("📊 Stats: {} items stored, {} KB database size", count, size / 1024);
                                info!("");
                            }
                        }
                    }
                }
            }

            monitor_handle.await.ok();
        });
    });

    // Create menu bar app with database and encryptor access
    // Need to create separate connections for UI thread
    let db_path2 = data_dir.join("clipboard.db");
    let db_for_ui = Database::new(db_path2)
        .expect("Failed to initialize database for UI");

    let key_path2 = data_dir.join("encryption.key");
    let encryptor_for_ui = Encryptor::new(key_path2)
        .expect("Failed to initialize encryptor for UI");

    let app = MenuBarApp::new(db_for_ui, encryptor_for_ui, data_dir);

    info!("Launching menu bar app...");

    // Start background thread that polls global hotkey events directly
    // and dispatches toggle to main thread
    let popup_for_polling = app.get_popup_arc();
    std::thread::spawn(move || {
        use global_hotkey::GlobalHotKeyEvent;
        use std::time::{Duration, Instant};
        use dispatch::Queue;

        let mut last_toggle = Instant::now();
        let debounce_duration = Duration::from_millis(200); // Ignore events within 200ms

        loop {
            std::thread::sleep(Duration::from_millis(50)); // Poll at 20Hz

            // Check for hotkey events
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                let now = Instant::now();

                // Debounce: ignore if too soon after last toggle
                if now.duration_since(last_toggle) >= debounce_duration {
                    log::info!("🔥 Hotkey event received: {:?}", event.id);
                    last_toggle = now;

                    // Dispatch to main thread using dispatch queue
                    let popup_clone = Arc::clone(&popup_for_polling);

                    Queue::main().exec_async(move || {
                        // Catch any panics to prevent crashes through Obj-C boundary
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            if let Ok(mut popup) = popup_clone.lock() {
                                popup.toggle();
                            }
                        }));
                    });
                } else {
                    log::debug!("Ignoring duplicate hotkey event (debouncing)");
                }
            }

            // Key events are handled by KeyHandlingTextView::keyDown: directly
        }
    });

    // Run the app (this blocks)
    App::new("com.clipboard-manager.app", app).run();
}
