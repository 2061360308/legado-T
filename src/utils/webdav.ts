import { platform } from "@/runtime";
import { kvGet } from "@/core/application/config";

function encodeBasicAuth(account: string, password: string): string {
  return btoa(unescape(encodeURIComponent(`${account}:${password}`)));
}

export async function uploadWebdav(
  fileName: string,
  data: Uint8Array,
  contentType = "application/octet-stream",
): Promise<string> {
  const base = (await kvGet("webdavUrl"))?.trim() || "https://dav.jianguoyun.com/dav/";
  const dir = ((await kvGet("webdavDir")) ?? "legado")
    .trim()
    .replace(/^\/+|\/+$/g, "");
  const account = (await kvGet("webdavAccount")) ?? (await kvGet("webdavUser")) ?? "";
  const password = (await kvGet("webdavPassword")) ?? "";
  const root = base.endsWith("/") ? base : `${base}/`;
  const folderUrl = `${root}${dir ? `${dir}/` : ""}`;
  const url = `${folderUrl}${fileName.replace(/^\/+/, "")}`;
  const headers: Record<string, string> = { "Content-Type": contentType };
  if (account || password) {
    headers.Authorization = `Basic ${encodeBasicAuth(account, password)}`;
  }
  if (dir) {
    await platform.http
      .fetch(folderUrl, { method: "MKCOL", headers })
      .catch(() => undefined);
  }
  const response = await platform.http.fetch(url, {
    method: "PUT",
    headers,
    body: data as unknown as BodyInit,
  });
  if (!response.ok) throw new Error(`WebDAV 上传失败 HTTP ${response.status}`);
  return url;
}
