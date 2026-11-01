//! Re-exports of the vendored Legado engine crates.
//!
//! These crates come from `SCX-snow/legado-rust-engine`. The XPath and regex
//! backends were patched to use pure-Rust `sxd-xpath` and `fancy-regex` so the
//! workspace builds without libxml2 / ICU native libraries.

pub use engine_flow as book;
pub use engine_model as core;
pub use engine_js as js;
pub use engine_http_native as net;
pub use engine_regex as regex_engine;
pub use engine_replace as replace;
pub use engine_rss as rss;
pub use engine_rule as rule;
