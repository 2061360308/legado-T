use tauri::State;

use crate::data::backup::{self, BackupReport, RestoreReport};
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn backup_export(
    state: State<AppState>,
    path: String,
    password: Option<String>,
) -> CmdResult<BackupReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    backup::export_backup(&conn, &path, password.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn backup_import(
    state: State<AppState>,
    path: String,
    password: Option<String>,
    ignore: Option<Vec<String>>,
) -> CmdResult<RestoreReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let ignore = ignore.unwrap_or_default();
    backup::import_backup(&conn, &path, password.as_deref(), &ignore).map_err(|e| e.to_string())
}
