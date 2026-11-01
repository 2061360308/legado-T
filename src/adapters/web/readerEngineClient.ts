import type {
  Book,
  LayoutPage,
  LayoutStyle,
  ReaderBlock,
  ReplaceRule,
} from "@/core/domain";
import type { LayoutOutput } from "./readerEngine";

export interface KotlinTxtChapter {
  title: string;
  start: number;
  end: number;
  isVolume: boolean;
  wordCount: number;
}

export interface KotlinTxtResult {
  chapters: KotlinTxtChapter[];
  intro: string;
  rule: string;
  charset: string;
  totalWordCount: number;
}

export interface WasmSearchResult {
  chapterIndex: number;
  chapterTitle: string;
  pos: number;
  snippet: string;
}

export interface WasmProcessedContent {
  displayTitle: string;
  sameTitleRemoved: boolean;
  contents: string[];
  effectiveRules: ReplaceRule[];
}

export interface WasmChapterOutput {
  displayTitle: string;
  blocks: ReaderBlock[];
  pages: LayoutPage[];
  paragraphs: string[];
  effectiveRules: ReplaceRule[];
  marginFellBack: boolean;
}

export interface WasmUmdBook {
  title: string;
  author: string;
  kind: string;
  coverBase64?: string | null;
  coverExt?: string | null;
  chapters: { title: string; text: string }[];
}

let worker: Worker | null = null;
let sequence = 0;
const pending = new Map<
  number,
  {
    resolve: (value: unknown) => void;
    reject: (reason: Error) => void;
    timer: number;
  }
>();

function getWorker(): Worker {
  if (!worker) {
    worker = new Worker(new URL("./reader.worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (event: MessageEvent) => {
      const data = event.data as {
        id: number;
        result?: unknown;
        error?: string;
      };
      const item = pending.get(data.id);
      if (!item) return;
      pending.delete(data.id);
      window.clearTimeout(item.timer);
      if (data.error) item.reject(new Error(data.error));
      else item.resolve(data.result);
    };
    worker.onerror = (event) => {
      const error = new Error(event.message || "Reader Worker 失败");
      for (const item of pending.values()) {
        window.clearTimeout(item.timer);
        item.reject(error);
      }
      pending.clear();
      worker?.terminate();
      worker = null;
    };
  }
  return worker;
}

function request<T>(payload: Record<string, unknown>): Promise<T> {
  const target = getWorker();
  const id = ++sequence;
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(() => {
      pending.delete(id);
      worker?.terminate();
      worker = null;
      reject(new Error("Reader Worker 超时"));
    }, 15000);
    pending.set(id, {
      resolve: (value) => resolve(value as T),
      reject,
      timer,
    });
    target.postMessage({ id, ...payload });
  });
}

export function layoutInWorker(
  title: string,
  blocks: ReaderBlock[],
  style: LayoutStyle,
): Promise<LayoutOutput> {
  return request<LayoutOutput>({
    kind: "layout",
    title,
    blocks,
    style,
  });
}

export function searchTextInWorker(
  chapters: { index: number; title: string; content: string }[],
  keyword: string,
  limit: number,
): Promise<WasmSearchResult[]> {
  return request<WasmSearchResult[]>({
    kind: "search",
    chapters,
    keyword,
    limit,
  });
}

export function processContentInWorker(
  payload: Record<string, unknown>,
): Promise<WasmProcessedContent> {
  return request<WasmProcessedContent>({
    kind: "processContent",
    request: payload,
  });
}

export function processChapterInWorker(
  payload: Record<string, unknown>,
): Promise<WasmChapterOutput> {
  return request<WasmChapterOutput>({
    kind: "chapter",
    request: payload,
  });
}

export function parseUmdInWorker(
  bytes: Uint8Array,
  titleFallback: string,
): Promise<WasmUmdBook> {
  return request<WasmUmdBook>({
    kind: "parseUmd",
    bytes,
    titleFallback,
  });
}

export function shelfBooksInWorker(
  books: Book[],
  groupId: number,
  sort: number,
): Promise<Book[]> {
  return request<Book[]>({
    kind: "shelfBooks",
    books,
    groupId,
    sort,
  });
}

export function analyzeTxtInWorker(
  bytes: Uint8Array,
  charset: string,
  rules: unknown[],
  splitLong: boolean,
  ruleExpr = "",
): Promise<KotlinTxtResult> {
  return request<KotlinTxtResult>({
    kind: "analyzeTxt",
    bytes,
    charset,
    rules,
    splitLong,
    ruleExpr,
  });
}