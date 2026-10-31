use std::io::Cursor;

use fancy_regex::Regex;
use html_escape::decode_html_entities;
use rbook::epub::Epub;

use crate::ebook_types::{EbookBlock, EbookBook, EbookChapter, EbookCover};

fn regex(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

fn normalize_path(value: &str) -> String {
    value
        .split('#')
        .next()
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string()
}

fn strip_html(html: &str) -> String {
    let mut text = html.to_string();
    for pattern in [
        r"(?is)<script\b[^>]*>.*?</script>",
        r"(?is)<style\b[^>]*>.*?</style>",
        r"(?is)<head\b[^>]*>.*?</head>",
    ] {
        if let Some(regex) = regex(pattern) {
            text = regex.replace_all(&text, "").into_owned();
        }
    }
    for pattern in [r"(?i)<br\s*/?>", r"(?i)</p\s*>", r"(?i)</div\s*>"] {
        if let Some(regex) = regex(pattern) {
            text = regex.replace_all(&text, "\n").into_owned();
        }
    }
    for pattern in [
        r"(?i)</h[1-6]\s*>",
        r"(?i)</li\s*>",
        r"(?i)</tr\s*>",
        r"(?i)</section\s*>",
        r"(?i)</article\s*>",
        r"(?i)</blockquote\s*>",
    ] {
        if let Some(regex) = regex(pattern) {
            text = regex.replace_all(&text, "\n").into_owned();
        }
    }
    if let Some(regex) = regex(r"(?s)<[^>]+>") {
        text = regex.replace_all(&text, "").into_owned();
    }
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
    lines.join("\n")
}

fn first_heading(html: &str) -> Option<String> {
    let pattern = regex(r"(?is)<h[1-6]\b[^>]*>(.*?)</h[1-6]>")?;
    let capture = pattern.captures(html).ok()??;
    let raw = capture.get(1)?.as_str();
    let text = regex(r"(?s)<[^>]+>")?.replace_all(raw, "").into_owned();
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
    let regex = regex(&pattern)?;
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
    let regex = regex(r"(?is)<(?:img|image)\b[^>]*>")?;
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

fn image_block(epub: &Epub, base_href: &str, src: &str, alt: Option<String>) -> Option<EbookBlock> {
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
    Some(EbookBlock::Image {
        href: resolved.clone(),
        bytes,
        ext: image_extension(&resolved),
        width: size.width as u32,
        height: size.height as u32,
        alt,
    })
}

fn html_to_blocks(epub: &Epub, html: &str, href: &str) -> Vec<EbookBlock> {
    let Some(image_regex) = regex(r"(?is)<img\b[^>]*>") else {
        return text_blocks(&strip_html(html));
    };
    let mut blocks = Vec::new();
    let mut last = 0usize;
    for found in image_regex.find_iter(html) {
        let Ok(found) = found else {
            continue;
        };
        blocks.extend(text_blocks(&strip_html(&html[last..found.start()])));
        if let Some(src) = attr(found.as_str(), "src") {
            let alt = attr(found.as_str(), "alt");
            if let Some(block) = image_block(epub, href, &src, alt) {
                blocks.push(block);
            }
        }
        last = found.end();
    }
    blocks.extend(text_blocks(&strip_html(&html[last..])));
    blocks
}

fn text_blocks(text: &str) -> Vec<EbookBlock> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| EbookBlock::Paragraph {
            text: line.to_string(),
        })
        .collect()
}

struct TocChapter {
    title: String,
    path: String,
    fragment: Option<String>,
    level: i32,
    is_volume: bool,
}

fn collect_toc(entry: rbook::epub::toc::EpubTocEntry<'_>, level: i32, out: &mut Vec<TocChapter>) {
    let has_children = !entry.is_empty();
    let mut node_index = None;
    if let Some(href) = entry.href() {
        let path = href.path().as_str().to_string();
        let fragment = href.fragment().map(str::to_string);
        let title = entry.label().trim().to_string();
        if !path.is_empty() {
            out.push(TocChapter {
                title,
                path,
                fragment,
                level,
                is_volume: has_children,
            });
            node_index = Some(out.len() - 1);
        }
    }
    if has_children {
        if let Some(index) = node_index {
            out[index].is_volume = true;
        }
        for child in entry.iter() {
            collect_toc(child, level + 1, out);
        }
    }
}

fn find_fragment_start(html: &str, fragment: &str) -> Option<usize> {
    for quote in ['"', '\''] {
        let needle = format!("id={quote}{fragment}{quote}");
        if let Some(position) = html.find(&needle) {
            return html[..position].rfind('<').or(Some(position));
        }
    }
    None
}

fn fragment_html(html: &str, fragment: &str, next: Option<&str>) -> String {
    let Some(start) = find_fragment_start(html, fragment) else {
        return html.to_string();
    };
    let end = next
        .and_then(|value| find_fragment_start(html, value))
        .filter(|value| *value > start)
        .unwrap_or(html.len());
    html[start..end].to_string()
}

fn cover_from_epub(epub: &Epub) -> Option<EbookCover> {
    if let Some(entry) = epub.manifest().cover_image() {
        if let Ok(bytes) = entry.read_bytes() {
            if !bytes.is_empty() {
                let href = entry.href().to_string();
                return Some(EbookCover {
                    bytes,
                    ext: cover_extension(entry.media_type(), &href),
                });
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
                return Some(EbookCover {
                    bytes,
                    ext: cover_extension(image_entry.media_type(), &resolved),
                });
            }
        }
    }
    None
}

pub fn extract_cover(bytes: &[u8]) -> Result<Option<EbookCover>, String> {
    let epub = Epub::read(Cursor::new(bytes.to_vec()))
        .map_err(|error| format!("EPUB open failed: {error}"))?;
    Ok(cover_from_epub(&epub))
}

pub fn parse(bytes: &[u8], title_fallback: &str) -> Result<EbookBook, String> {
    let epub = Epub::read(Cursor::new(bytes.to_vec()))
        .map_err(|error| format!("EPUB open failed: {error}"))?;
    let title = epub
        .metadata()
        .title()
        .map(|value| value.value().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| title_fallback.to_string());
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
        .map(|value| strip_html(value.value()))
        .unwrap_or_default();
    let mut toc_nodes = Vec::new();
    if let Some(root) = epub.toc().contents() {
        for entry in root.iter() {
            collect_toc(entry, 0, &mut toc_nodes);
        }
    }

    let mut chapters = Vec::new();
    if !toc_nodes.is_empty() {
        let first_path = normalize_path(&toc_nodes[0].path);
        for entry in epub.spine().iter() {
            let Some(manifest) = entry.manifest_entry() else {
                continue;
            };
            let href = manifest.href().to_string();
            if normalize_path(&href) == first_path {
                break;
            }
            let html = manifest
                .read_str()
                .map_err(|error| format!("EPUB chapter read failed: {error}"))?;
            let blocks = html_to_blocks(&epub, &html, &href);
            if blocks.is_empty() {
                continue;
            }
            let title =
                first_heading(&html).unwrap_or_else(|| format!("Chapter {}", chapters.len() + 1));
            chapters.push(EbookChapter {
                title,
                href: href.clone(),
                blocks,
                is_volume: false,
                toc_level: 0,
            });
        }
        for (index, node) in toc_nodes.iter().enumerate() {
            let path = node.path.clone();
            let Some(manifest) = epub
                .manifest()
                .by_href(&path)
                .or_else(|| epub.manifest().by_href(path.trim_start_matches('/')))
            else {
                continue;
            };
            let html = manifest
                .read_str()
                .map_err(|error| format!("EPUB chapter read failed: {error}"))?;
            let next_fragment = toc_nodes[index + 1..]
                .iter()
                .find(|other| other.path == path)
                .and_then(|other| other.fragment.as_deref());
            let content = match node.fragment.as_deref() {
                Some(fragment) => fragment_html(&html, fragment, next_fragment),
                None => html.clone(),
            };
            let blocks = html_to_blocks(&epub, &content, &path);
            if blocks.is_empty() {
                continue;
            }
            let title = if node.title.trim().is_empty() {
                first_heading(&html).unwrap_or_else(|| format!("Chapter {}", chapters.len() + 1))
            } else {
                node.title.clone()
            };
            let href = match node.fragment.as_deref() {
                Some(fragment) => format!("{path}#{fragment}"),
                None => path,
            };
            chapters.push(EbookChapter {
                title,
                href,
                blocks,
                is_volume: node.is_volume,
                toc_level: node.level,
            });
        }
    } else {
        for (index, entry) in epub.spine().iter().enumerate() {
            let Some(manifest) = entry.manifest_entry() else {
                continue;
            };
            let href = manifest.href().to_string();
            let html = manifest
                .read_str()
                .map_err(|error| format!("EPUB chapter read failed: {error}"))?;
            let blocks = html_to_blocks(&epub, &html, &href);
            if blocks.is_empty() {
                continue;
            }
            let title = first_heading(&html).unwrap_or_else(|| format!("Chapter {}", index + 1));
            chapters.push(EbookChapter {
                title,
                href,
                blocks,
                is_volume: false,
                toc_level: 0,
            });
        }
    }
    if chapters.is_empty() {
        return Err("EPUB has no readable text".to_string());
    }
    Ok(EbookBook {
        title,
        author,
        intro,
        kind: "EPUB".to_string(),
        cover: cover_from_epub(&epub),
        chapters,
    })
}
