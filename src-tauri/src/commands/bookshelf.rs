//! 书架与导入 IPC 命令（M2 / M3）。

use serde::Serialize;
use tauri::State;

use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::data::shelf_export;
use crate::domain::bookshelf::{self, BookSort, VirtualGroup};
use crate::domain::import_service;
use crate::domain::local_import;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterWordCount {
    pub index: i32,
    pub count: usize,
}

#[tauri::command]
pub fn bookshelf_list(
    state: State<AppState>,
    group_id: i64,
    sort: i32,
) -> CmdResult<Vec<crate::domain::model::Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let group = VirtualGroup::from_id(group_id).ok_or("无效分组")?;
    let sort = BookSort::from_i32(sort).unwrap_or_default();
    let mut books = bookshelf::list_by_group(&conn, group, sort).map_err(|e| e.to_string())?;
    for book in &mut books {
        if import_service::ensure_epub_cover(book, &state.storage).unwrap_or(false) {
            let _ = BookRepo::upsert(&conn, book);
        }
        state.storage.hydrate_book(book);
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
pub fn book_set_remote_origin(
    state: State<AppState>,
    book_url: String,
    origin: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut book = BookRepo::get(&conn, &book_url)
        .map_err(|e| e.to_string())?
        .ok_or("书籍不存在")?;
    book.origin = origin;
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
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
pub fn book_add_group(state: State<AppState>, book_url: String, group: u64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut book = BookRepo::get(&conn, &book_url)
        .map_err(|e| e.to_string())?
        .ok_or("书籍不存在")?;
    book.group |= group;
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_batch_add_group(
    state: State<AppState>,
    book_urls: Vec<String>,
    group: u64,
) -> CmdResult<usize> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut count = 0;
    for url in book_urls {
        if let Some(mut book) = BookRepo::get(&conn, &url).map_err(|e| e.to_string())? {
            book.group |= group;
            BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn book_set_can_update(
    state: State<AppState>,
    book_url: String,
    value: bool,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut book = BookRepo::get(&conn, &book_url)
        .map_err(|e| e.to_string())?
        .ok_or("书籍不存在")?;
    book.can_update = value;
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_batch_set_can_update(
    state: State<AppState>,
    book_urls: Vec<String>,
    value: bool,
) -> CmdResult<usize> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut count = 0;
    for url in book_urls {
        if let Some(mut book) = BookRepo::get(&conn, &url).map_err(|e| e.to_string())? {
            book.can_update = value;
            BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn import_txt(
    state: State<AppState>,
    path: String,
    name: Option<String>,
    group: Option<u64>,
    charset: Option<String>,
    split_long: Option<bool>,
) -> CmdResult<import_service::ImportResult> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::import_txt(
        &conn,
        &path,
        &state.storage,
        name.as_deref(),
        group.unwrap_or(0),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn import_book(
    state: State<AppState>,
    path: String,
    name: Option<String>,
    group: Option<u64>,
    charset: Option<String>,
    split_long: Option<bool>,
) -> CmdResult<import_service::ImportResult> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::import_book(
        &conn,
        &path,
        &state.storage,
        name.as_deref(),
        group.unwrap_or(0),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn local_import_list(
    state: State<AppState>,
    path: String,
) -> CmdResult<Vec<local_import::LocalImportEntry>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::list_dir(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn local_import_scan(
    state: State<AppState>,
    path: String,
) -> CmdResult<Vec<local_import::LocalImportEntry>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::scan_dir(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn local_import_import(
    state: State<AppState>,
    paths: Vec<String>,
    group: Option<u64>,
    name_script: Option<String>,
    charset: Option<String>,
) -> CmdResult<Vec<local_import::LocalImportResult>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Ok(local_import::import_files(
        &conn,
        &state.storage,
        &paths,
        group.unwrap_or(0),
        name_script.as_deref(),
        charset.as_deref(),
    ))
}

#[tauri::command(async)]
pub fn local_import_archive_list(path: String) -> CmdResult<Vec<local_import::ArchiveEntry>> {
    local_import::archive_list(&path).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn local_import_archive_import(
    state: State<AppState>,
    path: String,
    names: Vec<String>,
    group: Option<u64>,
    name_script: Option<String>,
    charset: Option<String>,
) -> CmdResult<Vec<local_import::LocalImportResult>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::archive_import(
        &conn,
        &state.storage,
        &path,
        &names,
        group.unwrap_or(0),
        name_script.as_deref(),
        charset.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn local_import_reimport(
    state: State<AppState>,
    path: String,
    charset: Option<String>,
) -> CmdResult<local_import::LocalImportResult> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    local_import::reimport_file(&conn, &state.storage, &path, charset.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn local_import_delete(paths: Vec<String>) -> CmdResult<usize> {
    local_import::delete_files(&paths).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_cover_save(
    state: State<AppState>,
    book_url: String,
    src_path: String,
) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let cover = local_import::save_cover_from_file(&conn, &state.storage, &book_url, &src_path)
        .map_err(|e| e.to_string())?;
    Ok(state.storage.resolve(&cover))
}

#[tauri::command]
pub fn book_cover_refresh(state: State<AppState>, book_url: String) -> CmdResult<Option<String>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let cover =
        local_import::refresh_cover(&conn, &state.storage, &book_url).map_err(|e| e.to_string())?;
    Ok(cover.map(|value| state.storage.resolve(&value)))
}

#[tauri::command]
pub fn book_file_size(state: State<AppState>, book_url: String) -> CmdResult<i64> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let book = BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())?;
    let Some(book) = book else {
        return Ok(0);
    };
    if !book.is_local() && book.source_path.is_none() {
        return Ok(0);
    }
    Ok(state
        .storage
        .storage_size(&book)
        .map_err(|e| e.to_string())?
        .total as i64)
}

#[tauri::command]
pub fn book_clear_cache(state: State<AppState>, book_url: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let book = BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())?;
    let Some(book) = book else {
        return Ok(());
    };
    if !book.is_local() && book.source_path.is_none() {
        conn.execute(
            "DELETE FROM kv WHERE key LIKE ?1",
            [format!("chapterOverride:{}:%", book.book_url)],
        )
        .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if import_service::is_cached_book(&book, &state.storage) {
        return Err("该格式新章节缓存由阅读器管理，请使用“更新目录”或“新章节”".into());
    }
    state.storage.clear_cache(&book).map_err(|e| e.to_string())
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
        &state.storage,
        &book_url,
        rule.as_deref(),
        charset.as_deref(),
        split_long.unwrap_or(true),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn chapter_word_counts(
    state: State<AppState>,
    book_url: String,
) -> CmdResult<Vec<ChapterWordCount>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let chapters = ChapterRepo::list(&conn, &book_url).map_err(|e| e.to_string())?;
    let mut result = Vec::with_capacity(chapters.len());
    for chapter in chapters {
        let content = import_service::read_chapter(
            &conn,
            &state.storage,
            &book_url,
            chapter.index,
        )
        .unwrap_or_default();
        result.push(ChapterWordCount {
            index: chapter.index,
            count: content.chars().filter(|ch| !ch.is_whitespace()).count(),
        });
    }
    Ok(result)
}

#[tauri::command]
pub fn read_chapter(state: State<AppState>, book_url: String, index: i32) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::read_chapter(&conn, &state.storage, &book_url, index).map_err(|e| e.to_string())
}
