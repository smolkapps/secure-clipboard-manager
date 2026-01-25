// Clipboard Manager - macOS Native Clipboard History Manager
// Phase 1: Core Clipboard Monitoring

mod clipboard;

use clipboard::ClipboardMonitor;
use log::{error, info};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Clipboard Manager - Phase 1: Core Monitoring");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✓ Clipboard monitor initialized (polling every 500ms)");
    info!("   Monitoring NSPasteboard for changes...");
    info!("");

    // Create channel for clipboard change notifications
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Create and start clipboard monitor
    let mut monitor = ClipboardMonitor::new();

    // Spawn monitor task
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = monitor.start(tx).await {
            error!("Clipboard monitor error: {}", e);
        }
    });

    // Process clipboard changes
    while let Some(change) = rx.recv().await {
        info!("📋 Clipboard changed (count: {})", change.change_count);
        info!("   Types: {:?}", change.types);

        // Get string content if available
        if let Some(text) = ClipboardMonitor::get_string() {
            let preview = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text
            };
            info!("   Content: {}", preview);
        }

        info!("");
    }

    monitor_handle.await.ok();
}
