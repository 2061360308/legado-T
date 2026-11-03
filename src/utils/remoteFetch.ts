import { platform } from "@/runtime";

export async function fetchRemoteText(url: string): Promise<string> {
  const response = platform.capabilities.webview
    ? await platform.http.fetch(url)
    : await fetch("/api/engine/fetch", {
        cache: "no-store",
        headers: { "X-Remote-Target": url },
      });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.text();
}
