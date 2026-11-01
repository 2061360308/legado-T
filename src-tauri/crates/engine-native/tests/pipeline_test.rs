use std::sync::Arc;

use engine_native::parse_sources;
use engine_native::upstream::{
    book::{search_book_to_book, BookEngine, PageFetcher},
    core::{
        Book, BookChapter, BoxFuture, Bytes, HeaderMap, HttpResponse, ReplaceRule,
        Result as CoreResult, StatusCode,
    },
    js::JsEngine,
    net::UrlRule,
    rule::RuleEngine,
};

const SOURCE: &str = r#"[
  {
    "bookSourceName": "Mock",
    "bookSourceUrl": "https://example.com",
    "bookSourceType": 0,
    "enabled": true,
    "searchUrl": "https://example.com/search?q={{key}}",
    "ruleSearch": {
      "checkKeyWord": "Book",
      "bookList": "class.item",
      "name": "class.name@text",
      "author": "class.author@text",
      "intro": "class.intro@text",
      "bookUrl": "class.name@href",
      "tocUrl": "class.toc@href"
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

const REVIEW_HTML: &str = r#"
<div class="review"><img src="/avatar/1.png"><span class="time">2026-01-01</span>Good</div>
<div class="review"><img src="/avatar/2.png"><span class="time">2026-01-02</span>Nice</div>
"#;

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
            } else if url.ends_with("/review") {
                REVIEW_HTML
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

#[tokio::test]
async fn runs_search_info_toc_content_pipeline() {
    let sources = parse_sources(SOURCE).expect("parse source");
    let source = &sources[0];
    let pipeline = engine_native::Pipeline::new().expect("pipeline");
    assert_eq!(pipeline.check_keyword(source, "fallback"), "Book");
    let engine = engine();

    let results = engine.search(source, "kw", 1).await.expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Book");
    assert_eq!(results[0].author, "Author");

    let mut book: Book = search_book_to_book(&results[0]);
    engine
        .book_info(source, &mut book)
        .await
        .expect("book info");
    assert_eq!(book.name, "Book");
    assert!(book.toc_url.ends_with("/book/1/toc"));

    let downloads = engine
        .download_urls(source, &mut book)
        .await
        .expect("download urls");
    assert_eq!(downloads, vec!["https://example.com/book/1.txt"]);

    let mut chapters: Vec<BookChapter> =
        engine.chapters(source, &mut book).await.expect("chapters");
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0].title, "Chapter 1");

    let text = engine
        .content(source, &mut book, &mut chapters[0], None)
        .await
        .expect("content");
    assert!(text.contains("Hello"), "content={text:?}");

    let rules = vec![ReplaceRule {
        pattern: "Hello".to_string(),
        replacement: "Hi".to_string(),
        is_regex: false,
        is_enabled: true,
        scope_content: true,
        timeout_millisecond: 3000,
        ..Default::default()
    }];
    let replaced = engine
        .content_with_replace(source, &mut book, &mut chapters[0], None, &rules)
        .await
        .expect("content with replace");
    assert_eq!(replaced.trim(), "Hi");
}

#[tokio::test]
async fn evaluates_pay_action_and_image_style() {
    let mut sources = parse_sources(SOURCE).expect("parse source");
    let source = &mut sources[0];
    source.rule_content.as_mut().unwrap().pay_action = Some("baseUrl + '/pay'".into());
    source.rule_content.as_mut().unwrap().image_style = Some("full".into());
    let engine = engine();
    let mut book = Book {
        book_url: "https://example.com/book/1".into(),
        ..Default::default()
    };
    let mut chapter = BookChapter {
        url: "https://example.com/chapter/1".into(),
        ..Default::default()
    };
    let action = engine
        .pay_action(source, &mut book, &mut chapter)
        .await
        .unwrap();
    assert_eq!(
        action,
        engine_native::PayActionResult::Url("https://example.com/chapter/1/pay".into())
    );
    engine.book_info(source, &mut book).await.unwrap();
    assert_eq!(book.image_style.as_deref(), Some("full"));
}

#[tokio::test]
async fn parses_review_list() {
    let mut sources = parse_sources(SOURCE).expect("parse source");
    let source = &mut sources[0];
    source.rule_review = Some(engine_native::upstream::core::ReviewRule {
        review_url: Some("https://example.com/review".into()),
        avatar_rule: Some("tag.img@src".into()),
        content_rule: Some("class.review@text".into()),
        post_time_rule: Some("class.time@text".into()),
        ..Default::default()
    });
    let engine = engine();
    let mut book = Book::default();
    let mut chapter = BookChapter {
        url: "https://example.com/chapter/1".into(),
        ..Default::default()
    };
    let reviews = engine
        .reviews(source, &mut book, &mut chapter)
        .await
        .unwrap();
    assert_eq!(reviews.len(), 2);
    assert_eq!(reviews[0].content, "2026-01-01Good");
    assert_eq!(reviews[0].post_time.as_deref(), Some("2026-01-01"));
}

#[test]
fn classifies_pay_action_like_kotlin() {
    use engine_native::PayActionResult;
    assert_eq!(
        engine_native::classify_pay_action("https://pay.example.com/x"),
        PayActionResult::Url("https://pay.example.com/x".into())
    );
    assert_eq!(
        engine_native::classify_pay_action("true"),
        PayActionResult::Success
    );
    assert_eq!(
        engine_native::classify_pay_action("1"),
        PayActionResult::Success
    );
    assert_eq!(
        engine_native::classify_pay_action("false"),
        PayActionResult::Noop
    );
    assert_eq!(
        engine_native::classify_pay_action("no"),
        PayActionResult::Noop
    );
    assert_eq!(
        engine_native::classify_pay_action("0"),
        PayActionResult::Noop
    );
    assert_eq!(
        engine_native::classify_pay_action("0.0"),
        PayActionResult::Noop
    );
    assert_eq!(
        engine_native::classify_pay_action(""),
        PayActionResult::Noop
    );
    assert_eq!(
        engine_native::classify_pay_action("null"),
        PayActionResult::Noop
    );
}

#[tokio::test]
async fn decodes_image_bytes_with_js() {
    let mut sources = parse_sources(SOURCE).expect("parse source");
    let source = &mut sources[0];
    source.rule_content.as_mut().unwrap().image_decode =
        Some("[result[0] + 1, result[1] + 1]".into());
    let engine = engine();
    let mut book = Book::default();
    let mut chapter = BookChapter::default();
    let bytes = engine
        .decode_image(source, &mut book, &mut chapter, &[64, 65], false)
        .await
        .unwrap();
    assert_eq!(bytes, vec![65, 66]);
}

#[tokio::test]
async fn non_text_source_returns_resource_url() {
    let mut sources = parse_sources(SOURCE).expect("parse source");
    let source = &mut sources[0];
    source.book_source_type = 1;
    let engine = engine();
    let mut book = Book::default();
    let mut chapter = BookChapter {
        url: "https://example.com/fallback.mp3".into(),
        resource_url: Some("https://example.com/audio.mp3".into()),
        ..Default::default()
    };
    let resource = engine
        .resource(source, &mut book, &mut chapter, None)
        .await
        .unwrap();
    assert_eq!(resource, "https://example.com/audio.mp3");
}
