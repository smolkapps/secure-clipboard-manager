// Menu bar application using Cacao
use cacao::appkit::AppDelegate;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::path::PathBuf;
use crate::storage::{AppConfig, Database, Encryptor};
use crate::ui::popup::PopupWindow;
use crate::ui::statusbar::StatusBarController;
use crate::ui::hotkey::HotkeyManager;
use crate::ui::launch_at_login;

pub struct MenuBarApp {
    db: Arc<Mutex<Database>>,
    encryptor: Arc<Mutex<Encryptor>>,
    popup: Arc<Mutex<PopupWindow>>,
    data_dir: PathBuf,
    status_bar: RefCell<Option<StatusBarController>>,
    hotkey: RefCell<Option<HotkeyManager>>,
}

impl MenuBarApp {
    pub fn new(db: Database, encryptor: Encryptor, data_dir: PathBuf) -> Self {
        log::info!("Creating menu bar app...");
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
            data_dir,
            status_bar: RefCell::new(None),
            hotkey: RefCell::new(None),
        }
    }

    /// Get a clone of the popup Arc for sharing with polling thread
    pub fn get_popup_arc(&self) -> Arc<Mutex<PopupWindow>> {
        Arc::clone(&self.popup)
    }

    /// Run first-launch setup: ask user about launch-at-login preference.
    /// Deferred via dispatch to run after the app's run loop is active.
    fn first_run_setup(&self) {
        let data_dir = self.data_dir.clone();

        dispatch::Queue::main().exec_async(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use objc2_app_kit::{NSAlert, NSAlertStyle, NSAlertFirstButtonReturn,
                                    NSApplication, NSApplicationActivationPolicy};
                use objc2_foundation::{NSString, MainThreadMarker};

                log::info!("Running first-launch setup dialog");

                unsafe {
                    let mtm = MainThreadMarker::new()
                        .expect("must be on main thread");

                    // Temporarily become a regular app so the alert is visible
                    let app = NSApplication::sharedApplication(mtm);
                    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
                    #[allow(deprecated)]
                    app.activateIgnoringOtherApps(true);

                    let alert = NSAlert::new(mtm);
                    alert.setAlertStyle(NSAlertStyle::Informational);
                    alert.setMessageText(&NSString::from_str(
                        "Welcome to ClipVault!"
                    ));
                    alert.setInformativeText(&NSString::from_str(
                        "ClipVault keeps your clipboard history accessible \
                         from the menu bar.\n\n\
                         Would you like ClipVault to start automatically \
                         when you log in?"
                    ));
                    alert.addButtonWithTitle(&NSString::from_str("Yes, Start at Login"));
                    alert.addButtonWithTitle(&NSString::from_str("No Thanks"));

                    let response = alert.runModal();
                    let wants_login = response == NSAlertFirstButtonReturn;

                    // Switch back to accessory (no dock icon)
                    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

                    let mut config = AppConfig::load(&data_dir);
                    config.launch_at_login = wants_login;
                    config.first_run_complete = true;

                    if let Err(e) = config.save(&data_dir) {
                        log::error!("Failed to save config: {}", e);
                    }

                    if let Err(e) = launch_at_login::sync(wants_login) {
                        log::error!("Failed to configure launch at login: {}", e);
                    }

                    log::info!("Launch at Login: {}",
                        if wants_login { "enabled" } else { "disabled" });
                }
            }));
        });
    }
}

impl AppDelegate for MenuBarApp {
    fn did_finish_launching(&self) {
        log::info!("Menu bar app launched");

        // First-run setup (must happen before building menus so toggle state is correct)
        let config = AppConfig::load(&self.data_dir);
        if !config.first_run_complete {
            self.first_run_setup();
        } else {
            // Sync launch-at-login state on every launch to keep plist in sync
            if let Err(e) = launch_at_login::sync(config.launch_at_login) {
                log::error!("Failed to sync launch at login: {}", e);
            }
        }

        // Create status bar icon (pass popup, encryptor, and data_dir so menu items work)
        *self.status_bar.borrow_mut() = Some(StatusBarController::new(
            Arc::clone(&self.db),
            Arc::clone(&self.popup),
            Arc::clone(&self.encryptor),
            self.data_dir.clone(),
        ));

        // Register global hotkey (events polled in main.rs)
        match HotkeyManager::new() {
            Ok(hotkey_mgr) => {
                *self.hotkey.borrow_mut() = Some(hotkey_mgr);
                log::info!("Global hotkey registered: Cmd+Shift+C");
            }
            Err(e) => log::error!("Failed to register hotkey: {}", e),
        }

        // Get clipboard history stats
        if let Ok(db) = self.db.lock() {
            match db.count_items() {
                Ok(count) => log::info!("  {} items in clipboard history", count),
                Err(e) => log::error!("  Failed to count items: {}", e),
            }
        }

        log::info!("Menu bar app running! Press Cmd+Shift+C to show clipboard history");
    }

    fn should_terminate_after_last_window_closed(&self) -> bool {
        false // Keep running as menu bar app
    }
}
