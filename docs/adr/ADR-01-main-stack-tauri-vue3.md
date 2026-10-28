# ADR-01：主技术栈 = Tauri 2.x + Vue 3（唯一）

## 状态
已定案（冻结）。

## 背景
需求要求跨平台复刻 legado 的本地 TXT 阅读功能，并严苛对齐其数据格式与行为。候选技术栈曾并列比较 Flutter 与 Tauri。

## 决策
**唯一实现栈确定为 Tauri 2.x + Vue 3 + TypeScript + Vite + Pinia**：
- 后端：Rust（Tauri 主进程）负责数据、文件、编码、分章、净化、排版测量、朗读、HTTP、备份、WebDAV、JS 沙箱。
- 前端：Vue 3（WebView 内 SPA）负责全部界面、Canvas 绘制/高亮、翻页手势、弹窗与设置。
- IPC：Tauri command + event，强类型契约（见 `src/api/bindings.ts`）。
- 不做 Flutter 实现，不做双技术栈并列对照。

## 理由
- 桌面（Win/macOS/Linux）为主目标，Tauri 体积小、系统 WebView 复用、Rust 后端在高强度文本处理（分章、排版度量、正则）上性能可控。
- 前端用 Vue 3 生态快速构建复杂界面（书架、阅读器菜单、14 个定制弹窗）。
- 平台相关能力收敛为适配层，便于后续多目标（桌面/移动）复用。

## 后果
- 移动端后台朗读能力弱于原生，按 ADR-03 降级处理。
- 自绘文本需自行实现字符度量与分页，按 ADR-04 处理。
- 原 `design.md` 中「Flutter / Tauri 对照」内容需按本 ADR 同步改为 Tauri 单栈口径。