// Status bar (menu bar) icon and menu using native macOS APIs
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::ClassType;
use objc2_app_kit::{NSStatusBar, NSStatusItem, NSMenu, NSMenuItem, NSVariableStatusItemLength};
use objc2_foundation::{NSString, MainThreadMarker};
use std::sync::{Arc, Mutex};
use crate::storage::Database;

pub struct StatusBarController {
    #[allow(dead_code)]
    status_item: Retained<NSStatusItem>,
    #[allow(dead_code)]
    db: Arc<Mutex<Database>>,
}

impl StatusBarController {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().expect("Must be on main thread");

            // Get system status bar
            let status_bar = NSStatusBar::systemStatusBar();

            // Create status bar item with variable length
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            // Set title/icon (using emoji for now, proper icon later)
            if let Some(button) = status_item.button(mtm) {
                let title = NSString::from_str("📋");
                button.setTitle(&title);
            }

            // Create menu
            let menu = NSMenu::new(mtm);
            menu.setAutoenablesItems(false);

            // Add menu items
            Self::add_menu_item(&menu, "Clipboard History (Cmd+Shift+V)", None, false, mtm);
            Self::add_separator(&menu, mtm);
            Self::add_menu_item(&menu, "Clear History", None, true, mtm);
            Self::add_separator(&menu, mtm);
            Self::add_menu_item(&menu, "Quit", Some("q"), true, mtm);

            // Set menu
            status_item.setMenu(Some(&menu));

            log::info!("✓ Status bar icon created with menu");

            StatusBarController {
                status_item,
                db,
            }
        }
    }

    unsafe fn add_menu_item(
        menu: &NSMenu,
        title: &str,
        key_equiv: Option<&str>,
        enabled: bool,
        mtm: MainThreadMarker,
    ) {
        let title_ns = NSString::from_str(title);
        let key_ns = NSString::from_str(key_equiv.unwrap_or(""));

        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &title_ns,
            None,
            &key_ns,
        );

        item.setEnabled(enabled);
        menu.addItem(&item);
    }

    unsafe fn add_separator(menu: &NSMenu, mtm: MainThreadMarker) {
        let separator = NSMenuItem::separatorItem(mtm);
        menu.addItem(&separator);
    }
}
