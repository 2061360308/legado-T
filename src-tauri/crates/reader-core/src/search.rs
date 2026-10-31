use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchChapter {
    pub index: i32,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub chapter_index: i32,
    pub chapter_title: String,
    pub pos: i32,
    pub snippet: String,
    pub result_count_within_chapter: i32,
    pub result_text: String,
    pub query: String,
    pub query_index_in_result: i32,
    pub query_index_in_chapter: i32,
}

fn snippet(content: &str, position: usize, query_len: usize) -> (String, i32) {
    let chars: Vec<char> = content.chars().collect();
    let start = position.saturating_sub(20);
    let end = (position + query_len + 20).min(chars.len());
    let text = chars[start..end].iter().collect::<String>();
    (text, (position - start) as i32)
}

pub fn search_text(chapters: &[SearchChapter], keyword: &str, limit: usize) -> Vec<SearchResult> {
    if keyword.is_empty() || limit == 0 {
        return Vec::new();
    }
    let query_len = keyword.chars().count();
    let mut results = Vec::new();
    for chapter in chapters {
        if results.len() >= limit {
            break;
        }
        let mut from = 0usize;
        let mut count = 0i32;
        while results.len() < limit {
            let Some(found) = chapter.content[from..].find(keyword) else {
                break;
            };
            let byte_pos = from + found;
            let char_pos = chapter.content[..byte_pos].chars().count();
            let (result_text, query_index_in_result) =
                snippet(&chapter.content, char_pos, query_len);
            results.push(SearchResult {
                chapter_index: chapter.index,
                chapter_title: chapter.title.clone(),
                pos: char_pos as i32,
                snippet: result_text.replace(['\r', '\n'], " "),
                result_count_within_chapter: count,
                result_text,
                query: keyword.to_string(),
                query_index_in_result,
                query_index_in_chapter: char_pos as i32,
            });
            count += 1;
            from = byte_pos + keyword.len();
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_case_sensitive_and_uses_twenty_char_context() {
        let chapters = vec![SearchChapter {
            index: 0,
            title: "第一章".into(),
            content: "abc ABC abc".into(),
        }];
        let results = search_text(&chapters, "abc", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].result_count_within_chapter, 0);
        assert_eq!(results[1].result_count_within_chapter, 1);
        assert_eq!(results[0].result_text, "abc ABC abc");
        assert_eq!(results[1].query_index_in_result, 8);
    }
}
