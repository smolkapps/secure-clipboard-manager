// Popup window for clipboard history with native Cocoa UI
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell;
use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id};
use objc2::ClassType;
use objc2::DeclaredClass;
use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType, NSTextView, NSScrollView, NSApplication, NSApplicationActivationPolicy, NSEvent, NSScreen, NSFont, NSColor};
use objc2_foundation::{NSString, NSRect, NSPoint, NSSize, MainThreadMarker, NSMutableAttributedString, NSRange, NSData};
use objc2::msg_send;
use crate::storage::{Database, Encryptor, ClipboardItem};
use objc2_app_kit::NSPasteboard;

// Global reference to the popup so ObjC key handler can access it
pub(crate) static POPUP_FOR_KEYS: OnceLock<Arc<Mutex<PopupWindow>>> = OnceLock::new();

// Custom NSTextView subclass that intercepts key events for navigation
declare_class!(
    struct KeyHandlingTextView;

    unsafe impl ClassType for KeyHandlingTextView {
        type Super = NSTextView;
        type Mutability = objc2::mutability::MainThreadOnly;
        const NAME: &'static str = "ClipVaultKeyHandlingTextView";
    }

    impl DeclaredClass for KeyHandlingTextView {
        type Ivars = ();
    }

    unsafe impl KeyHandlingTextView {
        #[method(keyDown:)]
        fn key_down(&self, event: &NSEvent) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let key_code = unsafe { event.keyCode() };
                // Check by keyCode first: arrows, return, escape
                // Arrow down = 125, Arrow up = 126, Return = 36, Escape = 53
                match key_code {
                    125 => {
                        // Down arrow
                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                            if let Ok(popup) = popup.lock() {
                                popup.move_selection_down();
                            }
                        }
                    }
                    126 => {
                        // Up arrow
                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                            if let Ok(popup) = popup.lock() {
                                popup.move_selection_up();
                            }
                        }
                    }
                    36 => {
                        // Return/Enter - paste and close
                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                            if let Ok(mut popup) = popup.lock() {
                                popup.paste_and_close();
                            }
                        }
                    }
                    53 => {
                        // Escape - hide window
                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                            if let Ok(mut popup) = popup.lock() {
                                popup.hide();
                            }
                        }
                    }
                    _ => {
                        // Check for modifier keys (Cmd+C etc.) - forward to super
                        let has_cmd = unsafe {
                            event.modifierFlags().contains(
                                objc2_app_kit::NSEventModifierFlags::NSEventModifierFlagCommand
                            )
                        };
                        if has_cmd {
                            // Forward Cmd+key combos (like Cmd+C) to NSTextView
                            unsafe {
                                let _: () = objc2::msg_send![super(self), keyDown: event];
                            }
                            return;
                        }

                        // Check character keys (j/k for vim-style navigation)
                        let handled = unsafe {
                            if let Some(chars) = event.charactersIgnoringModifiers() {
                                let s = chars.to_string();
                                match s.as_str() {
                                    "j" => {
                                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                                            if let Ok(popup) = popup.lock() {
                                                popup.move_selection_down();
                                            }
                                        }
                                        true
                                    }
                                    "k" => {
                                        if let Some(popup) = POPUP_FOR_KEYS.get() {
                                            if let Ok(popup) = popup.lock() {
                                                popup.move_selection_up();
                                            }
                                        }
                                        true
                                    }
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        };
                        if !handled {
                            // Swallow unhandled keys to prevent beeping
                        }
                    }
                }
            }));
        }
    }
);

impl KeyHandlingTextView {
    fn new_with_frame(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), initWithFrame: frame] }
    }
}

pub struct PopupWindow {
    db: Arc<Mutex<Database>>,
    encryptor: Arc<Mutex<Encryptor>>,
    window: RefCell<Option<Retained<NSWindow>>>,
    text_view: RefCell<Option<Retained<NSTextView>>>,
    items: RefCell<Vec<ClipboardItem>>,
    selected_index: RefCell<usize>,
    visible: bool,
    auto_refresh_active: Arc<AtomicBool>,
}

// SAFETY: PopupWindow contains NSWindow which is !Send, but we only access it
// from the main thread (via MainThreadMarker checks in show/hide methods).
// The toggle() method only flips a boolean and calls show/hide which are safe.
unsafe impl Send for PopupWindow {}

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
            auto_refresh_active: Arc::new(AtomicBool::new(false)),
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
        window.setReleasedWhenClosed(false); // Prevent dealloc when red X is clicked
        // Do NOT set setHidesOnDeactivate(true) - in a menu bar app (accessory
        // activation policy), the app is never truly "active", so the window
        // would immediately hide itself after being shown.

        // Allow window to receive keyboard events
        window.setAcceptsMouseMovedEvents(true);

        // Create scroll view sized to the window content area
        let scroll_view = NSScrollView::new(mtm);
        scroll_view.setHasVerticalScroller(true);
        scroll_view.setFrame(content_rect);
        scroll_view.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::NSViewWidthSizable
            | objc2_app_kit::NSAutoresizingMaskOptions::NSViewHeightSizable,
        );

        // Create custom text view (with key handling) sized to scroll view content
        let content_size = scroll_view.contentSize();
        let text_frame = NSRect::new(NSPoint::new(0.0, 0.0), content_size);
        let text_view = KeyHandlingTextView::new_with_frame(mtm, text_frame);
        text_view.setEditable(false);
        text_view.setSelectable(true);
        text_view.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::NSViewWidthSizable,
        );

        // Configure text view for readability
        if let Some(container) = text_view.textContainer() {
            container.setWidthTracksTextView(true);
        }
        text_view.setMinSize(NSSize::new(0.0, content_size.height));
        text_view.setMaxSize(NSSize::new(f64::MAX, f64::MAX));
        text_view.setVerticallyResizable(true);
        text_view.setHorizontallyResizable(false);

        // Set font size
        let font = objc2_app_kit::NSFont::monospacedSystemFontOfSize_weight(13.0, 0.0);
        text_view.setFont(Some(&font));

        scroll_view.setDocumentView(Some(&text_view));

        window.setContentView(Some(&scroll_view));

        // Store text view for later updates (cast subclass to NSTextView)
        let text_view_as_super: Retained<NSTextView> = Retained::into_super(text_view);
        *self.text_view.borrow_mut() = Some(text_view_as_super);

        log::info!("✓ Window created with keyboard navigation support");

        window
    }

    fn load_items(&self, reset_selection: bool) {
        if let Ok(db) = self.db.lock() {
            match db.get_recent_items(20) {
                Ok(items) => {
                    if reset_selection {
                        *self.selected_index.borrow_mut() = 0;
                    } else {
                        let mut idx = self.selected_index.borrow_mut();
                        if *idx >= items.len() {
                            *idx = if items.is_empty() { 0 } else { items.len() - 1 };
                        }
                    }
                    *self.items.borrow_mut() = items;
                }
                Err(e) => log::error!("Failed to load items: {}", e),
            }
        }
    }

    fn refresh_display(&self) {
        let items = self.items.borrow();
        let selected_idx = *self.selected_index.borrow();

        let text_view = self.text_view.borrow();
        let Some(text_view) = text_view.as_ref() else { return };

        unsafe {
            let mut result = NSMutableAttributedString::new();

            let mono_font = NSFont::monospacedSystemFontOfSize_weight(13.0, 0.0);
            let bold_font = NSFont::monospacedSystemFontOfSize_weight(14.0, 0.3);
            let small_font = NSFont::monospacedSystemFontOfSize_weight(11.0, 0.0);

            let fg_key = NSString::from_str("NSColor");
            let bg_key = NSString::from_str("NSBackgroundColor");
            let font_key = NSString::from_str("NSFont");

            // Header
            Self::append_styled_line(
                &mut result, "  Clipboard History\n",
                &bold_font, &NSColor::labelColor(), None, &font_key, &fg_key, &bg_key,
            );
            Self::append_styled_line(
                &mut result, "  ↑↓/j/k navigate • Enter paste • Esc close\n\n",
                &small_font, &NSColor::secondaryLabelColor(), None, &font_key, &fg_key, &bg_key,
            );

            if items.is_empty() {
                Self::append_styled_line(
                    &mut result, "  No clipboard history yet.\n  Copy something to get started!\n",
                    &mono_font, &NSColor::secondaryLabelColor(), None, &font_key, &fg_key, &bg_key,
                );
            } else {
                for (i, item) in items.iter().enumerate() {
                    let is_selected = i == selected_idx;
                    let icon = match item.data_type.as_str() {
                        "image" => "🖼️",
                        "url" => "🔗",
                        _ => "📝",
                    };
                    let lock = if item.is_sensitive { " 🔒" } else { "" };

                    let preview = item.preview_text.as_deref().unwrap_or("[No preview]");
                    let preview_short = if preview.chars().count() > 70 {
                        format!("{}...", preview.chars().take(70).collect::<String>())
                    } else {
                        preview.to_string()
                    };

                    let count_badge = if item.copy_count > 1 {
                        format!(" (×{})", item.copy_count)
                    } else {
                        String::new()
                    };

                    let marker = if is_selected { "▶" } else { " " };
                    let line = format!(" {} {} {}{}{}\n", marker, icon, preview_short, count_badge, lock);

                    let bg_color = if is_selected {
                        Some(NSColor::selectedContentBackgroundColor())
                    } else if i % 2 == 1 {
                        Some(NSColor::controlBackgroundColor())
                    } else {
                        None
                    };

                    let fg_color = if is_selected {
                        NSColor::selectedMenuItemTextColor()
                    } else {
                        NSColor::labelColor()
                    };

                    Self::append_styled_line(
                        &mut result, &line,
                        &mono_font, &fg_color, bg_color.as_deref(), &font_key, &fg_key, &bg_key,
                    );
                }
            }

            // Preview pane: show full text of selected item
            if let Some(selected_item) = items.get(selected_idx) {
                Self::append_styled_line(
                    &mut result, "\n  ─────────────────────────────────────────\n",
                    &small_font, &NSColor::separatorColor(), None, &font_key, &fg_key, &bg_key,
                );

                let type_label = match selected_item.data_type.as_str() {
                    "image" => "Image",
                    "url" => "URL",
                    _ => "Text",
                };
                let count_info = if selected_item.copy_count > 1 {
                    format!(" • copied ×{}", selected_item.copy_count)
                } else {
                    String::new()
                };
                let header = format!("  {}{}\n\n", type_label, count_info);
                Self::append_styled_line(
                    &mut result, &header,
                    &bold_font, &NSColor::secondaryLabelColor(), None, &font_key, &fg_key, &bg_key,
                );

                let full_text = selected_item.preview_text.as_deref().unwrap_or("[No preview]");
                // Wrap long text at ~80 chars for readability
                let wrapped = Self::word_wrap(full_text, 80);
                let padded = wrapped.lines()
                    .map(|line| format!("  {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                Self::append_styled_line(
                    &mut result, &format!("{}\n", padded),
                    &mono_font, &NSColor::labelColor(), None, &font_key, &fg_key, &bg_key,
                );
            }

            // Replace text storage contents
            if let Some(mut storage) = text_view.textStorage() {
                let full_range = NSRange::new(0, storage.length());
                storage.replaceCharactersInRange_withAttributedString(full_range, &result);
            }
        }
    }

    unsafe fn append_styled_line(
        result: &mut NSMutableAttributedString,
        text: &str,
        font: &NSFont,
        fg_color: &NSColor,
        bg_color: Option<&NSColor>,
        font_key: &NSString,
        fg_key: &NSString,
        bg_key: &NSString,
    ) {
        let ns_str = NSString::from_str(text);
        let line_attr = NSMutableAttributedString::initWithString(
            objc2_foundation::NSMutableAttributedString::alloc(),
            &ns_str,
        );
        let range = NSRange::new(0, ns_str.length());

        // Set attributes using msg_send! since addAttribute:value:range: takes id values
        let _: () = msg_send![&line_attr, addAttribute: font_key, value: font, range: range];
        let _: () = msg_send![&line_attr, addAttribute: fg_key, value: fg_color, range: range];

        if let Some(bg) = bg_color {
            let _: () = msg_send![&line_attr, addAttribute: bg_key, value: bg, range: range];
        }

        result.appendAttributedString(&line_attr);
    }

    fn word_wrap(text: &str, width: usize) -> String {
        let mut result = String::new();
        for line in text.lines() {
            if line.chars().count() <= width {
                result.push_str(line);
                result.push('\n');
            } else {
                let mut col = 0;
                for word in line.split_whitespace() {
                    let wlen = word.chars().count();
                    if col > 0 && col + 1 + wlen > width {
                        result.push('\n');
                        col = 0;
                    }
                    if col > 0 {
                        result.push(' ');
                        col += 1;
                    }
                    result.push_str(word);
                    col += wlen;
                }
                result.push('\n');
            }
        }
        result
    }

    pub fn toggle(&mut self) {
        // Sync visible state with actual window visibility
        // (handles case where user closed window via red X button)
        if let Some(window) = self.window.borrow().as_ref() {
            self.visible = window.isVisible();
        }

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
                self.load_items(true);
                self.refresh_display();

                // Start auto-refresh thread (refreshes every 1s while popup is open)
                if !self.auto_refresh_active.load(Ordering::Relaxed) {
                    self.auto_refresh_active.store(true, Ordering::Relaxed);
                    let active = Arc::clone(&self.auto_refresh_active);
                    std::thread::spawn(move || {
                        while active.load(Ordering::Relaxed) {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            if !active.load(Ordering::Relaxed) { break; }
                            let active_inner = Arc::clone(&active);
                            dispatch::Queue::main().exec_async(move || {
                                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    if !active_inner.load(Ordering::Relaxed) { return; }
                                    if let Some(popup) = POPUP_FOR_KEYS.get() {
                                        if let Ok(popup) = popup.lock() {
                                            popup.load_items(false);
                                            popup.refresh_display();
                                        }
                                    }
                                }));
                            });
                        }
                    });
                }

                // Show window
                if let Some(window) = self.window.borrow().as_ref() {
                    log::info!("Calling makeKeyAndOrderFront on window");

                    let app = NSApplication::sharedApplication(mtm);

                    // Set activation policy to Accessory once (allows windows
                    // without a dock icon). Only set on first show to avoid
                    // conflicting with cacao's activation policy management.
                    use std::sync::atomic::{AtomicBool, Ordering};
                    static POLICY_SET: AtomicBool = AtomicBool::new(false);
                    if !POLICY_SET.load(Ordering::Relaxed) {
                        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                        POLICY_SET.store(true, Ordering::Relaxed);
                    }

                    // Position window near the mouse cursor
                    let mouse_loc = NSEvent::mouseLocation();
                    let win_size = window.frame().size;
                    let cursor_offset = 10.0;
                    let mut top_left_x = mouse_loc.x + cursor_offset;
                    let mut top_left_y = mouse_loc.y + cursor_offset;
                    if let Some(screen) = NSScreen::mainScreen(mtm) {
                        let sf = screen.visibleFrame();
                        let smin_x = sf.origin.x;
                        let smin_y = sf.origin.y;
                        let smax_x = smin_x + sf.size.width;
                        let smax_y = smin_y + sf.size.height;
                        if top_left_x + win_size.width > smax_x {
                            top_left_x = mouse_loc.x - win_size.width - cursor_offset;
                        }
                        if top_left_y > smax_y {
                            top_left_y = smax_y;
                        }
                        if top_left_y - win_size.height < smin_y {
                            top_left_y = smin_y + win_size.height;
                        }
                        if top_left_x < smin_x {
                            top_left_x = smin_x;
                        }
                    }
                    window.setFrameTopLeftPoint(NSPoint::new(top_left_x, top_left_y));

                    // Make window visible and bring to front
                    window.makeKeyAndOrderFront(None);
                    window.orderFrontRegardless();

                    // Make text view first responder so it receives key events
                    if let Some(tv) = self.text_view.borrow().as_ref() {
                        window.makeFirstResponder(Some(tv));
                    }

                    // Activate the app so it comes to the foreground
                    #[allow(deprecated)]
                    app.activateIgnoringOtherApps(true);

                    log::info!("Window visible: {}, near cursor ({}, {})",
                        window.isVisible(), top_left_x, top_left_y);
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
        self.auto_refresh_active.store(false, Ordering::Relaxed);
        log::info!("Popup window hidden");

        if let Some(window) = self.window.borrow().as_ref() {
            window.orderOut(None);
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn move_selection_down(&self) {
        let items_len = self.items.borrow().len();
        if items_len > 0 {
            {
                let mut idx = self.selected_index.borrow_mut();
                *idx = (*idx + 1) % items_len;
            } // RefMut dropped here — must release before refresh_display() borrows
            self.refresh_display();
        }
    }

    pub fn move_selection_up(&self) {
        let items_len = self.items.borrow().len();
        if items_len > 0 {
            {
                let mut idx = self.selected_index.borrow_mut();
                *idx = if *idx == 0 { items_len - 1 } else { *idx - 1 };
            } // RefMut dropped here
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
                                let ns_data = NSData::with_bytes(&data);
                                let type_str = NSString::from_str("public.png");
                                pb.setData_forType(Some(&ns_data), &type_str);
                                log::info!("✓ Pasted image to clipboard");
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
