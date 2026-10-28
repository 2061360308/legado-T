use std::fs::File;
use std::io::{Read, Write};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::data::keyvalue::{is_whitelisted_key, KeyValue};
use crate::data::repositories::{
    BookRepo, BookmarkRepo, DictRuleRepo, GroupRepo, HttpTtsRepo, ReplaceRuleRepo, TxtTocRuleRepo,
};
use crate::domain::model::{Book, BookGroup, Bookmark, DictRule, HttpTts, ReplaceRule, TxtTocRule};
use crate::platform::zip_aes;
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
}

fn write_json<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    name: &str,
    json: &str,
    files: &mut usize,
    items: usize,
) -> Result<()> {
    if json == "[]" || json == "{}" || json.is_empty() {
        return Ok(());
    }
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(name, options)
        .map_err(|e| Error::Io(e.to_string()))?;
    zip.write_all(json.as_bytes())?;
    *files += 1;
    let _ = items;
    Ok(())
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn is_sensitive(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("password") || lower.contains("secret") || key.contains("密码")
}

fn write_config_xml(pairs: &[(String, String)], password: Option<&str>) -> String {
    let mut xml =
        String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\" ?>\n<map>\n");
    for (key, value) in pairs {
        if is_sensitive(key) {
            if let Some(pass) = password {
                let encrypted = zip_aes::encrypt(value.as_bytes(), pass);
                xml.push_str(&format!(
                    "  <string name=\"{}\" encrypted=\"true\">{}</string>\n",
                    xml_escape(key),
                    zip_aes::to_hex(&encrypted)
                ));
                continue;
            }
        }
        xml.push_str(&format!(
            "  <string name=\"{}\">{}</string>\n",
            xml_escape(key),
            xml_escape(value)
        ));
    }
    xml.push_str("</map>\n");
    xml
}

fn parse_config_xml(xml: &str) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("<string ") {
        rest = &rest[pos + 8..];
        let Some(tag_end) = rest.find('>') else { break };
        let tag = &rest[..tag_end];
        let encrypted = tag.contains("encrypted=\"true\"");
        let name = tag
            .split("name=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .map(xml_unescape)
            .unwrap_or_default();
        let body_start = tag_end + 1;
        let Some(body_end) = rest[body_start..].find("</string>") else {
            break;
        };
        let body = &rest[body_start..body_start + body_end];
        if !name.is_empty() {
            out.push((name, xml_unescape(body), encrypted));
        }
        rest = &rest[body_start + body_end + 9..];
    }
    out
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
) -> Result<BackupReport> {
    let file = File::create(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut zip = ZipWriter::new(file);
    let mut files = 0usize;
    let mut items = 0usize;

    let books = BookRepo::list(conn)?;
    items += books.len();
    write_json(
        &mut zip,
        "books.json",
        &serde_json::to_string(&books)?,
        &mut files,
        books.len(),
    )?;

    let groups = GroupRepo::list(conn)?;
    items += groups.len();
    write_json(
        &mut zip,
        "book_groups.json",
        &serde_json::to_string(&groups)?,
        &mut files,
        groups.len(),
    )?;

    let bookmarks = BookmarkRepo::list(conn)?;
    items += bookmarks.len();
    write_json(
        &mut zip,
        "bookmarks.json",
        &serde_json::to_string(&bookmarks)?,
        &mut files,
        bookmarks.len(),
    )?;

    let toc_rules = TxtTocRuleRepo::list(conn)?;
    items += toc_rules.len();
    write_json(
        &mut zip,
        "txtTocRules.json",
        &serde_json::to_string(&toc_rules)?,
        &mut files,
        toc_rules.len(),
    )?;

    let replace_rules = ReplaceRuleRepo::list(conn)?;
    items += replace_rules.len();
    write_json(
        &mut zip,
        "replace_rules.json",
        &serde_json::to_string(&replace_rules)?,
        &mut files,
        replace_rules.len(),
    )?;

    let dict_rules = DictRuleRepo::list(conn)?;
    items += dict_rules.len();
    write_json(
        &mut zip,
        "dictRules.json",
        &serde_json::to_string(&dict_rules)?,
        &mut files,
        dict_rules.len(),
    )?;

    let tts = HttpTtsRepo::list(conn)?;
    items += tts.len();
    write_json(
        &mut zip,
        "httpTTS.json",
        &serde_json::to_string(&tts)?,
        &mut files,
        tts.len(),
    )?;

    let read_records = json_value_rows(conn, "SELECT * FROM readRecord")?;
    items += read_records.len();
    write_json(
        &mut zip,
        "readRecord.json",
        &serde_json::to_string(&read_records)?,
        &mut files,
        read_records.len(),
    )?;

    let schemes = json_value_rows(conn, "SELECT * FROM read_style_schemes ORDER BY seqNo")?;
    items += schemes.len();
    write_json(
        &mut zip,
        "read_style_schemes.json",
        &serde_json::to_string(&schemes)?,
        &mut files,
        schemes.len(),
    )?;

    let themes = json_value_rows(conn, "SELECT * FROM themes")?;
    items += themes.len();
    write_json(
        &mut zip,
        "themes.json",
        &serde_json::to_string(&themes)?,
        &mut files,
        themes.len(),
    )?;

    let config = KeyValue::all(conn)?;
    if !config.is_empty() {
        let xml = write_config_xml(&config, password);
        write_json(&mut zip, "config.xml", &xml, &mut files, config.len())?;
    }

    zip.finish().map_err(|e| Error::Io(e.to_string()))?;
    Ok(BackupReport {
        path: path.to_string(),
        files,
        items,
    })
}

fn read_zip_entries(path: &str) -> Result<Vec<(String, String)>> {
    let file = File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut archive = ZipArchive::new(file).map_err(|_| Error::Invalid("备份文件无效".into()))?;
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| Error::Invalid(e.to_string()))?;
        let name = entry.name().to_string();
        let mut text = String::new();
        entry.read_to_string(&mut text)?;
        out.push((name, text));
    }
    Ok(out)
}

fn restore_config(
    conn: &Connection,
    xml: &str,
    password: Option<&str>,
    ignore: &[String],
) -> Result<usize> {
    let pairs = parse_config_xml(xml);
    let mut count = 0;
    for (key, value, encrypted) in pairs {
        if ignore.iter().any(|i| i == &key) || !is_whitelisted_key(&key) {
            continue;
        }
        let plain = if encrypted {
            let Some(pass) = password else { continue };
            let Some(bytes) = zip_aes::from_hex(&value) else {
                continue;
            };
            let Some(decoded) = zip_aes::decrypt(&bytes, pass) else {
                continue;
            };
            let Ok(text) = String::from_utf8(decoded) else {
                continue;
            };
            text
        } else {
            value
        };
        KeyValue::set(conn, &key, &plain)?;
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

    for (name, text) in entries {
        if ignore.iter().any(|i| i == &name) {
            skipped.push(name);
            continue;
        }
        match name.as_str() {
            "books.json" => {
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
            "book_groups.json" => {
                let groups: Vec<BookGroup> = serde_json::from_str(&text)?;
                for group in &groups {
                    GroupRepo::upsert(conn, group)?;
                }
                items += groups.len();
                files += 1;
            }
            "bookmarks.json" => {
                let bookmarks: Vec<Bookmark> = serde_json::from_str(&text)?;
                for bookmark in &bookmarks {
                    BookmarkRepo::upsert(conn, bookmark)?;
                }
                items += bookmarks.len();
                files += 1;
            }
            "txtTocRules.json" => {
                let rules: Vec<TxtTocRule> = serde_json::from_str(&text)?;
                for rule in &rules {
                    TxtTocRuleRepo::upsert(conn, rule)?;
                }
                items += rules.len();
                files += 1;
            }
            "replace_rules.json" => {
                let rules: Vec<ReplaceRule> = serde_json::from_str(&text)?;
                for rule in &rules {
                    ReplaceRuleRepo::upsert(conn, rule)?;
                }
                items += rules.len();
                files += 1;
            }
            "dictRules.json" => {
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
    })
}
