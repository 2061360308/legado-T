use base64::Engine;
use reader_core::ebook_types::{blocks_text, blocks_to_json};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_epub_json(bytes: &[u8], title_fallback: &str) -> Result<String, JsValue> {
    let book =
        reader_core::epub::parse(bytes, title_fallback).map_err(|error| JsValue::from_str(&error))?;
    let cover_base64 = book
        .cover
        .as_ref()
        .map(|cover| base64::engine::general_purpose::STANDARD.encode(&cover.bytes));
    let cover_ext = book.cover.as_ref().map(|cover| cover.ext.clone());
    let sections = book
        .chapters
        .into_iter()
        .enumerate()
        .map(|(index, chapter)| {
            let text = blocks_text(&chapter.blocks);
            let blocks = blocks_to_json(&chapter.blocks);
            serde_json::json!({
                "index": index,
                "title": chapter.title,
                "text": text,
                "blocks": blocks,
            })
        })
        .collect::<Vec<_>>();
    let value = serde_json::json!({
        "title": book.title,
        "author": book.author,
        "intro": book.intro,
        "cover_base64": cover_base64,
        "cover_ext": cover_ext,
        "sections": sections,
    });
    serde_json::to_string(&value).map_err(|error| JsValue::from_str(&error.to_string()))
}
