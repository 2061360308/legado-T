//! 编码探测（T8.3）：BOM 判定 + 文件头统计探测（GBK / UTF-8 / UTF-8 BOM）。
//!
//! 用户手动指定字符集时直接覆盖探测结果（由调用方处理）。

/// 探测结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeCharset {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Gbk,
    Other(String),
}

impl ProbeCharset {
    pub fn label(&self) -> String {
        match self {
            ProbeCharset::Utf8 => "UTF-8".to_string(),
            ProbeCharset::Utf8Bom => "UTF-8 BOM".to_string(),
            ProbeCharset::Utf16Le => "UTF-16LE".to_string(),
            ProbeCharset::Utf16Be => "UTF-16BE".to_string(),
            ProbeCharset::Gbk => "GBK".to_string(),
            ProbeCharset::Other(name) => name.clone(),
        }
    }
}

/// 探测字节流的字符集：先看 BOM，再做 UTF-8 严格校验，失败回退 GBK。
pub fn probe(bytes: &[u8]) -> ProbeCharset {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return ProbeCharset::Utf8Bom;
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return ProbeCharset::Utf16Le;
    }
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        return ProbeCharset::Utf16Be;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return ProbeCharset::Utf8;
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    match encoding.name() {
        "GBK" | "GB18030" | "gb18030" => ProbeCharset::Gbk,
        other => ProbeCharset::Other(other.to_string()),
    }
}

/// 探测文件头（默认 64KB）字符集。
pub fn probe_file_head(bytes_of_file: &[u8]) -> ProbeCharset {
    let head = &bytes_of_file[..bytes_of_file.len().min(64 * 1024)];
    probe(head)
}
