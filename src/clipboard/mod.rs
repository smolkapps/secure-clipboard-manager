// Clipboard module - handles NSPasteboard monitoring and data extraction
pub mod monitor;
pub mod processor;
pub mod history;

pub use monitor::{ClipboardMonitor, ClipboardChange};
