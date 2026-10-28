//! 字符集编解码与偏移换算（T9.4）：CharsetCodec。
//!
//! 核心：章节 start/end 为**原文件编码的字节偏移**（spec 5.2.1-15 / 术语「章节字节偏移」）。
//! 匹配在解码后的 UTF-8 文本上进行，命中位置需换算回原编码字节偏移；
//! 正文读取按原编码字节窗口取字节再解码。

use encoding_rs::{Encoding, UTF_8};

/// 编解码器：封装原编码与 UTF-8 之间的换算。
#[derive(Debug, Clone)]
pub struct CharsetCodec {
    encoding: &'static Encoding,
}

impl CharsetCodec {
    /// 按标签（"UTF-8"/"GBK"/...）构造；未识别回退 UTF-8。
    pub fn from_label(label: &str) -> Self {
        let label = label.trim().trim_start_matches("UTF-8 BOM").trim();
        let encoding = Encoding::for_label(label.as_bytes()).unwrap_or(UTF_8);
        CharsetCodec { encoding }
    }

    pub fn utf8() -> Self {
        CharsetCodec { encoding: UTF_8 }
    }

    pub fn gbk() -> Self {
        CharsetCodec {
            encoding: encoding_rs::GBK,
        }
    }

    pub fn label(&self) -> String {
        self.encoding.name().to_string()
    }

    /// 解码原编码字节为 UTF-8 字符串；UTF-8 BOM 被剥离（spec 5.2.1-15 BOM 场景）。
    pub fn decode(&self, bytes: &[u8]) -> String {
        let mut bytes = bytes;
        if self.encoding == UTF_8 && bytes.len() >= 3 && &bytes[..3] == b"\xEF\xBB\xBF" {
            bytes = &bytes[3..];
        }
        let (cow, _, _) = self.encoding.decode(bytes);
        cow.into_owned()
    }

    /// 构建字符索引 → 原编码字节偏移 映射（O(n)，分章时一次性构建）。
    pub fn build_char_to_byte_map(&self, bytes: &[u8]) -> Vec<usize> {
        // 逐段解码：按该编码的最小/最大字符宽度推进。
        let mut map = Vec::new();
        let mut offset = 0usize;
        if self.encoding == UTF_8 {
            if bytes.len() >= 3 && &bytes[..3] == b"\xEF\xBB\xBF" {
                offset = 3;
            }
            while offset < bytes.len() {
                map.push(offset);
                let b = bytes[offset];
                let len = if b < 0x80 {
                    1
                } else if b >> 5 == 0b110 {
                    2
                } else if b >> 4 == 0b1110 {
                    3
                } else {
                    4
                };
                offset = (offset + len).min(bytes.len());
            }
        } else {
            // GBK 等双字节可变长编码：单字节 ASCII 或双字节汉字。
            while offset < bytes.len() {
                map.push(offset);
                let b = bytes[offset];
                let len = if b < 0x80 { 1 } else { 2 };
                offset = (offset + len).min(bytes.len());
            }
        }
        // 末尾哨兵：char_index == 字符总数（文本末尾）时返回 bytes.len()。
        map.push(bytes.len());
        map
    }

    /// 把解码后 UTF-8 文本中的字节偏移换算为原编码字节偏移。
    pub fn utf8_offset_to_src(&self, decoded: &str, utf8_offset: usize, map: &[usize]) -> usize {
        let char_index = decoded[..utf8_offset].chars().count();
        map.get(char_index)
            .copied()
            .unwrap_or_else(|| map.last().map(|x| *x).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gbk_roundtrip_and_offsets() {
        let text = "第一章开始正文内容测试";
        let (gbk_bytes, _, _) = encoding_rs::GBK.encode(text);
        let codec = CharsetCodec::gbk();
        let decoded = codec.decode(&gbk_bytes);
        assert_eq!(decoded, text);
        let map = codec.build_char_to_byte_map(&gbk_bytes);
        // 第 4 个字符「始」的源字节偏移：前 3 个汉字各 2 字节 = 6。
        assert_eq!(map[3], 6);
    }

    #[test]
    fn utf8_bom_stripped() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("标题".as_bytes());
        let codec = CharsetCodec::utf8();
        assert_eq!(codec.decode(&bytes), "标题");
        let map = codec.build_char_to_byte_map(&bytes);
        assert_eq!(map[0], 3);
    }

    #[test]
    fn unknown_label_falls_back_utf8() {
        let codec = CharsetCodec::from_label("不存在的编码");
        assert_eq!(codec.label(), "UTF-8");
    }
}
