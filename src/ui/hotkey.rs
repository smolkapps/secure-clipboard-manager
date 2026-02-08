// Global hotkey registration for clipboard popup
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl HotkeyManager {
    /// Register Cmd+Shift+C as the global hotkey.
    /// Events are polled separately in main.rs via GlobalHotKeyEvent::receiver().
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Failed to create hotkey manager: {}", e))?;

        let hotkey = HotKey::new(
            Some(Modifiers::SUPER | Modifiers::SHIFT),
            Code::KeyC,
        );

        manager.register(hotkey)
            .map_err(|e| format!("Failed to register hotkey: {}", e))?;

        Ok(HotkeyManager { manager, hotkey })
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        if let Err(e) = self.manager.unregister(self.hotkey) {
            log::error!("Failed to unregister hotkey: {}", e);
        }
    }
}
