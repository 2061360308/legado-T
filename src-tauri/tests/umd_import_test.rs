use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use local_reader_lib::data::db;
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::umd_import;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_umd_test_{stamp}"))
}

fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(|value| value.to_le_bytes()).collect()
}

fn segment(out: &mut Vec<u8>, kind: u8, flag: u8, payload: &[u8]) {
    out.extend_from_slice(&[b'#', kind, 0, flag, (payload.len() + 5) as u8]);
    out.extend_from_slice(payload);
}

fn write_umd(path: &Path) {
    const COVER: &[u8] = &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
        0x3B,
    ];
    let content = utf16("第一章 开始\u{2029}这是正文。");
    let title = utf16("第一章 开始");
    let mut out = Vec::new();
    out.extend_from_slice(&[0x89, 0x9b, 0x9a, 0xde]);
    segment(&mut out, 1, 0, &[1, 0, 0]);
    segment(&mut out, 2, 0, &utf16("测试 UMD"));
    segment(&mut out, 3, 0, &utf16("测试作者"));
    segment(&mut out, 7, 0, &utf16("小说"));
    segment(&mut out, 130, 0, &[0, 0, 0, 0, 0]);
    out.push(b'$');
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&((COVER.len() + 9) as u32).to_le_bytes());
    out.extend_from_slice(COVER);
    segment(&mut out, 11, 0, &(content.len() as u32).to_le_bytes());

    let offsets = 0u32.to_le_bytes();
    segment(&mut out, 131, 0, &[0, 0, 0, 0]);
    out.extend_from_slice(&[b'$', 0, 0, 0, 0]);
    out.extend_from_slice(&((offsets.len() + 9) as u32).to_le_bytes());
    out.extend_from_slice(&offsets);

    let mut title_payload = Vec::new();
    title_payload.push(title.len() as u8);
    title_payload.extend_from_slice(&title);
    segment(&mut out, 132, 1, &[0, 0, 0, 0]);
    out.extend_from_slice(&[b'$', 0, 0, 0, 0]);
    out.extend_from_slice(&((title_payload.len() + 9) as u32).to_le_bytes());
    out.extend_from_slice(&title_payload);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&content).unwrap();
    let compressed = encoder.finish().unwrap();
    out.push(b'$');
    out.extend_from_slice(&[1, 2, 3, 4]);
    out.extend_from_slice(&((compressed.len() + 9) as u32).to_le_bytes());
    out.extend_from_slice(&compressed);
    out.push(0);

    fs::write(path, out).unwrap();
}

#[test]
fn parses_umd_metadata_chapters_and_newlines() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("book.umd");
    write_umd(&path);
    let parsed = umd_import::parse(&path).unwrap();
    assert_eq!(parsed.title, "测试 UMD");
    assert_eq!(parsed.author, "测试作者");
    assert_eq!(parsed.kind, "小说");
    assert!(parsed.cover.is_some());
    assert_eq!(parsed.chapters.len(), 1);
    assert_eq!(parsed.chapters[0].title, "第一章 开始");
    let text = parsed.chapters[0]
        .blocks
        .iter()
        .filter_map(|block| match block {
            local_reader_lib::domain::model::ContentBlock::Paragraph { text } => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("这是正文。"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn imports_umd_into_cached_chapters_and_cover() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("book.umd");
    write_umd(&path);
    let conn = db::open_in_memory().unwrap();
    let result =
        import_service::import_umd(&conn, &path.to_string_lossy(), &dir.join("data"), None, 0)
            .unwrap();
    assert_eq!(result.book.name, "测试 UMD");
    assert_eq!(result.chapter_count, 1);
    assert!(result.book.cover_url.as_deref().unwrap().ends_with(".gif"));
    let blocks = import_service::read_epub_chapter_blocks(&result.book.book_url, 0).unwrap();
    assert!(!blocks.is_empty());
    fs::remove_dir_all(&dir).ok();
}
