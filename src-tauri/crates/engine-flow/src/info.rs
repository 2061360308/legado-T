use engine_model::{Book, Result};
use engine_http_native::absolute_url;

use crate::eval::{Eval, content_for, split_value};
use crate::fmt::{format_book_author, format_book_name, word_count_format};

pub(crate) async fn analyze_book_info(
    eval: &Eval<'_>,
    book: &mut Book,
    can_re_name: bool,
    body: &str,
) -> Result<()> {
    let info = eval.source.rule_book_info.as_ref();
    let mut content = content_for(body);

    if let Some(init) = info
        .and_then(|rule| rule.init.as_deref())
        .filter(|rule| !rule.trim().is_empty())
        && let Ok(value) = eval.value(init, &content, Some(&mut *book), None).await
        && let Some(first) = split_value(value).into_iter().next()
    {
        content = first;
    }

    book.book_type = eval.source.book_source_type;

    let name = format_book_name(
        &eval
            .string(
                info.and_then(|rule| rule.name.as_deref()),
                &content,
                false,
                true,
                Some(&mut *book),
                None,
            )
            .await
            .unwrap_or_default(),
    );
    if !name.is_empty() && (can_re_name || book.name.is_empty()) {
        book.name = name;
    }

    let author = format_book_author(
        &eval
            .string(
                info.and_then(|rule| rule.author.as_deref()),
                &content,
                false,
                true,
                Some(&mut *book),
                None,
            )
            .await
            .unwrap_or_default(),
    );
    if !author.is_empty() && (can_re_name || book.author.is_empty()) {
        book.author = author;
    }

    if let Ok(kinds) = eval
        .list(
            info.and_then(|rule| rule.kind.as_deref()),
            &content,
            false,
            Some(&mut *book),
            None,
        )
        .await
        && !kinds.is_empty()
    {
        book.kind = Some(kinds.join(","));
    }

    if let Ok(word_count) = eval
        .string(
            info.and_then(|rule| rule.word_count.as_deref()),
            &content,
            false,
            true,
            Some(&mut *book),
            None,
        )
        .await
    {
        let word_count = word_count_format(&word_count);
        if !word_count.is_empty() {
            book.word_count = Some(word_count);
        }
    }

    if let Ok(last_chapter) = eval
        .string(
            info.and_then(|rule| rule.last_chapter.as_deref()),
            &content,
            false,
            true,
            Some(&mut *book),
            None,
        )
        .await
        && !last_chapter.trim().is_empty()
    {
        book.latest_chapter_title = Some(last_chapter);
    }

    if let Ok(intro) = eval
        .string(
            info.and_then(|rule| rule.intro.as_deref()),
            &content,
            false,
            true,
            Some(&mut *book),
            None,
        )
        .await
        && !intro.trim().is_empty()
    {
        book.intro = Some(engine_js::html_format::format_text(&intro));
    }

    if let Ok(cover) = eval
        .string(
            info.and_then(|rule| rule.cover_url.as_deref()),
            &content,
            false,
            true,
            Some(&mut *book),
            None,
        )
        .await
        && !cover.trim().is_empty()
    {
        book.cover_url = Some(absolute_url(&eval.redirect_url, cover.trim()));
    }

    let toc_url = eval
        .string(
            info.and_then(|rule| rule.toc_url.as_deref()),
            &content,
            true,
            true,
            Some(&mut *book),
            None,
        )
        .await?;
    book.toc_url = if toc_url.is_empty() || toc_url.to_ascii_lowercase().contains("javascript:") {
        eval.redirect_url.clone()
    } else {
        toc_url
    };
    book.toc_url = fill_missing_book_id(&book.toc_url, &book.book_url);
    if book.image_style.is_none() {
        book.image_style = eval
            .source
            .rule_content
            .as_ref()
            .and_then(|rule| rule.image_style.clone());
    }

    Ok(())
}
fn fill_missing_book_id(toc_url: &str, book_url: &str) -> String {
    let marker = if toc_url.contains("bookId=&") {
        "bookId="
    } else if toc_url.contains("id=&") {
        "id="
    } else {
        return toc_url.to_string();
    };
    let Some(start) = book_url.find(marker).map(|index| index + marker.len()) else {
        return toc_url.to_string();
    };
    let digits: String = book_url[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return toc_url.to_string();
    }
    toc_url
        .replace(&format!("{marker}&"), &format!("{marker}{digits}&"))
        .replace(&format!("{marker}$"), &format!("{marker}{digits}"))
}
