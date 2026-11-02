import Dexie, { type Table } from "dexie";
import type {
  Book,
  BookChapter,
  BookGroup,
  Bookmark,
  BookSource,
  DictRule,
  HttpTts,
  ReadRecord,
  ReaderBlock,
  ReplaceRule,
  RssArticle,
  RssReadRecord,
  RssSource,
  RssStar,
  TxtTocRule,
} from "@/core/domain";

export interface KvRow {
  key: string;
  value: string;
}

export interface ChapterRow extends BookChapter {
  key: string;
  bookUrl: string;
}

export interface ChapterContentRow {
  key: string;
  bookUrl: string;
  index: number;
  content: string;
}

export interface ChapterBlocksRow {
  key: string;
  bookUrl: string;
  index: number;
  blocks: ReaderBlock[];
}

export interface SourceVariableRow {
  bookSourceUrl: string;
  key: string;
  value: string;
}

export interface FileRow {
  path: string;
  bookUrl?: string | null;
  kind: "book" | "cover" | "font" | "image" | "backup" | "dir" | "other";
  name: string;
  size: number;
  mime: string;
  blob: Blob;
  storage?: "idb" | "opfs";
  updated: number;
}

export class ReaderDatabase extends Dexie {
  books!: Table<Book, string>;
  chapters!: Table<ChapterRow, string>;
  chapterContents!: Table<ChapterContentRow, string>;
  chapterBlocks!: Table<ChapterBlocksRow, string>;
  groups!: Table<BookGroup, number>;
  kv!: Table<KvRow, string>;
  bookmarks!: Table<Bookmark, number>;
  readRecords!: Table<ReadRecord, [string, string]>;
  txtTocRules!: Table<TxtTocRule, number>;
  replaceRules!: Table<ReplaceRule, number>;
  dictRules!: Table<DictRule, string>;
  httpTts!: Table<HttpTts, number>;
  bookSources!: Table<BookSource, string>;
  sourceVariables!: Table<SourceVariableRow, [string, string]>;
  rssSources!: Table<RssSource, string>;
  rssArticles!: Table<RssArticle, [string, string]>;
  rssStars!: Table<RssStar, [string, string]>;
  rssReadRecords!: Table<RssReadRecord, string>;
  files!: Table<FileRow, string>;

  constructor() {
    super("local-reader");
    this.version(1).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
    });
    this.version(2).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
      files: "path, bookUrl, kind, size",
    });
    this.version(3).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      chapterBlocks: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
      files: "path, bookUrl, kind, size",
    });
    this.version(4).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      chapterBlocks: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
      files: "path, bookUrl, kind, size",
      bookSources: "bookSourceUrl, bookSourceName, bookSourceGroup, customOrder",
      sourceVariables: "[bookSourceUrl+key], bookSourceUrl",
    });
    this.version(5).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      chapterBlocks: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
      files: "path, bookUrl, kind, size",
      bookSources: "bookSourceUrl, bookSourceName, bookSourceGroup, customOrder",
      sourceVariables: "[bookSourceUrl+key], bookSourceUrl",
      rssSources: "sourceUrl, sourceName, sourceGroup, customOrder",
    });
    this.version(6).stores({
      books: "bookUrl, order, durChapterTime, syncTime, group",
      chapters: "key, bookUrl, [bookUrl+index]",
      chapterContents: "key, bookUrl",
      chapterBlocks: "key, bookUrl",
      groups: "groupId, order",
      kv: "key",
      bookmarks: "time, bookUrl, bookName",
      readRecords: "[deviceId+bookName], deviceId, lastRead",
      txtTocRules: "id, serialNumber",
      replaceRules: "id, sortOrder",
      dictRules: "name, sortNumber",
      httpTts: "id, name",
      files: "path, bookUrl, kind, size",
      bookSources: "bookSourceUrl, bookSourceName, bookSourceGroup, customOrder",
      sourceVariables: "[bookSourceUrl+key], bookSourceUrl",
      rssSources: "sourceUrl, sourceName, sourceGroup, customOrder",
      rssArticles: "[origin+link], origin, sort, order",
      rssStars: "[origin+link], origin, group, starTime",
      rssReadRecords: "record, readTime",
    });
  }
}

export const db = new ReaderDatabase();

export function chapterKey(bookUrl: string, index: number): string {
  return `${bookUrl}\u0000${index}`;
}

export function nowMs(): number {
  return Date.now();
}

export function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}
