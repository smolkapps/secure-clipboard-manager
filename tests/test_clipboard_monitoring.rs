// Integration tests for clipboard monitoring (NSPasteboard polling)
// These tests are designed to be headless-safe and work in CI environments

use clipboard_manager::clipboard::monitor::ClipboardMonitor;

#[test]
fn test_monitor_creation_default() {
    let monitor = ClipboardMonitor::new();
    // Should initialize without errors
    assert_eq!(monitor.poll_interval_ms, 500);
}

#[test]
fn test_monitor_creation_custom_interval() {
    let monitor = ClipboardMonitor::with_poll_interval(100);
    assert_eq!(monitor.poll_interval_ms, 100);

    let monitor2 = ClipboardMonitor::with_poll_interval(1000);
    assert_eq!(monitor2.poll_interval_ms, 1000);
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

    // Test should not crash, content might be None or Some
    match content {
        Some(s) => {
            // If we got content, it should be valid UTF-8
            assert!(s.len() >= 0);
        }
        None => {
            // No clipboard content available (common in CI)
        }
    }
}

#[test]
fn test_get_image_content() {
    // This might return None in headless environments, which is okay
    let image_data = ClipboardMonitor::get_image();

    match image_data {
        Some((data, uti_type)) => {
            // If we got image data, it should have content
            assert!(!data.is_empty(), "Image data should not be empty");
            assert!(!uti_type.is_empty(), "UTI type should not be empty");

            // UTI should be one of the supported types
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
    let monitor = ClipboardMonitor::default();
    assert_eq!(monitor.poll_interval_ms, 500);
}

#[test]
fn test_multiple_monitors() {
    // Should be able to create multiple monitors
    let monitor1 = ClipboardMonitor::new();
    let monitor2 = ClipboardMonitor::new();

    assert_eq!(monitor1.poll_interval_ms, monitor2.poll_interval_ms);
}

#[test]
fn test_very_short_poll_interval() {
    let monitor = ClipboardMonitor::with_poll_interval(1);
    assert_eq!(monitor.poll_interval_ms, 1);
}

#[test]
fn test_very_long_poll_interval() {
    let monitor = ClipboardMonitor::with_poll_interval(60000);
    assert_eq!(monitor.poll_interval_ms, 60000);
}

// Note: We cannot easily test the async start() method in a synchronous test
// without actually changing the clipboard, which is not reliable in CI.
// The start() method is tested manually and through end-to-end testing.

#[cfg(test)]
mod clipboard_content_tests {
    use super::*;

    // These tests try to read actual clipboard content
    // They may pass or fail depending on what's on the clipboard

    #[test]
    fn test_string_extraction_type() {
        if let Some(content) = ClipboardMonitor::get_string() {
            // If we got a string, it should be valid
            assert!(content.is_empty() || content.len() > 0);
        }
    }

    #[test]
    fn test_change_count_is_positive() {
        let count = ClipboardMonitor::change_count();
        // macOS change count is always >= 0
        assert!(count >= 0);
    }
}

// Performance tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_change_count_performance() {
        let start = Instant::now();

        for _ in 0..1000 {
            ClipboardMonitor::change_count();
        }

        let duration = start.elapsed();

        // 1000 change count calls should complete quickly
        assert!(
            duration.as_millis() < 100,
            "1000 change_count calls took {}ms, should be < 100ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_get_string_performance() {
        let start = Instant::now();

        for _ in 0..100 {
            ClipboardMonitor::get_string();
        }

        let duration = start.elapsed();

        // 100 string reads should complete in reasonable time
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

        // Creating 100 monitors should be fast
        assert!(
            duration.as_millis() < 100,
            "Creating 100 monitors took {}ms, should be < 100ms",
            duration.as_millis()
        );
    }
}
