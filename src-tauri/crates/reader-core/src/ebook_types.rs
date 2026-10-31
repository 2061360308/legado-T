use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EbookBlock {
    Paragraph {
        text: String,
    },
    Image {
        href: String,
        bytes: Vec<u8>,
        ext: String,
        width: u32,
        height: u32,
        alt: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EbookChapter {
    pub title: String,
    pub href: String,
    pub blocks: Vec<EbookBlock>,
    pub is_volume: bool,
    pub toc_level: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EbookCover {
    pub bytes: Vec<u8>,
    pub ext: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EbookBook {
    pub title: String,
    pub author: String,
    pub intro: String,
    pub kind: String,
    pub cover: Option<EbookCover>,
    pub chapters: Vec<EbookChapter>,
}

pub fn blocks_text(blocks: &[EbookBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            EbookBlock::Paragraph { text } => Some(text.as_str()),
            EbookBlock::Image { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn blocks_to_json(blocks: &[EbookBlock]) -> Vec<Value> {
    blocks
        .iter()
        .map(|block| match block {
            EbookBlock::Paragraph { text } => json!({
                "type": "paragraph",
                "text": text,
            }),
            EbookBlock::Image {
                bytes,
                ext,
                width,
                height,
                alt,
                ..
            } => json!({
                "type": "image",
                "base64": base64::engine::general_purpose::STANDARD.encode(bytes),
                "ext": ext,
                "width": width,
                "height": height,
                "alt": alt,
            }),
        })
        .collect()
}
