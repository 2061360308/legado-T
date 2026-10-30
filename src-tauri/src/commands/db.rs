//! 数据访问 IPC 命令（对前端暴露）。
//!
//! 参数与返回值一律强类型；错误以字符串归一化返回。

use std::fs;
use std::path::Path;

use md5::{Digest, Md5};
use tauri::State;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{
    BookRepo, ChapterRepo, DictRuleRepo, GroupRepo, HttpTtsRepo, ReplaceRuleRepo, TxtTocRuleRepo,
};
use crate::domain::model::{
    Book, BookChapter, BookGroup, DictRule, HttpTts, ReplaceRule, TxtTocRule,
};
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn book_list(state: State<AppState>) -> CmdResult<Vec<Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut books = BookRepo::list(&conn).map_err(|e| e.to_string())?;
    for book in &mut books {
        state.storage.hydrate_book(book);
    }
    Ok(books)
}

#[tauri::command]
pub fn book_get(state: State<AppState>, book_url: String) -> CmdResult<Option<Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut book = BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())?;
    if let Some(book) = book.as_mut() {
        state.storage.hydrate_book(book);
    }
    Ok(book)
}

#[tauri::command]
pub fn book_upsert(state: State<AppState>, mut book: Book) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    state.storage.normalize_book(&mut book);
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_delete(
    state: State<AppState>,
    book_url: String,
    delete_files: Option<bool>,
) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let book = if delete_files.unwrap_or(true) {
        BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())?
    } else {
        None
    };
    BookRepo::delete(&conn, &book_url).map_err(|e| e.to_string())?;
    if let Some(book) = book {
        state
            .storage
            .delete_managed(&book)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn chapter_list(state: State<AppState>, book_url: String) -> CmdResult<Vec<BookChapter>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    ChapterRepo::list(&conn, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn group_list(state: State<AppState>) -> CmdResult<Vec<BookGroup>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut groups = GroupRepo::list(&conn).map_err(|e| e.to_string())?;
    for group in &mut groups {
        group.cover_url = group
            .cover_url
            .as_deref()
            .map(|value| state.storage.resolve(value));
    }
    Ok(groups)
}

#[tauri::command]
pub fn group_save(state: State<AppState>, mut group: BookGroup) -> CmdResult<BookGroup> {
    group.group_name = group.group_name.trim().to_string();
    if group.group_name.is_empty() {
        return Err("分组名称不能为空".into());
    }
    group.cover_url = group
        .cover_url
        .as_deref()
        .map(|value| state.storage.normalize(value));
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let groups = GroupRepo::list(&conn).map_err(|e| e.to_string())?;
    if groups.iter().any(|item| {
        item.group_id != group.group_id
            && item.group_id > 0
            && item.group_name.eq_ignore_ascii_case(&group.group_name)
    }) {
        return Err("分组名称已存在".into());
    }
    if group.group_id == 0 {
        let used = groups
            .iter()
            .filter(|item| item.group_id > 0)
            .fold(0u64, |value, item| value | item.group_id);
        let mut group_id = 1u64;
        while group_id & used != 0 {
            group_id <<= 1;
        }
        if group_id == 0 {
            return Err("分组数量已达上限".into());
        }
        group.group_id = group_id;
        group.order = groups
            .iter()
            .map(|item| item.order)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        group.show = true;
    } else if !groups.iter().any(|item| item.group_id == group.group_id) {
        return Err("分组不存在".into());
    }
    GroupRepo::upsert(&conn, &group).map_err(|e| e.to_string())?;
    let mut result = group;
    result.cover_url = result
        .cover_url
        .as_deref()
        .map(|value| state.storage.resolve(value));
    Ok(result)
}

#[tauri::command]
pub fn group_delete(state: State<AppState>, group_id: u64) -> CmdResult<()> {
    if group_id == 0 {
        return Err("无效分组".into());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    GroupRepo::delete(&conn, group_id).map_err(|e| e.to_string())?;
    conn.execute(
        r#"UPDATE books SET "group" = "group" & ?1"#,
        [!(group_id as i64)],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn group_cover_save(state: State<AppState>, src_path: String) -> CmdResult<String> {
    let src = Path::new(&src_path);
    if !src.is_file() {
        return Err(format!("图片不存在: {src_path}"));
    }
    let bytes = fs::read(src).map_err(|e| e.to_string())?;
    let ext = src
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| value.chars().all(|ch| ch.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "jpg".to_string());
    let digest = Md5::digest(&bytes);
    let dir = state.storage.root().join("groupCovers");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{digest:x}.{ext}"));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(state.storage.resolve(&state.storage.relative_string(&path)))
}

#[tauri::command]
pub fn txt_toc_rule_list(state: State<AppState>) -> CmdResult<Vec<TxtTocRule>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    TxtTocRuleRepo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn kv_get(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    KeyValue::get(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn kv_set(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    KeyValue::set(&conn, &key, &value).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn replace_rule_list(state: State<AppState>) -> CmdResult<Vec<ReplaceRule>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    ReplaceRuleRepo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dict_rule_list(state: State<AppState>) -> CmdResult<Vec<DictRule>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    DictRuleRepo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn http_tts_list(state: State<AppState>) -> CmdResult<Vec<HttpTts>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    HttpTtsRepo::list(&conn).map_err(|e| e.to_string())
}
