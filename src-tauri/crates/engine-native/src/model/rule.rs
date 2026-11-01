use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookListRule {
    pub book_list: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub intro: Option<String>,
    pub kind: Option<String>,
    pub last_chapter: Option<String>,
    pub update_time: Option<String>,
    pub book_url: Option<String>,
    pub cover_url: Option<String>,
    pub word_count: Option<String>,
}

impl BookListRule {
    pub fn all_rules(&self) -> Vec<&str> {
        [
            &self.book_list,
            &self.name,
            &self.author,
            &self.intro,
            &self.kind,
            &self.last_chapter,
            &self.update_time,
            &self.book_url,
            &self.cover_url,
            &self.word_count,
        ]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect()
    }
}

pub type ExploreRule = BookListRule;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchRule {
    pub check_key_word: Option<String>,
    #[serde(flatten)]
    pub list: BookListRule,
}

impl SearchRule {
    pub fn all_rules(&self) -> Vec<&str> {
        let mut rules = self.list.all_rules();
        if let Some(value) = self.check_key_word.as_deref() {
            rules.push(value);
        }
        rules
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookInfoRule {
    pub init: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub intro: Option<String>,
    pub kind: Option<String>,
    pub last_chapter: Option<String>,
    pub update_time: Option<String>,
    pub cover_url: Option<String>,
    pub toc_url: Option<String>,
    pub word_count: Option<String>,
    pub can_re_name: Option<String>,
    pub download_urls: Option<String>,
}

impl BookInfoRule {
    pub fn all_rules(&self) -> Vec<&str> {
        [
            &self.init,
            &self.name,
            &self.author,
            &self.intro,
            &self.kind,
            &self.last_chapter,
            &self.update_time,
            &self.cover_url,
            &self.toc_url,
            &self.word_count,
            &self.can_re_name,
            &self.download_urls,
        ]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TocRule {
    pub pre_update_js: Option<String>,
    pub chapter_list: Option<String>,
    pub chapter_name: Option<String>,
    pub chapter_url: Option<String>,
    pub format_js: Option<String>,
    pub is_volume: Option<String>,
    pub is_vip: Option<String>,
    pub is_pay: Option<String>,
    pub update_time: Option<String>,
    pub next_toc_url: Option<String>,
}

impl TocRule {
    pub fn all_rules(&self) -> Vec<&str> {
        [
            &self.pre_update_js,
            &self.chapter_list,
            &self.chapter_name,
            &self.chapter_url,
            &self.format_js,
            &self.is_volume,
            &self.is_vip,
            &self.is_pay,
            &self.update_time,
            &self.next_toc_url,
        ]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ContentRule {
    pub content: Option<String>,
    pub title: Option<String>,
    pub next_content_url: Option<String>,
    pub web_js: Option<String>,
    pub source_regex: Option<String>,
    pub replace_regex: Option<String>,
    pub image_style: Option<String>,
    pub image_decode: Option<String>,
    pub pay_action: Option<String>,
}

impl ContentRule {
    pub fn all_rules(&self) -> Vec<&str> {
        [
            &self.content,
            &self.title,
            &self.next_content_url,
            &self.web_js,
            &self.source_regex,
            &self.replace_regex,
            &self.image_style,
            &self.image_decode,
            &self.pay_action,
        ]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReviewRule {
    pub review_url: Option<String>,
    pub avatar_rule: Option<String>,
    pub content_rule: Option<String>,
    pub post_time_rule: Option<String>,
    pub review_quote_url: Option<String>,
    pub vote_up_url: Option<String>,
    pub vote_down_url: Option<String>,
    pub post_review_url: Option<String>,
    pub post_quote_url: Option<String>,
    pub delete_url: Option<String>,
}

impl ReviewRule {
    pub fn all_rules(&self) -> Vec<&str> {
        [
            &self.review_url,
            &self.avatar_rule,
            &self.content_rule,
            &self.post_time_rule,
            &self.review_quote_url,
            &self.vote_up_url,
            &self.vote_down_url,
            &self.post_review_url,
            &self.post_quote_url,
            &self.delete_url,
        ]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect()
    }
}
