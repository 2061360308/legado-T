use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLogEntry {
    pub id: i64,
    pub time: i64,
    pub level: String,
    pub source: String,
    pub message: String,
}

#[tauri::command]
pub fn app_log_add(
    state: State<'_, AppState>,
    level: String,
    source: String,
    message: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO app_logs (time, level, source, message) VALUES (?1, ?2, ?3, ?4)",
        params![
            chrono::Utc::now().timestamp_millis(),
            level,
            source,
            message
        ],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn app_log_list(
    state: State<'_, AppState>,
    limit: Option<i64>,
    source: Option<String>,
) -> CmdResult<Vec<AppLogEntry>> {
    let limit = limit.unwrap_or(500).clamp(1, 2000);
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    if let Some(source) = source.filter(|value| !value.trim().is_empty()) {
        let mut stmt = conn
            .prepare(
                "SELECT id, time, level, source, message FROM app_logs WHERE source = ?1 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params![source, limit], map_log)
            .map_err(|error| error.to_string())?;
        for row in rows {
            out.push(row.map_err(|error| error.to_string())?);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT id, time, level, source, message FROM app_logs ORDER BY id DESC LIMIT ?1")
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params![limit], map_log)
            .map_err(|error| error.to_string())?;
        for row in rows {
            out.push(row.map_err(|error| error.to_string())?);
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn app_log_clear(state: State<'_, AppState>) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute("DELETE FROM app_logs", [])
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn map_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppLogEntry> {
    Ok(AppLogEntry {
        id: row.get(0)?,
        time: row.get(1)?,
        level: row.get(2)?,
        source: row.get(3)?,
        message: row.get(4)?,
    })
}
