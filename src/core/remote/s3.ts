import { AwsClient } from "aws4fetch";
import { XMLParser } from "fast-xml-parser";
import type { RemoteEntry, S3ServerConfig } from "@/core/domain";
import { textValue } from "./http";
import type { RemoteHttpTransport, RemoteStorage } from "./types";

const parser = new XMLParser({
  ignoreAttributes: false,
  removeNSPrefix: true,
  trimValues: true,
});

function arrayValue(value: unknown): unknown[] {
  if (value === undefined || value === null || value === "") return [];
  return Array.isArray(value) ? value : [value];
}

function encodeKey(key: string): string {
  return key
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
}

function decodeKey(path: string): { bucket: string; key: string } {
  const url = new URL(path);
  if (url.protocol !== "s3:") throw new Error("invalid S3 path");
  return {
    bucket: url.hostname,
    key: decodeURIComponent(url.pathname.replace(/^\/+/, "")),
  };
}

function extensionOf(name: string): string {
  const index = name.lastIndexOf(".");
  return index >= 0 ? name.slice(index + 1).toLowerCase() : "";
}

export class S3Storage implements RemoteStorage {
  readonly type = "s3";
  readonly rootPath: string;
  private readonly signer: AwsClient;
  private readonly endpoint: string;

  constructor(
    private readonly config: S3ServerConfig,
    private readonly transport: RemoteHttpTransport,
  ) {
    const endpoint = (config.endpoint.trim() ||
      `https://s3.${config.region.trim()}.amazonaws.com`).replace(/\/+$/, "");
    this.endpoint = endpoint;
    const prefix = (config.prefix ?? "").replace(/^\/+|\/+$/g, "");
    this.rootPath = prefix
      ? `s3://${config.bucket}/${prefix}/`
      : `s3://${config.bucket}/`;
    this.signer = new AwsClient({
      accessKeyId: config.accessKeyId,
      secretAccessKey: config.secretAccessKey,
      sessionToken: config.sessionToken,
      service: "s3",
      region: config.region,
    });
  }

  private objectUrl(key: string): string {
    const clean = key.replace(/^\/+/, "");
    if (this.config.pathStyle) {
      return `${this.endpoint}/${encodeURIComponent(this.config.bucket)}${
        clean ? `/${encodeKey(clean)}` : ""
      }`;
    }
    const endpoint = new URL(this.endpoint);
    endpoint.hostname = `${this.config.bucket}.${endpoint.hostname}`;
    endpoint.pathname = clean ? `/${encodeKey(clean)}` : "/";
    return endpoint.toString();
  }

  private async signedRequest(
    url: string,
    init: RequestInit,
  ): Promise<Response> {
    const request = new Request(url, init);
    const signed = await this.signer.sign(request);
    return this.transport.request(signed.url, {
      method: signed.method,
      headers: signed.headers,
      body: init.body,
    });
  }

  private async listPage(
    prefix: string,
    continuationToken?: string,
    maxKeys = 1000,
  ): Promise<Record<string, unknown>> {
    const url = new URL(this.objectUrl(""));
    url.searchParams.set("list-type", "2");
    url.searchParams.set("delimiter", "/");
    url.searchParams.set("prefix", prefix);
    url.searchParams.set("max-keys", String(maxKeys));
    if (continuationToken) {
      url.searchParams.set("continuation-token", continuationToken);
    }
    const response = await this.signedRequest(url.toString(), {
      method: "GET",
      headers: { Accept: "application/xml" },
    });
    if (!response.ok) {
      throw new Error(`S3 ${response.status} ${response.statusText}`.trim());
    }
    const xml = await response.text();
    return parser.parse(xml) as Record<string, unknown>;
  }

  async test(): Promise<void> {
    const parsed = await this.listPage(decodeKey(this.rootPath).key, undefined, 1);
    if (!parsed.ListBucketResult) {
      throw new Error("S3 ListObjectsV2 response is invalid");
    }
  }

  async list(path: string): Promise<RemoteEntry[]> {
    const { bucket, key } = decodeKey(path || this.rootPath);
    if (bucket !== this.config.bucket) {
      throw new Error("S3 bucket does not match server configuration");
    }
    const prefix = key && !key.endsWith("/") ? `${key}/` : key;
    const entries: RemoteEntry[] = [];
    let token: string | undefined;
    for (let page = 0; page < 20; page += 1) {
      const parsed = await this.listPage(prefix, token);
      const result = (parsed.ListBucketResult ?? {}) as Record<string, unknown>;
      for (const item of arrayValue(result.CommonPrefixes) as Record<string, unknown>[]) {
        const childPrefix = textValue(item.Prefix);
        const name = childPrefix.slice(prefix.length).replace(/\/+$/, "");
        if (!name || name.startsWith(".")) continue;
        entries.push({
          path: `s3://${bucket}/${childPrefix}`,
          name,
          isDir: true,
          size: 0,
          modified: 0,
          contentType: "folder",
          isOnShelf: false,
          bookUrl: null,
        });
      }
      for (const item of arrayValue(result.Contents) as Record<string, unknown>[]) {
        const objectKey = textValue(item.Key);
        if (!objectKey || objectKey === prefix) continue;
        const name = objectKey.slice(prefix.length);
        if (!name || name.includes("/") || name.startsWith(".")) continue;
        const modified = Date.parse(textValue(item.LastModified));
        entries.push({
          path: `s3://${bucket}/${objectKey}`,
          name,
          isDir: false,
          size: Number(textValue(item.Size)) || 0,
          modified: Number.isFinite(modified) ? Math.floor(modified / 1000) : 0,
          contentType: extensionOf(name),
          isOnShelf: false,
          bookUrl: null,
        });
      }
      const truncated = textValue(result.IsTruncated).toLowerCase() === "true";
      token = textValue(result.NextContinuationToken) || undefined;
      if (!truncated || !token) break;
    }
    return entries.sort((left, right) => {
      if (left.isDir !== right.isDir) return left.isDir ? -1 : 1;
      return left.name.localeCompare(right.name, "zh-CN", { numeric: true });
    });
  }

  async get(path: string): Promise<Response> {
    const { bucket, key } = decodeKey(path);
    if (bucket !== this.config.bucket) {
      throw new Error("S3 bucket does not match server configuration");
    }
    return this.signedRequest(this.objectUrl(key), {
      method: "GET",
    });
  }

  childPath(path: string, name: string): string {
    const { bucket, key } = decodeKey(path);
    const prefix = key && !key.endsWith("/") ? `${key}/` : key;
    return `s3://${bucket}/${prefix}${name}`;
  }

  async put(
    path: string,
    data: Blob | Uint8Array,
    contentType = "application/octet-stream",
  ): Promise<void> {
    const { bucket, key } = decodeKey(path);
    if (bucket !== this.config.bucket) {
      throw new Error("S3 bucket does not match server configuration");
    }
    const response = await this.signedRequest(this.objectUrl(key), {
      method: "PUT",
      headers: { "Content-Type": contentType },
      body: data as BodyInit,
    });
    if (!response.ok) {
      throw new Error(`S3 ${response.status} ${response.statusText}`.trim());
    }
  }

  async delete(path: string): Promise<void> {
    const { bucket, key } = decodeKey(path);
    if (bucket !== this.config.bucket) {
      throw new Error("S3 bucket does not match server configuration");
    }
    const response = await this.signedRequest(this.objectUrl(key), {
      method: "DELETE",
    });
    if (!response.ok && response.status !== 404) {
      throw new Error(`S3 ${response.status} ${response.statusText}`.trim());
    }
  }
}
