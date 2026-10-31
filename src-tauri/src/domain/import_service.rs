use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{BookRepo, ChapterRepo, TxtTocRuleRepo};
use crate::domain::epub_import;
use crate::domain::mobi_import;
use crate::domain::model::{book_type, Book, BookChapter, ContentBlock, TxtTocRule};
use crate::domain::pdf_import;
use crate::domain::storage::{hash_bytes, BookStorage};
use crate::domain::txt_parser;
use crate::domain::umd_import;
use crate::platform::charset_codec::CharsetCodec;
use crate::platform::charset_probe;
use crate::platform::filesystem::read_file_range;
use reader_core::txt::TxtTocRule as CoreTxtTocRule;
use reader_core::txt_kotlin;
use crate::{Error, Result};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub book: Book,
    pub chapter_count: usize,
    pub charset: String,
    pub rule: String,
    pub intro: String,
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|_| Error::Io(format!("文件读取失败: {}", path.display())))
}

fn mtime_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "txt".to_string())
}

fn source_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}

fn local_id(hash: &str) -> String {
    format!("local:{hash}")
}

fn existing_result(storage: &BookStorage, conn: &Connection, book: Book) -> Result<ImportResult> {
    let chapter_count = ChapterRepo::count(conn, &book.book_url)? as usize;
    let charset = book.charset.clone().unwrap_or_else(|| "UTF-8".to_string());
    let rule = book.toc_url.clone();
    let intro = book.intro.clone().unwrap_or_default();
    let mut book = book;
    storage.hydrate_book(&mut book);
    Ok(ImportResult {
        book,
        chapter_count,
        charset,
        rule,
        intro,
    })
}

fn cleanup_on_error<T>(
    storage: &BookStorage,
    id: &str,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let result = operation();
    if result.is_err() {
        storage.remove_book_dir(id);
    }
    result
}

fn write_cover_for_id(
    storage: &BookStorage,
    id: &str,
    cover: epub_import::CoverData,
) -> Result<String> {
    let path = storage.cover_path_for_id(id, &cover.ext);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, cover.bytes)?;
    Ok(storage.relative_string(&path))
}

fn write_cover_for_book(
    storage: &BookStorage,
    book: &Book,
    cover: epub_import::CoverData,
) -> Result<String> {
    let path = storage.cover_path(book, &cover.ext);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, cover.bytes)?;
    Ok(storage.relative_string(&path))
}

pub fn ensure_local_cover(book: &mut Book, storage: &BookStorage) -> Result<bool> {
    if book
        .cover_url
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return Ok(false);
    }
    let paths = storage.paths(book)?;
    if !paths.source.exists() {
        return Ok(false);
    }
    let source = paths.source.to_string_lossy();
    let cover = if is_epub(&source) {
        epub_import::extract_cover(&paths.source)?
    } else if is_umd(&source) {
        umd_import::parse(&paths.source)?.cover
    } else if is_mobi(&source) {
        mobi_import::parse(&paths.source, &paths.images)?.cover
    } else {
        None
    };
    let Some(cover) = cover else {
        return Ok(false);
    };
    book.cover_url = Some(write_cover_for_book(storage, book, cover)?);
    Ok(true)
}

pub fn ensure_epub_cover(book: &mut Book, storage: &BookStorage) -> Result<bool> {
    ensure_local_cover(book, storage)
}

pub fn is_epub(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("epub"))
        .unwrap_or(false)
}

pub fn is_pdf(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

pub fn is_umd(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("umd"))
        .unwrap_or(false)
}

pub fn is_mobi(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            value.eq_ignore_ascii_case("mobi")
                || value.eq_ignore_ascii_case("azw")
                || value.eq_ignore_ascii_case("azw3")
        })
        .unwrap_or(false)
}

pub fn is_cached_format(path: &str) -> bool {
    is_epub(path) || is_pdf(path) || is_umd(path) || is_mobi(path)
}

pub fn is_cached_book(book: &Book, storage: &BookStorage) -> bool {
    storage
        .paths(book)
        .map(|paths| is_cached_format(&paths.source.to_string_lossy()))
        .unwrap_or(false)
}

fn chapter_text_path(storage: &BookStorage, book: &Book, index: i32) -> Result<PathBuf> {
    Ok(storage.paths(book)?.chapters.join(format!("{index}.txt")))
}

fn chapter_blocks_path(storage: &BookStorage, book: &Book, index: i32) -> Result<PathBuf> {
    Ok(storage.paths(book)?.chapters.join(format!("{index}.json")))
}

pub fn read_epub_chapter(storage: &BookStorage, book: &Book, index: i32) -> Result<String> {
    let text_path = chapter_text_path(storage, book, index)?;
    if let Ok(text) = fs::read_to_string(&text_path) {
        return Ok(text);
    }
    let blocks = read_epub_chapter_blocks(storage, book, index)?;
    Ok(blocks_text(&blocks))
}

pub fn read_epub_chapter_blocks(
    storage: &BookStorage,
    book: &Book,
    index: i32,
) -> Result<Vec<ContentBlock>> {
    let path = chapter_blocks_path(storage, book, index)?;
    if let Ok(raw) = fs::read_to_string(&path) {
        let mut blocks = serde_json::from_str::<Vec<ContentBlock>>(&raw)
            .map_err(|e| Error::Internal(format!("EPUB chapter blocks parse failed: {e}")))?;
        resolve_blocks(storage, &mut blocks);
        return Ok(blocks);
    }
    let text = fs::read_to_string(chapter_text_path(storage, book, index)?)
        .map_err(|_| Error::Io(format!("章节缓存不存在: {}#{index}", book.book_url)))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ContentBlock::paragraph)
        .collect())
}

fn blocks_text(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Paragraph { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn relativize_blocks(storage: &BookStorage, blocks: &mut [ContentBlock]) {
    for block in blocks {
        match block {
            ContentBlock::Image { src, .. } | ContentBlock::ImagePage { src, .. } => {
                *src = storage.relative_string(Path::new(src));
            }
            ContentBlock::Paragraph { .. } => {}
        }
    }
}

fn resolve_blocks(storage: &BookStorage, blocks: &mut [ContentBlock]) {
    for block in blocks {
        match block {
            ContentBlock::Image { src, .. } | ContentBlock::ImagePage { src, .. } => {
                *src = storage.resolve(src);
            }
            ContentBlock::Paragraph { .. } => {}
        }
    }
}

fn core_rules(rules: &[TxtTocRule]) -> Vec<CoreTxtTocRule> {
    rules
        .iter()
        .map(|rule| CoreTxtTocRule {
            id: rule.id,
            name: rule.name.clone(),
            rule: rule.rule.clone(),
            enable: rule.enable,
            serial_number: rule.serial_number,
        })
        .collect()
}

fn core_chapters(
    chapters: &[reader_core::txt_kotlin::KChapter],
    book_url: &str,
    base_url: &str,
) -> Vec<BookChapter> {
    chapters
        .iter()
        .enumerate()
        .map(|(index, chapter)| BookChapter {
            url: format!("{base_url}#{index}"),
            title: chapter.title.clone(),
            index: index as i32,
            is_volume: chapter.is_volume,
            start: chapter.start,
            end: chapter.end,
            resource_url: Some(book_url.to_string()),
            ..Default::default()
        })
        .collect()
}
pub fn import_txt(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
    charset: Option<&str>,
    split_long: bool,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let bytes = read_file(src)?;
    let hash = hash_bytes(&bytes);
    let id = local_id(&hash);
    if let Some(book) = BookRepo::get(conn, &id)? {
        return existing_result(storage, conn, book);
    }

    let ext = extension(src);
    let origin_name = source_name(src);
    cleanup_on_error(storage, &id, || {
        storage.create_book_dir(&id, &ext)?;
        let dst = storage.copy_source(src, &id, &ext)?;

        let probed = charset_probe::probe_file_head(&bytes);
        let effective_charset = charset
            .map(|s| s.to_string())
            .unwrap_or_else(|| probed.label().to_string());
        let file_stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("未命名");
        let book_name = name.unwrap_or(file_stem).to_string();

        let rules = TxtTocRuleRepo::list(conn)?;
        let analyzed = txt_kotlin::analyze(
            &bytes,
            &effective_charset,
            &core_rules(&rules),
            split_long,
        )
        .map_err(Error::Invalid)?;
        let intro = analyzed.intro.clone();
        let used_rule = analyzed.rule.clone();

        let book = Book {
            book_url: id.clone(),
            name: book_name,
            author: "未知作者".to_string(),
            origin: "local".to_string(),
            origin_name,
            charset: Some(effective_charset.clone()),
            r#type: book_type::TEXT | book_type::LOCAL,
            group,
            toc_url: used_rule.clone(),
            intro: if intro.is_empty() {
                None
            } else {
                Some(intro.clone())
            },
            total_chapter_num: analyzed.chapters.len() as i32,
            can_update: true,
            sync_time: mtime_secs(&dst),
            source_path: Some(storage.relative_string(&dst)),
            source_hash: Some(hash.clone()),
            original_path: Some(src.to_string_lossy().to_string()),
            ..Default::default()
        };

        let chapters = core_chapters(&analyzed.chapters, &id, &id);
        let count = chapters.len();

        BookRepo::upsert(conn, &book)?;
        ChapterRepo::replace_all(conn, &id, &chapters)?;
        KeyValue::cache_set(
            conn,
            &txt_parser::cache_key(&id, mtime_secs(&dst)),
            &serde_json::to_string(&analyzed.chapters).unwrap_or_default(),
        )?;

        let mut result = ImportResult {
            book,
            chapter_count: count,
            charset: effective_charset,
            rule: used_rule,
            intro,
        };
        storage.hydrate_book(&mut result.book);
        Ok(result)
    })
}

pub fn import_epub(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let hash = BookStorage::hash_file(src)?;
    let id = local_id(&hash);
    if let Some(book) = BookRepo::get(conn, &id)? {
        return existing_result(storage, conn, book);
    }

    let ext = extension(src);
    let origin_name = source_name(src);

    cleanup_on_error(storage, &id, || {
        let paths = storage.create_book_dir(&id, &ext)?;
        let dst = storage.copy_source(src, &id, &ext)?;

        if paths.chapters.exists() {
            fs::remove_dir_all(&paths.chapters)?;
        }
        fs::create_dir_all(&paths.chapters)?;
        fs::create_dir_all(&paths.images)?;

        let parsed = epub_import::parse(&dst, &paths.images)?;
        let book_name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| parsed.title.clone());

        let mut chapters = Vec::with_capacity(parsed.chapters.len());
        for (index, chapter) in parsed.chapters.iter().enumerate() {
            let text = blocks_text(&chapter.blocks);
            let mut cached_blocks = chapter.blocks.clone();
            relativize_blocks(storage, &mut cached_blocks);
            fs::write(paths.chapters.join(format!("{index}.txt")), text.as_bytes())?;
            fs::write(
                paths.chapters.join(format!("{index}.json")),
                serde_json::to_vec(&cached_blocks)?,
            )?;
            chapters.push(BookChapter {
                url: format!("{id}#{index}"),
                title: chapter.title.clone(),
                index: index as i32,
                start: 0,
                end: 0,
                resource_url: Some(chapter.href.clone()),
                is_volume: chapter.is_volume,
                toc_level: chapter.toc_level,
                ..Default::default()
            });
        }

        let intro = if parsed.intro.trim().is_empty() {
            blocks_text(&parsed.chapters[0].blocks)
                .chars()
                .take(500)
                .collect::<String>()
        } else {
            parsed.intro.clone()
        };
        let cover_url = epub_import::extract_cover(&dst)?
            .map(|cover| write_cover_for_id(storage, &id, cover))
            .transpose()?;
        let book = Book {
            book_url: id.clone(),
            name: book_name,
            author: parsed.author,
            origin: "local".to_string(),
            origin_name,
            charset: Some("UTF-8".to_string()),
            r#type: book_type::TEXT | book_type::LOCAL,
            group,
            intro: if intro.is_empty() {
                None
            } else {
                Some(intro.clone())
            },
            cover_url,
            total_chapter_num: chapters.len() as i32,
            can_update: false,
            sync_time: mtime_secs(&dst),
            source_path: Some(storage.relative_string(&dst)),
            source_hash: Some(hash.clone()),
            original_path: Some(src.to_string_lossy().to_string()),
            ..Default::default()
        };
        let count = chapters.len();
        BookRepo::upsert(conn, &book)?;
        ChapterRepo::replace_all(conn, &id, &chapters)?;

        let mut result = ImportResult {
            book,
            chapter_count: count,
            charset: "UTF-8".to_string(),
            rule: String::new(),
            intro,
        };
        storage.hydrate_book(&mut result.book);
        Ok(result)
    })
}

#[allow(clippy::too_many_arguments)]
fn import_cached_chapters(
    conn: &Connection,
    storage: &BookStorage,
    id: &str,
    dst: &Path,
    hash: &str,
    origin_name: String,
    book_name: String,
    author: String,
    intro: String,
    kind: Option<String>,
    cover: Option<epub_import::CoverData>,
    chapters: Vec<(String, Option<String>, Vec<ContentBlock>, bool, i32)>,
    group: u64,
) -> Result<ImportResult> {
    let paths = storage.paths_for_id(id, &extension(dst));
    if paths.chapters.exists() {
        for entry in fs::read_dir(&paths.chapters)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("txt")
                || path.extension().and_then(|value| value.to_str()) == Some("json")
            {
                fs::remove_file(path)?;
            }
        }
    }
    fs::create_dir_all(&paths.chapters)?;

    let mut book_chapters = Vec::with_capacity(chapters.len());
    for (index, (title, href, blocks, is_volume, toc_level)) in chapters.iter().enumerate() {
        fs::write(
            paths.chapters.join(format!("{index}.txt")),
            blocks_text(blocks).as_bytes(),
        )?;
        let mut cached_blocks = blocks.clone();
        relativize_blocks(storage, &mut cached_blocks);
        fs::write(
            paths.chapters.join(format!("{index}.json")),
            serde_json::to_vec(&cached_blocks)?,
        )?;
        book_chapters.push(BookChapter {
            url: format!("{id}#{index}"),
            title: title.clone(),
            index: index as i32,
            start: 0,
            end: 0,
            resource_url: href.clone(),
            is_volume: *is_volume,
            toc_level: *toc_level,
            ..Default::default()
        });
    }

    let intro = if intro.trim().is_empty() {
        chapters
            .first()
            .map(|(_, _, blocks, _, _)| blocks_text(blocks).chars().take(500).collect())
            .unwrap_or_default()
    } else {
        intro
    };
    let cover_url = cover
        .map(|cover| write_cover_for_id(storage, id, cover))
        .transpose()?;
    let book = Book {
        book_url: id.to_string(),
        name: book_name,
        author: if author.trim().is_empty() {
            "未知作者".to_string()
        } else {
            author
        },
        origin: "local".to_string(),
        origin_name,
        charset: Some("UTF-8".to_string()),
        r#type: book_type::TEXT | book_type::LOCAL,
        group,
        kind,
        intro: if intro.is_empty() {
            None
        } else {
            Some(intro.clone())
        },
        cover_url,
        total_chapter_num: book_chapters.len() as i32,
        can_update: false,
        sync_time: mtime_secs(dst),
        source_path: Some(storage.relative_string(dst)),
        source_hash: Some(hash.to_string()),
        ..Default::default()
    };
    let count = book_chapters.len();
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, id, &book_chapters)?;
    let mut result = ImportResult {
        book,
        chapter_count: count,
        charset: "UTF-8".to_string(),
        rule: String::new(),
        intro,
    };
    storage.hydrate_book(&mut result.book);
    Ok(result)
}

pub fn import_umd(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let hash = BookStorage::hash_file(src)?;
    let id = local_id(&hash);
    if let Some(book) = BookRepo::get(conn, &id)? {
        return existing_result(storage, conn, book);
    }
    let ext = extension(src);
    let origin_name = source_name(src);

    cleanup_on_error(storage, &id, || {
        storage.create_book_dir(&id, &ext)?;
        let dst = storage.copy_source(src, &id, &ext)?;
        let parsed = umd_import::parse(&dst)?;
        let book_name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or(parsed.title);
        let chapters = parsed
            .chapters
            .into_iter()
            .map(|chapter| (chapter.title, None, chapter.blocks, false, 0))
            .collect();
        import_cached_chapters(
            conn,
            storage,
            &id,
            &dst,
            &hash,
            origin_name,
            book_name,
            parsed.author,
            String::new(),
            Some(parsed.kind).filter(|value| !value.trim().is_empty()),
            parsed.cover,
            chapters,
            group,
        )
    })
}

pub fn import_mobi(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let hash = BookStorage::hash_file(src)?;
    let id = local_id(&hash);
    if let Some(book) = BookRepo::get(conn, &id)? {
        return existing_result(storage, conn, book);
    }
    let ext = extension(src);
    let origin_name = source_name(src);

    cleanup_on_error(storage, &id, || {
        let paths = storage.create_book_dir(&id, &ext)?;
        let dst = storage.copy_source(src, &id, &ext)?;
        let parsed = mobi_import::parse(&dst, &paths.images)?;
        let book_name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or(parsed.title);
        let chapters = parsed
            .chapters
            .into_iter()
            .map(|chapter| {
                (
                    chapter.title,
                    None,
                    chapter.blocks,
                    chapter.is_volume,
                    chapter.toc_level,
                )
            })
            .collect();
        import_cached_chapters(
            conn,
            storage,
            &id,
            &dst,
            &hash,
            origin_name,
            book_name,
            parsed.author,
            parsed.intro,
            Some(parsed.kind).filter(|value| !value.trim().is_empty()),
            parsed.cover,
            chapters,
            group,
        )
    })
}

pub fn import_pdf(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    const PAGES_PER_CHAPTER: usize = 10;
    let src = Path::new(src_path);
    let hash = BookStorage::hash_file(src)?;
    let id = local_id(&hash);
    if let Some(book) = BookRepo::get(conn, &id)? {
        return existing_result(storage, conn, book);
    }
    let ext = extension(src);
    let origin_name = source_name(src);
    let fallback_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            src.file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Untitled".to_string());

    cleanup_on_error(storage, &id, || {
        let paths = storage.create_book_dir(&id, &ext)?;
        let dst = storage.copy_source(src, &id, &ext)?;

        if paths.chapters.exists() {
            fs::remove_dir_all(&paths.chapters)?;
        }
        fs::create_dir_all(&paths.chapters)?;
        fs::create_dir_all(&paths.images)?;

        let parsed = pdf_import::parse(&dst, &paths.images)?;
        let book_name = name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| fallback_name.clone());

        let mut chapters = Vec::new();
        let mut intro = String::new();
        for (index, pages) in parsed.pages.chunks(PAGES_PER_CHAPTER).enumerate() {
            let blocks = pages
                .iter()
                .flat_map(pdf_import::page_blocks)
                .collect::<Vec<_>>();
            let text = blocks_text(&blocks);
            if intro.is_empty() {
                intro = text.chars().take(500).collect();
            }
            let mut cached_blocks = blocks.clone();
            relativize_blocks(storage, &mut cached_blocks);
            fs::write(paths.chapters.join(format!("{index}.txt")), text.as_bytes())?;
            fs::write(
                paths.chapters.join(format!("{index}.json")),
                serde_json::to_vec(&cached_blocks)?,
            )?;
            let first = pages.first().map(|page| page.page_number).unwrap_or(0);
            let last = pages.last().map(|page| page.page_number).unwrap_or(first);
            let title = if first == last {
                format!("第 {first} 页")
            } else {
                format!("第 {first}-{last} 页")
            };
            chapters.push(BookChapter {
                url: format!("{id}#{index}"),
                title,
                index: index as i32,
                start: 0,
                end: 0,
                ..Default::default()
            });
        }

        let book = Book {
            book_url: id.clone(),
            name: book_name,
            author: "Unknown".to_string(),
            origin: "local".to_string(),
            origin_name,
            charset: Some("UTF-8".to_string()),
            r#type: book_type::TEXT | book_type::LOCAL,
            group,
            intro: if intro.is_empty() {
                None
            } else {
                Some(intro.clone())
            },
            total_chapter_num: chapters.len() as i32,
            can_update: false,
            sync_time: mtime_secs(&dst),
            source_path: Some(storage.relative_string(&dst)),
            source_hash: Some(hash.clone()),
            original_path: Some(src.to_string_lossy().to_string()),
            ..Default::default()
        };
        let count = chapters.len();
        BookRepo::upsert(conn, &book)?;
        ChapterRepo::replace_all(conn, &id, &chapters)?;

        let mut result = ImportResult {
            book,
            chapter_count: count,
            charset: "UTF-8".to_string(),
            rule: String::new(),
            intro,
        };
        storage.hydrate_book(&mut result.book);
        Ok(result)
    })
}

pub fn import_book(
    conn: &Connection,
    src_path: &str,
    storage: &BookStorage,
    name: Option<&str>,
    group: u64,
    charset: Option<&str>,
    split_long: bool,
) -> Result<ImportResult> {
    if is_epub(src_path) {
        import_epub(conn, src_path, storage, name, group)
    } else if is_pdf(src_path) {
        import_pdf(conn, src_path, storage, name, group)
    } else if is_umd(src_path) {
        import_umd(conn, src_path, storage, name, group)
    } else if is_mobi(src_path) {
        import_mobi(conn, src_path, storage, name, group)
    } else {
        import_txt(conn, src_path, storage, name, group, charset, split_long)
    }
}

pub fn re_split(
    conn: &Connection,
    storage: &BookStorage,
    book_url: &str,
    rule_expr: Option<&str>,
    charset: Option<&str>,
    split_long: bool,
) -> Result<usize> {
    let mut book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let paths = storage.paths(&book)?;
    let bytes = read_file(&paths.source)?;

    let effective_charset = charset
        .map(|s| s.to_string())
        .or_else(|| book.charset.clone())
        .unwrap_or_else(|| "UTF-8".to_string());
    let rules = TxtTocRuleRepo::list(conn)?;
    let analyzed = txt_kotlin::analyze_with_rule(
        &bytes,
        &effective_charset,
        &core_rules(&rules),
        rule_expr,
        split_long,
    )
    .map_err(Error::Invalid)?;

    let chapters = core_chapters(&analyzed.chapters, book_url, book_url);
    let count = chapters.len();

    book.charset = Some(effective_charset);
    book.toc_url = analyzed.rule.clone();
    book.total_chapter_num = count as i32;
    if !analyzed.intro.is_empty() {
        book.intro = Some(analyzed.intro.clone());
    }
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, book_url, &chapters)?;
    KeyValue::cache_set(
        conn,
        &txt_parser::cache_key(book_url, mtime_secs(&paths.source)),
        &serde_json::to_string(&analyzed.chapters).unwrap_or_default(),
    )?;
    Ok(count)
}

pub fn read_chapter(
    conn: &Connection,
    storage: &BookStorage,
    book_url: &str,
    index: i32,
) -> Result<String> {
    let book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let chapters = ChapterRepo::list(conn, book_url)?;
    let ch = chapters
        .iter()
        .find(|c| c.index == index)
        .ok_or_else(|| Error::NotFound(format!("chapter {index}")))?;
    let paths = storage.paths(&book)?;
    if is_cached_format(&paths.source.to_string_lossy()) {
        return read_epub_chapter(storage, &book, index);
    }
    let bytes = read_file_range(&paths.source, ch.start, ch.end)?;
    let codec = CharsetCodec::from_label(book.charset.as_deref().unwrap_or("UTF-8"));
    Ok(txt_parser::read_chapter_content(
        &bytes,
        &codec,
        0,
        bytes.len() as i64,
        &ch.title,
    ))
}
