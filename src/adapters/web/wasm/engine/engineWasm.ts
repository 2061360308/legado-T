import type { EngineEvalInput } from "./engineWasmCore";

export type { EngineEvalInput } from "./engineWasmCore";

interface ResponseMessage {
  id: number;
  result?: unknown;
  error?: string;
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
    worker = new Worker(new URL("./engine.worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (event: MessageEvent<ResponseMessage>) => {
      const data = event.data;
      const item = pending.get(data.id);
      if (!item) return;
      pending.delete(data.id);
      window.clearTimeout(item.timer);
      if (data.error) item.reject(new Error(data.error));
      else item.resolve(data.result);
    };
    worker.onerror = (event) => {
      const error = new Error(event.message || "引擎 Worker 失败");
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

function request<T>(op: string, args: unknown[]): Promise<T> {
  const target = getWorker();
  const id = ++sequence;
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(() => {
      pending.delete(id);
      worker?.terminate();
      worker = null;
      reject(new Error("引擎 Worker 超时"));
    }, 120_000);
    pending.set(id, {
      resolve: (value) => resolve(value as T),
      reject,
      timer,
    });
    target.postMessage({ id, op, args });
  });
}

export function evalEngine(input: EngineEvalInput): Promise<string> {
  return request<string>("eval", [input]);
}

export function searchSource(
  sourceJson: string,
  keyword: string,
  page = 1,
): Promise<unknown[]> {
  return request<unknown[]>("search", [sourceJson, keyword, page]);
}

export function exploreSource(
  sourceJson: string,
  url: string,
  page = 1,
): Promise<unknown[]> {
  return request<unknown[]>("explore", [sourceJson, url, page]);
}

export function sourceInfo(sourceJson: string, bookJson: string): Promise<unknown> {
  return request<unknown>("info", [sourceJson, bookJson]);
}

export function sourceToc(sourceJson: string, bookJson: string): Promise<unknown[]> {
  return request<unknown[]>("toc", [sourceJson, bookJson]);
}

export function sourceContent(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
  nextUrl?: string | null,
): Promise<string> {
  return request<string>("content", [sourceJson, bookJson, chapterJson, nextUrl ?? null]);
}

export function rssArticles(
  sourceJson: string,
  sortName: string,
  sortUrl: string,
  page = 1,
): Promise<unknown> {
  return request<unknown>("rssArticles", [sourceJson, sortName, sortUrl, page]);
}

export function rssContent(
  sourceJson: string,
  articleJson: string,
  ruleContent: string,
): Promise<string> {
  return request<string>("rssContent", [sourceJson, articleJson, ruleContent]);
}

export function sourceReviews(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
): Promise<unknown[]> {
  return request<unknown[]>("reviews", [sourceJson, bookJson, chapterJson]);
}

export function sourceDownloadUrls(
  sourceJson: string,
  bookJson: string,
): Promise<unknown[]> {
  return request<unknown[]>("downloadUrls", [sourceJson, bookJson]);
}

export function sourcePay(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
): Promise<unknown> {
  return request<unknown>("pay", [sourceJson, bookJson, chapterJson]);
}

export function sourceResource(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
  nextUrl?: string | null,
): Promise<string> {
  return request<string>("resource", [sourceJson, bookJson, chapterJson, nextUrl ?? null]);
}

export function setSourceVariables(
  sourceKey: string,
  variables: Record<string, string>,
): Promise<void> {
  return request<void>("setVariables", [sourceKey, variables]);
}

export function setEngineCookie(url: string, cookie: string): Promise<void> {
  return request<void>("setCookie", [url, cookie]);
}

export function removeEngineCookie(url: string): Promise<void> {
  return request<void>("removeCookie", [url]);
}

export function getEngineCookie(url: string): Promise<string> {
  return request<string>("getCookie", [url]);
}

export function warmupEngine(): Promise<void> {
  return request<void>("warmup", []);
}
