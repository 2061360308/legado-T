import type {
  EngineOperationArgs,
  EngineOperationName,
  EngineOperationResult,
  EngineOperations,
  EngineRequest,
  EngineResponseMessage,
} from "./engineProtocol";

export type { EngineEvalInput } from "./engineProtocol";

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
    worker.onmessage = (event: MessageEvent<EngineResponseMessage>) => {
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

function request<K extends EngineOperationName>(
  op: K,
  args: EngineOperationArgs<K>,
): Promise<EngineOperationResult<K>> {
  const target = getWorker();
  const id = ++sequence;
  return new Promise<EngineOperationResult<K>>((resolve, reject) => {
    const timer = window.setTimeout(() => {
      pending.delete(id);
      worker?.terminate();
      worker = null;
      reject(new Error("引擎 Worker 超时"));
    }, 120_000);
    pending.set(id, {
      resolve: (value) => resolve(value as EngineOperationResult<K>),
      reject,
      timer,
    });
    target.postMessage({ id, op, args } as EngineRequest);
  });
}

export const evalEngine: EngineOperations["eval"] = (input) =>
  request("eval", [input]);

export const searchSource: EngineOperations["search"] = (
  sourceJson,
  keyword,
  page = 1,
) => request("search", [sourceJson, keyword, page]);

export const exploreSource: EngineOperations["explore"] = (
  sourceJson,
  url,
  page = 1,
) => request("explore", [sourceJson, url, page]);

export const exploreCategories: EngineOperations["exploreCategories"] = (
  sourceJson,
  raw,
) => request("exploreCategories", [sourceJson, raw]);

export const sourceInfo: EngineOperations["info"] = (sourceJson, bookJson) =>
  request("info", [sourceJson, bookJson]);

export const sourceToc: EngineOperations["toc"] = (sourceJson, bookJson) =>
  request("toc", [sourceJson, bookJson]);

export const sourceContent: EngineOperations["content"] = (
  sourceJson,
  bookJson,
  chapterJson,
  nextUrl,
) => request("content", [sourceJson, bookJson, chapterJson, nextUrl ?? null]);

export const sourceImage: EngineOperations["image"] = (
  sourceJson,
  bookJson,
  chapterJson,
  url,
) => request("image", [sourceJson, bookJson, chapterJson, url ?? null]);

export const rssArticles: EngineOperations["rssArticles"] = (
  sourceJson,
  sortName,
  sortUrl,
  page = 1,
) => request("rssArticles", [sourceJson, sortName, sortUrl, page]);

export const rssContent: EngineOperations["rssContent"] = (
  sourceJson,
  articleJson,
  ruleContent,
) => request("rssContent", [sourceJson, articleJson, ruleContent]);

export const sourceReviews: EngineOperations["reviews"] = (
  sourceJson,
  bookJson,
  chapterJson,
) => request("reviews", [sourceJson, bookJson, chapterJson]);

export const sourceDownloadUrls: EngineOperations["downloadUrls"] = (
  sourceJson,
  bookJson,
) => request("downloadUrls", [sourceJson, bookJson]);

export const sourcePay: EngineOperations["pay"] = (
  sourceJson,
  bookJson,
  chapterJson,
) => request("pay", [sourceJson, bookJson, chapterJson]);

export const sourceResource: EngineOperations["resource"] = (
  sourceJson,
  bookJson,
  chapterJson,
  nextUrl,
) => request("resource", [sourceJson, bookJson, chapterJson, nextUrl ?? null]);

export const setSourceVariables: EngineOperations["setVariables"] = (
  sourceKey,
  variables,
) => request("setVariables", [sourceKey, variables]);

export const setEngineCookie: EngineOperations["setCookie"] = (url, cookie) =>
  request("setCookie", [url, cookie]);

export const removeEngineCookie: EngineOperations["removeCookie"] = (url) =>
  request("removeCookie", [url]);

export const getEngineCookie: EngineOperations["getCookie"] = (url) =>
  request("getCookie", [url]);

export const warmupEngine: EngineOperations["warmup"] = () =>
  request("warmup", []);
