# Android XML 全量复刻执行计划

## 1. 目标

以 legado 原始 `app/src/main/res` 为唯一视觉与结构依据，在 `local-reader` 中逐 XML 语义转化。未实现的业务功能保留完整视觉壳子，交互按钮统一提示“功能暂未接入”。

## 2. 范围

共 286 个 XML：

- `res/layout`：183
- `res/layout-land`：1
- `res/menu`：90
- `res/xml`：12

`file_paths.xml`、`network_security_config.xml`、`spen_remote_actions.xml` 标记为“不适用”，不做视觉复刻。

## 3. 执行顺序

1. `res/layout/*.xml` 按文件名升序
2. `res/layout-land/*.xml`
3. `res/menu/*.xml`
4. `res/xml/*.xml`

## 4. 转换规则

1. 解析 XML 层级、宽高、weight、margin、padding、gravity、orientation、constraint。
2. 解析 `@dimen/@color/@string/@style/@drawable` 与 `values-night`。
3. 对照 Kotlin 自定义 View，理解 XML 无法表达的绘制与交互。
4. 转换映射：
   - `LinearLayout` → flex
   - `FrameLayout` → relative + absolute
   - `ConstraintLayout` → grid/flex 或基准绝对定位
   - `RecyclerView` → `v-for`
   - `ViewPager` → tabs/路由
   - 自定义 View → Vue 组件或 Canvas
   - drawable XML → SVG/CSS
5. 不新增 XML 中不存在的结构。
6. 不为了统一风格修改 XML 的边框、圆角、背景、分隔线、间距、字号。

## 5. 壳子规则

- 保留 XML 层级、文字、图标、按钮、列表项、颜色、间距。
- 不实现业务逻辑。
- 点击未实现功能统一提示“功能暂未接入”。
- 不删除 XML 中存在的入口。

## 6. 抽象规则

- 第一次出现：页面内局部实现。
- 第二次出现：立即抽到 `src/components/android/`。
- 抽取后回头替换第一次使用位置。
- 不提前规划组件清单。

## 7. 验证规则

- 执行过程中不跑 build，不截图。
- 最后执行一次 `npm run typecheck` 与 `npm run build` 作为编译兜底。
- 视觉由用户统一验收。

## 8. 交付物

- `docs/ui-port/00-inventory.md`：286 个 XML 全量清单
- `docs/ui-port/01-plan.md`：本计划
- `src/design/layouts/*.json`：XML 语义中间产物
- Vue 页面/组件实现
- 最终变更摘要

## 9. 全量清单

完整列表见 `docs/ui-port/00-inventory.md`，排序与状态逐项维护。

## 10. 对账机制（L1 契约 + L2 Kotlin 账本）

目标：XML 层已全量转换，但手写页面会与 XML 漂移。本机制把「已对齐页面」上锁，任何漂移在 `npm run android:ui` 直接失败。

### L2 Kotlin 账本

- 脚本：`scripts/android-ui/extract-binding-usage.mjs`（共享解析：`kt-scan.mjs`）；偏好定义：`parse-prefs.mjs`
- 产物：`src/design/reports/binding-usage.json`
- 内容：每个 Kotlin 类 → 自身布局（viewBinding / R.layout / 泛型 / inflate 加权 + 继承回退）、`binding.*` 引用 id、点击绑定 id、`R.menu.*` 引用；按布局/菜单汇总索引
- `unmatched` 记录未落在布局 id 上的引用（多为 adapter 克隆视图、动态创建控件）

### L1 UI 契约

- 偏好树：`parse-prefs.mjs` → `src/design/prefs/*.json`（根标签兼容 `androidx.preference.PreferenceScreen`，含 entries/entryValues/dependency；validator 断言 9 个 PreferenceScreen 非空）
- 菜单树：`parse-menu.mjs` 使用 preserveOrder 解析，保留 `<group>` 与文档顺序；validator 支持 `modes`（同一菜单按 tab/书型多套顺序）
- 契约：`scripts/android-ui/ui-contract.json`
- 校验：`scripts/android-ui/validate-ui-contract.mjs` → `src/design/reports/ui-contract-report.json`
- 规则：
  - `layout/*` 条目：源布局每个 `android:id` 必须出现在 `requiredIds`（目标视图含 `data-id="..."`）或 `exemptIds`（必须写理由）；未记账即失败
  - `menu/*` 条目：`showAsAction=always` 校验文案存在；`never` 按 XML 顺序严格校验；`optionalTitles` 为按书籍类型动态出现的项
  - 交叉校验：Kotlin 使用但契约标 exempt 的 id → warning

### 已上锁页面（15 条目，116 id / 6 菜单）

| 源 XML | 目标视图 |
| --- | --- |
| layout/view_read_menu.xml + menu/book_read.xml | ReaderView.vue |
| layout/dialog_read_book_style.xml | ReadStyleDialog.vue |
| layout/dialog_read_aloud.xml | ReadAloudPanel.vue |
| layout/item_read_style.xml | ReadStyleDialog.vue（背景方案项） |
| layout/item_bookshelf_list.xml | BookItem.vue |
| layout/fragment_bookshelf1.xml / fragment_bookshelf2.xml / menu/main_bookshelf.xml | BookshelfView.vue |
| layout/activity_chapter_list.xml / fragment_chapter_list.xml + menu/book_toc.xml（目录/书签两模式） | TocView.vue |
| layout/fragment_my_config.xml / menu/main_my.xml | ConfigView.vue |
| menu/main_explore.xml | ExploreView.vue |
| menu/main_rss.xml | RssSourcesView.vue |

### 新增/修改页面流程

1. 对照源 XML 校正布局、文案、菜单
2. 在目标视图关键节点加 `data-id="android:id"`（组件属性会落到根元素）
3. 在 `ui-contract.json` 登记 requiredIds / exemptIds / menus
4. `npm run android:ui` 必须 0 failures

## 11. 阅读器弹窗与排版复刻（与原版 Kotlin 对齐）

### FAB 行

- `view_read_menu.xml` 的 `ll_floating_button` 位于 `bottom_menu` 内、`ll_bottom_bg` 之上：左对齐、`fabSize=mini`（40dp）、每个 FAB margin 16dp、容器 padding 16dp、尾部 3 个 `weight=1` Space
- 仅 FAB 行使用 `margin-top:auto`，底部面板不得再用 auto（否则两者平分空白，FAB 会浮到屏幕中部）

### 弹窗数据来源（复用原版定义）

| 按钮 | 原版 | 当前实现 |
| --- | --- | --- |
| 界面 | `dialog_read_book_style.xml` + `ReadStyleDialog.kt` | `ReadStyleDialog.vue`（同节点顺序、DetailSeekBar、加粗/缩进/简繁/背景方案） |
| 设置 | PreferenceScreen `pref_config_read.xml` | `MoreConfigDialog.vue` 渲染 `prefs/pref_config_read.json` |
| 朗读 | `dialog_read_aloud.xml` + `ReadAloudDialog.kt` | `ReadAloudPanel.vue`（定时/语速/跟随系统/主菜单/后台/设置） |
| 朗读设置 | PreferenceScreen `pref_config_aloud.xml` | `ReadAloudConfigDialog.vue` 渲染 `prefs/pref_config_aloud.json` |

### 控件公式（与 Kotlin 一致）

- 字号：progress = textSize − 5，max 45，显示 progress + 5
- 字距：progress = letterSpacing × 100 + 50，max 100，显示 (progress − 50) / 100
- 行距：progress = lineSpacing × 10，max 20，显示 (progress − 10) / 10
- 段距：progress = paragraphSpacing，max 20，显示 progress / 10；实际间距 = fontSize × progress / 10
- 缩进：`indent` 数组 0..4（无缩进/一至四字符缩进）→ 段首缩进 N 个全角字符
- 加粗：`textBold` 0 常规 / 1 加粗 / 2 细体
- 简繁：`chineseConverterType`（0 关闭 / 1 繁转简 / 2 简转繁）写入 KV，排版缓存 key 含该值
- 朗读语速：滑条 0..45，显示 (v + 5) / 10；定时 0..180 分钟；跟随系统时禁用滑条与 ±

### 后端字段（LayoutStyle）

`textBold`、`letterSpacing`、`paragraphIndent` 已加入 Rust `LayoutStyle` 与排版引擎：

- 字距参与字符步进（advance = charWidth + letterSpacing × fontSize）
- 段首缩进按 paragraphIndent × fontSize 缩减首行可用宽度并平移 x
- 正文行 bold / light 标记由 textBold 决定，Canvas 以 600 / 300 字重绘制
- `style_key` 纳入新字段与简繁模式，确保设置变化后重新排版

### Android 专属设置

`pref_config_read` 全量 28 项保留；桌面端可生效项（中文禁则/两端对齐/底部对齐/亮度条/标题附注/鼠标滚轮翻页/翻页阈值）已接线，其余（屏幕方向/常亮/状态栏/音量键/墨水屏优化等）保留状态并提示“桌面版暂不支持”。

## 12. 原生对话框、标题栏与目录页对齐

### 对话框（AndroidDialog）

- 底部面板按原生：全宽、直角、默认无遮罩、无标题栏/关闭按钮、滑入动画；`MoreConfigDialog` 固定 360dp，界面/朗读面板 wrap
- 列表型偏好点击弹 AndroidX 风格 `PreferenceListDialog`（标题 + 单选 + 取消/确定），翻页阈值弹 `NumberPickerDialog`（0..9999）
- 嵌套叠弹（父面板不关闭）：设置 → 列表/阈值/翻页键/点击区域；界面 → 字体/信息；朗读设置 → 语音引擎
- 所有对话框经 `Teleport` 挂到 body，保证叠层顺序

### 阅读器 TitleBar

- `TitleBar` 增加 `#below` 槽，对齐 `AppBarLayout` 纵向语义：Toolbar 行显示书名，第二行显示章节名（小字）；在线书另显示章节 URL 与换源 pill（`title_bar_addition` / `tv_chapter_name` / `tv_chapter_url` / `tv_source_action`）
- 操作区顺序对齐 `book_read.xml`：在线组（换源/刷新/离线缓存）→ TXT 目录规则（`ic_exchange`）→ 设置编码（`ic_translate`，仅本地）→ 溢出（`ic_more_vert`）

### FAB 行

- 居中、`max-width: 420px`、`gap: clamp(8px, 5vw, 44px)`，小屏自适应收紧

### 目录页

- 无标题时 `TitleBar` 不再渲染 `.titles`，tabs 用 `flex:1 + justify-content:center` 居中
- 搜索切换为 Toolbar 内嵌输入：搜索时隐藏 tabs、实时过滤当前 tab，关闭后恢复
- 更多菜单按 `book_toc.xml` 全量：目录 tab（本地 txt：TXT 目录规则、拆分超长章节；反转目录、使用替换、加载字数；日志）、书签 tab（导出、导出(MD)；日志），可勾选项显示勾选
- 行为：TXT 规则跳转 `/toc-rules`；反转目录真实生效；导出生成 JSON/MD；拆分/替换/字数先持久化并提示“桌面版暂不生效”

## 13. 阅读器菜单收合、界面操作行与全宽阅读

### 底部菜单按钮（对齐原版 runMenuOut）

- 目录/朗读/界面/设置点击时先收起阅读菜单（title bar、FAB 行、亮度条随菜单隐藏），再执行目标动作
- FAB 搜索/自动翻页同样先收菜单（原版 `runMenuOut`）；FAB 替换/夜间保持菜单打开（原版行为）

### 界面弹窗顶部操作行（dialog_read_book_style.xml）

- 6 个按钮均为 `StrokeTextView` 风格：1px 描边、3px 圆角、14sp、padding 6/4，等距分布（XML 中每个按钮间有 `weight=1` Space）
- 加粗按钮显示 `中/粗/细`，当前字重字符用 accent 高亮；点击弹出 `SelectorDialog`（标题“文章字重切换”，选项 正常/粗体/细体）
- 简繁按钮显示 `简/繁`，按当前模式高亮简或繁；点击弹出 `SelectorDialog`（标题“中文简繁体转换”，选项 关闭/繁体转简体/简体转繁体）
- 缩进点击弹出 `SelectorDialog`（标题“缩进”，选项取自 `indent` 数组）
- 字体/信息在原面板上叠弹；边距/背景按原版先关闭界面面板再打开

### 阅读器全宽与窗口自适应

- 移除 760px 最大宽度限制，`viewWidth = innerWidth`、`viewHeight = innerHeight`
- `.reader-page` 固定为 100% × 100vh 且 `overflow:hidden`，背景改用当前阅读背景色（`readerBackground`）
- 拖拽调整窗口时，旧画布按容器居中拉伸，四周显示与画布相同的背景色，不再出现 app-content 黑边；260ms 后按新尺寸重新排版

## 14. 翻页动画（覆盖 / 仿真，跟手）

### 结构

- `canvas/drawPage.ts`：正文页绘制的公共实现（背景/字符/行高亮/下划线/字重），`ReaderCanvas` 复用
- `canvas/simulationCurl.ts`：逐函数移植 `SimulationPageDelegate`（`calcCornerXY`、`calcPoints`、`getCross`、当前页区域、下一页区域+阴影、折角阴影、翻起页背面），阴影颜色/宽度与原版 ARGB 常量一致
- `canvas/PageTurnOverlay.vue`：双页离屏画布 + 覆盖/仿真绘制 + 300ms 自动动画，`begin/follow/auto/showStatic` 由阅读器驱动

### 交互

- 覆盖/仿真支持跟手：水平拖动超过 `pageTouchSlop` 即进入翻页，进度随指针更新；松手按位移阈值决定完成或回弹
- 点击区、键盘、滚轮、自动翻页、上一章/下一章均走动画；章节切换先显示当前页，加载完成后播放到新章节页
- 翻页模式分流：0 覆盖 / 1 平移（保留 CSS 过渡）/ 2 仿真 / 3 滚动 / 4 无动画；墨水屏禁用
- 动画期间锁定输入并隐藏页码/信息条，避免跳变；窗口缩放会取消动画并重新排版

### 指针事件与翻页层（修复点击无响应）

- 翻页层仅作视觉呈现：.turn-layer { pointer-events: none; }，拖动/点击始终由下层 canvas-wrap 接收
- pointerdown 对 canvas-wrap 调用 setPointerCapture，pointerup/pointercancel 释放，避免翻页层出现后丢失 move/up
- 翻页状态兜底：动画启动后 1.2s 看门狗、章节加载 15s 看门狗（动画阶段重置 1.5s），onTurnDone 用 try/finally 复位 	urnActive
- rAF 绘制异常时强制 done(false)，防止覆盖层永久遮挡

### 交接无闪烁

- 仅平移模式使用 <Transition>；覆盖/仿真/滚动/无动画直接渲染画布，避免 leave 帧回显旧页
- ReaderCanvas 在 watcher 中同步重绘，翻页层卸载前底层画布已是新页
### 仿真修复（几何与性能）

- 修正裁剪：next-area 与 back-area 使用不同多边形；背面改为 p0 内裁切；卷曲前铺底避免缝隙
- 自动 NEXT 改为右侧起手（0.9w,0.9h），折角 (w,h)，与拖拽形态一致
- 动画期间 DPR 上限 1.5、复用离屏位图、pointermove 合并到 rAF、亮度烘焙进位图、隐藏底层画布
- folder 阴影颜色按 Android ARGB 修正：0x333333 的 alpha 为 0（此前误按不透明处理，导致折痕出现硬实边）
### 仿真折痕与滚轮

- 仿真触摸规则严格对齐 Kotlin：折角由 ACTION_DOWN 点经 setDirection 计算；PREV 或中 1/3 拖动强制 touchY=页底，NEXT 的 (h/3,h/2) 强制 touchY=1；松手按最后一次移动方向判定 isCancel，位移/时长照搬 onAnimStart
- 自动翻页折角与起点照搬 nextPageByAnim/prevPageByAnim（PREV 起点 (0,h)），仅保留非有限坐标兜底
- 滚轮按原版 200ms trailing 防抖；滚轮/键盘翻页速度 100ms，点击/滑动保持 300ms
- 仿真底图与 ReadView.dispatchDraw 一致：底层始终绘制当前页（curPage），再由卷曲层覆盖；避免 p0 缝隙露出目标页导致左边线被切断
- 仿真拖动坐标钳制在页面内（x∈[0,w], y∈[0,h]），模拟安卓手指无法离开屏幕：到达边缘后折痕不再跟手，松手由 onAnimStart 播放剩余动画
- 鼠标拖出窗口：window/document 的 pointerout/mouseleave 与坐标边界检测将折痕冻结在边缘，不再跟手；pointerup/lostpointercapture/window blur 时再按 isCancel 播放剩余动画
## 15. 页眉页脚（info bar）并入页面位图

- 对齐 Kotlin view_book_page.xml：页眉/页脚各三槽（左 weight=1、中居中、右右对齐）、12sp 文字、11sp 电池文字 + 描边电池图标、0.5px 分隔线
- 新增 src/views/reader/infoBar：types/format/style/draw，统一输出槽位文本、页眉页脚高度与绘制
- ReaderCanvas 与 PageTurnOverlay 共用 drawPage(infoBar)，仿真背面镜像包含页眉页脚；删除 DOM ReaderInfoBar 与自创页码指示
- 分页：paddingTop/Bottom 追加页眉/页脚高度，正文只排在分隔线之间
- 默认值对齐 Kotlin（页眉 time/none/battery，页脚 chapterTitle/none/pageAndTotal，headerLine=false，footerLine=true）；旧默认自动迁移
- 信息弹窗 TipConfigDialog 增加 totalProgress1 选项与实时 canvas 预览

### 边距/信息弹窗对齐 Kotlin

- 边距：居中弹窗（90%% 宽），页眉/正文/页脚三段，各 4 条 DetailSeekBar + 页眉/页脚显示分隔线开关
- 信息：居中弹窗，标题字号改为相对正文 0–10（默认 0），页眉/页脚显示与槽位、颜色全部使用居中 SelectorDialog，颜色支持同正文/默认/自定义+取色器
- 标题字号 = 正文字号 + titleSize；删除高亮/ReaderView 中不可达的 TipConfig 实例
- fab-row 最大宽度 600，底部与菜单栏保留 16px 间距

### 样式列表与编辑弹窗

- fab-row 改为 4 等分网格（max-width 600，底部与菜单栏 16px 间距）
- 样式列表支持横向滚动（滚轮转横向）并显示滚动条；单击选中，长按 500ms 或鼠标右击进入编辑
- BgTextConfigDialog 对齐 Kotlin：底部面板，样式名称+编辑、还原、深色状态栏/下划线开关、文字/背景颜色、导入/导出/删除、背景透明度 0–100、背景图片横列表（支持选择本地图片）
- 背景图片写入主题方案并参与静态页/翻页位图渲染，透明度生效；允许删除任意样式（至少保留一个）
- 触摸端横向滚动改为 pointer 拖拽兜底（touch-action: pan-y，保留纵向原生滚动）；鼠标滚轮/滚动条/长按/右击不变

## 16. TXT 区间读取与文本选择

- TXT 章节正文改为按 start/end 区间读取（platform::filesystem::read_file_range），不再每章 fs::read 整本文件
- 阅读器新增当前页跨行文本选择：长按选词（中文单字、英文/数字整词）、左右手柄调整、选择高亮
- 选择菜单第一版：复制、搜索、书签、词典；选择期间暂停翻页与自动翻页
- PDF 的 TextFlow 页面 + OCR 文字层架构已确定，OCR 与 PDF 接入下一轮实现

## 17. EPUB 文本导入（第一版）

- 使用 rbook 解析 EPUB 元数据、spine、TOC 标签与章节 XHTML
- 仅提取纯文本（去脚本/样式/标签、解码实体、合并空白），图片与富内容下一轮
- 章节正文写入源文件旁的 `.chapters/<index>.txt`，阅读时按缓存读取，避免每次打开重新解析
- 导入对话框支持 TXT/EPUB，新增 `import_book` IPC 命令
- EPUB 测试覆盖：元数据、TOC 标题、章节缓存、阅读器打开
