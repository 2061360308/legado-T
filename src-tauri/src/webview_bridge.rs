use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use legado_core::{BoxFuture, Error as CoreError, Result as CoreResult, WebViewBridge, WebViewRequest, WebViewResponse};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebViewBridgeResponse {
    pub url: String,
    pub text: String,
    pub cookies: Vec<String>,
}

#[derive(Clone)]
pub struct TauriWebViewBridge {
    app: AppHandle,
}

impl TauriWebViewBridge {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl WebViewBridge for TauriWebViewBridge {
    fn fetch<'a>(&'a self, request: WebViewRequest) -> BoxFuture<'a, CoreResult<WebViewResponse>> {
        let app = self.app.clone();
        Box::pin(async move {
            let response = fetch_inner(&app, request.url, request.web_js, request.delay_ms)
                .await
                .map_err(CoreError::Other)?;
            Ok(WebViewResponse {
                url: response.url,
                status: 200,
                headers: Default::default(),
                text: response.text,
                cookies: response.cookies,
            })
        })
    }
}

fn bridge_label(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or_default()
    )
}

fn decode_eval(value: String) -> String {
    serde_json::from_str::<String>(&value).unwrap_or(value)
}

async fn sleep_ms(ms: u64) {
    let _ = tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(ms.min(120_000)));
    })
    .await;
}

async fn eval_script(window: &tauri::WebviewWindow, script: String) -> Result<String, String> {
    let (tx, rx) = mpsc::channel::<String>();
    window
        .eval_with_callback(script, move |value| {
            let _ = tx.send(value);
        })
        .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(30))
            .map(decode_eval)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

async fn document_cookies(window: &tauri::WebviewWindow) -> Vec<String> {
    let Ok(value) = eval_script(window, "document.cookie".to_string()).await else {
        return Vec::new();
    };
    value
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(target_os = "android")]
fn android_cookie_header(window: &tauri::WebviewWindow, url: &str) -> Option<String> {
    use jni::objects::{JString, JValue};
    let (tx, rx) = mpsc::channel();
    let url = url.to_string();
    let _ = window.with_webview(move |webview| {
        let _ = webview.jni_handle().exec(move |env, _activity, _webview| {
            let result = (|| -> jni::errors::Result<Option<String>> {
                let manager = env
                    .call_static_method(
                        "android/webkit/CookieManager",
                        "getInstance",
                        "()Landroid/webkit/CookieManager;",
                        &[],
                    )
                    .and_then(|value| value.l())?;
                let url = env.new_string(&url)?;
                let cookie = env
                    .call_method(
                        manager,
                        "getCookie",
                        "(Ljava/lang/String;)Ljava/lang/String;",
                        &[JValue::Object(&url)],
                    )
                    .and_then(|value| value.l())?;
                if cookie.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(env.get_string(&JString::from(cookie))?.into()))
                }
            })();
            let _ = tx.send(result.ok().flatten());
        });
    });
    rx.recv_timeout(Duration::from_secs(5)).ok().flatten()
}

async fn fetch_inner(
    app: &AppHandle,
    url: String,
    web_js: Option<String>,
    delay_ms: u64,
) -> Result<WebViewBridgeResponse, String> {
    let target: tauri::Url = url.parse().map_err(|error| format!("invalid url: {error}"))?;
    let label = bridge_label("webview-fetch");
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(target))
        .title("WebView")
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?;

    sleep_ms(delay_ms).await;

    let script = web_js.unwrap_or_else(|| "document.documentElement.outerHTML".to_string());
    let text = eval_script(&window, script).await?;
    let final_url = window
        .url()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let mut cookies = window
        .cookies()
        .unwrap_or_default()
        .into_iter()
        .map(|cookie| cookie.to_string())
        .collect::<Vec<_>>();
    for cookie in document_cookies(&window).await {
        if !cookies.contains(&cookie) {
            cookies.push(cookie);
        }
    }
    #[cfg(target_os = "android")]
    if let Some(header) = android_cookie_header(&window, &final_url) {
        for cookie in header.split(';').map(str::trim).filter(|value| !value.is_empty()) {
            let cookie = cookie.to_string();
            if !cookies.contains(&cookie) {
                cookies.push(cookie);
            }
        }
    }
    let _ = window.close();

    Ok(WebViewBridgeResponse {
        url: final_url,
        text,
        cookies,
    })
}

#[tauri::command]
pub async fn webview_fetch(
    app: AppHandle,
    url: String,
    web_js: Option<String>,
    delay_ms: u64,
) -> Result<WebViewBridgeResponse, String> {
    fetch_inner(&app, url, web_js, delay_ms).await
}

#[tauri::command]
pub async fn webview_login(
    app: AppHandle,
    url: String,
    wait_ms: u64,
) -> Result<WebViewBridgeResponse, String> {
    let target: tauri::Url = url.parse().map_err(|error| format!("invalid url: {error}"))?;
    let label = bridge_label("webview-login");
    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(target))
        .title("登录")
        .build()
        .map_err(|error| error.to_string())?;

    sleep_ms(wait_ms.max(1_000)).await;

    let final_url = window
        .url()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let mut cookies = window
        .cookies()
        .unwrap_or_default()
        .into_iter()
        .map(|cookie| cookie.to_string())
        .collect::<Vec<_>>();
    for cookie in document_cookies(&window).await {
        if !cookies.contains(&cookie) {
            cookies.push(cookie);
        }
    }
    #[cfg(target_os = "android")]
    if let Some(header) = android_cookie_header(&window, &final_url) {
        for cookie in header.split(';').map(str::trim).filter(|value| !value.is_empty()) {
            let cookie = cookie.to_string();
            if !cookies.contains(&cookie) {
                cookies.push(cookie);
            }
        }
    }
    let _ = window.close();

    Ok(WebViewBridgeResponse {
        url: final_url,
        text: String::new(),
        cookies,
    })
}

#[tauri::command]
pub async fn webview_engine_probe(app: AppHandle, url: String) -> Result<String, String> {
    let pipeline = legado_engine::Pipeline::new_with_webview(Some(Arc::new(
        TauriWebViewBridge::new(app),
    )))
    .map_err(|error| error.to_string())?;
    let ctx = legado_engine::upstream::net::UrlContext::new(&url);
    let rule = legado_engine::upstream::net::UrlRule::parse(
        &format!("{url},{{\"webView\":true}}"),
        &ctx,
    )
    .await
    .map_err(|error| error.to_string())?;
    let response = pipeline
        .http()
        .send_rule(&rule)
        .await
        .map_err(|error| error.to_string())?;
    Ok(response.text)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayActionResponse {
    pub result: String,
    pub value: Option<String>,
}

#[tauri::command]
pub fn execute_pay_action(
    app: AppHandle,
    source: serde_json::Value,
    book: serde_json::Value,
    chapter: serde_json::Value,
) -> Result<PayActionResponse, String> {
    enum Outcome {
        Url(String),
        Success(usize),
        Noop,
    }

    let source: legado_engine::upstream::core::BookSource =
        serde_json::from_value(source).map_err(|error| error.to_string())?;
    let mut book: legado_engine::upstream::core::Book =
        serde_json::from_value(book).map_err(|error| error.to_string())?;
    let mut chapter: legado_engine::upstream::core::BookChapter =
        serde_json::from_value(chapter).map_err(|error| error.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let outcome: Outcome = runtime.block_on(async {
        let pipeline = legado_engine::Pipeline::new().map_err(|error| error.to_string())?;
        match pipeline
            .pay_action(&source, &mut book, &mut chapter)
            .await
            .map_err(|error| error.to_string())?
        {
            legado_engine::PayActionResult::Url(url) => Ok::<Outcome, String>(Outcome::Url(url)),
            legado_engine::PayActionResult::Success => {
                let chapters = pipeline
                    .chapters(&source, &mut book)
                    .await
                    .map_err(|error| error.to_string())?;
                Ok(Outcome::Success(chapters.len()))
            }
            legado_engine::PayActionResult::Noop => Ok::<Outcome, String>(Outcome::Noop),
        }
    })?;
    match outcome {
        Outcome::Url(url) => {
            let target: tauri::Url = url
                .parse()
                .map_err(|error| format!("invalid pay url: {error}"))?;
            let label = bridge_label("pay-action");
            let _window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(target))
                .title("章节购买")
                .build()
                .map_err(|error| error.to_string())?;
            Ok(PayActionResponse {
                result: "url".to_string(),
                value: Some(url),
            })
        }
        Outcome::Success(count) => Ok(PayActionResponse {
            result: "success".to_string(),
            value: Some(count.to_string()),
        }),
        Outcome::Noop => Ok(PayActionResponse {
            result: "noop".to_string(),
            value: None,
        }),
    }
}
#[tauri::command]
pub async fn webview_audit_sources(
    app: AppHandle,
    sources: Vec<serde_json::Value>,
    book_map: std::collections::HashMap<String, String>,
) -> Result<Vec<serde_json::Value>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        runtime.block_on(async move {
            let bridge = Arc::new(TauriWebViewBridge::new(app));
            let pipeline = legado_engine::Pipeline::new_with_webview(Some(bridge))
                .map_err(|error| error.to_string())?;
            let mut out = Vec::new();
            for value in sources {
                let source: legado_engine::upstream::core::BookSource =
                    serde_json::from_value(value).map_err(|error| error.to_string())?;
                let Some(book_url) = book_map.get(&source.book_source_url).cloned() else {
                    continue;
                };
                let mut record = serde_json::json!({
                    "name": source.book_source_name,
                    "url": source.book_source_url,
                    "bookUrl": book_url,
                });
                let mut book = legado_engine::upstream::core::Book {
                    book_url: book_url.clone(),
                    origin: source.book_source_url.clone(),
                    origin_name: source.book_source_name.clone(),
                    ..Default::default()
                };
                record["infoOk"] = pipeline.book_info(&source, &mut book).await.is_ok().into();
                match pipeline.chapters(&source, &mut book).await {
                    Ok(chapters) => {
                        record["tocOk"] = true.into();
                        record["chapterCount"] = chapters.len().into();
                        if let Some(chapter) = chapters.first() {
                            let mut chapter = chapter.clone();
                            record["chapterUrl"] = chapter.url.clone().into();
                            let content =
                                pipeline.content(&source, &mut book, &mut chapter, None).await;
                            record["contentOk"] = content.is_ok().into();
                            if let Err(error) = content {
                                record["contentError"] = error.to_string().into();
                            }
                        }
                    }
                    Err(error) => {
                        record["tocOk"] = false.into();
                        record["tocError"] = error.to_string().into();
                    }
                }
                out.push(record);
            }
            Ok::<_, String>(out)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
