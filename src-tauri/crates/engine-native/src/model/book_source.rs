use serde::{Deserialize, Serialize};

use crate::model::de;
use crate::model::rule::{
    BookInfoRule, BookListRule, ContentRule, ExploreRule, ReviewRule, SearchRule, TocRule,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookSource {
    pub book_source_url: String,
    pub book_source_name: String,
    pub book_source_group: Option<String>,
    #[serde(deserialize_with = "de::i32_or_string")]
    pub book_source_type: i32,
    pub book_url_pattern: Option<String>,
    #[serde(deserialize_with = "de::i32_or_string")]
    pub custom_order: i32,
    #[serde(deserialize_with = "de::bool_or_string", default = "de::default_true")]
    pub enabled: bool,
    #[serde(deserialize_with = "de::bool_or_string", default = "de::default_true")]
    pub enabled_explore: bool,
    #[serde(deserialize_with = "de::opt_bool_or_string")]
    pub enabled_cookie_jar: Option<bool>,
    pub js_lib: Option<String>,
    pub concurrent_rate: Option<String>,
    #[serde(deserialize_with = "de::opt_string_or_json")]
    pub header: Option<String>,
    pub login_url: Option<String>,
    #[serde(deserialize_with = "de::opt_string_or_json")]
    pub login_ui: Option<String>,
    pub login_check_js: Option<String>,
    pub cover_decode_js: Option<String>,
    pub book_source_comment: Option<String>,
    pub variable_comment: Option<String>,
    #[serde(deserialize_with = "de::opt_i64_or_string")]
    pub last_update_time: Option<i64>,
    #[serde(deserialize_with = "de::opt_i64_or_string")]
    pub respond_time: Option<i64>,
    #[serde(deserialize_with = "de::i32_or_string")]
    pub weight: i32,
    pub explore_url: Option<String>,
    pub explore_screen: Option<String>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_explore: Option<ExploreRule>,
    pub search_url: Option<String>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_search: Option<SearchRule>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_book_info: Option<BookInfoRule>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_toc: Option<TocRule>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_content: Option<ContentRule>,
    #[serde(deserialize_with = "de::rule_option")]
    pub rule_review: Option<ReviewRule>,
}

impl BookSource {
    pub fn key(&self) -> &str {
        self.book_source_url.trim()
    }

    pub fn tag(&self) -> &str {
        self.book_source_name.trim()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn rule_strings(&self) -> Vec<&str> {
        let mut rules = Vec::new();
        if let Some(value) = self.search_url.as_deref() {
            rules.push(value);
        }
        if let Some(value) = self.explore_url.as_deref() {
            rules.push(value);
        }
        if let Some(rule) = self.rule_search.as_ref() {
            rules.extend(rule.all_rules());
        }
        if let Some(rule) = self.rule_explore.as_ref() {
            rules.extend(rule.all_rules());
        }
        if let Some(rule) = self.rule_book_info.as_ref() {
            rules.extend(rule.all_rules());
        }
        if let Some(rule) = self.rule_toc.as_ref() {
            rules.extend(rule.all_rules());
        }
        if let Some(rule) = self.rule_content.as_ref() {
            rules.extend(rule.all_rules());
        }
        if let Some(rule) = self.rule_review.as_ref() {
            rules.extend(rule.all_rules());
        }
        rules
    }

    pub fn book_list_rule(&self) -> Option<&BookListRule> {
        self.rule_search.as_ref().map(|rule| &rule.list)
    }

    /// Rule strings used for selector mode detection.
    ///
    /// URL-ish fields (`bookUrl`, `coverUrl`, `tocUrl`, `nextTocUrl`, ...)
    /// are excluded so a relative URL such as `/book/1` is not mistaken for
    /// an XPath rule.
    pub fn selector_rule_strings(&self) -> Vec<&str> {
        let mut rules = Vec::new();
        if let Some(rule) = self.rule_search.as_ref() {
            if let Some(value) = rule.check_key_word.as_deref() {
                rules.push(value);
            }
            let list = &rule.list;
            rules.extend(
                [
                    &list.book_list,
                    &list.name,
                    &list.author,
                    &list.intro,
                    &list.kind,
                    &list.last_chapter,
                    &list.update_time,
                    &list.word_count,
                ]
                .into_iter()
                .filter_map(Option::as_deref),
            );
        }
        if let Some(rule) = self.rule_explore.as_ref() {
            rules.extend(
                [
                    &rule.book_list,
                    &rule.name,
                    &rule.author,
                    &rule.intro,
                    &rule.kind,
                    &rule.last_chapter,
                    &rule.update_time,
                    &rule.word_count,
                ]
                .into_iter()
                .filter_map(Option::as_deref),
            );
        }
        if let Some(rule) = self.rule_book_info.as_ref() {
            rules.extend(
                [
                    &rule.init,
                    &rule.name,
                    &rule.author,
                    &rule.intro,
                    &rule.kind,
                    &rule.last_chapter,
                    &rule.update_time,
                    &rule.word_count,
                    &rule.can_re_name,
                ]
                .into_iter()
                .filter_map(Option::as_deref),
            );
        }
        if let Some(rule) = self.rule_toc.as_ref() {
            rules.extend(
                [
                    &rule.pre_update_js,
                    &rule.chapter_list,
                    &rule.chapter_name,
                    &rule.chapter_url,
                    &rule.format_js,
                    &rule.is_volume,
                    &rule.is_vip,
                    &rule.is_pay,
                    &rule.update_time,
                ]
                .into_iter()
                .filter_map(Option::as_deref),
            );
        }
        if let Some(rule) = self.rule_content.as_ref() {
            rules.extend(
                [
                    &rule.content,
                    &rule.title,
                    &rule.web_js,
                    &rule.source_regex,
                    &rule.replace_regex,
                    &rule.image_style,
                    &rule.image_decode,
                    &rule.pay_action,
                ]
                .into_iter()
                .filter_map(Option::as_deref),
            );
        }
        if let Some(rule) = self.rule_review.as_ref() {
            rules.extend(rule.all_rules());
        }
        rules
    }
}
