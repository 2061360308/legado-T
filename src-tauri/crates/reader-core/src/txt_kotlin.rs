use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use fancy_regex::Regex;
use serde::Serialize;

use crate::txt::TxtTocRule;

const MAX_LENGTH_WITH_TOC: usize = 102_400;
const MAX_LENGTH_WITHOUT_TOC: usize = 10 * 1024;
const INTRO_MAX_CHARS: usize = 500;
const RULE_PROBE_CHARS: usize = 100_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KChapter {
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub is_volume: bool,
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KResult {
    pub chapters: Vec<KChapter>,
    pub intro: String,
    pub rule: String,
    pub charset: String,
    pub total_word_count: usize,
}

fn probe(bytes: &[u8]) -> &'static Encoding {
    if bytes.len() >= 3 && &bytes[..3] == b"\xEF\xBB\xBF" {
        return UTF_8;
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return encoding_rs::UTF_16LE;
    }
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        return encoding_rs::UTF_16BE;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return UTF_8;
    }
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    detector.guess(None, true)
}

fn resolve_encoding(label: &str, bytes: &[u8]) -> &'static Encoding {
    let label = label.trim().trim_start_matches("UTF-8 BOM").trim();
    if label.is_empty() {
        probe(bytes)
    } else {
        Encoding::for_label(label.as_bytes()).unwrap_or_else(|| probe(bytes))
    }
}

fn decode(bytes: &[u8], encoding: &'static Encoding) -> (String, usize) {
    let mut offset = 0usize;
    if encoding == UTF_8 && bytes.len() >= 3 && &bytes[..3] == b"\xEF\xBB\xBF" {
        offset = 3;
    }
    let (text, _, _) = encoding.decode(&bytes[offset..]);
    (text.into_owned(), offset)
}

fn encoded_len(text: &str, encoding: &'static Encoding) -> usize {
    encoding.encode(text).0.len()
}

fn compile(rule: &str) -> Option<Regex> {
    let wrapped = if rule.contains("(?m") {
        rule.to_string()
    } else {
        format!("(?m){rule}")
    };
    Regex::new(&wrapped).ok()
}

fn pick_rule(text: &str, rules: &[TxtTocRule]) -> Option<(String, String)> {
    let mut best: Option<(usize, String, String)> = None;
    let mut ordered: Vec<&TxtTocRule> = rules.iter().filter(|rule| rule.enable).collect();
    ordered.sort_by(|a, b| b.serial_number.cmp(&a.serial_number));
    let probe: String = text.chars().take(RULE_PROBE_CHARS).collect();
    for rule in ordered {
        let Some(regex) = compile(&rule.rule) else {
            continue;
        };
        let mut start = 0usize;
        let mut count = 0usize;
        for item in regex.find_iter(&probe).flatten() {
            if start == 0 || item.start().saturating_sub(start) > 1000 {
                count += 1;
                start = item.end();
            }
        }
        if count >= best.as_ref().map(|item| item.0).unwrap_or(1) {
            best = Some((count, rule.rule.clone(), rule.name.clone()));
        }
    }
    best.map(|(_, rule, name)| (rule, name))
}

fn split_long_chapter(
    title: &str,
    start: i64,
    text: &str,
    encoding: &'static Encoding,
    out: &mut Vec<KChapter>,
) {
    if text.chars().count() <= MAX_LENGTH_WITH_TOC {
        out.push(KChapter {
            title: title.to_string(),
            start,
            end: start + encoded_len(text, encoding) as i64,
            is_volume: false,
            word_count: text.chars().count(),
        });
        return;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    let mut part = 1usize;
    while index < chars.len() {
        let mut end = (index + MAX_LENGTH_WITHOUT_TOC).min(chars.len());
        if end < chars.len() {
            if let Some(offset) = chars[end..].iter().position(|ch| *ch == '\n') {
                end += offset + 1;
            }
        }
        let content: String = chars[index..end].iter().collect();
        let content_start =
            start + encoded_len(&chars[..index].iter().collect::<String>(), encoding) as i64;
        out.push(KChapter {
            title: format!("{title}({part})"),
            start: content_start,
            end: content_start + encoded_len(&content, encoding) as i64,
            is_volume: false,
            word_count: content.chars().count(),
        });
        index = end;
        part += 1;
    }
}

fn split_by_rule(
    text: &str,
    rule: &str,
    start_offset: usize,
    encoding: &'static Encoding,
    split_long: bool,
) -> Option<(Vec<KChapter>, String)> {
    let regex = compile(rule)?;
    let matches: Vec<(usize, usize)> = regex
        .find_iter(text)
        .flatten()
        .map(|item| (item.start(), item.end()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let mut chapters: Vec<KChapter> = Vec::new();
    let mut seek = 0usize;
    let mut cur_offset = start_offset;
    let mut intro = String::new();
    for (match_start, match_end) in matches {
        if match_start < seek {
            continue;
        }
        let content = &text[seek..match_start];
        let content_len = encoded_len(content, encoding);
        if chapters.is_empty() && !content.trim().is_empty() {
            let preface_end = cur_offset + content_len;
            chapters.push(KChapter {
                title: "前言".to_string(),
                start: cur_offset as i64,
                end: preface_end as i64,
                is_volume: false,
                word_count: content.chars().count(),
            });
            intro = content.trim().chars().take(INTRO_MAX_CHARS).collect();
        } else if let Some(last) = chapters.last_mut() {
            last.end = (cur_offset + content_len) as i64;
            last.is_volume = content.trim().is_empty();
        }
        cur_offset += content_len;
        let title = text[match_start..match_end].trim().to_string();
        chapters.push(KChapter {
            title,
            start: cur_offset as i64,
            end: cur_offset as i64,
            is_volume: false,
            word_count: 0,
        });
        seek = match_end;
    }
    if let Some(last) = chapters.last_mut() {
        let rest = &text[seek..];
        last.end = (cur_offset + encoded_len(rest, encoding)) as i64;
        last.word_count = rest.chars().count();
    }
    if split_long {
        let mut split = Vec::new();
        for chapter in chapters {
            let s = chapter.start.max(0) as usize;
            let e = chapter.end.max(chapter.start) as usize;
            let source_start = s.saturating_sub(start_offset);
            let source_end = e.saturating_sub(start_offset);
            let bytes = encoding.encode(text).0;
            let content = encoding
                .decode(&bytes[source_start.min(bytes.len())..source_end.min(bytes.len())])
                .0
                .into_owned();
            split_long_chapter(
                &chapter.title,
                chapter.start,
                &content,
                encoding,
                &mut split,
            );
        }
        chapters = split;
    }
    Some((chapters, intro))
}

fn fallback(text: &str, start_offset: usize, encoding: &'static Encoding) -> KResult {
    let chars: Vec<char> = text.chars().collect();
    let mut chapters = Vec::new();
    let mut index = 0usize;
    let mut block = 1usize;
    let mut part = 1usize;
    let mut consumed = 0usize;
    while index < chars.len() {
        let mut end = (index + MAX_LENGTH_WITHOUT_TOC).min(chars.len());
        if end < chars.len() {
            if let Some(offset) = chars[end..].iter().position(|ch| *ch == '\n') {
                end += offset + 1;
            }
        }
        let content: String = chars[index..end].iter().collect();
        let start =
            start_offset + encoded_len(&chars[..index].iter().collect::<String>(), encoding);
        chapters.push(KChapter {
            title: format!("第{block}段({part})"),
            start: start as i64,
            end: (start + encoded_len(&content, encoding)) as i64,
            is_volume: false,
            word_count: content.chars().count(),
        });
        consumed += content.chars().count();
        index = end;
        part += 1;
        if part > 3 {
            part = 1;
            block += 1;
        }
    }
    KResult {
        chapters,
        intro: text.trim().chars().take(INTRO_MAX_CHARS).collect(),
        rule: String::new(),
        charset: encoding.name().to_string(),
        total_word_count: consumed,
    }
}

pub fn analyze(
    bytes: &[u8],
    charset: &str,
    rules: &[TxtTocRule],
    split_long: bool,
) -> Result<KResult, String> {
    analyze_with_rule(bytes, charset, rules, None, split_long)
}

pub fn analyze_with_rule(
    bytes: &[u8],
    charset: &str,
    rules: &[TxtTocRule],
    rule_expr: Option<&str>,
    split_long: bool,
) -> Result<KResult, String> {
    let encoding = resolve_encoding(charset, bytes);
    let (text, start_offset) = decode(bytes, encoding);
    if text.is_empty() {
        return Err("TXT 文件为空".to_string());
    }
    let picked = match rule_expr.filter(|value| !value.trim().is_empty()) {
        Some(expr) => Some((expr.to_string(), String::new())),
        None => pick_rule(&text, rules),
    };
    if let Some((rule, _name)) = picked {
        if let Some((chapters, intro)) =
            split_by_rule(&text, &rule, start_offset, encoding, split_long)
        {
            return Ok(KResult {
                chapters,
                intro,
                rule,
                charset: encoding.name().to_string(),
                total_word_count: text.chars().count(),
            });
        }
    }
    Ok(fallback(&text, start_offset, encoding))
}
