// Application configuration stored as JSON
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub launch_at_login: bool,
    pub first_run_complete: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            launch_at_login: true, // default on
            first_run_complete: false,
        }
    }
}

impl AppConfig {
    /// Load config from disk, or return defaults if not found
    pub fn load(data_dir: &PathBuf) -> Self {
        let path = data_dir.join("config.json");
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to disk
    pub fn save(&self, data_dir: &PathBuf) -> Result<(), String> {
        let path = data_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write config: {}", e))
    }
}
