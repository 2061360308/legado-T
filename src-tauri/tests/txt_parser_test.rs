use local_reader_lib::domain::model::TxtTocRule;
use local_reader_lib::domain::txt_parser;
use local_reader_lib::platform::charset_codec::CharsetCodec;
use local_reader_lib::platform::charset_probe;

fn rule(id: i64, serial: i32, expr: &str, enable: bool) -> TxtTocRule {
    TxtTocRule {
        id,
        name: format!("规则{id}"),
        rule: expr.to_string(),
        example: String::new(),
        serial_number: serial,
        enable,
    }
}

/// 标准测试文本：含相邻命中间距 >1000 字符的「第x章」标题。
fn sample_text() -> String {
    let mut s = String::from("书名：测试书\n作者：测试\n\n");
    s.push_str("第一章 起点\n");
    s.push_str(&"正文内容。".repeat(300)); // >1000 字符
    s.push_str("\n第二章 转折\n");
    s.push_str(&"情节推进。".repeat(300));
    s.push_str("\n第三章 结局\n");
    s.push_str(&"尾声内容。".repeat(100));
    s
}

const SIMPLE_RULE: &str = r"^第[一二三]章.*$";

#[test]
fn probe_rule_finds_best() {
    let text = sample_text();
    let rules = vec![
        rule(1, 0, SIMPLE_RULE, true),
        rule(2, 1, r"^(永远不存在的标题).*$", true), // 零命中
    ];
    let probed = txt_parser::probe_rule(&text, &rules).unwrap().unwrap();
    assert_eq!(probed.rule, SIMPLE_RULE);
    // 跳变次数语义：3 命中 → 2 次相邻间距 >1000 的跳变。
    assert_eq!(probed.hits, 2);
}

#[test]
fn probe_rule_ignores_close_hits() {
    // 三个命中彼此紧邻（<1000 字符）→ 不计为有效命中。
    let text = "第一章\n短\n第二章\n短\n第三章\n";
    let rules = vec![rule(1, 0, SIMPLE_RULE, true)];
    let probed = txt_parser::probe_rule(text, &rules).unwrap();
    assert!(probed.is_none());
}

#[test]
fn split_by_rule_basic() {
    let text = sample_text();
    let codec = CharsetCodec::utf8();
    let (outlines, intro) = txt_parser::split_by_rule(&text, SIMPLE_RULE, &codec, false).unwrap();
    // 首个标题前有非空白内容 → 前言章 + 3 章标题 = 4 章（spec 5.2.1-9）。
    assert_eq!(outlines.len(), 4);
    assert_eq!(outlines[0].title, "前言");
    assert_eq!(outlines[1].title, "第一章 起点");
    assert_eq!(outlines[2].title, "第二章 转折");
    assert!(intro.contains("书名"));
    // 章节区间连续覆盖：前章 end = 后章 start。
    for i in 0..outlines.len() - 1 {
        assert_eq!(outlines[i].end, outlines[i + 1].start);
    }
    assert_eq!(outlines.last().unwrap().end, text.len() as i64);
}

#[test]
fn split_by_rule_volume_flag() {
    // 标题行后无正文（直接下一个标题）→ isVolume；首标题前有内容 → 前言章。
    let text = String::new()
        + &"前置内容。".repeat(1200)
        + "\n第一卷\n\n第二卷\n\n"
        + &"正文。".repeat(1200);
    let expr = r"^第[一二]卷$";
    let codec = CharsetCodec::utf8();
    let (outlines, _) = txt_parser::split_by_rule(&text, expr, &codec, false).unwrap();
    // 前言 + 第一卷 + 第二卷 = 3 章。
    assert_eq!(outlines.len(), 3);
    assert_eq!(outlines[0].title, "前言");
    assert!(outlines[1].is_volume);
    assert!(!outlines[2].is_volume);
}

#[test]
fn analyze_fallback_when_rule_miss() {
    // 无规则命中 → 兜底 10KB 分章。
    let text = "纯正文无标题。".to_string() + &"内容。".repeat(3000);
    let bytes = text.as_bytes().to_vec();
    let codec = CharsetCodec::utf8();
    let rules = vec![rule(1, 0, SIMPLE_RULE, true)];
    let (outlines, _intro, used) =
        txt_parser::analyze(&bytes, &codec, None, &rules, false).unwrap();
    assert!(used.is_empty());
    assert!(!outlines.is_empty());
    assert_eq!(outlines[0].title, "第1部分");
}

#[test]
fn split_fallback_chunks_10k() {
    let mut bytes = vec![b'x'; 25 * 1024];
    // 换行放在每 ~10KB 边界稍后位置（模拟真实段落间隔）。
    let mut i = 10500;
    while i < bytes.len() {
        bytes[i] = b'\n';
        i += 10245;
    }
    let codec = CharsetCodec::utf8();
    let outlines = txt_parser::split_fallback(&bytes, &codec);
    // 25KB ≈ 3 块（~10.5KB / ~10.2KB / ~4.9KB，尾块 >100 字节不并入）。
    assert_eq!(outlines.len(), 3);
    assert_eq!(outlines[0].start, 0);
}

#[test]
fn fallback_tail_merged_when_tiny() {
    // 尾块 <100 字节并入上章（spec 5.2.1-7）。
    let mut bytes = vec![b'x'; 10 * 1024 + 50];
    bytes[10 * 1024] = b'\n'; // 第一块边界换行。
    let codec = CharsetCodec::utf8();
    let outlines = txt_parser::split_fallback(&bytes, &codec);
    assert_eq!(outlines.len(), 1);
    assert_eq!(outlines[0].end, bytes.len() as i64);
}

#[test]
fn analyze_preface_before_first_title() {
    // 首个标题之前存在正文内容 → 前言章 + 简介填充（spec 5.2.1-9）。
    let text = "这是开篇引言。\n作者的话。\n\n第一章 开始\n".to_string()
        + &"正文。".repeat(3000)
        + "\n第二章 结束\n"
        + &"正文。".repeat(3000);
    let bytes = text.as_bytes().to_vec();
    let codec = CharsetCodec::utf8();
    let rules = vec![rule(1, 0, SIMPLE_RULE, true)];
    let (outlines, intro, used) = txt_parser::analyze(&bytes, &codec, None, &rules, false).unwrap();
    assert_eq!(used, SIMPLE_RULE);
    assert_eq!(outlines[0].title, "前言");
    assert!(intro.contains("开篇引言"));
    assert!(outlines.len() >= 3);
}

#[test]
fn gbk_offsets_roundtrip() {
    // GBK 编码文件：start/end 为 GBK 字节偏移；正文读取按字节窗口解码。
    let text = sample_text();
    let (gbk, _, _) = encoding_rs::GBK.encode(&text);
    let codec = CharsetCodec::gbk();

    let (outlines, _) = txt_parser::split_by_rule(&text, SIMPLE_RULE, &codec, false).unwrap();
    // outlines[0] 为前言章；取第一个正文章节。
    let first = &outlines[1];
    assert_eq!(first.title, "第一章 起点");
    let content =
        txt_parser::read_chapter_content(&gbk, &codec, first.start, first.end, &first.title);
    assert!(content.contains("正文内容。"));
    assert!(!content.contains("第一章 起点")); // 首行标题被去除
    assert!(content.starts_with("\u{3000}\u{3000}")); // 首行缩进
}

#[test]
fn long_chapter_split() {
    // 单章 >100KB 开启拆分 → (1)(2) 后缀。
    let mut text = String::from("第一章 长章\n");
    text.push_str(&"很长的内容。".repeat(25000)); // ~100KB+
    let codec = CharsetCodec::utf8();
    let (outlines, _) = txt_parser::split_by_rule(&text, SIMPLE_RULE, &codec, true).unwrap();
    assert!(outlines.len() >= 2);
    assert_eq!(outlines[0].title, "第一章 长章(1)");
    assert_eq!(outlines[1].title, "第一章 长章(2)");
    // 拆分后区间连续。
    assert_eq!(outlines[0].end, outlines[1].start);
}

#[test]
fn cache_key_format() {
    assert_eq!(txt_parser::cache_key("/a.txt", 123), "toc:/a.txt:123");
}

#[test]
fn charset_probe_kinds() {
    assert_eq!(
        charset_probe::probe(b"plain ascii"),
        charset_probe::ProbeCharset::Utf8
    );
    assert_eq!(
        charset_probe::probe(&[0xEF, 0xBB, 0xBF, b'a']),
        charset_probe::ProbeCharset::Utf8Bom
    );
    let (gbk, _, _) = encoding_rs::GBK.encode("中文内容测试");
    assert_eq!(charset_probe::probe(&gbk), charset_probe::ProbeCharset::Gbk);
}

#[test]
fn performance_1m_chars_under_3s() {
    // 100 万字符、200 章 → 分章 ≤3s（spec 4.1.1）。
    let mut text = String::with_capacity(4 * 1024 * 1024);
    for i in 1..=200 {
        text.push_str(&format!("第{}章 标题{}\n", i, i));
        text.push_str(&"章节正文内容。".repeat(700)); // 每章约 4200 字
    }
    let bytes = text.as_bytes().to_vec();
    let codec = CharsetCodec::utf8();
    let rules = vec![rule(1, 0, r"^第\d+章.*$", true)];
    let start = std::time::Instant::now();
    let (outlines, _, _) = txt_parser::analyze(&bytes, &codec, None, &rules, false).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(outlines.len(), 200);
    assert!(elapsed.as_secs_f64() < 3.0, "分章耗时 {:?} 超标", elapsed);
}
