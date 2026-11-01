use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Book {
    pub book_url: String,
    pub toc_url: String,
    pub origin: String,
    pub origin_name: String,
    pub name: String,
    pub author: String,
    pub kind: Option<String>,
    pub cover_url: Option<String>,
    pub intro: Option<String>,
    #[serde(rename = "type")]
    pub book_type: i32,
    pub latest_chapter_title: Option<String>,
    pub total_chapter_num: i32,
    pub word_count: Option<String>,
    pub image_style: Option<String>,
    #[serde(rename = "variable")]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookChapter {
    pub url: String,
    pub title: String,
    pub is_volume: bool,
    pub base_url: String,
    pub book_url: String,
    pub index: i32,
    pub is_vip: bool,
    pub is_pay: bool,
    pub resource_url: Option<String>,
    pub tag: Option<String>,
    #[serde(rename = "variable")]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchBook {
    pub book_url: String,
    pub origin: String,
    pub origin_name: String,
    pub name: String,
    pub author: String,
    pub kind: Option<String>,
    pub cover_url: Option<String>,
    pub intro: Option<String>,
    #[serde(rename = "type")]
    pub book_type: i32,
    pub latest_chapter_title: Option<String>,
    pub toc_url: String,
    pub word_count: Option<String>,
    pub image_style: Option<String>,
    #[serde(rename = "variable")]
    pub variables: HashMap<String, String>,
}
