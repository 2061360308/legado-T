import { defineStore } from "pinia";

export enum ThemeMode {
  Day = 0,
  Night = 1,
  FollowSystem = 2,
  Eink = 3,
}

export interface ModeColors {
  bgKind: "color" | "builtin" | "external";
  bgValue: string;
  bgImage: string | null;
  textColor: string;
  darkStatusIcon: boolean;
  underline: boolean;
  bgAlpha: number;
}

export interface ReadStyleScheme {
  name: string;
  day: ModeColors;
  night: ModeColors;
  eink: ModeColors;
}

export const BUILTIN_SCHEME_COUNT = 5;

export interface GlobalThemeColors {
  primary: string;
  accent: string;
  background: string;
  bottomBackground: string;
}

export interface GlobalThemeTokens {
  day: GlobalThemeColors;
  night: GlobalThemeColors;
}

export const DEFAULT_GLOBAL_TOKENS: GlobalThemeTokens = {
  day: {
    primary: "#039BE5",
    accent: "#AD1457",
    background: "#FAFAFA",
    bottomBackground: "#EEEEEE",
  },
  night: {
    primary: "#546E7A",
    accent: "#E64A19",
    background: "#212121",
    bottomBackground: "#424242",
  },
};

function isDark(hex: string): boolean {
  const value = hex.replace("#", "");
  if (value.length !== 6) return false;
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return r + g + b < 3 * 128;
}

function colors(bg: string, text: string): ModeColors {
  return {
    bgKind: "color",
    bgValue: bg,
    bgImage: null,
    textColor: text,
    darkStatusIcon: isDark(text),
    underline: false,
    bgAlpha: 255,
  };
}

function normalizeMode(mode: Partial<ModeColors> | undefined): ModeColors {
  return {
    bgKind: mode?.bgKind ?? "color",
    bgValue: mode?.bgValue ?? "#ffffff",
    bgImage: mode?.bgImage ?? null,
    textColor: mode?.textColor ?? "#1a1a1a",
    darkStatusIcon: mode?.darkStatusIcon ?? false,
    underline: mode?.underline ?? false,
    bgAlpha: typeof mode?.bgAlpha === "number" ? mode.bgAlpha : 255,
  };
}

function normalizeSchemes(list: ReadStyleScheme[]): ReadStyleScheme[] {
  return list.map((scheme) => ({
    name: scheme.name ?? "方案",
    day: normalizeMode(scheme.day),
    night: normalizeMode(scheme.night),
    eink: normalizeMode(scheme.eink),
  }));
}

export const PRESET_SCHEMES: ReadStyleScheme[] = [
  { name: "白底黑字", day: colors("#ffffff", "#1a1a1a"), night: colors("#121212", "#e8e8e8"), eink: colors("#ffffff", "#000000") },
  { name: "米黄仿纸", day: colors("#f7f1e3", "#2f2a24"), night: colors("#1d1a16", "#d8cfc0"), eink: colors("#f7f1e3", "#000000") },
  { name: "绿色护眼", day: colors("#cce8cf", "#1e3a24"), night: colors("#102014", "#c4dcc8"), eink: colors("#e8f2e9", "#000000") },
  { name: "黑底白字", day: colors("#1e1e1e", "#e8e8e8"), night: colors("#0a0a0a", "#f0f0f0"), eink: colors("#ffffff", "#000000") },
  { name: "羊皮纸", day: colors("#f5e6c8", "#4a3728"), night: colors("#2a2118", "#e0d2b8"), eink: colors("#f5e6c8", "#000000") },
];

const STORAGE_KEY = "local-reader:theme";

interface Persisted {
  mode: ThemeMode;
  schemes: ReadStyleScheme[];
  currentScheme: number;
  globalTokens: GlobalThemeTokens;
}

function loadPersisted(): Persisted {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<Persisted>;
      const schemes =
        Array.isArray(parsed.schemes) && parsed.schemes.length > 0
          ? normalizeSchemes(parsed.schemes)
          : clonePresets();
      return {
        mode: parsed.mode ?? ThemeMode.FollowSystem,
        schemes,
        currentScheme: Math.min(Math.max(parsed.currentScheme ?? 1, 0), schemes.length - 1),
        globalTokens: parsed.globalTokens ?? cloneGlobalTokens(),
      };
    }
  } catch {
    // ignore corrupt storage
  }
  return {
    mode: ThemeMode.FollowSystem,
    schemes: clonePresets(),
    currentScheme: 1,
    globalTokens: cloneGlobalTokens(),
  };
}

function cloneGlobalTokens(): GlobalThemeTokens {
  return JSON.parse(JSON.stringify(DEFAULT_GLOBAL_TOKENS)) as GlobalThemeTokens;
}

function clonePresets(): ReadStyleScheme[] {
  return JSON.parse(JSON.stringify(PRESET_SCHEMES)) as ReadStyleScheme[];
}

export const useThemeStore = defineStore("theme", {
  state: () => {
    const persisted = loadPersisted();
    return {
      mode: persisted.mode,
      schemes: persisted.schemes,
      currentScheme: persisted.currentScheme,
      globalTokens: persisted.globalTokens,
      systemDark: window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false,
    };
  },
  getters: {
    effectiveMode(state): ThemeMode {
      if (state.mode === ThemeMode.FollowSystem) {
        return state.systemDark ? ThemeMode.Night : ThemeMode.Day;
      }
      return state.mode;
    },
    isEink(): boolean {
      return this.effectiveMode === ThemeMode.Eink;
    },
    currentSchemeData(state): ReadStyleScheme {
      return state.schemes[state.currentScheme] ?? state.schemes[0] ?? PRESET_SCHEMES[0];
    },
    readerColors(): ModeColors {
      const scheme = this.currentSchemeData;
      if (this.effectiveMode === ThemeMode.Night) return scheme.night;
      if (this.effectiveMode === ThemeMode.Eink) return scheme.eink;
      return scheme.day;
    },
  },
  actions: {
    persist() {
      const data: Persisted = {
        mode: this.mode,
        schemes: this.schemes,
        currentScheme: this.currentScheme,
        globalTokens: this.globalTokens,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
    },
    setMode(mode: ThemeMode) {
      this.mode = mode;
      this.applyToDocument();
      this.persist();
    },
    setScheme(index: number) {
      if (index < 0 || index >= this.schemes.length) return;
      this.currentScheme = index;
      this.applyToDocument();
      this.persist();
    },
    updateScheme(index: number, scheme: ReadStyleScheme) {
      this.schemes[index] = JSON.parse(JSON.stringify(scheme));
      this.applyToDocument();
      this.persist();
    },
    addScheme(name: string) {
      const base = JSON.parse(JSON.stringify(this.currentSchemeData)) as ReadStyleScheme;
      base.name = name || `方案 ${this.schemes.length + 1}`;
      this.schemes.push(base);
      this.currentScheme = this.schemes.length - 1;
      this.persist();
    },
    removeScheme(index: number) {
      if (index < 0 || index >= this.schemes.length || this.schemes.length <= 1) return false;
      this.schemes.splice(index, 1);
      if (this.currentScheme >= this.schemes.length) {
        this.currentScheme = this.schemes.length - 1;
      }
      this.persist();
      return true;
    },
    updateGlobal(
      part: "day" | "night",
      key: keyof GlobalThemeColors,
      value: string,
    ) {
      this.globalTokens[part][key] = value;
      this.applyToDocument();
      this.persist();
    },
    resetGlobal() {
      this.globalTokens = cloneGlobalTokens();
      this.applyToDocument();
      this.persist();
    },
    applyToDocument() {
      const colors = this.readerColors;
      const root = document.documentElement;
      const night = this.effectiveMode === ThemeMode.Night;
      root.classList.toggle("theme-night", night);
      root.classList.toggle("eink-mode", this.isEink);
      const globalColors = night ? this.globalTokens.night : this.globalTokens.day;
      root.style.setProperty("--ac-primary", globalColors.primary);
      root.style.setProperty("--ac-accent", globalColors.accent);
      root.style.setProperty("--ac-background", globalColors.background);
      root.style.setProperty("--ac-background-menu", globalColors.bottomBackground);
      root.style.setProperty("--lr-reader-bg", colors.bgValue);
      root.style.setProperty("--lr-reader-text", colors.textColor);
      if (this.isEink) {
        root.style.setProperty("--ac-background", "#ffffff");
        root.style.setProperty("--ac-background-card", "#ffffff");
        root.style.setProperty("--ac-background-menu", "#ffffff");
        root.style.setProperty("--ac-primaryText", "#000000");
        root.style.setProperty("--lr-color-background", "#ffffff");
        root.style.setProperty("--lr-color-on-background", "#000000");
      } else {
        root.style.removeProperty("--ac-background-card");
        root.style.removeProperty("--ac-primaryText");
        root.style.removeProperty("--lr-color-background");
        root.style.removeProperty("--lr-color-on-background");
      }
    },
    watchSystem() {
      window.matchMedia?.("(prefers-color-scheme: dark)").addEventListener?.("change", (event) => {
        this.systemDark = event.matches;
        this.applyToDocument();
      });
    },
  },
});
