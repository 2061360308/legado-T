export type RemoteServerType = "webdav" | "s3";

export interface WebDavServerConfig {
  url: string;
  username: string;
  password: string;
}

export interface S3ServerConfig {
  endpoint: string;
  region: string;
  bucket: string;
  accessKeyId: string;
  secretAccessKey: string;
  sessionToken?: string;
  prefix?: string;
  pathStyle?: boolean;
}

export type RemoteServerConfig = WebDavServerConfig | S3ServerConfig;

export interface RemoteServer {
  id: number;
  name: string;
  type: RemoteServerType;
  config: RemoteServerConfig;
  sortNumber: number;
}

export interface RemoteEntry {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  modified: number;
  contentType: string;
  isOnShelf: boolean;
  bookUrl?: string | null;
}

export const DEFAULT_REMOTE_SERVER_ID = -1;
export const WEBDAV_ORIGIN_PREFIX = "webDav::";
export const S3_ORIGIN_PREFIX = "s3::";

export interface RemoteOrigin {
  type: RemoteServerType;
  path: string;
  serverId: number;
}

export function isWebDavConfig(config: RemoteServerConfig): config is WebDavServerConfig {
  return "url" in config;
}

export function isS3Config(config: RemoteServerConfig): config is S3ServerConfig {
  return "bucket" in config;
}

export function buildRemoteOrigin(
  type: RemoteServerType,
  path: string,
  serverId: number,
): string {
  const prefix = type === "s3" ? S3_ORIGIN_PREFIX : WEBDAV_ORIGIN_PREFIX;
  const separator = path.includes("?") ? "&" : "?";
  return `${prefix}${path}${separator}serverID=${serverId}`;
}

export function parseRemoteOrigin(origin: string | null | undefined): RemoteOrigin | null {
  if (!origin) return null;
  const type = origin.startsWith(S3_ORIGIN_PREFIX)
    ? "s3"
    : origin.startsWith(WEBDAV_ORIGIN_PREFIX)
      ? "webdav"
      : null;
  if (!type) return null;
  const prefix = type === "s3" ? S3_ORIGIN_PREFIX : WEBDAV_ORIGIN_PREFIX;
  const raw = origin.slice(prefix.length);
  let path = raw;
  let serverId = DEFAULT_REMOTE_SERVER_ID;
  try {
    const target = new URL(raw);
    const id = Number(target.searchParams.get("serverID"));
    if (Number.isFinite(id)) serverId = id;
    target.searchParams.delete("serverID");
    path = target.toString();
  } catch {
    const marker = raw.lastIndexOf("?serverID=");
    if (marker >= 0) {
      const id = Number(raw.slice(marker + 10));
      if (Number.isFinite(id)) serverId = id;
      path = raw.slice(0, marker);
    }
  }
  return path ? { type, path, serverId } : null;
}
