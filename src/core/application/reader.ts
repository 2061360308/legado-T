import { bookGet } from "./books";
import { invoke } from "./backend";
import {
  COMMANDS,
  type BookSearchResult,
  type LayoutStyle,
  type ReaderSnapshot,
  type ReplaceRule,
} from "./bindings";

export async function readerOpen(
  bookUrl: string,
  chapterIndex: number | null,
  chapterPos: number | null,
  style: LayoutStyle,
): Promise<ReaderSnapshot> {
  const book = await bookGet(bookUrl);
  const online =
    book !== null &&
    !book.sourcePath &&
    /^https?:\/\//i.test(String(book.origin || ""));
  return invoke<ReaderSnapshot>(
    online ? COMMANDS.onlineReaderOpen : COMMANDS.readerOpen,
    {
      bookUrl,
      chapterIndex,
      chapterPos,
      style,
    },
  );
}

export function readerSaveProgress(
  bookUrl: string,
  chapterIndex: number,
  chapterPos: number,
  chapterTitle: string,
): Promise<void> {
  return invoke<void>(COMMANDS.readerSaveProgress, {
    bookUrl,
    chapterIndex,
    chapterPos,
    chapterTitle,
  });
}

export function readerSaveChapterContent(
  bookUrl: string,
  chapterIndex: number,
  content: string,
): Promise<void> {
  return invoke<void>(COMMANDS.readerSaveChapterContent, {
    bookUrl,
    chapterIndex,
    content,
  });
}

export function readerInvalidateBook(bookUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.readerInvalidateBook, { bookUrl });
}

export function readerInvalidateAll(): Promise<void> {
  return invoke<void>(COMMANDS.readerInvalidateAll);
}

export function readerSearchBook(
  bookUrl: string,
  keyword: string,
  limit = 200,
): Promise<BookSearchResult[]> {
  return invoke<BookSearchResult[]>(COMMANDS.readerSearchBook, {
    bookUrl,
    keyword,
    limit,
  });
}

export function readerEffectiveRules(
  bookUrl: string,
  chapterIndex: number,
  style: LayoutStyle,
): Promise<ReplaceRule[]> {
  return invoke<ReplaceRule[]>(COMMANDS.readerEffectiveRules, {
    bookUrl,
    chapterIndex,
    style,
  });
}
