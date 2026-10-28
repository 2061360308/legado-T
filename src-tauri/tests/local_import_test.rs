use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::domain::local_import;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_import_test_{stamp}"))
}

#[test]
fn lists_and_imports_with_filename_script() {
    let dir = temp_dir();
    let books = dir.join("books");
    let nested = books.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        books.join("张三 - 李四.txt"),
        "第一章 开始\n这是正文。\n",
    )
    .unwrap();
    fs::write(nested.join("另一本.epub"), b"not an epub").unwrap();
    fs::write(books.join("ignore.doc"), b"ignored").unwrap();

    let conn = db::open_in_memory().unwrap();
    let entries = local_import::list_dir(&conn, &books.to_string_lossy()).unwrap();
    assert!(entries.iter().any(|entry| entry.name == "张三 - 李四.txt"));
    assert!(!entries.iter().any(|entry| entry.name == "ignore.doc"));
    assert!(entries.iter().any(|entry| entry.is_dir && entry.name == "nested"));

    let scanned = local_import::scan_dir(&conn, &books.to_string_lossy()).unwrap();
    assert_eq!(scanned.len(), 2);

    let results = local_import::import_files(
        &conn,
        &dir.join("data"),
        &[books.join("张三 - 李四.txt").to_string_lossy().to_string()],
        0,
        Some(r#"name = src.split(" - ")[0]; author = src.split(" - ")[1] || "";"#),
    );
    assert_eq!(results.len(), 1);
    assert!(results[0].error.is_none(), "{:?}", results[0].error);
    let book_url = results[0].book_url.clone().unwrap();
    let book = local_reader_lib::data::repositories::BookRepo::get(&conn, &book_url)
        .unwrap()
        .unwrap();
    assert_eq!(book.name, "张三");
    assert_eq!(book.author, "李四");

    let mut book = book;
    book.custom_intro = Some("自定义简介".to_string());
    local_reader_lib::data::repositories::BookRepo::upsert(&conn, &book).unwrap();
    let loaded = local_reader_lib::data::repositories::BookRepo::get(&conn, &book_url)
        .unwrap()
        .unwrap();
    assert_eq!(loaded.custom_intro.as_deref(), Some("自定义简介"));

    fs::remove_dir_all(&dir).ok();
}
