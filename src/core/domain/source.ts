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
  originCount?: number;
  origins?: string[];
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
  sourceComment?: string | null;
  enabled: boolean;
  customOrder: number;
  lastUpdateTime: number;
  sortUrl?: string | null;
  singleUrl?: boolean;
  articleStyle?: number;
  ruleArticles?: string | null;
  ruleNextPage?: string | null;
  ruleTitle?: string | null;
  rulePubDate?: string | null;
  ruleDescription?: string | null;
  ruleImage?: string | null;
  ruleLink?: string | null;
  ruleContent?: string | null;
  header?: string | null;
  loginUrl?: string | null;
  loginUi?: string | null;
  loginCheckJs?: string | null;
  coverDecodeJs?: string | null;
  variableComment?: string | null;
  enabledCookieJar?: boolean | null;
  concurrentRate?: string | null;
  jsLib?: string | null;
  enableJs?: boolean;
  loadWithBaseUrl?: boolean;
  style?: string | null;
  injectJs?: string | null;
  contentWhitelist?: string | null;
  contentBlacklist?: string | null;
  shouldOverrideUrlLoading?: string | null;
  [key: string]: unknown;
}

export interface ExploreCategory {
  title: string;
  url: string;
  style?: Record<string, unknown> | null;
  [key: string]: unknown;
}

export interface SourceCheckResult {
  bookSourceUrl: string;
  bookSourceName: string;
  ok: boolean;
  count: number;
  elapsedMs: number;
  error?: string;
}

export interface OnlineResource {
  url: string;
  title: string;
  index: number;
  isVip?: boolean;
  isPay?: boolean;
}

export interface OnlineImage extends OnlineResource {
  dataUrl: string;
}

export interface OnlineImagePage {
  title: string;
  index: number;
  images: { url: string; dataUrl?: string; error?: string }[];
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
  order?: number;
  group?: string;
  read?: boolean;
  pubDate?: string | null;
  description?: string | null;
  content?: string | null;
  image?: string | null;
  variable?: Record<string, string>;
  [key: string]: unknown;
}

export interface RssStar {
  origin: string;
  link: string;
  sort: string;
  title: string;
  starTime: number;
  pubDate?: string | null;
  description?: string | null;
  content?: string | null;
  image?: string | null;
  group: string;
  variable?: Record<string, string>;
  [key: string]: unknown;
}

export interface RssReadRecord {
  record: string;
  title?: string | null;
  readTime?: number | null;
  read: boolean;
  [key: string]: unknown;
}
