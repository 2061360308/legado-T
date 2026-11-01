use std::sync::Arc;

use engine_native::Pipeline;
use tauri::{AppHandle, Manager};

use crate::webview_bridge::TauriWebViewBridge;

pub fn create_pipeline(app: &AppHandle) -> Result<Pipeline, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let cookie_path = dir.join("engine-cookies.json");
    Pipeline::with_cookie_file_and_webview(
        cookie_path,
        Some(Arc::new(TauriWebViewBridge::new(app.clone()))),
    )
    .map_err(|error| error.to_string())
}
