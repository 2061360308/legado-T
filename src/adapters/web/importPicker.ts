import type {
  ImportEntry,
  ImportLocation,
  ImportPickerCapabilities,
  ImportPickerPort,
  PreparedImport,
} from "@/platform/contract";
import { db, nowMs } from "./database";
import { pickWebFiles } from "./picker";
import {
  WEB_ROOT,
  deleteWebPath,
  extensionOf,
  joinPath,
  normalizePath,
  readWebDir,
  webFileExists,
  writeWebFile,
} from "./storage";

const LOCATION_KEY = "importBookLocation";
const FSA_DB = "local-reader-fsa";
const FSA_STORE = "handles";
const SUPPORTED = new Set([
  "txt",
  "epub",
  "pdf",
  "umd",
  "mobi",
  "azw",
  "azw3",
  "zip",
]);

const fsaRoots = new Map<string, FileSystemDirectoryHandle>();

type DirectoryPicker = (options?: {
  mode?: "read" | "readwrite";
}) => Promise<FileSystemDirectoryHandle>;

type PermissionHandle = FileSystemDirectoryHandle & {
  queryPermission?: (descriptor: { mode: "read" | "readwrite" }) => Promise<PermissionState>;
};

type DirectoryEntries = FileSystemDirectoryHandle & {
  entries(): AsyncIterableIterator<[string, FileSystemHandle]>;
};

function supported(ext: string): boolean {
  return SUPPORTED.has(ext);
}

function directoryPicker(): DirectoryPicker | null {
  const picker = (window as unknown as { showDirectoryPicker?: DirectoryPicker })
    .showDirectoryPicker;
  return typeof picker === "function" ? picker.bind(window) : null;
}

function fsaToken(id: string, path = "", isDir = false): string {
  const clean = path.replace(/^\/+|\/+$/g, "");
  return `fsa://${encodeURIComponent(id)}/${clean}${isDir && clean ? "/" : ""}`;
}

function parseFsaToken(token: string): { id: string; path: string } {
  const match = /^fsa:\/\/([^/]+)\/?(.*)$/.exec(token);
  if (!match) throw new Error("无效的文件夹位置");
  return {
    id: decodeURIComponent(match[1]),
    path: match[2].replace(/\/+$/, ""),
  };
}

function openHandleDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(FSA_DB, 1);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(FSA_STORE)) {
        request.result.createObjectStore(FSA_STORE);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function saveFsaHandle(id: string, handle: FileSystemDirectoryHandle): Promise<void> {
  try {
    const database = await openHandleDb();
    await new Promise<void>((resolve, reject) => {
      const tx = database.transaction(FSA_STORE, "readwrite");
      tx.objectStore(FSA_STORE).put(handle, id);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    database.close();
  } catch {
    // The handle still works for the current page session.
  }
}

async function readFsaHandle(id: string): Promise<FileSystemDirectoryHandle | null> {
  try {
    const database = await openHandleDb();
    const handle = await new Promise<FileSystemDirectoryHandle | null>((resolve, reject) => {
      const tx = database.transaction(FSA_STORE, "readonly");
      const request = tx.objectStore(FSA_STORE).get(id);
      request.onsuccess = () => resolve((request.result as FileSystemDirectoryHandle) ?? null);
      request.onerror = () => reject(request.error);
    });
    database.close();
    return handle;
  } catch {
    return null;
  }
}

async function hasReadPermission(handle: FileSystemDirectoryHandle): Promise<boolean> {
  const permission = (handle as PermissionHandle).queryPermission;
  if (!permission) return true;
  try {
    return (await permission.call(handle, { mode: "read" })) === "granted";
  } catch {
    return false;
  }
}

async function resolveFsaRoot(id: string): Promise<FileSystemDirectoryHandle> {
  const cached = fsaRoots.get(id);
  if (cached) return cached;
  const handle = await readFsaHandle(id);
  if (!handle) throw new Error("文件夹授权已失效，请重新选择文件夹");
  if (!(await hasReadPermission(handle))) {
    throw new Error("文件夹读取权限已失效，请重新选择文件夹");
  }
  fsaRoots.set(id, handle);
  return handle;
}

async function resolveFsaDir(token: string): Promise<FileSystemDirectoryHandle> {
  const { id, path } = parseFsaToken(token);
  let dir = await resolveFsaRoot(id);
  for (const part of path.split("/").filter(Boolean)) {
    dir = await dir.getDirectoryHandle(part);
  }
  return dir;
}

async function readFsaFile(token: string): Promise<File> {
  const { id, path } = parseFsaToken(token);
  const parts = path.split("/").filter(Boolean);
  const name = parts.pop();
  if (!name) throw new Error("文件路径无效");
  let dir = await resolveFsaRoot(id);
  for (const part of parts) {
    dir = await dir.getDirectoryHandle(part);
  }
  const handle = await dir.getFileHandle(name);
  return handle.getFile();
}

async function shelfNames(): Promise<Map<string, string>> {
  const books = await db.books.toArray();
  const result = new Map<string, string>();
  for (const book of books) {
    const name = book.originName?.trim();
    if (name) result.set(name, book.bookUrl);
  }
  return result;
}

async function saveLocation(location: ImportLocation): Promise<void> {
  await db.kv.put({ key: LOCATION_KEY, value: JSON.stringify(location) });
}

async function loadSavedLocation(): Promise<ImportLocation | null> {
  const row = await db.kv.get(LOCATION_KEY);
  if (!row?.value) return null;
  try {
    const parsed = JSON.parse(row.value) as ImportLocation;
    if (!parsed.token) return null;
    if (parsed.kind === "fsa") {
      const { id } = parseFsaToken(parsed.token);
      const handle = await readFsaHandle(id);
      if (!handle || !(await hasReadPermission(handle))) return null;
      fsaRoots.set(id, handle);
      return parsed;
    }
    if (parsed.kind !== "web") return null;
    if (!(await webFileExists(parsed.token))) return null;
    return parsed;
  } catch {
    return null;
  }
}

async function listDirectory(
  location: ImportLocation,
  token: string,
): Promise<ImportEntry[]> {
  if (location.kind === "fsa") {
    return listFsaDirectory(token);
  }
  const root = normalizePath(token);
  const [entries, shelf] = await Promise.all([readWebDir(root), shelfNames()]);
  const out: ImportEntry[] = [];
  for (const entry of entries) {
    if (entry.name.startsWith(".")) continue;
    const path = joinPath(root, entry.name);
    if (entry.isDirectory) {
      out.push({
        key: path,
        name: entry.name,
        isDir: true,
        size: 0,
        modified: 0,
        ext: "",
        supported: false,
        onShelf: false,
        bookUrl: null,
      });
      continue;
    }
    const row = await db.files.get(path);
    if (!row || row.kind === "dir") continue;
    const ext = extensionOf(path);
    const bookUrl = shelf.get(entry.name) ?? null;
    out.push({
      key: path,
      name: entry.name,
      isDir: false,
      size: row.size,
      modified: Math.floor(row.updated / 1000),
      ext,
      supported: supported(ext),
      onShelf: Boolean(bookUrl),
      bookUrl,
    });
  }
  return out;
}

async function listFsaDirectory(token: string): Promise<ImportEntry[]> {
  const { id, path } = parseFsaToken(token);
  const dir = await resolveFsaDir(token);
  const shelf = await shelfNames();
  const out: ImportEntry[] = [];
  for await (const [name, handle] of (dir as DirectoryEntries).entries()) {
    if (name.startsWith(".")) continue;
    const childPath = path ? `${path}/${name}` : name;
    if (handle.kind === "directory") {
      out.push({
        key: fsaToken(id, childPath, true),
        name,
        isDir: true,
        size: 0,
        modified: 0,
        ext: "",
        supported: false,
        onShelf: false,
        bookUrl: null,
      });
      continue;
    }
    const file = await (handle as FileSystemFileHandle).getFile();
    const ext = extensionOf(name);
    const bookUrl = shelf.get(name) ?? null;
    out.push({
      key: fsaToken(id, childPath),
      name,
      isDir: false,
      size: file.size,
      modified: Math.floor(file.lastModified / 1000),
      ext,
      supported: supported(ext),
      onShelf: Boolean(bookUrl),
      bookUrl,
    });
  }
  return out;
}

async function pickLocation(): Promise<ImportLocation | null> {
  const picker = directoryPicker();
  if (picker) {
    let handle: FileSystemDirectoryHandle;
    try {
      handle = await picker({ mode: "read" });
    } catch (error) {
      if ((error as DOMException).name === "AbortError") return null;
      throw error;
    }
    const id = String(nowMs());
    fsaRoots.set(id, handle);
    await saveFsaHandle(id, handle);
    const location: ImportLocation = {
      kind: "fsa",
      token: fsaToken(id),
      rootToken: fsaToken(id),
      name: handle.name,
    };
    await saveLocation(location);
    return location;
  }

  const picked = await pickWebFiles({ multiple: true });
  if (!picked.length) return null;
  const previous = await loadSavedLocation();
  if (previous?.token.startsWith(`${WEB_ROOT}/import/`)) {
    await deleteWebPath(previous.token).catch(() => undefined);
  }
  const session = joinPath(WEB_ROOT, "import", String(nowMs()));
  for (const item of picked) {
    const original = item.file.name || "book";
    const ext = extensionOf(original);
    const stem = ext ? original.slice(0, -(ext.length + 1)) : original;
    let target = joinPath(session, original);
    let index = 1;
    while (await webFileExists(target)) {
      target = joinPath(session, `${stem} (${index})${ext ? `.${ext}` : ""}`);
      index += 1;
    }
    await writeWebFile(target, item.file, {
      kind: "other",
      mime: item.file.type,
    });
  }
  const location: ImportLocation = {
    kind: "web",
    token: session,
    rootToken: session,
    name: "已选文件",
  };
  await saveLocation(location);
  return location;
}

async function prepareImport(
  location: ImportLocation,
  entries: ImportEntry[],
): Promise<PreparedImport> {
  if (location.kind === "fsa") {
    const session = joinPath(WEB_ROOT, "import", `fsa-${nowMs()}`);
    const paths: string[] = [];
    let skipped = 0;
    for (const entry of entries.filter((item) => !item.isDir)) {
      try {
        const file = await readFsaFile(entry.key);
        const ext = extensionOf(entry.name);
        const stem = ext ? entry.name.slice(0, -(ext.length + 1)) : entry.name;
        let target = joinPath(session, entry.name);
        let index = 1;
        while (await webFileExists(target)) {
          target = joinPath(session, `${stem} (${index})${ext ? `.${ext}` : ""}`);
          index += 1;
        }
        await writeWebFile(target, file, {
          kind: "other",
          mime: file.type,
        });
        paths.push(target);
      } catch {
        skipped += 1;
      }
    }
    return { paths, sessionId: session, skipped };
  }

  return {
    paths: entries.filter((entry) => !entry.isDir).map((entry) => entry.key),
    skipped: 0,
  };
}

async function cleanupPreparedImport(prepared: PreparedImport): Promise<void> {
  if (prepared.sessionId) {
    await deleteWebPath(prepared.sessionId).catch(() => undefined);
  }
}

async function scanDirectory(): Promise<ImportEntry[]> {
  return [];
}

async function deleteEntries(
  _location: ImportLocation,
  entries: ImportEntry[],
): Promise<number> {
  let count = 0;
  for (const entry of entries) {
    if (entry.isDir) continue;
    await deleteWebPath(entry.key);
    count += 1;
  }
  return count;
}

function capabilities(location: ImportLocation | null): ImportPickerCapabilities {
  if (location?.kind === "fsa") {
    return {
      scan: false,
      delete: false,
      directoryNavigation: true,
      autoPick: false,
    };
  }
  const web = location?.kind === "web";
  return { scan: false, delete: web, directoryNavigation: false, autoPick: false };
}

export const webImportPicker: ImportPickerPort = {
  pickLocation,
  loadSavedLocation,
  saveLocation,
  listDirectory,
  prepareImport,
  cleanupPreparedImport,
  capabilities,
  scanDirectory,
  deleteEntries,
};
