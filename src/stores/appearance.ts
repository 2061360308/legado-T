import { defineStore } from "pinia";
import { setLauncherIcon as applyLauncherIcon } from "@/platform";
import { useThemeStore } from "./theme";

export type LauncherIcon =
  | "default"
  | "launcher1"
  | "launcher2"
  | "launcher3"
  | "launcher4"
  | "launcher5"
  | "launcher6";

interface Persisted {
  launcherIcon: LauncherIcon;
  defaultCover: string | null;
  defaultCoverDark: string | null;
  coverShowName: boolean;
  coverShowNameDark: boolean;
  coverShowAuthor: boolean;
  coverShowAuthorDark: boolean;
  customWelcome: boolean;
  welcomeImage: string | null;
  welcomeImageDark: string | null;
  welcomeShowText: boolean;
  welcomeShowTextDark: boolean;
  welcomeShowIcon: boolean;
  welcomeShowIconDark: boolean;
}

const STORAGE_KEY = "local-reader:appearance";

const defaults: Persisted = {
  launcherIcon: "default",
  defaultCover: null,
  defaultCoverDark: null,
  coverShowName: true,
  coverShowNameDark: true,
  coverShowAuthor: true,
  coverShowAuthorDark: true,
  customWelcome: false,
  welcomeImage: null,
  welcomeImageDark: null,
  welcomeShowText: true,
  welcomeShowTextDark: true,
  welcomeShowIcon: true,
  welcomeShowIconDark: true,
};

function loadPersisted(): Persisted {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<Persisted>;
      return {
        launcherIcon: parsed.launcherIcon ?? defaults.launcherIcon,
        defaultCover: parsed.defaultCover ?? null,
        defaultCoverDark: parsed.defaultCoverDark ?? null,
        coverShowName: parsed.coverShowName ?? true,
        coverShowNameDark: parsed.coverShowNameDark ?? true,
        coverShowAuthor: parsed.coverShowAuthor ?? true,
        coverShowAuthorDark: parsed.coverShowAuthorDark ?? true,
        customWelcome: parsed.customWelcome ?? false,
        welcomeImage: parsed.welcomeImage ?? null,
        welcomeImageDark: parsed.welcomeImageDark ?? null,
        welcomeShowText: parsed.welcomeShowText ?? true,
        welcomeShowTextDark: parsed.welcomeShowTextDark ?? true,
        welcomeShowIcon: parsed.welcomeShowIcon ?? true,
        welcomeShowIconDark: parsed.welcomeShowIconDark ?? true,
      };
    }
  } catch {
    // ignore corrupt storage
  }
  return { ...defaults };
}

export const useAppearanceStore = defineStore("appearance", {
  state: () => loadPersisted(),
  getters: {
    activeDefaultCover(state): string | null {
      const theme = useThemeStore();
      return theme.isNight ? state.defaultCoverDark : state.defaultCover;
    },
    coverShowNameActive(state): boolean {
      const theme = useThemeStore();
      return theme.isNight ? state.coverShowNameDark : state.coverShowName;
    },
    coverShowAuthorActive(state): boolean {
      const theme = useThemeStore();
      return theme.isNight ? state.coverShowAuthorDark : state.coverShowAuthor;
    },
    activeWelcomeImage(state): string | null {
      const theme = useThemeStore();
      return theme.isNight ? state.welcomeImageDark : state.welcomeImage;
    },
    welcomeShowTextActive(state): boolean {
      const theme = useThemeStore();
      return theme.isNight ? state.welcomeShowTextDark : state.welcomeShowText;
    },
    welcomeShowIconActive(state): boolean {
      const theme = useThemeStore();
      return theme.isNight ? state.welcomeShowIconDark : state.welcomeShowIcon;
    },
  },
  actions: {
    persist() {
      const data: Persisted = {
        launcherIcon: this.launcherIcon,
        defaultCover: this.defaultCover,
        defaultCoverDark: this.defaultCoverDark,
        coverShowName: this.coverShowName,
        coverShowNameDark: this.coverShowNameDark,
        coverShowAuthor: this.coverShowAuthor,
        coverShowAuthorDark: this.coverShowAuthorDark,
        customWelcome: this.customWelcome,
        welcomeImage: this.welcomeImage,
        welcomeImageDark: this.welcomeImageDark,
        welcomeShowText: this.welcomeShowText,
        welcomeShowTextDark: this.welcomeShowTextDark,
        welcomeShowIcon: this.welcomeShowIcon,
        welcomeShowIconDark: this.welcomeShowIconDark,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
    },
    setLauncherIcon(icon: LauncherIcon) {
      this.launcherIcon = icon;
      this.persist();
      void applyLauncherIcon(icon).catch(() => undefined);
    },
    setDefaultCover(part: "day" | "night", path: string | null) {
      if (part === "day") this.defaultCover = path;
      else this.defaultCoverDark = path;
      this.persist();
    },
    setCoverShowName(part: "day" | "night", value: boolean) {
      if (part === "day") this.coverShowName = value;
      else this.coverShowNameDark = value;
      this.persist();
    },
    setCoverShowAuthor(part: "day" | "night", value: boolean) {
      if (part === "day") this.coverShowAuthor = value;
      else this.coverShowAuthorDark = value;
      this.persist();
    },
    setCustomWelcome(value: boolean) {
      this.customWelcome = value;
      this.persist();
    },
    setWelcomeImage(part: "day" | "night", path: string | null) {
      if (part === "day") this.welcomeImage = path;
      else this.welcomeImageDark = path;
      this.persist();
    },
    setWelcomeShowText(part: "day" | "night", value: boolean) {
      if (part === "day") this.welcomeShowText = value;
      else this.welcomeShowTextDark = value;
      this.persist();
    },
    setWelcomeShowIcon(part: "day" | "night", value: boolean) {
      if (part === "day") this.welcomeShowIcon = value;
      else this.welcomeShowIconDark = value;
      this.persist();
    },
  },
});
