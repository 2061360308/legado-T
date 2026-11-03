use engine_model::{BookSource, JsEvaluator, JsRequest, Source};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExploreCategory {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Value>,
}

impl ExploreCategory {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            title: format!("ERROR:{}", message.into()),
            url: String::new(),
            style: None,
        }
    }
}

pub async fn resolve_explore_categories(
    source: &BookSource,
    raw: &str,
    js: &dyn JsEvaluator,
) -> Vec<ExploreCategory> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    let text = if let Some(script) = js_script(raw) {
        let request = JsRequest {
            script: &script,
            source: Some(source as &dyn Source),
            base_url: &source.book_source_url,
            page: Some(1),
            key: None,
            result: None,
            result_bytes: None,
            src: None,
            response: None,
            book: None,
            chapter: None,
            variables: None,
        };
        match js.eval(request).await {
            Ok(text) => text,
            Err(error) => return vec![ExploreCategory::error(error.to_string())],
        }
    } else {
        raw.to_string()
    };
    parse_categories(&text)
}

fn js_script(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("@js:") {
        return Some(trimmed[4..].trim().to_string());
    }
    if lower.starts_with("<js") {
        let start = trimmed.find('>')?;
        let end = lower
            .rfind("</js>")
            .filter(|end| *end > start)
            .unwrap_or(trimmed.len());
        return Some(trimmed[start + 1..end].trim().to_string());
    }
    None
}

fn parse_categories(text: &str) -> Vec<ExploreCategory> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if matches!(value, Value::Array(_) | Value::Object(_)) {
            return categories_from_value(value);
        }
        let categories = categories_from_value(value);
        if !categories.is_empty() {
            return categories;
        }
    }
    let categories = categories_from_text(text);
    if !categories.is_empty() {
        return categories;
    }
    vec![ExploreCategory {
        title: text.to_string(),
        url: String::new(),
        style: None,
    }]
}

fn categories_from_value(value: Value) -> Vec<ExploreCategory> {
    match value {
        Value::Array(items) => items
            .into_iter()
            .flat_map(categories_from_item)
            .collect(),
        Value::Object(object) => category_from_object(object).into_iter().collect(),
        Value::String(text) => categories_from_text(&text),
        _ => Vec::new(),
    }
}

fn categories_from_item(value: Value) -> Vec<ExploreCategory> {
    match value {
        Value::Object(object) => category_from_object(object).into_iter().collect(),
        Value::Array(items) => {
            let title = items.first().and_then(Value::as_str).unwrap_or("");
            let url = items.get(1).and_then(Value::as_str).unwrap_or("");
            category(title, url, None).into_iter().collect()
        }
        Value::String(text) => categories_from_text(&text),
        _ => Vec::new(),
    }
}

fn category_from_object(mut object: serde_json::Map<String, Value>) -> Option<ExploreCategory> {
    let title = object
        .get("title")
        .or_else(|| object.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let url = object
        .get("url")
        .or_else(|| object.get("link"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let style = object.remove("style");
    category(&title, &url, style)
}

fn categories_from_text(text: &str) -> Vec<ExploreCategory> {
    let mut out = Vec::new();
    for part in text.split("&&") {
        for line in part.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (title, url) = match line.split_once("::") {
                Some((title, url)) => (title.trim(), url.trim()),
                None => (line, ""),
            };
            if let Some(category) = category(title, url, None) {
                out.push(category);
            }
        }
    }
    out
}

fn category(title: &str, url: &str, style: Option<Value>) -> Option<ExploreCategory> {
    let title = title.trim();
    let url = url.trim();
    if title.is_empty() && url.is_empty() {
        return None;
    }
    Some(ExploreCategory {
        title: if title.is_empty() {
            url.to_string()
        } else {
            title.to_string()
        },
        url: url.to_string(),
        style,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use engine_model::{BoxFuture, Error};
    use futures::executor::block_on;
    use serde_json::json;

    use super::*;

    struct MockJs {
        output: Result<String, String>,
        expected_script: Option<String>,
        calls: AtomicUsize,
    }

    impl MockJs {
        fn output(text: &str) -> Self {
            Self {
                output: Ok(text.to_string()),
                expected_script: None,
                calls: AtomicUsize::new(0),
            }
        }

        fn failing(message: &str) -> Self {
            Self {
                output: Err(message.to_string()),
                expected_script: None,
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }

    impl JsEvaluator for MockJs {
        fn eval<'a>(
            &'a self,
            request: JsRequest<'a>,
        ) -> BoxFuture<'a, engine_model::Result<String>> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if let Some(expected) = &self.expected_script {
                assert_eq!(request.script, expected);
            }
            let output = self.output.clone();
            Box::pin(async move { output.map_err(Error::Js) })
        }
    }

    struct PanicJs;

    impl JsEvaluator for PanicJs {
        fn eval<'a>(
            &'a self,
            _request: JsRequest<'a>,
        ) -> BoxFuture<'a, engine_model::Result<String>> {
            panic!("plain text must not be evaluated")
        }
    }

    fn source() -> BookSource {
        serde_json::from_value(json!({
            "bookSourceUrl": "https://example.com",
            "bookSourceName": "test"
        }))
        .unwrap()
    }

    #[test]
    fn parses_plain_title_url_without_js() {
        let categories = block_on(resolve_explore_categories(
            &source(),
            "其它小说::http://m.niliuxs.org/sort-1-1/",
            &PanicJs,
        ));
        assert_eq!(
            categories,
            vec![ExploreCategory {
                title: "其它小说".into(),
                url: "http://m.niliuxs.org/sort-1-1/".into(),
                style: None,
            }]
        );
    }

    #[test]
    fn parses_multiline_and_double_ampersand_rules() {
        let categories = block_on(resolve_explore_categories(
            &source(),
            "玄幻::/a\n奇幻::/b&&武侠::/c",
            &PanicJs,
        ));
        assert_eq!(
            categories,
            vec![
                ExploreCategory {
                    title: "玄幻".into(),
                    url: "/a".into(),
                    style: None,
                },
                ExploreCategory {
                    title: "奇幻".into(),
                    url: "/b".into(),
                    style: None,
                },
                ExploreCategory {
                    title: "武侠".into(),
                    url: "/c".into(),
                    style: None,
                },
            ]
        );
    }

    #[test]
    fn parses_json_string_categories() {
        let categories = block_on(resolve_explore_categories(
            &source(),
            r#""玄幻::/a\n奇幻::/b""#,
            &PanicJs,
        ));
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[1].title, "奇幻");
    }

    #[test]
    fn evaluates_at_js_marker() {
        let mut js = MockJs::output("玄幻::/a\n奇幻::/b");
        js.expected_script = Some("return 1".into());
        let categories = block_on(resolve_explore_categories(
            &source(),
            "@JS: return 1",
            &js,
        ));
        assert_eq!(js.calls(), 1);
        assert_eq!(categories.len(), 2);
    }

    #[test]
    fn evaluates_js_tag_marker() {
        let mut js = MockJs::output("玄幻::/a");
        js.expected_script = Some("return 2".into());
        let categories = block_on(resolve_explore_categories(
            &source(),
            "<js>return 2</js>",
            &js,
        ));
        assert_eq!(js.calls(), 1);
        assert_eq!(categories[0].title, "玄幻");
    }

    #[test]
    fn converts_js_errors_to_error_category() {
        let categories = block_on(resolve_explore_categories(
            &source(),
            "@js: throw new Error('boom')",
            &MockJs::failing("boom"),
        ));
        assert_eq!(categories[0].title, "ERROR:JS 执行失败: boom");
        assert_eq!(categories[0].url, "");
    }

    #[test]
    fn plain_text_with_arrow_is_not_evaluated() {
        let categories = block_on(resolve_explore_categories(
            &source(),
            "a => b",
            &PanicJs,
        ));
        assert_eq!(categories[0].title, "a => b");
        assert_eq!(categories[0].url, "");
    }
}
