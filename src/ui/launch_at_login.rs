// Launch at Login management via macOS LaunchAgent plist
use std::path::PathBuf;

const PLIST_LABEL: &str = "com.smolkapps.clipboard-manager";

/// Get the LaunchAgent plist path
fn plist_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join("Library/LaunchAgents").join(format!("{}.plist", PLIST_LABEL)))
}

/// Get the current executable path (resolves to the actual binary)
fn exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| std::fs::canonicalize(p).ok())
}

/// Enable launch at login by creating a LaunchAgent plist
pub fn enable() -> Result<(), String> {
    let plist = plist_path().ok_or("Could not determine LaunchAgents directory")?;
    let exe = exe_path().ok_or("Could not determine executable path")?;

    // Ensure LaunchAgents directory exists
    if let Some(parent) = plist.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create LaunchAgents dir: {}", e))?;
    }

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
        label = PLIST_LABEL,
        exe = exe.display(),
    );

    std::fs::write(&plist, plist_content)
        .map_err(|e| format!("Failed to write LaunchAgent plist: {}", e))?;

    log::info!("Launch at Login enabled: {}", plist.display());
    Ok(())
}

/// Disable launch at login by removing the LaunchAgent plist
pub fn disable() -> Result<(), String> {
    let plist = plist_path().ok_or("Could not determine LaunchAgents directory")?;

    if plist.exists() {
        std::fs::remove_file(&plist)
            .map_err(|e| format!("Failed to remove LaunchAgent plist: {}", e))?;
        log::info!("Launch at Login disabled (plist removed)");
    }

    Ok(())
}

/// Sync the plist state with the desired setting
pub fn sync(enabled: bool) -> Result<(), String> {
    if enabled {
        enable()
    } else {
        disable()
    }
}
