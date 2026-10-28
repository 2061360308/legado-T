import { invoke } from "@tauri-apps/api/core";
import {
  COMMANDS,
  type Book,
  type BookChapter,
  type BookGroup,
  type LocalImportEntry,
  type LocalImportResult,
  type TxtTocRule,
} from "./bindings";

export function bookList(): Promise<Book[]> {
  return invoke<Book[]>(COMMANDS.bookList);
}

export function bookGet(bookUrl: string): Promise<Book | null> {
  return invoke<Book | null>(COMMANDS.bookGet, { bookUrl });
}

export function bookUpsert(book: Book): Promise<void> {
  return invoke<void>(COMMANDS.bookUpsert, { book });
}

export function bookDelete(bookUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.bookDelete, { bookUrl });
}

export function chapterList(bookUrl: string): Promise<BookChapter[]> {
  return invoke<BookChapter[]>(COMMANDS.chapterList, { bookUrl });
}

export function groupList(): Promise<BookGroup[]> {
  return invoke<BookGroup[]>(COMMANDS.groupList);
}

export function txtTocRuleList(): Promise<TxtTocRule[]> {
  return invoke<TxtTocRule[]>(COMMANDS.txtTocRuleList);
}

// ---- M2 书架服务 ----

/** 虚拟组 ID：全部/本地/音频/网络未分组/本地未分组/错误；>0 为用户分组位掩码。 */
export const VIRTUAL_GROUP = {
  all: -1,
  local: -2,
  audio: -3,
  networkUngrouped: -4,
  localUngrouped: -5,
  error: -6,
} as const;

export const BOOK_SORT = {
  lastRead: 0,
  updateTime: 1,
  bookNamePinyin: 2,
  manual: 3,
  comprehensive: 4,
  authorPinyin: 5,
} as const;

export const BOOK_SORT_LABELS: Record<number, string> = {
  0: "最近阅读",
  1: "更新时间",
  2: "书名",
  3: "手动",
  4: "综合时间",
  5: "作者",
};

export function bookshelfList(groupId: number, sort: number): Promise<Book[]> {
  return invoke<Book[]>("bookshelf_list", { groupId, sort });
}

export function bookshelfExport(): Promise<string> {
  return invoke<string>("bookshelf_export");
}

export function bookMoveToTop(bookUrl: string): Promise<void> {
  return invoke<void>("book_move_to_top", { bookUrl });
}

export function bookSetGroup(bookUrl: string, group: number): Promise<void> {
  return invoke<void>("book_set_group", { bookUrl, group });
}

// ---- M3 导入与目录 ----

export interface ImportResult {
  book: Book;
  chapterCount: number;
  charset: string;
  rule: string;
  intro: string;
}

export function localImportList(path: string): Promise<LocalImportEntry[]> {
  return invoke<LocalImportEntry[]>(COMMANDS.localImportList, { path });
}

export function localImportScan(path: string): Promise<LocalImportEntry[]> {
  return invoke<LocalImportEntry[]>(COMMANDS.localImportScan, { path });
}

export function localImportImport(
  paths: string[],
  group: number | null,
  nameScript: string | null,
): Promise<LocalImportResult[]> {
  return invoke<LocalImportResult[]>(COMMANDS.localImportImport, {
    paths,
    group,
    nameScript,
  });
}

export function localImportDelete(paths: string[]): Promise<number> {
  return invoke<number>(COMMANDS.localImportDelete, { paths });
}

export function bookCoverSave(bookUrl: string, srcPath: string): Promise<string> {
  return invoke<string>(COMMANDS.bookCoverSave, { bookUrl, srcPath });
}

export function bookCoverRefresh(bookUrl: string): Promise<string | null> {
  return invoke<string | null>(COMMANDS.bookCoverRefresh, { bookUrl });
}

export function bookFileSize(bookUrl: string): Promise<number> {
  return invoke<number>(COMMANDS.bookFileSize, { bookUrl });
}

export function bookClearCache(bookUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.bookClearCache, { bookUrl });
}

export function importTxt(
  path: string,
  name: string | null,
  group: number | null,
  charset: string | null,
  splitLong: boolean | null,
): Promise<ImportResult> {
  return invoke<ImportResult>("import_txt", {
    path,
    name,
    group,
    charset,
    splitLong,
  });
}

export function importBook(
  path: string,
  name: string | null,
  group: number | null,
  charset: string | null,
  splitLong: boolean | null,
): Promise<ImportResult> {
  return invoke<ImportResult>("import_book", {
    path,
    name,
    group,
    charset,
    splitLong,
  });
}

export function reSplitBook(
  bookUrl: string,
  rule: string | null,
  charset: string | null,
  splitLong: boolean | null,
): Promise<number> {
  return invoke<number>("re_split_book", { bookUrl, rule, charset, splitLong });
}

export function readChapter(bookUrl: string, index: number): Promise<string> {
  return invoke<string>("read_chapter", { bookUrl, index });
}