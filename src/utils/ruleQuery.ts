import { fetch } from "@tauri-apps/plugin-http";

export interface BuiltRequest {
  url: string;
  method: string;
  body?: string;
  headers: Record<string, string>;
  charset?: string;
}

function expand(text: string, vars: Record<string, string>, encode: boolean): string {
  return text.replace(/\{\{\s*([\w.]+)\s*\}\}/g, (_, key: string) => {
    const value = vars[key] ?? "";
    return encode ? encodeURIComponent(value) : value;
  });
}

export function buildRequest(rule: string, vars: Record<string, string>): BuiltRequest {
  let urlPart = rule.trim();
  let options: Record<string, unknown> = {};
  const idx = rule.lastIndexOf(",{");
  if (idx >= 0) {
    const maybeJson = rule.slice(idx + 1);
    try {
      options = JSON.parse(maybeJson) as Record<string, unknown>;
      urlPart = rule.slice(0, idx);
    } catch {
      options = {};
    }
  }

  const url = expand(urlPart, vars, true);
  const method = String(options.method ?? "GET").toUpperCase();
  const headers: Record<string, string> = {};
  const rawHeaders = options.headers;
  if (rawHeaders && typeof rawHeaders === "object") {
    for (const [key, value] of Object.entries(rawHeaders as Record<string, unknown>)) {
      headers[key] = String(value);
    }
  }
  const body = options.body !== undefined ? expand(String(options.body), vars, false) : undefined;
  const charset = options.charset !== undefined ? String(options.charset) : undefined;
  return { url, method, body, headers, charset };
}

function decode(buffer: ArrayBuffer, charset?: string): string {
  try {
    return new TextDecoder(charset || "utf-8").decode(buffer);
  } catch {
    return new TextDecoder("utf-8").decode(buffer);
  }
}

export async function requestText(req: BuiltRequest, signal?: AbortSignal): Promise<string> {
  const response = await fetch(req.url, {
    method: req.method,
    headers: req.headers,
    body: req.body,
    signal,
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return decode(await response.arrayBuffer(), req.charset);
}

function regexExtract(pattern: string, subject: string): string | null {
  try {
    const source = pattern.startsWith("/") && pattern.lastIndexOf("/") > 0
      ? pattern.slice(1, pattern.lastIndexOf("/"))
      : pattern;
    const regex = new RegExp(source, "s");
    const match = regex.exec(subject);
    if (!match) return null;
    return match[1] ?? match[0];
  } catch {
    return null;
  }
}

export function extractResult(showRule: string, body: string): string {
  const rule = showRule.trim();
  if (!rule) return body;

  if (rule.includes("##")) {
    const parts = rule.split("##");
    let subject = body;
    if (parts[0]) {
      subject = regexExtract(parts[0], body) ?? body;
    }
    for (let i = 1; i + 1 < parts.length; i += 2) {
      const pattern = parts[i];
      const replacement = parts[i + 1];
      try {
        const regex = new RegExp(pattern, "gs");
        subject = subject.replace(regex, replacement);
      } catch {
        // ignore invalid replacement
      }
    }
    return subject;
  }

  return regexExtract(rule, body) ?? body;
}

export function sanitizeHtml(input: string): string {
  let html = input;
  const looksLikeHtml = /<[a-z][\s\S]*>/i.test(html);
  if (!looksLikeHtml) {
    html = html
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/\n/g, "<br>");
  }
  return html
    .replace(/<\s*(script|style|iframe|object|embed)[\s\S]*?<\s*\/\s*\1\s*>/gi, "")
    .replace(/\son\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "")
    .replace(/javascript:/gi, "");
}
