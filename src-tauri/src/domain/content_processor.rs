use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use fancy_regex::Regex;
use rusqlite::Connection;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::ReplaceRuleRepo;
use crate::domain::chinese_convert;
use crate::domain::model::{Book, ReplaceRule};
use crate::domain::regex_guard::{self, GuardOutcome};
use crate::{Error, Result};

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookContent {
    pub same_title_removed: bool,
    pub contents: Vec<String>,
    pub effective_rules: Vec<ReplaceRule>,
}

fn content_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn scope_tokens(scope: &str) -> Vec<&str> {
    scope
        .split(|c| c == ',' || c == '，')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

fn scope_matches(scope: &str, book: &Book) -> bool {
    if scope.trim().is_empty() {
        return true;
    }
    scope_tokens(scope)
        .iter()
        .any(|token| book.name.contains(*token) || book.origin_name.contains(*token))
}

fn rule_applies(rule: &ReplaceRule, book: &Book) -> bool {
    if !rule.is_enabled {
        return false;
    }
    if !scope_matches(&rule.scope, book) {
        return false;
    }
    if !rule.exclude_scope.trim().is_empty()
        && scope_tokens(&rule.exclude_scope)
            .iter()
            .any(|token| book.name.contains(*token) || book.origin_name.contains(*token))
    {
        return false;
    }
    true
}

fn strip_duplicate_title(title: &str, content: &str) -> (bool, String) {
    let title = title.trim();
    if title.is_empty() {
        return (false, content.to_string());
    }
    let pattern = format!(
        r"^[\s\p{{P}}\p{{S}}]*{}[\s\p{{P}}]*(?:\r?\n)?",
        fancy_regex::escape(title)
    );
    if let Ok(regex) = Regex::new(&pattern) {
        if let Ok(Some(found)) = regex.find(content) {
            if found.start() == 0 {
                return (true, content[found.end()..].to_string());
            }
        }
    }
    let mut lines = content.split_inclusive('\n');
    if let Some(first) = lines.next() {
        if first.trim() == title {
            return (true, lines.collect::<String>());
        }
    }
    (false, content.to_string())
}

fn re_segment(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let mut buf = String::new();
        let mut len = 0usize;
        for ch in line.chars() {
            buf.push(ch);
            len += 1;
            let at_break = matches!(ch, '。' | '！' | '？' | '!' | '?' | '；' | ';');
            if (at_break && len >= 40) || len >= 120 {
                out.push(buf.trim().to_string());
                buf.clear();
                len = 0;
            }
        }
        if !buf.trim().is_empty() {
            out.push(buf.trim().to_string());
        }
    }
    out.join("\n")
}

fn apply_rules(
    conn: &Connection,
    text: &str,
    book: &Book,
    for_title: bool,
) -> Result<(String, Vec<ReplaceRule>)> {
    let rules = ReplaceRuleRepo::list(conn)?;
    let mut current = text.to_string();
    let mut effective = Vec::new();
    for rule in rules {
        if !rule_applies(&rule, book) {
            continue;
        }
        let track_on = if for_title {
            rule.scope_title
        } else {
            rule.scope_content
        };
        if !track_on {
            continue;
        }

        if !rule.is_regex {
            if !rule.pattern.is_empty() && current.contains(&rule.pattern) {
                let updated = current.replace(&rule.pattern, &rule.replacement);
                if updated != current {
                    current = updated;
                    effective.push(rule);
                }
            }
            continue;
        }

        let pattern = rule.pattern.clone();
        let replacement = rule.replacement.clone();
        let input = current.clone();
        let timeout = Duration::from_millis(rule.effective_timeout_ms() as u64);
        let outcome = regex_guard::run_with_timeout(timeout, move || {
            let regex = match Regex::new(&pattern) {
                Ok(r) => r,
                Err(e) => return Err(e.to_string()),
            };
            let updated = regex.replace_all(&input, replacement.as_str()).into_owned();
            Ok(updated)
        });

        match outcome {
            GuardOutcome::Ok(Ok(updated)) => {
                if updated != current {
                    current = updated;
                    effective.push(rule);
                }
            }
            GuardOutcome::Ok(Err(_)) => {}
            GuardOutcome::Timeout => {
                let mut disabled = rule.clone();
                disabled.is_enabled = false;
                ReplaceRuleRepo::upsert(conn, &disabled)?;
                current.push_str(&format!(
                    "\n【替换规则「{}」执行超时，已自动禁用】",
                    rule.name
                ));
            }
            GuardOutcome::Panicked => {}
        }
    }
    Ok((current, effective))
}

pub fn display_title(conn: &Connection, title: &str, book: &Book) -> Result<String> {
    let (processed, _) = apply_rules(conn, title, book, true)?;
    Ok(processed.trim().to_string())
}

pub fn process(
    conn: &Connection,
    title: &str,
    content: &str,
    book: &Book,
    include_title: bool,
) -> Result<BookContent> {
    let key = format!(
        "dedup:{}:{}:{}",
        book.book_url,
        title,
        content_hash(content)
    );
    let (same_title_removed, deduped) = match KeyValue::cache_get(conn, &key)? {
        Some(cached) => (true, cached),
        None => {
            let (removed, value) = strip_duplicate_title(title, content);
            if removed {
                KeyValue::cache_set(conn, &key, &value)?;
            }
            (removed, value)
        }
    };

    let mut text = deduped;
    if book
        .read_config
        .as_ref()
        .map(|c| c.re_segment)
        .unwrap_or(false)
    {
        text = re_segment(&text);
    }

    let direction = KeyValue::get(conn, "chineseConverterType")?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(chinese_convert::DIRECTION_NONE);
    if chinese_convert::is_valid_direction(direction) {
        text = chinese_convert::convert(&text, direction);
    }

    let normalized: String = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let (processed, mut effective) = apply_rules(conn, &normalized, book, false)?;

    let mut contents: Vec<String> = Vec::new();
    if include_title {
        let display = display_title(conn, title, book)?;
        if !display.is_empty() {
            contents.push(format!("\u{3000}\u{3000}{display}"));
        }
    }
    contents.extend(
        processed
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| format!("\u{3000}\u{3000}{line}")),
    );

    effective.sort_by_key(|r| r.sort_order);
    Ok(BookContent {
        same_title_removed,
        contents,
        effective_rules: effective,
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
    fn strip_title_only_once() {
        let (removed, value) = strip_duplicate_title("第一章", "第一章\n正文");
        assert!(removed);
        assert_eq!(value, "正文");
    }

    #[test]
    fn scope_matching_is_substring() {
        let mut book = Book::default();
        book.name = "斗破苍穹".into();
        book.origin_name = "book.txt".into();
        assert!(scope_matches("", &book));
        assert!(scope_matches("斗破,完美", &book));
        assert!(!scope_matches("完美世界", &book));
        assert!(scope_matches("book", &book));
    }

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
