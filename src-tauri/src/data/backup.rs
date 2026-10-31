use backup_core::BackupFile;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

use crate::data::keyvalue::{is_whitelisted_key, KeyValue};
use crate::data::repositories::{
    BookRepo, BookmarkRepo, DictRuleRepo, GroupRepo, HttpTtsRepo, ReplaceRuleRepo, TxtTocRuleRepo,
};
use crate::domain::model::{Book, BookGroup, Bookmark, DictRule, HttpTts, ReplaceRule, TxtTocRule};
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupReport {
    pub path: String,
    pub files: usize,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub files: usize,
    pub items: usize,
    pub skipped: Vec<String>,
    pub settings_json: Option<String>,
}

fn write_json(
    files: &mut Vec<BackupFile>,
    name: &str,
    json: &str,
    count: &mut usize,
    items: usize,
) -> Result<()> {
    if json == "[]" || json == "{}" || json.is_empty() {
        return Ok(());
    }
    files.push(BackupFile {
        name: name.to_string(),
        bytes: json.as_bytes().to_vec(),
    });
    *count += 1;
    let _ = items;
    Ok(())
}

fn json_value_rows(conn: &Connection, sql: &str) -> Result<Vec<Value>> {
    let mut stmt = conn.prepare(sql)?;
    let count = stmt.column_count();
    let names: Vec<String> = (0..count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let mut map = serde_json::Map::new();
        for i in 0..count {
            let value: rusqlite::types::Value = row.get(i)?;
            let json = match value {
                rusqlite::types::Value::Null => Value::Null,
                rusqlite::types::Value::Integer(v) => Value::from(v),
                rusqlite::types::Value::Real(v) => Value::from(v),
                rusqlite::types::Value::Text(v) => Value::from(v),
                rusqlite::types::Value::Blob(v) => Value::from(v),
            };
            map.insert(names[i].clone(), json);
        }
        out.push(Value::Object(map));
    }
    Ok(out)
}

pub fn export_backup(
    conn: &Connection,
    path: &str,
    password: Option<&str>,
    settings_json: Option<&str>,
) -> Result<BackupReport> {
    let mut backup_files = Vec::new();
    let mut files = 0usize;
    let mut items = 0usize;

    let books = BookRepo::list(conn)?;
    items += books.len();
    let books_json = serde_json::to_string(&books)?;
    write_json(
        &mut backup_files,
        "books.json",
        &books_json,
        &mut files,
        books.len(),
    )?;
    write_json(
        &mut backup_files,
        "bookshelf.json",
        &books_json,
        &mut files,
        books.len(),
    )?;

    let groups = GroupRepo::list(conn)?;
    items += groups.len();
    let groups_json = serde_json::to_string(&groups)?;
    write_json(
        &mut backup_files,
        "book_groups.json",
        &groups_json,
        &mut files,
        groups.len(),
    )?;
    write_json(
        &mut backup_files,
        "bookGroup.json",
        &groups_json,
        &mut files,
        groups.len(),
    )?;

    let bookmarks = BookmarkRepo::list(conn)?;
    items += bookmarks.len();
    let bookmarks_json = serde_json::to_string(&bookmarks)?;
    write_json(
        &mut backup_files,
        "bookmarks.json",
        &bookmarks_json,
        &mut files,
        bookmarks.len(),
    )?;
    write_json(
        &mut backup_files,
        "bookmark.json",
        &bookmarks_json,
        &mut files,
        bookmarks.len(),
    )?;

    let toc_rules = TxtTocRuleRepo::list(conn)?;
    items += toc_rules.len();
    let toc_rules_json = serde_json::to_string(&toc_rules)?;
    write_json(
        &mut backup_files,
        "txtTocRules.json",
        &toc_rules_json,
        &mut files,
        toc_rules.len(),
    )?;
    write_json(
        &mut backup_files,
        "txtTocRule.json",
        &toc_rules_json,
        &mut files,
        toc_rules.len(),
    )?;

    let replace_rules = ReplaceRuleRepo::list(conn)?;
    items += replace_rules.len();
    let replace_rules_json = serde_json::to_string(&replace_rules)?;
    write_json(
        &mut backup_files,
        "replace_rules.json",
        &replace_rules_json,
        &mut files,
        replace_rules.len(),
    )?;
    write_json(
        &mut backup_files,
        "replaceRule.json",
        &replace_rules_json,
        &mut files,
        replace_rules.len(),
    )?;

    let dict_rules = DictRuleRepo::list(conn)?;
    items += dict_rules.len();
    let dict_rules_json = serde_json::to_string(&dict_rules)?;
    write_json(
        &mut backup_files,
        "dictRules.json",
        &dict_rules_json,
        &mut files,
        dict_rules.len(),
    )?;
    write_json(
        &mut backup_files,
        "dictRule.json",
        &dict_rules_json,
        &mut files,
        dict_rules.len(),
    )?;

    let tts = HttpTtsRepo::list(conn)?;
    items += tts.len();
    write_json(
        &mut backup_files,
        "httpTTS.json",
        &serde_json::to_string(&tts)?,
        &mut files,
        tts.len(),
    )?;

    let read_records = json_value_rows(conn, "SELECT * FROM readRecord")?;
    items += read_records.len();
    write_json(
        &mut backup_files,
        "readRecord.json",
        &serde_json::to_string(&read_records)?,
        &mut files,
        read_records.len(),
    )?;

    let schemes = json_value_rows(conn, "SELECT * FROM read_style_schemes ORDER BY seqNo")?;
    items += schemes.len();
    write_json(
        &mut backup_files,
        "read_style_schemes.json",
        &serde_json::to_string(&schemes)?,
        &mut files,
        schemes.len(),
    )?;

    let themes = json_value_rows(conn, "SELECT * FROM themes")?;
    items += themes.len();
    write_json(
        &mut backup_files,
        "themes.json",
        &serde_json::to_string(&themes)?,
        &mut files,
        themes.len(),
    )?;

    let config = KeyValue::all(conn)?;
    if !config.is_empty() {
        let xml = backup_core::write_config_xml(&config, password);
        write_json(
            &mut backup_files,
            "config.xml",
            &xml,
            &mut files,
            config.len(),
        )?;
    }

    if let Some(settings) = settings_json {
        if !settings.trim().is_empty() {
            write_json(
                &mut backup_files,
                "local-reader-settings.json",
                settings,
                &mut files,
                1,
            )?;
        }
    }

    let bytes = backup_core::pack(&backup_files).map_err(Error::Io)?;
    std::fs::write(path, bytes).map_err(|e| Error::Io(e.to_string()))?;
    Ok(BackupReport {
        path: path.to_string(),
        files,
        items,
    })
}

fn read_zip_entries(path: &str) -> Result<Vec<(String, String)>> {
    let data = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
    let files = backup_core::unpack(&data).map_err(|_| Error::Invalid("备份文件无效".into()))?;
    Ok(files
        .into_iter()
        .map(|file| (file.name, String::from_utf8_lossy(&file.bytes).to_string()))
        .collect())
}

fn restore_config(
    conn: &Connection,
    xml: &str,
    password: Option<&str>,
    ignore: &[String],
) -> Result<usize> {
    let pairs = backup_core::parse_config_xml(xml, password);
    let mut count = 0;
    for entry in pairs {
        if ignore.iter().any(|item| item == &entry.key) || !is_whitelisted_key(&entry.key) {
            continue;
        }
        if entry.encrypted && !entry.decrypted {
            continue;
        }
        KeyValue::set(conn, &entry.key, &entry.value)?;
        count += 1;
    }
    Ok(count)
}

pub fn import_backup(
    conn: &Connection,
    path: &str,
    password: Option<&str>,
    ignore: &[String],
) -> Result<RestoreReport> {
    let entries = read_zip_entries(path)?;
    let mut files = 0usize;
    let mut items = 0usize;
    let mut skipped = Vec::new();
    let mut settings_json = None;

    for (name, text) in entries {
        if ignore.iter().any(|i| i == &name) {
            skipped.push(name);
            continue;
        }
        match name.as_str() {
            "books.json" | "bookshelf.json" => {
                if ignore.iter().any(|i| i == "localBooks") {
                    continue;
                }
                let books: Vec<Book> = serde_json::from_str(&text)?;
                for book in &books {
                    BookRepo::upsert(conn, book)?;
                }
                items += books.len();
                files += 1;
            }
            "book_groups.json" | "bookGroup.json" => {
                let mut values: Vec<Value> = serde_json::from_str(&text)?;
                for value in &mut values {
                    if let Some(map) = value.as_object_mut() {
                        if !map.contains_key("coverUrl") {
                            if let Some(cover) = map.get("cover").cloned() {
                                map.insert("coverUrl".into(), cover);
                            }
                        }
                    }
                }
                let groups: Vec<BookGroup> = values
                    .into_iter()
                    .filter_map(|value| serde_json::from_value(value).ok())
                    .collect();
                for group in &groups {
                    GroupRepo::upsert(conn, group)?;
                }
                items += groups.len();
                files += 1;
            }
            "bookmarks.json" | "bookmark.json" => {
                let bookmarks: Vec<Bookmark> = serde_json::from_str(&text)?;
                for bookmark in &bookmarks {
                    BookmarkRepo::upsert(conn, bookmark)?;
                }
                items += bookmarks.len();
                files += 1;
            }
            "txtTocRules.json" | "txtTocRule.json" => {
                let rules: Vec<TxtTocRule> = serde_json::from_str(&text)?;
                for rule in &rules {
                    TxtTocRuleRepo::upsert(conn, rule)?;
                }
                items += rules.len();
                files += 1;
            }
            "replace_rules.json" | "replaceRule.json" => {
                let rules: Vec<ReplaceRule> = serde_json::from_str(&text)?;
                for rule in &rules {
                    ReplaceRuleRepo::upsert(conn, rule)?;
                }
                items += rules.len();
                files += 1;
            }
            "dictRules.json" | "dictRule.json" => {
                let rules: Vec<DictRule> = serde_json::from_str(&text)?;
                for rule in &rules {
                    DictRuleRepo::upsert(conn, rule)?;
                }
                items += rules.len();
                files += 1;
            }
            "httpTTS.json" => {
                let engines: Vec<HttpTts> = serde_json::from_str(&text)?;
                for engine in &engines {
                    HttpTtsRepo::upsert(conn, engine)?;
                }
                items += engines.len();
                files += 1;
            }
            "readRecord.json" => {
                let rows: Vec<Value> = serde_json::from_str(&text)?;
                for row in &rows {
                    let device = row.get("deviceId").and_then(|v| v.as_str()).unwrap_or("");
                    let book_name = row.get("bookName").and_then(|v| v.as_str()).unwrap_or("");
                    if device.is_empty() || book_name.is_empty() {
                        continue;
                    }
                    let cloud_time = row.get("readTime").and_then(|v| v.as_i64()).unwrap_or(0);
                    let local_time: Option<i64> = conn
                        .query_row(
                            "SELECT readTime FROM readRecord WHERE deviceId=?1 AND bookName=?2",
                            params![device, book_name],
                            |r| r.get(0),
                        )
                        .optional()?;
                    if local_time.unwrap_or(0) > cloud_time {
                        continue;
                    }
                    conn.execute(
                        r#"INSERT OR REPLACE INTO readRecord
                           (deviceId,bookName,author,readTime,lastRead,durChapterIndex,durChapterPos,durChapterTitle)
                           VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                        params![
                            device,
                            book_name,
                            row.get("author").and_then(|v| v.as_str()).unwrap_or(""),
                            cloud_time,
                            row.get("lastRead").and_then(|v| v.as_i64()).unwrap_or(0),
                            row.get("durChapterIndex").and_then(|v| v.as_i64()).unwrap_or(0),
                            row.get("durChapterPos").and_then(|v| v.as_i64()).unwrap_or(0),
                            row.get("durChapterTitle").and_then(|v| v.as_str()).unwrap_or(""),
                        ],
                    )?;
                    items += 1;
                }
                files += 1;
            }
            "read_style_schemes.json" => {
                let rows: Vec<Value> = serde_json::from_str(&text)?;
                for row in &rows {
                    conn.execute(
                        "INSERT OR REPLACE INTO read_style_schemes (seqNo,name,payload) VALUES (?1,?2,?3)",
                        params![
                            row.get("seqNo").and_then(|v| v.as_i64()).unwrap_or(0),
                            row.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            row.get("payload").and_then(|v| v.as_str()).unwrap_or("{}"),
                        ],
                    )?;
                    items += 1;
                }
                files += 1;
            }
            "themes.json" => {
                let rows: Vec<Value> = serde_json::from_str(&text)?;
                for row in &rows {
                    conn.execute(
                        r#"INSERT OR REPLACE INTO themes
                           (themeName,isNightTheme,primaryColor,accentColor,backgroundColor,bottomBackground)
                           VALUES (?1,?2,?3,?4,?5,?6)"#,
                        params![
                            row.get("themeName").and_then(|v| v.as_str()).unwrap_or(""),
                            row.get("isNightTheme").and_then(|v| v.as_i64()).unwrap_or(0),
                            row.get("primaryColor").and_then(|v| v.as_str()).unwrap_or(""),
                            row.get("accentColor").and_then(|v| v.as_str()).unwrap_or(""),
                            row.get("backgroundColor").and_then(|v| v.as_str()).unwrap_or(""),
                            row.get("bottomBackground").and_then(|v| v.as_str()).unwrap_or(""),
                        ],
                    )?;
                    items += 1;
                }
                files += 1;
            }
            "local-reader-settings.json" => {
                settings_json = Some(text);
                files += 1;
            }
            "config.xml" => {
                items += restore_config(conn, &text, password, ignore)?;
                files += 1;
            }
            _ => skipped.push(name),
        }
    }

    Ok(RestoreReport {
        files,
        items,
        skipped,
        settings_json,
    })
}
