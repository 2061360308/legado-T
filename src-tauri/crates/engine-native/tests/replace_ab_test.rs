use std::fs;
use std::path::PathBuf;

use engine_native::upstream::core::ReplaceRule;
use engine_native::upstream::replace::apply_to_content;
use serde_json::Value;

#[tokio::test]
async fn matches_kotlin_replace_rule() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../app/build/ab/replace.jsonl");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "read Kotlin replace oracle {}: {error}; run ReplaceOracleTest first",
            path.display()
        )
    });
    let oracle: Value = serde_json::from_str(text.trim()).expect("replace oracle");
    assert_eq!(oracle["ok"], Value::Bool(true));

    let input = oracle["input"].as_str().expect("replace input");
    let rules: Vec<ReplaceRule> = oracle["rules"]
        .as_array()
        .expect("replace rules")
        .iter()
        .map(|value| ReplaceRule {
            name: value["name"].as_str().unwrap_or_default().to_string(),
            pattern: value["pattern"].as_str().unwrap_or_default().to_string(),
            replacement: value["replacement"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            is_regex: value["isRegex"].as_bool().unwrap_or(true),
            is_enabled: value["isEnabled"].as_bool().unwrap_or(true),
            scope_content: value["scopeContent"].as_bool().unwrap_or(true),
            scope_title: value["scopeTitle"].as_bool().unwrap_or(false),
            order: value["order"].as_i64().unwrap_or(0) as i32,
            ..Default::default()
        })
        .collect();

    let outcome = apply_to_content(&rules, "Book", "Mock", input, None).await;
    assert_eq!(outcome.text, oracle["result"].as_str().unwrap_or_default());
}
