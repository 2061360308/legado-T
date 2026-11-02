import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  bookSourceList,
  bookSourceSearch,
  bookSourceSearchHistory,
  bookSourceSearchHistoryClear,
  bookSourceSearchHistoryDelete,
  bookSourceSearchHistorySave,
  bookSourceSearchCancel,
  bookSourceSearchStart,
} from "@/core/application/bookSources";
import { bookList } from "@/core/application/books";
import type { Book, BookSource, OnlineSearchResult } from "@/core/domain";

export type SearchScopeType = "all" | "groups" | "source";

export interface SearchScope {
  type: SearchScopeType;
  groups: string[];
  sourceUrl: string;
}

export type SearchScopeValue = SearchScope;

const SCOPE_KEY = "local-reader:book-search-scope";
const PRECISION_KEY = "local-reader:book-search-precision";
const SEARCH_CONCURRENCY = 9;

function loadScope(): SearchScope {
  try {
    const raw = localStorage.getItem(SCOPE_KEY);
    if (!raw) return { type: "all", groups: [], sourceUrl: "" };
    const parsed = JSON.parse(raw) as Partial<SearchScope>;
    if (parsed.type === "groups") {
      return {
        type: "groups",
        groups: Array.isArray(parsed.groups)
          ? parsed.groups.filter((item): item is string => typeof item === "string")
          : [],
        sourceUrl: "",
      };
    }
    if (parsed.type === "source") {
      return { type: "source", groups: [], sourceUrl: String(parsed.sourceUrl || "") };
    }
    if (parsed.type === "all") return { type: "all", groups: [], sourceUrl: "" };
  } catch {
    // ignore invalid saved scope
  }
  return { type: "all", groups: [], sourceUrl: "" };
}

function splitGroups(value?: string | null): string[] {
  return String(value || "")
    .split(/[,，;；]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function resultKey(item: OnlineSearchResult): string {
  const name = String(item.name || "").trim();
  const author = String(item.author || "").trim();
  if (name || author) return `${name}\u0000${author}`;
  return String(item.bookUrl || item.bookSourceUrl || "");
}

export const useBookSearchStore = defineStore("bookSearch", () => {
  const keyword = ref("");
  const scope = ref<SearchScope>(loadScope());
  const precision = ref(localStorage.getItem(PRECISION_KEY) === "1");
  const sources = ref<BookSource[]>([]);
  const books = ref<Book[]>([]);
  const history = ref<string[]>([]);
  const results = ref<OnlineSearchResult[]>([]);
  const running = ref(false);
  const paused = ref(false);
  const manualStopped = ref(false);
  const completed = ref(0);
  const total = ref(0);
  const failures = ref(0);
  const page = ref(1);
  const hasMore = ref(false);
  const searchId = ref(0);
  const emptyDialogVisible = ref(false);
  const inputHelpVisible = ref(false);
  let activeId = 0;

  const scopeSources = computed(() => {
    const sorted = [...sources.value].sort(
      (a, b) =>
        (a.customOrder ?? 0) - (b.customOrder ?? 0) ||
        String(a.bookSourceName || "").localeCompare(
          String(b.bookSourceName || ""),
          "zh-CN",
        ),
    );
    if (scope.value.type === "source") {
      return sorted.filter(
        (source) =>
          source.bookSourceUrl === scope.value.sourceUrl &&
          Boolean(String(source.searchUrl || "").trim()),
      );
    }
    const enabled = sorted.filter(
      (source) =>
        source.enabled !== false && Boolean(String(source.searchUrl || "").trim()),
    );
    if (scope.value.type === "groups") {
      return enabled.filter((source) =>
        splitGroups(source.bookSourceGroup).some((group) =>
          scope.value.groups.includes(group),
        ),
      );
    }
    return enabled;
  });

  const scopeLabel = computed(() => {
    if (scope.value.type === "groups") {
      return scope.value.groups.length
        ? `分组：${scope.value.groups.join("、")}`
        : "全部分组";
    }
    if (scope.value.type === "source") {
      const source = sources.value.find(
        (item) => item.bookSourceUrl === scope.value.sourceUrl,
      );
      return `书源：${source?.bookSourceName || scope.value.sourceUrl}`;
    }
    return "全部书源";
  });

  const groups = computed(() => {
    const values = new Set<string>();
    for (const source of sources.value) {
      if (source.enabled === false) continue;
      for (const group of splitGroups(source.bookSourceGroup)) values.add(group);
    }
    return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
  });

  const filteredHistory = computed(() => {
    const key = keyword.value.trim().toLowerCase();
    if (!key) return history.value;
    return history.value.filter((item) => item.toLowerCase().includes(key));
  });

  const bookshelfSuggestions = computed(() => {
    const key = keyword.value.trim().toLowerCase();
    if (!key) return [];
    return books.value
      .filter(
        (book) =>
          book.name.toLowerCase().includes(key) ||
          String(book.author || "").toLowerCase().includes(key),
      )
      .slice(0, 30);
  });

  const bookshelfKeys = computed(() => {
    const keys = new Set<string>();
    for (const book of books.value) {
      keys.add(`url:${book.bookUrl}`);
      keys.add(`name:${book.name}`);
      if (book.author) keys.add(`name:${book.name}\u0000${book.author}`);
    }
    return keys;
  });

  function isInBookshelf(item: OnlineSearchResult): boolean {
    const keys = bookshelfKeys.value;
    if (item.bookUrl && keys.has(`url:${item.bookUrl}`)) return true;
    if (item.name && item.author && keys.has(`name:${item.name}\u0000${item.author}`)) {
      return true;
    }
    return Boolean(item.name && keys.has(`name:${item.name}`));
  }

  async function loadSources() {
    try {
      sources.value = await bookSourceList();
    } catch {
      sources.value = [];
    }
  }

  async function loadBooks() {
    try {
      books.value = await bookList();
    } catch {
      books.value = [];
    }
  }

  async function loadHistory() {
    try {
      history.value = await bookSourceSearchHistory(100);
    } catch {
      history.value = [];
    }
  }

  function setScope(next: SearchScope) {
    scope.value = next;
    localStorage.setItem(SCOPE_KEY, JSON.stringify(next));
    cancel();
  }

  function setPrecision(value: boolean) {
    precision.value = value;
    localStorage.setItem(PRECISION_KEY, value ? "1" : "0");
    if (keyword.value.trim()) void startSearch(keyword.value);
  }

  function pause() {
    paused.value = true;
  }

  function resume() {
    paused.value = false;
  }

  function cancel() {
    activeId += 1;
    searchId.value = activeId;
    running.value = false;
    paused.value = false;
    manualStopped.value = false;
    emptyDialogVisible.value = false;
    void bookSourceSearchCancel().catch(() => undefined);
  }

  function stop() {
    cancel();
    manualStopped.value = true;
  }

  function reset() {
    cancel();
    keyword.value = "";
    results.value = [];
    completed.value = 0;
    total.value = 0;
    failures.value = 0;
    page.value = 1;
    hasMore.value = false;
    emptyDialogVisible.value = false;
    inputHelpVisible.value = false;
  }

  async function waitUntilResumed(id: number) {
    while (paused.value && activeId === id) {
      await new Promise((resolve) => window.setTimeout(resolve, 40));
    }
  }

  async function runPool(
    id: number,
    items: BookSource[],
    pageNumber: number,
  ) {
    let cursor = 0;
    let pageHadResults = false;
    total.value = items.length;
    completed.value = 0;
    const runners = Array.from(
      { length: Math.min(SEARCH_CONCURRENCY, items.length) },
      async () => {
        while (activeId === id) {
          const index = cursor++;
          if (index >= items.length) return;
          await waitUntilResumed(id);
          if (activeId !== id) return;
          const source = items[index];
          try {
            const found = await bookSourceSearch(
              keyword.value,
              [source.bookSourceUrl],
              pageNumber,
              false,
            );
            if (activeId !== id) return;
            const errors = found.filter((item) => item.error);
            failures.value += errors.length;
            const booksFound = found.filter((item) => !item.error);
            if (booksFound.length) pageHadResults = true;
            mergeResults(booksFound);
          } catch {
            if (activeId === id) failures.value += 1;
          } finally {
            if (activeId === id) completed.value += 1;
          }
        }
      },
    );
    await Promise.all(runners);
    if (activeId !== id) return;
    running.value = false;
    hasMore.value = pageHadResults;
    page.value = pageNumber;
    if (!results.value.length && scope.value.type !== "all") {
      emptyDialogVisible.value = true;
    }
  }

  async function startSearch(query: string) {
    const key = query.trim();
    if (!key) return;
    cancel();
    await bookSourceSearchStart().catch(() => undefined);
    keyword.value = key;
    const id = ++activeId;
    searchId.value = id;
    running.value = true;
    paused.value = false;
    manualStopped.value = false;
    results.value = [];
    completed.value = 0;
    total.value = 0;
    failures.value = 0;
    page.value = 1;
    hasMore.value = false;
    emptyDialogVisible.value = false;
    await bookSourceSearchHistorySave(key).catch(() => undefined);
    await loadHistory();
    const targets = scopeSources.value;
    if (!targets.length) {
      running.value = false;
      return;
    }
    await runPool(id, targets, 1);
  }

  async function nextPage() {
    if (running.value || paused.value || manualStopped.value || !hasMore.value) return;
    const targets = scopeSources.value;
    if (!targets.length) return;
    await bookSourceSearchStart().catch(() => undefined);
    const id = ++activeId;
    searchId.value = id;
    running.value = true;
    failures.value = 0;
    completed.value = 0;
    await runPool(id, targets, page.value + 1);
  }

  function mergeResults(booksFound: OnlineSearchResult[]) {
    const byKey = new Map<string, OnlineSearchResult>();
    for (const item of results.value) byKey.set(resultKey(item), item);
    for (const book of booksFound) {
      const key = resultKey(book);
      const existing = byKey.get(key);
      if (existing) {
        existing.originCount = (existing.originCount ?? 1) + 1;
        const origin = String(book.bookSourceName || "").trim();
        if (origin && !(existing.origins ?? []).includes(origin)) {
          existing.origins = [...(existing.origins ?? []), origin];
        }
        continue;
      }
      book.originCount = 1;
      book.origins = book.bookSourceName ? [book.bookSourceName] : [];
      byKey.set(key, book);
      results.value.push(book);
    }
    const keywordValue = keyword.value.trim();
    const relevance = (item: OnlineSearchResult) => {
      const name = String(item.name || "");
      const author = String(item.author || "");
      if (name === keywordValue || author === keywordValue) return 0;
      if (name.includes(keywordValue) || author.includes(keywordValue)) return 1;
      return 2;
    };
    results.value = results.value
      .filter(
        (item) =>
          !precision.value ||
          String(item.name || "").includes(keywordValue) ||
          String(item.author || "").includes(keywordValue),
      )
      .sort(
        (a, b) =>
          relevance(a) - relevance(b) ||
          (b.originCount ?? 1) - (a.originCount ?? 1),
      );
  }

  async function deleteHistory(item: string) {
    await bookSourceSearchHistoryDelete(item).catch(() => undefined);
    await loadHistory();
  }

  async function clearHistory() {
    await bookSourceSearchHistoryClear().catch(() => undefined);
    await loadHistory();
  }

  return {
    keyword,
    scope,
    precision,
    sources,
    books,
    history,
    results,
    running,
    paused,
    manualStopped,
    completed,
    total,
    failures,
    page,
    hasMore,
    searchId,
    emptyDialogVisible,
    inputHelpVisible,
    scopeSources,
    scopeLabel,
    groups,
    filteredHistory,
    bookshelfSuggestions,
    isInBookshelf,
    loadSources,
    loadBooks,
    loadHistory,
    setScope,
    setPrecision,
    pause,
    resume,
    cancel,
    stop,
    reset,
    startSearch,
    nextPage,
    deleteHistory,
    clearHistory,
  };
});
