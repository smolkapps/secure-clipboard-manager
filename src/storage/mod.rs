// Storage module for clipboard data persistence
pub mod database;
pub mod processor;
pub mod encryption;
pub mod search;
pub mod config;
pub mod license;

pub use database::{Database, ClipboardItem};
pub use processor::DataProcessor;
pub use encryption::Encryptor;
pub use config::AppConfig;
pub use license::LicenseManager;
