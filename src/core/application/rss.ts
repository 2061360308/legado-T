import { invoke } from "./backend";
import { COMMANDS } from "./commands";
import type { RssArticle, RssSource } from "@/core/domain";

export interface RssArticlePage {
  articles: RssArticle[];
  next?: string | null;
}

export function rssSourceList(): Promise<RssSource[]> {
  return invoke<RssSource[]>(COMMANDS.rssSourceList);
}

export function rssSourceSave(source: RssSource): Promise<void> {
  return invoke<void>(COMMANDS.rssSourceSave, { source });
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
