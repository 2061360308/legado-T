# Legado 3.0 Rust 规则引擎对比报告

> 历史快照报告：以下提交号、外部路径和统计仅代表 2026-10-29 的调研状态，不代表当前 LegadoT 仓库状态。

- 对比日期：2026-10-29
- Kotlin 基线：`C:\Users\20613\Desktop\legado`，提交 `dfb2fdd`
- 目标项目：`C:\Users\20613\Desktop\legado\local-reader`
- 对比对象：
  - `hadc188/reader`，提交 `e06707fe`（2026-10-29）
  - `SCX-snow/legado-rust-engine`，提交 `bb548b48`（2026-10-21）
- 本地副本：
  - `C:\Users\20613\AppData\Local\Temp\opencode\legado-research\hadc188-reader`
  - `C:\Users\20613\AppData\Local\Temp\opencode\legado-research\legado-rust-engine`

> 本报告基于源码静态对比和仓库自带测试/对拍数据，未在本机完成两个仓库的完整构建。A/B 统计来自仓库内已录制的 `legado.jsonl` / `rust.jsonl`，不是本次重新运行的结果。

## 1. 结论摘要

两个项目都已经是“可用级别”的 Legado 3.0 规则引擎，不是玩具实现。

- **`hadc188/reader`：更适合 local-reader 当前 GPL-3.0 项目。**
  - MIT 许可证，和 local-reader 的 GPL-3.0 兼容。
  - 规则引擎、JS/Java 宿主、URL/HTTP、搜索/详情/目录/正文/发现流程完整。
  - WebView、登录、`sourceRegex`、`webJs`、`coverDecodeJs`、`downloadUrls` 等长尾能力明显更全。
  - 自带 1103 行 `book-source-rules.md` 兼容规格和 237 个集成测试，工程参考价值很高。
  - 主要短板：Java 正则兼容性、TTF、文件/压缩包辅助函数、JS 运行时超时/内存限制、部分数据模型字段。

- **`SCX-snow/legado-rust-engine`：引擎设计最像“可复用库”，但许可证和原生依赖更重。**
  - 纯 Rust workspace，`legado-core` / `legado-rule` / `legado-js` / `legado-net` / `legado-book` / `legado-regex` / `legado-replace` / `legado-rss` 分层清晰。
  - Java 正则用 ICU4C/AOSP PatternNative，TTF、RSS、替换净化、A/B 对拍都很强。
  - JS 运行时具备内存限制、栈限制、中断和超时，安全性更好。
  - 主要短板：AGPL-3.0、需要 libxml2/libclang/ICU4C；WebView/登录/验证码、`sourceRegex`、`webJs`、`downloadUrls` 缺失或仅建模；JS 全局 `title` / `nextChapterUrl` 未绑定。

**推荐路线：**

1. **如果 local-reader 继续使用 GPL-3.0：以 `hadc188/reader` 为底座抽取引擎，SCX 只作为行为参考/对拍参考，不复制其 AGPL 代码。**
2. **如果 local-reader 可以升级为 AGPL-3.0：直接依赖 SCX 更省引擎开发量，但要补齐 WebView/登录、`sourceRegex`/`webJs`、`title`/`nextChapterUrl` 等缺口。**
3. 不建议从零手写规则引擎；两个项目都已经覆盖了最难的规则切分、URL 编译、JS 宿主和书源流水线。

## 2. Kotlin 原版能力基线

Kotlin 原版的能力以代码为准，教程只作语法说明。核心文件：

- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeRule.kt`
  - 模式：`Mode.XPath / Json / Default / Js / Regex`（`:733`）
  - 规则拆分：`splitSourceRule`（`:485`）
  - JS 绑定：`evalJS`（`:774`）
- `AnalyzeUrl.kt`
  - URL 初始化顺序：`initUrl`（`:120`）
  - URL 参数正则：`paramPattern` / `pagePattern`（`:668`）
- `RuleAnalyzer.kt`：`&&` / `||` / `%%` / `@` 的平衡切分
- `AnalyzeByJSoup.kt`：Default/CSS 选择器
- `AnalyzeByXPath.kt`：JXDocument XPath
- `AnalyzeByJSonPath.kt`：Jayway JSONPath
- `AnalyzeByRegex.kt`：正则链
- `BookList.kt` / `BookInfo.kt` / `BookChapterList.kt` / `BookContent.kt`：书源流水线
- `JsExtensions.kt`：`java.*`、Cookie、Cache、文件、TTF、加密等
- `BookSource.kt` + `data/entities/rule/*.kt`：书源数据模型

Kotlin 侧的关键能力：

- 五种规则模式：Default/CSS、XPath、JSONPath、Regex、JS
- `@CSS:`、`@@`、`@XPath:`、`@Json:`、`@js:`、`<js>`、`:`、`$.`、`/`
- `&&` / `||` / `%%`、`##` / `###`、`$1`、`@get` / `@put`、`{{}}`
- URL 规则：`@js` / `<js>`、`{{key}}` / `{{page}}`、`<a,b,c>`、`,{...}` UrlOption、相对 URL、charset、proxy、Cookie
- 书源流程：搜索、发现、详情、目录、正文、精确搜索
- 目录/正文分页、`nextTocUrl`、`nextContentUrl`、`formatJs`、`preUpdateJs`
- `sourceRegex`、`webJs`、`replaceRegex`、`imageDecode`、`payAction`
- JS 绑定：`java`、`cookie`、`cache`、`source`、`book`、`chapter`、`title`、`src`、`result`、`baseUrl`、`nextChapterUrl`、`rssArticle`、`page`、`key`
- Java 扩展：HTTP、编码、哈希、AES、文件、压缩包、TTF、WebView、登录、验证码等

## 3. `hadc188/reader` 分析

### 3.1 仓库形态

- 根包名 `reader-rust`，版本 `1.6.0`，MIT。
- 仓库是完整应用：Rust 服务端 + Tauri 桌面端 + Vue 前端。
- 规则引擎相关代码在 `src/parser`、`src/crawler`、`src/model`、`src/service`，不依赖 JVM。
- 依赖包含 `scraper 0.20`、`sxd-xpath 0.4`、`jsonpath_lib 0.3`、`regex 1`、`rquickjs 0.7`、`reqwest 0.12`。
- 没有 libxml2 / libclang / ICU4C 这类原生库依赖；Windows 构建相对简单。

### 3.2 规则引擎

关键文件：

- `src/parser/rule_engine.rs`：约 202 KB，规则求值与书源字段解析
- `src/parser/rule_analyzer.rs`：`&&` / `||` / `%%` / `@` 平衡切分
- `src/parser/html.rs`：约 67 KB，Jsoup Default/CSS 选择器
- `src/parser/jsonpath.rs`：JSONPath
- `src/parser/js.rs`：约 322 KB，JS 运行时与 Java 兼容层
- `src/parser/java_compat.js`：约 46 KB，Java 类模拟
- `src/crawler/url_analyzer.rs`：约 34 KB，URL 规则与 UrlOption
- `src/model/rule.rs` / `book_source.rs`：规则模型与书源模型

能力覆盖：

- `ParseMode` 覆盖 `Css` / `XPath` / `JsonPath` / `Regex` / `Js`。
- `@CSS:`、`@@`、`@XPath:`、`@Json:`、`@js:`、`<js>`、`:`、`$.`、`/` 都有处理。
- `&&` / `||` / `%%` 使用平衡切分，并专门处理 `<js>...</js>` 内部的 `||` / `&&`。
- `##` 替换、`###` 只替换第一个、`$N` 分组。
- `@get` / `@put`、`{{}}` 内嵌 JS/规则。
- `bookUrlPattern`、列表倒序、去重、封面绝对化、字数格式化。
- 搜索、发现、详情、目录、正文、目录分页、正文分页。
- `preUpdateJs`、`formatJs`、`nextTocUrl`、`nextContentUrl`。
- `sourceRegex`、`webJs`、`replaceRegex`、`coverDecodeJs`、`downloadUrls`。
- `canReName`、`isVip` / `isPay` / `isVolume`、`exploreKind`。

`html.rs` 的 Jsoup 兼容层很强，包含：

- `class.xxx` / `id.xxx` / `tag.xxx` / `children` / `text.xxx`
- `:eq` / `:gt` / `:lt` / `:first` / `:last`
- `:contains` / `:containsOwn` / `:containsData` / `:containsWholeText`
- `:matches` / `:matchesOwn` / `:matchesWholeText`
- `:has` / `:not` / 标准 CSS 伪类
- `[index]` / `[!index]` / `[start:end:step]` / 负数索引 / 反向区间
- 片段 HTML 包装，处理 `<tr>` / `<td>` / `<li>` 等被 HTML5 解析器丢弃的场景

### 3.3 JS / Java 宿主

- `rquickjs 0.7`。
- `js.rs` 里有完整的 JS prelude，绑定 `java`、`source`、`book`、`chapter`、`cookie`、`cache`、`result`、`baseUrl`、`title`、`src`、`nextChapterUrl` 等。
- `java_compat.js` 覆盖：
  - `BigInteger`、`Charset`、`SecureRandom`、`URL`、`URLEncoder` / `URLDecoder`
  - `Base64`、`Cipher`、`Mac`、`SecretKeySpec`、`IvParameterSpec`、`GCMParameterSpec`
  - `ByteArrayInputStream` / `OutputStream`、`ByteBuffer`、`GZIPInputStream` / `GZIPOutputStream`、`Inflater`
  - `BufferedReader`、`InputStreamReader`
- `java` 对象支持：
  - `ajax` / `ajaxAll` / `connect` / `get` / `post` / `head`
  - `getString` / `getStringList` / `getElement` / `getElements` / `setContent`
  - Base64 / Hex / MD5 / SHA / HMAC / AES / DES / RSA
  - `timeFormat` / `timeFormatUTC` / `encodeURI` / `htmlFormat` / `t2s` / `s2t`
  - `getCookie` / `getCache` / `putCache` / `ensureGlobalVariable`
  - `webView` / `webViewGetSource` / `webViewGetOverrideUrl`
  - `startBrowser` / `startBrowserAwait` / `showBrowser`
  - `openUrl` / `toURL` / `toNumChapter` / `randomUUID` / `androidId` / `getWebViewUA`
  - `refreshExplore` / `searchBook` / `openBook`
- 支持 `Packages`、`importClass`、`JavaImporter`、`with()` 等 Rhino 风格写法。
- `book` / `chapter` 暴露 `getKey()`、`getVariable()`、`putVariable()`、`getCustomVariable()`、`putCustomVariable()`。
- `source` 暴露 `getKey()`、`getVariable()`、`setVariable()`、`getLoginHeaderMap()`、`putLoginHeader()` 等。

### 3.4 URL / HTTP

`src/crawler/url_analyzer.rs` 覆盖：

- `@js` / `<js>` URL 片段
- `{{key}}` / `{{page}}`
- `<a,b,c>` 页码选择
- `,{...}` UrlOption：`method` / `charset` / `headers` / `body` / `type` / `js` / `retry` / `serverID` / `webView` / `webJs` / `webViewDelayTime`
- 相对 URL、GET query 编码、POST form/json 编码
- charset、proxy、Cookie、重试、原始字节响应

`src/crawler/fetcher.rs` / `http_client.rs` / `source_session.rs` 覆盖：

- 每书源独立 Cookie/Session
- 代理、gzip/brotli/deflate、重试、超时
- 原始字节路径，支持 `java.net.URL(...).openConnection()` 这类脚本
- 并发限流、取消信号、Host 头保持等细节

### 3.5 书源流水线

`src/service/book_service.rs` 和 `src/parser/rule_engine.rs` 覆盖：

- 搜索：`search_books`、`bookUrlPattern`、详情页回退、去重、倒序
- 发现：`explore_books`、`exploreUrl` 解析、`ExploreKind` 控件字段
- 详情：`book_info`、`init`、`canReName`、`tocUrl`、`downloadUrls`、封面绝对化
- 目录：`chapter_list`、`nextTocUrl`、`formatJs`、`preUpdateJs`、`isVip` / `isPay` / `isVolume`、倒序、去重、并行目录
- 正文：`content`、`title`、`nextContentUrl`、`sourceRegex`、`webJs`、`replaceRegex`、多页拼接
- 登录：`loginUrl` / `loginUi` / `loginCheckJs`、`source_browser` 面板、Cookie 回写
- 封面解密：`coverDecodeJs`

### 3.6 测试与文档

- `book-source-rules.md`：1103 行，按 Kotlin `AnalyzeRule` / `AnalyzeUrl` / `RuleAnalyzer` / `AnalyzeBy*` / 书源流水线写的实现规格。
- `src/parser/rule_engine.rs` 内约 27 个 `#[test]`。
- `tests/` 目录约 237 个 `#[test]` / `#[tokio::test]`，重点文件：
  - `source_rule_regressions.rs`
  - `source_rule_chain_and_replace.rs`
  - `source_legado_semantics.rs`
  - `source_jsoup_element_api.rs`
  - `source_css_pseudo_titles.rs`
  - `source_xpath_and_redirect.rs`
  - `source_webview_bridge.rs`
  - `source_login_*`
  - `source_jca_crypto.rs`
  - `source_android_java_compat.rs`
  - `offline_source_matrix.rs`
  - `live_installed_sources.rs`
  - `live_source_smoke.rs`
- `source_webview_bridge.rs` 覆盖 `java.webView` / `webViewGetSource`。
- `source_login_*` 覆盖登录面板、`loginCheckJs`、Cookie/Header 回写。
- `source_jca_crypto.rs` 覆盖 JCA/HMAC/AES/GCM/PBKDF2。
- `offline_source_matrix.rs` 用真实书源做离线加载/规则编译/登录 UI 解析。

### 3.7 优势

- MIT，和 local-reader GPL-3.0 兼容。
- 纯 Rust 依赖为主，Windows 构建成本低。
- 规则、JS、URL、书源流水线覆盖完整。
- WebView/登录/`sourceRegex`/`webJs`/`coverDecodeJs`/`downloadUrls` 等长尾能力明显更全。
- 自带 1103 行兼容规格，可直接作为 local-reader 的实现契约。
- 测试数量和真实书源测试覆盖较好。

### 3.8 差距与风险

- **Java 正则兼容性**：使用 Rust `regex`，不支持 lookbehind、backreference 等 Java 正则特性。Java 正则重的书源会失败。
- **TTF**：未找到 `queryTTF` / `replaceFont` 实现。
- **文件/压缩包**：未找到 `downloadFile` / `cacheFile` / `readFile` / `unzip` 等完整实现。
- **JS 运行时安全**：`Runtime::new()` + `set_max_stack_size`，但没有 SCX 那样的内存限制、中断处理器、执行超时；死循环脚本可能挂住线程。
- **模型字段**：`BookChapter` 只有 `title/url/index/tag/isVip/isPay/isVolume`，没有 `bookUrl/baseUrl/variable`；`Book` 没有 `variable/tocHtml/infoHtml/downloadUrls`。JS prelude 通过 `JsBookContext` 和持久化 KV 补了一部分，但模型层不如 SCX 忠实。
- **`webView` URL 选项**：`url_analyzer` 解析了 `webView/webJs/webViewDelayTime`，但抓取层没有明确执行；`java.webView` / `showBrowser` 有宿主桥接。
- **`imageDecode` / `payAction`**：模型字段存在，未找到执行实现。
- **`updateTime` / `exploreScreen`**：模型字段存在，当前不执行（与 Kotlin 现有行为一致）。
- **MIT 声明来源**：README 说参考了 `reader-rust`。如果上游代码含 AGPL/GPL，MIT 声明可能不成立；分发前需核对上游许可证。local-reader 是 GPL-3.0，GPL 上游没问题；AGPL 上游则需升级。

## 4. `SCX-snow/legado-rust-engine` 分析

### 4.1 仓库形态

- 纯 Rust workspace，版本 `0.1.0`，AGPL-3.0。
- 分层：
  - `legado-core`：数据模型、trait、缓存、错误
  - `legado-rule`：规则编译/AST/求值、CSS/XPath/JSONPath
  - `legado-js`：rquickjs 运行时、`java.*`、`javax`、`Packages`、Jsoup shim、TTF
  - `legado-net`：URL 规则、HTTP、Cookie、编码、限流
  - `legado-book`：搜索、发现、详情、目录、正文
  - `legado-regex`：ICU4C/AOSP PatternNative 正则
  - `legado-replace`：替换净化
  - `legado-rss`：RSS
- 依赖包含 `libxml`、`scraper`、`serde_json_path`、`rquickjs 0.13`、`reqwest`、`moka`、`zhconv`。
- 需要系统 `libxml2`、`libclang`；`legado-regex` 需要 ICU4C。

### 4.2 规则引擎

关键文件：

- `crates/legado-rule/src/compile.rs`：规则编译为 `CompiledRule` / `Step` / `Rule` AST
- `crates/legado-rule/src/engine.rs`：`RuleEngine` / `EvalContext` / `RuleRunner`
- `crates/legado-rule/src/split.rs`：`split_js`、`split_combinator`、`split_replace`、`take_puts`、`split_pieces`
- `crates/legado-rule/src/selector/css.rs`：CSS / Jsoup Default
- `crates/legado-rule/src/selector/xpath.rs`：libxml2 XPath
- `crates/legado-rule/src/selector/json.rs`：serde_json_path
- `crates/legado-rule/src/selector/index.rs`：索引/排除/区间
- `crates/legado-rule/src/content.rs`：HTML/JSON/Text 内容抽象

能力覆盖：

- `Kind::Css` / `XPath` / `AutoXPath` / `Json` / `Regex` / `Auto`
- `@css:` / `@@` / `@xpath:` / `@json:` / `$.` / `$[` / `/` / `@js:` / `<js>`
- `&&` / `||` / `%%`、`##` / `###`、`$N`
- `@put` / `@get`、`{{}}`
- CSS 选择器：`class/id/tag/text/children`、索引、排除、区间、`:eq/:lt/:gt/:contains/:has/:not/:matches` 等
- XPath 1.0、JSONPath（RFC 9535 + 模板）
- 正则：ICU4C/AOSP PatternNative，带时间限制
- 规则编译缓存（`RuleEngine::compile` + `cached_rules`）

### 4.3 JS / Java 宿主

关键文件：

- `crates/legado-js/src/lib.rs`：`JsEngine` / `JsEngineBuilder` / `JsEvaluator`
- `crates/legado-js/src/worker.rs`：独立 JS 线程、内存限制、栈限制、中断、超时、Context 缓存
- `crates/legado-js/src/host.rs`：绑定 `java`、`cookie`、`cache`、`source`、`book`、`chapter`、`result`、`baseUrl`、`src`、`page`、`key`
- `crates/legado-js/src/java.rs`：`java.*` 宿主函数
- `crates/legado-js/src/javax.rs`：`javax.crypto` 等
- `crates/legado-js/src/packages.rs`：`Packages` / `importClass` / `with()` 树
- `crates/legado-js/src/jsoup.rs`：Jsoup shim
- `crates/legado-js/src/rules.rs`：`java.getString` / `getStringList` / `getElement(s)` 回调
- `crates/legado-js/src/ttf.rs`：TTF 字体解析与替换

`java` 对象支持：

- `ajax` / `ajaxAll` / `connect` / `get` / `post` / `head`
- `getString` / `getStringList` / `getElement` / `getElements` / `setContent`
- Base64 / Hex / MD5 / SHA / HMAC / AES / DES / RSA
- `timeFormat` / `timeFormatUTC` / `encodeURI` / `htmlFormat` / `t2s` / `s2t`
- `getCookie` / `getUrl` / `put` / `cacheFile` / `importScript`
- `queryTTF` / `queryBase64TTF` / `replaceFont`
- `randomUUID` / `androidId` / `toNumChapter`
- `log` / `toast` / `longToast`
- `createSymmetricCrypto` / `createAsymmetricCrypto` / `createSign` / `digestHex` / `HMacHex` / `HMacBase64`

### 4.4 URL / HTTP

`crates/legado-net/src/url.rs`：

- `@js` / `<js>`、`{{key}}` / `{{page}}`、`<a,b,c>`
- `,{...}` UrlOption：`method` / `charset` / `headers` / `body` / `js` / `retry` / `serverID`
- 相对 URL、GET query、POST form/json、charset
- **没有 `webView` / `webJs` / `webViewDelayTime` 字段**

`crates/legado-net/src/client.rs`：

- 每书源 Cookie/Session、代理、超时、重试
- gzip/brotli、编码检测
- 限流 `RateLimiter`
- `send` / `send_retry` / `send_js` / `send_rule`

### 4.5 书源流水线

`crates/legado-book`：

- `engine.rs`：`BookEngine::search` / `explore` / `precise_search` / `book_info` / `chapters` / `content` / `content_with_replace`
- `list.rs`：搜索/发现列表、`bookUrlPattern`、倒序、去重、字数/封面
- `info.rs`：详情、`init`、`canReName`、`tocUrl`
- `toc.rs`：目录、`nextTocUrl`、`formatJs`、`isVip/isPay/isVolume`、去重、倒序、分页
- `content.rs`：正文、`title`、`nextContentUrl`、`replaceRegex`、多页
- `eval.rs`：`PageFetcher`、`loginCheckJs`、`fetch_page`

### 4.6 测试与对拍

SCX 的测试策略不是大量 `#[test]`，而是：

- `scripts/run-tests.sh` 运行 12 个离线 example + `cargo fmt` + `cargo clippy`
- `fixtures/legado_ab/`：1156 行、1007 个唯一 case，和真实 legado 输出逐条对拍
- `fixtures/legado_ab_real/`：112 行、100 个唯一 case，真实书源 + 真实页面
- `fixtures/replace_ab/`：48 行、22 个唯一 case，净化规则对拍
- `examples/rule_eval.rs`、`book_offline.rs`、`js_eval.rs`、`query_ttf.rs`、`legado_ab.rs`、`replace_eval.rs`

本次从仓库内 `legado.jsonl` / `rust.jsonl` 统计：

| 语料 | 唯一 case | 状态一致 | 两边都成功 | 结果完全一致 | 结果不一致 | 状态不一致 | 两边都失败 |
|---|---:|---:|---:|---:|---:|---:|---:|
| `legado_ab` | 1007 | 933 (92.7%) | 716 | 668 (93.3% of both-ok) | 48 | 67 | 217 |
| `legado_ab_real` | 100 | 93 (93.0%) | 66 | 59 (89.4% of both-ok) | 7 | 7 | 27 |
| `replace_ab` | 22 | 22 (100%) | 22 | 22 (100%) | 0 | 0 | 0 |

注意：

- `legado_ab_real` 按 README 是“刻意偏向我们与 legado 不一致的 case”，所以 89.4% 不是随机样本的兼容率。
- 这些数字来自仓库已录制的 baseline，不是本次重新运行。
- 但“有 1000+ 条真实对拍语料”本身是 SCX 的最大优势之一。

### 4.7 优势

- 最像“可复用库”，API 抽象清晰，`PageFetcher` / `HttpFetcher` / `JsEvaluator` / `RuleRunner` 都可替换。
- Java 正则用 ICU4C/AOSP PatternNative，最接近 Android `java.util.regex`。
- TTF、RSS、替换净化完整。
- JS 运行时内存限制、栈限制、中断、超时、Context 缓存，安全性最好。
- 数据模型最忠实 Kotlin：`Book` / `SearchBook` / `BookChapter` 都有 `variables`，`BookChapter` 有 `baseUrl` / `bookUrl`。
- A/B 对拍语料和真实页面语料是极强的兼容性证据。

### 4.8 差距与风险

- **许可证**：AGPL-3.0。local-reader 是 GPL-3.0；链接/复制 SCX 代码通常需要把 local-reader 整体升级为 AGPL-3.0。
- **原生依赖**：libxml2、libclang、ICU4C；Windows 构建和分发明显更麻烦。
- **WebView / 登录 / 验证码**：没有 `webView` / `startBrowser` / `getVerificationCode` / `loginUi` 的完整实现，只有 `loginCheckJs`。
- **`sourceRegex` / `webJs`**：`ContentRule` 有字段，但 `legado-book/src/content.rs` 没有执行。
- **`downloadUrls` / `coverDecodeJs` / `imageDecode` / `payAction`**：模型字段存在，未找到执行实现。
- **JS 全局**：绑定 `result/baseUrl/src/page/key/source/book/chapter`，但没有绑定裸 `title`、`nextChapterUrl`、`rssArticle`。使用 `title` 的正文规则会拿到 `undefined`，需要改用 `chapter.title` 或补绑定。
- **响应对象**：`body()` / `url` / `code` / `headers()` / `header` / `statusCode` 有；`cookies()` / `message()` / `isSuccessful()` 等不如 Kotlin 完整。
- **文件/压缩包**：没有 `readFile` / `getFile` / `unzip` / `downloadFile` 等；只有 `cacheFile` / `importScript`。
- **`enabledCookieJar` 默认值**：`Source::cookie_jar()` 在未设置时返回 `false`，Kotlin 默认是 `true`，需要核对。
- **API 版本**：0.1.0，API 可能调整。

## 5. 能力矩阵

说明：`支持` = 有实现且有测试/明确代码路径；`部分` = 有实现但有语义差异或依赖宿主；`缺失` = 未找到实现或仅模型字段。

| 能力 | Kotlin 原版 | hadc188/reader | SCX-snow |
|---|---|---|---|
| Default/CSS 规则 | 支持 | 支持 | 支持 |
| XPath 规则 | 支持 | 支持 | 支持 |
| JSONPath 规则 | 支持 | 支持 | 支持 |
| Regex 规则 | 支持（Java regex） | 部分（Rust regex） | 支持（ICU/AOSP） |
| JS 规则 | 支持（Rhino） | 支持（rquickjs 0.7） | 支持（rquickjs 0.13） |
| `@CSS:` / `@@` / `@XPath:` / `@Json:` / `@js:` / `<js>` | 支持 | 支持 | 支持 |
| `&&` / `||` / `%%` | 支持 | 支持 | 支持 |
| `##` / `###` / `$N` | 支持 | 支持 | 支持 |
| `@get` / `@put` | 支持 | 支持 | 支持 |
| `{{}}` | 支持 | 支持 | 支持 |
| URL `@js` / `<js>` | 支持 | 支持 | 支持 |
| URL `{{key}}` / `{{page}}` / `<a,b,c>` | 支持 | 支持 | 支持 |
| URL `,{...}` 常用选项 | 支持 | 支持 | 支持 |
| URL `webView` / `webJs` | 支持 | 部分（解析 + 宿主桥接） | 缺失 |
| Jsoup Default 伪类/索引 | 支持 | 支持 | 支持 |
| XPath 函数/HTML 容错 | 支持 | 支持 | 支持（libxml2 差异） |
| JSONPath Jayway 扩展 | 支持 | 部分 | 部分 |
| Java 正则 lookbehind/backref | 支持 | 缺失 | 支持 |
| JS `java.*` 高频函数 | 支持 | 支持 | 支持 |
| JS `Packages` / `importClass` / `with` | 支持 | 支持 | 支持 |
| JS `source` / `book` / `chapter` / `cookie` / `cache` | 支持 | 支持 | 支持 |
| JS 裸 `title` | 支持 | 支持 | 缺失 |
| JS `nextChapterUrl` | 支持 | 支持 | 缺失 |
| JS `rssArticle` | 支持 | 部分 | 缺失 |
| JS 超时/内存/中断 | 部分 | 缺失 | 支持 |
| WebView `java.webView` | 支持 | 支持 | 缺失 |
| 登录/`loginUi`/`startBrowser` | 支持 | 支持 | 缺失 |
| 验证码 `getVerificationCode` | 支持 | 缺失 | 缺失 |
| TTF `queryTTF` / `replaceFont` | 支持 | 缺失 | 支持 |
| 文件/压缩包辅助 | 支持 | 部分/缺失 | 部分/缺失 |
| `sourceRegex` / `webJs` | 支持 | 支持 | 缺失 |
| `coverDecodeJs` / `downloadUrls` | 支持 | 支持 | 缺失 |
| `imageDecode` / `payAction` | 支持 | 缺失（仅模型） | 缺失（仅模型） |
| 搜索/发现/详情/目录/正文 | 支持 | 支持 | 支持 |
| 目录/正文分页 | 支持 | 支持 | 支持 |
| 精确搜索 | 支持 | 支持 | 支持 |
| 书源模型字段 | 支持 | 部分 | 支持 |
| Book/Chapter 变量 | 支持 | 支持（JS/KV 补偿） | 支持 |
| HTTP Cookie/Proxy/Retry/编码/限流 | 支持 | 支持 | 支持 |
| 并发/取消 | 支持 | 支持 | 支持 |
| 回归测试 | Kotlin 原版 | 237 个测试 + 1103 行规格 | 1000+ A/B case + examples |
| 许可证 | GPL-3.0 | MIT | AGPL-3.0 |
| 构建复杂度 | Android/JVM | 较低（纯 Rust 为主） | 较高（libxml2/libclang/ICU4C） |

## 6. 关键差异详解

### 6.1 规则语法层

两者都实现了 Legado 的规则语法层，但实现方式不同：

- hadc188：运行时字符串解析 + 平衡切分，`rule_engine.rs` 直接处理 `ParseMode` 和字段链。
- SCX：先编译成 AST（`CompiledRule` / `Step` / `Rule`），再求值；有规则编译缓存。

对 local-reader 的影响：

- SCX 的 AST/编译缓存更适合高频目录/正文解析。
- hadc188 的实现更贴近 Kotlin 原始字符串行为，调试时更容易对照。

### 6.2 CSS / Jsoup Default

- hadc188 的 `html.rs` 对 Jsoup 伪类覆盖非常细，包含 `:containsData`、`:containsWholeText`、`:matchesWholeOwnText`、`:empty`、`:has`、`:not`、`:is` 等，并处理 HTML 片段包装。
- SCX 的 `selector/css.rs` 也有 `Eq/Lt/Gt/Contains/ContainsOwn/Matches/Has/Not` 和索引/区间，覆盖主流书源，但细节数量少于 hadc188。

结论：两者都够用；Jsoup 兼容细节 hadc188 更强。

### 6.3 XPath

- Kotlin：JXDocument（JsoupXpath），支持 `::text` / `::ownText` / `::html` 等扩展。
- hadc188：`sxd-xpath` + 自写扩展，测试覆盖 malformed HTML、列表字段作用域、302 跳转。
- SCX：libxml2 XPath 1.0，HTML 解析用 libxml2，行为与 JXDocument 有差异。

结论：hadc188 更接近 JsoupXpath 的扩展写法；SCX 的 XPath 引擎更标准，但可能需要补 JXDocument 扩展。

### 6.4 JSONPath

- Kotlin：Jayway JSONPath，支持过滤器、函数、`$..`。
- hadc188：`jsonpath_lib`，有前导点递归、`{$...}` 内嵌路径；测试覆盖 Jayway 前导点差异。
- SCX：`serde_json_path`，RFC 9535，支持模板和过滤器；Jayway 特有函数可能缺失。

结论：普通 `$.a.b` / `$..books[*]` 都没问题；复杂 Jayway 函数需要补。

### 6.5 Regex

- Kotlin：Java `java.util.regex`，支持 lookbehind、backreference、占有量词等。
- hadc188：Rust `regex`，不支持 lookbehind/backreference。
- SCX：ICU4C/AOSP PatternNative，最接近 Android Java regex，并带时间限制。

结论：如果书源里大量 Java 正则，SCX 优势明显；hadc188 需要补 `fancy-regex` 或 ICU。

### 6.6 JS / Java 宿主

- hadc188：rquickjs + 大量 Java shim；WebView/登录/`Packages`/`JavaImporter`/`StringBuilder`/`HashMap`/`BigInteger`/`Charset`/`SecureRandom`/`GZIP` 都有。
- SCX：rquickjs + `java`/`javax`/`packages`/`jsoup`/`ttf`；内存/超时/中断更好；但裸 `title`、`nextChapterUrl`、`rssArticle` 未绑定，WebView/登录缺失。

结论：高频 `java.*` 两者都能覆盖；WebView/登录选 hadc188，TTF/Java 正则/运行时安全选 SCX。

### 6.7 URL / HTTP

- hadc188：URL 选项完整，`webView` 字段解析，HTTP 层有每源 Session、代理、重试、原始字节、并发限流。
- SCX：URL 选项完整，但没有 `webView` / `webJs`；HTTP 层有每源 Cookie、代理、重试、编码、限流。

结论：普通 HTTP 书源两者都够；`webView` 书源 hadc188 更合适。

### 6.8 书源流水线

- hadc188：`book_service` 覆盖搜索/发现/详情/目录/正文/登录/封面解密/评论/RSS/TTS，完整应用级。
- SCX：`BookEngine` 覆盖搜索/发现/详情/目录/正文/精确搜索/替换，库级抽象更干净；缺 WebView/登录/封面解密/下载链接。

结论：hadc188 更“全”，SCX 更“纯”。

### 6.9 长尾能力

| 长尾 | hadc188 | SCX |
|---|---|---|
| `sourceRegex` / `webJs` | 有实现（内容后处理） | 缺失 |
| `coverDecodeJs` | 有实现 | 缺失 |
| `downloadUrls` | 有实现 | 缺失 |
| `imageDecode` / `payAction` | 仅模型 | 仅模型 |
| TTF | 缺失 | 有实现 |
| 文件/压缩包 | 部分/缺失 | 部分/缺失 |
| WebView | 有宿主桥接 | 缺失 |
| 登录/验证码 | 登录有，验证码缺 | 登录检查有，UI/验证码缺 |
| RSS | 有 | 有 |
| 替换净化 | 有 | 有 |

### 6.10 运行时安全

- hadc188：`Runtime::new()` + `set_max_stack_size`，没有内存上限、中断处理器、执行超时。恶意/死循环脚本可能挂住 JS 线程。
- SCX：`set_memory_limit`、`set_max_stack_size`、`set_interrupt_handler`、`timeout`、Context 缓存。明显更适合执行不可信书源。

对 local-reader 来说，这是 SCX 的重要优势；如果选 hadc188，必须补 JS 超时/中断/内存限制。

### 6.11 测试

- hadc188：237 个集成测试 + 27 个内联测试 + 1103 行规格；覆盖真实书源、WebView、登录、JCA、Jsoup Element API、XPath、编码、并发取消。
- SCX：1000+ A/B case + 112 真实页面 case + 48 替换 case + 12 个离线 example；对拍真实 legado 输出，兼容性证据最强。

结论：hadc188 的测试更偏“工程回归”，SCX 的测试更偏“与 Kotlin 行为对拍”。

### 6.12 许可证与构建

- local-reader：GPL-3.0。
- hadc188：MIT，兼容 GPL-3.0；纯 Rust 为主，构建简单。
- SCX：AGPL-3.0；如果链接/复制，通常需要 local-reader 整体升级为 AGPL-3.0；需要 libxml2/libclang/ICU4C，构建复杂。

## 7. 对 local-reader 的选型建议

### 7.1 推荐 A：以 `hadc188/reader` 为底座（优先）

适用条件：

- local-reader 继续使用 GPL-3.0。
- 希望 Windows 构建简单。
- 需要 WebView/登录/`sourceRegex`/`webJs`/`coverDecodeJs`/`downloadUrls`。
- 可以接受后续补 Java 正则、TTF、文件/压缩包、JS 超时。

抽取建议：

```text
src-tauri/src/legado_engine/
├── parser/          # rule_engine, rule_analyzer, html, jsonpath, js, java_compat.js
├── crawler/         # url_analyzer, fetcher, http_client, source_session
├── model/           # rule, book_source, book, book_chapter, search
└── util/            # text, hash, encoding 等
```

需要改造：

- 用 local-reader 的 `rusqlite`/仓储替换 hadc188 的 `sqlx`/服务端存储。
- 用 local-reader 的 `Book`/`BookChapter` 模型，或先扩展模型字段（`variable`、`bookUrl`、`baseUrl` 等）。
- 保留 rquickjs；不要为了 Boa 重写 Java shim。
- 补 `fancy-regex` 或 ICU 以覆盖 Java 正则。
- 补 TTF（`ttf-parser`/`fontdue`）和文件/压缩包辅助。
- 给 JS 运行时补内存限制、中断、超时。
- 把 `book_source_rules.md` 作为兼容性验收清单。

### 7.2 推荐 B：依赖 SCX（如果可升级 AGPL）

适用条件：

- local-reader 可以升级为 AGPL-3.0。
- 能接受 libxml2/libclang/ICU4C 构建依赖。
- 更看重 Java 正则、TTF、A/B 对拍、JS 运行时安全。
- 可以自己补 WebView/登录/`sourceRegex`/`webJs`/`title`/`nextChapterUrl`。

集成建议：

- 直接把 `legado-rust-engine` 作为 git/path 依赖。
- 实现 `PageFetcher` / `HttpFetcher`，复用 local-reader 的 reqwest/代理/Cookie。
- 用 `BookEngine` 的 `search` / `book_info` / `chapters` / `content`。
- 补 `title` / `nextChapterUrl` JS 绑定。
- 补 `sourceRegex` / `webJs` 内容后处理。
- 用 Tauri WebView 实现 `webView` / 登录 / 验证码。
- 把 `fixtures/legado_ab` 作为回归门禁（注意语料许可证）。

### 7.3 不建议直接使用

- `Alucard826/reader-rs`：`src/analyze/mod.rs` 注释明确是 MVP，不支持 JS/put/get/$N/{{}}，不能单独承担完整书源兼容。
- `LegadoTeam/Legado-Tauri`：268 star 但 README 基本为空，没有可用后端。
- `learnzwk/Legado-Tauri`：Vue 前端，Rust 后端不在仓库。
- `Charmillionaire/xread`：Rust 服务端，能力看起来不错，但许可证不明确，且是服务端形态；可作为参考，不作为首选底座。

### 7.4 建议的集成顺序

1. 先选一个真实书源（CSS + 正则 + JS 各一条），用 hadc188 的 `rule_engine` 做离线求值。
2. 把 `BookSource` / `Book` / `BookChapter` 映射到 local-reader 模型。
3. 接 `search` / `book_info` / `chapters` / `content` 四个 Tauri 命令。
4. 接阅读器远程章节缓存。
5. 再接 WebView/登录/TTF/文件/压缩包等长尾。

## 8. 结论与下一步

**结论：**

- 如果目标是“尽快在 local-reader 里跑通经典 JSON 书源，同时保持 GPL-3.0”，选 **`hadc188/reader`**。
- 如果目标是“引擎兼容性优先，愿意升级 AGPL 并承担原生依赖”，选 **`SCX-snow/legado-rust-engine`**。
- 如果两者都想要：**hadc188 作为代码底座，SCX 作为行为参考/对拍基准**，不要直接把 SCX 的 AGPL 代码复制进 GPL 项目。

**下一步建议：**

1. 先在 Windows 上尝试编译 `hadc188/reader` 的 `parser`/`crawler` 子集（不编译 Tauri/服务端/PDF）。
2. 用 `book-source-rules.md` 的 L2 规则用例做第一批 golden test。
3. 用 SCX 的 A/B 语料做对照测试（只作为测试参考，注意许可证）。
4. 决定许可证路线后，再开始把引擎接入 local-reader 的 `read_state` 和 UI。

## 附录 A：关键文件索引

### Kotlin

- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeRule.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeUrl.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/RuleAnalyzer.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeByJSoup.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeByXPath.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeByJSonPath.kt`
- `app/src/main/java/io/legado/app/model/analyzeRule/AnalyzeByRegex.kt`
- `app/src/main/java/io/legado/app/model/webBook/BookList.kt`
- `app/src/main/java/io/legado/app/model/webBook/BookInfo.kt`
- `app/src/main/java/io/legado/app/model/webBook/BookChapterList.kt`
- `app/src/main/java/io/legado/app/model/webBook/BookContent.kt`
- `app/src/main/java/io/legado/app/help/JsExtensions.kt`
- `app/src/main/java/io/legado/app/data/entities/BookSource.kt`
- `app/src/main/java/io/legado/app/data/entities/rule/*.kt`

### hadc188/reader

- `src/parser/rule_engine.rs`
- `src/parser/rule_analyzer.rs`
- `src/parser/html.rs`
- `src/parser/jsonpath.rs`
- `src/parser/js.rs`
- `src/parser/java_compat.js`
- `src/crawler/url_analyzer.rs`
- `src/crawler/fetcher.rs`
- `src/crawler/http_client.rs`
- `src/crawler/source_session.rs`
- `src/model/rule.rs`
- `src/model/book_source.rs`
- `src/model/book.rs`
- `src/model/book_chapter.rs`
- `src/model/search.rs`
- `src/service/book_service.rs`
- `src/service/source_browser.rs`
- `src/service/source_login.rs`
- `book-source-rules.md`
- `tests/`

### SCX-snow/legado-rust-engine

- `crates/legado-rule/src/compile.rs`
- `crates/legado-rule/src/engine.rs`
- `crates/legado-rule/src/split.rs`
- `crates/legado-rule/src/selector/css.rs`
- `crates/legado-rule/src/selector/xpath.rs`
- `crates/legado-rule/src/selector/json.rs`
- `crates/legado-rule/src/selector/index.rs`
- `crates/legado-js/src/java.rs`
- `crates/legado-js/src/javax.rs`
- `crates/legado-js/src/packages.rs`
- `crates/legado-js/src/jsoup.rs`
- `crates/legado-js/src/rules.rs`
- `crates/legado-js/src/ttf.rs`
- `crates/legado-js/src/worker.rs`
- `crates/legado-js/src/host.rs`
- `crates/legado-book/src/engine.rs`
- `crates/legado-book/src/list.rs`
- `crates/legado-book/src/info.rs`
- `crates/legado-book/src/toc.rs`
- `crates/legado-book/src/content.rs`
- `crates/legado-net/src/url.rs`
- `crates/legado-net/src/client.rs`
- `crates/legado-regex/src/lib.rs`
- `fixtures/legado_ab/`
- `fixtures/legado_ab_real/`
- `fixtures/replace_ab/`
- `scripts/run-tests.sh`

## 附录 B：A/B 统计口径

统计脚本按 `id` 关联 `legado.jsonl` 与 `rust.jsonl`：

- `状态一致`：两边 `ok` 布尔值相同。
- `两边都成功`：两边 `ok=true`。
- `结果完全一致`：两边 `ok=true` 且 `result` 字符串完全相同。
- `结果不一致`：两边 `ok=true` 但 `result` 不同。
- `状态不一致`：一边成功、一边失败。
- `两边都失败`：两边 `ok=false`。

统计结果：

| 语料 | 唯一 case | 状态一致 | 两边都成功 | 结果完全一致 | 结果不一致 | 状态不一致 | 两边都失败 |
|---|---:|---:|---:|---:|---:|---:|---:|
| `legado_ab` | 1007 | 933 | 716 | 668 | 48 | 67 | 217 |
| `legado_ab_real` | 100 | 93 | 66 | 59 | 7 | 7 | 27 |
| `replace_ab` | 22 | 22 | 22 | 22 | 0 | 0 | 0 |

再次强调：这些是仓库已录制的 baseline，不是本次重新运行；`legado_ab_real` 是偏向差异的切片，不能直接当作随机样本兼容率。
