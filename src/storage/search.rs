// Fuzzy search for clipboard history
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::storage::database::ClipboardItem;

pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn new() -> Self {
        SearchEngine {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search clipboard items by query string
    /// Returns items sorted by relevance score (highest first)
    pub fn search<'a>(&self, items: &'a [ClipboardItem], query: &str) -> Vec<(i64, &'a ClipboardItem)> {
        if query.is_empty() {
            // No query - return all items with neutral score
            return items.iter().map(|item| (0, item)).collect();
        }

        let mut results: Vec<(i64, &ClipboardItem)> = items
            .iter()
            .filter_map(|item| {
                // Search in preview text
                if let Some(preview) = &item.preview_text {
                    if let Some(score) = self.matcher.fuzzy_match(preview, query) {
                        return Some((score, item));
                    }
                }

                // Search in data type
                if let Some(score) = self.matcher.fuzzy_match(&item.data_type, query) {
                    return Some((score, item));
                }

                None
            })
            .collect();

        // Sort by score (highest first), then by timestamp (newest first)
        results.sort_by(|a, b| {
            b.0.cmp(&a.0).then_with(|| b.1.timestamp.cmp(&a.1.timestamp))
        });

        results
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_item(id: i64, preview: &str, timestamp: i64) -> ClipboardItem {
        ClipboardItem {
            id,
            timestamp,
            data_type: "text".to_string(),
            is_sensitive: false,
            is_encrypted: false,
            preview_text: Some(preview.to_string()),
            data_size: preview.len() as i64,
            data_blob_id: id,
            metadata: None,
        }
    }

    #[test]
    fn test_fuzzy_search() {
        let engine = SearchEngine::new();
        let items = vec![
            create_test_item(1, "Hello World", 100),
            create_test_item(2, "Fuzzy Search Test", 200),
            create_test_item(3, "Another test item", 150),
        ];

        let results = engine.search(&items, "test");
        assert!(results.len() >= 2); // Should match "test" items

        // First result should have highest score
        assert!(results[0].0 >= results[1].0);
    }

    #[test]
    fn test_empty_query() {
        let engine = SearchEngine::new();
        let items = vec![
            create_test_item(1, "Item 1", 100),
            create_test_item(2, "Item 2", 200),
        ];

        let results = engine.search(&items, "");
        assert_eq!(results.len(), 2); // Should return all items
    }

    #[test]
    fn test_no_matches() {
        let engine = SearchEngine::new();
        let items = vec![
            create_test_item(1, "Hello World", 100),
        ];

        let results = engine.search(&items, "xyz123notfound");
        assert_eq!(results.len(), 0); // Should return no matches
    }

    #[test]
    fn test_sorted_by_relevance() {
        let engine = SearchEngine::new();
        let items = vec![
            create_test_item(1, "test", 100),           // Exact match
            create_test_item(2, "testing is fun", 200), // Partial match
            create_test_item(3, "t e s t", 150),        // Scattered match
        ];

        let results = engine.search(&items, "test");

        // Should return results
        assert!(!results.is_empty());

        // All results should contain "test" in some form
        assert_eq!(results.len(), 3);

        // Scores should be in descending order
        for i in 0..results.len() - 1 {
            assert!(results[i].0 >= results[i + 1].0);
        }
    }
}
