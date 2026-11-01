# 书源引擎完整接入方案

## 状态

原生主链路已落地并验证：

- `engine-*` 已迁入 `src-tauri/crates`，原生 `cargo check -p local-reader` 通过。
- 书源导入/列表/启用/删除/搜索/详情/目录/正文/加入书架/刷新已接入。
- 在线阅读已接入：正文抓取后写入章节缓存，复用本地排版与滚动阅读。
- RSS 源导入/导出/文章列表/正文已接入。
- 同源代理 `/api/engine/fetch` 已加入 `server/src/routes/engine.ts`。
- PC 已验证 141 源导入、在线搜索、加入书架、目录、正文、在线阅读、刷新、RSS 列表。
- 书源编辑已改为 8 个视图：基础、搜索、发现、详情、目录、正文、评论、JSON；保留未知字段。
- 书源调试页已接入搜索 → 详情 → 目录 → 正文全链路，并显示耗时与错误。
- 书源登录页已接入 WebView 登录、Cookie 读取/保存/清除。
- 书籍详情已接入评论、下载地址；阅读器菜单已接入购买/登录本章。
- 已验证 `npm run typecheck`、`cargo check -p local-reader`、`npm run tauri:build -- --no-bundle`。

尚未完成：

- `engine-js-wasm` 尚未构建。`rquickjs-sys` 在 `wasm32-unknown-unknown` 需要 wasm clang/wasi-sysroot；当前机器只有 Android NDK clang，缺少 wasm libc 头文件。
- `engine-web`、Web Worker、Web adapter 未接入。
- 书源编辑仍为基础 JSON 编辑器，未完成 6 Tab、调试、登录、变量、URL Option。
- 评论、购买、下载、图片、TTF、音频/图片/文件源 UI 未完成。
- Android WebView/CookieManager 未联调。

## 目标

将 `book-source-engine` 的全部能力完整接入 `local-reader`：

- PC：Tauri + 原生 Rust
- Android：Tauri + 原生 Rust + WebView
- Web：Vue + rquickjs wasm + 同源请求代理
- UI：以原版 Kotlin 页面为规格，在 Vue 中逐页替换当前占位页面

功能不裁剪、不分批交付。本文中的顺序只是依赖顺序，不是功能优先级。

## 核心决策

### 1. 引擎按层拆分

不再把 `book-source-engine` 当作单体依赖，也不继续依赖项目外的：

```text
../../book-source-engine
```

引擎拆分为纯核心、原生适配、Web 适配三层。

### 2. JS 引擎统一

PC/Android 和 Web 都使用 QuickJS/rquickjs，不引入 V8 Worker。

```text
PC/Android: engine-js-native -> rquickjs native
Web:        engine-js-wasm   -> rquickjs wasm
```

两边共享：

- `engine-js` API
- `java_compat.js`
- 宿主绑定协议
- JS 规则 fixture
- 超时、内存、异常返回契约

平台差异只保留：

- 原生 HTTP 使用 reqwest
- Web HTTP 使用同源代理
- 原生 WebView 可用
- Web WebView 明确禁用

### 3. Web 使用 rquickjs wasm

当前项目没有 rquickjs wasm 产物，但构建可行。参考：

```text
https://github.com/golemcloud/wasm-rquickjs
```

该项目的目标平台是 `wasm32-wasip2` WASI Component，不是浏览器直接加载的普通 WASM 模块。

本项目采用：

```text
wasm32-unknown-unknown
+ wasm-bindgen
+ Web Worker
```

如果该方案 spike 失败，再 fallback 到：

```text
wasm32-wasip2 + jco + 浏览器 WASI shim
```

fallback 不是首选。

建议原生和 wasm 统一到：

```text
rquickjs 0.14
QuickJS-ng 0.16.2
```

### 4. Web 请求代理

Web 端不能直接跨源请求书源。所有 HTTP 请求走同源代理：

```text
Worker -> /api/engine/fetch -> 书源站
```

代理负责：

- 跨域请求
- Cookie
- User-Agent
- Referer
- 重定向
- gzip/deflate
- 二进制响应
- 域名白名单
- 私网/云元数据拦截
- 请求/响应体积限制

代理不存储用户数据。

### 5. WebView 策略

- PC/Android：使用 Tauri WebView 实现 `WebViewBridge`
- Web：`WebViewBridge` 返回 `Unsupported`

Web 端明确禁用：

- `webView`
- `webJs`
- `sourceRegex`
- 交互式登录
- `startBrowser`
- `startBrowserAwait`

UI 必须提示“Web 端不支持该能力”，不能静默失败。

### 6. 命名

Rust crate 统一使用 `engine-*`，不使用 `legado` 前缀。

---

## 目标目录结构

### Rust

```text
src-tauri/crates/
  engine-model/          # 模型 + trait
  engine-regex/          # 正则
  engine-rule/           # CSS/XPath/JSONPath/规则切分
  engine-replace/        # 替换净化
  engine-url/            # URL 模板/UrlOption/编码
  engine-flow/           # search/explore/info/toc/content/reviews/pay/downloads
  engine-rss/            # RSS 解析/规则
  engine-js/             # JsEvaluator/JsRequest/JsValue/变量
  engine-js-native/      # rquickjs native + java_compat
  engine-js-wasm/        # rquickjs wasm + wasm-bindgen
  engine-http-native/    # reqwest/Cookie/代理/重试
  engine-http-web/       # 同源代理客户端
  engine-webview/        # WebView trait
  engine-native/         # PC/Android 组装
  engine-web/            # Web/WASM 组装
  reader-core/           # 现有
  backup-core/           # 现有
```

### 原生应用层

```text
src-tauri/src/
  engine/
    mod.rs
    manager.rs           # Pipeline/Cookie/WebView/并发/取消
    convert.rs           # engine model <-> app model
    jobs.rs              # 搜索/更新任务
    webview.rs           # Tauri WebView 实现
    stores.rs            # SQLite/File Cookie/变量/缓存

  commands/
    book_source.rs
    online_reader.rs
    rss.rs
    source_debug.rs
    source_login.rs

  data/
    db/schema.rs
    repositories/
```

### 前端

```text
src/
  core/
    domain/
      source/
        bookSource.ts
        sourceRules.ts
        search.ts
        rss.ts
        reviews.ts
      online/
        book.ts
    application/
      source/
        sources.ts
        search.ts
        onlineBooks.ts
        rss.ts
        sourceDebug.ts
        sourceLogin.ts
      books.ts
      reader.ts
    remote/                  # 现有 WebDAV/S3，保持独立

  platform/
    contract.ts              # EnginePort / WebViewPort / capabilities
    capabilities.ts

  adapters/
    shared/tauri/engine.ts   # PC/Android Tauri 命令
    pc/index.ts
    android/index.ts
    web/
      engine/
        wasm.ts              # engine-web wasm glue
        worker.ts            # rquickjs wasm
        httpProxy.ts         # /api/engine/fetch
        store.ts             # IndexedDB Cookie/变量/缓存

  views/
    source/                  # BookSource / Edit / Debug / Login
    search/                  # 在线搜索
    explore/                 # 从 misc 迁入
    rss/                     # 从 misc 迁入并扩展
    bookshelf/               # BookInfo 在线化
    toc/                     # 在线目录
    reader/                  # 在线阅读
    config/                  # replace/dict/aloud 对接
```

### 代理服务

```text
server/src/routes/engine.ts
```

---

## 依赖方向

```text
engine-model
engine-regex
engine-rule
engine-replace
engine-url
engine-js          # 只有 trait/API
engine-flow        # 只依赖 core + trait
engine-rss

engine-http-native -> engine-flow + engine-model + reqwest
engine-http-web    -> engine-flow + engine-model + proxy
engine-js-native   -> engine-js + rquickjs native
engine-js-wasm     -> engine-js + rquickjs wasm
engine-webview     -> engine-model + WebView trait

engine-native      -> engine-flow + engine-http-native + engine-js-native + engine-webview
engine-web         -> engine-flow + engine-http-web + engine-js-wasm
```

`engine-flow` 禁止直接依赖：

- `reqwest`
- `rquickjs`
- `tokio`
- `std::thread`
- `std::fs`
- Tauri

`engine-flow` 只调用以下 trait：

```text
PageFetcher
JsEvaluator
CookieStore
CacheStore
SourceVariableStore
WebViewBridge
```

---

## 引擎能力范围

必须完整覆盖原 `book-source-engine`：

- 搜索
- 发现
- 精确搜索
- 详情
- 下载链接
- 目录
- 正文
- 正文分页
- 目录分页
- 替换净化
- 评论
- 购买
- 图片抓取
- 图片解码
- TTF
- RSS
- 登录
- Cookie
- source variables
- WebView
- sourceRegex
- webJs
- 非文本源
- 音频/图片/文件源

---

## 数据模型

### 新增表

```text
book_sources
book_source_groups
rss_sources
rss_articles
source_variables
search_history
chapter_contents
```

### 扩展表

```text
books:
  variables
  wordCount
  imageStyle

chapters:
  baseUrl
  bookUrl
  variables
```

### 原则

- 书源 JSON 原样保存，UI 编辑不能丢字段
- `java.put` / source variables 持久化
- 原生 Cookie 使用 `FileCookieStore`
- Web Cookie 使用 IndexedDB/内存
- 章节正文缓存进 `chapter_contents`
- 备份包含书源、RSS、变量、章节缓存
- 旧备份继续可导入

---

## 后端命令

### 书源管理

```text
book_source_list
book_source_get
book_source_save
book_source_delete
book_source_import
book_source_export
book_source_group_list
book_source_group_save
book_source_group_delete
book_source_check
book_source_clear_cookie
```

### 搜索与发现

```text
book_source_search
book_source_search_cancel
book_source_explore
book_source_explore_cancel
```

### 在线书籍

```text
book_source_info
book_source_toc
book_source_content
book_source_update_info
book_source_update_toc
book_source_reviews
book_source_pay_action
book_source_download_urls
book_source_fetch_image
book_source_decode_image
```

### 调试与登录

```text
book_source_login
book_source_debug
book_source_eval_rule
book_source_set_variable
book_source_get_variables
```

### RSS

```text
rss_source_list
rss_source_save
rss_source_delete
rss_articles
rss_content
rss_favorite
```

所有命令要求：

- async / `spawn_blocking`
- 支持进度事件
- 支持取消
- 错误包含阶段
- 不记录 Cookie、密码、token

---

## JS 统一设计

### engine-js

```text
JsRequest {
  script
  bindings
  timeoutMs
  memoryLimit
  libKey
}

JsValue
JsEvaluator
JsVariables
```

统一绑定：

```text
src
source
book
chapter
cookie
cache
result
baseUrl
title
nextChapterUrl
page
key
rssArticle
```

### engine-js-native

- rquickjs native
- QuickJS-ng
- `java_compat.js`
- 独立 JS 线程
- 内存限制
- 超时/中断
- `java.*` / `Packages` / `JavaImporter`
- `java.ajax` 同步 reqwest
- TTF/加密/压缩

### engine-js-wasm

- rquickjs wasm
- `wasm32-unknown-unknown`
- wasm-bindgen
- Web Worker
- 不依赖 WASI
- 不依赖 `std::thread`
- 不使用 `new Function` / `eval`
- 不需要 `unsafe-eval`
- 超时由 Worker `terminate()` 兜底

### 现有 rquickjs 迁移

当前 `rquickjs` 实际只用于：

```text
src-tauri/src/domain/local_import.rs:310  run_name_script()
```

迁移方式：

- `local_import.rs` 不再直接 `use rquickjs`
- 改为调用 `engine-js` 的 `eval`
- 原生注入 `engine-js-native`
- Web 注入 `engine-js-wasm`
- 删除 `src-tauri/src/platform/js_engine.rs` 占位
- 删除 `src-tauri/Cargo.toml` 的直接 `rquickjs` 依赖

Web 文件名脚本当前缺失：

- `src/adapters/web/backend.ts` 没有读取 `nameScript`
- `src/adapters/web/importService.ts` 没有 `nameScript` 参数

需要补齐：

- `ImportView` 传 `nameScript`
- backend 读取 `nameScript`
- Worker 执行 `name = ...; author = ...`
- 与书源 JS 共用 `engine-js-wasm`

---

## Web 请求代理

### 接口

```text
POST /api/engine/fetch
```

请求：

```json
{
  "url": "https://example.com",
  "method": "GET",
  "headers": {},
  "bodyBase64": null,
  "timeoutMs": 60000
}
```

响应：

```json
{
  "status": 200,
  "headers": {},
  "bodyBase64": "...",
  "finalUrl": "https://example.com"
}
```

### 要求

- 同源部署，避免 CORS
- 域名白名单
- 禁止私网/localhost/云元数据
- 限制请求/响应体
- 支持 Cookie、Authorization、gzip/deflate、重定向、二进制
- 代理设置 User-Agent / Referer
- 代理返回 Set-Cookie 的可读形式
- 前端维护 Cookie jar
- 不存用户数据

### 同步 API

`java.ajax` / `java.get` / `java.post`：

- Worker 内用同步 XHR 调同源代理
- 或使用 `SharedArrayBuffer` + `Atomics.wait`（需要 COOP/COEP）
- 优先同步 XHR
- 不可用时该源标记 unsupported

---

## UI 覆盖

不保留占位页面。

| Kotlin | Vue |
|---|---|
| `BookSourceActivity` | `views/source/BookSourceView.vue` |
| `BookSourceEditActivity` | `views/source/BookSourceEditView.vue` |
| `BookSourceDebugActivity` | `views/source/BookSourceDebugView.vue` |
| `SourceLoginActivity` | `views/source/SourceLoginView.vue` |
| `SearchActivity` | `views/search/BookSearchView.vue` |
| `ExploreFragment` | `views/explore/ExploreView.vue` |
| `ExploreShowActivity` | `views/explore/ExploreShowView.vue` |
| `BookInfoActivity` | `views/bookshelf/BookInfoView.vue` |
| `TocActivity` | `views/toc/TocView.vue` |
| `ReadBookActivity` | `views/reader/ReaderView.vue` |
| `RssFragment` / `RssSourceActivity` | `views/rss/*` |
| `CacheActivity` | `views/source/CacheBookView.vue` |
| `ReplaceRuleActivity` | `views/replace/ReplaceView.vue` |
| `DictRuleActivity` | `views/dict/DictView.vue` |
| `SpeakEngineDialog` / `HttpTtsEditDialog` | `views/aloud/AloudView.vue` |
| `BookshelfManageActivity` | `views/bookshelf/ArrangeBooksView.vue` |

功能必须覆盖：

- 书源列表、分组、排序、启用、批量、导入导出、检查、分享、二维码
- 书源编辑 6 Tab：基础、搜索、发现、详情、目录、正文
- 变量、URL Option、调试、登录、清 Cookie、复制粘贴
- 多源搜索、精确搜索、搜索范围、历史、加入书架、换源
- 发现、分页、筛选
- 在线详情、刷新、评论
- 在线目录、刷新、倒序、缓存
- 在线正文、更新、登录、购买、图片、音频、文件源
- RSS 源、文章、正文、收藏、已读、登录
- 调试分阶段源码、规则结果
- 缓存下载、导出、清理
- 替换、字典、TTS 与引擎规则打通

---

## 执行顺序

所有功能都在本次范围内；以下是依赖顺序。

1. 建立 `engine-*` crate，迁移并拆分原引擎。
2. 原生 `engine-native` 组装，保留原 A/B 测试。
3. 构建 `engine-js-wasm` spike：eval、timeout、memory、wasm-bindgen。
4. 构建 `engine-web`，接入 rquickjs wasm。
5. 新增同源代理 `/api/engine/fetch`。
6. 新增 `src-tauri/src/engine`、数据库表、全部命令。
7. 替换全部书源、搜索、发现、详情、目录、正文、RSS、调试、登录 UI。
8. 接入 Web adapter：WASM + Worker + 代理。
9. Android WebView / CookieManager 真机验证。
10. 全量回归。

---

## 构建与验证

```powershell
cargo test --workspace
cargo check -p engine-web --target wasm32-unknown-unknown
npm run typecheck
npm run build:web
npm run tauri:build -- --no-bundle
```

Android：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-android.ps1 -Target all
```

`scripts/build-wasm.mjs` 增加：

```text
engine-web -> src/adapters/web/wasm/engine-web
```

验证内容：

- 原引擎规则级 A/B
- pipeline A/B
- cookie A/B
- replace A/B
- native vs wasm 同一 JS fixture 对比
- Web Worker eval / timeout / memory
- 代理请求测试
- 727 源导入
- PC/Android 真机 WebView 登录
- Web 端 WebView 源 unsupported 提示
- 本地书、WebDAV、备份回归

---

## 风险

1. rquickjs wasm 尚未落地，必须先做 spike。
2. `wasm32-unknown-unknown` 与 `wasm-rquickjs` 的 `wasm32-wasip2` 目标不同。
3. 同步 API 需要同步 XHR 或 Atomics 桥接。
4. Web 必须同源代理，独立代理域会遇到 CORS/preflight/credentials 问题。
5. WebView 源在 Web 明确禁用。
6. QuickJS native/wasm 版本必须对齐。
7. source variables / Cookie 持久化需要双端实现。
8. AGPL 许可需要保留来源和许可证说明。
9. rquickjs wasm 包体和内存需要实测。
10. 书源站变化需要保留 A/B 语料和 live audit。

---

## 完成标准

- 书源引擎全部能力在 PC/Android 可用。
- Web 除 WebView 相关能力外可用。
- Web 使用 rquickjs wasm，不使用 V8 Worker。
- Web 请求全部走同源代理。
- 文件名脚本和书源 JS 共用 `engine-js`。
- 不再维护两套 JS 引擎。
- 不再依赖项目外 `book-source-engine`。
- 无占位页面。
- PC/Android/Web 构建通过。
- 727 源导入、搜索、详情、目录、正文、RSS、评论、购买、下载、图片、TTF 全链路验证。
