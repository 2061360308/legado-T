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

转换脚本位于 `scripts/android-ui/`，生成物位于 `src/design/` 与 `src/assets/android/`。
如需重新生成：

```bash
npm run android:ui
```

本项目整体按 GPL-3.0 兼容方式分发。
