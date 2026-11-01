const LOCAL_HOSTS = new Set(["localhost", "127.0.0.1", "::1"]);
const LOCAL_RESET_PARAM = "pwa-reset";

function hexToRgb(color: string): [number, number, number] | null {
  const value = color.trim().replace(/^#/, "");
  if (!/^[0-9a-f]{6}$/i.test(value)) return null;
  return [
    parseInt(value.slice(0, 2), 16),
    parseInt(value.slice(2, 4), 16),
    parseInt(value.slice(4, 6), 16),
  ];
}

function solidThemeColor(color: string, alpha: number, backdrop: string): string {
  const foreground = hexToRgb(color);
  if (!foreground || alpha >= 255) return color;
  const background = hexToRgb(backdrop) ?? [255, 255, 255];
  const ratio = Math.max(0, Math.min(255, alpha)) / 255;
  const channels = foreground.map((channel, index) =>
    Math.round(channel * ratio + background[index] * (1 - ratio)),
  );
  return `#${channels.map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
}

export function syncPwaThemeColor(
  color: string,
  alpha = 255,
  backdrop = "#ffffff",
): void {
  if (typeof document === "undefined") return;
  let meta = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]');
  if (!meta) {
    if (import.meta.env.MODE !== "web") return;
    meta = document.createElement("meta");
    meta.name = "theme-color";
    document.head.appendChild(meta);
  }
  const value = solidThemeColor(color, alpha, backdrop);
  if (meta.content !== value) meta.content = value;
}

async function clearLocalPwa(): Promise<boolean> {
  const controlled = Boolean(navigator.serviceWorker.controller);
  const registrations = await navigator.serviceWorker.getRegistrations();
  await Promise.all(registrations.map((registration) => registration.unregister()));
  if ("caches" in window) {
    const keys = await caches.keys();
    await Promise.all(
      keys
        .filter((key) => key.startsWith("local-reader-"))
        .map((key) => caches.delete(key)),
    );
  }
  return controlled;
}

function removeResetParam(): void {
  const url = new URL(window.location.href);
  if (!url.searchParams.has(LOCAL_RESET_PARAM)) return;
  url.searchParams.delete(LOCAL_RESET_PARAM);
  window.history.replaceState(window.history.state, "", url.toString());
}

export function registerWebPwa(): void {
  if (import.meta.env.MODE !== "web" || import.meta.env.DEV) return;
  if (!("serviceWorker" in navigator)) return;

  if (LOCAL_HOSTS.has(window.location.hostname)) {
    void clearLocalPwa()
      .then((controlled) => {
        if (!controlled) {
          removeResetParam();
          return;
        }
        const url = new URL(window.location.href);
        if (url.searchParams.has(LOCAL_RESET_PARAM)) return;
        url.searchParams.set(LOCAL_RESET_PARAM, "1");
        window.location.replace(url.toString());
      })
      .catch(() => undefined);
    return;
  }

  window.addEventListener("load", () => {
    void navigator.serviceWorker
      .register(`/sw.js?v=${encodeURIComponent(__BUILD_ID__)}`)
      .catch(() => undefined);
  });
}
