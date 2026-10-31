//! MOBI / AZW / AZW3 import wrapper around the shared reader-core parser.

use std::path::Path;

use md5::{Digest, Md5};

use crate::domain::epub_import::CoverData;
use crate::domain::model::ContentBlock;
use crate::{Error, Result};

pub struct ParsedMobiChapter {
    pub title: String,
    pub blocks: Vec<ContentBlock>,
    pub is_volume: bool,
    pub toc_level: i32,
}

pub struct ParsedMobi {
    pub title: String,
    pub author: String,
    pub intro: String,
    pub kind: String,
    pub cover: Option<CoverData>,
    pub chapters: Vec<ParsedMobiChapter>,
}

fn write_image(
    bytes: &[u8],
    ext: &str,
    width: u32,
    height: u32,
    alt: Option<String>,
    image_dir: &Path,
) -> Option<ContentBlock> {
    if bytes.is_empty() || width == 0 || height == 0 {
        return None;
    }
    let digest = Md5::digest(bytes);
    let path = image_dir.join(format!("{digest:x}.{ext}"));
    if !path.exists() {
        std::fs::create_dir_all(image_dir).ok()?;
        std::fs::write(&path, bytes).ok()?;
    }
    Some(ContentBlock::Image {
        src: path.to_string_lossy().to_string(),
        width,
        height,
        alt,
    })
}

pub fn parse(path: &Path, image_dir: &Path) -> Result<ParsedMobi> {
    let bytes = std::fs::read(path)?;
    let fallback = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled");
    let parsed = reader_core::ebook::parse(&bytes, fallback).map_err(Error::Invalid)?;
    let chapters = parsed
        .chapters
        .into_iter()
        .map(|chapter| ParsedMobiChapter {
            title: chapter.title,
            blocks: chapter
                .blocks
                .into_iter()
                .filter_map(|block| match block {
                    reader_core::ebook::EbookBlock::Paragraph { text } => {
                        Some(ContentBlock::paragraph(text))
                    }
                    reader_core::ebook::EbookBlock::Image {
                        bytes,
                        ext,
                        width,
                        height,
                        alt,
                        ..
                    } => write_image(&bytes, &ext, width, height, alt, image_dir),
                })
                .collect(),
            is_volume: chapter.is_volume,
            toc_level: chapter.toc_level,
        })
        .collect();
    Ok(ParsedMobi {
        title: parsed.title,
        author: parsed.author,
        intro: parsed.intro,
        kind: parsed.kind,
        cover: parsed.cover.map(|cover| CoverData {
            bytes: cover.bytes,
            ext: cover.ext,
        }),
        chapters,
    })
}
