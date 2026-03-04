// Launch at Login management via Apple ServiceManagement Framework
use objc2::{class, msg_send, msg_send_id, rc::Id};
use objc2_foundation::{NSString, NSProcessInfo};

#[link(name = "ServiceManagement", kind = "framework")]
extern "C" {
    fn SMLoginItemSetEnabled(identifier: *mut objc2::runtime::AnyObject, enabled: bool) -> bool;
}

// SMAppService management for macOS 13+
fn register_sm_app_service() -> Result<(), String> {
    let cls = class!(SMAppService);
    let service: Id<objc2::runtime::AnyObject> = unsafe { msg_send_id![cls, mainAppService] };
    
    let mut error: *mut std::ffi::c_void = std::ptr::null_mut();
    let success: bool = unsafe { msg_send![&service, registerAndReturnError:&mut error] };
    if success {
        Ok(())
    } else {
        Err("Failed to register SMAppService".to_string())
    }
}

fn unregister_sm_app_service() -> Result<(), String> {
    let cls = class!(SMAppService);
    let service: Id<objc2::runtime::AnyObject> = unsafe { msg_send_id![cls, mainAppService] };
    
    let mut error: *mut std::ffi::c_void = std::ptr::null_mut();
    let success: bool = unsafe { msg_send![&service, unregisterAndReturnError:&mut error] };
    if success {
        Ok(())
    } else {
        Err("Failed to unregister SMAppService".to_string())
    }
}

// Check if running on macOS 13.0 or later
fn is_macos_13_or_later() -> bool {
    let process_info = NSProcessInfo::processInfo();
    
    // We'll use alternative ways to check version to avoid NSOperatingSystemVersion trait errors
    // Use respondsToSelector approach for safety
    let selector = objc2::sel!(majorVersion);
    let responds: bool = unsafe { msg_send![&process_info, respondsToSelector:selector] };
    
    if responds {
        let version: objc2_foundation::NSOperatingSystemVersion = process_info.operatingSystemVersion();
        return version.majorVersion >= 13;
    }
    
    // Fallback if operatingSystemVersion is somehow unavailable
    let str = unsafe { process_info.operatingSystemVersionString() };
    let version_str = str.to_string();
    
    version_str.contains("Version 13") || version_str.contains("Version 14") || version_str.contains("Version 15")
}

/// Enable launch at login
pub fn enable() -> Result<(), String> {
    if is_macos_13_or_later() {
        log::info!("macOS 13+ detected, using SMAppService for Launch at Login");
        register_sm_app_service()
    } else {
        log::info!("macOS <13 detected, using SMLoginItemSetEnabled for Launch at Login");
        let identifier = NSString::from_str("com.smolkapps.clipboard-manager.Launcher");
        let success = unsafe { SMLoginItemSetEnabled(Id::as_ptr(&identifier) as *mut _, true) };
        if success {
            Ok(())
        } else {
            Err("SMLoginItemSetEnabled returned false".to_string())
        }
    }
}

/// Disable launch at login
pub fn disable() -> Result<(), String> {
    if is_macos_13_or_later() {
        log::info!("macOS 13+ detected, using SMAppService to disable Launch at Login");
        unregister_sm_app_service()
    } else {
        log::info!("macOS <13 detected, using SMLoginItemSetEnabled to disable Launch at Login");
        let identifier = NSString::from_str("com.smolkapps.clipboard-manager.Launcher");
        let success = unsafe { SMLoginItemSetEnabled(Id::as_ptr(&identifier) as *mut _, false) };
        if success {
            Ok(())
        } else {
            Err("SMLoginItemSetEnabled returned false".to_string())
        }
    }
}

/// Sync the plist state with the desired setting
pub fn sync(enabled: bool) -> Result<(), String> {
    if enabled {
        enable()
    } else {
        disable()
    }
}
