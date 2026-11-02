import type {
  RssArticle,
  RssReadRecord,
  RssSource,
  RssStar,
} from "@/core/domain";
import { db } from "./database";
import {
  evalEngine,
  rssArticles as rssArticlesWasm,
  rssContent as rssContentWasm,
} from "./wasm/engine/engineWasm";

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

export async function saveRssSource(
  source: RssSource,
  oldSourceUrl?: string | null,
): Promise<void> {
  const url = source.sourceUrl?.trim();
  if (!url) throw new Error("RSS 源 URL 不能为空");
  const oldUrl = oldSourceUrl?.trim();
  if (oldUrl && oldUrl !== url) {
    const stars = await db.rssStars.where("origin").equals(oldUrl).toArray();
    for (const star of stars) {
      await db.rssStars.delete([star.origin, star.link]);
      await db.rssStars.put({ ...star, origin: url });
    }
    const articles = await db.rssArticles.where("origin").equals(oldUrl).toArray();
    for (const article of articles) {
      await db.rssArticles.delete([article.origin, article.link]);
      await db.rssArticles.put({ ...article, origin: url });
    }
    const variables = await db.sourceVariables
      .where("bookSourceUrl")
      .equals(oldUrl)
      .toArray();
    for (const row of variables) {
      await db.sourceVariables.delete([row.bookSourceUrl, row.key]);
      await db.sourceVariables.put({ ...row, bookSourceUrl: url });
    }
    await db.rssSources.delete(oldUrl);
  }
  await db.rssSources.put(normalize({ ...source, sourceUrl: url }));
}

export async function deleteRssSource(sourceUrl: string): Promise<void> {
  const articles = await db.rssArticles.where("origin").equals(sourceUrl).toArray();
  await db.rssArticles.bulkDelete(
    articles.map((article) => [article.origin, article.link] as [string, string]),
  );
  const variables = await db.sourceVariables
    .where("bookSourceUrl")
    .equals(sourceUrl)
    .toArray();
  await db.sourceVariables.bulkDelete(
    variables.map((row) => [row.bookSourceUrl, row.key] as [string, string]),
  );
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

function absoluteUrl(base: string, url: string): string {
  if (url.startsWith("{{")) return url;
  try {
    return new URL(url, base).toString();
  } catch {
    return url;
  }
}

function parseSortUrls(text: string, base: string): { name: string; url: string }[] {
  const result: { name: string; url: string }[] = [];
  for (const line of text.split(/(?:&&|\n)+/)) {
    const value = line.trim();
    if (!value) continue;
    const separator = value.indexOf("::");
    const name = separator >= 0 ? value.slice(0, separator).trim() : "";
    const url = separator >= 0 ? value.slice(separator + 2).trim() : value;
    if (!url) continue;
    result.push({ name, url: absoluteUrl(base, url) });
  }
  return result.length ? result : [{ name: "", url: base }];
}

export async function rssSortUrls(sourceUrl: string): Promise<{ name: string; url: string }[]> {
  const source = await db.rssSources.get(sourceUrl);
  if (!source) throw new Error("RSS 源不存在");
  const raw = String(source.sortUrl || "").trim();
  const lower = raw.toLowerCase();
  if (!lower.startsWith("@js:") && !lower.startsWith("<js>")) {
    return parseSortUrls(raw, source.sourceUrl);
  }
  const script = lower.startsWith("@js:")
    ? raw.slice(4).trim()
    : raw.slice(4, raw.lastIndexOf("</") >= 0 ? raw.lastIndexOf("</") : raw.length).trim();
  const text = await evalEngine({
    script,
    baseUrl: source.sourceUrl,
    sourceJson: JSON.stringify(normalize(source)),
  });
  return parseSortUrls(text, source.sourceUrl);
}

export async function rssArticles(
  sourceUrl: string,
  sortName = "",
  sortUrl = "",
  page = 1,
): Promise<RssArticlePage> {
  const source = await db.rssSources.get(sourceUrl);
  if (!source) throw new Error("RSS 源不存在");
  const pageNumber = Math.max(page, 1);
  const value = (await rssArticlesWasm(
    JSON.stringify(normalize(source)),
    sortName,
    sortUrl.trim() || source.sourceUrl,
    pageNumber,
  )) as { articles?: RssArticle[]; next?: string | null };
  const orderBase = Date.now() - (pageNumber - 1) * 10_000;
  const fetched = value.articles ?? [];
  for (const [index, article] of fetched.entries()) {
    const key: [string, string] = [article.origin || sourceUrl, article.link];
    const existing = await db.rssArticles.get(key);
    await db.rssArticles.put({
      ...article,
      origin: article.origin || sourceUrl,
      sort: article.sort || sortName,
      group: existing?.group ?? article.group ?? "默认分组",
      read: existing?.read ?? article.read ?? false,
      order: orderBase - index,
    });
  }
  return {
    articles: await rssArticleList(sourceUrl, sortName),
    next: value.next ?? null,
  };
}

export async function rssArticleList(
  sourceUrl: string,
  sortName = "",
): Promise<RssArticle[]> {
  const articles = await db.rssArticles
    .where("origin")
    .equals(sourceUrl)
    .filter((article) => (article.sort || "") === sortName)
    .toArray();
  const records = await db.rssReadRecords.toArray();
  const readLinks = new Set(records.map((record) => record.record));
  return articles
    .map((article) => ({
      ...article,
      read: article.read === true || readLinks.has(article.link),
    }))
    .sort((a, b) => (b.order ?? 0) - (a.order ?? 0));
}

export async function rssArticleClear(
  sourceUrl?: string | null,
  sortName?: string | null,
): Promise<void> {
  if (!sourceUrl) {
    await db.rssArticles.clear();
    return;
  }
  const articles = await db.rssArticles.where("origin").equals(sourceUrl).toArray();
  const keys = articles
    .filter((article) => !sortName || (article.sort || "") === sortName)
    .map((article) => [article.origin, article.link] as [string, string]);
  await db.rssArticles.bulkDelete(keys);
}

export async function rssStarList(group?: string | null): Promise<RssStar[]> {
  const stars = await db.rssStars.toArray();
  return stars
    .filter((star) => !group || star.group === group)
    .sort((a, b) => b.starTime - a.starTime);
}

export async function rssStarGroups(): Promise<string[]> {
  const stars = await db.rssStars.toArray();
  return [...new Set(stars.map((star) => star.group))].sort((a, b) =>
    a.localeCompare(b, "zh-CN"),
  );
}

export async function rssStarGet(
  origin: string,
  link: string,
): Promise<RssStar | null> {
  return (await db.rssStars.get([origin, link])) ?? null;
}

export async function rssStarSave(
  article: RssArticle,
  group?: string | null,
): Promise<void> {
  if (!article.origin || !article.link) {
    throw new Error("收藏文章缺少 origin 或 link");
  }
  await db.rssStars.put({
    origin: article.origin,
    link: article.link,
    sort: article.sort ?? "",
    title: article.title ?? "",
    starTime: Date.now(),
    pubDate: article.pubDate ?? null,
    description: article.description ?? null,
    content: article.content ?? null,
    image: article.image ?? null,
    group: group?.trim() || article.group || "默认分组",
    variable: article.variable ?? {},
  });
}

export async function rssStarUpdate(
  origin: string,
  link: string,
  title?: string | null,
  group?: string | null,
): Promise<void> {
  const star = await db.rssStars.get([origin, link]);
  if (!star) return;
  await db.rssStars.put({
    ...star,
    title: title?.trim() || star.title,
    group: group?.trim() || star.group,
  });
}

export async function rssStarDelete(origin: string, link: string): Promise<void> {
  await db.rssStars.delete([origin, link]);
}

export async function rssStarDeleteGroup(group: string): Promise<void> {
  const stars = await db.rssStars.where("group").equals(group).toArray();
  await db.rssStars.bulkDelete(
    stars.map((star) => [star.origin, star.link] as [string, string]),
  );
}

export async function rssStarClear(): Promise<void> {
  await db.rssStars.clear();
}

export async function rssReadRecordAdd(
  record: string,
  title?: string | null,
): Promise<void> {
  if (!record.trim()) return;
  await db.rssReadRecords.put({
    record,
    title: title ?? null,
    readTime: Date.now(),
    read: true,
  });
}

export async function rssReadRecordList(): Promise<RssReadRecord[]> {
  return (await db.rssReadRecords.toArray()).sort(
    (a, b) => (b.readTime ?? 0) - (a.readTime ?? 0),
  );
}

export async function rssReadRecordClear(): Promise<void> {
  await db.rssReadRecords.clear();
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
