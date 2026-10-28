use std::collections::HashSet;

use local_reader_lib::data::db;
use local_reader_lib::data::default_data;
use local_reader_lib::data::keyvalue::KeyValue;
use local_reader_lib::data::repositories::{
    group_add, group_has, group_remove, BookRepo, ChapterRepo, GroupRepo, TxtTocRuleRepo,
};
use local_reader_lib::data::rule_io::{import_http_tts, import_replace_rules, DiffStatus};
use local_reader_lib::domain::model::{book_type, Book, BookChapter, BookGroup, ReplaceRule};

fn sample_book(url: &str) -> Book {
    Book {
        book_url: url.to_string(),
        name: "测试书".into(),
        author: "作者".into(),
        origin: "local".into(),
        origin_name: "本地".into(),
        r#type: book_type::TEXT | book_type::LOCAL,
        ..Default::default()
    }
}

#[test]
fn schema_and_book_crud() {
    let conn = db::open_in_memory().unwrap();
    let b = sample_book("/tmp/a.txt");
    BookRepo::upsert(&conn, &b).unwrap();

    let got = BookRepo::get(&conn, "/tmp/a.txt").unwrap().unwrap();
    assert_eq!(got.name, "测试书");
    assert!(got.is_local());
    assert_eq!(BookRepo::count(&conn).unwrap(), 1);

    BookRepo::delete(&conn, "/tmp/a.txt").unwrap();
    assert!(BookRepo::get(&conn, "/tmp/a.txt").unwrap().is_none());
}

#[test]
fn chapter_cascade_delete() {
    let conn = db::open_in_memory().unwrap();
    BookRepo::upsert(&conn, &sample_book("/tmp/b.txt")).unwrap();
    let chapters = vec![
        BookChapter {
            url: "/tmp/b.txt#0".into(),
            title: "第一章".into(),
            index: 0,
            start: 0,
            end: 100,
            ..Default::default()
        },
        BookChapter {
            url: "/tmp/b.txt#1".into(),
            title: "第二章".into(),
            index: 1,
            start: 100,
            end: 200,
            ..Default::default()
        },
    ];
    ChapterRepo::replace_all(&conn, "/tmp/b.txt", &chapters).unwrap();
    assert_eq!(ChapterRepo::count(&conn, "/tmp/b.txt").unwrap(), 2);

    BookRepo::delete(&conn, "/tmp/b.txt").unwrap();
    assert_eq!(ChapterRepo::count(&conn, "/tmp/b.txt").unwrap(), 0);
}

#[test]
fn group_bitmask_ops() {
    let conn = db::open_in_memory().unwrap();
    GroupRepo::upsert(
        &conn,
        &BookGroup {
            group_id: 2,
            group_name: "玄幻".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let mut group = 0u64;
    group = group_add(group, 2);
    assert!(group_has(group, 2));
    group = group_add(group, 8);
    group = group_remove(group, 2);
    assert!(!group_has(group, 2));
    assert!(group_has(group, 8));
    assert_eq!(GroupRepo::list(&conn).unwrap().len(), 1);
}

#[test]
fn keyvalue_roundtrip() {
    let conn = db::open_in_memory().unwrap();
    KeyValue::set(&conn, "themeMode", "2").unwrap();
    assert_eq!(
        KeyValue::get(&conn, "themeMode").unwrap().as_deref(),
        Some("2")
    );
    KeyValue::cache_set(&conn, "dedup:x", "1").unwrap();
    assert_eq!(
        KeyValue::cache_get(&conn, "dedup:x").unwrap().as_deref(),
        Some("1")
    );
}

#[test]
fn replace_rule_legacy_field_mapping_and_diff() {
    let json = r#"[
        {"id":1,"replaceSummary":"旧名","regex":"a+","useTo":"某书","enable":false,"serialNumber":5},
        {"id":2,"name":"新名","pattern":"b+","isEnabled":true,"sortOrder":1}
    ]"#;
    let existing: HashSet<i64> = [1i64].into_iter().collect();
    let (rules, report) = import_replace_rules(json, &existing).unwrap();

    assert_eq!(rules.len(), 2);
    let r1 = rules.iter().find(|r| r.id == 1).unwrap();
    assert_eq!(r1.name, "旧名");
    assert_eq!(r1.pattern, "a+");
    assert_eq!(r1.scope, "某书");
    assert!(!r1.is_enabled);
    assert_eq!(r1.sort_order, 5);

    assert_eq!(report.count(DiffStatus::Updated), 1);
    assert_eq!(report.count(DiffStatus::New), 1);
    assert_eq!(report.new_keys(), vec!["2"]);
}

#[test]
fn replace_rule_timeout_fallback() {
    let r = ReplaceRule {
        id: 1,
        pattern: "x".into(),
        timeout_millisecond: 0,
        ..Default::default()
    };
    assert_eq!(r.effective_timeout_ms(), 3000);
    assert_eq!(ReplaceRule::default().timeout_millisecond, 0);
}

#[test]
fn http_tts_missing_fields_default_to_now() {
    let json = r#"[{"name":"源","url":"http://x/{text}/{speed}"}]"#;
    let existing: HashSet<i64> = HashSet::new();
    let (rules, _report) = import_http_tts(json, &existing, 1700000000000).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, 1700000000000);
    assert_eq!(rules[0].last_update_time, 1700000000000);
}

#[test]
fn default_data_loads_26_rules() {
    let conn = db::open_in_memory().unwrap();
    let json = std::fs::read_to_string("resources/defaultData/txtTocRule.json").unwrap();
    let n = default_data::load_toc_rules_from_json(&conn, &json).unwrap();
    assert_eq!(n, 26);
    assert_eq!(TxtTocRuleRepo::count(&conn).unwrap(), 26);

    // 再次装载不重复插入。
    let n2 = default_data::load_toc_rules_from_json(&conn, &json).unwrap();
    assert_eq!(n2, 0);
}
