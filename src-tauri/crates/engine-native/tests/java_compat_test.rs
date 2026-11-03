use engine_native::eval_text;

async fn eval(script: &str) -> String {
    eval_text(&format!("<js>{script}</js>"), "")
        .await
        .unwrap_or_else(|error| panic!("{script}\n{error}"))
}

#[tokio::test]
async fn supports_java_math_biginteger() {
    assert_eq!(
        eval(r#"new java.math.BigInteger("ff", 16).toString(10)"#).await,
        "255"
    );
}

#[tokio::test]
async fn supports_java_util_arrays_and_base64() {
    assert_eq!(
        eval("java.util.Arrays.toString([1,2,3])").await,
        "[1, 2, 3]"
    );
    assert_eq!(
        eval(r#"java.util.Base64.getEncoder().encodeToString("abc")"#).await,
        "YWJj"
    );
}

#[tokio::test]
async fn supports_android_base64() {
    assert_eq!(
        eval(r#"Packages.android.util.Base64.encodeToString("abc", 0)"#).await,
        "YWJj"
    );
}

#[tokio::test]
async fn supports_java_nio_byte_buffer() {
    assert_eq!(
        eval("java.util.Arrays.toString(java.nio.ByteBuffer.allocate(4).putInt(1).array())").await,
        "[0, 0, 0, 1]"
    );
}

#[tokio::test]
async fn supports_java_net_url_encoder() {
    assert_eq!(
        eval(r#"java.net.URLEncoder.encode("a b", "UTF-8")"#).await,
        "a+b"
    );
}

#[tokio::test]
async fn supports_java_nio_charset() {
    assert_eq!(
        eval(r#"java.nio.charset.Charset.forName("UTF-8").name()"#).await,
        "UTF-8"
    );
}

#[tokio::test]
async fn supports_java_security_message_digest() {
    assert_eq!(
        eval(r#"java.util.Base64.getEncoder().encodeToString(java.security.MessageDigest.getInstance("MD5").digest("abc"))"#).await,
        "kAFQmDzST7DWlj99KOF/cg=="
    );
}

#[tokio::test]
async fn supports_javax_crypto_hmac() {
    assert_eq!(
        eval(r#"(function(){var m=Packages.javax.crypto.Mac.getInstance("HmacSHA256");m.init(new Packages.javax.crypto.spec.SecretKeySpec("key","HmacSHA256"));return java.util.Base64.getEncoder().encodeToString(m.doFinal("abc"));})()"#).await,
        "nBluMtwBdfhvSxy4konWYZ3mvuaZ5MN45oMJ7Zehpqs="
    );
}

#[tokio::test]
async fn supports_javax_crypto_aes() {
    assert_eq!(
        eval(r#"(function(){var c=Packages.javax.crypto.Cipher.getInstance("AES/ECB/PKCS5Padding");c.init(1,new Packages.javax.crypto.spec.SecretKeySpec("1234567890123456","AES"));return java.util.Base64.getEncoder().encodeToString(c.doFinal("hello"));})()"#).await,
        "67fHA+Z12z2jlwOLTBeCPA=="
    );
}

#[tokio::test]
async fn supports_hutool_str_util_base64_and_gzip() {
    assert_eq!(
        eval(r#"Packages.cn.hutool.core.util.StrUtil.reverse("abc")"#).await,
        "cba"
    );
    assert_eq!(
        eval(r#"Packages.cn.hutool.core.util.StrUtil.isEmpty("")"#).await,
        "true"
    );
    assert_eq!(
        eval(r#"Packages.cn.hutool.core.util.StrUtil.isBlank(" ")"#).await,
        "true"
    );
    assert_eq!(
        eval(r#"Packages.cn.hutool.core.util.StrUtil.format("a{}b", 1)"#).await,
        "a1b"
    );
    assert_eq!(
        eval(r#"Packages.cn.hutool.core.codec.Base64.encode("abc")"#).await,
        "YWJj"
    );
    assert_eq!(
        eval(r#"java.bytesToStr(Packages.cn.hutool.core.codec.Base64.decode("YWJj"))"#).await,
        "abc"
    );
    assert_eq!(
        eval(r#"java.bytesToStr(Packages.cn.hutool.core.util.ZipUtil.unGzip(Packages.cn.hutool.core.util.ZipUtil.gzip("abc", "UTF-8")))"#).await,
        "abc"
    );
}

#[tokio::test]
async fn supports_okhttp3_compat_classes() {
    assert_eq!(
        eval(r#"typeof Packages.okhttp3.OkHttpClient"#).await,
        "function"
    );
    assert_eq!(
        eval(r#"new Packages.okhttp3.Request.Builder().url("https://example.com").get().build().url"#).await,
        "https://example.com"
    );
    assert_eq!(
        eval(r#"Packages.okhttp3.MediaType.parse("application/json").__mediaType"#).await,
        "application/json"
    );
}
