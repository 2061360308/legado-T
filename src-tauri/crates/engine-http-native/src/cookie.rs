use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine_model::CookieStore;

use crate::domain::sub_domain;

pub fn cookie_to_map(cookie: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if cookie.trim().is_empty() {
        return map;
    }
    for pair in cookie.split(';') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() {
            continue;
        }
        if !value.is_empty() || value == "null" {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

pub fn map_to_cookie(map: &BTreeMap<String, String>) -> String {
    map.iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn merge_cookies(cookies: &[&str]) -> String {
    let mut map = BTreeMap::new();
    for cookie in cookies {
        map.extend(cookie_to_map(cookie));
    }
    map_to_cookie(&map)
}

#[derive(Default)]
pub struct MemoryCookieStore {
    cookies: Mutex<BTreeMap<String, String>>,
}

impl MemoryCookieStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn with<T>(&self, action: impl FnOnce(&mut BTreeMap<String, String>) -> T) -> T {
        let mut cookies = self
            .cookies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        action(&mut cookies)
    }
}

impl CookieStore for MemoryCookieStore {
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
                Some(existing) if !existing.is_empty() => merge_cookies(&[existing, cookie]),
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

/// Cookie store persisted as a JSON map of `domain -> cookie`.
///
/// This is intentionally small and dependency-free: the engine only needs to
/// keep login cookies across process restarts. The app can point it at any
/// writable path (for example `{app_data}/engine-native/cookies.json`).
pub struct FileCookieStore {
    cookies: Mutex<BTreeMap<String, String>>,
    path: PathBuf,
}

impl FileCookieStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let cookies = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            cookies: Mutex::new(cookies),
            path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn with<T>(&self, action: impl FnOnce(&BTreeMap<String, String>) -> T) -> T {
        let cookies = self
            .cookies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        action(&cookies)
    }

    fn with_mut<T>(&self, action: impl FnOnce(&mut BTreeMap<String, String>) -> T) -> T {
        let mut cookies = self
            .cookies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = action(&mut cookies);
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec_pretty(&*cookies) {
            let _ = fs::write(&self.path, bytes);
        }
        result
    }
}

impl CookieStore for FileCookieStore {
    fn get_cookie(&self, url: &str) -> String {
        let domain = sub_domain(url);
        self.with(|cookies| cookies.get(&domain).cloned().unwrap_or_default())
    }

    fn set_cookie(&self, url: &str, cookie: &str) {
        let domain = sub_domain(url);
        self.with_mut(|cookies| {
            cookies.insert(domain, cookie.to_string());
        });
    }

    fn replace_cookie(&self, url: &str, cookie: &str) {
        let domain = sub_domain(url);
        self.with_mut(|cookies| {
            let merged = match cookies.get(&domain) {
                Some(existing) if !existing.is_empty() => merge_cookies(&[existing, cookie]),
                _ => cookie.to_string(),
            };
            cookies.insert(domain, merged);
        });
    }

    fn remove_cookie(&self, url: &str) {
        let domain = sub_domain(url);
        self.with_mut(|cookies| {
            cookies.remove(&domain);
        });
    }
}
