use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::backup::{export_backup, import_backup};
use local_reader_lib::data::db;
use local_reader_lib::data::keyvalue::KeyValue;
use local_reader_lib::data::repositories::{BookRepo, ChapterRepo, GroupRepo, TxtTocRuleRepo};
use local_reader_lib::domain::model::{book_type, Book, BookChapter, BookGroup, TxtTocRule};

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_backup_{stamp}"))
}

#[test]
fn export_and_restore_merge_without_chapters() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("backup.zip");
    let conn = db::open_in_memory().unwrap();

    let book = Book {
        book_url: "/tmp/book.txt".into(),
        name: "备份书".into(),
        author: "作者".into(),
        origin: "local".into(),
        origin_name: "book.txt".into(),
        r#type: book_type::TEXT | book_type::LOCAL,
        ..Default::default()
    };
    BookRepo::upsert(&conn, &book).unwrap();
    ChapterRepo::replace_all(
        &conn,
        &book.book_url,
        &[BookChapter {
            url: "/tmp/book.txt#0".into(),
            title: "第一章".into(),
            index: 0,
            start: 0,
            end: 10,
            ..Default::default()
        }],
    )
    .unwrap();
    GroupRepo::upsert(
        &conn,
        &BookGroup {
            group_id: 1,
            group_name: "分组".into(),
            show: true,
            enable_refresh: true,
            ..Default::default()
        },
    )
    .unwrap();
    TxtTocRuleRepo::upsert(
        &conn,
        &TxtTocRule {
            id: 7,
            name: "规则".into(),
            rule: "^第.*章".into(),
            enable: true,
            serial_number: 1,
            ..Default::default()
        },
    )
    .unwrap();
    KeyValue::set(&conn, "password", "secret").unwrap();
    KeyValue::set(&conn, "themeMode", "1").unwrap();

    let report = export_backup(&conn, &path.to_string_lossy(), Some("123456"), None).unwrap();
    assert!(report.files >= 3);
    assert!(path.exists());

    let file = fs::File::open(&path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.contains(&"books.json".to_string()));
    assert!(!names.contains(&"chapters.json".to_string()));
    assert!(names.contains(&"config.xml".to_string()));

    let restored = db::open_in_memory().unwrap();
    let restore = import_backup(&restored, &path.to_string_lossy(), Some("123456"), &[]).unwrap();
    assert!(restore.items >= 4);
    assert!(BookRepo::get(&restored, "/tmp/book.txt").unwrap().is_some());
    assert_eq!(GroupRepo::list(&restored).unwrap().len(), 1);
    assert_eq!(TxtTocRuleRepo::list(&restored).unwrap().len(), 1);
    assert_eq!(
        KeyValue::get(&restored, "password").unwrap().as_deref(),
        Some("secret")
    );
    assert_eq!(
        KeyValue::get(&restored, "themeMode").unwrap().as_deref(),
        Some("1")
    );
    assert!(ChapterRepo::list(&restored, "/tmp/book.txt")
        .unwrap()
        .is_empty());

    fs::remove_dir_all(&dir).ok();
}
