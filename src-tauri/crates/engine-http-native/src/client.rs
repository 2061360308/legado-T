use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http::header::{ACCEPT_ENCODING, CONTENT_TYPE, COOKIE, USER_AGENT};
use engine_model::{
    BoxFuture, CookieStore, Error, HttpFetcher, HttpRequest, HttpResponse, JsRequest, Result,
    WebViewBridge, WebViewRequest,
};
use reqwest::header::{HeaderMap as ReqHeaderMap, HeaderValue as ReqHeaderValue};

use crate::cookie::{MemoryCookieStore, merge_cookies};
use crate::encode;
use crate::rate::RateLimiter;
use crate::url::{DEFAULT_USER_AGENT, UrlContext, UrlRule};

#[derive(Clone)]
pub struct HttpClientBuilder {
    connect_timeout: Duration,
    read_timeout: Duration,
    total_timeout: Duration,
    proxy: Option<String>,
    accept_invalid_certs: bool,
    user_agent: String,
    cookies: Option<Arc<dyn CookieStore>>,
    webview: Option<Arc<dyn WebViewBridge>>,
    max_redirects: usize,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(15),
            read_timeout: Duration::from_secs(60),
            total_timeout: Duration::from_secs(60),
            proxy: None,
            accept_invalid_certs: true,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            cookies: None,
            webview: None,
            max_redirects: 10,
        }
    }
}

impl HttpClientBuilder {
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    pub fn total_timeout(mut self, timeout: Duration) -> Self {
        self.total_timeout = timeout;
        self
    }

    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn webview(mut self, webview: Arc<dyn WebViewBridge>) -> Self {
        self.webview = Some(webview);
        self
    }

    pub fn cookies(mut self, cookies: Arc<dyn CookieStore>) -> Self {
        self.cookies = Some(cookies);
        self
    }

    pub fn build(self) -> Result<HttpClient> {
        let cookies = self
            .cookies
            .clone()
            .unwrap_or_else(|| Arc::new(MemoryCookieStore::new()));
        let base = self.build_pair(None)?;
        let webview = self.webview.clone();
        Ok(HttpClient {
            base,
            config: self,
            cookies,
            webview,
            proxied: Mutex::new(HashMap::new()),
            rate: RateLimiter::new(),
        })
    }

    fn build_pair(&self, proxy: Option<&str>) -> Result<ClientPair> {
        let proxy = proxy.map(str::trim).filter(|text| !text.is_empty());
        Ok(ClientPair {
            plain: self.build_client(proxy, false, true)?,
            jar: self.build_client(proxy, true, true)?,
            plain_no_ua: self.build_client(proxy, false, false)?,
            jar_no_ua: self.build_client(proxy, true, false)?,
        })
    }

    fn build_client(
        &self,
        proxy: Option<&str>,
        cookie_jar: bool,
        include_user_agent: bool,
    ) -> Result<reqwest::Client> {
        let mut default_headers = ReqHeaderMap::new();
        if include_user_agent {
            let user_agent = ReqHeaderValue::from_str(&self.user_agent)
                .map_err(|error| Error::Http(format!("User-Agent �Ƿ�: {error}")))?;
            default_headers.insert(http::header::USER_AGENT, user_agent);
        }
        default_headers.insert(
            http::header::CACHE_CONTROL,
            ReqHeaderValue::from_static("no-cache"),
        );

        default_headers.insert(
            http::header::ACCEPT_ENCODING,
            ReqHeaderValue::from_static("gzip, deflate"),
        );

        let mut builder = reqwest::Client::builder()
            .connect_timeout(self.connect_timeout)
            .read_timeout(self.read_timeout)
            .timeout(self.total_timeout)
            .redirect(reqwest::redirect::Policy::limited(self.max_redirects))
            .danger_accept_invalid_certs(self.accept_invalid_certs)
            .danger_accept_invalid_hostnames(self.accept_invalid_certs)
            .default_headers(default_headers)
            .cookie_store(cookie_jar);

        let proxy = proxy.or(self.proxy.as_deref());
        if let Some(proxy) = proxy {
            builder = builder.proxy(parse_proxy(proxy)?);
        }
        builder
            .build()
            .map_err(|error| Error::Http(format!("构造 HTTP 客户端失败: {error}")))
    }
}

#[derive(Clone)]
struct ClientPair {
    plain: reqwest::Client,
    jar: reqwest::Client,
    plain_no_ua: reqwest::Client,
    jar_no_ua: reqwest::Client,
}

pub struct HttpClient {
    base: ClientPair,
    config: HttpClientBuilder,
    cookies: Arc<dyn CookieStore>,
    webview: Option<Arc<dyn WebViewBridge>>,
    proxied: Mutex<HashMap<String, ClientPair>>,
    rate: RateLimiter,
}

impl HttpClient {
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }

    pub fn new() -> Result<Self> {
        HttpClient::builder().build()
    }

    pub fn cookies(&self) -> &dyn CookieStore {
        self.cookies.as_ref()
    }

    fn client_for(
        &self,
        proxy: Option<&str>,
        cookie_jar: bool,
        include_user_agent: bool,
    ) -> Result<reqwest::Client> {
        let proxy = proxy.map(str::trim).filter(|text| !text.is_empty());
        let Some(proxy) = proxy else {
            return Ok(pick_client(&self.base, cookie_jar, include_user_agent));
        };
        {
            let cache = self
                .proxied
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(pair) = cache.get(proxy) {
                return Ok(pick_client(pair, cookie_jar, include_user_agent));
            }
        }
        let pair = self.config.build_pair(Some(proxy))?;
        let picked = pick_client(&pair, cookie_jar, include_user_agent);
        self.proxied
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(proxy.to_string(), pair);
        Ok(picked)
    }

    pub async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let include_user_agent = !request_disables_user_agent(&request);
        let client = self.client_for(None, false, include_user_agent)?;
        self.send_with(&client, request).await
    }

    pub async fn send_retry(&self, request: HttpRequest, retry: u32) -> Result<HttpResponse> {
        let include_user_agent = !request_disables_user_agent(&request);
        let client = self.client_for(None, false, include_user_agent)?;
        self.send_retry_with(&client, request, retry).await
    }

    pub async fn send_js(
        &self,
        request: HttpRequest,
        rate: Option<&str>,
        rate_key: &str,
    ) -> Result<HttpResponse> {
        let _guard = self.rate_guard(rate_key, rate).await;
        self.send(request).await
    }

    pub async fn send_rule(&self, rule: &UrlRule) -> Result<HttpResponse> {
        self.send_rule_with_source_regex(rule, None).await
    }

    pub async fn send_rule_with_source_regex(
        &self,
        rule: &UrlRule,
        source_regex: Option<&str>,
    ) -> Result<HttpResponse> {
        let rate_key = rule.rate_key().unwrap_or_default();
        let _guard = self.rate_guard(rate_key, rule.rate()).await;

        let mut request = rule.request();
        let cookie = self.cookies.get_cookie(rule.cookie_url());
        if !cookie.is_empty() {
            let existing = request
                .headers
                .get(COOKIE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let merged = merge_cookies(&[&existing, &cookie]);
            if let Ok(value) = http::HeaderValue::from_str(&merged) {
                request.headers.insert(COOKIE, value);
            }
        }

        if rule.web_view() {
            let bridge = self
                .webview
                .as_ref()
                .ok_or_else(|| Error::Other(format!("WebView unsupported: {}", request.url)))?;
            let headers = request
                .headers
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.as_str().to_string(), value.to_string()))
                })
                .collect::<BTreeMap<_, _>>();
            let response = bridge
                .fetch(WebViewRequest {
                    url: request.url,
                    web_js: rule.web_js().map(str::to_string),
                    source_regex: source_regex.map(str::to_string),
                    delay_ms: rule.web_view_delay_time().max(0) as u64,
                    headers,
                })
                .await?;
            let text = match source_regex {
                Some(rule) => extract_source_regex(&response.text, rule)?,
                None => response.text,
            };
            if !response.cookies.is_empty() {
                self.cookies
                    .set_cookie(rule.cookie_url(), &response.cookies.join("; "));
            }
            let status = if response.status == 0 {
                http::StatusCode::OK
            } else {
                http::StatusCode::from_u16(response.status).unwrap_or(http::StatusCode::OK)
            };
            let headers = response
                .headers
                .iter()
                .filter_map(|(name, value)| {
                    Some((
                        http::HeaderName::from_bytes(name.as_bytes()).ok()?,
                        http::HeaderValue::from_str(value).ok()?,
                    ))
                })
                .collect();
            return Ok(HttpResponse {
                url: response.url,
                status,
                headers,
                body: text.clone().into(),
                text,
            });
        }

        let include_user_agent = !request_disables_user_agent(&request);
        let client = self.client_for(rule.proxy(), rule.cookie_jar(), include_user_agent)?;
        self.send_retry_with(&client, request, rule.retry()).await
    }

    async fn rate_guard<'a>(
        &'a self,
        key: &'a str,
        rate: Option<&str>,
    ) -> Option<crate::rate::RateGuard<'a>> {
        self.rate.acquire(key, rate).await
    }

    async fn send_retry_with(
        &self,
        client: &reqwest::Client,
        request: HttpRequest,
        retry: u32,
    ) -> Result<HttpResponse> {
        let mut last = None;
        for attempt in 0..=retry {
            if attempt > 0 {
                tracing::debug!("第 {attempt} 次重试 {}", request.url);
            }
            let response = self.send_with(client, request.clone()).await?;
            if response.status.is_success() {
                return Ok(response);
            }
            last = Some(response);
        }
        Ok(last.expect("至少发过一次请求"))
    }

    async fn send_with(
        &self,
        client: &reqwest::Client,
        request: HttpRequest,
    ) -> Result<HttpResponse> {
        let response = self.send_request(client, &request).await?;
        match self
            .read_response(response, request.charset.as_deref())
            .await
        {
            Ok(response) => Ok(response),
            Err(error) if is_body_decode_error(&error) => {
                let mut fallback = request;
                fallback
                    .headers
                    .insert(ACCEPT_ENCODING, http::HeaderValue::from_static("identity"));
                let response = self.send_request(client, &fallback).await?;
                self.read_response(response, fallback.charset.as_deref())
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn send_request(
        &self,
        client: &reqwest::Client,
        request: &HttpRequest,
    ) -> Result<reqwest::Response> {
        match self.send_request_once(client, request).await {
            Ok(response) => Ok(response),
            Err(error) if is_connection_failure(&error) => {
                self.send_request_once(client, request).await
            }
            Err(error) => Err(error),
        }
    }

    async fn send_request_once(
        &self,
        client: &reqwest::Client,
        request: &HttpRequest,
    ) -> Result<reqwest::Response> {
        let method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
            .map_err(|error| Error::Http(format!("?????????: {error}")))?;
        let mut builder = client.request(method, &request.url);
        for (name, value) in request.headers.iter() {
            if name == USER_AGENT
                && value
                    .to_str()
                    .is_ok_and(|text| text.eq_ignore_ascii_case("null"))
            {
                continue;
            }
            builder = builder.header(name, value);
        }
        if let Some(body) = request.body.as_ref() {
            builder = builder.body(body.clone());
        }
        builder
            .send()
            .await
            .map_err(|error| Error::Http(format!("{} ???????: {error}", request.url)))
    }

    async fn read_response(
        &self,
        response: reqwest::Response,
        charset: Option<&str>,
    ) -> Result<HttpResponse> {
        let url = response.url().to_string();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Error::Http(format!("{url} 读响应体失败: {error}")))?;
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let encoding = encode::resolve_encoding(&bytes, content_type.as_deref(), charset);
        let text = encode::decode(&bytes, encoding);
        Ok(HttpResponse {
            url,
            status,
            headers,
            body: bytes,
            text,
        })
    }
}

fn request_disables_user_agent(request: &HttpRequest) -> bool {
    request
        .headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("null"))
}
fn is_connection_failure(error: &Error) -> bool {
    match error {
        Error::Http(message)
            if !message.contains("timed out")
                && !message.contains("timeout")
                && (message.contains("error sending request")
                    || message.contains("connection")
                    || message.contains("broken pipe")
                    || message.contains("connection reset")) =>
        {
            true
        }
        _ => false,
    }
}
fn is_body_decode_error(error: &Error) -> bool {
    matches!(error, Error::Http(message) if message.contains("error decoding response body"))
}

fn parse_proxy(proxy: &str) -> Result<reqwest::Proxy> {
    let text = proxy.trim();
    let text = if text.contains("://") {
        text.to_string()
    } else {
        format!("http://{text}")
    };
    let Some((scheme, rest)) = text.split_once("://") else {
        return reqwest::Proxy::all(&text)
            .map_err(|error| Error::Http(format!("proxy `{proxy}` invalid: {error}")));
    };
    let at_count = rest.matches('@').count();
    let normalized = if at_count == 2 {
        let Some((host_port, auth)) = rest.split_once('@') else {
            return reqwest::Proxy::all(&text)
                .map_err(|error| Error::Http(format!("proxy `{proxy}` invalid: {error}")));
        };
        let Some((user, password)) = auth.split_once('@') else {
            return reqwest::Proxy::all(&text)
                .map_err(|error| Error::Http(format!("proxy `{proxy}` invalid: {error}")));
        };
        let mut url = ::url::Url::parse(&format!("{scheme}://{host_port}"))
            .map_err(|error| Error::Http(format!("proxy `{proxy}` invalid: {error}")))?;
        url.set_username(user)
            .map_err(|_| Error::Http(format!("proxy `{proxy}` username invalid")))?;
        url.set_password(Some(password))
            .map_err(|_| Error::Http(format!("proxy `{proxy}` password invalid")))?;
        url.to_string()
    } else {
        text
    };
    reqwest::Proxy::all(&normalized)
        .map_err(|error| Error::Http(format!("proxy `{proxy}` invalid: {error}")))
}
fn pick_client(pair: &ClientPair, cookie_jar: bool, include_user_agent: bool) -> reqwest::Client {
    match (cookie_jar, include_user_agent) {
        (true, true) => pair.jar.clone(),
        (true, false) => pair.jar_no_ua.clone(),
        (false, true) => pair.plain.clone(),
        (false, false) => pair.plain_no_ua.clone(),
    }
}

impl HttpFetcher for HttpClient {
    fn fetch(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse>> {
        Box::pin(async move { self.send(request).await })
    }

    fn fetch_limited(
        &self,
        request: HttpRequest,
        rate: Option<String>,
        rate_key: String,
    ) -> BoxFuture<'_, Result<HttpResponse>> {
        Box::pin(async move { self.send_js(request, rate.as_deref(), &rate_key).await })
    }
}

pub async fn check_login(
    js: &str,
    response: HttpResponse,
    ctx: &UrlContext<'_>,
) -> Result<HttpResponse> {
    if js.trim().is_empty() {
        return Ok(response);
    }
    let Some(evaluator) = ctx.js else {
        return Err(Error::Js("loginCheckJs 需要 JsEvaluator（M3 接上）".into()));
    };
    evaluator
        .eval_response(JsRequest {
            script: js,
            source: ctx.source,
            base_url: ctx.base_url,
            page: ctx.page,
            key: ctx.key,
            result: Some(&response.text),
            result_bytes: None,
            src: None,
            response: Some(&response),
            book: None,
            chapter: None,
            variables: None,
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kotlin_proxy_format() {
        assert!(parse_proxy("http://127.0.0.1:8080@user@pass").is_ok());
        assert!(parse_proxy("socks5://127.0.0.1:1080").is_ok());
    }

    #[tokio::test]
    async fn post_form_body_uses_form_content_type() {
        let ctx = UrlContext::new("http://example.com");
        let rule = UrlRule::parse(
            r#"http://example.com,{"method":"POST","body":"a=1&b=2"}"#,
            &ctx,
        )
        .await
        .unwrap();
        let request = rule.request();
        assert_eq!(
            request.headers.get(CONTENT_TYPE).unwrap(),
            "application/x-www-form-urlencoded"
        );
    }

    #[tokio::test]
    async fn parses_percent_encoded_url_options() {
        let ctx = UrlContext::new("http://example.com");
        let rule = UrlRule::parse(
            "http://example.com/api,%7B%22method%22%3A%22POST%22%2C%22body%22%3A%22a%3D1%22%7D",
            &ctx,
        )
        .await
        .unwrap();
        assert_eq!(rule.method(), http::Method::POST);
        assert_eq!(rule.request().body.as_deref(), Some(b"a=1".as_slice()));
    }

    #[tokio::test]
    async fn post_json_body_uses_json_content_type() {
        let ctx = UrlContext::new("http://example.com");
        let rule = UrlRule::parse(
            r#"http://example.com,{"method":"POST","body":"{\"a\":1}"}"#,
            &ctx,
        )
        .await
        .unwrap();
        let request = rule.request();
        assert_eq!(
            request.headers.get(CONTENT_TYPE).unwrap(),
            "application/json; charset=UTF-8"
        );
    }
    struct FakeWebView;

    impl WebViewBridge for FakeWebView {
        fn fetch<'a>(
            &'a self,
            request: WebViewRequest,
        ) -> engine_model::BoxFuture<'a, Result<engine_model::WebViewResponse>> {
            Box::pin(async move {
                assert_eq!(request.web_js.as_deref(), Some("document.title"));
                Ok(engine_model::WebViewResponse {
                    url: request.url,
                    status: 200,
                    headers: BTreeMap::new(),
                    text: "rendered".to_string(),
                    cookies: vec!["sid=1".to_string()],
                })
            })
        }
    }

    #[tokio::test]
    async fn webview_bridge_handles_webview_rule() {
        let client = HttpClient::builder()
            .webview(Arc::new(FakeWebView))
            .build()
            .unwrap();
        let ctx = UrlContext::new("https://example.com");
        let rule = UrlRule::parse(
            r#"https://example.com/book,{"webView":true,"webJs":"document.title"}"#,
            &ctx,
        )
        .await
        .unwrap();
        let response = client.send_rule(&rule).await.unwrap();
        assert_eq!(response.text, "rendered");
        assert_eq!(client.cookies().get_cookie("https://example.com"), "sid=1");
    }

    #[tokio::test]
    async fn webview_rule_without_bridge_is_unsupported() {
        let client = HttpClient::builder().build().unwrap();
        let ctx = UrlContext::new("https://example.com");
        let rule = UrlRule::parse(r#"https://example.com/book,{"webView":true}"#, &ctx)
            .await
            .unwrap();
        let error = client.send_rule(&rule).await.unwrap_err().to_string();
        assert!(error.contains("WebView unsupported"), "{error}");
    }

    #[test]
    fn detects_user_agent_null() {
        let mut request = HttpRequest::get("https://example.com");
        request
            .headers
            .insert(USER_AGENT, http::HeaderValue::from_static("null"));
        assert!(request_disables_user_agent(&request));
    }
}
fn extract_source_regex(text: &str, rule: &str) -> Result<String> {
    let regex = regex::Regex::new(rule)
        .map_err(|error| Error::Other(format!("sourceRegex invalid: {error}")))?;
    let value = regex
        .captures(text)
        .and_then(|captures| captures.get(1).or_else(|| captures.get(0)))
        .map(|value| value.as_str().to_string())
        .unwrap_or_default();
    Ok(value)
}
