use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::model::BookSource;
use crate::{EngineError, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub url: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Corpus {
    pub sources: Vec<BookSource>,
    pub duplicates: Vec<DuplicateGroup>,
    pub files: Vec<String>,
}

impl Corpus {
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|error| EngineError::InvalidSourceFile {
                path: dir.to_path_buf(),
                message: error.to_string(),
            })?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            })
            .collect();
        paths.sort();
        Self::load_files(&paths)
    }

    pub fn load_files(paths: &[PathBuf]) -> Result<Self> {
        let mut sources = Vec::new();
        let mut files = Vec::new();
        for path in paths {
            files.push(path.to_string_lossy().to_string());
            sources.extend(load_source_file(path)?);
        }
        Ok(Self::from_sources(sources, files))
    }

    pub fn from_sources(sources: Vec<BookSource>, files: Vec<String>) -> Self {
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut unique = Vec::new();
        let mut duplicate_counts: HashMap<String, usize> = HashMap::new();

        for source in sources {
            let key = source.key().to_string();
            if key.is_empty() {
                unique.push(source);
                continue;
            }
            if seen.contains_key(&key) {
                *duplicate_counts.entry(key).or_insert(0) += 1;
                continue;
            }
            seen.insert(key, unique.len());
            unique.push(source);
        }

        let mut duplicates: Vec<DuplicateGroup> = duplicate_counts
            .into_iter()
            .map(|(url, count)| DuplicateGroup {
                url,
                count: count + 1,
            })
            .collect();
        duplicates.sort_by(|left, right| left.url.cmp(&right.url));

        Self {
            sources: unique,
            duplicates,
            files,
        }
    }

    pub fn duplicate_extra(&self) -> usize {
        self.duplicates
            .iter()
            .map(|group| group.count.saturating_sub(1))
            .sum()
    }
}

fn load_source_file(path: &Path) -> Result<Vec<BookSource>> {
    let text = std::fs::read_to_string(path).map_err(|error| EngineError::InvalidSourceFile {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let value: Value =
        serde_json::from_str(&text).map_err(|error| EngineError::InvalidSourceFile {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let values = match value {
        Value::Array(values) => values,
        Value::Object(_) => vec![value],
        _ => {
            return Err(EngineError::InvalidSourceFile {
                path: path.to_path_buf(),
                message: "expected a book source object or array".to_string(),
            })
        }
    };

    let mut sources = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        sources.push(serde_json::from_value(value).map_err(|error| {
            EngineError::InvalidSourceFile {
                path: path.to_path_buf(),
                message: format!("source #{index}: {error}"),
            }
        })?);
    }
    Ok(sources)
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureStats {
    pub files: Vec<String>,
    pub total: usize,
    pub unique_urls: usize,
    pub duplicate_groups: usize,
    pub duplicate_extra: usize,
    pub enabled: usize,
    pub source_types: BTreeMap<i32, usize>,
    pub with_search: usize,
    pub with_explore: usize,
    pub with_js_lib: usize,
    pub with_login: usize,
    pub with_header: usize,
    pub modes: BTreeMap<String, usize>,
    pub features: BTreeMap<String, usize>,
    pub java_methods: BTreeMap<String, usize>,
    pub packages: BTreeMap<String, usize>,
}

const FEATURE_PATTERNS: &[(&str, &[&str])] = &[
    ("webView", &["webView"]),
    ("java.webView", &["java.webView("]),
    ("startBrowser", &["startBrowser("]),
    ("startBrowserAwait", &["startBrowserAwait("]),
    ("sourceRegex", &["sourceRegex"]),
    ("webJs", &["webJs"]),
    ("queryTTF", &["queryTTF"]),
    ("replaceFont", &["replaceFont"]),
    ("downloadUrls", &["downloadUrls"]),
    ("payAction", &["payAction"]),
    ("imageDecode", &["imageDecode"]),
    ("coverDecodeJs", &["coverDecodeJs"]),
    ("JavaImporter", &["JavaImporter"]),
    ("Packages.okhttp3", &["Packages.okhttp3"]),
    ("Packages.cn.hutool", &["Packages.cn.hutool"]),
    ("cookie.getKey", &["cookie.getKey"]),
    ("source.getLoginInfoMap", &["source.getLoginInfoMap"]),
    ("source.getLoginHeader", &["source.getLoginHeader"]),
    ("java.ajaxAll", &["java.ajaxAll"]),
    (
        "java.createSymmetricCrypto",
        &["java.createSymmetricCrypto"],
    ),
    ("java.HMac", &["java.HMac"]),
    ("java.md5Encode", &["java.md5Encode"]),
    ("java.getString", &["java.getString"]),
    ("java.get", &["java.get(", "java.get ("]),
    ("java.put", &["java.put(", "java.put ("]),
    ("java.timeFormatUTC", &["java.timeFormatUTC"]),
    ("java.toNumChapter", &["java.toNumChapter"]),
];

pub fn analyze(corpus: &Corpus) -> FeatureStats {
    let mut stats = FeatureStats {
        files: corpus.files.clone(),
        total: corpus.sources.len(),
        duplicate_groups: corpus.duplicates.len(),
        duplicate_extra: corpus.duplicate_extra(),
        ..Default::default()
    };

    let mut unique_urls = HashSet::new();
    for source in &corpus.sources {
        if source.is_enabled() {
            stats.enabled += 1;
        }
        if !source.key().is_empty() {
            unique_urls.insert(source.key().to_string());
        }
        *stats
            .source_types
            .entry(source.book_source_type)
            .or_insert(0) += 1;
        if source.search_url.as_deref().is_some_and(non_blank) {
            stats.with_search += 1;
        }
        if source.explore_url.as_deref().is_some_and(non_blank) {
            stats.with_explore += 1;
        }
        if source.js_lib.as_deref().is_some_and(non_blank) {
            stats.with_js_lib += 1;
        }
        if source.login_url.as_deref().is_some_and(non_blank) {
            stats.with_login += 1;
        }
        if source.header.as_deref().is_some_and(non_blank) {
            stats.with_header += 1;
        }

        let rules = source.selector_rule_strings();
        if !rules.is_empty() {
            let mut modes = detect_modes(&rules);
            let text = source_text(source);
            if text.contains("@js:") || text.contains("<js>") || text.contains("{{") {
                modes.insert("js");
            }
            for mode in modes {
                *stats.modes.entry(mode.to_string()).or_insert(0) += 1;
            }
        }

        let text = source_text(source);
        for (name, needles) in FEATURE_PATTERNS {
            let present = match *name {
                "downloadUrls" => source
                    .rule_book_info
                    .as_ref()
                    .and_then(|rule| rule.download_urls.as_deref())
                    .is_some_and(non_blank),
                "sourceRegex" => source
                    .rule_content
                    .as_ref()
                    .and_then(|rule| rule.source_regex.as_deref())
                    .is_some_and(non_blank),
                "webJs" => source
                    .rule_content
                    .as_ref()
                    .and_then(|rule| rule.web_js.as_deref())
                    .is_some_and(non_blank),
                "imageDecode" => source
                    .rule_content
                    .as_ref()
                    .and_then(|rule| rule.image_decode.as_deref())
                    .is_some_and(non_blank),
                "payAction" => source
                    .rule_content
                    .as_ref()
                    .and_then(|rule| rule.pay_action.as_deref())
                    .is_some_and(non_blank),
                "coverDecodeJs" => source.cover_decode_js.as_deref().is_some_and(non_blank),
                _ => needles.iter().any(|needle| text.contains(needle)),
            };
            if present {
                *stats.features.entry((*name).to_string()).or_insert(0) += 1;
            }
        }
    }

    stats.unique_urls = unique_urls.len();

    let java_regex = Regex::new(r"java\.([A-Za-z_][A-Za-z0-9_]*)").expect("valid java regex");
    let package_regex = Regex::new(r"Packages\.([A-Za-z0-9_\.]+)").expect("valid package regex");
    for source in &corpus.sources {
        let text = source_text(source);
        for captures in java_regex.captures_iter(&text) {
            let name = &captures[1];
            if is_java_package_segment(name) {
                continue;
            }
            *stats.java_methods.entry(name.to_string()).or_insert(0) += 1;
        }
        for captures in package_regex.captures_iter(&text) {
            let name = captures[1].trim_end_matches('.').to_string();
            *stats.packages.entry(name).or_insert(0) += 1;
        }
    }

    stats
}

fn source_text(source: &BookSource) -> String {
    let value = serde_json::to_value(source).unwrap_or(Value::Null);
    let mut values = Vec::new();
    collect_strings(&value, &mut values);
    values.join("\n")
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_strings(item, out);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_strings(item, out);
            }
        }
        _ => {}
    }
}

fn non_blank(value: &str) -> bool {
    !value.trim().is_empty()
}

fn is_java_package_segment(name: &str) -> bool {
    matches!(
        name,
        "lang" | "util" | "io" | "security" | "net" | "text" | "math" | "nio"
    )
}

fn detect_modes<'a>(rules: &[&'a str]) -> BTreeSet<&'static str> {
    let mut modes = BTreeSet::new();
    for raw in rules {
        let rule = raw.trim();
        if rule.is_empty() {
            continue;
        }
        if rule.contains("@js:")
            || rule.contains("<js>")
            || rule.contains("{{")
            || rule.to_ascii_lowercase().contains("@js:")
        {
            modes.insert("js");
        }
        if rule.contains("@XPath:") || rule.contains("@xpath:") || rule.starts_with('/') {
            modes.insert("xpath");
        }
        if rule.contains("@Json:")
            || rule.contains("@json:")
            || rule.starts_with("$.")
            || rule.starts_with("$[")
        {
            modes.insert("jsonpath");
        }
        if rule.contains("##") || rule.starts_with(':') || rule.contains("@regex:") {
            modes.insert("regex");
        }
        if rule.contains("@CSS:") || rule.contains("@css:") || rule.contains("@@") {
            modes.insert("css");
        }
    }
    if modes.is_empty() && rules.iter().any(|rule| !rule.trim().is_empty()) {
        modes.insert("css");
    }
    modes
}

pub fn render_markdown(stats: &FeatureStats) -> String {
    let mut out = String::new();
    out.push_str("# Legado 书源语料特征报告\n\n");
    out.push_str("> 由 `engine-native stats` 生成，用于确定规则引擎实现优先级。\n\n");

    out.push_str("## 语料文件\n\n");
    for file in &stats.files {
        let _ = writeln!(out, "- `{file}`");
    }
    out.push('\n');

    out.push_str("## 汇总\n\n");
    out.push_str("| 指标 | 数值 |\n|---|---:|\n");
    let _ = writeln!(out, "| 书源总数 | {} |", stats.total);
    let _ = writeln!(out, "| 唯一 URL | {} |", stats.unique_urls);
    let _ = writeln!(out, "| 重复 URL 组 | {} |", stats.duplicate_groups);
    let _ = writeln!(out, "| 重复多余条数 | {} |", stats.duplicate_extra);
    let _ = writeln!(out, "| 启用 | {} |", stats.enabled);
    let _ = writeln!(out, "| 有 searchUrl | {} |", stats.with_search);
    let _ = writeln!(out, "| 有 exploreUrl | {} |", stats.with_explore);
    let _ = writeln!(out, "| 有 jsLib | {} |", stats.with_js_lib);
    let _ = writeln!(out, "| 有 loginUrl | {} |", stats.with_login);
    let _ = writeln!(out, "| 有 header | {} |", stats.with_header);
    out.push('\n');

    out.push_str("## 书源类型\n\n");
    out.push_str("| bookSourceType | 数量 |\n|---|---:|\n");
    for (kind, count) in &stats.source_types {
        let label = match kind {
            0 => "0 文本",
            1 => "1 音频",
            2 => "2 图片",
            3 => "3 文件",
            _ => "其他",
        };
        let _ = writeln!(out, "| {label} | {count} |");
    }
    out.push('\n');

    out.push_str("## 规则模式\n\n");
    out.push_str("| 模式 | 书源数 |\n|---|---:|\n");
    for (mode, count) in &stats.modes {
        let _ = writeln!(out, "| {mode} | {count} |");
    }
    out.push('\n');

    out.push_str("## 长尾能力\n\n");
    out.push_str("| 能力 | 书源数 |\n|---|---:|\n");
    for (name, count) in &stats.features {
        let _ = writeln!(out, "| {name} | {count} |");
    }
    out.push('\n');

    out.push_str("## java.* 方法频次\n\n");
    out.push_str("| 方法 | 出现次数 |\n|---|---:|\n");
    let mut methods: Vec<_> = stats.java_methods.iter().collect();
    methods.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    for (name, count) in methods {
        let _ = writeln!(out, "| java.{name} | {count} |");
    }
    out.push('\n');

    out.push_str("## Packages 类/包频次\n\n");
    out.push_str("| Packages 路径 | 出现次数 |\n|---|---:|\n");
    let mut packages: Vec<_> = stats.packages.iter().collect();
    packages.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    for (name, count) in packages {
        let _ = writeln!(out, "| Packages.{name} | {count} |");
    }
    out.push('\n');

    out.push_str("## 实施优先级建议\n\n");
    out.push_str("1. `java.*` 高频函数、`cookie` / `source` / `book` / `chapter` 绑定。\n");
    out.push_str("2. JS 规则、`jsLib`、`JavaImporter`、`Packages`。\n");
    out.push_str("3. JSONPath、Regex、URL/HTTP。\n");
    out.push_str("4. XPath。\n");
    out.push_str("5. WebView / 登录 / 文件源 / TTF / payAction。\n");
    out
}
