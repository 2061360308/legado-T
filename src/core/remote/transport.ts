export type RemoteTransportMode = "unknown" | "direct" | "proxy";

function isAbortError(error: unknown): boolean {
  if (
    error instanceof DOMException &&
    (error.name === "AbortError" || error.name === "TimeoutError")
  ) {
    return true;
  }
  if (
    error instanceof Error &&
    (error.name === "AbortError" || error.name === "TimeoutError")
  ) {
    return true;
  }
  const text = String(error);
  return text.includes("AbortError") || text.includes("TimeoutError");
}

export interface RemoteHttpTransport {
  readonly mode: RemoteTransportMode;
  request(target: string, init?: RequestInit): Promise<Response>;
}

export interface RemoteTransportOptions {
  fetch: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;
  proxyUrl?: string | null;
  proxyToken?: string | null;
}

export class RemoteTransport implements RemoteHttpTransport {
  private currentMode: RemoteTransportMode = "unknown";

  constructor(private readonly options: RemoteTransportOptions) {}

  get mode(): RemoteTransportMode {
    return this.currentMode;
  }

  async request(target: string, init: RequestInit = {}): Promise<Response> {
    if (this.currentMode === "proxy") {
      return this.proxyRequest(target, init);
    }
    try {
      const response = await this.options.fetch(target, init);
      this.currentMode = "direct";
      return response;
    } catch (error) {
      if (isAbortError(error)) throw error;
      if (!this.options.proxyUrl) throw error;
      if (this.currentMode === "direct") throw error;
      const response = await this.proxyRequest(target, init);
      this.currentMode = "proxy";
      return response;
    }
  }

  private proxyRequest(target: string, init: RequestInit): Promise<Response> {
    const proxyUrl = this.options.proxyUrl?.trim();
    if (!proxyUrl) {
      throw new Error("WebDAV 未配置代理，且浏览器直连被 CORS 阻止");
    }
    const headers = new Headers(init.headers);
    headers.set("X-Remote-Target", target);
    if (this.options.proxyToken) {
      headers.set("X-Proxy-Token", this.options.proxyToken);
    }
    return this.options.fetch(proxyUrl, {
      ...init,
      headers,
      redirect: "follow",
    });
  }
}
