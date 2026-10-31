import { defineStore } from "pinia";
import { kvGet, kvSet } from "@/core/application/config";

export interface GlobalConfig {
  themeMode: number;
  pageAnim: number;
  preDownloadNum: number;
  useZhLayout: boolean;
  textFullJustify: boolean;
  chineseConverterType: number;
  bookshelfSort: number;
  gridColumns: number;
  bookshelfLayout: number;
  bookGroupStyle: number;
  showUnread: boolean;
  showLastUpdateTime: boolean;
  showWaitUpCount: boolean;
  showBookshelfFastScroller: boolean;
}

const defaults: GlobalConfig = {
  themeMode: 0,
  pageAnim: 0,
  preDownloadNum: 3,
  useZhLayout: true,
  textFullJustify: true,
  chineseConverterType: 0,
  bookshelfSort: 0,
  gridColumns: 1,
  bookshelfLayout: 0,
  bookGroupStyle: 0,
  showUnread: true,
  showLastUpdateTime: true,
  showWaitUpCount: false,
  showBookshelfFastScroller: true,
};

export const useConfigStore = defineStore("config", {
  state: () => ({
    global: { ...defaults },
    bookOverrides: {} as Record<string, Partial<GlobalConfig>>,
  }),
  actions: {
    /**
     * 配置求值：书级优先，缺省回退全局（T5.3 / design 2.1.2）。
     * `useReplaceRule` 为 null 时取全局默认（由调用方处理 null 语义）。
     */
    resolve<K extends keyof GlobalConfig>(bookUrl: string | null, key: K): GlobalConfig[K] {
      if (bookUrl) {
        const v = this.bookOverrides[bookUrl]?.[key];
        if (v !== undefined && v !== null) return v;
      }
      return this.global[key];
    },

    setBookOverride(bookUrl: string, patch: Partial<GlobalConfig>) {
      this.bookOverrides[bookUrl] = { ...this.bookOverrides[bookUrl], ...patch };
    },

    /** 从持久化 KV 载入全局配置。 */
    async load() {
      const entries = Object.entries(defaults) as [keyof GlobalConfig, number | boolean][];
      for (const [key, fallback] of entries) {
        const raw = await kvGet(key);
        if (raw === null) continue;
        if (typeof fallback === "boolean") {
          (this.global[key] as boolean) = raw === "true";
        } else {
          (this.global[key] as number) = Number(raw);
        }
      }
    },

    async persist(key: keyof GlobalConfig) {
      await kvSet(key, String(this.global[key]));
    },
  },
});