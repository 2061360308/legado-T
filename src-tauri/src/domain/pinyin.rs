//! 拼音比较器：统一使用 reader-core 实现。

pub use reader_core::shelf::compare_pinyin as cmp_pinyin;

#[derive(Debug, Default, Clone, Copy)]
pub struct PinyinComparator;

impl PinyinComparator {
    pub fn compare(&self, left: &str, right: &str) -> std::cmp::Ordering {
        reader_core::shelf::compare_pinyin(left, right)
    }

    pub fn sort_strings(&self, items: &mut [String]) {
        items.sort_by(|left, right| self.compare(left, right));
    }
}
