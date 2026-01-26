// Global hotkey handler for clipboard popup
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use std::sync::{Arc, Mutex};
use crate::ui::PopupWindow;

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    popup: Arc<Mutex<PopupWindow>>,
}

impl HotkeyManager {
    /// Create new hotkey manager with Cmd+Shift+C
    pub fn new(popup: Arc<Mutex<PopupWindow>>) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Failed to create hotkey manager: {}", e))?;

        // Cmd+Shift+C (like Maccy)
        let hotkey = HotKey::new(
            Some(Modifiers::SUPER | Modifiers::SHIFT),
            Code::KeyC,
        );

        log::info!("🔥 Registering global hotkey: Cmd+Shift+C");

        manager.register(hotkey)
            .map_err(|e| format!("Failed to register hotkey: {}", e))?;

        Ok(HotkeyManager {
            manager,
            hotkey,
            popup,
        })
    }

    /// Process hotkey events
    pub fn handle_events(&self) {
        use global_hotkey::GlobalHotKeyEvent;

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey.id() {
                log::info!("🔥 Hotkey pressed: Cmd+Shift+C");

                // Toggle popup window
                if let Ok(mut popup) = self.popup.lock() {
                    popup.toggle();
                }
            }
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        if let Err(e) = self.manager.unregister(self.hotkey) {
            log::error!("Failed to unregister hotkey: {}", e);
        }
    }
}
