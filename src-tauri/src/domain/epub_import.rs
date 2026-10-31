//! EPUB import wrapper around the shared reader-core parser.

use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

use crate::domain::model::ContentBlock;
use crate::{Error, Result};

pub struct ParsedChapter {
    pub title: String,
    pub href: String,
    pub blocks: Vec<ContentBlock>,
    pub is_volume: bool,
    pub toc_level: i32,
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
    let path: PathBuf = image_dir.join(format!("{digest:x}.{ext}"));
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

pub fn extract_cover(path: &Path) -> Result<Option<CoverData>> {
    let bytes = std::fs::read(path)?;
    Ok(reader_core::epub::extract_cover(&bytes)
        .map_err(Error::Invalid)?
        .map(|cover| CoverData {
            bytes: cover.bytes,
            ext: cover.ext,
        }))
}

pub fn parse(path: &Path, image_dir: &Path) -> Result<ParsedEpub> {
    let bytes = std::fs::read(path)?;
    let fallback = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled");
    let parsed = reader_core::epub::parse(&bytes, fallback).map_err(Error::Invalid)?;
    let chapters = parsed
        .chapters
        .into_iter()
        .map(|chapter| ParsedChapter {
            title: chapter.title,
            href: chapter.href,
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
    Ok(ParsedEpub {
        title: parsed.title,
        author: parsed.author,
        intro: parsed.intro,
        chapters,
    })
}
