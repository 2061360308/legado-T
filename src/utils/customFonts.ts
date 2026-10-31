import { platform } from "@/runtime";

export interface FontFile {
  name: string;
  path: string;
}

export interface SystemFont {
  value: number;
  name: string;
  family: string;
}

export const ACTIVE_FONT_FAMILY = "LocalReaderFont";

export const SYSTEM_FONTS: SystemFont[] = [
  {
    value: 0,
    name: "默认字体",
    family: 'system-ui, "Microsoft YaHei", "PingFang SC", sans-serif',
  },
  {
    value: 1,
    name: "衬线字体",
    family: 'Georgia, "Songti SC", "Noto Serif SC", serif',
  },
  {
    value: 2,
    name: "等宽字体",
    family: 'ui-monospace, "Cascadia Mono", "Noto Sans Mono", monospace',
  },
];

const FONT_EXTENSIONS = new Set([".ttf", ".otf"]);
const previewFaces = new Map<string, FontFace>();
let activeFace: FontFace | null = null;
let activePath = "";

export function systemFontFamily(index: number): string {
  return (SYSTEM_FONTS[index] ?? SYSTEM_FONTS[0]).family;
}

export function customFontFamily(systemTypeface: number): string {
  return `${ACTIVE_FONT_FAMILY}, ${systemFontFamily(systemTypeface)}`;
}

function baseName(path: string): string {
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  return index >= 0 ? value.slice(index + 1) : value;
}

function extensionOf(name: string): string {
  const index = name.lastIndexOf(".");
  return index >= 0 ? name.slice(index).toLowerCase() : "";
}

function safeName(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, "_").trim() || "font.ttf";
}

function previewFamily(path: string): string {
  let hash = 0;
  for (let index = 0; index < path.length; index += 1) {
    hash = (hash * 31 + path.charCodeAt(index)) | 0;
  }
  return `LocalReaderPreview${Math.abs(hash).toString(36)}`;
}

async function fontDirectory(): Promise<string> {
  const dir = await platform.files.join(await platform.files.appDataDir(), "font");
  await platform.files.mkdir(dir, { recursive: true }).catch(() => undefined);
  return dir;
}

async function uniqueTarget(dir: string, name: string): Promise<string> {
  const safe = safeName(name);
  const dot = safe.lastIndexOf(".");
  const base = dot > 0 ? safe.slice(0, dot) : safe;
  const ext = dot > 0 ? safe.slice(dot) : "";
  let candidate = await platform.files.join(dir, safe);
  let index = 1;
  while (await platform.files.exists(candidate).catch(() => false)) {
    candidate = await platform.files.join(dir, `${base} (${index})${ext}`);
    index += 1;
  }
  return candidate;
}

export async function loadCustomFonts(): Promise<FontFile[]> {
  const dir = await fontDirectory();
  const entries = await platform.files.readDir(dir).catch(() => []);
  return entries
    .filter((entry) => entry.isFile && FONT_EXTENSIONS.has(extensionOf(entry.name)))
    .map((entry) => ({
      name: entry.name,
      path: `${dir.replace(/[\\/]+$/, "")}/${entry.name}`,
    }))
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
}

export async function loadPreviewFont(path: string): Promise<string | null> {
  const family = previewFamily(path);
  if (previewFaces.has(family)) return family;
  try {
    const bytes = await platform.files.readFile(path);
    const face = new FontFace(family, bytes);
    await face.load();
    document.fonts.add(face);
    previewFaces.set(family, face);
    return family;
  } catch {
    return null;
  }
}

export async function loadActiveFont(path: string): Promise<boolean> {
  if (!path) {
    activePath = "";
    return false;
  }
  if (path === activePath && activeFace) return true;
  try {
    const bytes = await platform.files.readFile(path);
    const face = new FontFace(ACTIVE_FONT_FAMILY, bytes);
    await face.load();
    if (activeFace) document.fonts.delete(activeFace);
    document.fonts.add(face);
    activeFace = face;
    activePath = path;
    return true;
  } catch {
    return false;
  }
}

export async function importCustomFont(): Promise<FontFile | null> {
  if (platform.isAndroid()) {
    const picked = await platform.pickFontFile();
    if (picked.cancelled || !picked.path) return null;
    return { name: picked.name || baseName(picked.path), path: picked.path };
  }

  const picked = await platform.dialog.open({
    multiple: false,
    directory: false,
    title: "选择字体文件",
    filters: [{ name: "字体文件", extensions: ["ttf", "otf"] }],
  });
  if (typeof picked !== "string" || !picked) return null;
  const dir = await fontDirectory();
  const target = await uniqueTarget(dir, baseName(picked));
  const bytes = await platform.files.readFile(picked);
  await platform.files.writeFile(target, bytes);
  return { name: baseName(target), path: target };
}
