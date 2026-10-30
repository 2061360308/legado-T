use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::mobi_import;
use local_reader_lib::domain::model::ContentBlock;
use local_reader_lib::domain::storage::BookStorage;

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_mobi_test_{stamp}"))
}

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn parses_mobi_and_azw3_with_toc_cover_and_images() {
    for name in ["alice.mobi", "alice.azw3"] {
        let path = fixture(name);
        assert!(path.exists(), "missing fixture {}", path.display());
        let dir = temp_dir();
        let image_dir = dir.join("images");
        let parsed = mobi_import::parse(&path, &image_dir).unwrap();
        assert!(!parsed.title.trim().is_empty(), "{name}: title");
        assert!(!parsed.chapters.is_empty(), "{name}: chapters");
        assert!(
            parsed
                .chapters
                .iter()
                .any(|chapter| !chapter.blocks.is_empty()),
            "{name}: chapter content"
        );
        assert!(parsed.cover.is_some(), "{name}: cover");

        let has_image = parsed
            .chapters
            .iter()
            .flat_map(|chapter| chapter.blocks.iter())
            .any(|block| matches!(block, ContentBlock::Image { .. }));
        assert!(has_image, "{name}: image block");
        assert!(
            fs::read_dir(&image_dir)
                .map(|entries| entries.flatten().next().is_some())
                .unwrap_or(false),
            "{name}: cached image"
        );
        fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn imports_mobi_into_reader_cache() {
    let dir = temp_dir();
    let data_dir = dir.join("data");
    fs::create_dir_all(&data_dir).unwrap();
    let conn = db::open_in_memory().unwrap();
    let storage = BookStorage::new(&data_dir);
    storage.ensure_layout().unwrap();
    let path = fixture("alice.mobi");
    let result =
        import_service::import_mobi(&conn, &path.to_string_lossy(), &storage, None, 0).unwrap();
    assert!(result.book.name.contains("Alice"));
    assert!(result.chapter_count > 1);
    assert!(result.book.cover_url.as_deref().is_some());
    let mut has_image = false;
    for index in 0..result.chapter_count as i32 {
        let blocks =
            import_service::read_epub_chapter_blocks(&storage, &result.book, index).unwrap();
        has_image |= blocks
            .iter()
            .any(|block| matches!(block, ContentBlock::Image { .. }));
        if index == 0 {
            assert!(!blocks.is_empty());
            assert!(blocks
                .iter()
                .any(|block| matches!(block, ContentBlock::Paragraph { .. })));
        }
    }
    assert!(has_image);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn rejects_drm_protected_mobi() {
    let mut bytes = vec![0u8; 94 + 16];
    bytes[..4].copy_from_slice(b"DRM!");
    bytes[60..68].copy_from_slice(b"BOOKMOBI");
    bytes[76..78].copy_from_slice(&2u16.to_be_bytes());
    bytes[78..82].copy_from_slice(&94u32.to_be_bytes());
    bytes[86..90].copy_from_slice(&110u32.to_be_bytes());
    bytes[94..96].copy_from_slice(&1u16.to_be_bytes());
    bytes[102..104].copy_from_slice(&1u16.to_be_bytes());
    bytes[106..108].copy_from_slice(&2u16.to_be_bytes());
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("drm.mobi");
    fs::write(&path, bytes).unwrap();
    let error = mobi_import::parse(&path, &dir.join("images"))
        .err()
        .expect("DRM error");
    assert!(error.to_string().contains("DRM"));
    fs::remove_dir_all(&dir).ok();
}
