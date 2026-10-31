use tauri::State;

use crate::data::repositories::BookmarkRepo;
use crate::domain::model::Bookmark;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn bookmark_list(state: State<AppState>) -> CmdResult<Vec<Bookmark>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    BookmarkRepo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bookmark_add(
    state: State<AppState>,
    book_url: String,
    book_name: String,
    book_author: String,
    chapter_index: i32,
    chapter_pos: i32,
    chapter_name: String,
    book_text: String,
    content: Option<String>,
) -> CmdResult<Bookmark> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let bookmark = Bookmark {
        time: chrono::Utc::now().timestamp_millis(),
        book_name,
        book_author,
        book_url,
        chapter_index,
        chapter_pos,
        chapter_name,
        book_text,
        content,
    };
    BookmarkRepo::upsert(&conn, &bookmark).map_err(|e| e.to_string())?;
    Ok(bookmark)
}

#[tauri::command]
pub fn bookmark_update(
    state: State<AppState>,
    time: i64,
    book_text: String,
    content: Option<String>,
) -> CmdResult<Bookmark> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut bookmark = BookmarkRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|item| item.time == time)
        .ok_or("书签不存在")?;
    bookmark.book_text = book_text;
    bookmark.content = content;
    BookmarkRepo::upsert(&conn, &bookmark).map_err(|e| e.to_string())?;
    Ok(bookmark)
}

#[tauri::command]
pub fn bookmark_delete(state: State<AppState>, time: i64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    BookmarkRepo::delete(&conn, time).map_err(|e| e.to_string())
}
