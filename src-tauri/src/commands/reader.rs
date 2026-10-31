use serde::Serialize;
use tauri::State;

use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::domain::content_processor;
use crate::domain::import_service;
use crate::domain::layout_engine::LayoutStyle;
use crate::domain::model::ReplaceRule;
use crate::domain::read_state::{self, ReaderSnapshot};
use crate::state::AppState;
use reader_core::search::{search_text, SearchChapter};

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSearchResult {
    pub chapter_index: i32,
    pub chapter_title: String,
    pub pos: i32,
    pub snippet: String,
    pub result_count_within_chapter: i32,
    pub result_text: String,
    pub query: String,
    pub query_index_in_result: i32,
    pub query_index_in_chapter: i32,
}

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
        &state.storage,
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
pub fn reader_invalidate_book(
    state: State<AppState>,
    book_url: String,
) -> CmdResult<()> {
    let mut cache = state.reader_cache.lock().map_err(|e| e.to_string())?;
    cache.invalidate_book(&book_url);
    Ok(())
}

#[tauri::command]
pub fn reader_invalidate_all(state: State<AppState>) -> CmdResult<()> {
    let mut cache = state.reader_cache.lock().map_err(|e| e.to_string())?;
    cache.invalidate_all();
    Ok(())
}

#[tauri::command]
pub fn reader_search_book(
    state: State<AppState>,
    book_url: String,
    keyword: String,
    limit: Option<usize>,
) -> CmdResult<Vec<BookSearchResult>> {
    let keyword = keyword.trim().to_string();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let book = BookRepo::get(&conn, &book_url)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "书籍不存在".to_string())?;
    let chapters = ChapterRepo::list(&conn, &book_url).map_err(|e| e.to_string())?;
    let mut search_chapters = Vec::new();
    for chapter in chapters {
        let Ok(content) = import_service::read_chapter(
            &conn,
            &state.storage,
            &book_url,
            chapter.index,
        ) else {
            continue;
        };
        let processed = content_processor::process_with_replace(
            &conn,
            &chapter.title,
            &content,
            &book,
            true,
            false,
        )
        .map_err(|e| e.to_string())?;
        search_chapters.push(SearchChapter {
            index: chapter.index,
            title: chapter.title.clone(),
            content: processed.contents.join("\n"),
        });
    }
    Ok(search_text(&search_chapters, &keyword, limit)
        .into_iter()
        .map(|result| BookSearchResult {
            chapter_index: result.chapter_index,
            chapter_title: result.chapter_title,
            pos: result.pos,
            snippet: result.snippet,
            result_count_within_chapter: result.result_count_within_chapter,
            result_text: result.result_text,
            query: result.query,
            query_index_in_result: result.query_index_in_result,
            query_index_in_chapter: result.query_index_in_chapter,
        })
        .collect())
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
    read_state::effective_rules(
        &conn,
        &state.storage,
        &mut cache,
        &book_url,
        chapter_index,
        &style,
    )
    .map_err(|e| e.to_string())
}
