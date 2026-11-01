import type {
  Book,
  BookGroup,
  Bookmark,
  DictRule,
  HttpTts,
  ReadRecord,
  ReplaceRule,
  TxtTocRule,
} from "@/core/domain";
import { db } from "./database";
import { normalizePath, readWebBlob, writeWebFile } from "./storage";

export interface BackupReport {
  path: string;
  files: number;
  items: number;
}

export interface RestoreReport {
  files: number;
  items: number;
  skipped: string[];
  settingsJson?: string | null;
}

interface ConfigEntry {
  key: string;
  value: string;
  encrypted: boolean;
  decrypted: boolean;
}

interface BackupTextFile {
  name: string;
  text: string;
}

type BackupWasmModule = typeof import("./wasm/backup/backup_wasm.js");

let backupWasmPromise: Promise<BackupWasmModule> | null = null;

async function backupWasm(): Promise<BackupWasmModule> {
  if (!backupWasmPromise) {
    backupWasmPromise = import("./wasm/backup/backup_wasm.js").then(async (module) => {
      await module.default();
      return module;
    });
  }
  return backupWasmPromise;
}

async function writeConfigXml(password: string | null): Promise<string> {
  const rows = await db.kv.toArray();
  const wasm = await backupWasm();
  return wasm.write_config_xml_json(
    JSON.stringify(rows.map((row) => ({ key: row.key, value: row.value }))),
    password ?? undefined,
  );
}

async function parseConfigXml(xml: string, password: string | null): Promise<ConfigEntry[]> {
  const wasm = await backupWasm();
  return JSON.parse(
    wasm.parse_config_xml_json(xml, password ?? undefined),
  ) as ConfigEntry[];
}

async function decryptBackupText(text: string, password: string | null): Promise<string | null> {
  if (!password) return null;
  const wasm = await backupWasm();
  return wasm.decrypt_backup_text(text, password) ?? null;
}

function parseLegacyServers(text: string): unknown[] | null {
  try {
    const value = JSON.parse(text) as unknown;
    return Array.isArray(value) ? value : [value];
  } catch {
    return null;
  }
}

async function importLegacyServers(
  text: string,
  password: string | null,
): Promise<number> {
  let values = parseLegacyServers(text);
  if (!values) {
    const plain = await decryptBackupText(text.trim(), password);
    if (!plain) return 0;
    values = parseLegacyServers(plain);
  }
  if (!values) return 0;
  const row = await db.kv.get("remoteServers");
  const existing = (() => {
    if (!row?.value) return [] as unknown[];
    try {
      const value = JSON.parse(row.value) as unknown;
      return Array.isArray(value) ? value : [];
    } catch {
      return [];
    }
  })();
  const byId = new Map<number, Record<string, unknown>>();
  for (const entry of existing) {
    if (entry && typeof entry === "object") {
      const value = entry as Record<string, unknown>;
      const id = Number(value.id);
      if (Number.isFinite(id)) byId.set(id, value);
    }
  }
  let imported = 0;
  for (const entry of values) {
    if (!entry || typeof entry !== "object") continue;
    const value = entry as Record<string, unknown>;
    const id = Number(value.id);
    if (!Number.isFinite(id)) continue;
    const kind = String(value.type ?? "WEBDAV");
    if (kind.toLowerCase() !== "webdav") continue;
    let config = value.config as unknown;
    if (typeof config === "string") {
      try {
        config = JSON.parse(config) as unknown;
      } catch {
        continue;
      }
    }
    if (!config || typeof config !== "object") continue;
    const configValue = config as Record<string, unknown>;
    const url = String(configValue.url ?? "").trim();
    if (!url) continue;
    const passwordValue = String(configValue.password ?? "");
    byId.set(id, {
      id,
      name: String(value.name ?? "WebDAV") || "WebDAV",
      type: "webdav",
      config: {
        url,
        username: String(configValue.username ?? ""),
        password: "",
      },
      sortNumber: Number(value.sortNumber ?? 0) || 0,
    });
    await db.kv.put({ key: `remoteServer:password:${id}`, value: passwordValue });
    if (!(await db.kv.get("remoteServerId"))) {
      await db.kv.put({ key: "remoteServerId", value: String(id) });
    }
    imported += 1;
  }
  await db.kv.put({
    key: "remoteServers",
    value: JSON.stringify([...byId.values()]),
  });
  return imported;
}

function jsonText(value: unknown): string {
  return JSON.stringify(value);
}

function kotlinGroups(groups: BookGroup[]): unknown[] {
  return groups.map((group) => ({
    ...group,
    cover: group.coverUrl ?? null,
  }));
}

function mergeByKey<T>(
  items: T[],
  key: (item: T) => unknown,
  seen: Set<unknown>,
): T[] {
  const out: T[] = [];
  for (const item of items) {
    const value = key(item);
    if (seen.has(value)) continue;
    seen.add(value);
    out.push(item);
  }
  return out;
}

function parseJson<T>(text: string | undefined): T[] {
  if (!text) return [];
  try {
    const value = JSON.parse(text) as T | T[];
    return Array.isArray(value) ? value : [value];
  } catch {
    return [];
  }
}

export async function exportWebBackup(
  path: string,
  password: string | null,
  _settingsJson?: string | null,
): Promise<BackupReport> {
  const [
    books,
    groups,
    bookmarks,
    readRecords,
    txtTocRules,
    replaceRules,
    dictRules,
    httpTts,
  ] = await Promise.all([
    db.books.toArray(),
    db.groups.toArray(),
    db.bookmarks.toArray(),
    db.readRecords.toArray(),
    db.txtTocRules.toArray(),
    db.replaceRules.toArray(),
    db.dictRules.toArray(),
    db.httpTts.toArray(),
  ]);
  const files: BackupTextFile[] = [
    { name: "bookshelf.json", text: jsonText(books) },
    { name: "bookGroup.json", text: jsonText(kotlinGroups(groups)) },
    { name: "bookmark.json", text: jsonText(bookmarks) },
    { name: "readRecord.json", text: jsonText(readRecords) },
    { name: "txtTocRule.json", text: jsonText(txtTocRules) },
    { name: "replaceRule.json", text: jsonText(replaceRules) },
    { name: "dictRule.json", text: jsonText(dictRules) },
    { name: "httpTTS.json", text: jsonText(httpTts) },
    { name: "config.xml", text: await writeConfigXml(password) },
  ];
  const wasm = await backupWasm();
  const zip = wasm.pack_backup_json(JSON.stringify(files)) as Uint8Array<ArrayBuffer>;
  await writeWebFile(normalizePath(path), zip, {
    kind: "backup",
    mime: "application/zip",
  });
  if (!path.includes("backup-upload")) {
    const blob = new Blob([zip], { type: "application/zip" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = path.split("/").pop() || "backup.zip";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    setTimeout(() => URL.revokeObjectURL(url), 30000);
  }
  return {
    path,
    files: files.length,
    items:
      books.length +
      groups.length +
      bookmarks.length +
      readRecords.length +
      txtTocRules.length +
      replaceRules.length +
      dictRules.length +
      httpTts.length,
  };
}

export async function importWebBackup(
  path: string,
  password: string | null,
  ignore: string[],
): Promise<RestoreReport> {
  const blob = await readWebBlob(normalizePath(path));
  const wasm = await backupWasm();
  const unpacked = JSON.parse(
    wasm.unpack_backup_json(new Uint8Array(await blob.arrayBuffer())),
  ) as BackupTextFile[];
  const entries = new Map(unpacked.map((file) => [file.name, file.text]));
  const skipped: string[] = [];
  let files = 0;
  let items = 0;

  const bookSeen = new Set<string>();
  const groupSeen = new Set<number>();
  const bookmarkSeen = new Set<number>();
  const readSeen = new Set<string>();
  const txtSeen = new Set<number>();
  const replaceSeen = new Set<number>();
  const dictSeen = new Set<string>();
  const ttsSeen = new Set<number>();

  for (const name of entries.keys()) {
    if (ignore.includes(name)) skipped.push(name);
  }

  const books = [
    ...parseJson<Book>(entries.get("bookshelf.json")),
    ...parseJson<Book>(entries.get("books.json")),
  ];
  for (const book of mergeByKey(books, (item) => item.bookUrl, bookSeen)) {
    await db.books.put(book);
    items += 1;
  }
  if (books.length) files += 1;

  const groups = [
    ...parseJson<BookGroup & { cover?: string | null }>(entries.get("bookGroup.json")),
    ...parseJson<BookGroup>(entries.get("book_groups.json")),
  ];
  for (const group of mergeByKey(groups, (item) => item.groupId, groupSeen)) {
    const legacyCover = (group as BookGroup & { cover?: string | null }).cover;
    await db.groups.put({
      ...group,
      coverUrl: group.coverUrl ?? legacyCover ?? null,
    });
    items += 1;
  }
  if (groups.length) files += 1;

  const bookmarks = [
    ...parseJson<Bookmark>(entries.get("bookmark.json")),
    ...parseJson<Bookmark>(entries.get("bookmarks.json")),
  ];
  for (const bookmark of mergeByKey(bookmarks, (item) => item.time, bookmarkSeen)) {
    await db.bookmarks.put(bookmark);
    items += 1;
  }
  if (bookmarks.length) files += 1;

  const records = parseJson<ReadRecord>(entries.get("readRecord.json"));
  for (const record of records) {
    const key = `${record.deviceId}\u0000${record.bookName}`;
    if (readSeen.has(key)) continue;
    readSeen.add(key);
    await db.readRecords.put({
      ...record,
      author: record.author ?? "",
      readTime: record.readTime ?? 0,
      lastRead: record.lastRead ?? 0,
      durChapterIndex: record.durChapterIndex ?? 0,
      durChapterPos: record.durChapterPos ?? 0,
      durChapterTitle: record.durChapterTitle ?? "",
    });
    items += 1;
  }
  if (records.length) files += 1;

  const txtRules = [
    ...parseJson<TxtTocRule>(entries.get("txtTocRule.json")),
    ...parseJson<TxtTocRule>(entries.get("txtTocRules.json")),
  ];
  for (const rule of mergeByKey(txtRules, (item) => item.id, txtSeen)) {
    await db.txtTocRules.put(rule);
    items += 1;
  }
  if (txtRules.length) files += 1;

  const replaceRules = [
    ...parseJson<ReplaceRule>(entries.get("replaceRule.json")),
    ...parseJson<ReplaceRule>(entries.get("replace_rules.json")),
  ];
  for (const rule of mergeByKey(replaceRules, (item) => item.id, replaceSeen)) {
    await db.replaceRules.put(rule);
    items += 1;
  }
  if (replaceRules.length) files += 1;

  const dictRules = [
    ...parseJson<DictRule>(entries.get("dictRule.json")),
    ...parseJson<DictRule>(entries.get("dictRules.json")),
  ];
  for (const rule of mergeByKey(dictRules, (item) => item.name, dictSeen)) {
    await db.dictRules.put(rule);
    items += 1;
  }
  if (dictRules.length) files += 1;

  const httpTts = parseJson<HttpTts>(entries.get("httpTTS.json"));
  for (const engine of mergeByKey(httpTts, (item) => item.id, ttsSeen)) {
    await db.httpTts.put(engine);
    items += 1;
  }
  if (httpTts.length) files += 1;

  if (entries.get("config.xml")) {
    const config = await parseConfigXml(entries.get("config.xml")!, password);
    for (const entry of config) {
      if (ignore.includes(entry.key)) continue;
      if (entry.encrypted && !entry.decrypted) continue;
      await db.kv.put({ key: entry.key, value: entry.value });
      items += 1;
    }
    files += 1;
  }

  if (entries.get("servers.json")) {
    const count = await importLegacyServers(entries.get("servers.json")!, password);
    if (count > 0) {
      items += count;
      files += 1;
    }
  }

  for (const name of entries.keys()) {
    if (
      ![
        "bookshelf.json",
        "books.json",
        "bookGroup.json",
        "book_groups.json",
        "bookmark.json",
        "bookmarks.json",
        "readRecord.json",
        "txtTocRule.json",
        "txtTocRules.json",
        "replaceRule.json",
        "replace_rules.json",
        "dictRule.json",
        "dictRules.json",
        "httpTTS.json",
        "servers.json",
        "config.xml",
      ].includes(name) &&
      !skipped.includes(name)
    ) {
      skipped.push(name);
    }
  }

  return { files, items, skipped, settingsJson: null };
}
