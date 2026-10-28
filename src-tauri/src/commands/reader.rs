use tauri::State;

use crate::domain::layout_engine::LayoutStyle;
use crate::domain::model::ReplaceRule;
use crate::domain::read_state::{self, ReaderSnapshot};
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn reader_open(
    state: State<AppState>,
    book_url: String,
    chapter_index: Option<i32>,
    chapter_pos: Option<i32>,
    style: Option<LayoutStyle>,
) -> CmdResult<ReaderSnapshot> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut cache = state.reader_cache.lock().map_err(|e| e.to_string())?;
    let style = style.unwrap_or_default();
    read_state::open(
        &conn,
        &mut cache,
        &book_url,
        chapter_index,
        chapter_pos,
        &style,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reader_save_progress(
    state: State<AppState>,
    book_url: String,
    chapter_index: i32,
    chapter_pos: i32,
    chapter_title: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    read_state::save_progress(&conn, &book_url, chapter_index, chapter_pos, &chapter_title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reader_save_chapter_content(
    state: State<AppState>,
    book_url: String,
    chapter_index: i32,
    content: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut cache = state.reader_cache.lock().map_err(|e| e.to_string())?;
    read_state::save_chapter_content(&conn, &book_url, chapter_index, &content)
        .map_err(|e| e.to_string())?;
    cache.invalidate_chapter(chapter_index);
    Ok(())
}

#[tauri::command]
pub fn reader_effective_rules(
    state: State<AppState>,
    book_url: String,
    chapter_index: i32,
    style: Option<LayoutStyle>,
) -> CmdResult<Vec<ReplaceRule>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut cache = state.reader_cache.lock().map_err(|e| e.to_string())?;
    let style = style.unwrap_or_default();
    read_state::effective_rules(&conn, &mut cache, &book_url, chapter_index, &style)
        .map_err(|e| e.to_string())
}
