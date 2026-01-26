// Popup window for clipboard history with native Cocoa UI
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType, NSTextView, NSScrollView};
use objc2_foundation::{NSString, NSRect, NSPoint, NSSize, MainThreadMarker};
use crate::storage::{Database, Encryptor, ClipboardItem};
use objc2_app_kit::NSPasteboard;

pub struct PopupWindow {
    db: Arc<Mutex<Database>>,
    encryptor: Arc<Mutex<Encryptor>>,
    window: RefCell<Option<Retained<NSWindow>>>,
    text_view: RefCell<Option<Retained<NSTextView>>>,
    items: RefCell<Vec<ClipboardItem>>,
    selected_index: RefCell<usize>,
    visible: bool,
}

impl PopupWindow {
    pub fn new(db: Arc<Mutex<Database>>, encryptor: Arc<Mutex<Encryptor>>) -> Self {
        log::info!("✓ Popup window system initialized");

        PopupWindow {
            db,
            encryptor,
            window: RefCell::new(None),
            text_view: RefCell::new(None),
            items: RefCell::new(Vec::new()),
            selected_index: RefCell::new(0),
            visible: false,
        }
    }

    unsafe fn build_window(&self, mtm: MainThreadMarker) -> Retained<NSWindow> {
        let content_rect = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(600.0, 400.0),
        );

        let style_mask = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Resizable;

        let window = NSWindow::initWithContentRect_styleMask_backing_defer(
            mtm.alloc(),
            content_rect,
            style_mask,
            NSBackingStoreType::NSBackingStoreBuffered,
            false,
        );

        window.setTitle(&NSString::from_str("Clipboard History"));
        window.center();
        window.setLevel(3); // NSFloatingWindowLevel
        window.setHidesOnDeactivate(true);

        // Create text view for displaying items
        let scroll_view = NSScrollView::new(mtm);
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setFrame(content_rect);

        let text_view = NSTextView::new(mtm);
        text_view.setEditable(false);
        scroll_view.setDocumentView(Some(&text_view));

        window.setContentView(Some(&scroll_view));

        // Store text view for later updates
        *self.text_view.borrow_mut() = Some(text_view);

        window
    }

    fn load_items(&self) {
        if let Ok(db) = self.db.lock() {
            match db.get_recent_items(20) {
                Ok(items) => {
                    *self.items.borrow_mut() = items;
                    *self.selected_index.borrow_mut() = 0;
                }
                Err(e) => log::error!("Failed to load items: {}", e),
            }
        }
    }

    fn refresh_display(&self) {
        let items = self.items.borrow();
        let selected_idx = *self.selected_index.borrow();

        let mut display_text = String::new();
        display_text.push_str("📋 Clipboard History\n");
        display_text.push_str("═══════════════════════════════════════════\n\n");

        if items.is_empty() {
            display_text.push_str("No clipboard history yet.\n");
            display_text.push_str("Copy something to get started!\n");
        } else {
            display_text.push_str("Navigation: ↑↓ or j/k • Enter to paste • Esc to close\n\n");

            for (i, item) in items.iter().enumerate() {
                let marker = if i == selected_idx { "▶ " } else { "  " };
                let icon = match item.data_type.as_str() {
                    "image" => "🖼️",
                    "url" => "🔗",
                    _ => "📝",
                };
                let lock = if item.is_sensitive { " 🔒" } else { "" };

                let preview = item.preview_text.as_deref().unwrap_or("[No preview]");
                let preview_short = if preview.len() > 60 {
                    format!("{}...", &preview[..60])
                } else {
                    preview.to_string()
                };

                display_text.push_str(&format!("{}{} {}{}\n", marker, icon, preview_short, lock));
            }
        }

        // Update text view
        if let Some(text_view) = self.text_view.borrow().as_ref() {
            unsafe {
                let ns_string = NSString::from_str(&display_text);
                text_view.setString(&ns_string);
            }
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
        log::info!("📋 Popup window shown (Cmd+Shift+C pressed)");

        unsafe {
            if let Some(mtm) = MainThreadMarker::new() {
                // Create window if it doesn't exist
                if self.window.borrow().is_none() {
                    let window = self.build_window(mtm);
                    *self.window.borrow_mut() = Some(window);
                }

                // Load and display items
                self.load_items();
                self.refresh_display();

                // Show window
                if let Some(window) = self.window.borrow().as_ref() {
                    window.makeKeyAndOrderFront(None);
                }
            }
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        log::info!("✖ Popup window hidden");

        if let Some(window) = self.window.borrow().as_ref() {
            unsafe {
                window.orderOut(None);
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn move_selection_down(&self) {
        let items_len = self.items.borrow().len();
        if items_len > 0 {
            let mut idx = self.selected_index.borrow_mut();
            *idx = (*idx + 1) % items_len;
            self.refresh_display();
        }
    }

    pub fn move_selection_up(&self) {
        let items_len = self.items.borrow().len();
        if items_len > 0 {
            let mut idx = self.selected_index.borrow_mut();
            *idx = if *idx == 0 { items_len - 1 } else { *idx - 1 };
            self.refresh_display();
        }
    }

    pub fn paste_and_close(&mut self) {
        let idx = *self.selected_index.borrow();

        // Clone the item we need before borrowing
        let item_to_paste = {
            let items = self.items.borrow();
            items.get(idx).cloned()
        };

        if let Some(item) = item_to_paste {
            log::info!("📋 Pasting item #{}", item.id);

            if let Ok(db) = self.db.lock() {
                if let Ok(blob) = db.get_blob(item.data_blob_id) {
                    // Decrypt if needed
                    let data = if item.is_encrypted {
                        if let Ok(enc) = self.encryptor.lock() {
                            enc.decrypt(&blob).unwrap_or_else(|e| {
                                log::error!("Decryption failed: {}", e);
                                blob.clone()
                            })
                        } else {
                            blob
                        }
                    } else {
                        blob
                    };

                    // Put on pasteboard
                    unsafe {
                        let pb = NSPasteboard::generalPasteboard();
                        pb.clearContents();

                        match item.data_type.as_str() {
                            "image" => {
                                // TODO: Set image data
                                log::info!("✓ Image paste not yet implemented");
                            }
                            _ => {
                                let text = String::from_utf8_lossy(&data);
                                let ns_str = NSString::from_str(&text);
                                let type_str = NSString::from_str("public.utf8-plain-text");
                                pb.setString_forType(&ns_str, &type_str);
                                log::info!("✓ Pasted text to clipboard");
                            }
                        }
                    }
                }
            }
        }

        self.hide();
    }
}
