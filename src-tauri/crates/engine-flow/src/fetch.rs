use engine_model::{BoxFuture, HttpResponse, Result};
#[cfg(feature = "native")]
use engine_http_native::HttpClient;
use engine_http_native::UrlRule;

pub trait PageFetcher: Send + Sync {
    fn fetch_rule<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, Result<HttpResponse>>;

    fn fetch_rule_bytes<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, Result<Vec<u8>>> {
        Box::pin(async move {
            self.fetch_rule(rule)
                .await
                .map(|response| response.body.to_vec())
        })
    }

    fn fetch_rule_with_source_regex<'a>(
        &'a self,
        rule: &'a UrlRule,
        source_regex: Option<&'a str>,
    ) -> BoxFuture<'a, Result<HttpResponse>> {
        let _ = source_regex;
        self.fetch_rule(rule)
    }
}

#[cfg(feature = "native")]
impl PageFetcher for HttpClient {
    fn fetch_rule<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move { self.send_rule(rule).await })
    }

    fn fetch_rule_bytes<'a>(&'a self, rule: &'a UrlRule) -> BoxFuture<'a, Result<Vec<u8>>> {
        Box::pin(async move {
            self.fetch_rule(rule)
                .await
                .map(|response| response.body.to_vec())
        })
    }

    fn fetch_rule_with_source_regex<'a>(
        &'a self,
        rule: &'a UrlRule,
        source_regex: Option<&'a str>,
    ) -> BoxFuture<'a, Result<HttpResponse>> {
        Box::pin(async move { self.send_rule_with_source_regex(rule, source_regex).await })
    }
}
