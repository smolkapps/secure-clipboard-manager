// In-memory clipboard history storage
use super::ClipboardChange;
use std::sync::{Arc, RwLock};

/// Clipboard history item with content
#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data_type: String, // "text", "image", etc.
}

/// In-memory clipboard history manager
pub struct ClipboardHistory {
    items: Arc<RwLock<Vec<HistoryItem>>>,
    max_items: usize,
}

impl ClipboardHistory {
    /// Create new history with max capacity
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
            max_items,
        }
    }

    /// Add a clipboard change to history
    pub fn add(&self, change: &ClipboardChange, content: Option<String>) {
        if let Some(text) = content {
            let item = HistoryItem {
                content: text,
                timestamp: change.timestamp,
                data_type: Self::determine_type(&change.types),
            };

            let mut items = self.items.write().unwrap();

            // Don't add duplicates of the most recent item
            if let Some(last) = items.first() {
                if last.content == item.content {
                    return;
                }
            }

            items.insert(0, item);

            // Trim to max size
            if items.len() > self.max_items {
                items.truncate(self.max_items);
            }
        }
    }

    /// Get all items (newest first)
    pub fn get_items(&self) -> Vec<HistoryItem> {
        self.items.read().unwrap().clone()
    }

    /// Get item count
    pub fn count(&self) -> usize {
        self.items.read().unwrap().len()
    }

    /// Clear all history
    pub fn clear(&self) {
        self.items.write().unwrap().clear();
    }

    /// Determine data type from UTI types
    fn determine_type(types: &[String]) -> String {
        for t in types {
            let t_lower = t.to_lowercase();
            if t_lower.contains("image") || t_lower.contains("png") || t_lower.contains("tiff") {
                return "image".to_string();
            }
            if t_lower.contains("file") {
                return "file".to_string();
            }
        }
        "text".to_string()
    }

    /// Get shared reference for use in multiple places
    pub fn clone_ref(&self) -> Arc<RwLock<Vec<HistoryItem>>> {
        Arc::clone(&self.items)
    }
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new(100) // Default to 100 items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_item() {
        let history = ClipboardHistory::new(5);
        let change = ClipboardChange {
            change_count: 1,
            types: vec!["public.utf8-plain-text".to_string()],
            timestamp: chrono::Utc::now(),
        };

        history.add(&change, Some("test content".to_string()));
        assert_eq!(history.count(), 1);
    }

    #[test]
    fn test_max_capacity() {
        let history = ClipboardHistory::new(3);
        let change = ClipboardChange {
            change_count: 1,
            types: vec!["public.utf8-plain-text".to_string()],
            timestamp: chrono::Utc::now(),
        };

        for i in 0..5 {
            history.add(&change, Some(format!("item {}", i)));
        }

        assert_eq!(history.count(), 3);
    }

    #[test]
    fn test_no_duplicates() {
        let history = ClipboardHistory::new(10);
        let change = ClipboardChange {
            change_count: 1,
            types: vec!["public.utf8-plain-text".to_string()],
            timestamp: chrono::Utc::now(),
        };

        history.add(&change, Some("same content".to_string()));
        history.add(&change, Some("same content".to_string()));

        assert_eq!(history.count(), 1);
    }
}
