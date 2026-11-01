import { XMLParser } from "fast-xml-parser";
import type { RemoteEntry, WebDavServerConfig } from "@/core/domain";
import { basicAuth, secondsFromDate, textValue, withTrailingSlash } from "./http";
import type { RemoteHttpTransport, RemoteStorage } from "./types";

const PROPFIND_BODY =
  '<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:displayname/><d:resourcetype/><d:getcontentlength/><d:getlastmodified/><d:getcontenttype/></d:prop></d:propfind>';

const parser = new XMLParser({
  ignoreAttributes: false,
  removeNSPrefix: true,
  trimValues: true,
});

function arrayValue(value: unknown): unknown[] {
  if (value === undefined || value === null || value === "") return [];
  return Array.isArray(value) ? value : [value];
}

function isCollection(value: unknown): boolean {
  if (value === undefined || value === null || value === "") return false;
  if (typeof value === "object") return true;
  return textValue(value).toLowerCase().includes("collection");
}

function propOf(response: Record<string, unknown>): Record<string, unknown> {
  const propstats = arrayValue(response.propstat) as Record<string, unknown>[];
  const usable =
    propstats.find((item) => {
      const status = textValue(item.status);
      return !status || /\s2\d\d\s/.test(status);
    }) ?? propstats[0];
  const prop = usable?.prop;
  return prop && typeof prop === "object" ? (prop as Record<string, unknown>) : {};
}

function resolvePath(href: string, base: string): URL {
  return new URL(href, withTrailingSlash(base));
}

export class WebDavStorage implements RemoteStorage {
  readonly type = "webdav";
  readonly rootPath: string;
  private readonly authHeader: string | null;

  constructor(
    private readonly config: WebDavServerConfig,
    private readonly transport: RemoteHttpTransport,
  ) {
    this.rootPath = withTrailingSlash(config.url.trim());
    this.authHeader =
      config.username || config.password
        ? basicAuth(config.username, config.password)
        : null;
  }

  private headers(extra: Record<string, string> = {}): Headers {
    const headers = new Headers(extra);
    if (this.authHeader) headers.set("Authorization", this.authHeader);
    return headers;
  }

  private request(path: string, init: RequestInit): Promise<Response> {
    return this.transport.request(path, init);
  }

  async test(): Promise<void> {
    const response = await this.request(this.rootPath, {
      method: "PROPFIND",
      headers: this.headers({ Depth: "0", "Content-Type": "application/xml" }),
      body: PROPFIND_BODY,
    });
    if (!response.ok) {
      throw new Error(`WebDAV ${response.status} ${response.statusText}`.trim());
    }
  }

  async list(path: string): Promise<RemoteEntry[]> {
    const base = withTrailingSlash(path || this.rootPath);
    const response = await this.request(base, {
      method: "PROPFIND",
      headers: this.headers({ Depth: "1", "Content-Type": "application/xml" }),
      body: PROPFIND_BODY,
    });
    if (!response.ok) {
      throw new Error(`WebDAV ${response.status} ${response.statusText}`.trim());
    }
    const xml = await response.text();
    const parsed = parser.parse(xml) as Record<string, unknown>;
    const multistatus = (parsed.multistatus ?? {}) as Record<string, unknown>;
    const responses = arrayValue(multistatus.response) as Record<string, unknown>[];
    const baseWithoutSlash = base.replace(/\/+$/, "");
    const entries: RemoteEntry[] = [];
    for (const responseEntry of responses) {
      const prop = propOf(responseEntry);
      const href = textValue(responseEntry.href);
      if (!href) continue;
      let url: URL;
      try {
        url = resolvePath(href, base);
      } catch {
        continue;
      }
      if (url.toString().replace(/\/+$/, "") === baseWithoutSlash) continue;
      const isDir = isCollection(prop.resourcetype) ||
        textValue(prop.getcontenttype).toLowerCase() === "httpd/unix-directory";
      if (isDir) url.pathname = withTrailingSlash(url.pathname);
      const decodedName = decodeURIComponent(
        url.pathname.split("/").filter(Boolean).pop() || "",
      );
      const displayName = textValue(prop.displayname);
      const name = displayName || decodedName;
      if (!name || name.startsWith(".")) continue;
      entries.push({
        path: url.toString(),
        name,
        isDir,
        size: Number(textValue(prop.getcontentlength)) || 0,
        modified: secondsFromDate(prop.getlastmodified),
        contentType: textValue(prop.getcontenttype),
        isOnShelf: false,
        bookUrl: null,
      });
    }
    return entries.sort((left, right) => {
      if (left.isDir !== right.isDir) return left.isDir ? -1 : 1;
      return left.name.localeCompare(right.name, "zh-CN", { numeric: true });
    });
  }

  async get(path: string): Promise<Response> {
    return this.request(path, {
      method: "GET",
      headers: this.headers(),
    });
  }

  childPath(path: string, name: string): string {
    return new URL(encodeURIComponent(name), withTrailingSlash(path)).toString();
  }

  async put(
    path: string,
    data: Blob | Uint8Array,
    contentType = "application/octet-stream",
  ): Promise<void> {
    const response = await this.request(path, {
      method: "PUT",
      headers: this.headers({ "Content-Type": contentType }),
      body: data as BodyInit,
    });
    if (!response.ok) {
      throw new Error(`WebDAV ${response.status} ${response.statusText}`.trim());
    }
  }

  async delete(path: string): Promise<void> {
    const response = await this.request(path, {
      method: "DELETE",
      headers: this.headers(),
    });
    if (!response.ok && response.status !== 404) {
      throw new Error(`WebDAV ${response.status} ${response.statusText}`.trim());
    }
  }
}
