use engine_model::{RssArticle, RssSource};
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::data::source_repo::RssSourceRepo;
use crate::engine::create_pipeline;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn load_sources(state: &State<'_, AppState>) -> Result<Vec<RssSource>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    RssSourceRepo::list(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_list(state: State<'_, AppState>) -> CmdResult<Vec<Value>> {
    load_sources(&state)?
        .into_iter()
        .map(|source| serde_json::to_value(source).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn rss_source_save(state: State<'_, AppState>, source: Value) -> CmdResult<()> {
    let source: RssSource = serde_json::from_value(source).map_err(|error| error.to_string())?;
    if source.source_url.trim().is_empty() {
        return Err("RSS 源 URL 不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    RssSourceRepo::upsert(&conn, &source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_delete(state: State<'_, AppState>, source_url: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    RssSourceRepo::delete(&conn, &source_url).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rss_source_import(state: State<'_, AppState>, json: String) -> CmdResult<usize> {
    let value: Value = serde_json::from_str(&json).map_err(|error| error.to_string())?;
    let values = value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value]);
    let mut sources = Vec::with_capacity(values.len());
    for value in values {
        sources.push(serde_json::from_value::<RssSource>(value).map_err(|error| error.to_string())?);
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    for source in &sources {
        RssSourceRepo::upsert(&conn, source).map_err(|error| error.to_string())?;
    }
    Ok(sources.len())
}

#[tauri::command]
pub fn rss_source_export(state: State<'_, AppState>) -> CmdResult<String> {
    let sources = load_sources(&state)?;
    serde_json::to_string_pretty(&sources).map_err(|error| error.to_string())
}

#[tauri::command(async)]
pub async fn rss_articles(
    app: AppHandle,
    state: State<'_, AppState>,
    source_url: String,
    sort_name: String,
    sort_url: String,
    page: Option<i32>,
) -> CmdResult<Value> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        RssSourceRepo::list(&conn)
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|source| source.source_url == source_url)
            .ok_or_else(|| "RSS 源不存在".to_string())?
    };
    let sort_url = if sort_url.trim().is_empty() {
        source.source_url.clone()
    } else {
        sort_url
    };
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        let (articles, next) = runtime
            .block_on(pipeline.rss_articles(&source, &sort_name, &sort_url, page.unwrap_or(1)))
            .map_err(|error| error.to_string())?;
        Ok(serde_json::json!({
            "articles": articles,
            "next": next,
        }))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn rss_content(
    app: AppHandle,
    state: State<'_, AppState>,
    source_url: String,
    article: Value,
    rule_content: Option<String>,
) -> CmdResult<String> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        RssSourceRepo::list(&conn)
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|source| source.source_url == source_url)
            .ok_or_else(|| "RSS 源不存在".to_string())?
    };
    let article: RssArticle = serde_json::from_value(article).map_err(|error| error.to_string())?;
    let rule = rule_content
        .or_else(|| source.rule_content.clone())
        .unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        runtime
            .block_on(pipeline.rss_content(&source, &article, &rule))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
