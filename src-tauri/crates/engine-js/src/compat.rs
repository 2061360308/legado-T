//! hadc188-compatible Java standard-library shim.
//!
//! `java_compat.js` is copied from `hadc188/reader`. It expects a small set of
//! native helpers (`__binary`, `__str_to_bytes`, `__bytes_to_str`, `__sleep`),
//! which are provided here on top of the existing `engine-js` crypto/encoding
//! primitives.

use std::io::Read;

use base64::Engine as _;
use rand::RngCore;
use rquickjs::function::Opt;
use rquickjs::{Ctx, Function};
use serde_json::{Value, json};

use crate::crypto::{self, Symmetric};

pub(crate) fn install(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    globals.set(
        "__binary",
        Function::new(ctx.clone(), |op: String, raw: String| -> String {
            let result = serde_json::from_str(&raw)
                .map_err(|error| error.to_string())
                .and_then(|payload| run(&op, &payload));
            match result {
                Ok(value) => json!({ "value": value }).to_string(),
                Err(error) => json!({ "error": error }).to_string(),
            }
        })?,
    )?;

    globals.set(
        "__sleep",
        Function::new(ctx.clone(), |millis: f64| {
            if millis.is_finite() && millis > 0.0 {
                std::thread::sleep(std::time::Duration::from_millis(millis.min(60_000.0) as u64));
            }
        })?,
    )?;

    globals.set(
        "__str_to_bytes",
        Function::new(
            ctx.clone(),
            |text: String, charset: Opt<String>| -> String {
                let charset = charset
                    .0
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "UTF-8".to_string());
                let bytes = encode_charset(&text, &charset).unwrap_or_else(|_| text.into_bytes());
                serde_json::to_string(&bytes).unwrap_or_else(|_| "[]".to_string())
            },
        )?,
    )?;

    globals.set(
        "__bytes_to_str",
        Function::new(ctx.clone(), |raw: String, charset: Opt<String>| -> String {
            let charset = charset
                .0
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "UTF-8".to_string());
            let bytes = serde_json::from_str::<Vec<i64>>(&raw)
                .map(|values| {
                    values
                        .into_iter()
                        .map(|value| value as u8)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            crypto::decode_charset(&bytes, Some(&charset))
        })?,
    )?;

    ctx.eval::<rquickjs::Value, _>(include_str!("java_compat.js"))?;
    ctx.eval::<rquickjs::Value, _>(
        r#"
(function () {
  const define = (name, value) => {
    if (!String.prototype[name]) {
      Object.defineProperty(String.prototype, name, {
        value,
        writable: true,
        configurable: true
      });
    }
  };
  define("text", function () {
    return this.replace(/<script[\s\S]*?<\/script>/gi, "")
      .replace(/<style[\s\S]*?<\/style>/gi, "")
      .replace(/<[^>]*>/g, "")
      .replace(/&nbsp;/gi, " ")
      .replace(/&lt;/gi, "<")
      .replace(/&gt;/gi, ">")
      .replace(/&amp;/gi, "&")
      .trim();
  });
  define("ownText", function () { return this.text(); });
  define("html", function () { return String(this); });
  define("outerHtml", function () { return String(this); });
  define("attr", function (name) {
    const pattern = new RegExp("\\s" + String(name).replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\s*=\\s*(?:\"([^\"]*)\"|'([^']*)'|([^\\s>]+))", "i");
    const match = String(this).match(pattern);
    return match ? (match[1] ?? match[2] ?? match[3] ?? "") : "";
  });
})();
"#,
    )?;
    Ok(())
}

fn run(operation: &str, payload: &Value) -> Result<Value, String> {
    let text = |key: &str| payload[key].as_str().unwrap_or("");
    match operation {
        "encode" => {
            let charset = if text("charset").is_empty() {
                "UTF-8"
            } else {
                text("charset")
            };
            Ok(output(encode_charset(text("text"), charset)?))
        }
        "decode" => {
            let charset = if text("charset").is_empty() {
                "UTF-8"
            } else {
                text("charset")
            };
            Ok(json!(crypto::decode_charset(
                &bytes(&payload["data"])?,
                Some(charset)
            )))
        }
        "base64Encode" => Ok(json!(
            base64::engine::general_purpose::STANDARD.encode(bytes(&payload["data"])?)
        )),
        "base64Decode" => {
            let text = if text("text").is_empty() {
                String::from_utf8_lossy(&bytes(&payload["data"])?).into_owned()
            } else {
                text("text").to_string()
            };
            let decoded = crate::java::decode_base64(&text)
                .ok_or_else(|| format!("base64Decode 失败：{text}"))?;
            Ok(output(decoded))
        }
        "digest" => {
            let data = bytes(&payload["data"])?;
            let algorithm = text("algorithm").replace('-', "").to_ascii_uppercase();
            Ok(output(crypto::digest_bytes(&algorithm, &data)?))
        }
        "mac" => {
            let data = bytes(&payload["data"])?;
            let key = bytes(&payload["key"])?;
            Ok(output(crypto::hmac_bytes(text("algorithm"), &key, &data)?))
        }
        "random" => {
            let length = payload["length"]
                .as_u64()
                .filter(|length| *length <= 16 * 1024 * 1024)
                .ok_or_else(|| "无效数组长度".to_string())? as usize;
            let mut data = vec![0u8; length];
            rand::rngs::OsRng.fill_bytes(&mut data);
            Ok(output(data))
        }
        "cipher" => {
            let (algorithm, mode, padding) = crypto::parse_transformation(text("transformation"))?;
            let symmetric = Symmetric::new(
                algorithm,
                mode,
                padding,
                bytes(&payload["key"])?,
                bytes(&payload["iv"])?,
            )?;
            let data = bytes(&payload["data"])?;
            let out = if payload["encrypt"].as_bool().unwrap_or(false) {
                symmetric.encrypt(&data)?
            } else {
                symmetric.decrypt(&data)?
            };
            Ok(output(out))
        }
        "gunzip" => {
            let data = bytes(&payload["data"])?;
            let mut decoder =
                flate2::read::GzDecoder::new(data.as_slice()).take(64 * 1024 * 1024 + 1);
            let mut result = Vec::new();
            decoder
                .read_to_end(&mut result)
                .map_err(|error| error.to_string())?;
            if result.len() > 64 * 1024 * 1024 {
                return Err("解压内容过大".into());
            }
            Ok(output(result))
        }
        "inflate" => {
            let data = bytes(&payload["data"])?;
            let nowrap = payload["nowrap"].as_bool().unwrap_or(true);
            let mut result = Vec::new();
            if nowrap {
                let mut decoder =
                    flate2::read::DeflateDecoder::new(data.as_slice()).take(64 * 1024 * 1024 + 1);
                decoder
                    .read_to_end(&mut result)
                    .map_err(|error| error.to_string())?;
            } else {
                let mut decoder =
                    flate2::read::ZlibDecoder::new(data.as_slice()).take(64 * 1024 * 1024 + 1);
                decoder
                    .read_to_end(&mut result)
                    .map_err(|error| error.to_string())?;
            }
            if result.len() > 64 * 1024 * 1024 {
                return Err("解压内容过大".into());
            }
            Ok(output(result))
        }
        "gzip" => {
            use std::io::Write;
            let data = bytes(&payload["data"])?;
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&data)
                .map_err(|error| error.to_string())?;
            Ok(output(encoder.finish().map_err(|error| error.to_string())?))
        }
        other => Err(format!("不支持字节操作：{other}")),
    }
}

fn bytes(value: &Value) -> Result<Vec<u8>, String> {
    value
        .as_array()
        .ok_or_else(|| "需要字节数组".to_string())?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .map(|value| value as u8)
                .ok_or_else(|| "无效字节".to_string())
        })
        .collect()
}

fn output(bytes: Vec<u8>) -> Value {
    json!(bytes.into_iter().map(|byte| byte as i8).collect::<Vec<_>>())
}

fn encode_charset(text: &str, charset: &str) -> Result<Vec<u8>, String> {
    let charset = charset.trim().replace("_", "-");
    let encoding = encoding_rs::Encoding::for_label(charset.as_bytes())
        .ok_or_else(|| format!("不支持字符编码：{charset}"))?;
    Ok(encoding.encode(text).0.into_owned())
}
