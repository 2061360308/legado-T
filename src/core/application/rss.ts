import { invoke } from "./backend";
import { COMMANDS } from "./commands";
import type {
  RssArticle,
  RssReadRecord,
  RssSource,
  RssStar,
} from "@/core/domain";

export interface RssArticlePage {
  articles: RssArticle[];
  next?: string | null;
}

export interface RssSort {
  name: string;
  url: string;
}

export function rssSourceList(): Promise<RssSource[]> {
  return invoke<RssSource[]>(COMMANDS.rssSourceList);
}

export function rssSourceSave(
  source: RssSource,
  oldSourceUrl?: string | null,
): Promise<void> {
  return invoke<void>(COMMANDS.rssSourceSave, {
    source,
    oldSourceUrl: oldSourceUrl ?? null,
  });
}

export function rssSourceDelete(sourceUrl: string): Promise<void> {
  return invoke<void>(COMMANDS.rssSourceDelete, { sourceUrl });
}

export function rssSourceImport(json: string): Promise<number> {
  return invoke<number>(COMMANDS.rssSourceImport, { json });
}

export function rssSourceExport(): Promise<string> {
  return invoke<string>(COMMANDS.rssSourceExport);
}

export function rssSortUrls(sourceUrl: string): Promise<RssSort[]> {
  return invoke<RssSort[]>(COMMANDS.rssSortUrls, { sourceUrl });
}

export function rssArticles(
  sourceUrl: string,
  sortName = "",
  sortUrl = "",
  page = 1,
): Promise<RssArticlePage> {
  return invoke<RssArticlePage>(COMMANDS.rssArticles, {
    sourceUrl,
    sortName,
    sortUrl,
    page,
  });
}

export function rssArticleList(
  sourceUrl: string,
  sortName = "",
): Promise<RssArticle[]> {
  return invoke<RssArticle[]>(COMMANDS.rssArticleList, {
    sourceUrl,
    sortName,
  });
}

export function rssArticleClear(
  sourceUrl?: string | null,
  sortName?: string | null,
): Promise<void> {
  return invoke<void>(COMMANDS.rssArticleClear, {
    sourceUrl: sourceUrl ?? null,
    sortName: sortName ?? null,
  });
}

export function rssStarList(group?: string | null): Promise<RssStar[]> {
  return invoke<RssStar[]>(COMMANDS.rssStarList, { group: group ?? null });
}

export function rssStarGroups(): Promise<string[]> {
  return invoke<string[]>(COMMANDS.rssStarGroups);
}

export function rssStarGet(
  origin: string,
  link: string,
): Promise<RssStar | null> {
  return invoke<RssStar | null>(COMMANDS.rssStarGet, { origin, link });
}

export function rssStarSave(
  article: RssArticle,
  group?: string | null,
): Promise<void> {
  return invoke<void>(COMMANDS.rssStarSave, {
    article,
    group: group ?? null,
  });
}

export function rssStarUpdate(
  origin: string,
  link: string,
  title?: string | null,
  group?: string | null,
): Promise<void> {
  return invoke<void>(COMMANDS.rssStarUpdate, {
    origin,
    link,
    title: title ?? null,
    group: group ?? null,
  });
}

export function rssStarDelete(origin: string, link: string): Promise<void> {
  return invoke<void>(COMMANDS.rssStarDelete, { origin, link });
}

export function rssStarDeleteGroup(group: string): Promise<void> {
  return invoke<void>(COMMANDS.rssStarDeleteGroup, { group });
}

export function rssStarClear(): Promise<void> {
  return invoke<void>(COMMANDS.rssStarClear);
}

export function rssReadRecordAdd(
  record: string,
  title?: string | null,
): Promise<void> {
  return invoke<void>(COMMANDS.rssReadRecordAdd, {
    record,
    title: title ?? null,
  });
}

export function rssReadRecordList(): Promise<RssReadRecord[]> {
  return invoke<RssReadRecord[]>(COMMANDS.rssReadRecordList);
}

export function rssReadRecordClear(): Promise<void> {
  return invoke<void>(COMMANDS.rssReadRecordClear);
}

export function rssContent(
  sourceUrl: string,
  article: RssArticle,
  ruleContent?: string | null,
): Promise<string> {
  return invoke<string>(COMMANDS.rssContent, {
    sourceUrl,
    article,
    ruleContent: ruleContent ?? null,
  });
}
