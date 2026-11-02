import type { BookSource } from "@/core/domain";
import { db, nowSeconds } from "./database";
import {
  evalEngine,
  exploreSource,
  removeEngineCookie,
  searchSource,
  setEngineCookie,
  setSourceVariables,
  sourceContent,
  sourceDownloadUrls as sourceDownloadUrlsWasm,
  sourceInfo,
  sourcePay as sourcePayWasm,
  sourceResource as sourceResourceWasm,
  sourceReviews as sourceReviewsWasm,
  sourceToc,
} from "./wasm/engine/engineWasm";

const HISTORY_KEY = "bookSourceSearchHistory";
const HISTORY_LIMIT = 100;

function normalize(source: BookSource): BookSource {
  return {
    ...source,
    bookSourceName: source.bookSourceName?.trim() || source.bookSourceUrl,
    bookSourceType: source.bookSourceType ?? 0,
    enabled: source.enabled !== false,
    enabledExplore: source.enabledExplore !== false,
    customOrder: source.customOrder ?? 0,
    weight: source.weight ?? 0,
    lastUpdateTime: source.lastUpdateTime ?? nowSeconds(),
    respondTime: source.respondTime ?? 0,
  };
}

export async function listBookSources(): Promise<BookSource[]> {
  const sources = await db.bookSources.toArray();
  sources.sort(
    (a, b) =>
      (a.customOrder ?? 0) - (b.customOrder ?? 0) ||
      (a.bookSourceName ?? "").localeCompare(b.bookSourceName ?? "", "zh-CN"),
  );
  return sources.map(normalize);
}

export async function getBookSource(bookSourceUrl: string): Promise<BookSource | null> {
  const source = await db.bookSources.get(bookSourceUrl);
  return source ? normalize(source) : null;
}

async function prepareSource(source: BookSource): Promise<BookSource> {
  await setSourceVariables(source.bookSourceUrl, await getSourceVariables(source.bookSourceUrl));
  return source;
}

export async function saveBookSource(source: BookSource): Promise<void> {
  const url = source.bookSourceUrl?.trim();
  if (!url) throw new Error("书源 URL 不能为空");
  await db.bookSources.put(normalize({ ...source, bookSourceUrl: url }));
}

export async function deleteBookSource(bookSourceUrl: string): Promise<void> {
  await db.transaction("rw", db.bookSources, db.sourceVariables, async () => {
    await db.bookSources.delete(bookSourceUrl);
    await db.sourceVariables.where("bookSourceUrl").equals(bookSourceUrl).delete();
  });
}

export async function setBookSourceEnabled(
  bookSourceUrl: string,
  enabled: boolean,
): Promise<void> {
  const source = await db.bookSources.get(bookSourceUrl);
  if (!source) throw new Error("书源不存在");
  await db.bookSources.put({ ...source, enabled });
}

export async function setBookSourceExploreEnabled(
  bookSourceUrl: string,
  enabled: boolean,
): Promise<void> {
  const source = await db.bookSources.get(bookSourceUrl);
  if (!source) throw new Error("书源不存在");
  await db.bookSources.put({ ...source, enabledExplore: enabled });
}

export async function importBookSources(json: string): Promise<number> {
  const parsed = JSON.parse(json) as unknown;
  const list = Array.isArray(parsed)
    ? parsed
    : parsed && typeof parsed === "object" && Array.isArray((parsed as { sources?: unknown }).sources)
      ? ((parsed as { sources: unknown[] }).sources)
      : [parsed];
  let count = 0;
  for (const item of list) {
    if (!item || typeof item !== "object") continue;
    const source = normalize(item as BookSource);
    if (!source.bookSourceUrl?.trim()) continue;
    await db.bookSources.put(source);
    count += 1;
  }
  return count;
}

export async function exportBookSources(): Promise<string> {
  return JSON.stringify(await listBookSources(), null, 2);
}

export async function bookSourceGroups(): Promise<string[]> {
  const groups = new Set<string>();
  for (const source of await db.bookSources.toArray()) {
    const value = source.bookSourceGroup?.trim();
    if (value) groups.add(value);
  }
  return [...groups].sort((a, b) => a.localeCompare(b, "zh-CN"));
}

export async function getSourceVariables(
  bookSourceUrl: string,
): Promise<Record<string, string>> {
  const rows = await db.sourceVariables.where("bookSourceUrl").equals(bookSourceUrl).toArray();
  return Object.fromEntries(rows.map((row) => [row.key, row.value]));
}

export async function setSourceVariable(
  bookSourceUrl: string,
  key: string,
  value: string,
): Promise<void> {
  await db.sourceVariables.put({ bookSourceUrl, key, value });
}

export async function deleteSourceVariable(
  bookSourceUrl: string,
  key: string,
): Promise<void> {
  await db.sourceVariables.delete([bookSourceUrl, key]);
}

async function searchHistory(): Promise<string[]> {
  const row = await db.kv.get(HISTORY_KEY);
  if (!row?.value) return [];
  try {
    const value = JSON.parse(row.value) as unknown;
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
  } catch {
    return [];
  }
}

async function saveSearchHistory(keyword: string): Promise<void> {
  const text = keyword.trim();
  if (!text) return;
  const history = [text, ...(await searchHistory()).filter((item) => item !== text)].slice(
    0,
    HISTORY_LIMIT,
  );
  await db.kv.put({ key: HISTORY_KEY, value: JSON.stringify(history) });
}

export async function listSourceSearchHistory(limit = 20): Promise<string[]> {
  return (await searchHistory()).slice(0, Math.min(Math.max(limit, 1), HISTORY_LIMIT));
}

export async function searchBookSources(
  keyword: string,
  sourceUrls?: string[],
  page = 1,
): Promise<unknown[]> {
  const text = keyword.trim();
  if (!text) return [];
  const selected = new Set(sourceUrls?.filter(Boolean) ?? []);
  const sources = (await listBookSources()).filter(
    (source) =>
      source.enabled !== false &&
      Boolean(source.searchUrl?.trim()) &&
      (!selected.size || selected.has(source.bookSourceUrl)),
  );
  if (!sources.length) return [];
  await saveSearchHistory(text);
  const results: unknown[] = [];
  for (const source of sources) {
    try {
      await setSourceVariables(source.bookSourceUrl, await getSourceVariables(source.bookSourceUrl));
      const books = (await searchSource(JSON.stringify(source), text, Math.max(page, 1))) as Record<
        string,
        unknown
      >[];
      for (const book of books) {
        results.push({
          ...book,
          bookSourceUrl: source.bookSourceUrl,
          bookSourceName: source.bookSourceName,
        });
      }
    } catch (error) {
      results.push({
        bookSourceUrl: source.bookSourceUrl,
        bookSourceName: source.bookSourceName,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  return results;
}

export async function sourceBookInfo(
  bookSourceUrl: string,
  bookUrl: string,
): Promise<unknown> {
  const source = await getBookSource(bookSourceUrl);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  return sourceInfo(
    JSON.stringify(source),
    JSON.stringify({
      bookUrl,
      origin: source.bookSourceUrl,
      originName: source.bookSourceName,
    }),
  );
}

export async function sourceBookToc(
  bookSourceUrl: string,
  book: unknown,
): Promise<unknown[]> {
  const source = await getBookSource(bookSourceUrl);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  return sourceToc(JSON.stringify(source), JSON.stringify(book)) as Promise<unknown[]>;
}

export async function sourceBookContent(
  bookSourceUrl: string,
  book: unknown,
  chapter: unknown,
  nextChapterUrl?: string | null,
): Promise<string> {
  const source = await getBookSource(bookSourceUrl);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  return sourceContent(
    JSON.stringify(source),
    JSON.stringify(book),
    JSON.stringify(chapter),
    nextChapterUrl,
  );
}

export async function saveSourceCookies(
  sourceUrl: string,
  cookies: string[],
  cookieUrl?: string | null,
): Promise<number> {
  const header = cookies.join("; ");
  if (!header) return 0;
  await setEngineCookie(cookieUrl || sourceUrl, header);
  if (cookieUrl && cookieUrl !== sourceUrl) await setEngineCookie(sourceUrl, header);
  return cookies.length;
}

export async function clearSourceCookies(sourceUrl: string): Promise<void> {
  await removeEngineCookie(sourceUrl);
}

export async function sourceChapterContent(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<string> {
  const context = await onlineContext(bookUrl, chapterIndex);
  return sourceContent(
    JSON.stringify(context.source),
    JSON.stringify(context.engineBook),
    JSON.stringify(context.engineChapter),
  );
}

export async function sourceExplore(
  bookSourceUrl: string,
  url: string,
  page = 1,
): Promise<unknown[]> {
  const source = await getBookSource(bookSourceUrl);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  const books = (await exploreSource(JSON.stringify(source), url, Math.max(page, 1))) as Record<
    string,
    unknown
  >[];
  return books.map((book) => ({
    ...book,
    bookSourceUrl: source.bookSourceUrl,
    bookSourceName: source.bookSourceName,
  }));
}

function category(title: string, url: string, style?: unknown) {
  const name = title.trim() || url.trim();
  if (!name && !url.trim()) return null;
  return style && typeof style === "object"
    ? { title: name, url: url.trim(), style }
    : { title: name, url: url.trim() };
}

function categoriesFromValue(value: unknown): { title: string; url: string; style?: unknown }[] {
  const items = Array.isArray(value) ? value : value && typeof value === "object" ? [value] : [];
  const out: { title: string; url: string; style?: unknown }[] = [];
  for (const item of items) {
    if (Array.isArray(item)) {
      const entry = category(String(item[0] ?? ""), String(item[1] ?? ""));
      if (entry) out.push(entry);
    } else if (item && typeof item === "object") {
      const object = item as Record<string, unknown>;
      const entry = category(
        String(object.title ?? object.name ?? ""),
        String(object.url ?? object.link ?? ""),
        object.style,
      );
      if (entry) out.push(entry);
    } else if (typeof item === "string") {
      for (const line of item.split(/\r?\n/)) {
        const [title, url] = line.split("::");
        if (url) {
          const entry = category(title ?? "", url);
          if (entry) out.push(entry);
        }
      }
    }
  }
  return out;
}

export async function exploreCategories(
  bookSourceUrl: string,
): Promise<{ title: string; url: string; style?: unknown }[]> {
  const source = await getBookSource(bookSourceUrl);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  const raw = source.exploreUrl?.trim() ?? "";
  if (!raw) return [];
  if (!raw.startsWith("@js:") && !raw.startsWith("[") && !raw.startsWith("{")) {
    const lines = categoriesFromValue(raw);
    if (lines.length) return lines;
  }
  let value: unknown = raw;
  try {
    value = JSON.parse(raw);
  } catch {
    const script = raw.startsWith("@js:") ? raw.slice(4).trim() : raw;
    const text = await evalEngine({
      script,
      baseUrl: source.bookSourceUrl,
      sourceJson: JSON.stringify(source),
    });
    try {
      value = JSON.parse(text);
    } catch {
      value = text;
    }
  }
  const categories = categoriesFromValue(value);
  return categories.length ? categories : [category("全部", raw) as { title: string; url: string }];
}

export async function checkBookSources(
  sourceUrls?: string[],
  keyword = "斗破",
): Promise<
  {
    bookSourceUrl: string;
    bookSourceName: string;
    ok: boolean;
    count: number;
    elapsedMs: number;
    error?: string;
  }[]
> {
  const selected = new Set(sourceUrls?.filter(Boolean) ?? []);
  const sources = (await listBookSources()).filter(
    (source) =>
      Boolean(source.searchUrl?.trim()) &&
      (!selected.size || selected.has(source.bookSourceUrl)),
  );
  const results = [];
  for (const source of sources) {
    const started = Date.now();
    try {
      await setSourceVariables(source.bookSourceUrl, await getSourceVariables(source.bookSourceUrl));
      const books = (await searchSource(JSON.stringify(source), keyword, 1)) as unknown[];
      results.push({
        bookSourceUrl: source.bookSourceUrl,
        bookSourceName: source.bookSourceName,
        ok: true,
        count: books.length,
        elapsedMs: Date.now() - started,
      });
    } catch (error) {
      results.push({
        bookSourceUrl: source.bookSourceUrl,
        bookSourceName: source.bookSourceName,
        ok: false,
        count: 0,
        elapsedMs: Date.now() - started,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  return results;
}

async function onlineContext(bookUrl: string, chapterIndex?: number | null) {
  const book = await db.books.get(bookUrl);
  if (!book?.origin) throw new Error("书籍没有关联书源");
  const source = await getBookSource(book.origin);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  const chapters = await db.chapters.where("bookUrl").equals(bookUrl).sortBy("index");
  const index = Math.min(
    Math.max(chapterIndex ?? book.durChapterIndex ?? 0, 0),
    Math.max(chapters.length - 1, 0),
  );
  const chapter = chapters[index];
  if (!chapter) throw new Error("章节不存在");
  return {
    book,
    chapter,
    source,
    engineBook: {
      bookUrl: book.bookUrl,
      tocUrl: book.tocUrl ?? "",
      origin: book.origin,
      originName: book.originName ?? source.bookSourceName,
      name: book.name,
      author: book.author,
      kind: book.kind ?? null,
      coverUrl: book.coverUrl ?? null,
      intro: book.intro ?? null,
      type: book.type ?? 0,
      latestChapterTitle: book.latestChapterTitle ?? null,
      totalChapterNum: book.totalChapterNum ?? chapters.length,
    },
    engineChapter: {
      url: chapter.url,
      title: chapter.title,
      isVolume: chapter.isVolume ?? false,
      baseUrl: book.bookUrl,
      bookUrl: book.bookUrl,
      index: chapter.index,
      isVip: chapter.isVip ?? false,
      isPay: chapter.isPay ?? false,
      resourceUrl: chapter.resourceUrl ?? null,
      tag: chapter.tag ?? null,
    },
  };
}

export async function sourceReviews(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<unknown[]> {
  const context = await onlineContext(bookUrl, chapterIndex);
  return sourceReviewsWasm(
    JSON.stringify(context.source),
    JSON.stringify(context.engineBook),
    JSON.stringify(context.engineChapter),
  );
}

export async function sourceDownloadUrls(bookUrl: string): Promise<unknown[]> {
  const book = await db.books.get(bookUrl);
  if (!book?.origin) throw new Error("书籍没有关联书源");
  const source = await getBookSource(book.origin);
  if (source) await prepareSource(source);
  if (!source) throw new Error("书源不存在");
  const chapters = await db.chapters.where("bookUrl").equals(bookUrl).sortBy("index");
  return sourceDownloadUrlsWasm(
    JSON.stringify(source),
    JSON.stringify({
      bookUrl: book.bookUrl,
      tocUrl: book.tocUrl ?? "",
      origin: book.origin,
      originName: book.originName ?? source.bookSourceName,
      name: book.name,
      author: book.author,
      type: book.type ?? 0,
      totalChapterNum: book.totalChapterNum ?? chapters.length,
    }),
  );
}

export async function sourcePay(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<unknown> {
  const context = await onlineContext(bookUrl, chapterIndex);
  return sourcePayWasm(
    JSON.stringify(context.source),
    JSON.stringify(context.engineBook),
    JSON.stringify(context.engineChapter),
  );
}

export async function sourceResource(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<{
  url: string;
  title: string;
  index: number;
  isVip?: boolean;
  isPay?: boolean;
}> {
  const context = await onlineContext(bookUrl, chapterIndex);
  const value = await sourceResourceWasm(
    JSON.stringify(context.source),
    JSON.stringify(context.engineBook),
    JSON.stringify(context.engineChapter),
  );
  const url = (() => {
    try {
      return new URL(value, context.book.bookUrl).toString();
    } catch {
      return value;
    }
  })();
  return {
    url,
    title: context.chapter.title,
    index: context.chapter.index,
    isVip: context.chapter.isVip,
    isPay: context.chapter.isPay,
  };
}

function sourceHeader(source: { header?: string | null }): Record<string, string> {
  const raw = source.header?.trim();
  if (!raw) return {};
  try {
    const value = JSON.parse(raw) as unknown;
    if (!value || typeof value !== "object" || Array.isArray(value)) return {};
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>).map(([key, item]) => [
        key,
        String(item),
      ]),
    );
  } catch {
    return {};
  }
}

async function fetchProxied(url: string, source: { header?: string | null }): Promise<Blob> {
  const response = await fetch("/api/engine/fetch", {
    headers: {
      "X-Remote-Target": url,
      ...sourceHeader(source),
    },
  });
  if (!response.ok) throw new Error(`图片请求失败：${response.status}`);
  return response.blob();
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error ?? new Error("图片读取失败"));
    reader.readAsDataURL(blob);
  });
}

function imageUrls(html: string, baseUrl: string): string[] {
  const urls: string[] = [];
  const pattern = /<img[^>]+src\s*=\s*["']([^"']+)["']/gi;
  for (const match of html.matchAll(pattern)) {
    const raw = (match[1] ?? "")
      .replace(/&quot;/g, '"')
      .replace(/&#34;/g, '"')
      .replace(/&amp;/g, "&")
      .trim();
    if (!raw) continue;
    try {
      const value = new URL(raw, baseUrl).toString();
      if (!urls.includes(value)) urls.push(value);
    } catch {
      // ignore invalid image URL
    }
  }
  return urls;
}

export async function sourceImage(
  bookUrl: string,
  chapterIndex?: number | null,
  url?: string | null,
): Promise<{ url: string; title: string; index: number; dataUrl: string }> {
  const context = await onlineContext(bookUrl, chapterIndex);
  const resource = url?.trim()
    ? url.trim()
    : (
        await sourceResourceWasm(
          JSON.stringify(context.source),
          JSON.stringify(context.engineBook),
          JSON.stringify(context.engineChapter),
        )
      );
  const absolute = (() => {
    try {
      return new URL(resource, context.book.bookUrl).toString();
    } catch {
      return resource;
    }
  })();
  const blob = await fetchProxied(absolute, context.source);
  return {
    url: absolute,
    title: context.chapter.title,
    index: context.chapter.index,
    dataUrl: await blobToDataUrl(blob),
  };
}

export async function sourceImages(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<{
  title: string;
  index: number;
  images: { url: string; dataUrl?: string; error?: string }[];
}> {
  const context = await onlineContext(bookUrl, chapterIndex);
  const html = await sourceContent(
    JSON.stringify(context.source),
    JSON.stringify(context.engineBook),
    JSON.stringify(context.engineChapter),
  );
  let urls = imageUrls(html, context.chapter.url || context.book.bookUrl);
  if (!urls.length) {
    const resource = await sourceResourceWasm(
      JSON.stringify(context.source),
      JSON.stringify(context.engineBook),
      JSON.stringify(context.engineChapter),
    );
    if (resource.trim()) {
      try {
        urls = [new URL(resource, context.book.bookUrl).toString()];
      } catch {
        urls = [resource];
      }
    }
  }
  const images: { url: string; dataUrl?: string; error?: string }[] = [];
  for (const url of urls.slice(0, 80)) {
    try {
      const blob = await fetchProxied(url, context.source);
      images.push({ url, dataUrl: await blobToDataUrl(blob) });
    } catch (error) {
      images.push({
        url,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  return {
    title: context.chapter.title,
    index: context.chapter.index,
    images,
  };
}
