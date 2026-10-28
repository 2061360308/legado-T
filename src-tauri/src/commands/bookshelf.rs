//! 书架与导入 IPC 命令（M2 / M3）。

use tauri::{Manager, State};

use crate::data::repositories::BookRepo;
use crate::data::shelf_export;
use crate::domain::bookshelf::{self, BookSort, VirtualGroup};
use crate::domain::import_service;
use crate::domain::local_import;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn bookshelf_list(
    state: State<AppState>,
    app: tauri::AppHandle,
    group_id: i64,
    sort: i32,
) -> CmdResult<Vec<crate::domain::model::Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let group = VirtualGroup::from_id(group_id).ok_or("无效分组")?;
    let sort = BookSort::from_i32(sort).unwrap_or_default();
    let mut books = bookshelf::list_by_group(&conn, group, sort).map_err(|e| e.to_string())?;
    if let Ok(data_dir) = app.path().app_data_dir() {
        for book in &mut books {
            if import_service::ensure_epub_cover(book, &data_dir).unwrap_or(false) {
                let _ = BookRepo::upsert(&conn, book);
            }
        }
    }
    Ok(books)
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
pub fn local_import_list(
    state: State<AppState>,
    path: String,
) -> CmdResult<Vec<local_import::LocalImportEntry>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::list_dir(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn local_import_scan(
    state: State<AppState>,
    path: String,
) -> CmdResult<Vec<local_import::LocalImportEntry>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::scan_dir(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn local_import_import(
    state: State<AppState>,
    app: tauri::AppHandle,
    paths: Vec<String>,
    group: Option<u64>,
    name_script: Option<String>,
) -> CmdResult<Vec<local_import::LocalImportResult>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(local_import::import_files(
        &conn,
        &data_dir,
        &paths,
        group.unwrap_or(0),
        name_script.as_deref(),
    ))
}

#[tauri::command]
pub fn local_import_delete(paths: Vec<String>) -> CmdResult<usize> {
    local_import::delete_files(&paths).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_cover_save(
    state: State<AppState>,
    app: tauri::AppHandle,
    book_url: String,
    src_path: String,
) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    local_import::save_cover_from_file(&conn, &data_dir, &book_url, &src_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_cover_refresh(
    state: State<AppState>,
    app: tauri::AppHandle,
    book_url: String,
) -> CmdResult<Option<String>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    local_import::refresh_cover(&conn, &data_dir, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_file_size(book_url: String) -> i64 {
    local_import::file_size(&book_url)
}

#[tauri::command]
pub fn book_clear_cache(book_url: String) -> CmdResult<()> {
    local_import::clear_cache(&book_url).map_err(|e| e.to_string())
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
