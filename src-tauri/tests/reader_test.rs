use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::data::repositories::TxtTocRuleRepo;
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::layout_engine::LayoutStyle;
use local_reader_lib::domain::model::TxtTocRule;
use local_reader_lib::domain::read_state::{self, ReaderCache};
use local_reader_lib::domain::storage::BookStorage;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_test_{stamp}"))
}

#[test]
fn import_then_open_reader_and_save_progress() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("book.txt");
    let mut text = String::new();
    text.push_str("第一章 起点\n");
    text.push_str(&"正文内容。".repeat(400));
    text.push_str("\n第二章 转折\n");
    text.push_str(&"情节推进。".repeat(400));
    fs::write(&txt, text.as_bytes()).unwrap();

    let conn = db::open_in_memory().unwrap();
    let storage = BookStorage::new(&dir);
    storage.ensure_layout().unwrap();
    TxtTocRuleRepo::upsert(
        &conn,
        &TxtTocRule {
            id: 1,
            name: "章节规则".into(),
            rule: r"^第[一二]章.*$".into(),
            example: String::new(),
            serial_number: 1,
            enable: true,
        },
    )
    .unwrap();

    let import = import_service::import_txt(
        &conn,
        &txt.to_string_lossy(),
        &storage,
        Some("测试书"),
        0,
        Some("UTF-8"),
        false,
    )
    .unwrap();
    assert_eq!(import.chapter_count, 2);

    let style = LayoutStyle {
        view_width: 400.0,
        view_height: 600.0,
        font_size: 20.0,
        padding_left: 20.0,
        padding_right: 20.0,
        padding_top: 20.0,
        padding_bottom: 20.0,
        ..Default::default()
    };
    let mut cache = ReaderCache::default();
    let first = read_state::open(
        &conn,
        &storage,
        &mut cache,
        &import.book.book_url,
        None,
        None,
        &style,
    )
    .unwrap();
    assert_eq!(first.chapter_index, 0);
    assert_eq!(first.chapter_title, "第一章 起点");
    assert!(!first.pages.is_empty());
    assert_eq!(first.page_index, 0);

    read_state::save_progress(&conn, &import.book.book_url, 1, 0, "第二章 转折").unwrap();
    let second = read_state::open(
        &conn,
        &storage,
        &mut cache,
        &import.book.book_url,
        None,
        None,
        &style,
    )
    .unwrap();
    assert_eq!(second.chapter_index, 1);
    assert_eq!(second.chapter_title, "第二章 转折");

    let rules = read_state::effective_rules(
        &conn,
        &storage,
        &mut cache,
        &import.book.book_url,
        1,
        &style,
    )
    .unwrap();
    assert!(rules.is_empty());

    fs::remove_dir_all(&dir).ok();
}
