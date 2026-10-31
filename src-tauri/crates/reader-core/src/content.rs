use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

use crate::domain::chinese_convert;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceRule {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub replacement: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub exclude_scope: String,
    #[serde(default)]
    pub scope_title: bool,
    #[serde(default = "default_true")]
    pub scope_content: bool,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default = "default_true")]
    pub is_regex: bool,
    #[serde(default)]
    pub timeout_millisecond: i64,
    #[serde(default)]
    pub sort_order: i32,
}

impl Default for ReplaceRule {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            group: String::new(),
            pattern: String::new(),
            replacement: String::new(),
            scope: String::new(),
            exclude_scope: String::new(),
            scope_title: false,
            scope_content: true,
            is_enabled: true,
            is_regex: true,
            timeout_millisecond: 0,
            sort_order: 0,
        }
    }
}

impl ReplaceRule {
    pub fn effective_timeout_ms(&self) -> i64 {
        if self.timeout_millisecond <= 0 {
            3000
        } else {
            self.timeout_millisecond
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRequest {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub book_name: String,
    #[serde(default)]
    pub book_origin: String,
    #[serde(default)]
    pub rules: Vec<ReplaceRule>,
    #[serde(default)]
    pub include_title: bool,
    #[serde(default = "default_true")]
    pub use_replace: bool,
    #[serde(default = "default_true")]
    pub use_replace_rule: bool,
    #[serde(default = "default_true")]
    pub chinese_convert: bool,
    #[serde(default)]
    pub chinese_converter_type: i32,
    #[serde(default = "default_true")]
    pub re_segment: bool,
    #[serde(default)]
    pub keep_duplicate_title: bool,
    #[serde(default)]
    pub paragraph_indent: String,
    #[serde(default)]
    pub is_android8: bool,
}

impl Default for ProcessRequest {
    fn default() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            book_name: String::new(),
            book_origin: String::new(),
            rules: Vec::new(),
            include_title: false,
            use_replace: true,
            use_replace_rule: true,
            chinese_convert: true,
            chinese_converter_type: 0,
            re_segment: true,
            keep_duplicate_title: false,
            paragraph_indent: "\u{3000}\u{3000}".to_string(),
            is_android8: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedContent {
    pub display_title: String,
    pub same_title_removed: bool,
    pub contents: Vec<String>,
    pub effective_rules: Vec<ReplaceRule>,
}

fn default_true() -> bool {
    true
}

fn scope_matches(rule: &ReplaceRule, book_name: &str, book_origin: &str) -> bool {
    let scope = rule.scope.trim();
    let scope_hit =
        scope.contains(book_name) || (!book_origin.is_empty() && scope.contains(book_origin));
    if !scope.is_empty() && !scope_hit {
        return false;
    }
    let exclude = rule.exclude_scope.trim();
    if !exclude.is_empty()
        && (exclude.contains(book_name)
            || (!book_origin.is_empty() && exclude.contains(book_origin)))
    {
        return false;
    }
    true
}

fn selected_rules<'a>(
    rules: &'a [ReplaceRule],
    book_name: &str,
    book_origin: &str,
    for_title: bool,
) -> Vec<&'a ReplaceRule> {
    let mut selected = rules
        .iter()
        .filter(|rule| {
            rule.is_enabled
                && !rule.pattern.is_empty()
                && if for_title {
                    rule.scope_title
                } else {
                    rule.scope_content
                }
                && scope_matches(rule, book_name, book_origin)
        })
        .collect::<Vec<_>>();
    selected.sort_by_key(|rule| rule.sort_order);
    selected
}

fn escape_title(value: &str) -> String {
    let escaped = fancy_regex::escape(value);
    Regex::new(r"\s+")
        .ok()
        .map(|regex| regex.replace_all(&escaped, r"\s*").into_owned())
        .unwrap_or(escaped.to_string())
}

fn title_matcher(book_name: &str, title: &str) -> Option<Regex> {
    let pattern = format!(
        r"^(\s|\p{{P}}|{})*{}(\s)*",
        fancy_regex::escape(book_name),
        escape_title(title)
    );
    Regex::new(&pattern).ok()
}

fn strip_duplicate_title(
    title: &str,
    content: &str,
    book_name: &str,
    title_rules: &[ReplaceRule],
    use_replace: bool,
) -> (bool, String) {
    let title = title.trim();
    if title.is_empty() {
        return (false, content.to_string());
    }
    if let Some(regex) = title_matcher(book_name, title) {
        if let Ok(Some(found)) = regex.find(content) {
            if found.start() == 0 {
                return (true, content[found.end()..].to_string());
            }
        }
    }
    if use_replace {
        let display = display_title_with_rules(title, title_rules, false, 0);
        if !display.is_empty() && display != title {
            if let Some(regex) = title_matcher(book_name, &display) {
                if let Ok(Some(found)) = regex.find(content) {
                    if found.start() == 0 {
                        return (true, content[found.end()..].to_string());
                    }
                }
            }
        }
    }
    (false, content.to_string())
}

pub fn display_title(
    title: &str,
    rules: &[ReplaceRule],
    book_name: &str,
    book_origin: &str,
    use_replace: bool,
    chinese_convert_enabled: bool,
    direction: i32,
) -> String {
    let title_rules = if use_replace {
        selected_rules(rules, book_name, book_origin, true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    display_title_with_rules(title, &title_rules, chinese_convert_enabled, direction)
}

pub fn display_title_with_rules(
    title: &str,
    rules: &[ReplaceRule],
    chinese_convert_enabled: bool,
    direction: i32,
) -> String {
    let mut display = title.replace(['\r', '\n'], "");
    if chinese_convert_enabled && chinese_convert::is_valid_direction(direction) {
        display = chinese_convert::convert(&display, direction);
    }
    for rule in rules {
        if rule.pattern.is_empty() {
            continue;
        }
        let next = if rule.is_regex {
            match Regex::new(&rule.pattern) {
                Ok(regex) => regex
                    .replace_all(&display, rule.replacement.as_str())
                    .into_owned(),
                Err(_) => continue,
            }
        } else {
            display.replace(&rule.pattern, &rule.replacement)
        };
        if !next.trim().is_empty() {
            display = next;
        }
    }
    display
}

fn apply_content_rules(
    text: &str,
    rules: &[ReplaceRule],
    book_name: &str,
    book_origin: &str,
) -> (String, Vec<ReplaceRule>) {
    let mut current = text.to_string();
    let mut effective = Vec::new();
    for rule in selected_rules(rules, book_name, book_origin, false) {
        let next = if rule.is_regex {
            match Regex::new(&rule.pattern) {
                Ok(regex) => regex
                    .replace_all(&current, rule.replacement.as_str())
                    .into_owned(),
                Err(_) => continue,
            }
        } else {
            current.replace(&rule.pattern, &rule.replacement)
        };
        if next != current {
            current = next;
            effective.push(rule.clone());
        }
    }
    (current, effective)
}

pub fn process(request: &ProcessRequest) -> ProcessedContent {
    let mut text = request.content.clone();
    let mut same_title_removed = false;
    let display_title = display_title(
        &request.title,
        &request.rules,
        &request.book_name,
        &request.book_origin,
        request.use_replace && request.use_replace_rule,
        request.chinese_convert,
        request.chinese_converter_type,
    );
    if text != "null" {
        if !request.keep_duplicate_title {
            let title_rules = selected_rules(
                &request.rules,
                &request.book_name,
                &request.book_origin,
                true,
            );
            let (removed, next) = strip_duplicate_title(
                &request.title,
                &text,
                &request.book_name,
                &title_rules
                    .into_iter()
                    .cloned()
                    .collect::<Vec<ReplaceRule>>(),
                request.use_replace && request.use_replace_rule,
            );
            same_title_removed = removed;
            text = next;
        }

        if request.re_segment {
            text = crate::resegment::re_segment(&text, &request.title);
        }

        if request.chinese_convert
            && chinese_convert::is_valid_direction(request.chinese_converter_type)
        {
            text = chinese_convert::convert(&text, request.chinese_converter_type);
        }

        if request.use_replace && request.use_replace_rule {
            let normalized = text.lines().map(str::trim).collect::<Vec<_>>().join("\n");
            let (next, effective) = apply_content_rules(
                &normalized,
                &request.rules,
                &request.book_name,
                &request.book_origin,
            );
            text = next;
            let mut contents = Vec::new();
            if request.include_title && !display_title.trim().is_empty() {
                text = format!("{display_title}\n{text}");
            }
            contents.extend(paragraphs(
                &text,
                request.include_title,
                &request.paragraph_indent,
                request.is_android8,
            ));
            return ProcessedContent {
                display_title,
                same_title_removed,
                contents,
                effective_rules: effective,
            };
        }
    }

    if request.include_title && !display_title.trim().is_empty() {
        text = format!("{display_title}\n{text}");
    }

    if request.is_android8 {
        text = text.replace('\u{00a0}', " ");
    }

    ProcessedContent {
        display_title,
        same_title_removed,
        contents: paragraphs(
            &text,
            request.include_title,
            &request.paragraph_indent,
            false,
        ),
        effective_rules: Vec::new(),
    }
}

fn paragraphs(text: &str, include_title: bool, indent: &str, android8: bool) -> Vec<String> {
    let mut contents = Vec::new();
    for raw in text.split('\n') {
        let paragraph = raw.trim_matches(|c: char| c as u32 <= 0x20 || c == '\u{3000}');
        if paragraph.is_empty() {
            continue;
        }
        if contents.is_empty() && include_title {
            contents.push(paragraph.to_string());
        } else {
            contents.push(format!("{indent}{paragraph}"));
        }
    }
    if android8 {
        for item in &mut contents {
            *item = item.replace('\u{00a0}', " ");
        }
    }
    contents
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pattern: &str, replacement: &str) -> ReplaceRule {
        ReplaceRule {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn applies_plain_and_regex_rules_in_order() {
        let request = ProcessRequest {
            title: "第一章".into(),
            content: "第一章\nabc 123".into(),
            book_name: "测试书".into(),
            rules: vec![rule(r"abc", "xyz"), rule(r"(\d+)", "[$1]")],
            include_title: true,
            re_segment: false,
            ..Default::default()
        };
        let result = process(&request);
        assert!(result.same_title_removed);
        assert_eq!(result.contents[0], "第一章");
        assert_eq!(result.contents[1], "\u{3000}\u{3000}xyz [123]");
        assert_eq!(result.effective_rules.len(), 2);
    }

    #[test]
    fn excludes_scope_and_respects_title_only_rules() {
        let mut title_rule = rule("第一章", "序章");
        title_rule.scope_title = true;
        title_rule.scope_content = false;
        let request = ProcessRequest {
            title: "第一章".into(),
            content: "正文".into(),
            book_name: "测试书".into(),
            rules: vec![title_rule],
            include_title: true,
            re_segment: false,
            ..Default::default()
        };
        let result = process(&request);
        assert_eq!(result.contents[0], "序章");
        assert!(result.effective_rules.is_empty());
    }

    #[test]
    fn scope_matches_substrings() {
        let mut rule = rule("a", "b");
        rule.scope = "测试书,其他".into();
        assert!(scope_matches(&rule, "测试书", ""));
        rule.scope = "不存在".into();
        assert!(!scope_matches(&rule, "测试书", ""));
        rule.scope.clear();
        rule.exclude_scope = "测试书".into();
        assert!(!scope_matches(&rule, "测试书", ""));
    }
}
