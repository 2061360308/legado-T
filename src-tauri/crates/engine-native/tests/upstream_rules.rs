use engine_native::{eval_html, eval_list, eval_text};

const HTML: &str = r#"<html><body>
<div class="book">
  <a href="/1">One</a>
  <a href="/2">Two</a>
</div>
</body></html>"#;

#[tokio::test]
async fn evaluates_css_rule() {
    let links = eval_list("class.book@tag.a@href", HTML).await.unwrap();
    assert_eq!(links, vec!["/1", "/2"]);
}

#[tokio::test]
async fn evaluates_xpath_rule() {
    let link = eval_html("//div[@class='book']/a[1]/@href", HTML)
        .await
        .unwrap();
    assert_eq!(link, "/1");
}

#[tokio::test]
async fn evaluates_jsonpath_rule() {
    let json = r#"{"data":{"books":[{"name":"One"},{"name":"Two"}]}}"#;
    let names = eval_text("$.data.books[*].name", json).await.unwrap();
    assert!(names.contains("One"));
    assert!(names.contains("Two"));
}

#[tokio::test]
async fn evaluates_regex_rule() {
    let value = eval_text(
        ":href=\"(\\d+)\"##href=\"(\\d+)\"##$1",
        r#"<a href="123">x</a>"#,
    )
    .await
    .unwrap();
    assert_eq!(value, "123");
}

#[tokio::test]
async fn evaluates_js_rule() {
    let value = eval_text("<js>1+2</js>", "").await.unwrap();
    assert_eq!(value, "3.0");
}
