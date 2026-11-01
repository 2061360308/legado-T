export function basicAuth(username: string, password: string): string {
  const bytes = new TextEncoder().encode(`${username}:${password}`);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return `Basic ${btoa(binary)}`;
}

export function withTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}

export function withoutTrailingSlash(value: string): string {
  return value.replace(/\/+$/, "");
}

export function textValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    return textValue(record["#text"] ?? record.text ?? "");
  }
  return "";
}

export function secondsFromDate(value: unknown): number {
  const text = textValue(value);
  const time = Date.parse(text);
  return Number.isFinite(time) ? Math.floor(time / 1000) : 0;
}
