//! High-level Legado book-source pipeline.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use engine_flow::BookEngine;
use engine_model::{
    Book, BookChapter, BookReview, BookSource, CookieStore, ReplaceRule, RssArticle, RssSource,
    SearchBook, WebViewBridge,
};
use engine_js::JsEngine;
use engine_http_native::{FileCookieStore, HttpClient, MemoryCookieStore};
use engine_rss::RssEngine;
use engine_rule::RuleEngine;
use serde_json::Value;

use crate::{EngineError, Result};

pub struct Pipeline {
    rules: Arc<RuleEngine>,
    js: Arc<JsEngine>,
    http: Arc<HttpClient>,
    cookies: Arc<dyn CookieStore>,
    books: BookEngine,
    rss: RssEngine,
}

impl Pipeline {
    pub fn new() -> Result<Self> {
        Self::with_cookie_store(Arc::new(MemoryCookieStore::new()))
    }

    pub fn new_with_webview(webview: Option<Arc<dyn WebViewBridge>>) -> Result<Self> {
        Self::with_cookie_store_and_webview(Arc::new(MemoryCookieStore::new()), webview)
    }

    pub fn with_cookie_file(path: impl Into<PathBuf>) -> Result<Self> {
        Self::with_cookie_store(Arc::new(FileCookieStore::new(path)))
    }

    pub fn with_cookie_file_and_webview(
        path: impl Into<PathBuf>,
        webview: Option<Arc<dyn WebViewBridge>>,
    ) -> Result<Self> {
        Self::with_cookie_store_and_webview(Arc::new(FileCookieStore::new(path)), webview)
    }

    pub fn with_cookie_store(cookies: Arc<dyn CookieStore>) -> Result<Self> {
        Self::with_cookie_store_and_webview(cookies, None)
    }

    pub fn with_cookie_store_and_webview(
        cookies: Arc<dyn CookieStore>,
        webview: Option<Arc<dyn WebViewBridge>>,
    ) -> Result<Self> {
        let mut builder = HttpClient::builder().cookies(cookies.clone());
        if let Some(webview) = webview {
            builder = builder.webview(webview);
        }
        let http = Arc::new(
            builder
                .build()
                .map_err(|error| EngineError::Rule(error.to_string()))?,
        );
        let rules = Arc::new(RuleEngine::new());
        let js = Arc::new(
            JsEngine::builder()
                .http(http.clone())
                .cookies(cookies.clone())
                .rules(rules.clone())
                .memory_limit(512 * 1024 * 1024)
                .timeout(Duration::from_secs(70))
                .build()
                .map_err(|error| EngineError::Rule(error.to_string()))?,
        );
        rules.set_js(js.clone());
        let books = BookEngine::new(rules.clone(), js.clone(), http.clone());
        let rss = RssEngine::new(rules.clone(), js.clone(), http.clone());
        Ok(Self {
            rules,
            js,
            http,
            cookies,
            books,
            rss,
        })
    }

    pub fn rules(&self) -> &Arc<RuleEngine> {
        &self.rules
    }

    pub fn js(&self) -> &Arc<JsEngine> {
        &self.js
    }

    pub fn http(&self) -> &Arc<HttpClient> {
        &self.http
    }

    pub fn cookies(&self) -> &Arc<dyn CookieStore> {
        &self.cookies
    }

    pub fn books(&self) -> &BookEngine {
        &self.books
    }

    pub fn get_cookie(&self, url: &str) -> String {
        self.cookies.get_cookie(url)
    }

    pub fn set_cookie(&self, url: &str, cookie: &str) {
        self.cookies.set_cookie(url, cookie);
    }

    pub fn replace_cookie(&self, url: &str, cookie: &str) {
        self.cookies.replace_cookie(url, cookie);
    }

    pub fn remove_cookie(&self, url: &str) {
        self.cookies.remove_cookie(url);
    }

    pub fn check_keyword(&self, source: &BookSource, default: &str) -> String {
        source
            .rule_search
            .as_ref()
            .and_then(|rule| rule.check_key_word.as_deref())
            .filter(|keyword| !keyword.trim().is_empty())
            .unwrap_or(default)
            .to_string()
    }

    pub async fn search(
        &self,
        source: &BookSource,
        key: &str,
        page: i32,
    ) -> Result<Vec<SearchBook>> {
        self.books
            .search(source, key, page)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn explore(
        &self,
        source: &BookSource,
        url: &str,
        page: i32,
    ) -> Result<Vec<SearchBook>> {
        self.books
            .explore(source, url, page)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn book_info(&self, source: &BookSource, book: &mut Book) -> Result<()> {
        self.books
            .book_info(source, book)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn download_urls(&self, source: &BookSource, book: &mut Book) -> Result<Vec<String>> {
        self.books
            .download_urls(source, book)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn chapters(&self, source: &BookSource, book: &mut Book) -> Result<Vec<BookChapter>> {
        self.books
            .chapters(source, book)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn content(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
    ) -> Result<String> {
        self.books
            .content(source, book, chapter, next_chapter_url)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn reviews(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
    ) -> Result<Vec<BookReview>> {
        self.books
            .reviews(source, book, chapter)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn pay_action(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
    ) -> Result<engine_flow::PayActionResult> {
        self.books
            .pay_action(source, book, chapter)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn rss_articles(
        &self,
        source: &RssSource,
        sort_name: &str,
        sort_url: &str,
        page: i32,
    ) -> Result<(Vec<RssArticle>, Option<String>)> {
        self.rss
            .articles(source, sort_name, sort_url, page)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn rss_content(
        &self,
        source: &RssSource,
        article: &RssArticle,
        rule_content: &str,
    ) -> Result<String> {
        self.rss
            .content(source, article, rule_content)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn fetch_image(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        url: &str,
        is_cover: bool,
    ) -> Result<Vec<u8>> {
        self.books
            .fetch_image(source, book, chapter, url, is_cover)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn decode_image(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        bytes: &[u8],
        is_cover: bool,
    ) -> Result<Vec<u8>> {
        self.books
            .decode_image(source, book, chapter, bytes, is_cover)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn resource(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
    ) -> Result<String> {
        self.books
            .resource(source, book, chapter, next_chapter_url)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn content_with_replace(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
        rules: &[ReplaceRule],
    ) -> Result<String> {
        self.books
            .content_with_replace(source, book, chapter, next_chapter_url, rules)
            .await
            .map_err(|error| EngineError::Rule(error.to_string()))
    }

    pub async fn apply_title_replace(
        &self,
        book: &Book,
        chapter: &mut BookChapter,
        rules: &[ReplaceRule],
    ) -> Result<engine_replace::ReplaceOutcome> {
        Ok(self.books.apply_title_replace(book, chapter, rules).await)
    }
}

pub fn parse_rss(sort_name: &str, xml: &str, source_url: &str) -> Vec<engine_model::RssArticle> {
    engine_rss::parse_default(sort_name, xml, source_url)
}

pub fn parse_sources(json: &str) -> Result<Vec<BookSource>> {
    let value: Value = serde_json::from_str(json)?;
    let items = match value {
        Value::Array(items) => items,
        Value::Object(mut map) => map
            .remove("bookSources")
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| {
                EngineError::InvalidSource("expected an array of book sources".into())
            })?,
        other => {
            return Err(EngineError::InvalidSource(format!(
                "expected book source array, got {}",
                json_type(&other)
            )))
        }
    };
    serde_json::from_value(Value::Array(items)).map_err(EngineError::Json)
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
