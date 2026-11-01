use std::fs;
use std::path::PathBuf;

use engine_native::parse_sources;

#[test]
fn parses_both_corpus_files_into_engine_model_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sources");
    for (name, expected) in [
        ("1000-pure-novel.json", 141usize),
        ("qq-2026-01-28.json", 600usize),
    ] {
        let path = root.join(name);
        let json = fs::read_to_string(&path).expect("read corpus");
        let sources = parse_sources(&json).expect("parse corpus");
        assert_eq!(sources.len(), expected, "{name}");
    }
}
