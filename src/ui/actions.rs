// Menu action handlers
use crate::storage::Database;
use std::sync::{Arc, Mutex};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::NSString;

pub struct MenuActions {
    db: Arc<Mutex<Database>>,
}

impl MenuActions {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        MenuActions { db }
    }

    pub fn show_history(&self) {
        log::info!("📋 Show History action triggered");
        
        if let Ok(db) = self.db.lock() {
            match db.get_recent_items(20) {
                Ok(items) => {
                    log::info!("   Displaying {} clipboard items:", items.len());
                    for (i, item) in items.iter().enumerate() {
                        if let Some(preview) = &item.preview_text {
                            let preview_short = if preview.chars().count() > 60 {
                                format!("{}...", preview.chars().take(60).collect::<String>())
                            } else {
                                preview.clone()
                            };
                            let sensitive = if item.is_sensitive { " 🔒" } else { "" };
                            log::info!("   {}. {}{}", i + 1, preview_short, sensitive);
                        }
                    }
                }
                Err(e) => log::error!("   Failed to get items: {}", e),
            }
        }
    }

    pub fn clear_history(&self) {
        log::info!("🗑️  Clear History action triggered");
        
        if let Ok(db) = self.db.lock() {
            // Delete all items older than 0 days (i.e., all items)
            match db.cleanup_old_items(0) {
                Ok(count) => log::info!("   ✓ Cleared {} items from history", count),
                Err(e) => log::error!("   ✗ Failed to clear history: {}", e),
            }
        }
    }

    pub fn paste_item(&self, item_id: i64) {
        log::info!("📋 Paste item #{} action triggered", item_id);
        
        if let Ok(db) = self.db.lock() {
            // Get the item
            match db.get_recent_items(100) {
                Ok(items) => {
                    if let Some(item) = items.iter().find(|i| i.id == item_id) {
                        match db.get_blob(item.data_blob_id) {
                            Ok(blob) => {
                                // Decrypt if encrypted
                                let data = if item.is_encrypted {
                                    log::info!("   🔓 Decrypting sensitive data...");
                                    // TODO: Decrypt with encryptor
                                    blob
                                } else {
                                    blob
                                };

                                // Put on clipboard
                                unsafe {
                                    let pasteboard = NSPasteboard::generalPasteboard();
                                    pasteboard.clearContents();
                                    
                                    let text = String::from_utf8_lossy(&data);
                                    let ns_string = NSString::from_str(&text);
                                    pasteboard.setString_forType(&ns_string, objc2_app_kit::NSPasteboardTypeString);
                                    
                                    log::info!("   ✓ Pasted to clipboard");
                                }
                            }
                            Err(e) => log::error!("   ✗ Failed to get blob: {}", e),
                        }
                    }
                }
                Err(e) => log::error!("   ✗ Failed to get items: {}", e),
            }
        }
    }

    pub fn quit(&self) {
        log::info!("👋 Quit action triggered");
        log::info!("   Shutting down clipboard manager...");
        std::process::exit(0);
    }
}
