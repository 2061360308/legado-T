//! 书籍相关领域实体（字段名与语义对齐 legado，JSON 为 camelCase）。

use serde::{Deserialize, Serialize};

/// 书籍类型位标志。
pub mod book_type {
    pub const NONE: u64 = 0;
    pub const TEXT: u64 = 1 << 0;
    pub const AUDIO: u64 = 1 << 1;
    pub const IMAGE: u64 = 1 << 2;
    pub const WEB_FILE: u64 = 1 << 3;
    /// 本地书标志。
    pub const LOCAL: u64 = 1 << 4;
    pub const UPDATE_ERROR: u64 = 1 << 5;
}

/// 阅读配置（内嵌于书籍）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadConfig {
    #[serde(default)]
    pub reverse_toc: bool,
    #[serde(default)]
    pub re_segment: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_anim: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_replace_rule: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts_engine: Option<String>,
    #[serde(default)]
    pub split_long_chapter: bool,
    #[serde(default)]
    pub image_style: String,
    #[serde(default)]
    pub read_aloud_simulate: bool,
    #[serde(default)]
    pub daily_chapter: i32,
}

/// 阅读进度（由书籍当前进度派生）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadProgress {
    pub name: String,
    pub author: String,
    pub chapter_index: i32,
    pub chapter_pos: i32,
    pub chapter_title: String,
    pub time: i64,
}

/// 书籍实体。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    /// 访问地址（本地书为文件绝对路径），唯一。
    pub book_url: String,
    pub name: String,
    pub author: String,
    pub origin: String,
    pub origin_name: String,
    #[serde(default)]
    pub charset: Option<String>,
    /// 类型位标志。
    #[serde(default)]
    pub r#type: u64,
    /// 分组位掩码。
    #[serde(default)]
    pub group: u64,
    /// 本地书当前选中的目录规则表达式。
    #[serde(default)]
    pub toc_url: String,
    #[serde(default)]
    pub intro: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub custom_tag: Option<String>,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub custom_cover_url: Option<String>,
    #[serde(default)]
    pub latest_chapter_title: Option<String>,
    #[serde(default)]
    pub dur_chapter_index: i32,
    #[serde(default)]
    pub dur_chapter_pos: i32,
    #[serde(default)]
    pub dur_chapter_title: Option<String>,
    #[serde(default)]
    pub dur_chapter_time: i64,
    #[serde(default)]
    pub total_chapter_num: i32,
    #[serde(default)]
    pub can_update: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub sync_time: i64,
    #[serde(default)]
    pub read_config: Option<ReadConfig>,
    #[serde(default)]
    pub custom_intro: Option<String>,
}

impl Book {
    /// 是否为本地书。
    pub fn is_local(&self) -> bool {
        self.r#type & book_type::LOCAL != 0
    }

    /// 派生阅读进度。
    pub fn progress(&self) -> ReadProgress {
        ReadProgress {
            name: self.name.clone(),
            author: self.author.clone(),
            chapter_index: self.dur_chapter_index,
            chapter_pos: self.dur_chapter_pos,
            chapter_title: self.dur_chapter_title.clone().unwrap_or_default(),
            time: self.dur_chapter_time,
        }
    }
}

/// 章节实体。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookChapter {
    /// 章节 url；本地书为文件路径基址。
    pub url: String,
    pub title: String,
    pub index: i32,
    #[serde(default)]
    pub is_volume: bool,
    /// 正文起始字节偏移（按书籍 charset 换算）。
    #[serde(default)]
    pub start: i64,
    /// 正文结束字节偏移。
    #[serde(default)]
    pub end: i64,
    #[serde(default)]
    pub is_vip: bool,
    #[serde(default)]
    pub is_pay: bool,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub resource_url: Option<String>,
    #[serde(default)]
    pub toc_level: i32,
}

impl BookChapter {
    pub fn display_title(&self) -> &str {
        &self.title
    }
}

/// 目录大纲项（分章产出）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChapterOutline {
    pub title: String,
    pub start: i64,
    pub end: i64,
    #[serde(default)]
    pub is_volume: bool,
}

/// 书架分组。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookGroup {
    /// 分组位掩码取值（唯一）。
    pub group_id: u64,
    pub group_name: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub book_sort: i32,
    #[serde(default)]
    pub show: bool,
    #[serde(default)]
    pub enable_refresh: bool,
    #[serde(default)]
    pub cover_url: Option<String>,
}

impl BookGroup {
    /// 排序值 -1 表示跟随全局排序。
    pub fn follows_global_sort(&self) -> bool {
        self.book_sort == -1
    }
}

/// 书签（弱关联，不级联删除）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    /// 以创建时间戳为唯一标识。
    pub time: i64,
    #[serde(default)]
    pub book_name: String,
    #[serde(default)]
    pub book_author: String,
    #[serde(default)]
    pub book_url: String,
    #[serde(default)]
    pub chapter_index: i32,
    #[serde(default)]
    pub chapter_pos: i32,
    #[serde(default)]
    pub chapter_name: String,
    /// 摘录原文。
    #[serde(default)]
    pub book_text: String,
    /// 笔记。
    #[serde(default)]
    pub content: Option<String>,
}

/// 阅读记录（按设备标识与书名复合唯一）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadRecord {
    pub device_id: String,
    pub book_name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub read_time: i64,
    #[serde(default)]
    pub last_read: i64,
    #[serde(default)]
    pub dur_chapter_index: i32,
    #[serde(default)]
    pub dur_chapter_pos: i32,
    #[serde(default)]
    pub dur_chapter_title: String,
}
