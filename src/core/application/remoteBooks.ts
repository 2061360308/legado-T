import { platform } from "@/runtime";
import {
  bookDelete,
  bookGet,
  bookList,
  bookSetRemoteOrigin,
  bookUpsert,
  localImportDelete,
  localImportImport,
} from "./books";
import {
  buildRemoteOrigin,
  parseRemoteOrigin,
  type RemoteEntry,
  type RemoteServer,
} from "@/core/domain";
import { createRemoteStorage } from "@/core/remote/factory";
import { RemoteTransport } from "@/core/remote/transport";
import type { RemoteStorage } from "@/core/remote/types";
import { getRemoteProxyUrl } from "./remoteServers";

export interface RemoteAddResult {
  path: string;
  name: string;
  bookUrl: string | null;
  error: string | null;
  duplicate: boolean;
}

const transports = new Map<string, RemoteTransport>();

function createStorage(server: RemoteServer, proxyUrl: string): RemoteStorage {
  const key = `${server.id}:${server.type}:${proxyUrl}`;
  let transport = transports.get(key);
  if (!transport) {
    transport = new RemoteTransport({
      fetch: platform.http.fetch,
      proxyUrl,
    });
    transports.set(key, transport);
  }
  return createRemoteStorage(server, transport);
}

function sourceName(path: string): string {
  return path.replace(/\\/g, "/").split("/").filter(Boolean).pop() || "book";
}

function contentTypeOf(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  if (ext === "epub") return "application/epub+zip";
  if (ext === "pdf") return "application/pdf";
  if (ext === "txt") return "text/plain";
  if (ext === "mobi") return "application/x-mobipocket-ebook";
  if (ext === "azw" || ext === "azw3") return "application/vnd.amazon.ebook";
  return "application/octet-stream";
}

async function localSourcePath(book: { sourcePath?: string | null }): Promise<string> {
  if (!book.sourcePath) throw new Error("书籍没有本地文件");
  if (book.sourcePath.startsWith("idb://")) return book.sourcePath;
  const root = await platform.files.appDataDir();
  return platform.files.join(root, book.sourcePath);
}

function pathKey(path: string): string {
  return path.replace(/\/+$/, "");
}

export async function testRemoteServer(server: RemoteServer): Promise<void> {
  const storage = createStorage(server, await getRemoteProxyUrl());
  await storage.test();
}

export async function listRemoteEntries(
  server: RemoteServer,
  path?: string | null,
): Promise<RemoteEntry[]> {
  const storage = createStorage(server, await getRemoteProxyUrl());
  await storage.test();
  const [entries, books] = await Promise.all([
    storage.list(path || storage.rootPath),
    bookList(),
  ]);
  const exact = new Map<string, string>();
  const byName = new Map<string, string>();
  for (const book of books) {
    if (book.originName) byName.set(book.originName, book.bookUrl);
    const parsed = parseRemoteOrigin(book.origin);
    if (parsed) {
      exact.set(`${parsed.serverId}:${parsed.type}:${pathKey(parsed.path)}`, book.bookUrl);
    }
  }
  return entries.map((entry) => {
    const bookUrl =
      exact.get(`${server.id}:${server.type}:${pathKey(entry.path)}`) ??
      byName.get(entry.name) ??
      null;
    return { ...entry, isOnShelf: Boolean(bookUrl), bookUrl };
  });
}

async function stagingPath(name: string): Promise<string> {
  const root = await platform.files.appDataDir();
  const dir = await platform.files.join(
    root,
    "remote-staging",
    `${Date.now()}-${Math.random().toString(36).slice(2)}`,
  );
  await platform.files.mkdir(dir, { recursive: true });
  return platform.files.join(dir, name);
}

async function writeResponse(path: string, response: Response): Promise<void> {
  const stream = response.body;
  if (stream && platform.files.writeStream) {
    await platform.files.writeStream(path, stream);
    return;
  }
  const data = new Uint8Array(await response.arrayBuffer());
  await platform.files.writeFile(path, data);
}

export async function addRemoteBooks(
  server: RemoteServer,
  entries: RemoteEntry[],
  group = 0,
  onProgress?: (name: string, index: number, total: number) => void,
): Promise<RemoteAddResult[]> {
  const storage = createStorage(server, await getRemoteProxyUrl());
  await storage.test();
  const files = entries.filter((entry) => !entry.isDir);
  const results: RemoteAddResult[] = [];
  for (let index = 0; index < files.length; index += 1) {
    const entry = files[index];
    onProgress?.(entry.name, index + 1, files.length);
    let staging = "";
    try {
      const response = await storage.get(entry.path);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      staging = await stagingPath(entry.name);
      await writeResponse(staging, response);
      const imported = await localImportImport([staging], group, null, null);
      const result = imported[0];
      if (!result) throw new Error("导入失败");
      if (result.bookUrl) {
        await bookSetRemoteOrigin(
          result.bookUrl,
          buildRemoteOrigin(server.type, entry.path, server.id),
        );
      }
      results.push({
        path: entry.path,
        name: entry.name,
        bookUrl: result.bookUrl ?? null,
        error: result.error ?? null,
        duplicate: result.duplicate,
      });
    } catch (error) {
      results.push({
        path: entry.path,
        name: entry.name,
        bookUrl: null,
        error: String(error),
        duplicate: false,
      });
    } finally {
      if (staging) {
        await localImportDelete([staging]).catch(() => undefined);
      }
    }
  }
  return results;
}

export async function deleteRemoteEntries(
  server: RemoteServer,
  entries: RemoteEntry[],
): Promise<number> {
  const storage = createStorage(server, await getRemoteProxyUrl());
  await storage.test();
  let deleted = 0;
  for (const entry of entries) {
    if (entry.isDir) continue;
    try {
      await storage.delete(entry.path);
      deleted += 1;
    } catch {
      // A missing remote object is already deleted.
    }
  }
  return deleted;
}

export async function uploadBook(
  server: RemoteServer,
  bookUrl: string,
  directoryPath: string,
): Promise<string> {
  const storage = createStorage(server, await getRemoteProxyUrl());
  await storage.test();
  const book = await bookGet(bookUrl);
  if (!book) throw new Error("书籍不存在");
  const source = await localSourcePath(book);
  const data = await platform.files.readFile(source);
  const name = book.originName || sourceName(source);
  const target = storage.childPath(directoryPath || storage.rootPath, name);
  await storage.put(target, data, contentTypeOf(name));
  await bookSetRemoteOrigin(bookUrl, buildRemoteOrigin(server.type, target, server.id));
  return target;
}

export async function refreshRemoteEntry(
  server: RemoteServer,
  entry: RemoteEntry,
): Promise<RemoteAddResult> {
  const books = await bookList();
  const old = books.find((book) => {
    const origin = parseRemoteOrigin(book.origin);
    return (
      origin?.type === server.type &&
      origin.serverId === server.id &&
      pathKey(origin.path) === pathKey(entry.path)
    );
  });
  const results = await addRemoteBooks(server, [entry]);
  const result = results[0] ?? {
    path: entry.path,
    name: entry.name,
    bookUrl: null,
    error: "导入失败",
    duplicate: false,
  };
  if (result.bookUrl && old && old.bookUrl !== result.bookUrl) {
    const next = await bookGet(result.bookUrl);
    if (next) {
      await bookUpsert({
        ...next,
        group: old.group,
        order: old.order,
        readConfig: old.readConfig,
        durChapterIndex: old.durChapterIndex,
        durChapterPos: old.durChapterPos,
        durChapterTitle: old.durChapterTitle,
        durChapterTime: old.durChapterTime,
      });
      await bookDelete(old.bookUrl, true).catch(() => undefined);
    }
  }
  return result;
}
