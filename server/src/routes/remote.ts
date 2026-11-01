import { Hono, type Context } from "hono";
import {
  resolveAllowedHosts,
  resolveCorsOrigin,
  resolveLimits,
  type Env,
} from "../config";

const FORWARDED_HEADERS = [
  "authorization",
  "content-type",
  "depth",
  "range",
  "if-match",
  "if-none-match",
  "if-modified-since",
  "if-unmodified-since",
  "destination",
  "overwrite",
  "x-amz-date",
  "x-amz-content-sha256",
  "x-amz-security-token",
  "x-amz-copy-source",
  "x-amz-metadata-directive",
  "x-amz-user-agent",
];

const REDIRECT_STATUS = new Set([301, 302, 303, 307, 308]);
const PRIVATE_HOSTS = new Set(["localhost", "localhost.localdomain"]);

function corsHeaders(request: Request, env: Env): Headers {
  const headers = new Headers();
  const configured = resolveCorsOrigin(env);
  const origin = request.headers.get("origin");
  headers.set(
    "Access-Control-Allow-Origin",
    configured === "*" ? "*" : origin === configured ? origin : configured,
  );
  headers.set("Vary", "Origin");
  headers.set(
    "Access-Control-Allow-Methods",
    "GET, HEAD, OPTIONS, PROPFIND, PUT, DELETE, MKCOL, MOVE",
  );
  const requested = request.headers.get("access-control-request-headers");
  headers.set(
    "Access-Control-Allow-Headers",
    requested ||
      "Authorization, Content-Type, Depth, Range, X-Remote-Target, X-Proxy-Token, X-Amz-Date, X-Amz-Content-Sha256, X-Amz-Security-Token, X-Amz-Copy-Source, X-Amz-Metadata-Directive",
  );
  headers.set(
    "Access-Control-Expose-Headers",
    "Content-Length, Content-Type, ETag, Last-Modified, DAV, Accept-Ranges",
  );
  return headers;
}

function withCors(response: Response, request: Request, env: Env): Response {
  const headers = new Headers(response.headers);
  for (const [key, value] of corsHeaders(request, env)) {
    headers.set(key, value);
  }
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function errorResponse(
  c: Context<{ Bindings: Env }>,
  status: number,
  message: string,
): Response {
  return withCors(
    new Response(message, {
      status,
      headers: { "Content-Type": "text/plain; charset=utf-8" },
    }),
    c.req.raw,
    c.env,
  );
}

function isPrivateIpv4(host: string): boolean {
  const parts = host.split(".").map((part) => Number(part));
  if (parts.length !== 4 || parts.some((part) => !Number.isInteger(part))) {
    return false;
  }
  const [a, b] = parts;
  return (
    a === 0 ||
    a === 10 ||
    a === 127 ||
    (a === 169 && b === 254) ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168) ||
    (a === 100 && b >= 64 && b <= 127) ||
    a >= 224
  );
}

function isPrivateHost(hostname: string): boolean {
  const host = hostname.toLowerCase().replace(/^\[|\]$/g, "");
  if (PRIVATE_HOSTS.has(host) || host.endsWith(".localhost") || host.endsWith(".local")) {
    return true;
  }
  if (host === "::1" || host.startsWith("fc") || host.startsWith("fd") || host.startsWith("fe80")) {
    return true;
  }
  return isPrivateIpv4(host);
}

function validateTarget(
  value: string | null,
  env: Env,
): URL {
  if (!value) throw new Error("missing X-Remote-Target");
  let target: URL;
  try {
    target = new URL(value);
  } catch {
    throw new Error("invalid remote target");
  }
  if (target.protocol !== "http:" && target.protocol !== "https:") {
    throw new Error("only http/https targets are allowed");
  }
  if (target.username || target.password) {
    throw new Error("target URL must not include credentials");
  }
  if (!env.ALLOW_PRIVATE_HOSTS?.trim() && isPrivateHost(target.hostname)) {
    throw new Error("private targets are disabled");
  }
  const allowed = resolveAllowedHosts(env);
  if (allowed.length) {
    const host = target.hostname.toLowerCase();
    const permitted = allowed.some(
      (item) => host === item || host.endsWith(`.${item}`),
    );
    if (!permitted) throw new Error("target host is not allowed");
  }
  return target;
}

function applyUploadLimit(
  body: ReadableStream<Uint8Array> | null,
  maxBytes: number,
): ReadableStream<Uint8Array> | null {
  if (!body || maxBytes <= 0) return body;
  let total = 0;
  const reader = body.getReader();
  return new ReadableStream<Uint8Array>({
    async pull(controller) {
      const { done, value } = await reader.read();
      if (done) {
        controller.close();
        return;
      }
      total += value.byteLength;
      if (total > maxBytes) {
        await reader.cancel("upload limit exceeded");
        controller.error(new Error("upload limit exceeded"));
        return;
      }
      controller.enqueue(value);
    },
    async cancel(reason) {
      await reader.cancel(reason);
    },
  });
}

function applyDownloadLimit(
  body: ReadableStream<Uint8Array> | null,
  maxBytes: number,
): ReadableStream<Uint8Array> | null {
  if (!body || maxBytes <= 0) return body;
  let total = 0;
  const reader = body.getReader();
  return new ReadableStream<Uint8Array>({
    async pull(controller) {
      const { done, value } = await reader.read();
      if (done) {
        controller.close();
        return;
      }
      total += value.byteLength;
      if (total > maxBytes) {
        await reader.cancel("download limit exceeded");
        controller.error(new Error("download limit exceeded"));
        return;
      }
      controller.enqueue(value);
    },
    async cancel(reason) {
      await reader.cancel(reason);
    },
  });
}

function requestHeaders(request: Request): Headers {
  const headers = new Headers();
  for (const name of FORWARDED_HEADERS) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  return headers;
}

async function fetchWithRedirects(
  target: URL,
  init: RequestInit,
  env: Env,
): Promise<Response> {
  const limits = resolveLimits(env);
  const controller = new AbortController();
  const timer =
    limits.timeoutMs > 0
      ? setTimeout(() => controller.abort(), limits.timeoutMs)
      : undefined;
  let current = target;
  let currentInit = { ...init, signal: controller.signal, redirect: "manual" as RequestRedirect };

  try {
    for (let redirects = 0; ; redirects += 1) {
      const response = await fetch(current, currentInit);
      if (!REDIRECT_STATUS.has(response.status) || !response.headers.get("location")) {
        return response;
      }
      if (redirects >= limits.maxRedirects) {
        await response.body?.cancel();
        throw new Error("too many redirects");
      }
      if (currentInit.body) {
        await response.body?.cancel();
        throw new Error("redirect with request body is not supported");
      }
      const next = validateTarget(
        new URL(response.headers.get("location")!, current).toString(),
        env,
      );
      if (next.origin !== current.origin) {
        const headers = new Headers(currentInit.headers);
        headers.delete("authorization");
        headers.delete("cookie");
        currentInit = { ...currentInit, headers };
      }
      await response.body?.cancel();
      current = next;
    }
  } finally {
    if (timer) clearTimeout(timer);
  }
}

async function handleProxy(c: Context<{ Bindings: Env }>): Promise<Response> {
  const request = c.req.raw;
  const targetValue =
    request.headers.get("x-remote-target") || c.req.query("target") || null;

  if (
    request.method === "OPTIONS" &&
    (!targetValue || request.headers.has("access-control-request-method"))
  ) {
    return withCors(new Response(null, { status: 204 }), request, c.env);
  }

  const expectedToken = c.env.PROXY_TOKEN?.trim();
  if (expectedToken && request.headers.get("x-proxy-token") !== expectedToken) {
    return errorResponse(c, 401, "invalid proxy token");
  }

  let target: URL;
  try {
    target = validateTarget(targetValue, c.env);
  } catch (error) {
    return errorResponse(c, 400, (error as Error).message);
  }

  const limits = resolveLimits(c.env);
  const headers = requestHeaders(request);
  const uploadMethod =
    request.method === "PUT" ||
    request.method === "POST" ||
    request.method === "PATCH";
  const body = uploadMethod
    ? applyUploadLimit(request.body, limits.maxUploadBytes)
    : request.method === "GET" ||
        request.method === "HEAD" ||
        request.method === "OPTIONS"
      ? undefined
      : request.body;

  if (uploadMethod) {
    const declared = Number(request.headers.get("content-length"));
    if (limits.maxUploadBytes > 0 && declared > limits.maxUploadBytes) {
      return errorResponse(c, 413, "upload limit exceeded");
    }
  }

  let upstream: Response;
  try {
    upstream = await fetchWithRedirects(
      target,
      {
        method: request.method,
        headers,
        body,
        ...(body ? ({ duplex: "half" } as RequestInit) : {}),
      },
      c.env,
    );
  } catch (error) {
    return errorResponse(c, 502, (error as Error).message || "upstream request failed");
  }

  const declaredDownload = Number(upstream.headers.get("content-length"));
  if (
    limits.maxDownloadBytes > 0 &&
    Number.isFinite(declaredDownload) &&
    declaredDownload > limits.maxDownloadBytes
  ) {
    await upstream.body?.cancel();
    return errorResponse(c, 413, "download limit exceeded");
  }

  const responseHeaders = new Headers(upstream.headers);
  for (const name of [
    "connection",
    "content-encoding",
    "content-length",
    "set-cookie",
    "transfer-encoding",
  ]) {
    responseHeaders.delete(name);
  }
  for (const [key, value] of corsHeaders(request, c.env)) {
    responseHeaders.set(key, value);
  }

  return new Response(
    applyDownloadLimit(upstream.body, limits.maxDownloadBytes),
    {
      status: upstream.status,
      statusText: upstream.statusText,
      headers: responseHeaders,
    },
  );
}

export const remoteRoutes = new Hono<{ Bindings: Env }>();

remoteRoutes.all("/remote", handleProxy);
