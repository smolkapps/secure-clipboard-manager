// Popup window for clipboard history
// Phase 5: Simplified popup - will enhance with full UI later
use std::sync::{Arc, Mutex};
use crate::storage::Database;

pub struct PopupWindow {
    db: Arc<Mutex<Database>>,
    visible: bool,
}

impl PopupWindow {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        log::info!("✓ Popup window system initialized");

        PopupWindow {
            db,
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;

        if self.visible {
            self.show();
        } else {
            self.hide();
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        log::info!("📋 Popup window shown (Cmd+Shift+V pressed)");

        // Get recent items from database
        if let Ok(db) = self.db.lock() {
            match db.get_recent_items(20) {
                Ok(items) => {
                    log::info!("   Showing {} clipboard items:", items.len());
                    for (i, item) in items.iter().enumerate().take(5) {
                        if let Some(preview) = &item.preview_text {
                            let preview_short = if preview.len() > 40 {
                                format!("{}...", &preview[..40])
                            } else {
                                preview.clone()
                            };
                            log::info!("   {}. {}", i + 1, preview_short);
                        }
                    }
                    if items.len() > 5 {
                        log::info!("   ... and {} more items", items.len() - 5);
                    }
                }
                Err(e) => log::error!("   Failed to get items: {}", e),
            }
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        log::info!("✖ Popup window hidden");
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}
