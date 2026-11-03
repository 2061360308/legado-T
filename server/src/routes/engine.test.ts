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

function captureUpstreamHeaders(): { headers: Headers | null } {
  const captured: { headers: Headers | null } = { headers: null };
  globalThis.fetch = (async (_input: RequestInfo | URL, init?: RequestInit) => {
    captured.headers = new Headers(init?.headers);
    return new Response("ok", {
      status: 200,
      headers: { "content-type": "text/plain" },
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
