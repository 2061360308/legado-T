use engine_native::upstream::net::{HttpClient, UrlContext, UrlRule};

#[tokio::test]
async fn parses_webview_options() {
    let ctx = UrlContext::new("https://example.com");
    let rule = UrlRule::parse(
        r#"https://example.com/book,{"webView":true,"webJs":"document.body.innerText"}"#,
        &ctx,
    )
    .await
    .expect("parse url rule");
    assert!(rule.web_view());
    assert_eq!(rule.web_js(), Some("document.body.innerText"));
}

#[tokio::test]
async fn webview_fetch_fails_explicitly() {
    let ctx = UrlContext::new("https://example.com");
    let rule = UrlRule::parse(
        r#"https://example.com/book,{"webView":true,"webJs":"return 1"}"#,
        &ctx,
    )
    .await
    .expect("parse url rule");
    let client = HttpClient::new().expect("http client");
    let error = client.send_rule(&rule).await.expect_err("must fail");
    assert!(error.to_string().contains("WebView"), "{error}");
}

#[tokio::test]
async fn false_webview_is_not_treated_as_webview() {
    let ctx = UrlContext::new("https://example.com");
    let rule = UrlRule::parse(r#"https://example.com/book,{"webView":false}"#, &ctx)
        .await
        .expect("parse url rule");
    assert!(!rule.web_view());
}
