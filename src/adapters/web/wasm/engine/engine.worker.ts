/// <reference lib="webworker" />
import * as core from "./engineWasmCore";

interface RequestMessage {
  id: number;
  op: string;
  args: unknown[];
}

interface ResponseMessage {
  id: number;
  result?: unknown;
  error?: string;
}

async function run(op: string, args: unknown[]): Promise<unknown> {
  switch (op) {
    case "eval":
      return core.evalEngine(args[0] as Parameters<typeof core.evalEngine>[0]);
    case "search":
      return core.searchSource(args[0] as string, args[1] as string, args[2] as number);
    case "explore":
      return core.exploreSource(args[0] as string, args[1] as string, args[2] as number);
    case "info":
      return core.sourceInfo(args[0] as string, args[1] as string);
    case "toc":
      return core.sourceToc(args[0] as string, args[1] as string);
    case "content":
      return core.sourceContent(
        args[0] as string,
        args[1] as string,
        args[2] as string,
        (args[3] as string | null | undefined) ?? null,
      );
    case "image":
      return core.sourceImage(
        args[0] as string,
        args[1] as string,
        args[2] as string,
        (args[3] as string | null | undefined) ?? null,
      );
    case "rssArticles":
      return core.rssArticles(
        args[0] as string,
        args[1] as string,
        args[2] as string,
        args[3] as number,
      );
    case "rssContent":
      return core.rssContent(args[0] as string, args[1] as string, args[2] as string);
    case "reviews":
      return core.sourceReviews(args[0] as string, args[1] as string, args[2] as string);
    case "downloadUrls":
      return core.sourceDownloadUrls(args[0] as string, args[1] as string);
    case "pay":
      return core.sourcePay(args[0] as string, args[1] as string, args[2] as string);
    case "resource":
      return core.sourceResource(
        args[0] as string,
        args[1] as string,
        args[2] as string,
        (args[3] as string | null | undefined) ?? null,
      );
    case "setVariables":
      return core.setSourceVariables(
        args[0] as string,
        args[1] as Record<string, string>,
      );
    case "setCookie":
      return core.setEngineCookie(args[0] as string, args[1] as string);
    case "removeCookie":
      return core.removeEngineCookie(args[0] as string);
    case "getCookie":
      return core.getEngineCookie(args[0] as string);
    case "warmup":
      return core.warmupEngine();
    default:
      throw new Error(`未知引擎操作：${op}`);
  }
}

self.onmessage = async (event: MessageEvent<RequestMessage>) => {
  const { id, op, args } = event.data;
  try {
    const result = await run(op, args);
    const response: ResponseMessage = { id, result };
    self.postMessage(response);
  } catch (error) {
    const response: ResponseMessage = {
      id,
      error: error instanceof Error ? error.message : String(error),
    };
    self.postMessage(response);
  }
};
