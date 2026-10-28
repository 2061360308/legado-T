//! 文本度量与分行骨架（T2.3 spike）。
//!
//! 正式实现（ADR-04）由 Rust 字体 crate 输出字形度量；此处提供可单测的纯逻辑：
//! 以「中日韩字符按全宽、其余按半宽」的近似宽度做分行，验证分行与分页算法骨架。

/// 单个字符的近似宽度（单位：em 的千分比，简化为整数单位）。
pub fn char_width(c: char) -> u32 {
    if is_wide(c) {
        1000
    } else if c == ' ' {
        500
    } else {
        500
    }
}

fn is_wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
    )
}

/// 一行（字符区间，按字符索引，闭开区间）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineBox {
    pub start: usize,
    pub end: usize,
    pub width: u32,
}

/// 将一段字符文本按最大行宽（与 char_width 同单位）切分为若干行。
pub fn split_lines(text: &str, max_width: u32) -> Vec<LineBox> {
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut width = 0u32;
    for (i, c) in chars.iter().enumerate() {
        if *c == '\n' {
            lines.push(LineBox {
                start,
                end: i,
                width,
            });
            start = i + 1;
            width = 0;
            continue;
        }
        let w = char_width(*c);
        if width + w > max_width && i > start {
            lines.push(LineBox {
                start,
                end: i,
                width,
            });
            start = i;
            width = 0;
        }
        width += w;
    }
    if start < chars.len() {
        lines.push(LineBox {
            start,
            end: chars.len(),
            width,
        });
    }
    lines
}

/// 分页结果（页 = 行区间）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub line_start: usize,
    pub line_end: usize,
}

/// 按「每页最大行数」把行切分为页。
pub fn paginate(lines: &[LineBox], lines_per_page: usize) -> Vec<Page> {
    if lines_per_page == 0 {
        return Vec::new();
    }
    let mut pages = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let end = (i + lines_per_page).min(lines.len());
        pages.push(Page {
            line_start: i,
            line_end: end,
        });
        i = end;
    }
    pages
}
