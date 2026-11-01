import type { RemoteServer } from "@/core/domain";
import { isS3Config } from "@/core/domain";
import { S3Storage } from "./s3";
import type { RemoteHttpTransport, RemoteStorage } from "./types";
import { WebDavStorage } from "./webdav";

export function createRemoteStorage(
  server: RemoteServer,
  transport: RemoteHttpTransport,
): RemoteStorage {
  if (server.type === "s3") {
    if (!isS3Config(server.config)) {
      throw new Error("S3 配置无效");
    }
    return new S3Storage(server.config, transport);
  }
  if (!("url" in server.config)) {
    throw new Error("WebDAV 配置无效");
  }
  return new WebDavStorage(server.config, transport);
}
