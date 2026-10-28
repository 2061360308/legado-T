import { defineStore } from "pinia";

export type ClickAction =
  | "none"
  | "menu"
  | "next"
  | "prev"
  | "nextChapter"
  | "prevChapter"
  | "readAloudPrev"
  | "readAloudNext"
  | "addBookmark"
  | "editContent"
  | "replaceState"
  | "chapterList"
  | "search"
  | "syncProgress"
  | "readAloudToggle";

export interface ReaderPrefs {
  fontSize: number;
  titleSize: number;
  titleBold: boolean;
  fontFamily: string;
  lineSpacing: number;
  paragraphSpacing: number;
  textBold: number;
  letterSpacing: number;
  paragraphIndent: number;
  chineseMode: number;
  shareLayout: boolean;
  paddingLeft: number;
  paddingRight: number;
  paddingTop: number;
  paddingBottom: number;
  fullJustify: boolean;
  bottomJustify: boolean;
  useZhLayout: boolean;
  mouseWheelPage: boolean;
  showTitleAddition: boolean;
  pageAnim: number;
  autoReadSeconds: number;
  pageKeyPrev: string;
  pageKeyNext: string;
  showBrightness: boolean;
  pageTouchSlop: number;
  titleMode: number;
  titleTopSpacing: number;
  titleBottomSpacing: number;
  headerMode: number;
  footerMode: number;
  tipHeaderLeft: string;
  tipHeaderMiddle: string;
  tipHeaderRight: string;
  tipFooterLeft: string;
  tipFooterMiddle: string;
  tipFooterRight: string;
  tipColor: string;
  tipDividerColor: string;
  headerPaddingLeft: number;
  headerPaddingTop: number;
  headerPaddingRight: number;
  headerPaddingBottom: number;
  footerPaddingLeft: number;
  footerPaddingTop: number;
  footerPaddingRight: number;
  footerPaddingBottom: number;
  showHeaderLine: boolean;
  showFooterLine: boolean;
  clickActions: ClickAction[];
  timerMinutes: number;
  readAloudRate: number;
  readAloudPitch: number;
  readAloudByPage: boolean;
  streamReadAloudAudio: boolean;
  ignoreAudioFocus: boolean;
  ttsFollowSystem: boolean;
  ttsEngine: string;
}

const STORAGE_KEY = "local-reader:reader-prefs";
const INFO_BAR_MIGRATION_KEY = "local-reader:info-bar-migrated-v1";

const DEFAULT_ACTIONS: ClickAction[] = [
  "prev",
  "prev",
  "prev",
  "next",
  "menu",
  "next",
  "next",
  "next",
  "next",
];

const defaults: ReaderPrefs = {
  fontSize: 20,
  titleSize: 0,
  titleBold: true,
  fontFamily: 'system-ui, "Microsoft YaHei", "PingFang SC", sans-serif',
  lineSpacing: 1.2,
  paragraphSpacing: 2,
  textBold: 0,
  letterSpacing: 0.1,
  paragraphIndent: 2,
  chineseMode: 0,
  shareLayout: false,
  paddingLeft: 24,
  paddingRight: 24,
  paddingTop: 36,
  paddingBottom: 24,
  fullJustify: true,
  bottomJustify: true,
  useZhLayout: true,
  mouseWheelPage: true,
  showTitleAddition: true,
  pageAnim: 1,
  autoReadSeconds: 0,
  pageKeyPrev: "ArrowLeft",
  pageKeyNext: "ArrowRight",
  showBrightness: true,
  pageTouchSlop: 48,
  titleMode: 0,
  titleTopSpacing: 0,
  titleBottomSpacing: 0,
  headerMode: 0,
  footerMode: 0,
  tipHeaderLeft: "time",
  tipHeaderMiddle: "none",
  tipHeaderRight: "battery",
  tipFooterLeft: "chapterTitle",
  tipFooterMiddle: "none",
  tipFooterRight: "pageAndTotal",
  tipColor: "",
  tipDividerColor: "",
  headerPaddingLeft: 16,
  headerPaddingTop: 0,
  headerPaddingRight: 16,
  headerPaddingBottom: 0,
  footerPaddingLeft: 16,
  footerPaddingTop: 6,
  footerPaddingRight: 16,
  footerPaddingBottom: 6,
  showHeaderLine: false,
  showFooterLine: true,
  clickActions: [...DEFAULT_ACTIONS],
  timerMinutes: 0,
  readAloudRate: 1,
  readAloudPitch: 1,
  readAloudByPage: false,
  streamReadAloudAudio: false,
  ignoreAudioFocus: false,
  ttsFollowSystem: true,
  ttsEngine: "",
};

function load(): ReaderPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      localStorage.setItem(INFO_BAR_MIGRATION_KEY, "1");
      return { ...defaults, clickActions: [...DEFAULT_ACTIONS] };
    }
    const parsed = JSON.parse(raw) as Partial<ReaderPrefs>;
    const result: ReaderPrefs = {
      ...defaults,
      ...parsed,
      clickActions:
        Array.isArray(parsed.clickActions) && parsed.clickActions.length === 9
          ? parsed.clickActions
          : [...DEFAULT_ACTIONS],
    };
    if (!localStorage.getItem(INFO_BAR_MIGRATION_KEY)) {
      const legacyInfoBar =
        parsed.headerMode === 2 &&
        (parsed.tipHeaderLeft ?? "none") === "none" &&
        (parsed.tipHeaderMiddle ?? "none") === "none" &&
        (parsed.tipHeaderRight ?? "none") === "none" &&
        (parsed.tipFooterMiddle ?? "page") === "page" &&
        (parsed.tipFooterRight ?? "totalProgress") === "totalProgress";
      if (legacyInfoBar) {
        Object.assign(result, {
          headerMode: 0,
          footerMode: 0,
          tipHeaderLeft: "time",
          tipHeaderMiddle: "none",
          tipHeaderRight: "battery",
          tipFooterLeft: "chapterTitle",
          tipFooterMiddle: "none",
          tipFooterRight: "pageAndTotal",
        });
      }
      localStorage.setItem(INFO_BAR_MIGRATION_KEY, "1");
      localStorage.setItem(STORAGE_KEY, JSON.stringify(result));
    }
    return result;
  } catch {
    return { ...defaults, clickActions: [...DEFAULT_ACTIONS] };
  }
}

export const useReaderPrefsStore = defineStore("readerPrefs", {
  state: () => load(),
  actions: {
    persist() {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.$state));
    },
    patch(patch: Partial<ReaderPrefs>) {
      Object.assign(this, patch);
      this.persist();
    },
    resetDefaults() {
      Object.assign(this, JSON.parse(JSON.stringify(defaults)));
      this.persist();
    },
  },
});
