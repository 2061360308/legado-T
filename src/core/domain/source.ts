export interface BookSourceRule {
  [key: string]: unknown;
}

export interface BookSource {
  bookSourceUrl: string;
  bookSourceName: string;
  bookSourceGroup?: string | null;
  bookSourceType: number;
  enabled: boolean;
  enabledExplore: boolean;
  enabledCookieJar?: boolean | null;
  customOrder: number;
  weight: number;
  lastUpdateTime: number;
  respondTime: number;
  bookUrlPattern?: string | null;
  bookSourceComment?: string | null;
  header?: string | null;
  jsLib?: string | null;
  loginUrl?: string | null;
  loginCheckJs?: string | null;
  loginUi?: string | null;
  coverDecodeJs?: string | null;
  concurrentRate?: string | null;
  variableComment?: string | null;
  searchUrl?: string | null;
  exploreUrl?: string | null;
  exploreScreen?: string | null;
  ruleSearch?: BookSourceRule | null;
  ruleExplore?: BookSourceRule | null;
  ruleBookInfo?: BookSourceRule | null;
  ruleToc?: BookSourceRule | null;
  ruleContent?: BookSourceRule | null;
  ruleReview?: BookSourceRule | null;
  [key: string]: unknown;
}

export interface OnlineSearchResult {
  bookUrl?: string;
  origin?: string;
  originName?: string;
  name?: string;
  author?: string;
  kind?: string | null;
  coverUrl?: string | null;
  intro?: string | null;
  latestChapterTitle?: string | null;
  tocUrl?: string;
  wordCount?: string | null;
  bookSourceUrl?: string;
  bookSourceName?: string;
  error?: string;
  [key: string]: unknown;
}

export interface OnlineBook {
  bookUrl: string;
  tocUrl?: string;
  origin?: string;
  originName?: string;
  name?: string;
  author?: string;
  kind?: string | null;
  coverUrl?: string | null;
  intro?: string | null;
  latestChapterTitle?: string | null;
  totalChapterNum?: number;
  wordCount?: string | null;
  imageStyle?: string | null;
  variable?: Record<string, string>;
  [key: string]: unknown;
}

export interface OnlineChapter {
  url: string;
  title: string;
  index: number;
  isVolume?: boolean;
  isVip?: boolean;
  isPay?: boolean;
  baseUrl?: string;
  bookUrl?: string;
  tag?: string | null;
  resourceUrl?: string | null;
  variable?: Record<string, string>;
  [key: string]: unknown;
}

export interface RssSource {
  sourceUrl: string;
  sourceName: string;
  sourceGroup?: string | null;
  sourceIcon?: string;
  enabled: boolean;
  customOrder: number;
  lastUpdateTime: number;
  sortUrl?: string | null;
  ruleArticles?: string | null;
  ruleNextPage?: string | null;
  ruleContent?: string | null;
  header?: string | null;
  loginUrl?: string | null;
  [key: string]: unknown;
}

export interface BookReview {
  avatar?: string | null;
  content: string;
  postTime?: string | null;
  reviewUrl?: string | null;
  voteUpUrl?: string | null;
  voteDownUrl?: string | null;
  postReviewUrl?: string | null;
  postQuoteUrl?: string | null;
  deleteUrl?: string | null;
  [key: string]: unknown;
}

export interface PayActionResult {
  result: "url" | "success" | "noop" | string;
  value?: string | null;
}

export interface WebViewLoginResult {
  url: string;
  text: string;
  cookies: string[];
}

export interface RssArticle {
  origin: string;
  sort: string;
  title: string;
  link: string;
  pubDate?: string | null;
  description?: string | null;
  content?: string | null;
  image?: string | null;
  variable?: Record<string, string>;
  [key: string]: unknown;
}
