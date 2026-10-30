use serde::{Deserialize, Serialize};

use crate::domain::model::ContentBlock;
use crate::domain::text_measure::char_width;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LayoutStyle {
    pub font_size: f32,
    pub title_font_size: f32,
    pub title_bold: bool,
    pub title_mode: i32,
    pub title_top_spacing: f32,
    pub title_bottom_spacing: f32,
    pub line_spacing: f32,
    pub paragraph_spacing: f32,
    pub text_bold: i32,
    pub letter_spacing: f32,
    pub paragraph_indent: u8,
    pub padding_left: f32,
    pub padding_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub full_justify: bool,
    pub bottom_justify: bool,
    pub use_zh_layout: bool,
    pub view_width: f32,
    pub view_height: f32,
    #[serde(default)]
    pub image_style: String,
    #[serde(default)]
    pub scroll: bool,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            font_size: 20.0,
            title_font_size: 24.0,
            title_bold: true,
            title_mode: 0,
            title_top_spacing: 0.0,
            title_bottom_spacing: 0.0,
            line_spacing: 1.2,
            paragraph_spacing: 4.0,
            text_bold: 0,
            letter_spacing: 0.1,
            paragraph_indent: 2,
            padding_left: 20.0,
            padding_right: 20.0,
            padding_top: 32.0,
            padding_bottom: 20.0,
            full_justify: true,
            bottom_justify: true,
            use_zh_layout: true,
            view_width: 360.0,
            view_height: 640.0,
            image_style: "DEFAULT".to_string(),
            scroll: false,
        }
    }
}

impl LayoutStyle {
    pub fn normalized(mut self) -> (Self, bool) {
        let width_too_small = self.padding_left + self.padding_right >= self.view_width;
        let height_too_small = self.padding_top + self.padding_bottom >= self.view_height;
        let fell_back = width_too_small || height_too_small;
        if width_too_small {
            self.padding_left = 20.0;
            self.padding_right = 20.0;
        }
        if height_too_small {
            self.padding_top = 20.0;
            self.padding_bottom = 5.0;
        }
        if self.font_size < 10.0 {
            self.font_size = 10.0;
        }
        if self.line_spacing < 1.0 {
            self.line_spacing = 1.0;
        }
        (self, fell_back)
    }

    pub fn line_height(&self) -> f32 {
        self.font_size * self.line_spacing
    }

    pub fn content_width(&self) -> f32 {
        (self.view_width - self.padding_left - self.padding_right).max(1.0)
    }

    pub fn content_height(&self) -> f32 {
        (self.view_height - self.padding_top - self.padding_bottom).max(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharPos {
    pub ch: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageLine {
    pub src: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub bold: bool,
    pub light: bool,
    pub justified: bool,
    pub start_char: i32,
    pub end_char: i32,
    pub chars: Vec<CharPos>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutPage {
    pub index: usize,
    pub lines: Vec<TextLine>,
    pub start_char: i32,
    pub end_char: i32,
    pub chapter_title: String,
    #[serde(default)]
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutResult {
    pub pages: Vec<LayoutPage>,
    pub margin_fell_back: bool,
}

struct BrokenLine {
    text: String,
    width: f32,
    start_char: usize,
    end_char: usize,
    paragraph_end: bool,
}

struct BuildLine {
    text: String,
    width: f32,
    height: f32,
    font_size: f32,
    bold: bool,
    indent: f32,
    start_char: i32,
    end_char: i32,
    gap_before: f32,
    paragraph_end: bool,
}

enum BuildItem {
    Text(BuildLine),
    Image {
        src: String,
        width: f32,
        height: f32,
        gap_before: f32,
        full_page: bool,
    },
}

fn is_forbidden_at_line_start(c: char) -> bool {
    matches!(
        c,
        '，' | '。'
            | '！'
            | '？'
            | '；'
            | '：'
            | '、'
            | '）'
            | '】'
            | '》'
            | '」'
            | '』'
            | '〕'
            | '〉'
            | '…'
            | '—'
            | '～'
            | '·'
            | ','
            | '.'
            | '!'
            | '?'
            | ';'
            | ':'
            | ')'
            | ']'
            | '}'
            | '»'
    )
}

fn is_forbidden_at_line_end(c: char) -> bool {
    matches!(
        c,
        '（' | '【' | '《' | '「' | '『' | '〔' | '〈' | '“' | '‘' | '(' | '[' | '{' | '«'
    )
}

fn width_px(c: char, font_size: f32) -> f32 {
    char_width(c) as f32 / 1000.0 * font_size
}

fn advance_px(c: char, font_size: f32, letter_spacing: f32) -> f32 {
    width_px(c, font_size) + letter_spacing * font_size
}

fn break_text(
    text: &str,
    max_width: f32,
    font_size: f32,
    zh_layout: bool,
    letter_spacing: f32,
    first_indent: f32,
) -> Vec<BrokenLine> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    if chars.is_empty() {
        return out;
    }
    let mut limit = (max_width - first_indent).max(font_size);
    let mut start = 0usize;
    let mut width = 0.0f32;
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\n' {
            if i > start {
                out.push(BrokenLine {
                    text: chars[start..i].iter().collect(),
                    width,
                    start_char: start,
                    end_char: i,
                    paragraph_end: true,
                });
            }
            start = i + 1;
            width = 0.0;
            i += 1;
            continue;
        }
        let w = advance_px(chars[i], font_size, letter_spacing);
        if width + w > limit && i > start {
            let mut brk = i;
            if zh_layout {
                while brk > start + 1 && is_forbidden_at_line_start(chars[brk]) {
                    brk -= 1;
                }
                while brk > start + 1 && is_forbidden_at_line_end(chars[brk - 1]) {
                    brk -= 1;
                }
            }
            let line_width = chars[start..brk]
                .iter()
                .map(|c| advance_px(*c, font_size, letter_spacing))
                .sum();
            out.push(BrokenLine {
                text: chars[start..brk].iter().collect(),
                width: line_width,
                start_char: start,
                end_char: brk,
                paragraph_end: false,
            });
            start = brk;
            width = 0.0;
            limit = max_width;
            continue;
        }
        width += w;
        i += 1;
    }
    if start < chars.len() {
        out.push(BrokenLine {
            text: chars[start..].iter().collect(),
            width,
            start_char: start,
            end_char: chars.len(),
            paragraph_end: true,
        });
    }
    out
}

fn build_line(
    broken: BrokenLine,
    x: f32,
    y: f32,
    max_width: f32,
    font_size: f32,
    bold: bool,
    light: bool,
    justify: bool,
    letter_spacing: f32,
    height: f32,
) -> TextLine {
    let chars: Vec<char> = broken.text.chars().collect();
    let n = chars.len();
    let extra = if justify && n > 1 && broken.width < max_width {
        (max_width - broken.width) / (n - 1) as f32
    } else {
        0.0
    };
    let mut cursor = x;
    let mut positions = Vec::with_capacity(n);
    for c in chars {
        let w = advance_px(c, font_size, letter_spacing);
        positions.push(CharPos {
            ch: c.to_string(),
            x: cursor,
            y,
            width: w,
        });
        cursor += w + extra;
    }
    TextLine {
        text: broken.text,
        x,
        y,
        width: if justify { max_width } else { broken.width },
        height,
        font_size,
        bold,
        light,
        justified: justify,
        start_char: broken.start_char as i32,
        end_char: broken.end_char as i32,
        chars: positions,
        image: None,
    }
}

pub fn layout(title: &str, paragraphs: &[String], style: &LayoutStyle) -> LayoutResult {
    let blocks = paragraphs
        .iter()
        .cloned()
        .map(ContentBlock::paragraph)
        .collect::<Vec<_>>();
    layout_blocks(title, &blocks, style)
}

pub fn layout_blocks(title: &str, blocks: &[ContentBlock], style: &LayoutStyle) -> LayoutResult {
    let (style, margin_fell_back) = style.clone().normalized();
    let max_width = style.content_width();
    let content_height = style.content_height();
    let line_height = style.line_height();

    let mut title_lines: Vec<TextLine> = Vec::new();
    if style.title_mode != 2 && !title.trim().is_empty() {
        for broken in break_text(
            title.trim(),
            max_width,
            style.title_font_size,
            style.use_zh_layout,
            0.0,
            0.0,
        ) {
            let x = if style.title_mode == 1 {
                (style.view_width - broken.width) / 2.0
            } else {
                style.padding_left
            };
            let line = build_line(
                broken,
                x,
                style.padding_top + style.title_top_spacing,
                max_width,
                style.title_font_size,
                style.title_bold,
                false,
                false,
                0.0,
                style.title_font_size * 1.5,
            );
            title_lines.push(line);
        }
    }

    if blocks.is_empty() {
        let total_title_height: f32 = title_lines.iter().map(|l| l.height).sum();
        let offset = if total_title_height < content_height {
            (content_height - total_title_height) / 2.0
        } else {
            0.0
        };
        for line in &mut title_lines {
            line.y += offset;
            for ch in &mut line.chars {
                ch.y = line.y;
            }
        }
        return LayoutResult {
            pages: vec![LayoutPage {
                index: 0,
                lines: title_lines,
                start_char: 0,
                end_char: 0,
                chapter_title: title.to_string(),
                height: style.view_height,
            }],
            margin_fell_back,
        };
    }

    let mut items: Vec<BuildItem> = Vec::new();
    let mut paragraph_start = 0usize;
    let mut first_item = true;
    for block in blocks {
        match block {
            ContentBlock::Paragraph { text } => {
                let first_indent = if style.paragraph_indent > 0 && !text.trim().is_empty() {
                    style.paragraph_indent as f32 * style.font_size
                } else {
                    0.0
                };
                let broken = break_text(
                    text,
                    max_width,
                    style.font_size,
                    style.use_zh_layout,
                    style.letter_spacing,
                    first_indent,
                );
                let para_char_len = text.chars().count();
                for (i, b) in broken.into_iter().enumerate() {
                    let gap = if i == 0 {
                        if first_item {
                            10.0 + style.title_bottom_spacing
                        } else {
                            style.paragraph_spacing
                        }
                    } else {
                        0.0
                    };
                    first_item = false;
                    items.push(BuildItem::Text(BuildLine {
                        text: b.text,
                        width: b.width,
                        height: line_height,
                        font_size: style.font_size,
                        bold: style.text_bold == 1,
                        indent: if i == 0 { first_indent } else { 0.0 },
                        start_char: (paragraph_start + b.start_char) as i32,
                        end_char: (paragraph_start + b.end_char) as i32,
                        gap_before: gap,
                        paragraph_end: b.paragraph_end,
                    }));
                }
                paragraph_start += para_char_len + 1;
            }
            ContentBlock::Image {
                src, width, height, ..
            } => {
                if *width == 0 || *height == 0 {
                    continue;
                }
                let image_style = if style.image_style.trim().is_empty() {
                    "DEFAULT"
                } else {
                    style.image_style.as_str()
                };
                let mut image_width = *width as f32;
                let mut image_height = *height as f32;
                let full_page = image_style.eq_ignore_ascii_case("SINGLE");
                if image_style.eq_ignore_ascii_case("FULL") {
                    image_width = max_width;
                    image_height = image_height * max_width / (*width as f32);
                    if image_height > content_height {
                        image_height = content_height;
                        image_width = *width as f32 * content_height / (*height as f32);
                    }
                } else if full_page {
                    let scale = (max_width / image_width).min(content_height / image_height);
                    image_width *= scale;
                    image_height *= scale;
                } else {
                    let mut scale = (max_width / image_width).min(1.0);
                    if image_height * scale > content_height {
                        scale = content_height / image_height;
                    }
                    image_width *= scale;
                    image_height *= scale;
                }
                let gap = if first_item {
                    10.0 + style.title_bottom_spacing
                } else {
                    style.paragraph_spacing
                };
                first_item = false;
                items.push(BuildItem::Image {
                    src: src.clone(),
                    width: image_width,
                    height: image_height,
                    gap_before: gap,
                    full_page,
                });
            }
            ContentBlock::ImagePage {
                src, width, height, ..
            } => {
                if *width == 0 || *height == 0 {
                    continue;
                }
                let scale = (max_width / (*width as f32))
                    .min(content_height / (*height as f32))
                    .min(1.0);
                let gap = if first_item {
                    10.0 + style.title_bottom_spacing
                } else {
                    style.paragraph_spacing
                };
                first_item = false;
                items.push(BuildItem::Image {
                    src: src.clone(),
                    width: *width as f32 * scale,
                    height: *height as f32 * scale,
                    gap_before: gap,
                    full_page: true,
                });
            }
        }
    }

    if style.scroll {
        let mut lines = title_lines;
        let mut cursor_y = style.padding_top;
        for line in &lines {
            cursor_y = cursor_y.max(line.y + line.height);
        }
        let mut start_char: i32 = 0;
        let mut end_char: i32 = 0;
        let mut has_content = false;
        for item in items {
            match item {
                BuildItem::Text(b) => {
                    let gap = if lines.is_empty() { 0.0 } else { b.gap_before };
                    cursor_y += gap;
                    if !has_content {
                        start_char = b.start_char;
                        has_content = true;
                    }
                    end_char = b.end_char;
                    let line = build_line(
                        BrokenLine {
                            text: b.text,
                            width: b.width,
                            start_char: b.start_char as usize,
                            end_char: b.end_char as usize,
                            paragraph_end: b.paragraph_end,
                        },
                        style.padding_left + b.indent,
                        cursor_y,
                        (max_width - b.indent).max(1.0),
                        b.font_size,
                        b.bold,
                        style.text_bold == 2,
                        style.full_justify && !b.paragraph_end,
                        style.letter_spacing,
                        b.height,
                    );
                    cursor_y += b.height;
                    lines.push(line);
                }
                BuildItem::Image {
                    src,
                    width,
                    height,
                    gap_before,
                    full_page,
                } => {
                    let _ = full_page;
                    let gap = if lines.is_empty() { 0.0 } else { gap_before };
                    cursor_y += gap;
                    let x = style.padding_left + (max_width - width).max(0.0) / 2.0;
                    let y = cursor_y;
                    cursor_y += height;
                    lines.push(TextLine {
                        text: String::new(),
                        x,
                        y,
                        width,
                        height,
                        font_size: 0.0,
                        bold: false,
                        light: false,
                        justified: false,
                        start_char,
                        end_char,
                        chars: Vec::new(),
                        image: Some(ImageLine { src, width, height }),
                    });
                    has_content = true;
                }
            }
        }
        let total_height = (cursor_y + style.padding_bottom).max(style.view_height);
        return LayoutResult {
            pages: vec![LayoutPage {
                index: 0,
                lines,
                start_char,
                end_char,
                chapter_title: title.to_string(),
                height: total_height,
            }],
            margin_fell_back,
        };
    }

    let mut pages: Vec<LayoutPage> = Vec::new();
    let mut current_lines: Vec<TextLine> = title_lines;
    let mut cursor_y = style.padding_top;
    for line in &current_lines {
        cursor_y = cursor_y.max(line.y + line.height);
    }
    let mut page_start_char: i32 = 0;
    let mut page_end_char: i32 = 0;
    let mut has_content = false;

    for item in items {
        match item {
            BuildItem::Text(b) => {
                let gap = if current_lines.is_empty() {
                    0.0
                } else {
                    b.gap_before
                };
                let projected = cursor_y + gap + b.height;
                if !current_lines.is_empty() && projected > style.padding_top + content_height {
                    pages.push(LayoutPage {
                        index: pages.len(),
                        lines: std::mem::take(&mut current_lines),
                        start_char: page_start_char,
                        end_char: page_end_char,
                        chapter_title: title.to_string(),
                        height: style.view_height,
                    });
                    cursor_y = style.padding_top;
                    page_start_char = b.start_char;
                    page_end_char = b.end_char;
                    has_content = true;
                } else {
                    cursor_y += gap;
                    if !has_content {
                        page_start_char = b.start_char;
                        has_content = true;
                    }
                    page_end_char = b.end_char;
                }
                let line = build_line(
                    BrokenLine {
                        text: b.text,
                        width: b.width,
                        start_char: b.start_char as usize,
                        end_char: b.end_char as usize,
                        paragraph_end: b.paragraph_end,
                    },
                    style.padding_left + b.indent,
                    cursor_y,
                    (max_width - b.indent).max(1.0),
                    b.font_size,
                    b.bold,
                    style.text_bold == 2,
                    style.full_justify && !b.paragraph_end,
                    style.letter_spacing,
                    b.height,
                );
                cursor_y += b.height;
                current_lines.push(line);
            }
            BuildItem::Image {
                src,
                width,
                height,
                gap_before,
                full_page,
            } => {
                if full_page {
                    if !current_lines.is_empty() {
                        pages.push(LayoutPage {
                            index: pages.len(),
                            lines: std::mem::take(&mut current_lines),
                            start_char: page_start_char,
                            end_char: page_end_char,
                            chapter_title: title.to_string(),
                            height: style.view_height,
                        });
                    }
                    let y = style.padding_top + (content_height - height).max(0.0) / 2.0;
                    let x = style.padding_left + (max_width - width).max(0.0) / 2.0;
                    current_lines.push(TextLine {
                        text: String::new(),
                        x,
                        y,
                        width,
                        height,
                        font_size: 0.0,
                        bold: false,
                        light: false,
                        justified: false,
                        start_char: page_start_char,
                        end_char: page_end_char,
                        chars: Vec::new(),
                        image: Some(ImageLine { src, width, height }),
                    });
                    pages.push(LayoutPage {
                        index: pages.len(),
                        lines: std::mem::take(&mut current_lines),
                        start_char: page_start_char,
                        end_char: page_end_char,
                        chapter_title: title.to_string(),
                        height: style.view_height,
                    });
                    cursor_y = style.padding_top;
                    has_content = true;
                    continue;
                }

                let gap = if current_lines.is_empty() {
                    0.0
                } else {
                    gap_before
                };
                let projected = cursor_y + gap + height;
                if !current_lines.is_empty() && projected > style.padding_top + content_height {
                    pages.push(LayoutPage {
                        index: pages.len(),
                        lines: std::mem::take(&mut current_lines),
                        start_char: page_start_char,
                        end_char: page_end_char,
                        chapter_title: title.to_string(),
                        height: style.view_height,
                    });
                    cursor_y = style.padding_top;
                } else {
                    cursor_y += gap;
                }
                let x = style.padding_left + (max_width - width).max(0.0) / 2.0;
                let y = cursor_y;
                cursor_y += height;
                current_lines.push(TextLine {
                    text: String::new(),
                    x,
                    y,
                    width,
                    height,
                    font_size: 0.0,
                    bold: false,
                    light: false,
                    justified: false,
                    start_char: page_start_char,
                    end_char: page_end_char,
                    chars: Vec::new(),
                    image: Some(ImageLine { src, width, height }),
                });
                has_content = true;
            }
        }
    }

    if !current_lines.is_empty() {
        pages.push(LayoutPage {
            index: pages.len(),
            lines: current_lines,
            start_char: page_start_char,
            end_char: page_end_char,
            chapter_title: title.to_string(),
            height: style.view_height,
        });
    }

    if pages.is_empty() {
        pages.push(LayoutPage {
            index: 0,
            lines: Vec::new(),
            start_char: 0,
            end_char: 0,
            chapter_title: title.to_string(),
            height: style.view_height,
        });
    }

    LayoutResult {
        pages,
        margin_fell_back,
    }
}

pub fn find_scroll_offset(page: &LayoutPage, char_pos: i32, padding_top: f32) -> f32 {
    if page.lines.is_empty() {
        return 0.0;
    }
    let line = page
        .lines
        .iter()
        .find(|line| char_pos >= line.start_char && char_pos < line.end_char)
        .or_else(|| page.lines.last())
        .unwrap();
    (line.y - padding_top).max(0.0)
}

pub fn find_page_index(pages: &[LayoutPage], char_pos: i32) -> usize {
    if pages.is_empty() {
        return 0;
    }
    if char_pos <= 0 {
        return 0;
    }
    for (i, page) in pages.iter().enumerate() {
        if char_pos < page.end_char {
            return i;
        }
    }
    pages.len() - 1
}

pub fn page_char_pos(page: &LayoutPage) -> i32 {
    page.start_char
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> LayoutStyle {
        LayoutStyle {
            view_width: 320.0,
            view_height: 480.0,
            font_size: 20.0,
            padding_left: 10.0,
            padding_right: 10.0,
            padding_top: 10.0,
            padding_bottom: 10.0,
            ..Default::default()
        }
    }

    #[test]
    fn no_forbidden_punctuation_at_line_start() {
        let text = "这是一个很长的句子，用来测试中文禁则断行功能是否生效，标点不应该出现在行首。"
            .repeat(3);
        let result = layout("标题", &[text], &style());
        for page in &result.pages {
            for line in &page.lines {
                if let Some(first) = line.text.chars().next() {
                    assert!(
                        !is_forbidden_at_line_start(first),
                        "line starts with forbidden: {}",
                        line.text
                    );
                }
            }
        }
    }

    #[test]
    fn pagination_and_char_offsets() {
        let paragraphs: Vec<String> = (0..30)
            .map(|i| format!("第{i}段内容，长度足够用于分页测试。"))
            .collect();
        let result = layout("第一章", &paragraphs, &style());
        assert!(result.pages.len() > 1);
        assert_eq!(result.pages[0].start_char, 0);
        assert!(result.pages.last().unwrap().end_char > 0);
    }

    #[test]
    fn scroll_layout_is_one_tall_page() {
        let mut s = style();
        s.scroll = true;
        let paragraphs: Vec<String> = (0..30)
            .map(|i| format!("Paragraph {i} with enough content for a tall scroll layout."))
            .collect();
        let result = layout("Scroll", &paragraphs, &s);
        assert_eq!(result.pages.len(), 1);
        assert!(result.pages[0].height > s.view_height);
        assert!(result.pages[0].lines.len() > 10);
    }

    #[test]
    fn empty_chapter_centers_title() {
        let result = layout("空章", &[], &style());
        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.pages[0].lines.len(), 1);
        assert!(result.pages[0].lines[0].y > 10.0);
    }

    #[test]
    fn oversized_margin_falls_back() {
        let mut s = style();
        s.padding_left = 500.0;
        let result = layout("标题", &["正文".to_string()], &s);
        assert!(result.margin_fell_back);
        assert_eq!(result.pages[0].lines[0].x, 20.0);
    }

    #[test]
    fn justify_spreads_chars() {
        let text = "一二三四五六七八九十一二三四五六七八九十".to_string();
        let mut s = style();
        s.view_width = 260.0;
        s.font_size = 20.0;
        s.full_justify = true;
        let result = layout("", &[text], &s);
        let line = &result.pages[0].lines[0];
        if line.text.chars().count() > 1 {
            let last = line.chars.last().unwrap();
            let first = line.chars.first().unwrap();
            assert!(last.x > first.x + 20.0);
        }
    }
    #[test]
    fn letter_spacing_expands_advance() {
        let text = "一二三四五".to_string();
        let mut a = style();
        a.full_justify = false;
        let mut b = style();
        b.full_justify = false;
        b.letter_spacing = 0.2;
        let ra = layout("", &[text.clone()], &a);
        let rb = layout("", &[text], &b);
        assert!(rb.pages[0].lines[0].width > ra.pages[0].lines[0].width);
        assert!(rb.pages[0].lines[0].chars[1].x > ra.pages[0].lines[0].chars[1].x);
    }

    #[test]
    fn paragraph_indent_offsets_first_line() {
        let mut s = style();
        s.font_size = 20.0;
        s.paragraph_indent = 2;
        let result = layout("", &["第一段内容".to_string()], &s);
        let line = &result.pages[0].lines[0];
        assert!(line.x > s.padding_left);
        assert!(line.chars[0].x >= s.padding_left + 39.0);
    }

    #[test]
    fn text_bold_flag_follows_style() {
        let mut s = style();
        s.text_bold = 1;
        let result = layout("", &["正文".to_string()], &s);
        assert!(result.pages[0].lines[0].bold);
        let mut light = style();
        light.text_bold = 2;
        let result = layout("", &["正文".to_string()], &light);
        assert!(result.pages[0].lines[0].light);
    }
}
