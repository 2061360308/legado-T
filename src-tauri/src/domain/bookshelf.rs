//! 书架查询与排序服务（T7.1 / T7.3 / T7.4）。
//!
//! - 虚拟组查询：全部（最近阅读降序）/本地/未分组（本地且分组为 0）。
//! - 六种排序：最近阅读/更新时间/书名拼音升序/手动/综合时间/作者拼音升序。
//! - 置顶：排序值取当前最小值减 1；删书级联删章由外键保证。
//! - 组级排序：分组 bookSort = -1 时回退全局排序。

use rusqlite::Connection;

use crate::domain::model::{book_type, Book};
use crate::domain::pinyin::PinyinComparator;
use crate::Result;

/// 书架六种排序（spec 5.1.1-5/6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookSort {
    /// 最近阅读（durChapterTime 降序）。
    LastRead = 0,
    /// 更新时间。
    UpdateTime = 1,
    /// 书名拼音升序。
    BookNamePinyin = 2,
    /// 手动排序（order 升序）。
    Manual = 3,
    /// 综合时间（阅读时间与更新时间取大）。
    Comprehensive = 4,
    /// 作者拼音升序。
    AuthorPinyin = 5,
}

impl Default for BookSort {
    fn default() -> Self {
        BookSort::LastRead
    }
}

impl BookSort {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(BookSort::LastRead),
            1 => Some(BookSort::UpdateTime),
            2 => Some(BookSort::BookNamePinyin),
            3 => Some(BookSort::Manual),
            4 => Some(BookSort::Comprehensive),
            5 => Some(BookSort::AuthorPinyin),
            _ => None,
        }
    }
}

/// 虚拟组（spec 6.3-5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualGroup {
    All,
    Local,
    Audio,
    NetworkUngrouped,
    LocalUngrouped,
    Error,
    /// 用户分组（位掩码值）。
    User(u64),
}

impl VirtualGroup {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            -1 => Some(VirtualGroup::All),
            -2 => Some(VirtualGroup::Local),
            -3 => Some(VirtualGroup::Audio),
            -4 => Some(VirtualGroup::NetworkUngrouped),
            -5 => Some(VirtualGroup::LocalUngrouped),
            -6 => Some(VirtualGroup::Error),
            _ if id > 0 => Some(VirtualGroup::User(id as u64)),
            _ => None,
        }
    }
}

/// 对书籍列表按指定排序键排序（内存排序，书量级足够）。
pub fn sort_books(books: &mut [Book], sort: BookSort) {
    match sort {
        BookSort::LastRead => books.sort_by(|a, b| b.dur_chapter_time.cmp(&a.dur_chapter_time)),
        BookSort::UpdateTime => books.sort_by(|a, b| b.sync_time.cmp(&a.sync_time)),
        BookSort::BookNamePinyin => {
            let cmp = PinyinComparator;
            books.sort_by(|a, b| cmp.compare(&a.name, &b.name));
        }
        BookSort::Manual => books.sort_by(|a, b| a.order.cmp(&b.order)),
        BookSort::Comprehensive => books.sort_by(|a, b| {
            let ka = a.dur_chapter_time.max(a.sync_time);
            let kb = b.dur_chapter_time.max(b.sync_time);
            kb.cmp(&ka)
        }),
        BookSort::AuthorPinyin => {
            let cmp = PinyinComparator;
            books.sort_by(|a, b| cmp.compare(&a.author, &b.author));
        }
    }
}

/// 按「全部」（最近阅读降序）取书架。
pub fn list_all(conn: &Connection) -> Result<Vec<Book>> {
    let mut books = crate::data::repositories::BookRepo::list(conn)?;
    sort_books(&mut books, BookSort::LastRead);
    Ok(books)
}

/// 虚拟组查询：返回组内书籍（已按给定排序）。
pub fn list_by_group(conn: &Connection, group: VirtualGroup, sort: BookSort) -> Result<Vec<Book>> {
    use crate::data::repositories::BookRepo;

    let mut books: Vec<Book> = match group {
        VirtualGroup::All => BookRepo::list(conn)?,
        VirtualGroup::Local => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.r#type & book_type::LOCAL != 0)
            .collect(),
        VirtualGroup::Audio => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.r#type & book_type::AUDIO != 0)
            .collect(),
        VirtualGroup::NetworkUngrouped => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.r#type & book_type::LOCAL == 0 && b.group == 0)
            .collect(),
        VirtualGroup::LocalUngrouped => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.r#type & book_type::LOCAL != 0 && b.group == 0)
            .collect(),
        VirtualGroup::Error => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.r#type & book_type::UPDATE_ERROR != 0)
            .collect(),
        VirtualGroup::User(bit) => BookRepo::list(conn)?
            .into_iter()
            .filter(|b| b.group & bit != 0)
            .collect(),
    };
    sort_books(&mut books, sort);
    Ok(books)
}

/// 组级排序：分组 bookSort == -1 时回退全局排序（spec 6.3-4）。
pub fn resolve_group_sort(group_book_sort: i32, global_sort: BookSort) -> BookSort {
    if group_book_sort == -1 {
        global_sort
    } else {
        BookSort::from_i32(group_book_sort).unwrap_or(global_sort)
    }
}

/// 置顶：order 取当前最小值减 1（spec 5.1.1-9）。
pub fn move_to_top(conn: &Connection, book_url: &str) -> Result<()> {
    use crate::data::repositories::BookRepo;

    let book = BookRepo::get(conn, book_url)?
        .ok_or_else(|| crate::Error::NotFound(book_url.to_string()))?;
    let min_order: i32 = conn
        .query_row("SELECT MIN(\"order\") FROM books", [], |r| r.get(0))
        .unwrap_or(0);
    let mut book = book;
    book.order = min_order.saturating_sub(1);
    BookRepo::upsert(conn, &book)?;
    Ok(())
}
