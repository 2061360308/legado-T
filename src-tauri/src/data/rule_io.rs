//! 规则 JSON 导入导出、旧字段映射与导入对比（T4.1~T4.3）。
//!
//! 字段名与 UTF-8 编码对齐 legado；旧字段（`regex`/`replaceSummary`/`useTo`/`enable`/
//! `serialNumber`）在导入时归一化为新字段。

use std::collections::HashSet;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::domain::model::{DictRule, HttpTts, ReplaceRule, TxtTocRule};
use crate::{Error, Result};

/// 导入对比状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffStatus {
    /// 本批次新增。
    New,
    /// 已存在同键，被覆盖更新。
    Updated,
    /// 输入内重复，已合并。
    Duplicate,
}

/// 单条对比结果。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffEntry {
    pub key: String,
    pub status: DiffStatus,
}

/// 导入对比报告。
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffReport {
    pub entries: Vec<DiffEntry>,
}

impl DiffReport {
    pub fn count(&self, status: DiffStatus) -> usize {
        self.entries.iter().filter(|e| e.status == status).count()
    }

    /// 默认仅勾选新增项（spec 5.9.1-3）。
    pub fn new_keys(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.status == DiffStatus::New)
            .map(|e| e.key.as_str())
            .collect()
    }
}

/// 把旧字段搬运为对应新字段（仅当新字段缺失时）。
fn alias(v: &mut Value, new_key: &str, old_key: &str) {
    if v.get(new_key).is_none() {
        if let Some(x) = v.get(old_key).cloned() {
            if let Value::Object(map) = v {
                map.insert(new_key.to_string(), x);
            }
        }
    }
}

/// 归一化替换规则：`regex→pattern`、`replaceSummary→name`、`useTo→scope`、
/// `enable→isEnabled`、`serialNumber→sortOrder`（spec 4.5.3 / T4.3）。
pub fn normalize_replace_rule(v: &mut Value) {
    alias(v, "pattern", "regex");
    alias(v, "name", "replaceSummary");
    alias(v, "scope", "useTo");
    alias(v, "isEnabled", "enable");
    alias(v, "sortOrder", "serialNumber");
}

/// 归一化 HttpTTS：缺失 `id`/`lastUpdateTime` 取当前时间（spec 6.9-6）。
pub fn normalize_http_tts(v: &mut Value, now_ms: i64) {
    let last = v
        .get("lastUpdateTime")
        .and_then(|x| x.as_i64())
        .unwrap_or(0);
    let effective_last = if last > 0 { last } else { now_ms };
    if v.get("lastUpdateTime").is_none() {
        if let Value::Object(map) = v {
            map.insert("lastUpdateTime".into(), Value::from(effective_last));
        }
    }
    let id = v.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
    if id <= 0 {
        if let Value::Object(map) = v {
            map.insert("id".into(), Value::from(effective_last));
        }
    }
}

fn parse_array(json: &str) -> Result<Vec<Value>> {
    let v: Value = serde_json::from_str(json)?;
    match v {
        Value::Array(items) => Ok(items),
        Value::Object(_) => Ok(vec![v]),
        _ => Err(Error::Invalid("规则 JSON 必须是数组或对象".into())),
    }
}

fn from_value<T: DeserializeOwned>(v: Value) -> Result<T> {
    serde_json::from_value(v).map_err(|e| Error::Invalid(e.to_string()))
}

/// 导入替换规则，返回归一化后的规则与对比报告。
pub fn import_replace_rules(
    json: &str,
    existing_ids: &HashSet<i64>,
) -> Result<(Vec<ReplaceRule>, DiffReport)> {
    let mut report = DiffReport::default();
    let mut seen: HashSet<i64> = HashSet::new();
    let mut out = Vec::new();
    for mut item in parse_array(json)? {
        normalize_replace_rule(&mut item);
        let rule: ReplaceRule = from_value(item)?;
        let status = if !seen.insert(rule.id) {
            DiffStatus::Duplicate
        } else if existing_ids.contains(&rule.id) {
            DiffStatus::Updated
        } else {
            DiffStatus::New
        };
        report.entries.push(DiffEntry {
            key: rule.id.to_string(),
            status,
        });
        if status != DiffStatus::Duplicate {
            out.push(rule);
        }
    }
    Ok((out, report))
}

/// 导入目录规则。
pub fn import_txt_toc_rules(
    json: &str,
    existing_ids: &HashSet<i64>,
) -> Result<(Vec<TxtTocRule>, DiffReport)> {
    let mut report = DiffReport::default();
    let mut seen: HashSet<i64> = HashSet::new();
    let mut out = Vec::new();
    for item in parse_array(json)? {
        let rule: TxtTocRule = from_value(item)?;
        let status = if !seen.insert(rule.id) {
            DiffStatus::Duplicate
        } else if existing_ids.contains(&rule.id) {
            DiffStatus::Updated
        } else {
            DiffStatus::New
        };
        report.entries.push(DiffEntry {
            key: rule.id.to_string(),
            status,
        });
        if status != DiffStatus::Duplicate {
            out.push(rule);
        }
    }
    Ok((out, report))
}

/// 导入字典规则（以 name 为去重键，同名覆盖）。
pub fn import_dict_rules(
    json: &str,
    existing_names: &HashSet<String>,
) -> Result<(Vec<DictRule>, DiffReport)> {
    let mut report = DiffReport::default();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for item in parse_array(json)? {
        let rule: DictRule = from_value(item)?;
        let status = if !seen.insert(rule.name.clone()) {
            DiffStatus::Duplicate
        } else if existing_names.contains(&rule.name) {
            DiffStatus::Updated
        } else {
            DiffStatus::New
        };
        report.entries.push(DiffEntry {
            key: rule.name.clone(),
            status,
        });
        if status != DiffStatus::Duplicate {
            out.push(rule);
        }
    }
    Ok((out, report))
}

/// 导入在线语音引擎。
pub fn import_http_tts(
    json: &str,
    existing_ids: &HashSet<i64>,
    now_ms: i64,
) -> Result<(Vec<HttpTts>, DiffReport)> {
    let mut report = DiffReport::default();
    let mut seen: HashSet<i64> = HashSet::new();
    let mut out = Vec::new();
    for mut item in parse_array(json)? {
        normalize_http_tts(&mut item, now_ms);
        let rule: HttpTts = from_value(item)?;
        let status = if !seen.insert(rule.id) {
            DiffStatus::Duplicate
        } else if existing_ids.contains(&rule.id) {
            DiffStatus::Updated
        } else {
            DiffStatus::New
        };
        report.entries.push(DiffEntry {
            key: rule.id.to_string(),
            status,
        });
        if status != DiffStatus::Duplicate {
            out.push(rule);
        }
    }
    Ok((out, report))
}

/// 导出规则为 legado 可识别的 JSON（UTF-8）。
pub fn export_rules<T: Serialize>(rules: &[T]) -> Result<String> {
    serde_json::to_string_pretty(rules).map_err(Error::from)
}
