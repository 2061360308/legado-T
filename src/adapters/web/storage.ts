import { db, nowMs, type FileRow } from "./database";

export const WEB_ROOT = "idb://app-data";

const assetUrls = new Map<string, string>();
const assetPaths = new Map<string, string>();

const OPFS_ROOT_NAME = "app-data";

let opfsRootPromise: Promise<FileSystemDirectoryHandle | null> | null = null;
let migrationPromise: Promise<number> | null = null;

function opfsSupported(): boolean {
  try {
    return (
      typeof navigator !== "undefined" &&
      Boolean(navigator.storage?.getDirectory) &&
      typeof FileSystemFileHandle !== "undefined" &&
      "createWritable" in FileSystemFileHandle.prototype
    );
  } catch {
    return false;
  }
}

async function getOriginRoot(): Promise<FileSystemDirectoryHandle | null> {
  if (!opfsSupported()) return null;
  try {
    return await navigator.storage.getDirectory();
  } catch {
    return null;
  }
}

async function getOpfsRoot(): Promise<FileSystemDirectoryHandle | null> {
  if (!opfsRootPromise) {
    opfsRootPromise = (async () => {
      const origin = await getOriginRoot();
      if (!origin) return null;
      try {
        const root = await origin.getDirectoryHandle(OPFS_ROOT_NAME, { create: true });
        void navigator.storage.persist?.().catch(() => undefined);
        return root;
      } catch {
        return null;
      }
    })();
  }
  return opfsRootPromise;
}

function opfsParts(path: string): string[] {
  const value = normalizePath(path);
  if (value === WEB_ROOT) return [];
  return value.slice(WEB_ROOT.length + 1).split("/").filter(Boolean);
}

async function opfsParent(
  path: string,
  create: boolean,
): Promise<{ dir: FileSystemDirectoryHandle; name: string } | null> {
  const root = await getOpfsRoot();
  const parts = opfsParts(path);
  if (!root || !parts.length) return null;
  let dir = root;
  for (const part of parts.slice(0, -1)) {
    try {
      dir = await dir.getDirectoryHandle(part, { create });
    } catch {
      return null;
    }
  }
  return { dir, name: parts[parts.length - 1] };
}

async function ensureOpfsDirectory(path: string): Promise<void> {
  const root = await getOpfsRoot();
  if (!root) return;
  let dir = root;
  for (const part of opfsParts(path)) {
    dir = await dir.getDirectoryHandle(part, { create: true });
  }
}

async function writeOpfsBlob(path: string, blob: Blob): Promise<boolean> {
  const parent = await opfsParent(path, true);
  if (!parent) return false;
  try {
    const handle = await parent.dir.getFileHandle(parent.name, { create: true });
    const writable = await handle.createWritable();
    await writable.write(blob);
    await writable.close();
    return true;
  } catch {
    return false;
  }
}

async function readOpfsBlob(path: string): Promise<Blob | null> {
  const parent = await opfsParent(path, false);
  if (!parent) return null;
  try {
    const handle = await parent.dir.getFileHandle(parent.name);
    return await handle.getFile();
  } catch {
    return null;
  }
}

async function opfsFileExists(path: string): Promise<boolean> {
  return (await readOpfsBlob(path)) !== null;
}

async function deleteOpfsEntry(path: string): Promise<void> {
  const parent = await opfsParent(path, false);
  if (!parent) return;
  try {
    await parent.dir.removeEntry(parent.name, { recursive: true });
  } catch {
    return;
  }
}

export async function migrateLegacyFilesToOpfs(): Promise<number> {
  if (!opfsSupported()) return 0;
  if (!migrationPromise) {
    migrationPromise = (async () => {
      let migrated = 0;
      const rows = await db.files.toArray();
      for (const row of rows) {
        if (row.kind === "dir" || row.storage === "opfs" || row.blob.size === 0) {
          continue;
        }
        if (!(await writeOpfsBlob(row.path, row.blob))) continue;
        await db.files.put({ ...row, blob: new Blob(), storage: "opfs" });
        migrated += 1;
      }
      return migrated;
    })();
  }
  return migrationPromise;
}

function clean(path: string): string {
  const value = path.replace(/\\/g, "/");
  if (value.toLowerCase().startsWith("idb://")) {
    const rest = value.slice(6).replace(/\/+/g, "/").replace(/\/$/, "");
    return `idb://${rest}`;
  }
  return value.replace(/\/+/g, "/").replace(/\/$/, "");
}

export function normalizePath(path: string): string {
  const value = clean(path.trim());
  if (!value) return WEB_ROOT;
  if (value.startsWith("idb://")) return value;
  if (value.startsWith("/")) return `${WEB_ROOT}${value}`;
  return `${WEB_ROOT}/${value}`;
}

export function joinPath(...parts: string[]): string {
  return normalizePath(parts.filter(Boolean).join("/"));
}

export function parentPath(path: string): string {
  const value = normalizePath(path);
  const index = value.lastIndexOf("/");
  return index > "idb://".length ? value.slice(0, index) : WEB_ROOT;
}

export function baseName(path: string): string {
  const value = normalizePath(path);
  return value.slice(value.lastIndexOf("/") + 1);
}

export function extensionOf(path: string): string {
  const name = baseName(path);
  const index = name.lastIndexOf(".");
  return index >= 0 ? name.slice(index + 1).toLowerCase() : "";
}

export function isWebPath(path: string): boolean {
  return path.startsWith("idb://");
}

export async function ensureDirectory(path: string): Promise<void> {
  const value = normalizePath(path);
  if (value === WEB_ROOT) return;
  const parts = value.slice(WEB_ROOT.length + 1).split("/").filter(Boolean);
  let current = WEB_ROOT;
  for (const part of parts) {
    current = `${current}/${part}`;
    const existing = await db.files.get(current);
    if (!existing) {
      await db.files.put({
        path: current,
        kind: "dir",
        name: part,
        size: 0,
        mime: "",
        blob: new Blob(),
        updated: nowMs(),
      });
    }
  }
  if (opfsSupported()) {
    await ensureOpfsDirectory(value).catch(() => undefined);
  }
}

export async function writeWebFile(
  path: string,
  data: Blob | ArrayBuffer | Uint8Array<ArrayBuffer>,
  options: {
    kind?: FileRow["kind"];
    bookUrl?: string | null;
    mime?: string;
  } = {},
): Promise<FileRow> {
  const value = normalizePath(path);
  await ensureDirectory(parentPath(value));
  const blob =
    data instanceof Blob
      ? data
      : new Blob([data], { type: options.mime ?? "application/octet-stream" });
  const row: FileRow = {
    path: value,
    bookUrl: options.bookUrl ?? null,
    kind: options.kind ?? "other",
    name: baseName(value),
    size: blob.size,
    mime: options.mime ?? blob.type,
    blob,
    storage: "idb",
    updated: nowMs(),
  };
  if (opfsSupported()) {
    if (await writeOpfsBlob(value, blob)) {
      row.blob = new Blob();
      row.storage = "opfs";
    } else {
      await deleteOpfsEntry(value);
    }
  }
  await db.files.put(row);
  return row;
}

export async function writeWebStream(
  path: string,
  stream: ReadableStream<Uint8Array>,
): Promise<void> {
  const value = normalizePath(path);
  await ensureDirectory(parentPath(value));
  if (opfsSupported()) {
    const parent = await opfsParent(value, true);
    if (parent) {
      const handle = await parent.dir.getFileHandle(parent.name, { create: true });
      const writable = await handle.createWritable();
      await stream.pipeTo(writable);
      const file = await handle.getFile();
      await db.files.put({
        path: value,
        bookUrl: null,
        kind: "other",
        name: baseName(value),
        size: file.size,
        mime: file.type || "application/octet-stream",
        blob: new Blob(),
        storage: "opfs",
        updated: nowMs(),
      });
      return;
    }
  }
  const blob = await new Response(stream).blob();
  await writeWebFile(value, blob);
}

export async function readWebFile(path: string): Promise<Uint8Array<ArrayBuffer>> {
  const blob = await readWebBlob(path);
  return new Uint8Array(await blob.arrayBuffer());
}

export async function readWebBlob(path: string): Promise<Blob> {
  const value = normalizePath(path);
  const row = await db.files.get(value);
  if (row?.kind === "dir") throw new Error(`文件不存在: ${path}`);
  const opfsBlob = await readOpfsBlob(value);
  if (opfsBlob) {
    if (row && row.storage !== "opfs") {
      await db.files.put({ ...row, blob: new Blob(), storage: "opfs" });
    }
    return opfsBlob;
  }
  if (!row) throw new Error(`文件不存在: ${path}`);
  if (row.storage !== "opfs" && row.blob.size > 0 && opfsSupported()) {
    if (await writeOpfsBlob(value, row.blob)) {
      await db.files.put({ ...row, blob: new Blob(), storage: "opfs" });
    }
  }
  return row.blob;
}

export async function webFileExists(path: string): Promise<boolean> {
  const value = normalizePath(path);
  if (await db.files.get(value)) return true;
  if (await opfsFileExists(value)) return true;
  const prefix = `${value}/`;
  return (await db.files.where("path").startsWith(prefix).count()) > 0;
}

export async function readWebDir(
  path: string,
): Promise<{ name: string; isFile: boolean; isDirectory: boolean }[]> {
  const value = normalizePath(path);
  const prefix = `${value}/`;
  const rows = await db.files.where("path").startsWith(prefix).toArray();
  const entries = new Map<string, { name: string; isFile: boolean; isDirectory: boolean }>();
  for (const row of rows) {
    const rest = row.path.slice(prefix.length);
    if (!rest) continue;
    const slash = rest.indexOf("/");
    if (slash >= 0) {
      const name = rest.slice(0, slash);
      entries.set(name, { name, isFile: false, isDirectory: true });
    } else {
      entries.set(rest, {
        name: rest,
        isFile: row.kind !== "dir",
        isDirectory: row.kind === "dir",
      });
    }
  }
  return [...entries.values()].sort((left, right) =>
    left.name.localeCompare(right.name, "zh-CN"),
  );
}

export async function deleteWebPath(path: string): Promise<void> {
  const value = normalizePath(path);
  const rows = await db.files.where("path").startsWith(`${value}/`).toArray();
  const exact = await db.files.get(value);
  const all = exact ? [exact, ...rows] : rows;
  for (const row of all) {
    releaseAssetUrl(row.path);
  }
  await deleteOpfsEntry(value);
  for (const row of all) {
    if (row.path !== value) await deleteOpfsEntry(row.path);
  }
  await db.files.bulkDelete(all.map((row) => row.path));
}

export async function deleteBookFiles(bookUrl: string): Promise<void> {
  const rows = await db.files.where("bookUrl").equals(bookUrl).toArray();
  for (const row of rows) releaseAssetUrl(row.path);
  await deleteOpfsEntry(joinPath(WEB_ROOT, "books", bookUrl));
  for (const row of rows) await deleteOpfsEntry(row.path);
  await db.files.bulkDelete(rows.map((row) => row.path));
}

export async function webFileSize(path: string): Promise<number> {
  const value = normalizePath(path);
  const row = await db.files.get(value);
  if (row) return row.size;
  return (await readOpfsBlob(value))?.size ?? 0;
}

export async function ensureAssetUrl(path: string): Promise<string> {
  if (!isWebPath(path)) return path;
  const cached = assetUrls.get(path);
  if (cached) return cached;
  try {
    const blob = await readWebBlob(path);
    const url = URL.createObjectURL(blob);
    assetUrls.set(path, url);
    assetPaths.set(url, path);
    return url;
  } catch {
    return path;
  }
}

export function assetUrl(path: string): string {
  return assetUrls.get(normalizePath(path)) ?? path;
}

export function pathForAssetUrl(url: string): string | null {
  return assetPaths.get(url) ?? null;
}

export function releaseAssetUrl(path: string): void {
  const value = normalizePath(path);
  const url = assetUrls.get(value);
  if (!url) return;
  URL.revokeObjectURL(url);
  assetUrls.delete(value);
  assetPaths.delete(url);
}

export async function primeAssetUrls(kinds: FileRow["kind"][] = ["cover", "image"]): Promise<void> {
  await migrateLegacyFilesToOpfs().catch(() => 0);
  const rows = await db.files.toArray();
  for (const row of rows) {
    if (row.kind === "dir" || !kinds.includes(row.kind)) continue;
    if (!assetUrls.has(row.path)) await ensureAssetUrl(row.path);
  }
}

export async function resetWebStorage(): Promise<void> {
  for (const url of assetUrls.values()) URL.revokeObjectURL(url);
  assetUrls.clear();
  assetPaths.clear();
  const origin = await getOriginRoot();
  if (origin) {
    await origin.removeEntry(OPFS_ROOT_NAME, { recursive: true }).catch(() => undefined);
  }
  opfsRootPromise = null;
  migrationPromise = null;
  await db.files.clear();
}
