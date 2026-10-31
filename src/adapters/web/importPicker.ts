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

function supported(ext: string): boolean {
  return SUPPORTED.has(ext);
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
    if (parsed.kind !== "web" || !parsed.token) return null;
    if (!(await webFileExists(parsed.token))) return null;
    return parsed;
  } catch {
    return null;
  }
}

async function listDirectory(
  _location: ImportLocation,
  token: string,
): Promise<ImportEntry[]> {
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

async function pickLocation(): Promise<ImportLocation | null> {
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
  _location: ImportLocation,
  entries: ImportEntry[],
): Promise<PreparedImport> {
  return {
    paths: entries.filter((entry) => !entry.isDir).map((entry) => entry.key),
    skipped: 0,
  };
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
  const web = location?.kind === "web";
  return { scan: false, delete: web, directoryNavigation: false, autoPick: false };
}

export const webImportPicker: ImportPickerPort = {
  pickLocation,
  loadSavedLocation,
  saveLocation,
  listDirectory,
  prepareImport,
  async cleanupPreparedImport() {},
  capabilities,
  scanDirectory,
  deleteEntries,
};
