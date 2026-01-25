// Menu bar application using Cacao
use cacao::appkit::AppDelegate;
use std::sync::{Arc, Mutex};
use crate::storage::Database;
use crate::ui::popup::PopupWindow;

pub struct MenuBarApp {
    db: Arc<Mutex<Database>>,
    popup: Arc<Mutex<PopupWindow>>,
}

impl MenuBarApp {
    pub fn new(db: Database) -> Self {
        log::info!("📱 Creating menu bar app...");
        let db_arc = Arc::new(Mutex::new(db));
        let popup = Arc::new(Mutex::new(PopupWindow::new(Arc::clone(&db_arc))));

        MenuBarApp {
            db: db_arc,
            popup,
        }
    }
}

impl AppDelegate for MenuBarApp {
    fn did_finish_launching(&self) {
        log::info!("✓ Menu bar app launched");

        // Get clipboard history stats
        if let Ok(db) = self.db.lock() {
            match db.count_items() {
                Ok(count) => log::info!("  {} items in clipboard history", count),
                Err(e) => log::error!("  Failed to count items: {}", e),
            }
        }

        log::info!("");
        log::info!("🎯 Menu bar app running!");
        log::info!("   Press Ctrl+C to quit");
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        false // Keep running as menu bar app
    }
}
