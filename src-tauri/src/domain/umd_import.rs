//! UMD import wrapper around the shared reader-core parser.

use std::path::Path;

use crate::domain::epub_import::CoverData;
use crate::domain::model::ContentBlock;
use crate::{Error, Result};

pub struct ParsedUmdChapter {
    pub title: String,
    pub blocks: Vec<ContentBlock>,
}

pub struct ParsedUmd {
    pub title: String,
    pub author: String,
    pub kind: String,
    pub cover: Option<CoverData>,
    pub chapters: Vec<ParsedUmdChapter>,
}

fn text_blocks(text: &str) -> Vec<ContentBlock> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ContentBlock::paragraph)
        .collect()
}

pub fn parse(path: &Path) -> Result<ParsedUmd> {
    let bytes = std::fs::read(path)?;
    let parsed = reader_core::umd::parse(&bytes).map_err(Error::Invalid)?;
    Ok(ParsedUmd {
        title: parsed.title,
        author: parsed.author,
        kind: parsed.kind,
        cover: parsed.cover.map(|cover| CoverData {
            bytes: cover.bytes,
            ext: cover.ext,
        }),
        chapters: parsed
            .chapters
            .into_iter()
            .map(|chapter| ParsedUmdChapter {
                title: chapter.title,
                blocks: text_blocks(&chapter.text),
            })
            .collect(),
    })
}
