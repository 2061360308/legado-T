use engine_model::{Book, BookChapter, BookSource};
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::commands::online_reader::{engine_book, engine_chapter};
use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::data::source_repo::{
    get_source_variables, list_search_history, save_search_history, set_source_variable,
    BookSourceRepo,
};
use crate::domain::model::{Book as AppBook, BookChapter as AppChapter};
use crate::engine::create_pipeline;
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn source_value(source: BookSource) -> Result<Value, String> {
    serde_json::to_value(source).map_err(|error| error.to_string())
}

fn load_sources(state: &State<'_, AppState>) -> Result<Vec<BookSource>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::list(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_list(state: State<'_, AppState>) -> CmdResult<Vec<Value>> {
    load_sources(&state)?
        .into_iter()
        .map(source_value)
        .collect()
}

#[tauri::command]
pub fn book_source_get(
    state: State<'_, AppState>,
    book_source_url: String,
) -> CmdResult<Option<Value>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let source = BookSourceRepo::get(&conn, &book_source_url).map_err(|error| error.to_string())?;
    source.map(source_value).transpose()
}

#[tauri::command]
pub fn book_source_save(state: State<'_, AppState>, source: Value) -> CmdResult<()> {
    let source: BookSource = serde_json::from_value(source).map_err(|error| error.to_string())?;
    if source.book_source_url.trim().is_empty() {
        return Err("书源 URL 不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::upsert(&conn, &source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_delete(
    state: State<'_, AppState>,
    book_source_url: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::delete(&conn, &book_source_url).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_set_enabled(
    state: State<'_, AppState>,
    book_source_url: String,
    enabled: bool,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::set_enabled(&conn, &book_source_url, enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_set_explore_enabled(
    state: State<'_, AppState>,
    book_source_url: String,
    enabled: bool,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::set_explore_enabled(&conn, &book_source_url, enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_import(state: State<'_, AppState>, json: String) -> CmdResult<usize> {
    let sources = engine_native::parse_sources(&json).map_err(|error| error.to_string())?;
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    for source in &sources {
        BookSourceRepo::upsert(&conn, source).map_err(|error| error.to_string())?;
    }
    Ok(sources.len())
}

#[tauri::command]
pub fn book_source_export(state: State<'_, AppState>) -> CmdResult<String> {
    let sources = load_sources(&state)?;
    serde_json::to_string_pretty(&sources).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_groups(state: State<'_, AppState>) -> CmdResult<Vec<String>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    BookSourceRepo::groups(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_search_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> CmdResult<Vec<String>> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    list_search_history(&conn, limit.unwrap_or(20).clamp(1, 100))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_set_variable(
    state: State<'_, AppState>,
    book_source_url: String,
    key: String,
    value: String,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    set_source_variable(&conn, &book_source_url, &key, &value)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn book_source_get_variables(
    state: State<'_, AppState>,
    book_source_url: String,
) -> CmdResult<Value> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    get_source_variables(&conn, &book_source_url).map_err(|error| error.to_string())
}

#[tauri::command(async)]
pub async fn book_source_search(
    app: AppHandle,
    state: State<'_, AppState>,
    keyword: String,
    source_urls: Option<Vec<String>>,
    page: Option<i32>,
) -> CmdResult<Vec<Value>> {
    let keyword = keyword.trim().to_string();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }
    let mut sources = load_sources(&state)?;
    if let Some(urls) = source_urls.filter(|items| !items.is_empty()) {
        sources.retain(|source| urls.iter().any(|url| url == &source.book_source_url));
    }
    sources.retain(|source| {
        source.enabled
            && source
                .search_url
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
    });
    if sources.is_empty() {
        return Ok(Vec::new());
    }
    if let Ok(conn) = state.conn.lock() {
        let _ = save_search_history(&conn, &keyword);
    }
    let page = page.unwrap_or(1).max(1);
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Value>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        let mut results = Vec::new();
        for source in sources {
            let source_url = source.book_source_url.clone();
            let source_name = source.book_source_name.clone();
            match runtime.block_on(pipeline.search(&source, &keyword, page)) {
                Ok(books) => {
                    for book in books {
                        let mut value =
                            serde_json::to_value(book).map_err(|error| error.to_string())?;
                        value["bookSourceUrl"] = Value::String(source_url.clone());
                        value["bookSourceName"] = Value::String(source_name.clone());
                        results.push(value);
                    }
                }
                Err(error) => {
                    results.push(serde_json::json!({
                        "bookSourceUrl": source_url,
                        "bookSourceName": source_name,
                        "error": error.to_string(),
                    }));
                }
            }
        }
        Ok(results)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_info(
    app: AppHandle,
    state: State<'_, AppState>,
    book_source_url: String,
    book_url: String,
) -> CmdResult<Value> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookSourceRepo::get(&conn, &book_source_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?
    };
    let mut book = Book {
        book_url,
        origin: source.book_source_url.clone(),
        origin_name: source.book_source_name.clone(),
        ..Default::default()
    };
    tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        runtime
            .block_on(pipeline.book_info(&source, &mut book))
            .map_err(|error| error.to_string())?;
        serde_json::to_value(book).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_toc(
    app: AppHandle,
    state: State<'_, AppState>,
    book_source_url: String,
    book: Value,
) -> CmdResult<Vec<Value>> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookSourceRepo::get(&conn, &book_source_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?
    };
    let mut book: Book = serde_json::from_value(book).map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Value>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        let chapters = runtime
            .block_on(pipeline.chapters(&source, &mut book))
            .map_err(|error| error.to_string())?;
        chapters
            .into_iter()
            .map(|chapter| serde_json::to_value(chapter).map_err(|error| error.to_string()))
            .collect()
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_content(
    app: AppHandle,
    state: State<'_, AppState>,
    book_source_url: String,
    book: Value,
    chapter: Value,
    next_chapter_url: Option<String>,
) -> CmdResult<String> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookSourceRepo::get(&conn, &book_source_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?
    };
    let mut book: Book = serde_json::from_value(book).map_err(|error| error.to_string())?;
    let mut chapter: BookChapter =
        serde_json::from_value(chapter).map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        runtime
            .block_on(pipeline.content(
                &source,
                &mut book,
                &mut chapter,
                next_chapter_url.as_deref(),
            ))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_add_to_shelf(
    app: AppHandle,
    state: State<'_, AppState>,
    book_source_url: String,
    book_url: String,
    group: Option<u64>,
) -> CmdResult<Value> {
    let source = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookSourceRepo::get(&conn, &book_source_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?
    };
    let group = group.unwrap_or(0);
    let source_type = source.book_source_type;
    let source_url = source.book_source_url.clone();
    let source_name = source.book_source_name.clone();
    let (engine_book, engine_chapters) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(Book, Vec<BookChapter>), String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let pipeline = create_pipeline(&app)?;
            let mut book = Book {
                book_url,
                origin: source.book_source_url.clone(),
                origin_name: source.book_source_name.clone(),
                ..Default::default()
            };
            runtime
                .block_on(pipeline.book_info(&source, &mut book))
                .map_err(|error| error.to_string())?;
            let chapters = runtime
                .block_on(pipeline.chapters(&source, &mut book))
                .map_err(|error| error.to_string())?;
            Ok((book, chapters))
        })
        .await
        .map_err(|error| error.to_string())??;

    let book = AppBook {
        book_url: engine_book.book_url,
        name: engine_book.name,
        author: engine_book.author,
        origin: source_url,
        origin_name: source_name,
        charset: None,
        r#type: 1u64 << source_type.clamp(0, 3),
        group,
        toc_url: engine_book.toc_url,
        intro: engine_book.intro,
        kind: engine_book.kind,
        cover_url: engine_book.cover_url,
        latest_chapter_title: engine_book.latest_chapter_title,
        total_chapter_num: engine_chapters.len() as i32,
        can_update: true,
        sync_time: chrono::Utc::now().timestamp(),
        ..Default::default()
    };
    let chapters = engine_chapters
        .into_iter()
        .map(|chapter| AppChapter {
            url: chapter.url,
            title: chapter.title,
            index: chapter.index,
            is_volume: chapter.is_volume,
            is_vip: chapter.is_vip,
            is_pay: chapter.is_pay,
            tag: chapter.tag,
            resource_url: chapter.resource_url,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookRepo::upsert(&conn, &book).map_err(|error| error.to_string())?;
        ChapterRepo::replace_all(&conn, &book.book_url, &chapters)
            .map_err(|error| error.to_string())?;
    }
    serde_json::to_value(book).map_err(|error| error.to_string())
}

#[tauri::command(async)]
pub async fn book_source_refresh(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
) -> CmdResult<Value> {
    let (mut app_book, source) = {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        let book = BookRepo::get(&conn, &book_url)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书籍不存在".to_string())?;
        if book.origin.trim().is_empty() {
            return Err("书籍没有关联书源".to_string());
        }
        let source = BookSourceRepo::get(&conn, &book.origin)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "书源不存在".to_string())?;
        (book, source)
    };
    let source_url = source.book_source_url.clone();
    let source_name = source.book_source_name.clone();
    let book_for_fetch = app_book.clone();
    let (engine_book, engine_chapters) =
        tauri::async_runtime::spawn_blocking(move || -> Result<(Book, Vec<BookChapter>), String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let pipeline = create_pipeline(&app)?;
            let mut book = Book {
                book_url: book_for_fetch.book_url.clone(),
                toc_url: book_for_fetch.toc_url.clone(),
                origin: book_for_fetch.origin.clone(),
                origin_name: book_for_fetch.origin_name.clone(),
                name: book_for_fetch.name.clone(),
                author: book_for_fetch.author.clone(),
                kind: book_for_fetch.kind.clone(),
                cover_url: book_for_fetch.cover_url.clone(),
                intro: book_for_fetch.intro.clone(),
                book_type: book_for_fetch.r#type as i32,
                latest_chapter_title: book_for_fetch.latest_chapter_title.clone(),
                total_chapter_num: book_for_fetch.total_chapter_num,
                ..Default::default()
            };
            runtime
                .block_on(pipeline.book_info(&source, &mut book))
                .map_err(|error| error.to_string())?;
            let chapters = runtime
                .block_on(pipeline.chapters(&source, &mut book))
                .map_err(|error| error.to_string())?;
            Ok((book, chapters))
        })
        .await
        .map_err(|error| error.to_string())??;

    app_book.origin = source_url;
    app_book.origin_name = source_name;
    app_book.name = engine_book.name;
    app_book.author = engine_book.author;
    app_book.toc_url = engine_book.toc_url;
    app_book.intro = engine_book.intro;
    app_book.kind = engine_book.kind;
    app_book.cover_url = engine_book.cover_url;
    app_book.latest_chapter_title = engine_book.latest_chapter_title;
    app_book.total_chapter_num = engine_chapters.len() as i32;
    app_book.can_update = true;
    app_book.sync_time = chrono::Utc::now().timestamp();
    let chapters = engine_chapters
        .into_iter()
        .map(|chapter| AppChapter {
            url: chapter.url,
            title: chapter.title,
            index: chapter.index,
            is_volume: chapter.is_volume,
            is_vip: chapter.is_vip,
            is_pay: chapter.is_pay,
            tag: chapter.tag,
            resource_url: chapter.resource_url,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    {
        let conn = state.conn.lock().map_err(|error| error.to_string())?;
        BookRepo::upsert(&conn, &app_book).map_err(|error| error.to_string())?;
        ChapterRepo::replace_all(&conn, &app_book.book_url, &chapters)
            .map_err(|error| error.to_string())?;
        conn.execute(
            "DELETE FROM kv WHERE key LIKE ?1",
            [format!("chapterOverride:{}:%", app_book.book_url)],
        )
        .map_err(|error| error.to_string())?;
    }
    if let Ok(mut cache) = state.reader_cache.lock() {
        cache.invalidate_book(&app_book.book_url);
    }
    serde_json::to_value(app_book).map_err(|error| error.to_string())
}

fn load_online_context(
    state: &State<'_, AppState>,
    book_url: &str,
) -> Result<(AppBook, Vec<AppChapter>, BookSource), String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let book = BookRepo::get(&conn, book_url)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "书籍不存在".to_string())?;
    if book.origin.trim().is_empty() {
        return Err("书籍没有关联书源".to_string());
    }
    let source = BookSourceRepo::get(&conn, &book.origin)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "书源不存在".to_string())?;
    let chapters = ChapterRepo::list(&conn, book_url).map_err(|error| error.to_string())?;
    Ok((book, chapters, source))
}

#[tauri::command(async)]
pub async fn book_source_reviews(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
    chapter_index: Option<i32>,
) -> CmdResult<Vec<Value>> {
    let (book, chapters, source) = load_online_context(&state, &book_url)?;
    let index = chapter_index.unwrap_or(book.dur_chapter_index);
    let chapter = chapters
        .iter()
        .find(|chapter| chapter.index == index)
        .cloned()
        .unwrap_or_else(|| AppChapter {
            title: book.name.clone(),
            index,
            ..Default::default()
        });
    let mut engine_book = engine_book(&book);
    let mut engine_chapter = engine_chapter(&book, &chapter);
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<Value>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        let reviews = runtime
            .block_on(pipeline.reviews(&source, &mut engine_book, &mut engine_chapter))
            .map_err(|error| error.to_string())?;
        reviews
            .into_iter()
            .map(|review| serde_json::to_value(review).map_err(|error| error.to_string()))
            .collect()
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_download_urls(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
) -> CmdResult<Vec<String>> {
    let (book, _chapters, source) = load_online_context(&state, &book_url)?;
    let mut engine_book = engine_book(&book);
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<String>, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let pipeline = create_pipeline(&app)?;
        runtime
            .block_on(pipeline.download_urls(&source, &mut engine_book))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn book_source_pay(
    app: AppHandle,
    state: State<'_, AppState>,
    book_url: String,
    chapter_index: Option<i32>,
) -> CmdResult<crate::webview_bridge::PayActionResponse> {
    let (book, chapters, source) = load_online_context(&state, &book_url)?;
    let index = chapter_index.unwrap_or(book.dur_chapter_index);
    let chapter = chapters
        .iter()
        .find(|chapter| chapter.index == index)
        .cloned()
        .ok_or_else(|| "章节不存在".to_string())?;
    let source_value = serde_json::to_value(source).map_err(|error| error.to_string())?;
    let book_value =
        serde_json::to_value(engine_book(&book)).map_err(|error| error.to_string())?;
    let chapter_value =
        serde_json::to_value(engine_chapter(&book, &chapter)).map_err(|error| error.to_string())?;
    crate::webview_bridge::execute_pay_action(app, source_value, book_value, chapter_value)
}

#[tauri::command(async)]
pub async fn book_source_save_cookies(
    app: AppHandle,
    source_url: String,
    cookies: Vec<String>,
    cookie_url: Option<String>,
) -> CmdResult<usize> {
    let count = cookies.len();
    let header = cookies.join("; ");
    let target = cookie_url.unwrap_or_else(|| source_url.clone());
    tauri::async_runtime::spawn_blocking(move || -> Result<usize, String> {
        let pipeline = create_pipeline(&app)?;
        pipeline.replace_cookie(&target, &header);
        pipeline.replace_cookie(&source_url, &header);
        Ok(count)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(async)]
pub async fn book_source_clear_cookies(
    app: AppHandle,
    source_url: String,
) -> CmdResult<()> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let pipeline = create_pipeline(&app)?;
        pipeline.remove_cookie(&source_url);
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}
