// License management for ClipVault Pro
// Validates against Lemon Squeezy License API (client-side, no API key needed)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const VALIDATE_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/validate";
const ACTIVATE_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/activate";
const DEACTIVATE_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/deactivate";
const EXPECTED_PRODUCT: &str = "ClipVault Pro";
const REVALIDATE_SECS: i64 = 7 * 24 * 3600;
const GRACE_PERIOD_SECS: i64 = 30 * 24 * 3600;

/// Maximum clipboard history items for free tier
pub const FREE_HISTORY_LIMIT: usize = 25;

/// Checkout URL for ClipVault Pro
pub const CHECKOUT_URL: &str =
    "https://smolkapps.lemonsqueezy.com/checkout/buy/76a9fa07-c442-4f26-a9d6-3642f7486dbc";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_key: String,
    pub instance_id: String,
    pub status: String,
    pub validated_at: i64,
    pub customer_email: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    valid: Option<bool>,
    activated: Option<bool>,
    error: Option<serde_json::Value>,
    license_key: Option<ApiLicenseKey>,
    instance: Option<ApiInstance>,
    meta: Option<ApiMeta>,
}

#[derive(Debug, Deserialize)]
struct ApiLicenseKey {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiInstance {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ApiMeta {
    product_name: Option<String>,
    customer_email: Option<String>,
}

/// Validate that a license key contains only safe characters.
fn validate_key_format(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 256 {
        return Err("Invalid license key length".to_string());
    }
    if !key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("License key contains invalid characters".to_string());
    }
    Ok(())
}

pub struct LicenseManager {
    data_dir: PathBuf,
    pro_flag: Arc<AtomicBool>,
}

impl LicenseManager {
    pub fn new(data_dir: &Path, pro_flag: Arc<AtomicBool>) -> Self {
        LicenseManager {
            data_dir: data_dir.to_path_buf(),
            pro_flag,
        }
    }

    pub fn is_pro(&self) -> bool {
        self.pro_flag.load(Ordering::Relaxed)
    }

    fn license_path(&self) -> PathBuf {
        self.data_dir.join("license.json")
    }

    pub fn load(&self) -> Option<LicenseInfo> {
        let contents = std::fs::read_to_string(self.license_path()).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn save(&self, info: &LicenseInfo) -> Result<(), String> {
        let json = serde_json::to_string_pretty(info)
            .map_err(|e| format!("Serialize error: {}", e))?;
        let path = self.license_path();
        std::fs::write(&path, json)
            .map_err(|e| format!("Write error: {}", e))?;

        // Restrict to owner read/write only (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }
        Ok(())
    }

    fn remove(&self) {
        let _ = std::fs::remove_file(self.license_path());
    }

    /// Check license status on startup. Returns true if Pro is active.
    pub fn check_on_startup(&self) -> bool {
        let Some(info) = self.load() else {
            self.pro_flag.store(false, Ordering::Relaxed);
            return false;
        };

        let now = chrono::Utc::now().timestamp();
        let age = now - info.validated_at;

        if age <= REVALIDATE_SECS {
            self.pro_flag.store(true, Ordering::Relaxed);
            return true;
        }

        // Try online revalidation
        match self.validate_online(&info.license_key, Some(&info.instance_id)) {
            Ok(true) => {
                let mut updated = info;
                updated.validated_at = now;
                let _ = self.save(&updated);
                self.pro_flag.store(true, Ordering::Relaxed);
                true
            }
            Ok(false) => {
                log::warn!("License no longer valid — reverting to free tier");
                self.remove();
                self.pro_flag.store(false, Ordering::Relaxed);
                false
            }
            Err(e) => {
                log::warn!(
                    "Cannot revalidate license ({}). Offline grace: {} days left",
                    e,
                    (GRACE_PERIOD_SECS - age).max(0) / 86400
                );
                if age < GRACE_PERIOD_SECS {
                    self.pro_flag.store(true, Ordering::Relaxed);
                    true
                } else {
                    self.pro_flag.store(false, Ordering::Relaxed);
                    false
                }
            }
        }
    }

    /// Activate a license key. Calls Lemon Squeezy API.
    pub fn activate(&self, key: &str) -> Result<LicenseInfo, String> {
        validate_key_format(key)?;
        let hostname = get_hostname();
        let resp = curl_post(ACTIVATE_URL, &[
            ("license_key", key),
            ("instance_name", &hostname),
        ])?;

        if let Some(err) = &resp.error {
            return Err(format_api_error(err));
        }
        if !resp.activated.unwrap_or(false) {
            return Err("Activation not confirmed by server".to_string());
        }

        // Verify this is for ClipVault Pro
        if let Some(meta) = &resp.meta {
            if let Some(name) = &meta.product_name {
                if name != EXPECTED_PRODUCT {
                    return Err(format!("Wrong product: '{}'", name));
                }
            }
        }

        let instance_id = resp
            .instance
            .map(|i| i.id)
            .ok_or("No instance ID in response")?;

        let info = LicenseInfo {
            license_key: key.to_string(),
            instance_id,
            status: resp
                .license_key
                .and_then(|lk| lk.status)
                .unwrap_or_else(|| "active".to_string()),
            validated_at: chrono::Utc::now().timestamp(),
            customer_email: resp.meta.as_ref().and_then(|m| m.customer_email.clone()),
            product_name: resp.meta.as_ref().and_then(|m| m.product_name.clone()),
        };

        self.save(&info)?;
        self.pro_flag.store(true, Ordering::Relaxed);
        log::info!("License activated successfully");
        Ok(info)
    }

    /// Deactivate the current license on this machine.
    pub fn deactivate(&self) -> Result<(), String> {
        if let Some(info) = self.load() {
            let resp = curl_post(DEACTIVATE_URL, &[
                ("license_key", &info.license_key),
                ("instance_id", &info.instance_id),
            ])?;

            if let Some(err) = &resp.error {
                return Err(format_api_error(err));
            }
        }

        self.remove();
        self.pro_flag.store(false, Ordering::Relaxed);
        log::info!("License deactivated");
        Ok(())
    }

    fn validate_online(&self, key: &str, instance_id: Option<&str>) -> Result<bool, String> {
        let mut fields: Vec<(&str, &str)> = vec![("license_key", key)];
        if let Some(iid) = instance_id {
            fields.push(("instance_id", iid));
        }

        let resp = curl_post(VALIDATE_URL, &fields)?;

        if let Some(meta) = &resp.meta {
            if let Some(name) = &meta.product_name {
                if name != EXPECTED_PRODUCT {
                    return Ok(false);
                }
            }
        }

        Ok(resp.valid.unwrap_or(false))
    }
}

/// POST via curl (ships with macOS — zero extra dependencies).
fn curl_post(url: &str, fields: &[(&str, &str)]) -> Result<ApiResponse, String> {
    let mut args: Vec<String> = vec![
        "-s".to_string(),
        "--max-time".to_string(),
        "10".to_string(),
        "-X".to_string(),
        "POST".to_string(),
        "-H".to_string(),
        "Accept: application/json".to_string(),
    ];

    for (key, value) in fields {
        args.push("--data-urlencode".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(url.to_string());

    let output = std::process::Command::new("curl")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl error: {}", stderr.trim()));
    }

    let body = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&body).map_err(|e| format!("Invalid response: {}", e))
}

/// Get machine hostname via libc (no extra dependency).
fn get_hostname() -> String {
    let mut buf = [0u8; 256];
    let result =
        unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if result == 0 {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        "ClipVault-Mac".to_string()
    }
}

fn format_api_error(err: &serde_json::Value) -> String {
    match err {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
