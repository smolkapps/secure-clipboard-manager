// SQLite database management for clipboard history
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use log::info;

const SCHEMA_VERSION: i32 = 1;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open database at the given path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        let mut db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&mut self) -> Result<()> {
        // Enable foreign keys
        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create clipboard_items table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                data_type TEXT NOT NULL,
                is_sensitive BOOLEAN DEFAULT 0,
                is_encrypted BOOLEAN DEFAULT 0,
                preview_text TEXT,
                data_size INTEGER,
                data_blob_id INTEGER,
                metadata TEXT,
                FOREIGN KEY(data_blob_id) REFERENCES clipboard_data(id)
            )",
            [],
        )?;

        // Create indexes for efficient queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp
             ON clipboard_items(timestamp DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_type
             ON clipboard_items(data_type)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_preview_search
             ON clipboard_items(preview_text)",
            [],
        )?;

        // Create clipboard_data table (blob storage)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                data BLOB NOT NULL
            )",
            [],
        )?;

        // Create config table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Set schema version
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params!["schema_version", SCHEMA_VERSION.to_string()],
        )?;

        // Set default config values
        self.set_config_default("retention_days", "7")?;
        self.set_config_default("polling_interval_ms", "500")?;

        info!("✓ Database schema initialized (version {})", SCHEMA_VERSION);

        Ok(())
    }

    /// Set config value if it doesn't exist
    fn set_config_default(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Store clipboard data blob
    pub fn store_blob(&self, data: &[u8]) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_data (data) VALUES (?1)",
            params![data],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Retrieve clipboard data blob
    pub fn get_blob(&self, blob_id: i64) -> Result<Vec<u8>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM clipboard_data WHERE id = ?1"
        )?;

        let data = stmt.query_row(params![blob_id], |row| {
            row.get(0)
        })?;

        Ok(data)
    }

    /// Store clipboard item metadata
    pub fn store_item(
        &self,
        timestamp: i64,
        data_type: &str,
        is_sensitive: bool,
        is_encrypted: bool,
        preview_text: Option<&str>,
        data_size: i64,
        data_blob_id: i64,
        metadata: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_items
             (timestamp, data_type, is_sensitive, is_encrypted, preview_text, data_size, data_blob_id, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                timestamp,
                data_type,
                is_sensitive,
                is_encrypted,
                preview_text,
                data_size,
                data_blob_id,
                metadata,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent clipboard items (limit by count)
    pub fn get_recent_items(&self, limit: i32) -> Result<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, data_type, is_sensitive, is_encrypted,
                    preview_text, data_size, data_blob_id, metadata
             FROM clipboard_items
             ORDER BY timestamp DESC
             LIMIT ?1"
        )?;

        let items = stmt.query_map(params![limit], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                data_type: row.get(2)?,
                is_sensitive: row.get(3)?,
                is_encrypted: row.get(4)?,
                preview_text: row.get(5)?,
                data_size: row.get(6)?,
                data_blob_id: row.get(7)?,
                metadata: row.get(8)?,
            })
        })?;

        items.collect()
    }

    /// Clean up items older than retention period (in days)
    pub fn cleanup_old_items(&self, retention_days: i64) -> Result<usize> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (retention_days * 86400);

        // Get blob IDs to delete
        let mut stmt = self.conn.prepare(
            "SELECT data_blob_id FROM clipboard_items WHERE timestamp < ?1"
        )?;

        let blob_ids: Vec<i64> = stmt.query_map(params![cutoff_timestamp], |row| {
            row.get(0)
        })?.collect::<Result<Vec<_>>>()?;

        // Delete clipboard items
        let deleted_items = self.conn.execute(
            "DELETE FROM clipboard_items WHERE timestamp < ?1",
            params![cutoff_timestamp],
        )?;

        // Delete orphaned blobs
        for blob_id in blob_ids {
            self.conn.execute(
                "DELETE FROM clipboard_data WHERE id = ?1",
                params![blob_id],
            )?;
        }

        if deleted_items > 0 {
            info!("🗑️  Cleaned up {} old clipboard items", deleted_items);
        }

        Ok(deleted_items)
    }

    /// Get total item count
    pub fn count_items(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM clipboard_items",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get database size in bytes
    pub fn get_db_size(&self) -> Result<i64> {
        let page_count: i64 = self.conn.query_row(
            "PRAGMA page_count",
            [],
            |row| row.get(0),
        )?;

        let page_size: i64 = self.conn.query_row(
            "PRAGMA page_size",
            [],
            |row| row.get(0),
        )?;

        Ok(page_count * page_size)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub timestamp: i64,
    pub data_type: String,
    pub is_sensitive: bool,
    pub is_encrypted: bool,
    pub preview_text: Option<String>,
    pub data_size: i64,
    pub data_blob_id: i64,
    pub metadata: Option<String>,
}
