use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::data::keyvalue::KeyValue;
use local_reader_lib::data::migration;
use local_reader_lib::data::repositories::{BookRepo, ChapterRepo};
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::model::{book_type, Book, BookChapter, ContentBlock};
use local_reader_lib::domain::storage::BookStorage;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_storage_test_{stamp}"))
}

#[test]
fn imports_into_managed_layout_and_deduplicates() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("book.txt");
    fs::write(&source, "第一章 开始\n正文内容。\n").unwrap();

    let conn = db::open_in_memory().unwrap();
    let storage = BookStorage::new(&dir);
    storage.ensure_layout().unwrap();

    let first = import_service::import_txt(
        &conn,
        &source.to_string_lossy(),
        &storage,
        None,
        0,
        Some("UTF-8"),
        false,
    )
    .unwrap();
    assert!(first.book.book_url.starts_with("local:"));
    assert_eq!(
        first.book.source_hash.as_deref(),
        first.book.book_url.strip_prefix("local:")
    );
    assert!(first
        .book
        .source_path
        .as_deref()
        .unwrap()
        .starts_with("books/"));
    let paths = storage.paths(&first.book).unwrap();
    assert!(paths.source.exists());
    assert_eq!(BookRepo::count(&conn).unwrap(), 1);

    let second = import_service::import_txt(
        &conn,
        &source.to_string_lossy(),
        &storage,
        Some("另一个书名"),
        0,
        Some("UTF-8"),
        false,
    )
    .unwrap();
    assert_eq!(first.book.book_url, second.book.book_url);
    assert_eq!(BookRepo::count(&conn).unwrap(), 1);

    storage.clear_cache(&first.book).unwrap();
    assert!(!paths.chapters.exists());
    assert!(paths.source.exists());

    assert!(storage.delete_managed(&first.book).unwrap());
    assert!(!paths.dir.exists());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn adopts_staged_file_without_second_copy() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let storage = BookStorage::new(&dir);
    storage.ensure_layout().unwrap();
    let staged = storage.staging_path("epub").unwrap();
    fs::write(&staged, b"staged").unwrap();

    let source = storage.adopt_staged(&staged, "local:abc", "epub").unwrap();
    assert!(!staged.exists());
    assert_eq!(fs::read(&source).unwrap(), b"staged");
    assert!(source.ends_with("books/abc/source.epub"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn migrates_legacy_absolute_book_paths() {
    let dir = temp_dir();
    let legacy = dir.join("booksData");
    fs::create_dir_all(&legacy).unwrap();
    let source = legacy.join("old.txt");
    fs::write(&source, "第一章 开始\n正文。\n").unwrap();
    let legacy_chapters = legacy.join("old.txt.chapters");
    fs::create_dir_all(legacy_chapters.join("images")).unwrap();
    fs::write(legacy_chapters.join("0.txt"), "正文。\n").unwrap();
    let image = legacy_chapters.join("images").join("a.png");
    fs::write(&image, b"png").unwrap();
    let blocks = vec![ContentBlock::Image {
        src: image.to_string_lossy().to_string(),
        width: 1,
        height: 1,
        alt: None,
    }];
    fs::write(
        legacy_chapters.join("0.json"),
        serde_json::to_vec(&blocks).unwrap(),
    )
    .unwrap();

    let conn = db::open_in_memory().unwrap();
    let storage = BookStorage::new(&dir);
    storage.ensure_layout().unwrap();
    let old_url = source.to_string_lossy().to_string();
    let book = Book {
        book_url: old_url.clone(),
        name: "旧书".into(),
        author: "作者".into(),
        origin: "local".into(),
        origin_name: "old.txt".into(),
        r#type: book_type::TEXT | book_type::LOCAL,
        total_chapter_num: 1,
        ..Default::default()
    };
    BookRepo::upsert(&conn, &book).unwrap();
    ChapterRepo::replace_all(
        &conn,
        &old_url,
        &[BookChapter {
            url: format!("{old_url}#0"),
            title: "第一章".into(),
            index: 0,
            resource_url: Some(old_url.clone()),
            ..Default::default()
        }],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO bookmarks (time,bookName,bookUrl) VALUES (1,'旧书',?1)",
        [&old_url],
    )
    .unwrap();
    KeyValue::set(&conn, &format!("chapterOverride:{old_url}:0"), "x").unwrap();
    KeyValue::cache_set(&conn, &format!("toc:{old_url}:1"), "[]").unwrap();

    assert_eq!(migration::migrate_legacy_books(&conn, &storage).unwrap(), 1);
    let migrated = BookRepo::list(&conn).unwrap();
    assert_eq!(migrated.len(), 1);
    let book = &migrated[0];
    assert!(book.book_url.starts_with("local:"));
    assert!(book.source_path.is_some());
    assert!(storage.paths(book).unwrap().source.exists());
    assert!(!source.exists());
    assert!(!legacy_chapters.exists());

    let chapters = ChapterRepo::list(&conn, &book.book_url).unwrap();
    assert_eq!(chapters.len(), 1);
    assert!(chapters[0].url.starts_with(&book.book_url));
    assert_eq!(
        chapters[0].resource_url.as_deref(),
        Some(book.book_url.as_str())
    );

    let paths = storage.paths(book).unwrap();
    let raw = fs::read_to_string(paths.chapters.join("0.json")).unwrap();
    let blocks: Vec<ContentBlock> = serde_json::from_str(&raw).unwrap();
    let image = blocks
        .iter()
        .find_map(|block| match block {
            ContentBlock::Image { src, .. } => Some(src),
            _ => None,
        })
        .unwrap();
    assert!(!std::path::Path::new(image).is_absolute());
    assert!(std::path::Path::new(&storage.resolve(image)).exists());

    let bookmark: String = conn
        .query_row("SELECT bookUrl FROM bookmarks WHERE time=1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(bookmark, book.book_url);
    assert!(
        KeyValue::get(&conn, &format!("chapterOverride:{}:0", book.book_url))
            .unwrap()
            .is_some()
    );
    assert!(
        KeyValue::cache_get(&conn, &format!("toc:{}:1", book.book_url))
            .unwrap()
            .is_some()
    );
    fs::remove_dir_all(&dir).ok();
}
