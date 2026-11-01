use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

use crate::rule::rule_analyzer::split_replace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Default,
    Css,
    XPath,
    Json,
    Regex,
    Js,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRule {
    pub raw: String,
    pub rule: String,
    pub mode: Mode,
    pub replace_regex: Option<String>,
    pub replacement: Option<String>,
    pub replace_first: bool,
    pub puts: Vec<(String, String)>,
}

impl SourceRule {
    pub fn parse(raw: &str, content_is_json: bool) -> Self {
        let raw = raw.trim().to_string();
        let (without_puts, puts) = split_puts(&raw);
        let (mode, stripped) = detect_mode(&without_puts, content_is_json);
        let replace = split_replace(&stripped);
        let body = replace.body;
        let mode = if mode != Mode::Js && mode != Mode::Regex && contains_dollar_ref(&body) {
            Mode::Regex
        } else {
            mode
        };

        Self {
            raw,
            rule: body,
            mode,
            replace_regex: replace.pattern,
            replacement: replace.replacement,
            replace_first: replace.first,
            puts,
        }
    }

    pub fn has_puts(&self) -> bool {
        !self.puts.is_empty()
    }
}

fn detect_mode(rule: &str, content_is_json: bool) -> (Mode, String) {
    let trimmed = rule.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("@css:") {
        return (Mode::Css, trimmed[5..].trim().to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("@@") {
        return (Mode::Default, rest.trim().to_string());
    }
    if lower.starts_with("@xpath:") {
        return (Mode::XPath, trimmed[7..].trim().to_string());
    }
    if lower.starts_with("@json:") {
        return (Mode::Json, trimmed[6..].trim().to_string());
    }
    if lower.starts_with("@js:") {
        return (Mode::Js, trimmed[4..].trim().to_string());
    }
    if lower.starts_with("<js>") {
        return (Mode::Js, trimmed.to_string());
    }
    if content_is_json || trimmed.starts_with("$.") || trimmed.starts_with("$[") {
        return (Mode::Json, trimmed.to_string());
    }
    if trimmed.starts_with('/') {
        return (Mode::XPath, trimmed.to_string());
    }
    if trimmed.starts_with(':') {
        return (Mode::Regex, trimmed.to_string());
    }
    (Mode::Default, trimmed.to_string())
}

fn contains_dollar_ref(text: &str) -> bool {
    let selector = text.split("##").next().unwrap_or(text);
    let bytes = selector.as_bytes();
    bytes
        .windows(2)
        .any(|pair| pair[0] == b'$' && pair[1].is_ascii_digit())
}

fn split_puts(rule: &str) -> (String, Vec<(String, String)>) {
    static PUT_RE: OnceLock<Regex> = OnceLock::new();
    let put_re =
        PUT_RE.get_or_init(|| Regex::new(r"(?i)@put:(\{[^}]+?\})").expect("valid @put regex"));

    let mut puts = Vec::new();
    let mut cleaned = rule.to_string();
    for captures in put_re.captures_iter(rule) {
        let Some(json) = captures.get(1) else {
            continue;
        };
        let normalized = quote_unquoted_keys(json.as_str());
        if let Ok(value) = serde_json::from_str::<Value>(&normalized) {
            if let Some(map) = value.as_object() {
                for (key, value) in map {
                    puts.push((key.clone(), value_to_string(value)));
                }
            }
        }
        cleaned = cleaned.replace(captures.get(0).map(|m| m.as_str()).unwrap_or(""), "");
    }
    (cleaned.trim().to_string(), puts)
}

fn quote_unquoted_keys(input: &str) -> String {
    static KEY_RE: OnceLock<Regex> = OnceLock::new();
    let key_re = KEY_RE.get_or_init(|| {
        Regex::new(r"([{,\s])([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("valid put key regex")
    });
    key_re.replace_all(input, "$1\"$2\":").to_string()
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_modes() {
        assert_eq!(SourceRule::parse("@css:.title@text", false).mode, Mode::Css);
        assert_eq!(
            SourceRule::parse("@@.title@text", false).mode,
            Mode::Default
        );
        assert_eq!(
            SourceRule::parse("@XPath://div[@id='x']/text()", false).mode,
            Mode::XPath
        );
        assert_eq!(SourceRule::parse("@Json:$.data[*]", false).mode, Mode::Json);
        assert_eq!(SourceRule::parse("$.data[*]", false).mode, Mode::Json);
        assert_eq!(SourceRule::parse("/html/body/div", false).mode, Mode::XPath);
        assert_eq!(
            SourceRule::parse(":href=\"(\\d+)\"", false).mode,
            Mode::Regex
        );
        assert_eq!(SourceRule::parse("@js:result + '!'", false).mode, Mode::Js);
        assert_eq!(SourceRule::parse("<js>result</js>", false).mode, Mode::Js);
    }

    #[test]
    fn detects_regex_from_dollar_reference() {
        let rule = SourceRule::parse("class.book##href=\"(\\d+)\"##$1", false);
        assert_eq!(rule.mode, Mode::Default);
        assert_eq!(rule.replace_regex.as_deref(), Some("href=\"(\\d+)\""));
        assert_eq!(rule.replacement.as_deref(), Some("$1"));

        let rule = SourceRule::parse(":href=\"(\\d+)\"##$1", false);
        assert_eq!(rule.mode, Mode::Regex);
    }

    #[test]
    fn parses_put_rules() {
        let rule = SourceRule::parse(r#"class.title@text@put:{bid:"$.id", page:1}"#, false);
        assert_eq!(rule.puts.len(), 2);
        assert!(rule
            .puts
            .iter()
            .any(|(key, value)| key == "bid" && value == "$.id"));
        assert!(rule
            .puts
            .iter()
            .any(|(key, value)| key == "page" && value == "1"));
        assert_eq!(rule.rule, "class.title@text");
    }

    #[test]
    fn parses_replace_first() {
        let rule = SourceRule::parse("class.title##\\s+##-###", false);
        assert_eq!(rule.rule, "class.title");
        assert_eq!(rule.replace_regex.as_deref(), Some("\\s+"));
        assert_eq!(rule.replacement.as_deref(), Some("-"));
        assert!(rule.replace_first);
    }
}
