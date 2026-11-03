use std::cell::RefCell;
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, Mutex};

use engine_flow::PageFetcher;
use engine_http_native::UrlRule;
use engine_js::wasm::WasmEngine;
use engine_model::{
    Book, BookChapter, BookSource, BoxFuture, Bytes, CacheStore, CookieStore, HeaderMap,
    HttpFetcher, HttpRequest, HttpResponse, JsEvaluator, JsRequest, JsValue, Result, RssArticle,
    RssSource, StatusCode,
};
use engine_pipeline::EnginePipeline;
use engine_rule::RuleEngine;
use futures::executor::block_on;

use crate::cookie::{COOKIES, WasmCookieStore, apply_set_cookie};

#[link(wasm_import_module = "engine_http")]
unsafe extern "C" {
    fn engine_http_fetch(
        url_ptr: *const u8,
        url_len: usize,
        init_ptr: *const u8,
        init_len: usize,
    ) -> *mut u8;
}

#[derive(Default)]
struct WasmCacheStore {
    entries: Mutex<HashMap<String, Bytes>>,
}

impl CacheStore for WasmCacheStore {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            Ok(self
                .entries
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(key)
                .cloned())
        })
    }

    fn put<'a>(
        &'a self,
        key: &'a str,
        value: Bytes,
        _ttl: Option<std::time::Duration>,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.entries
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(key.to_string(), value);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.entries
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(key);
            Ok(())
        })
    }
}

struct BrowserHttpFetcher;

impl HttpFetcher for BrowserHttpFetcher {
    fn fetch<'a>(&'a self, request: HttpRequest) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move {
            let mut headers = serde_json::Map::new();
            for (name, value) in &request.headers {
                if let Ok(value) = value.to_str() {
                    headers.insert(
                        name.as_str().to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
            let cookie = COOKIES.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .map(|store| store.get_cookie(&request.url))
                    .unwrap_or_default()
            });
            if !cookie.is_empty() {
                headers.insert(
                    "cookie".to_string(),
                    serde_json::Value::String(cookie),
                );
            }
            let init = serde_json::json!({
                "method": request.method.as_str(),
                "headers": headers,
                "body": request.body.as_ref().map(|body| String::from_utf8_lossy(body).into_owned()),
            });
            let raw = fetch_raw(&request.url, &init.to_string())
                .map_err(|error| engine_model::Error::Http(format!("proxy response: {error}")))?;
            let value: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|error| engine_model::Error::Http(format!("proxy response: {error}")))?;
            let status = value
                .get("status")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u16;
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
            let url = value
                .get("url")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&request.url)
                .to_string();
            let text = value
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let mut response_headers = HeaderMap::new();
            if let Some(values) = value.get("headers").and_then(serde_json::Value::as_object) {
                for (name, value) in values {
                    if let (Ok(name), Some(value)) = (
                        name.parse::<http::header::HeaderName>(),
                        value.as_str(),
                    ) {
                        if let Ok(value) = http::header::HeaderValue::from_str(value) {
                            response_headers.insert(name, value);
                        }
                    }
                }
            }
            if let Some(cookies) = value.get("setCookies").and_then(serde_json::Value::as_array) {
                for item in cookies {
                    if let Some(cookie) = item.as_str() {
                        apply_set_cookie(&url, cookie);
                    } else if let (Some(cookie), Some(cookie_url)) = (
                        item.get("cookie").and_then(serde_json::Value::as_str),
                        item.get("url").and_then(serde_json::Value::as_str),
                    ) {
                        apply_set_cookie(cookie_url, cookie);
                    }
                }
            }
            Ok(HttpResponse {
                url,
                status,
                headers: response_headers,
                body: Bytes::from(text.clone()),
                text,
            })
        })
    }
}

struct WasmPageFetcher;

impl PageFetcher for WasmPageFetcher {
    fn fetch_rule<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move {
            if rule.web_view() {
                return Err(engine_model::Error::Other(
                    "WebView 在 Web 运行时不可用".to_string(),
                ));
            }
            BrowserHttpFetcher.fetch(rule.request()).await
        })
    }
}

struct WasmJs;

impl JsEvaluator for WasmJs {
    fn eval<'a>(&'a self, request: JsRequest<'a>) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            ENGINE.with(|cell| {
                let cell = cell.borrow();
                let engine = cell
                    .as_ref()
                    .ok_or_else(|| engine_model::Error::Js("engine is not initialized".into()))?;
                engine
                    .eval_request(&request, None)
                    .map(|evaluated| evaluated.value)
            })
        })
    }

    fn eval_value<'a>(
        &'a self,
        request: JsRequest<'a>,
        result_items: Option<&'a [String]>,
    ) -> BoxFuture<'a, Result<JsValue>> {
        Box::pin(async move {
            ENGINE.with(|cell| {
                let cell = cell.borrow();
                let engine = cell
                    .as_ref()
                    .ok_or_else(|| engine_model::Error::Js("engine is not initialized".into()))?;
                let evaluated = engine.eval_request(&request, result_items)?;
                Ok(match evaluated.items {
                    Some(items) => JsValue::List(items),
                    None => match evaluated.kind {
                        engine_js::worker::JsKind::Text => JsValue::Text(evaluated.value),
                        engine_js::worker::JsKind::Null => JsValue::Null,
                        engine_js::worker::JsKind::Other => JsValue::Other(evaluated.value),
                    },
                })
            })
        })
    }

    fn eval_bytes<'a>(&'a self, request: JsRequest<'a>) -> BoxFuture<'a, Result<Vec<u8>>> {
        Box::pin(async move {
            ENGINE.with(|cell| {
                let cell = cell.borrow();
                let engine = cell
                    .as_ref()
                    .ok_or_else(|| engine_model::Error::Js("engine is not initialized".into()))?;
                let evaluated = engine.eval_request(&request, None)?;
                if let Some(items) = evaluated.items {
                    let mut bytes = Vec::with_capacity(items.len());
                    for item in items {
                        let value = item.trim().parse::<f64>().map_err(|error| {
                            engine_model::Error::Js(format!(
                                "image byte is not numeric: {error}"
                            ))
                        })?;
                        bytes.push(value as u8);
                    }
                    return Ok(bytes);
                }
                Ok(evaluated.value.into_bytes())
            })
        })
    }

    fn eval_response<'a>(&'a self, request: JsRequest<'a>) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move {
            let original = request.response.cloned();
            ENGINE.with(|cell| {
                let cell = cell.borrow();
                let engine = cell
                    .as_ref()
                    .ok_or_else(|| engine_model::Error::Js("engine is not initialized".into()))?;
                let evaluated = engine.eval_request(&request, None)?;
                if let Some(response) = evaluated.response {
                    return Ok(response);
                }
                match original {
                    Some(mut response) if !evaluated.value.is_empty() => {
                        response.body = evaluated.value.clone().into();
                        response.text = evaluated.value;
                        Ok(response)
                    }
                    Some(response) => Ok(response),
                    None => Err(engine_model::Error::Js(
                        "loginCheckJs 没有输入响应".into(),
                    )),
                }
            })
        })
    }
}

fn with_pipeline<T>(task: impl FnOnce(&EnginePipeline) -> Result<T>) -> Result<T> {
    PIPELINE.with(|slot| {
        let slot = slot.borrow();
        let pipeline = slot
            .as_ref()
            .ok_or_else(|| engine_model::Error::Other("engine is not initialized".into()))?;
        task(pipeline)
    })
}

fn text_arg(ptr: *const u8, len: usize) -> std::result::Result<String, String> {
    let bytes = unsafe { bytes(ptr, len) };
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|error| error.to_string())
}

fn write_json<T: serde::Serialize>(value: &T) -> std::result::Result<(), String> {
    let text = serde_json::to_string(value).map_err(|error| error.to_string())?;
    set_last(text.as_bytes());
    Ok(())
}

fn source_arg(ptr: *const u8, len: usize) -> std::result::Result<BookSource, String> {
    let text = text_arg(ptr, len)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn rss_source_arg(ptr: *const u8, len: usize) -> std::result::Result<RssSource, String> {
    let text = text_arg(ptr, len)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn rss_article_arg(ptr: *const u8, len: usize) -> std::result::Result<RssArticle, String> {
    let text = text_arg(ptr, len)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn book_arg(ptr: *const u8, len: usize) -> std::result::Result<Book, String> {
    let text = text_arg(ptr, len)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn chapter_arg(ptr: *const u8, len: usize) -> std::result::Result<BookChapter, String> {
    let text = text_arg(ptr, len)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn run_source<T>(task: impl FnOnce() -> Result<T>) -> i32
where
    T: serde::Serialize,
{
    match task() {
        Ok(value) => match write_json(&value) {
            Ok(()) => 0,
            Err(error) => set_error(error),
        },
        Err(error) => set_error(error),
    }
}

fn fetch_raw(url: &str, init: &str) -> std::result::Result<String, String> {
    let response = unsafe {
        engine_http_fetch(
            url.as_ptr(),
            url.len(),
            init.as_ptr(),
            init.len(),
        )
    };
    if response.is_null() {
        return Err("host fetch returned null".to_string());
    }
    let len = unsafe {
        let head = slice::from_raw_parts(response, 4);
        u32::from_le_bytes([head[0], head[1], head[2], head[3]]) as usize
    };
    let text = unsafe {
        let body = slice::from_raw_parts(response.add(4), len);
        String::from_utf8_lossy(body).into_owned()
    };
    engine_wasm_free(response, len + 4);
    Ok(text)
}

thread_local! {
    static ENGINE: RefCell<Option<WasmEngine>> = const { RefCell::new(None) };
    static PIPELINE: RefCell<Option<EnginePipeline>> = const { RefCell::new(None) };
    static LAST: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_last(value: &[u8]) {
    LAST.with(|last| {
        let mut last = last.borrow_mut();
        last.clear();
        last.extend_from_slice(value);
    });
}

fn set_error(value: impl std::fmt::Display) -> i32 {
    set_last(value.to_string().as_bytes());
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_ping() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_init() -> i32 {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        if engine.is_some() {
            set_last(b"ok");
            return 0;
        }
        let cookies = Arc::new(WasmCookieStore::default());
        let cache = Arc::new(WasmCacheStore::default());
        let js: Arc<dyn JsEvaluator> = Arc::new(WasmJs);
        let rules = Arc::new(RuleEngine::new());
        let pipeline = EnginePipeline::new(rules.clone(), js.clone(), Arc::new(WasmPageFetcher));
        match WasmEngine::with_deps(
            Some(Arc::new(BrowserHttpFetcher)),
            Some(cache),
            Some(cookies.clone()),
            Some(rules),
            "wasm",
        ) {
            Ok(value) => {
                COOKIES.with(|slot| *slot.borrow_mut() = Some(cookies));
                PIPELINE.with(|slot| *slot.borrow_mut() = Some(pipeline));
                *engine = Some(value);
                set_last(b"ok");
                0
            }
            Err(error) => set_error(error),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_get_cookie(url_ptr: *const u8, url_len: usize) -> i32 {
    let url = match text_arg(url_ptr, url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let value = COOKIES.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|store| store.get_cookie(&url))
            .unwrap_or_default()
    });
    set_last(value.as_bytes());
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_set_cookie(
    url_ptr: *const u8,
    url_len: usize,
    cookie_ptr: *const u8,
    cookie_len: usize,
) -> i32 {
    let url = match text_arg(url_ptr, url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let cookie = match text_arg(cookie_ptr, cookie_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    COOKIES.with(|slot| {
        if let Some(store) = slot.borrow().as_ref() {
            store.set_cookie(&url, &cookie);
            0
        } else {
            set_error("engine is not initialized")
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_remove_cookie(url_ptr: *const u8, url_len: usize) -> i32 {
    let url = match text_arg(url_ptr, url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    COOKIES.with(|slot| {
        if let Some(store) = slot.borrow().as_ref() {
            store.remove_cookie(&url);
            0
        } else {
            set_error("engine is not initialized")
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_set_variables(
    source_key_ptr: *const u8,
    source_key_len: usize,
    variables_ptr: *const u8,
    variables_len: usize,
) -> i32 {
    let source_key = match text_arg(source_key_ptr, source_key_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let variables_json = match text_arg(variables_ptr, variables_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let variables = match serde_json::from_str::<HashMap<String, String>>(&variables_json) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        let Some(engine) = engine.as_ref() else {
            return set_error("engine is not initialized");
        };
        engine.set_source_variables(&source_key, variables);
        set_last(b"ok");
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_eval(
    script_ptr: *const u8,
    script_len: usize,
    base_url_ptr: *const u8,
    base_url_len: usize,
    result_ptr: *const u8,
    result_len: usize,
    source_ptr: *const u8,
    source_len: usize,
) -> i32 {
    let script = unsafe { bytes(script_ptr, script_len) };
    let base_url = unsafe { bytes(base_url_ptr, base_url_len) };
    let result = unsafe { bytes(result_ptr, result_len) };
    let source = if source_len == 0 {
        None
    } else {
        Some(unsafe { bytes(source_ptr, source_len) })
    };
    let script = match std::str::from_utf8(script) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let base_url = match std::str::from_utf8(base_url) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let result = match std::str::from_utf8(result) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let source = match source.map(std::str::from_utf8) {
        Some(Ok(value)) => Some(value),
        Some(Err(error)) => return set_error(error),
        None => None,
    };
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        let Some(engine) = engine.as_ref() else {
            return set_error("engine is not initialized");
        };
        match engine.eval(script, base_url, result, source) {
            Ok(value) => {
                set_last(value.as_bytes());
                0
            }
            Err(error) => set_error(error),
        }
    })
}

fn run_text(task: impl FnOnce() -> Result<String>) -> i32 {
    match task() {
        Ok(value) => {
            set_last(value.as_bytes());
            0
        }
        Err(error) => set_error(error),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_search(
    source_ptr: *const u8,
    source_len: usize,
    key_ptr: *const u8,
    key_len: usize,
    page: i32,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let key = match text_arg(key_ptr, key_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let page = page.max(1);
    run_source(|| {
        with_pipeline(|pipeline| block_on(pipeline.books().search(&source, &key, page)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_explore(
    source_ptr: *const u8,
    source_len: usize,
    url_ptr: *const u8,
    url_len: usize,
    page: i32,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let url = match text_arg(url_ptr, url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let page = page.max(1);
    run_source(|| {
        with_pipeline(|pipeline| block_on(pipeline.books().explore(&source, &url, page)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_info(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    if book.origin.is_empty() {
        book.origin = source.book_source_url.clone();
    }
    if book.origin_name.is_empty() {
        book.origin_name = source.book_source_name.clone();
    }
    run_source(|| {
        with_pipeline(|pipeline| {
            block_on(pipeline.books().book_info(&source, &mut book))?;
            Ok(book)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_toc(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_source(|| {
        with_pipeline(|pipeline| block_on(pipeline.books().chapters(&source, &mut book)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_reviews(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
    chapter_ptr: *const u8,
    chapter_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut chapter = match chapter_arg(chapter_ptr, chapter_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_source(|| {
        with_pipeline(|pipeline| {
            block_on(pipeline.books().reviews(&source, &mut book, &mut chapter))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_download_urls(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_source(|| {
        with_pipeline(|pipeline| {
            block_on(pipeline.books().download_urls(&source, &mut book))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_pay(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
    chapter_ptr: *const u8,
    chapter_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut chapter = match chapter_arg(chapter_ptr, chapter_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    match with_pipeline(|pipeline| {
        block_on(pipeline.books().pay_action(&source, &mut book, &mut chapter))
    }) {
        Ok(engine_flow::PayActionResult::Url(url)) => {
            set_last(serde_json::json!({ "result": "url", "value": url }).to_string().as_bytes());
            0
        }
        Ok(engine_flow::PayActionResult::Success) => {
            set_last(br#"{"result":"success"}"#);
            0
        }
        Ok(engine_flow::PayActionResult::Noop) => {
            set_last(br#"{"result":"noop"}"#);
            0
        }
        Err(error) => set_error(error),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_resource(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
    chapter_ptr: *const u8,
    chapter_len: usize,
    next_url_ptr: *const u8,
    next_url_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut chapter = match chapter_arg(chapter_ptr, chapter_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let next_url = if next_url_len == 0 {
        None
    } else {
        match text_arg(next_url_ptr, next_url_len) {
            Ok(value) => Some(value),
            Err(error) => return set_error(error),
        }
    };
    run_text(|| {
        with_pipeline(|pipeline| {
            block_on(pipeline.books().resource(
                &source,
                &mut book,
                &mut chapter,
                next_url.as_deref(),
            ))
        })
    })
}

fn image_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.starts_with(b"BM") {
        "image/bmp"
    } else if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        "image/svg+xml"
    } else {
        "image/jpeg"
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_image(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
    chapter_ptr: *const u8,
    chapter_len: usize,
    url_ptr: *const u8,
    url_len: usize,
) -> i32 {
    use base64::Engine as _;
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut chapter = match chapter_arg(chapter_ptr, chapter_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let url = match text_arg(url_ptr, url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_text(|| {
        with_pipeline(|pipeline| {
            let resource = if url.trim().is_empty() {
                block_on(pipeline.books().resource(&source, &mut book, &mut chapter, None))?
            } else {
                url
            };
            let bytes = block_on(pipeline.books().fetch_image(
                &source,
                &mut book,
                &mut chapter,
                &resource,
                false,
            ))?;
            let mime = image_mime(&bytes);
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(format!("data:{mime};base64,{encoded}"))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_explore_categories(
    source_ptr: *const u8,
    source_len: usize,
    raw_ptr: *const u8,
    raw_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let raw = match text_arg(raw_ptr, raw_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_source(|| {
        Ok(block_on(engine_explore::resolve_explore_categories(
            &source,
            &raw,
            &WasmJs,
        )))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_rss_articles(
    source_ptr: *const u8,
    source_len: usize,
    sort_name_ptr: *const u8,
    sort_name_len: usize,
    sort_url_ptr: *const u8,
    sort_url_len: usize,
    page: i32,
) -> i32 {
    let source = match rss_source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let sort_name = match text_arg(sort_name_ptr, sort_name_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let sort_url = match text_arg(sort_url_ptr, sort_url_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_source(|| {
        with_pipeline(|pipeline| {
            let (articles, next) = block_on(pipeline.rss().articles(
                &source,
                &sort_name,
                &sort_url,
                page.max(1),
            ))?;
            Ok(serde_json::json!({ "articles": articles, "next": next }))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_rss_content(
    source_ptr: *const u8,
    source_len: usize,
    article_ptr: *const u8,
    article_len: usize,
    rule_ptr: *const u8,
    rule_len: usize,
) -> i32 {
    let source = match rss_source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let article = match rss_article_arg(article_ptr, article_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let rule = match text_arg(rule_ptr, rule_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    run_text(|| {
        with_pipeline(|pipeline| block_on(pipeline.rss().content(&source, &article, &rule)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_source_content(
    source_ptr: *const u8,
    source_len: usize,
    book_ptr: *const u8,
    book_len: usize,
    chapter_ptr: *const u8,
    chapter_len: usize,
    next_url_ptr: *const u8,
    next_url_len: usize,
) -> i32 {
    let source = match source_arg(source_ptr, source_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut book = match book_arg(book_ptr, book_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let mut chapter = match chapter_arg(chapter_ptr, chapter_len) {
        Ok(value) => value,
        Err(error) => return set_error(error),
    };
    let next_url = if next_url_len == 0 {
        None
    } else {
        match text_arg(next_url_ptr, next_url_len) {
            Ok(value) => Some(value),
            Err(error) => return set_error(error),
        }
    };
    run_text(|| {
        with_pipeline(|pipeline| {
            block_on(pipeline.books().content(
                &source,
                &mut book,
                &mut chapter,
                next_url.as_deref(),
            ))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_last_ptr() -> *const u8 {
    LAST.with(|last| last.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_last_len() -> usize {
    LAST.with(|last| last.borrow().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_wasm_free(ptr: *mut u8, cap: usize) {
    if ptr.is_null() || cap == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, 0, cap));
    }
}

unsafe fn bytes<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}
