//! Java/ICU-compatible regex surface backed by `fancy-regex`.
//!
//! The original crate bound ICU4C directly. This build keeps the same public
//! API but uses a pure-Rust engine so the workspace can compile on Windows
//! without ICU native libraries. `fancy-regex` supports look-around and
//! back-references, which are common in Legado book-source rules.

use fancy_regex::Regex as InnerRegex;

#[derive(Debug, Clone)]
pub enum Error {
    Syntax(String),
    Runtime(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Syntax(m) => write!(f, "正则编译失败：{m}"),
            Error::Runtime(m) => write!(f, "正则求值失败：{m}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

const UREGEX_CASE_INSENSITIVE: u32 = 2;
const UREGEX_COMMENTS: u32 = 4;
const UREGEX_MULTILINE: u32 = 8;
const UREGEX_DOTALL: u32 = 32;

fn inline_flags(flags: u32) -> String {
    let mut out = String::new();
    if flags & UREGEX_CASE_INSENSITIVE != 0 {
        out.push('i');
    }
    if flags & UREGEX_MULTILINE != 0 {
        out.push('m');
    }
    if flags & UREGEX_DOTALL != 0 {
        out.push('s');
    }
    if flags & UREGEX_COMMENTS != 0 {
        out.push('x');
    }
    if out.is_empty() {
        String::new()
    } else {
        format!("(?{out})")
    }
}

#[derive(Debug, Clone)]
pub struct Captures {
    groups: Vec<Option<String>>,
    start: usize,
    end: usize,
}

impl Captures {
    fn from_inner(caps: &fancy_regex::Captures<'_>) -> Self {
        let mut groups = Vec::with_capacity(caps.len());
        for index in 0..caps.len() {
            groups.push(caps.get(index).map(|m| m.as_str().to_string()));
        }
        let (start, end) = caps.get(0).map(|m| (m.start(), m.end())).unwrap_or((0, 0));
        Self { groups, start, end }
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        self.groups.get(i).and_then(|g| g.as_deref())
    }

    pub fn byte_span(&self, _text: &str) -> (usize, usize) {
        (self.start, self.end)
    }
}

pub struct Regex {
    pattern: String,
    flags: u32,
    inner: InnerRegex,
    time_limit_ms: u64,
}

impl Clone for Regex {
    fn clone(&self) -> Self {
        Self::with_flags(&self.pattern, self.flags)
            .expect("已编译过的正则克隆失败")
            .with_time_limit(self.time_limit_ms)
    }
}

impl std::fmt::Debug for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Regex({:?})", self.pattern)
    }
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self> {
        Self::with_flags(pattern, 0)
    }

    pub fn with_flags(pattern: &str, flags: u32) -> Result<Self> {
        let source = format!("{}{}", inline_flags(flags), pattern);
        let inner = InnerRegex::new(&source).map_err(|e| Error::Syntax(e.to_string()))?;
        Ok(Self {
            pattern: pattern.to_string(),
            flags,
            inner,
            time_limit_ms: 0,
        })
    }

    pub fn with_time_limit(mut self, millis: u64) -> Self {
        self.time_limit_ms = millis;
        self
    }

    pub fn as_str(&self) -> &str {
        &self.pattern
    }

    pub fn is_match(&self, text: &str) -> Result<bool> {
        self.inner
            .is_match(text)
            .map_err(|e| Error::Runtime(e.to_string()))
    }

    pub fn find(&self, text: &str) -> Result<Option<(usize, usize)>> {
        Ok(self.captures(text)?.map(|caps| caps.byte_span(text)))
    }

    pub fn captures(&self, text: &str) -> Result<Option<Captures>> {
        self.inner
            .captures(text)
            .map(|caps| caps.map(|c| Captures::from_inner(&c)))
            .map_err(|e| Error::Runtime(e.to_string()))
    }

    pub fn captures_iter(&self, text: &str) -> Result<Vec<Captures>> {
        let mut out = Vec::new();
        for caps in self.inner.captures_iter(text) {
            let caps = caps.map_err(|e| Error::Runtime(e.to_string()))?;
            out.push(Captures::from_inner(&caps));
        }
        Ok(out)
    }

    pub fn replace(&self, text: &str, replacement: &str) -> Result<String> {
        self.replacen(text, 1, replacement)
    }

    pub fn replace_all(&self, text: &str, replacement: &str) -> Result<String> {
        self.replacen(text, 0, replacement)
    }

    pub fn replacen(&self, text: &str, limit: usize, replacement: &str) -> Result<String> {
        self.replace_with(text, limit, |caps| {
            Ok(expand_replacement(replacement, caps))
        })
    }

    pub fn replace_with<F>(&self, text: &str, limit: usize, mut replacement: F) -> Result<String>
    where
        F: FnMut(&Captures) -> Result<String>,
    {
        let mut out = String::with_capacity(text.len());
        let mut last = 0usize;
        let mut done = 0usize;
        for caps in self.inner.captures_iter(text) {
            let caps = caps.map_err(|e| Error::Runtime(e.to_string()))?;
            let caps = Captures::from_inner(&caps);
            if limit != 0 && done >= limit {
                break;
            }
            if caps.start < last || caps.end < caps.start {
                continue;
            }
            out.push_str(&text[last..caps.start]);
            out.push_str(&replacement(&caps)?);
            last = caps.end;
            done += 1;
        }
        out.push_str(&text[last..]);
        Ok(out)
    }
}

fn expand_replacement(replacement: &str, caps: &Captures) -> String {
    let chars: Vec<char> = replacement.chars().collect();
    let mut out = String::with_capacity(replacement.len());
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            '\\' => {
                if i < chars.len() {
                    out.push(chars[i]);
                    i += 1;
                } else {
                    out.push('\\');
                }
            }
            '$' => {
                if i < chars.len() && chars[i] == '{' {
                    let mut j = i + 1;
                    while j < chars.len() && chars[j] != '}' {
                        j += 1;
                    }
                    let name: String = chars[i + 1..j].iter().collect();
                    if let Ok(idx) = name.parse::<usize>() {
                        out.push_str(caps.get(idx).unwrap_or("null"));
                    }
                    i = (j + 1).min(chars.len());
                } else {
                    let mut idx: Option<usize> = None;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        let d = chars[i] as usize - '0' as usize;
                        idx = Some(idx.unwrap_or(0) * 10 + d);
                        i += 1;
                    }
                    match idx {
                        Some(n) if n < caps.len() => out.push_str(caps.get(n).unwrap_or("null")),
                        Some(n) => {
                            out.push('$');
                            out.push_str(&n.to_string());
                        }
                        None => out.push('$'),
                    }
                }
            }
            other => out.push(other),
        }
    }
    out
}
