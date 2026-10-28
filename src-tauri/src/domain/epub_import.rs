//! EPUB import (rbook): metadata, spine order, TOC labels, text and inline images.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fancy_regex::Regex;
use html_escape::decode_html_entities;
use md5::{Digest, Md5};
use rbook::epub::Epub;

use crate::domain::model::ContentBlock;
use crate::{Error, Result};

pub struct ParsedChapter {
    pub title: String,
    pub href: String,
    pub blocks: Vec<ContentBlock>,
}

pub struct ParsedEpub {
    pub title: String,
    pub author: String,
    pub intro: String,
    pub chapters: Vec<ParsedChapter>,
}

pub struct CoverData {
    pub bytes: Vec<u8>,
    pub ext: String,
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

fn attr(tag: &str, name: &str) -> Option<String> {
    let pattern = format!(
        r#"(?is)\b{}\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#,
        fancy_regex::escape(name)
    );
    let regex = Regex::new(&pattern).ok()?;
    let capture = regex.captures(tag).ok()??;
    let value = capture
        .get(1)
        .or_else(|| capture.get(2))
        .or_else(|| capture.get(3))?
        .as_str();
    Some(decode_html_entities(value).trim().to_string())
}

fn resolve_href(base_href: &str, src: &str) -> Option<String> {
    let src = src.split('#').next().unwrap_or("").trim();
    if src.is_empty() || src.contains("://") || src.starts_with("data:") {
        return None;
    }
    if src.starts_with('/') {
        return Some(normalize_path(src));
    }
    let base = base_href
        .split('#')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");
    let base_dir = match base.rfind('/') {
        Some(index) => &base[..index],
        None => "",
    };
    let joined = if base_dir.is_empty() {
        src.to_string()
    } else {
        format!("{base_dir}/{src}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

fn image_extension(src: &str) -> String {
    let ext = src
        .rsplit('.')
        .next()
        .unwrap_or("bin")
        .split('?')
        .next()
        .unwrap_or("bin")
        .to_ascii_lowercase();
    if ext.chars().all(|ch| ch.is_ascii_alphanumeric()) && ext.len() <= 5 {
        ext
    } else {
        "bin".to_string()
    }
}

fn cover_extension(media_type: &str, href: &str) -> String {
    match media_type.to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/gif" => "gif".to_string(),
        "image/webp" => "webp".to_string(),
        "image/svg+xml" => "svg".to_string(),
        "image/bmp" => "bmp".to_string(),
        _ => image_extension(href),
    }
}

fn first_image_src(html: &str) -> Option<String> {
    let regex = compile(r"(?is)<(?:img|image)\b[^>]*>").ok()?;
    for found in regex.find_iter(html) {
        let Ok(found) = found else {
            continue;
        };
        let tag = found.as_str();
        if let Some(src) = attr(tag, "src").or_else(|| attr(tag, "xlink:href")) {
            if !src.trim().is_empty() {
                return Some(src);
            }
        }
    }
    None
}

/// Extract the EPUB cover without parsing every chapter.
pub fn extract_cover(path: &Path) -> Result<Option<CoverData>> {
    let epub = Epub::open(path).map_err(|e| Error::Invalid(format!("EPUB open failed: {e}")))?;
    if let Some(entry) = epub.manifest().cover_image() {
        if let Ok(bytes) = entry.read_bytes() {
            if !bytes.is_empty() {
                let href = entry.href().to_string();
                return Ok(Some(CoverData {
                    bytes,
                    ext: cover_extension(entry.media_type(), &href),
                }));
            }
        }
    }

    for entry in epub.spine().iter() {
        let Some(manifest) = entry.manifest_entry() else {
            continue;
        };
        let href = manifest.href().to_string();
        let lower = href.to_ascii_lowercase();
        if !lower.contains("cover") && !lower.contains("titlepage") {
            continue;
        }
        let Ok(html) = manifest.read_str() else {
            continue;
        };
        let Some(src) = first_image_src(&html) else {
            continue;
        };
        let Some(resolved) = resolve_href(&href, &src) else {
            continue;
        };
        let Some(image_entry) = epub
            .manifest()
            .by_href(&format!("/{resolved}"))
            .or_else(|| epub.manifest().by_href(&resolved))
        else {
            continue;
        };
        if let Ok(bytes) = image_entry.read_bytes() {
            if !bytes.is_empty() {
                return Ok(Some(CoverData {
                    bytes,
                    ext: cover_extension(image_entry.media_type(), &resolved),
                }));
            }
        }
    }
    Ok(None)
}

fn cache_image(
    epub: &Epub,
    base_href: &str,
    src: &str,
    image_dir: &Path,
    alt: Option<String>,
) -> Option<ContentBlock> {
    let resolved = resolve_href(base_href, src)?;
    let entry = epub
        .manifest()
        .by_href(&format!("/{resolved}"))
        .or_else(|| epub.manifest().by_href(&resolved))?;
    let bytes = entry.read_bytes().ok()?;
    let size = imagesize::blob_size(&bytes).ok()?;
    if size.width == 0 || size.height == 0 {
        return None;
    }
    let digest = Md5::digest(&bytes);
    let file_name = format!("{digest:x}.{}", image_extension(&resolved));
    let path: PathBuf = image_dir.join(file_name);
    if !path.exists() {
        std::fs::create_dir_all(image_dir).ok()?;
        std::fs::write(&path, &bytes).ok()?;
    }
    Some(ContentBlock::Image {
        src: path.to_string_lossy().to_string(),
        width: size.width as u32,
        height: size.height as u32,
        alt,
    })
}

fn html_to_blocks(
    epub: &Epub,
    html: &str,
    href: &str,
    image_dir: &Path,
) -> Result<Vec<ContentBlock>> {
    let image_regex = compile(r"(?is)<img\b[^>]*>")?;
    let mut blocks = Vec::new();
    let mut last = 0usize;
    for found in image_regex.find_iter(html) {
        let Ok(found) = found else {
            continue;
        };
        let segment = &html[last..found.start()];
        push_paragraphs(&mut blocks, segment)?;
        if let Some(src) = attr(found.as_str(), "src") {
            let alt = attr(found.as_str(), "alt");
            if let Some(block) = cache_image(epub, href, &src, image_dir, alt) {
                blocks.push(block);
            }
        }
        last = found.end();
    }
    push_paragraphs(&mut blocks, &html[last..])?;
    Ok(blocks)
}

fn push_paragraphs(blocks: &mut Vec<ContentBlock>, html: &str) -> Result<()> {
    let text = strip_html(html)?;
    for line in text.lines() {
        let line = line.trim();
        if !line.is_empty() {
            blocks.push(ContentBlock::paragraph(line));
        }
    }
    Ok(())
}

/// Parse an EPUB and return chapters in spine order.
pub fn parse(path: &Path, image_dir: &Path) -> Result<ParsedEpub> {
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
        let blocks = html_to_blocks(&epub, &html, &href, image_dir)?;
        if blocks.is_empty() {
            continue;
        }
        let title = toc_titles
            .get(&normalize_path(&href))
            .cloned()
            .or_else(|| first_heading(&html))
            .unwrap_or_else(|| format!("Chapter {}", index + 1));
        chapters.push(ParsedChapter {
            title,
            href,
            blocks,
        });
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