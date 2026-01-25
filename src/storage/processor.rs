// Data processor for clipboard content
use image::{ImageFormat, DynamicImage};
use std::io::Cursor;
use log::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessedDataType {
    PlainText,
    Rtf,
    Html,
    Image,
    File,
    Url,
}

impl ProcessedDataType {
    pub fn as_str(&self) -> &str {
        match self {
            ProcessedDataType::PlainText => "text",
            ProcessedDataType::Rtf => "rtf",
            ProcessedDataType::Html => "html",
            ProcessedDataType::Image => "image",
            ProcessedDataType::File => "file",
            ProcessedDataType::Url => "url",
        }
    }
}

pub struct ProcessedData {
    pub data_type: ProcessedDataType,
    pub blob: Vec<u8>,
    pub preview_text: Option<String>,
    pub is_sensitive: bool,
    pub metadata: Option<String>,
}

pub struct DataProcessor;

impl DataProcessor {
    /// Process raw clipboard text
    pub fn process_text(text: &str, uti_types: &[String]) -> ProcessedData {
        let data_type = Self::detect_text_type(text, uti_types);
        let preview_text = Self::generate_text_preview(text);
        let is_sensitive = Self::detect_sensitive_content(text);

        ProcessedData {
            data_type,
            blob: text.as_bytes().to_vec(),
            preview_text: Some(preview_text),
            is_sensitive,
            metadata: Some(Self::create_metadata(uti_types)),
        }
    }

    /// Process raw clipboard image data
    pub fn process_image(image_data: &[u8], uti_type: &str) -> Result<ProcessedData, String> {
        // Detect source format
        let source_format = Self::detect_image_format(uti_type);

        // Try to load the image
        let img = image::load_from_memory(image_data)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        // Convert to optimized PNG
        let png_data = Self::convert_to_png(&img)?;

        // Generate preview text (dimensions)
        let preview_text = format!("{}x{} image", img.width(), img.height());

        info!("🖼️  Converted {} to PNG ({} -> {} bytes)",
              source_format, image_data.len(), png_data.len());

        Ok(ProcessedData {
            data_type: ProcessedDataType::Image,
            blob: png_data,
            preview_text: Some(preview_text),
            is_sensitive: false,
            metadata: Some(format!("{{\"width\":{},\"height\":{},\"format\":\"{}\"}}",
                                   img.width(), img.height(), source_format)),
        })
    }

    /// Detect text type from content and UTI types
    fn detect_text_type(text: &str, uti_types: &[String]) -> ProcessedDataType {
        // Check UTI types first
        for uti in uti_types {
            if uti.contains("rtf") {
                return ProcessedDataType::Rtf;
            }
            if uti.contains("html") {
                return ProcessedDataType::Html;
            }
        }

        // Check if it's a URL
        if Self::is_url(text) {
            return ProcessedDataType::Url;
        }

        // Check if it's RTF by content
        if text.starts_with("{\\rtf") {
            return ProcessedDataType::Rtf;
        }

        // Check if it's HTML by content
        if text.trim_start().starts_with("<!DOCTYPE") ||
           text.trim_start().starts_with("<html") ||
           text.contains("</html>") {
            return ProcessedDataType::Html;
        }

        ProcessedDataType::PlainText
    }

    /// Check if text is a URL
    fn is_url(text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.starts_with("http://") ||
        trimmed.starts_with("https://") ||
        trimmed.starts_with("ftp://")
    }

    /// Generate preview text (first 200 chars)
    fn generate_text_preview(text: &str) -> String {
        const MAX_PREVIEW: usize = 200;

        let cleaned = text.trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if cleaned.len() <= MAX_PREVIEW {
            cleaned
        } else {
            format!("{}...", &cleaned[..MAX_PREVIEW])
        }
    }

    /// Detect sensitive content (passwords, API keys, etc.)
    fn detect_sensitive_content(text: &str) -> bool {
        let text_lower = text.to_lowercase();

        // Pattern 1: Common password-like patterns
        // - Min 8 chars, contains special chars, no spaces
        if text.len() >= 8 &&
           text.len() <= 128 &&
           !text.contains(' ') &&
           text.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) &&
           text.chars().any(|c| c.is_ascii_digit()) {
            return true;
        }

        // Pattern 2: API keys and tokens
        let sensitive_prefixes = [
            "sk-",          // OpenAI
            "ghp_",         // GitHub personal access token
            "gho_",         // GitHub OAuth token
            "github_pat_",  // GitHub PAT
            "glpat-",       // GitLab
            "AKIA",         // AWS access key
            "ya29.",        // Google OAuth
            "AIza",         // Google API key
        ];

        for prefix in &sensitive_prefixes {
            if text.starts_with(prefix) {
                return true;
            }
        }

        // Pattern 3: JWT tokens
        if text.starts_with("eyJ") && text.matches('.').count() == 2 {
            return true;
        }

        // Pattern 4: Private keys
        if text.contains("BEGIN PRIVATE KEY") ||
           text.contains("BEGIN RSA PRIVATE KEY") ||
           text.contains("BEGIN OPENSSH PRIVATE KEY") {
            return true;
        }

        // Pattern 5: Environment-like variables
        if (text_lower.contains("password") ||
            text_lower.contains("secret") ||
            text_lower.contains("api_key") ||
            text_lower.contains("apikey") ||
            text_lower.contains("token")) &&
           text.contains('=') {
            return true;
        }

        false
    }

    /// Detect image format from UTI type
    fn detect_image_format(uti: &str) -> &str {
        if uti.contains("tiff") {
            "TIFF"
        } else if uti.contains("jpeg") || uti.contains("jpg") {
            "JPEG"
        } else if uti.contains("png") {
            "PNG"
        } else if uti.contains("gif") {
            "GIF"
        } else if uti.contains("bmp") {
            "BMP"
        } else {
            "Unknown"
        }
    }

    /// Convert image to optimized PNG
    fn convert_to_png(img: &DynamicImage) -> Result<Vec<u8>, String> {
        let mut buffer = Cursor::new(Vec::new());

        img.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        Ok(buffer.into_inner())
    }

    /// Create JSON metadata string
    fn create_metadata(uti_types: &[String]) -> String {
        format!("{{\"uti_types\":{}}}", serde_json::to_string(uti_types).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_plain_text() {
        let data = DataProcessor::process_text("Hello, world!", &[]);
        assert_eq!(data.data_type, ProcessedDataType::PlainText);
        assert_eq!(data.preview_text, Some("Hello, world!".to_string()));
        assert!(!data.is_sensitive);
    }

    #[test]
    fn test_detect_url() {
        let data = DataProcessor::process_text("https://example.com", &[]);
        assert_eq!(data.data_type, ProcessedDataType::Url);
    }

    #[test]
    fn test_detect_sensitive_api_key() {
        let data = DataProcessor::process_text("sk-1234567890abcdef", &[]);
        assert!(data.is_sensitive);
    }

    #[test]
    fn test_detect_sensitive_github_token() {
        let data = DataProcessor::process_text("ghp_abcdefghij1234567890", &[]);
        assert!(data.is_sensitive);
    }

    #[test]
    fn test_detect_sensitive_password_like() {
        let data = DataProcessor::process_text("P@ssw0rd123!", &[]);
        assert!(data.is_sensitive);
    }

    #[test]
    fn test_preview_truncation() {
        let long_text = "a".repeat(300);
        let data = DataProcessor::process_text(&long_text, &[]);
        assert!(data.preview_text.unwrap().len() <= 203); // 200 + "..."
    }

    #[test]
    fn test_multiline_preview() {
        let text = "Line 1\n\nLine 2\n\n\nLine 3";
        let data = DataProcessor::process_text(text, &[]);
        assert_eq!(data.preview_text, Some("Line 1 Line 2 Line 3".to_string()));
    }
}
