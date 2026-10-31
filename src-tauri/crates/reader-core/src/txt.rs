use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

const PROBE_MIN_GAP: usize = 1000;
const FALLBACK_CHUNK_CHARS: usize = 10 * 1024;
const INTRO_MAX_CHARS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxtTocRule {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rule: String,
    #[serde(default = "default_true")]
    pub enable: bool,
    #[serde(default)]
    pub serial_number: i32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitChapter {
    pub title: String,
    pub content: String,
    pub is_volume: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitResult {
    pub chapters: Vec<SplitChapter>,
    pub intro: String,
    pub rule: String,
}

fn builtin_rules() -> Vec<(String, String)> {
    vec![
        (
            "章节标题".to_string(),
            "^[ \\t　]{0,4}(?:序章|楔子|前言|后记|尾声|番外|第\\s{0,4}[0-9〇零一二两三四五六七八九十百千万壹贰叁肆伍陆柒捌玖拾佰仟]+\\s{0,4}[章节回卷篇].{0,30})$".to_string(),
        ),
        (
            "卷标题".to_string(),
            "^[ \\t　]{0,4}(?:第\\s{0,4}[0-9〇零一二两三四五六七八九十百千万壹贰叁肆伍陆柒捌玖拾佰仟]+\\s{0,4}卷.{0,30})$".to_string(),
        ),
    ]
}

fn compile(expr: &str) -> Option<Regex> {
    let wrapped = if expr.contains("(?m") {
        expr.to_string()
    } else {
        format!("(?m){expr}")
    };
    Regex::new(&wrapped).ok()
}

fn effective_hits(lines: &[&str], regex: &Regex) -> usize {
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            if regex.is_match(line).unwrap_or(false) {
                Some(index)
            } else {
                None
            }
        })
        .collect();
    let mut count = 0usize;
    for window in hits.windows(2) {
        let gap: usize = lines[window[0]..window[1]]
            .iter()
            .map(|line| line.chars().count() + 1)
            .sum();
        if gap > PROBE_MIN_GAP {
            count += 1;
        }
    }
    count
}

fn pick_rule(text: &str, rules: &[TxtTocRule]) -> Option<(String, String)> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut candidates: Vec<(String, String)> = rules
        .iter()
        .filter(|rule| rule.enable && !rule.rule.trim().is_empty())
        .map(|rule| (rule.rule.clone(), rule.name.clone()))
        .collect();
    candidates.extend(builtin_rules());
    let mut best: Option<(usize, String, String)> = None;
    for (expr, name) in candidates {
        let Some(regex) = compile(&expr) else {
            continue;
        };
        let hits = effective_hits(&lines, &regex);
        if hits > 0 && best.as_ref().map(|item| hits > item.0).unwrap_or(true) {
            best = Some((hits, expr, name));
        }
    }
    best.map(|(_, expr, name)| (expr, name))
}

fn split_by_rule(text: &str, expr: &str) -> SplitResult {
    let Some(regex) = compile(expr) else {
        return split_fallback(text);
    };
    let mut chapters: Vec<SplitChapter> = Vec::new();
    let mut current: Option<SplitChapter> = None;
    let mut intro_lines: Vec<String> = Vec::new();
    for line in text.split('\n') {
        if regex.is_match(line).unwrap_or(false) {
            if let Some(chapter) = current.take() {
                chapters.push(chapter);
            }
            current = Some(SplitChapter {
                title: line.trim().to_string(),
                content: String::new(),
                is_volume: false,
            });
        } else if let Some(chapter) = current.as_mut() {
            chapter.content.push_str(line);
            chapter.content.push('\n');
        } else {
            intro_lines.push(line.to_string());
        }
    }
    if let Some(chapter) = current.take() {
        chapters.push(chapter);
    }
    for chapter in &mut chapters {
        chapter.content = chapter.content.trim().to_string();
    }
    let intro = if chapters.is_empty() {
        intro_lines
            .join("\n")
            .trim()
            .chars()
            .take(INTRO_MAX_CHARS)
            .collect()
    } else {
        intro_lines
            .join("\n")
            .trim()
            .chars()
            .take(INTRO_MAX_CHARS)
            .collect()
    };
    if chapters.is_empty() {
        return split_fallback(text);
    }
    SplitResult {
        chapters,
        intro,
        rule: expr.to_string(),
    }
}

fn split_fallback(text: &str) -> SplitResult {
    let chars: Vec<char> = text.chars().collect();
    let mut chapters = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let mut end = (start + FALLBACK_CHUNK_CHARS).min(chars.len());
        if end < chars.len() {
            if let Some(offset) = chars[end..].iter().position(|ch| *ch == '\n') {
                end += offset + 1;
            }
        }
        let content: String = chars[start..end].iter().collect();
        let content = content.trim().to_string();
        if !content.is_empty() {
            chapters.push(SplitChapter {
                title: format!("第{}部分", chapters.len() + 1),
                content,
                is_volume: false,
            });
        }
        start = end;
    }
    SplitResult {
        chapters,
        intro: text.trim().chars().take(INTRO_MAX_CHARS).collect(),
        rule: String::new(),
    }
}

pub fn split_txt(text: &str, rules: &[TxtTocRule], rule_expr: Option<&str>) -> SplitResult {
    if let Some(expr) = rule_expr.filter(|value| !value.trim().is_empty()) {
        let result = split_by_rule(text, expr);
        if !result.chapters.is_empty() {
            return result;
        }
    }
    if let Some((expr, _name)) = pick_rule(text, rules) {
        let result = split_by_rule(text, &expr);
        if !result.chapters.is_empty() {
            return result;
        }
    }
    split_fallback(text)
}
