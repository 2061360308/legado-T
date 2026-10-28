/**
 * 前后端 IPC 强类型契约（T1.3）
 *
 * 约定：
 * - 命令名（command）：snake_case，例如 `app_version`、`book_list`。
 * - 参数键：camelCase（Tauri 默认映射到 Rust snake_case）。
 * - 事件名（event）：`domain://action`，例如 `reader://progress`。
 * - 前端一律经 `src/api/` 封装调用，禁止在视图层直接 invoke 字符串。
 */

export interface AppInfo {
  version: string;
  name: string;
}

export interface ReadConfig {
  reverseToc: boolean;
  reSegment: boolean;
  pageAnim?: string | null;
  useReplaceRule?: boolean | null;
  ttsEngine?: string | null;
  splitLongChapter: boolean;
  imageStyle?: string;
  readAloudSimulate?: boolean;
  dailyChapter?: number;
}

export interface Book {
  bookUrl: string;
  name: string;
  author: string;
  origin?: string;
  originName?: string;
  charset?: string | null;
  type?: number;
  group?: number;
  tocUrl?: string;
  intro?: string | null;
  kind?: string | null;
  customTag?: string | null;
  coverUrl?: string | null;
  customCoverUrl?: string | null;
  latestChapterTitle?: string | null;
  durChapterIndex?: number;
  durChapterPos?: number;
  durChapterTitle?: string | null;
  durChapterTime?: number;
  totalChapterNum?: number;
  canUpdate?: boolean;
  order?: number;
  syncTime?: number;
  readConfig?: ReadConfig | null;
}

export interface BookChapter {
  url: string;
  title: string;
  index: number;
  isVolume?: boolean;
  start?: number;
  end?: number;
  isVip?: boolean;
  isPay?: boolean;
  tag?: string | null;
  resourceUrl?: string | null;
}

export interface BookGroup {
  groupId: number;
  groupName: string;
  order?: number;
  bookSort?: number;
  show?: boolean;
  enableRefresh?: boolean;
  coverUrl?: string | null;
}

export interface TxtTocRule {
  id: number;
  name: string;
  rule: string;
  example?: string;
  serialNumber?: number;
  enable?: boolean;
}

export interface ReplaceRule {
  id: number;
  name: string;
  group?: string;
  pattern: string;
  replacement?: string;
  scope?: string;
  excludeScope?: string;
  scopeTitle?: boolean;
  scopeContent?: boolean;
  isEnabled?: boolean;
  isRegex?: boolean;
  timeoutMillisecond?: number;
  sortOrder?: number;
}

export interface DictRule {
  name: string;
  urlRule: string;
  showRule?: string;
  enabled?: boolean;
  sortNumber?: number;
  jsLib?: string;
}

export interface HttpTts {
  id: number;
  name: string;
  url: string;
  contentType?: string;
  concurrentRate?: string;
  loginUrl?: string;
  loginUi?: string;
  header?: string;
  jsLib?: string;
  enabledCookieJar?: boolean;
  loginCheckJs?: string;
  lastUpdateTime?: number;
}

export interface LayoutStyle {
  fontSize: number;
  titleFontSize: number;
  titleBold: boolean;
  titleMode: number;
  titleTopSpacing: number;
  titleBottomSpacing: number;
  lineSpacing: number;
  paragraphSpacing: number;
  textBold: number;
  letterSpacing: number;
  paragraphIndent: number;
  paddingLeft: number;
  paddingRight: number;
  paddingTop: number;
  paddingBottom: number;
  fullJustify: boolean;
  bottomJustify: boolean;
  useZhLayout: boolean;
  viewWidth: number;
  viewHeight: number;
  imageStyle: string;
}

export interface CharPos {
  ch: string;
  x: number;
  y: number;
  width: number;
}

export interface ImageLine {
  src: string;
  width: number;
  height: number;
}

export interface TextLine {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  fontSize: number;
  bold: boolean;
  light: boolean;
  justified: boolean;
  startChar: number;
  endChar: number;
  chars: CharPos[];
  image?: ImageLine;
}

export interface LayoutPage {
  index: number;
  lines: TextLine[];
  startChar: number;
  endChar: number;
  chapterTitle: string;
}

export interface ReaderSnapshot {
  book: Book;
  chapterIndex: number;
  pageIndex: number;
  chapterTitle: string;
  pages: LayoutPage[];
  effectiveRules: ReplaceRule[];
  chapterParagraphs: string[];
  totalChapters: number;
  pageStartChar: number;
  pageEndChar: number;
  marginFellBack: boolean;
}

export interface Bookmark {
  time: number;
  bookName?: string;
  bookAuthor?: string;
  bookUrl?: string;
  chapterIndex?: number;
  chapterPos?: number;
  chapterName?: string;
  bookText?: string;
  content?: string | null;
}

export interface DiffEntry {
  key: string;
  status: "new" | "updated" | "duplicate";
}

export interface DiffReport {
  entries: DiffEntry[];
}

export type AppVersionResult = string;

export const COMMANDS = {
  appVersion: "app_version",
  bookList: "book_list",
  bookGet: "book_get",
  bookUpsert: "book_upsert",
  bookDelete: "book_delete",
  chapterList: "chapter_list",
  groupList: "group_list",
  txtTocRuleList: "txt_toc_rule_list",
  kvGet: "kv_get",
  kvSet: "kv_set",
  readerOpen: "reader_open",
  readerSaveProgress: "reader_save_progress",
  readerEffectiveRules: "reader_effective_rules",
  readerSaveChapterContent: "reader_save_chapter_content",
  txtTocRuleSave: "txt_toc_rule_save",
  txtTocRuleDelete: "txt_toc_rule_delete",
  txtTocRuleReorder: "txt_toc_rule_reorder",
  txtTocRuleRestoreDefault: "txt_toc_rule_restore_default",
  txtTocRuleImport: "txt_toc_rule_import",
  txtTocRuleExport: "txt_toc_rule_export",
  bookmarkList: "bookmark_list",
  bookmarkAdd: "bookmark_add",
  bookmarkDelete: "bookmark_delete",
  backupExport: "backup_export",
  backupImport: "backup_import",
} as const;

export const EVENTS = {
  readerProgress: "reader://progress",
  backupDone: "backup://done",
} as const;

export type CommandName = (typeof COMMANDS)[keyof typeof COMMANDS];
export type EventName = (typeof EVENTS)[keyof typeof EVENTS];