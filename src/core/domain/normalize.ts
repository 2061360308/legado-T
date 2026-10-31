import type { Book, HttpTts, ReplaceRule } from "./models";

export function normalizeBook(book: Book): Book {
  return {
    ...book,
    origin: book.origin ?? "local",
    originName: book.originName ?? "",
    type: book.type ?? 0,
    group: book.group ?? 0,
    tocUrl: book.tocUrl ?? "",
    durChapterIndex: book.durChapterIndex ?? 0,
    durChapterPos: book.durChapterPos ?? 0,
    durChapterTime: book.durChapterTime ?? 0,
    totalChapterNum: book.totalChapterNum ?? 0,
    canUpdate: book.canUpdate ?? false,
    order: book.order ?? 0,
    syncTime: book.syncTime ?? 0,
  };
}

export function normalizeReplaceRule(rule: ReplaceRule): ReplaceRule {
  return {
    ...rule,
    group: rule.group ?? "",
    replacement: rule.replacement ?? "",
    scope: rule.scope ?? "",
    excludeScope: rule.excludeScope ?? "",
    scopeTitle: rule.scopeTitle ?? false,
    scopeContent: rule.scopeContent ?? true,
    isEnabled: rule.isEnabled ?? true,
    isRegex: rule.isRegex ?? true,
    timeoutMillisecond: rule.timeoutMillisecond ?? 0,
    sortOrder: rule.sortOrder ?? 0,
  };
}

export function normalizeHttpTts(engine: HttpTts): HttpTts {
  return {
    ...engine,
    contentType: engine.contentType ?? "",
    concurrentRate: engine.concurrentRate ?? "",
    loginUrl: engine.loginUrl ?? "",
    loginUi: engine.loginUi ?? "",
    header: engine.header ?? "",
    jsLib: engine.jsLib ?? "",
    enabledCookieJar: engine.enabledCookieJar ?? false,
    loginCheckJs: engine.loginCheckJs ?? "",
    lastUpdateTime: engine.lastUpdateTime || Date.now(),
  };
}

export function parseRuleArray(json: string): unknown[] {
  const value = JSON.parse(json) as unknown;
  if (Array.isArray(value)) return value;
  if (value && typeof value === "object") return [value];
  throw new Error("规则 JSON 必须是数组或对象");
}
