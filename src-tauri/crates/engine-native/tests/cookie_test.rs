use std::time::{SystemTime, UNIX_EPOCH};

use engine_native::upstream::core::CookieStore;
use engine_native::upstream::net::FileCookieStore;
use engine_native::Pipeline;

fn temp_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "engine-native-cookies-{}-{nanos}.json",
        std::process::id()
    ))
}

#[test]
fn persists_cookies_across_store_instances() {
    let path = temp_path();
    {
        let store = FileCookieStore::new(&path);
        store.set_cookie("https://example.com/login", "sid=abc");
        store.replace_cookie("https://example.com/login", "token=xyz");
    }

    let reloaded = FileCookieStore::new(&path);
    let cookie = reloaded.get_cookie("https://example.com/book/1");
    assert!(cookie.contains("sid=abc"), "{cookie}");
    assert!(cookie.contains("token=xyz"), "{cookie}");

    reloaded.remove_cookie("https://example.com/book/1");
    let empty = FileCookieStore::new(&path);
    assert!(empty.get_cookie("https://example.com").is_empty());

    let _ = std::fs::remove_file(path);
}

#[test]
fn pipeline_shares_persistent_cookie_store_with_js_and_http() {
    let path = temp_path();
    {
        let pipeline = Pipeline::with_cookie_file(&path).expect("pipeline");
        pipeline.set_cookie("https://example.com/login", "sid=abc");
    }

    let pipeline = Pipeline::with_cookie_file(&path).expect("pipeline");
    assert_eq!(pipeline.get_cookie("https://example.com/book/1"), "sid=abc");

    let _ = std::fs::remove_file(path);
}
