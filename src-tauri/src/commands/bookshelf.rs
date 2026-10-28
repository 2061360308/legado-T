//! 书架与导入 IPC 命令（M2 / M3）。

use tauri::{Manager, State};

use crate::data::repositories::BookRepo;
use crate::data::shelf_export;
use crate::domain::bookshelf::{self, BookSort, VirtualGroup};
use crate::domain::import_service;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn bookshelf_list(
    state: State<AppState>,
    group_id: i64,
    sort: i32,
) -> CmdResult<Vec<crate::domain::model::Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let group = VirtualGroup::from_id(group_id).ok_or("无效分组")?;
    let sort = BookSort::from_i32(sort).unwrap_or_default();
    bookshelf::list_by_group(&conn, group, sort).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bookshelf_export(state: State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let books = BookRepo::list(&conn).map_err(|e| e.to_string())?;
    shelf_export::export_shelf(&books).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_move_to_top(state: State<AppState>, book_url: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    bookshelf::move_to_top(&conn, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_set_group(state: State<AppState>, book_url: String, group: u64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut book = BookRepo::get(&conn, &book_url)
        .map_err(|e| e.to_string())?
        .ok_or("书籍不存在")?;
    book.group = group;
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_txt(
    state: State<AppState>,
    app: tauri::AppHandle,
    path: String,
    name: Option<String>,
    group: Option<u64>,
    charset: Option<String>,
    split_long: Option<bool>,
) -> CmdResult<import_service::ImportResult> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    import_service::import_txt(
        &conn,
        &path,
        &data_dir,
        name.as_deref(),
        group.unwrap_or(0),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_book(
    state: State<AppState>,
    app: tauri::AppHandle,
    path: String,
    name: Option<String>,
    group: Option<u64>,
    charset: Option<String>,
    split_long: Option<bool>,
) -> CmdResult<import_service::ImportResult> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    import_service::import_book(
        &conn,
        &path,
        &data_dir,
        name.as_deref(),
        group.unwrap_or(0),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn re_split_book(
    state: State<AppState>,
    book_url: String,
    rule: Option<String>,
    charset: Option<String>,
    split_long: Option<bool>,
) -> CmdResult<usize> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::re_split(
        &conn,
        &book_url,
        rule.as_deref(),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_chapter(state: State<AppState>, book_url: String, index: i32) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::read_chapter(&conn, &book_url, index).map_err(|e| e.to_string())
}
