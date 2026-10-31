use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentBlock {
    Paragraph {
        text: String,
    },
    Image {
        src: String,
        width: u32,
        height: u32,
        #[serde(default)]
        alt: Option<String>,
    },
    ImagePage {
        src: String,
        page: u32,
        #[serde(default)]
        width: u32,
        #[serde(default)]
        height: u32,
    },
}

impl ContentBlock {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::Paragraph { text: text.into() }
    }
}
