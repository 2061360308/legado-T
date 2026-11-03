export interface EngineEvalInput {
  script: string;
  baseUrl: string;
  result?: string;
  sourceJson?: string | null;
}

export interface EngineOperations {
  eval(input: EngineEvalInput): Promise<string>;
  search(sourceJson: string, keyword: string, page?: number): Promise<unknown[]>;
  explore(sourceJson: string, url: string, page?: number): Promise<unknown[]>;
  info(sourceJson: string, bookJson: string): Promise<unknown>;
  toc(sourceJson: string, bookJson: string): Promise<unknown[]>;
  content(
    sourceJson: string,
    bookJson: string,
    chapterJson: string,
    nextUrl?: string | null,
  ): Promise<string>;
  image(
    sourceJson: string,
    bookJson: string,
    chapterJson: string,
    url?: string | null,
  ): Promise<string>;
  rssArticles(
    sourceJson: string,
    sortName: string,
    sortUrl: string,
    page?: number,
  ): Promise<unknown>;
  rssContent(
    sourceJson: string,
    articleJson: string,
    ruleContent: string,
  ): Promise<string>;
  reviews(
    sourceJson: string,
    bookJson: string,
    chapterJson: string,
  ): Promise<unknown[]>;
  downloadUrls(sourceJson: string, bookJson: string): Promise<unknown[]>;
  pay(
    sourceJson: string,
    bookJson: string,
    chapterJson: string,
  ): Promise<unknown>;
  resource(
    sourceJson: string,
    bookJson: string,
    chapterJson: string,
    nextUrl?: string | null,
  ): Promise<string>;
  setVariables(
    sourceKey: string,
    variables: Record<string, string>,
  ): Promise<void>;
  setCookie(url: string, cookie: string): Promise<void>;
  removeCookie(url: string): Promise<void>;
  getCookie(url: string): Promise<string>;
  warmup(): Promise<void>;
}

export type EngineOperationName = keyof EngineOperations;
export type EngineOperationArgs<K extends EngineOperationName> = Parameters<
  EngineOperations[K]
>;
export type EngineOperationResult<K extends EngineOperationName> = Awaited<
  ReturnType<EngineOperations[K]>
>;

export type EngineRequest = {
  [K in EngineOperationName]: {
    id: number;
    op: K;
    args: EngineOperationArgs<K>;
  };
}[EngineOperationName];

export interface EngineResponseMessage {
  id: number;
  result?: unknown;
  error?: string;
}

export const ENGINE_OPERATION_NAMES = [
  "eval",
  "search",
  "explore",
  "info",
  "toc",
  "content",
  "image",
  "rssArticles",
  "rssContent",
  "reviews",
  "downloadUrls",
  "pay",
  "resource",
  "setVariables",
  "setCookie",
  "removeCookie",
  "getCookie",
  "warmup",
] as const satisfies readonly EngineOperationName[];
