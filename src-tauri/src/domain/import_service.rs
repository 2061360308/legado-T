//! TXT 导入服务（T8.2 / T8.4）。
//!
//! - 复制文件到应用私有目录，默认以文件名作书名（可改），可选加入分组。
//! - 探测编码；失败/解码异常回退默认字符集并允许手动指定后重新分章。
//! - 文件不可读时中止并保留原状态。

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use md5::{Digest, Md5};

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{BookRepo, ChapterRepo, TxtTocRuleRepo};
use crate::domain::epub_import;
use crate::domain::mobi_import;
use crate::domain::pdf_import;
use crate::domain::umd_import;
use crate::domain::model::{book_type, Book, BookChapter, ContentBlock};
use crate::domain::txt_parser;
use crate::platform::charset_codec::CharsetCodec;
use crate::platform::charset_probe;
use crate::platform::filesystem::read_file_range;
use crate::{Error, Result};

/// 导入结果。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub book: Book,
    pub chapter_count: usize,
    pub charset: String,
    /// 使用的目录规则表达式（空 = 兜底分章）。
    pub rule: String,
    pub intro: String,
}

/// 检查文件可读（spec 5.2.3-3「文件读取失败」提示）。
fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|_| Error::Io(format!("文件读取失败: {}", path.display())))
}

/// 复制到私有目录（booksData/ 下以书名 + 扩展命名；重名追加序号）。
fn copy_into_private(src: &Path, data_dir: &Path, book_name: &str) -> Result<PathBuf> {
    fs::create_dir_all(data_dir.join("booksData"))?;
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt")
        .to_string();
    let safe: String = book_name
        .chars()
        .map(|c| {
            if c.is_whitespace() || "/\\:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let mut dst = data_dir.join("booksData").join(format!("{safe}.{ext}"));
    let mut seq = 1;
    while dst.exists() {
        dst = data_dir
            .join("booksData")
            .join(format!("{safe}({seq}).{ext}"));
        seq += 1;
    }
    fs::copy(src, &dst).map_err(|e| Error::Io(e.to_string()))?;
    Ok(dst)
}

/// 文件 mtime（秒）。
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

fn write_cover(data_dir: &Path, book_url: &str, cover: epub_import::CoverData) -> Result<String> {
    let dir = data_dir.join("covers");
    fs::create_dir_all(&dir)?;
    let digest = Md5::digest(book_url.as_bytes());
    let path = dir.join(format!("{digest:x}.{}", cover.ext));
    fs::write(&path, cover.bytes)?;
    Ok(path.to_string_lossy().to_string())
}

pub fn ensure_local_cover(book: &mut Book, data_dir: &Path) -> Result<bool> {
    if book
        .cover_url
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return Ok(false);
    }
    let path = Path::new(&book.book_url);
    if !path.exists() {
        return Ok(false);
    }
    let cover = if is_epub(&book.book_url) {
        epub_import::extract_cover(path)?
    } else if is_umd(&book.book_url) {
        umd_import::parse(path)?.cover
    } else if is_mobi(&book.book_url) {
        mobi_import::parse(path, &chapter_image_dir(&book.book_url))?.cover
    } else {
        None
    };
    let Some(cover) = cover else {
        return Ok(false);
    };
    book.cover_url = Some(write_cover(data_dir, &book.book_url, cover)?);
    Ok(true)
}

pub fn ensure_epub_cover(book: &mut Book, data_dir: &Path) -> Result<bool> {
    ensure_local_cover(book, data_dir)
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

pub fn chapter_cache_path(book_url: &str, index: i32) -> PathBuf {
    let path = Path::new(book_url);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("book.epub");
    path.with_file_name(format!("{name}.chapters"))
        .join(format!("{index}.txt"))
}

pub fn chapter_blocks_path(book_url: &str, index: i32) -> PathBuf {
    chapter_cache_path(book_url, index).with_extension("json")
}

pub fn chapter_image_dir(book_url: &str) -> PathBuf {
    chapter_cache_path(book_url, 0)
        .parent()
        .map(|dir| dir.join("images"))
        .unwrap_or_else(|| PathBuf::from("images"))
}

pub fn read_epub_chapter(book_url: &str, index: i32) -> Result<String> {
    let text_path = chapter_cache_path(book_url, index);
    if let Ok(text) = fs::read_to_string(&text_path) {
        return Ok(text);
    }
    let blocks = read_epub_chapter_blocks(book_url, index)?;
    Ok(blocks_text(&blocks))
}

pub fn read_epub_chapter_blocks(book_url: &str, index: i32) -> Result<Vec<ContentBlock>> {
    let path = chapter_blocks_path(book_url, index);
    if let Ok(raw) = fs::read_to_string(&path) {
        return serde_json::from_str::<Vec<ContentBlock>>(&raw)
            .map_err(|e| Error::Internal(format!("EPUB chapter blocks parse failed: {e}")));
    }
    let text = fs::read_to_string(chapter_cache_path(book_url, index))
        .map_err(|_| Error::Io(format!("EPUB 章节缓存不存在: {book_url}#{index}")))?;
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

/// 导入 TXT：复制 → 探测编码 → 分章 → 建书记录。
/// `charset` 手动指定（Some）覆盖探测结果。
pub fn import_txt(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
    charset: Option<&str>,
    split_long: bool,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let bytes = read_file(src)?;

    // 编码：手动指定优先（spec 5.2.1-3）。
    let probed = charset_probe::probe_file_head(&bytes);
    let effective_charset = charset
        .map(|s| s.to_string())
        .unwrap_or_else(|| probed.label().to_string());
    let codec = CharsetCodec::from_label(&effective_charset);

    // 复制到私有目录。
    let file_stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("未命名");
    let book_name = name.unwrap_or(file_stem).to_string();
    let dst = copy_into_private(src, data_dir, &book_name)?;
    let book_url = dst.to_string_lossy().to_string();

    // 分章。
    let rules = TxtTocRuleRepo::list(conn)?;
    let (outlines, intro, used_rule) =
        txt_parser::analyze(&bytes, &codec, None, &rules, split_long)?;

    let book = Book {
        book_url: book_url.clone(),
        name: book_name,
        author: "未知作者".to_string(),
        origin: "local".to_string(),
        origin_name: dst
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string(),
        charset: Some(effective_charset.clone()),
        r#type: book_type::TEXT | book_type::LOCAL,
        group,
        toc_url: used_rule.clone(),
        intro: if intro.is_empty() {
            None
        } else {
            Some(intro.clone())
        },
        total_chapter_num: outlines.len() as i32,
        can_update: true,
        sync_time: mtime_secs(&dst),
        ..Default::default()
    };

    let chapters = txt_parser::outlines_to_chapters(&outlines, &book_url, &book_url);
    let count = chapters.len();

    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, &book_url, &chapters)?;
    KeyValue::cache_set(
        conn,
        &txt_parser::cache_key(&book_url, mtime_secs(&dst)),
        &serde_json::to_string(&outlines).unwrap_or_default(),
    )?;

    Ok(ImportResult {
        book,
        chapter_count: count,
        charset: effective_charset,
        rule: used_rule,
        intro,
    })
}

/// 导入 EPUB：解析 spine 目录，章节正文写入相邻的 `.chapters` 目录。
pub fn import_epub(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let fallback_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            src.file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Untitled".to_string());
    let dst = copy_into_private(src, data_dir, &fallback_name)?;
    let book_url = dst.to_string_lossy().to_string();

    let cache_dir = chapter_cache_path(&book_url, 0)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dst.with_file_name("chapters"));
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(&cache_dir)?;
    let image_dir = chapter_image_dir(&book_url);
    fs::create_dir_all(&image_dir)?;

    let parsed = epub_import::parse(&dst, &image_dir)?;
    let book_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| parsed.title.clone());

    let mut chapters = Vec::with_capacity(parsed.chapters.len());
    for (index, chapter) in parsed.chapters.iter().enumerate() {
        let text = blocks_text(&chapter.blocks);
        fs::write(
            cache_dir.join(format!("{index}.txt")),
            text.as_bytes(),
        )?;
        fs::write(
            cache_dir.join(format!("{index}.json")),
            serde_json::to_vec(&chapter.blocks)?,
        )?;
        chapters.push(BookChapter {
            url: format!("{book_url}#{index}"),
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
        .map(|cover| write_cover(data_dir, &book_url, cover))
        .transpose()?;
    let book = Book {
        book_url: book_url.clone(),
        name: book_name,
        author: parsed.author,
        origin: "local".to_string(),
        origin_name: dst
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
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
        ..Default::default()
    };
    let count = chapters.len();
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, &book_url, &chapters)?;

    Ok(ImportResult {
        book,
        chapter_count: count,
        charset: "UTF-8".to_string(),
        rule: String::new(),
        intro,
    })
}

fn import_cached_chapters(
    conn: &Connection,
    data_dir: &Path,
    dst: &Path,
    book_name: String,
    author: String,
    intro: String,
    kind: Option<String>,
    cover: Option<epub_import::CoverData>,
    chapters: Vec<(String, Option<String>, Vec<ContentBlock>, bool, i32)>,
    group: u64,
) -> Result<ImportResult> {
    let book_url = dst.to_string_lossy().to_string();
    let cache_dir = chapter_cache_path(&book_url, 0)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dst.with_file_name("chapters"));
    if cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("txt")
                || path.extension().and_then(|value| value.to_str()) == Some("json")
            {
                fs::remove_file(path)?;
            }
        }
    }
    fs::create_dir_all(&cache_dir)?;

    let mut book_chapters = Vec::with_capacity(chapters.len());
    for (index, (title, href, blocks, is_volume, toc_level)) in chapters.iter().enumerate() {
        fs::write(
            cache_dir.join(format!("{index}.txt")),
            blocks_text(blocks).as_bytes(),
        )?;
        fs::write(
            cache_dir.join(format!("{index}.json")),
            serde_json::to_vec(blocks)?,
        )?;
        book_chapters.push(BookChapter {
            url: format!("{book_url}#{index}"),
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
        .map(|cover| write_cover(data_dir, &book_url, cover))
        .transpose()?;
    let book = Book {
        book_url: book_url.clone(),
        name: book_name,
        author: if author.trim().is_empty() {
            "未知作者".to_string()
        } else {
            author
        },
        origin: "local".to_string(),
        origin_name: dst
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
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
        ..Default::default()
    };
    let count = book_chapters.len();
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, &book_url, &book_chapters)?;
    Ok(ImportResult {
        book,
        chapter_count: count,
        charset: "UTF-8".to_string(),
        rule: String::new(),
        intro,
    })
}

pub fn import_umd(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let fallback_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            src.file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Untitled".to_string());
    let dst = copy_into_private(src, data_dir, &fallback_name)?;
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
        data_dir,
        &dst,
        book_name,
        parsed.author,
        String::new(),
        Some(parsed.kind).filter(|value| !value.trim().is_empty()),
        parsed.cover,
        chapters,
        group,
    )
}

pub fn import_mobi(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    let src = Path::new(src_path);
    let fallback_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            src.file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Untitled".to_string());
    let dst = copy_into_private(src, data_dir, &fallback_name)?;
    let book_url = dst.to_string_lossy().to_string();
    let image_dir = chapter_image_dir(&book_url);
    let parsed = mobi_import::parse(&dst, &image_dir)?;
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
        data_dir,
        &dst,
        book_name,
        parsed.author,
        parsed.intro,
        Some(parsed.kind).filter(|value| !value.trim().is_empty()),
        parsed.cover,
        chapters,
        group,
    )
}

pub fn import_pdf(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
) -> Result<ImportResult> {
    const PAGES_PER_CHAPTER: usize = 10;
    let src = Path::new(src_path);
    let fallback_name = name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            src.file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Untitled".to_string());
    let dst = copy_into_private(src, data_dir, &fallback_name)?;
    let book_url = dst.to_string_lossy().to_string();

    let cache_dir = chapter_cache_path(&book_url, 0)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dst.with_file_name("chapters"));
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(&cache_dir)?;
    let image_dir = chapter_image_dir(&book_url);
    fs::create_dir_all(&image_dir)?;

    let parsed = pdf_import::parse(&dst, &image_dir)?;
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
        fs::write(cache_dir.join(format!("{index}.txt")), text.as_bytes())?;
        fs::write(
            cache_dir.join(format!("{index}.json")),
            serde_json::to_vec(&blocks)?,
        )?;
        let first = pages.first().map(|page| page.page_number).unwrap_or(0);
        let last = pages.last().map(|page| page.page_number).unwrap_or(first);
        let title = if first == last {
            format!("第 {first} 页")
        } else {
            format!("第 {first}-{last} 页")
        };
        chapters.push(BookChapter {
            url: format!("{book_url}#{index}"),
            title,
            index: index as i32,
            start: 0,
            end: 0,
            ..Default::default()
        });
    }

    let book = Book {
        book_url: book_url.clone(),
        name: book_name,
        author: "Unknown".to_string(),
        origin: "local".to_string(),
        origin_name: dst
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
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
        ..Default::default()
    };
    let count = chapters.len();
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, &book_url, &chapters)?;

    Ok(ImportResult {
        book,
        chapter_count: count,
        charset: "UTF-8".to_string(),
        rule: String::new(),
        intro,
    })
}

pub fn import_book(
    conn: &Connection,
    src_path: &str,
    data_dir: &Path,
    name: Option<&str>,
    group: u64,
    charset: Option<&str>,
    split_long: bool,
) -> Result<ImportResult> {
    if is_epub(src_path) {
        import_epub(conn, src_path, data_dir, name, group)
    } else if is_pdf(src_path) {
        import_pdf(conn, src_path, data_dir, name, group)
    } else if is_umd(src_path) {
        import_umd(conn, src_path, data_dir, name, group)
    } else if is_mobi(src_path) {
        import_mobi(conn, src_path, data_dir, name, group)
    } else {
        import_txt(conn, src_path, data_dir, name, group, charset, split_long)
    }
}

/// 重新分章（T10.3）：手动选规则/指定字符集后重建目录并更新当前规则记录。
pub fn re_split(
    conn: &Connection,
    book_url: &str,
    rule_expr: Option<&str>,
    charset: Option<&str>,
    split_long: bool,
) -> Result<usize> {
    let mut book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let path = Path::new(book_url);
    let bytes = read_file(path)?;

    let effective_charset = charset
        .map(|s| s.to_string())
        .or_else(|| book.charset.clone())
        .unwrap_or_else(|| "UTF-8".to_string());
    let codec = CharsetCodec::from_label(&effective_charset);

    let rules = TxtTocRuleRepo::list(conn)?;
    let (outlines, intro, used_rule) =
        txt_parser::analyze(&bytes, &codec, rule_expr, &rules, split_long)?;

    let chapters = txt_parser::outlines_to_chapters(&outlines, book_url, book_url);
    let count = chapters.len();

    book.charset = Some(effective_charset);
    book.toc_url = used_rule;
    book.total_chapter_num = count as i32;
    if !intro.is_empty() {
        book.intro = Some(intro);
    }
    BookRepo::upsert(conn, &book)?;
    ChapterRepo::replace_all(conn, book_url, &chapters)?;
    KeyValue::cache_set(
        conn,
        &txt_parser::cache_key(book_url, mtime_secs(path)),
        &serde_json::to_string(&outlines).unwrap_or_default(),
    )?;
    Ok(count)
}

/// 读取章节正文（T9.5）。
pub fn read_chapter(conn: &Connection, book_url: &str, index: i32) -> Result<String> {
    let book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let chapters = ChapterRepo::list(conn, book_url)?;
    let ch = chapters
        .iter()
        .find(|c| c.index == index)
        .ok_or_else(|| Error::NotFound(format!("chapter {index}")))?;
    if is_cached_format(book_url) {
        return read_epub_chapter(book_url, index);
    }
    let bytes = read_file_range(Path::new(book_url), ch.start, ch.end)?;
    let codec = CharsetCodec::from_label(book.charset.as_deref().unwrap_or("UTF-8"));
    Ok(txt_parser::read_chapter_content(
        &bytes,
        &codec,
        0,
        bytes.len() as i64,
        &ch.title,
    ))
}
