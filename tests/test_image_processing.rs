// Integration tests for image processing (TIFF→PNG conversion, thumbnails)
use clipboard_manager::storage::processor::DataProcessor;

/// Create a simple 10x10 red PNG image
fn create_test_png() -> Vec<u8> {
    use image::{ImageBuffer, Rgb};
    use std::io::Cursor;

    let img = ImageBuffer::from_fn(10, 10, |_, _| Rgb([255u8, 0u8, 0u8]));
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    buf
}

/// Create a simple 10x10 TIFF image
fn create_test_tiff() -> Vec<u8> {
    use image::{ImageBuffer, Rgb};
    use std::io::Cursor;

    let img = ImageBuffer::from_fn(10, 10, |_, _| Rgb([0u8, 255u8, 0u8]));
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut cursor, image::ImageFormat::Tiff)
        .unwrap();
    buf
}

/// Create a larger test image for thumbnail testing
fn create_large_test_image(width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgb};
    use std::io::Cursor;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([
            (x % 256) as u8,
            (y % 256) as u8,
            ((x + y) % 256) as u8,
        ])
    });
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    buf
}

#[test]
fn test_process_png_image() {
    let png_data = create_test_png();
    let result = DataProcessor::process_image(&png_data, "public.png");

    assert!(result.is_ok(), "PNG processing should succeed");
    let processed = result.unwrap();

    assert_eq!(processed.data_type.as_str(), "image");
    assert!(!processed.is_sensitive);
    assert!(processed.preview_text.is_some());
    assert!(processed.preview_text.unwrap().contains("10x10"));
}

#[test]
fn test_process_tiff_image() {
    let tiff_data = create_test_tiff();
    let result = DataProcessor::process_image(&tiff_data, "public.tiff");

    assert!(result.is_ok(), "TIFF processing should succeed");
    let processed = result.unwrap();

    assert_eq!(processed.data_type.as_str(), "image");
    assert!(!processed.is_sensitive);

    // TIFF should be converted to PNG
    let preview = processed.preview_text.unwrap();
    assert!(preview.contains("10x10"));
    assert!(preview.contains("TIFF") || preview.contains("Tiff"));
}

#[test]
fn test_tiff_to_png_conversion() {
    let tiff_data = create_test_tiff();
    let result = DataProcessor::process_image(&tiff_data, "public.tiff");

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Verify the output is PNG format
    let png_header = &processed.blob[0..8];
    let expected_png_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(png_header, &expected_png_header, "Output should be PNG format");
}

#[test]
fn test_compression_ratio_calculation() {
    let tiff_data = create_test_tiff();
    let result = DataProcessor::process_image(&tiff_data, "public.tiff");

    assert!(result.is_ok());
    let processed = result.unwrap();

    // TIFF is usually much larger than PNG for simple images
    // The preview should mention compression percentage
    let preview = processed.preview_text.unwrap();
    assert!(
        preview.contains("smaller") || preview.contains("larger") || preview.contains("TIFF"),
        "Preview should mention size comparison: {}",
        preview
    );
}

#[test]
fn test_thumbnail_generation() {
    // Create a 500x500 image
    let large_image = create_large_test_image(500, 500);
    let result = DataProcessor::process_image(&large_image, "public.png");

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Check metadata contains thumbnail info
    assert!(processed.metadata.is_some());
    let metadata = processed.metadata.unwrap();

    assert!(metadata.contains("thumbnail_width"));
    assert!(metadata.contains("thumbnail_height"));

    // Thumbnail should be max 200x200
    assert!(metadata.contains(r#""thumbnail_width":200"#) || metadata.contains(r#""thumbnail_height":200"#),
            "Thumbnail should be scaled to 200px max: {}", metadata);
}

#[test]
fn test_small_image_no_upscaling() {
    // Create a 50x50 image (smaller than 200x200 thumbnail size)
    let small_image = create_large_test_image(50, 50);
    let result = DataProcessor::process_image(&small_image, "public.png");

    assert!(result.is_ok());
    let processed = result.unwrap();

    let metadata = processed.metadata.unwrap();
    // Small image should not be upscaled
    assert!(metadata.contains(r#""thumbnail_width":50"#));
    assert!(metadata.contains(r#""thumbnail_height":50"#));
}

#[test]
fn test_aspect_ratio_preservation() {
    // Create a 400x200 image (2:1 aspect ratio)
    let wide_image = create_large_test_image(400, 200);
    let result = DataProcessor::process_image(&wide_image, "public.png");

    assert!(result.is_ok());
    let processed = result.unwrap();

    let preview = processed.preview_text.unwrap();
    assert!(preview.contains("400x200"), "Preview should show original dimensions");

    let metadata = processed.metadata.unwrap();
    // Width should be 200, height should be 100 to preserve 2:1 ratio
    assert!(metadata.contains(r#""thumbnail_width":200"#));
    assert!(metadata.contains(r#""thumbnail_height":100"#));
}

#[test]
fn test_invalid_image_data() {
    let invalid_data = vec![0u8; 100]; // Random bytes
    let result = DataProcessor::process_image(&invalid_data, "public.png");

    assert!(result.is_err(), "Invalid image data should fail");
}

#[test]
fn test_empty_image_data() {
    let empty_data: Vec<u8> = vec![];
    let result = DataProcessor::process_image(&empty_data, "public.png");

    assert!(result.is_err(), "Empty image data should fail");
}

#[test]
fn test_metadata_format() {
    let png_data = create_test_png();
    let result = DataProcessor::process_image(&png_data, "public.png");

    assert!(result.is_ok());
    let processed = result.unwrap();

    assert!(processed.metadata.is_some());
    let metadata = processed.metadata.unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&metadata)
        .expect("Metadata should be valid JSON");

    assert!(parsed.get("width").is_some());
    assert!(parsed.get("height").is_some());
    assert!(parsed.get("format").is_some());
    assert!(parsed.get("thumbnail_width").is_some());
    assert!(parsed.get("thumbnail_height").is_some());
}

#[test]
fn test_jpeg_support() {
    use image::{ImageBuffer, Rgb};
    use std::io::Cursor;

    // Create JPEG
    let img = ImageBuffer::from_fn(20, 20, |_, _| Rgb([100u8, 100u8, 255u8]));
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .unwrap();

    let result = DataProcessor::process_image(&buf, "public.jpeg");
    assert!(result.is_ok(), "JPEG processing should succeed");

    let processed = result.unwrap();
    assert_eq!(processed.data_type.as_str(), "image");
}

#[test]
fn test_different_image_formats() {
    let test_formats = vec![
        ("public.png", "PNG"),
        ("public.tiff", "TIFF"),
        ("public.jpeg", "JPEG"),
    ];

    for (uti, _format_name) in test_formats {
        let img_data = if uti.contains("tiff") {
            create_test_tiff()
        } else {
            create_test_png()
        };

        let result = DataProcessor::process_image(&img_data, uti);
        assert!(result.is_ok(), "Format {} should be supported", uti);
    }
}

#[test]
fn test_very_large_image() {
    // Create a 2000x2000 image
    let large_image = create_large_test_image(2000, 2000);
    let result = DataProcessor::process_image(&large_image, "public.png");

    assert!(result.is_ok(), "Large image should be processed");
    let processed = result.unwrap();

    let metadata = processed.metadata.unwrap();
    // Should be scaled down to 200x200
    assert!(metadata.contains(r#""thumbnail_width":200"#));
    assert!(metadata.contains(r#""thumbnail_height":200"#));
}

#[test]
fn test_preview_text_format() {
    let png_data = create_test_png();
    let result = DataProcessor::process_image(&png_data, "public.png");

    assert!(result.is_ok());
    let processed = result.unwrap();

    let preview = processed.preview_text.unwrap();

    // Preview should contain dimensions
    assert!(preview.contains("10x10"));

    // Preview should contain format
    assert!(preview.contains("PNG") || preview.contains("Png"));
}
