use std::collections::HashMap;

use rusqlite::Connection;
use serde::Serialize;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::domain::content_processor;
use crate::domain::import_service;
use crate::domain::layout_engine::{self, LayoutPage, LayoutStyle};
use crate::domain::model::{Book, BookChapter, ContentBlock, ReplaceRule};
use crate::domain::storage::BookStorage;
use crate::domain::txt_parser;
use crate::platform::charset_codec::CharsetCodec;
use crate::platform::filesystem::read_file_range;
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
struct CachedChapter {
    style_key: String,
    pages: Vec<LayoutPage>,
    effective_rules: Vec<ReplaceRule>,
    chapter_title: String,
    paragraphs: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ReaderCache {
    book_url: String,
    chapters: HashMap<i32, CachedChapter>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderSnapshot {
    pub book: Book,
    pub chapter_index: i32,
    pub page_index: usize,
    pub chapter_title: String,
    pub pages: Vec<LayoutPage>,
    pub effective_rules: Vec<ReplaceRule>,
    pub chapter_paragraphs: Vec<String>,
    pub total_chapters: usize,
    pub page_start_char: i32,
    pub page_end_char: i32,
    pub scroll_offset: f32,
    pub margin_fell_back: bool,
}

fn style_key(style: &LayoutStyle, converter: &str) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        style.font_size,
        style.title_font_size,
        style.title_bold,
        style.line_spacing,
        style.paragraph_spacing,
        style.text_bold,
        style.letter_spacing,
        style.paragraph_indent,
        style.padding_left,
        style.padding_right,
        style.padding_top,
        style.padding_bottom,
        style.full_justify,
        style.bottom_justify,
        style.use_zh_layout,
        style.view_width,
        style.view_height,
        style.title_mode,
        style.title_top_spacing,
        style.title_bottom_spacing,
        style.image_style,
        style.scroll,
        converter
    )
}

fn current_style_key(conn: &Connection, style: &LayoutStyle) -> String {
    let converter = KeyValue::get(conn, "chineseConverterType")
        .ok()
        .flatten()
        .unwrap_or_default();
    style_key(style, &converter)
}

fn find_chapter<'a>(chapters: &'a [BookChapter], index: i32) -> Result<&'a BookChapter> {
    chapters
        .iter()
        .find(|c| c.index == index)
        .ok_or_else(|| Error::NotFound(format!("chapter {index}")))
}

fn core_style(style: &LayoutStyle) -> reader_core::domain::layout_engine::LayoutStyle {
    serde_json::from_value(serde_json::to_value(style).expect("layout style serializes"))
        .expect("layout style deserializes")
}

fn core_blocks(blocks: &[ContentBlock]) -> Vec<reader_core::domain::model::ContentBlock> {
    serde_json::from_value(serde_json::to_value(blocks).expect("content blocks serialize"))
        .expect("content blocks deserialize")
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorePipelinePages {
    display_title: String,
    pages: Vec<LayoutPage>,
    paragraphs: Vec<String>,
}

fn core_pages(result: &reader_core::pipeline::ChapterOutput) -> CorePipelinePages {
    serde_json::from_value(serde_json::to_value(result).expect("pipeline result serializes"))
        .expect("pipeline result deserializes")
}

fn load_chapter(
    conn: &Connection,
    storage: &BookStorage,
    book: &Book,
    chapters: &[BookChapter],
    index: i32,
    style: &LayoutStyle,
) -> Result<CachedChapter> {
    let chapter = find_chapter(chapters, index)?;
    let override_key = format!("chapterOverride:{}:{}", book.book_url, index);
    let override_content = KeyValue::get(conn, &override_key)?;
    let is_epub = import_service::is_cached_book(book, storage);
    let (raw, blocks) = if is_epub && override_content.is_none() {
        (
            String::new(),
            Some(import_service::read_epub_chapter_blocks(
                storage,
                book,
                chapter.index,
            )?),
        )
    } else {
        let raw = match override_content {
            Some(content) => content,
            None => {
                let paths = storage.paths(book)?;
                let bytes = read_file_range(&paths.source, chapter.start, chapter.end)?;
                let codec = CharsetCodec::from_label(book.charset.as_deref().unwrap_or("UTF-8"));
                txt_parser::read_chapter_raw(&bytes, &codec, 0, bytes.len() as i64, &chapter.title)
            }
        };
        (raw, None)
    };
    let use_replace_rule = book
        .read_config
        .as_ref()
        .and_then(|config| config.use_replace_rule)
        .unwrap_or(true);
    let request = reader_core::pipeline::ChapterRequest {
        process: reader_core::content::ProcessRequest {
            title: chapter.title.clone(),
            content: raw,
            book_name: book.name.clone(),
            book_origin: book.origin_name.clone(),
            rules: content_processor::core_rules(conn)?,
            include_title: false,
            use_replace: true,
            use_replace_rule,
            chinese_convert: true,
            chinese_converter_type: content_processor::converter_direction(conn),
            re_segment: book
                .read_config
                .as_ref()
                .map(|config| config.re_segment)
                .unwrap_or(false),
            keep_duplicate_title: KeyValue::get(
                conn,
                &format!("keepDuplicateTitle:{}", book.book_url),
            )?
            .map(|value| value == "true")
            .unwrap_or(false),
            paragraph_indent: "\u{3000}\u{3000}".to_string(),
            is_android8: false,
        },
        blocks: blocks.as_deref().map(core_blocks),
        style: core_style(style),
    };
    let result = reader_core::pipeline::process_chapter(&request);
    let native_rules = content_processor::native_rules(conn)?;
    let effective_rules = result
        .effective_rules
        .iter()
        .filter_map(|rule| native_rules.get(&rule.id).cloned())
        .collect();
    let pages = core_pages(&result);
    Ok(CachedChapter {
        style_key: current_style_key(conn, style),
        pages: pages.pages,
        effective_rules,
        chapter_title: pages.display_title,
        paragraphs: pages.paragraphs,
    })
}

fn get_or_load(
    conn: &Connection,
    storage: &BookStorage,
    cache: &mut ReaderCache,
    book: &Book,
    chapters: &[BookChapter],
    index: i32,
    style: &LayoutStyle,
) -> Result<CachedChapter> {
    let key = current_style_key(conn, style);
    if let Some(cached) = cache.chapters.get(&index) {
        if cached.style_key == key {
            return Ok(cached.clone());
        }
    }
    let loaded = load_chapter(conn, storage, book, chapters, index, style)?;
    cache.chapters.insert(index, loaded.clone());
    Ok(loaded)
}

fn trim_cache(cache: &mut ReaderCache, current: i32) {
    cache
        .chapters
        .retain(|index, _| (*index - current).abs() <= 1);
}

pub fn open(
    conn: &Connection,
    storage: &BookStorage,
    cache: &mut ReaderCache,
    book_url: &str,
    chapter_index: Option<i32>,
    char_pos: Option<i32>,
    style: &LayoutStyle,
) -> Result<ReaderSnapshot> {
    let book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let chapters = ChapterRepo::list(conn, book_url)?;
    if chapters.is_empty() {
        return Err(Error::Invalid("书籍没有章节".into()));
    }

    if cache.book_url != book_url {
        cache.book_url = book_url.to_string();
        cache.chapters.clear();
    }

    let target = chapter_index
        .unwrap_or(book.dur_chapter_index)
        .clamp(0, chapters.len() as i32 - 1);
    let pos = char_pos.unwrap_or(if chapter_index.is_some() {
        0
    } else {
        book.dur_chapter_pos
    });

    let current = get_or_load(conn, storage, cache, &book, &chapters, target, style)?;
    for neighbor in [target - 1, target + 1] {
        if neighbor >= 0 && (neighbor as usize) < chapters.len() {
            if let Ok(loaded) = get_or_load(conn, storage, cache, &book, &chapters, neighbor, style)
            {
                cache.chapters.insert(neighbor, loaded);
            }
        }
    }
    trim_cache(cache, target);

    let page_index = if style.scroll {
        0
    } else {
        layout_engine::find_page_index(&current.pages, pos)
    };
    let page = current.pages.get(page_index);
    let (start_char, end_char) = page.map(|p| (p.start_char, p.end_char)).unwrap_or((0, 0));
    let scroll_offset = if style.scroll {
        page.map(|p| layout_engine::find_scroll_offset(p, pos, style.padding_top))
            .unwrap_or(0.0)
    } else {
        0.0
    };
    let mut book = book;
    storage.hydrate_book(&mut book);

    Ok(ReaderSnapshot {
        book,
        chapter_index: target,
        page_index,
        chapter_title: current.chapter_title,
        pages: current.pages,
        effective_rules: current.effective_rules,
        chapter_paragraphs: current.paragraphs,
        total_chapters: chapters.len(),
        page_start_char: start_char,
        page_end_char: end_char,
        scroll_offset,
        margin_fell_back: false,
    })
}

pub fn save_chapter_content(
    conn: &Connection,
    book_url: &str,
    chapter_index: i32,
    content: &str,
) -> Result<()> {
    let key = format!("chapterOverride:{book_url}:{chapter_index}");
    KeyValue::set(conn, &key, content)
}

impl ReaderCache {
    pub fn invalidate_chapter(&mut self, chapter_index: i32) {
        self.chapters.remove(&chapter_index);
    }

    pub fn invalidate_book(&mut self, book_url: &str) {
        if self.book_url == book_url {
            self.chapters.clear();
        }
    }

    pub fn invalidate_all(&mut self) {
        self.chapters.clear();
    }
}

pub fn save_progress(
    conn: &Connection,
    book_url: &str,
    chapter_index: i32,
    chapter_pos: i32,
    chapter_title: &str,
) -> Result<()> {
    let mut book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    book.dur_chapter_index = chapter_index;
    book.dur_chapter_pos = chapter_pos;
    book.dur_chapter_title = Some(chapter_title.to_string());
    book.dur_chapter_time = chrono::Utc::now().timestamp();
    BookRepo::upsert(conn, &book)?;
    Ok(())
}

pub fn effective_rules(
    conn: &Connection,
    storage: &BookStorage,
    cache: &mut ReaderCache,
    book_url: &str,
    chapter_index: i32,
    style: &LayoutStyle,
) -> Result<Vec<ReplaceRule>> {
    let book =
        BookRepo::get(conn, book_url)?.ok_or_else(|| Error::NotFound(book_url.to_string()))?;
    let chapters = ChapterRepo::list(conn, book_url)?;
    if cache.book_url != book_url {
        cache.book_url = book_url.to_string();
        cache.chapters.clear();
    }
    let cached = get_or_load(conn, storage, cache, &book, &chapters, chapter_index, style)?;
    Ok(cached.effective_rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_key_changes_with_font_size() {
        let a = LayoutStyle::default();
        let mut b = a.clone();
        b.font_size = 22.0;
        assert_ne!(style_key(&a, ""), style_key(&b, ""));
    }
}
