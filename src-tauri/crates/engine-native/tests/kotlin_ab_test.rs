use std::fs;
use std::path::PathBuf;

use engine_native::upstream::regex_engine::Regex;
use engine_native::{eval_list, eval_text};
use serde_json::{json, Value};

fn oracle_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../app/build/ab/kotlin.jsonl")
}

fn cases_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../app/src/test/resources/ab/cases.jsonl")
}

async fn rust_result(case: &Value) -> Value {
    let mode = case["mode"].as_str().unwrap_or("string");
    let rule = case["rule"].as_str().unwrap_or_default();
    let content = case["content"].as_str().unwrap_or_default();
    match mode {
        "list" => json!(eval_list(rule, content).await.unwrap()),
        "element" => {
            let pattern = rule.strip_prefix(':').unwrap_or(rule);
            let regex = Regex::new(pattern).unwrap();
            match regex.captures(content).unwrap() {
                Some(captures) => {
                    let groups = (0..captures.len())
                        .map(|index| captures.get(index).unwrap_or_default().to_string())
                        .collect::<Vec<_>>();
                    json!(groups)
                }
                None => Value::Null,
            }
        }
        _ => json!(eval_text(rule, content).await.unwrap()),
    }
}

#[tokio::test]
async fn matches_kotlin_rule_oracle() {
    let path = oracle_path();
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "read Kotlin oracle {}: {error}; run the RuleOracleTest first",
            path.display()
        )
    });

    let cases_text = fs::read_to_string(cases_path()).expect("read A/B cases");
    let cases: std::collections::HashMap<String, Value> = cases_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let case: Value = serde_json::from_str(line).expect("case json line");
            let id = case["id"].as_str().expect("case id").to_string();
            (id, case)
        })
        .collect();

    let mut compared = 0usize;
    let mut mismatches = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let kotlin: Value = serde_json::from_str(line).expect("oracle json line");
        let id = kotlin["id"].as_str().expect("oracle id");
        assert_eq!(kotlin["ok"], Value::Bool(true), "Kotlin failed: {line}");
        let case = cases.get(id).unwrap_or_else(|| panic!("missing case {id}"));
        let actual = rust_result(case).await;
        let expected = &kotlin["result"];
        if &actual != expected {
            mismatches.push(format!(
                "{id}: kotlin={} rust={}",
                serde_json::to_string(expected).unwrap_or_default(),
                serde_json::to_string(&actual).unwrap_or_default()
            ));
        }
        compared += 1;
    }

    assert_eq!(compared, 32, "unexpected oracle case count");
    assert!(
        mismatches.is_empty(),
        "Kotlin/Rust mismatches:\n{}",
        mismatches.join("\n")
    );
}
