use engine_model::{BookChapter, BookSource};
use engine_native::eval_html;
use engine_rule::{Content, EvalContext, RuleEngine};
use std::sync::Arc;

#[tokio::test]
async fn handles_multibyte_self_closing_html() {
    let html = r#"<div class="content"><p>中文内容</p><br/></div>"#;
    let value = eval_html("class.content@html", html).await.unwrap();
    assert!(value.contains("中文内容"), "{value}");
}

#[tokio::test]
async fn supports_leading_attribute_js_step() {
    let html = r#"<div onclick="hello">x</div>"#;
    let values = engine_native::eval_list("@onclick@js:result", html)
        .await
        .unwrap();
    assert_eq!(values, vec!["hello"]);
}

#[tokio::test]
async fn supports_double_quoted_xpath_literals() {
    let html = r#"<div id="list"><dl><dt>x</dt><dd><a href="/1">A</a></dd><dd><a href="/2">B</a></dd></dl></div>"#;
    let values = engine_native::eval_list(r#"//*[@id="list"]//dd/a/text()"#, html)
        .await
        .unwrap();
    assert_eq!(values, vec!["A", "B"]);
}

#[tokio::test]
async fn supports_double_quoted_xpath_following_sibling() {
    let html = r#"<div id="list"><dl><dt>old</dt><dd><a href="/0">old</a></dd><dt>new</dt><dd><a href="/1">A</a></dd><dd><a href="/2">B</a></dd></dl></div>"#;
    let values = engine_native::eval_list(
        r#"//*[@id="list"]//dt[2]/following-sibling::dd/a/text()"#,
        html,
    )
    .await
    .unwrap();
    assert_eq!(values, vec!["A", "B"]);
}

#[tokio::test]
async fn xpath_list_rules_keep_element_html() {
    let html = r#"<div id="list"><dl><dt>old</dt><dd><a href="/0">old</a></dd><dt>new</dt><dd><a href="/1">A</a></dd><dd><a href="/2">B</a></dd></dl></div>"#;
    let engine = RuleEngine::new();
    let mut ctx = EvalContext::new(Content::html(html));
    let values = engine
        .eval(
            r#"//*[@id="list"]//dt[2]/following-sibling::dd/a"#,
            &mut ctx,
        )
        .await
        .unwrap()
        .into_list();
    assert_eq!(values.len(), 2);
    assert!(values[0].contains(r#"href="/1""#), "{values:?}");
    assert!(values[1].contains(r#"href="/2""#), "{values:?}");
}

#[tokio::test]
async fn supports_jsoup_positional_pseudos() {
    let html = r#"<div class="list"><p>one</p><p>two</p><p>three</p></div><h2>Title</h2>"#;
    let first = engine_native::eval_list("class.list@tag.p:first-child@text", html)
        .await
        .unwrap();
    let second = engine_native::eval_list("class.list@tag.p:nth-child(2)@text", html)
        .await
        .unwrap();
    let last = engine_native::eval_list("class.list@tag.p:last-child@text", html)
        .await
        .unwrap();
    let header = engine_native::eval_list("tag.h2:header@text", html)
        .await
        .unwrap();
    assert_eq!(first, vec!["one"]);
    assert_eq!(second, vec!["two"]);
    assert_eq!(last, vec!["three"]);
    assert_eq!(header, vec!["Title"]);
}

#[tokio::test]
async fn replaces_title_placeholder_without_js() {
    let engine = RuleEngine::new();
    let mut chapter = BookChapter {
        title: "第一章".to_string(),
        ..Default::default()
    };
    let mut ctx = EvalContext::new(Content::html(r#"<div class="content">第一章 正文</div>"#))
        .with_chapter(&mut chapter);
    let value = engine
        .eval_string(r#"class.content@text##{{title}}"#, &mut ctx)
        .await
        .unwrap();
    assert_eq!(value.trim(), "正文");
}

#[tokio::test]
async fn java_des_helpers_round_trip() {
    let value = engine_native::eval_text(
        r#"@js:
let key = "12345678";
let encrypted = java.desEncodeToBase64String("hello", key, "DES/ECB/PKCS5Padding", "");
java.desBase64DecodeToString(encrypted, key, "DES/ECB/PKCS5Padding", "")
"#,
        "",
    )
    .await
    .unwrap();
    assert_eq!(value, "hello");
}

#[tokio::test]
async fn java_triple_des_helper_round_trip() {
    let value = engine_native::eval_text(
        r#"@js:
let key = "123456789012345678901234";
let iv = "12345678";
let encrypted = java.tripleDESEncodeBase64Str("hello", key, "CBC", "PKCS5Padding", iv);
java.createSymmetricCrypto("DESede/CBC/PKCS5Padding", key, iv).decryptStr(encrypted)
"#,
        "",
    )
    .await
    .unwrap();
    assert_eq!(value, "hello");
}

#[tokio::test]
async fn java_compat_helpers_are_available() {
    let value = engine_native::eval_text(
        r#"@js:
let ua = java.getWebViewUA();
let decoded = java.base64Decoder("aGVsbG8=");
ua.length > 0 && decoded === "hello" ? "ok" : "bad"
"#,
        "",
    )
    .await
    .unwrap();
    assert_eq!(value, "ok");
}

#[tokio::test]
async fn java_list_items_expose_element_methods() {
    let html =
        r#"<div class="item"><a href="/1">A</a></div><div class="item"><a href="/2">B</a></div>"#;
    let values = engine_native::eval_list(
        r#"class.item@js:result.toArray().map(x => x.text() + "|" + x.attr("href"))"#,
        html,
    )
    .await
    .unwrap();
    assert_eq!(values, vec!["A|/1", "B|/2"]);
}

#[tokio::test]
async fn keeps_table_fragment_context() {
    let engine = RuleEngine::new();
    let mut ctx = EvalContext::new(Content::fragment(
        r#"<tr class="book"><td class="name"><a href="/txt/1.html">书名</a></td></tr>"#,
    ));
    let value = engine
        .eval_string("class.name@tag.a@href", &mut ctx)
        .await
        .unwrap();
    assert_eq!(value, "/txt/1.html");
}

#[tokio::test]
async fn keeps_tbody_fragment_context() {
    let engine = RuleEngine::new();
    let mut ctx = EvalContext::new(Content::fragment(
        r#"<tbody><tr><td class="name"><a href="/xs/1.html">书名</a></td></tr></tbody>"#,
    ));
    let value = engine
        .eval_string("class.name@tag.a@href", &mut ctx)
        .await
        .unwrap();
    assert_eq!(value, "/xs/1.html");
}

#[tokio::test]
async fn java_put_persists_across_evaluations() {
    let source: BookSource = serde_json::from_str(r#"{"bookSourceUrl":"test"}"#).unwrap();
    let js = Arc::new(engine_js::JsEngine::new().unwrap());
    let engine = RuleEngine::new();
    engine.set_js(js.clone());
    let mut first = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());
    engine
        .eval_string("@js:java.put('shared', 'value'); 'ok'", &mut first)
        .await
        .unwrap();
    let mut second = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());
    let value = engine
        .eval_string("@js:java.get('shared')", &mut second)
        .await
        .unwrap();
    assert_eq!(value, "value");
}

#[tokio::test]
async fn js_lib_errors_include_the_real_js_message_and_stack() {
    let source: BookSource = serde_json::from_str(
        r#"{"bookSourceUrl":"https://example.com","jsLib":"throw new Error('boom-js-lib')"}"#,
    )
    .unwrap();
    let js = Arc::new(engine_js::JsEngine::new().unwrap());
    let engine = RuleEngine::new();
    engine.set_js(js.clone());
    let mut ctx = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());

    let error = engine
        .eval_string("@js:1", &mut ctx)
        .await
        .unwrap_err()
        .to_string();

    assert!(error.contains("jsLib 执行失败"), "{error}");
    assert!(error.contains("boom-js-lib"), "{error}");
    assert!(error.contains('\n'), "expected a stack, got: {error}");
}

#[tokio::test]
async fn js_lib_errors_name_the_missing_package() {
    let source: BookSource = serde_json::from_str(
        r#"{"bookSourceUrl":"https://example.com","jsLib":"Packages.missing"}"#,
    )
    .unwrap();
    let js = Arc::new(engine_js::JsEngine::new().unwrap());
    let engine = RuleEngine::new();
    engine.set_js(js.clone());
    let mut ctx = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());

    let error = engine
        .eval_string("@js:1", &mut ctx)
        .await
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("Packages 里没有注册 Packages.missing"),
        "{error}"
    );
}

#[tokio::test]
async fn tomato_js_lib_supports_hutool_and_okhttp_compat() {
    let source: BookSource = serde_json::from_value(serde_json::json!({
        "bookSourceUrl": "https://reading.snssdk.com",
        "bookSourceName": "tomato",
        "jsLib": include_str!("fixtures/tomato_js_lib.js"),
    }))
    .unwrap();
    let js = Arc::new(engine_js::JsEngine::new().unwrap());
    let engine = RuleEngine::new();
    engine.set_js(js.clone());
    let mut ctx = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());

    let value = engine
        .eval_string(
            r#"@js:[
                typeof OkHttpClient,
                StrUtil.reverse("abc"),
                Packages.cn.hutool.core.codec.Base64.encode("abc"),
                DigestUtil.md5Hex("abc"),
                typeof ZipUtil.gzip
            ].join("|")"#,
            &mut ctx,
        )
        .await
        .unwrap();

    assert_eq!(
        value,
        "function|cba|YWJj|900150983cd24fb0d6963f7d28e17f72|function"
    );
}

#[tokio::test]
async fn tomato_xgorgon_without_body_returns_a_signed_url() {
    let source: BookSource = serde_json::from_value(serde_json::json!({
        "bookSourceUrl": "https://reading.snssdk.com",
        "bookSourceName": "tomato",
        "jsLib": include_str!("fixtures/tomato_js_lib.js"),
    }))
    .unwrap();
    let js = Arc::new(engine_js::JsEngine::new().unwrap());
    let engine = RuleEngine::new();
    engine.set_js(js.clone());
    let mut ctx = EvalContext::new(Content::text(""))
        .with_source(&source)
        .with_js(js.as_ref());

    let value = engine
        .eval_string(
            r#"@js:xGorgon("new_category/front", "a=1", null, "")"#,
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(value.contains("reading.snssdk.com"), "{value}");
    assert!(value.contains("X-Gorgon"), "{value}");
    assert!(value.contains("X-Khronos"), "{value}");
}

#[tokio::test]
async fn bare_json_member_rule_reads_object_field() {
    let engine = RuleEngine::new();
    let mut ctx = EvalContext::new(Content::Json(serde_json::json!({
        "name": "chapter",
        "url": "/1.html"
    })));
    let value = engine.eval_string("name", &mut ctx).await.unwrap();
    assert_eq!(value, "chapter");
}

#[tokio::test]
async fn js_object_array_keeps_json_items() {
    let values = engine_native::eval_list("@js: [{name:'a'},{name:'b'}]", "{}")
        .await
        .unwrap();
    assert_eq!(values, vec![r#"{"name":"a"}"#, r#"{"name":"b"}"#]);
}

#[tokio::test]
async fn java_importer_exposes_string_bytes_and_base64() {
    let value = engine_native::eval_text(
        r#"@js:
var importer = new JavaImporter();
importer.importPackage(Packages.java.lang, Packages.java.util, Packages.javax.crypto.spec);
[typeof SecretKeySpec, String('x').getBytes().length, typeof Base64.getDecoder].join('|')
"#,
        "{}",
    )
    .await
    .unwrap();
    assert_eq!(value, "function|1|function");
}
