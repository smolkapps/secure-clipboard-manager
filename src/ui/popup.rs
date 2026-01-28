// Popup window for clipboard history with native Cocoa UI
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::declare_class;
use objc2::mutability::InteriorMutable;
use objc2::ClassType;
use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType, NSTextView, NSScrollView, NSTextViewDelegate};
use objc2_foundation::{NSString, NSRect, NSPoint, NSSize, MainThreadMarker, NSObject, NSObjectProtocol};
use crate::storage::{Database, Encryptor, ClipboardItem};
use objc2_app_kit::NSPasteboard;
use dispatch::Queue;

// Keyboard event handler delegate for NSTextView
declare_class!(
    struct PopupKeyHandler;

    unsafe impl ClassType for PopupKeyHandler {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "PopupKeyHandler";
    }

    unsafe impl NSObjectProtocol for PopupKeyHandler {}

    unsafe impl NSTextViewDelegate for PopupKeyHandler {
        // Handle special key commands (arrow keys, Enter, Escape)
        #[method(textView:doCommandBySelector:)]
        fn text_view_do_command(
            &self,
            _text_view: &NSTextView,
            selector: objc2::runtime::Sel,
        ) -> bool {
            let sel_name = selector.name();

            log::debug!("🔑 Key command: {}", sel_name);

            // Map selector names to key actions
            match sel_name {
                "moveDown:" => {
                    // Arrow down - send to main thread handler
                    Self::handle_navigation_key("down");
                    true // Consumed event
                }
                "moveUp:" => {
                    // Arrow up
                    Self::handle_navigation_key("up");
                    true
                }
                "insertNewline:" => {
                    // Enter key
                    Self::handle_navigation_key("enter");
                    true
                }
                "cancelOperation:" => {
                    // Escape key
                    Self::handle_navigation_key("escape");
                    true
                }
                _ => false // Let system handle it
            }
        }

        // Handle regular character input (for vim keys)
        #[method(insertText:)]
        fn insert_text(&self, string: &AnyObject) {
            // Try to get the string value
            if let Some(ns_string) = string.downcast_ref::<NSString>() {
                let text = ns_string.to_string();
                if let Some(ch) = text.chars().next() {
                    log::debug!("🔑 Character typed: '{}'", ch);

                    match ch {
                        'j' => Self::handle_navigation_key("down"),
                        'k' => Self::handle_navigation_key("up"),
                        _ => {} // Ignore other characters
                    }
                }
            }
        }
    }
);

// Global channel for key events
use std::sync::OnceLock;
use std::sync::mpsc::{channel, Sender, Receiver};

static KEY_EVENT_SENDER: OnceLock<Mutex<Sender<String>>> = OnceLock::new();

impl PopupKeyHandler {
    fn handle_navigation_key(key: &str) {
        log::info!("🔑 Navigation key pressed: {}", key);

        // Send key event through channel
        if let Some(sender_lock) = KEY_EVENT_SENDER.get() {
            if let Ok(sender) = sender_lock.lock() {
                let _ = sender.send(key.to_string());
            }
        }
    }
}

pub struct PopupWindow {
    db: Arc<Mutex<Database>>,
    encryptor: Arc<Mutex<Encryptor>>,
    window: RefCell<Option<Retained<NSWindow>>>,
    text_view: RefCell<Option<Retained<NSTextView>>>,
    delegate: RefCell<Option<Retained<PopupKeyHandler>>>,
    items: RefCell<Vec<ClipboardItem>>,
    selected_index: RefCell<usize>,
    visible: bool,
    key_event_receiver: Receiver<String>,
}

// SAFETY: PopupWindow contains NSWindow which is !Send, but we only access it
// from the main thread (via MainThreadMarker checks in show/hide methods).
// The toggle() method only flips a boolean and calls show/hide which are safe.
unsafe impl Send for PopupWindow {}

impl PopupWindow {
    pub fn new(db: Arc<Mutex<Database>>, encryptor: Arc<Mutex<Encryptor>>) -> Self {
        log::info!("✓ Popup window system initialized");

        // Create channel for keyboard events
        let (sender, receiver) = channel();

        // Store sender globally for delegate to use
        let _ = KEY_EVENT_SENDER.set(Mutex::new(sender));

        PopupWindow {
            db,
            encryptor,
            window: RefCell::new(None),
            text_view: RefCell::new(None),
            delegate: RefCell::new(None),
            items: RefCell::new(Vec::new()),
            selected_index: RefCell::new(0),
            visible: false,
            key_event_receiver: receiver,
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

        // Allow window to receive keyboard events
        window.setAcceptsMouseMovedEvents(true);

        // Create text view for displaying items
        let scroll_view = NSScrollView::new(mtm);
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setFrame(content_rect);

        let text_view = NSTextView::new(mtm);
        text_view.setEditable(true); // Make editable to receive key events
        text_view.setSelectable(false); // But don't allow text selection

        // Create and set the keyboard event delegate
        let delegate = PopupKeyHandler::new();
        text_view.setDelegate(Some(&delegate));

        // Store delegate to prevent deallocation
        *self.delegate.borrow_mut() = Some(delegate);

        scroll_view.setDocumentView(Some(&text_view));

        window.setContentView(Some(&scroll_view));

        // Store text view for later updates
        *self.text_view.borrow_mut() = Some(text_view);

        log::info!("✓ Keyboard event delegate registered");

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
                let preview_short = if preview.chars().count() > 60 {
                    format!("{}...", preview.chars().take(60).collect::<String>())
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
                log::info!("✓ On main thread, creating/showing window");

                // Create window if it doesn't exist
                if self.window.borrow().is_none() {
                    log::info!("Creating new window...");
                    let window = self.build_window(mtm);
                    *self.window.borrow_mut() = Some(window);
                }

                // Load and display items
                self.load_items();
                self.refresh_display();

                // Show window
                if let Some(window) = self.window.borrow().as_ref() {
                    log::info!("Calling makeKeyAndOrderFront on window");

                    // Make window visible and bring to front
                    window.makeKeyAndOrderFront(None);
                    window.orderFrontRegardless();

                    // Check if window is visible
                    let is_visible = window.isVisible();
                    log::info!("Window visibility after makeKeyAndOrderFront: {}", is_visible);

                    // Try to activate the app
                    use objc2_app_kit::NSApplication;
                    let app = NSApplication::sharedApplication(mtm);
                    app.activate();

                    log::info!("Window should now be visible and app activated");
                } else {
                    log::error!("Window is None, cannot show!");
                }
            } else {
                log::error!("⚠️  NOT on main thread! Cannot create window. This is the bug!");
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

    /// Process any pending keyboard events (call this periodically from main thread)
    pub fn process_key_events(&mut self) {
        // Process all pending key events
        while let Ok(key) = self.key_event_receiver.try_recv() {
            log::info!("📋 Processing key event: {}", key);

            match key.as_str() {
                "up" => self.move_selection_up(),
                "down" => self.move_selection_down(),
                "enter" => self.paste_and_close(),
                "escape" => self.hide(),
                _ => log::warn!("Unknown key event: {}", key),
            }
        }
    }
}
