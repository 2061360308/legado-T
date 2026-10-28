use local_reader_lib::data::db;
use local_reader_lib::data::repositories::ReplaceRuleRepo;
use local_reader_lib::domain::content_processor;
use local_reader_lib::domain::model::{book_type, Book, ReplaceRule};

fn book() -> Book {
    Book {
        book_url: "/tmp/content.txt".into(),
        name: "测试书".into(),
        author: "作者".into(),
        origin: "local".into(),
        origin_name: "content.txt".into(),
        r#type: book_type::TEXT | book_type::LOCAL,
        ..Default::default()
    }
}

fn rule(id: i64, pattern: &str, replacement: &str, sort: i32) -> ReplaceRule {
    ReplaceRule {
        id,
        name: format!("规则{id}"),
        pattern: pattern.into(),
        replacement: replacement.into(),
        scope_content: true,
        is_enabled: true,
        is_regex: false,
        sort_order: sort,
        ..Default::default()
    }
}

#[test]
fn removes_title_and_applies_ordered_rules() {
    let conn = db::open_in_memory().unwrap();
    ReplaceRuleRepo::upsert(&conn, &rule(1, "内容", "文本", 2)).unwrap();
    ReplaceRuleRepo::upsert(&conn, &rule(2, "正文", "内容", 1)).unwrap();

    let result = content_processor::process(
        &conn,
        "第一章 开始",
        "第一章 开始\n正文一\n正文二",
        &book(),
        false,
    )
    .unwrap();

    assert!(result.same_title_removed);
    assert_eq!(result.contents, vec!["　　文本一", "　　文本二"]);
    assert_eq!(result.effective_rules.len(), 2);
    assert_eq!(result.effective_rules[0].id, 2);
    assert_eq!(result.effective_rules[1].id, 1);
}

#[test]
fn disabled_and_excluded_rules_are_skipped() {
    let conn = db::open_in_memory().unwrap();
    let mut disabled = rule(1, "正文", "内容", 0);
    disabled.is_enabled = false;
    ReplaceRuleRepo::upsert(&conn, &disabled).unwrap();
    let mut excluded = rule(2, "正文", "内容", 0);
    excluded.exclude_scope = "测试书".into();
    ReplaceRuleRepo::upsert(&conn, &excluded).unwrap();

    let result = content_processor::process(&conn, "标题", "正文", &book(), false).unwrap();
    assert_eq!(result.contents, vec!["　　正文"]);
    assert!(result.effective_rules.is_empty());
}

#[test]
fn title_rules_apply_to_display_title() {
    let conn = db::open_in_memory().unwrap();
    let mut title_rule = rule(1, "开始", "启程", 0);
    title_rule.scope_title = true;
    title_rule.scope_content = false;
    ReplaceRuleRepo::upsert(&conn, &title_rule).unwrap();

    let title = content_processor::display_title(&conn, "第一章 开始", &book()).unwrap();
    assert_eq!(title, "第一章 启程");
}

#[test]
fn dedup_cache_reuses_removed_content() {
    let conn = db::open_in_memory().unwrap();
    let first = content_processor::process(&conn, "标题", "标题\n正文", &book(), false).unwrap();
    let second = content_processor::process(&conn, "标题", "标题\n正文", &book(), false).unwrap();
    assert_eq!(first.contents, second.contents);
    assert!(second.same_title_removed);
}
