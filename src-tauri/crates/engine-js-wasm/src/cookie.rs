#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use engine_http_native::sub_domain;
use engine_model::CookieStore;

thread_local! {
    pub(crate) static COOKIES: RefCell<Option<Arc<WasmCookieStore>>> = const { RefCell::new(None) };
}

#[derive(Default)]
pub(crate) struct WasmCookieStore {
    cookies: Mutex<BTreeMap<String, String>>,
}

impl WasmCookieStore {
    fn with<T>(&self, action: impl FnOnce(&mut BTreeMap<String, String>) -> T) -> T {
        let mut cookies = self
            .cookies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        action(&mut cookies)
    }

    fn remove_key(&self, url: &str, key: &str) {
        let domain = sub_domain(url);
        self.with(|cookies| {
            let Some(existing) = cookies.get(&domain).cloned() else {
                return;
            };
            let mut map = BTreeMap::new();
            for pair in existing.split(';') {
                if let Some((name, value)) = pair.split_once('=') {
                    let name = name.trim();
                    if !name.is_empty() {
                        map.insert(name.to_string(), value.trim().to_string());
                    }
                }
            }
            map.remove(key);
            if map.is_empty() {
                cookies.remove(&domain);
            } else {
                cookies.insert(
                    domain,
                    map.into_iter()
                        .map(|(name, value)| format!("{name}={value}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                );
            }
        });
    }
}

impl CookieStore for WasmCookieStore {
    fn get_cookie(&self, url: &str) -> String {
        let domain = sub_domain(url);
        self.with(|cookies| cookies.get(&domain).cloned().unwrap_or_default())
    }

    fn set_cookie(&self, url: &str, cookie: &str) {
        let domain = sub_domain(url);
        self.with(|cookies| {
            cookies.insert(domain, cookie.to_string());
        });
    }

    fn replace_cookie(&self, url: &str, cookie: &str) {
        let domain = sub_domain(url);
        self.with(|cookies| {
            let merged = match cookies.get(&domain) {
                Some(existing) if !existing.is_empty() => merge_cookie_text(existing, cookie),
                _ => cookie.to_string(),
            };
            cookies.insert(domain, merged);
        });
    }

    fn remove_cookie(&self, url: &str) {
        let domain = sub_domain(url);
        self.with(|cookies| {
            cookies.remove(&domain);
        });
    }
}

fn merge_cookie_text(left: &str, right: &str) -> String {
    let mut map = BTreeMap::new();
    for pair in left.split(';').chain(right.split(';')) {
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim();
            if !key.is_empty() {
                map.insert(key.to_string(), value.trim().to_string());
            }
        }
    }
    map.into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn apply_set_cookie(url: &str, header: &str) {
    let Some(pair) = header.split(';').next() else {
        return;
    };
    let Some((name, value)) = pair.split_once('=') else {
        return;
    };
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    let value = value.trim();
    let lower = header.to_ascii_lowercase();
    let expired = value.is_empty()
        || lower.contains("max-age=0")
        || lower.contains("expires=thu, 01 jan 1970");
    COOKIES.with(|slot| {
        let store = slot.borrow().as_ref().cloned();
        let Some(store) = store else {
            return;
        };
        if expired {
            store.remove_key(url, name);
        } else {
            store.replace_cookie(url, &format!("{name}={value}"));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_and_removes_response_cookies() {
        COOKIES.with(|slot| *slot.borrow_mut() = Some(Arc::new(WasmCookieStore::default())));
        apply_set_cookie("https://example.com/book", "sid=1; Path=/; HttpOnly");
        apply_set_cookie("https://example.com/book", "theme=dark; Path=/");

        let store = COOKIES.with(|slot| slot.borrow().as_ref().cloned()).unwrap();
        assert_eq!(store.get_cookie("https://example.com/book"), "sid=1; theme=dark");

        apply_set_cookie("https://example.com/book", "sid=; Max-Age=0");
        assert_eq!(store.get_cookie("https://example.com/book"), "theme=dark");

        COOKIES.with(|slot| *slot.borrow_mut() = None);
    }
}
