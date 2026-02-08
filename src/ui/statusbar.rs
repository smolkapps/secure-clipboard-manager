// Status bar (menu bar) icon and menu using native macOS APIs
use objc2::rc::Retained;
use objc2::{declare_class, msg_send, msg_send_id, sel, ClassType, DeclaredClass};
use objc2::mutability::InteriorMutable;
use objc2::runtime::AnyObject;
use objc2_app_kit::{
    NSStatusBar, NSStatusItem, NSMenu, NSMenuItem, NSVariableStatusItemLength,
    NSApplication, NSPasteboard, NSPasteboardTypeString,
    NSAlert, NSAlertStyle, NSAlertFirstButtonReturn,
};
use objc2_foundation::{NSString, NSObject, MainThreadMarker};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use crate::storage::{AppConfig, Database, Encryptor};
use crate::ui::popup::PopupWindow;
use crate::ui::launch_at_login;

// Global references accessible from ObjC action methods
static SHARED_POPUP: OnceLock<Arc<Mutex<PopupWindow>>> = OnceLock::new();
static SHARED_DB: OnceLock<Arc<Mutex<Database>>> = OnceLock::new();
static SHARED_ENCRYPTOR: OnceLock<Arc<Mutex<Encryptor>>> = OnceLock::new();
static SHARED_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

declare_class!(
    struct MenuTarget;

    unsafe impl ClassType for MenuTarget {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "ClipVaultMenuTarget";
    }

    impl DeclaredClass for MenuTarget {
        type Ivars = ();
    }

    unsafe impl MenuTarget {
        #[method(showHistory:)]
        fn show_history(&self, _sender: &AnyObject) {
            log::info!("Show/Hide History menu item clicked");
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(popup) = SHARED_POPUP.get() {
                    if let Ok(mut popup) = popup.lock() {
                        popup.toggle();
                    } else {
                        log::error!("Popup mutex poisoned");
                    }
                }
            }));
        }

        #[method(pasteItem:)]
        fn paste_item(&self, sender: &AnyObject) {
            unsafe {
                let menu_item: &NSMenuItem = &*(sender as *const AnyObject as *const NSMenuItem);
                let item_id = menu_item.tag();
                log::info!("Paste item (id={}) clicked", item_id);
                if let Some(db_arc) = SHARED_DB.get() {
                    if let Ok(db) = db_arc.lock() {
                        if let Ok(items) = db.get_recent_items(100) {
                            if let Some(item) = items.iter().find(|i| i.id == item_id as i64) {
                                if let Ok(blob) = db.get_blob(item.data_blob_id) {
                                    let data = if item.is_encrypted {
                                        if let Some(enc_arc) = SHARED_ENCRYPTOR.get() {
                                            if let Ok(enc) = enc_arc.lock() {
                                                match enc.decrypt(&blob) {
                                                    Ok(decrypted) => decrypted,
                                                    Err(e) => {
                                                        log::error!("Failed to decrypt item: {}", e);
                                                        return;
                                                    }
                                                }
                                            } else {
                                                return;
                                            }
                                        } else {
                                            return;
                                        }
                                    } else {
                                        blob
                                    };
                                    let pb = NSPasteboard::generalPasteboard();
                                    pb.clearContents();
                                    let text = String::from_utf8_lossy(&data);
                                    let ns_str = NSString::from_str(&text);
                                    pb.setString_forType(&ns_str, NSPasteboardTypeString);
                                    log::info!("Pasted item {} to clipboard", item_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[method(clearHistory:)]
        fn clear_history(&self, _sender: &AnyObject) {
            log::info!("Clear History clicked");
            dispatch::Queue::main().exec_async(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    unsafe {
                        let mtm = MainThreadMarker::new()
                            .expect("must be on main thread");
                        let alert = NSAlert::new(mtm);
                        alert.setAlertStyle(NSAlertStyle::Warning);
                        alert.setMessageText(&NSString::from_str(
                            "Clear All Clipboard History?"
                        ));
                        alert.setInformativeText(&NSString::from_str(
                            "This will remove all items from your clipboard history."
                        ));
                        alert.addButtonWithTitle(&NSString::from_str("Clear History"));
                        alert.addButtonWithTitle(&NSString::from_str("Cancel"));

                        let response = alert.runModal();
                        if response == NSAlertFirstButtonReturn {
                            log::info!("User confirmed clear");
                            if let Some(db_arc) = SHARED_DB.get() {
                                if let Ok(db) = db_arc.lock() {
                                    match db.soft_delete_all_items() {
                                        Ok(count) => log::info!("Soft-deleted {} items", count),
                                        Err(e) => log::error!("Failed to clear: {}", e),
                                    }
                                }
                            }
                        }
                    }
                }));
            });
        }

        #[method(toggleLaunchAtLogin:)]
        fn toggle_launch_at_login(&self, _sender: &AnyObject) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(data_dir) = SHARED_DATA_DIR.get() {
                    let mut config = AppConfig::load(data_dir);
                    config.launch_at_login = !config.launch_at_login;

                    if let Err(e) = launch_at_login::sync(config.launch_at_login) {
                        log::error!("Failed to toggle launch at login: {}", e);
                        return;
                    }

                    if let Err(e) = config.save(data_dir) {
                        log::error!("Failed to save config: {}", e);
                        return;
                    }

                    log::info!("Launch at Login: {}", if config.launch_at_login { "enabled" } else { "disabled" });
                }
            }));
        }

        #[method(menuNeedsUpdate:)]
        fn menu_needs_update(&self, menu: &NSMenu) {
            unsafe {
                menu.removeAllItems();

                let mtm = MainThreadMarker::new()
                    .expect("menuNeedsUpdate: must be called on main thread");

                StatusBarController::populate_menu(menu, self, mtm);
            }
        }
    }
);

impl MenuTarget {
    fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}

pub struct StatusBarController {
    #[allow(dead_code)]
    status_item: Retained<NSStatusItem>,
    #[allow(dead_code)]
    menu_target: Retained<MenuTarget>,
}

impl StatusBarController {
    pub fn new(
        db: Arc<Mutex<Database>>,
        popup: Arc<Mutex<PopupWindow>>,
        encryptor: Arc<Mutex<Encryptor>>,
        data_dir: PathBuf,
    ) -> Self {
        let _ = SHARED_DB.set(Arc::clone(&db));
        let _ = crate::ui::popup::POPUP_FOR_KEYS.set(Arc::clone(&popup));
        let _ = SHARED_POPUP.set(popup);
        let _ = SHARED_ENCRYPTOR.set(encryptor);
        let _ = SHARED_DATA_DIR.set(data_dir);

        unsafe {
            let mtm = MainThreadMarker::new().expect("Must be on main thread");
            let menu_target = MenuTarget::new(mtm);

            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            if let Some(button) = status_item.button(mtm) {
                button.setTitle(&NSString::from_str("📋"));
            }

            let menu = NSMenu::new(mtm);
            menu.setAutoenablesItems(false);

            let delegate_ptr: *const MenuTarget = &*menu_target;
            let _: () = msg_send![&menu, setDelegate: delegate_ptr];

            Self::populate_menu(&menu, &menu_target, mtm);

            status_item.setMenu(Some(&menu));
            log::info!("Status bar icon created");

            StatusBarController { status_item, menu_target }
        }
    }

    /// Populate (or repopulate) the given menu with all standard items.
    unsafe fn populate_menu(
        menu: &NSMenu,
        target: &MenuTarget,
        mtm: MainThreadMarker,
    ) {
        // Toggle history window
        let history_label = if let Some(popup_arc) = SHARED_POPUP.get() {
            if let Ok(popup) = popup_arc.lock() {
                if popup.is_visible() { "Hide History Window" } else { "Show All History" }
            } else {
                "Show All History"
            }
        } else {
            "Show All History"
        };
        Self::add_action_item(menu, history_label, Some("h"), sel!(showHistory:), target, mtm);
        Self::add_separator(menu, mtm);

        // Recent clipboard items
        if let Some(db_arc) = SHARED_DB.get() {
            if let Ok(db) = db_arc.lock() {
                match db.get_recent_items(10) {
                    Ok(items) if items.is_empty() => {
                        Self::add_disabled_item(menu, "(No clipboard history yet)", mtm);
                    }
                    Ok(items) => {
                        for (i, item) in items.iter().enumerate() {
                            let title = match &item.preview_text {
                                Some(preview) => {
                                    let short = if preview.chars().count() > 50 {
                                        format!("{}...", preview.chars().take(50).collect::<String>())
                                    } else {
                                        preview.clone()
                                    };
                                    let lock = if item.is_sensitive { " 🔒" } else { "" };
                                    let count = if item.copy_count > 1 {
                                        format!(" (×{})", item.copy_count)
                                    } else {
                                        String::new()
                                    };
                                    format!("{}{}{}", short, count, lock)
                                }
                                None => format!("{} item", item.data_type),
                            };

                            let title_ns = NSString::from_str(&title);
                            let key_ns = NSString::from_str("");
                            let mi = NSMenuItem::initWithTitle_action_keyEquivalent(
                                mtm.alloc(), &title_ns, Some(sel!(pasteItem:)), &key_ns,
                            );
                            mi.setEnabled(true);
                            mi.setTarget(Some(target));
                            mi.setTag(item.id as isize);
                            menu.addItem(&mi);

                            if i == 4 && items.len() > 5 {
                                Self::add_separator(menu, mtm);
                            }
                        }
                    }
                    Err(_) => {
                        Self::add_disabled_item(menu, "(Error loading history)", mtm);
                    }
                }
            }
        }

        Self::add_separator(menu, mtm);
        Self::add_action_item(menu, "Clear History", None, sel!(clearHistory:), target, mtm);
        Self::add_separator(menu, mtm);

        // Launch at Login toggle (with checkmark for current state)
        let launch_enabled = SHARED_DATA_DIR.get()
            .map(|dir| AppConfig::load(dir).launch_at_login)
            .unwrap_or(false);

        let login_title = NSString::from_str("Launch at Login");
        let login_key = NSString::from_str("");
        let login_item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(), &login_title, Some(sel!(toggleLaunchAtLogin:)), &login_key,
        );
        login_item.setEnabled(true);
        login_item.setTarget(Some(target));
        if launch_enabled {
            let _: () = msg_send![&login_item, setState: 1_isize]; // NSOnState = 1
        }
        menu.addItem(&login_item);
        Self::add_separator(menu, mtm);

        // Quit
        let quit_title = NSString::from_str("Quit");
        let quit_key = NSString::from_str("q");
        let quit_item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(), &quit_title, Some(sel!(terminate:)), &quit_key,
        );
        let app = NSApplication::sharedApplication(mtm);
        quit_item.setTarget(Some(&app));
        menu.addItem(&quit_item);
    }

    unsafe fn add_action_item(
        menu: &NSMenu,
        title: &str,
        key_equiv: Option<&str>,
        action: objc2::runtime::Sel,
        target: &MenuTarget,
        mtm: MainThreadMarker,
    ) {
        let title_ns = NSString::from_str(title);
        let key_ns = NSString::from_str(key_equiv.unwrap_or(""));
        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(), &title_ns, Some(action), &key_ns,
        );
        item.setEnabled(true);
        item.setTarget(Some(target));
        menu.addItem(&item);
    }

    unsafe fn add_disabled_item(menu: &NSMenu, title: &str, mtm: MainThreadMarker) {
        let title_ns = NSString::from_str(title);
        let key_ns = NSString::from_str("");
        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(), &title_ns, None, &key_ns,
        );
        item.setEnabled(false);
        menu.addItem(&item);
    }

    unsafe fn add_separator(menu: &NSMenu, mtm: MainThreadMarker) {
        menu.addItem(&NSMenuItem::separatorItem(mtm));
    }
}
