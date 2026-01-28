// Status bar (menu bar) icon and menu using native macOS APIs
use objc2::rc::Retained;
use objc2::ClassType;
use objc2::sel;
use objc2_app_kit::{NSStatusBar, NSStatusItem, NSMenu, NSMenuItem, NSVariableStatusItemLength, NSApplication};
use objc2_foundation::{NSString, MainThreadMarker};
use std::sync::{Arc, Mutex};
use crate::storage::Database;
use crate::ui::actions::MenuActions;

pub struct StatusBarController {
    #[allow(dead_code)]
    status_item: Retained<NSStatusItem>,
    db: Arc<Mutex<Database>>,
    actions: Arc<MenuActions>,
}

impl StatusBarController {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        let actions = Arc::new(MenuActions::new(Arc::clone(&db)));

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
            let menu = Self::build_menu(&db, mtm);

            // Set menu
            status_item.setMenu(Some(&menu));

            log::info!("✓ Status bar icon created with menu");

            StatusBarController {
                status_item,
                db,
                actions,
            }
        }
    }

    unsafe fn build_menu(db: &Arc<Mutex<Database>>, mtm: MainThreadMarker) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        // Add "Show All History" item
        Self::add_menu_item(&menu, "Show All History", Some("h"), true, mtm);
        Self::add_separator(&menu, mtm);

        // Add recent clipboard items
        if let Ok(db_lock) = db.lock() {
            match db_lock.get_recent_items(10) {
                Ok(items) => {
                    if items.is_empty() {
                        Self::add_menu_item(&menu, "(No clipboard history yet)", None, false, mtm);
                    } else {
                        for (i, item) in items.iter().enumerate() {
                            let title = if let Some(preview) = &item.preview_text {
                                let preview_short = if preview.chars().count() > 50 {
                                    format!("{}...", preview.chars().take(50).collect::<String>())
                                } else {
                                    preview.clone()
                                };
                                let sensitive = if item.is_sensitive { " 🔒" } else { "" };
                                format!("{}{}", preview_short, sensitive)
                            } else {
                                format!("{} item", item.data_type)
                            };

                            Self::add_menu_item(&menu, &title, None, true, mtm);

                            // Add separator after every 5 items for readability
                            if i == 4 && items.len() > 5 {
                                Self::add_separator(&menu, mtm);
                            }
                        }
                    }
                }
                Err(_) => {
                    Self::add_menu_item(&menu, "(Error loading history)", None, false, mtm);
                }
            }
        }

        Self::add_separator(&menu, mtm);
        Self::add_menu_item(&menu, "Clear History", None, true, mtm);
        Self::add_separator(&menu, mtm);

        // Add Quit menu item with NSApp terminate action
        unsafe {
            let title = NSString::from_str("Quit");
            let key = NSString::from_str("q");
            let item = NSMenuItem::initWithTitle_action_keyEquivalent(
                mtm.alloc(),
                &title,
                Some(sel!(terminate:)),
                &key,
            );

            // Set target to NSApp
            let app = NSApplication::sharedApplication(mtm);
            item.setTarget(Some(&app));

            menu.addItem(&item);
        }

        menu
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
