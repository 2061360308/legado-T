import { COMMANDS } from "@/core/application/commands";
import type {
  Book,
  BookChapter,
  BookGroup,
  Bookmark,
  BookSearchResult,
  BookSource,
  DictRule,
  DiffEntry,
  DiffReport,
  HttpTts,
  LayoutStyle,
  ReadRecord,
  ReaderSnapshot,
  ReplaceRule,
  RssArticle,
  RssSource,
  TxtTocRule,
} from "@/core/domain";
import {
  normalizeBook,
  normalizeHttpTts,
  normalizeReplaceRule,
  parseRuleArray,
} from "@/core/domain";
import { chapterKey, db, nowMs, nowSeconds } from "./database";
import { findPageIndex, findScrollOffset } from "./readerEngine";
import {
  analyzeTxtInWorker,
  processChapterInWorker,
  processContentInWorker,
  searchTextInWorker,
  shelfBooksInWorker,
} from "./readerEngineClient";
import { exportWebBackup, importWebBackup } from "./backupService";
import * as bookSources from "./bookSources";
import * as rssSources from "./rssSources";
import defaultTxtTocRules from "./defaultTxtTocRules.json";
import {
  archiveList,

  deleteImportPaths,
  extractEbookCoverBytes,
  extractEpubCoverBytes,
  extractPdfCoverBytes,
  extractUmdCoverBytes,
  importWebArchive,
  importWebPaths,

} from "./importService";
import {
  deleteBookFiles,
  ensureAssetUrl,
  pathForAssetUrl,
  primeAssetUrls,
  readWebBlob,
  readWebDir,
  readWebFile,
  releaseAssetUrl,
  webFileSize,
  writeWebFile,
  baseName,
  extensionOf,
} from "./storage";

const DEVICE_ID_KEY = "readRecordDeviceId";

type Args = Record<string, unknown>;

function argString(args: Args, key: string): string {
  const value = args[key];
  return typeof value === "string" ? value : "";
}

function argNumber(args: Args, key: string): number {
  const value = args[key];
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function argBoolean(args: Args, key: string): boolean {
  return args[key] === true;
}

function argArray<T>(args: Args, key: string): T[] {
  const value = args[key];
  return Array.isArray(value) ? (value as T[]) : [];
}

async function deviceId(): Promise<string> {
  const row = await db.kv.get(DEVICE_ID_KEY);
  if (row?.value) return row.value;
  const value = `local-${nowMs()}`;
  await db.kv.put({ key: DEVICE_ID_KEY, value });
  return value;
}

async function listBooks(): Promise<Book[]> {
  await primeAssetUrls(["cover"]).catch(() => undefined);
  const books = await db.books.toArray();
  return shelfBooksInWorker(books, -1, 3);
}

function normalizeAssetRef(value: string | null | undefined): string | null | undefined {
  if (!value) return value;
  return pathForAssetUrl(value) ?? value;
}

async function saveBook(book: Book): Promise<void> {
  await db.books.put(
    normalizeBook({
      ...book,
      coverUrl: normalizeAssetRef(book.coverUrl),
      customCoverUrl: normalizeAssetRef(book.customCoverUrl),
    }),
  );
}

async function deleteBook(bookUrl: string): Promise<void> {
  await db.transaction(
    "rw",
    db.books,
    db.chapters,
    db.chapterContents,
    db.chapterBlocks,
    async () => {
      await db.books.delete(bookUrl);
      await db.chapters.where("bookUrl").equals(bookUrl).delete();
      await db.chapterContents.where("bookUrl").equals(bookUrl).delete();
      await db.chapterBlocks.where("bookUrl").equals(bookUrl).delete();
    },
  );
  await deleteBookFiles(bookUrl);
}

async function listChapters(bookUrl: string): Promise<BookChapter[]> {
  const rows = await db.chapters.where("bookUrl").equals(bookUrl).sortBy("index");
  return rows.map(({ key: _key, bookUrl: _bookUrl, ...chapter }) => chapter);
}

async function saveGroup(group: BookGroup): Promise<BookGroup> {
  const name = group.groupName.trim();
  if (!name) throw new Error("分组名称不能为空");
  return db.transaction("rw", db.groups, async () => {
    const groups = await db.groups.toArray();
    if (
      groups.some(
        (item) =>
          item.groupId !== group.groupId &&
          item.groupId > 0 &&
          item.groupName.toLocaleLowerCase() === name.toLocaleLowerCase(),
      )
    ) {
      throw new Error("分组名称已存在");
    }
    let groupId = group.groupId;
    let order = group.order ?? 0;
    let show = group.show ?? true;
    if (groupId === 0) {
      let used = 0;
      for (const item of groups) {
        if (item.groupId > 0) used |= item.groupId;
      }
      groupId = 1;
      while ((used & groupId) !== 0) groupId <<= 1;
      if (groupId === 0) throw new Error("分组数量已达上限");
      order = groups.reduce((max, item) => Math.max(max, item.order ?? 0), 0) + 1;
      show = true;
    } else if (!groups.some((item) => item.groupId === groupId)) {
      throw new Error("分组不存在");
    }
    const saved: BookGroup = {
      ...group,
      groupId,
      groupName: name,
      order,
      show,
      bookSort: group.bookSort ?? 0,
      enableRefresh: group.enableRefresh ?? true,
    };
    await db.groups.put(saved);
    return saved;
  });
}

async function deleteGroup(groupId: number): Promise<void> {
  if (groupId === 0) throw new Error("无效分组");
  await db.transaction("rw", db.groups, db.books, async () => {
    await db.groups.delete(groupId);
    const books = await db.books.toArray();
    for (const book of books) {
      if (((book.group ?? 0) & groupId) !== 0) {
        await db.books.put({ ...book, group: (book.group ?? 0) & ~groupId });
      }
    }
  });
}

async function saveTxtTocRule(rule: TxtTocRule): Promise<TxtTocRule> {
  const saved: TxtTocRule = {
    ...rule,
    id: rule.id > 0 ? rule.id : nowMs(),
    serialNumber:
      rule.serialNumber && rule.serialNumber !== 0
        ? rule.serialNumber
        : (await db.txtTocRules.count()) + 1,
    enable: rule.enable ?? true,
  };
  await db.txtTocRules.put(saved);
  return saved;
}

const bundledTxtTocRules = (defaultTxtTocRules as TxtTocRule[]).map((rule) => ({
  ...rule,
  enable: rule.enable ?? true,
}));

let defaultTxtTocRulesReady: Promise<void> | null = null;

async function ensureDefaultTxtTocRules(): Promise<void> {
  if (!defaultTxtTocRulesReady) {
    defaultTxtTocRulesReady = (async () => {
      if ((await db.txtTocRules.count()) > 0) return;
      await db.txtTocRules.bulkPut(bundledTxtTocRules);
    })().catch((error) => {
      defaultTxtTocRulesReady = null;
      throw error;
    });
  }
  await defaultTxtTocRulesReady;
}

async function restoreDefaultTxtTocRules(): Promise<number> {
  await db.transaction("rw", db.txtTocRules, async () => {
    await db.txtTocRules.clear();
    await db.txtTocRules.bulkPut(bundledTxtTocRules);
  });
  defaultTxtTocRulesReady = Promise.resolve();
  return bundledTxtTocRules.length;
}

async function importTxtTocRules(json: string): Promise<DiffReport> {
  const items = parseRuleArray(json) as TxtTocRule[];
  const existing = new Set((await db.txtTocRules.toArray()).map((rule) => rule.id));
  const seen = new Set<number>();
  const entries: DiffEntry[] = [];
  for (const item of items) {
    const key = item.id;
    if (seen.has(key)) {
      entries.push({ key: String(key), status: "duplicate" });
      continue;
    }
    seen.add(key);
    const status = existing.has(key) ? "updated" : "new";
    entries.push({ key: String(key), status });
    await db.txtTocRules.put({ ...item, enable: item.enable ?? true });
  }
  return { entries };
}

async function saveReplaceRule(rule: ReplaceRule): Promise<ReplaceRule> {
  const saved = normalizeReplaceRule({
    ...rule,
    id: rule.id > 0 ? rule.id : nowMs(),
  });
  await db.replaceRules.put(saved);
  return saved;
}

async function importReplaceRules(json: string): Promise<DiffReport> {
  const items = parseRuleArray(json) as ReplaceRule[];
  const existing = new Set((await db.replaceRules.toArray()).map((rule) => rule.id));
  const seen = new Set<number>();
  const entries: DiffEntry[] = [];
  for (const item of items) {
    const rule = normalizeReplaceRule(item);
    if (seen.has(rule.id)) {
      entries.push({ key: String(rule.id), status: "duplicate" });
      continue;
    }
    seen.add(rule.id);
    const status = existing.has(rule.id) ? "updated" : "new";
    entries.push({ key: String(rule.id), status });
    await db.replaceRules.put(rule);
  }
  return { entries };
}

async function saveDictRule(rule: DictRule): Promise<DictRule> {
  const saved: DictRule = {
    ...rule,
    showRule: rule.showRule ?? "",
    enabled: rule.enabled ?? true,
    sortNumber: rule.sortNumber ?? 0,
    jsLib: rule.jsLib ?? "",
  };
  await db.dictRules.put(saved);
  return saved;
}

async function importDictRules(json: string): Promise<DiffReport> {
  const items = parseRuleArray(json) as DictRule[];
  const existing = new Set((await db.dictRules.toArray()).map((rule) => rule.name));
  const seen = new Set<string>();
  const entries: DiffEntry[] = [];
  for (const item of items) {
    const key = item.name;
    if (seen.has(key)) {
      entries.push({ key, status: "duplicate" });
      continue;
    }
    seen.add(key);
    const status = existing.has(key) ? "updated" : "new";
    entries.push({ key, status });
    await saveDictRule(item);
  }
  return { entries };
}

async function saveHttpTts(engine: HttpTts): Promise<HttpTts> {
  const saved = normalizeHttpTts({
    ...engine,
    id: engine.id > 0 ? engine.id : nowMs(),
  });
  await db.httpTts.put(saved);
  return saved;
}

async function importHttpTts(json: string): Promise<DiffReport> {
  const items = parseRuleArray(json) as HttpTts[];
  const existing = new Set((await db.httpTts.toArray()).map((engine) => engine.id));
  const seen = new Set<number>();
  const entries: DiffEntry[] = [];
  for (const item of items) {
    const engine = normalizeHttpTts({
      ...item,
      id: item.id > 0 ? item.id : nowMs(),
    });
    if (seen.has(engine.id)) {
      entries.push({ key: String(engine.id), status: "duplicate" });
      continue;
    }
    seen.add(engine.id);
    const status = existing.has(engine.id) ? "updated" : "new";
    entries.push({ key: String(engine.id), status });
    await db.httpTts.put(engine);
  }
  return { entries };
}

async function searchBook(
  bookUrl: string,
  keyword: string,
  limit: number,
): Promise<BookSearchResult[]> {
  if (!keyword.trim()) return [];
  const book = await db.books.get(bookUrl);
  if (!book) return [];
  const [rows, rules, converter, keepTitle] = await Promise.all([
    db.chapterContents.where("bookUrl").equals(bookUrl).toArray(),
    db.replaceRules.toArray(),
    db.kv.get("chineseConverterType"),
    db.kv.get(`keepDuplicateTitle:${bookUrl}`),
  ]);
  const direction = Number(converter?.value ?? 0) || 0;
  const chapters = [];
  for (const row of rows.sort((a, b) => a.index - b.index)) {
    const chapter = await db.chapters.get(chapterKey(bookUrl, row.index));
    const title = chapter?.title ?? "";
    const processed = await processContentInWorker({
      title,
      content: row.content,
      bookName: book.name,
      bookOrigin: book.originName ?? "",
      rules,
      includeTitle: true,
      useReplace: false,
      useReplaceRule: book.readConfig?.useReplaceRule ?? true,
      chineseConvert: true,
      chineseConverterType: direction,
      reSegment: book.readConfig?.reSegment ?? false,
      keepDuplicateTitle: keepTitle?.value === "true",
      paragraphIndent: "\u3000\u3000",
      isAndroid8: false,
    });
    chapters.push({
      index: row.index,
      title: processed.displayTitle || title,
      content: processed.contents.join("\n"),
    });
  }
  return searchTextInWorker(chapters, keyword, limit);
}

const DEFAULT_STYLE: LayoutStyle = {
  fontSize: 20,
  titleFontSize: 24,
  titleBold: true,
  titleMode: 0,
  titleTopSpacing: 0,
  titleBottomSpacing: 0,
  lineSpacing: 1.2,
  paragraphSpacing: 4,
  textBold: 0,
  letterSpacing: 0.1,
  paragraphIndent: 2,
  paddingLeft: 20,
  paddingRight: 20,
  paddingTop: 32,
  paddingBottom: 20,
  fullJustify: true,
  bottomJustify: true,
  useZhLayout: true,
  viewWidth: 360,
  viewHeight: 640,
  imageStyle: "DEFAULT",
};



async function openReader(
  bookUrl: string,
  chapterIndex: number | null,
  chapterPos: number | null,
  style: LayoutStyle,
): Promise<ReaderSnapshot> {
  let book = await db.books.get(bookUrl);
  if (!book) throw new Error("书籍不存在");
  if (!book.tocUrl && book.sourcePath && /\.txt$/i.test(book.sourcePath)) {
    const migrationKey = `txtTocMigration:${bookUrl}`;
    if (!(await db.kv.get(migrationKey))) {
      await db.kv.put({ key: migrationKey, value: "1" });
      try {
        await resplitBook(bookUrl, null, book.charset ?? null);
        book = (await db.books.get(bookUrl)) ?? book;
      } catch {
        // Keep the existing chapters if migration cannot parse this file.
      }
    }
  }
  const chapters = await listChapters(bookUrl);
  if (!chapters.length) throw new Error("书籍没有章节");
  const target = Math.min(
    Math.max(chapterIndex ?? book.durChapterIndex ?? 0, 0),
    chapters.length - 1,
  );
  const chapter = chapters.find((item) => item.index === target) ?? chapters[target];
  const key = chapterKey(bookUrl, target);
  const [row, blockRow] = await Promise.all([
    db.chapterContents.get(key),
    db.chapterBlocks.get(key),
  ]);
  if (!row && !blockRow) throw new Error("章节内容不存在，请重新导入");
  const [rules, converter, keepTitle] = await Promise.all([
    db.replaceRules.toArray(),
    db.kv.get("chineseConverterType"),
    db.kv.get(`keepDuplicateTitle:${bookUrl}`),
  ]);
  const direction = Number(converter?.value ?? 0) || 0;
  const useReplaceRule = book.readConfig?.useReplaceRule ?? true;
  const effectiveStyle = style ?? DEFAULT_STYLE;
  const output = await processChapterInWorker({
    process: {
      title: chapter.title,
      content: row?.content ?? "",
      bookName: book.name,
      bookOrigin: book.originName ?? "",
      rules,
      includeTitle: false,
      useReplace: true,
      useReplaceRule,
      chineseConvert: true,
      chineseConverterType: direction,
      reSegment: book.readConfig?.reSegment ?? false,
      keepDuplicateTitle: keepTitle?.value === "true",
      paragraphIndent: "\u3000\u3000",
      isAndroid8: false,
    },
    blocks: blockRow?.blocks,
    style: effectiveStyle,
  });
  const paragraphs = output.paragraphs;
  const chapterTitle = output.displayTitle || chapter.title;
  for (const block of output.blocks) {
    if (block.type === "image") await ensureAssetUrl(block.src).catch(() => undefined);
  }
  const pos = chapterPos ?? book.durChapterPos ?? 0;
  const pageIndex = effectiveStyle.scroll ? 0 : findPageIndex(output.pages, pos);
  const page = output.pages[pageIndex];
  const scrollOffset = effectiveStyle.scroll && page
    ? findScrollOffset(page, pos, effectiveStyle.paddingTop)
    : 0;
  return {
    book,
    chapterIndex: target,
    pageIndex,
    chapterTitle,
    pages: output.pages,
    effectiveRules: output.effectiveRules,
    chapterParagraphs: paragraphs,
    totalChapters: chapters.length,
    pageStartChar: page?.startChar ?? 0,
    pageEndChar: page?.endChar ?? 0,
    scrollOffset,
    marginFellBack: output.marginFellBack,
  };
}

async function resplitBook(
  bookUrl: string,
  rule: string | null,
  charset: string | null,
): Promise<number> {
  const book = await db.books.get(bookUrl);
  if (!book?.sourcePath) throw new Error("书籍源文件不存在");
  const bytes = await readWebFile(book.sourcePath);
  const rules = await db.txtTocRules.toArray();
  const analyzed = await analyzeTxtInWorker(
    bytes,
    charset ?? book.charset ?? "",
    rules,
    true,
    rule ?? book.tocUrl ?? "",
  );
  if (!analyzed.chapters.length) throw new Error("没有解析到章节");
  const decoder = new TextDecoder(analyzed.charset);
  const contents = analyzed.chapters.map((chapter) => {
    let text = decoder.decode(bytes.subarray(chapter.start, chapter.end));
    const firstLine = text.split(/\r?\n/, 1)[0]?.trim();
    if (firstLine === chapter.title.trim()) text = text.slice(text.indexOf("\n") + 1);
    return text.replace(/^[ \t]+/gm, "\u3000\u3000");
  });
  await db.transaction(
    "rw",
    db.books,
    db.chapters,
    db.chapterContents,
    db.chapterBlocks,
    async () => {
      await db.chapters.where("bookUrl").equals(bookUrl).delete();
      await db.chapterContents.where("bookUrl").equals(bookUrl).delete();
      await db.chapterBlocks.where("bookUrl").equals(bookUrl).delete();
      const chapters: BookChapter[] = analyzed.chapters.map((chapter, index) => ({
        url: `${bookUrl}#${index}`,
        title: chapter.title,
        index,
        isVolume: chapter.isVolume,
        start: chapter.start,
        end: chapter.end,
        resourceUrl: bookUrl,
      }));
      await db.chapters.bulkPut(
        chapters.map((chapter) => ({
          ...chapter,
          key: chapterKey(bookUrl, chapter.index),
          bookUrl,
        })),
      );
      await db.chapterContents.bulkPut(
        contents.map((content, index) => ({
          key: chapterKey(bookUrl, index),
          bookUrl,
          index,
          content,
        })),
      );
      await db.books.put({
        ...book,
        charset: analyzed.charset,
        tocUrl: analyzed.rule,
        intro: analyzed.intro || book.intro,
        totalChapterNum: analyzed.chapters.length,
      });
    },
  );
  return analyzed.chapters.length;
}

export function createWebBackend() {
  return {
    async invoke<T>(command: string, args: Args = {}): Promise<T> {
      await ensureDefaultTxtTocRules();
      switch (command) {
        case COMMANDS.appVersion:
          return "0.1.0-web" as T;
        case COMMANDS.bookList:
          return (await listBooks()) as T;
        case COMMANDS.bookGet:
          await primeAssetUrls(["cover"]).catch(() => undefined);
          return ((await db.books.get(argString(args, "bookUrl"))) ?? null) as T;
        case COMMANDS.bookUpsert:
          await saveBook(args.book as Book);
          return undefined as T;
        case COMMANDS.bookSetRemoteOrigin:
          await db.books.update(argString(args, "bookUrl"), {
            origin: argString(args, "origin"),
          });
          return undefined as T;
        case COMMANDS.bookDelete:
          await deleteBook(argString(args, "bookUrl"));
          return undefined as T;
        case COMMANDS.chapterList:
          return (await listChapters(argString(args, "bookUrl"))) as T;
        case COMMANDS.groupList: {
          const groups = await db.groups.toArray();
          groups.sort(
            (a, b) => (a.order ?? 0) - (b.order ?? 0) || a.groupId - b.groupId,
          );
          return groups as T;
        }
        case COMMANDS.groupSave:
          return (await saveGroup(args.group as BookGroup)) as T;
        case COMMANDS.groupDelete:
          await deleteGroup(argNumber(args, "groupId"));
          return undefined as T;
        case COMMANDS.groupCoverSave: {
          const srcPath = argString(args, "srcPath");
          const blob = await readWebBlob(srcPath);
          const ext = extensionOf(srcPath) || "jpg";
          const target = `idb://app-data/covers/group-${nowMs()}.${ext}`;
          await writeWebFile(target, blob, {
            kind: "cover",
            mime: blob.type || `image/${ext}`,
          });
          await ensureAssetUrl(target);
          return target as T;
        }
        case COMMANDS.txtTocRuleList: {
          const rules = await db.txtTocRules.toArray();
          rules.sort((a, b) => (b.serialNumber ?? 0) - (a.serialNumber ?? 0));
          return rules as T;
        }
        case COMMANDS.txtTocRuleSave:
          return (await saveTxtTocRule(args.rule as TxtTocRule)) as T;
        case COMMANDS.txtTocRuleDelete:
          await db.txtTocRules.delete(argNumber(args, "id"));
          return undefined as T;
        case COMMANDS.txtTocRuleReorder: {
          const ids = argArray<number>(args, "ids");
          const rules = await db.txtTocRules.toArray();
          const byId = new Map(rules.map((rule) => [rule.id, rule]));
          const total = ids.length;
          for (let index = 0; index < ids.length; index += 1) {
            const rule = byId.get(ids[index]);
            if (rule) await db.txtTocRules.put({ ...rule, serialNumber: total - index });
          }
          return undefined as T;
        }
        case COMMANDS.txtTocRuleRestoreDefault:
          return (await restoreDefaultTxtTocRules()) as T;
        case COMMANDS.txtTocRuleImport:
          return (await importTxtTocRules(argString(args, "json"))) as T;
        case COMMANDS.txtTocRuleExport: {
          const rules = await db.txtTocRules.toArray();
          rules.sort((a, b) => (b.serialNumber ?? 0) - (a.serialNumber ?? 0));
          return JSON.stringify(rules, null, 2) as T;
        }
        case COMMANDS.kvGet:
          return ((await db.kv.get(argString(args, "key")))?.value ?? null) as T;
        case COMMANDS.kvSet:
          await db.kv.put({ key: argString(args, "key"), value: argString(args, "value") });
          return undefined as T;
        case COMMANDS.readRecordAddTime: {
          const seconds = argNumber(args, "seconds");
          if (seconds <= 0) return undefined as T;
          const book = await db.books.get(argString(args, "bookUrl"));
          if (!book) return undefined as T;
          const device = await deviceId();
          const existing = await db.readRecords.get([device, book.name]);
          const record: ReadRecord = {
            deviceId: device,
            bookName: book.name,
            author: book.author,
            readTime: (existing?.readTime ?? 0) + seconds,
            lastRead: nowSeconds(),
            durChapterIndex: book.durChapterIndex ?? 0,
            durChapterPos: book.durChapterPos ?? 0,
            durChapterTitle: book.durChapterTitle ?? "",
          };
          await db.readRecords.put(record);
          return undefined as T;
        }
        case COMMANDS.readRecordList: {
          const records = await db.readRecords.toArray();
          records.sort((a, b) => (b.lastRead ?? 0) - (a.lastRead ?? 0));
          return records as T;
        }
        case COMMANDS.readRecordDelete:
          await db.readRecords.delete([
            argString(args, "deviceId"),
            argString(args, "bookName"),
          ]);
          return undefined as T;
        case COMMANDS.readRecordClear:
          await db.readRecords.clear();
          return undefined as T;
        case COMMANDS.readerSaveProgress: {
          const bookUrl = argString(args, "bookUrl");
          const book = await db.books.get(bookUrl);
          if (!book) return undefined as T;
          await db.books.put({
            ...book,
            durChapterIndex: argNumber(args, "chapterIndex"),
            durChapterPos: argNumber(args, "chapterPos"),
            durChapterTitle: argString(args, "chapterTitle"),
            durChapterTime: nowMs(),
          });
          return undefined as T;
        }
        case COMMANDS.readerSaveChapterContent: {
          const bookUrl = argString(args, "bookUrl");
          const index = argNumber(args, "chapterIndex");
          const key = chapterKey(bookUrl, index);
          await db.chapterContents.put({
            key,
            bookUrl,
            index,
            content: argString(args, "content"),
          });
          await db.chapterBlocks.delete(key);
          return undefined as T;
        }
        case COMMANDS.readerInvalidateBook:
        case COMMANDS.readerInvalidateAll:
          return undefined as T;
        case COMMANDS.readerSearchBook:
          return (await searchBook(
            argString(args, "bookUrl"),
            argString(args, "keyword"),
            argNumber(args, "limit") || 200,
          )) as T;
        case COMMANDS.readerEffectiveRules: {
          const rules = await db.replaceRules.toArray();
          return rules.filter((rule) => rule.isEnabled !== false) as T;
        }
        case COMMANDS.bookmarkList: {
          const bookmarks = await db.bookmarks.toArray();
          bookmarks.sort((a, b) => b.time - a.time);
          return bookmarks as T;
        }
        case COMMANDS.bookmarkAdd: {
          const bookmark: Bookmark = {
            time: nowMs(),
            bookName: argString(args, "bookName"),
            bookAuthor: argString(args, "bookAuthor"),
            bookUrl: argString(args, "bookUrl"),
            chapterIndex: argNumber(args, "chapterIndex"),
            chapterPos: argNumber(args, "chapterPos"),
            chapterName: argString(args, "chapterName"),
            bookText: argString(args, "bookText"),
            content: (args.content as string | null | undefined) ?? null,
          };
          await db.bookmarks.put(bookmark);
          return bookmark as T;
        }
        case COMMANDS.bookmarkUpdate: {
          const time = argNumber(args, "time");
          const bookmark = await db.bookmarks.get(time);
          if (!bookmark) throw new Error("书签不存在");
          const updated: Bookmark = {
            ...bookmark,
            bookText: argString(args, "bookText"),
            content: (args.content as string | null | undefined) ?? null,
          };
          await db.bookmarks.put(updated);
          return updated as T;
        }
        case COMMANDS.bookmarkDelete:
          await db.bookmarks.delete(argNumber(args, "time"));
          return undefined as T;
        case "bookshelf_list": {
          const groupId = argNumber(args, "groupId");
          const sort = argNumber(args, "sort");
          await primeAssetUrls(["cover"]).catch(() => undefined);
          return (await shelfBooksInWorker(await db.books.toArray(), groupId, sort)) as T;
        }
        case "bookshelf_export":
          return JSON.stringify(await listBooks(), null, 2) as T;
        case "book_move_to_top": {
          const bookUrl = argString(args, "bookUrl");
          const book = await db.books.get(bookUrl);
          if (!book) throw new Error("书籍不存在");
          const minOrder = (await db.books.toArray()).reduce(
            (min, item) => Math.min(min, item.order ?? 0),
            0,
          );
          await db.books.put({ ...book, order: minOrder - 1 });
          return undefined as T;
        }
        case "book_set_group": {
          const book = await db.books.get(argString(args, "bookUrl"));
          if (!book) throw new Error("书籍不存在");
          await db.books.put({ ...book, group: argNumber(args, "group") });
          return undefined as T;
        }
        case "book_add_group": {
          const book = await db.books.get(argString(args, "bookUrl"));
          if (!book) throw new Error("书籍不存在");
          await db.books.put({ ...book, group: (book.group ?? 0) | argNumber(args, "group") });
          return undefined as T;
        }
        case "book_batch_add_group": {
          const group = argNumber(args, "group");
          let count = 0;
          for (const bookUrl of argArray<string>(args, "bookUrls")) {
            const book = await db.books.get(bookUrl);
            if (!book) continue;
            await db.books.put({ ...book, group: (book.group ?? 0) | group });
            count += 1;
          }
          return count as T;
        }
        case "book_set_can_update": {
          const book = await db.books.get(argString(args, "bookUrl"));
          if (!book) throw new Error("书籍不存在");
          await db.books.put({ ...book, canUpdate: argBoolean(args, "value") });
          return undefined as T;
        }
        case "book_batch_set_can_update": {
          const value = argBoolean(args, "value");
          let count = 0;
          for (const bookUrl of argArray<string>(args, "bookUrls")) {
            const book = await db.books.get(bookUrl);
            if (!book) continue;
            await db.books.put({ ...book, canUpdate: value });
            count += 1;
          }
          return count as T;
        }
        case "book_file_size": {
          const bookUrl = argString(args, "bookUrl");
          const book = await db.books.get(bookUrl);
          const rows = await db.files.where("bookUrl").equals(bookUrl).toArray();
          const generated = rows
            .filter((row) => row.kind !== "book")
            .reduce((sum, row) => sum + row.size, 0);
          const source = book?.sourcePath ? await webFileSize(book.sourcePath) : 0;
          return (source + generated) as T;
        }
        case "book_clear_cache": {
          const bookUrl = argString(args, "bookUrl");
          await db.chapterContents.where("bookUrl").equals(bookUrl).delete();
          await db.chapterBlocks.where("bookUrl").equals(bookUrl).delete();
          const images = await db.files
            .where("bookUrl")
            .equals(bookUrl)
            .filter((row) => row.kind === "image")
            .toArray();
          for (const row of images) {
            await db.files.delete(row.path);
            releaseAssetUrl(row.path);
          }
          return undefined as T;
        }
        case "chapter_word_counts": {
          const bookUrl = argString(args, "bookUrl");
          const rows = await db.chapterContents.where("bookUrl").equals(bookUrl).toArray();
          const counts = new Map<number, number>();
          for (const row of rows) counts.set(row.index, Array.from(row.content).length);
          return Array.from(counts, ([index, count]) => ({ index, count })) as T;
        }
        case "replace_rule_list": {
          const rules = await db.replaceRules.toArray();
          rules.sort((a, b) => (a.sortOrder ?? 0) - (b.sortOrder ?? 0));
          return rules as T;
        }
        case "replace_rule_save":
          return (await saveReplaceRule(args.rule as ReplaceRule)) as T;
        case "replace_rule_delete":
          await db.replaceRules.delete(argNumber(args, "id"));
          return undefined as T;
        case "replace_rule_import":
          return (await importReplaceRules(argString(args, "json"))) as T;
        case "replace_rule_export": {
          const rules = await db.replaceRules.toArray();
          rules.sort((a, b) => (a.sortOrder ?? 0) - (b.sortOrder ?? 0));
          return JSON.stringify(rules, null, 2) as T;
        }
        case "dict_rule_list": {
          const rules = await db.dictRules.toArray();
          rules.sort((a, b) => (b.sortNumber ?? 0) - (a.sortNumber ?? 0));
          return rules as T;
        }
        case "dict_rule_save":
          return (await saveDictRule(args.rule as DictRule)) as T;
        case "dict_rule_delete":
          await db.dictRules.delete(argString(args, "name"));
          return undefined as T;
        case "dict_rule_import":
          return (await importDictRules(argString(args, "json"))) as T;
        case "dict_rule_export": {
          const rules = await db.dictRules.toArray();
          rules.sort((a, b) => (b.sortNumber ?? 0) - (a.sortNumber ?? 0));
          return JSON.stringify(rules, null, 2) as T;
        }
        case "http_tts_list": {
          const engines = await db.httpTts.toArray();
          engines.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
          return engines as T;
        }
        case "http_tts_save":
          return (await saveHttpTts(args.engine as HttpTts)) as T;
        case "http_tts_delete":
          await db.httpTts.delete(argNumber(args, "id"));
          return undefined as T;
        case "http_tts_import":
          return (await importHttpTts(argString(args, "json"))) as T;
        case "http_tts_export": {
          const engines = await db.httpTts.toArray();
          engines.sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
          return JSON.stringify(engines, null, 2) as T;
        }
        case "import_txt":
        case "import_book": {
          const results = await importWebPaths(
            [argString(args, "path")],
            argNumber(args, "group"),
            (args.charset as string | null | undefined) ?? null,
          );
          const result = results[0];
          if (!result || result.error) throw new Error(result?.error ?? "导入失败");
          const book = result.bookUrl ? await db.books.get(result.bookUrl) : null;
          return {
            book,
            chapterCount: book?.totalChapterNum ?? 0,
            charset: book?.charset ?? "UTF-8",
            rule: book?.tocUrl ?? "",
            intro: book?.intro ?? "",
          } as T;
        }
        case "re_split_book":
          return (await resplitBook(
            argString(args, "bookUrl"),
            (args.rule as string | null | undefined) ?? null,
            (args.charset as string | null | undefined) ?? null,
          )) as T;
        case "read_chapter": {
          const row = await db.chapterContents.get(
            chapterKey(argString(args, "bookUrl"), argNumber(args, "index")),
          );
          if (!row) throw new Error("章节内容不存在");
          return row.content as T;
        }
        case COMMANDS.onlineReaderOpen: {
          const bookUrl = argString(args, "bookUrl");
          const book = await db.books.get(bookUrl);
          if (!book) throw new Error("书籍不存在");
          const chapters = await listChapters(bookUrl);
          if (!chapters.length) throw new Error("书籍没有章节");
          const target = Math.min(
            Math.max(
              args.chapterIndex == null ? book.durChapterIndex ?? 0 : argNumber(args, "chapterIndex"),
              0,
            ),
            chapters.length - 1,
          );
          const content = await bookSources.sourceChapterContent(bookUrl, target);
          if (!content.trim()) throw new Error("章节正文为空");
          await db.chapterContents.put({
            key: chapterKey(bookUrl, target),
            bookUrl,
            index: target,
            content,
          });
          return (await openReader(
            bookUrl,
            target,
            args.chapterPos == null ? null : argNumber(args, "chapterPos"),
            (args.style as LayoutStyle | undefined) ?? DEFAULT_STYLE,
          )) as T;
        }
        case COMMANDS.readerOpen:
          return (await openReader(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
            args.chapterPos == null ? null : argNumber(args, "chapterPos"),
            (args.style as LayoutStyle | undefined) ?? DEFAULT_STYLE,
          )) as T;
        case COMMANDS.localImportList:
        case COMMANDS.localImportScan: {
          const path = argString(args, "path");
          const [entries, books] = await Promise.all([
            readWebDir(path),
            db.books.toArray(),
          ]);
          const shelf = new Map<string, string>();
          for (const book of books) {
            if (book.originName) shelf.set(book.originName, book.bookUrl);
          }
          const result = [];
          for (const entry of entries) {
            const entryPath = `${path.replace(/\/$/, "")}/${entry.name}`;
            const row = entry.isFile ? await db.files.get(entryPath) : null;
            const ext = entry.isFile ? extensionOf(entry.name) : "";
            const bookUrl = entry.isFile ? shelf.get(entry.name) ?? null : null;
            result.push({
              path: entryPath,
              name: entry.name,
              isDir: entry.isDirectory,
              size: row?.size ?? 0,
              modified: row ? Math.floor(row.updated / 1000) : 0,
              ext,
              supported: ["txt", "epub", "pdf", "umd", "mobi", "azw", "azw3", "zip"].includes(ext),
              onShelf: Boolean(bookUrl),
              bookUrl,
            });
          }
          return result as T;
        }
        case COMMANDS.localImportImport:
          return (await importWebPaths(
            argArray<string>(args, "paths"),
            argNumber(args, "group"),
            (args.charset as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.localImportArchiveList:
          return (await archiveList(argString(args, "path"))) as T;
        case COMMANDS.localImportArchiveImport:
          return (await importWebArchive(
            argString(args, "path"),
            argArray<string>(args, "names"),
            argNumber(args, "group"),
            (args.charset as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.localImportReimport: {
          const path = argString(args, "path");
          const name = baseName(path);
          const books = await db.books.toArray();
          const book = books.find((item) => item.originName === name);
          if (book) {
            return {
              path,
              name,
              bookUrl: book.bookUrl,
              duplicate: true,
              error: null,
            } as T;
          }
          const results = await importWebPaths(
            [path],
            argNumber(args, "group"),
            (args.charset as string | null | undefined) ?? null,
          );
          return (results[0] ?? {
            path,
            name,
            bookUrl: null,
            duplicate: false,
            error: "导入失败",
          }) as T;
        }
        case COMMANDS.localImportDelete:
          return (await deleteImportPaths(argArray<string>(args, "paths"))) as T;
        case COMMANDS.bookCoverSave: {
          const bookUrl = argString(args, "bookUrl");
          const srcPath = argString(args, "srcPath");
          const book = await db.books.get(bookUrl);
          if (!book) throw new Error("书籍不存在");
          const blob = await readWebBlob(srcPath);
          const ext = extensionOf(srcPath) || "jpg";
          const target = `idb://app-data/books/${bookUrl}/cover.${ext}`;
          await writeWebFile(target, blob, {
            kind: "cover",
            bookUrl,
            mime: blob.type || `image/${ext}`,
          });
          await db.books.put({ ...book, coverUrl: target });
          await ensureAssetUrl(target);
          return target as T;
        }
        case COMMANDS.bookCoverRefresh: {
          const bookUrl = argString(args, "bookUrl");
          const book = await db.books.get(bookUrl);
          if (!book?.sourcePath) return null as T;
          const bytes = await readWebFile(book.sourcePath);
          const ext = extensionOf(book.sourcePath);
          const cover =
            ext === "epub"
              ? await extractEpubCoverBytes(bytes, book.name)
              : ext === "umd"
                ? await extractUmdCoverBytes(bytes)
                : ext === "pdf"
                  ? await extractPdfCoverBytes(bytes)
                  : ext === "mobi" || ext === "azw" || ext === "azw3"
                    ? await extractEbookCoverBytes(bytes, book.name)
                    : null;
          if (!cover) return null as T;
          const target = `idb://app-data/books/${bookUrl}/cover.${cover.ext}`;
          await writeWebFile(target, cover.bytes, {
            kind: "cover",
            bookUrl,
            mime: `image/${cover.ext}`,
          });
          await db.books.put({ ...book, coverUrl: target });
          await ensureAssetUrl(target);
          return target as T;
        }
        case COMMANDS.backupExport:
          return (await exportWebBackup(
            argString(args, "path"),
            (args.password as string | null | undefined) ?? null,
            (args.settingsJson as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.backupImport:
          return (await importWebBackup(
            argString(args, "path"),
            (args.password as string | null | undefined) ?? null,
            argArray<string>(args, "ignore"),
          )) as T;
        case COMMANDS.bookSourceList:
          return (await bookSources.listBookSources()) as T;
        case COMMANDS.bookSourceGet:
          return (await bookSources.getBookSource(argString(args, "bookSourceUrl"))) as T;
        case COMMANDS.bookSourceSave:
          await bookSources.saveBookSource(args.source as BookSource);
          return undefined as T;
        case COMMANDS.bookSourceDelete:
          await bookSources.deleteBookSource(argString(args, "bookSourceUrl"));
          return undefined as T;
        case COMMANDS.bookSourceSetEnabled:
          await bookSources.setBookSourceEnabled(
            argString(args, "bookSourceUrl"),
            argBoolean(args, "enabled"),
          );
          return undefined as T;
        case COMMANDS.bookSourceSetExploreEnabled:
          await bookSources.setBookSourceExploreEnabled(
            argString(args, "bookSourceUrl"),
            argBoolean(args, "enabled"),
          );
          return undefined as T;
        case COMMANDS.bookSourceImport:
          return (await bookSources.importBookSources(argString(args, "json"))) as T;
        case COMMANDS.bookSourceExport:
          return (await bookSources.exportBookSources()) as T;
        case COMMANDS.bookSourceGroups:
          return (await bookSources.bookSourceGroups()) as T;
        case COMMANDS.bookSourceSearchHistory:
          return (await bookSources.listSourceSearchHistory(argNumber(args, "limit") || 20)) as T;
        case COMMANDS.bookSourceSetVariable:
          await bookSources.setSourceVariable(
            argString(args, "bookSourceUrl"),
            argString(args, "key"),
            argString(args, "value"),
          );
          return undefined as T;
        case COMMANDS.bookSourceDeleteVariable:
          await bookSources.deleteSourceVariable(
            argString(args, "bookSourceUrl"),
            argString(args, "key"),
          );
          return undefined as T;
        case COMMANDS.bookSourceGetVariables:
          return (await bookSources.getSourceVariables(argString(args, "bookSourceUrl"))) as T;
        case COMMANDS.bookSourceSearch:
          return (await bookSources.searchBookSources(
            argString(args, "keyword"),
            args.sourceUrls as string[] | undefined,
            argNumber(args, "page") || 1,
          )) as T;
        case COMMANDS.bookSourceInfo:
          return (await bookSources.sourceBookInfo(
            argString(args, "bookSourceUrl"),
            argString(args, "bookUrl"),
          )) as T;
        case COMMANDS.bookSourceToc:
          return (await bookSources.sourceBookToc(
            argString(args, "bookSourceUrl"),
            args.book,
          )) as T;
        case COMMANDS.bookSourceContent:
          return (await bookSources.sourceBookContent(
            argString(args, "bookSourceUrl"),
            args.book,
            args.chapter,
            (args.nextChapterUrl as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.bookSourceExploreCategories:
          return (await bookSources.exploreCategories(argString(args, "bookSourceUrl"))) as T;
        case COMMANDS.bookSourceExplore:
          return (await bookSources.sourceExplore(
            argString(args, "bookSourceUrl"),
            argString(args, "url"),
            argNumber(args, "page") || 1,
          )) as T;
        case COMMANDS.bookSourceCheck:
          return (await bookSources.checkBookSources(
            args.sourceUrls as string[] | undefined,
            argString(args, "keyword") || "斗破",
          )) as T;
        case COMMANDS.bookSourceAddToShelf: {
          const sourceUrl = argString(args, "bookSourceUrl");
          const info = (await bookSources.sourceBookInfo(
            sourceUrl,
            argString(args, "bookUrl"),
          )) as Record<string, unknown>;
          const source = await bookSources.getBookSource(sourceUrl);
          if (!source) throw new Error("书源不存在");
          const chapters = (await bookSources.sourceBookToc(sourceUrl, info)) as Record<
            string,
            unknown
          >[];
          const book: Book = {
            bookUrl: String(info.bookUrl ?? argString(args, "bookUrl")),
            name: String(info.name ?? ""),
            author: String(info.author ?? ""),
            origin: sourceUrl,
            originName: source.bookSourceName,
            type: 1 << Math.max(0, Math.min(3, source.bookSourceType ?? 0)),
            group: argNumber(args, "group"),
            tocUrl: String(info.tocUrl ?? ""),
            intro: (info.intro as string | null | undefined) ?? null,
            kind: (info.kind as string | null | undefined) ?? null,
            coverUrl: (info.coverUrl as string | null | undefined) ?? null,
            latestChapterTitle:
              (info.latestChapterTitle as string | null | undefined) ?? null,
            totalChapterNum: chapters.length,
            canUpdate: true,
            syncTime: nowSeconds(),
          };
          await saveBook(book);
          await db.transaction("rw", db.chapters, async () => {
            await db.chapters.where("bookUrl").equals(book.bookUrl).delete();
            await db.chapters.bulkPut(
              chapters.map((chapter, index) => ({
                ...(chapter as unknown as BookChapter),
                index: Number(chapter.index ?? index),
                key: chapterKey(book.bookUrl, Number(chapter.index ?? index)),
                bookUrl: book.bookUrl,
              })),
            );
          });
          return info as T;
        }
        case COMMANDS.bookSourceRefresh: {
          const book = await db.books.get(argString(args, "bookUrl"));
          if (!book?.origin) throw new Error("书籍不存在");
          const info = (await bookSources.sourceBookInfo(book.origin, book.bookUrl)) as Record<
            string,
            unknown
          >;
          await db.books.put({
            ...book,
            name: String(info.name ?? book.name),
            author: String(info.author ?? book.author),
            intro: (info.intro as string | null | undefined) ?? book.intro,
            kind: (info.kind as string | null | undefined) ?? book.kind,
            coverUrl: (info.coverUrl as string | null | undefined) ?? book.coverUrl,
            tocUrl: String(info.tocUrl ?? book.tocUrl ?? ""),
            latestChapterTitle:
              (info.latestChapterTitle as string | null | undefined) ?? book.latestChapterTitle,
            totalChapterNum:
              Number(info.totalChapterNum ?? book.totalChapterNum ?? 0) || book.totalChapterNum,
            syncTime: nowSeconds(),
          });
          return info as T;
        }
        case COMMANDS.bookSourceReviews:
          return (await bookSources.sourceReviews(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
          )) as T;
        case COMMANDS.bookSourceDownloadUrls:
          return (await bookSources.sourceDownloadUrls(argString(args, "bookUrl"))) as T;
        case COMMANDS.bookSourcePay:
          return (await bookSources.sourcePay(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
          )) as T;
        case COMMANDS.bookSourceResource:
          return (await bookSources.sourceResource(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
          )) as T;
        case COMMANDS.bookSourceImage:
          return (await bookSources.sourceImage(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
            (args.url as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.bookSourceImages:
          return (await bookSources.sourceImages(
            argString(args, "bookUrl"),
            args.chapterIndex == null ? null : argNumber(args, "chapterIndex"),
          )) as T;
        case COMMANDS.bookSourceSaveCookies:
          return (await bookSources.saveSourceCookies(
            argString(args, "sourceUrl"),
            argArray<string>(args, "cookies"),
            (args.cookieUrl as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.bookSourceClearCookies:
          await bookSources.clearSourceCookies(argString(args, "sourceUrl"));
          return undefined as T;
        case COMMANDS.rssSourceList:
          return (await rssSources.listRssSources()) as T;
        case COMMANDS.rssSourceSave:
          await rssSources.saveRssSource(args.source as RssSource);
          return undefined as T;
        case COMMANDS.rssSourceDelete:
          await rssSources.deleteRssSource(argString(args, "sourceUrl"));
          return undefined as T;
        case COMMANDS.rssSourceImport:
          return (await rssSources.importRssSources(argString(args, "json"))) as T;
        case COMMANDS.rssSourceExport:
          return (await rssSources.exportRssSources()) as T;
        case COMMANDS.rssArticles:
          return (await rssSources.rssArticles(
            argString(args, "sourceUrl"),
            argString(args, "sortName"),
            argString(args, "sortUrl"),
            argNumber(args, "page") || 1,
          )) as T;
        case COMMANDS.rssContent:
          return (await rssSources.rssContent(
            argString(args, "sourceUrl"),
            args.article as RssArticle,
            (args.ruleContent as string | null | undefined) ?? null,
          )) as T;
        case COMMANDS.webviewLogin:
        case COMMANDS.webviewFetch:
          throw new Error("Web 运行时不支持 Android WebView 登录/抓取");
        default:
          throw new Error(`Web 端未知命令：${command}`);
      }
    },
  };
}
