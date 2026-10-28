use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use local_reader_lib::data::db;
use local_reader_lib::domain::import_service;
use local_reader_lib::domain::layout_engine::LayoutStyle;
use local_reader_lib::domain::read_state::{self, ReaderCache};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

fn temp_dir() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("local_reader_epub_test_{stamp}"))
}

fn add(zip: &mut ZipWriter<fs::File>, name: &str, content: &str, stored: bool) {
    let options = if stored {
        FileOptions::default().compression_method(CompressionMethod::Stored)
    } else {
        FileOptions::default().compression_method(CompressionMethod::Deflated)
    };
    zip.start_file(name, options).unwrap();
    zip.write_all(content.as_bytes()).unwrap();
}

fn add_bytes(zip: &mut ZipWriter<fs::File>, name: &str, content: &[u8], stored: bool) {
    let options = if stored {
        FileOptions::default().compression_method(CompressionMethod::Stored)
    } else {
        FileOptions::default().compression_method(CompressionMethod::Deflated)
    };
    zip.start_file(name, options).unwrap();
    zip.write_all(content).unwrap();
}

const ONE_PIXEL_GIF: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];

fn write_epub(path: &std::path::Path) {
    let mut zip = ZipWriter::new(fs::File::create(path).unwrap());
    add(&mut zip, "mimetype", "application/epub+zip", true);
    add(
        &mut zip,
        "META-INF/container.xml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        false,
    );
    add(
        &mut zip,
        "OEBPS/content.opf",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="book-id">urn:test:epub</dc:identifier>
    <dc:title>测试 EPUB</dc:title>
    <dc:creator>测试作者</dc:creator>
    <dc:language>zh</dc:language>
    <dc:description>一本测试书</dc:description>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="pic" href="images/pic.gif" media-type="image/gif"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
    <itemref idref="c2"/>
  </spine>
</package>"#,
        false,
    );
    add(
        &mut zip,
        "OEBPS/nav.xhtml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<body><nav epub:type="toc"><ol>
  <li><a href="ch1.xhtml">第一章 起点</a></li>
  <li><a href="ch2.xhtml">第二章 转折</a></li>
</ol></nav></body></html>"#,
        false,
    );
    add(
        &mut zip,
        "OEBPS/ch1.xhtml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>ignored</title><style>p{color:red}</style></head>
<body><h1>第一章 起点</h1><p>这是第一段正文。</p><img src="images/pic.gif" alt="插图"/><p>这是第二段正文。</p><script>ignored()</script></body></html>"#,
        false,
    );
    add_bytes(&mut zip, "OEBPS/images/pic.gif", ONE_PIXEL_GIF, false);
    add(
        &mut zip,
        "OEBPS/ch2.xhtml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><h1>第二章 转折</h1><p>第二章内容。</p></body></html>"#,
        false,
    );
    zip.finish().unwrap();
}

#[test]
fn imports_epub_text_and_reads_cached_chapters() {
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let epub = dir.join("book.epub");
    write_epub(&epub);

    let conn = db::open_in_memory().unwrap();
    let import =
        import_service::import_epub(&conn, &epub.to_string_lossy(), &dir, None, 0).unwrap();

    assert_eq!(import.book.name, "测试 EPUB");
    assert_eq!(import.book.author, "测试作者");
    assert_eq!(import.chapter_count, 2);
    assert_eq!(import.intro, "一本测试书");

    let chapters =
        local_reader_lib::data::repositories::ChapterRepo::list(&conn, &import.book.book_url)
            .unwrap();
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].title, "第一章 起点");
    assert_eq!(chapters[1].title, "第二章 转折");

    let style = LayoutStyle {
        view_width: 400.0,
        view_height: 600.0,
        ..Default::default()
    };
    let mut cache = ReaderCache::default();
    let first =
        read_state::open(&conn, &mut cache, &import.book.book_url, None, None, &style).unwrap();
    assert_eq!(first.chapter_title, "第一章 起点");
    assert!(first
        .chapter_paragraphs
        .join("")
        .contains("这是第一段正文。"));

    let blocks = import_service::read_epub_chapter_blocks(&import.book.book_url, 0).unwrap();
    let image = blocks
        .iter()
        .find_map(|block| match block {
            local_reader_lib::domain::model::ContentBlock::Image {
                src, width, height, ..
            } => Some((src, *width, *height)),
            _ => None,
        })
        .expect("EPUB image block");
    assert_eq!((image.1, image.2), (1, 1));
    assert!(std::path::Path::new(image.0).exists());
    assert!(first
        .pages
        .iter()
        .flat_map(|page| page.lines.iter())
        .any(|line| line.image.is_some()));

    let second = read_state::open(
        &conn,
        &mut cache,
        &import.book.book_url,
        Some(1),
        Some(0),
        &style,
    )
    .unwrap();
    assert_eq!(second.chapter_title, "第二章 转折");
    assert!(second.chapter_paragraphs.join("").contains("第二章内容。"));

    fs::remove_dir_all(&dir).ok();
}
