//! EPUB text import (rbook): reads metadata, spine order, TOC labels and plain text.
//! Images and rich blocks are intentionally left for the next stage.

use std::collections::HashMap;
use std::path::Path;

use fancy_regex::Regex;
use html_escape::decode_html_entities;
use rbook::epub::Epub;

use crate::{Error, Result};

pub struct ParsedChapter {
    pub title: String,
    pub href: String,
    pub text: String,
}

pub struct ParsedEpub {
    pub title: String,
    pub author: String,
    pub intro: String,
    pub chapters: Vec<ParsedChapter>,
}

fn compile(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| Error::Internal(format!("EPUB HTML regex error: {e}")))
}

fn normalize_path(value: &str) -> String {
    value
        .split('#')
        .next()
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string()
}

fn strip_html(html: &str) -> Result<String> {
    let mut text = html.to_string();
    for pattern in [
        r"(?is)<script\b[^>]*>.*?</script>",
        r"(?is)<style\b[^>]*>.*?</style>",
        r"(?is)<head\b[^>]*>.*?</head>",
    ] {
        text = compile(pattern)?.replace_all(&text, "").to_string();
    }
    for pattern in [r"(?i)<br\s*/?>", r"(?i)</p\s*>", r"(?i)</div\s*>"] {
        text = compile(pattern)?.replace_all(&text, "\n").to_string();
    }
    for pattern in [
        r"(?i)</h[1-6]\s*>",
        r"(?i)</li\s*>",
        r"(?i)</tr\s*>",
        r"(?i)</section\s*>",
        r"(?i)</article\s*>",
        r"(?i)</blockquote\s*>",
    ] {
        text = compile(pattern)?.replace_all(&text, "\n").to_string();
    }
    text = compile(r"(?s)<[^>]+>")?.replace_all(&text, "").to_string();
    text = decode_html_entities(&text).to_string();

    let mut lines = Vec::new();
    for line in text.replace('\r', "\n").split('\n') {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        let line = line.trim();
        if line.is_empty() {
            if lines
                .last()
                .map(|value: &String| !value.is_empty())
                .unwrap_or(false)
            {
                lines.push(String::new());
            }
        } else {
            lines.push(line.to_string());
        }
    }
    while lines.last().map(|value| value.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    Ok(lines.join("\n"))
}

fn first_heading(html: &str) -> Option<String> {
    let pattern = Regex::new(r"(?is)<h[1-6]\b[^>]*>(.*?)</h[1-6]>").ok()?;
    let capture = pattern.captures(html).ok()??;
    let raw = capture.get(1)?.as_str();
    let text = compile(r"(?s)<[^>]+>")
        .ok()?
        .replace_all(raw, "")
        .to_string();
    let text = decode_html_entities(&text).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Parse an EPUB and return chapters in spine order.
pub fn parse(path: &Path) -> Result<ParsedEpub> {
    let epub = Epub::open(path).map_err(|e| Error::Invalid(format!("EPUB open failed: {e}")))?;

    let title = epub
        .metadata()
        .title()
        .map(|value| value.value().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });
    let author = epub
        .metadata()
        .creators()
        .next()
        .map(|value| value.value().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string());
    let intro = epub
        .metadata()
        .description()
        .map(|value| strip_html(value.value()).unwrap_or_default())
        .unwrap_or_default();

    let mut toc_titles = HashMap::new();
    if let Some(root) = epub.toc().contents() {
        for entry in root.flatten() {
            let Some(manifest) = entry.manifest_entry() else {
                continue;
            };
            let label = entry.label().trim();
            if !label.is_empty() {
                toc_titles
                    .entry(normalize_path(&manifest.href().to_string()))
                    .or_insert_with(|| label.to_string());
            }
        }
    }

    let mut chapters = Vec::new();
    for (index, entry) in epub.spine().iter().enumerate() {
        let Some(manifest) = entry.manifest_entry() else {
            continue;
        };
        let href = manifest.href().to_string();
        let html = manifest
            .read_str()
            .map_err(|e| Error::Invalid(format!("EPUB chapter read failed: {e}")))?;
        let text = strip_html(&html)?;
        if text.trim().is_empty() {
            continue;
        }
        let title = toc_titles
            .get(&normalize_path(&href))
            .cloned()
            .or_else(|| first_heading(&html))
            .unwrap_or_else(|| format!("Chapter {}", index + 1));
        chapters.push(ParsedChapter { title, href, text });
    }

    if chapters.is_empty() {
        return Err(Error::Invalid("EPUB has no readable text".into()));
    }
    Ok(ParsedEpub {
        title,
        author,
        intro,
        chapters,
    })
}
