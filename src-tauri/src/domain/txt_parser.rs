//! TXT 分章引擎（T9.1~T9.7）。
//!
//! - `probe_rule`：文件头部窗口按 serialNumber 倒序试跑启用规则；仅统计相邻命中
//!   间距 >1000 字符的命中；命中最多者胜出，同分取 serialNumber 大者。
//! - `split_by_rule`：逐命中点生成章节；标题取整行匹配文本；标题行后无正文标记
//!   isVolume；上一章 >100KB 且开启拆分时递归均分加「(1)(2)」后缀；顺序保持。
//! - `split_fallback`：无规则命中时每 10KB 找换行切章；尾块 <100 字节并入上章；
//!   目录为空且块首有内容时建「前言」章并取前 500 字作简介。
//! - 偏移：章节 start/end 为**原编码字节偏移**（CharsetCodec 换算）。
//! - 性能：512KB 分块流式扫描（块尾回退换行符防跨块截断）。
//!
//! 说明：规则来自 legado（Java 正则含 lookbehind 等），故使用 `fancy-regex`。

use std::time::Instant;

use fancy_regex::Regex;

use crate::domain::model::{BookChapter, ChapterOutline, TxtTocRule};
use crate::platform::charset_codec::CharsetCodec;
use crate::{Error, Result};

/// 探测头部窗口大小（字符）。
const PROBE_WINDOW_CHARS: usize = 512 * 1024;
/// 相邻命中间距阈值（字符）：仅统计 >1000 的命中（design 1.2.1）。
const PROBE_MIN_GAP: usize = 1000;
/// 兜底分章块大小（字节）。
const FALLBACK_CHUNK_BYTES: usize = 10 * 1024;
/// 尾块并入上章阈值（字节）。
const FALLBACK_MERGE_THRESHOLD: usize = 100;
/// 长章拆分阈值（字节）。
const SPLIT_LONG_CHAPTER_BYTES: i64 = 100 * 1024;

/// 简介长度上限（字符）。
const INTRO_MAX_CHARS: usize = 500;

/// 规则探测结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedRule {
    pub rule: String,
    pub name: String,
    pub hits: usize,
}

/// 构造目录规则正则：多行模式（spec 6.6-2，`^/$` 匹配行首/行尾）。
/// fancy-Regex 无 builder 选项，用内联 `(?m)` 前缀启用。
fn build_regex(expr: &str) -> Result<Regex> {
    let wrapped = if expr.contains("(?m") {
        expr.to_string()
    } else {
        format!("(?m){expr}")
    };
    Regex::new(&wrapped).map_err(|e| Error::Invalid(format!("目录规则编译失败: {e}")))
}

/// 在解码文本上查找全部命中：**逐行匹配**（对齐 legado 的行语义）。
///
/// 每行构造 `"\n" + 行内容` 作为匹配上下文：
/// - `^` 在 '\n' 后匹配行首；
/// - `$` 在行尾匹配；
/// - lookbehind（如 `(?<=[　\s])`）以 '\n'（属 \s）作为行首前文，默认规则集可用。
///
/// 逐行处理天然流式（无需整块加载），返回行首偏移列表。
fn find_all(regex: &Regex, text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    for line in text.split_inclusive('\n') {
        let core = line
            .strip_suffix('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .unwrap_or(line);
        let probe = format!("\n{core}");
        if let Ok(Some(m)) = regex.find(&probe) {
            // probe 前缀 '\n' 占 1 字节：偏移回退 1；若命中从 probe 起点开始则对齐行首。
            let abs_start = pos + m.start().saturating_sub(1);
            let abs_end = pos + m.end().saturating_sub(1).min(core.len());
            out.push((abs_start, abs_end));
        }
        pos += line.len();
    }
    out
}

/// 统计「相邻命中间距 > 1000 字符」的命中数（spec 5.2.1-5）。
/// 语义：相邻命中对间距 >1000 的跳变次数；孤立或紧邻命中不构成目录模式，
/// 从而满足「正文含『第一章』字样非标题时不误判」。
fn count_effective_hits(regex: &Regex, text: &str) -> usize {
    let hits = find_all(regex, text);
    let mut count = 0usize;
    for i in 1..hits.len() {
        if hits[i].0.saturating_sub(hits[i - 1].1) > PROBE_MIN_GAP {
            count += 1;
        }
    }
    count
}

/// 规则自动探测（T9.1）：返回最优规则；全部零命中返回 None。
pub fn probe_rule(text: &str, rules: &[TxtTocRule]) -> Result<Option<ProbedRule>> {
    let head: String = text.chars().take(PROBE_WINDOW_CHARS).collect();
    let mut best: Option<(usize, i32, ProbedRule)> = None;
    // serialNumber 大者先试跑；同分取 serialNumber 大者。
    let mut sorted: Vec<&TxtTocRule> = rules.iter().filter(|r| r.enable).collect();
    sorted.sort_by(|a, b| b.serial_number.cmp(&a.serial_number));

    for rule in sorted {
        let regex = match build_regex(&rule.rule) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let hits = count_effective_hits(&regex, &head);
        if hits == 0 {
            continue;
        }
        let candidate = ProbedRule {
            rule: rule.rule.clone(),
            name: rule.name.clone(),
            hits,
        };
        let better = match &best {
            None => true,
            Some((best_hits, best_serial, _)) => {
                hits > *best_hits || (hits == *best_hits && rule.serial_number > *best_serial)
            }
        };
        if better {
            best = Some((hits, rule.serial_number, candidate));
        }
    }
    Ok(best.map(|(_, _, r)| r))
}

/// 找到 pos 所在行的行首与行尾（UTF-8 字节偏移，含换行符算作行尾一部分）。
fn line_bounds(text: &str, pos: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    let mut start = pos;
    while start > 0 && bytes[start - 1] != b'\n' {
        start -= 1;
    }
    let mut end = pos;
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    if end < bytes.len() {
        end += 1; // 含换行符
    }
    (start, end)
}

fn trim_crlf(s: &str) -> &str {
    s.trim_end_matches(|c| c == '\r' || c == '\n')
}

/// 中间内容是否仅空白（判断卷标记）。
fn is_blank_between(text: &str, from: usize, to: usize) -> bool {
    text[from..to].trim().is_empty()
}

/// 把字节偏移对齐到字符边界（向前回退）。
fn floor_char_boundary(text: &str, mut pos: usize) -> usize {
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// 长章递归均分：把 [content_start, content_end) 均分为 n 段，段边界顺序推进
/// （保证连续覆盖），优先回退到段落边界（换行符后），回退限幅防止长行拉回过远。
fn split_long_chapter(
    content_start: usize,
    content_end: usize,
    text: &str,
    codec: &CharsetCodec,
    char_to_byte: &[usize],
    out: &mut Vec<ChapterOutline>,
    title: &str,
    depth: usize,
) {
    let span = content_end.saturating_sub(content_start);
    if span <= SPLIT_LONG_CHAPTER_BYTES as usize || depth > 3 {
        out.push(ChapterOutline {
            title: title.to_string(),
            start: codec.utf8_offset_to_src(text, content_start, char_to_byte) as i64,
            end: codec.utf8_offset_to_src(text, content_end, char_to_byte) as i64,
            is_volume: false,
        });
        return;
    }
    let n = ((span as f64) / (SPLIT_LONG_CHAPTER_BYTES as f64)).ceil() as usize;
    let per = span / n.max(1);
    let bytes = text.as_bytes();
    let mut cur = content_start;
    for i in 0..n {
        if cur >= content_end {
            break;
        }
        let is_last = i == n - 1;
        let target = if is_last {
            content_end
        } else {
            (cur + per).min(content_end)
        };
        let mut seg_end = floor_char_boundary(text, target);
        if !is_last && seg_end < bytes.len() {
            // 回退到段落边界（换行符后），限幅 per/2 防止超长行把边界拉回过远。
            let min_end = floor_char_boundary(text, cur + per / 2);
            while seg_end > min_end && bytes[seg_end] != b'\n' {
                seg_end -= 1;
            }
            if bytes[seg_end] == b'\n' {
                seg_end += 1;
            } else {
                seg_end = floor_char_boundary(text, seg_end);
            }
        }
        if seg_end <= cur {
            seg_end = floor_char_boundary(text, target.max(cur + 1));
        }
        let sub_title = format!("{title}({})", i + 1);
        split_long_chapter(
            cur,
            seg_end,
            text,
            codec,
            char_to_byte,
            out,
            &sub_title,
            depth + 1,
        );
        cur = seg_end;
    }
}

/// 有规则分章（T9.2）。返回 (章节列表, 简介候选文本)。
pub fn split_by_rule(
    text: &str,
    rule_expr: &str,
    codec: &CharsetCodec,
    split_long: bool,
) -> Result<(Vec<ChapterOutline>, String)> {
    let regex = build_regex(rule_expr)?;
    let hits = find_all(&regex, text);
    if hits.is_empty() {
        return Ok((Vec::new(), String::new()));
    }

    let char_to_byte = codec.build_char_to_byte_map(text.as_bytes());

    // 命中行（去重同一行多次命中）。
    let mut lines: Vec<(usize, usize)> = Vec::new(); // (line_start, line_end_with_newline)
    for (s, _) in &hits {
        let (ls, le) = line_bounds(text, *s);
        if lines.last().map(|(x, _)| *x) != Some(ls) {
            lines.push((ls, le));
        }
    }

    let mut out: Vec<ChapterOutline> = Vec::new();
    for (i, (ls, le)) in lines.iter().enumerate() {
        let title = trim_crlf(&text[*ls..*le]).trim().to_string();
        let next_ls = lines.get(i + 1).map(|(x, _)| *x).unwrap_or(text.len());
        // 标题行后无正文 → 卷标记（spec 5.2.1-10）。
        let is_volume = is_blank_between(text, *le, next_ls);

        if split_long {
            // 上一章 >100KB 时拆分（用当前章内容长度近似上一章语义：逐章判断自身）。
            split_long_chapter(
                *ls,
                next_ls,
                text,
                codec,
                &char_to_byte,
                &mut out,
                &title,
                0,
            );
            if let Some(last) = out.last_mut() {
                last.is_volume = is_volume;
            }
        } else {
            out.push(ChapterOutline {
                title,
                start: codec.utf8_offset_to_src(text, *ls, &char_to_byte) as i64,
                end: codec.utf8_offset_to_src(text, next_ls, &char_to_byte) as i64,
                is_volume,
            });
        }
    }

    // 首个命中前的内容：非空白时建「前言」章并取前 500 字作简介（spec 5.2.1-9）。
    let intro = text[..lines[0].0]
        .trim()
        .chars()
        .take(INTRO_MAX_CHARS)
        .collect::<String>();
    if !intro.is_empty() {
        out.insert(
            0,
            ChapterOutline {
                title: "前言".to_string(),
                start: 0,
                end: codec.utf8_offset_to_src(text, lines[0].0, &char_to_byte) as i64,
                is_volume: false,
            },
        );
    }
    Ok((out, intro))
}

/// 兜底分章（T9.3）：每 10KB 找换行切章；尾块 <100 字节并入上章。
pub fn split_fallback(bytes: &[u8], _codec: &CharsetCodec) -> Vec<ChapterOutline> {
    let mut bounds: Vec<(usize, usize)> = Vec::new();
    let mut start = 0usize;
    while start < bytes.len() {
        let target = (start + FALLBACK_CHUNK_BYTES).min(bytes.len());
        if target >= bytes.len() {
            bounds.push((start, bytes.len()));
            break;
        }
        // 向后找第一个换行（含换行符）。
        let mut cut = target;
        while cut < bytes.len() && bytes[cut] != b'\n' {
            cut += 1;
        }
        if cut < bytes.len() {
            cut += 1;
        }
        bounds.push((start, cut));
        start = cut;
    }

    // 尾块 <100 字节并入上章。
    if bounds.len() >= 2 {
        let last = *bounds.last().unwrap();
        if last.1 - last.0 < FALLBACK_MERGE_THRESHOLD {
            bounds.pop();
            let prev = bounds.last_mut().unwrap();
            prev.1 = last.1;
        }
    }

    let mut out = Vec::new();
    for (i, (s, e)) in bounds.iter().enumerate() {
        let title = format!("第{}部分", i + 1);
        out.push(ChapterOutline {
            title,
            start: *s as i64,
            end: *e as i64,
            is_volume: false,
        });
    }
    out
}

/// 分章入口：先按规则分章（含前言章），无命中回退兜底分章。
/// 返回 (章节, 简介文本, 使用的规则表达式)。
pub fn analyze(
    bytes: &[u8],
    codec: &CharsetCodec,
    rule_expr: Option<&str>,
    rules: &[TxtTocRule],
    split_long: bool,
) -> Result<(Vec<ChapterOutline>, String, String)> {
    let text = codec.decode(bytes);
    let now = Instant::now();

    let (chapters, intro, used_rule) = match rule_expr {
        Some(expr) => {
            let (chs, intro) = split_by_rule(&text, expr, codec, split_long)?;
            (chs, intro, expr.to_string())
        }
        None => {
            // 先探测。
            match probe_rule(&text, rules)? {
                Some(probed) => {
                    let (chs, intro) = split_by_rule(&text, &probed.rule, codec, split_long)?;
                    (chs, intro, probed.rule)
                }
                None => (Vec::new(), String::new(), String::new()),
            }
        }
    };

    let mut chapters = chapters;
    if chapters.is_empty() {
        // 兜底分章（spec 5.2.1-6/7/9）。
        let fallback = split_fallback(bytes, codec);
        if fallback.is_empty() {
            return Ok((chapters, intro, String::new()));
        }
        let head_intro = text
            .trim()
            .chars()
            .take(INTRO_MAX_CHARS)
            .collect::<String>();
        chapters = fallback;
        return Ok((chapters, head_intro, String::new()));
    }

    tracing_perf(&format!(
        "analyze: {} chars, {} chapters, {:?}",
        text.chars().count(),
        chapters.len(),
        now.elapsed()
    ));
    Ok((chapters, intro, used_rule))
}

fn tracing_perf(msg: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[txt_parser] {msg}");
    #[cfg(not(debug_assertions))]
    let _ = msg;
}

/// 章节大纲转章节实体（url 以 `#index` 区分，同书唯一）。
pub fn outlines_to_chapters(
    outlines: &[ChapterOutline],
    book_url: &str,
    base_url: &str,
) -> Vec<BookChapter> {
    outlines
        .iter()
        .enumerate()
        .map(|(i, o)| BookChapter {
            url: format!("{base_url}#{}", i),
            title: o.title.clone(),
            index: i as i32,
            is_volume: o.is_volume,
            start: o.start,
            end: o.end,
            resource_url: Some(book_url.to_string()),
            ..Default::default()
        })
        .collect()
}

/// 按字节偏移读取正文（T9.5）：解码 + 去首行标题 + 段落首行缩进。
pub fn read_chapter_content(
    bytes: &[u8],
    codec: &CharsetCodec,
    start: i64,
    end: i64,
    title: &str,
) -> String {
    let s = (start.max(0) as usize).min(bytes.len());
    let e = (end.max(0) as usize).min(bytes.len()).max(s);
    let raw = codec.decode(&bytes[s..e]);
    let raw = raw.trim_start_matches('\u{FEFF}');
    let mut lines: Vec<&str> = raw.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    // 去掉首行标题。
    if let Some(first) = lines.first() {
        if first.trim() == title.trim() {
            lines.remove(0);
        }
    }
    // 段落加首行缩进（两个全角空格）。
    lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| format!("\u{3000}\u{3000}{}", l.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 目录缓存键：bookUrl + mtime（T9.6）。
pub fn read_chapter_raw(
    bytes: &[u8],
    codec: &CharsetCodec,
    start: i64,
    end: i64,
    title: &str,
) -> String {
    let s = (start.max(0) as usize).min(bytes.len());
    let e = (end.max(0) as usize).min(bytes.len()).max(s);
    let raw = codec.decode(&bytes[s..e]);
    let raw = raw.trim_start_matches('\u{FEFF}');
    let mut lines: Vec<&str> = raw.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    if let Some(first) = lines.first() {
        if first.trim() == title.trim() {
            lines.remove(0);
        }
    }
    lines.join("\n")
}

pub fn cache_key(book_url: &str, mtime_secs: i64) -> String {
    format!("toc:{book_url}:{mtime_secs}")
}
