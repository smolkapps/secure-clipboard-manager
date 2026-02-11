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
        // Enable WAL mode for concurrent reads/writes (returns a row, so use query_row)
        let _: String = self.conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;

        // Set busy timeout to 5 seconds (retry on SQLITE_BUSY instead of failing immediately)
        let _: i64 = self.conn.query_row("PRAGMA busy_timeout=5000", [], |row| row.get(0))?;

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

        // Create deleted_items table (soft-delete trash, mirrors clipboard_items)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS deleted_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                deleted_at INTEGER NOT NULL,
                data_type TEXT NOT NULL,
                is_sensitive BOOLEAN DEFAULT 0,
                is_encrypted BOOLEAN DEFAULT 0,
                preview_text TEXT,
                data_size INTEGER,
                deleted_blob_id INTEGER,
                metadata TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_deleted_at
             ON deleted_items(deleted_at)",
            [],
        )?;

        // Create deleted_data table (blob storage for soft-deleted items)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS deleted_data (
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

        // Migration: add copy_count column (ignore error if column already exists)
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN copy_count INTEGER DEFAULT 1",
            [],
        );

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
        copy_count: i64,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_items
             (timestamp, data_type, is_sensitive, is_encrypted, preview_text, data_size, data_blob_id, metadata, copy_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                timestamp,
                data_type,
                is_sensitive,
                is_encrypted,
                preview_text,
                data_size,
                data_blob_id,
                metadata,
                copy_count,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent clipboard items (limit by count)
    pub fn get_recent_items(&self, limit: i32) -> Result<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, data_type, is_sensitive, is_encrypted,
                    preview_text, data_size, data_blob_id, metadata,
                    COALESCE(copy_count, 1)
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
                copy_count: row.get(9)?,
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

    /// Soft-delete all clipboard items (move to deleted_items/deleted_data tables)
    pub fn soft_delete_all_items(&self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        let tx = self.conn.unchecked_transaction()?;

        // Get all items with their blob IDs (scope stmt so it's dropped before commit)
        let items: Vec<(i64, i64, String, bool, bool, Option<String>, i64, i64, Option<String>)> = {
            let mut stmt = tx.prepare(
                "SELECT id, timestamp, data_type, is_sensitive, is_encrypted,
                        preview_text, data_size, data_blob_id, metadata
                 FROM clipboard_items"
            )?;
            let result = stmt.query_map([], |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                    row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?,
                ))
            })?.collect::<Result<Vec<_>>>()?;
            result
        };

        let count = items.len();
        if count == 0 {
            tx.commit()?;
            return Ok(0);
        }

        for (id, timestamp, data_type, is_sensitive, is_encrypted, preview_text, data_size, blob_id, metadata) in &items {
            // Copy blob data to deleted_data
            let blob_data: Vec<u8> = tx.query_row(
                "SELECT data FROM clipboard_data WHERE id = ?1",
                params![blob_id],
                |row| row.get(0),
            )?;
            tx.execute(
                "INSERT INTO deleted_data (data) VALUES (?1)",
                params![blob_data],
            )?;
            let deleted_blob_id = tx.last_insert_rowid();

            // Copy item to deleted_items
            tx.execute(
                "INSERT INTO deleted_items
                 (original_id, timestamp, deleted_at, data_type, is_sensitive, is_encrypted,
                  preview_text, data_size, deleted_blob_id, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![id, timestamp, now, data_type, is_sensitive, is_encrypted,
                        preview_text, data_size, deleted_blob_id, metadata],
            )?;
        }

        // Delete originals
        let blob_ids: Vec<i64> = items.iter().map(|i| i.7).collect();
        tx.execute("DELETE FROM clipboard_items", [])?;
        for blob_id in blob_ids {
            tx.execute("DELETE FROM clipboard_data WHERE id = ?1", params![blob_id])?;
        }

        tx.commit()?;
        info!("🗑️  Soft-deleted {} clipboard items (recoverable for 7 days)", count);
        Ok(count)
    }

    /// Permanently purge deleted items older than 7 days
    pub fn purge_deleted_items(&self) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (7 * 86400);

        // Get blob IDs of expired deleted items
        let mut stmt = self.conn.prepare(
            "SELECT deleted_blob_id FROM deleted_items WHERE deleted_at < ?1"
        )?;
        let blob_ids: Vec<i64> = stmt.query_map(params![cutoff], |row| {
            row.get(0)
        })?.collect::<Result<Vec<_>>>()?;

        let purged = self.conn.execute(
            "DELETE FROM deleted_items WHERE deleted_at < ?1",
            params![cutoff],
        )?;

        for blob_id in blob_ids {
            self.conn.execute(
                "DELETE FROM deleted_data WHERE id = ?1",
                params![blob_id],
            )?;
        }

        if purged > 0 {
            info!("🗑️  Purged {} expired deleted items (older than 7 days)", purged);
        }
        Ok(purged)
    }

    /// Remove existing items that match the given preview_text and data_type (deduplication).
    /// Skips dedup when preview_text is None (can't reliably compare NULL values).
    /// Returns (removed_count, max_copy_count) so the caller can increment the count.
    pub fn remove_duplicates(&self, preview_text: Option<&str>, data_type: &str) -> Result<(usize, i64)> {
        let preview = match preview_text {
            Some(t) => t,
            None => return Ok((0, 0)),
        };

        // Find matching items, their blob IDs, and copy counts
        let mut stmt = self.conn.prepare(
            "SELECT id, data_blob_id, COALESCE(copy_count, 1) FROM clipboard_items
             WHERE preview_text = ?1 AND data_type = ?2"
        )?;

        let matches: Vec<(i64, i64, i64)> = stmt.query_map(params![preview, data_type], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;

        if matches.is_empty() {
            return Ok((0, 0));
        }

        let max_copy_count = matches.iter().map(|m| m.2).max().unwrap_or(0);

        // Delete the items and their blobs
        for (item_id, blob_id, _) in &matches {
            self.conn.execute(
                "DELETE FROM clipboard_items WHERE id = ?1",
                params![item_id],
            )?;
            self.conn.execute(
                "DELETE FROM clipboard_data WHERE id = ?1",
                params![blob_id],
            )?;
        }

        let count = matches.len();
        if count > 0 {
            info!("♻️  Removed {} duplicate(s) for {:?} (prev count: {})", count, preview, max_copy_count);
        }

        Ok((count, max_copy_count))
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
    pub copy_count: i64,
}
