#[cfg(feature = "native")]
pub mod client;
#[cfg(feature = "native")]
pub mod cookie;
pub mod domain;
pub mod encode;
pub mod json;
pub mod login;
#[cfg(feature = "native")]
pub mod rate;
pub mod url;
#[cfg(feature = "native")]
pub mod webview;

#[cfg(feature = "native")]
pub use client::{HttpClient, HttpClientBuilder};
#[cfg(feature = "native")]
pub use cookie::{FileCookieStore, MemoryCookieStore, cookie_to_map, map_to_cookie, merge_cookies};
pub use domain::{base_url, sub_domain};
pub use encode::{
    content_type_charset, decode, detect_encoding, escape, has_url_encoded, resolve_encoding,
    url_encode,
};
pub use json::parse as parse_lenient_json;
pub use login::check_login;
#[cfg(feature = "native")]
pub use rate::{RateGuard, RateLimiter};
pub use url::{DEFAULT_USER_AGENT, UrlContext, UrlRule, absolute_url};
#[cfg(feature = "native")]
pub use webview::CommandWebViewBridge;
