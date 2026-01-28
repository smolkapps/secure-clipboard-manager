// Integration tests for sensitive data detection
use clipboard_manager::storage::processor::DataProcessor;

#[test]
fn test_detect_openai_api_key() {
    let text = "sk-1234567890abcdefghijklmnopqrstuvwxyz";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "OpenAI API key should be detected as sensitive");
}

#[test]
fn test_detect_github_token() {
    let text = "ghp_1234567890abcdefghijklmnopqrstuvwxyz";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "GitHub token should be detected as sensitive");
}

#[test]
fn test_detect_aws_key() {
    let text = "AKIAIOSFODNN7EXAMPLE";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "AWS access key should be detected as sensitive");
}

#[test]
fn test_detect_jwt_token() {
    let text = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "JWT token should be detected as sensitive");
}

#[test]
fn test_detect_private_key() {
    let text = "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC\n-----END PRIVATE KEY-----";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "Private key should be detected as sensitive");
}

#[test]
fn test_detect_password_like() {
    let test_cases = vec![
        "password=mysecretpassword123",
        "PASSWORD: SuperSecret456!",
        "pwd=test123",
        "secret_key=abcd1234",
    ];

    for text in test_cases {
        let processed = DataProcessor::process_text(text, &[]);
        assert!(processed.is_sensitive, "Password-like string '{}' should be detected as sensitive", text);
    }
}

#[test]
fn test_detect_env_var_with_secret() {
    let text = "export DATABASE_PASSWORD=mysecretdb123";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "Environment variable with password should be detected as sensitive");
}

#[test]
fn test_normal_text_not_sensitive() {
    let test_cases = vec![
        "Hello, World!",
        "This is a regular text message",
        "Meeting notes from today",
        "https://github.com/user/repo",
        "Just some code: function test() { return 42; }",
        "Shopping list: milk, eggs, bread",
    ];

    for text in test_cases {
        let processed = DataProcessor::process_text(text, &[]);
        assert!(!processed.is_sensitive, "Normal text '{}' should not be detected as sensitive", text);
    }
}

#[test]
fn test_code_snippet_not_sensitive() {
    let text = r#"
const apiKey = process.env.API_KEY;
fetch('https://api.example.com', {
    headers: { Authorization: `Bearer ${token}` }
});
"#;
    let processed = DataProcessor::process_text(text, &[]);

    // Code that references API keys but doesn't contain them shouldn't be marked sensitive
    assert!(!processed.is_sensitive, "Code snippet with variable names should not be sensitive");
}

#[test]
fn test_url_not_sensitive() {
    let urls = vec![
        "https://www.google.com",
        "http://example.com/path?query=value",
        "https://github.com/user/repo/issues/123",
    ];

    for url in urls {
        let processed = DataProcessor::process_text(url, &[]);
        assert!(!processed.is_sensitive, "URL '{}' should not be detected as sensitive", url);
    }
}

#[test]
fn test_mixed_content_with_sensitive() {
    let text = "Here is my API key: sk-1234567890abcdefghijklmnopqrstuvwxyz for testing";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "Mixed content with API key should be detected as sensitive");
}

#[test]
fn test_base64_encoded_data() {
    // Some base64 might look like tokens but aren't necessarily sensitive
    let text = "SGVsbG8gV29ybGQh"; // "Hello World!" in base64
    let processed = DataProcessor::process_text(text, &[]);

    // This is just base64 of normal text, shouldn't be sensitive
    assert!(!processed.is_sensitive, "Base64 encoded normal text should not be sensitive");
}

#[test]
fn test_api_key_in_json() {
    let text = r#"{"api_key": "sk-1234567890abcdefghijklmnopqrstuvwxyz", "user": "test"}"#;
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "JSON with API key should be detected as sensitive");
}

#[test]
fn test_connection_string() {
    let text = "postgresql://user:password@localhost:5432/database";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(processed.is_sensitive, "Database connection string should be detected as sensitive");
}

#[test]
fn test_empty_string() {
    let text = "";
    let processed = DataProcessor::process_text(text, &[]);

    assert!(!processed.is_sensitive, "Empty string should not be sensitive");
}

#[test]
fn test_very_long_text() {
    let text = "a".repeat(10000);
    let processed = DataProcessor::process_text(&text, &[]);

    assert!(!processed.is_sensitive, "Long non-sensitive text should not be detected as sensitive");
}

#[test]
fn test_sensitive_data_types() {
    let sensitive_texts = vec![
        ("sk-proj-1234567890", "OpenAI project key"),
        ("github_pat_1234567890", "GitHub PAT"),
        ("xoxb-1234567890", "Slack bot token"),
        ("AIzaSy1234567890", "Google API key"),
        ("-----BEGIN RSA PRIVATE KEY-----", "RSA private key"),
    ];

    for (text, description) in sensitive_texts {
        let processed = DataProcessor::process_text(text, &[]);
        assert!(processed.is_sensitive, "{} should be detected as sensitive", description);
    }
}
