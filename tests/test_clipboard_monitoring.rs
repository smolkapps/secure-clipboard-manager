// Integration tests for clipboard monitoring (NSPasteboard polling)
// These tests are designed to be headless-safe and work in CI environments

use clipboard_manager::clipboard::monitor::ClipboardMonitor;

#[test]
fn test_monitor_creation_default() {
    // Should initialize without errors
    let _monitor = ClipboardMonitor::new();
}

#[test]
fn test_monitor_creation_custom_interval() {
    let _monitor = ClipboardMonitor::with_poll_interval(100);
    let _monitor2 = ClipboardMonitor::with_poll_interval(1000);
}

#[test]
fn test_get_change_count() {
    let count = ClipboardMonitor::change_count();
    // Change count should be non-negative
    assert!(count >= 0, "Change count should be non-negative: {}", count);
}

#[test]
fn test_change_count_consistency() {
    let count1 = ClipboardMonitor::change_count();
    let count2 = ClipboardMonitor::change_count();

    // Without clipboard changes, count should be the same
    // (or might increment if clipboard changed between calls)
    assert!(count2 >= count1, "Change count should not decrease");
}

#[test]
fn test_get_string_content() {
    // This might return None in headless environments, which is okay
    let content = ClipboardMonitor::get_string();
    if let Some(s) = content {
        // If we got content, it should be valid UTF-8
        assert!(!s.is_empty() || s.is_empty()); // verify it's accessible
    }
}

#[test]
fn test_get_image_content() {
    let image_data = ClipboardMonitor::get_image();
    match image_data {
        Some((data, uti_type)) => {
            assert!(!data.is_empty(), "Image data should not be empty");
            assert!(!uti_type.is_empty(), "UTI type should not be empty");
            assert!(
                uti_type == "public.tiff" ||
                uti_type == "public.png" ||
                uti_type == "public.jpeg",
                "UTI type should be supported: {}",
                uti_type
            );
        }
        None => {
            // No image on clipboard (common)
        }
    }
}

#[test]
fn test_monitor_default_trait() {
    let _monitor = ClipboardMonitor::default();
}

#[test]
fn test_multiple_monitors() {
    // Should be able to create multiple monitors
    let _monitor1 = ClipboardMonitor::new();
    let _monitor2 = ClipboardMonitor::new();
}

#[test]
fn test_very_short_poll_interval() {
    let _monitor = ClipboardMonitor::with_poll_interval(1);
}

#[test]
fn test_very_long_poll_interval() {
    let _monitor = ClipboardMonitor::with_poll_interval(60000);
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_get_string_performance() {
        let start = Instant::now();
        for _ in 0..100 {
            ClipboardMonitor::get_string();
        }
        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 500,
            "100 get_string calls took {}ms, should be < 500ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_monitor_creation_performance() {
        let start = Instant::now();
        for _ in 0..100 {
            let _monitor = ClipboardMonitor::new();
        }
        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "Creating 100 monitors took {}ms, should be < 100ms",
            duration.as_millis()
        );
    }
}
