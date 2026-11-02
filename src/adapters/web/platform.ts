import type {
  Platform,
  PlatformCapabilities,
  PlatformInfo,
  PickedFont,
  PickedImage,
  SafeAreaInsets,
  SafDirectoryLocation,
  SafEntry,
  SafPrepareResult,
} from "@/platform/contract";
import { createWebBackend } from "./backend";
import { webImportPicker } from "./importPicker";
import { pickWebFile, pickWebFiles } from "./picker";
import {
  WEB_ROOT,
  assetUrl,
  baseName,
  ensureAssetUrl,
  ensureDirectory,
  joinPath,
  primeAssetUrls,
  readWebDir,
  readWebFile,
  webFileExists,
  writeWebFile,
  writeWebStream,
} from "./storage";

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

function unsupported(command: string): never {
  throw new Error(`${command} is not implemented in the web runtime yet`);
}

function downloadBytes(path: string, data: Uint8Array<ArrayBuffer>) {
  const name = baseName(path) || "download";
  const blob = new Blob([data]);
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 30000);
}

export function createWebPlatform(): Platform {
  let cachedInfo: PlatformInfo | null = null;
  const capabilities: PlatformCapabilities = {
    android: false,
    safDirectory: false,
    systemBars: false,
    volumeKeys: false,
    backgroundAudio: false,
    opfs:
      typeof navigator !== "undefined" &&
      typeof navigator.storage?.getDirectory === "function",
    sqliteWasm: false,
    webdavProxy: true,
  };

  return {
    capabilities,
    backend: createWebBackend(),
    events: {
      async listen() {
        return () => undefined;
      },
    },
    assets: {
      fileSrc(path: string) {
        return assetUrl(path);
      },
    },
    http: {
      fetch: globalThis.fetch.bind(globalThis),
    },
    files: {
      readFile: readWebFile,
      async writeFile(path: string, data: Uint8Array<ArrayBuffer>) {
        await writeWebFile(path, data);
        if (path.startsWith(`${WEB_ROOT}/exports/`)) downloadBytes(path, data);
      },
      writeStream: writeWebStream,
      readDir: readWebDir,
      exists: webFileExists,
      async mkdir(path: string) {
        await ensureDirectory(path);
      },
      async join(...parts: string[]) {
        return joinPath(...parts);
      },
      async appDataDir() {
        return WEB_ROOT;
      },
    },
    dialog: {
      async open(options) {
        const picked = await pickWebFiles({
          multiple: options?.multiple,
          directory: options?.directory,
          filters: options?.filters,
        });
        if (!picked.length) return null;
        const paths: string[] = [];
        const session = joinPath(WEB_ROOT, "uploads", String(Date.now()));
        for (const item of picked) {
          const target = joinPath(session, baseName(item.relativePath));
          await writeWebFile(target, item.file, {
            kind: "other",
            mime: item.file.type,
          });
          paths.push(target);
        }
        return options?.multiple ? paths : paths[0] ?? null;
      },
      async save(options) {
        const name = baseName(options?.defaultPath || `backup-${Date.now()}.zip`);
        const target = joinPath(WEB_ROOT, "exports", name);
        await ensureDirectory(joinPath(WEB_ROOT, "exports"));
        return target;
      },
    },
    importPicker: webImportPicker,
    isAndroid() {
      return false;
    },
    getPlatformInfo() {
      return cachedInfo;
    },
    async refreshPlatform() {
      await navigator.storage?.persist?.().catch(() => false);
      await primeAssetUrls(["cover", "image"]).catch(() => undefined);
      cachedInfo = {
        os: "web",
        apiLevel: 0,
        abi: "wasm",
        screenWidth: window.innerWidth,
        screenHeight: window.innerHeight,
        density: window.devicePixelRatio || 1,
        safeArea: EMPTY_INSETS,
      };
      return cachedInfo;
    },
    async refreshSafeArea() {
      return EMPTY_INSETS;
    },
    async setKeepScreenOn() {},
    async setScreenOrientation() {},
    async setCutoutMode() {},
    async setSystemBars() {},
    async setSystemBarStyle() {},
    async setTextZoom() {},
    async setLauncherIcon() {},
    async pickThemeImage(): Promise<PickedImage> {
      const picked = await pickWebFile({
        filters: [
          {
            name: "图片",
            extensions: ["jpg", "jpeg", "png", "webp", "gif", "bmp"],
          },
        ],
      });
      if (!picked) return { cancelled: true, path: "", name: "" };
      const target = joinPath(
        WEB_ROOT,
        "theme",
        String(Date.now()),
        baseName(picked.file.name),
      );
      await writeWebFile(target, picked.file, {
        kind: "image",
        mime: picked.file.type,
      });
      await ensureAssetUrl(target);
      return { cancelled: false, path: target, name: picked.file.name };
    },
    async pickFontFile(): Promise<PickedFont> {
      const picked = await pickWebFile({
        filters: [{ name: "字体文件", extensions: ["ttf", "otf"] }],
      });
      if (!picked) return { cancelled: true, path: "", name: "" };
      const target = joinPath(WEB_ROOT, "font", baseName(picked.file.name));
      await writeWebFile(target, picked.file, {
        kind: "font",
        mime: picked.file.type,
      });
      return { cancelled: false, path: target, name: picked.file.name };
    },
    async setReaderKeyMode() {},
    async pickSafDirectory(): Promise<SafDirectoryLocation> {
      return unsupported("pickSafDirectory");
    },
    async listSafDirectory(): Promise<SafEntry[]> {
      return unsupported("listSafDirectory");
    },
    async scanSafDirectory(): Promise<SafEntry[]> {
      return unsupported("scanSafDirectory");
    },
    async deleteSafDocuments(): Promise<number> {
      return unsupported("deleteSafDocuments");
    },
    async prepareSafImport(): Promise<SafPrepareResult> {
      return unsupported("prepareSafImport");
    },
    async cleanupSafImport() {},
    closeWindow() {
      window.close();
    },
  };
}
