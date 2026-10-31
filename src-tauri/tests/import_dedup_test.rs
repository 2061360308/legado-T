use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::data::repositories::BookRepo;
use local_reader_lib::domain::local_import;
use local_reader_lib::domain::storage::BookStorage;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_dedup_test_{stamp}"))
}

#[test]
fn importing_the_same_file_twice_keeps_one_book() {
    let dir = temp_dir();
    let books = dir.join("books");
    fs::create_dir_all(&books).unwrap();
    let file = books.join("dedup-book.txt");
    fs::write(&file, "Chapter 1\nHello\n\nChapter 2\nWorld\n").unwrap();

    let conn = db::open_in_memory().unwrap();
    let storage = BookStorage::new(&dir.join("data"));
    storage.ensure_layout().unwrap();
    let paths = vec![file.to_string_lossy().to_string()];

    let first = local_import::import_files(&conn, &storage, &paths, 0, None, None);
    assert_eq!(first.len(), 1);
    assert!(first[0].error.is_none(), "{:?}", first[0].error);
    assert!(!first[0].duplicate);

    let second = local_import::import_files(&conn, &storage, &paths, 0, None, None);
    assert_eq!(second.len(), 1);
    assert!(second[0].error.is_none(), "{:?}", second[0].error);
    assert!(second[0].duplicate);
    assert_eq!(second[0].book_url, first[0].book_url);
    assert_eq!(BookRepo::count(&conn).unwrap(), 1);

    fs::remove_dir_all(&dir).ok();
}
