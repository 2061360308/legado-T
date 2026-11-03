import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { app } from "../app";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

function encodeEngineHeaders(headers: Record<string, string>): string {
  return encodeURIComponent(JSON.stringify(headers));
}

function captureUpstreamHeaders(
  responseHeaders?: HeadersInit,
): { headers: Headers | null } {
  const captured: { headers: Headers | null } = { headers: null };
  globalThis.fetch = (async (_input: RequestInfo | URL, init?: RequestInit) => {
    captured.headers = new Headers(init?.headers);
    return new Response("ok", {
      status: 200,
      headers: responseHeaders ?? { "content-type": "text/plain" },
    });
  }) as typeof fetch;
  return captured;
}

async function request(
  headers: Record<string, string>,
  body = "body",
): Promise<Response> {
  return await app.request(
    "/api/engine/fetch",
    {
      method: "POST",
      headers,
      body,
    },
    {},
  );
}

test("forwards restricted headers from the engine envelope", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    "x-engine-headers": encodeEngineHeaders({
      cookie: "sid=1",
      "user-agent": "EngineUA",
      "sec-ch-ua-platform": '"Windows"',
      referer: "https://ref.example/",
    }),
    cookie: "app=secret",
    referer: "http://app.local/",
    "user-agent": "BrowserUA",
  });

  assert.equal(response.status, 200);
  assert.ok(captured.headers);
  assert.equal(captured.headers.get("cookie"), "sid=1");
  assert.equal(captured.headers.get("user-agent"), "EngineUA");
  assert.equal(captured.headers.get("sec-ch-ua-platform"), '"Windows"');
  assert.equal(captured.headers.get("referer"), "https://ref.example/");
  assert.equal(captured.headers.get("x-engine-headers"), null);
});

test("keeps browser fallback headers when the envelope is partial", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    "x-engine-headers": encodeEngineHeaders({ cookie: "sid=1" }),
    "user-agent": "BrowserUA",
    "content-type": "application/json",
    cookie: "app=secret",
  });

  assert.equal(response.status, 200);
  assert.ok(captured.headers);
  assert.equal(captured.headers.get("cookie"), "sid=1");
  assert.equal(captured.headers.get("user-agent"), "BrowserUA");
  assert.equal(captured.headers.get("content-type"), "application/json");
});

test("suppresses the fallback user agent when the engine disables it", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    "x-engine-headers": encodeEngineHeaders({ "user-agent": "null" }),
    "user-agent": "BrowserUA",
  });

  assert.equal(response.status, 200);
  assert.ok(captured.headers);
  assert.equal(captured.headers.get("user-agent"), null);
});

test("does not forward the web app's own cookie or referer", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    cookie: "app=secret",
    referer: "http://app.local/",
    "user-agent": "BrowserUA",
  });

  assert.equal(response.status, 200);
  assert.ok(captured.headers);
  assert.equal(captured.headers.get("cookie"), null);
  assert.equal(captured.headers.get("referer"), null);
  assert.equal(captured.headers.get("user-agent"), "BrowserUA");
});

test("strips hop-by-hop and internal proxy headers", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    "x-engine-headers": encodeEngineHeaders({
      connection: "keep-alive",
      "content-length": "999",
      host: "evil.example",
      "x-engine-headers": "nested",
      "x-proxy-token": "secret",
      "x-remote-target": "https://evil.example/",
      "x-test": "ok",
      "transfer-encoding": "chunked",
      upgrade: "h2c",
    }),
  });

  assert.equal(response.status, 200);
  assert.ok(captured.headers);
  assert.equal(captured.headers.get("x-test"), "ok");
  assert.equal(captured.headers.get("connection"), null);
  assert.equal(captured.headers.get("content-length"), null);
  assert.equal(captured.headers.get("host"), null);
  assert.equal(captured.headers.get("x-engine-headers"), null);
  assert.equal(captured.headers.get("x-proxy-token"), null);
  assert.equal(captured.headers.get("x-remote-target"), null);
  assert.equal(captured.headers.get("transfer-encoding"), null);
  assert.equal(captured.headers.get("upgrade"), null);
});

test("returns upstream cookies in a readable response header", async () => {
  const upstreamHeaders = new Headers();
  upstreamHeaders.append("set-cookie", "sid=1; Path=/; HttpOnly");
  upstreamHeaders.append("set-cookie", "theme=dark; Path=/");
  captureUpstreamHeaders(upstreamHeaders);
  const response = await request({
    "x-remote-target": "https://example.com/book",
  });

  assert.equal(response.status, 200);
  assert.equal(response.headers.get("set-cookie"), null);
  const raw = response.headers.get("x-engine-set-cookie");
  assert.ok(raw);
  assert.deepEqual(JSON.parse(decodeURIComponent(raw)), [
    { url: "https://example.com/book", cookie: "sid=1; Path=/; HttpOnly" },
    { url: "https://example.com/book", cookie: "theme=dark; Path=/" },
  ]);
});

test("collects cookies set during redirects", async () => {
  const redirectHeaders = new Headers();
  redirectHeaders.append("location", "https://example.com/final");
  redirectHeaders.append("set-cookie", "redirect=1; Path=/");
  const finalHeaders = new Headers();
  finalHeaders.append("set-cookie", "final=2; Path=/");
  const responses = [
    new Response(null, { status: 302, headers: redirectHeaders }),
    new Response("ok", { status: 200, headers: finalHeaders }),
  ];
  globalThis.fetch = (async () => responses.shift() ?? new Response("ok")) as typeof fetch;

  const response = await request({
    "x-remote-target": "https://example.com/start",
  });

  assert.equal(response.status, 200);
  const raw = response.headers.get("x-engine-set-cookie");
  assert.ok(raw);
  assert.deepEqual(JSON.parse(decodeURIComponent(raw)), [
    { url: "https://example.com/start", cookie: "redirect=1; Path=/" },
    { url: "https://example.com/final", cookie: "final=2; Path=/" },
  ]);
});

test("does not let the browser cache proxy responses across targets", async () => {
  const upstreamHeaders = new Headers({
    "cache-control": "public, max-age=3600",
    etag: '"upstream"',
    "last-modified": "Wed, 21 Oct 2015 07:28:00 GMT",
    expires: "Wed, 21 Oct 2015 07:28:00 GMT",
    age: "120",
    pragma: "cache",
    "content-type": "text/html",
  });
  captureUpstreamHeaders(upstreamHeaders);

  const response = await request({
    "x-remote-target": "https://example.com/book",
  });

  assert.equal(response.status, 200);
  assert.match(response.headers.get("cache-control") ?? "", /no-store/i);
  assert.equal(response.headers.get("etag"), null);
  assert.equal(response.headers.get("last-modified"), null);
  assert.equal(response.headers.get("expires"), "0");
  assert.equal(response.headers.get("age"), null);
  assert.match(response.headers.get("vary") ?? "", /x-remote-target/i);
});

test("marks proxy errors as non-cacheable", async () => {
  const response = await request({ "x-remote-target": "not-a-url" });

  assert.equal(response.status, 400);
  assert.match(response.headers.get("cache-control") ?? "", /no-store/i);
});

test("rejects an invalid engine header envelope", async () => {
  const captured = captureUpstreamHeaders();
  const response = await request({
    "x-remote-target": "https://example.com/book",
    "x-engine-headers": "%",
  });

  assert.equal(response.status, 400);
  assert.equal(captured.headers, null);
});

test("allows the engine header envelope in CORS preflight", async () => {
  const response = await app.request(
    "/api/engine/fetch",
    {
      method: "OPTIONS",
      headers: {
        origin: "https://app.example",
        "access-control-request-method": "POST",
        "access-control-request-headers": "X-Engine-Headers",
      },
    },
    { CORS_ORIGIN: "https://app.example" },
  );

  assert.equal(response.status, 204);
  assert.match(
    response.headers.get("access-control-allow-headers") ?? "",
    /x-engine-headers/i,
  );
});
