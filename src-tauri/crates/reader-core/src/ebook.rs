use base64::Engine;
use ebook_rs::{Book, EbookError};
use fancy_regex::Regex;
use html_escape::decode_html_entities;

pub use crate::ebook_types::{EbookBlock, EbookBook, EbookChapter, EbookCover};

fn extension_for_mime(mime: &str, fallback: &str) -> String {
    match mime.to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/gif" => "gif".to_string(),
        "image/webp" => "webp".to_string(),
        "image/svg+xml" => "svg".to_string(),
        _ => {
            let ext = fallback
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
    }
}

fn normalize_href(value: &str) -> String {
    value
        .split('#')
        .next()
        .unwrap_or("")
        .trim_start_matches('/')
        .to_string()
}

fn decode_data_uri(src: &str) -> Option<(Vec<u8>, String)> {
    let rest = src.strip_prefix("data:")?;
    let (meta, payload) = rest.split_once(',')?;
    if !meta.contains(";base64") {
        return None;
    }
    let mime = meta.split(';').next().unwrap_or("image/jpeg");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;
    Some((bytes, extension_for_mime(mime, "")))
}

fn text_blocks(text: &str) -> Vec<EbookBlock> {
    text.replace('\r', "\n")
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .map(|text| EbookBlock::Paragraph { text })
        .collect()
}

fn strip_html(html: &str) -> String {
    let mut text = html.to_string();
    for pattern in [
        r"(?is)<script\b[^>]*>.*?</script>",
        r"(?is)<style\b[^>]*>.*?</style>",
        r"(?is)<head\b[^>]*>.*?</head>",
    ] {
        if let Ok(regex) = Regex::new(pattern) {
            text = regex.replace_all(&text, "").to_string();
        }
    }
    for pattern in [r"(?i)<br\s*/?>", r"(?i)</p\s*>", r"(?i)</div\s*>"] {
        if let Ok(regex) = Regex::new(pattern) {
            text = regex.replace_all(&text, "\n").to_string();
        }
    }
    if let Ok(regex) = Regex::new(r"(?s)<[^>]+>") {
        text = regex.replace_all(&text, "").to_string();
    }
    decode_html_entities(&text).to_string()
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

fn image_block(
    book: &Book,
    section_href: &str,
    src: &str,
    alt: Option<String>,
) -> Option<EbookBlock> {
    if let Some((bytes, ext)) = decode_data_uri(src) {
        let size = imagesize::blob_size(&bytes).ok()?;
        if size.width == 0 || size.height == 0 {
            return None;
        }
        return Some(EbookBlock::Image {
            href: src.to_string(),
            bytes,
            ext,
            width: size.width as u32,
            height: size.height as u32,
            alt,
        });
    }
    let src_path = src.split('?').next().unwrap_or(src).to_string();
    let base = section_href
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");
    let resolved = if base.is_empty() {
        src_path.clone()
    } else {
        format!("{base}/{src_path}")
    };
    let mut candidates = vec![src_path.clone(), resolved.clone()];
    if !src_path.starts_with("OEBPS/") {
        candidates.push(format!("OEBPS/{src_path}"));
    }
    if !resolved.starts_with("OEBPS/") {
        candidates.push(format!("OEBPS/{resolved}"));
    }
    for candidate in candidates {
        if let Ok((bytes, mime)) = book.get_resource_bytes(&candidate) {
            let size = imagesize::blob_size(&bytes).ok()?;
            if size.width == 0 || size.height == 0 {
                continue;
            }
            return Some(EbookBlock::Image {
                href: candidate.clone(),
                bytes,
                ext: extension_for_mime(mime, &candidate),
                width: size.width as u32,
                height: size.height as u32,
                alt,
            });
        }
    }
    None
}

fn html_blocks(book: &Book, section_href: &str, html: &str) -> Vec<EbookBlock> {
    let Ok(image_regex) = Regex::new(r"(?is)<img\b[^>]*>") else {
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
            if let Some(block) = image_block(book, section_href, &src, attr(found.as_str(), "alt"))
            {
                blocks.push(block);
            }
        }
        last = found.end();
    }
    blocks.extend(text_blocks(&strip_html(&html[last..])));
    blocks
}

fn section_blocks(book: &Book, section: &ebook_rs::Section) -> Vec<EbookBlock> {
    let html = if section.processed_html.trim().is_empty() {
        &section.raw_html
    } else {
        &section.processed_html
    };
    if html.to_ascii_lowercase().contains("<img") {
        let blocks = html_blocks(book, &section.href, html);
        if !blocks.is_empty() {
            return blocks;
        }
    }
    text_blocks(&section.plain_text)
}

fn collect_toc(
    points: &[ebook_rs::NavPoint],
    level: i32,
    out: &mut Vec<(String, String, i32, bool)>,
) {
    for point in points {
        if !point.label.trim().is_empty() && !point.href.trim().is_empty() {
            out.push((
                point.label.trim().to_string(),
                point.href.clone(),
                level,
                !point.subitems.is_empty(),
            ));
        }
        collect_toc(&point.subitems, level + 1, out);
    }
}

fn find_section(sections: &[ebook_rs::Section], href: &str) -> Option<usize> {
    let target = normalize_href(href);
    sections.iter().position(|section| {
        normalize_href(&section.href) == target
            || normalize_href(&section.full_path) == target
            || normalize_href(&section.idref) == target
            || normalize_href(&section.href).ends_with(&format!("/{target}"))
            || normalize_href(&section.full_path).ends_with(&format!("/{target}"))
    })
}

pub fn parse(bytes: &[u8], title_fallback: &str) -> Result<EbookBook, String> {
    let book = Book::from_bytes_with_title(bytes, title_fallback).map_err(|error| match error {
        EbookError::DrmProtected(_) => "DRM protected file is not supported".to_string(),
        other => format!("MOBI/AZW parse failed: {other}"),
    })?;
    if book.sections.is_empty() {
        return Err("MOBI/AZW has no readable sections".to_string());
    }

    let metadata = book.metadata();
    let title = if metadata.title.trim().is_empty() {
        title_fallback.to_string()
    } else {
        metadata.title.clone()
    };
    let author = metadata.creator().trim().to_string();
    let intro = metadata
        .description
        .as_deref()
        .map(strip_html)
        .unwrap_or_default();
    let kind = metadata
        .subjects
        .first()
        .cloned()
        .unwrap_or_else(|| "MOBI".to_string());
    let cover = book
        .cover_image()
        .map(|(bytes, mime)| EbookCover {
            ext: extension_for_mime(mime, ""),
            bytes,
        })
        .or_else(|| {
            for index in 1..=20 {
                for ext in ["jpg", "jpeg", "png", "gif", "webp"] {
                    let key = format!("OEBPS/images/img_{index:04}.{ext}");
                    if let Ok((bytes, mime)) = book.get_resource_bytes(&key) {
                        return Some(EbookCover {
                            ext: extension_for_mime(mime, &key),
                            bytes,
                        });
                    }
                }
            }
            None
        });

    let mut toc = Vec::new();
    collect_toc(&book.toc, 0, &mut toc);
    let mut starts: Vec<(usize, String, bool, i32)> = Vec::new();
    for (label, href, level, is_volume) in toc {
        if let Some(index) = find_section(&book.sections, &href) {
            if !starts.iter().any(|(value, _, _, _)| *value == index) {
                starts.push((index, label, is_volume, level));
            }
        }
    }
    starts.sort_by_key(|(index, _, _, _)| *index);

    if starts.is_empty() {
        for (index, section) in book.sections.iter().enumerate() {
            let blocks = section_blocks(&book, section);
            if blocks.is_empty() {
                continue;
            }
            let first_line = section
                .plain_text
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or("正文");
            starts.push((
                index,
                first_line.chars().take(40).collect::<String>(),
                false,
                0,
            ));
        }
    }

    let mut chapters = Vec::new();
    for (position, (index, chapter_title, is_volume, toc_level)) in starts.iter().enumerate() {
        let end = starts
            .get(position + 1)
            .map(|(next, _, _, _)| *next)
            .unwrap_or(book.sections.len());
        let mut blocks = Vec::new();
        for section in book.sections.iter().take(end).skip(*index) {
            blocks.extend(section_blocks(&book, section));
        }
        if blocks.is_empty() {
            continue;
        }
        chapters.push(EbookChapter {
            title: chapter_title.clone(),
            href: String::new(),
            blocks,
            is_volume: *is_volume,
            toc_level: *toc_level,
        });
    }

    if chapters.is_empty() {
        for (index, section) in book.sections.iter().enumerate() {
            let blocks = section_blocks(&book, section);
            if !blocks.is_empty() {
                chapters.push(EbookChapter {
                    title: format!("第{}章", index + 1),
                    href: String::new(),
                    blocks,
                    is_volume: false,
                    toc_level: 0,
                });
            }
        }
    }
    if chapters.is_empty() {
        return Err("MOBI/AZW has no readable content".to_string());
    }
    Ok(EbookBook {
        title,
        author,
        intro,
        kind,
        cover,
        chapters,
    })
}
