//! 数据访问 IPC 命令（对前端暴露）。
//!
//! 参数与返回值一律强类型；错误以字符串归一化返回。

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
    BookRepo::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_get(state: State<AppState>, book_url: String) -> CmdResult<Option<Book>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    BookRepo::get(&conn, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_upsert(state: State<AppState>, book: Book) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    BookRepo::upsert(&conn, &book).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn book_delete(state: State<AppState>, book_url: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    BookRepo::delete(&conn, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn chapter_list(state: State<AppState>, book_url: String) -> CmdResult<Vec<BookChapter>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    ChapterRepo::list(&conn, &book_url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn group_list(state: State<AppState>) -> CmdResult<Vec<BookGroup>> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    GroupRepo::list(&conn).map_err(|e| e.to_string())
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
