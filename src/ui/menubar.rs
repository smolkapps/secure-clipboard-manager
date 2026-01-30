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
    encryptor: Arc<Mutex<Encryptor>>,
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
            encryptor: enc_arc,
            popup,
            status_bar: RefCell::new(None),
            hotkey: RefCell::new(None),
        }
    }

    /// Get a clone of the popup Arc for sharing with polling thread
    pub fn get_popup_arc(&self) -> Arc<Mutex<PopupWindow>> {
        Arc::clone(&self.popup)
    }
}

impl AppDelegate for MenuBarApp {
    fn did_finish_launching(&self) {
        log::info!("✓ Menu bar app launched");

        // Create status bar icon (pass popup and encryptor so menu items can trigger it)
        *self.status_bar.borrow_mut() = Some(StatusBarController::new(
            Arc::clone(&self.db),
            Arc::clone(&self.popup),
            Arc::clone(&self.encryptor),
        ));

        // Register global hotkey
        match HotkeyManager::new(Arc::clone(&self.popup)) {
            Ok(hotkey_mgr) => {
                *self.hotkey.borrow_mut() = Some(hotkey_mgr);
                log::info!("✓ Global hotkey registered: Cmd+Shift+C");
                log::info!("  Note: Press Cmd+Shift+C then check console");
            }
            Err(e) => log::error!("  Failed to register hotkey: {}", e),
        }

        // TODO: Start a timer to poll hotkey events from main thread
        // NSTimer with repeating callback is complex in Rust
        // For now, the hotkey manager's handle_events() needs to be called periodically
        log::warn!("⚠️  Hotkey event polling not yet implemented");
        log::warn!("   The hotkey is registered but events won't be processed");
        log::warn!("   Next step: Add NSTimer or dispatch_after to poll handle_events()");
        log::warn!("   Workaround: Manually call handle_events() for testing");

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
