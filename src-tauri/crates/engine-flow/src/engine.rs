use std::sync::Arc;

use engine_model::{
    Book, BookChapter, BookReview, BookSource, Error, JsEvaluator, JsRequest, ReplaceRule, Result,
    SearchBook, Source,
};
use engine_rule::{Content, RuleEngine};

use crate::eval::{Eval, content_for, fetch_image_bytes, fetch_page, fetch_page_with_source_regex};
use crate::fetch::PageFetcher;
use crate::{content, info, list, toc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayActionResult {
    Url(String),
    Success,
    Noop,
}

pub struct BookEngine {
    rules: Arc<RuleEngine>,
    js: Arc<dyn JsEvaluator>,
    fetcher: Arc<dyn PageFetcher>,
}

impl BookEngine {
    pub fn new(
        rules: Arc<RuleEngine>,
        js: Arc<dyn JsEvaluator>,
        fetcher: Arc<dyn PageFetcher>,
    ) -> Self {
        Self { rules, js, fetcher }
    }

    #[allow(clippy::too_many_arguments)]
    fn eval<'a>(
        &'a self,
        source: &'a BookSource,
        base_url: impl Into<String>,
        redirect_url: impl Into<String>,
        page: Option<i32>,
        key: Option<String>,
    ) -> Eval<'a> {
        Eval::new(
            &self.rules,
            &*self.js,
            source,
            &*self.fetcher,
            base_url,
            redirect_url,
            page,
            key,
        )
    }

    pub async fn fetch_page(
        &self,
        source: &BookSource,
        rule: &str,
        base_url: &str,
        key: Option<&str>,
        page: Option<i32>,
    ) -> Result<engine_model::HttpResponse> {
        fetch_page(&*self.js, source, &*self.fetcher, rule, base_url, key, page).await
    }

    pub async fn fetch_page_with_source_regex(
        &self,
        source: &BookSource,
        rule: &str,
        base_url: &str,
        key: Option<&str>,
        page: Option<i32>,
        source_regex: Option<&str>,
    ) -> Result<engine_model::HttpResponse> {
        fetch_page_with_source_regex(
            &*self.js,
            source,
            &*self.fetcher,
            rule,
            base_url,
            key,
            page,
            source_regex,
        )
        .await
    }

    pub async fn search(
        &self,
        source: &BookSource,
        key: &str,
        page: i32,
    ) -> Result<Vec<SearchBook>> {
        let rule = source
            .search_url
            .as_deref()
            .filter(|rule| !rule.trim().is_empty())
            .ok_or_else(|| Error::Other(format!("搜索 url 为空: {}", source.tag())))?;
        let response = self
            .fetch_page(source, rule, source.key(), Some(key), Some(page))
            .await?;
        let eval = self.eval(
            source,
            response.url.clone(),
            response.url.clone(),
            Some(page),
            Some(key.to_string()),
        );
        list::analyze_book_list(&eval, true, &response.text).await
    }

    pub async fn explore(
        &self,
        source: &BookSource,
        url: &str,
        page: i32,
    ) -> Result<Vec<SearchBook>> {
        let response = self
            .fetch_page(source, url, source.key(), None, Some(page))
            .await?;
        let eval = self.eval(
            source,
            response.url.clone(),
            response.url.clone(),
            Some(page),
            None,
        );
        list::analyze_book_list(&eval, false, &response.text).await
    }

    pub async fn precise_search(
        &self,
        sources: &[BookSource],
        name: &str,
        author: &str,
    ) -> Result<(Book, usize)> {
        for (index, source) in sources.iter().enumerate() {
            let matched = match self.search(source, name, 1).await {
                Ok(results) => results
                    .into_iter()
                    .find(|book| book.name == name && book.author == author),
                Err(_) => None,
            };
            let Some(matched) = matched else {
                continue;
            };
            let mut book = crate::search_book_to_book(&matched);
            if book.toc_url.trim().is_empty() && self.book_info(source, &mut book).await.is_err() {
                continue;
            }
            return Ok((book, index));
        }
        Err(Error::Other(format!("未搜索到<{name}>{author}")))
    }

    pub async fn book_info(&self, source: &BookSource, book: &mut Book) -> Result<()> {
        let response = self
            .fetch_page(source, &book.book_url, source.key(), None, None)
            .await?;
        let eval = self.eval(
            source,
            book.book_url.clone(),
            response.url.clone(),
            None,
            None,
        );
        info::analyze_book_info(&eval, book, true, &response.text).await
    }

    pub async fn download_urls(&self, source: &BookSource, book: &mut Book) -> Result<Vec<String>> {
        let Some(rule) = source
            .rule_book_info
            .as_ref()
            .and_then(|rule| rule.download_urls.as_deref())
            .filter(|rule| !rule.trim().is_empty())
        else {
            return Ok(Vec::new());
        };
        let response = self
            .fetch_page(source, &book.book_url, source.key(), None, None)
            .await?;
        let eval = self.eval(
            source,
            book.book_url.clone(),
            response.url.clone(),
            None,
            None,
        );
        eval.list(
            Some(rule),
            &Content::html(&response.text),
            true,
            Some(book),
            None,
        )
        .await
    }

    pub async fn chapters(&self, source: &BookSource, book: &mut Book) -> Result<Vec<BookChapter>> {
        if let Some(script) = source
            .rule_toc
            .as_ref()
            .and_then(|rule| rule.pre_update_js.as_deref())
            .filter(|script| !script.trim().is_empty())
        {
            self.js
                .eval(JsRequest {
                    script,
                    source: Some(source),
                    base_url: &book.book_url,
                    page: None,
                    key: None,
                    result: None,
                    result_bytes: None,
                    src: None,
                    response: None,
                    book: Some(book),
                    chapter: None,
                    variables: None,
                })
                .await?;
        }
        let response = self
            .fetch_page(source, &book.toc_url, &book.book_url, None, None)
            .await?;
        let eval = self.eval(
            source,
            book.toc_url.clone(),
            response.url.clone(),
            None,
            None,
        );
        toc::analyze_chapters(&eval, book, &response.text).await
    }

    pub async fn reviews(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
    ) -> Result<Vec<BookReview>> {
        let Some(rule) = source.rule_review.as_ref() else {
            return Ok(Vec::new());
        };
        let Some(review_url_rule) = rule
            .review_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        else {
            return Ok(Vec::new());
        };
        let eval = self.eval(
            source,
            chapter.url.clone(),
            chapter.base_url.clone(),
            None,
            None,
        );
        let response = eval
            .fetch_page(review_url_rule, &chapter.url, None, None)
            .await?;
        let review_url = response.url.clone();
        let content = content_for(&response.text);
        let eval = self.eval(source, review_url, response.url.clone(), None, None);
        let contents = eval
            .list(
                rule.content_rule.as_deref(),
                &content,
                false,
                Some(book),
                Some(chapter),
            )
            .await?;
        let avatars = eval
            .list(
                rule.avatar_rule.as_deref(),
                &content,
                false,
                Some(book),
                Some(chapter),
            )
            .await
            .unwrap_or_default();
        let times = eval
            .list(
                rule.post_time_rule.as_deref(),
                &content,
                false,
                Some(book),
                Some(chapter),
            )
            .await
            .unwrap_or_default();
        Ok(contents
            .into_iter()
            .enumerate()
            .map(|(index, content)| BookReview {
                avatar: avatars.get(index).cloned(),
                content,
                post_time: times.get(index).cloned(),
                ..Default::default()
            })
            .collect())
    }

    pub async fn pay_action(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
    ) -> Result<PayActionResult> {
        let Some(rule) = source
            .rule_content
            .as_ref()
            .and_then(|rule| rule.pay_action.as_deref())
            .filter(|value| !value.trim().is_empty())
        else {
            return Ok(PayActionResult::Noop);
        };
        let action = self
            .js
            .eval(JsRequest {
                script: rule,
                source: Some(source),
                base_url: &chapter.url,
                page: None,
                key: None,
                result: None,
                result_bytes: None,
                src: None,
                response: None,
                book: Some(book),
                chapter: Some(chapter),
                variables: None,
            })
            .await?;
        Ok(classify_pay_action(&action))
    }

    pub async fn fetch_image(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        url: &str,
        is_cover: bool,
    ) -> Result<Vec<u8>> {
        let bytes =
            fetch_image_bytes(&*self.js, source, &*self.fetcher, url, &chapter.base_url).await?;
        self.decode_image(source, book, chapter, &bytes, is_cover)
            .await
    }

    pub async fn decode_image(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        bytes: &[u8],
        is_cover: bool,
    ) -> Result<Vec<u8>> {
        let rule = if is_cover {
            source.cover_decode_js()
        } else {
            source
                .rule_content
                .as_ref()
                .and_then(|rule| rule.image_decode.as_deref())
        }
        .filter(|value| !value.trim().is_empty());
        let Some(rule) = rule else {
            return Ok(bytes.to_vec());
        };
        self.js
            .eval_bytes(JsRequest {
                script: rule,
                source: Some(source),
                base_url: &chapter.url,
                page: None,
                key: None,
                result: None,
                result_bytes: Some(bytes),
                src: Some(&chapter.url),
                response: None,
                book: Some(book),
                chapter: Some(chapter),
                variables: None,
            })
            .await
    }

    pub async fn resource(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
    ) -> Result<String> {
        if source.book_source_type == 0 {
            self.content(source, book, chapter, next_chapter_url).await
        } else {
            Ok(chapter
                .resource_url
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| chapter.url.clone()))
        }
    }

    pub async fn content(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
    ) -> Result<String> {
        let has_content = source
            .rule_content
            .as_ref()
            .and_then(|rule| rule.content.as_deref())
            .is_some_and(|rule| !rule.trim().is_empty());
        if !has_content {
            return Ok(chapter.url.clone());
        }
        if chapter.is_volume && chapter.url.starts_with(&chapter.title) {
            return Ok(chapter.tag.clone().unwrap_or_default());
        }
        let source_regex = source
            .rule_content
            .as_ref()
            .and_then(|rule| rule.source_regex.as_deref())
            .filter(|value| !value.trim().is_empty());
        let response = self
            .fetch_page_with_source_regex(
                source,
                &chapter.url,
                &chapter.base_url,
                None,
                None,
                source_regex,
            )
            .await?;
        let eval = self.eval(
            source,
            chapter.url.clone(),
            response.url.clone(),
            None,
            None,
        );
        content::analyze_content(&eval, book, chapter, next_chapter_url, &response.text).await
    }

    pub async fn content_with_replace(
        &self,
        source: &BookSource,
        book: &mut Book,
        chapter: &mut BookChapter,
        next_chapter_url: Option<&str>,
        rules: &[ReplaceRule],
    ) -> Result<String> {
        let text = self
            .content(source, book, chapter, next_chapter_url)
            .await?;
        if rules.is_empty() {
            return Ok(text);
        }
        let outcome = engine_replace::apply_to_content(
            rules,
            &book.name,
            &book.origin,
            &text,
            Some(&*self.js),
        )
        .await;
        Ok(outcome.text)
    }

    pub async fn apply_title_replace(
        &self,
        book: &Book,
        chapter: &mut BookChapter,
        rules: &[ReplaceRule],
    ) -> engine_replace::ReplaceOutcome {
        let outcome = engine_replace::apply_to_title(
            rules,
            &book.name,
            &book.origin,
            &chapter.title,
            Some(&*self.js),
        )
        .await;
        chapter.title = outcome.text.clone();
        outcome
    }
}
pub fn classify_pay_action(value: &str) -> PayActionResult {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        return PayActionResult::Noop;
    }
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return PayActionResult::Url(value.to_string());
    }
    let normalized = lower
        .strip_suffix(".0")
        .filter(|text| text.chars().all(|ch| ch.is_ascii_digit()))
        .unwrap_or(&lower);
    if matches!(normalized, "false" | "no" | "not" | "0") {
        PayActionResult::Noop
    } else {
        PayActionResult::Success
    }
}
