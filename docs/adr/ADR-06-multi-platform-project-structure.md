# ADR-06：多端项目结构与平台适配

## 状态

已实施。

相关执行计划：`docs/web-migration-plan.md`。

## 背景

项目目标从 Tauri 桌面端、Android 端扩展到 Web 端。三端共享同一套阅读业务，但运行时能力不同：

- 桌面端：Tauri + Rust，本地文件、窗口、系统能力完整。
- Android 端：Tauri + Rust + Kotlin 插件，具备 SAF、系统栏、音量键等原生能力。
- Web 端：Vue + 浏览器 API，使用 IndexedDB、File API、WASM、Service Worker。

如果继续让页面、API、Rust 后端直接判断平台，平台差异会渗透到业务核心，导致后续 Web 适配和测试困难。

本 ADR 确定最终项目结构、平台边界和迁移顺序。

## 决策

### 1. 分层原则

采用“核心 + 平台契约 + 适配器 + 运行时选择”的结构：

```text
views
  -> core/application
    -> core/domain
    -> platform/contract
      <- adapters/pc | adapters/android | adapters/web
runtime.ts -> adapters
```

核心原则：

- `core/domain` 只放纯业务逻辑，不依赖平台。
- `core/application` 负责用例编排，消费 `platform` 契约。
- `platform` 只放接口、类型、能力声明，不放实现。
- `adapters` 只放平台实现，不放业务规则。
- `views` 只调用 `core/application` 和 `core/domain`。
- `runtime.ts` 是唯一选择具体适配器的地方。
- `src-tauri` 是原生后端和 Android 插件，不被前端直接引用。

### 2. 最终目录结构

```text
local-reader/
  src/
    core/
      domain/                    # 纯业务模型、纯函数、排版/规则/状态计算
        models/
        reader/
        rules/
        backup/
        ...

      application/               # 原 api 层，用例编排，消费 platform
        books.ts
        reader.ts
        backup.ts
        rules.ts
        ...

    platform/                    # 平台契约，只有接口和类型
      contract.ts
      capabilities.ts
      index.ts                   # 只 re-export 契约

    adapters/
      pc/                        # 仅桌面端 Tauri 实现
        index.ts
        storage.ts
        database.ts
        dialog.ts
        assets.ts
        system.ts

      android/                   # 仅 Android 实现（前端桥接层）
        index.ts
        saf.ts
        system.ts
        fonts.ts
        importPicker.ts
        assets.ts

      web/                       # 仅 Web 实现
        index.ts
        storage.ts               # 第一阶段 IndexedDB
        database.ts              # Dexie，后续可换 SQLite WASM
        filePicker.ts            # File API / input / 拖拽
        assets.ts                # Blob URL
        fonts.ts                 # FontFace
        http.ts                  # fetch / Worker 反代
        crypto.ts                # MD5 / AES-ECB 兼容
        wasm/
          readerCore.ts          # WASM glue
          pkg/                   # wasm-pack 产物，gitignore
        worker/
          reader.worker.ts       # 解析/排版/搜索在 Web Worker 中执行
        pwa/
          serviceWorker.ts       # 构建到 dist/sw.js

      shared/                    # 适配器之间共享的实现，不是第四个平台
        tauri/
          invoke.ts
          dialog.ts
          assets.ts
        ...

    views/                       # 现有页面和组件，只依赖 core/application
      bookshelf/
      reader/
      config/
      ...

    runtime.ts                   # 组合根：选择 pc / android / web
    main.ts

  src-tauri/
    Cargo.toml
    src/                         # Tauri 壳、native commands、native 数据层
    crates/
      reader-core/               # 纯 Rust，native + WASM 共用
    plugins/
      reader-platform/           # Android 原生插件（Kotlin + Rust bridge）
        android/
        src/
        permissions/
        build.rs
        Cargo.toml

  worker/                        # 可选 Cloudflare 反代，不存数据
    src/index.ts

  public/
    manifest.webmanifest
    icons/
```

### 3. 各层职责

#### `src/core/domain`

纯业务逻辑，必须满足：

- 不 import `@tauri-apps/*`
- 不 import `@/adapters/*`
- 不使用 `indexedDB`、`navigator`、`window`、`fetch`
- 不依赖文件系统、数据库、网络

包含：

- 书籍、章节、书签、阅读记录等模型
- TXT/EPUB/MOBI/UMD 解析后的纯数据结构
- 排版、分页、文本度量
- 替换规则匹配、简繁转换、拼音
- 阅读进度、排序、格式化等纯计算

#### `src/core/application`

原 `src/api` 的职责，负责用例编排：

- 调用 `platform` 契约
- 组合 `core/domain`
- 处理异步流程和错误
- 不直接 import 具体适配器
- 不直接 import Tauri 或浏览器 API

示例：

```ts
export async function importBook(file: File) {
  const bytes = await platform.filePicker.read(file);
  const parsed = core.parseTxt(bytes);
  await platform.storage.saveSource(parsed.book.bookUrl, bytes);
  await platform.database.upsertBook(parsed.book);
  return parsed;
}
```

#### `src/platform`

平台契约层，包含：

- `Platform` 接口
- `StoragePort`、`DatabasePort`、`FilePickerPort`、`HttpPort`、`CryptoPort`、`SystemPort`、`TtsPort`
- `PlatformCapabilities`
- 共享 DTO 类型

`platform` 不包含任何具体实现，也不判断平台名称。

#### `src/views`

页面与组件：

- 只调用 `core/application` 和 `core/domain`
- 需要平台差异时，通过 `platform.capabilities` 判断能力
- 不直接 import `@tauri-apps/*`
- 不直接操作 IndexedDB、File API、WASM

#### `src/adapters/pc`

桌面端 Tauri 实现：

- Tauri command / event
- 桌面文件对话框
- 本地文件系统
- 窗口、系统能力
- `convertFileSrc` 资源地址

#### `src/adapters/android`

Android 前端桥接层：

- 调用 `plugin:reader-platform|...`
- SAF 目录选择、递归扫描、导入准备、删除
- 系统栏、屏幕常亮、方向、启动图标
- Android 字体选择、图片选择

注意：`adapters/android` 是 TypeScript 桥接层，不是 Kotlin 插件本体。Kotlin 插件仍在：

```text
src-tauri/plugins/reader-platform/android/src/main/java/com/localreader/platform/ReaderPlatformPlugin.kt
```

#### `src/adapters/web`

Web 实现：

- 第一阶段：IndexedDB / Dexie
- 文件导入：File API、`<input type="file">`、拖拽、ZIP
- 资源：Blob URL、`FontFace`
- 计算：Web Worker + `reader-core.wasm`
- 网络：fetch；规则源/WebDAV/HTTP TTS 通过可选 Worker 反代
- 离线：Service Worker / PWA
- 加密：JS MD5、JS AES-ECB，兼容现有备份格式

#### `src/adapters/shared`

只放“至少两个适配器共用”的实现，例如：

- PC + Android 共用的 Tauri invoke
- PC + Android 共用的文件对话框封装
- PC + Android 共用的资源地址处理

判断规则：

- 只有 Web 用 → `adapters/web`
- 只有 Android 用 → `adapters/android`
- 只有 PC 用 → `adapters/pc`
- PC + Android 共用 → `adapters/shared/tauri`
- 三端共用的纯逻辑 → `core/domain`
- 三端共用的接口/类型 → `platform`

`shared` 不是第四个平台，不能被 `runtime.ts` 选择。

#### `src/runtime.ts`

组合根，唯一选择适配器的地方：

```ts
import { createPcPlatform } from "@/adapters/pc";
import { createAndroidPlatform } from "@/adapters/android";
import { createWebPlatform } from "@/adapters/web";

function selectAdapter() {
  if (import.meta.env.VITE_RUNTIME === "web") return createWebPlatform();
  if (import.meta.env.TAURI_ENV_PLATFORM === "android") return createAndroidPlatform();
  return createPcPlatform();
}

export const platform = selectAdapter();
```

构建时建议使用 Vite alias，避免 Web 包打入 Tauri 代码：

```text
@platform-impl -> adapters/web       # Web 构建
@platform-impl -> adapters/pc        # 桌面构建
@platform-impl -> adapters/android   # Android 构建
```

### 4. Rust 与 WASM 边界

新增纯 Rust crate：

```text
src-tauri/crates/reader-core/
```

职责：

- 排版、分页、文本度量
- TXT/EPUB/MOBI/UMD 纯解析
- 替换规则、简繁转换、拼音
- 搜索、内容处理

约束：

- 不依赖 Tauri
- 不依赖 `rusqlite`
- 不依赖 `std::fs`
- 不依赖 PDFium
- 可编译到 `wasm32-unknown-unknown`
- 原生端和 Web 端共用同一套核心逻辑

不要放进 `reader-core`：

- `rusqlite`：数据库使用独立 SQLite WASM 或 Dexie
- `storage.rs` 文件 IO：由平台适配器负责
- Tauri commands：由原生适配层负责
- `pdfium-render`：Web 端使用 pdf.js
- `rbook`/`ebook-rs`：先验证 WASM 编译，失败则使用 JS 库或改纯解析
- `boa_engine`：Web 已有 JS 引擎，不打包 Boa
- 正则线程超时：改为 Web Worker 超时终止

### 5. 数据与备份

#### 第一阶段存储

- 元数据：IndexedDB / Dexie
- 书文件、封面、字体、缓存：IndexedDB Blob
- 设置：IndexedDB 或 localStorage
- 后续优化：OPFS（支持时）

不把 `rusqlite` 直接编入 Web。后续如需 SQL 语义，使用独立 SQLite WASM 引擎 + OPFS，不进入 `reader-core`。

#### 备份格式

跨设备通过备份/还原同步，因此备份必须包含：

- `books.json`
- `book_groups.json`
- `bookmarks.json`
- `readRecord.json`
- 规则、主题、配置
- `local-reader-settings.json`
- 书源文件：`books/{id}/source.*`
- 封面
- 自定义字体

兼容要求：

- 旧版 JSON-only 备份仍可导入
- 书籍 ID 保持 `local:{md5}`，使用 JS/WASM MD5
- 备份密码兼容现有 `AES-128-ECB + MD5` 格式
- 新备份格式带版本号

### 6. Cloudflare Worker

Web 端不需要多设备云同步，不需要 D1/R2/KV/登录/定时任务。

Worker 仅作为可选无状态反代：

- 规则源
- WebDAV
- HTTP TTS
- 字典/规则 URL

要求：

- 不存储用户数据
- 域名白名单，避免 SSRF
- 透传认证信息，不记录敏感日志
- 大文件 WebDAV 备份注意 Worker 请求体限制，能直连时优先直连

静态前端部署到 Cloudflare Pages 即可。

### 7. 迁移顺序

1. 建立 `platform/contract`、`capabilities`、`runtime.ts` 骨架。
2. 把现有 Tauri 专用 `src/platform` 代码迁到 `adapters/pc` 和 `adapters/android`。
3. 把 `src/api` 合并为 `src/core/application`。
4. 把 `src/api` 中的纯逻辑下沉到 `src/core/domain`。
5. 抽 `src-tauri/crates/reader-core`，原生端先接入并跑通测试。
6. 建立 `adapters/web` 的 IndexedDB 实现，先支持 TXT。
7. 加 `reader-core.wasm` + `reader.worker.ts`。
8. 加字体、备份、规则、书签、阅读记录。
9. 加 EPUB/MOBI/UMD；PDF 使用 pdf.js。
10. 加 PWA、Cloudflare Pages、可选 Worker 反代。

## 后果

- 页面不再直接依赖 Tauri/浏览器 API，Web 与原生共享业务层。
- Android 原生代码仍在 `src-tauri/plugins`，Web 代码仍在 `src/adapters/web`，物理位置按运行时归属，逻辑上通过 `platform` 统一。
- 迁移期会增加一层适配器代码，但后续新增平台只需实现契约。
- Web 端 PDF 渲染与 PDFium 不完全一致，但满足“可显示”。
- Web 端文件导入不再具备 Android SAF 的后台目录监听能力，使用文件选择/拖拽/ZIP/OPFS 平替。
- Web 端第一阶段使用 IndexedDB，后续再评估 OPFS 和 SQLite WASM。

## 非目标

- 不做多设备实时同步。
- 不引入 D1/R2/KV/登录/定时任务。
- 不要求 Web 端复刻 Android 系统栏、音量键、返回键、启动图标、后台音频。
- 不要求 PDF 与 PDFium 像素级一致。
- 不在第一阶段引入 SQLite WASM/OPFS 复杂度。

## 验收标准

- `core/domain` 不依赖平台代码，可独立单测。
- `core/application` 不直接 import Tauri/浏览器 API。
- `views` 不直接 import `@tauri-apps/*`。
- `runtime.ts` 是唯一 import 具体适配器的文件。
- `src-tauri/plugins/reader-platform` 仍是 Android 原生实现。
- Web 端可导入 TXT、阅读、搜索、替换、书签、阅读记录、字体、备份/还原。
- Web 备份可与桌面/移动端互相导入。
- `npm run typecheck`、Rust 测试、Web 构建通过。
