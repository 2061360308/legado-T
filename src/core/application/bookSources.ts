import { invoke } from "./backend";
import { COMMANDS } from "./commands";
import type {
  BookReview,
  BookSource,
  ExploreCategory,
  OnlineBook,
  OnlineChapter,
  OnlineImage,
  OnlineImagePage,
  OnlineResource,
  OnlineSearchResult,
  PayActionResult,
  SourceCheckResult,
  WebViewLoginResult,
} from "@/core/domain";

export function bookSourceList(): Promise<BookSource[]> {
  return invoke<BookSource[]>(COMMANDS.bookSourceList);
}

export function bookSourceGet(bookSourceUrl: string): Promise<BookSource | null> {
  return invoke<BookSource | null>(COMMANDS.bookSourceGet, { bookSourceUrl });
}

export function bookSourceSave(source: BookSource): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceSave, { source });
}

export function bookSourceDelete(bookSourceUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceDelete, { bookSourceUrl });
}

export function bookSourceSetEnabled(
  bookSourceUrl: string,
  enabled: boolean,
): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceSetEnabled, { bookSourceUrl, enabled });
}

export function bookSourceSetExploreEnabled(
  bookSourceUrl: string,
  enabled: boolean,
): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceSetExploreEnabled, {
    bookSourceUrl,
    enabled,
  });
}

export function bookSourceImport(json: string): Promise<number> {
  return invoke<number>(COMMANDS.bookSourceImport, { json });
}

export function bookSourceExport(): Promise<string> {
  return invoke<string>(COMMANDS.bookSourceExport);
}

export function bookSourceGroups(): Promise<string[]> {
  return invoke<string[]>(COMMANDS.bookSourceGroups);
}

export function bookSourceSearchHistory(limit = 20): Promise<string[]> {
  return invoke<string[]>(COMMANDS.bookSourceSearchHistory, { limit });
}

export function bookSourceSetVariable(
  bookSourceUrl: string,
  key: string,
  value: string,
): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceSetVariable, {
    bookSourceUrl,
    key,
    value,
  });
}

export function bookSourceGetVariables(
  bookSourceUrl: string,
): Promise<Record<string, string>> {
  return invoke<Record<string, string>>(COMMANDS.bookSourceGetVariables, {
    bookSourceUrl,
  });
}

export function bookSourceSearch(
  keyword: string,
  sourceUrls?: string[],
  page = 1,
): Promise<OnlineSearchResult[]> {
  return invoke<OnlineSearchResult[]>(COMMANDS.bookSourceSearch, {
    keyword,
    sourceUrls,
    page,
  });
}

export function bookSourceInfo(
  bookSourceUrl: string,
  bookUrl: string,
): Promise<OnlineBook> {
  return invoke<OnlineBook>(COMMANDS.bookSourceInfo, {
    bookSourceUrl,
    bookUrl,
  });
}

export function bookSourceToc(
  bookSourceUrl: string,
  book: OnlineBook,
): Promise<OnlineChapter[]> {
  return invoke<OnlineChapter[]>(COMMANDS.bookSourceToc, {
    bookSourceUrl,
    book,
  });
}

export function bookSourceContent(
  bookSourceUrl: string,
  book: OnlineBook,
  chapter: OnlineChapter,
  nextChapterUrl?: string | null,
): Promise<string> {
  return invoke<string>(COMMANDS.bookSourceContent, {
    bookSourceUrl,
    book,
    chapter,
    nextChapterUrl: nextChapterUrl ?? null,
  });
}

export function bookSourceAddToShelf(
  bookSourceUrl: string,
  bookUrl: string,
  group = 0,
): Promise<OnlineBook> {
  return invoke<OnlineBook>(COMMANDS.bookSourceAddToShelf, {
    bookSourceUrl,
    bookUrl,
    group,
  });
}

export function bookSourceRefresh(bookUrl: string): Promise<OnlineBook> {
  return invoke<OnlineBook>(COMMANDS.bookSourceRefresh, { bookUrl });
}

export function bookSourceReviews(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<BookReview[]> {
  return invoke<BookReview[]>(COMMANDS.bookSourceReviews, {
    bookUrl,
    chapterIndex: chapterIndex ?? null,
  });
}

export function bookSourceDownloadUrls(bookUrl: string): Promise<string[]> {
  return invoke<string[]>(COMMANDS.bookSourceDownloadUrls, { bookUrl });
}

export function bookSourcePay(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<PayActionResult> {
  return invoke<PayActionResult>(COMMANDS.bookSourcePay, {
    bookUrl,
    chapterIndex: chapterIndex ?? null,
  });
}

export function bookSourceSaveCookies(
  sourceUrl: string,
  cookies: string[],
  cookieUrl?: string | null,
): Promise<number> {
  return invoke<number>(COMMANDS.bookSourceSaveCookies, {
    sourceUrl,
    cookies,
    cookieUrl: cookieUrl ?? null,
  });
}

export function bookSourceClearCookies(sourceUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceClearCookies, { sourceUrl });
}

export function bookSourceDeleteVariable(
  bookSourceUrl: string,
  key: string,
): Promise<void> {
  return invoke<void>(COMMANDS.bookSourceDeleteVariable, {
    bookSourceUrl,
    key,
  });
}

export function bookSourceExploreCategories(
  bookSourceUrl: string,
): Promise<ExploreCategory[]> {
  return invoke<ExploreCategory[]>(COMMANDS.bookSourceExploreCategories, {
    bookSourceUrl,
  });
}

export function bookSourceExplore(
  bookSourceUrl: string,
  url: string,
  page = 1,
): Promise<OnlineSearchResult[]> {
  return invoke<OnlineSearchResult[]>(COMMANDS.bookSourceExplore, {
    bookSourceUrl,
    url,
    page,
  });
}

export function bookSourceCheck(
  sourceUrls?: string[],
  keyword = "斗破",
): Promise<SourceCheckResult[]> {
  return invoke<SourceCheckResult[]>(COMMANDS.bookSourceCheck, {
    sourceUrls,
    keyword,
  });
}

export function bookSourceResource(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<OnlineResource> {
  return invoke<OnlineResource>(COMMANDS.bookSourceResource, {
    bookUrl,
    chapterIndex: chapterIndex ?? null,
  });
}

export function bookSourceImage(
  bookUrl: string,
  chapterIndex?: number | null,
  url?: string | null,
): Promise<OnlineImage> {
  return invoke<OnlineImage>(COMMANDS.bookSourceImage, {
    bookUrl,
    chapterIndex: chapterIndex ?? null,
    url: url ?? null,
  });
}

export function bookSourceImages(
  bookUrl: string,
  chapterIndex?: number | null,
): Promise<OnlineImagePage> {
  return invoke<OnlineImagePage>(COMMANDS.bookSourceImages, {
    bookUrl,
    chapterIndex: chapterIndex ?? null,
  });
}

export function webviewLogin(
  url: string,
  waitMs = 120_000,
): Promise<WebViewLoginResult> {
  return invoke<WebViewLoginResult>(COMMANDS.webviewLogin, { url, waitMs });
}
