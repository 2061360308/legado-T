use std::rc::Rc;

use rand::Rng as _;
use rquickjs::function::Opt;
use rquickjs::{Ctx, Function, Object, Value as JsValue};

use crate::crypto::{self, Algorithm};
use crate::host::Host;
use crate::jsoup;

pub(crate) fn install(ctx: &Ctx<'_>, host: &Rc<Host>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    globals.set("__legado_aes", crypto::class(ctx, Algorithm::Aes)?)?;
    globals.set("__legado_des", crypto::class(ctx, Algorithm::Des)?)?;
    globals.set("__legado_desede", crypto::class(ctx, Algorithm::DesEde)?)?;
    globals.set("__legado_digest_util", crypto::digest_util(ctx)?)?;
    globals.set("__legado_rsa", crate::asymmetric::rsa_class(ctx)?)?;
    globals.set("__legado_key_type", crate::asymmetric::key_type(ctx)?)?;
    globals.set("__legado_sign", crate::asymmetric::sign_class(ctx)?)?;
    globals.set("__legado_javax_mac", crate::javax::mac(ctx)?)?;
    globals.set("__legado_javax_cipher", crate::javax::cipher(ctx)?)?;
    globals.set("__legado_key_spec", crate::javax::key_spec(ctx)?)?;
    globals.set("__legado_iv_spec", crate::javax::key_spec(ctx)?)?;
    globals.set("__legado_datatype", crate::javax::datatype_converter(ctx)?)?;
    globals.set("__legado_random_util", random_util(ctx)?)?;
    globals.set("__legado_zip_util", zip_util(ctx)?)?;
    globals.set("__legado_url_decoder", url_codec(ctx, false)?)?;
    globals.set("__legado_url_encoder", url_codec(ctx, true)?)?;
    globals.set("__engine_jsoup", jsoup::class(ctx, host)?)?;
    globals.set("__legado_java_lang_system", java_lang_system(ctx)?)?;
    globals.set("__legado_java_lang_thread", java_lang_thread(ctx)?)?;
    ctx.eval::<rquickjs::Value, _>(TREE)?;
    for name in [
        "__legado_aes",
        "__legado_des",
        "__legado_desede",
        "__legado_digest_util",
        "__legado_rsa",
        "__legado_key_type",
        "__legado_sign",
        "__legado_javax_mac",
        "__legado_javax_cipher",
        "__legado_key_spec",
        "__legado_iv_spec",
        "__legado_datatype",
        "__legado_random_util",
        "__legado_zip_util",
        "__legado_url_decoder",
        "__legado_url_encoder",
        "__engine_jsoup",
        "__legado_java_lang_system",
        "__legado_java_lang_thread",
    ] {
        let _ = globals.remove(name);
    }
    Ok(())
}

fn java_lang_system<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.prop(
        "currentTimeMillis",
        Function::new(ctx.clone(), || -> i64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_millis() as i64)
                .unwrap_or_default()
        })?,
    )?;
    object.prop("nanoTime", Function::new(ctx.clone(), || -> i64 { 0 })?)?;
    object.prop(
        "getProperty",
        Function::new(ctx.clone(), |_name: Opt<String>| -> String {
            String::new()
        })?,
    )?;
    for (name, to_stderr) in [("out", false), ("err", true)] {
        let stream = Object::new(ctx.clone())?;
        for method in ["print", "println"] {
            stream.prop(
                method,
                Function::new(ctx.clone(), move |text: Opt<String>| {
                    let text = text.0.unwrap_or_default();
                    if to_stderr {
                        eprintln!("[java.lang.System.err] {text}");
                    }
                })?,
            )?;
        }
        object.prop(name, stream)?;
    }
    Ok(object)
}

fn java_lang_thread<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.prop(
        "sleep",
        Function::new(ctx.clone(), |millis: JsValue<'_>| {
            let millis = crate::java::js_to_f64(&millis).clamp(0.0, 10_000.0) as u64;
            std::thread::sleep(std::time::Duration::from_millis(millis));
        })?,
    )?;
    Ok(object)
}

fn zip_util<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    use std::io::{Read as _, Write as _};

    let object = Object::new(ctx.clone())?;
    for (name, gzip) in [("unGzip", true), ("unZlib", false)] {
        object.set(
            name,
            Function::new(ctx.clone(), move |ctx: Ctx<'js>, data: JsValue<'js>| {
                let bytes = crate::java::js_bytes(&data);
                let mut out = Vec::new();
                let result = if gzip {
                    flate2::read::GzDecoder::new(bytes.as_slice()).read_to_end(&mut out)
                } else {
                    flate2::read::ZlibDecoder::new(bytes.as_slice()).read_to_end(&mut out)
                };
                result.map_err(|error| {
                    rquickjs::Error::new_from_js_message("zip", "java", error.to_string())
                })?;
                Ok::<_, rquickjs::Error>(crate::java::bytes_to_js(&ctx, &out))
            })?,
        )?;
    }
    object.set(
        "gzip",
        Function::new(
            ctx.clone(),
            |ctx: Ctx<'js>, data: JsValue<'js>, _charset: Opt<String>| {
                let bytes = crate::java::js_bytes(&data);
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(&bytes).map_err(|error| {
                    rquickjs::Error::new_from_js_message("zip", "java", error.to_string())
                })?;
                let out = encoder.finish().map_err(|error| {
                    rquickjs::Error::new_from_js_message("zip", "java", error.to_string())
                })?;
                Ok::<_, rquickjs::Error>(crate::java::bytes_to_js(&ctx, &out))
            },
        )?,
    )?;
    Ok(object)
}

fn random_util<'js>(ctx: &Ctx<'js>) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.set(
        "randomNumbers",
        rquickjs::Function::new(ctx.clone(), |count: usize| -> String {
            let mut rng = rand::thread_rng();
            (0..count)
                .map(|_| char::from(b'0' + rng.gen_range(0..10u8)))
                .collect()
        })?,
    )?;
    object.set(
        "randomString",
        rquickjs::Function::new(ctx.clone(), |count: usize| -> String {
            const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
            let mut rng = rand::thread_rng();
            (0..count)
                .map(|_| char::from(CHARS[rng.gen_range(0..CHARS.len())]))
                .collect()
        })?,
    )?;
    object.set(
        "randomInt",
        rquickjs::Function::new(ctx.clone(), || -> i32 { rand::random() })?,
    )?;
    object.set(
        "randomLong",
        rquickjs::Function::new(ctx.clone(), || -> f64 { rand::random() })?,
    )?;
    Ok(object)
}

fn url_codec<'js>(ctx: &Ctx<'js>, encode: bool) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    if encode {
        object.set(
            "encode",
            rquickjs::Function::new(
                ctx.clone(),
                |text: String, charset: rquickjs::function::Opt<String>| -> String {
                    let encoding = encoding_of(charset.0.as_deref());
                    let bytes = encoding.encode(&text).0;
                    let mut out = String::with_capacity(bytes.len() * 3);
                    for byte in bytes.iter() {
                        match byte {
                            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => {
                                out.push(char::from(*byte));
                            }
                            b' ' => out.push('+'),
                            other => out.push_str(&format!("%{other:02X}")),
                        }
                    }
                    out
                },
            )?,
        )?;
    } else {
        object.set(
            "decode",
            rquickjs::Function::new(
                ctx.clone(),
                |text: String, charset: rquickjs::function::Opt<String>| -> String {
                    let bytes = percent_decode(&text);
                    encoding_of(charset.0.as_deref())
                        .decode(&bytes)
                        .0
                        .into_owned()
                },
            )?,
        )?;
    }
    Ok(object)
}

fn encoding_of(label: Option<&str>) -> &'static encoding_rs::Encoding {
    label
        .and_then(|label| encoding_rs::Encoding::for_label(label.trim().as_bytes()))
        .unwrap_or(encoding_rs::UTF_8)
}

fn percent_decode(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                match u8::from_str_radix(&text[index + 1..index + 3], 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    out
}

const TREE: &str = r#"
(function () {
  // rquickjs 造出来的宿主函数**不可 `new`**（`new f()` 报 not a constructor），
  // 而书源里 `new Packages.cn.hutool.crypto.symmetric.AES(...)` 这种写法很常见：
  // 统一包一层 JS 函数，`f(...)` 和 `new f(...)` 就都能用了（返回的还是宿主那个对象）。
  function constructible(host) {
    return function () { return host.apply(this, arguments); };
  }
  const listed = [
    [__legado_aes, 'AES'], [__legado_des, 'DES'], [__legado_desede, 'DESede'],
    [__legado_rsa, 'RSA'], [__legado_key_spec, 'SecretKeySpec'],
    [__legado_iv_spec, 'IvParameterSpec'], [__legado_sign, 'Sign'],
    [__legado_digest_util, 'DigestUtil'], [__legado_key_type, 'KeyType'],
    [__legado_javax_mac, 'Mac'], [__legado_javax_cipher, 'Cipher'],
    [__legado_datatype, 'DatatypeConverter'], [__legado_random_util, 'RandomUtil'],
    [__legado_zip_util, 'ZipUtil'],
    [__legado_url_decoder, 'URLDecoder'], [__legado_url_encoder, 'URLEncoder'],
    [__engine_jsoup, 'Jsoup'],
  ];
  const reg = {};
  for (const [raw, name] of listed) {
    const cls = typeof raw === 'function' ? constructible(raw) : raw;
    cls.__legado_name = name;
    reg[name] = cls;
  }
  const AES = reg.AES, DES = reg.DES, DESede = reg.DESede, DigestUtil = reg.DigestUtil;
  const RSA = reg.RSA, KeyType = reg.KeyType, Sign = reg.Sign;
  const Mac = reg.Mac, Cipher = reg.Cipher;
  const SecretKeySpec = reg.SecretKeySpec, IvParameterSpec = reg.IvParameterSpec;
  const DatatypeConverter = reg.DatatypeConverter, RandomUtil = reg.RandomUtil, ZipUtil = reg.ZipUtil;
  const URLDecoder = reg.URLDecoder, URLEncoder = reg.URLEncoder, Jsoup = reg.Jsoup;

  const StrUtil = {
    reverse: function (value) {
      return Array.from(String(value === undefined || value === null ? '' : value)).reverse().join('');
    },
    isEmpty: function (value) {
      return value === undefined || value === null || String(value).length === 0;
    },
    isBlank: function (value) {
      return value === undefined || value === null || String(value).trim().length === 0;
    },
    format: function (template) {
      const args = Array.prototype.slice.call(arguments, 1);
      return String(template).replace(/\{\}/g, function () {
        return args.length ? String(args.shift()) : '{}';
      });
    },
    toString: function (value) {
      return value === undefined || value === null ? '' : String(value);
    },
  };

  function binaryCall(op, payload) {
    const result = JSON.parse(__binary(op, JSON.stringify(payload)));
    if (result.error) throw new Error(result.error);
    return result.value;
  }

  const HutoolBase64 = {
    encode: function (value) {
      if (Array.isArray(value)) return String(binaryCall('base64Encode', { data: value }));
      return String(java.base64Encode(String(value)));
    },
    decode: function (value) {
      return java.base64DecodeToByteArray(String(value));
    },
  };

  function okhttpHeaders(text) {
    const headers = {};
    String(text || '').split(/\r?\n/).forEach(function (line) {
      const index = line.indexOf(':');
      if (index > 0) {
        headers[line.slice(0, index).trim().toLowerCase()] = line.slice(index + 1).trim();
      }
    });
    return headers;
  }

  function okhttpResponse(response) {
    const bodyText = String(response && response.body ? response.body() : '');
    const headers = okhttpHeaders(response && response.headers ? response.headers() : '');
    return {
      code: function () { return Number(response && response.code ? response.code : 0); },
      body: function () {
        return {
          string: function () { return bodyText; },
          bytes: function () {
            return response && response.bodyAsBytes ? response.bodyAsBytes() : [];
          },
        };
      },
      headers: function () {
        return {
          names: function () { return Object.keys(headers); },
          get: function (name) {
            const key = String(name).toLowerCase();
            return Object.prototype.hasOwnProperty.call(headers, key) ? headers[key] : null;
          },
        };
      },
    };
  }

  function MediaType() {}
  MediaType.parse = function (value) {
    return { __mediaType: String(value) };
  };

  function RequestBody() {}
  RequestBody.create = function (body, mediaType) {
    return { __body: body, mediaType: mediaType };
  };

  function RequestBuilder() {
    this.headers = {};
    this.method = 'GET';
    this.body = null;
  }
  RequestBuilder.prototype.url = function (value) {
    this.url = String(value);
    return this;
  };
  RequestBuilder.prototype.get = function () {
    this.method = 'GET';
    this.body = null;
    return this;
  };
  RequestBuilder.prototype.post = function (body) {
    this.method = 'POST';
    this.body = body && body.__body !== undefined ? body.__body : body;
    return this;
  };
  RequestBuilder.prototype.addHeader = function (name, value) {
    this.headers[String(name)] = String(value);
    return this;
  };
  RequestBuilder.prototype.build = function () {
    return {
      url: this.url,
      method: this.method,
      body: this.body,
      headers: this.headers,
    };
  };

  function Request() {}
  Request.Builder = RequestBuilder;

  function OkHttpClient() {}
  OkHttpClient.prototype.newCall = function (request) {
    return {
      execute: function () {
        const option = {
          method: request.method || 'GET',
          headers: request.headers || {},
        };
        if (request.body !== undefined && request.body !== null) {
          option.body = request.body;
        }
        const response = java.ajax(String(request.url) + ',' + JSON.stringify(option));
        return okhttpResponse(response);
      },
    };
  };

  function Response() {}

  const tree = {
    cn: { hutool: {
      crypto: {
        symmetric: { AES, DES, DESede },
        asymmetric: { RSA, KeyType },
        digest: { DigestUtil },
      },
      core: {
        util: { RandomUtil, ZipUtil, StrUtil },
        codec: { Base64: HutoolBase64 },
      },
    } },
    java: { net: { URLDecoder, URLEncoder },
            lang: { System: __legado_java_lang_system, Thread: __legado_java_lang_thread } },
    okhttp3: { MediaType, RequestBody, Request, OkHttpClient, Response },
    org: { jsoup: { Jsoup } },
    javax: { crypto: { Mac, Cipher, spec: { SecretKeySpec, IvParameterSpec } },
             xml: { bind: { DatatypeConverter } } },
  };
  // 每层包一个 Proxy：没注册的包 / 类名直接报出来
  function wrap(node, path) {
    for (const key of Object.keys(node)) {
      const child = node[key];
      if (child && typeof child === 'object') node[key] = wrap(child, path + '.' + key);
    }
    return new Proxy(node, {
      get(target, key) {
        if (key in target) return target[key];
        if (typeof key === 'symbol' || key === 'then' || String(key).startsWith('__')) return undefined;
        throw new Error('Packages 里没有注册 ' + path + '.' + String(key));
      },
      has(target, key) { return key in target; },
    });
  }
  globalThis.Packages = wrap(tree, 'Packages');
  globalThis.cn = globalThis.Packages.cn;
  globalThis.org = globalThis.Packages.org;
  globalThis.javax = globalThis.Packages.javax;

  // Rhino 的 importClass 是可变参数，书源里常见一次导入好几个
  globalThis.importClass = function () {
    let last;
    for (const cls of arguments) {
      const name = cls && cls.__legado_name;
      if (!name) throw new Error('importClass 只认注册表里的类（Packages.*）');
      globalThis[name] = cls;
      last = cls;
    }
    return last;
  };
  globalThis.JavaImporter = function () {
    const importer = {};
    const seen = new Set();
    const collect = (node) => {
      if (!node || typeof node !== 'object' || seen.has(node)) return;
      seen.add(node);
      for (const key of Object.keys(node)) {
        const value = node[key];
        if (!value || (typeof value !== 'function' && typeof value !== 'object')) continue;
        importer[key] = value;
        if (globalThis[key] === undefined) globalThis[key] = value;
        collect(value);
      }
    };
    importer.importPackage = function () {
      for (const pkg of arguments) collect(pkg);
      return importer;
    };
    importer.importClass = function (cls) {
      const name = cls && cls.__legado_name;
      if (!name) throw new Error('importClass ֻ��ע�������ࣨPackages.*��');
      importer[name] = cls;
      return cls;
    };
    for (const cls of arguments) {
      const name = cls && cls.__legado_name;
      if (!name) throw new Error('JavaImporter ֻ��ע�������ࣨPackages.*��');
      importer[name] = cls;
    }
    return importer;
  };
})()
"#;
