use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use engine_native::parse_sources;
use engine_native::upstream::{
    book::{search_book_to_book, BookEngine, PageFetcher},
    core::{
        Book, BookChapter, BoxFuture, Bytes, HeaderMap, HttpResponse, Result as CoreResult,
        StatusCode,
    },
    js::JsEngine,
    net::{absolute_url, UrlRule},
    rule::RuleEngine,
};
use serde_json::{json, Value};

const SOURCE: &str = r#"[
  {
    "bookSourceName": "Mock",
    "bookSourceUrl": "https://example.com",
    "bookSourceType": 0,
    "enabled": true,
    "searchUrl": "https://example.com/search?q={{key}}",
    "ruleSearch": {
      "bookList": "class.item",
      "name": "class.name@text",
      "author": "class.author@text",
      "intro": "class.intro@text",
      "bookUrl": "class.name@href"
    },
    "ruleBookInfo": {
      "name": "class.title@text",
      "author": "class.author@text",
      "intro": "class.intro@text",
      "tocUrl": "class.toc@href",
      "downloadUrls": "class.download@href"
    },
    "ruleToc": {
      "chapterList": "class.chapter",
      "chapterName": "tag.a@text",
      "chapterUrl": "tag.a@href"
    },
    "ruleContent": {
      "content": "class.content@html"
    }
  }
]"#;

const SEARCH_HTML: &str = r#"
<div class="item">
  <a class="name" href="/book/1">Book</a>
  <span class="author">Author</span>
  <p class="intro">Intro</p>
  <a class="toc" href="/book/1/toc">TOC</a>
</div>
"#;

const INFO_HTML: &str = r#"
<div class="title">Book</div>
<div class="author">Author</div>
<p class="intro">Intro</p>
<a class="toc" href="/book/1/toc">TOC</a>
<a class="download" href="/book/1.txt">TXT</a>
"#;

const TOC_HTML: &str = r#"
<div class="chapter"><a href="/chapter/1">Chapter 1</a></div>
"#;

const CONTENT_HTML: &str = r#"<div class="content"><p>Hello</p></div>"#;

struct MockFetcher;

impl PageFetcher for MockFetcher {
    fn fetch_rule<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, CoreResult<HttpResponse>> {
        let url = rule.url();
        Box::pin(async move {
            let text = if url.contains("/search") {
                SEARCH_HTML
            } else if url.ends_with("/book/1") {
                INFO_HTML
            } else if url.ends_with("/book/1/toc") {
                TOC_HTML
            } else if url.ends_with("/chapter/1") {
                CONTENT_HTML
            } else {
                ""
            };
            Ok(HttpResponse {
                url,
                status: StatusCode::OK,
                headers: HeaderMap::new(),
                body: Bytes::from(text.to_string()),
                text: text.to_string(),
            })
        })
    }
}

fn engine() -> BookEngine {
    let rules = Arc::new(RuleEngine::new());
    let js = Arc::new(JsEngine::new().expect("js engine"));
    rules.set_js(js.clone());
    BookEngine::new(rules, js, Arc::new(MockFetcher))
}

fn visible_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[tokio::test]
async fn matches_kotlin_pipeline_oracle() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../app/build/ab/pipeline.jsonl");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "read Kotlin pipeline oracle {}: {error}; run PipelineOracleTest first",
            path.display()
        )
    });
    let oracle: Value = serde_json::from_str(text.trim()).expect("pipeline oracle");
    assert_eq!(oracle["ok"], Value::Bool(true));

    let sources = parse_sources(SOURCE).expect("parse source");
    let source = &sources[0];
    let engine = engine();

    let results = engine.search(source, "kw", 1).await.expect("search");
    let search: Vec<Value> = results
        .iter()
        .map(|book| {
            json!({
                "name": book.name,
                "author": book.author,
                "intro": book.intro.clone().unwrap_or_default(),
                "bookUrl": book.book_url,
            })
        })
        .collect();
    assert_eq!(Value::Array(search), oracle["search"], "search");

    let mut book: Book = search_book_to_book(&results[0]);
    engine
        .book_info(source, &mut book)
        .await
        .expect("book info");
    let info = json!({
        "name": book.name,
        "author": book.author,
        "intro": book.intro.clone().unwrap_or_default(),
        "tocUrl": book.toc_url,
    });
    assert_eq!(info, oracle["info"], "info");
    let downloads = engine
        .download_urls(source, &mut book)
        .await
        .expect("download urls");
    assert_eq!(
        Value::Array(downloads.into_iter().map(Value::String).collect()),
        oracle["downloads"]
    );

    let mut chapters: Vec<BookChapter> =
        engine.chapters(source, &mut book).await.expect("chapters");
    let chapter_values: Vec<Value> = chapters
        .iter()
        .map(|chapter| {
            json!({
                "title": chapter.title,
                "url": absolute_url(&book.toc_url, &chapter.url),
            })
        })
        .collect();
    assert_eq!(Value::Array(chapter_values), oracle["chapters"], "chapters");

    let content = engine
        .content(source, &mut book, &mut chapters[0], None)
        .await
        .expect("content");
    assert_eq!(
        visible_text(&content),
        visible_text(oracle["content"].as_str().unwrap_or_default()),
        "content"
    );
}
