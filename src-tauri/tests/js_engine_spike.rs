use local_reader_lib::platform::js_engine::{JsEngine, SandboxPolicy};
use std::time::Duration;

#[test]
fn blocks_network_tokens_by_default() {
    let engine = JsEngine::default();
    assert!(engine
        .precheck("const r = fetch('http://example.com');")
        .is_err());
    assert!(engine.precheck("new XMLHttpRequest()").is_err());
}

#[test]
fn allows_plain_script() {
    let engine = JsEngine::default();
    assert!(engine.precheck("var a = 1 + 1; a").is_ok());
}

#[test]
fn timeout_aborts_long_task() {
    let engine = JsEngine::new(SandboxPolicy {
        timeout: Duration::from_millis(50),
        allow_net: false,
        allow_fs: false,
    });
    let result = engine.eval_with_timeout(|| {
        std::thread::sleep(Duration::from_millis(500));
        1
    });
    assert!(result.is_err());
}

#[test]
fn fast_task_returns_value() {
    let engine = JsEngine::default();
    assert_eq!(engine.eval_with_timeout(|| 42).unwrap(), 42);
}
