//! 拼音比较器（T7.2）：中文书名/作者按拼音序比较。
//!
//! 规则：汉字取全拼（小写，忽略声调）逐字比较；非汉字字符按原字符比较。
//! 两个字串先比较拼音序列，相同时再按原始字串比较保证稳定性。

use std::cmp::Ordering;

/// 把字符串转换为可比较的拼音键序列。
fn pinyin_keys(s: &str) -> Vec<String> {
    use pinyin::ToPinyin;
    let mut out = Vec::new();
    for c in s.chars() {
        match c.to_pinyin() {
            Some(p) => out.push(p.plain().to_string()),
            None => out.push(c.to_string()),
        }
    }
    out
}

/// 拼音比较器。
#[derive(Debug, Default, Clone, Copy)]
pub struct PinyinComparator;

impl PinyinComparator {
    /// 比较两个字串：先拼音逐字比较，再原串比较。
    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        let ka = pinyin_keys(a);
        let kb = pinyin_keys(b);
        let ord = ka.cmp(&kb);
        if ord != Ordering::Equal {
            return ord;
        }
        a.cmp(b)
    }

    /// 稳定排序辅助：按拼音升序排序。
    pub fn sort_strings(&self, items: &mut [String]) {
        items.sort_by(|a, b| self.compare(a, b));
    }
}

/// 中文拼音比较便捷函数。
pub fn cmp_pinyin(a: &str, b: &str) -> Ordering {
    PinyinComparator.compare(a, b)
}
