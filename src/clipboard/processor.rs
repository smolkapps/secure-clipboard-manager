// Clipboard data processing and type detection
use log::debug;

/// Supported clipboard data types
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Text,
    RTF,
    HTML,
    Image,
    File,
    URL,
    Unknown,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Text => "text",
            DataType::RTF => "rtf",
            DataType::HTML => "html",
            DataType::Image => "image",
            DataType::File => "file",
            DataType::URL => "url",
            DataType::Unknown => "unknown",
        }
    }
}

/// Processed clipboard data ready for storage
#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub data_type: DataType,
    pub content: Vec<u8>,
    pub preview_text: Option<String>,
    pub metadata: serde_json::Value,
}

impl ClipboardData {
    /// Create new clipboard data from raw bytes and UTI types
    pub fn from_types(types: &[String], raw_data: Vec<u8>) -> Self {
        let data_type = Self::detect_type(types);
        let preview_text = Self::generate_preview(&data_type, &raw_data);
        let metadata = Self::build_metadata(types);

        debug!("Processed clipboard data: type={:?}, size={} bytes", data_type, raw_data.len());

        Self {
            data_type,
            content: raw_data,
            preview_text,
            metadata,
        }
    }

    /// Detect clipboard data type from UTI type list
    fn detect_type(types: &[String]) -> DataType {
        // Check types in priority order
        for t in types {
            let t_lower = t.to_lowercase();

            if t_lower.contains("image") || t_lower.contains("png") || t_lower.contains("tiff") {
                return DataType::Image;
            }

            if t_lower.contains("rtf") {
                return DataType::RTF;
            }

            if t_lower.contains("html") {
                return DataType::HTML;
            }

            if t_lower.contains("file-url") || t_lower.contains("file") {
                return DataType::File;
            }

            if t_lower.contains("url") {
                return DataType::URL;
            }

            if t_lower.contains("string") || t_lower.contains("text") || t_lower.contains("utf8") {
                return DataType::Text;
            }
        }

        DataType::Unknown
    }

    /// Generate preview text for search indexing (first 200 chars)
    fn generate_preview(data_type: &DataType, raw_data: &[u8]) -> Option<String> {
        match data_type {
            DataType::Text | DataType::HTML | DataType::URL => {
                String::from_utf8(raw_data.to_vec())
                    .ok()
                    .map(|s| {
                        let preview: String = s.chars().take(200).collect();
                        preview.trim().to_string()
                    })
            }
            DataType::RTF => {
                // For RTF, extract plain text preview (simplified)
                String::from_utf8_lossy(raw_data)
                    .chars()
                    .take(200)
                    .collect::<String>()
                    .into()
            }
            DataType::Image => Some("[Image]".to_string()),
            DataType::File => Some("[File]".to_string()),
            DataType::Unknown => None,
        }
    }

    /// Build JSON metadata from UTI types
    fn build_metadata(types: &[String]) -> serde_json::Value {
        serde_json::json!({
            "uti_types": types,
            "type_count": types.len(),
        })
    }

    /// Get size of content in bytes
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Check if this data is likely sensitive (will be expanded in Phase 3)
    pub fn is_sensitive(&self) -> bool {
        // Basic check for now - will implement pattern matching in Phase 3
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_text_type() {
        let types = vec!["public.utf8-plain-text".to_string(), "String".to_string()];
        let data_type = ClipboardData::detect_type(&types);
        assert_eq!(data_type, DataType::Text);
    }

    #[test]
    fn test_detect_image_type() {
        let types = vec!["public.png".to_string(), "public.tiff".to_string()];
        let data_type = ClipboardData::detect_type(&types);
        assert_eq!(data_type, DataType::Image);
    }

    #[test]
    fn test_generate_text_preview() {
        let long_text = "Hello world! ".repeat(50);
        let preview = ClipboardData::generate_preview(
            &DataType::Text,
            long_text.as_bytes()
        );

        assert!(preview.is_some());
        let preview_str = preview.unwrap();
        assert!(preview_str.len() <= 200);
        assert!(preview_str.starts_with("Hello world!"));
    }

    #[test]
    fn test_image_preview() {
        let preview = ClipboardData::generate_preview(&DataType::Image, &[]);
        assert_eq!(preview, Some("[Image]".to_string()));
    }

    #[test]
    fn test_clipboard_data_creation() {
        let types = vec!["String".to_string()];
        let content = b"Test content".to_vec();
        let data = ClipboardData::from_types(&types, content.clone());

        assert_eq!(data.data_type, DataType::Text);
        assert_eq!(data.content, content);
        assert_eq!(data.size(), 12);
    }
}
