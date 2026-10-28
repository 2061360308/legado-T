//! 领域实体定义（M1 / T3.2）。
//!
//! 字段名与语义严格对齐 legado 实体；JSON 序列化为 camelCase。

pub mod book;
pub mod rules;
pub mod theme;

pub use book::{
    book_type, Book, BookChapter, BookGroup, Bookmark, ChapterOutline, ReadConfig, ReadProgress,
    ReadRecord,
};
pub use rules::{DictRule, HttpTts, ReplaceRule, TxtTocRule};
pub use theme::{BgKind, ModeColors, ReadStyleScheme, ThemeConfig, BUILTIN_SCHEME_COUNT};
