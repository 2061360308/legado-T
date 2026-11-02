import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { appDataDir, join } from "@tauri-apps/api/path";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import { create, exists, mkdir, readDir, readFile, writeFile } from "@tauri-apps/plugin-fs";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { tauriImportPicker } from "./bookImportPicker";
import type {
  DialogOpenOptions,
  DialogSaveOptions,
  Platform,
  PlatformCapabilities,
  PlatformInfo,
  PickedFont,
  PickedImage,
  SafeAreaInsets,
  SafDirectoryLocation,
  SafEntry,
  SafPrepareResult,
  Unlisten,
} from "@/platform/contract";

const EMPTY_INSETS: SafeAreaInsets = {
  left: 0,
  top: 0,
  right: 0,
  bottom: 0,
  cutoutLeft: 0,
  cutoutTop: 0,
  cutoutRight: 0,
  cutoutBottom: 0,
};

function pluginInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(`plugin:reader-platform|${command}`, args);
}

function toCssInsets(insets: SafeAreaInsets, density: number): SafeAreaInsets {
  const scale = density > 0 ? density : 1;
  return {
    left: insets.left / scale,
    top: insets.top / scale,
    right: insets.right / scale,
    bottom: insets.bottom / scale,
    cutoutLeft: insets.cutoutLeft / scale,
    cutoutTop: insets.cutoutTop / scale,
    cutoutRight: insets.cutoutRight / scale,
    cutoutBottom: insets.cutoutBottom / scale,
  };
}

function applyInsets(insets: SafeAreaInsets) {
  const root = document.documentElement;
  root.style.setProperty("--safe-left", `${insets.left}px`);
  root.style.setProperty("--safe-top", `${insets.top}px`);
  root.style.setProperty("--safe-right", `${insets.right}px`);
  root.style.setProperty("--safe-bottom", `${insets.bottom}px`);
  root.style.setProperty("--cutout-left", `${insets.cutoutLeft}px`);
  root.style.setProperty("--cutout-top", `${insets.cutoutTop}px`);
  root.style.setProperty("--cutout-right", `${insets.cutoutRight}px`);
  root.style.setProperty("--cutout-bottom", `${insets.cutoutBottom}px`);
}

export function createTauriPlatform(): Platform {
  let cachedInfo: PlatformInfo | null = null;
  const capabilities: PlatformCapabilities = {
    android: false,
    safDirectory: false,
    systemBars: false,
    volumeKeys: false,
    backgroundAudio: false,
    opfs: false,
    sqliteWasm: false,
    webdavProxy: false,
  };

  const platform: Platform = {
    capabilities,
    backend: {
      invoke,
    },
    events: {
      async listen<T>(event: string, cb: (payload: T) => void): Promise<Unlisten> {
        return listen<T>(event, (payload) => cb(payload.payload));
      },
    },
    assets: {
      fileSrc: convertFileSrc,
    },
    http: {
      fetch: tauriFetch,
    },
    files: {
      readFile,
      writeFile,
      async writeStream(path: string, stream: ReadableStream<Uint8Array>) {
        const file = await create(path);
        const reader = stream.getReader();
        try {
          for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            await file.write(value);
          }
        } finally {
          await file.close();
        }
      },
      readDir,
      exists,
      mkdir,
      join,
      appDataDir,
    },
    dialog: {
      async open(options?: DialogOpenOptions) {
        return open(options);
      },
      async save(options?: DialogSaveOptions) {
        return save(options);
      },
    },
    importPicker: tauriImportPicker,
    isAndroid() {
      return cachedInfo?.os === "android";
    },
    getPlatformInfo() {
      return cachedInfo;
    },
    async refreshPlatform() {
      try {
        const info = await pluginInvoke<PlatformInfo>("platform_info");
        const safeArea = toCssInsets(info.safeArea ?? EMPTY_INSETS, info.density);
        cachedInfo = { ...info, safeArea };
        capabilities.android = info.os === "android";
        capabilities.safDirectory = capabilities.android;
        capabilities.systemBars = capabilities.android;
        capabilities.volumeKeys = capabilities.android;
        capabilities.backgroundAudio = capabilities.android;
        applyInsets(safeArea);
        return cachedInfo;
      } catch {
        cachedInfo = {
          os: "desktop",
          apiLevel: 0,
          abi: "",
          screenWidth: window.innerWidth,
          screenHeight: window.innerHeight,
          density: window.devicePixelRatio || 1,
          safeArea: EMPTY_INSETS,
        };
        applyInsets(EMPTY_INSETS);
        return cachedInfo;
      }
    },
    async refreshSafeArea() {
      try {
        const raw = await pluginInvoke<SafeAreaInsets>("get_safe_area_insets");
        const insets = toCssInsets(
          raw,
          (cachedInfo?.density ?? window.devicePixelRatio) || 1,
        );
        applyInsets(insets);
        return insets;
      } catch {
        applyInsets(EMPTY_INSETS);
        return EMPTY_INSETS;
      }
    },
    async setKeepScreenOn(enabled, timeoutSeconds) {
      await pluginInvoke<void>("set_keep_screen_on", {
        enabled,
        timeoutSeconds: timeoutSeconds ?? null,
      });
    },
    async setScreenOrientation(mode) {
      await pluginInvoke<void>("set_screen_orientation", { mode });
    },
    async setCutoutMode(mode) {
      await pluginInvoke<void>("set_cutout_mode", { mode });
    },
    async setSystemBars(options) {
      await pluginInvoke<void>("set_system_bars", {
        hideStatusBar: options.hideStatusBar,
        hideNavigationBar: options.hideNavigationBar,
        lightStatusBar: options.lightStatusBar ?? false,
        lightNavigationBar: options.lightNavigationBar ?? false,
      });
    },
    async setSystemBarStyle(options) {
      if (!platform.isAndroid()) return;
      await pluginInvoke<void>("set_system_bar_style", {
        transparentStatusBar: options.transparentStatusBar,
        transparentNavigationBar: options.transparentNavigationBar,
        statusBarColor: options.statusBarColor ?? "",
        navigationBarColor: options.navigationBarColor ?? "",
      });
    },
    async setTextZoom(percent) {
      if (!platform.isAndroid()) return;
      await pluginInvoke<void>("set_text_zoom", { percent: Math.round(percent) });
    },
    async setLauncherIcon(icon) {
      if (!platform.isAndroid()) return;
      await pluginInvoke<void>("set_launcher_icon", { icon });
    },
    async pickThemeImage() {
      if (platform.isAndroid()) {
        return pluginInvoke<PickedImage>("pick_image");
      }
      const picked = await open({
        multiple: false,
        directory: false,
        title: "选择背景图片",
        filters: [
          {
            name: "图片",
            extensions: ["jpg", "jpeg", "png", "webp", "gif", "bmp"],
          },
        ],
      });
      if (typeof picked !== "string" || !picked) {
        return { cancelled: true, path: "", name: "" };
      }
      const parts = picked.split(/[\\/]/);
      return {
        cancelled: false,
        path: picked,
        name: parts[parts.length - 1] ?? picked,
      };
    },
    async pickFontFile() {
      return pluginInvoke<PickedFont>("pick_font_file");
    },
    async setReaderKeyMode(options) {
      await pluginInvoke<void>("set_reader_key_mode", options);
    },
    async startBackgroundAudio(title) {
      await pluginInvoke<void>("start_background_audio", { title });
    },
    async stopBackgroundAudio() {
      await pluginInvoke<void>("stop_background_audio");
    },
    async pickSafDirectory() {
      return pluginInvoke<SafDirectoryLocation>("pick_books_directory");
    },
    async listSafDirectory(treeUri, documentId) {
      return pluginInvoke<SafEntry[]>("list_books_directory", {
        treeUri,
        documentId,
      });
    },
    async scanSafDirectory(treeUri, documentId) {
      return pluginInvoke<SafEntry[]>("scan_books_directory", {
        treeUri,
        documentId,
      });
    },
    async deleteSafDocuments(treeUri, documentIds) {
      const result = await pluginInvoke<{ count: number }>("delete_saf_documents", {
        treeUri,
        documentIds,
      });
      return result.count;
    },
    async prepareSafImport(treeUri, documentIds) {
      return pluginInvoke<SafPrepareResult>("prepare_books_import", {
        treeUri,
        documentIds,
      });
    },
    async cleanupSafImport(sessionId) {
      if (!sessionId) return;
      await pluginInvoke<void>("cleanup_saf_import", { sessionId });
    },
    closeWindow() {
      void getCurrentWindow().close();
    },
  };

  return platform;
}
