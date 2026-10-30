import { open } from "@tauri-apps/plugin-dialog";
import {
  bookList,
  localImportDelete,
  localImportList,
  localImportScan,
} from "@/api/books";
import type { LocalImportEntry } from "@/api/bindings";
import { kvGet, kvSet } from "@/api/config";
import {
  cleanupSafImport,
  isAndroid,
  listSafDirectory,
  pickSafDirectory,
  prepareSafImport,
} from "./index";

export interface ImportLocation {
  kind: "desktop" | "saf";
  token: string;
  rootToken: string;
  name: string;
}

export interface ImportEntry {
  key: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  ext: string;
  supported: boolean;
  onShelf: boolean;
  bookUrl?: string | null;
}

export interface PreparedImport {
  paths: string[];
  sessionId?: string;
  skipped: number;
}

export interface ImportPickerCapabilities {
  scan: boolean;
  delete: boolean;
}

const LOCATION_KEY = "importBookLocation";
const LEGACY_PATH_KEY = "importBookPath";

function baseName(path: string): string {
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  return index >= 0 ? value.slice(index + 1) : value;
}

function desktopLocation(path: string): ImportLocation {
  return {
    kind: "desktop",
    token: path,
    rootToken: path,
    name: baseName(path),
  };
}

function mapDesktopEntry(entry: LocalImportEntry): ImportEntry {
  return {
    key: entry.path,
    name: entry.name,
    isDir: entry.isDir,
    size: entry.size,
    modified: entry.modified,
    ext: entry.ext,
    supported: entry.supported,
    onShelf: entry.onShelf,
    bookUrl: entry.bookUrl,
  };
}

async function shelfByName(): Promise<Map<string, string>> {
  const books = await bookList();
  const result = new Map<string, string>();
  for (const book of books) {
    const name = book.originName?.trim();
    if (name) result.set(name, book.bookUrl);
  }
  return result;
}

export async function pickImportLocation(): Promise<ImportLocation | null> {
  if (isAndroid()) {
    const picked = await pickSafDirectory();
    if (picked.cancelled) return null;
    const location: ImportLocation = {
      kind: "saf",
      token: picked.treeUri,
      rootToken: picked.documentId,
      name: picked.rootName,
    };
    await saveImportLocation(location);
    return location;
  }

  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择文件夹",
  });
  if (typeof picked !== "string" || !picked) return null;
  const location = desktopLocation(picked);
  await saveImportLocation(location);
  return location;
}

export async function loadSavedImportLocation(): Promise<ImportLocation | null> {
  const raw = await kvGet(LOCATION_KEY);
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as ImportLocation;
      if (
        (parsed.kind === "desktop" || parsed.kind === "saf") &&
        parsed.token &&
        parsed.rootToken
      ) {
        return parsed;
      }
    } catch {
      // Ignore malformed legacy values.
    }
  }
  if (isAndroid()) return null;
  const legacyPath = await kvGet(LEGACY_PATH_KEY);
  return legacyPath ? desktopLocation(legacyPath) : null;
}

export async function saveImportLocation(location: ImportLocation): Promise<void> {
  await kvSet(LOCATION_KEY, JSON.stringify(location));
  if (location.kind === "desktop") {
    await kvSet(LEGACY_PATH_KEY, location.token);
  }
}

export async function listImportDirectory(
  location: ImportLocation,
  token: string,
): Promise<ImportEntry[]> {
  if (location.kind === "saf") {
    const [entries, shelf] = await Promise.all([
      listSafDirectory(location.token, token),
      shelfByName(),
    ]);
    return entries.map((entry) => {
      const bookUrl = entry.isDir ? null : shelf.get(entry.name) ?? null;
      return {
        key: entry.documentId,
        name: entry.name,
        isDir: entry.isDir,
        size: entry.size,
        modified: entry.modified,
        ext: entry.ext,
        supported: entry.supported,
        onShelf: Boolean(bookUrl),
        bookUrl,
      };
    });
  }
  const entries = await localImportList(token);
  return entries.map(mapDesktopEntry);
}

export async function prepareImport(
  location: ImportLocation,
  entries: ImportEntry[],
): Promise<PreparedImport> {
  if (location.kind === "saf") {
    const prepared = await prepareSafImport(
      location.token,
      entries.map((entry) => entry.key),
    );
    return {
      paths: prepared.files.map((file) => file.path),
      sessionId: prepared.sessionId,
      skipped: prepared.skipped,
    };
  }
  return {
    paths: entries.map((entry) => entry.key),
    skipped: 0,
  };
}

export async function cleanupPreparedImport(prepared: PreparedImport): Promise<void> {
  if (!prepared.sessionId) return;
  await cleanupSafImport(prepared.sessionId).catch(() => undefined);
}

export function importPickerCapabilities(
  location: ImportLocation | null,
): ImportPickerCapabilities {
  const desktop = location?.kind === "desktop";
  return { scan: desktop, delete: desktop };
}

export async function scanImportDirectory(
  location: ImportLocation,
  token: string,
): Promise<ImportEntry[]> {
  if (location.kind !== "desktop") return [];
  const entries = await localImportScan(token);
  return entries.map(mapDesktopEntry);
}

export async function deleteImportEntries(
  location: ImportLocation,
  entries: ImportEntry[],
): Promise<number> {
  if (location.kind !== "desktop") return 0;
  return localImportDelete(entries.map((entry) => entry.key));
}
