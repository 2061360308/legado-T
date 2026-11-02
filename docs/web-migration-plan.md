# Web 端迁移开发计划

## 状态

已实施，部分浏览器/真机能力仍需验证。

相关架构决策：`docs/adr/ADR-06-multi-platform-project-structure.md`。

## 目标

- Web 端作为单机本地阅读器运行，不做多设备实时同步。
- 数据保存在浏览器本地，第一阶段使用 IndexedDB。
- 跨设备通过完整备份/还原同步。
- Rust 纯计算能力编译为 WASM，在 Web Worker 中运行。
- Cloudflare Worker 仅作为可选无状态反代，不存储数据。
- TXT 优先，EPUB/MOBI/UMD 后续，PDF 使用 `pdf.js`。

## 非目标

- 不做账号、登录、D1/R2/KV、多设备实时同步。
- 不要求 Web 端复刻 Android 系统栏、音量键、返回键、启动图标、后台音频。
- 不要求 PDF 与 PDFium 像素级一致。
- 第一阶段不引入 SQLite WASM/OPFS 复杂度。
- 不把 `rusqlite`、`std::fs`、Tauri commands、PDFium 放进 `reader-core`。

## 当前基线

- 前端通过 `src/core/application` 和 `src/runtime.ts` 调用平台适配层。
- PC/Android 数据层依赖 `rusqlite`，Web 数据层依赖 IndexedDB。
- Android 原生能力位于 `src-tauri/plugins/reader-platform`。
- Web 已具备运行时适配器、IndexedDB 数据层和 WASM 构建。

## 核心决策

- 业务核心：`src/core/domain`。
- 用例编排：`src/core/application`，原 `src/api` 合并到这里。
- 平台契约：`src/platform/contract.ts`。
- 平台实现：`src/adapters/pc`、`src/adapters/android`、`src/adapters/web`。
- 适配器共享：`src/adapters/shared`。
- 运行时选择：`src/runtime.ts`。
- Web 存储：第一阶段 IndexedDB / Dexie，后续再评估 OPFS。
- Web 文件：IndexedDB Blob；后续可迁移 OPFS。
- Web 计算：`reader-core.wasm` + `reader.worker.ts`。
- 备份：ZIP + JSON + 书文件 + 封面 + 字体。
- 书籍 ID：保持 `local:{md5}`，使用 JS/WASM MD5。
- 备份加密：兼容现有 `AES-128-ECB + MD5`。
- Worker：规则源、WebDAV、HTTP TTS、字典 URL 反代。

## Phase 0：迁移基线与契约

### 目标

固定数据格式、接口和构建入口，避免边迁移边改协议。

### 任务

- 记录现有数据库字段、书籍 ID 规则、备份 JSON 字段。
- 定义 Web 服务接口：
  - 书籍
  - 章节
  - 分组
  - 规则
  - 书签
  - 阅读记录
  - 备份
- 增加 Web 构建入口：
  - `build:web`
  - `VITE_RUNTIME=web`
- 给 Web 适配器建立空实现，保证页面能启动。
- 增加依赖约束：`views` 不直接 import `@tauri-apps/*`。

### 产出

- `src/core/application` 服务接口骨架。
- `src/platform/contract.ts`。
- `src/adapters/web` 空实现。
- `src/runtime.ts` 运行时选择。

### 验收

- `npm run build:web` 能生成静态包。
- Web 包不包含 Tauri 运行时代码。
- 页面能打开，数据层返回空数据。

## Phase 1：Web 本地数据层

### 目标

用 IndexedDB 替代 SQLite/rusqlite。

### 任务

- 建立 Dexie 数据库，表对应现有 SQLite：
  - `books`
  - `chapters`
  - `bookGroups`
  - `bookmarks`
  - `readRecords`
  - `txtTocRules`
  - `replaceRules`
  - `dictRules`
  - `httpTts`
  - `kv`
  - `settings`
  - `files`（第一阶段存 Blob）
- 实现 CRUD：
  - 书籍
  - 章节
  - 分组
  - 书签
  - 阅读记录
  - 规则
  - KV
- `bookFileSize` 改为 Blob 大小。
- 封面/图片存 Blob，通过 `URL.createObjectURL` 显示。
- 所有 Blob URL 在组件卸载时释放。

### 产出

- `src/adapters/web/database.ts`
- `src/adapters/web/storage.ts`
- `src/core/application/books.ts`
- `src/core/application/reader.ts`

### 验收

- 新增/删除/编辑书籍正常。
- 分组、书签、阅读记录、规则持久化。
- 刷新页面数据不丢。
- 删除书籍时能同时清理章节、封面、缓存。

## Phase 2：TXT 导入与阅读

### 目标

先跑通 Web 端最核心链路。

### 任务

- 文件选择、拖拽、多选、ZIP 导入。
- 读取 `ArrayBuffer`，编码探测，TXT 分章。
- 使用 MD5 生成 `local:{hash}`。
- 章节、简介、字数、封面写入 IndexedDB。
- 阅读器打开流程：
  1. 读书籍和章节
  2. 读源文件 Blob
  3. 解析/排版
  4. 生成 `ReaderSnapshot`
  5. Canvas 绘制
- 阅读进度保存。
- 目录跳转、全书搜索、替换净化。
- 重复文件识别。

### 产出

- `src/adapters/web/filePicker.ts`
- `src/core/application/import.ts`
- `src/core/application/reader.ts`
- Web 端 TXT 导入与阅读页面接通。

### 验收

- 导入 TXT 后书架出现。
- 能打开、翻页、跳章、搜索。
- 刷新后阅读进度保留。
- 重复导入同一文件能识别重复。
- 大文件解析不阻塞主线程；必要时切到 Worker。

## Phase 3：设置、主题、字体、书签、阅读记录

### 目标

补齐日常阅读功能。

### 任务

- 阅读设置：字号、行距、段距、边距、翻页模式、信息栏。
- 主题、背景色、背景图。
- 字体导入：
  - File API 选择字体
  - `FontFace` 加载
  - 字体 Blob 存 IndexedDB
- 书签新增/编辑/删除/跳转。
- 阅读记录累计时长、列表、删除、清空。
- 分组管理、书架排序、批量操作。

### 产出

- `src/adapters/web/fonts.ts`
- `src/adapters/web/assets.ts`
- 主题、字体、书签、阅读记录 Web 实现。

### 验收

- 字体重启后仍生效。
- 主题和阅读设置持久化。
- 书签、阅读记录与原生备份字段兼容。
- 删除字体后回退系统字体。

## Phase 4：规则、字典、TTS

### 目标

迁移规则类功能。

### 任务

- 替换规则 CRUD、启用/禁用、分组、排序。
- 字典规则、查词。
- 系统 TTS 使用 Web Speech API。
- HTTP TTS 使用 fetch；跨域时走可选 Worker 反代。
- 规则源/字典 URL 跨域时走 Worker 反代。

### 产出

- 规则 Web 实现。
- 字典 Web 实现。
- TTS Web 实现。
- `worker/` 反代骨架（可选）。

### 验收

- 替换规则能影响正文。
- 字典能查询并展示结果。
- 系统 TTS 能朗读。
- HTTP TTS 在允许跨域或反代下可用。

## Phase 5：备份/还原

### 目标

实现跨设备同步的核心能力。

### 任务

- 完整备份 ZIP：
  - `books.json`
  - `book_groups.json`
  - `bookmarks.json`
  - `readRecord.json`
  - 规则、主题、配置
  - `local-reader-settings.json`
  - `books/{id}/source.*`
  - 封面
  - 自定义字体
- 兼容旧版 JSON-only 备份导入。
- 使用 JS MD5 保持书籍 ID 一致。
- 使用 JS AES-128-ECB + MD5 保持加密备份兼容。
- 备份格式增加版本号。
- 导入时按主键合并，保留现有冲突策略。

### 产出

- `src/adapters/web/crypto.ts`
- `src/core/application/backup.ts`
- 备份/还原页面接通。

### 验收

- Web 导出的备份能导入桌面/移动端。
- 桌面/移动端备份能导入 Web。
- 还原后书能正常打开，不是只有书架记录。
- 旧版 JSON-only 备份仍可导入。

## Phase 6：WASM 核心

### 目标

把重计算从前端 TS 移到共享 Rust 核心。

### 任务

- 抽 `src-tauri/crates/reader-core`：
  - 排版
  - 分页
  - TXT 解析
  - 规则
  - 简繁转换
  - 拼音
- 编译 `wasm32-unknown-unknown`。
- 放 Web Worker，主线程只做 UI。
- 大文件使用 `ArrayBuffer` 传递，避免复制。
- 正则超时用 Worker 终止。

### 产出

- `reader-core` crate。
- `src/adapters/web/wasm/readerCore.ts`
- `src/adapters/web/worker/reader.worker.ts`

### 验收

- 同一 TXT、同一设置下，Web 与原生分页结果基本一致。
- 解析大文件不阻塞界面。
- 主线程无长时间任务。
- 正则异常不会卡死页面。

## Phase 7：EPUB / MOBI / UMD / PDF

### 目标

补齐格式支持。

### 任务

- EPUB：优先验证 `rbook` WASM；失败则使用 JS EPUB 库或改纯解析。
- MOBI：验证 `ebook-rs` WASM；失败则使用 JS 库。
- UMD：将 `flate2`/纯解析改为 bytes 接口，可进 WASM。
- PDF：使用 `pdf.js` 渲染页面和文本层。
- PDF 不要求与 PDFium 像素级一致。

### 产出

- EPUB/MOBI/UMD Web 解析实现。
- PDF `pdf.js` 阅读实现。

### 验收

- 各格式能导入、打开、翻页、搜索。
- PDF 能显示页面和文本。
- 不支持的格式给出明确错误提示。

## Phase 8：PWA 与 Cloudflare

### 目标

可安装、可离线、可部署。

### 任务

- `manifest.webmanifest`、图标、Service Worker。
- 缓存应用壳、WASM、字体。
- 用户数据不放入 Cache API。
- 部署到 Cloudflare Pages。
- 可选 `worker/` 反代：
  - 规则源
  - WebDAV
  - HTTP TTS
- Worker 域名白名单，不存数据，不记录敏感信息。

### 产出

- PWA 配置与 Service Worker。
- Cloudflare Pages 构建产物。
- 可选 Worker 反代。

### 验收

- 断网后能打开已缓存应用。
- 静态资源更新正常。
- WebDAV/规则源通过反代可用。
- Worker 不保存用户数据。

## Phase 9：验证与收尾

### 任务

- `npm run typecheck`
- `npm run build:web`
- Playwright 测试：
  - 导入
  - 阅读
  - 搜索
  - 备份
  - 字体
  - 规则
- 备份 round-trip：Web ↔ 桌面/移动端。
- 浏览器兼容：Chrome/Edge 必测，Safari 可选。
- 性能测试：
  - 大 TXT
  - 首次加载
  - WASM 内存
- 检查 Web 包不包含 Tauri 运行时代码。

### 验收

- 核心阅读链路无阻塞。
- 备份可互导。
- Web 端不需要 Tauri 运行。
- 所有页面无直接 `@tauri-apps/*` 引用。

## API 迁移映射

| 现有 API | Web 实现 |
|---|---|
| `bookList` / `bookGet` / `bookUpsert` / `bookDelete` | Dexie `books` |
| `chapterList` | Dexie `chapters` |
| `groupList` / `groupSave` / `groupDelete` | Dexie `bookGroups` |
| `bookmarkList` / `bookmarkAdd` / `bookmarkUpdate` / `bookmarkDelete` | Dexie `bookmarks` |
| `readRecord*` | Dexie `readRecords` |
| `txtTocRule*` / `replaceRule*` / `dictRule*` / `httpTts*` | Dexie 对应表 |
| `localImport*` | File API + 解析 + Dexie |
| `bookCoverSave` / `bookCoverRefresh` | Blob + IndexedDB |
| `bookFileSize` | Blob.size |
| `bookClearCache` | 删除章节/图片缓存 |
| `readerOpen` | Dexie + Blob + reader-core 排版 |
| `readerSaveProgress` | Dexie 更新书籍进度 |
| `readerSearchBook` | reader-core 搜索 |
| `readerEffectiveRules` | Dexie 规则 + reader-core |
| `readerSaveChapterContent` | IndexedDB 章节覆盖 |
| `backupExport` / `backupImport` | JS ZIP + Dexie + JS 加密 |
| `kvGet` / `kvSet` | Dexie `kv` |

## 数据与兼容

### IndexedDB 表

```text
books
chapters
bookGroups
bookmarks
readRecords
txtTocRules
replaceRules
dictRules
httpTts
kv
settings
files
```

### 备份格式

```text
backup.zip
  meta/
    books.json
    book_groups.json
    bookmarks.json
    readRecord.json
    txtTocRules.json
    replace_rules.json
    dictRules.json
    httpTTS.json
    themes.json
    config.xml
    local-reader-settings.json
  books/{id}/source.*
  books/{id}/cover.*
  fonts/*.ttf
```

### 兼容要求

- 旧版 JSON-only 备份可导入。
- 书籍 ID 保持 `local:{md5}`。
- 备份密码兼容 `AES-128-ECB + MD5`。
- 备份格式带版本号。
- 书文件缺失时给出明确提示，不生成“空书”。

## 风险

- EPUB/MOBI/UMD 的 Rust WASM 编译兼容性。
- IndexedDB 大文件和容量限制。
- 备份加密/MD5 与现有格式兼容。
- PDF 渲染与 PDFium 不完全一致。
- iOS Safari 的存储和文件 API 限制。
- 大 TXT/PDF 的 WASM 内存占用。
- WebDAV 反代的请求体限制和 CORS 行为。

## 建议实施顺序

1. IndexedDB 数据层
2. TXT 导入 + 阅读
3. 设置/字体/书签/阅读记录
4. 备份/还原
5. 规则/字典/TTS
6. WASM 核心
7. EPUB/MOBI/UMD
8. PDF
9. PWA + Cloudflare Pages + Worker 反代

## 完成标准

- Web 端可导入 TXT、阅读、搜索、替换、书签、阅读记录、字体、备份/还原。
- Web 备份可与桌面/移动端互相导入。
- 核心计算由 WASM/Worker 承担，主线程不阻塞。
- Web 包不依赖 Tauri 运行时。
- 数据全部保存在浏览器本地。
- 不需要账号、D1/R2/KV、多设备实时同步。
