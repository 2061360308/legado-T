#[cfg(any(
    feature = "layout",
    feature = "txt",
    feature = "search",
    feature = "content",
    feature = "umd",
    feature = "pipeline",
    feature = "shelf"
))]
use wasm_bindgen::prelude::*;

#[cfg(feature = "content")]
pub mod content;
#[cfg(any(feature = "layout", feature = "content", feature = "font"))]
pub mod domain;
#[cfg(feature = "ebook")]
pub mod ebook;
#[cfg(any(feature = "ebook", feature = "epub"))]
pub mod ebook_types;
#[cfg(feature = "epub")]
pub mod epub;
#[cfg(feature = "font")]
pub mod font;
#[cfg(feature = "pipeline")]
pub mod pipeline;
#[cfg(feature = "content")]
pub mod resegment;
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "shelf")]
pub mod shelf;
#[cfg(feature = "txt")]
pub mod txt;
#[cfg(feature = "txt")]
pub mod txt_kotlin;
#[cfg(feature = "umd")]
pub mod umd;

#[cfg(feature = "content")]
use content::{ProcessRequest, ProcessedContent};
#[cfg(feature = "layout")]
use domain::layout_engine::{layout, layout_blocks, LayoutStyle};
#[cfg(feature = "layout")]
use domain::model::ContentBlock;
#[cfg(feature = "pipeline")]
use pipeline::ChapterRequest;
#[cfg(feature = "search")]
use search::SearchChapter;
#[cfg(feature = "txt")]
use txt::TxtTocRule;

#[cfg(feature = "layout")]
#[wasm_bindgen]
pub fn layout_chapter_json(
    title: &str,
    paragraphs_json: &str,
    style_json: &str,
) -> Result<String, JsValue> {
    let paragraphs: Vec<String> =
        serde_json::from_str(paragraphs_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let style: LayoutStyle =
        serde_json::from_str(style_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = layout(title, &paragraphs, &style);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "layout")]
#[wasm_bindgen]
pub fn layout_blocks_json(
    title: &str,
    blocks_json: &str,
    style_json: &str,
) -> Result<String, JsValue> {
    let blocks: Vec<ContentBlock> =
        serde_json::from_str(blocks_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let style: LayoutStyle =
        serde_json::from_str(style_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = layout_blocks(title, &blocks, &style);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "txt")]
#[wasm_bindgen]
pub fn split_txt_json(text: &str, rules_json: &str, rule_expr: &str) -> Result<String, JsValue> {
    let rules: Vec<TxtTocRule> =
        serde_json::from_str(rules_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = txt::split_txt(text, &rules, Some(rule_expr));
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "search")]
#[wasm_bindgen]
pub fn search_text_json(
    chapters_json: &str,
    keyword: &str,
    limit: usize,
) -> Result<String, JsValue> {
    let chapters: Vec<SearchChapter> =
        serde_json::from_str(chapters_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = search::search_text(&chapters, keyword, limit);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
#[cfg(feature = "txt")]
#[wasm_bindgen]
pub fn analyze_txt_json(
    bytes: &[u8],
    charset: &str,
    rules_json: &str,
    split_long: bool,
) -> Result<String, JsValue> {
    let rules: Vec<TxtTocRule> =
        serde_json::from_str(rules_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = txt_kotlin::analyze(bytes, charset, &rules, split_long)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
#[cfg(feature = "txt")]
#[wasm_bindgen]
pub fn analyze_txt_with_rule_json(
    bytes: &[u8],
    charset: &str,
    rules_json: &str,
    rule_expr: &str,
    split_long: bool,
) -> Result<String, JsValue> {
    let rules: Vec<TxtTocRule> =
        serde_json::from_str(rules_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = txt_kotlin::analyze_with_rule(
        bytes,
        charset,
        &rules,
        if rule_expr.trim().is_empty() {
            None
        } else {
            Some(rule_expr)
        },
        split_long,
    )
    .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
#[cfg(feature = "content")]
#[wasm_bindgen]
pub fn process_content_json(request_json: &str) -> Result<String, JsValue> {
    let request: ProcessRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result: ProcessedContent = content::process(&request);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "umd")]
#[wasm_bindgen]
pub fn parse_umd_json(bytes: &[u8], title_fallback: &str) -> Result<String, JsValue> {
    use base64::Engine as _;
    let mut parsed = umd::parse(bytes).map_err(|e| JsValue::from_str(&e))?;
    if parsed.title.trim().is_empty() {
        parsed.title = title_fallback.to_string();
    }
    let cover_base64 = parsed
        .cover
        .as_ref()
        .map(|cover| base64::engine::general_purpose::STANDARD.encode(&cover.bytes));
    let cover_ext = parsed.cover.as_ref().map(|cover| cover.ext.clone());
    let value = serde_json::json!({
        "title": parsed.title,
        "author": parsed.author,
        "kind": parsed.kind,
        "coverBase64": cover_base64,
        "coverExt": cover_ext,
        "chapters": parsed.chapters,
    });
    serde_json::to_string(&value).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "pipeline")]
#[wasm_bindgen]
pub fn process_chapter_json(request_json: &str) -> Result<String, JsValue> {
    let request: ChapterRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = pipeline::process_chapter(&request);
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "shelf")]
#[wasm_bindgen]
pub fn sort_books_json(books_json: &str, sort: i32) -> Result<String, JsValue> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(books_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut books = values
        .into_iter()
        .map(|value| {
            let key = serde_json::from_value::<shelf::ShelfBook>(value.clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, JsValue>>()?;
    let sort = shelf::BookSort::from_i32(sort).unwrap_or_default();
    books.sort_by(|left, right| shelf::compare_books(&left.0, &right.0, sort));
    serde_json::to_string(
        &books
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>(),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "shelf")]
#[wasm_bindgen]
pub fn matches_group_json(book_json: &str, group_id: i32) -> Result<bool, JsValue> {
    let book: shelf::ShelfBook =
        serde_json::from_str(book_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(shelf::matches_group(&book, group_id as i64))
}

#[cfg(feature = "shelf")]
#[wasm_bindgen]
pub fn group_books_json(books_json: &str, group_id: i32, sort: i32) -> Result<String, JsValue> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(books_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut books = values
        .into_iter()
        .filter_map(|value| {
            let key = serde_json::from_value::<shelf::ShelfBook>(value.clone()).ok()?;
            shelf::matches_group(&key, group_id as i64).then_some((key, value))
        })
        .collect::<Vec<_>>();
    let sort = shelf::BookSort::from_i32(sort).unwrap_or_default();
    books.sort_by(|left, right| shelf::compare_books(&left.0, &right.0, sort));
    serde_json::to_string(
        &books
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>(),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "shelf")]
#[wasm_bindgen]
pub fn pinyin_compare(left: &str, right: &str) -> i32 {
    match shelf::compare_pinyin(left, right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}
