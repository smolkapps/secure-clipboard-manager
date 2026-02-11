// NSPasteboard monitoring implementation using objc2
use log::{debug, info};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::NSString;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

/// Represents a clipboard change event
#[derive(Debug, Clone)]
pub struct ClipboardChange {
    pub change_count: i64,
    pub types: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// ClipboardMonitor polls NSPasteboard for changes
pub struct ClipboardMonitor {
    last_change_count: i64,
    poll_interval_ms: u64,
}

impl ClipboardMonitor {
    /// Create a new clipboard monitor with default 500ms polling
    pub fn new() -> Self {
        Self::with_poll_interval(500)
    }

    /// Create a monitor with custom polling interval in milliseconds
    pub fn with_poll_interval(interval_ms: u64) -> Self {
        let last_change_count = unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            pasteboard.changeCount() as i64
        };

        info!("Initialized clipboard monitor with {}ms polling", interval_ms);
        debug!("Initial change count: {}", last_change_count);

        Self {
            last_change_count,
            poll_interval_ms: interval_ms,
        }
    }

    /// Start monitoring clipboard changes, sending events to the provided channel
    pub async fn start(
        &mut self,
        tx: mpsc::UnboundedSender<ClipboardChange>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting clipboard monitor...");
        let mut tick = interval(Duration::from_millis(self.poll_interval_ms));

        loop {
            tick.tick().await;

            let (current_count, types) = unsafe {
                let pasteboard = NSPasteboard::generalPasteboard();
                let count = pasteboard.changeCount() as i64;
                let types = Self::get_available_types(&pasteboard);
                (count, types)
            };

            if current_count != self.last_change_count {
                debug!(
                    "Clipboard changed: {} -> {}",
                    self.last_change_count, current_count
                );

                let change = ClipboardChange {
                    change_count: current_count,
                    types: types.clone(),
                    timestamp: chrono::Utc::now(),
                };

                info!("Clipboard change detected: {:?}", types);

                // Send change notification (non-fatal: log error but continue monitoring)
                if let Err(e) = tx.send(change) {
                    log::error!("Failed to send clipboard change (channel error, continuing): {}", e);
                    // DO NOT break - keep monitoring even if channel fails temporarily
                }

                self.last_change_count = current_count;
            }
        }

        Ok(())
    }

    /// Get list of available UTI types on the pasteboard
    fn get_available_types(pasteboard: &NSPasteboard) -> Vec<String> {
        unsafe {
            if let Some(types) = pasteboard.types() {
                let mut result = Vec::new();
                for i in 0..types.count() {
                    let type_obj = types.objectAtIndex(i);
                    // NSString implements Display, so we can just use that
                    result.push(type_obj.to_string());
                }
                result
            } else {
                Vec::new()
            }
        }
    }

    /// Extract string content from clipboard
    pub fn get_string() -> Option<String> {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let utf8_type = NSString::from_str("public.utf8-plain-text");
            pasteboard
                .stringForType(&utf8_type)
                .map(|ns_str| ns_str.to_string())
                .or_else(|| {
                    // Fallback to NSStringPboardType
                    let string_type = NSString::from_str("NSStringPboardType");
                    pasteboard
                        .stringForType(&string_type)
                        .map(|ns_str| ns_str.to_string())
                })
        }
    }

    /// Extract image data from clipboard (TIFF, PNG, JPEG)
    pub fn get_image() -> Option<(Vec<u8>, String)> {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();

            // Try TIFF first (macOS default screenshot format)
            let tiff_type = NSString::from_str("public.tiff");
            if let Some(data) = pasteboard.dataForType(&tiff_type) {
                let bytes = data.bytes();
                return Some((bytes.to_vec(), "public.tiff".to_string()));
            }

            // Try PNG
            let png_type = NSString::from_str("public.png");
            if let Some(data) = pasteboard.dataForType(&png_type) {
                let bytes = data.bytes();
                return Some((bytes.to_vec(), "public.png".to_string()));
            }

            // Try JPEG
            let jpeg_type = NSString::from_str("public.jpeg");
            if let Some(data) = pasteboard.dataForType(&jpeg_type) {
                let bytes = data.bytes();
                return Some((bytes.to_vec(), "public.jpeg".to_string()));
            }

            None
        }
    }

    /// Get current change count (useful for testing)
    pub fn change_count() -> i64 {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            pasteboard.changeCount() as i64
        }
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ClipboardMonitor::new();
        assert_eq!(monitor.poll_interval_ms, 500);
    }

    #[test]
    fn test_custom_interval() {
        let monitor = ClipboardMonitor::with_poll_interval(100);
        assert_eq!(monitor.poll_interval_ms, 100);
    }

    #[test]
    fn test_change_count() {
        let count = ClipboardMonitor::change_count();
        assert!(count >= 0);
    }
}
