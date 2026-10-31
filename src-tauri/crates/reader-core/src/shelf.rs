use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

const BOOK_TYPE_AUDIO: u64 = 1 << 1;
const BOOK_TYPE_LOCAL: u64 = 1 << 4;
const BOOK_TYPE_UPDATE_ERROR: u64 = 1 << 5;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShelfBook {
    #[serde(default)]
    pub book_url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub dur_chapter_time: i64,
    #[serde(default)]
    pub sync_time: i64,
    #[serde(default)]
    pub order: i32,
    #[serde(default, rename = "type")]
    pub book_type: u64,
    #[serde(default)]
    pub group: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookSort {
    LastRead = 0,
    UpdateTime = 1,
    BookNamePinyin = 2,
    Manual = 3,
    Comprehensive = 4,
    AuthorPinyin = 5,
}

impl Default for BookSort {
    fn default() -> Self {
        Self::LastRead
    }
}

impl BookSort {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::LastRead),
            1 => Some(Self::UpdateTime),
            2 => Some(Self::BookNamePinyin),
            3 => Some(Self::Manual),
            4 => Some(Self::Comprehensive),
            5 => Some(Self::AuthorPinyin),
            _ => None,
        }
    }
}

fn pinyin_keys(value: &str) -> Vec<String> {
    use pinyin::ToPinyin;
    value
        .chars()
        .map(|ch| {
            ch.to_pinyin()
                .map(|pinyin| pinyin.plain().to_string())
                .unwrap_or_else(|| ch.to_string())
        })
        .collect()
}

pub fn compare_pinyin(left: &str, right: &str) -> Ordering {
    let left_keys = pinyin_keys(left);
    let right_keys = pinyin_keys(right);
    left_keys.cmp(&right_keys).then_with(|| left.cmp(right))
}

pub fn compare_books(left: &ShelfBook, right: &ShelfBook, sort: BookSort) -> Ordering {
    match sort {
        BookSort::LastRead => right.dur_chapter_time.cmp(&left.dur_chapter_time),
        BookSort::UpdateTime => right.sync_time.cmp(&left.sync_time),
        BookSort::BookNamePinyin => compare_pinyin(&left.name, &right.name),
        BookSort::Manual => left.order.cmp(&right.order),
        BookSort::Comprehensive => right
            .dur_chapter_time
            .max(right.sync_time)
            .cmp(&left.dur_chapter_time.max(left.sync_time)),
        BookSort::AuthorPinyin => compare_pinyin(&left.author, &right.author),
    }
}

pub fn sort_books(books: &mut [ShelfBook], sort: BookSort) {
    books.sort_by(|left, right| compare_books(left, right, sort));
}

pub fn matches_group(book: &ShelfBook, group_id: i64) -> bool {
    match group_id {
        -1 => true,
        -2 => book.book_type & BOOK_TYPE_LOCAL != 0,
        -3 => book.book_type & BOOK_TYPE_AUDIO != 0,
        -4 => book.book_type & BOOK_TYPE_LOCAL == 0 && book.group == 0,
        -5 => book.book_type & BOOK_TYPE_LOCAL != 0 && book.group == 0,
        -6 => book.book_type & BOOK_TYPE_UPDATE_ERROR != 0,
        value if value > 0 => book.group & value as u64 != 0,
        _ => false,
    }
}

pub fn resolve_group_sort(group_book_sort: i32, global_sort: BookSort) -> BookSort {
    if group_book_sort == -1 {
        global_sort
    } else {
        BookSort::from_i32(group_book_sort).unwrap_or(global_sort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(name: &str, author: &str) -> ShelfBook {
        ShelfBook {
            name: name.into(),
            author: author.into(),
            ..Default::default()
        }
    }

    #[test]
    fn pinyin_sorts_chinese_names() {
        let mut books = vec![book("张三", ""), book("李四", ""), book("王五", "")];
        sort_books(&mut books, BookSort::BookNamePinyin);
        assert_eq!(
            books.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(),
            vec!["李四", "王五", "张三"]
        );
    }

    #[test]
    fn virtual_groups_match_kotlin_bits() {
        let mut item = book("书", "");
        item.book_type = BOOK_TYPE_LOCAL;
        item.group = 2;
        assert!(matches_group(&item, -1));
        assert!(matches_group(&item, -2));
        assert!(matches_group(&item, 2));
        assert!(!matches_group(&item, -4));
    }
}
