use engine_model::{Book as EngineBook, BookChapter as EngineChapter};
use tauri::{AppHandle, State};

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::data::source_repo::BookSourceRepo;
use crate::domain::layout_engine::LayoutStyle;
use crate::domain::model::{Book as AppBook, BookChapter as AppChapter};
use crate::domain::read_state::{self, ReaderSnapshot};
use crate::engine::create_pipeline;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn override_key(book_url: &str, chapter_index: i32) -> String {
    format!("chapterOverride:{book_url}:{chapter_index}")
}

pub(crate) fn engine_book(book: &AppBook) -> EngineBook {
    EngineBook {
        book_url: book.book_url.clone(),
        toc_url: book.toc_url.clone(),
        origin: book.origin.clone(),
        origin_name: book.origin_name.clone(),
        name: book.name.clone(),
        author: book.author.clone(),
        kind: book.kind.clone(),
        cover_url: book.cover_url.clone(),
        intro: book.intro.clone(),
        book_type: book.r#type as i32,
        latest_chapter_title: book.latest_chapter_title.clone(),
        total_chapter_num: book.total_chapter_num,
        ..Default::default()
    }
}

pub(crate) fn engine_chapter(book: &AppBook, chapter: &AppChapter) -> EngineChapter {
    EngineChapter {
        url: chapter.url.clone(),
        title: chapter.title.clone(),
        is_volume: chapter.is_volume,
        base_url: book.book_url.clone(),
        book_url: book.book_url.clone(),
        index: chapter.index,
        is_vip: chapter.is_vip,
        is_pay: chapter.is_pay,
        resource_url: chapter.resource_url.clone(),
        tag: chapter.tag.clone(),
        ..Default::default()
    }
}

fn find_chapter(chapters: &[AppChapter], index: i32) -> Result<AppChapter, String> {
    chapters
        .iter()
        .find(|chapter| chapter.index == index)
        .cloned()
        .ok_or_else(|| format!("章节不存在: {index}"))
}

#[tauri::command(async)]
pub async fn online_reader_download(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
    chapter_indexes: Option<Vec<i32>>,
) -> CmdResult<i32> {
    let (book, source, chapters) = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        let book = BookRepo::get(&conn, &book_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书籍不存在".to_string())?;
        if book.origin.trim().is_empty() {
            return Err("书籍没有关联书源".to_string());
        }
        let mut chapters = ChapterRepo::list(&conn, &book_url).map_err(|error| error.to_string())?;
        if let Some(indexes) = chapter_indexes.as_ref() {
            chapters.retain(|chapter| indexes.contains(&chapter.index));
        }
        chapters.retain(|chapter| !chapter.is_volume);
        let source = BookSourceRepo::get(&conn, &book.origin)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?;
        (book, source, chapters)
    };

    let mut downloaded = 0;
    for chapter in chapters {
        let cached = {
            let conn = state.conn.lock().map_err(|error| error.to_string())?;
            KeyValue::get(&conn, &override_key(&book_url, chapter.index))
                .map_err(|error| error.to_string())?
        };
        if cached.is_some() {
            continue;
        }
        let mut engine_book = engine_book(&book);
        let mut engine_chapter = engine_chapter(&book, &chapter);
        let source_for_fetch = source.clone();
        let app_for_fetch = app.clone();
        let content = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let pipeline = create_pipeline(&app_for_fetch)?;
            runtime
                .block_on(pipeline.content(
                    &source_for_fetch,
                    &mut engine_book,
                    &mut engine_chapter,
                    None,
                ))
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| error.to_string())??;
        if content.trim().is_empty() {
            continue;
        }
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        KeyValue::set(&conn, &override_key(&book_url, chapter.index), &content)
            .map_err(|error| error.to_string())?;
        downloaded += 1;
    }
    Ok(downloaded)
}

#[tauri::command(async)]
pub async fn online_reader_open(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
    chapter_index: Option<i32>,
    chapter_pos: Option<i32>,
    style: Option<LayoutStyle>,
) -> CmdResult<ReaderSnapshot> {
    let style = style.unwrap_or_default();
    let (book, chapter, source, cached) = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        let book = BookRepo::get(&conn, &book_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书籍不存在".to_string())?;
        if book.origin.trim().is_empty() {
            return Err("书籍没有关联书源".to_string());
        }
        let chapters = ChapterRepo::list(&conn, &book_url).map_err(|error| error.to_string())?;
        let target = chapter_index
            .unwrap_or(book.dur_chapter_index)
            .clamp(0, chapters.len() as i32 - 1);
        let chapter = find_chapter(&chapters, target)?;
        let source = BookSourceRepo::get(&conn, &book.origin)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?;
        let cached = KeyValue::get(&conn, &override_key(&book_url, target))
            .map_err(|error| error.to_string())?;
        (book, chapter, source, cached)
    };

    if cached.is_none() {
        let mut engine_book = engine_book(&book);
        let mut engine_chapter = engine_chapter(&book, &chapter);
        let source_for_fetch = source.clone();
        let content = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let pipeline = create_pipeline(&app)?;
            runtime
                .block_on(pipeline.content(
                    &source_for_fetch,
                    &mut engine_book,
                    &mut engine_chapter,
                    None,
                ))
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| error.to_string())??;
        if content.trim().is_empty() {
            return Err("章节正文为空".to_string());
        }
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        KeyValue::set(&conn, &override_key(&book_url, chapter.index), &content)
            .map_err(|error| error.to_string())?;
    }

    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut cache = state.reader_cache.lock().map_err(|error| error.to_string())?;
    read_state::open(
        &conn,
        &state.storage,
        &mut cache,
        &book_url,
        Some(chapter.index),
        chapter_pos,
        &style,
    )
    .map_err(|error| error.to_string())
}
