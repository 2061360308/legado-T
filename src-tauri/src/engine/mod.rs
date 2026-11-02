use std::sync::{Arc, Mutex, OnceLock};

use engine_http_native::FileCookieStore;
use engine_model::CookieStore;
use engine_native::Pipeline;
use tauri::{AppHandle, Manager};

use crate::webview_bridge::TauriWebViewBridge;

static SEARCH_PIPELINE: OnceLock<Arc<Pipeline>> = OnceLock::new();
static INTERACTIVE_PIPELINE: OnceLock<Arc<Pipeline>> = OnceLock::new();
static COOKIES: OnceLock<Arc<dyn CookieStore>> = OnceLock::new();
static PIPELINE_INIT: Mutex<()> = Mutex::new(());

fn cookies(app: &AppHandle) -> Result<Arc<dyn CookieStore>, String> {
    if let Some(cookies) = COOKIES.get() {
        return Ok(cookies.clone());
    }
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let store: Arc<dyn CookieStore> =
        Arc::new(FileCookieStore::new(dir.join("engine-cookies.json")));
    let _ = COOKIES.set(store.clone());
    Ok(COOKIES.get().cloned().unwrap_or(store))
}

fn build_pipeline(app: &AppHandle) -> Result<Arc<Pipeline>, String> {
    let cookies = cookies(app)?;
    Ok(Arc::new(
        Pipeline::with_cookie_store_and_webview(
            cookies,
            Some(Arc::new(TauriWebViewBridge::new(app.clone()))),
        )
        .map_err(|error| error.to_string())?,
    ))
}

fn lane_pipeline(
    app: &AppHandle,
    lane: &'static OnceLock<Arc<Pipeline>>,
) -> Result<Arc<Pipeline>, String> {
    if let Some(pipeline) = lane.get() {
        return Ok(pipeline.clone());
    }
    let _guard = PIPELINE_INIT
        .lock()
        .map_err(|error| error.to_string())?;
    if let Some(pipeline) = lane.get() {
        return Ok(pipeline.clone());
    }
    let pipeline = build_pipeline(app)?;
    let _ = lane.set(pipeline.clone());
    Ok(lane.get().cloned().unwrap_or(pipeline))
}

pub fn create_pipeline(app: &AppHandle) -> Result<Arc<Pipeline>, String> {
    lane_pipeline(app, &INTERACTIVE_PIPELINE)
}

pub fn create_search_pipeline(app: &AppHandle) -> Result<Arc<Pipeline>, String> {
    lane_pipeline(app, &SEARCH_PIPELINE)
}
