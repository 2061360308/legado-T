# 资源来源说明

本项目的界面视觉与部分资源来自 legado 原项目（GPL-3.0）：

- 原始工程：https://github.com/gedoor/legado
- 许可证：GNU General Public License v3.0，见 `LICENSE`
- 复用的资源：
  - `app/src/main/res/layout/*.xml` 作为界面结构与视觉基准
  - `app/src/main/res/values/{colors,dimens,strings,styles}.xml` 作为设计令牌来源
  - `app/src/main/res/drawable/*`（vector/selector/shape）转换后的 SVG/图片
  - `app/src/main/assets/font/number.ttf`
  - `app/src/main/assets/bg/*` 背景图
  - `app/src/main/res/anim/anim_readbook_*` 阅读菜单动画参数

Android UI 已转为仓库内直接维护的 Vue 组件；旧的 XML 转换脚本和中间产物已移除，不再提供 `npm run android:ui`。

本项目整体按 GPL-3.0 兼容方式分发。

注意：`src-tauri/crates/engine-*` 中的书源引擎 crate 在 `Cargo.toml` 中声明 `AGPL-3.0`。分发这些 crate 时需同时遵守 AGPL-3.0；项目许可证口径后续需统一确认。
