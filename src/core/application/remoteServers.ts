import { kvGet, kvSet } from "./config";
import type {
  RemoteServer,
  RemoteServerConfig,
  RemoteServerType,
  S3ServerConfig,
  WebDavServerConfig,
} from "@/core/domain";
import {
  DEFAULT_REMOTE_SERVER_ID,
  isS3Config,
} from "@/core/domain";

const SERVERS_KEY = "remoteServers";
const SELECTED_KEY = "remoteServerId";
const PROXY_URL_KEY = "remoteProxyUrl";
const PASSWORD_PREFIX = "remoteServer:password:";
const SECRET_PREFIX = "remoteServer:secret:";

interface StoredServer {
  id: number;
  name: string;
  type: RemoteServerType;
  config: RemoteServerConfig;
  sortNumber: number;
}

function normalizeRoot(base: string, dir?: string | null): string {
  let root = base.trim().replace(/\/+$/, "");
  const sub = (dir ?? "").trim().replace(/^\/+|\/+$/g, "");
  if (sub) root = `${root}/${sub}`;
  return `${root}/books/`;
}

async function readStored(): Promise<StoredServer[]> {
  const raw = await kvGet(SERVERS_KEY);
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as StoredServer[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function hydrate(stored: StoredServer): Promise<RemoteServer> {
  const config = { ...stored.config } as RemoteServerConfig;
  if (isS3Config(config)) {
    const secret = await kvGet(`${SECRET_PREFIX}${stored.id}`);
    (config as S3ServerConfig).secretAccessKey = secret ?? "";
  } else {
    const password = await kvGet(`${PASSWORD_PREFIX}${stored.id}`);
    (config as WebDavServerConfig).password = password ?? "";
  }
  return { ...stored, config };
}

function stripSecrets(server: RemoteServer): StoredServer {
  const config = { ...server.config } as RemoteServerConfig;
  if (isS3Config(config)) {
    (config as S3ServerConfig).secretAccessKey = "";
  } else {
    (config as WebDavServerConfig).password = "";
  }
  return { ...server, config };
}

export async function loadDefaultRemoteServer(): Promise<RemoteServer | null> {
  const base = (await kvGet("webdavUrl"))?.trim();
  if (!base) return null;
  const root = normalizeRoot(base, await kvGet("webdavDir"));
  const username =
    (await kvGet("webdavAccount"))?.trim() ||
    (await kvGet("webdavUser"))?.trim() ||
    "";
  return {
    id: DEFAULT_REMOTE_SERVER_ID,
    name: "默认 WebDAV",
    type: "webdav",
    config: {
      url: root,
      username,
      password: (await kvGet("webdavPassword")) ?? "",
    },
    sortNumber: -1,
  };
}

export async function listRemoteServers(): Promise<RemoteServer[]> {
  const [stored, fallback] = await Promise.all([readStored(), loadDefaultRemoteServer()]);
  const servers = await Promise.all(stored.map(hydrate));
  if (fallback) servers.unshift(fallback);
  return servers.sort((left, right) => {
    if (left.id === DEFAULT_REMOTE_SERVER_ID) return -1;
    if (right.id === DEFAULT_REMOTE_SERVER_ID) return 1;
    return left.sortNumber - right.sortNumber || left.name.localeCompare(right.name, "zh-CN");
  });
}

export async function getRemoteServer(id: number): Promise<RemoteServer | null> {
  if (id === DEFAULT_REMOTE_SERVER_ID) return loadDefaultRemoteServer();
  const stored = (await readStored()).find((item) => item.id === id);
  return stored ? hydrate(stored) : null;
}

export async function saveRemoteServer(server: RemoteServer): Promise<RemoteServer> {
  const id = server.id === DEFAULT_REMOTE_SERVER_ID ? Date.now() : server.id;
  const saved: RemoteServer = { ...server, id };
  const stored = await readStored();
  const next = stored.filter((item) => item.id !== id);
  next.push(stripSecrets(saved));
  await kvSet(SERVERS_KEY, JSON.stringify(next));
  if (isS3Config(saved.config)) {
    await kvSet(`${SECRET_PREFIX}${id}`, saved.config.secretAccessKey);
    await kvSet(`${PASSWORD_PREFIX}${id}`, "");
  } else {
    await kvSet(`${PASSWORD_PREFIX}${id}`, saved.config.password);
    await kvSet(`${SECRET_PREFIX}${id}`, "");
  }
  return saved;
}

export async function deleteRemoteServer(id: number): Promise<void> {
  if (id === DEFAULT_REMOTE_SERVER_ID) return;
  const stored = await readStored();
  await kvSet(
    SERVERS_KEY,
    JSON.stringify(stored.filter((item) => item.id !== id)),
  );
  await kvSet(`${PASSWORD_PREFIX}${id}`, "");
  await kvSet(`${SECRET_PREFIX}${id}`, "");
  if ((await getSelectedRemoteServerId()) === id) {
    await selectRemoteServer(DEFAULT_REMOTE_SERVER_ID);
  }
}

export async function getSelectedRemoteServerId(): Promise<number> {
  const value = Number(await kvGet(SELECTED_KEY));
  return Number.isFinite(value) ? value : DEFAULT_REMOTE_SERVER_ID;
}

export async function selectRemoteServer(id: number): Promise<void> {
  await kvSet(SELECTED_KEY, String(id));
}

export async function getRemoteProxyUrl(): Promise<string> {
  return (await kvGet(PROXY_URL_KEY))?.trim() ?? "";
}

export async function setRemoteProxyUrl(value: string): Promise<void> {
  await kvSet(PROXY_URL_KEY, value.trim());
}

export async function getSelectedRemoteServer(): Promise<RemoteServer | null> {
  return getRemoteServer(await getSelectedRemoteServerId());
}
