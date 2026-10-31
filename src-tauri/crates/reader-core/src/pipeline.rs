use serde::{Deserialize, Serialize};

use crate::content::{self, ProcessRequest, ProcessedContent, ReplaceRule};
use crate::domain::layout_engine::{layout_blocks, LayoutPage, LayoutStyle};
use crate::domain::model::ContentBlock;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterRequest {
    pub process: ProcessRequest,
    #[serde(default)]
    pub blocks: Option<Vec<ContentBlock>>,
    pub style: LayoutStyle,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterOutput {
    pub display_title: String,
    pub blocks: Vec<ContentBlock>,
    pub pages: Vec<LayoutPage>,
    pub paragraphs: Vec<String>,
    pub effective_rules: Vec<ReplaceRule>,
    pub margin_fell_back: bool,
}

fn paragraph_texts(blocks: &[ContentBlock]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Paragraph { text } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

pub fn process_chapter(request: &ChapterRequest) -> ChapterOutput {
    let (processed, blocks) = if let Some(blocks) = request.blocks.clone() {
        let display_title = content::display_title(
            &request.process.title,
            &request.process.rules,
            &request.process.book_name,
            &request.process.book_origin,
            request.process.use_replace && request.process.use_replace_rule,
            request.process.chinese_convert,
            request.process.chinese_converter_type,
        );
        let processed = ProcessedContent {
            display_title,
            same_title_removed: false,
            contents: paragraph_texts(&blocks),
            effective_rules: Vec::new(),
        };
        (processed, blocks)
    } else {
        let processed = content::process(&request.process);
        let blocks = processed
            .contents
            .iter()
            .cloned()
            .map(ContentBlock::paragraph)
            .collect::<Vec<_>>();
        (processed, blocks)
    };
    let layout = layout_blocks(&processed.display_title, &blocks, &request.style);
    ChapterOutput {
        display_title: processed.display_title,
        blocks,
        pages: layout.pages,
        paragraphs: processed.contents,
        effective_rules: processed.effective_rules,
        margin_fell_back: layout.margin_fell_back,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_pipeline_processes_and_lays_out() {
        let request = ChapterRequest {
            process: ProcessRequest {
                title: "第一章".into(),
                content: "第一章\n正文内容".into(),
                book_name: "测试".into(),
                include_title: false,
                re_segment: false,
                ..Default::default()
            },
            blocks: None,
            style: LayoutStyle::default(),
        };
        let output = process_chapter(&request);
        assert_eq!(output.display_title, "第一章");
        assert_eq!(output.paragraphs[0], "\u{3000}\u{3000}正文内容");
        assert!(!output.pages.is_empty());
    }
}
