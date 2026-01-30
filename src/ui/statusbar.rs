// Status bar (menu bar) icon and menu using native macOS APIs
use objc2::rc::Retained;
use objc2::{declare_class, msg_send, msg_send_id, sel, ClassType, DeclaredClass};
use objc2::mutability::InteriorMutable;
use objc2::runtime::AnyObject;
use objc2_app_kit::{
    NSStatusBar, NSStatusItem, NSMenu, NSMenuItem, NSVariableStatusItemLength,
    NSApplication, NSPasteboard, NSPasteboardTypeString,
};
use objc2_foundation::{NSString, NSObject, MainThreadMarker};
use std::sync::{Arc, Mutex, OnceLock};
use crate::storage::{Database, Encryptor};
use crate::ui::popup::PopupWindow;

// Global references accessible from ObjC action methods
static SHARED_POPUP: OnceLock<Arc<Mutex<PopupWindow>>> = OnceLock::new();
static SHARED_DB: OnceLock<Arc<Mutex<Database>>> = OnceLock::new();
static SHARED_ENCRYPTOR: OnceLock<Arc<Mutex<Encryptor>>> = OnceLock::new();

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
            // catch_unwind prevents panics from unwinding across ObjC boundary
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
                                                    Ok(decrypted) => {
                                                        log::info!("   Decrypted encrypted item successfully");
                                                        decrypted
                                                    }
                                                    Err(e) => {
                                                        log::error!("   Failed to decrypt item: {}", e);
                                                        return;
                                                    }
                                                }
                                            } else {
                                                log::error!("   Failed to lock encryptor");
                                                return;
                                            }
                                        } else {
                                            log::error!("   Encryptor not available, cannot decrypt item");
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
                                    log::info!("   Pasted to clipboard");
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
            if let Some(db_arc) = SHARED_DB.get() {
                if let Ok(db) = db_arc.lock() {
                    match db.cleanup_old_items(0) {
                        Ok(count) => log::info!("   Cleared {} items", count),
                        Err(e) => log::error!("   Failed to clear: {}", e),
                    }
                }
            }
        }

        /// Called by macOS each time the menu is about to be displayed.
        /// This is the NSMenuDelegate method that triggers a dynamic rebuild.
        #[method(menuNeedsUpdate:)]
        fn menu_needs_update(&self, menu: &NSMenu) {
            log::info!("Menu about to open, rebuilding items dynamically");
            unsafe {
                menu.removeAllItems();

                let mtm = MainThreadMarker::new()
                    .expect("menuNeedsUpdate: must be called on main thread");

                // self IS the MenuTarget, use it directly as the action target
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
    pub fn new(db: Arc<Mutex<Database>>, popup: Arc<Mutex<PopupWindow>>, encryptor: Arc<Mutex<Encryptor>>) -> Self {
        let _ = SHARED_DB.set(Arc::clone(&db));
        let _ = SHARED_POPUP.set(popup);
        let _ = SHARED_ENCRYPTOR.set(encryptor);

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

            // Set MenuTarget as the menu's delegate so menuNeedsUpdate: fires
            // each time the menu is opened. We use msg_send! to call setDelegate:
            // with our MenuTarget which implements the method informally.
            let delegate_ptr: *const MenuTarget = &*menu_target;
            let _: () = msg_send![&menu, setDelegate: delegate_ptr];

            // Initial population so the menu isn't empty before first open
            Self::populate_menu(&menu, &menu_target, mtm);

            status_item.setMenu(Some(&menu));
            log::info!("Status bar icon created with dynamic menu");

            StatusBarController { status_item, menu_target }
        }
    }

    /// Populate (or repopulate) the given menu with all standard items.
    /// Called both during initialization and from `menuNeedsUpdate:`.
    unsafe fn populate_menu(
        menu: &NSMenu,
        target: &MenuTarget,
        mtm: MainThreadMarker,
    ) {
        // Toggle history window - label reflects current state
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

        // Recent clipboard items - wired to pasteItem: action
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
                                    format!("{}{}", short, lock)
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
