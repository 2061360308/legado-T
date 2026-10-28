//! 默认数据装载与版本升级（T4.4）。
//!
//! 首次启动目录规则表为空时插入 26 条默认规则；默认版本升高时仅补插新增、不覆盖用户修改。

use std::fs;

use rusqlite::Connection;

use crate::data::repositories::TxtTocRuleRepo;
use crate::domain::model::TxtTocRule;
use crate::Result;

/// 默认数据版本；随 `resources/defaultData` 资源变更递增。
pub const DEFAULT_DATA_VERSION: i32 = 1;

/// KV 键：已装载的默认数据版本。
pub const KV_DEFAULT_VERSION: &str = "defaultDataVersion";

/// 从 JSON 文本装载目录规则；仅当表为空时插入。
pub fn load_toc_rules_from_json(conn: &Connection, json: &str) -> Result<usize> {
    if TxtTocRuleRepo::count(conn)? > 0 {
        return Ok(0);
    }
    let rules: Vec<TxtTocRule> = serde_json::from_str(json)?;
    let n = rules.len();
    for r in &rules {
        TxtTocRuleRepo::upsert(conn, r)?;
    }
    Ok(n)
}

/// 从资源文件装载目录规则；文件缺失时返回 0。
pub fn load_toc_rules_from_file(conn: &Connection, path: &str) -> Result<usize> {
    let json = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Ok(0),
    };
    load_toc_rules_from_json(conn, &json)
}

/// 默认版本升级：仅补插新增的规则、不覆盖用户修改。
pub fn maybe_upgrade_default_rules(
    conn: &Connection,
    json: &str,
    current_version: i32,
) -> Result<usize> {
    use crate::data::keyvalue::KeyValue;

    let installed: i32 = KeyValue::get(conn, KV_DEFAULT_VERSION)?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if installed >= current_version {
        return Ok(0);
    }

    let bundled: Vec<TxtTocRule> = serde_json::from_str(json)?;
    let existing: std::collections::HashSet<i64> = TxtTocRuleRepo::list(conn)?
        .into_iter()
        .map(|r| r.id)
        .collect();

    let mut inserted = 0usize;
    for r in bundled {
        if !existing.contains(&r.id) {
            TxtTocRuleRepo::upsert(conn, &r)?;
            inserted += 1;
        }
    }
    KeyValue::set(conn, KV_DEFAULT_VERSION, &current_version.to_string())?;
    Ok(inserted)
}
