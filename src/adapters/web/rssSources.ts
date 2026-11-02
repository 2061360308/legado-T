import type { RssArticle, RssSource } from "@/core/domain";
import { db } from "./database";
import { rssArticles as rssArticlesWasm, rssContent as rssContentWasm } from "./wasm/engine/engineWasm";

export interface RssArticlePage {
  articles: RssArticle[];
  next?: string | null;
}

function normalize(source: RssSource): RssSource {
  return {
    ...source,
    sourceName: source.sourceName?.trim() || source.sourceUrl,
    enabled: source.enabled !== false,
    customOrder: source.customOrder ?? 0,
    lastUpdateTime: source.lastUpdateTime ?? Math.floor(Date.now() / 1000),
  };
}

export async function listRssSources(): Promise<RssSource[]> {
  const sources = await db.rssSources.toArray();
  sources.sort(
    (a, b) =>
      (a.customOrder ?? 0) - (b.customOrder ?? 0) ||
      (a.sourceName ?? "").localeCompare(b.sourceName ?? "", "zh-CN"),
  );
  return sources.map(normalize);
}

export async function saveRssSource(source: RssSource): Promise<void> {
  const url = source.sourceUrl?.trim();
  if (!url) throw new Error("RSS 源 URL 不能为空");
  await db.rssSources.put(normalize({ ...source, sourceUrl: url }));
}

export async function deleteRssSource(sourceUrl: string): Promise<void> {
  await db.rssSources.delete(sourceUrl);
}

export async function importRssSources(json: string): Promise<number> {
  const parsed = JSON.parse(json) as unknown;
  const list = Array.isArray(parsed) ? parsed : [parsed];
  let count = 0;
  for (const item of list) {
    if (!item || typeof item !== "object") continue;
    const source = normalize(item as RssSource);
    if (!source.sourceUrl?.trim()) continue;
    await db.rssSources.put(source);
    count += 1;
  }
  return count;
}

export async function exportRssSources(): Promise<string> {
  return JSON.stringify(await listRssSources(), null, 2);
}

export async function rssArticles(
  sourceUrl: string,
  sortName = "",
  sortUrl = "",
  page = 1,
): Promise<RssArticlePage> {
  const source = await db.rssSources.get(sourceUrl);
  if (!source) throw new Error("RSS 源不存在");
  const value = (await rssArticlesWasm(
    JSON.stringify(normalize(source)),
    sortName,
    sortUrl.trim() || source.sourceUrl,
    Math.max(page, 1),
  )) as { articles?: RssArticle[]; next?: string | null };
  return {
    articles: value.articles ?? [],
    next: value.next ?? null,
  };
}

export async function rssContent(
  sourceUrl: string,
  article: RssArticle,
  ruleContent?: string | null,
): Promise<string> {
  const source = await db.rssSources.get(sourceUrl);
  if (!source) throw new Error("RSS 源不存在");
  return rssContentWasm(
    JSON.stringify(normalize(source)),
    JSON.stringify(article),
    ruleContent ?? source.ruleContent ?? "",
  );
}
