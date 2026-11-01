use std::path::PathBuf;

use engine_native::Corpus;

#[test]
fn parses_sources_and_preserves_download_urls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sources = root.join("../../fixtures/sources");
    let corpus = Corpus::load_dir(&sources).expect("load corpus");

    let found = corpus.sources.iter().find(|source| {
        source
            .rule_book_info
            .as_ref()
            .and_then(|rule| rule.download_urls.as_deref())
            .is_some_and(|value| !value.trim().is_empty())
    });

    assert!(found.is_some(), "expected at least one downloadUrls source");
    let source = found.unwrap();
    println!(
        "{} -> {:?}",
        source.book_source_name,
        source
            .rule_book_info
            .as_ref()
            .and_then(|rule| rule.download_urls.as_deref())
    );
}
