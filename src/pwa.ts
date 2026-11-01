const LOCAL_HOSTS = new Set(["localhost", "127.0.0.1", "::1"]);
const LOCAL_RESET_PARAM = "pwa-reset";

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
