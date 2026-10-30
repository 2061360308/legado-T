import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface SafeAreaInsets {
  left: number;
  top: number;
  right: number;
  bottom: number;
  cutoutLeft: number;
  cutoutTop: number;
  cutoutRight: number;
  cutoutBottom: number;
}

export interface PlatformInfo {
  os: string;
  apiLevel: number;
  abi: string;
  screenWidth: number;
  screenHeight: number;
  density: number;
  safeArea: SafeAreaInsets;
}

export interface SafDirectoryLocation {
  cancelled: boolean;
  treeUri: string;
  documentId: string;
  rootName: string;
}

export interface SafEntry {
  documentId: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  ext: string;
  supported: boolean;
}

export interface SafPrepareFile {
  name: string;
  path: string;
  size: number;
  relativePath: string;
}

export interface SafPrepareResult {
  sessionId: string;
  files: SafPrepareFile[];
  skipped: number;
}

export interface PickedImage {
  cancelled: boolean;
  path: string;
  name: string;
}

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

let cachedInfo: PlatformInfo | null = null;

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

export async function refreshPlatform(): Promise<PlatformInfo> {
  try {
    const info = await pluginInvoke<PlatformInfo>("platform_info");
    const safeArea = toCssInsets(info.safeArea ?? EMPTY_INSETS, info.density);
    cachedInfo = { ...info, safeArea };
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
}

export function getPlatformInfo(): PlatformInfo | null {
  return cachedInfo;
}

export function isAndroid(): boolean {
  return cachedInfo?.os === "android";
}

export async function refreshSafeArea(): Promise<SafeAreaInsets> {
  try {
    const raw = await pluginInvoke<SafeAreaInsets>("get_safe_area_insets");
    const insets = toCssInsets(raw, (cachedInfo?.density ?? window.devicePixelRatio) || 1);
    applyInsets(insets);
    return insets;
  } catch {
    applyInsets(EMPTY_INSETS);
    return EMPTY_INSETS;
  }
}

export async function setKeepScreenOn(
  enabled: boolean,
  timeoutSeconds?: number | null,
): Promise<void> {
  await pluginInvoke<void>("set_keep_screen_on", {
    enabled,
    timeoutSeconds: timeoutSeconds ?? null,
  });
}

export async function setScreenOrientation(
  mode: "system" | "portrait" | "landscape" | "sensor" | "reversePortrait",
): Promise<void> {
  await pluginInvoke<void>("set_screen_orientation", { mode });
}

export async function setCutoutMode(mode: "never" | "shortEdges"): Promise<void> {
  await pluginInvoke<void>("set_cutout_mode", { mode });
}

export async function setSystemBars(options: {
  hideStatusBar: boolean;
  hideNavigationBar: boolean;
  lightStatusBar?: boolean;
  lightNavigationBar?: boolean;
}): Promise<void> {
  await pluginInvoke<void>("set_system_bars", {
    hideStatusBar: options.hideStatusBar,
    hideNavigationBar: options.hideNavigationBar,
    lightStatusBar: options.lightStatusBar ?? false,
    lightNavigationBar: options.lightNavigationBar ?? false,
  });
}

export async function setSystemBarStyle(options: {
  transparentStatusBar: boolean;
  transparentNavigationBar: boolean;
  statusBarColor?: string;
  navigationBarColor?: string;
}): Promise<void> {
  if (!isAndroid()) return;
  await pluginInvoke<void>("set_system_bar_style", {
    transparentStatusBar: options.transparentStatusBar,
    transparentNavigationBar: options.transparentNavigationBar,
    statusBarColor: options.statusBarColor ?? "",
    navigationBarColor: options.navigationBarColor ?? "",
  });
}

export async function setTextZoom(percent: number): Promise<void> {
  if (!isAndroid()) return;
  await pluginInvoke<void>("set_text_zoom", {
    percent: Math.round(percent),
  });
}

export async function setLauncherIcon(icon: string): Promise<void> {
  if (!isAndroid()) return;
  await pluginInvoke<void>("set_launcher_icon", { icon });
}

export async function pickThemeImage(): Promise<PickedImage> {
  if (isAndroid()) {
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
}

export async function setReaderKeyMode(options: {
  active: boolean;
  volumeKeyEnabled: boolean;
  keyPageOnLongPress: boolean;
  disableReturnKey: boolean;
}): Promise<void> {
  await pluginInvoke<void>("set_reader_key_mode", options);
}

export async function pickSafDirectory(): Promise<SafDirectoryLocation> {
  return pluginInvoke<SafDirectoryLocation>("pick_books_directory");
}

export async function listSafDirectory(
  treeUri: string,
  documentId: string,
): Promise<SafEntry[]> {
  return pluginInvoke<SafEntry[]>("list_books_directory", { treeUri, documentId });
}

export async function prepareSafImport(
  treeUri: string,
  documentIds: string[],
): Promise<SafPrepareResult> {
  return pluginInvoke<SafPrepareResult>("prepare_books_import", {
    treeUri,
    documentIds,
  });
}

export async function cleanupSafImport(sessionId: string): Promise<void> {
  if (!sessionId) return;
  await pluginInvoke<void>("cleanup_saf_import", { sessionId });
}
