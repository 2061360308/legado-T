use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use crate::data::keyvalue::KeyValue;
use crate::data::repositories::{BookRepo, ChapterRepo};
use crate::domain::content_processor;
use crate::domain::layout_engine::{self, LayoutPage, LayoutStyle};
use crate::domain::model::{Book, BookChapter, ReplaceRule};
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
    pub margin_fell_back: bool,
}

fn style_key(style: &LayoutStyle, converter: &str) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
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

fn load_chapter(
    conn: &Connection,
    book: &Book,
    chapters: &[BookChapter],
    index: i32,
    style: &LayoutStyle,
) -> Result<CachedChapter> {
    let chapter = find_chapter(chapters, index)?;
    let override_key = format!("chapterOverride:{}:{}", book.book_url, index);
    let raw = match KeyValue::get(conn, &override_key)? {
        Some(content) => content,
        None => {
            let bytes = read_file_range(Path::new(&book.book_url), chapter.start, chapter.end)?;
            let codec = CharsetCodec::from_label(book.charset.as_deref().unwrap_or("UTF-8"));
            txt_parser::read_chapter_raw(&bytes, &codec, 0, bytes.len() as i64, &chapter.title)
        }
    };
    let processed = content_processor::process(conn, &chapter.title, &raw, book, false)?;
    let display_title = content_processor::display_title(conn, &chapter.title, book)?;
    let result = layout_engine::layout(&display_title, &processed.contents, style);
    Ok(CachedChapter {
        style_key: current_style_key(conn, style),
        pages: result.pages,
        effective_rules: processed.effective_rules,
        chapter_title: display_title,
        paragraphs: processed.contents,
    })
}

fn get_or_load(
    conn: &Connection,
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
    let loaded = load_chapter(conn, book, chapters, index, style)?;
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

    let current = get_or_load(conn, cache, &book, &chapters, target, style)?;
    for neighbor in [target - 1, target + 1] {
        if neighbor >= 0 && (neighbor as usize) < chapters.len() {
            if let Ok(loaded) = get_or_load(conn, cache, &book, &chapters, neighbor, style) {
                cache.chapters.insert(neighbor, loaded);
            }
        }
    }
    trim_cache(cache, target);

    let page_index = layout_engine::find_page_index(&current.pages, pos);
    let page = current.pages.get(page_index);
    let (start_char, end_char) = page.map(|p| (p.start_char, p.end_char)).unwrap_or((0, 0));

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
    let cached = get_or_load(conn, cache, &book, &chapters, chapter_index, style)?;
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
