use engine_native::pipeline::parse_rss;

#[test]
fn parses_default_rss() {
    let xml = r#"<?xml version="1.0"?>
        <rss><channel>
          <item>
            <title>Article</title>
            <link>https://example.com/a</link>
            <description><![CDATA[<p>Hello</p>]]></description>
            <pubDate>Wed, 01 Jan 2026 00:00:00 GMT</pubDate>
          </item>
        </channel></rss>"#;
    let articles = parse_rss("Mock", xml, "https://example.com/rss");
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].title, "Article");
    assert_eq!(articles[0].link, "https://example.com/a");
    assert_eq!(articles[0].origin, "https://example.com/rss");
}
