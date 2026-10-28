//! Local book import helpers: directory browsing, recursive scanning, batch
//! import, cover files and filename scripts.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::Connection;
use serde::Serialize;

use md5::{Digest, Md5};

use crate::data::repositories::BookRepo;
use crate::domain::import_service;
use crate::{Error, Result};

const BOOK_EXTENSIONS: [&str; 7] = ["txt", "epub", "pdf", "umd", "mobi", "azw3", "azw"];
const ARCHIVE_EXTENSIONS: [&str; 3] = ["zip", "rar", "7z"];
const SUPPORTED_EXTENSIONS: [&str; 3] = ["txt", "epub", "pdf"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalImportEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub ext: String,
    pub supported: bool,
    pub on_shelf: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalImportResult {
    pub path: String,
    pub name: String,
    pub book_url: Option<String>,
    pub error: Option<String>,
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn is_book_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    BOOK_EXTENSIONS
        .iter()
        .chain(ARCHIVE_EXTENSIONS.iter())
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn is_supported(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn is_archive(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ARCHIVE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn modified_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn on_shelf_names(conn: &Connection) -> Result<HashSet<String>> {
    let books = BookRepo::list(conn)?;
    let mut names = HashSet::new();
    for book in books {
        if !book.origin_name.trim().is_empty() {
            names.insert(book.origin_name);
        }
        if let Some(name) = Path::new(&book.book_url).file_name().and_then(|v| v.to_str()) {
            names.insert(name.to_string());
        }
    }
    Ok(names)
}

fn entry_from_path(path: PathBuf, on_shelf: &HashSet<String>) -> Option<LocalImportEntry> {
    let name = path.file_name()?.to_str()?.to_string();
    if name.starts_with('.') {
        return None;
    }
    let meta = fs::metadata(&path).ok()?;
    let is_dir = meta.is_dir();
    if !is_dir && !is_book_file(&name) {
        return None;
    }
    Some(LocalImportEntry {
        path: path.to_string_lossy().to_string(),
        ext: extension(&path),
        name: name.clone(),
        is_dir,
        size: if is_dir { 0 } else { meta.len() },
        modified: modified_secs(&path),
        supported: !is_dir && is_supported(&name) && !is_archive(&name),
        on_shelf: !is_dir && on_shelf.contains(&name),
    })
}

pub fn list_dir(conn: &Connection, path: &str) -> Result<Vec<LocalImportEntry>> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(Error::Invalid(format!("目录不存在: {path}")));
    }
    let on_shelf = on_shelf_names(conn)?;
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        if let Some(entry) = entry_from_path(item.path(), &on_shelf) {
            entries.push(entry);
        }
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

pub fn scan_dir(conn: &Connection, path: &str) -> Result<Vec<LocalImportEntry>> {
    let root = Path::new(path);
    if !root.is_dir() {
        return Err(Error::Invalid(format!("目录不存在: {path}")));
    }
    let on_shelf = on_shelf_names(conn)?;
    let mut entries = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for item in read_dir.flatten() {
            let item_path = item.path();
            let Some(name) = item_path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            if item_path.is_dir() {
                stack.push(item_path);
            } else if is_book_file(name) {
                if let Some(entry) = entry_from_path(item_path, &on_shelf) {
                    entries.push(entry);
                }
            }
        }
    }
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

pub fn delete_files(paths: &[String]) -> Result<usize> {
    let mut deleted = 0;
    for path in paths {
        let target = Path::new(path);
        if target.is_file() {
            fs::remove_file(target)?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

pub fn import_files(
    conn: &Connection,
    data_dir: &Path,
    paths: &[String],
    group: u64,
    name_script: Option<&str>,
) -> Vec<LocalImportResult> {
    let mut results = Vec::new();
    for path in paths {
        let target = Path::new(path);
        let file_name = target
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(path)
            .to_string();
        let (name, author) = analyze_file_name(&file_name, name_script);
        match import_service::import_book(
            conn,
            path,
            data_dir,
            name.as_deref(),
            group,
            None,
            true,
        ) {
            Ok(mut result) => {
                let current = result.book.author.trim();
                if !author.is_empty()
                    && (current.is_empty() || current == "未知作者" || current == "Unknown")
                {
                    result.book.author = author;
                    if let Err(error) = BookRepo::upsert(conn, &result.book) {
                        results.push(LocalImportResult {
                            path: path.clone(),
                            name: file_name,
                            book_url: None,
                            error: Some(error.to_string()),
                        });
                        continue;
                    }
                }
                results.push(LocalImportResult {
                    path: path.clone(),
                    name: file_name,
                    book_url: Some(result.book.book_url),
                    error: None,
                });
            }
            Err(error) => results.push(LocalImportResult {
                path: path.clone(),
                name: file_name,
                book_url: None,
                error: Some(error.to_string()),
            }),
        }
    }
    results
}

fn analyze_file_name(file_name: &str, script: Option<&str>) -> (Option<String>, String) {
    let stem = file_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(file_name)
        .to_string();
    if let Some(script) = script.map(str::trim).filter(|value| !value.is_empty()) {
        if let Some((name, author)) = run_name_script(script, &stem) {
            return (Some(name), author);
        }
    }
    for (pattern, name_group, author_groups) in [
        (r"^(.*?)《([^《》]+)》.*?作者[:：](.*)$", 2, vec![1, 3]),
        (r"^(.*?)《([^《》]+)》(.*)$", 2, vec![1, 3]),
        (r"^(?)(.+) 作者[:：](.+)$", 2, vec![1, 3]),
        (r"^(?)(.+) by (.+)$", 2, vec![1, 3]),
    ] {
        let Ok(regex) = fancy_regex::Regex::new(pattern) else {
            continue;
        };
        let Ok(Some(captures)) = regex.captures(&stem) else {
            continue;
        };
        let Some(name) = captures.get(name_group).map(|value| value.as_str().trim()) else {
            continue;
        };
        let author = author_groups
            .iter()
            .filter_map(|index| captures.get(*index).map(|value| value.as_str()))
            .collect::<String>()
            .trim()
            .to_string();
        if !name.is_empty() {
            return (Some(name.to_string()), author);
        }
    }
    (None, String::new())
}

fn run_name_script(script: &str, src: &str) -> Option<(String, String)> {
    use boa_engine::{Context, JsString, JsValue, Source};

    let mut context = Context::default();
    let src = JsValue::from(JsString::from(src));
    context
        .global_object()
        .set(boa_engine::js_string!("src"), src, true, &mut context)
        .ok()?;
    let wrapped = format!(
        "{script}\nJSON.stringify({{name: typeof name === 'undefined' ? '' : String(name), author: typeof author === 'undefined' ? '' : String(author)}})"
    );
    let value = context.eval(Source::from_bytes(&wrapped)).ok()?;
    let json = value.to_string(&mut context).ok()?.to_std_string_escaped();
    let parsed: serde_json::Value = serde_json::from_str(&json).ok()?;
    let name = parsed
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let author = parsed
        .get("author")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some((name, author))
    }
}

pub fn save_cover_from_file(
    _conn: &Connection,
    data_dir: &Path,
    _book_url: &str,
    src_path: &str,
) -> Result<String> {
    let src = Path::new(src_path);
    if !src.is_file() {
        return Err(Error::Invalid(format!("图片不存在: {src_path}")));
    }
    let ext = extension(src);
    let ext = if ext.is_empty() { "jpg".to_string() } else { ext };
    let bytes = fs::read(src)?;
    let digest = Md5::digest(&bytes);
    let dir = data_dir.join("covers");
    fs::create_dir_all(&dir)?;
    let dst = dir.join(format!("{digest:x}.{ext}"));
    if !dst.exists() {
        fs::write(&dst, bytes)?;
    }
    Ok(dst.to_string_lossy().to_string())
}

pub fn refresh_cover(
    conn: &Connection,
    data_dir: &Path,
    book_url: &str,
) -> Result<Option<String>> {
    let Some(mut book) = BookRepo::get(conn, book_url)? else {
        return Err(Error::NotFound(book_url.to_string()));
    };
    book.cover_url = None;
    if import_service::ensure_epub_cover(&mut book, data_dir)? {
        let cover = book.cover_url.clone();
        BookRepo::upsert(conn, &book)?;
        Ok(cover)
    } else {
        Ok(None)
    }
}

pub fn file_size(book_url: &str) -> i64 {
    fs::metadata(book_url)
        .map(|meta| meta.len() as i64)
        .unwrap_or(0)
}

pub fn clear_cache(book_url: &str) -> Result<()> {
    let path = Path::new(book_url);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if name.is_empty() {
        return Ok(());
    }
    let cache = path.with_file_name(format!("{name}.chapters"));
    if cache.exists() {
        fs::remove_dir_all(cache)?;
    }
    Ok(())
}
