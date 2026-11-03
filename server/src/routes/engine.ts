import { Hono, type Context } from "hono";
import {
  resolveAllowedHosts,
  resolveCorsOrigin,
  resolveLimits,
  type Env,
} from "../config";

const ENGINE_HEADER_ENVELOPE = "x-engine-headers";
const DIRECT_FALLBACK_HEADERS = [
  "authorization",
  "content-type",
  "accept",
  "accept-language",
  "user-agent",
  "range",
  "if-none-match",
  "if-modified-since",
  "x-requested-with",
];
const SKIPPED_UPSTREAM_HEADERS = new Set([
  "connection",
  "content-length",
  "host",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
  "x-engine-headers",
  "x-proxy-token",
  "x-remote-target",
]);
const MAX_ENGINE_HEADER_BYTES = 64 * 1024;
const MAX_ENGINE_HEADER_COUNT = 100;

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
    "GET, HEAD, OPTIONS, POST, PUT, PATCH, DELETE",
  );
  headers.set(
    "Access-Control-Allow-Headers",
    request.headers.get("access-control-request-headers") ||
      "Authorization, Content-Type, Accept, X-Remote-Target, X-Proxy-Token, X-Engine-Headers, X-Final-Url",
  );
  headers.set(
    "Access-Control-Expose-Headers",
    "Content-Length, Content-Type, X-Final-Url, X-Upstream-Status",
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

function validateTarget(value: string | null, env: Env): URL {
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

function parseEngineHeaders(value: string | null): Headers {
  const headers = new Headers();
  if (!value) return headers;
  if (value.length > MAX_ENGINE_HEADER_BYTES) {
    throw new Error("engine header envelope is too large");
  }
  let text: string;
  try {
    text = decodeURIComponent(value);
  } catch {
    throw new Error("invalid engine header envelope");
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("invalid engine header envelope");
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("invalid engine header envelope");
  }
  const entries = Object.entries(parsed as Record<string, unknown>);
  if (entries.length > MAX_ENGINE_HEADER_COUNT) {
    throw new Error("engine header envelope has too many headers");
  }
  for (const [rawName, rawValue] of entries) {
    const name = rawName.trim().toLowerCase();
    if (!name || SKIPPED_UPSTREAM_HEADERS.has(name)) continue;
    if (typeof rawValue !== "string") continue;
    try {
      headers.set(name, rawValue);
    } catch {
      continue;
    }
  }
  return headers;
}

function requestHeaders(request: Request): Headers {
  const headers = new Headers();
  for (const name of DIRECT_FALLBACK_HEADERS) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  for (const [name, value] of parseEngineHeaders(
    request.headers.get(ENGINE_HEADER_ENVELOPE),
  )) {
    if (name === "user-agent" && value.trim().toLowerCase() === "null") {
      headers.delete("user-agent");
      continue;
    }
    headers.set(name, value);
  }
  return headers;
}

async function handleEngineFetch(c: Context<{ Bindings: Env }>): Promise<Response> {
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

  let upstreamHeaders: Headers;
  try {
    upstreamHeaders = requestHeaders(request);
  } catch (error) {
    return errorResponse(c, 400, (error as Error).message);
  }

  const limits = resolveLimits(c.env);
  const declaredUpload = Number(request.headers.get("content-length") ?? "0");
  if (
    limits.maxUploadBytes > 0 &&
    Number.isFinite(declaredUpload) &&
    declaredUpload > limits.maxUploadBytes
  ) {
    return errorResponse(c, 413, "request body is too large");
  }
  const controller = new AbortController();
  const timer =
    limits.timeoutMs > 0
      ? setTimeout(() => controller.abort(), limits.timeoutMs)
      : undefined;

  try {
    const init: RequestInit & { duplex?: "half" } = {
      method: request.method,
      headers: upstreamHeaders,
      body:
        request.method === "GET" ||
        request.method === "HEAD" ||
        request.method === "OPTIONS"
          ? undefined
          : request.body,
      redirect: "follow",
      signal: controller.signal,
      duplex: "half",
    };
    let current = target;
    let redirects = 0;
    let upstream: Response;
    for (;;) {
      upstream = await fetch(current, {
        ...init,
        redirect: "manual",
        method: redirects === 0 ? init.method : "GET",
        body: redirects === 0 ? init.body : undefined,
      });
      if (
        upstream.status >= 300 &&
        upstream.status < 400 &&
        upstream.headers.has("location")
      ) {
        if (redirects >= limits.maxRedirects) {
          return errorResponse(c, 502, "too many redirects");
        }
        const location = upstream.headers.get("location");
        if (!location) break;
        try {
          current = validateTarget(
            new URL(location, current).toString(),
            c.env,
          );
        } catch (error) {
          return errorResponse(c, 400, (error as Error).message);
        }
        redirects += 1;
        continue;
      }
      break;
    }

    if (limits.maxDownloadBytes > 0) {
      const declared = Number(upstream.headers.get("content-length") ?? "0");
      if (Number.isFinite(declared) && declared > limits.maxDownloadBytes) {
        return errorResponse(c, 413, "upstream response is too large");
      }
    }

    const headers = new Headers(upstream.headers);
    headers.delete("set-cookie");
    headers.delete("content-encoding");
    headers.delete("content-length");
    headers.delete("transfer-encoding");
    headers.set("X-Final-Url", upstream.url);
    headers.set("X-Upstream-Status", String(upstream.status));
    for (const [key, value] of corsHeaders(request, c.env)) {
      headers.set(key, value);
    }
    return new Response(upstream.body, {
      status: upstream.status,
      statusText: upstream.statusText,
      headers,
    });
  } catch (error) {
    return errorResponse(c, 502, (error as Error).message || "upstream request failed");
  } finally {
    if (timer) clearTimeout(timer);
  }
}

export const engineRoutes = new Hono<{ Bindings: Env }>();

engineRoutes.post("/engine/fetch", handleEngineFetch);
engineRoutes.get("/engine/fetch", handleEngineFetch);
engineRoutes.options("/engine/fetch", handleEngineFetch);
