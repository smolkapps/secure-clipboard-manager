// Menu bar application using Cacao
use cacao::appkit::AppDelegate;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use crate::storage::{Database, Encryptor};
use crate::ui::popup::PopupWindow;
use crate::ui::statusbar::StatusBarController;
use crate::ui::hotkey::HotkeyManager;

pub struct MenuBarApp {
    db: Arc<Mutex<Database>>,
    popup: Arc<Mutex<PopupWindow>>,
    status_bar: RefCell<Option<StatusBarController>>,
    hotkey: RefCell<Option<HotkeyManager>>,
}

impl MenuBarApp {
    pub fn new(db: Database, encryptor: Encryptor) -> Self {
        log::info!("📱 Creating menu bar app...");
        let db_arc = Arc::new(Mutex::new(db));
        let enc_arc = Arc::new(Mutex::new(encryptor));
        let popup = Arc::new(Mutex::new(PopupWindow::new(
            Arc::clone(&db_arc),
            Arc::clone(&enc_arc)
        )));

        MenuBarApp {
            db: db_arc,
            popup,
            status_bar: RefCell::new(None),
            hotkey: RefCell::new(None),
        }
    }
}

impl AppDelegate for MenuBarApp {
    fn did_finish_launching(&self) {
        log::info!("✓ Menu bar app launched");

        // Create status bar icon
        *self.status_bar.borrow_mut() = Some(StatusBarController::new(Arc::clone(&self.db)));

        // Register global hotkey
        match HotkeyManager::new(Arc::clone(&self.popup)) {
            Ok(hotkey_mgr) => {
                *self.hotkey.borrow_mut() = Some(hotkey_mgr);
                log::info!("✓ Global hotkey registered: Cmd+Shift+C");

                // Note: Keyboard event handling is done in the popup window itself
                // via local event monitor when window is shown
            }
            Err(e) => log::error!("  Failed to register hotkey: {}", e),
        }

        // Get clipboard history stats
        if let Ok(db) = self.db.lock() {
            match db.count_items() {
                Ok(count) => log::info!("  {} items in clipboard history", count),
                Err(e) => log::error!("  Failed to count items: {}", e),
            }
        }

        log::info!("");
        log::info!("🎯 Menu bar app running!");
        log::info!("   Look for the 📋 icon in your menu bar");
        log::info!("   Press Cmd+Shift+C to show clipboard history");
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        false // Keep running as menu bar app
    }
}
