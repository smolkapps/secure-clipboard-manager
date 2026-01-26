// UI module - menu bar and popup interfaces
pub mod menubar;
pub mod popup;
pub mod statusbar;
pub mod actions;
pub mod hotkey;

pub use menubar::MenuBarApp;
pub use popup::PopupWindow;
pub use statusbar::StatusBarController;
pub use actions::MenuActions;
pub use hotkey::HotkeyManager;
