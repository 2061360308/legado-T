use std::collections::HashMap;

use rusqlite::Connection;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::ReplaceRuleRepo;
use crate::domain::model::{Book, ReplaceRule};
use crate::{Error, Result};

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookContent {
    pub same_title_removed: bool,
    pub contents: Vec<String>,
    pub effective_rules: Vec<ReplaceRule>,
}

pub fn core_rules(conn: &Connection) -> Result<Vec<reader_core::content::ReplaceRule>> {
    Ok(ReplaceRuleRepo::list(conn)?
        .iter()
        .map(to_core_rule)
        .collect())
}

pub fn native_rules(conn: &Connection) -> Result<HashMap<i64, ReplaceRule>> {
    Ok(ReplaceRuleRepo::list(conn)?
        .into_iter()
        .map(|rule| (rule.id, rule))
        .collect())
}

fn to_core_rule(rule: &ReplaceRule) -> reader_core::content::ReplaceRule {
    reader_core::content::ReplaceRule {
        id: rule.id,
        name: rule.name.clone(),
        group: rule.group.clone(),
        pattern: rule.pattern.clone(),
        replacement: rule.replacement.clone(),
        scope: rule.scope.clone(),
        exclude_scope: rule.exclude_scope.clone(),
        scope_title: rule.scope_title,
        scope_content: rule.scope_content,
        is_enabled: rule.is_enabled,
        is_regex: rule.is_regex,
        timeout_millisecond: rule.timeout_millisecond as i64,
        sort_order: rule.sort_order,
    }
}

pub fn converter_direction(conn: &Connection) -> i32 {
    KeyValue::get(conn, "chineseConverterType")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0)
}

pub fn display_title(conn: &Connection, title: &str, book: &Book) -> Result<String> {
    let rules = ReplaceRuleRepo::list(conn)?;
    let core_rules = rules.iter().map(to_core_rule).collect::<Vec<_>>();
    let use_replace = book
        .read_config
        .as_ref()
        .and_then(|config| config.use_replace_rule)
        .unwrap_or(true);
    Ok(reader_core::content::display_title(
        title,
        &core_rules,
        &book.name,
        &book.origin_name,
        use_replace,
        true,
        converter_direction(conn),
    )
    .trim()
    .to_string())
}

pub fn process(
    conn: &Connection,
    title: &str,
    content: &str,
    book: &Book,
    include_title: bool,
) -> Result<BookContent> {
    process_with_replace(conn, title, content, book, include_title, true)
}

pub fn process_with_replace(
    conn: &Connection,
    title: &str,
    content: &str,
    book: &Book,
    include_title: bool,
    use_replace: bool,
) -> Result<BookContent> {
    let rules = ReplaceRuleRepo::list(conn)?;
    let core_rules = rules.iter().map(to_core_rule).collect::<Vec<_>>();
    let use_replace_rule = book
        .read_config
        .as_ref()
        .and_then(|config| config.use_replace_rule)
        .unwrap_or(true);
    let re_segment = book
        .read_config
        .as_ref()
        .map(|config| config.re_segment)
        .unwrap_or(false);
    let keep_duplicate_title = KeyValue::get(conn, &format!("keepDuplicateTitle:{}", book.book_url))?
        .map(|value| value == "true")
        .unwrap_or(false);
    let request = reader_core::content::ProcessRequest {
        title: title.to_string(),
        content: content.to_string(),
        book_name: book.name.clone(),
        book_origin: book.origin_name.clone(),
        rules: core_rules,
        include_title,
        use_replace,
        use_replace_rule,
        chinese_convert: true,
        chinese_converter_type: converter_direction(conn),
        re_segment,
        keep_duplicate_title,
        paragraph_indent: "\u{3000}\u{3000}".to_string(),
        is_android8: false,
    };
    let result = reader_core::content::process(&request);
    let by_id = rules
        .into_iter()
        .map(|rule| (rule.id, rule))
        .collect::<HashMap<_, _>>();
    let effective_rules = result
        .effective_rules
        .iter()
        .filter_map(|rule| by_id.get(&rule.id).cloned())
        .collect();
    Ok(BookContent {
        same_title_removed: result.same_title_removed,
        contents: result.contents,
        effective_rules,
    })
}

pub fn validate_pattern(rule: &ReplaceRule) -> Result<()> {
    if rule.is_regex && rule.pattern.ends_with('|') && !rule.pattern.ends_with("\\|") {
        return Err(Error::Invalid("正则不能以未转义竖线结尾".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_trailing_pipe() {
        let rule = ReplaceRule {
            pattern: "abc|".into(),
            is_regex: true,
            ..Default::default()
        };
        assert!(validate_pattern(&rule).is_err());
    }
}
