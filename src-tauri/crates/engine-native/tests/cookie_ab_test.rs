use std::fs;
use std::path::PathBuf;

use engine_native::upstream::net::cookie_to_map;
use serde_json::{json, Value};

#[test]
fn matches_kotlin_cookie_parser() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cookie.jsonl");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("read cookie oracle {}: {error}", path.display())
    });
    let oracle: Value = serde_json::from_str(text.trim()).expect("cookie oracle");
    assert_eq!(oracle["ok"], Value::Bool(true));
    let input = oracle["input"].as_str().expect("cookie input");
    let actual = json!(cookie_to_map(input));
    assert_eq!(actual, oracle["map"]);
}
