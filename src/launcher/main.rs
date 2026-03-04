use objc2::{msg_send, msg_send_id, rc::Id, class};
use objc2_foundation::{NSString, NSURL};
use objc2_app_kit::NSWorkspace;

fn main() {
    if let Ok(mut path) = std::env::current_exe() {
        path.pop(); // .../MacOS
        path.pop(); // .../Contents
        path.pop(); // .../ClipVaultLauncher.app
        path.pop(); // .../LoginItems
        path.pop(); // .../Library
        path.pop(); // .../Contents
        path.pop(); // .../ClipVault.app
        
        let path_str = path.to_string_lossy();
        let ns_string = NSString::from_str(&path_str);
        
        // Use msg_send_id! for methods returning an object
        let url: Id<NSURL> = unsafe { msg_send_id![class!(NSURL), fileURLWithPath:&*ns_string] };
        
        // Use msg_send_id! for NSWorkspace sharedWorkspace
        let workspace: Id<NSWorkspace> = unsafe { msg_send_id![class!(NSWorkspace), sharedWorkspace] };
        
        // Use msg_send! for boolean return, NSWorkspaceLaunchDefault is 0
        unsafe {
            let _: bool = msg_send![&workspace, launchApplicationAtURL:&*url options:0usize configuration:std::ptr::null_mut::<std::ffi::c_void>() error:std::ptr::null_mut::<std::ffi::c_void>()];
        }
    }
}
