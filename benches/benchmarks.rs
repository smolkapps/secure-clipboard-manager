// Performance benchmarks for ClipVault
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clipboard_manager::storage::{
    database::{Database, ClipboardItem},
    encryption::Encryptor,
    search::SearchEngine,
    processor::DataProcessor,
};
use tempfile::TempDir;

// Helper to create test items
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
        copy_count: 1,
        is_pinned: false,
    }
}

// Helper to insert an item into the database using the actual API
fn insert_item(db: &Database, text: &str, timestamp: i64) -> i64 {
    let blob_id = db.store_blob(text.as_bytes()).unwrap();
    db.store_item(
        timestamp,
        "text",
        false,
        false,
        Some(text),
        text.len() as i64,
        blob_id,
        None,
        1,
    ).unwrap()
}

// Benchmark: Database insert operations
fn bench_database_insert(c: &mut Criterion) {
    c.bench_function("database_insert_text", |b| {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("bench.db")).unwrap();
        let text = "This is a test clipboard item for benchmarking";
        let mut ts = chrono::Utc::now().timestamp();

            ts += 1;
            insert_item(&db, text, ts);
    });
}

// Benchmark: Database query operations
fn bench_database_query(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let db = Database::new(temp_dir.path().join("bench.db")).unwrap();

    // Insert 100 items
    let base_ts = chrono::Utc::now().timestamp();
    for i in 0..100 {
        let text = format!("Item {}", i);
        insert_item(&db, &text, base_ts + i);
    }

    c.bench_function("database_query_recent_10", |b| {
        b.iter(|| {
            black_box(db.get_recent_items(10).unwrap());
        });
    });

    c.bench_function("database_query_recent_100", |b| {
        b.iter(|| {
            black_box(db.get_recent_items(100).unwrap());
        });
    });
}

// Benchmark: Encryption operations
fn bench_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption");
    let temp_dir = TempDir::new().unwrap();
    let encryptor = Encryptor::new(temp_dir.path().join("bench.key")).unwrap();

    for size in [100, 1000, 10000] {
        let data = vec![0u8; size];

        group.bench_with_input(
            BenchmarkId::new("encrypt", size),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(encryptor.encrypt(data).unwrap());
                });
            },
        );

        let encrypted = encryptor.encrypt(&data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("decrypt", size),
            &encrypted,
            |b, encrypted| {
                b.iter(|| {
                    black_box(encryptor.decrypt(encrypted).unwrap());
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Fuzzy search with different dataset sizes
fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    let engine = SearchEngine::new();

    for item_count in [10, 100, 500, 1000] {
        let items: Vec<ClipboardItem> = (0..item_count)
            .map(|i| create_test_item(
                i,
                &format!("Test clipboard item number {} with some searchable text", i),
                i,
            ))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("search_exact_match", item_count),
            &items,
            |b, items| {
                b.iter(|| {
                    black_box(engine.search(items, "test"));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("search_fuzzy_match", item_count),
            &items,
            |b, items| {
                b.iter(|| {
                    black_box(engine.search(items, "clipbrd"));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("search_no_match", item_count),
            &items,
            |b, items| {
                b.iter(|| {
                    black_box(engine.search(items, "xyznotfound"));
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Sensitive data detection
fn bench_sensitive_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("sensitive_detection");

    let test_cases = vec![
        ("normal_text", "This is just a normal text message"),
        ("api_key", "sk-1234567890abcdefghijklmnopqrstuvwxyz"),
        ("jwt", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"),
        ("code", "function test() { return 42; }"),
        ("url", "https://github.com/user/repo"),
    ];

    for (name, text) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(DataProcessor::process_text(text, &[]));
            });
        });
    }

    group.finish();
}

// Benchmark: Image processing
fn bench_image_processing(c: &mut Criterion) {
    use image::{ImageBuffer, Rgb};
    use std::io::Cursor;

    let mut group = c.benchmark_group("image_processing");

    for size in [100, 500, 1000] {
        let img = ImageBuffer::from_fn(size, size, |x, y| {
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

        group.bench_with_input(
            BenchmarkId::new("process_png", size),
            &buf,
            |b, data| {
                b.iter(|| {
                    black_box(DataProcessor::process_image(data, "public.png").unwrap());
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Full workflow (insert + search)
fn bench_full_workflow(c: &mut Criterion) {
    c.bench_function("workflow_insert_and_search", |b| {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("bench.db")).unwrap();
        let engine = SearchEngine::new();
        let mut ts = chrono::Utc::now().timestamp();

        b.iter(|| {
            ts += 1;
            // Insert item
            let text = "This is a test clipboard item";
            insert_item(&db, text, ts);

            // Query recent items
            let items = db.get_recent_items(10).unwrap();
            let _results = engine.search(&items, "test");
        });
    });
}

criterion_group!(
    benches,
    bench_database_insert,
    bench_database_query,
    bench_encryption,
    bench_search,
    bench_sensitive_detection,
    bench_image_processing,
    bench_full_workflow,
);

criterion_main!(benches);
