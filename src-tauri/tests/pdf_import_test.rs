use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::layout_engine::LayoutStyle;
use local_reader_lib::domain::read_state::{self, ReaderCache};

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_pdf_test_{stamp}"))
}

fn write_pdf(path: &Path, text: &str) {
    let stream = format!("BT /F1 24 Tf 20 100 Td ({text}) Tj ET").into_bytes();
    let objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_vec(),
        [
            format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes(),
            stream.clone(),
            b"\nendstream".to_vec(),
        ]
        .concat(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];

    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(object);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    fs::write(path, out).unwrap();
}

#[test]
fn imports_pdf_pages_and_reads_cached_chapters() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("book.pdf");
    write_pdf(&pdf, "Hello PDF this is a text layer page for testing");

    let conn = db::open_in_memory().unwrap();
    let import =
        import_service::import_pdf(&conn, &pdf.to_string_lossy(), &dir, None, 0).unwrap();
    assert_eq!(import.chapter_count, 1);
    assert_eq!(import.book.name, "book");

    let blocks = import_service::read_epub_chapter_blocks(&import.book.book_url, 0).unwrap();
    let image = blocks
        .iter()
        .find_map(|block| match block {
            local_reader_lib::domain::model::ContentBlock::ImagePage {
                src, width, height, ..
            } => Some((src, *width, *height)),
            _ => None,
        })
        .expect("PDF page image block");
    assert!(image.1 > 0 && image.2 > 0);
    assert!(Path::new(image.0).exists());

    let style = LayoutStyle {
        view_width: 400.0,
        view_height: 600.0,
        ..Default::default()
    };
    let mut cache = ReaderCache::default();
    let snapshot =
        read_state::open(&conn, &mut cache, &import.book.book_url, None, None, &style).unwrap();
    assert!(snapshot
        .pages
        .iter()
        .flat_map(|page| page.lines.iter())
        .any(|line| line.image.is_some()));

    fs::remove_dir_all(&dir).ok();
}
