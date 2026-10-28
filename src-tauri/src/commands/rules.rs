use std::collections::HashSet;

use tauri::{Manager, State};

use crate::data::default_data;
use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{DictRuleRepo, HttpTtsRepo, ReplaceRuleRepo, TxtTocRuleRepo};
use crate::data::rule_io::{
    self, import_dict_rules, import_http_tts, import_replace_rules, import_txt_toc_rules,
    DiffReport,
};
use crate::domain::model::{DictRule, HttpTts, ReplaceRule, TxtTocRule};
use crate::state::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn next_id() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn txt_toc_rule_save(state: State<AppState>, mut rule: TxtTocRule) -> CmdResult<TxtTocRule> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if rule.id <= 0 {
        rule.id = next_id();
    }
    if rule.serial_number == 0 {
        rule.serial_number = (TxtTocRuleRepo::count(&conn).map_err(|e| e.to_string())? as i32) + 1;
    }
    TxtTocRuleRepo::upsert(&conn, &rule).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
pub fn txt_toc_rule_delete(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    TxtTocRuleRepo::delete(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn txt_toc_rule_reorder(state: State<AppState>, ids: Vec<i64>) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let existing: std::collections::HashMap<i64, TxtTocRule> = TxtTocRuleRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| (r.id, r))
        .collect();
    let total = ids.len() as i32;
    for (i, id) in ids.iter().enumerate() {
        if let Some(rule) = existing.get(id) {
            let mut updated = rule.clone();
            updated.serial_number = total - i as i32;
            TxtTocRuleRepo::upsert(&conn, &updated).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn txt_toc_rule_restore_default(
    state: State<AppState>,
    app: tauri::AppHandle,
) -> CmdResult<usize> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let res_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let path = res_dir.join("defaultData").join("txtTocRule.json");
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let rules: Vec<TxtTocRule> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    TxtTocRuleRepo::delete_all(&conn).map_err(|e| e.to_string())?;
    for rule in &rules {
        TxtTocRuleRepo::upsert(&conn, rule).map_err(|e| e.to_string())?;
    }
    KeyValue::set(
        &conn,
        default_data::KV_DEFAULT_VERSION,
        &default_data::DEFAULT_DATA_VERSION.to_string(),
    )
    .map_err(|e| e.to_string())?;
    Ok(rules.len())
}

#[tauri::command]
pub fn txt_toc_rule_import(state: State<AppState>, json: String) -> CmdResult<DiffReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let existing: HashSet<i64> = TxtTocRuleRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let (rules, report) = import_txt_toc_rules(&json, &existing).map_err(|e| e.to_string())?;
    for rule in &rules {
        TxtTocRuleRepo::upsert(&conn, rule).map_err(|e| e.to_string())?;
    }
    Ok(report)
}

#[tauri::command]
pub fn txt_toc_rule_export(state: State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let rules = TxtTocRuleRepo::list(&conn).map_err(|e| e.to_string())?;
    rule_io::export_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn replace_rule_save(state: State<AppState>, mut rule: ReplaceRule) -> CmdResult<ReplaceRule> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if rule.id <= 0 {
        rule.id = next_id();
    }
    crate::domain::content_processor::validate_pattern(&rule).map_err(|e| e.to_string())?;
    ReplaceRuleRepo::upsert(&conn, &rule).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
pub fn replace_rule_delete(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    ReplaceRuleRepo::delete(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn replace_rule_import(state: State<AppState>, json: String) -> CmdResult<DiffReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let existing: HashSet<i64> = ReplaceRuleRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let (rules, report) = import_replace_rules(&json, &existing).map_err(|e| e.to_string())?;
    for rule in &rules {
        ReplaceRuleRepo::upsert(&conn, rule).map_err(|e| e.to_string())?;
    }
    Ok(report)
}

#[tauri::command]
pub fn replace_rule_export(state: State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let rules = ReplaceRuleRepo::list(&conn).map_err(|e| e.to_string())?;
    rule_io::export_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dict_rule_save(state: State<AppState>, rule: DictRule) -> CmdResult<DictRule> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    DictRuleRepo::upsert(&conn, &rule).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
pub fn dict_rule_delete(state: State<AppState>, name: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    DictRuleRepo::delete(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dict_rule_import(state: State<AppState>, json: String) -> CmdResult<DiffReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let existing: HashSet<String> = DictRuleRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| r.name)
        .collect();
    let (rules, report) = import_dict_rules(&json, &existing).map_err(|e| e.to_string())?;
    for rule in &rules {
        DictRuleRepo::upsert(&conn, rule).map_err(|e| e.to_string())?;
    }
    Ok(report)
}

#[tauri::command]
pub fn dict_rule_export(state: State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let rules = DictRuleRepo::list(&conn).map_err(|e| e.to_string())?;
    rule_io::export_rules(&rules).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn http_tts_save(state: State<AppState>, mut engine: HttpTts) -> CmdResult<HttpTts> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if engine.id <= 0 {
        engine.id = next_id();
    }
    if engine.last_update_time <= 0 {
        engine.last_update_time = next_id();
    }
    HttpTtsRepo::upsert(&conn, &engine).map_err(|e| e.to_string())?;
    Ok(engine)
}

#[tauri::command]
pub fn http_tts_delete(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    HttpTtsRepo::delete(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn http_tts_import(state: State<AppState>, json: String) -> CmdResult<DiffReport> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let existing: HashSet<i64> = HttpTtsRepo::list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let (engines, report) =
        import_http_tts(&json, &existing, next_id()).map_err(|e| e.to_string())?;
    for engine in &engines {
        HttpTtsRepo::upsert(&conn, engine).map_err(|e| e.to_string())?;
    }
    Ok(report)
}

#[tauri::command]
pub fn http_tts_export(state: State<AppState>) -> CmdResult<String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let engines = HttpTtsRepo::list(&conn).map_err(|e| e.to_string())?;
    rule_io::export_rules(&engines).map_err(|e| e.to_string())
}
