use std::path::PathBuf;

use engine_native::upstream::rule::RuleEngine;
use engine_native::Corpus;

#[test]
fn compiles_every_selector_rule_in_corpus() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sources");
    let corpus = Corpus::load_dir(&root).expect("load corpus");
    let engine = RuleEngine::new();

    let mut total = 0usize;
    let mut errors = Vec::new();
    for source in &corpus.sources {
        for rule in source.selector_rule_strings() {
            if rule.trim().is_empty() {
                continue;
            }
            total += 1;
            if let Err(error) = engine.compile(rule) {
                if errors.len() < 20 {
                    errors.push(format!("{} | {rule} | {error}", source.book_source_name));
                }
            }
        }
    }

    eprintln!("compiled {total} selector rules, {} errors", errors.len());
    for error in &errors {
        eprintln!("{error}");
    }
    assert!(total > 1000, "expected a substantial rule corpus");
    assert!(errors.is_empty(), "rule compile errors: {errors:#?}");
}
