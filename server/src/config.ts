export interface Env {
  MAX_UPLOAD_BYTES?: string;
  MAX_DOWNLOAD_BYTES?: string;
  TIMEOUT_MS?: string;
  MAX_REDIRECTS?: string;
  ALLOWED_HOSTS?: string;
  ALLOW_PRIVATE_HOSTS?: string;
  CORS_ORIGIN?: string;
  PROXY_TOKEN?: string;
}

export const DEFAULT_LIMITS = {
  maxUploadBytes: 100 * 1024 * 1024,
  maxDownloadBytes: 0,
  timeoutMs: 120_000,
  maxRedirects: 5,
};

function numberValue(value: string | undefined, fallback: number): number {
  if (value === undefined || value.trim() === "") return fallback;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? Math.floor(parsed) : fallback;
}

export function resolveLimits(env: Env) {
  return {
    maxUploadBytes: numberValue(env.MAX_UPLOAD_BYTES, DEFAULT_LIMITS.maxUploadBytes),
    maxDownloadBytes: numberValue(env.MAX_DOWNLOAD_BYTES, DEFAULT_LIMITS.maxDownloadBytes),
    timeoutMs: numberValue(env.TIMEOUT_MS, DEFAULT_LIMITS.timeoutMs),
    maxRedirects: numberValue(env.MAX_REDIRECTS, DEFAULT_LIMITS.maxRedirects),
  };
}

export function resolveAllowedHosts(env: Env): string[] {
  return (env.ALLOWED_HOSTS ?? "")
    .split(",")
    .map((item) => item.trim().toLowerCase())
    .filter(Boolean);
}

export function resolveCorsOrigin(env: Env): string {
  return (env.CORS_ORIGIN ?? "*").trim() || "*";
}
